//! `MemGraphStore`: in-memory [`GraphStore`] reference backend.
//!
//! B-tree node and adjacency maps with the same semantics every adapter must
//! implement: budgets validated first, cancellation checked per batch,
//! authorization applied during expansion, deterministic ordering, and
//! explicit truncation. Conformance tests run against this backend; the
//! SQLite adjacency adapter must pass the same suite before activation.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::atomic::AtomicBool;

use crate::error::GraphError;
use crate::model::{
    Assertion, GraphEdge, GraphNode, PathsResult, QueryBudgets, TraversalResult, is_allowed_edge,
};
use crate::pattern::{Pattern, execute_pattern};
use crate::store::{
    AuthzScope, BuildInspection, Direction, EdgeFilter, GraphStore, cancel_requested,
};

/// In-memory reference [`GraphStore`] implementation.
///
/// Assertion authority records live beside adjacency here (in SQLite they are
/// separate tables per migration 0019); clearing staged adjacency never drops
/// them.
#[derive(Debug, Clone, Default)]
pub struct MemGraphStore {
    projection_id: String,
    nodes: BTreeMap<String, GraphNode>,
    out_edges: BTreeMap<String, Vec<GraphEdge>>,
    in_edges: BTreeMap<String, Vec<GraphEdge>>,
    assertions: BTreeMap<String, Assertion>,
}

impl MemGraphStore {
    /// Create an empty store for one projection family.
    pub fn new(projection_id: impl Into<String>) -> Self {
        Self {
            projection_id: projection_id.into(),
            nodes: BTreeMap::new(),
            out_edges: BTreeMap::new(),
            in_edges: BTreeMap::new(),
            assertions: BTreeMap::new(),
        }
    }

    /// Insert or replace an authority record.
    pub fn insert_assertion(&mut self, assertion: Assertion) {
        self.assertions.insert(assertion.id.clone(), assertion);
    }

    /// Look up an authority record by ID.
    pub fn get_assertion(&self, id: &str) -> Option<&Assertion> {
        self.assertions.get(id)
    }

    /// Whether an edge may be traversed or disclosed under `authz`.
    ///
    /// Tombstoned assertions (rejected/superseded) hide their edges even when
    /// listed in the scope. Edges pointing at unknown assertion IDs are
    /// fail-closed under a restricted scope and visible under an unrestricted
    /// one.
    pub fn is_edge_visible(&self, edge: &GraphEdge, authz: &AuthzScope) -> bool {
        let Some(aid) = edge.assertion_id.as_deref() else {
            return true;
        };
        match self.assertions.get(aid) {
            Some(record) => {
                if !record.is_effective() {
                    return false;
                }
                authz.edge_visible(Some(aid))
            }
            None => authz.visible_assertions.is_none(),
        }
    }

    /// Visible outgoing + incoming edges of one node, in deterministic
    /// `(edge, neighbor)` order.
    fn visible_adjacent(
        &self,
        stable_id: &str,
        filter: &EdgeFilter,
        authz: &AuthzScope,
    ) -> Vec<(GraphEdge, bool)> {
        let mut out = Vec::new();
        if matches!(filter.direction, Direction::Outgoing | Direction::Both)
            && let Some(edges) = self.out_edges.get(stable_id)
        {
            for e in edges {
                if filter.matches(&e.edge, true) && self.is_edge_visible(e, authz) {
                    out.push((e.clone(), true));
                }
            }
        }
        if matches!(filter.direction, Direction::Incoming | Direction::Both)
            && let Some(edges) = self.in_edges.get(stable_id)
        {
            for e in edges {
                if filter.matches(&e.edge, false) && self.is_edge_visible(e, authz) {
                    out.push((e.clone(), false));
                }
            }
        }
        out.sort_by(|a, b| {
            let ka = (&a.0.edge, neighbor_of(&a.0, a.1));
            let kb = (&b.0.edge, neighbor_of(&b.0, b.1));
            ka.cmp(&kb)
        });
        out
    }

    fn push_edge(&mut self, edge: GraphEdge) {
        self.out_edges.entry(edge.src.clone()).or_default().push(edge.clone());
        self.in_edges.entry(edge.dst.clone()).or_default().push(edge);
    }
}

fn neighbor_of(edge: &GraphEdge, outgoing: bool) -> &str {
    if outgoing { &edge.dst } else { &edge.src }
}

fn edge_key(edge: &GraphEdge) -> (String, String, String, String) {
    (
        edge.src.clone(),
        edge.edge.clone(),
        edge.dst.clone(),
        edge.assertion_id.clone().unwrap_or_default(),
    )
}

impl GraphStore for MemGraphStore {
    fn resolve_node(
        &self,
        stable_id: &str,
        budgets: &QueryBudgets,
        cancel: &AtomicBool,
        _authz: &AuthzScope,
    ) -> Result<Option<GraphNode>, GraphError> {
        budgets.check()?;
        if cancel_requested(cancel) {
            return Err(GraphError::BudgetExceeded {
                detail: "cancelled before node resolution".to_string(),
            });
        }
        Ok(self.nodes.get(stable_id).cloned())
    }

    fn neighbors(
        &self,
        stable_id: &str,
        filter: &EdgeFilter,
        budgets: &QueryBudgets,
        cancel: &AtomicBool,
        authz: &AuthzScope,
    ) -> Result<TraversalResult, GraphError> {
        budgets.check()?;
        let Some(center) = self.nodes.get(stable_id).cloned() else {
            return Err(GraphError::NodeNotFound { stable_id: stable_id.to_string() });
        };
        if cancel_requested(cancel) {
            return Ok(TraversalResult::incomplete(
                vec![center],
                Vec::new(),
                "cancelled before neighbor expansion",
                0,
                0,
            ));
        }
        let mut adjacent = self.visible_adjacent(stable_id, filter, authz);
        let mut truncated_reason: Option<String> = None;
        if adjacent.len() > budgets.max_fanout {
            truncated_reason = Some(format!(
                "fanout {} exceeds per-node cap {} at '{stable_id}'",
                adjacent.len(),
                budgets.max_fanout
            ));
            adjacent.truncate(budgets.max_fanout);
        }
        if adjacent.len() > budgets.max_edges {
            truncated_reason = Some(format!(
                "neighbor edges {} exceed edge budget {}",
                adjacent.len(),
                budgets.max_edges
            ));
            adjacent.truncate(budgets.max_edges);
        }

        let mut nodes = vec![center];
        let mut edges = Vec::with_capacity(adjacent.len());
        for (edge, outgoing) in &adjacent {
            edges.push(edge.clone());
            let other = neighbor_of(edge, *outgoing).to_string();
            if other != stable_id
                && let Some(node) = self.nodes.get(&other)
            {
                nodes.push(node.clone());
            }
        }
        nodes.sort_by(|a, b| a.stable_id.cmp(&b.stable_id));
        nodes.dedup_by(|a, b| a.stable_id == b.stable_id);
        if nodes.len() > budgets.max_nodes {
            nodes.truncate(budgets.max_nodes);
            truncated_reason =
                Some(format!("neighbor nodes exceed node budget {}", budgets.max_nodes));
        }
        edges.sort_by(|a, b| (&a.src, &a.edge, &a.dst).cmp(&(&b.src, &b.edge, &b.dst)));
        match truncated_reason {
            Some(reason) => {
                Ok(TraversalResult::incomplete(nodes, edges, reason, 1, adjacent.len()))
            }
            None => Ok(TraversalResult {
                nodes,
                edges,
                truncated: false,
                incomplete_reason: None,
                expanded_nodes: 1,
                expanded_edges: adjacent.len(),
            }),
        }
    }

    fn bounded_paths(
        &self,
        src: &str,
        dst: &str,
        max_hops: usize,
        budgets: &QueryBudgets,
        cancel: &AtomicBool,
        authz: &AuthzScope,
    ) -> Result<PathsResult, GraphError> {
        budgets.check()?;
        if !self.nodes.contains_key(src) {
            return Err(GraphError::NodeNotFound { stable_id: src.to_string() });
        }
        if !self.nodes.contains_key(dst) {
            return Err(GraphError::NodeNotFound { stable_id: dst.to_string() });
        }
        let effective_hops = max_hops.min(budgets.max_hops);
        if cancel_requested(cancel) {
            return Ok(PathsResult::incomplete(Vec::new(), "cancelled before path search", 0, 0));
        }
        if src == dst {
            return Ok(PathsResult::complete(
                vec![crate::model::GraphPath { node_ids: vec![src.to_string()], edges: vec![] }],
                0,
                0,
            ));
        }
        if effective_hops == 0 {
            return Ok(PathsResult::complete(Vec::new(), 0, 0));
        }

        let filter = EdgeFilter::any();
        let mut found: Vec<crate::model::GraphPath> = Vec::new();
        let mut queue: VecDeque<(Vec<String>, Vec<GraphEdge>)> =
            VecDeque::from([(vec![src.to_string()], Vec::new())]);
        let mut expanded_nodes: usize = 0;
        let mut expanded_edges: usize = 0;
        let mut enqueued: usize = 1;

        while let Some((node_ids, edges)) = queue.pop_front() {
            if cancel_requested(cancel) {
                return Ok(PathsResult::incomplete(
                    found,
                    "cancelled during path search",
                    expanded_nodes,
                    expanded_edges,
                ));
            }
            if found.len() >= budgets.max_paths {
                break;
            }
            let head = node_ids[node_ids.len() - 1].clone();
            if edges.len() >= effective_hops {
                continue;
            }
            expanded_nodes += 1;
            let mut adjacent = self.visible_adjacent(&head, &filter, authz);
            if adjacent.len() > budgets.max_fanout {
                adjacent.truncate(budgets.max_fanout);
                return Ok(PathsResult::incomplete(
                    found,
                    format!("fanout cap {} hit at '{head}'", budgets.max_fanout),
                    expanded_nodes,
                    expanded_edges,
                ));
            }
            for (edge, outgoing) in adjacent {
                expanded_edges += 1;
                if expanded_edges > budgets.max_edges {
                    return Ok(PathsResult::incomplete(
                        found,
                        format!("edge budget {} exhausted", budgets.max_edges),
                        expanded_nodes,
                        expanded_edges,
                    ));
                }
                let next = neighbor_of(&edge, outgoing).to_string();
                if node_ids.contains(&next) {
                    continue;
                }
                let mut next_ids = node_ids.clone();
                next_ids.push(next.clone());
                let mut next_edges = edges.clone();
                next_edges.push(edge);
                if next == dst {
                    found.push(crate::model::GraphPath { node_ids: next_ids, edges: next_edges });
                    if found.len() >= budgets.max_paths {
                        break;
                    }
                } else {
                    enqueued += 1;
                    if enqueued > budgets.max_nodes {
                        return Ok(PathsResult::incomplete(
                            found,
                            format!("node budget {} exhausted", budgets.max_nodes),
                            expanded_nodes,
                            expanded_edges,
                        ));
                    }
                    queue.push_back((next_ids, next_edges));
                }
            }
        }
        found.sort_by(|a, b| (a.edges.len(), &a.node_ids).cmp(&(b.edges.len(), &b.node_ids)));
        found.truncate(budgets.max_paths);
        Ok(PathsResult::complete(found, expanded_nodes, expanded_edges))
    }

    fn subgraph(
        &self,
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
                "cancelled before subgraph expansion",
                0,
                0,
            ));
        }
        let filter = EdgeFilter::any();
        let mut visited: BTreeSet<String> = BTreeSet::new();
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();
        for seed in seeds {
            if self.nodes.contains_key(seed) && visited.insert(seed.clone()) {
                queue.push_back((seed.clone(), 0));
            }
        }
        // Unknown seeds are skipped; an empty frontier is a complete empty result.
        let mut seen_edges: BTreeSet<(String, String, String, String)> = BTreeSet::new();
        let mut edges: Vec<GraphEdge> = Vec::new();
        let mut expanded_nodes: usize = 0;
        let mut expanded_edges: usize = 0;

        while let Some((node_id, depth)) = queue.pop_front() {
            if cancel_requested(cancel) {
                return Ok(finish_subgraph(
                    self,
                    visited,
                    edges,
                    "cancelled during subgraph expansion",
                    expanded_nodes,
                    expanded_edges,
                ));
            }
            if depth >= budgets.max_hops {
                continue;
            }
            expanded_nodes += 1;
            let mut adjacent = self.visible_adjacent(&node_id, &filter, authz);
            if adjacent.len() > budgets.max_fanout {
                adjacent.truncate(budgets.max_fanout);
                return Ok(finish_subgraph(
                    self,
                    visited,
                    edges,
                    format!("fanout cap {} hit at '{node_id}'", budgets.max_fanout),
                    expanded_nodes,
                    expanded_edges,
                ));
            }
            for (edge, outgoing) in adjacent {
                expanded_edges += 1;
                if expanded_edges > budgets.max_edges {
                    return Ok(finish_subgraph(
                        self,
                        visited,
                        edges,
                        format!("edge budget {} exhausted", budgets.max_edges),
                        expanded_nodes,
                        expanded_edges,
                    ));
                }
                if seen_edges.insert(edge_key(&edge)) {
                    edges.push(edge.clone());
                }
                let next = neighbor_of(&edge, outgoing).to_string();
                if !self.nodes.contains_key(&next) || !visited.insert(next.clone()) {
                    continue;
                }
                if visited.len() > budgets.max_nodes {
                    return Ok(finish_subgraph(
                        self,
                        visited,
                        edges,
                        format!("node budget {} exhausted", budgets.max_nodes),
                        expanded_nodes,
                        expanded_edges,
                    ));
                }
                queue.push_back((next, depth + 1));
            }
        }
        let mut nodes: Vec<GraphNode> =
            visited.iter().filter_map(|id| self.nodes.get(id).cloned()).collect();
        nodes.sort_by(|a, b| a.stable_id.cmp(&b.stable_id));
        edges.sort_by(|a, b| (&a.src, &a.edge, &a.dst).cmp(&(&b.src, &b.edge, &b.dst)));
        Ok(TraversalResult {
            nodes,
            edges,
            truncated: false,
            incomplete_reason: None,
            expanded_nodes,
            expanded_edges,
        })
    }

    fn pattern_query(
        &self,
        pattern: &Pattern,
        seeds: &[String],
        budgets: &QueryBudgets,
        cancel: &AtomicBool,
        authz: &AuthzScope,
    ) -> Result<TraversalResult, GraphError> {
        execute_pattern(self, pattern, seeds, budgets, cancel, authz)
    }

    fn stage_nodes(&mut self, nodes: Vec<GraphNode>) -> Result<(), GraphError> {
        for node in nodes {
            if node.stable_id.trim().is_empty() {
                return Err(GraphError::BuildFailed {
                    stage: "stage_nodes".to_string(),
                    detail: "refused node with empty stable ID".to_string(),
                });
            }
            self.nodes.insert(node.stable_id.clone(), node);
        }
        Ok(())
    }

    fn stage_edges(&mut self, edges: Vec<GraphEdge>) -> Result<(), GraphError> {
        for edge in &edges {
            if !is_allowed_edge(&edge.edge) {
                return Err(GraphError::PatternRejected {
                    detail: format!("cannot stage unknown edge '{}'", edge.edge),
                });
            }
            if !self.nodes.contains_key(&edge.src) {
                return Err(GraphError::BuildFailed {
                    stage: "stage_edges".to_string(),
                    detail: format!("dangling edge source '{}'", edge.src),
                });
            }
            if !self.nodes.contains_key(&edge.dst) {
                return Err(GraphError::BuildFailed {
                    stage: "stage_edges".to_string(),
                    detail: format!("dangling edge destination '{}'", edge.dst),
                });
            }
        }
        for edge in edges {
            self.push_edge(edge);
        }
        Ok(())
    }

    fn inspect(&self) -> BuildInspection {
        BuildInspection {
            projection_id: self.projection_id.clone(),
            node_count: self.nodes.len(),
            edge_count: self.out_edges.values().map(Vec::len).sum(),
        }
    }

    fn clear_staged(&mut self) {
        self.nodes.clear();
        self.out_edges.clear();
        self.in_edges.clear();
    }

    fn capabilities(&self) -> Vec<String> {
        [
            "resolve_node",
            "neighbors",
            "bounded_paths",
            "subgraph",
            "pattern_query",
            "build_inspect",
            "authz_filter",
            "budgets",
            "cancellation",
            "deterministic_order",
        ]
        .iter()
        .map(ToString::to_string)
        .collect()
    }
}

fn finish_subgraph(
    store: &MemGraphStore,
    visited: BTreeSet<String>,
    mut edges: Vec<GraphEdge>,
    reason: impl Into<String>,
    expanded_nodes: usize,
    expanded_edges: usize,
) -> TraversalResult {
    let mut nodes: Vec<GraphNode> =
        visited.iter().filter_map(|id| store.nodes.get(id).cloned()).collect();
    nodes.sort_by(|a, b| a.stable_id.cmp(&b.stable_id));
    edges.sort_by(|a, b| (&a.src, &a.edge, &a.dst).cmp(&(&b.src, &b.edge, &b.dst)));
    TraversalResult::incomplete(nodes, edges, reason, expanded_nodes, expanded_edges)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::NodeKind;

    fn open() -> (MemGraphStore, AtomicBool, AuthzScope) {
        (MemGraphStore::new("test"), AtomicBool::new(false), AuthzScope::all_visible())
    }

    #[test]
    fn stage_rejects_dangling_edges() {
        let (mut store, _, _) = open();
        let err = store
            .stage_edges(vec![GraphEdge::structural("a", "NEXT", "b", serde_json::Value::Null)])
            .unwrap_err();
        assert!(matches!(err, GraphError::BuildFailed { .. }));
    }

    #[test]
    fn stage_rejects_unknown_predicates() {
        let (mut store, _, _) = open();
        store
            .stage_nodes(vec![
                GraphNode::new("a", NodeKind::Ayah, serde_json::Value::Null),
                GraphNode::new("b", NodeKind::Ayah, serde_json::Value::Null),
            ])
            .unwrap();
        let err = store
            .stage_edges(vec![GraphEdge::structural("a", "PRECEDES", "b", serde_json::Value::Null)])
            .unwrap_err();
        assert!(matches!(err, GraphError::PatternRejected { .. }));
    }

    #[test]
    fn pattern_query_validates_first() {
        let (store, cancel, authz) = open();
        let bad = Pattern::new(vec![]);
        let err =
            store.pattern_query(&bad, &[], &QueryBudgets::default(), &cancel, &authz).unwrap_err();
        assert!(matches!(err, GraphError::PatternRejected { .. }));
    }
}
