---
phase: 03-quran-search-linguistics
reviewed: 2026-09-28T12:00:00Z
depth: standard
files_reviewed: 36
files_reviewed_list:
  - crates/application/src/lib.rs
  - crates/application/src/quran_cli.rs
  - crates/application/src/quran_counting.rs
  - crates/application/src/quran_lexicon_api.rs
  - crates/application/src/quran_morphology.rs
  - crates/application/src/quran_tools.rs
  - crates/application/tests/alpha_e2e.rs
  - crates/application/tests/alpha_smoke.rs
  - crates/application/tests/canonical_display_identity.rs
  - crates/application/tests/counting.rs
  - crates/application/tests/doctor_indexes.rs
  - crates/application/tests/family_goldens.rs
  - crates/application/tests/morphology_import.rs
  - crates/application/tests/quran_tools.rs
  - crates/application/tests/search_goldens.rs
  - crates/application/tests/search_latency.rs
  - crates/cli/src/lib.rs
  - crates/cli/src/quran.rs
  - crates/cli/tests/quran.rs
  - crates/cli/tests/quran/counting_graph_s2.trycmd
  - crates/cli/tests/quran/family_s1.trycmd
  - crates/cli/tests/quran/family_s2.trycmd
  - crates/cli/tests/quran/search_s2.trycmd
  - crates/quran-morphology/src/lib.rs
  - crates/quran-morphology/src/license.rs
  - crates/quran-normalization/src/rules/mod.rs
  - crates/server/src/api.rs
  - crates/server/tests/api.rs
  - crates/storage-sqlite/src/quran.rs
  - crates/storage/src/quran.rs
  - crates/tool-registry/src/lib.rs
  - fixtures/quran/lexicon/families/curated.jsonl
  - fixtures/quran/morphology/license-matrix.json
  - fixtures/quran/performance/budgets.json
  - fixtures/quran/search/concatenated.jsonl
  - scripts/gen_concatenated_goldens.py
findings:
  critical: 0
  warning: 3
  info: 4
  total: 7
status: issues_found
---

# Phase 03: Code Review Report

**Reviewed:** 2026-09-28T12:00:00Z
**Depth:** standard
**Files Reviewed:** 36
**Status:** issues_found

## Summary

Scope was the source files changed by phase-3's non-doc commits (29 commits: 03-01…03-08),
covering normalized/concatenated search evidence, the lexicon counting/family engine,
the license-evidence activation gate, the CLI/HTTP/tool lexicon surfaces, and the
reviewer-visible fixtures/scripts.

Overall the phase is well constructed against the project's principles: search hits
carry the mandatory trace + canonical span + verified quotation, lexicon reads fail
closed with typed `UnavailableDataset` errors instead of empty envelopes, SQL is fully
parameterized, and the new fixtures pin `reviewed_by: pending-linguist` /
`synthetic_test_only` throughout. `cargo check` on the affected crates passes.

Three Warning issues and four Info items follow. No Critical issue found: there is no
injection, panic-on-input, secret, or crash path in the reviewed surface — the two
substantive risks are a provenance mismatch in the new search tool and a fail-open
default in the license-status derivation.

## Warnings

### WR-01: Family-relation dedup is applied before the pair filter, silently dropping valid relations

**File:** `crates/application/src/quran_morphology.rs:1900-1910`
**Issue:** In `typed_relation_rows`, the unordered token pair is inserted into `emitted`
**before** `pair_filter` is evaluated:

```rust
if !emitted.insert((left, right)) {
    continue;
}
let left_analysis = &per_token[left][*key];
let right_analysis = &per_token[right][*key];
if !pair_filter(left_analysis, right_analysis) {
    continue;
}
```

A token may carry several analyses, and `key_of` can map a token to several keys (e.g.
`derived` keys on the root, `inflectional` on the lemma, `affix` on each prefix/suffix).
When a pair shares more than one key, the representative analyses differ per key. If the
pair fails `pair_filter` under the first emitted key but would pass under a later key, the
later key is skipped (`emitted.insert` returns `false`) and the valid relation is dropped.
Example: token X = {root r1/lemma lA, root r2/lemma lB}, token Y = {root r1/lemma lA,
root r2/lemma lC}; under r1 the `derived` filter (`lA != lA`) fails and marks the pair
consumed, so the r2 pair (lB != lC) is never emitted. The doc comment promises "one row per
pair linked by several shared keys", but the code can emit zero.
**Fix:** Only record a pair as emitted once it has passed the filter (and keep the dedup
keyed per pair, not per key):

```rust
if !pair_filter(left_analysis, right_analysis) {
    continue;
}
if !emitted.insert((left, right)) {
    continue;
}
```

### WR-02: `quran.search` tool attributes results to the active edition even when a different edition is searched

**File:** `crates/application/src/quran_tools.rs:184-215` (and `:189`)
**Issue:** `backend_search` builds its envelope metadata from `active_meta(&self.reader)`
(the **active** edition), then passes `params.edition` straight into `search_normalized`.
The search service honours `params.edition` (`crates/application/src/quran_search.rs:903`
→ `open_serving(db, index_root, params.edition.as_deref())`), so when a caller supplies a
non-active edition the hits come from that edition while the envelope's `edition_id`,
`edition_version`, and the `reproducibility` checksum still name the **active** edition.
`canonical_references` (built from the hits themselves) will name the searched edition,
making the envelope internally inconsistent. This defeats the project's traceability
requirement for a user-selectable parameter.
**Fix:** Resolve the metadata from the same selector the search uses, or reject a
non-active edition for this tool. For example, resolve the `QuranEdition` for
`params.edition` (falling back to `EditionSelector::Active`) and feed that into
`meta_from_edition` before calling `search_normalized`.

### WR-03: `derive_license_status` invents a permissive `OpenLicense` from any non-empty `spdx_id`

**File:** `crates/application/src/quran_cli.rs:2788-2801`
**Issue:** When `--license-evidence` is supplied, the status used by activation gate 6 is
derived as: any non-empty `spdx_id` ⇒ `OpenLicense` (permissive, accepted by
`is_permissive_status`). Any string qualifies — including `LicenseRef-Proprietary`,
`NOASSERTION`, or a typo — so a capture that declares `redistribution_allowed: true` will
pass the gate and activate. This contradicts the module's own stated invariant in
`crates/quran-morphology/src/license.rs` ("Nothing here invents a status…") and the plan's
fail-closed posture (T-03-22: "never guess a status"). The gate is a licensing/attribution
control, so a fail-open default is a real risk.
**Fix:** Never map presence-of-a-string to permissiveness. Either (a) validate `spdx_id`
against an explicit allowlist of known-open identifiers (e.g. `CC0-1.0`, `CC-BY-*`, `MIT`,
`Public Domain`, …) and otherwise return `Unspecified`, or (b) require the operator to pass
`--license-status` explicitly and refuse to derive one from the capture.

## Info

### IN-01: Family relation builders are not idempotent

**File:** `crates/storage-sqlite/src/quran.rs:2672-2707` (INSERT at `:2678`)
**Issue:** `build_same_form_relations`/`build_same_lemma_relations`/`…` (in
`crates/application/src/quran_morphology.rs:1974+`) generate deterministic row ids
(`fam:{dataset}:{relation}:{from}:{to}`) and `insert_family_relations` uses a plain
`INSERT`, so re-running any builder against an already-built dataset fails on the primary
key. No production caller runs them today (only tests), so this is latent, but the
functions are `pub` and are the natural handlers for a future rebuild surface.
**Fix:** Make the insert idempotent (`INSERT … ON CONFLICT(id) DO NOTHING` / upsert) or
have each builder clear its own `(dataset_id, relation)` rows before inserting.

### IN-02: Family selectors are validated after `trim()` but passed untrimmed

**File:** `crates/application/src/quran_lexicon_api.rs:41-48`; `crates/application/src/quran_cli.rs:3154-3158`; `crates/tool-registry/src/lib.rs:444-450`
**Issue:** `FamilyArgs::new` (and the CLI/tool equivalents) reject only when
`kind.trim()`/`id.trim()` is empty, then store/pass the **untrimmed** value to
`word_family`. A selector such as `" token:1:1:1 "` passes the guard, matches no row, and
returns `Ok((dataset, vec![]))` — a 200/CLI success with an empty relation list, which is
the exact "empty 200" shape the phase's tests intend to rule out.
**Fix:** Store and forward trimmed values (`kind.trim().to_string()`,
`id.trim().to_string()`).

### IN-03: `quran.search` rule trace is taken from only the first hit

**File:** `crates/application/src/quran_tools.rs:211-218`
**Issue:** `normalization_rules` is populated from `output.hits.first()` only. If hits
carry heterogeneous traces (or if the first hit is empty), the envelope's disclosed rule
set under-reports what was applied to the other hits.
**Fix:** Union the rule ids across all hits (deduped, deterministically ordered), or state
in the field docs that it is the first hit's trace.

### IN-04: Sole-entry license-matrix fallback can attach an unrelated artifact's evidence

**File:** `crates/application/src/quran_cli.rs:2717-2737` (`select_evidence_entry`)
**Issue:** When a matrix has exactly one `artifacts` entry and its key does not match the
requested dataset slug, the sole entry is selected anyway. If the operator points
`--license-evidence` at a single-artifact matrix for a different artifact, that evidence
is silently bound to the imported dataset.
**Fix:** Require the slug to match (or require an explicit `--license-status` alongside),
so the sole-entry convenience cannot cross-wire attribution.

---

_Reviewed: 2026-09-28T12:00:00Z_
_Reviewer: DeepSeek V4.1 Flash (gsd-code-reviewer)_
_Depth: standard_
