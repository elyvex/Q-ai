---
phase: 03-quran-search-linguistics
fixed_at: 2026-10-02T00:00:00Z
review_path: /Users/ali/dev/rust/Q-ai/.planning/phases/03-quran-search-linguistics/03-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 03: Code Review Fix Report

**Fixed at:** 2026-10-02T00:00:00Z
**Source review:** `.planning/phases/03-quran-search-linguistics/03-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 3 (WR-01…WR-03; the `warning` findings)
- Fixed: 3
- Skipped: 0
- Info items IN-01…IN-04 were left as recorded in the review (not in scope).

## Fixed Issues

### WR-01: Family-relation dedup is applied before the pair filter

**Files modified:** `crates/application/src/quran_morphology.rs`
**Commit:** 9ae0646
**Applied fix:** In `typed_relation_rows`, the unordered pair is now recorded in
`emitted` only *after* `pair_filter` accepts it. A pair linked by several keys is
represented by different analyses per key; previously the first (BTreeMap-ordered)
key consumed the pair even when its representative analyses failed the filter, so a
later key that would have accepted the relation was skipped. The emitted row now
uses the first key that actually passes, and `evidence_json.shared_key` names that
key. Added `pair_linked_by_several_keys_is_not_dropped_by_a_failing_key`, which
reproduces the reviewer's example (`{r1/lA, r2/lB}` × `{r1/lA, r2/lC}` with a
lemma-difference filter): 1 row emitted where the old code emitted 0.

### WR-02: `quran.search` attributes results to the active edition even when another edition is searched

**Files modified:** `crates/application/src/quran_tools.rs`
**Commit:** 545d59e
**Applied fix:** `backend_search` now builds its envelope metadata with the new
`search_meta(reader, params.edition)` helper instead of `active_meta`. A new
`search_edition_selector` maps the request to the same selector the search uses:
`None` → `EditionSelector::Active`, `slug` → `EditionSelector::Slug`, and
`slug@version` → `EditionSelector::Pinned` (an unparseable version is a typed
`ToolError::InvalidInput` for `quran.search`). The envelope's `edition_id`,
`edition_version`, and `reproducibility` checksum now name the edition the hits
come from. Added three selector unit tests (bare slug, pinned version, invalid
version).

### WR-03: `derive_license_status` invents a permissive `OpenLicense` from any non-empty `spdx_id`

**Files modified:** `crates/quran-morphology/src/license.rs`, `crates/application/src/quran_cli.rs`
**Commit:** 83f4690
**Applied fix:** Added `KNOWN_OPEN_SPDX_IDS` (an explicit, case-insensitive
allowlist of recognized open identifiers: CC0/CC-BY/CC-BY-SA, MIT, Apache-2.0,
BSD, ISC, MPL, GPL/LGPL/AGPL, Unlicense, ODbL/PDDL, OFL, etc.) and
`is_known_open_spdx` to the pure `license` module. `derive_license_status` now
maps only a recognized identifier to `OpenLicense`; any other non-empty
`spdx_id` (`LicenseRef-Proprietary`, `NOASSERTION`, a typo) is `Unspecified` and
is never treated as permissive — even when the capture asserts
`redistribution_allowed: true`. With no `spdx_id`, an explicit
`redistribution_allowed: true` still derives `PermissionGranted`. This keeps the
gate fail-closed and satisfies the plan's prohibition T-03-23 ("never guess a
status"). Extended the CLI derivation test with the arbitrary-identifier case and
added an allowlist unit test.

## Skipped Issues

None — all in-scope findings were fixed. Info items IN-01…IN-04 remain as
recorded in the review.

## Verification (where it ran)

All gates ran in the main checkout (`/Users/ali/dev/rust/Q-ai`, branch `main`).

- `cargo check -p quran-morphology -p application` — ok (only 3 pre-existing
  `quran_index.rs` warnings)
- `cargo clippy -p quran-morphology -p application --lib` — no new warnings
- `cargo test -p quran-morphology --lib` — 43 passed, 0 failed (incl. new
  `spdx_allowlist_recognizes_open_ids_and_rejects_arbitrary_strings`)
- `cargo test -p application --lib` — 55 passed, 0 failed (incl. new
  `pair_linked_by_several_keys_is_not_dropped_by_a_failing_key` and the three
  `search_edition_tests`)
- `cargo test -p application --test counting` — 11 passed
- `cargo test -p application --test morphology_import` — 15 passed
- `cargo test -p application --test family_goldens all_curated_family_goldens_pass`
  — passed

## Pre-existing failures (out of scope, untouched)

These were confirmed to fail identically with the WR fixes stashed, so they are
not regressions:

- `cargo test -p application --test family_goldens all_root_lemma_goldens_pass`
  — fails at `rl-0304` ("token must have family members"). This belongs to
  uncommitted in-progress work (untracked
  `fixtures/quran/lexicon/root-lemma-goldens.jsonl` + modified
  `crates/application/tests/family_goldens.rs`), not to Phase 03's committed
  surface.
- `cargo test -p application --test quran_tools search_tool_returns_attributed_envelope`
  — fails at `registry.tool_names().len() == 7` (actual 12).
- `cargo test -p cli --test quran` — 5 pre-existing trycmd snapshot failures
  (`edition_identity`, `quran_counting_graph`, `quran_index_verify`,
  `quran_reference`, `quran_verify`).

## Notes

- The working tree carried unrelated uncommitted changes (`.planning/ROADMAP.md`,
  `.planning/STATE.md`, `.planning/state.json`, `.gsd/dispatch-isolation-sentinel.json`,
  `crates/application/tests/family_goldens.rs`, and untracked planning/test files).
  The three fix commits staged only the four source files listed above; none of
  the unrelated work was touched.
- UAT item 7 (`03-UAT.md`: WR-03 disposition) can now be closed as *fixed via
  allowlist* rather than accepted-as-risk. `03-REVIEW.md` and `03-UAT.md` were
  left unmodified; this report is additive.

---

_Fixed: 2026-10-02T00:00:00Z_
_Fixer: the agent_
_Iteration: 1_
