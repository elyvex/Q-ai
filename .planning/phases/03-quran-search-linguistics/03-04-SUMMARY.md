---
phase: 03-quran-search-linguistics
plan: 04
subsystem: quran-linguistics
tags: [rust, sqlite, quran-lexicon, word-family, family-relations, affix-search, golden-set, tdd, od-12]

# Dependency graph
requires:
  - phase: 02-canonical-quran-core
    provides: word_family_relations (migration 0017) with the relation CHECK domain, FamilyRelation/relation_name/explain_relation/FamilyMember::new, insert_family_relations + family_relations_for, quran_token_analyses/roots/lemmas, approval-gated activation, MorphologyToolError (QAI-MORPH-0004)
  - phase: 03-quran-search-linguistics
    provides: 03-01 alpha-smoke tracer and SC5 per-hit canonical pin (wave 1); 03-03 lexicon counting over the same active-dataset resolution
provides:
  - "Six new typed family builders in crates/application/src/quran_morphology.rs: build_same_form_relations, build_same_lemma_relations, build_same_stem_relations, build_derived_relations, build_inflectional_relations, build_affix_relations"
  - "Widened build_same_root_relations: every distinct same-root token pair, no longer only adjacent windows(2) analyses"
  - "Shared typed_relation_rows/build_typed_relations helpers: one db.write() UoW, relation_name() strings, explain_relation() explanations, FamilyMember::new validation, dataset attribution, status proposed, evidence_json carrying the shared key"
  - "affix_search fails closed with MorphologyToolError::UnavailableDataset { capability: 'affix/morpheme index' } (QAI-MORPH-0004) instead of the silent Ok(Vec::new()) on the active-dataset path; the labeled L7 heuristic branch is unchanged for the no-dataset case"
  - "fixtures/quran/lexicon/families/curated.jsonl: 154-row synthetic family golden set (all seven typed kinds) with header reviewed_by: pending-linguist / reviewed_at: null"
  - "crates/application/tests/family_goldens.rs: curated golden runner over the real builders + word_family read path"
affects: [03-05, 03-06, 03-07, 03-08, phase-04-graph, phase-05-result-contract]

# Actuals (#2632) — same estimateTokens scale as the plan's estimate (chars/4 over the realized diff)
actuals:
  tokens: 16733
  tasks: 3
  commits: 5
plan_head_before: 8c559131942869e2e14612d11510a62d88b387f8

tech-stack:
  added: []
  patterns:
    - "Generic typed-relation emitter: one key function + one pair filter per relation kind; token pairs are deduped by (left,right) index so a pair linked by several shared keys yields exactly one row per relation"
    - "Canonical token identity for family members is token:<surah>:<ayah>:<position>; analyses are grouped per token and ordered by analysis_index, so multi-analysis tokens never produce self-relations"
    - "Fail-closed lexicon tools: an active dataset with an unbuilt derived capability is a typed UnavailableDataset (QAI-MORPH-0004), never an empty Ok result the caller could read as 'no such thing'"
    - "Synthetic oracle agreement: the curated golden fixture is computed independently from the same documented key scheme, so drift shows up as a missing curated relation instead of a weakened assertion"

key-files:
  created:
    - crates/application/tests/family_goldens.rs
    - fixtures/quran/lexicon/families/curated.jsonl
  modified:
    - crates/application/src/quran_morphology.rs
    - crates/application/tests/morphology_import.rs

key-decisions:
  - "Family relation kinds are built as typed, explained, dataset-attributed rows and are never merged: every builder emits relation_name() strings, explain_relation() text, status 'proposed', and the test additionally asserts no is_correct/is_primary/selected column exists and that no analysis row is ever flipped off 'imported' (I11/I13/ADR-0210, T-03-14/T-03-17)."
  - "affix_search fails closed with the plan-named MorphologyToolError::UnavailableDataset (QAI-MORPH-0004) rather than a new error variant, keeping the Diagnostic code map unchanged; the capability string is exactly 'affix/morpheme index' so the message names the missing capability (G-09/T-03-16)."
  - "The family synthetic lexicon keys must satisfy quran_lemmas UNIQUE(dataset_id, lemma): lemma = k % 9 with root = k % 3 makes lemma→root functional (9 is a multiple of 3), so a lemma text never collides across roots at activation. The first draft (root k%4 / lemma k%5) tripped 'unique constraint violation' during activation and was corrected (Rule 1)."
  - "Relation semantics asserted: same_form = identical surface; same_lemma = identical lemma; same_stem = identical stem; same_root = identical root; derived = identical root with different lemma; inflectional = identical lemma with different stem; affix = shares a declared non-empty prefix or suffix morpheme. These are mechanics, not linguistically ratified claims."
  - "The 154-family curated set stays reviewed_by: pending-linguist / reviewed_at: null and every row carries synthetic_test_only: true; it is a synthetic mechanics oracle, never scholarly ground truth (T-03-15/OD-12)."
  - "Cross-dataset computational unification stays opt-in through suggest_computational + ReviewPromotion; no builder auto-merges and no relation is emitted without a mandatory explanation."
  - "build_same_root_relations is now a thin wrapper over the shared emitter, widening it from adjacent-analysis pairs to all distinct same-root token pairs, as the plan required."

patterns-established:
  - "Family building in one UoW: load list_analyses + list_roots + list_lemmas, group by token, derive typed rows, insert_family_relations, commit — the same shape as the original build_same_root_relations template"
  - "Golden fixture as a typed oracle: header carries version/corpus/reviewed_by/reviewed_at, rows carry kind + from/to token ids + expected_relation + synthetic_test_only, and the runner pins the header labeling so a premature 'Accepted' flip fails"
  - "TDD in Rust: the RED commit lands the new public builder signatures as Ok(0) stubs so the target test compiles and fails on its assertion for the planned behavior; GREEN replaces the bodies"

requirements-completed: [REQ-quran-linguistics]

# Coverage metadata (#1602)
coverage:
  - id: D1
    description: "Every typed family relation kind is built (same_form/same_lemma/same_stem/same_root/derived/inflectional/affix) as explained, dataset-attributed, status-proposed rows; same_root is widened beyond adjacent windows(2); no analysis is merged or marked preferred."
    requirement: REQ-quran-linguistics
    verification:
      - kind: integration
        ref: "cargo test -p application --test morphology_import -- family_relations_explained --nocapture"
        status: pass
      - kind: integration
        ref: "cargo test -p application --test morphology_import"
        status: pass
    human_judgment: false
  - id: D2
    description: "affix_search fails closed: an active dataset without a populated morpheme index returns MorphologyToolError::UnavailableDataset (QAI-MORPH-0004) naming 'affix/morpheme index' instead of a silent Ok(Vec::new()); the labeled L7 heuristic path still serves the no-dataset case and stays labeled."
    requirement: REQ-quran-linguistics
    verification:
      - kind: integration
        ref: "cargo test -p application --test morphology_import -- affix --nocapture"
        status: pass
      - kind: integration
        ref: "cargo test -p cli --test quran -- quran_counting"
        status: pass
    human_judgment: false
  - id: D3
    description: "A repeatable curated family golden check exists: 154 synthetic families (>= 120 required) covering all seven typed kinds, each resolving through word_family to its typed relation with a non-empty explanation, on a header pinned to reviewed_by: pending-linguist / reviewed_at: null."
    requirement: REQ-quran-linguistics
    verification:
      - kind: integration
        ref: "cargo test -p application --test family_goldens"
        status: pass
      - kind: other
        ref: "cargo run -q -p xtask -- arch-check"
        status: pass
    human_judgment: false
  - id: D4
    description: "Linguistic correctness of the family relations and ratification of the 154-family golden set."
    requirement: REQ-quran-linguistics
    verification: []
    human_judgment: true
    rationale: "The builders prove typed relations exist, are explained, and are attributed, but ADR-0210/0215 and OD-12/D-08 are BLOCKED: no qualified Arabic linguist has signed off on the tagset, root conventions, alignment, or the relation semantics. Until that sign-off is recorded in docs/reviews/ the golden set must stay reviewed_by: pending-linguist and no relation may be treated as scholarly ground truth."

# Metrics
duration: 18 min
completed: 2026-09-28
status: complete
---

# Phase 3 Plan 04: Complete Word-Family Engine + Curated Family Golden Set Summary

**Every typed word-family relation kind (`same_form`, `same_lemma`, `same_stem`, `same_root`, `derived`, `inflectional`, `affix`) is now built, explained, and dataset-attributed in one transaction — `affix_search` fails closed with a typed `QAI-MORPH-0004` capability error instead of a silent empty — and a 154-row synthetic family golden set pins the engine's behaviour until the OD-12 linguist reviews it.**

## Performance

- **Duration:** 18 min
- **Started:** 2026-09-28T02:17:26Z
- **Completed:** 2026-09-28T02:35:37Z
- **Tasks:** 3/3
- **Files modified:** 4 (2 source/test edited, 1 test created, 1 fixture created)

## Accomplishments

- **SC3/G-05 closed:** the family engine no longer builds only `same_root` (adjacent pairs). `build_same_form_relations`, `build_same_lemma_relations`, `build_same_stem_relations`, `build_derived_relations`, `build_inflectional_relations`, and `build_affix_relations` all exist, and `build_same_root_relations` is widened to every distinct same-root token pair.
- **Typed + explained by construction:** a shared `typed_relation_rows`/`build_typed_relations` pair keeps the original `build_same_root_relations` template — one `db.write()` unit of work, `relation_name()` for the relation string (so the `0017` CHECK domain is satisfied by construction), `explain_relation()` for the mandatory non-empty explanation, `FamilyMember::new` validation, dataset attribution, `status: "proposed"`, `evidence_json` naming the shared key, and `commit()`.
- **No merge, asserted:** the strengthened `family_relations_explained` test proves every emitted relation is typed + explained + attributed + still `proposed`, that all seven kinds are present, that no `is_correct`/`is_primary`/`selected` column exists on any lexicon table, and that no analysis row is ever flipped off `imported` (I11/I13/ADR-0210).
- **G-09 closed:** with an active dataset and no populated morpheme index, `affix_search` now returns `MorphologyToolError::UnavailableDataset` naming `affix/morpheme index` (code `QAI-MORPH-0004`) instead of `Ok(Vec::new())`. The labeled L7 heuristic branch is untouched and a new contract test proves it still answers (and stays labeled) without a dataset.
- **G-05 evidence exists:** `fixtures/quran/lexicon/families/curated.jsonl` carries 154 curated families (>= 120 required) spanning all seven typed kinds, and `crates/application/tests/family_goldens.rs` imports + activates the synthetic lexicon, runs every builder, and asserts each curated row resolves through `word_family` to its typed relation with a non-empty explanation.
- **Owner gate held:** the header stays `reviewed_by: pending-linguist` / `reviewed_at: null`, every row is `synthetic_test_only: true`, and the runner asserts that labeling so a premature "Accepted" flip fails.

## Task Commits

Each task was committed atomically (Tasks 1 and 2 are TDD tasks: RED → GREEN):

1. **Task 1: Build every typed family relation kind (G-05)**
   - `3b824e6` (test — RED: family document + assertions, builders stubbed to `Ok(0)`)
   - `56cc6ec` (feat — GREEN: shared emitter + all seven builders, widened `same_root`)
2. **Task 2: Typed capability error instead of silent empty for affix_search (G-09)**
   - `358e973` (test — RED: `affix_search_typed_unavailable` + updated `compare_and_unavailable_tools`)
   - `13af141` (feat — GREEN: typed `UnavailableDataset` on the active-dataset path)
3. **Task 3: Curated family golden set + runner (G-05)** - `3c12072` (test)

**Plan metadata:** (this commit) `docs(03-04): complete [plan-name] plan`

## Files Created/Modified

- `crates/application/src/quran_morphology.rs` — added `FamilyToken`/`family_tokens_from_rows`/`typed_relation_rows`/`build_typed_relations`; implemented and re-homed the seven public builders (widened `build_same_root_relations`); replaced the silent `Ok(Vec::new())` in `affix_search` with the typed capability error.
- `crates/application/tests/morphology_import.rs` — added the `family_document` synthetic lexicon + `read_pool` helper; rewrote `family_relations_explained` (all kinds, typed/explained/attributed assertions, non-merge checks); updated `compare_and_unavailable_tools`; added `affix_search_typed_unavailable`.
- `crates/application/tests/family_goldens.rs` (new) — curated golden loader, family lexicon setup (import → activate → all builders), and the per-row `word_family` verification with header pinning.
- `fixtures/quran/lexicon/families/curated.jsonl` (new) — header + 154 synthetic family rows (4 `same_form`, 25 each of the other six kinds).

## Decisions Made

- **Relation semantics (mechanics, not ratified scholarship):** `same_form` = identical surface; `same_lemma` = identical lemma; `same_stem` = identical stem; `same_root` = identical root; `derived` = identical root with a *different* lemma; `inflectional` = identical lemma with a *different* stem; `affix` = shares a declared non-empty prefix or suffix morpheme from `segments_json`. Each builder emits via `relation_name()` and explains via `explain_relation()`.
- **Fail-closed over new surface:** the affix gap is closed with the plan-named `MorphologyToolError::UnavailableDataset` (`QAI-MORPH-0004`) rather than a new variant, so the Diagnostic code map and the CLI's `tool_exit` mapping are unchanged; the capability string is exactly `affix/morpheme index` so the operator-facing message names what is missing.
- **Computational suggestions stay opt-in:** cross-dataset unification continues through `suggest_computational` + `ReviewPromotion`; no builder auto-merges and no relation exists without a mandatory explanation.
- **Golden set stays unreviewed:** the 154 curated families are a synthetic mechanics oracle; header `reviewed_by: pending-linguist`, `reviewed_at: null`, rows `synthetic_test_only: true`, asserted by the runner (T-03-15).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Family synthetic lexicon violated `quran_lemmas` UNIQUE(dataset_id, lemma)**
- **Found during:** Task 1 (Build every typed family relation kind)
- **Issue:** The first `family_document` scheme used `root = k % 4` and `lemma = k % 5`, so the same lemma text appeared under two different roots. Activation resolves lemmas by a `(dataset_id, root, lemma)` key and inserts them, so the second `lem-N` under a different root tripped `quran_lemmas`' `UNIQUE(dataset_id, lemma)` — `activate_morphology` failed with `Storage("unique constraint violation")`.
- **Fix:** Changed the scheme to `root = k % 3` with `lemma = k % 9` so lemma→root is functional (9 is a multiple of 3): a lemma text can only belong to one root. Documented the constraint in both test files so the oracle and the runner cannot drift.
- **Files modified:** `crates/application/tests/morphology_import.rs`, `crates/application/tests/family_goldens.rs`
- **Verification:** `cargo test -p application --test morphology_import -- family_relations_explained` and `cargo test -p application --test family_goldens` both pass; both documents now import + activate cleanly.
- **Committed in:** `3b824e6` (Task 1 RED commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** The fix was required for the planned synthetic fixture to activate at all. No scope creep: no new files, no new dependencies, no package installs, and the declared `files_modified` set is unchanged (`crates/quran-morphology/src/family.rs` needed no edit — the existing `relation_name`/`explain_relation`/`FamilyMember::new` helpers already covered every builder).

## Issues Encountered

- The plan's Task 3 `<verify>` command was written `cargo test -p application --test family_goldens --nocapture`; cargo rejects a trailing `--nocapture` without the `--` separator. Ran the equivalent `cargo test -p application --test family_goldens` (and `-- --nocapture`), both green. No plan change needed.
- The `UnavailableDataset` display string reads "no active morphology dataset for affix/morpheme index; import and activate one first", which is slightly imprecise when a dataset *is* active but lacks the morpheme index. The plan explicitly pinned this variant/code (G-09/T-03-16), so it was implemented as specified; the capability string was chosen so the message names the missing capability. Worth a wording review alongside the 03-05 family surface.

## User Setup Required

None - no external service configuration required. No new dependencies or package installs (threat `T-03-SC` remains accepted).

## Known Stubs

None. The RED `Ok(0)` builder stubs were replaced by real implementations in the GREEN commits; the golden runner and curated fixture contain no placeholder rows (every row is a computed, verified token pair). The intentional non-implementation is `affix_search`'s dataset backend, which now fails closed with a typed capability error instead of returning guessed or empty data.

## Threat Flags

None — no new network endpoint, auth path, file-access pattern, or trust-boundary schema change was introduced. All changes are within the existing derived-lexicon tables and the existing `QAI-MORPH-*` error map.

## Next Phase Readiness

- SC3's engine and evidence are in place for plan 03-05 to wire `qai quran family <kind> <id>` and `POST/GET /api/v1/quran/family` to a real, explained relation set (G-02 remains the surface gap).
- Builders, the typed affix error, and the golden runner are all green: `cargo test -p application --test morphology_import` (12 pass), `cargo test -p application --test family_goldens` (1 pass), `cargo test -p cli --test quran -- quran_counting` (pass), `cargo run -q -p xtask -- arch-check` (OK), `cargo clippy -p application --all-targets -- -D warnings` (clean).
- **Blockers carried forward (do not silently pass):** OD-11 (morphology dataset selection & licensing, ADR-0203) — all family behaviour is proven on the synthetic `synthetic_test_only` lexicon only. OD-12 / D-08 (named linguist; tagset/root conventions/alignment) — the 154-family set stays `reviewed_by: pending-linguist` and ADR-0210/0215 stay not-Accepted until a qualified Arabic linguist signs off in `docs/reviews/`.

---
*Phase: 03-quran-search-linguistics*
*Completed: 2026-09-28*

## Self-Check: PASSED

- All 4 declared artifacts exist: `crates/application/src/quran_morphology.rs`, `crates/application/tests/morphology_import.rs`, `crates/application/tests/family_goldens.rs`, `fixtures/quran/lexicon/families/curated.jsonl`.
- All 5 task commits exist: `3b824e6`, `56cc6ec`, `358e973`, `13af141`, `3c12072`.
- Verification commands re-run and green: `cargo test -p application --test morphology_import` (12 passed), `cargo test -p application --test family_goldens` (1 passed), `cargo test -p cli --test quran -- quran_counting` (passed), `cargo run -q -p xtask -- arch-check` (OK), `cargo clippy -p application --all-targets -- -D warnings` (clean).
