//! Phase 2 — the `FullTextIndex` port (`plan.md` §4.1, P2-T29).
//!
//! The single choke point between search semantics and storage engines:
//! query planning, verification, hydration, and explanation live above this
//! trait; token streams, candidate retrieval, scoring, and lifecycle live
//! below it in backend adapters. Tantivy-specific query objects, document
//! addresses, and schema types must never escape the adapter.

use crate::error::IndexError;
use crate::model::{
    CommitStamp, FtsBackend, FtsIntegrityReport, FtsQuery, FtsResults, FtsSchema, FtsStats,
    IndexManifest, SearchOpts,
};

/// Backend-independent full-text operations over derived Quran text.
///
/// All methods are generation-scoped: a build writes a new generation, serves
/// it only after verification + atomic pointer flip, and retains the previous
/// generation for single-step rollback (I5 extended to indexes).
#[async_trait::async_trait]
pub trait FullTextIndex: Send + Sync {
    /// Which engine backs this instance.
    fn backend(&self) -> FtsBackend;

    /// Build inputs this instance serves (I14).
    fn manifest(&self) -> IndexManifest;

    /// Create the index structures for `schema` (idempotent).
    async fn create(&self, schema: &FtsSchema) -> Result<(), IndexError>;

    /// Stage a batch of documents into the in-progress generation.
    async fn add_batch(&self, docs: Vec<crate::model::FtsDoc>) -> Result<(), IndexError>;

    /// Commit the staged generation and return its receipt.
    async fn commit(&self) -> Result<CommitStamp, IndexError>;

    /// Search the active generation.
    async fn search(&self, query: &FtsQuery, opts: &SearchOpts) -> Result<FtsResults, IndexError>;

    /// Exact count for a query (separate from ranked search; never estimated).
    async fn count(&self, query: &FtsQuery) -> Result<u64, IndexError>;

    /// Delete a superseded generation; returns documents removed.
    async fn delete_by_generation(&self, generation: u64) -> Result<u64, IndexError>;

    /// Backend statistics for `doctor` and build reports.
    async fn stats(&self) -> Result<FtsStats, IndexError>;

    /// Verify doc counts, sampled round-trips, and manifest agreement.
    async fn verify(&self) -> Result<FtsIntegrityReport, IndexError>;
}
