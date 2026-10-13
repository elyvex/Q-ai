---
phase: 02-canonical-quran-core
fixed_at: 2026-10-13T04:56:44Z
review_path: /Users/ali/dev/rust/Q-ai/.planning/phases/02-canonical-quran-core/02-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 02: Code Review Fix Report

**Fixed at:** 2026-10-13T04:56:44Z
**Source review:** `.planning/phases/02-canonical-quran-core/02-REVIEW.md` (full-phase stale-refresh, 2026-10-13)
**Iteration:** 1

**Summary:**
- Findings in scope: 3 (WR-01…WR-03; `critical_warning` scope, IN-01…IN-05 excluded)
- Fixed: 3
- Skipped: 0

No carried finding was dismissed as a non-issue: WR-01 and WR-02 were real
guard-completeness gaps with small, test-verified fixes, so they were fixed
rather than re-dispositioned.

## Fixed Issues

### WR-01: `LEDGER` omits paths the doc claims are mechanically guarded

**Files modified:** `crates/application/tests/answer_path_ledger.rs`
**Commit:** `ec9bed6`
**Applied fix:** Added `(API, "citation_handler", "async fn")` and
`(API, "resolve_handler", "async fn")` to `LEDGER` (the reviewer's first
option). The `LEDGER` fix was chosen over doc-narrowing because the enforced
stored-verdict path deserves rename protection. `resolve_handler` emits
metadata only; it is recorded for drift protection, consistent with the
existing `struct` entries.
**Verification:** `cargo test -p application --test answer_path_ledger` 6/6 green.

### WR-02: Emission-marker scan is API-only; the build-failure claim is false elsewhere

**Files modified:** `crates/application/tests/answer_path_ledger.rs`
**Commit:** `6bf191d`
**Applied fix:** Full per-file scan (the reviewer's first option):
`emitting_markers_resolve_to_recorded_paths` now checks every ledger file
with per-file expected counts — API [3,5,2], CLI [4,0,0], TOOLS [1,0,0] —
asserting the HTTP-only markers absent outside `api.rs`. Added
`(TOOLS, "fetch_ayah_text", "async fn")` to `LEDGER` so the tool-surface
`.arabic_text()` occurrence (the verifying read behind `resolve()`)
attributes to a recorded path. The CLI hits all fall inside already-ledgered
symbols (`cmd_get`, `cmd_context`, `cmd_surah`, `cmd_division`), so no
further entries were needed. Module doc now states the per-file scope, and
the ceiling notes counts are per-file so a cross-file move breaks loudly.
**Verification:** `cargo test -p application --test answer_path_ledger` 6/6 green.

### WR-03: tool `quran.search` default path mis-attributes the envelope and drops the stale-index advisory

**Files modified:** `crates/application/src/quran_tools.rs`, `crates/application/tests/quran_tools.rs`
**Commit:** `3836090`
**Applied fix:** Both halves, as suggested. (1) `output.warnings` are
propagated into the `ToolResult` with the same `code: message` mapping the
HTTP `meta_from_search` uses. (2) Unpinned requests (absent `edition`, or a
bare slug — which `open_serving` serves without a version check while
`search_meta` resolves to the active version) now resolve the envelope from
the *served* edition via the first hit's pinned reference (new
`served_edition_of` helper), mirroring the HTTP `meta_from_search`
precedent. Pinned `slug@version` requests keep the early `search_meta`
attribution (already fail-closed via `EditionNotIndexed` in `open_serving`).
With zero hits there is nothing to misattribute, so the requested meta
stands; the propagated warnings still disclose any divergence.
Fail-closed-on-divergence was deliberately rejected: serving stale with a
disclosed warning is the established `open_serving` contract (drift is a
warning, never an auto-repair), and erroring the tool on a state the CLI and
HTTP surfaces serve would diverge the three surfaces.
**Verification:** new `search_tool_names_served_edition_and_warns_on_stale_index`
regression test — verified to FAIL without the `quran_tools.rs` fix and pass
with it; `cargo test -p application --test quran_tools` 12/12 green.
Blast-radius suites (server, tool-registry, cli quran + parity): see below.

## Skipped Issues

None — all in-scope findings were fixed.

## Verification (where it ran)

All gates ran in the main checkout (`/Users/ali/dev/rust/Q-ai`, branch
`agent-06-01-hadith-tracer`; no isolated worktree per orchestrator scope).

- `cargo test -p application --test answer_path_ledger` — 6 passed, 0 failed
- `cargo test -p application --test quran_tools` — 12 passed, 0 failed (incl. new stale-index test)
- `cargo fmt --all -- --check` — clean
- `cargo test -p server -p tool-registry` — all suites green (lib 4 + integration 25/3/10/6/10, incl. `tool_status_mapping`)
- `cargo test -p cli --test quran` — 17 passed, 0 failed (owned suites incl. `verify`, `reference`, `edition_identity`, `verify_quotation`, `read_flow`)
- `cargo test -p cli --test parity_cross_surface` — 5 passed, 0 failed
- `cargo run -p xtask -- arch-check` — OK (no forbidden edges)
- `cargo run -p xtask -- migrate-check` — OK (26 migrations, checksums stable)

## Notes

- `parity_s1.trycmd:99` asserts `"warnings": []` on a `quran.get_ayah`
  envelope — untouched by the WR-03 change (only `backend_search` changed;
  all other backends keep `warnings: Vec::new()`), so no snapshot updates.
- The stale-index test imports v2 with unchanged text (mirroring the
  `doctor_indexes.rs` drift pattern) so the v1-built index still serves hits
  after activation moves the pointer.

---

# Prior fix cycle (2026-09-26) — preserved

The report below covers a *different review generation* (the 2026-10-04
incremental delta review's findings, also numbered WR-01…WR-09 there — the
IDs collide with the current cycle's but denote different findings). Kept
verbatim for the audit trail; its frontmatter was:
`fixed_at: 2026-09-26T00:00:00Z, iteration: 1, findings_in_scope: 9, fixed: 9,
skipped: 0, status: all_fixed`.

---

# Phase 02: Code Review Fix Report (prior cycle)

**Fixed at:** 2026-09-26T00:00:00Z
**Source review:** `.planning/phases/02-canonical-quran-core/02-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 9 (WR-01…WR-09; `critical_warning` scope, IN-01…IN-05 excluded)
- Fixed: 9
- Skipped: 0

## Fixed Issues

### WR-01: New citation hard-failure codes fall through to HTTP 500

**Files modified:** `crates/server/src/api.rs`
**Commit:** 69db45f
**Applied fix:** `tool_status` now maps `QAI-QUR-0324`/`0325` to 404 and `QAI-QUR-0326` to 403, exactly as suggested. (WR-07 later added `0321` → 400 in the same function; see below.)

### WR-02: Canonical hashes are manifest-order-dependent; validator permits unsorted manifests

**Files modified:** `crates/quran-corpus/src/import.rs`
**Commit:** 5cd40bb
**Applied fix:** Added a single `canonical_order()` helper (indices sorted by `(surah, ayah)`; identity for sorted manifests, so all existing hashes/snapshots are unchanged). `hashes_computed` builds `text_hash`/`structure_hash`/`token_order_hash` over that order, `staged` assigns `global_ayah_index`/`global_token_index` in that order, and `build_divisions` now takes the ordered ayah slice so division `start_global`/`end_global` agree with the staged indices. An unsorted-but-valid manifest now imports identically to its sorted permutation instead of failing late at QV-014. Added two `canonical_order` unit tests (identity + permutation). Chose sorting over a new QV rejection rule so previously-accepted inputs keep importing.

### WR-03: Nonexistent edition reported as `NotStaged` in three canonical mutators

**Files modified:** `crates/application/src/quran.rs`, `crates/application/src/quran_cli.rs`
**Commit:** 56ac660
**Applied fix:** New `ActivationError::EditionNotFound { slug, version }` variant (code `QAI-QUR-0314`, remedy "Import and activate the edition first.") used by the three canonical-table lookups (`record_edition_verification`, `rollback_edition`, `deprecate_edition`). Staging-table lookups (`activate_edition`, `validate_staged`, translation/gloss alignment) keep `NotStaged`. CLI maps the new variant to exit 5 (`NOT_FOUND`); re-activation of a no-longer-staged edition still exits 6 with the old message (snapshot `read_flow.trycmd:250` unaffected).

### WR-04: Corrupt validation report degrades to misleading `Skipped`

**Files modified:** `crates/application/src/quran_doctor.rs`
**Commit:** e83c707
**Applied fix:** Unparseable `findings_json` now returns `Fail` ("persisted validation report is corrupt; integrity unknown") instead of falling through to the benign "no reference corpus configured" skip. Pass detection parses each QV-015 Info finding's `message` as JSON and matches the structured `outcome == "pass"` field instead of substring-matching the serialized report.

### WR-05: Unreachable `MatchAfterDeclaredNormalization` defaults to success

**Files modified:** `crates/citations/src/lib.rs`
**Commit:** 5dab096
**Applied fix:** `is_hard_failure` returns `true` and `require_exact` returns `Err(CitationError::Backend { detail: "declared-normalization matches are not supported in v1" })` for the producer-less variant, with doc comments updated to state the fail-closed default. No producer exists, so no live path changes behavior.

### WR-06: Translation import relaxes `attribution_required` to `false`, including for `Unknown` licenses

**Files modified:** `crates/application/src/quran.rs`, `crates/application/tests/quran_verification.rs`
**Commit:** 7a9caf6
**Applied fix:** Translation import only (reviewer-explicit scope; `quran_cli.rs` canonical-license literals predate the phase and are untouched). Undeclared-license branch now persists `attribution_required: true` (conservative default; never `false` for `Unknown`). Declared-SPDX branch omits the field rather than hardcoding `false` — the manifest carries no attribution signal, and the reader defaults a missing flag to `false`, so read behavior is unchanged while the stored bytes stop asserting a permission that was never declared. The `quran_verification.rs` assertion update in this commit belongs to WR-03's blast radius (the ghost-edition case now expects `EditionNotFound`, not `NotStaged`); it is recorded here because it was committed together with WR-06 after WR-03's commit had landed.

### WR-07: Out-of-range surah/ayah mapped to internal error instead of caller error

**Files modified:** `crates/application/src/quran_tools.rs`, `crates/server/src/api.rs`
**Commit:** 81b2376
**Applied fix:** `SurahNumber::new`/`AyahNumber::new` rejections in `fetch_ayah_text` and `verify_canonical_quotation` now map to `CitationError::InvalidReference` (CLI → USAGE/2 via the existing `map_citation_error`). Adaptation: the reviewer's "HTTP → 400 via `InvalidInput`" does not match the actual plumbing — the citations HTTP path wraps typed errors as `ToolError::Backend { code }` — so `tool_status` additionally maps code `QAI-QUR-0321` to 400 (previously 500). Extended the existing `tool_status_mapping` unit test with 0321→400, 0324/0325→404, 0326→403 assertions.

### WR-08: Read-only operator paths open write transactions and swallow storage errors

**Files modified:** `crates/application/src/quran_cli.rs`
**Commit:** 11cef34
**Applied fix:** Both read paths now propagate storage errors instead of swallowing them: `cmd_edition_show` returns `INTERNAL` with the fetch error rather than silently omitting the `license:` line; the `cmd_import` QV-015 branch returns the report-fetch error distinctly instead of masking it behind the import error. Adaptation: the `db.write()` + `rollback()` handles are retained — `ReadTx` exposes only raw-SQL `query_one`/`query_list`, so switching would mean interpolating operator-controlled slug/version strings into SQL inside the application layer (worse posture, plus an arch-layering violation). The retained pattern matches 15+ other read sites in the same module; the handles are always rolled back, never committed.

### WR-09: `rollback_edition` can reactivate a `Quarantined` edition

**Files modified:** `storage-sqlite/src/quran.rs`, `crates/storage-sqlite/tests/quran.rs`
**Commit:** a35e3d6
**Applied fix:** Storage-layer allowlist exactly as suggested: `Active` → `Conflict` (unchanged), anything other than `Deprecated` → `ConstraintViolation` ("rollback target {slug}@{version} is {status}, not Deprecated"). `Quarantined` (and never-served `Approved`) targets are rejected at the single enforcement point covering all callers. Added `rollback_rejects_non_deprecated_targets` regression test (quarantined + approved → `ConstraintViolation`). The existing `rollback_restores_a_prior_version` (Deprecated → Active) and the `read_flow` rollback snapshot are unaffected.

## Skipped Issues

None — all findings were fixed.

## Verification (where it ran, prior cycle)

All gates ran in the main checkout (`/Users/ali/dev/rust/Q-ai`, branch `main`; no isolated worktree per orchestrator scope — prior phase work lives on `main`).

- `cargo build --workspace` — ok
- `cargo test -p quran-corpus` — 75 passed, 0 failed (incl. 2 new `canonical_order` tests)
- `cargo test -p storage-sqlite` — 47 passed, 0 failed (incl. new rollback-allowlist test)
- `cargo test -p application` — all 23 suites ok (incl. updated `quran_verification` ghost-edition assertion)
- `cargo test -p server` — lib 4 + integration 18 ok (incl. extended `tool_status_mapping`)
- `cargo test -p citations` — 7 passed, 0 failed
- `cargo test -p cli --test quran` — owned suites ok: `verify`, `reference`, `edition_identity`, `verify_quotation`, `read_flow`
- `cargo xtask arch-check` — OK; `cargo xtask migrate-check` — OK (21 migrations, checksums stable)
- Full `cargo test --workspace` deliberately NOT run as a gate (two pre-existing `normalize`/`search` trycmd snapshot failures, out of scope and untouched).

## Notes (prior cycle)

- The tree carried pre-existing uncommitted `cargo fmt` reformatting (whitespace-only) in `crates/quran-corpus/src/import.rs`, `crates/provenance/src/lib.rs`, and test files. WR-02 and WR-09 commits absorbed the fmt hunks in their own files; remaining dirty files (`provenance`, `quran-corpus/tests/*`, `.planning/*`) belong to the foreground session and were not touched.
- Rollback safety: `import.rs` was backup-copied before editing (dirty tree ⇒ `git checkout --` would have destroyed foreground work); no rollback was needed — every fix verified clean.
- No trycmd snapshots required updates: the only behavior-changing outputs (WR-03 ghost-edition error, WR-06 translation-license JSON) are not asserted by any snapshot; the one related unit test was updated (see WR-06 note).

---

_Fixed: 2026-10-13T04:56:44Z (current cycle); prior cycle 2026-09-26T00:00:00Z_
_Fixer: the agent (gsd-code-fixer role, fix mode)_
_Iteration: 1_
