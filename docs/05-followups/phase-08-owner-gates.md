# Phase 8 (Multi-RAG Platform) — Owner Gates

> Phase 8 introduces no new owner gates. ADR-0701 (vector storage) and ADR-0301
> (retrieval topology) remain undecided from prior phases; Phase 8 chose SQLite
> BLOB vectors as the interim portable store. Closing commands below are for the
> owner to ratify if desired.

## Gates (inherited, unchanged)

| ID | Status | Blocks | Owner | Closing action (exact) |
|----|--------|--------|-------|------------------------|
| ADR-0701 | 🔴 BLOCKED (inherited) | Vector storage backend ratification | Owner | Ratify PostgreSQL/Qdrant adapter scope; record in `docs/05-followups/owner-decisions.md`. |
| ADR-0301 | 🔴 BLOCKED (inherited) | Retrieval topology/embeddings/chunking/reranking | Owner | Ratify RAG topology decisions; record in `docs/05-followups/owner-decisions.md`. |

## Phase 8 success criteria — evidence-backed

| # | Criterion | Evidence |
|---|-----------|----------|
| 1 | Portable RAG project from declarative config + import docs | `cargo test -p application rag` + CLI parity |
| 2 | Hybrid FTS+vector retrieval with reranking + metadata filters | `cargo test -p rag` + `rag_search` |
| 3 | See which router/tools/sources chosen | route_decision + checksum in ToolResult |
| 4 | Debug any retrieval + reproduce same result set | `qai rag debug` + reproducibility checksum |
| 5 | Tombstoned/deactivated sources disappear immediately | `cargo test -p application rag_tombstone` |

No owner decision answered by this record.
