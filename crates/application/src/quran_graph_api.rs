//! Graph read services with the full explainability payload (Phase 4,
//! D-09/D-10/D-14).
//!
//! # application::quran_graph_api
//!
//! Thin dispatch over [`crate::quran_graph_store::SqliteGraphStore`] plus the
//! existing [`quran_graph::traverse`] helpers. Routing lives here so `server`
//! and `cli` gain no new workspace edges (`arch-check`): both crates already
//! depend on `application`. No new traversal algorithm is implemented here —
//! path modes reuse [`quran_graph::traverse::up_to_k_paths`] (which drives the
//! port's `bounded_paths` BFS) and the other modes call the port operations
//! directly, so SQLite-backed reads inherit the reference budget,
//! authorization, cancellation, and ordering semantics by construction.
//!
//! Every result carries the full explainability payload ([`Explanation`]):
//! start/end nodes, ordered traversed node IDs plus per-edge type and
//! provenance (structural `input_version` vs assertion ID with reviewer,
//! decision, and timestamp), applied filters (budgets, edge filter,
//! direction, authz scope descriptor — never hidden assertion IDs), snapshot
//! identity (`projection_id` + `builder_version` + `corpus_generation`),
//! completion (`truncated` + `incomplete_reason`), and query duration in
//! milliseconds.
//!
//! No-path-requires-completeness is enforced in the result types: a truncated
//! search reports reachability as unknown (`None`), never as absent, and
//! [`ShortestOutput::no_path_proven`] is true only for complete-empty
//! results.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use async_trait::async_trait;
use storage::Database as _;
use storage_sqlite::SqliteDatabase;

use crate::quran_graph_store::SqliteGraphStore;
use crate::quran_morphology::{self, MorphologyToolError};
use quran_graph::{
    Assertion, AuthzScope, Direction, EdgeFilter, GraphEdge, GraphError, GraphPath, GraphStore,
    NodeKind, Pattern, PatternStep, ProjectionManifest, QueryBudgets, validate_pattern,
};

/// Read-service failures: graph errors plus the typed morphology-dataset
/// gate for word-root reads (never an empty result when no dataset is
/// active).
#[derive(Debug, Clone, thiserror::Error)]
pub enum GraphApiError {
    /// A typed graph failure (`QAI-GRAPH-*`).
    #[error(transparent)]
    Graph(#[from] GraphError),
    /// A typed morphology failure (`QAI-MORPH-*`), including the
    /// active-dataset gate for word-root reads.
    #[error(transparent)]
    Morphology(#[from] MorphologyToolError),
}

impl GraphApiError {
    /// Selector rejection for surfaces that cannot name [`GraphError`]
    /// variants (no `quran-graph` edge): unknown path modes, directions,
    /// node kinds, and other malformed selectors. Always `QAI-GRAPH-0003`.
    pub fn rejected(detail: impl Into<String>) -> Self {
        Self::Graph(GraphError::PatternRejected { detail: detail.into() })
    }

    /// Unknown-node error for surface fakes (`QAI-GRAPH-0004`).
    pub fn unknown_node(stable_id: impl Into<String>) -> Self {
        Self::Graph(GraphError::NodeNotFound { stable_id: stable_id.into() })
    }

    /// Access-denied error for surface fakes (`QAI-GRAPH-0006`). The detail
    /// must describe the refusal without leaking hidden content.
    pub fn access_denied(detail: impl Into<String>) -> Self {
        Self::Graph(GraphError::AuthzDenied { detail: detail.into() })
    }
}

impl storage::error::Diagnostic for GraphApiError {
    fn code(&self) -> storage::error::DiagnosticCode {
        match self {
            Self::Graph(inner) => {
                // Mirrors `quran_graph::error::codes` numbering so the HTTP
                // status map and the CLI human form stay on one code set.
                let number = match inner {
                    GraphError::UnknownProjection { .. } => 1,
                    GraphError::BudgetExceeded { .. } => 2,
                    GraphError::PatternRejected { .. } => 3,
                    GraphError::NodeNotFound { .. } => 4,
                    GraphError::BuildFailed { .. } => 5,
                    GraphError::AuthzDenied { .. } => 6,
                    GraphError::UnknownAssertion { .. } => 7,
                };
                storage::error::DiagnosticCode::new("QAI-GRAPH", number)
            }
            Self::Morphology(inner) => inner.code(),
        }
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn location(&self) -> Option<String> {
        match self {
            Self::Graph(inner) => {
                use quran_graph::error::Diagnostic as _;
                inner.location()
            }
            Self::Morphology(inner) => inner.location(),
        }
    }

    fn cause_chain(&self) -> Vec<String> {
        match self {
            Self::Graph(inner) => {
                use quran_graph::error::Diagnostic as _;
                inner.cause_chain()
            }
            Self::Morphology(inner) => inner.cause_chain(),
        }
    }

    fn remedy(&self) -> Option<String> {
        match self {
            Self::Graph(inner) => {
                use quran_graph::error::Diagnostic as _;
                inner.remedy()
            }
            Self::Morphology(inner) => inner.remedy(),
        }
    }

    fn next_command(&self) -> Option<String> {
        match self {
            Self::Graph(inner) => {
                use quran_graph::error::Diagnostic as _;
                inner.next_command()
            }
            Self::Morphology(inner) => inner.next_command(),
        }
    }
}

/// Shared read guards carried by every graph args struct: budgets validated
/// pre-flight, the neighbor edge filter, the authorization scope applied
/// during expansion, and the cancellation flag checked per batch.
#[derive(Debug, Clone)]
pub struct ReadOptions {
    /// Traversal and result budgets (pre-flight checked before any I/O).
    pub budgets: QueryBudgets,
    /// Neighbor edge filter (honored by neighbor-scoped reads; path-family
    /// modes expand direction-agnostic and record the effective filter).
    pub filter: EdgeFilter,
    /// Assertion visibility applied during expansion, never post-filter.
    pub authz: AuthzScope,
    /// Cancellation flag checked per batch (shared: callers may flag
    /// mid-flight through a clone).
    pub cancel: Arc<AtomicBool>,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            budgets: QueryBudgets::default(),
            filter: EdgeFilter::any(),
            authz: AuthzScope::all_visible(),
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }
}

/// Arguments for the bounded-neighbors read.
#[derive(Debug, Clone)]
pub struct NeighborsArgs {
    /// Stable node ID to open around.
    pub node: String,
    /// Read guards.
    pub options: ReadOptions,
}

/// Arguments for the two-node reachability check.
#[derive(Debug, Clone)]
pub struct PathArgs {
    /// Source stable ID.
    pub from: String,
    /// Destination stable ID.
    pub to: String,
    /// Read guards.
    pub options: ReadOptions,
}

/// Arguments for up-to-K ranked paths.
#[derive(Debug, Clone)]
pub struct PathsArgs {
    /// Source stable ID.
    pub from: String,
    /// Destination stable ID.
    pub to: String,
    /// Requested path count (`>= 1`; beyond `budgets.max_paths` is a
    /// pre-flight error, never a silent cap).
    pub k: usize,
    /// Read guards.
    pub options: ReadOptions,
}

/// Arguments for the bounded multi-seed subgraph.
#[derive(Debug, Clone)]
pub struct SubgraphArgs {
    /// Seed stable IDs (unknown seeds are skipped; all-unknown is a
    /// complete-empty result, never an error).
    pub seeds: Vec<String>,
    /// Read guards.
    pub options: ReadOptions,
}

/// Arguments for the typed pattern query.
#[derive(Debug, Clone)]
pub struct PatternArgs {
    /// Typed pattern (validated before any I/O).
    pub pattern: Pattern,
    /// Seed stable IDs.
    pub seeds: Vec<String>,
    /// Read guards.
    pub options: ReadOptions,
}

/// Arguments for the root-family ranked-ayah read (lexicon-gated).
#[derive(Debug, Clone)]
pub struct RootFamilyArgs {
    /// Normalized root spelling (never re-normalized here).
    pub root: String,
    /// Result cap (`>= 1` after clamping).
    pub limit: usize,
}

/// Optional per-field budget overrides for the HTTP/tool surfaces. Mirrors
/// [`crate::quran_cli::GraphBudgets`]: `None` keeps the default; explicit
/// values (including 0) fail pre-flight, never clamp — so boundary values
/// behave identically across surfaces.
#[derive(Debug, Clone, Default)]
pub struct BudgetPatch {
    /// Maximum traversal depth in hops.
    pub max_hops: Option<usize>,
    /// Maximum distinct nodes collected per query.
    pub max_nodes: Option<usize>,
    /// Maximum edge relaxations performed per query.
    pub max_edges: Option<usize>,
    /// Maximum paths returned per path query.
    pub max_paths: Option<usize>,
    /// Maximum neighbors expanded per single node visit.
    pub max_fanout: Option<usize>,
    /// Wall-clock budget in milliseconds.
    pub timeout_ms: Option<u64>,
}

impl BudgetPatch {
    /// Fold into concrete budgets. `default_hops` is the surface default
    /// when `max_hops` is absent (CLI verbs use their `--hops` default:
    /// neighbors 1, path/subgraph/pattern 4).
    pub fn into_budgets(self, default_hops: usize) -> QueryBudgets {
        let defaults = QueryBudgets::default();
        QueryBudgets {
            max_hops: self.max_hops.unwrap_or(default_hops),
            max_nodes: self.max_nodes.unwrap_or(defaults.max_nodes),
            max_edges: self.max_edges.unwrap_or(defaults.max_edges),
            max_paths: self.max_paths.unwrap_or(defaults.max_paths),
            max_fanout: self.max_fanout.unwrap_or(defaults.max_fanout),
            timeout_ms: self.timeout_ms.unwrap_or(defaults.timeout_ms),
        }
    }
}

/// Build [`ReadOptions`] from surface primitives, running the same
/// pre-flight validation as the read services: out-of-range budgets and
/// unknown direction spellings are typed [`GraphError`]s before any I/O.
pub fn read_options(
    patch: BudgetPatch,
    default_hops: usize,
    edge_types: Option<Vec<String>>,
    direction: Option<&str>,
) -> Result<ReadOptions, GraphApiError> {
    let budgets = patch.into_budgets(default_hops);
    budgets.check().map_err(GraphApiError::Graph)?;
    let direction = match direction {
        None | Some("both") => Direction::Both,
        Some("outgoing") => Direction::Outgoing,
        Some("incoming") => Direction::Incoming,
        Some(other) => {
            return Err(GraphApiError::rejected(format!(
                "unknown direction '{other}'; use both|outgoing|incoming"
            )));
        }
    };
    Ok(ReadOptions {
        budgets,
        filter: EdgeFilter { edge_types, direction },
        authz: AuthzScope::all_visible(),
        cancel: Arc::new(AtomicBool::new(false)),
    })
}

/// Path search mode selector shared by the CLI and HTTP surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathMode {
    /// Two-node reachability check (min-hop proof, never a path render).
    Reachability,
    /// Shortest min-hop path between two nodes.
    Shortest,
    /// Up-to-K ranked paths between two nodes.
    Paths,
}

/// Parse a path `mode` selector (`reachability`|`shortest`|`paths`,
/// default `paths` mirroring the CLI verb). Unknown spellings are
/// [`GraphApiError::rejected`] before any I/O.
pub fn parse_path_mode(raw: Option<&str>) -> Result<PathMode, GraphApiError> {
    match raw {
        None | Some("paths") => Ok(PathMode::Paths),
        Some("reachability") => Ok(PathMode::Reachability),
        Some("shortest") => Ok(PathMode::Shortest),
        Some(other) => Err(GraphApiError::rejected(format!(
            "unknown path mode '{other}'; use reachability|shortest|paths"
        ))),
    }
}

/// One HTTP/tool pattern step: an edge predicate plus an optional
/// node-kind constraint (`EDGE[:Kind]`, the CLI `--step` spelling split
/// into fields).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatternStepRequest {
    /// Edge predicate; must be in the edge vocabulary.
    pub edge: String,
    /// Optional node-kind constraint (`ayah`, `token`, `root`, …).
    pub node_kind: Option<String>,
}

fn parse_node_kind(raw: &str) -> Result<NodeKind, GraphApiError> {
    match raw {
        "edition" => Ok(NodeKind::Edition),
        "surah" => Ok(NodeKind::Surah),
        "ayah" => Ok(NodeKind::Ayah),
        "token" => Ok(NodeKind::Token),
        "division" => Ok(NodeKind::Division),
        "root" => Ok(NodeKind::Root),
        "lemma" => Ok(NodeKind::Lemma),
        "concept" => Ok(NodeKind::Concept),
        "entity" => Ok(NodeKind::Entity),
        "annotation" => Ok(NodeKind::Annotation),
        _ => Err(GraphApiError::rejected(format!(
            "unknown node kind '{raw}'; use edition|surah|ayah|token|division|root|lemma|concept|entity|annotation"
        ))),
    }
}

/// Build a validated [`Pattern`] from surface step requests: unknown kinds
/// and edges are typed rejections before any I/O (the service re-validates
/// before execution, so direct service callers keep the same guarantee).
pub fn parse_pattern_steps(steps: Vec<PatternStepRequest>) -> Result<Pattern, GraphApiError> {
    let mut parsed = Vec::with_capacity(steps.len());
    for req in steps {
        let step = match req.node_kind.as_deref() {
            None => PatternStep::edge(req.edge),
            Some(kind) => PatternStep::edge_to(req.edge, parse_node_kind(kind)?),
        };
        parsed.push(step);
    }
    let pattern = Pattern::new(parsed);
    validate_pattern(&pattern).map_err(GraphApiError::Graph)?;
    Ok(pattern)
}

/// Active-edition read context for HTTP envelopes and ETags: edition
/// identity plus the active corpus generation. A fresh database with no
/// active edition degrades to empty identity (no ETag), never a fabricated
/// edition.
#[derive(Debug, Clone, Default)]
pub struct GraphSnapshotMeta {
    /// Active edition slug.
    pub edition_slug: String,
    /// Active edition version.
    pub edition_version: String,
    /// Active edition row id (for tool-edition attribution).
    pub edition_id: String,
    /// Active edition text hash (`sha256:<hex>`) for ETags.
    pub text_hash: String,
    /// Active corpus generation.
    pub corpus_generation: u64,
}

fn snapshot_storage_error(error: storage::error::StorageError) -> GraphApiError {
    GraphApiError::Graph(GraphError::BuildFailed {
        stage: "snapshot-meta".to_string(),
        detail: error.to_string(),
    })
}

async fn active_snapshot_meta(db: &SqliteDatabase) -> Result<GraphSnapshotMeta, GraphApiError> {
    let mut uow = db.write().await.map_err(snapshot_storage_error)?;
    let active = uow.quran().get_active().await.map_err(snapshot_storage_error)?;
    let meta = match active {
        None => GraphSnapshotMeta::default(),
        Some(row) => {
            let edition =
                uow.quran().get_edition(&row.edition_id).await.map_err(snapshot_storage_error)?;
            let generation = u64::try_from(row.corpus_generation).unwrap_or(0);
            match edition {
                None => GraphSnapshotMeta {
                    corpus_generation: generation,
                    ..GraphSnapshotMeta::default()
                },
                Some(ed) => GraphSnapshotMeta {
                    edition_slug: ed.slug,
                    edition_version: ed.version,
                    edition_id: row.edition_id,
                    text_hash: ed.text_hash,
                    corpus_generation: generation,
                },
            }
        }
    };
    uow.rollback().await.map_err(snapshot_storage_error)?;
    Ok(meta)
}

/// Per-edge provenance: structural edges point at corpus input versions,
/// asserted edges point at review records. Unknown assertion IDs are
/// reported explicitly, never hidden and never invented.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProvenanceExplanation {
    /// Deterministic structural edge; provenance lives in the edge attrs.
    Structural {
        /// Corpus input version stamped at build time, when present.
        #[serde(default)]
        input_version: Option<String>,
    },
    /// Interpretive edge licensed by a review record.
    Asserted {
        /// Assertion ID.
        id: String,
        /// Reviewer identity, when decided.
        #[serde(default)]
        reviewer: Option<String>,
        /// Review decision (`pending`/`accepted`/`disputed`/…).
        decision: String,
        /// Decision timestamp, when decided.
        #[serde(default)]
        decided_at: Option<String>,
        /// Producing algorithm, for layer-D (computational) suggestions.
        #[serde(default)]
        algorithm: Option<String>,
        /// Algorithm version, for layer-D suggestions.
        #[serde(default)]
        algorithm_version: Option<String>,
        /// Attributed confidence metadata, for layer-D suggestions.
        #[serde(default)]
        confidence: Option<f64>,
    },
    /// The edge points at an assertion ID with no authority record. The
    /// edge stays visible under an unrestricted scope (port semantics); the
    /// gap is reported, never papered over.
    UnknownAssertion {
        /// Dangling assertion ID.
        id: String,
    },
}

/// One traversed edge with its type and provenance.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EdgeExplanation {
    /// Source stable ID.
    pub src: String,
    /// Edge predicate.
    pub edge: String,
    /// Destination stable ID.
    pub dst: String,
    /// Per-edge provenance.
    pub provenance: ProvenanceExplanation,
}

/// One ordered path with node IDs plus explained edges.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PathExplanation {
    /// Stable IDs from source to destination, inclusive.
    pub node_ids: Vec<String>,
    /// Explained edges joining consecutive `node_ids`.
    pub edges: Vec<EdgeExplanation>,
}

/// The applied-filters block: exactly what constrained the query. The authz
/// scope is a descriptor (never hidden assertion IDs).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AppliedFilters {
    /// Budgets enforced during expansion.
    pub budgets: QueryBudgets,
    /// Edge predicates admitted (`None` admits every allowlisted predicate).
    #[serde(default)]
    pub edge_types: Option<Vec<String>>,
    /// Expansion direction (`both`/`outgoing`/`incoming`).
    pub direction: String,
    /// Authz descriptor: `unrestricted`, or `restricted(N assertions)`.
    pub authz: String,
}

/// Snapshot identity: which projection build answered.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SnapshotIdentity {
    /// Projection family ID.
    pub projection_id: String,
    /// Builder name + version that produced the build.
    pub builder_version: String,
    /// Corpus generation read at build time.
    pub corpus_generation: u64,
}

impl From<&ProjectionManifest> for SnapshotIdentity {
    fn from(manifest: &ProjectionManifest) -> Self {
        Self {
            projection_id: manifest.projection_id.clone(),
            builder_version: manifest.builder_version.clone(),
            corpus_generation: manifest.corpus_generation,
        }
    }
}

/// Full explainability payload (D-14) attached to every read result.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Explanation {
    /// Start node, when the read has one.
    #[serde(default)]
    pub start: Option<String>,
    /// End node, for two-node reads.
    #[serde(default)]
    pub end: Option<String>,
    /// Ordered traversed node IDs (sorted stable-ID order).
    #[serde(default)]
    pub nodes: Vec<String>,
    /// Per-edge type plus provenance for every traversed edge.
    #[serde(default)]
    pub edges: Vec<EdgeExplanation>,
    /// Explained paths (path modes only; empty otherwise).
    #[serde(default)]
    pub paths: Vec<PathExplanation>,
    /// Applied filters.
    pub applied_filters: AppliedFilters,
    /// Snapshot identity.
    pub snapshot: SnapshotIdentity,
    /// Whether a budget or cancellation cut the search short.
    pub truncated: bool,
    /// Why the search stopped early. Always `Some` when `truncated`.
    #[serde(default)]
    pub incomplete_reason: Option<String>,
    /// Query duration in milliseconds (presentational only).
    pub duration_ms: u64,
}

/// Bounded-neighbors result with explanation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NeighborsOutput {
    /// Neighbor nodes (sorted by stable ID).
    pub nodes: Vec<quran_graph::GraphNode>,
    /// Traversed edges (sorted by `(src, edge, dst)`).
    pub edges: Vec<GraphEdge>,
    /// Whether a budget or cancellation cut the search short.
    pub truncated: bool,
    /// Why the search stopped early.
    #[serde(default)]
    pub incomplete_reason: Option<String>,
    /// Full explainability payload.
    pub explanation: Explanation,
}

/// Two-node reachability result. `reachable` is `None` whenever the search
/// truncated: a partial search never reports absence.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReachabilityOutput {
    /// Whether `to` is reachable from `from` (`None` when truncated).
    #[serde(default)]
    pub reachable: Option<bool>,
    /// Fewest hops, when proven (`None` when truncated or unreachable).
    #[serde(default)]
    pub hops: Option<usize>,
    /// Full explainability payload.
    pub explanation: Explanation,
}

/// Shortest min-hop path result. Absence is proven only when the search
/// completed: [`ShortestOutput::no_path_proven`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShortestOutput {
    /// The min-hop path, when one exists within budget.
    #[serde(default)]
    pub path: Option<GraphPath>,
    /// Full explainability payload.
    pub explanation: Explanation,
}

impl ShortestOutput {
    /// True only for a complete-empty result: no path exists within the
    /// visible projection. A truncated search with no path yet found is
    /// unknown, never absent.
    pub fn no_path_proven(&self) -> bool {
        self.path.is_none() && !self.explanation.truncated
    }
}

/// Up-to-K ranked paths result (shortest first, stable-ID tie-break).
/// Complete-empty (`truncated == false` with no paths) is the only proven
/// no-path claim; any truncated result refuses it.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PathsOutput {
    /// Ranked paths, shortest first.
    #[serde(default)]
    pub paths: Vec<GraphPath>,
    /// Whether enumeration stopped early.
    pub truncated: bool,
    /// Why enumeration stopped early.
    #[serde(default)]
    pub incomplete_reason: Option<String>,
    /// Full explainability payload.
    pub explanation: Explanation,
}

impl PathsOutput {
    /// True only for a complete-empty result: no path exists within the
    /// visible projection and budgets. Mirrors [`ShortestOutput`].
    pub fn no_path_proven(&self) -> bool {
        self.paths.is_empty() && !self.truncated
    }
}

/// Bounded-subgraph result with explanation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubgraphOutput {
    /// Collected nodes (sorted by stable ID).
    pub nodes: Vec<quran_graph::GraphNode>,
    /// Collected edges (sorted by `(src, edge, dst)`).
    pub edges: Vec<GraphEdge>,
    /// Whether a budget or cancellation cut the search short.
    pub truncated: bool,
    /// Why the search stopped early.
    #[serde(default)]
    pub incomplete_reason: Option<String>,
    /// Full explainability payload.
    pub explanation: Explanation,
}

/// Typed-pattern result with explanation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatternOutput {
    /// Matched nodes (sorted by stable ID).
    pub nodes: Vec<quran_graph::GraphNode>,
    /// Traversed edges (sorted by `(src, edge, dst)`).
    pub edges: Vec<GraphEdge>,
    /// Whether a budget or cancellation cut the search short.
    pub truncated: bool,
    /// Why the search stopped early.
    #[serde(default)]
    pub incomplete_reason: Option<String>,
    /// Full explainability payload.
    pub explanation: Explanation,
}

/// One ranked ayah in a root family (position deduplicated per ayah).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RankedAyah {
    /// Surah number.
    pub surah: i64,
    /// Ayah number.
    pub ayah: i64,
    /// Token position of the representative occurrence.
    pub position: i64,
}

/// Root-family result: ranked ayahs with dataset attribution. The read
/// resolves through the active-dataset gate: no active dataset is the typed
/// [`MorphologyToolError::UnavailableDataset`], never an empty family.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootFamilyOutput {
    /// Queried root spelling.
    pub root: String,
    /// Active dataset identity (`<slug>@<version>`).
    pub dataset: String,
    /// Ranked ayahs (sorted, deduplicated per ayah, capped at `limit`).
    pub ayahs: Vec<RankedAyah>,
    /// Applied result cap.
    pub limit: usize,
    /// Query duration in milliseconds (presentational only).
    pub duration_ms: u64,
}

/// Read-only graph backend. Implemented by [`GraphApiService`]; faked in
/// surface contract tests.
#[async_trait]
pub trait GraphBackend: Send + Sync {
    /// Bounded neighbors of one node with per-edge provenance.
    async fn neighbors(&self, args: NeighborsArgs) -> Result<NeighborsOutput, GraphApiError>;
    /// Two-node reachability check (min-hop proof, never a path render).
    async fn reachability(&self, args: PathArgs) -> Result<ReachabilityOutput, GraphApiError>;
    /// Shortest min-hop path between two nodes.
    async fn shortest_path(&self, args: PathArgs) -> Result<ShortestOutput, GraphApiError>;
    /// Up-to-K ranked paths between two nodes.
    async fn paths(&self, args: PathsArgs) -> Result<PathsOutput, GraphApiError>;
    /// Bounded multi-seed subgraph.
    async fn subgraph(&self, args: SubgraphArgs) -> Result<SubgraphOutput, GraphApiError>;
    /// Typed pattern query from seeds.
    async fn pattern(&self, args: PatternArgs) -> Result<PatternOutput, GraphApiError>;
    /// Root-family ranked ayahs (lexicon-gated).
    async fn root_family(&self, args: RootFamilyArgs) -> Result<RootFamilyOutput, GraphApiError>;
    /// Active-edition read context for HTTP envelopes and ETags: edition
    /// identity plus the active corpus generation. Faked in surface
    /// contract tests.
    async fn snapshot_meta(&self) -> Result<GraphSnapshotMeta, GraphApiError>;
}

/// Live read backend over one pinned projection build plus the active
/// morphology dataset for word-root reads.
pub struct GraphApiService {
    db: Arc<SqliteDatabase>,
    store: SqliteGraphStore,
}

impl GraphApiService {
    /// Open the active build of `projection_id` in the database at `db_path`.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::UnknownProjection`] when the family has no
    /// active build, or [`GraphError::BuildFailed`] when the database cannot
    /// be opened or the pinned rows are corrupt.
    pub async fn open_for_projection(
        db_path: &str,
        projection_id: &str,
    ) -> Result<Self, GraphError> {
        let db = SqliteDatabase::new(db_path, 4, true).await.map_err(|error| {
            GraphError::BuildFailed { stage: "open".to_string(), detail: error.to_string() }
        })?;
        let store = SqliteGraphStore::open_active(&db, projection_id).await?;
        Ok(Self { db: Arc::new(db), store })
    }

    /// Open the active structural projection (the default read scope every
    /// CLI verb resolves).
    ///
    /// # Errors
    ///
    /// Same as [`GraphApiService::open_for_projection`].
    pub async fn open(db_path: &str) -> Result<Self, GraphError> {
        Self::open_for_projection(db_path, quran_graph::STRUCTURAL_PROJECTION_ID).await
    }

    /// The pinned build's manifest.
    pub fn manifest(&self) -> &ProjectionManifest {
        self.store.manifest()
    }

    /// The read pool behind this service (for reader-resolved quotations and
    /// staleness probes that travel beside read results).
    pub fn database(&self) -> &Arc<SqliteDatabase> {
        &self.db
    }

    /// The full pinned node/edge/assertion sets for export surfaces.
    /// Deterministic order (stable-ID, then assertion-ID).
    pub fn export_sets(&self) -> (Vec<quran_graph::GraphNode>, Vec<GraphEdge>, Vec<Assertion>) {
        self.store.export_sets()
    }

    /// Look up an authority record by ID for provenance rendering.
    fn assertion(&self, id: &str) -> Option<&Assertion> {
        self.store.get_assertion(id)
    }

    fn explain_edge(&self, edge: &GraphEdge) -> EdgeExplanation {
        let assertion_id = edge.assertion_id.as_deref();
        let assertion = assertion_id.and_then(|id| self.assertion(id));
        // Structural edges pass `None`; asserted edges resolve (or report
        // the gap when the authority row is missing).
        match (assertion_id, assertion) {
            (None, _) => explain_edge_with(edge, None),
            (Some(_), record) => explain_edge_with(edge, record),
        }
    }

    fn explain_path(&self, path: &GraphPath) -> PathExplanation {
        PathExplanation {
            node_ids: path.node_ids.clone(),
            edges: path.edges.iter().map(|edge| self.explain_edge(edge)).collect(),
        }
    }

    /// Assemble the explainability payload. `effective_filter` is the filter
    /// actually enforced during expansion (path-family modes expand
    /// direction-agnostic over every allowlisted predicate).
    #[allow(clippy::too_many_arguments)]
    fn explain(
        &self,
        start: Option<String>,
        end: Option<String>,
        nodes: &[String],
        edges: &[GraphEdge],
        paths: &[GraphPath],
        effective_filter: &EdgeFilter,
        options: &ReadOptions,
        truncated: bool,
        incomplete_reason: Option<String>,
        duration_ms: u64,
    ) -> Explanation {
        Explanation {
            start,
            end,
            nodes: nodes.to_vec(),
            edges: edges.iter().map(|edge| self.explain_edge(edge)).collect(),
            paths: paths.iter().map(|path| self.explain_path(path)).collect(),
            applied_filters: AppliedFilters {
                budgets: options.budgets.clone(),
                edge_types: effective_filter.edge_types.clone(),
                direction: direction_name(effective_filter),
                authz: authz_descriptor(&options.authz),
            },
            snapshot: SnapshotIdentity::from(self.store.manifest()),
            truncated,
            incomplete_reason,
            duration_ms,
        }
    }
}

/// Authz scope descriptor: shape only, never assertion IDs. Shared with
/// file-backed CLI reads so both surfaces name scopes identically.
pub fn describe_authz(authz: &AuthzScope) -> String {
    authz_descriptor(authz)
}

/// Expansion-direction descriptor shared with file-backed CLI reads.
pub fn describe_direction(filter: &EdgeFilter) -> String {
    direction_name(filter)
}

/// Explain one edge against an already-resolved authority record (`None`
/// for structural edges or file-backed reads without an authority table).
/// Unknown IDs (a `Some` assertion ID with no record) are reported
/// explicitly, never hidden and never invented.
pub fn explain_edge_with(edge: &GraphEdge, assertion: Option<&Assertion>) -> EdgeExplanation {
    let provenance = match edge.assertion_id.as_deref() {
        None => ProvenanceExplanation::Structural {
            input_version: edge
                .attrs
                .get("input_version")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
        },
        Some(id) => match assertion {
            Some(record) => ProvenanceExplanation::Asserted {
                id: id.to_string(),
                reviewer: record.reviewer.clone(),
                decision: decision_name(record.decision),
                decided_at: record.decided_at.clone(),
                algorithm: record.algorithm.clone(),
                algorithm_version: record.algorithm_version.clone(),
                confidence: record.confidence,
            },
            None => ProvenanceExplanation::UnknownAssertion { id: id.to_string() },
        },
    };
    EdgeExplanation {
        src: edge.src.clone(),
        edge: edge.edge.clone(),
        dst: edge.dst.clone(),
        provenance,
    }
}

/// Validate a requested up-to-K count against the path budget before any
/// I/O: K beyond `max_paths` is a pre-flight error (T-04-07), never a
/// silent cap. Shared by the service and file-backed CLI reads.
///
/// # Errors
///
/// Returns [`GraphError::BudgetExceeded`] when `k` is zero or exceeds
/// `budgets.max_paths`.
pub fn validate_k(k: usize, budgets: &QueryBudgets) -> Result<(), GraphError> {
    if k == 0 {
        return Err(GraphError::BudgetExceeded { detail: "k must be at least 1".to_string() });
    }
    if k > budgets.max_paths {
        return Err(GraphError::BudgetExceeded {
            detail: format!(
                "k {} exceeds max_paths {}; raise max_paths explicitly or request fewer paths",
                k, budgets.max_paths
            ),
        });
    }
    Ok(())
}

/// Rank root occurrences into deduplicated ayahs: sorted by
/// `(surah, ayah, position)`, one entry per ayah, capped at `limit`
/// (clamped to `>= 1`). The single implementation behind the service and
/// the CLI verb, so both surfaces rank identically.
///
/// Returns the ranked ayahs plus the applied cap.
pub fn rank_family_ayahs(
    mut occurrences: Vec<quran_morphology::RootOccurrence>,
    limit: usize,
) -> (Vec<RankedAyah>, usize) {
    let cap = limit.max(1);
    occurrences.sort_by_key(|hit| (hit.surah, hit.ayah, hit.position));
    occurrences.dedup_by_key(|hit| (hit.surah, hit.ayah));
    occurrences.truncate(cap);
    let ayahs = occurrences
        .into_iter()
        .map(|hit| RankedAyah { surah: hit.surah, ayah: hit.ayah, position: hit.position })
        .collect();
    (ayahs, cap)
}

fn decision_name(decision: quran_graph::AssertionDecision) -> String {
    use quran_graph::AssertionDecision as D;
    match decision {
        D::Pending => "pending",
        D::Accepted => "accepted",
        D::Rejected => "rejected",
        D::Superseded => "superseded",
        D::Disputed => "disputed",
    }
    .to_string()
}

fn direction_name(filter: &EdgeFilter) -> String {
    use quran_graph::Direction as D;
    match filter.direction {
        D::Both => "both",
        D::Outgoing => "outgoing",
        D::Incoming => "incoming",
    }
    .to_string()
}

/// Authz scope descriptor: shape only, never assertion IDs.
fn authz_descriptor(authz: &AuthzScope) -> String {
    match &authz.visible_assertions {
        None => "unrestricted".to_string(),
        Some(set) => format!("restricted({} assertions)", set.len()),
    }
}

fn elapsed_ms(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

#[async_trait]
impl GraphBackend for GraphApiService {
    async fn neighbors(&self, args: NeighborsArgs) -> Result<NeighborsOutput, GraphApiError> {
        let start = Instant::now();
        args.options.budgets.check()?;
        let result = self.store.neighbors(
            &args.node,
            &args.options.filter,
            &args.options.budgets,
            &args.options.cancel,
            &args.options.authz,
        )?;
        let nodes: Vec<String> = result.nodes.iter().map(|node| node.stable_id.clone()).collect();
        let explanation = self.explain(
            Some(args.node.clone()),
            None,
            &nodes,
            &result.edges,
            &[],
            &args.options.filter,
            &args.options,
            result.truncated,
            result.incomplete_reason.clone(),
            elapsed_ms(start),
        );
        Ok(NeighborsOutput {
            nodes: result.nodes,
            edges: result.edges,
            truncated: result.truncated,
            incomplete_reason: result.incomplete_reason,
            explanation,
        })
    }

    async fn reachability(&self, args: PathArgs) -> Result<ReachabilityOutput, GraphApiError> {
        let start = Instant::now();
        args.options.budgets.check()?;
        let outcome = quran_graph::traverse::min_hops(
            &self.store,
            &args.from,
            &args.to,
            &args.options.budgets,
            &args.options.cancel,
            &args.options.authz,
        )?;
        // No-path-requires-completeness: a truncated search reports unknown,
        // never absence.
        let (reachable, hops) = if outcome.truncated {
            (None, None)
        } else {
            (Some(outcome.hops.is_some()), outcome.hops)
        };
        let explanation = self.explain(
            Some(args.from.clone()),
            Some(args.to.clone()),
            &[],
            &[],
            &[],
            &EdgeFilter::any(),
            &args.options,
            outcome.truncated,
            outcome.incomplete_reason.clone(),
            elapsed_ms(start),
        );
        Ok(ReachabilityOutput { reachable, hops, explanation })
    }

    async fn shortest_path(&self, args: PathArgs) -> Result<ShortestOutput, GraphApiError> {
        let start = Instant::now();
        args.options.budgets.check()?;
        let result = quran_graph::traverse::up_to_k_paths(
            &self.store,
            &args.from,
            &args.to,
            1,
            &args.options.budgets,
            &args.options.cancel,
            &args.options.authz,
        )?;
        let paths = result.paths.clone();
        let path = paths.first().cloned();
        let node_ids: Vec<String> =
            path.as_ref().map(|found| found.node_ids.clone()).unwrap_or_default();
        let edges: Vec<GraphEdge> =
            path.as_ref().map(|found| found.edges.clone()).unwrap_or_default();
        let explanation = self.explain(
            Some(args.from.clone()),
            Some(args.to.clone()),
            &node_ids,
            &edges,
            &paths,
            &EdgeFilter::any(),
            &args.options,
            result.truncated,
            result.incomplete_reason.clone(),
            elapsed_ms(start),
        );
        Ok(ShortestOutput { path, explanation })
    }

    async fn paths(&self, args: PathsArgs) -> Result<PathsOutput, GraphApiError> {
        let start = Instant::now();
        args.options.budgets.check()?;
        // Requesting more paths than the budget allows is a pre-flight
        // error (T-04-07), never a silent cap: the caller must either raise
        // max_paths explicitly or accept fewer paths.
        validate_k(args.k, &args.options.budgets)?;
        let result = quran_graph::traverse::up_to_k_paths(
            &self.store,
            &args.from,
            &args.to,
            args.k,
            &args.options.budgets,
            &args.options.cancel,
            &args.options.authz,
        )?;
        let explanation = self.explain(
            Some(args.from.clone()),
            Some(args.to.clone()),
            &[],
            &[],
            &result.paths,
            &EdgeFilter::any(),
            &args.options,
            result.truncated,
            result.incomplete_reason.clone(),
            elapsed_ms(start),
        );
        Ok(PathsOutput {
            truncated: result.truncated,
            incomplete_reason: result.incomplete_reason,
            paths: result.paths,
            explanation,
        })
    }

    async fn subgraph(&self, args: SubgraphArgs) -> Result<SubgraphOutput, GraphApiError> {
        let start = Instant::now();
        args.options.budgets.check()?;
        let result = quran_graph::traverse::bounded_subgraph(
            &self.store,
            &args.seeds,
            &args.options.budgets,
            &args.options.cancel,
            &args.options.authz,
        )?;
        let nodes: Vec<String> = result.nodes.iter().map(|node| node.stable_id.clone()).collect();
        let explanation = self.explain(
            args.seeds.first().cloned(),
            None,
            &nodes,
            &result.edges,
            &[],
            &EdgeFilter::any(),
            &args.options,
            result.truncated,
            result.incomplete_reason.clone(),
            elapsed_ms(start),
        );
        Ok(SubgraphOutput {
            nodes: result.nodes,
            edges: result.edges,
            truncated: result.truncated,
            incomplete_reason: result.incomplete_reason,
            explanation,
        })
    }

    async fn pattern(&self, args: PatternArgs) -> Result<PatternOutput, GraphApiError> {
        let start = Instant::now();
        args.options.budgets.check()?;
        // Typed rejection before any I/O: unknown edges and raw-text-looking
        // names never reach the store.
        validate_pattern(&args.pattern)?;
        let result = self.store.pattern_query(
            &args.pattern,
            &args.seeds,
            &args.options.budgets,
            &args.options.cancel,
            &args.options.authz,
        )?;
        let nodes: Vec<String> = result.nodes.iter().map(|node| node.stable_id.clone()).collect();
        let explanation = self.explain(
            args.seeds.first().cloned(),
            None,
            &nodes,
            &result.edges,
            &[],
            &EdgeFilter::any(),
            &args.options,
            result.truncated,
            result.incomplete_reason.clone(),
            elapsed_ms(start),
        );
        Ok(PatternOutput {
            nodes: result.nodes,
            edges: result.edges,
            truncated: result.truncated,
            incomplete_reason: result.incomplete_reason,
            explanation,
        })
    }

    async fn root_family(&self, args: RootFamilyArgs) -> Result<RootFamilyOutput, GraphApiError> {
        let start = Instant::now();
        // The same service the CLI verb calls, so both surfaces resolve
        // through the active-dataset gate with the typed unavailable error
        // (never heuristics, never an empty family).
        let (dataset, occurrences) = quran_morphology::root_search(&self.db, &args.root).await?;
        let (ayahs, cap) = rank_family_ayahs(occurrences, args.limit);
        Ok(RootFamilyOutput {
            root: args.root,
            dataset,
            ayahs,
            limit: cap,
            duration_ms: elapsed_ms(start),
        })
    }

    async fn snapshot_meta(&self) -> Result<GraphSnapshotMeta, GraphApiError> {
        active_snapshot_meta(&self.db).await
    }
}

/// Long-lived read backend over a database file (HTTP serve wiring): opens
/// the active structural projection per request, so rebuilds between
/// requests are picked up and a missing build is a per-request typed error
/// (never a serve-time failure). `root_family` is lexicon-gated and needs
/// no projection build.
pub struct FileGraphBackend {
    db_path: String,
}

impl FileGraphBackend {
    /// Read through the active structural projection (the default read scope
    /// every CLI verb resolves).
    pub fn structural(db_path: &str) -> Self {
        Self { db_path: db_path.to_string() }
    }

    async fn service(&self) -> Result<GraphApiService, GraphApiError> {
        GraphApiService::open(&self.db_path).await.map_err(GraphApiError::Graph)
    }

    async fn database(&self) -> Result<SqliteDatabase, GraphApiError> {
        SqliteDatabase::new(&self.db_path, 4, true).await.map_err(|error| {
            GraphApiError::Graph(GraphError::BuildFailed {
                stage: "open".to_string(),
                detail: error.to_string(),
            })
        })
    }
}

#[async_trait]
impl GraphBackend for FileGraphBackend {
    async fn neighbors(&self, args: NeighborsArgs) -> Result<NeighborsOutput, GraphApiError> {
        self.service().await?.neighbors(args).await
    }

    async fn reachability(&self, args: PathArgs) -> Result<ReachabilityOutput, GraphApiError> {
        self.service().await?.reachability(args).await
    }

    async fn shortest_path(&self, args: PathArgs) -> Result<ShortestOutput, GraphApiError> {
        self.service().await?.shortest_path(args).await
    }

    async fn paths(&self, args: PathsArgs) -> Result<PathsOutput, GraphApiError> {
        self.service().await?.paths(args).await
    }

    async fn subgraph(&self, args: SubgraphArgs) -> Result<SubgraphOutput, GraphApiError> {
        self.service().await?.subgraph(args).await
    }

    async fn pattern(&self, args: PatternArgs) -> Result<PatternOutput, GraphApiError> {
        self.service().await?.pattern(args).await
    }

    async fn root_family(&self, args: RootFamilyArgs) -> Result<RootFamilyOutput, GraphApiError> {
        // The lexicon-gated read needs no projection build: same service
        // call the CLI verb and `GraphApiService` make, over a per-request
        // handle.
        let start = Instant::now();
        let db = self.database().await?;
        let (dataset, occurrences) = quran_morphology::root_search(&db, &args.root).await?;
        let (ayahs, cap) = rank_family_ayahs(occurrences, args.limit);
        Ok(RootFamilyOutput {
            root: args.root,
            dataset,
            ayahs,
            limit: cap,
            duration_ms: elapsed_ms(start),
        })
    }

    async fn snapshot_meta(&self) -> Result<GraphSnapshotMeta, GraphApiError> {
        let db = self.database().await?;
        active_snapshot_meta(&db).await
    }
}
