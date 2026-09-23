//! Batched frontier traversal over the [`GraphStore`] port (TASK-403).
//!
//! These helpers drive any backend through its typed `neighbors` operation —
//! never through a backend-specific implementation. Every function:
//!
//! - expands one frontier batch at a time with counters;
//! - sorts frontiers by stable ID for deterministic results;
//! - checks the cancellation flag per batch;
//! - stays cycle-safe via visited sets and high-degree-safe via the
//!   per-node fanout cap inside `budgets`;
//! - returns explicit incomplete results instead of "no path".

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};

use crate::error::GraphError;
use crate::model::{GraphNode, PathsResult, QueryBudgets, TraversalResult};
use crate::store::{AuthzScope, EdgeFilter, GraphStore, cancel_requested};

/// Minimum-hop outcome. `hops: None` with `truncated == false` means the
/// destination is unreachable; with `truncated == true` it means the budget
/// stopped the search before reachability was proven.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinHopsOutcome {
    /// Fewest hops from source to destination, if proven.
    #[serde(default)]
    pub hops: Option<usize>,
    /// Whether a budget or cancellation cut the search short.
    pub truncated: bool,
    /// Why the search stopped early. Always `Some` when `truncated`.
    #[serde(default)]
    pub incomplete_reason: Option<String>,
    /// Nodes expanded during the search.
    #[serde(default)]
    pub expanded_nodes: usize,
}

fn collect_sorted(
    node_records: BTreeMap<String, GraphNode>,
    mut edges: Vec<crate::model::GraphEdge>,
) -> (Vec<GraphNode>, Vec<crate::model::GraphEdge>) {
    let mut nodes: Vec<GraphNode> = node_records.into_values().collect();
    nodes.sort_by(|a, b| a.stable_id.cmp(&b.stable_id));
    edges.sort_by(|a, b| (&a.src, &a.edge, &a.dst).cmp(&(&b.src, &b.edge, &b.dst)));
    (nodes, edges)
}

/// All nodes reachable from `seeds` within `budgets.max_hops` hops.
pub fn reachable<S: GraphStore>(
    store: &S,
    seeds: &[String],
    budgets: &QueryBudgets,
    cancel: &AtomicBool,
    authz: &AuthzScope,
) -> Result<TraversalResult, GraphError> {
    budgets.check()?;
    if cancel_requested(cancel) {
        return Ok(TraversalResult::incomplete(
            Vec::new(),
            Vec::new(),
            "cancelled before reachability search",
            0,
            0,
        ));
    }
    let filter = EdgeFilter::any();
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut node_records: BTreeMap<String, GraphNode> = BTreeMap::new();
    let mut seen_edges: BTreeSet<(String, String, String, String)> = BTreeSet::new();
    let mut edges = Vec::new();
    let mut expanded_nodes: usize = 0;
    let mut expanded_edges: usize = 0;

    let mut frontier: Vec<String> = Vec::new();
    for seed in seeds {
        if visited.insert(seed.clone())
            && let Ok(Some(node)) = store.resolve_node(seed, budgets, cancel, authz)
        {
            node_records.insert(seed.clone(), node);
            frontier.push(seed.clone());
        }
    }
    if frontier.is_empty() {
        return Ok(TraversalResult::empty());
    }

    for _ in 0..budgets.max_hops {
        if cancel_requested(cancel) {
            let (nodes, edges) = collect_sorted(node_records, edges);
            return Ok(TraversalResult::incomplete(
                nodes,
                edges,
                "cancelled during reachability search",
                expanded_nodes,
                expanded_edges,
            ));
        }
        frontier.sort();
        let mut next: Vec<String> = Vec::new();
        for node_id in &frontier {
            let nbrs = store.neighbors(node_id, &filter, budgets, cancel, authz)?;
            expanded_nodes += 1;
            expanded_edges += nbrs.expanded_edges;
            if nbrs.truncated {
                let (nodes, edges) = collect_sorted(node_records, edges);
                return Ok(TraversalResult::incomplete(
                    nodes,
                    edges,
                    format!(
                        "neighbor budget exhausted at '{node_id}': {}",
                        nbrs.incomplete_reason.unwrap_or_default()
                    ),
                    expanded_nodes,
                    expanded_edges,
                ));
            }
            for node in nbrs.nodes {
                node_records.entry(node.stable_id.clone()).or_insert(node.clone());
                if visited.insert(node.stable_id.clone()) {
                    if visited.len() > budgets.max_nodes {
                        let (nodes, edges) = collect_sorted(node_records, edges);
                        return Ok(TraversalResult::incomplete(
                            nodes,
                            edges,
                            format!("node budget {} exhausted", budgets.max_nodes),
                            expanded_nodes,
                            expanded_edges,
                        ));
                    }
                    next.push(node.stable_id.clone());
                }
            }
            for edge in nbrs.edges {
                let key = (
                    edge.src.clone(),
                    edge.edge.clone(),
                    edge.dst.clone(),
                    edge.assertion_id.clone().unwrap_or_default(),
                );
                if seen_edges.insert(key) {
                    edges.push(edge);
                    if edges.len() > budgets.max_edges {
                        let (nodes, edges) = collect_sorted(node_records, edges);
                        return Ok(TraversalResult::incomplete(
                            nodes,
                            edges,
                            format!("edge budget {} exhausted", budgets.max_edges),
                            expanded_nodes,
                            expanded_edges,
                        ));
                    }
                }
            }
        }
        if next.is_empty() {
            break;
        }
        frontier = next;
    }
    let (nodes, edges) = collect_sorted(node_records, edges);
    Ok(TraversalResult {
        nodes,
        edges,
        truncated: false,
        incomplete_reason: None,
        expanded_nodes,
        expanded_edges,
    })
}

/// Bounded induced subgraph over `seeds`: reachability expansion plus every
/// edge whose endpoints are both visited.
pub fn bounded_subgraph<S: GraphStore>(
    store: &S,
    seeds: &[String],
    budgets: &QueryBudgets,
    cancel: &AtomicBool,
    authz: &AuthzScope,
) -> Result<TraversalResult, GraphError> {
    let base = reachable(store, seeds, budgets, cancel, authz)?;
    if base.truncated {
        return Ok(base);
    }
    if cancel_requested(cancel) {
        return Ok(TraversalResult::incomplete(
            base.nodes,
            base.edges,
            "cancelled before subgraph induction",
            base.expanded_nodes,
            base.expanded_edges,
        ));
    }
    let visited: BTreeSet<&str> = base.nodes.iter().map(|n| n.stable_id.as_str()).collect();
    let mut induced: BTreeMap<(String, String, String, String), crate::model::GraphEdge> =
        BTreeMap::new();
    for edge in base.edges {
        induced.insert(
            (
                edge.src.clone(),
                edge.edge.clone(),
                edge.dst.clone(),
                edge.assertion_id.clone().unwrap_or_default(),
            ),
            edge,
        );
    }
    let filter = EdgeFilter::any();
    let mut ordered: Vec<&str> = visited.iter().copied().collect();
    ordered.sort();
    let mut expanded_edges = base.expanded_edges;
    for node_id in ordered {
        if cancel_requested(cancel) {
            let nodes = base.nodes.clone();
            let edges: Vec<_> = induced.into_values().collect();
            return Ok(TraversalResult::incomplete(
                nodes,
                edges,
                "cancelled during subgraph induction",
                base.expanded_nodes,
                expanded_edges,
            ));
        }
        let nbrs = store.neighbors(node_id, &filter, budgets, cancel, authz)?;
        expanded_edges += nbrs.expanded_edges;
        if nbrs.truncated {
            let edges: Vec<_> = induced.into_values().collect();
            return Ok(TraversalResult::incomplete(
                base.nodes.clone(),
                edges,
                format!(
                    "neighbor budget exhausted at '{node_id}': {}",
                    nbrs.incomplete_reason.unwrap_or_default()
                ),
                base.expanded_nodes,
                expanded_edges,
            ));
        }
        for edge in nbrs.edges {
            if visited.contains(edge.src.as_str()) && visited.contains(edge.dst.as_str()) {
                induced.insert(
                    (
                        edge.src.clone(),
                        edge.edge.clone(),
                        edge.dst.clone(),
                        edge.assertion_id.clone().unwrap_or_default(),
                    ),
                    edge,
                );
            }
        }
        if induced.len() > budgets.max_edges {
            let edges: Vec<_> = induced.into_values().collect();
            return Ok(TraversalResult::incomplete(
                base.nodes.clone(),
                edges,
                format!("edge budget {} exhausted", budgets.max_edges),
                base.expanded_nodes,
                expanded_edges,
            ));
        }
    }
    Ok(TraversalResult {
        nodes: base.nodes,
        edges: induced.into_values().collect(),
        truncated: false,
        incomplete_reason: None,
        expanded_nodes: base.expanded_nodes,
        expanded_edges,
    })
}

/// Fewest hops from `src` to `dst` (breadth-first, deterministic).
pub fn min_hops<S: GraphStore>(
    store: &S,
    src: &str,
    dst: &str,
    budgets: &QueryBudgets,
    cancel: &AtomicBool,
    authz: &AuthzScope,
) -> Result<MinHopsOutcome, GraphError> {
    budgets.check()?;
    if cancel_requested(cancel) {
        return Ok(MinHopsOutcome {
            hops: None,
            truncated: true,
            incomplete_reason: Some("cancelled before minimum-hop search".to_string()),
            expanded_nodes: 0,
        });
    }
    if src == dst {
        return Ok(MinHopsOutcome {
            hops: Some(0),
            truncated: false,
            incomplete_reason: None,
            expanded_nodes: 0,
        });
    }
    let filter = EdgeFilter::any();
    let mut visited: BTreeSet<String> = BTreeSet::from([src.to_string()]);
    let mut frontier = vec![src.to_string()];
    let mut expanded_nodes: usize = 0;

    for depth in 1..=budgets.max_hops {
        if cancel_requested(cancel) {
            return Ok(MinHopsOutcome {
                hops: None,
                truncated: true,
                incomplete_reason: Some("cancelled during minimum-hop search".to_string()),
                expanded_nodes,
            });
        }
        frontier.sort();
        let mut next: Vec<String> = Vec::new();
        for node_id in &frontier {
            let nbrs = store.neighbors(node_id, &filter, budgets, cancel, authz)?;
            expanded_nodes += 1;
            if nbrs.truncated {
                return Ok(MinHopsOutcome {
                    hops: None,
                    truncated: true,
                    incomplete_reason: Some(format!(
                        "neighbor budget exhausted at '{node_id}': {}",
                        nbrs.incomplete_reason.unwrap_or_default()
                    )),
                    expanded_nodes,
                });
            }
            for node in nbrs.nodes {
                if node.stable_id == dst {
                    return Ok(MinHopsOutcome {
                        hops: Some(depth),
                        truncated: false,
                        incomplete_reason: None,
                        expanded_nodes,
                    });
                }
                if visited.insert(node.stable_id.clone()) {
                    if visited.len() > budgets.max_nodes {
                        return Ok(MinHopsOutcome {
                            hops: None,
                            truncated: true,
                            incomplete_reason: Some(format!(
                                "node budget {} exhausted",
                                budgets.max_nodes
                            )),
                            expanded_nodes,
                        });
                    }
                    next.push(node.stable_id.clone());
                }
            }
        }
        if next.is_empty() {
            break;
        }
        frontier = next;
    }
    Ok(MinHopsOutcome { hops: None, truncated: false, incomplete_reason: None, expanded_nodes })
}

/// Up to `k` bounded paths from `src` to `dst`, shortest first with
/// deterministic tie-breaking. The effective path cap is
/// `min(k, budgets.max_paths)`.
pub fn up_to_k_paths<S: GraphStore>(
    store: &S,
    src: &str,
    dst: &str,
    k: usize,
    budgets: &QueryBudgets,
    cancel: &AtomicBool,
    authz: &AuthzScope,
) -> Result<PathsResult, GraphError> {
    if k == 0 {
        return Err(GraphError::BudgetExceeded { detail: "k must be at least 1".to_string() });
    }
    budgets.check()?;
    let mut capped = budgets.clone();
    capped.max_paths = capped.max_paths.min(k);
    store.bounded_paths(src, dst, budgets.max_hops, &capped, cancel, authz)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::MemGraphStore;
    use crate::model::{GraphEdge, GraphNode, NodeKind};
    use crate::store::GraphStore;

    fn chain_store() -> MemGraphStore {
        let mut s = MemGraphStore::new("t");
        s.stage_nodes(vec![
            GraphNode::new("a", NodeKind::Ayah, serde_json::Value::Null),
            GraphNode::new("b", NodeKind::Ayah, serde_json::Value::Null),
            GraphNode::new("c", NodeKind::Ayah, serde_json::Value::Null),
        ])
        .unwrap();
        s.stage_edges(vec![
            GraphEdge::structural("a", "NEXT", "b", serde_json::Value::Null),
            GraphEdge::structural("b", "NEXT", "c", serde_json::Value::Null),
        ])
        .unwrap();
        s
    }

    #[test]
    fn reachable_collects_chain() {
        let s = chain_store();
        let cancel = AtomicBool::new(false);
        let authz = AuthzScope::all_visible();
        let r =
            reachable(&s, &["a".to_string()], &QueryBudgets::default(), &cancel, &authz).unwrap();
        assert!(!r.truncated);
        assert_eq!(r.nodes.len(), 3);
        assert_eq!(r.edges.len(), 2);
    }

    #[test]
    fn min_hops_counts_correctly() {
        let s = chain_store();
        let cancel = AtomicBool::new(false);
        let authz = AuthzScope::all_visible();
        let b = QueryBudgets::default();
        let o = min_hops(&s, "a", "c", &b, &cancel, &authz).unwrap();
        assert_eq!(o.hops, Some(2));
        assert!(!o.truncated);
        let self_o = min_hops(&s, "a", "a", &b, &cancel, &authz).unwrap();
        assert_eq!(self_o.hops, Some(0));
    }

    #[test]
    fn k_paths_rejects_zero() {
        let s = chain_store();
        let cancel = AtomicBool::new(false);
        let authz = AuthzScope::all_visible();
        let err =
            up_to_k_paths(&s, "a", "c", 0, &QueryBudgets::default(), &cancel, &authz).unwrap_err();
        assert!(matches!(err, GraphError::BudgetExceeded { .. }));
    }
}
