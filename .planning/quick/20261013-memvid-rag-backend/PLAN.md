---
quick: 20261013-memvid-rag-backend
status: planned
created: 2026-10-13
---

# Quick Task: memvid as an alternative RAG retrieval backend

## Intent

Q-ai needs more than one retrieval backend for the Phase 8 Multi-RAG platform.
`memvid-core` (v2.0.140, Apache-2.0) is a single-file `.mv2` memory store with
built-in BM25 lexical search (tantivy), optional vector/hybrid search (hnsw +
ONNX), structure-aware chunking, and a reranker — all behind one Rust crate.

Ship a `RetrievalBackend` port in the existing `retrieval` crate plus one memvid
implementation, so memvid is a swappable retrieval option rather than a
hardcoded dependency.

## Decisions

- **Optional feature, off by default.** memvid-core's default features pull
  tantivy + pdf-extract + SIMD; the `vec` feature adds ONNX runtime + hnsw.
  Follows the existing `observability`/`otlp` precedent (opt-in, off by default)
  so the default build and the dependency allowlist stay untouched.
- **Lexical first.** `search_lex` is the only path exercised here. Vector/hybrid
  needs an embedder and the `vec` feature — that is Phase 8 scope, not this task.
- **No new surface.** No CLI, no HTTP, no typed tool in this task. The port and
  one adapter prove the option exists; surfaces come with the phase that needs them.
- **Determinism and honesty are preserved.** Hits carry the memvid score and the
  frame id; nothing is synthesized. A memvid score is a lexical relevance score,
  not a scholarly grading, and the type name says so.

## Scope

1. `retrieval`: `Document`, `SearchHit`, `RetrievalError`, `RetrievalBackend` trait.
2. `retrieval`: `MemvidRetrieval` behind `#[cfg(feature = "memvid")]`.
3. Tests proving put → commit → search round-trips (feature-gated).
4. `arch-check` allowlist entries if the new edges need them.

## Out of scope

- Vector / hybrid search (needs `vec` feature + embedder)
- Ingestion pipeline, CLI, HTTP, TUI surfaces
- Chunking strategy tuning (memvid has `StructuralChunker`; Phase 8 decides)

## Acceptance

- `cargo check -p retrieval --features memvid` compiles
- Feature-gated test proves: put two docs, commit, search returns the right one
- `cargo test -p retrieval` passes with the feature off (default build untouched)
- `arch-check` green

## Notes

memvid-core API (verified against docs.rs 2.0.140):

```rust
Memvid::create(path) -> Result<Memvid>          // new empty .mv2
Memvid::open(path)   -> Result<Memvid>          // open existing
mem.put_bytes(&[u8]) -> Result<u64>             // frame id
mem.commit()         -> Result<()>
mem.search_lex(q, limit) -> Result<Vec<LexSearchHit>>   // &mut self
// LexSearchHit { frame_id: FrameId, score: f32, match_count: usize, snippets: Vec<String> }
```