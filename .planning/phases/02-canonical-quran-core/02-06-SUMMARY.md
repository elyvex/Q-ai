---
phase: 02-canonical-quran-core
plan: 06
subsystem: database
tags: [quran, translation, hashing, license, layer-separation, domain-separation, provenance]

requires:
  - phase: 01-foundations
    provides: translation tables (migration 0010), the frozen quran-corpus hashing primitives, the QuranQuotation / AyahView type guards, and the application acceptance harness
  - phase: 02-canonical-quran-core
    provides: plan 02-01's verbatim declared-license storage pattern and plan 02-04's approval-gated canonical-write fence plus importer code-path audit

provides:
  - quran-corpus::hashing::translation_text_hash — an additive, domain-separated `qai-translation-hash-v1` recipe
  - real `sha256:<64-hex>` translation_editions.text_hash for every imported translation, computed over passages in ascending (surah, ayah) order
  - verbatim translation_editions.license_json — the declared SPDX identifier preserved character-for-character; an undeclared license stays explicit Unknown with no invented permission flags
  - a layer-separation negative test proving a translation cannot reach a canonical slot (read surface + constructor source scan)

affects: [02-05 quotation hard-failure wiring, 02-07 phase evidence of record, phase 3/5 translation display surfaces]

actuals:
  tokens: 4895     # chars/4 over the realized diff (19582 chars, crates/)
  tasks: 2
  commits: 2       # MEASURED: git rev-list --count 8837ccf..HEAD
  plan_head_before: 8837ccf995651e6573a3b1365be703754bddf1b4

tech-stack:
  added: []
  patterns:
    - "Additive hashing: a new content class gets its own length-prefixed domain label; the frozen v1 recipes and their domain strings are never edited (ADR-0108, D-12)"
    - "Declared-vs-invented licensing: store the operator-declared identifier verbatim; an absent declaration stays explicitly unknown with no inferred permission"
    - "Trust-boundary negative test: prove a layer boundary at the read surface and with a constructor source scan, not with prose"

key-files:
  created: []
  modified:
    - crates/quran-corpus/src/hashing.rs
    - crates/quran-corpus/src/lib.rs
    - crates/application/src/quran.rs
    - crates/application/tests/quran_translation.rs

key-decisions:
  - "The translation content hash reuses the frozen length-prefixed feed/finish primitives under a new `qai-translation-hash-v1` label; `text_hash`, `structure_hash`, and `token_order_hash` and their domain strings are untouched (D-12, ADR-0108)."
  - "`text_hash` is computed over passages sorted ascending by (surah, ayah) before insertion, so the digest is independent of the manifest's passage order (QC-08, T-02-25)."
  - "A declared translation license stores the manifest's SPDX identifier character-for-character (status `OpenLicense`); an undeclared license stays explicit `Unknown` with no redistribution/export permission invented (OD-01 remains the owner gate) (T-02-24)."
  - "Layer separation is construction-/table-/type-/test-level: canonical and translation rows live in `quran_*` versus `translation_*` tables, `AyahView.canonical` is a `QuranQuotation`, and a source scan asserts no canonical view constructor takes a translator/language pair (D-04, ADR-0112)."

patterns-established:
  - "Additive hashing recipe: new recipe names are domain-separated and length-prefixed; the frozen v1 digests stay byte-identical."
  - "Verbatim declared license: the operator-declared identifier is stored exactly; absence is explicit and never filled with an inferred permission."
  - "Boundary negative test: a served view proves canonical vs translation, plus a source scan of the canonical module for a translator/language constructor."

requirements-completed: [REQ-data-separation-layers, REQ-ingestion-validation-eval]

coverage:
  - id: D1
    description: "Every imported translation carries a real order-independent content hash produced by a new domain-separated recipe; the three frozen v1 recipes are byte-identical."
    requirement: "REQ-data-separation-layers"
    verification:
      - kind: unit
        ref: "crates/quran-corpus/src/hashing.rs#translation_recipe_is_domain_separated_from_the_frozen_v1_recipes"
        status: pass
      - kind: integration
        ref: "crates/application/tests/quran_translation.rs#import_translations_persists_attributed_edition_and_passages"
        status: pass
      - kind: integration
        ref: "crates/application/tests/quran_translation.rs#translation_hash_is_independent_of_manifest_passage_order"
        status: pass
    human_judgment: false
  - id: D2
    description: "A translation's declared license is persisted verbatim; an undeclared license yields an explicit unknown status with no invented redistribution permission."
    requirement: "REQ-data-separation-layers"
    verification:
      - kind: integration
        ref: "crates/application/tests/quran_translation.rs#import_translations_persists_the_declared_license_verbatim"
        status: pass
      - kind: integration
        ref: "crates/application/tests/quran_translation.rs#import_translations_keeps_an_undeclared_license_unknown"
        status: pass
    human_judgment: false
  - id: D3
    description: "A served view's canonical slot is the canonical quotation (canonical Arabic + canonical hash); the translation rides only in the attributed sidecar; no canonical view constructor accepts a translator/language pair."
    requirement: "REQ-data-separation-layers"
    verification:
      - kind: integration
        ref: "crates/application/tests/quran_translation.rs#translation_cannot_reach_a_canonical_slot"
        status: pass
      - kind: unit
        ref: "crates/quran-core/src/view.rs#translation_requires_translator_and_edition"
        status: pass
    human_judgment: false
  - id: D4
    description: "No frozen recipe, no migration, and no canonical type changed: the additive work is confined to a new recipe and the translation import path, and the architecture boundary still holds."
    requirement: "REQ-ingestion-validation-eval"
    verification:
      - kind: other
        ref: "cargo run -q -p xtask -- arch-check (OK — no forbidden dependency edges)"
        status: pass
      - kind: unit
        ref: "cargo test -p quran-corpus --lib (50 passed; frozen-v1 hashing tests green)"
        status: pass
      - kind: other
        ref: "git diff --name-only 8837ccf..HEAD -- migrations/ (empty)"
        status: pass
    human_judgment: false

duration: 12 min
completed: 2026-09-25
status: complete
---

# Phase 02 Plan 06: Translation-layer completion Summary

**Translations now carry a real order-independent `sha256:` content hash from an additive domain-separated recipe, a verbatim declared license (undeclared stays explicit `Unknown`), and an executable negative test proving a translation can never reach a canonical slot.**

## Performance

- **Duration:** 12 min
- **Started:** 2026-09-25T15:40:10Z
- **Completed:** 2026-09-25T15:52:41Z
- **Tasks:** 2
- **Files modified:** 4 (0 created, 4 modified)

## Accomplishments

- Closed the translation content-hash gap (QC-08, D-12): added `quran-corpus::hashing::translation_text_hash` under a new length-prefixed `qai-translation-hash-v1` label, reusing the frozen `feed`/`finish` primitives. `import_translations` now stores the tagged `sha256:<hex>` digest over the manifest's passages sorted ascending by `(surah, ayah)`, so the value is independent of insertion/manifest order. The frozen `qai-text-hash-v1`, `structure_hash`, and `token_order_hash` recipes and their domain strings are byte-identical, and a test proves the new digest differs from each (T-02-23, T-02-25).
- Closed the synthesized-license gap (QC-09 tail, D-04): `import_translations` now persists the manifest's declared SPDX identifier character-for-character instead of overwriting it with an `Unknown` placeholder. A declared license reads as `OpenLicense`; an undeclared license stays explicit `Unknown`; and no redistribution or export permission is invented either way (T-02-24, OD-01 remains the owner gate).
- Proved layer separation instead of asserting it (D-04, ADR-0112, T-02-22): a new negative test serves a real imported translation and asserts the canonical slot is the canonical `QuranQuotation` (canonical Arabic + the stored canonical per-ayah hash), the translation text appears only in the attributed sidecar with its translator, the serialized layers stay distinct, and a source scan over `crates/quran-core/src/view.rs` finds no canonical constructor that takes a translator/language pair. The test doc comment records that canonical and translation rows live in separate tables (`quran_*` versus `translation_*`).

## Task Commits

Each task was committed atomically:

1. **Task 1: Additive translation hash recipe and a real translation text_hash** - `271939d` (feat)
2. **Task 2: Verbatim translation license and a layer-separation negative test** - `cb21a61` (feat)

**Plan metadata:** committed with this SUMMARY (docs: complete plan)

## Files Created/Modified

- `crates/quran-corpus/src/hashing.rs` - `translation_text_hash` (`qai-translation-hash-v1`) plus the domain-separation test
- `crates/quran-corpus/src/lib.rs` - re-export `translation_text_hash` (was unreachable from `application` otherwise)
- `crates/application/src/quran.rs` - `import_translations` computes the sorted-passage hash and persists the verbatim declared license
- `crates/application/tests/quran_translation.rs` - hash format + order-independence assertions, declared/undeclared license tests, and the layer-separation negative test

## Decisions Made

- The translation content hash is a new recipe, not a reuse of `qai-text-hash-v1`: a translation is a non-canonical layer, and D-12/ADR-0108 permit only additive, domain-separated recipe names.
- The digest key is `(surah, ayah)` ascending, matching the canonical `text_hash` ordering convention so a translation hash is stable and comparable.
- A declared SPDX identifier maps to `LicenseStatus::OpenLicense` while permission flags stay `false`; nothing is inferred from the translator, publisher, or repository, and no permission is granted.
- The layer boundary is proven at three levels — table (`quran_*` vs `translation_*`), type (`QuranQuotation` vs `AttributedTranslation`), and test (read surface + constructor scan) — per the phase's "a test, not a comment" rule.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Re-exported `translation_text_hash` from the `quran-corpus` crate root**
- **Found during:** Task 1
- **Issue:** The plan's `<files>` listed `hashing.rs` and the application call site, but the `application` crate reaches `quran-corpus` through its crate-root re-exports; without a `pub use`, `quran_corpus::translation_text_hash` is unreachable and the crate does not compile.
- **Fix:** Added `translation_text_hash` to the existing `pub use hashing::{…}` list in `crates/quran-corpus/src/lib.rs`.
- **Files modified:** crates/quran-corpus/src/lib.rs
- **Verification:** `cargo test -p quran-corpus --lib` (50 passed), `cargo test -p application --test quran_translation` (13 passed).
- **Committed in:** `271939d` (Task 1 commit)

### Process Note

- The Task 2 test file was reformatted by `rustfmt` (`--edition 2024`) after the first commit; the one-line reflow was folded into `cb21a61` via `git commit --amend --no-edit` (no `--no-verify`). Pre-existing rustfmt drift in unrelated files (`quran_cli.rs`, `provenance/src/lib.rs`, `quran-corpus/src/import.rs`, `tests/fixtures.rs`, `tests/import_path.rs`, `storage-sqlite/tests/quran.rs`) was left untouched per the scope boundary.

---

**Total deviations:** 1 auto-fixed (Rule 3 blocking)
**Impact on plan:** The single auto-fix was required for the crate to compile; it is a pure re-export with no behavior change. No scope creep: no migration, no canonical type, no frozen recipe, and no canonical text was changed.

## Issues Encountered

- The plan's `<output>` line names `02-05-SUMMARY.md`, but this plan is `02-06`; this executor wrote `02-06-SUMMARY.md` as the plan file number (and the phase board) is authoritative. Noted, not corrected in the plan file (mirrors the 02-04 execution note).
- Pre-existing `cargo fmt --all -- --check` drift exists in files unrelated to this plan; out of scope and left untouched.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `REQ-data-separation-layers` is complete on the translation side: attributed (translator mandatory), aligned (`aligned_edition_id`), hashed (real `sha256:` content hash), licensed (verbatim declared or explicit Unknown), and provably unable to reach a canonical slot.
- Plan 02-05 (quotation hard-failure wiring) is unblocked; the frozen recipe/serverless surface was not touched, so `arch-check`, the reader, and the quotation type guards are unchanged.
- Owner gates OD-01 (dataset/license), OD-02 (reviewer), and OD-03 (reference corpus) remain open and recorded, not closed. `REQ-data-separation-layers` and `REQ-ingestion-validation-eval` are shared with sibling plans (02-05/02-07) that have no SUMMARY yet, so the shared-ID gate keeps them pending.

## Self-Check: PASSED

- Key files exist: `crates/quran-corpus/src/hashing.rs`, `crates/quran-corpus/src/lib.rs`, `crates/application/src/quran.rs`, `crates/application/tests/quran_translation.rs`, `.planning/phases/02-canonical-quran-core/02-06-SUMMARY.md`.
- Task commits exist: `271939d`, `cb21a61`.
- Plan verification re-run green: `cargo test -p quran-corpus --lib` (50 passed), `cargo test -p application --test quran_translation` (13 passed), `cargo test -p quran-core --lib` (39 passed), `cargo run -q -p xtask -- arch-check` (OK); `cargo clippy -p quran-corpus -p application --all-targets -- -D warnings` clean; no migration file added.

---

*Phase: 02-canonical-quran-core*
*Completed: 2026-09-25*
