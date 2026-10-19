---
phase: 06-hadith-tafsir
plan: 01
subsystem: database
tags: [hadith, sqlite, fts5, normalization, tool-registry, search]

# Dependency graph
requires:
  - phase: 02-quran-corpus
    provides: source-aware storage pattern (collection/edition/source numbering verbatim)
  - phase: 03-quran-search
    provides: quran-normalization pipeline reused for hadith search forms
  - phase: 05-typed-tools
    provides: typed-tool contract (ToolResult, research_checksum, ReproducibilityData)
provides:
  - Hadith domain types (HadithRecord, HadithCollection, GradingRecord) in hadith-core
  - Migrations 0023–0026 (hadith_collections, hadith_records, hadith_gradings, hadith_search_forms + FTS5)
  - Hadith reader/search/tool services in application crate with research_checksum envelopes
  - HadithBackend trait + hadith.get/search/gradings tools in tool-registry (15 tools total)
affects: [06-02 (tafsir tracer), 06-03 (verse linking), 06-04 (cross-surface parity + ingestion)]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
# Same estimateTokens scale (chars/4 over the realized diff), never a harness token count.
actuals:
  tokens: 30374
  tasks: 3
  commits: 2

# Tech tracking
tech-stack:
  added: []
  patterns: [derived-search-forms (normalized text in separate table, canonical immutable), per-scholar grading records with no consensus field, L5-normalize-both-sides for FTS5 Arabic search]

key-files:
  created: [crates/hadith-core/src/record.rs, crates/hadith-core/src/collection.rs, crates/hadith-core/src/grading.rs, crates/application/src/hadith_reader.rs, crates/application/src/hadith_search.rs, crates/application/src/hadith_tools.rs, migrations/sqlite/0023_hadith_collections.up.sql, migrations/sqlite/0024_hadith_records.up.sql, migrations/sqlite/0025_hadith_gradings.up.sql, migrations/sqlite/0026_hadith_search_forms.up.sql]
  modified: [crates/tool-registry/src/lib.rs, crates/application/src/lib.rs, crates/application/src/quran_tools.rs, crates/domain/src/ids.rs, crates/storage-sqlite/src/lib.rs, xtask/allowlist.toml]

key-decisions:
  - "Both FTS index and query normalize with L5 — SQLite remove_diacritics folds only Latin marks, not Arabic combining marks"
  - "Hadith cross-surface parity deferred to plan 06-04; parity fixture asserts exactly the 3 uncovered hadith.* tools"
  - "typed_id! macro exported from domain so hadith-core can define HadithId/HadithCollectionId/GradingId"

patterns-established:
  - "Derived search forms: normalized text lives in hadith_search_forms, hadith_records.matn_text is canonical-only and never written by search"
  - "Per-scholar grading records: no consensus field anywhere; attribution always shown (D-04)"
  - "Unconfigured hadith tools return typed QAI-HAD-0310 unsupported errors, never raw backend strings"

requirements-completed: [REQ-hadith-corpus]

# Coverage metadata (#1602) — one entry per shipped deliverable. Drives DETERMINISTIC UAT routing in verify-work.
coverage:
  - id: D1
    description: "Migrations 0023–0026 and hadith-core domain types (HadithRecord, HadithCollection, GradingRecord) with serde round-trips"
    requirement: "REQ-hadith-corpus"
    verification:
      - kind: unit
        ref: "cargo test -p hadith-core (12 tests)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Hadith repositories (insert/retrieve collections, records, per-scholar gradings) against test SQLite"
    requirement: "REQ-hadith-corpus"
    verification:
      - kind: unit
        ref: "cargo test -p application hadith (hadith_reader::tests, 4 tests)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Hadith search service reusing quran-normalization; exact + normalized queries match without mutating canonical matn"
    requirement: "REQ-hadith-corpus"
    verification:
      - kind: unit
        ref: "cargo test -p application hadith (hadith_search::tests, 2 tests)"
        status: pass
    human_judgment: false
  - id: D4
    description: "HadithBackend trait + hadith.get/search/gradings tools returning ToolResult with research_checksum and scholar attribution"
    requirement: "REQ-hadith-corpus"
    verification:
      - kind: unit
        ref: "cargo test -p application hadith (hadith_tools::tests, 4 tests)"
        status: pass
      - kind: unit
        ref: "cargo test -p tool-registry (16 tests incl. conformance)"
        status: pass
    human_judgment: false
  - id: D5
    description: "Service-dispatcher wiring for hadith.* tools; full CLI/HTTP parity explicitly deferred to plan 06-04"
    requirement: "REQ-hadith-corpus"
    verification:
      - kind: unit
        ref: "crates/cli/tests/parity_cross_surface.rs#fixtures_cover_all_tool_names (asserts exactly hadith.get/hadith.gradings/hadith.search uncovered)"
        status: pass
    human_judgment: true
    rationale: "Parity deferral is a scoping decision, not a testable behavior — a human must confirm 06-04 picks up the three uncovered tools."

# Metrics
duration: unknown (executor session interrupted; closed out via safe-resume)
completed: 2026-10-13
status: complete
---

# Phase 06 Plan 01: Hadith corpus tracer Summary

**End-to-end hadith tracer: domain types, 4 migrations, normalization-backed search, and 3 typed tools with research_checksum envelopes**

## Performance

- **Duration:** unknown (implementation by interrupted executor session; verification + close-out ~20 min)
- **Started:** 2026-10-12 (plan written)
- **Completed:** 2026-10-13
- **Tasks:** 3
- **Files modified:** 26

## Accomplishments

- Hadith domain types in hadith-core (HadithRecord, HadithCollection, GradingRecord; Tradition/RecordStatus/ReviewDecision/ProvenanceLayer enums) with serde round-trips
- Migrations 0023–0026: hadith_collections, hadith_records (matn/isnad split, verbatim source_number), hadith_gradings (per-scholar, no consensus field), hadith_search_forms + hadith_fts FTS5 index
- Hadith reader/search/tool services: insert/retrieve, L5-normalized exact + diacritic-insensitive search over immutable canonical text, ToolResult envelopes with research_checksum
- Tool registry extended to 15 tools (12 quran.* + hadith.get/search/gradings) with HADITH_*_VERSION 1.0.0 consts; arch-check and migrate-check clean

## Task Commits

All three tasks landed in a single commit (executor session was interrupted before per-task commits or SUMMARY):

1. **Task 1: Hadith migrations + domain types + storage repositories** - `799ac37` (feat, squashed with tasks 2–3)
2. **Task 2: Hadith search service reusing quran-normalization** - `799ac37` (feat, squashed)
3. **Task 3: Hadith typed tool + HadithBackend trait extension** - `799ac37` (feat, squashed)

**Plan metadata:** `TBD` (docs: this SUMMARY, committed separately per safe-resume close-out)

## Files Created/Modified

- `crates/hadith-core/src/record.rs` - HadithRecord domain type (matn/isnad, verbatim source_number)
- `crates/hadith-core/src/collection.rs` - HadithCollection type (slug/version, tradition metadata)
- `crates/hadith-core/src/grading.rs` - GradingRecord type (per-scholar, source_citation required)
- `crates/hadith-core/src/lib.rs` - Replaced placeholder with real module tree
- `migrations/sqlite/0023_hadith_collections.up.sql` - hadith_collections table
- `migrations/sqlite/0024_hadith_records.up.sql` - hadith_records table + (collection_id, source_number) index
- `migrations/sqlite/0025_hadith_gradings.up.sql` - hadith_gradings table + review CHECK + (hadith_id, decision) index
- `migrations/sqlite/0026_hadith_search_forms.up.sql` - hadith_search_forms + hadith_fts FTS5 table
- `crates/application/src/hadith_reader.rs` - HadithRepository trait + SqliteDatabase impl + HadithReaderService
- `crates/application/src/hadith_search.rs` - HadithSearchService over NormalizationPipeline
- `crates/application/src/hadith_tools.rs` - HadithBackend impl returning ToolResult envelopes
- `crates/tool-registry/src/lib.rs` - HadithBackend trait, HadithRef/HadithView/HadithSearchToolParams, 3 new tool registrations
- `crates/application/src/quran_tools.rs` - dispatch_registered_tool arms for hadith.get/search/gradings
- `crates/application/src/lib.rs` - Exposes hadith_reader/hadith_search/hadith_tools modules
- `crates/domain/src/ids.rs` - typed_id! macro exported (#[macro_export]) for cross-crate use
- `crates/storage-sqlite/src/lib.rs` - write_pool() accessor for ingest-style writes
- `xtask/allowlist.toml` - hadith-core allowlisted as domain-only (pure types, no storage/sqlx)
- Test/support updates: `tool-registry/tests/conformance.rs` (12→15 tools), `application/tests/quran_tools.rs`, `application/tests/graph_tools.rs`, `cli/tests/parity_cross_surface.rs` (hadith deferral assertion), 3 trycmd snapshots (schema version 22→26)

## Decisions Made

- L5-normalize-both-sides: both the FTS index content and the query normalize with L5 (diacritics + hamza/alif + Arabic code points) because SQLite's `remove_diacritics` folds only Latin marks and silently breaks Arabic normalized search (recorded in commit message).
- Cross-surface parity for hadith.* deferred to plan 06-04; the parity test was relaxed to assert exactly the 3 uncovered hadith tools rather than full equality.
- `typed_id!` exported from domain instead of duplicating an ID macro in hadith-core.
- `write_pool()` accessor added to SqliteDatabase for ingest-style writes outside UnitOfWork.

## Deviations from Plan

None in scope — all planned artifacts shipped. Structural deviations (all within the tracer's intent):

### Auto-fixed Issues

**1. Single squashed commit instead of per-task atomic commits**
- **Found during:** Safe-resume close-out (executor session interrupted before committing incrementally)
- **Issue:** No per-task commits or SUMMARY.md were written; work existed only as commit 799ac37
- **Fix:** Verified the squashed commit covers all 3 tasks, closed out with SUMMARY-only commit — no re-implementation
- **Files modified:** `.planning/phases/06-hadith-tafsir/06-01-SUMMARY.md`
- **Verification:** Targeted test runs + gates below, all green at close-out time
- **Committed in:** 799ac37 (implementation); SUMMARY in follow-up docs commit

**2. Plan frontmatter `files_modified` omits files the tracer necessarily touched**
- **Found during:** Close-out diff review
- **Issue:** Frontmatter lists only the core files; the diff also touches migration 0026, domain ids.rs, storage-sqlite lib.rs, quran_tools.rs dispatch, allowlist.toml, and 6 test/snapshot files
- **Fix:** None needed — all extra touches are in-scope tracer adaptations, recorded here
- **Files modified:** (as listed in Files Created/Modified)
- **Verification:** arch-check + migrate-check + test suites green
- **Committed in:** 799ac37

---

**Total deviations:** 2 recorded (1 process, 1 plan-metadata gap)
**Impact on plan:** No scope creep; all success criteria met as committed.

## Issues Encountered

- Executor session interrupted after pushing 799ac37 without SUMMARY.md — resolved via safe-resume manual close-out (inspect → verify → record, no re-execution). No code fixes were needed: all verification passed on the committed tree.

## Verification Evidence (re-run at close-out, 2026-10-13, not copied claims)

- `cargo test -p application hadith` — 10 passed, 0 failed (4 reader + 2 search + 4 tools)
- `cargo test -p tool-registry` — 16 passed (6 unit + 10 conformance), 0 failed
- `cargo test -p hadith-core` — 12 passed, 0 failed
- `cargo run -p xtask -- arch-check` — OK, no forbidden edges
- `cargo run -p xtask -- migrate-check` — OK, 26 migrations ordered, checksums stable
- Full `cargo test --workspace` was NOT re-run at close-out (commit message records it green at author time; scoped suites above all pass)

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plans 06-02 (tafsir tracer, depends on 06-01), 06-03 (verse linking, depends on 06-01 + 06-02), and 06-04 (cross-surface parity + ingestion, depends on 06-01..03) are all unblocked by this close-out: the hadith reader/search/tool foundation they build on is committed and green.
- 06-04 must add the three hadith.* fixture rows and restore full-set parity equality in `parity_cross_surface.rs` (currently asserts exactly the 3 uncovered tools).
- Blocker note: `.planning/phases/06-hadith-tafsir/` (including all four PLANs) is currently untracked in git — worth committing or confirming the planning workflow tracks it elsewhere before 06-02 executes.

---
*Phase: 06-hadith-tafsir plan 01 (safe-resume close-out)*
*Completed: 2026-10-13*
