---
phase: 07-isnad-narrator-research
plan: 01
subsystem: isnad-narrator-research
tags: [isnad, narrators, rijal, graph, migrations, tools]
requires: []
provides:
  - narrators table + rijal sources + isnad edges + identity assertions migrations
  - quran-graph Narrator/Hadith NodeKind + NarratorIdentity AssertionKind + TRANSMITS/NARRATES edges
  - isnad-graph port crate domain types
  - SqliteIsnadGraphStore adapter
  - IsnadReader service + IsnadToolBackend
  - IsnadBackend trait + tool names
affects: [application, tool-registry, quran-graph, isnad-graph]
tech-stack:
  added: []
  patterns: [per-scholar-rijal, owner-gate-check, tombstoned-edge-filter, confidence-bands]
key-files:
  created:
    - migrations/sqlite/0031_narrators.up.sql
    - migrations/sqlite/0032_narrator_rijal_sources.up.sql
    - migrations/sqlite/0033_isnad_edges.up.sql
    - migrations/sqlite/0034_narrator_identity_assertions.up.sql
    - crates/isnad-graph/Cargo.toml
    - crates/isnad-graph/src/lib.rs
    - crates/application/src/isnad_graph_store.rs
    - crates/application/src/isnad_reader.rs
    - crates/application/src/isnad_tools.rs
  modified:
    - crates/quran-graph/src/model.rs
    - crates/application/src/lib.rs
    - crates/application/src/quran_tools.rs
    - crates/application/Cargo.toml
    - crates/tool-registry/src/lib.rs
    - xtask/allowlist.toml
decisions:
  - "Confidence bands: high >0.9, medium 0.7-0.9 inclusive, low <0.7"
  - "Computational links start uncertain; owner gate at DB level"
  - "Disputed assertions stay effective; rejected/superseded tombstoned"
metrics:
  duration: 0
  completed: 2026-10-17
  tasks: 3
  commits: 3
  plan_head_before: 9e50325f99e16b93bba4c51f01a1dd7532a2a22a
  plan_head_after: d09a21c1efea19bd4e88c15d98b41ec612be4ffd
  actuals:
    tokens: 90000
    tasks: 3
    commits: 3
status: complete
---

# Phase 7 Plan 01: Isnad & Narrator Research Summary

Tracer slice: migrations 0031-0034, quran-graph additive extension, isnad-graph port crate, application adapter + IsnadReader service + IsnadBackend tools, allowlist/Cargo wiring, dispatch cases, tombstoned-edge filter, confidence boundary, supersede with IdentityProposalParams.

## Deviations from Plan

None - plan executed as written.

## Self-Check: PASSED
