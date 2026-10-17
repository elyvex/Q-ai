---
phase: 08-multi-rag-platform
plan: 05
subsystem: multi-rag
tags: [rag, tombstone, deactivation, propagation, sqlite]
requires:
  - phase: 08-04
    provides: routing transparency + debug surfaces
provides:
  - Transactional set_source_status with embedding/FTS propagation
  - SQL-level active-source filtering in search_fts
  - Idempotent deactivation/tombstoning
  - Audit seq recording on tombstone
affects: [application, rag, cli]
actuals:
  tokens: 3063
  tasks: 3
  commits: 1
tech-stack:
  added: []
  patterns: [single-transaction propagation, sql-level active filter]
key-files:
  created: []
  modified:
    - crates/application/src/rag_reader.rs
    - crates/application/src/rag_tools.rs
    - crates/rag/src/tombstone.rs
key-decisions:
  - "set_source_status now takes optional audit_seq (None auto-increments)"
  - "Tombstoned is terminal; deactivated can be reactivated but embeddings are not restored"
  - "search_fts enforces active status at SQL level via IN-subquery"
requirements-completed:
  - REQ-multi-rag
coverage:
  - id: D1
    description: "Deactivation propagates to all derived index entries"
    requirement: REQ-multi-rag
    verification:
      - kind: unit
        ref: "cargo test -p application --test rag_search"
        status: pass
    human_judgment: false
  - id: D2
    description: "Tombstoned sources disappear from results immediately"
    requirement: REQ-multi-rag
    verification:
      - kind: unit
        ref: "cargo test -p application --test rag_search"
        status: pass
    human_judgment: false
status: complete
duration: 15min
completed: 2026-10-17
plan_head_before: f4ff921339666d97088eceda06cd05c324d3f634
plan_head_after: 5c31d177d463de267a7be83681c893e4912c8ec7
---

# Phase 8 Plan 05: Tombstoning/Deactivation Propagation Summary

**Transactional propagation deletes embeddings and FTS rows on deactivation/tombstone; query filters active sources at SQL level; idempotent and audit-seq recorded.**

## Performance

- **Duration:** ~15 min
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- `set_source_status` runs in a single SQLite transaction: status update + embeddings delete + FTS delete.
- Optional `audit_seq` param: provided value used, else auto-increment max+1.
- Idempotent: repeated deactivation returns Ok without changes.
- Tombstoned is terminal; deactivated can be reactivated (embeddings not restored).
- `search_fts` enforces `rag_sources.status='active'` at SQL level via IN-subquery join.
- Tombstone helpers added to `crates/rag/src/tombstone.rs`.

## Task Commits

1. **Tasks 1-3** - `5c31d17` (feat)

## Files Created/Modified

- `crates/application/src/rag_reader.rs` - transactional propagation, idempotency, audit_seq, SQL-level filter
- `crates/application/src/rag_tools.rs` - pass None audit_seq
- `crates/rag/src/tombstone.rs` - propagation helpers

## Deviations from Plan

None - plan executed as written.

## Issues Encountered

None.

## Next Phase Readiness

Ready for 08-06 cross-surface parity and phase gate.

---
*Phase: 08-multi-rag-platform*
*Completed: 2026-10-17*
