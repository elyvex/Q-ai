//! `quran-graph`: the Quran knowledge-graph port and reference logic.
//!
//! This is a **port/trait crate**: it defines the typed graph contract —
//! model, [`GraphStore`](store::GraphStore) operations, budgets,
//! authorization scopes, batched traversal, typed patterns, the pure
//! structural builder, and Graph JSON export — plus an in-memory reference
//! backend ([`MemGraphStore`](mem::MemGraphStore)) used by conformance tests.
//!
//! The decided production backend is **SQLite adjacency with bounded
//! traversal** (ADR-0202). That adapter lives in the application layer and is
//! **not** implemented here; it must implement the same port and pass the
//! same conformance suite before activation. Likewise not implemented here:
//! the CLI (`qai graph …`), the HTTP routes, and the SQLite-backed assertion
//! authority — all application-layer concerns.
//!
//! No backend query language crosses this crate's boundary. The public API
//! exposes only typed operations over model types; backend handles and query
//! strings stay private to adapters. A dedicated test pins this: formatted
//! outputs and public API names never carry backend-language fragments.
//!
//! Module map:
//!
//! - [`error`]: `GraphError` with stable `QAI-GRAPH-*` codes.
//! - [`model`]: nodes, edges, assertions, manifests, budgets, results.
//! - [`store`]: the `GraphStore` port (node resolution, neighbors, bounded
//!   paths/subgraphs/patterns, build staging, capability discovery).
//! - [`mem`]: in-memory reference backend for conformance tests.
//! - [`traverse`]: batched frontier traversal over the port.
//! - [`pattern`]: typed, allowlisted pattern queries.
//! - [`structural`]: pure structural projection builder.
//! - [`export`]: Graph JSON export with tombstone/authorization filtering.

pub mod error;
pub mod export;
pub mod mem;
pub mod model;
pub mod pattern;
pub mod store;
pub mod structural;
pub mod traverse;

pub use error::{Diagnostic, DiagnosticCode, GraphError, codes};
pub use export::{ExportNotice, GRAPH_JSON_FORMAT, export_json, export_json_with_notice};
pub use mem::MemGraphStore;
pub use model::{
    Assertion, AssertionDecision, AssertionKind, EDGE_VOCABULARY, GraphEdge, GraphNode, GraphPath,
    NodeKind, PathsResult, ProjectionManifest, ProjectionStatus, ProvenanceLayer, QueryBudgets,
    TraversalResult, is_allowed_edge,
};
pub use pattern::{PATTERN_SIZE_BUDGET, Pattern, PatternStep, execute_pattern, validate_pattern};
pub use store::{AuthzScope, BuildInspection, Direction, EdgeFilter, GraphStore, cancel_requested};
pub use structural::{
    AyahInput, BuiltProjection, DivisionInput, STRUCTURAL_BUILDER_VERSION,
    STRUCTURAL_PROJECTION_ID, StructuralInput, SurahInput, TokenInput, ayah_stable_id,
    build_structural, division_stable_id, edition_stable_id, surah_stable_id, token_stable_id,
};
pub use traverse::{MinHopsOutcome, bounded_subgraph, min_hops, reachable, up_to_k_paths};
