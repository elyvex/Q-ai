---
quick: 20261013-memvid-rag-backend
status: complete
created: 2026-10-13
completed: 2026-10-13
---

# Quick Task: memvid as an alternative RAG retrieval backend — Summary

## Result

Shipped a backend-agnostic retrieval port in `crates/retrieval` with one
working memvid implementation. memvid is now a *swappable option*, not a
hardcoded dependency.

## What landed

**Port (`crates/retrieval/src/lib.rs`)** — `RetrievalBackend` trait
(`name`/`ingest`/`search`/`commit`), plus `BackendKind` that parses a
configured backend name and **fails closed** on unknown values (never silently
falls back to a different engine).

- `document.rs` — `Document { id, title, text, metadata }`
- `hit.rs` — `SearchHit { document_id, score, snippets, backend }`
- `error.rs` — typed `RetrievalError` implementing `storage::error::Diagnostic`
  (QAI-RET-1..5), each with a remedy.

**Adapter (`memvid_backend.rs`, feature-gated)** — `MemvidRetrieval` over a
single `.mv2` file: `create`/`open` → `enable_lex` → `put_bytes` → `commit` →
`search`.

## API findings that cost real debugging time

Worth recording — the obvious reading of the docs is wrong in three places:

1. **`search_lex` is deprecated and always fails** with
   `"Lexical index is not enabled"`, even right after `enable_lex()`. The
   working path is `search(SearchRequest) -> SearchResponse`.
2. **`enable_lex()` is mandatory** per store — without it every search errors.
3. **`put_bytes` returns a write *sequence*, not a frame id**; the two coincide
   only for committed frames. The `RetrievalBackend` contract documents this
   explicitly so callers do not treat the handle as a durable key — caller
   identity rides in `Document::id`, which is prefixed into the indexed text so
   it stays searchable and visible in hit text.
4. `SearchHit.score` is `Option<f32>`; an unranked hit reports `0.0` rather
   than inventing a number.

## Decisions

- **Optional `memvid` feature, off by default** — its default features pull
  tantivy + pdf-extract + SIMD. Follows the existing `observability`/`otlp`
  precedent so the default build, CI, and the dependency tree stay unchanged.
- **Lexical only.** Vector/hybrid needs the `vec` feature (ONNX runtime + hnsw)
  and an embedder — that is Phase 8 scope.
- **No CLI/HTTP/typed tool.** The port proves the option exists; surfaces come
  with the phase that needs them.
- **Scores labelled as retrieval relevance only**, never as confidence in a
  claim or a scholarly grading. Documented on `SearchHit::score` and in the
  crate header.

## Verification

| Check | Result |
|---|---|
| `cargo test -p retrieval` (feature off) | 6 passed — default build untouched |
| `cargo test -p retrieval --features memvid` | **11 passed** — real memvid-core round-trip |
| `cargo check --all-targets` | clean, no warnings |
| `xtask arch-check` | OK |
| `xtask migrate-check` | OK (26) |

memvid tests prove: ingest→commit→search finds the right document; caller id is
searchable and survives reopen; hits are score-ordered and backend-stamped;
empty query is a typed error, not an empty result; an empty store returns no
hits without erroring; and document ids are stable across repeated queries.

## Follow-ups (Phase 8, not this task)

- Vector + hybrid search (`vec` feature, embedder wiring)
- Chunking strategy (memvid ships `StructuralChunker`)
- Ingestion pipeline and CLI/HTTP/TUI surfaces
- Note: memvid takes an exclusive OS lock per `.mv2`, so one handle per store —
  matters if this goes concurrent.