//! Backend-agnostic conformance suite (TASK-409 slice) against
//! [`MemGraphStore`]: neighbors, paths, subgraph, patterns, budget
//! exhaustion (truncated, never empty), and authorization-filtered
//! intermediates excluded from paths *and* counts.

use std::collections::HashSet;
use std::sync::atomic::AtomicBool;

use quran_graph::{
    Assertion, AssertionDecision, AssertionKind, AuthzScope, Direction, EdgeFilter, GraphEdge,
    GraphNode, GraphStore, MemGraphStore, NodeKind, Pattern, PatternStep, ProvenanceLayer,
    QueryBudgets,
};

fn open() -> (AtomicBool, AuthzScope, QueryBudgets) {
    (AtomicBool::new(false), AuthzScope::all_visible(), QueryBudgets::default())
}

fn assertion(id: &str, decision: AssertionDecision) -> Assertion {
    Assertion {
        id: id.to_string(),
        kind: AssertionKind::ConceptLink,
        claim: serde_json::json!({"concept": "mercy"}),
        evidence: serde_json::json!({"source": "annotator-a"}),
        source_location: "dataset-v1:row-7".to_string(),
        reviewer: Some("reviewer-1".to_string()),
        decision,
        decided_at: Some("2026-01-02T00:00:00Z".to_string()),
        supersedes_id: None,
        layer: ProvenanceLayer::B,
        algorithm: None,
        algorithm_version: None,
        confidence: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
    }
}

fn node(id: &str, kind: NodeKind) -> GraphNode {
    GraphNode::new(id, kind, serde_json::Value::Null)
}

/// Triangle: `s -> mid -> t` via asserted edges plus a direct structural
/// `s -> t` edge, so authz tests can prove the hidden route vanishes.
fn triangle() -> MemGraphStore {
    let mut s = MemGraphStore::new("conformance");
    s.stage_nodes(vec![
        node("s", NodeKind::Ayah),
        node("mid", NodeKind::Ayah),
        node("t", NodeKind::Ayah),
        node("c1", NodeKind::Concept),
    ])
    .unwrap();
    s.insert_assertion(assertion("a-link", AssertionDecision::Accepted));
    s.insert_assertion(assertion("a-hidden", AssertionDecision::Accepted));
    s.insert_assertion(assertion("a-tomb", AssertionDecision::Rejected));
    s.stage_edges(vec![
        GraphEdge::asserted("s", "NEXT", "mid", "a-link"),
        GraphEdge::asserted("mid", "NEXT", "t", "a-link"),
        GraphEdge::structural("s", "NEXT", "t", serde_json::Value::Null),
        GraphEdge::asserted("s", "MENTIONS_CONCEPT", "c1", "a-hidden"),
        GraphEdge::asserted("mid", "MENTIONS_CONCEPT", "c1", "a-tomb"),
    ])
    .unwrap();
    s
}

#[test]
fn resolve_and_neighbors_basics() {
    let store = triangle();
    let (cancel, authz, budgets) = open();
    let found = store.resolve_node("s", &budgets, &cancel, &authz).unwrap();
    assert_eq!(found.map(|n| n.kind), Some(NodeKind::Ayah));
    assert!(store.resolve_node("ghost", &budgets, &cancel, &authz).unwrap().is_none());

    let all = store.neighbors("s", &EdgeFilter::any(), &budgets, &cancel, &authz).unwrap();
    assert!(!all.truncated);
    // s -> mid, s -> t, s -> c1 = 3 edges; center + 3 counterparts = 4 nodes.
    assert_eq!(all.edges.len(), 3);
    assert_eq!(all.nodes.len(), 4);

    let out_next = store
        .neighbors("s", &EdgeFilter::one("NEXT", Direction::Outgoing), &budgets, &cancel, &authz)
        .unwrap();
    assert_eq!(out_next.edges.len(), 2);

    let incoming = store
        .neighbors("t", &EdgeFilter::one("NEXT", Direction::Incoming), &budgets, &cancel, &authz)
        .unwrap();
    assert_eq!(incoming.edges.len(), 2);
}

#[test]
fn bounded_paths_are_deterministic() {
    let store = triangle();
    let (cancel, authz, budgets) = open();
    let first = store.bounded_paths("s", "t", 4, &budgets, &cancel, &authz).unwrap();
    let second = store.bounded_paths("s", "t", 4, &budgets, &cancel, &authz).unwrap();
    assert!(!first.truncated);
    assert_eq!(first, second);
    // Direct edge plus the two-hop route via mid.
    assert_eq!(first.paths.len(), 2);
    assert_eq!(first.paths[0].node_ids, vec!["s".to_string(), "t".to_string()]);
    assert_eq!(first.paths[1].node_ids, vec!["s".to_string(), "mid".to_string(), "t".to_string()]);
}

#[test]
fn subgraph_and_pattern_cover_seeds() {
    let store = triangle();
    let (cancel, authz, budgets) = open();
    let sub = store.subgraph(&["s".to_string()], &budgets, &cancel, &authz).unwrap();
    assert!(!sub.truncated);
    let ids: HashSet<&str> = sub.nodes.iter().map(|n| n.stable_id.as_str()).collect();
    assert!(ids.contains("s") && ids.contains("mid") && ids.contains("t") && ids.contains("c1"));

    let pattern = Pattern::new(vec![PatternStep::edge_to("MENTIONS_CONCEPT", NodeKind::Concept)]);
    let matched =
        store.pattern_query(&pattern, &["s".to_string()], &budgets, &cancel, &authz).unwrap();
    assert!(!matched.truncated);
    assert!(matched.nodes.iter().any(|n| n.stable_id == "c1"));
}

#[test]
fn budget_exhaustion_is_truncated_not_empty() {
    let mut store = MemGraphStore::new("long-chain");
    let ids: Vec<String> = (0..50).map(|i| format!("n{i}")).collect();
    store.stage_nodes(ids.iter().map(|id| node(id, NodeKind::Ayah)).collect()).unwrap();
    store
        .stage_edges(
            ids.windows(2)
                .map(|w| GraphEdge::structural(&w[0], "NEXT", &w[1], serde_json::Value::Null))
                .collect(),
        )
        .unwrap();
    let (cancel, authz, _) = open();
    let tight = QueryBudgets { max_nodes: 5, ..QueryBudgets::default() };

    let sub = store.subgraph(&["n0".to_string()], &tight, &cancel, &authz).unwrap();
    assert!(sub.truncated, "exhausted budgets must truncate");
    assert!(sub.incomplete_reason.is_some());

    let paths = store.bounded_paths("n0", "n49", 60, &tight, &cancel, &authz).unwrap();
    assert!(paths.truncated, "a far target under tight budgets is incomplete, not absent");
    assert!(!paths.incomplete_reason.as_deref().unwrap_or_default().is_empty());
}

#[test]
fn authz_hides_intermediates_from_paths_and_counts() {
    let store = triangle();
    let (cancel, _, budgets) = open();
    // Scope sees the direct structural edge and the hidden concept link, but
    // not the a-link route through mid.
    let scope = AuthzScope::restricted(["a-hidden".to_string()]);

    let paths = store.bounded_paths("s", "t", 4, &budgets, &cancel, &scope).unwrap();
    assert!(!paths.truncated);
    assert_eq!(paths.paths.len(), 1, "only the structural route may show");
    assert_eq!(paths.paths[0].node_ids, vec!["s".to_string(), "t".to_string()]);

    let nbrs = store.neighbors("s", &EdgeFilter::any(), &budgets, &cancel, &scope).unwrap();
    assert!(!nbrs.truncated);
    // s->mid hidden; s->t structural + s->c1 visible = 2 edges.
    assert_eq!(nbrs.edges.len(), 2);
    assert!(nbrs.edges.iter().all(|e| e.dst != "mid"));

    let sub = store.subgraph(&["s".to_string()], &budgets, &cancel, &scope).unwrap();
    assert!(!sub.truncated);
    let ids: HashSet<&str> = sub.nodes.iter().map(|n| n.stable_id.as_str()).collect();
    assert!(!ids.contains("mid"), "nodes reachable only via hidden edges vanish");
    assert!(ids.contains("c1"));

    // Tombstoned assertions hide their edges even under full visibility.
    let (_, full, _) = open();
    let mid_nbrs = store.neighbors("mid", &EdgeFilter::any(), &budgets, &cancel, &full).unwrap();
    assert!(
        mid_nbrs.edges.iter().all(|e| e.assertion_id.as_deref() != Some("a-tomb")),
        "rejected assertions never authorize traversal"
    );
}

#[test]
fn out_of_range_budgets_are_rejected_before_execution() {
    // ADR-0217 rule 1: a budget outside its valid range is an error, never a
    // clamped-and-truncated result and never a silent empty answer. Every port
    // operation must reject it, including operations that would otherwise
    // return an empty-but-complete result.
    let store = triangle();
    let (cancel, authz, _) = open();
    // `max_hops = 0` and `max_nodes = 0` are both below the accepted ranges.
    let invalid = QueryBudgets { max_hops: 0, max_nodes: 0, ..QueryBudgets::default() };
    let pattern = Pattern::new(vec![PatternStep::edge_to("MENTIONS_CONCEPT", NodeKind::Concept)]);

    let err = store.resolve_node("s", &invalid, &cancel, &authz).unwrap_err();
    assert!(matches!(err, quran_graph::GraphError::BudgetExceeded { .. }), "resolve: {err}");

    let err = store.neighbors("s", &EdgeFilter::any(), &invalid, &cancel, &authz).unwrap_err();
    assert!(matches!(err, quran_graph::GraphError::BudgetExceeded { .. }), "neighbors: {err}");

    let err = store.bounded_paths("s", "t", 4, &invalid, &cancel, &authz).unwrap_err();
    assert!(matches!(err, quran_graph::GraphError::BudgetExceeded { .. }), "paths: {err}");

    let err = store.subgraph(&["s".to_string()], &invalid, &cancel, &authz).unwrap_err();
    assert!(matches!(err, quran_graph::GraphError::BudgetExceeded { .. }), "subgraph: {err}");

    let err =
        store.pattern_query(&pattern, &["s".to_string()], &invalid, &cancel, &authz).unwrap_err();
    assert!(matches!(err, quran_graph::GraphError::BudgetExceeded { .. }), "pattern: {err}");

    // A single out-of-range field is enough; the rest of the budget stays legal.
    for bad in [
        QueryBudgets { max_paths: 0, ..QueryBudgets::default() },
        QueryBudgets { max_edges: 0, ..QueryBudgets::default() },
        QueryBudgets { max_fanout: 0, ..QueryBudgets::default() },
        QueryBudgets { timeout_ms: 0, ..QueryBudgets::default() },
    ] {
        assert!(
            store.neighbors("s", &EdgeFilter::any(), &bad, &cancel, &authz).is_err(),
            "out-of-range budget must be rejected: {bad:?}"
        );
    }
}

#[test]
fn pattern_queries_apply_authz_during_expansion() {
    // The authz suite above covers neighbors/paths/subgraph; pattern queries
    // expand a typed predicate chain and must filter the same way, otherwise a
    // restricted scope could enumerate hidden concepts through a pattern.
    let store = triangle();
    let (cancel, _, budgets) = open();
    let pattern = Pattern::new(vec![PatternStep::edge_to("MENTIONS_CONCEPT", NodeKind::Concept)]);

    let full = store
        .pattern_query(&pattern, &["s".to_string()], &budgets, &cancel, &AuthzScope::all_visible())
        .unwrap();
    assert!(full.nodes.iter().any(|n| n.stable_id == "c1"));

    // `a-hidden` is the only accepted assertion behind `s -> c1`.
    let scope = AuthzScope::restricted(["a-link".to_string()]);
    let restricted =
        store.pattern_query(&pattern, &["s".to_string()], &budgets, &cancel, &scope).unwrap();
    assert!(!restricted.truncated, "a policy-filtered empty result is complete, not truncated");
    assert!(
        !restricted.nodes.iter().any(|n| n.stable_id == "c1"),
        "hidden concepts must not leak through a pattern query"
    );
}

#[test]
fn build_inspect_and_capabilities() {
    let mut store = MemGraphStore::new("proj-1");
    store.stage_nodes(vec![node("a", NodeKind::Ayah), node("b", NodeKind::Ayah)]).unwrap();
    store
        .stage_edges(vec![GraphEdge::structural("a", "NEXT", "b", serde_json::Value::Null)])
        .unwrap();
    let report = store.inspect();
    assert_eq!(report.projection_id, "proj-1");
    assert_eq!((report.node_count, report.edge_count), (2, 1));
    let caps = store.capabilities();
    for want in ["neighbors", "bounded_paths", "subgraph", "pattern_query", "authz_filter"] {
        assert!(caps.contains(&want.to_string()), "missing capability {want}");
    }
    store.clear_staged();
    let cleared = store.inspect();
    assert_eq!((cleared.node_count, cleared.edge_count), (0, 0));
    // Authority records survive staging clears.
    store.insert_assertion(assertion("a-keep", AssertionDecision::Accepted));
    store.clear_staged();
    assert!(store.get_assertion("a-keep").is_some());
}
