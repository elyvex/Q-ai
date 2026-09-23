//! Traversal behavior (TASK-403 slice): cycles terminate, stars stay
//! bounded, hop counts are exact, K-path output is deterministic, and
//! cancellation yields explicit incomplete results.

use std::sync::atomic::{AtomicBool, Ordering};

use quran_graph::{
    AuthzScope, GraphEdge, GraphNode, GraphStore, MemGraphStore, NodeKind, QueryBudgets,
    bounded_subgraph, min_hops, reachable, up_to_k_paths,
};

fn cancel_open() -> AtomicBool {
    AtomicBool::new(false)
}

fn node(id: &str) -> GraphNode {
    GraphNode::new(id, NodeKind::Ayah, serde_json::Value::Null)
}

fn link(src: &str, dst: &str) -> GraphEdge {
    GraphEdge::structural(src, "NEXT", dst, serde_json::Value::Null)
}

fn cyclic() -> MemGraphStore {
    let mut s = MemGraphStore::new("cyclic");
    s.stage_nodes(vec![node("a"), node("b"), node("c")]).unwrap();
    s.stage_edges(vec![link("a", "b"), link("b", "c"), link("c", "a")]).unwrap();
    s
}

fn star(leaves: usize) -> MemGraphStore {
    let mut s = MemGraphStore::new("star");
    let mut nodes = vec![node("hub")];
    nodes.extend((0..leaves).map(|i| node(&format!("leaf-{i}"))));
    s.stage_nodes(nodes).unwrap();
    s.stage_edges((0..leaves).map(|i| link("hub", &format!("leaf-{i}"))).collect()).unwrap();
    s
}

fn diamond() -> MemGraphStore {
    let mut s = MemGraphStore::new("diamond");
    s.stage_nodes(vec![node("s"), node("x"), node("y"), node("t")]).unwrap();
    s.stage_edges(vec![
        link("s", "x"),
        link("s", "y"),
        link("x", "t"),
        link("y", "t"),
        link("s", "t"),
    ])
    .unwrap();
    s
}

#[test]
fn cyclic_fixture_terminates_within_limits() {
    let store = cyclic();
    let cancel = cancel_open();
    let authz = AuthzScope::all_visible();
    let budgets = QueryBudgets::default();
    let r = reachable(&store, &["a".to_string()], &budgets, &cancel, &authz).unwrap();
    assert!(!r.truncated);
    assert_eq!(r.nodes.len(), 3, "cycle visits each node exactly once");
    assert_eq!(r.edges.len(), 3);

    let sub = bounded_subgraph(&store, &["a".to_string()], &budgets, &cancel, &authz).unwrap();
    assert!(!sub.truncated);
    assert_eq!(sub.nodes.len(), 3);

    let hops = min_hops(&store, "a", "c", &budgets, &cancel, &authz).unwrap();
    // Reachability is direction-agnostic by design, so a reaches c in one
    // hop through the c -> a edge.
    assert_eq!(hops.hops, Some(1));
    assert!(!hops.truncated);
}

#[test]
fn high_degree_star_terminates_truncated() {
    let store = star(300);
    let cancel = cancel_open();
    let authz = AuthzScope::all_visible();
    let budgets = QueryBudgets { max_fanout: 10, ..QueryBudgets::default() };
    let r = reachable(&store, &["hub".to_string()], &budgets, &cancel, &authz).unwrap();
    assert!(r.truncated, "star fanout must hit the per-node cap");
    assert!(r.incomplete_reason.is_some());
    assert!(r.nodes.len() <= budgets.max_fanout + 1);
}

#[test]
fn min_hop_correctness_and_unreachable() {
    let store = diamond();
    let cancel = cancel_open();
    let authz = AuthzScope::all_visible();
    let budgets = QueryBudgets::default();
    let direct = min_hops(&store, "s", "t", &budgets, &cancel, &authz).unwrap();
    assert_eq!(direct.hops, Some(1));

    let mut chain = MemGraphStore::new("chain");
    chain.stage_nodes(vec![node("p"), node("q"), node("r"), node("far")]).unwrap();
    chain.stage_edges(vec![link("p", "q"), link("q", "r")]).unwrap();
    let two = min_hops(&chain, "p", "r", &budgets, &cancel, &authz).unwrap();
    assert_eq!(two.hops, Some(2));
    let missing = min_hops(&chain, "p", "far", &budgets, &cancel, &authz).unwrap();
    assert_eq!(missing.hops, None);
    assert!(!missing.truncated, "proven absence is complete, not truncated");
}

#[test]
fn k_paths_deterministic_across_runs() {
    let store = diamond();
    let cancel = cancel_open();
    let authz = AuthzScope::all_visible();
    let budgets = QueryBudgets::default();
    let first = up_to_k_paths(&store, "s", "t", 5, &budgets, &cancel, &authz).unwrap();
    let second = up_to_k_paths(&store, "s", "t", 5, &budgets, &cancel, &authz).unwrap();
    assert_eq!(first, second);
    assert!(!first.truncated);
    assert_eq!(first.paths.len(), 3);
    // Shortest first.
    assert_eq!(first.paths[0].node_ids.len(), 2);

    let one = up_to_k_paths(&store, "s", "t", 1, &budgets, &cancel, &authz).unwrap();
    assert_eq!(one.paths.len(), 1);
    assert_eq!(one.paths[0].node_ids, vec!["s".to_string(), "t".to_string()]);
}

#[test]
fn cancellation_is_explicitly_incomplete() {
    let store = star(50);
    let authz = AuthzScope::all_visible();
    let budgets = QueryBudgets::default();

    let cancelled = AtomicBool::new(true);
    let r = reachable(&store, &["hub".to_string()], &budgets, &cancelled, &authz).unwrap();
    assert!(r.truncated);
    let reason = r.incomplete_reason.unwrap_or_default().to_lowercase();
    assert!(reason.contains("cancel"), "reason was: {reason}");

    let mid = AtomicBool::new(false);
    // Flip the flag after the first batch by pre-arming through a wrapper is
    // overkill here: a pre-cancelled min-hop search must still be explicit.
    mid.store(true, Ordering::Relaxed);
    let o = min_hops(&store, "hub", "leaf-1", &budgets, &mid, &authz).unwrap();
    assert!(o.truncated);
    assert_eq!(o.hops, None);
    assert!(o.incomplete_reason.is_some());
}
