//! memvid-backed retrieval: a single `.mv2` file as the store.
//!
//! # retrieval::memvid_backend
//!
//! `memvid-core` keeps documents, its BM25 index, and crash-recovery state in
//! one portable file — no server, no container. That fits Q-ai's local-first
//! posture, so it is offered as a retrieval option rather than a dependency:
//! build with `--features memvid` to include it.
//!
//! Scope is deliberately lexical. memvid's vector and hybrid paths need an
//! embedder and the `vec` cargo feature (ONNX runtime + hnsw); that belongs to
//! the Multi-RAG phase, not to proving the port works.

use std::path::{Path, PathBuf};

use memvid_core::{Memvid, SearchRequest, types::AclEnforcementMode};

use crate::error::{Result, RetrievalError};
use crate::hit::SearchHit;
use crate::{Document, RetrievalBackend};

/// Stable backend name, recorded on every hit for reproducibility.
const BACKEND_NAME: &str = "memvid";

/// Characters of context captured around each match.
const SNIPPET_CHARS: usize = 200;

/// A retrieval backend over a single memvid `.mv2` file.
///
/// The file is held exclusively for the lifetime of the handle, so one handle
/// per store: memvid takes an OS lock, and a second handle would block or fail
/// rather than interleave.
pub struct MemvidRetrieval {
    store: PathBuf,
    mem: Memvid,
}

impl MemvidRetrieval {
    /// Open `path` if it exists, otherwise create a new store there.
    pub fn open_or_create(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut mem = if path.exists() { Memvid::open(&path) } else { Memvid::create(&path) }
            .map_err(|err| RetrievalError::Backend(err.to_string()))?;
        // The BM25 index is opt-in per store; without this every search fails
        // with "Lexical index is not enabled".
        mem.enable_lex().map_err(|err| RetrievalError::Backend(err.to_string()))?;
        Ok(Self { store: path, mem })
    }

    /// The `.mv2` file backing this store.
    pub fn path(&self) -> &Path {
        &self.store
    }
}

impl RetrievalBackend for MemvidRetrieval {
    fn name(&self) -> &'static str {
        BACKEND_NAME
    }

    fn ingest(&mut self, document: &Document) -> Result<String> {
        // memvid mints the frame id and `put_bytes` accepts no caller id, so
        // the caller's id and title are prefixed into the indexed text. That
        // keeps caller identity searchable and visible in the hit text — the
        // durable key, since the handle returned here is memvid's write
        // sequence and only coincides with a frame id once committed.
        let mut payload = String::new();
        if let Some(id) = &document.id {
            payload.push_str(id);
            payload.push('\n');
        }
        if let Some(title) = &document.title {
            payload.push_str(title);
            payload.push('\n');
        }
        payload.push_str(&document.text);

        let seq = self
            .mem
            .put_bytes(payload.as_bytes())
            .map_err(|err| RetrievalError::Backend(err.to_string()))?;
        Ok(seq.to_string())
    }

    fn search(&mut self, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        if query.trim().is_empty() {
            return Err(RetrievalError::EmptyQuery);
        }
        let request = SearchRequest {
            query: query.to_string(),
            top_k: limit.clamp(1, 1000),
            snippet_chars: SNIPPET_CHARS,
            uri: None,
            scope: None,
            cursor: None,
            as_of_frame: None,
            as_of_ts: None,
            no_sketch: false,
            acl_context: None,
            acl_enforcement_mode: AclEnforcementMode::Audit,
        };
        let response =
            self.mem.search(request).map_err(|err| RetrievalError::Backend(err.to_string()))?;
        Ok(response
            .hits
            .into_iter()
            .map(|hit| {
                // `score` is Option in memvid; an unranked hit reports 0.0
                // rather than inventing a number.
                let score = hit.score.unwrap_or(0.0);
                SearchHit::new(hit.frame_id.to_string(), score, vec![hit.text], BACKEND_NAME)
            })
            .collect())
    }

    fn commit(&mut self) -> Result<()> {
        self.mem.commit().map_err(|err| RetrievalError::Backend(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> (tempfile::TempDir, MemvidRetrieval) {
        let dir = tempfile::tempdir().unwrap();
        let backend = MemvidRetrieval::open_or_create(dir.path().join("knowledge.mv2")).unwrap();
        (dir, backend)
    }

    #[test]
    fn ingest_then_search_finds_the_document() {
        let (_dir, mut backend) = store();
        backend
            .ingest(&Document::new("Sahih al-Bukhari collection of hadith").with_id("bukhari"))
            .unwrap();
        backend
            .ingest(&Document::new("Sunan Abi Dawud collection of hadith").with_id("abudawud"))
            .unwrap();
        backend.commit().unwrap();

        let hits = backend.search("bukhari collection", 5).unwrap();
        assert!(!hits.is_empty(), "expected at least one hit");
        assert_eq!(hits[0].backend, "memvid");
        // Scores are relevance only, but must be ordered descending.
        assert!(hits.windows(2).all(|w| w[0].score >= w[1].score));
    }

    #[test]
    fn caller_id_is_searchable_and_visible_in_the_hit() {
        let (_dir, mut backend) = store();
        backend.ingest(&Document::new("a hadith about prayer").with_id("hadith-42")).unwrap();
        backend.commit().unwrap();

        // The caller's id is prefixed into the indexed text, so it is both
        // findable by query and visible in the returned hit text. That — not
        // the write handle — is the durable identity.
        let hits = backend.search("hadith-42", 5).unwrap();
        assert!(!hits.is_empty(), "caller id should be searchable");
        assert!(
            hits.iter().any(|h| h.snippets.iter().any(|s| s.contains("hadith-42"))),
            "hit text should expose the caller id: {hits:?}"
        );
    }

    #[test]
    fn empty_query_is_typed_error_not_empty_result() {
        let (_dir, mut backend) = store();
        assert!(matches!(backend.search("   ", 5), Err(RetrievalError::EmptyQuery)));
    }

    #[test]
    fn empty_store_returns_no_hits_without_erroring() {
        let (_dir, mut backend) = store();
        backend.commit().unwrap();
        assert!(backend.search("anything", 5).unwrap().is_empty());
    }

    #[test]
    fn reopen_sees_committed_documents() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reopen.mv2");
        {
            let mut backend = MemvidRetrieval::open_or_create(&path).unwrap();
            backend.ingest(&Document::new("persisted hadith text").with_id("p-1")).unwrap();
            backend.commit().unwrap();
        }
        // A dropped handle releases the OS lock; the reopened store must still
        // serve the committed document, with a stable frame id.
        let mut reopened = MemvidRetrieval::open_or_create(&path).unwrap();
        let first = reopened.search("persisted", 5).unwrap();
        assert!(!first.is_empty(), "committed document should survive reopen");
        assert!(
            first.iter().any(|h| h.snippets.iter().any(|s| s.contains("p-1"))),
            "caller id should survive reopen: {first:?}"
        );
        // Same query on the same store must yield the same document id.
        let second = reopened.search("persisted", 5).unwrap();
        assert_eq!(
            first.iter().map(|h| &h.document_id).collect::<Vec<_>>(),
            second.iter().map(|h| &h.document_id).collect::<Vec<_>>()
        );
    }
}
