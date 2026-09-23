//! Typed graph model: nodes, edges, assertions, manifests, budgets, results.
//!
//! Structs mirror `migrations/sqlite/0019_quran_graph.up.sql` columns:
//!
//! - [`GraphNode`] mirrors `graph_nodes` (`stable_id`, `node_kind`,
//!   `attrs_json`).
//! - [`GraphEdge`] mirrors `graph_edges` (`src_stable_id`, `edge`,
//!   `dst_stable_id`, `assertion_id`). Structural edges carry `assertion_id`
//!   `None` with input-version provenance inside `attrs`, exactly as the
//!   migration documents.
//! - [`Assertion`] mirrors `graph_assertions` (`assertion_kind`,
//!   `claim_json`, `evidence_json`, `source_location`, `reviewer`,
//!   `decision`, `decided_at`, `supersedes_id`, `provenance_layer`,
//!   `algorithm`, `algorithm_version`, `confidence`, `created_at`).
//! - [`ProjectionManifest`] mirrors `graph_projections`
//!   (`projection_id`, `builder_version`, `edition_id`, `corpus_generation`,
//!   `dataset_versions_json`, `dependency_snapshot_json`, `status`,
//!   `manifest_json`, `created_at`).
//!
//! Budget exhaustion is always explicit: result types carry `truncated` plus
//! an `incomplete_reason`. An empty result with `truncated == false` means
//! "no path exists"; `truncated == true` means "the budget stopped the search
//! before the answer was proven".

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::GraphError;

/// Node kinds. Mirrors the `graph_nodes.node_kind` check list in migration
/// 0019 exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
    Edition,
    Surah,
    Ayah,
    Token,
    Division,
    Root,
    Lemma,
    Concept,
    Entity,
    Annotation,
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Edition => "edition",
            Self::Surah => "surah",
            Self::Ayah => "ayah",
            Self::Token => "token",
            Self::Division => "division",
            Self::Root => "root",
            Self::Lemma => "lemma",
            Self::Concept => "concept",
            Self::Entity => "entity",
            Self::Annotation => "annotation",
        };
        write!(f, "{s}")
    }
}

/// Allowlisted edge predicates (TASK-414 reconciliation).
///
/// `NEXT` is the single canonical sequencing predicate; predecessor/successor
/// aliases are not admitted as separate edge names. Membership here is the
/// only way an edge name reaches a query: [`crate::pattern::validate_pattern`]
/// and staging both reject anything outside this list.
pub const EDGE_VOCABULARY: &[&str] = &[
    "CONTAINS",
    "NEXT",
    "MENTIONS_CONCEPT",
    "REFERS_TO",
    "MENTIONS",
    "CITES",
    "CONTRASTS_WITH",
    "PARALLELS",
    "EXPLAINS",
    "HAS_LEMMA",
    "HAS_ROOT",
    "HAS_STEM",
    "DERIVED_FROM",
    "SAME_ROOT_AS",
    "SAME_LEMMA_AS",
];

/// Check whether an edge name is in [`EDGE_VOCABULARY`].
pub fn is_allowed_edge(edge: &str) -> bool {
    EDGE_VOCABULARY.contains(&edge)
}

/// A graph node keyed by stable domain ID (never a backend rowid).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNode {
    /// Stable ID, e.g. `ayah:1:1` or `edition:madani@v1`.
    pub stable_id: String,
    /// Node family.
    pub kind: NodeKind,
    /// Extension properties only. Canonical text lives in the canonical
    /// store, never here.
    #[serde(default)]
    pub attrs: serde_json::Value,
}

impl GraphNode {
    /// Build a node with the given stable ID, kind, and attributes.
    pub fn new(stable_id: impl Into<String>, kind: NodeKind, attrs: serde_json::Value) -> Self {
        Self { stable_id: stable_id.into(), kind, attrs }
    }
}

/// A traversable adjacency edge with an attribution pointer.
///
/// Presentation deduplication must preserve attribution: two scholars may
/// assert the same `(src, edge, dst)` triple with different evidence, so the
/// stable edge identity includes `assertion_id`, not just the triple.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source stable ID.
    pub src: String,
    /// Edge predicate; must be in [`EDGE_VOCABULARY`].
    pub edge: String,
    /// Destination stable ID.
    pub dst: String,
    /// Attribution pointer into the assertion store. `None` marks a
    /// deterministic structural edge whose provenance lives in `attrs`
    /// (`input_version`, `edition_id`, `builder`).
    #[serde(default)]
    pub assertion_id: Option<String>,
    /// Extension properties, including structural input-version provenance.
    #[serde(default)]
    pub attrs: serde_json::Value,
}

impl GraphEdge {
    /// Build a structural edge (no assertion; provenance carried in `attrs`).
    pub fn structural(
        src: impl Into<String>,
        edge: impl Into<String>,
        dst: impl Into<String>,
        attrs: serde_json::Value,
    ) -> Self {
        Self { src: src.into(), edge: edge.into(), dst: dst.into(), assertion_id: None, attrs }
    }

    /// Build an interpretive edge pointing at an assertion record.
    pub fn asserted(
        src: impl Into<String>,
        edge: impl Into<String>,
        dst: impl Into<String>,
        assertion_id: impl Into<String>,
    ) -> Self {
        Self {
            src: src.into(),
            edge: edge.into(),
            dst: dst.into(),
            assertion_id: Some(assertion_id.into()),
            attrs: serde_json::Value::Null,
        }
    }
}

/// Assertion families. Mirrors the `graph_assertions.assertion_kind` list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssertionKind {
    Annotation,
    ConceptLink,
    EntityLink,
    FamilyLink,
    Import,
}

/// Review decisions. Mirrors the `graph_assertions.decision` list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssertionDecision {
    Pending,
    Accepted,
    Rejected,
    Superseded,
}

/// Provenance layers. Mirrors `graph_assertions.provenance_layer` (`B` for
/// source-backed, `D` for derived/computational).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProvenanceLayer {
    B,
    D,
}

/// An immutable, versioned scholarly record: the authority behind every
/// non-structural edge. Adjacency points here; it never lives inside the
/// adjacency row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assertion {
    /// Stable assertion ID referenced by [`GraphEdge::assertion_id`].
    pub id: String,
    /// Assertion family.
    pub kind: AssertionKind,
    /// The claim being made (typed payload).
    #[serde(default)]
    pub claim: serde_json::Value,
    /// Supporting evidence references.
    #[serde(default)]
    pub evidence: serde_json::Value,
    /// Exact source location (dataset, page, span, …).
    #[serde(default)]
    pub source_location: String,
    /// Reviewer identity; required unless the decision is pending.
    #[serde(default)]
    pub reviewer: Option<String>,
    /// Review decision.
    pub decision: AssertionDecision,
    /// Review timestamp, if decided.
    #[serde(default)]
    pub decided_at: Option<String>,
    /// Previous assertion this one supersedes, if any.
    #[serde(default)]
    pub supersedes_id: Option<String>,
    /// Provenance layer (`B` source-backed, `D` derived).
    pub layer: ProvenanceLayer,
    /// Producing algorithm, for layer `D`.
    #[serde(default)]
    pub algorithm: Option<String>,
    /// Algorithm version, for layer `D`.
    #[serde(default)]
    pub algorithm_version: Option<String>,
    /// Attributed confidence metadata, for layer `D`.
    #[serde(default)]
    pub confidence: Option<f64>,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: String,
}

impl Assertion {
    /// Whether this record is an effective tombstone: rejected or superseded
    /// assertions never authorize traversal or export, whatever the caller
    /// scope allows.
    pub fn is_tombstoned(&self) -> bool {
        matches!(self.decision, AssertionDecision::Rejected | AssertionDecision::Superseded)
    }

    /// Whether the record may authorize traversal or export.
    pub fn is_effective(&self) -> bool {
        !self.is_tombstoned()
    }
}

/// Build lifecycle states. Mirrors the `graph_projections.status` list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectionStatus {
    Building,
    Active,
    Superseded,
}

/// Per-projection identity, versions, and snapshot. Mirrors `graph_projections`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectionManifest {
    /// Build row ID (unique per build).
    pub id: String,
    /// Projection family ID (stable across rebuilds).
    pub projection_id: String,
    /// Builder name + version that produced this build.
    pub builder_version: String,
    /// Quran edition this projection covers.
    pub edition_id: String,
    /// Corpus generation read at build time.
    #[serde(default)]
    pub corpus_generation: u64,
    /// Dataset versions consumed by the build.
    #[serde(default)]
    pub dataset_versions: BTreeMap<String, String>,
    /// Durable dependency snapshot pinned at build reservation.
    #[serde(default)]
    pub dependency_snapshot: BTreeMap<String, String>,
    /// Build lifecycle state.
    pub status: ProjectionStatus,
    /// Opaque builder manifest payload.
    #[serde(default)]
    pub manifest: serde_json::Value,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: String,
}

/// Enforced traversal and result budgets. Every query carries one; limits are
/// validated before execution ([`QueryBudgets::check`]) and enforced while
/// running. A trailing SQL `LIMIT` is not a substitute: expansion itself is
/// capped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryBudgets {
    /// Maximum traversal depth in hops.
    pub max_hops: usize,
    /// Maximum distinct nodes collected per query.
    pub max_nodes: usize,
    /// Maximum paths returned per path query.
    pub max_paths: usize,
    /// Maximum edge relaxations performed per query.
    pub max_edges: usize,
    /// Maximum neighbors expanded per single node visit (high-degree guard).
    pub max_fanout: usize,
    /// Wall-clock budget in milliseconds (adapters enforce natively; the
    /// reference backend honors the cancellation flag).
    pub timeout_ms: u64,
}

impl Default for QueryBudgets {
    fn default() -> Self {
        Self {
            max_hops: 6,
            max_nodes: 500,
            max_paths: 10,
            max_edges: 2000,
            max_fanout: 128,
            timeout_ms: 5000,
        }
    }
}

impl QueryBudgets {
    /// Validate limits before execution. Violations are errors, not
    /// truncations: a query that cannot legally run never runs.
    pub fn check(&self) -> Result<(), GraphError> {
        let mut bad: Option<String> = None;
        if !(1..=32).contains(&self.max_hops) {
            bad = Some(format!("max_hops {} out of range 1..=32", self.max_hops));
        } else if !(1..=100_000).contains(&self.max_nodes) {
            bad = Some(format!("max_nodes {} out of range 1..=100000", self.max_nodes));
        } else if !(1..=1000).contains(&self.max_paths) {
            bad = Some(format!("max_paths {} out of range 1..=1000", self.max_paths));
        } else if !(1..=200_000).contains(&self.max_edges) {
            bad = Some(format!("max_edges {} out of range 1..=200000", self.max_edges));
        } else if !(1..=10_000).contains(&self.max_fanout) {
            bad = Some(format!("max_fanout {} out of range 1..=10000", self.max_fanout));
        } else if !(1..=300_000).contains(&self.timeout_ms) {
            bad = Some(format!("timeout_ms {} out of range 1..=300000", self.timeout_ms));
        }
        match bad {
            Some(detail) => Err(GraphError::BudgetExceeded { detail }),
            None => Ok(()),
        }
    }
}

/// A bounded node/edge collection. `truncated == false` with empty `nodes`
/// means the answer is genuinely empty; `truncated == true` means a budget or
/// cancellation stopped the search before completeness was proven.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraversalResult {
    /// Collected nodes, sorted by stable ID.
    #[serde(default)]
    pub nodes: Vec<GraphNode>,
    /// Collected edges, sorted by `(src, edge, dst)`.
    #[serde(default)]
    pub edges: Vec<GraphEdge>,
    /// Whether a budget or cancellation cut the search short.
    pub truncated: bool,
    /// Why the search stopped early. Always `Some` when `truncated`.
    #[serde(default)]
    pub incomplete_reason: Option<String>,
    /// Nodes expanded during the search.
    #[serde(default)]
    pub expanded_nodes: usize,
    /// Edge relaxations performed during the search.
    #[serde(default)]
    pub expanded_edges: usize,
}

impl TraversalResult {
    /// A complete empty result: genuinely nothing found, budgets intact.
    pub fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            truncated: false,
            incomplete_reason: None,
            expanded_nodes: 0,
            expanded_edges: 0,
        }
    }

    /// A partial result cut short by a budget or cancellation.
    pub fn incomplete(
        nodes: Vec<GraphNode>,
        edges: Vec<GraphEdge>,
        reason: impl Into<String>,
        expanded_nodes: usize,
        expanded_edges: usize,
    ) -> Self {
        Self {
            nodes,
            edges,
            truncated: true,
            incomplete_reason: Some(reason.into()),
            expanded_nodes,
            expanded_edges,
        }
    }
}

/// One ordered path: node IDs plus the edges joining them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphPath {
    /// Stable IDs from source to destination, inclusive.
    pub node_ids: Vec<String>,
    /// Edges joining consecutive `node_ids`; always `node_ids.len() - 1`.
    #[serde(default)]
    pub edges: Vec<GraphEdge>,
}

/// Up-to-K bounded paths with explicit truncation semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathsResult {
    /// Paths found, shortest first, ties in stable-ID order.
    #[serde(default)]
    pub paths: Vec<GraphPath>,
    /// Whether a budget or cancellation cut enumeration short.
    pub truncated: bool,
    /// Why enumeration stopped early. Always `Some` when `truncated`.
    #[serde(default)]
    pub incomplete_reason: Option<String>,
    /// Nodes expanded during the search.
    #[serde(default)]
    pub expanded_nodes: usize,
    /// Edge relaxations performed during the search.
    #[serde(default)]
    pub expanded_edges: usize,
}

impl PathsResult {
    /// A complete result: every path within budget is listed (possibly none).
    pub fn complete(paths: Vec<GraphPath>, expanded_nodes: usize, expanded_edges: usize) -> Self {
        Self { paths, truncated: false, incomplete_reason: None, expanded_nodes, expanded_edges }
    }

    /// A partial result cut short by a budget or cancellation.
    pub fn incomplete(
        paths: Vec<GraphPath>,
        reason: impl Into<String>,
        expanded_nodes: usize,
        expanded_edges: usize,
    ) -> Self {
        Self {
            paths,
            truncated: true,
            incomplete_reason: Some(reason.into()),
            expanded_nodes,
            expanded_edges,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vocabulary_has_expected_entries() {
        assert!(is_allowed_edge("CONTAINS"));
        assert!(is_allowed_edge("NEXT"));
        assert!(is_allowed_edge("MENTIONS_CONCEPT"));
        assert!(!is_allowed_edge("PRECEDES"));
        assert!(!is_allowed_edge("FOLLOWS"));
        assert!(!is_allowed_edge("contains"));
        assert_eq!(EDGE_VOCABULARY.len(), 15);
    }

    #[test]
    fn budgets_check_rejects_zero_and_huge() {
        let mut b = QueryBudgets::default();
        assert!(b.check().is_ok());
        b.max_hops = 0;
        assert!(matches!(b.check(), Err(GraphError::BudgetExceeded { .. })));
        b = QueryBudgets::default();
        b.max_nodes = 1_000_000;
        assert!(matches!(b.check(), Err(GraphError::BudgetExceeded { .. })));
    }

    #[test]
    fn tombstone_semantics() {
        let mk = |decision| Assertion {
            id: "a".into(),
            kind: AssertionKind::Annotation,
            claim: serde_json::Value::Null,
            evidence: serde_json::Value::Null,
            source_location: String::new(),
            reviewer: None,
            decision,
            decided_at: None,
            supersedes_id: None,
            layer: ProvenanceLayer::B,
            algorithm: None,
            algorithm_version: None,
            confidence: None,
            created_at: String::new(),
        };
        assert!(mk(AssertionDecision::Accepted).is_effective());
        assert!(mk(AssertionDecision::Pending).is_effective());
        assert!(mk(AssertionDecision::Rejected).is_tombstoned());
        assert!(mk(AssertionDecision::Superseded).is_tombstoned());
    }
}
