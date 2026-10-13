---
phase: 02-canonical-quran-core
reviewed: 2026-10-13T00:00:00Z
depth: standard
files_reviewed: 56
files_reviewed_list:
  - migrations/sqlite/0020_quran_edition_identity.up.sql
  - migrations/sqlite/0021_canonical_write_fence.up.sql
  - migrations/sqlite/checksums.json
  - docs/schemas/quran-edition-source.v1.schema.json
  - docs/07-technical/quran-canonical-core-decisions.md
  - docs/06-progress/phase-02-evidence.md
  - docs/05-followups/phase-02-owner-gates.md
  - docs/06-progress/corpus-integrity-report.json
  - crates/quran-core/src/edition.rs
  - crates/quran-core/src/enums.rs
  - crates/quran-core/src/reference/serializer.rs
  - crates/quran-corpus/src/format.rs
  - crates/quran-corpus/src/import.rs
  - crates/quran-corpus/src/hashing.rs
  - crates/quran-corpus/src/lib.rs
  - crates/quran-corpus/tests/fixtures.rs
  - crates/quran-corpus/tests/import_path.rs
  - crates/storage/src/quran.rs
  - crates/storage-sqlite/src/quran.rs
  - crates/storage-sqlite/tests/quran.rs
  - crates/provenance/src/lib.rs
  - crates/citations/src/lib.rs
  - crates/application/src/quran.rs
  - crates/application/src/quran_reader.rs
  - crates/application/src/quran_cli.rs
  - crates/application/src/quran_doctor.rs
  - crates/application/src/quran_tools.rs
  - crates/application/tests/quran_identity.rs
  - crates/application/tests/quran_reference.rs
  - crates/application/tests/quran_doctor.rs
  - crates/application/tests/quran_import.rs
  - crates/application/tests/quran_reader.rs
  - crates/application/tests/quran_tools.rs
  - crates/application/tests/quran_translation.rs
  - crates/application/tests/answer_path_ledger.rs
  - crates/cli/src/quran.rs
  - crates/cli/tests/quran.rs
  - crates/cli/tests/corpus_integrity.rs
  - crates/cli/tests/quran/verify_s1.trycmd
  - crates/cli/tests/quran/verify_s2.trycmd
  - crates/cli/tests/quran/reference_s1.trycmd
  - crates/cli/tests/quran/edition_identity_s1.trycmd
  - crates/cli/tests/quran/edition_identity_s2.trycmd
  - crates/cli/tests/quran/read_flow_s1.trycmd
  - crates/cli/tests/quran/read_flow_s2.trycmd
  - crates/cli/tests/quran/read_flow_s3.trycmd
  - crates/cli/tests/quran/read_flow_s4.trycmd
  - crates/cli/tests/quran/verify_quotation_s1.trycmd
  - crates/cli/tests/quran/verify_quotation_s2.trycmd
  - crates/server/src/api.rs
  - crates/server/tests/api.rs
  - fixtures/quran/test-edition-rich/manifest.json
  - fixtures/quran/test-edition-rich/ayahs.csv
  - fixtures/quran/test-edition-rich/reference.json
  - fixtures/quran/test-edition-identity/manifest.json
  - fixtures/quran/golden/ayah_texts.jsonl
  - xtask/src/coverage.rs
findings:
  critical: 0
  warning: 3
  info: 5
  total: 8
status: issues_found
---

# Phase 02: Code Review Report (full-phase stale-refresh)

**Reviewed:** 2026-10-13
**Depth:** standard
**Files Reviewed:** 56
**Status:** issues_found

## Summary

Full-phase review of all 8 plans (02-01…02-08) at HEAD, superseding the
incremental 02-08 delta review (2026-10-04). Scope is the union of every
`key-files` entry from the 8 SUMMARY.md files, with the 5 renamed trycmd
snapshots resolved to their current `*_sN.trycmd` counterparts
(`verify.trycmd` → `verify_s1/s2`, `reference.trycmd` → `reference_s1…`,
`edition_identity.trycmd` → `edition_identity_s1/s2`,
`read_flow.trycmd` → `read_flow_s1…s4`,
`verify_quotation.trycmd` → `verify_quotation_s1/s2`).

No critical issues. The core safety machinery is sound: append-only
migrations with registered checksums, coded trigger taxonomy (QAI-QUR-0006…0021)
that leaves 0007 byte-identical, a forge-proof `ApprovalToken` mint path with a
textual no-public-constructor guard, a single `require_exact` verdict mapping
with distinct stable codes and fail-closed declared-normalization, a
domain-separated translation hash recipe with a collision test, a state-derived
reference-corpus check that fails closed on a corrupt report, and verbatim
license storage that warns (never invents permission) on Unknown.

Of the 6 prior delta findings, 5 still hold verbatim at HEAD (WR-01, WR-02,
IN-01, IN-02) and 1 is partially fixed and reframed (prior WR-03 → new WR-03
below: the `Some(edition)` branch is now correctly attributed, the default
branch is not, and the tool path additionally drops the stale-index advisory).
3 new info-level findings come from the wider phase scope. None blocks phase
acceptance; all are guard-completeness or defense-in-depth items.

Verification: `cargo test -p quran-corpus -p citations -p provenance` green;
`cargo test -p application --test answer_path_ledger` 6/6 green;
`cargo test -p storage-sqlite --test quran` 11/11 green (activation, rollback,
trigger fence, gloss round-trip); application suites green — quran_identity 4/4,
quran_reference 7/7, quran_doctor 1/1, quran_translation 11/11, quran_tools 13/13;
API marker counts re-verified at HEAD ([3,5,2]).

## Warnings

### WR-01: `LEDGER` omits paths the doc claims are mechanically guarded

**File:** `crates/application/tests/answer_path_ledger.rs:37-58`
**Issue:** `every_ledger_symbol_resolves_under_its_declared_kind` iterates only
`LEDGER`, and `LEDGER` omits two paths the `ReaderCitationSource` doc
(`crates/application/src/quran_tools.rs:531-554`) presents as part of the sweep
"guarded mechanically by `crates/application/tests/answer_path_ledger.rs`":
`citation_handler` (`crates/server/src/api.rs:652`) — the **enforced** HTTP
stored-verdict path — and `resolve_handler` (`crates/server/src/api.rs:628`).
Neither emits via a scanned marker, so the marker test cannot catch them
either; a rename/removal of either is silent. `resolve_handler` emits no
canonical text (only `canonical_reference` + `edition_version`, `api.rs:643-646`),
so its omission is low-risk but contradicts the "complete sweep" wording; the
`citation_handler` omission is the safety-relevant one.
**Fix:** add `(API, "citation_handler", "async fn")` and
`(API, "resolve_handler", "async fn")` to `LEDGER`, or narrow the doc's
"complete sweep…guarded mechanically" wording to the marker-detectable set.
**Disposition (fix pass 2026-10-13):** fixed — both entries added
(`ec9bed6`); `answer_path_ledger` 6/6 green. The `LEDGER` fix was chosen over
doc-narrowing because the enforced path deserves rename protection.

### WR-02: Emission-marker scan is API-only; the build-failure claim is false elsewhere

**File:** `crates/application/tests/answer_path_ledger.rs:112-127`
**Issue:** `emitting_markers_resolve_to_recorded_paths` scans only `api.rs`
(counts re-verified at HEAD: 3/5/2). `.arabic_text()` also occurs 4× in
`crates/application/src/quran_cli.rs:238,292,364,399` and 1× in
`crates/application/src/quran_tools.rs:623` (`fetch_ayah_text`). A new emitting
handler in either file using a known marker will not fail the build,
contradicting the module doc (`answer_path_ledger.rs:6-8`). The documented
ceiling (`:14-23`) covers only a *new emission shape*; it does not disclose
this per-file blind spot.
**Fix:** scan all three ledger files per marker with per-file expected counts,
or narrow the doc claim to HTTP emitting handlers in `api.rs`.
**Disposition (fix pass 2026-10-13):** fixed — full scan implemented
(`6bf191d`): per-file counts API [3,5,2] / CLI [4,0,0] / TOOLS [1,0,0], with
`(TOOLS, "fetch_ayah_text", "async fn")` added so the tool-surface hit
attributes to a recorded path; module doc now states the per-file scope.
`answer_path_ledger` 6/6 green.

### WR-03 (refined, supersedes prior WR-03): tool `quran.search` default path mis-attributes the envelope and drops the stale-index advisory

**File:** `crates/application/src/quran_tools.rs:223-304`
**Issue:** The prior WR-03 (`search_meta(None)` naming active while serving
indexed) is half-fixed: the `Some(spec)` branch now resolves the requested
edition (`search_edition_selector`, `:113-124`, with unit tests `:841-869`),
but the `None` branch still uses `active_meta` (`:129-135`) while
`open_serving` serves the **indexed** edition by default and only *warns* on
divergence (`Warning::stale_index`, `crates/application/src/quran_search.rs:71,268-299`).
On a stale index the tool envelope (`edition_id`/`edition_version`,
`reproducibility`) names the active edition while `canonical_references` come
from the indexed one. Worse, `backend_search` hardcodes
`warnings: Vec::new()` (`:280`) even though `SearchOutput.warnings` exists
(`quran_search.rs:102`) and both sibling surfaces propagate it (HTTP
`crates/server/src/api.rs:971-976`, CLI `crates/application/src/quran_cli.rs:5278-5281`):
the one path whose envelope can lie is the one path that swallows the warning
that would disclose it.
Cross-phase note: `backend_search` is phase-03 (D-13) code living in a
phase-02-scoped file — recorded for cross-reference, not a 02-08 regression.
**Fix:** propagate `output.warnings` into the `ToolResult` (mirroring the HTTP
mapping), and resolve the `None` case from the served index manifest (or fail
closed when indexed ≠ active instead of serving with an active-named envelope).
**Disposition (fix pass 2026-10-13):** fixed (`3836090`). Unpinned requests
(absent `edition` or bare slug) now resolve the envelope from the served
edition via the first hit's pinned reference — the HTTP `meta_from_search`
precedent; pinned `slug@version` requests keep the early `search_meta`
attribution (already fail-closed in `open_serving`). `output.warnings` are
propagated with the same `code: message` mapping. New regression test
`search_tool_names_served_edition_and_warns_on_stale_index` (verified to fail
without the fix); `quran_tools` 12/12 green. Fail-closed-on-divergence was
rejected: serving stale with a disclosed warning is the established
`open_serving` contract (drift is a warning, never an auto-repair).

## Info

### IN-01: `tool_search_path_is_present` is effectively vacuous

**File:** `crates/application/tests/answer_path_ledger.rs:135-138`
**Issue:** Asserts `source_for(TOOLS).contains("\"quran.search\"")`; that literal
also appears in the error message at `quran_tools.rs:116` and the unit test at
`:867`, so deleting `backend_search` still passes. Given G-02-3's root cause
("tool `quran.search` was never dispositioned"), a guard that cannot detect its
removal is false assurance.
**Fix:** assert `contains("async fn backend_search")` or add
`(TOOLS, "backend_search", "async fn")` to `LEDGER`.

### IN-02: `scan_marker` attribution heuristic is coupled to a brittle split

**File:** `crates/application/tests/answer_path_ledger.rs:81-95`
**Issue:** Splitting on `"async fn "` mis-attributes markers in non-`async`
helpers, comments/doc-comments, `const`s, or before the first `async fn`; the
hard-coded `[3,5,2]` counts are coupled to the heuristic, so a behavior-neutral
refactor can break or silently re-attribute the guard.
**Fix:** anchor attribution on the enclosing `fn`/brace scope (or a symbol
table), or document the heuristic's assumptions alongside the ceiling.

### IN-03: `Citation` carries edition identity twice with no cross-check

**File:** `crates/citations/src/lib.rs:33-50,269-320`
**Issue:** `Citation` holds both `canonical_reference` (a string embedding the
edition) and separate `edition_slug`/`edition_version` fields. `resolve()`
parses the reference for routing but fetches exclusively via the fields, never
checking the parsed edition agrees with them. A caller constructing a `Citation`
with a mismatched reference string vs fields gets verification against the
fields while the stored reference claims otherwise. All in-tree constructors
build both together (`verify_canonical_quotation`,
`quran_tools.rs:689-702`), so this is latent, not live.
**Fix:** cross-check parsed-edition vs fields in `resolve()`, or expose a
constructor that derives one from the other.

### IN-04: `ApprovalGate::abort` is unconditional and session ids collide by construction

**File:** `crates/provenance/src/lib.rs:292-321`
**Issue:** `abort_canonical_change` sets `Aborted` regardless of current state —
a `Committed` session regresses to `Aborted` — and the session `id` is the
`subject_urn`, so two sessions for one subject share an id. Harmless today:
sessions are in-memory and consumed linearly
(`crates/application/src/quran.rs:318-340`), with double-commit already
rejected (`InvalidSessionState`).
**Fix:** guard `abort` on `Open`, and use unique session ids, if sessions ever
persist or overlap.

### IN-05: Read-only paths hold a write transaction for full scans

**File:** `crates/application/src/quran_doctor.rs:99`,
`crates/application/src/quran_tools.rs:60-71`
**Issue:** `run_quran_checks` and `meta_from_edition` open `db.write()` for pure
reads and `rollback()` at the end. Correct (read-only by construction, rolled
back) but holds a write lock for the whole doctor scan, serializing concurrent
writers for its duration.
**Fix:** use a read transaction if the storage layer offers one; otherwise
document why the write txn is required.

---

_Reviewed: 2026-10-13_
_Reviewer: the agent (gsd-code-reviewer refresh)_
_Depth: standard_
_Scope: SUMMARY.md union (8 plans) + renamed trycmd counterparts; prior delta findings re-verified at HEAD_
