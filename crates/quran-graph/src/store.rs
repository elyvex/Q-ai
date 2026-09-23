//! The `GraphStore` port (TASK-401): backend-neutral graph operations.
//!
//! Every query carries the same four guards:
//!
//! - `budgets: &QueryBudgets` — validated before execution, enforced while
//!   running;
//! - `cancel: &AtomicBool` — deadline/cancellation flag checked per batch;
//! - `authz: &AuthzScope` — assertion visibility applied during expansion,
//!   never as a post-filter, so hidden intermediates can neither appear in
//!   nor influence disclosed paths and counts;
//! - deterministic ordering — frontiers sort by stable ID so repeated runs
//!   agree.
//!
//! Incomplete-vs-empty: a complete search that finds nothing returns empty
//! collections with `truncated == false`. A search stopped by budgets or
//! cancellation returns `truncated == true` plus an `incomplete_reason`. It
//! never reports "no path" when it means "no path found within this budget".
//!
//! Only model types cross this trait. Backend handles and backend query
//! languages stay private to adapters.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

use crate::error::GraphError;
use crate::model::{GraphEdge, GraphNode, PathsResult, QueryBudgets, TraversalResult};
use crate::pattern::Pattern;

/// Traversal direction for neighbor expansion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Outgoing,
    Incoming,
    /// Both directions (direction-agnostic reachability).
    #[default]
    Both,
}

/// Neighbor filter: allowed edge predicates plus direction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeFilter {
    /// Allowed edge predicates. `None` admits every allowlisted predicate.
    /// Names outside the vocabulary match nothing.
    #[serde(default)]
    pub edge_types: Option<Vec<String>>,
    /// Expansion direction.
    #[serde(default)]
    pub direction: Direction,
}

impl EdgeFilter {
    /// Accept every edge predicate in both directions.
    pub fn any() -> Self {
        Self { edge_types: None, direction: Direction::Both }
    }

    /// Accept one predicate in the given direction.
    pub fn one(edge: impl Into<String>, direction: Direction) -> Self {
        Self { edge_types: Some(vec![edge.into()]), direction }
    }

    /// Whether an edge with this predicate traversed `outgoing` matches.
    pub fn matches(&self, edge: &str, outgoing: bool) -> bool {
        let direction_ok = match self.direction {
            Direction::Both => true,
            Direction::Outgoing => outgoing,
            Direction::Incoming => !outgoing,
        };
        if !direction_ok {
            return false;
        }
        match &self.edge_types {
            None => true,
            Some(names) => names.iter().any(|n| n == edge),
        }
    }
}

/// Authorization scope: which assertions the caller may see.
///
/// - `visible_assertions: None` — no restriction; every effective assertion
///   is visible (structural edges, which carry no assertion, are always
///   visible).
/// - `Some(set)` — only structural edges plus edges whose `assertion_id` is
///   in `set` are visible. Tombstoned assertions (rejected/superseded) stay
///   hidden even when listed.
///
/// Node records carry no assertions, so the scope never hides nodes
/// directly; it hides edges, which transitively removes anything reachable
/// only through hidden edges from paths and counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthzScope {
    /// Assertion IDs visible to the caller, or `None` for unrestricted.
    #[serde(default)]
    pub visible_assertions: Option<HashSet<String>>,
}

impl AuthzScope {
    /// Unrestricted visibility.
    pub fn all_visible() -> Self {
        Self { visible_assertions: None }
    }

    /// Restrict visibility to the given assertion IDs (plus structural edges).
    pub fn restricted(ids: impl IntoIterator<Item = String>) -> Self {
        Self { visible_assertions: Some(ids.into_iter().collect()) }
    }

    /// Whether an edge carrying this assertion pointer is visible.
    /// Structural edges (`None`) are always visible.
    pub fn edge_visible(&self, assertion_id: Option<&str>) -> bool {
        match (assertion_id, &self.visible_assertions) {
            (None, _) => true,
            (Some(_), None) => true,
            (Some(id), Some(set)) => set.contains(id),
        }
    }
}

/// Staged-build inspection summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildInspection {
    /// Projection family this staging area belongs to.
    pub projection_id: String,
    /// Staged node count.
    pub node_count: usize,
    /// Staged edge count.
    pub edge_count: usize,
}

/// Whether the caller requested cancellation.
pub fn cancel_requested(cancel: &AtomicBool) -> bool {
    cancel.load(Ordering::Relaxed)
}

/// Backend-neutral graph operations. Implemented by the reference in-memory
/// store ([`crate::mem::MemGraphStore`]); the SQLite adjacency adapter (a
/// separate application-layer crate, not implemented here) implements the
/// same port with identical truncation and authorization semantics.
pub trait GraphStore {
    /// Resolve one node by stable ID. `Ok(None)` means absent, not truncated.
    fn resolve_node(
        &self,
        stable_id: &str,
        budgets: &QueryBudgets,
        cancel: &AtomicBool,
        authz: &AuthzScope,
    ) -> Result<Option<GraphNode>, GraphError>;

    /// One-hop neighborhood honoring the edge filter, budgets, cancellation,
    /// and authorization scope.
    fn neighbors(
        &self,
        stable_id: &str,
        filter: &EdgeFilter,
        budgets: &QueryBudgets,
        cancel: &AtomicBool,
        authz: &AuthzScope,
    ) -> Result<TraversalResult, GraphError>;

    /// Up to `budgets.max_paths` paths from `src` to `dst` within
    /// `min(max_hops, budgets.max_hops)` hops. Deterministic order.
    fn bounded_paths(
        &self,
        src: &str,
        dst: &str,
        max_hops: usize,
        budgets: &QueryBudgets,
        cancel: &AtomicBool,
        authz: &AuthzScope,
    ) -> Result<PathsResult, GraphError>;

    /// Bounded multi-seed subgraph (BFS up to `budgets.max_hops` hops).
    fn subgraph(
        &self,
        seeds: &[String],
        budgets: &QueryBudgets,
        cancel: &AtomicBool,
        authz: &AuthzScope,
    ) -> Result<TraversalResult, GraphError>;

    /// Typed pattern match starting from `seeds`. The pattern is validated
    /// first; invalid patterns are errors, never silent empty results.
    fn pattern_query(
        &self,
        pattern: &Pattern,
        seeds: &[String],
        budgets: &QueryBudgets,
        cancel: &AtomicBool,
        authz: &AuthzScope,
    ) -> Result<TraversalResult, GraphError>;

    /// Stage nodes into the build area.
    fn stage_nodes(&mut self, nodes: Vec<GraphNode>) -> Result<(), GraphError>;

    /// Stage edges into the build area. Edge predicates must be allowlisted
    /// and both endpoints must already be staged: zero dangling edges.
    fn stage_edges(&mut self, edges: Vec<GraphEdge>) -> Result<(), GraphError>;

    /// Inspect the staged build.
    fn inspect(&self) -> BuildInspection;

    /// Drop staged adjacency. Assertion authority records are retained:
    /// clearing a projection never discards scholarly history.
    fn clear_staged(&mut self);

    /// Capability discovery: operation and guarantee names this backend
    /// implements (e.g. `neighbors`, `authz_filter`, `cancellation`).
    fn capabilities(&self) -> Vec<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_filter_matching() {
        let any = EdgeFilter::any();
        assert!(any.matches("NEXT", true));
        assert!(any.matches("NEXT", false));
        let out = EdgeFilter::one("NEXT", Direction::Outgoing);
        assert!(out.matches("NEXT", true));
        assert!(!out.matches("NEXT", false));
        assert!(!out.matches("CONTAINS", true));
    }

    #[test]
    fn authz_scope_visibility() {
        let all = AuthzScope::all_visible();
        assert!(all.edge_visible(None));
        assert!(all.edge_visible(Some("a1")));
        let scope = AuthzScope::restricted(["a1".to_string()]);
        assert!(scope.edge_visible(None));
        assert!(scope.edge_visible(Some("a1")));
        assert!(!scope.edge_visible(Some("a2")));
    }
}
