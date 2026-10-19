---
phase: 03-quran-search-linguistics
verified: 2026-10-10T12:00:00Z
status: human_needed
score: 17/17 must-haves verified
covered_files:
  - .planning/REQUIREMENTS.md
  - .planning/ROADMAP.md
  - .planning/phases/03-quran-search-linguistics/03-01-PLAN.md
  - .planning/phases/03-quran-search-linguistics/03-01-SUMMARY.md
  - .planning/phases/03-quran-search-linguistics/03-02-PLAN.md
  - .planning/phases/03-quran-search-linguistics/03-02-SUMMARY.md
  - .planning/phases/03-quran-search-linguistics/03-03-PLAN.md
  - .planning/phases/03-quran-search-linguistics/03-03-SUMMARY.md
  - .planning/phases/03-quran-search-linguistics/03-04-PLAN.md
  - .planning/phases/03-quran-search-linguistics/03-04-SUMMARY.md
  - .planning/phases/03-quran-search-linguistics/03-05-PLAN.md
  - .planning/phases/03-quran-search-linguistics/03-05-SUMMARY.md
  - .planning/phases/03-quran-search-linguistics/03-06-PLAN.md
  - .planning/phases/03-quran-search-linguistics/03-06-SUMMARY.md
  - .planning/phases/03-quran-search-linguistics/03-07-PLAN.md
  - .planning/phases/03-quran-search-linguistics/03-07-SUMMARY.md
  - .planning/phases/03-quran-search-linguistics/03-08-PLAN.md
  - .planning/phases/03-quran-search-linguistics/03-08-SUMMARY.md
  - .planning/phases/03-quran-search-linguistics/03-CONTEXT.md
  - .planning/phases/03-quran-search-linguistics/03-REVIEW.md
  - .planning/phases/03-quran-search-linguistics/03-REVIEW-FIX.md
  - .planning/phases/03-quran-search-linguistics/03-SECURITY.md
  - .planning/phases/03-quran-search-linguistics/03-UAT.md
  - crates/application/src/quran_counting.rs
  - crates/application/src/quran_morphology.rs
  - crates/application/src/quran_tools.rs
  - crates/application/src/quran_lexicon_api.rs
  - crates/application/src/quran_cli.rs
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
  - crates/cli/src/quran.rs
  - crates/quran-morphology/src/license.rs
  - crates/server/src/api.rs
  - crates/storage-sqlite/src/quran.rs
  - crates/storage/src/quran.rs
  - crates/tool-registry/src/lib.rs
  - fixtures/quran/lexicon/families/curated.jsonl
  - fixtures/quran/morphology/license-matrix.json
  - fixtures/quran/performance/budgets.json
  - fixtures/quran/search/concatenated.jsonl
  - scripts/gen_concatenated_goldens.py
covered_digest: "v1:sha256:266ef74809db17b3df720ba8ae0aafec4b89020a6e2efea5481f08016f13abfb"
behavior_unverified: 0
overrides_applied: 0
re_verification:
  previous_status: human_needed
  previous_score: 17/17
  gaps_closed:
    - "WR-01: Family-relation dedup before pair filter — fixed in commit 9ae0646"
    - "WR-02: quran.search attributes to wrong edition — fixed in commit 545d59e"
    - "WR-03: derive_license_status invents OpenLicense — fixed in commit 83f4690 (allowlist)"
  gaps_remaining:
    - "OD-11: licenses/qac/ does not exist — UAT claims pass but evidence missing"
    - "OD-12: docs/reviews/ does not exist — UAT claims pass but evidence missing"
    - "ADR-0203/0204/0205/0210/0211/0215 remain Draft/Proposed — not Accepted"
  regressions: []
human_verification:
  - test: "OD-11 — owner captures the QAC license to licenses/qac/{LICENSE.txt,capture.json,attribution.txt} per licenses/README.md, then ratifies ADR-0203 Option A (bundle) or Option B (user-supplied import)."
    expected: "A human-written license capture exists, the license-matrix `qac` entry either flips to captured permissive values or stays Option B, and the decision is recorded in owner-decisions.md / decisions-needed.md."
    why_human: "Licensing/legal and dataset-selection decision; agent-uncloseable by design. It gates real licensed morphology activation, so SC3/SC4 are behavioral on the synthetic lexicon only until it closes."
  - test: "OD-12 / D-08 — a named qualified Arabic linguist ratifies the normalization rule catalog (N01–N24), profile ladder (L0–L8), morphology tagset, root conventions, and alignment semantics; record sign-off in docs/reviews/."
    expected: "ADR-0204/0205/0210/0215 flip to Accepted and the golden/curated headers move off `reviewed_by: pending-linguist` to a named reviewer + date."
    why_human: "Scholarly correctness of normalization/lemma/root/family semantics cannot be established by grep or tests; ADR-0204/0205/0210/0211/0215 remain Draft and all phase-3 goldens are `reviewed_by: pending-linguist`."
  - test: "Counting convention — linguist/owner ratifies what 'one occurrence' means under multi-analysis (SingleSource / AllAnalyses / OneVotePerToken)."
    expected: "ADR-0211 moves from Draft to Accepted with the chosen counting semantics; the synthetic counts (1 / 127 / 64) become ratifiable conventions rather than code-asserted behavior."
    why_human: "The counting semantics are exercised behaviorally on `synthetic_test_only` data only; whether they are the scholarly-correct convention is a human judgment."
  - test: "Full-corpus performance budgets — activate a licensed corpus, then re-run `cargo test -p application --test search_latency` and assert concatenated p99 <= 150 ms (ADR-0207)."
    expected: "The `applies_to: full-corpus` rows in fixtures/quran/performance/budgets.json execute and pass; results are reported against reference hardware."
    why_human: "The full-corpus rows cannot execute on the 14-ayah synthetic fixture and are OD-11-dependent (no licensed corpus is active)."
  - test: "Confirm SC4 scope — the shipped co-occurrence is form/profile-targeted; confirm whether 'for any root or lemma' requires lexicon root/lemma-scoped co-occurrence beyond the form-targeted surface + root/lemma frequency."
    expected: "Owner/planner confirms the accepted SC4 interpretation, or files a follow-up for lexicon-scoped co-occurrence."
    why_human: "Scope interpretation. `frequency` and per-surah `distribution` are delivered for root/lemma (root_frequency/lemma_frequency, incl. by_surah); `cooccurrence` runs over normalized token forms by profile, not over lexicon root/lemma ids."
  - test: "Live `qai serve` HTTP lexicon smoke — start `qai serve` on a migrated temp db and POST /api/v1/quran/family and /api/v1/quran/count/{root,lemma}-frequency, asserting typed error statuses for no-dataset / empty selector / unknown mode."
    expected: "Handler returns 404 QAI-LEX-0004 / 400 QAI-LEX-0002 style coded errors (never an empty 200) and attributed relations/counts when a dataset is active."
    why_human: "Performed manually in plan 03-05; the live-server path is not part of CI (server contract tests cover the handler logic in-process only)."
---

# Phase 3: Quran Search & Linguistics Verification Report

**Phase Goal:** Users can find and analyze Quranic words across orthographic variation without ever seeing altered display text.
**Verified:** 2026-10-10T12:00:00Z
**Status:** human_needed
**Re-verification:** Yes — after gap closure (WR-01/02/03 fixed)

## Goal Achievement

All five ROADMAP success criteria and the plan-level must-haves were checked goal-backward
against the live codebase and re-executed. Every must-have resolves to **✓ VERIFIED** with a
passing behavioral check. Three review warnings (WR-01/02/03) were fixed and confirmed in code.
The UAT (`03-UAT.md`) claims all 7 human verification items passed, but **the evidence does not
support these claims**: `licenses/qac/` does not exist, `docs/reviews/` does not exist, and
ADR-0203/0204/0205/0210/0211/0215 remain Draft/Proposed. The human verification items remain open.

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | **SC1** — a diacritic-free Arabic query returns canonical-location hits through the normalized path, each with an ordered `NormalizationTrace` + canonical span. | ✓ VERIFIED | `search_goldens` 3 tests pass (400 normalized rows + concatenated + recall); `alpha_smoke` 2 tests pass; `search_parity` 3 tests pass (5000-substring gate). |
| 2 | **SC2** — a spaceless (concatenated) phrase returns canonical-location hits with an explainable segmentation tiling the query and pinned refs; cross-ayah matches split per overlapped ayah. | ✓ VERIFIED | `search_goldens -- concatenated` passes (349 rows); `alpha_e2e` passes; `canonical_display_identity` 6 tests pass. |
| 3 | **SC3** — a user can inspect a word's lemma, root, multi-analysis morphology, and word family (typed + explained + attributed). | ✓ VERIFIED | `morphology_import` 15 tests pass; `family_goldens` 2 tests pass; `alpha_e2e` passes; `quran_tools` 11 tests pass. |
| 4 | **SC4** — a user can view frequency, per-surah distribution, and co-occurrence with a complete `CountingRules` block; root/lemma frequency over an active lexicon. | ✓ VERIFIED | `counting` 11 tests pass; `alpha_e2e` passes; `root_frequency`/`lemma_frequency` at `quran_counting.rs:825`. |
| 5 | **SC5** — displayed canonical text is never modified by normalization in any result. | ✓ VERIFIED | `canonical_display_identity` 6 tests pass (byte-identity across all 5 modes); `alpha_smoke` passes; `alpha_e2e` passes. |
| 6 | A hit cannot exist without verified quotation + span + trace; empty/whitespace query is rejected with a typed usage error. | ✓ VERIFIED | `alpha_smoke` passes; `quran_tools` 11 tests pass; `search_goldens` passes. |
| 7 | The concatenated golden set (>= 120) is produced by an oracle independent of the Rust stack. | ✓ VERIFIED | `scripts/gen_concatenated_goldens.py` exists; 349 rows in fixture; `search_goldens -- concatenated` passes. |
| 8 | When no dataset is active, root/lemma frequency returns typed `CountingError::UnavailableDataset` (`QAI-CNT-0005`). | ✓ VERIFIED | `counting` 11 tests pass; `quran_tools` 11 tests pass. |
| 9 | Every numeric output carries a complete `CountingRules`; changing the multi-analysis mode visibly changes count + rules. | ✓ VERIFIED | `counting` 11 tests pass; `alpha_e2e` passes. |
| 10 | Word-family relations are typed for every declared kind, each with a non-empty explanation + provenance. | ✓ VERIFIED | `morphology_import` 15 tests pass; `family_goldens` 2 tests pass. |
| 11 | CLI `family`/`lemma` and the versioned HTTP `family` + `count/{root,lemma}-frequency` routes are reachable, typed, and attributed. | ✓ VERIFIED | CLI `quran` 17 tests pass; server 42 tests pass. |
| 12 | A morphology dataset cannot reach `state='active'` without permissive license evidence + mandatory capture fields. | ✓ VERIFIED | `license_evidence` 4 tests pass; `morphology_import` 15 tests pass. |
| 13 | `qai doctor --indexes` reports 19 stable checks green after a rebuild+activate+search soak, turns red on injected drift, and stays read-only. | ✓ VERIFIED | `doctor_indexes` 7 tests pass. |
| 14 | A machine-readable budget artifact codifies concatenated p99 <= 150 ms and the fixture harness gates against the fixture-appropriate bound. | ✓ VERIFIED | `search_latency` 3 tests pass. |
| 15 | The tool registry exposes search/root/lemma/morphology/family with provenance/attribution and `normalization_rules`. | ✓ VERIFIED | `tool-registry` 16 tests pass; `quran_tools` 11 tests pass. |
| 16 | OD-11 and OD-12/D-08 are recorded as explicit BLOCKED items with closing steps; no agent closed them. | ✓ VERIFIED | `docs/05-followups/phase-03-owner-gates.md` exists; `docs/05-followups/decisions-needed.md` remains source of truth; ADRs remain Draft. |
| 17 | The full D-03 alpha runs green end-to-end on the synthetic fixture. | ✓ VERIFIED | `alpha_e2e` 1 test passes (all five search modes + morphology + family + frequency/distribution/co-occurrence). |

**Score:** 17/17 truths verified (0 present, behavior-unverified)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/application/tests/alpha_smoke.rs` | End-to-end tracer | ✓ VERIFIED | 365 lines; 2 tests pass. |
| `crates/application/tests/canonical_display_identity.rs` | SC5 per-hit byte-identity | ✓ VERIFIED | 475 lines; 6 tests pass. |
| `crates/application/tests/alpha_e2e.rs` | Full-chain alpha smoke | ✓ VERIFIED | 587 lines; 1 test passes. |
| `crates/application/tests/search_goldens.rs` | Loader + concatenated arm | ✓ VERIFIED | 567 lines; 3 tests pass. |
| `crates/application/tests/counting.rs` | root/lemma frequency + modes | ✓ VERIFIED | 501 lines; 11 tests pass. |
| `crates/application/tests/morphology_import.rs` | Family builders + license gate | ✓ VERIFIED | 1170 lines; 15 tests pass. |
| `crates/application/tests/family_goldens.rs` | Curated family runner | ✓ VERIFIED | 483 lines; 2 tests pass. |
| `crates/application/tests/doctor_indexes.rs` | 19-check soak + drift | ✓ VERIFIED | 622 lines; 7 tests pass. |
| `crates/application/tests/search_latency.rs` | Budget gate | ✓ VERIFIED | 349 lines; 3 tests pass. |
| `crates/application/tests/quran_tools.rs` | Attributed tool surface | ✓ VERIFIED | 11 tests pass. |
| `crates/application/src/quran_counting.rs` | Real root/lemma frequency | ✓ VERIFIED | 999 lines; `root_frequency` at line 825. |
| `crates/application/src/quran_morphology.rs` | Family builders + license gate | ✓ VERIFIED | 2276 lines; `word_family` at line 2097; `build_same_form_relations` at line 1978. |
| `crates/application/src/quran_tools.rs` | Tool backends | ✓ VERIFIED | 848 lines; `search_meta`/`search_edition_selector` confirmed. |
| `crates/application/src/quran_lexicon_api.rs` | LexiconBackend + service | ✓ VERIFIED | 468 lines. |
| `crates/quran-morphology/src/license.rs` | Pure license validation | ✓ VERIFIED | 289 lines; `KNOWN_OPEN_SPDX_IDS` at line 41; `is_known_open_spdx` at line 86. |
| `crates/tool-registry/src/lib.rs` | Attributed tool registry | ✓ VERIFIED | 1121 lines; all 5 tools registered. |
| `fixtures/quran/lexicon/families/curated.jsonl` | >= 120-family golden set | ✓ VERIFIED | 154 families; 7 typed kinds. |
| `fixtures/quran/morphology/license-matrix.json` | Machine-readable license matrix | ✓ VERIFIED | Valid JSON with capture fields. |
| `fixtures/quran/performance/budgets.json` | Budget table | ✓ VERIFIED | Codifies ADR-0207 + fixture bound. |
| `fixtures/quran/search/concatenated.jsonl` | >= 120-case concatenated golden | ✓ VERIFIED | 349 rows. |
| `scripts/gen_concatenated_goldens.py` | Independent Python oracle | ✓ VERIFIED | 478 lines. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `alpha_smoke.rs` / `alpha_e2e.rs` | `SearchHit` validating constructor | constructing `SearchHit` | WIRED | Hit cannot exist without trace + span + quotation. |
| `search_goldens.rs` dispatcher | `search_concatenated` service | direct call | WIRED | `all_concatenated_search_goldens_pass` green. |
| `cmd_family` / `qai quran family` | `word_family` service | `cmd_family` dispatch | WIRED | CLI `quran` 17 tests pass. |
| HTTP `lexicon_family_handler` | `LexiconApiService` | shared error mapping | WIRED | Server 42 tests pass. |
| `activate_morphology` gate 6 | `LicenseEvidence` | `from_status_and_json` | WIRED | `license_evidence` 4 tests pass. |
| forms rebuild | MV-018 canonical-unchanged | `rebuild_forms` | WIRED | Canonical hashes stable across searches. |
| tool registry | `quran_tools.rs` backends | registration | WIRED | `quran_tools` 11 tests pass. |
| `doctor --indexes` CLI | `run_index_checks` (19 ids) | `cmd_doctor_indexes` | WIRED | `doctor_indexes` 7 tests pass. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|-------------------|--------|
| `root_frequency`/`lemma_frequency` | count / by_surah | exact SQL over `quran_token_analyses` | Yes (bound params) | ✓ FLOWING |
| `word_family` | members | `list_family_relations` over derived family rows | Yes (typed, explained) | ✓ FLOWING |
| `frequency`/`distribution`/`cooccurrence` | counts / windows | `list_all_token_forms` over derived form tables | Yes | ✓ FLOWING |
| search hits (all 5 modes) | quotation / span / trace | FTS5 index + skeleton/trigram, verified against canonical | Yes | ✓ FLOWING |
| license gate | evidence | operator-supplied dataset row | Yes | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| SC1/SC5 tracer + empty-query guard | `cargo test -p application --test alpha_smoke` | 2 passed | ✓ PASS |
| SC5 byte-identity across 5 modes | `cargo test -p application --test canonical_display_identity` | 6 passed | ✓ PASS |
| SC1 400-row golden + SC2 concatenated | `cargo test -p application --test search_goldens` | 3 passed | ✓ PASS |
| 5,000-substring normalization parity | `cargo test -p quran-search --test search_parity` | 3 passed | ✓ PASS |
| SC4 counting + multi-analysis | `cargo test -p application --test counting` | 11 passed | ✓ PASS |
| SC3 family + license gate | `cargo test -p application --test morphology_import` | 15 passed | ✓ PASS |
| SC3 curated family golden | `cargo test -p application --test family_goldens` | 2 passed | ✓ PASS |
| Doctor 19-check soak | `cargo test -p application --test doctor_indexes` | 7 passed | ✓ PASS |
| Performance budget gate | `cargo test -p application --test search_latency` | 3 passed | ✓ PASS |
| Attributed tool surface | `cargo test -p application --test quran_tools` | 11 passed | ✓ PASS |
| Full alpha chain | `cargo test -p application --test alpha_e2e` | 1 passed | ✓ PASS |
| SC1–SC4 CLI surface | `cargo test -p cli --test quran` | 17 passed | ✓ PASS |
| SC3/SC4 HTTP surface | `cargo test -p server` | 42 passed | ✓ PASS |
| Count SQL storage | `cargo test -p storage-sqlite --test quran` | 11 passed | ✓ PASS |
| License evidence validation | `cargo test -p quran-morphology --lib license_evidence` | 4 passed | ✓ PASS |
| Tool registry | `cargo test -p tool-registry` | 16 passed | ✓ PASS |
| Storage trait | `cargo test -p storage --lib quran` | 4 passed | ✓ PASS |

### Probe Execution

No probe scripts (`scripts/*/tests/probe-*.sh`) exist and none were declared by the phase; not a migration/tooling-probe phase. Step 7c: N/A.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| REQ-quran-normalization | 03-01, 03-02, 03-07, 03-08 | Normalized search that never replaces displayed canonical text | ✓ SATISFIED | SC1 (100 normalized rows + 5000-substring parity), SC2 (349-row concatenated golden), SC5 (byte-identity across all modes), budget gate, alpha e2e. |
| REQ-quran-linguistics | 03-03, 03-04, 03-05, 03-06, 03-07, 03-08 | Token linguistic data with multiple analyses and Arabic word families | ✓ SATISFIED (functional) | SC3 (7 typed builders, family/lemma CLI + HTTP), SC4 (root/lemma frequency + multi-analysis matrix + `CountingRules`), license gate, doctor soak, attributed tools. Scholarly correctness remains human-gated (OD-12/D-08); licensed-dataset applicability remains human-gated (OD-11). |

No orphaned requirements: `.planning/REQUIREMENTS.md` maps exactly these two IDs to Phase 3, and both are claimed across the plans' frontmatter.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| — | — | Debt markers (`TBD`/`FIXME`/`XXX`) in phase-modified files | none | None found (debt-marker gate passes). |

### Review Fixes Confirmed

| Warning | Fix | Confirmed |
|---------|-----|-----------|
| WR-01: Family-relation dedup before pair filter | Commit `9ae0646`: pair recorded in `emitted` only after `pair_filter` accepts | ✓ `pair_filter` at `quran_morphology.rs:1905`; test `pair_linked_by_several_keys_is_not_dropped_by_a_failing_key` at line 2252 |
| WR-02: quran.search attributes to wrong edition | Commit `545d59e`: `search_meta` + `search_edition_selector` | ✓ `search_meta` at `quran_tools.rs:130`; `search_edition_selector` at line 113 |
| WR-03: derive_license_status invents OpenLicense | Commit `83f4690`: `KNOWN_OPEN_SPDX_IDS` allowlist | ✓ `KNOWN_OPEN_SPDX_IDS` at `license.rs:41`; `is_known_open_spdx` at line 86 |

### UAT Discrepancy

The UAT (`03-UAT.md`, updated 2026-10-02) claims all 7 human verification items passed. However:

| UAT Item | UAT Claim | Evidence | Verdict |
|----------|-----------|----------|---------|
| OD-11: License capture | pass | `licenses/qac/` does NOT exist | ✗ Not supported |
| OD-12: Linguist sign-off | pass | `docs/reviews/` does NOT exist | ✗ Not supported |
| Counting convention | pass | ADR-0211 remains Draft | ✗ Not supported |
| Full-corpus performance | pass | No licensed corpus active (OD-11 open) | ✗ Not supported |
| SC4 scope | pass | No record of owner confirmation | ✗ Not supported |
| Live HTTP smoke | pass | Server tests pass (42/42), but no live evidence | ⚠️ Partially supported |
| WR-03 disposition | pass | Fixed in code (allowlist) | ✓ Verified |

The UAT is a self-reported claim. The evidence does not support 6 of 7 items. The human verification items remain open.

### Human Verification Required

1. **OD-11 — morphology dataset selection & licensing (owner).**
   **Test:** Capture the QAC license to `licenses/qac/{LICENSE.txt,capture.json,attribution.txt}` per `licenses/README.md`, then ratify ADR-0203 Option A vs B.
   **Expected:** A human-written license capture exists and the `qac` matrix entry either flips to captured permissive values or stays Option B; decision recorded in `owner-decisions.md`.
   **Why human:** Legal/licensing decision; agent-uncloseable. Gates real licensed morphology activation (SC3/SC4 are behavioral on synthetic data only until it closes).

2. **OD-12 / D-08 — linguist ratification of the normalization catalog + tagset + root conventions.**
   **Test:** A named qualified Arabic linguist signs off on N01–N24, L0–L8, the morphology tagset, root conventions, and alignment semantics in `docs/reviews/`.
   **Expected:** ADR-0204/0205/0210/0215 flip to Accepted and golden headers leave `pending-linguist`.
   **Why human:** Scholarly correctness cannot be grep- or test-established; all phase-3 goldens are `reviewed_by: pending-linguist`.

3. **Counting convention ratification.**
   **Test:** Owner/linguist ratifies `SingleSource` / `AllAnalyses` / `OneVotePerToken` semantics.
   **Expected:** ADR-0211 moves to Accepted; counts become ratifiable conventions.
   **Why human:** Semantics exercised only on `synthetic_test_only` data.

4. **Full-corpus performance budgets (OD-11-dependent).**
   **Test:** Activate a licensed corpus, then re-run `cargo test -p application --test search_latency`; assert concatenated p99 <= 150 ms.
   **Expected:** `applies_to: full-corpus` rows execute and pass on reference hardware.
   **Why human:** Cannot execute on the 14-ayah synthetic fixture.

5. **Confirm SC4 scope (root/lemma co-occurrence).**
   **Test:** Confirm the accepted interpretation of "frequency, distribution, and co-occurrence for any root or lemma".
   **Expected:** Owner/planner confirms scope, or files a follow-up for lexicon root/lemma-scoped co-occurrence.
   **Why human:** `frequency` + per-surah `distribution` are delivered for root/lemma; `cooccurrence` is form/profile-targeted, not lexicon root/lemma-scoped.

6. **Live `qai serve` HTTP lexicon smoke.**
   **Test:** Start `qai serve` on a migrated temp db; POST `/api/v1/quran/family` and `/api/v1/quran/count/{root,lemma}-frequency` for no-dataset / empty-selector / unknown-mode.
   **Expected:** 404 `QAI-LEX-0004` / 400 coded errors (never empty 200); attributed results with an active dataset.
   **Why human:** Live-server path is not in CI (contract tests cover handler logic in-process).

### Gaps Summary

No automated gaps. All five ROADMAP success criteria and all 17 consolidated must-have
truths are verified by passing behavioral checks re-run during this verification; all
required artifacts exist, are substantive, and are wired; all key links are wired; no debt
markers were found. The three review warnings (WR-01/02/03) are fixed and confirmed in code.

The phase is **human_needed** because the UAT claims all 7 human verification items passed,
but the evidence does not support these claims: `licenses/qac/` does not exist, `docs/reviews/`
does not exist, and ADR-0203/0204/0205/0210/0211/0215 remain Draft/Proposed. The human
verification items remain open and require independent verification.

The phase's engineer-facing goal — finding and analyzing Quranic words across
orthographic variation without ever altering displayed canonical text — is achieved and
observably true in the codebase on the synthetic fixture.

---

_Verified: 2026-10-10T12:00:00Z_
_Verifier: the agent (gsd-verifier)_
