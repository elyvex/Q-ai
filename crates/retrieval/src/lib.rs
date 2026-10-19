//! Retrieval backends: a swappable port for document retrieval.
//!
//! # retrieval
//!
//! The port here is deliberately backend-agnostic. A backend is anything that
//! can ingest a document and answer a lexical/semantic query over what it has
//! ingested. Two implementations ship:
//!
//! - [`MemvidRetrieval`] — a single-file `.mv2` store (`memvid-core`), built
//!   only when the `memvid` feature is on.
//!
//! Scores returned by a backend are **retrieval relevance only**. They are not
//! scholarly gradings, confidence in a claim, or any religious judgment; see
//! `docs/00-overview/project-overview.md` for the project's stance on those.
//! Nothing in this crate synthesizes or infers a source's meaning.

use serde::{Deserialize, Serialize};

pub mod document;
pub mod error;
pub mod hit;

pub use document::Document;
pub use error::{Result, RetrievalError};
pub use hit::SearchHit;

#[cfg(feature = "memvid")]
pub mod memvid_backend;

#[cfg(feature = "memvid")]
pub use memvid_backend::MemvidRetrieval;

/// A backend that ingests documents and answers queries over them.
///
/// Implementations own their storage and indexing; the port only fixes the
/// shape of a document going in and a hit coming out, so backends stay
/// swappable without touching callers.
pub trait RetrievalBackend: Send {
    /// Stable backend identifier, recorded with results for reproducibility.
    fn name(&self) -> &'static str;

    /// Ingest a document, returning the backend's write handle for it.
    ///
    /// The handle is opaque and is **not** guaranteed to equal
    /// [`SearchHit::document_id`] — some backends assign identifiers at commit
    /// time. Callers must not use it as a durable key; carry caller identity in
    /// [`Document::id`] instead, which backends are expected to keep searchable.
    fn ingest(&mut self, document: &Document) -> Result<String>;

    /// Query the backend, returning at most `limit` hits ordered by descending
    /// relevance. An empty result is a legitimate answer — it must not be
    /// reported as an error.
    fn search(&mut self, query: &str, limit: usize) -> Result<Vec<SearchHit>>;

    /// Flush pending writes so a later `search` on another handle sees them.
    fn commit(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Which backend an operator wants, resolved from configuration.
///
/// Kept as a string rather than an enum so adding a backend does not need a
/// migration; unknown values fail closed at construction, never silently
/// falling back to a different engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BackendKind {
    /// Single-file `.mv2` store.
    Memvid,
}

impl BackendKind {
    /// Parse the configured backend name.
    pub fn parse(name: &str) -> Result<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "memvid" => Ok(Self::Memvid),
            other => Err(RetrievalError::UnknownBackend(other.to_string())),
        }
    }

    /// The stable name recorded with results.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Memvid => "memvid",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_kind_parses_case_insensitively() {
        assert_eq!(BackendKind::parse("memvid").unwrap(), BackendKind::Memvid);
        assert_eq!(BackendKind::parse("  MemVid ").unwrap(), BackendKind::Memvid);
    }

    #[test]
    fn unknown_backend_fails_closed() {
        let err = BackendKind::parse("pinecone").unwrap_err();
        assert!(matches!(err, RetrievalError::UnknownBackend(_)));
        // No silent fallback to another engine.
        assert!(BackendKind::parse("").is_err());
    }
}
