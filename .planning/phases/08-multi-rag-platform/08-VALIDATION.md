# Phase 8 Validation Map

## Requirements

- **REQ-multi-rag**: hybrid retrieval, routing transparency, reproducibility, tombstoning.
- **REQ-generic-rag-platform**: portable RAG project from declarative config, import docs, cross-surface parity.

## Success criteria → tests

| # | Criterion | Test command | Status |
|---|-----------|--------------|--------|
| 1 | Portable RAG project from declarative config + import docs | `cargo test -p application rag_reader` | pass |
| 2 | Hybrid FTS+vector retrieval with reranking + metadata filters | `cargo test -p rag` + `cargo test -p application rag_search` | pass |
| 3 | See which router/tools/sources chosen | route_decision + checksum in ToolResult | pass |
| 4 | Debug any retrieval + reproduce same result set | `cargo test -p cli rag_parity` | pass |
| 5 | Tombstoned/deactivated sources disappear immediately | `cargo test -p application rag_tombstone` | pass |

## Phase gate

- `cargo test --workspace`: pass (0 failures)
- `cargo xtask arch-check`: pass
- `cargo xtask migrate-check`: pass (38 migrations ordered, checksums stable)
