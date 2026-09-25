---
phase: 02-canonical-quran-core
plan: 02
subsystem: testing
tags: [quran, fixtures, synthetic, golden, reference-corpus, edition-identity, qv-002, csv-parity]

requires:
  - phase: 01-foundations
    provides: JsonAdapter/CsvAdapter, the QV validator, the text_hash recipe, and the golden ayah_texts suite
  - phase: 02-canonical-quran-core
    provides: plan 02-01's EditionMeta upstream identity / qai_edition_id / is_primary / declared license fields

provides:
  - fixtures/quran/test-edition-rich/{manifest.json,ayahs.csv} — 6 surahs / 24 ayahs numbered 1..=6 exercising a basmala inside an ayah body, U+06D6..U+06ED pause marks, a mark-separated token, a long ayah, page/division boundaries, a muqatta'at standalone token, a repeated refrain, and U+0670
  - fixtures/quran/test-edition-rich/reference.json — synthetic companion reference, byte-identical ayah stream with both reference pins (reference_corpus_id + sha256 reference_text_hash)
  - fixtures/quran/test-edition-identity/manifest.json — synthetic identity/primary/license fixture for the operator path
  - edition-scoped golden loader + validation-clean fixture assertions in crates/quran-corpus/tests/fixtures.rs

affects: [02-03 operator snapshots, 02-07 corpus-integrity artifact]

actuals:
  tokens: 10281    # chars/4 over the realized diff (41126 chars)
  tasks: 2
  commits: 2
  plan_head_before: 0f44e24d5e64f59febe22574a9bea1b4024e4834

tech-stack:
  added: []
  patterns:
    - edition-scoped golden keys (an EDITION field per row) so multiple synthetic editions can all be numbered 1..=N
    - a fixture is proven import-viable by asserting validate_edition yields no Fatal/Error, not merely that it parses

key-files:
  created:
    - fixtures/quran/test-edition-rich/manifest.json
    - fixtures/quran/test-edition-rich/ayahs.csv
    - fixtures/quran/test-edition-rich/reference.json
    - fixtures/quran/test-edition-identity/manifest.json
  modified:
    - fixtures/quran/golden/ayah_texts.jsonl
    - crates/quran-corpus/tests/fixtures.rs

key-decisions:
  - "Renumbered the rich fixture to surahs 1..=6 (and the identity fixture to surah 1) because QV-002 is Fatal and run_import aborts on any Fatal finding; a 101..106/201 fixture could never be imported or activated by plan 02-03 (D-08 kept, QV-002 respected)."
  - "Golden keys are edition-scoped by an EDITION field because two synthetic editions are both numbered 1..=N after the QV-002 fix; a bare (surah, ayah) key collides across editions."
  - "The reference_text_hash pin reuses the existing text_hash() recipe verbatim (no new recipe): sha256:6c9e6916a7a04850805f4205bd0993042d338af1a82ddd37972745f3e409bb3a, independent of surah renumbering."
  - "All fixtures stay synthetic: true and name no real dataset, publisher, license, or signer (OD-01/OD-03 remain open owner gates)."

patterns-established:
  - "Synthetic fixture edition numbering: fixture surahs must be exactly 1..=N so the same validator that guards real imports (QV-002) accepts them."
  - "Edition-scoped golden rows: the golden set carries an EDITION discriminator and the loader filters per edition."

requirements-completed: [REQ-quran-corpus, REQ-ingestion-validation-eval]

coverage:
  - id: D1
    description: "Richer synthetic edition (test-edition-rich, 6 surahs / 24 ayahs) with a CSV mirror that reproduces its JSON ayahs field-for-field and a golden set covering every ayah."
    requirement: "REQ-quran-corpus"
    verification:
      - kind: unit
        ref: "crates/quran-corpus/tests/fixtures.rs#test_edition_rich_parses_with_expected_shape_and_coverage"
        status: pass
      - kind: unit
        ref: "crates/quran-corpus/tests/fixtures.rs#csv_adapter_reproduces_the_rich_ayahs"
        status: pass
      - kind: unit
        ref: "crates/quran-corpus/tests/fixtures.rs#golden_ayah_texts_cover_the_rich_fixture"
        status: pass
    human_judgment: false
  - id: D2
    description: "Companion synthetic reference edition (reference.json) with a declared synthetic reference_corpus_id and a sha256 reference_text_hash pin, byte-identical to the rich ayah stream."
    requirement: "REQ-ingestion-validation-eval"
    verification:
      - kind: unit
        ref: "crates/quran-corpus/tests/fixtures.rs#rich_reference_is_a_valid_synthetic_edition_with_both_pins"
        status: pass
      - kind: unit
        ref: "crates/quran-corpus/tests/fixtures.rs#rich_reference_and_identity_fixtures_validate_without_blocking_findings"
        status: pass
    human_judgment: false
  - id: D3
    description: "Synthetic identity/primary/license fixture (test-edition-identity) declaring is_primary: true, a non-null upstream_edition_slug/qai_edition_id, and a concrete declared license."
    requirement: "REQ-quran-corpus"
    verification:
      - kind: unit
        ref: "crates/quran-corpus/tests/fixtures.rs#identity_fixture_declares_identity_primary_and_declared_license"
        status: pass
    human_judgment: false
  - id: D4
    description: "All three new fixtures are import-viable: validate_edition yields no Fatal/Error finding, so run_import (and plan 02-03's import/activate snapshots) succeed."
    requirement: "REQ-ingestion-validation-eval"
    verification:
      - kind: unit
        ref: "crates/quran-corpus/tests/fixtures.rs#rich_reference_and_identity_fixtures_validate_without_blocking_findings"
        status: pass
      - kind: integration
        ref: "cargo test -p quran-corpus  (49 lib + 6 adversarial + 13 fixtures passed)"
        status: pass
    human_judgment: false

duration: 40 min
completed: 2026-09-25
status: complete
---

# Phase 02 Plan 02: Richer synthetic edition fixtures Summary

**A 6-surah / 24-ayah synthetic edition (1..=6), its CSV/golden mirrors, a byte-identical companion reference with a real text-hash pin, and an identity/primary/license fixture — all proven import-viable under QV-002.**

## Performance

- **Duration:** ~40 min
- **Started:** 2026-09-25T15:48:00Z (adopted in-flight work)
- **Completed:** 2026-09-25T12:57:39Z (commit clock)
- **Tasks:** 2
- **Files modified:** 6 (4 created, 2 modified)

## Accomplishments

- Deepened the exercised dataset (D-08) from 5 surahs / 14 ayahs to 6 surahs / 24 ayahs, stressing a basmala inside an ayah body (the An-Naml 27:30 shape), U+06D6…U+06ED pause marks, a mark-separated token, a long multi-token ayah, page/juz/hizb/rub/manzil/ruku/division boundaries, a muqattaʿat standalone token, a legitimate repeated refrain (QV-022 Info path), and a superscript alef U+0670.
- Landed the companion synthetic reference (`reference.json`) as a valid `EditionSource` whose ayah stream matches the rich edition byte-for-byte, with `reference_corpus_id: synthetic-rich-reference` and a `sha256:` pin derived from the existing `text_hash` recipe — no real corpus identity, license, or signer named (OD-03 stays open).
- Landed the identity/primary/license fixture whose declared `upstream_edition_slug`, `qai_edition_id`, `is_primary: true`, and concrete license now give plan 02-03's operator snapshots something real to import.
- Made the fixtures *import-viable*, not merely parseable: the rich fixture was renumbered to surahs 1..=6 (QV-002 is Fatal and `run_import` aborts on any Fatal finding), and a new test asserts `validate_edition` produces no Fatal/Error finding for every new fixture — closing the exact gap that would have broken plan 02-03.

## Task Commits

Each task was committed atomically:

1. **Task 1: Richer synthetic edition fixture with CSV and golden mirrors** — `0797dc7` (feat)
2. **Task 2: Companion synthetic reference and the identity/primary/license fixture** — `3e93af5` (feat)

**Plan metadata:** committed with this SUMMARY (docs: complete plan)

## Files Created/Modified

- `fixtures/quran/test-edition-rich/manifest.json` — 6 surahs / 24 ayahs numbered 1..=6, `synthetic: true`, both reference pins; all D-08 stressors.
- `fixtures/quran/test-edition-rich/ayahs.csv` — CSV mirror reproducing the JSON ayahs field-for-field under `CsvAdapter`.
- `fixtures/quran/test-edition-rich/reference.json` — synthetic companion reference, byte-identical ayah/surah stream, both pins set.
- `fixtures/quran/test-edition-identity/manifest.json` — synthetic identity fixture with `is_primary: true`, non-null upstream/qai identity, and a declared `verified` license.
- `fixtures/quran/golden/ayah_texts.jsonl` — extended to 38 rows with an `EDITION` discriminator (14 min + 24 rich).
- `crates/quran-corpus/tests/fixtures.rs` — edition-scoped golden loader, rich shape/coverage, CSV parity, reference/identity shape, and the validation-clean assertion.

## Decisions Made

- Fixture surah numbering must satisfy QV-002 (exactly 1..=N); the rich fixture is 1..=6 and identity is 1. This is what keeps the fixture on the real import path instead of a bypass.
- Golden rows are edition-scoped (`EDITION` field) because, after the QV-002 fix, more than one synthetic edition is numbered from 1 and a bare `(surah, ayah)` key would collide.
- The reference pin reuses the existing `text_hash()` primitive with its frozen `qai-text-hash-v1` recipe (verified independently: `sha256:6c9e69…`); no new recipe was invented. Renumbering surahs does not change the pin because `text_hash` feeds only the ayah text stream (plus slug/version).
- No real dataset slug, publisher, license, or signer is named anywhere; OD-01/OD-03 remain recorded owner gates.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Renumbered the rich fixture to surahs 1..=6 (and identity to surah 1)**

- **Found during:** Adoption/diff, before Task 1 completion
- **Issue:** The in-flight rich fixture used surah numbers 101–106 (and the identity fixture 201) to keep golden keys disjoint from `test-edition-min`'s 1–5. But `validate_edition`'s QV-002 is `Fatal` when surah numbers are not exactly `1..=N`, and `run_import` aborts on any Fatal finding (`import.rs` `validated()`). Plan 02-03 imports and activates both `test-edition-rich` (reference test) and `test-edition-identity` (edition-show snapshot), so both fixtures could never have been imported. The plan did not resolve the tension.
- **Fix:** Renumbered rich surahs/ayahs 101–106 → 1–6 and identity surah 201 → 1 (names/revelation_order updated to match); all D-08 stressors and every ayah's division metadata preserved. Chose this over keeping the offset because it is the only option that keeps the fixture on the real QV-002-guarded import path that 02-03 depends on.
- **Files modified:** `fixtures/quran/test-edition-rich/manifest.json`, `fixtures/quran/test-edition-rich/ayahs.csv`, `fixtures/quran/test-edition-rich/reference.json`, `fixtures/quran/test-edition-identity/manifest.json`
- **Verification:** `rich_reference_and_identity_fixtures_validate_without_blocking_findings` passes; `test_edition_rich_parses…` asserts surahs are 1..=6.
- **Committed in:** `0797dc7` (manifest/CSV) and `3e93af5` (reference/identity/test)

**2. [Rule 3 - Blocking] Edition-scoped golden keys**

- **Found during:** Task 1 (golden extension)
- **Issue:** With both `test-edition-min` and `test-edition-rich` numbered from 1, the golden loader's `(surah, ayah)` key (and its duplicate-row assertion) collides across editions.
- **Fix:** Added an `EDITION` discriminator to every golden row; `load_golden(edition)` filters by it and `assert_golden_covers` now also asserts exact coverage of the edition's ayahs. `test-edition-min` byte content is unchanged — only the discriminator field was added.
- **Files modified:** `fixtures/quran/golden/ayah_texts.jsonl`, `crates/quran-corpus/tests/fixtures.rs`
- **Verification:** `golden_ayah_texts_match_the_imported_fixture` (14 rows) and `golden_ayah_texts_cover_the_rich_fixture` (24 rows) both pass.
- **Committed in:** `0797dc7` (golden) and `3e93af5` (loader)

**3. [Rule 2 - Missing Critical] Proved fixture import-viability, not just parseability**

- **Found during:** Task 2
- **Issue:** The plan's tests only asserted parsed shape; nothing guarded against a fixture that parses but fails `validate_edition`, which is exactly the QV-002 failure mode above and would silently break 02-03.
- **Fix:** Added `rich_reference_and_identity_fixtures_validate_without_blocking_findings`, asserting `validate_edition` yields no `Fatal`/`Error` for rich, reference, and identity.
- **Files modified:** `crates/quran-corpus/tests/fixtures.rs`
- **Verification:** new test passes; `cargo test -p quran-corpus` green (49 lib + 6 adversarial + 13 fixtures).
- **Committed in:** `3e93af5`

### Commit-boundary note

The plan's two tasks share `crates/quran-corpus/tests/fixtures.rs` (Task 1 adds rich/CSV/golden tests, Task 2 adds reference/identity tests in the same file). Splitting a single file across commits would require interactive hunk staging, so commits are split by concern instead: `0797dc7` = Task 1 dataset (rich manifest/CSV/golden), `3e93af5` = Task 2 fixtures plus all fixture-shape/verification tests. Both commits are self-consistent and build/test green.

---

**Total deviations:** 3 auto-fixed (1 missing-critical, 2 blocking) + 1 commit-boundary note
**Impact on plan:** All three auto-fixes were required for the plan's own goal ("corpus integrity checks pass is only meaningful if the corpus stresses the checks") to hold on the real import path; the QV-002 renumber is what keeps plan 02-03 viable. No scope creep: no source, migration, `test-edition-min`, or adversarial fixture was modified, and no real identity was introduced.

## Issues Encountered

- Two pre-existing CLI snapshot failures (`quran_normalize_snapshots`, `quran_search_snapshots`) remain in `cargo test -p cli --test quran` from a non-deterministic index manifest hash; confirmed failing before plan 02-01. Out of scope; snapshots were not overwritten.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 02-03 is unblocked: `test-edition-rich` (json + csv + reference) and `test-edition-identity` now both import and validate cleanly under QV-002, so the verify/reference and edition-identity operator snapshots have real, importable fixtures.
- `reference_text_hash` pin is the genuine `text_hash` output, so 02-03's reference-configured import will report QV-015 `outcome: "pass"` rather than a hash mismatch.
- OD-01 (dataset identity/license), OD-02 (reviewer), and OD-03 (reference corpus) remain open owner gates; no real identity was named anywhere in the diff.
- `REQ-quran-corpus` and `REQ-ingestion-validation-eval` are shared with sibling plans (02-03…02-07) that have no SUMMARY yet; the shared-ID gate keeps them pending.

## Self-Check: PASSED

- Key files exist: `fixtures/quran/test-edition-rich/manifest.json`, `.../ayahs.csv`, `.../reference.json`, `fixtures/quran/test-edition-identity/manifest.json`, `fixtures/quran/golden/ayah_texts.jsonl`, `crates/quran-corpus/tests/fixtures.rs`.
- Task commits exist: `0797dc7`, `3e93af5`.
- Plan verification re-run green: `cargo test -p quran-corpus --test fixtures` (13 passed), `--test adversarial` (6 passed), `cargo test -p quran-corpus` (49 lib + 6 + 13); `git status --short fixtures/quran/test-edition-min fixtures/quran/adversarial` clean.

---
*Phase: 02-canonical-quran-core*
*Completed: 2026-09-25*
