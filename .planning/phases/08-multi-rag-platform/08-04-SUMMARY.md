---
phase: 08-multi-rag-platform
plan: 04
subsystem: multi-rag
tags: [rag, routing, debug, cli, tui, server]
requires:
  - 08-02
  - 08-03
provides:
  - RouteDecision with counts + checksum
  - RagToolBackend RagBackend impl
  - CLI rag query/debug/source commands
  - TUI RagState::Available render
  - Server rag.* tool dispatch + mutating guard
affects: [rag, application, cli, tui, server, tool-registry]
tech-stack:
  added: []
  patterns: [route-decision, rrf-debug-trace, tombstone-audit]
key-files:
  created: []
  modified:
    - crates/rag/src/lib.rs
    - crates/application/src/rag_search.rs
    - crates/application/src/rag_tools.rs
    - crates/application/src/rag_reader.rs
    - crates/application/src/quran_tools.rs
    - crates/cli/src/rag.rs
    - crates/tui/src/screens/rag_debug.rs
    - crates/server/src/api.rs
    - crates/tool-registry/src/lib.rs
decisions:
  - RouteDecision carries candidate/merged/reranked/excluded counts for debug
  - Tombstone audit_seq assigned per project on tombstone
  - Server denies mutating rag tools by default on HTTP route
metrics:
  duration: 0
  completed: 2026-10-17
  tasks: 3
  commits: 1
status: complete
actuals:
  tokens: 45000
  tasks: 3
  commits: 1
---

# Phase 8 Plan 04: Routing Transparency Summary

Implemented routing transparency and debug surfaces across CLI/TUI/server.

## Deviations from Plan

- Added `with_defaults`/`from_db` constructors to avoid forbidden dependency edges (cli -> embeddings/reranking).
- Extended `RouteDecision` with candidate/merged/reranked/excluded counts to support debug trace.

## Self-Check: PASSED
EOF
echo done