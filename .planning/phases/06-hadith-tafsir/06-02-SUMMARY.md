---
phase: 06-hadith-tafsir
plan: 02
subsystem: database
tags: [tafsir, sqlite, fts5, normalization, tool-registry, search]

# Dependency graph
requires:
  - phase: 02-quran-corpus
    provides: source-aware storage pattern (edition/work records with verbatim identity)
  - phase: 03-quran-search
    provides: quran-normalization pipeline (L2/L5) reused for tafsir search forms
  - phase: 05-typed-tools
    provides: typed-tool contract (ToolResult, research_checksum, ReproducibilityData)
  - phase: 06-hadith-tafsir plan 01
    provides: hadith tracer pattern (reader/search/tool services, HadithBackend, FTS5 standalone index, parity-deferral precedent)
provides:
  - Tafsir domain types (TafsirRecord, TafsirWork, TafsirDimensions, WorkStatus) in tafsir crate
  - Migrations 0027–0029 (tafsir_works, tafsir_records, tafsir_search_forms + FTS5)
  - Tafsir reader/search/tool services in application crate with research_checksum envelopes
  - TafsirBackend trait + tafsir.get/tafsir.search tools in tool-registry (17 tools total)
affects: [06-03 (verse linking), 06-04 (cross-surface parity + ingestion)]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
# Same estimateTokens scale (chars/4 over the realized diff), never a harness token count.
actuals:
  tokens: 24812
  tasks: 3
  commits: 3

# Tech tracking
tech-stack:
  added: []
  patterns: [derived-search-forms (normalized text in separate table, canonical immutable), work-author-dimension indexing, structural chapter navigation (surah-level) ahead of computational verse links]

key-files:
  created: [crates/tafsir/src/record.rs, crates/tafsir/src/work.rs, crates/application/src/tafsir_reader.rs, crates/application/src/tafsir_search.rs, crates/application/src/tafsir_tools.rs, migrations/sqlite/0027_tafsir_works.up.sql, migrations/sqlite/0028_tafsir_records.up.sql, migrations/sqlite/0029_tafsir_search_forms.up.sql]
  modified: [crates/tafsir/src/lib.rs, crates/tafsir/Cargo.toml, crates/tool-registry/src/lib.rs, crates/application/src/lib.rs, crates/application/src/quran_tools.rs, crates/application/Cargo.toml, xtask/allowlist.toml]

key-decisions:
  - "Tafsir IDs are manual uuid newtypes (hadith-core precedent) — domain::typed_id! is unusable cross-crate (its crate::primitives path resolves to the caller)"
  - "L5-normalize-both-sides for FTS5 Arabic search, same as the hadith tracer"
  - "Standalone FTS5 index (not content= external-content) — FTS columns do not mirror tafsir_search_forms"
  - "No query-time normalization-profile selector — unsound against the single L5 index; search standardizes on L5 like hadith"
  - "get_records_for_verse is structural (work chapter organization, surah-level); computational verse links arrive in 06-03"
  - "Tafsir cross-surface parity deferred to plan 06-04, extending the 06-01 hadith deferral"

patterns-established:
  - "Derived search forms: normalized text lives in tafsir_search_forms, tafsir_records.text is canonical-only and never written by search"
  - "Shared canonical_reference helper (work slug + chapter/volume/page) in tafsir_reader, reused by search and tools"
  - "Unconfigured tafsir tools return typed QAI-TAF-0310 unsupported errors, never raw backend strings"

requirements-completed: [REQ-tafsir-comparative]

# Coverage metadata (#1602) — one entry per shipped deliverable. Drives DETERMINISTIC UAT routing in verify-work.
coverage:
  - id: D1
    description: "Migrations 0027–0029 and tafsir domain types (TafsirRecord, TafsirWork, TafsirDimensions, WorkStatus) with serde round-trips"
    requirement: "REQ-tafsir-comparative"
    verification:
      - kind: unit
        ref: "cargo test -p tafsir (8 tests)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Tafsir repositories (insert/retrieve works and records, structural verse navigation via chapter) against test SQLite"
    requirement: "REQ-tafsir-comparative"
    verification:
      - kind: unit
        ref: "cargo test -p application --lib tafsir_reader (4 tests)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Tafsir search service reusing quran-normalization; exact + normalized queries match without mutating canonical text"
    requirement: "REQ-tafsir-comparative"
    verification:
      - kind: unit
        ref: "cargo test -p application --lib tafsir_search (2 tests)"
        status: pass
    human_judgment: false
  - id: D4
    description: "TafsirBackend trait + tafsir.get/tafsir.search tools returning ToolResult with research_checksum, deterministic checksums, canonical references"
    requirement: "REQ-tafsir-comparative"
    verification:
      - kind: unit
        ref: "cargo test -p application --lib tafsir_tools (4 tests)"
        status: pass
      - kind: unit
        ref: "cargo test -p tool-registry (6 unit + 10 conformance)"
        status: pass
    human_judgment: false
  - id: D5
    description: "Service-dispatcher wiring for tafsir.* tools; full CLI/HTTP parity explicitly deferred to plan 06-04"
    requirement: "REQ-tafsir-comparative"
    verification:
      - kind: unit
        ref: "crates/cli/tests/parity_cross_surface.rs#fixtures_cover_all_tool_names (asserts exactly the 5 uncovered hadith.* + tafsir.* tools)"
        status: pass
    human_judgment: true
    rationale: "Parity deferral is a scoping decision, not a testable behavior — a human must confirm 06-04 picks up the two uncovered tafsir tools."

# Metrics
duration: 55min
completed: 2026-10-13
status: complete
---

# Phase 06 Plan 02: Tafsir tracer Summary

**End-to-end tafsir tracer: domain types, 3 migrations, normalization-backed search, and 2 typed tools with research_checksum envelopes**

## Performance

- **Duration:** 55 min
- **Started:** 2026-10-13T06:58:42Z
- **Completed:** 2026-10-13T07:53:14Z
- **Tasks:** 3
- **Files modified:** 23

## Accomplishments

- Tafsir domain types in tafsir crate (TafsirRecord, TafsirWork, TafsirDimensions, WorkStatus) with serde round-trips
- Migrations 0027–0029: tafsir_works (author non-empty, dimensions_schema_json, catalog status lifecycle), tafsir_records (volume/page/chapter dimensions, canonical text + hash), tafsir_search_forms + tafsir_fts FTS5 index
- Tafsir reader/search/tool services: insert/retrieve, structural verse navigation via chapter organization, L5-normalized exact + diacritic-insensitive search over immutable canonical text, ToolResult envelopes with research_checksum
- Tool registry extended to 17 tools (12 quran.* + 3 hadith.* + tafsir.get/tafsir.search) with TAFSIR_*_VERSION 1.0.0 consts; arch-check and migrate-check clean; full workspace green

## Task Commits

Each task was committed atomically:

1. **Task 1: Tafsir migrations + domain types + storage repositories** - `f2cecc9` (feat)
2. **Task 2: Tafsir search service reusing quran-normalization** - `6963730` (feat)
3. **Task 3: Tafsir typed tool + TafsirBackend trait extension** - `007ee01` (feat)

**Plan metadata:** this SUMMARY, committed separately per close-out order below.

## Files Created/Modified

- `crates/tafsir/src/work.rs` - TafsirWorkId, TafsirWork (slug/version/author/dimensions_schema), TafsirDimensions, WorkStatus
- `crates/tafsir/src/record.rs` - TafsirId, TafsirRecord (work_id, volume/page/chapter, canonical text + hash)
- `crates/tafsir/src/lib.rs` - Replaced placeholder with real module tree
- `crates/tafsir/Cargo.toml` - domain/serde/serde_json/uuid deps (hadith-core mirror)
- `migrations/sqlite/0027_tafsir_works.up.sql` - tafsir_works table + status index
- `migrations/sqlite/0028_tafsir_records.up.sql` - tafsir_records table + (work_id, volume, page) index
- `migrations/sqlite/0029_tafsir_search_forms.up.sql` - tafsir_search_forms + tafsir_fts FTS5 table
- `crates/application/src/tafsir_reader.rs` - TafsirReader trait + TafsirReaderService + canonical_reference helper
- `crates/application/src/tafsir_search.rs` - TafsirSearchService over NormalizationPipeline
- `crates/application/src/tafsir_tools.rs` - TafsirBackend impl returning ToolResult envelopes
- `crates/tool-registry/src/lib.rs` - TafsirBackend trait, TafsirRef/TafsirView/TafsirSearchToolParams, 2 new tool registrations + registry methods
- `crates/application/src/quran_tools.rs` - dispatch_registered_tool arms for tafsir.get/tafsir.search
- `crates/application/src/lib.rs` - Exposes tafsir_reader/tafsir_search/tafsir_tools modules
- `crates/application/Cargo.toml` - tafsir workspace dep
- `xtask/allowlist.toml` - tafsir domain-only section; application admits tafsir edge
- Test/support updates: `tool-registry/tests/conformance.rs` (15→17 tools, tafsir unsupported arms), `application/tests/graph_tools.rs` + `quran_tools.rs` (17 names), `cli/tests/parity_cross_surface.rs` (tafsir deferral assertion), 3 trycmd snapshots (schema version 26→29)

## Decisions Made

- Manual uuid ID newtypes instead of the plan-specified `domain::typed_id!` macro: the macro's `FromStr` error path names `crate::primitives`, which resolves to the *calling* crate under `macro_rules!` hygiene and fails to compile cross-crate (no non-domain crate uses it). Follows the wave-1 hadith-core precedent exactly.
- L5-normalize-both-sides for FTS index and query (diacritics + hamza/alif + Arabic code points), same as the hadith tracer — SQLite `remove_diacritics` folds only Latin marks.
- Standalone FTS5 index rather than the plan's `content='tafsir_search_forms'`: the FTS columns do not mirror the forms table, so external-content would be invalid (same finding as wave-1 migration 0026).
- No query-time normalization-profile selector on the search params: against a single L5-normalized index, per-query profiles are unsound. Search standardizes on L5 exactly like the hadith tracer.
- `get_records_for_verse` is structural navigation via the works' chapter organization (surah-level; ayah refines nothing yet). Ayah-scoped computational links with confidence + owner gate arrive in plan 06-03.
- Cross-surface parity for tafsir.* deferred to plan 06-04, extending the 06-01 hadith deferral; the parity test asserts exactly the 5 uncovered tools.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] domain::typed_id! macro unusable cross-crate — used hadith-core manual ID pattern**
- **Found during:** Task 1 (tafsir domain types)
- **Issue:** Plan specifies `domain::typed_id!` for TafsirId/TafsirWorkId, but the macro's `FromStr` impl names `crate::primitives::UuidParseError`, which resolves to the calling crate and fails to compile outside `domain` (verified: zero non-domain callers exist)
- **Fix:** Manual uuid-backed newtypes with identical trait set (`Display`/`FromStr` with `String` errors, transparent serde), mirroring hadith-core record/collection/grading IDs
- **Files modified:** crates/tafsir/src/work.rs, crates/tafsir/src/record.rs
- **Verification:** cargo test -p tafsir (8 passed)
- **Committed in:** f2cecc9

**2. [Rule 3 - Blocking] Plan-specified FTS `content=` backing and `NormalizationProfile` type do not exist — followed wave-1 reality**
- **Found during:** Tasks 1–2 (migration 0029, search service)
- **Issue:** Plan asks for FTS5 `content='tafsir_search_forms'` (invalid: FTS columns do not mirror the forms table) and a `search(query, profile: NormalizationProfile)` signature (no such type exists; only ProfileRegistry/ProfileId)
- **Fix:** Standalone FTS5 table per migration-0026 precedent; `TafsirSearchParams { query, work_slug, limit }` struct using ProfileRegistry + L2/L5, mirroring HadithSearchService
- **Files modified:** migrations/sqlite/0029_tafsir_search_forms.up.sql, crates/application/src/tafsir_search.rs
- **Verification:** cargo test -p application --lib tafsir_search (2 passed); canonical text asserted byte-identical after search
- **Committed in:** 6963730

**3. [Rule 2 - Missing Critical] Reader listing + dispatcher arms needed for end-to-end tool reachability**
- **Found during:** Task 3 (tool backend needs work-scoped record resolution; shared dispatcher needs tafsir arms)
- **Issue:** Plan's 5-method TafsirReader has no way to resolve a record by dimensions, and lists only tool-registry/application-lib wiring — without dispatcher arms the tools are unreachable from the shared service path the hadith tools use
- **Fix:** Added `list_records_for_work` to trait + impl, `get_work_by_slug` (catalog pattern), shared `canonical_reference` helper, and `tafsir.get`/`tafsir.search` arms in `dispatch_registered_tool`
- **Files modified:** crates/application/src/tafsir_reader.rs, crates/application/src/quran_tools.rs
- **Verification:** tafsir_tools tests (4 passed); full workspace green
- **Committed in:** 007ee01

**4. [Rule 1 - Bug] Did not copy hadith_search's work-filter SQL construction (latent invalid SQL)**
- **Found during:** Task 2 (work_slug filter)
- **Issue:** Hadith precedent appends `" AND c.slug = ?"` *after* `"LIMIT ?"`, producing `LIMIT ? AND ...` — invalid SQL on the filtered path (latent: no hadith test filters by collection)
- **Fix:** Tafsir filter appends `AND w.slug = ?` before `LIMIT ?`; left the hadith file untouched (out of scope)
- **Files modified:** crates/application/src/tafsir_search.rs
- **Verification:** (filter path compiles; unfiltered search tests pass — a filtered-search test is 06-04 surface-work territory)
- **Committed in:** 6963730

**5. [Rule 1 - Bug] trycmd snapshots pinned schema version 26 — updated to 29**
- **Found during:** Task 3 verification (`cargo test --workspace`)
- **Issue:** 3 CLI snapshot files assert `schema version 26`; three new tafsir migrations make it 29 (same fallout class as 06-01's 22→26)
- **Fix:** `TRYCMD=overwrite` on the 3 failing tests; diff verified to touch only the version line in each file
- **Files modified:** crates/cli/tests/quran/edition_identity_s1.trycmd, reference_s1.trycmd, verify_s1.trycmd
- **Verification:** cargo test --workspace fully green after update
- **Committed in:** 007ee01

---

**Total deviations:** 5 auto-fixed (2 blocking, 1 missing-critical, 2 bugs)
**Impact on plan:** All auto-fixes necessary for correctness/compilation; no scope creep. All success criteria met as committed.

## Issues Encountered

- One self-inflicted bad Edit joined two lines in tafsir_reader.rs mid-Task-2; detected via immediate `git diff` review and restored before proceeding. No impact on committed code.

## Verification Evidence (run on the committed tree, 2026-10-13)

- `cargo test -p tafsir` — 8 passed, 0 failed
- `cargo test -p application --lib tafsir` — 10 passed (4 reader + 2 search + 4 tools), 0 failed
- `cargo test -p tool-registry` — 16 passed (6 unit + 10 conformance), 0 failed
- `cargo test --workspace` — fully green (no FAILED lines across all suites)
- `cargo run -p xtask -- arch-check` — OK, no forbidden edges
- `cargo run -p xtask -- migrate-check` — OK, 29 migrations ordered, checksums stable
- `cargo fmt --check` — clean

## Threat Flags

None — all new surface was in the plan's threat model: canonical text immutable with derived forms in a separate table (T-06-05/T-06-06 mitigated), every tool result carries research_checksum + ReproducibilityData with fully qualified references (T-06-07 mitigated). No new network endpoints, auth paths, or trust-boundary crossings.

## Known Stubs

None — `get_records_for_verse` is structural navigation (work chapter organization), not a stub: it is implemented, tested, and its ayah-scoped successor (verse_links) is explicitly plan 06-03's scope.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plans 06-03 (verse linking, migration 0030 + verse_link.rs, touches tafsir_tools.rs) and 06-04 (cross-surface parity + ingestion, owns the 5 deferred fixture rows) are unblocked: the tafsir reader/search/tool foundation they build on is committed and green.
- 06-04 must add the tafsir.get/tafsir.search (+ 3 hadith.*) fixture rows and restore full-set parity equality in `parity_cross_surface.rs` (currently asserts exactly the 5 uncovered tools).
- Note for 06-03: `TafsirRef::parse` takes `work_slug[:chapter]`; volume/page resolve programmatically. If 06-03's verse-anchored references need a richer string form, extend `TafsirRef` there.
- Known pre-existing issue (out of scope, untouched): `hadith_search.rs` appends its collection filter after `LIMIT ?` — invalid SQL if a caller ever filters by collection. Flagged for 06-04 or a hadith follow-up, not fixed here.

## Self-Check: PASSED

- All key-files.created exist on disk (verified via Write confirmations + test compilation referencing them)
- All task commits are ancestors of HEAD (f2cecc9, 6963730, 007ee01)
- All plan `<verify>` commands re-run green on the committed tree (see Verification Evidence)

---
*Phase: 06-hadith-tafsir plan 02*
*Completed: 2026-10-13*
