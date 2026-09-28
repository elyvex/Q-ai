---
phase: 03-quran-search-linguistics
verified: 2026-09-28T07:51:32Z
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
  - .planning/phases/03-quran-search-linguistics/03-SECURITY.md
  - .planning/phases/03-quran-search-linguistics/03-VALIDATION.md
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
  - crates/cli/src/quran.rs
  - crates/quran-morphology/src/family.rs
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
covered_digest: "v1:sha256:fc01d3105d40e27cba49a32e6d7676c37c88bc3d24118ac91a36aaa28c17c3fd"
behavior_unverified: 0
overrides_applied: 0
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
  - test: "Review the unfixed license-status derivation (03-REVIEW.md WR-03): `derive_license_status` maps any non-empty `spdx_id` to `OpenLicense`."
    expected: "Either accept as documented risk or require an explicit `--license-status` / allowlist so a license status is never inferred from an arbitrary string (plan 03-06 prohibition T-03-23)."
    why_human: "Declared prohibition vs implementation: the gate still requires `redistribution_allowed: true` (fail-closed core holds), but the status label is inferred. Disposition is a human call."
---

# Phase 3: Quran Search & Linguistics Verification Report

**Phase Goal:** Users can find and analyze Quranic words across orthographic variation without ever seeing altered display text.
**Verified:** 2026-09-28T07:51:32Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

All five ROADMAP success criteria and the plan-level must-haves were checked goal-backward
against the live codebase and re-executed. Every must-have resolves to **✓ VERIFIED** with a
passing behavioral check. Two things keep this phase out of `passed`: (a) both phase
requirements carry human-only scholarly/licensing ratification gates (OD-11 / OD-12 / D-08)
that no agent may close, and (b) one declared must-NOT (license-status inference) is
unfixed at warning level and needs an owner/reviewer disposition. Neither is an automated
failure; there are **no failed truths, no missing/stub artifacts, and no broken key links**.

### Observable Truths

| #   | Truth   | Status     | Evidence       |
| --- | ------- | ---------- | -------------- |
| 1 | **SC1** — a diacritic-free Arabic query returns canonical-location hits through the normalized path, each with an ordered `NormalizationTrace` + canonical span. | ✓ VERIFIED | `fixtures/quran/search/queries.jsonl` has 100 `search_normalized` rows (diacritic-free inputs, `profile: L3.diacritics`); `search_goldens` (400 rows, `all_400_search_goldens_pass`) + `search_parity` (`family_total_and_stable_over_5000_substrings`, 5000-substring gate) + `alpha_smoke` green. |
| 2 | **SC2** — a spaceless (concatenated) phrase returns canonical-location hits with an explainable segmentation tiling the query and pinned refs; cross-ayah matches split per overlapped ayah. | ✓ VERIFIED | `fixtures/quran/search/concatenated.jsonl` = 349 rows (independent Python oracle); `search_goldens -- concatenated` asserts references, `must_not_contain`, tiling, boundary + ayah-level-win dedup; CLI `search_s2.trycmd` shows `سانبل`→3:3 and cross-ayah `بلشا`→3:1+3:2 (never merged). |
| 3 | **SC3** — a user can inspect a word's lemma, root, multi-analysis morphology, and word family (typed + explained + attributed). | ✓ VERIFIED | `qai quran family`/`lemma` CLI (`family_s2.trycmd`, typed-unavailable without dataset) and `POST /api/v1/quran/family` (`lexicon_family_route_returns_attributed_relations`); `word_family` + 7 typed builders in `quran_morphology.rs`; `family_goldens` (154 curated families, all 7 kinds) + `morphology_import` green; `alpha_e2e` asserts no `winner/is_correct/is_primary/selected` in analyses. |
| 4 | **SC4** — a user can view frequency, per-surah distribution, and co-occurrence with a complete `CountingRules` block; root/lemma frequency over an active lexicon. | ✓ VERIFIED | `root_frequency`/`lemma_frequency` exact SQL over the active dataset (`root_lemma_frequency` + `multi_analysis_modes` green); `frequency`/`distribution`/`cooccurrence` reachable via `qai quran count …` and exercised in `alpha_e2e`; every report carries `CountingRules` (`frequency_exact_and_deterministic`, `distribution_and_numeric_report`, `cooccurrence_and_collocation`). See human item on co-occurrence target scope. |
| 5 | **SC5** — displayed canonical text is never modified by normalization in any result. | ✓ VERIFIED | `canonical_display_identity` (6 tests): per-hit displayed quotation is byte-identical to the reader-resolved canonical row across exact/normalized/phrase/concatenated/regex, spans slice back identically, and canonical hashes are unchanged across all five searches (MV-018 post-check in forms rebuild). |
| 6 | A hit cannot exist without verified quotation + span + trace; empty/whitespace query is rejected with a typed usage error (never all-match/panic). | ✓ VERIFIED | Validating `SearchHit` constructor (`quran-search/src/hit.rs`); `alpha_smoke` `empty_and_whitespace_queries_are_rejected_at_the_cli_boundary` green; `serve` snapshot `Unknown match modes fail closed` (exit 2). |
| 7 | The concatenated golden set (>= 120) is produced by an oracle independent of the Rust stack and runs green with `reviewed_by: pending-linguist`. | ✓ VERIFIED | `scripts/gen_concatenated_goldens.py` (ordered L6 rule list, 3-ayah windows); 349 rows in the fixture; `search_goldens -- concatenated` green; header stays `pending-linguist`. |
| 8 | When no dataset is active, root/lemma frequency returns typed `CountingError::UnavailableDataset` (`QAI-CNT-0005`) — never an empty report/guessed data. | ✓ VERIFIED | `quran_counting.rs` `UnavailableDataset` (+ code 5); `counting.rs`; CLI `counting_graph_s2.trycmd` typed-unavailable snapshots green. |
| 9 | Every numeric output carries a complete `CountingRules`; changing the multi-analysis mode visibly changes count + rules deterministically. | ✓ VERIFIED | `counting.rs` `multi_analysis_modes`; `alpha_e2e` `assert_rules_complete` on every numeric report; `CountingRules::canonical_json` field order is the fixed checksum input. |
| 10 | Word-family relations are typed for every declared kind, each with a non-empty explanation + provenance; `affix_search` fails closed with a typed error rather than a silent empty. | ✓ VERIFIED | 7 builders in `quran_morphology.rs`; `family_goldens` (154 rows, 7 kinds); `morphology_import -- family_relations_explained` + `affix_search_typed_unavailable` green; `QAI-MORPH-0004`. |
| 11 | CLI `family`/`lemma` and the versioned HTTP `family` + `count/{root,lemma}-frequency` routes are reachable, typed, and attributed — never an empty 200. | ✓ VERIFIED | `QuranAction::Family`/`Lemma` dispatch; `quran_lexicon_api::LexiconApiService` + `AppState.lexicon` wiring; routes at `api.rs:1467-1469`; `lexicon_family_route_never_returns_an_empty_envelope`, `lexicon_count_routes_are_typed_when_the_dataset_is_unavailable`, `lexicon_count_routes_return_the_counting_rules_block` green. |
| 12 | A morphology dataset cannot reach `state='active'` without permissive license evidence + mandatory capture fields (fail closed). | ✓ VERIFIED | Gate 6 in `activate_morphology` (after alignment, before promotion) consults pure `quran-morphology/src/license.rs`; `license_gate_rejects` / `license_gate_passes` + `license_evidence_*` unit tests green; `fixtures/quran/morphology/license-matrix.json` valid with capture fields. See human item re WR-03. |
| 13 | `qai doctor --indexes` reports 19 stable checks green after a rebuild+activate+search soak, turns red (QAI-IDX-0101) on injected drift, and stays read-only. | ✓ VERIFIED | `doctor_indexes` (7 tests) `index_checks_are_nineteen_with_stable_ids`, `index_checks_go_green_on_built_index`, `injected_drift_reports_stale_index_code`, `doctor_indexes_is_read_only`, `doctor_indexes_soak_green_on_activated_morphology_and_trigram` green; `cmd_doctor_indexes` wired in `cli/src/doctor.rs`. |
| 14 | A machine-readable budget artifact codifies concatenated p99 <= 150 ms and the fixture harness gates against the fixture-appropriate bound (full-corpus rows flagged OD-11-dependent). | ✓ VERIFIED | `fixtures/quran/performance/budgets.json`; `search_latency` `budget_artifact_codifies_adr_0207_and_fixture_bound` + `search_latency_within_fixture_bounds` green. |
| 15 | The tool registry exposes search/root/lemma/morphology/family with provenance/attribution and `normalization_rules` populated from the hit trace. | ✓ VERIFIED | `tool-registry/src/lib.rs` registers all five names + `ToolResult` envelope; `quran_tools.rs` backends set `analysis_sources` + `normalization_rules`; `quran_tools` (9 tests incl. `search_tool_returns_attributed_envelope`, `lexicon_tools_are_attributed_with_an_active_dataset`, `lexicon_tools_fail_closed_without_a_dataset`) + `tool-registry` (6 tests) green. |
| 16 | OD-11 and OD-12/D-08 are recorded as explicit BLOCKED items with closing steps; no agent closed them and no ADR/golden was marked Accepted. | ✓ VERIFIED | `docs/05-followups/phase-03-owner-gates.md` (OD-11, OD-12, closing commands, `licenses/qac/`); `docs/05-followups/decisions-needed.md` remains the source of truth; ADR-0203/0204/0205/0209/0210/0211/0215 all still Draft. |
| 17 | The full D-03 alpha runs green end-to-end on the synthetic fixture: normalize → forms → index → all five search modes → root/lemma → morphology → family → frequency/distribution/co-occurrence. | ✓ VERIFIED | `alpha_e2e` `alpha_end_to_end_synthetic` green: all five search modes with trace+span+byte-identical quotation, morphology with no elected winner, root/lemma search, typed+explained family, and every numeric report carrying `CountingRules`. |

**Score:** 17/17 truths verified (0 present, behavior-unverified)

### Required Artifacts

| Artifact | Expected    | Status | Details |
| -------- | ----------- | ------ | ------- |
| `crates/application/tests/alpha_smoke.rs` | End-to-end tracer (migrate→import→activate→forms→index→normalized search) | ✓ VERIFIED | 368 lines; 2 tests green; not a stub. |
| `crates/application/tests/canonical_display_identity.rs` | SC5 per-hit byte-identity across all modes | ✓ VERIFIED | 481 lines; 6 tests green; resolves canonical through the reader and compares bytes (non-tautological). |
| `crates/quran-normalization/src/rules/mod.rs` | Corrected rule-catalog doc (N18–N22; N23/N24 reserved) | ✓ VERIFIED | Doc-only change; clippy/fmt clean per SUMMARY. |
| `scripts/gen_concatenated_goldens.py` | Independent Python oracle for SC2 | ✓ VERIFIED | 478 lines; deterministic regeneration. |
| `fixtures/quran/search/concatenated.jsonl` | >= 120-case concatenated golden set | ✓ VERIFIED | 349 rows (349 `search_concatenated`), 16 cross-ayah, 10 Persian-codepoint; `pending-linguist`. |
| `crates/application/tests/search_goldens.rs` | Loader + `search_concatenated` arm with tiling assertions | ✓ VERIFIED | 567 lines; 3 tests green (400 + concatenated + normalized-recall). |
| `crates/storage/src/quran.rs` + `crates/storage-sqlite/src/quran.rs` | Root/lemma count trait + SQLite impls (bound params) | ✓ VERIFIED | Present; `storage` (4) + `storage-sqlite --test quran` (11) green. |
| `crates/application/src/quran_counting.rs` | Real `root_frequency`/`lemma_frequency`, selectable modes, `CountingRules` | ✓ VERIFIED | 999 lines; `pub async fn root_frequency`/`lemma_frequency`; `counting` (9) green. |
| `crates/application/tests/counting.rs` | root/lemma frequency + multi-analysis + QAI-CNT-0005 fallback | ✓ VERIFIED | 476 lines; 9 tests green. |
| `crates/application/src/quran_morphology.rs` | All typed family builders + typed affix error + license gate 6 | ✓ VERIFIED | 2220 lines; 7 builders; `morphology_import` (15) green. |
| `fixtures/quran/lexicon/families/curated.jsonl` | >= 120-family curated golden set | ✓ VERIFIED | 155 lines (154 families); 7 typed kinds; `synthetic_test_only`. |
| `crates/application/tests/family_goldens.rs` | Curated family runner | ✓ VERIFIED | 346 lines; `all_curated_family_goldens_pass` green. |
| `crates/application/src/quran_lexicon_api.rs` | `LexiconBackend` + `LexiconApiService` | ✓ VERIFIED | 293 lines; registered in `application/src/lib.rs`. |
| `crates/server/src/api.rs` | family + root/lemma-frequency routes + `AppState.lexicon` | ✓ VERIFIED | Routes at 1467-1469; `server` 22 tests green. |
| `crates/quran-morphology/src/license.rs` | Pure license-evidence validation | ✓ VERIFIED | 222 lines; 4 unit tests green. |
| `fixtures/quran/morphology/license-matrix.json` | Machine-readable license matrix | ✓ VERIFIED | Valid JSON with capture fields; `qac` pending (fail-closed). |
| `crates/application/tests/doctor_indexes.rs` | 19-check soak + drift + read-only | ✓ VERIFIED | 622 lines; 7 tests green. |
| `fixtures/quran/performance/budgets.json` | Budget table (concatenated p99 <= 150 ms) | ✓ VERIFIED | 133 lines; codifies ADR-0207 + plan 17.1 table. |
| `crates/tool-registry/src/lib.rs` + `crates/application/src/quran_tools.rs` | Attributed tool surface | ✓ VERIFIED | 5 tools registered; attribution + normalization_rules populated. |
| `crates/application/tests/alpha_e2e.rs` | Full-chain alpha smoke | ✓ VERIFIED | 587 lines; `alpha_end_to_end_synthetic` green. |
| `docs/05-followups/phase-03-owner-gates.md` + `docs/05-followups/phase-03-deferrals.md` | BLOCKED owner gates + deferral ledger | ✓ VERIFIED | Both exist; OD-11/OD-12 recorded; `hapax_search` deferral recorded. |

### Key Link Verification

| From | To  | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `alpha_smoke.rs` / `alpha_e2e.rs` | `crates/quran-search/src/hit.rs` validating constructor | constructing `SearchHit` | WIRED | A hit cannot exist without trace + span + quotation (`search_hits_require_*` constructor). |
| `search_goldens.rs` dispatcher | `search_concatenated` service | direct call passing per-row `allow_cross_ayah`/`max_ayah_span` | WIRED | `all_concatenated_search_goldens_pass` green. |
| `cmd_family` / `qai quran family` | `word_family` service | `cmd_family` dispatch | WIRED | `family_s2.trycmd` reaches handler (typed-unavailable without dataset). |
| HTTP `lexicon_family_handler` | `LexiconApiService` | shared `lexicon_error_status` mapping | WIRED | `server` contract tests green; `AppState.lexicon` wired in `cli/src/lib.rs`. |
| `activate_morphology` gate 6 | `quran-morphology/src/license.rs` | `LicenseEvidence::from_status_and_json` | WIRED | Gate after alignment, before promotion; reject/pass tests green. |
| forms rebuild | MV-018 canonical-unchanged post-check | `rebuild_forms` | WIRED | MV-018 pass line in CLI snapshots; canonical hashes stable across searches. |
| tool registry | `quran_tools.rs` backends | `ReaderToolBackend`-style registration | WIRED | `quran_tools` + `tool-registry` tests green; `normalization_rules` from trace. |
| `doctor --indexes` CLI | `run_index_checks` (19 ids) | `cmd_doctor_indexes` | WIRED | `doctor_indexes` (7) green; read-only asserted. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `root_frequency`/`lemma_frequency` | count / by_surah | exact SQL over `quran_token_analyses` joins to `quran_roots`/`quran_lemmas` | Yes (bound params, no FTS) | ✓ FLOWING |
| `word_family` | members | `list_family_relations` over derived family rows | Yes (typed, explained) | ✓ FLOWING |
| `frequency`/`distribution`/`cooccurrence` | counts / windows | `list_all_token_forms` over derived form tables | Yes | ✓ FLOWING |
| search hits (all 5 modes) | quotation / span / trace | FTS5 index + skeleton/trigram, verified against canonical | Yes | ✓ FLOWING |
| license gate | evidence | operator-supplied dataset row (`license_status`/`license_json`) | Yes | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| SC1/SC5 tracer + empty-query guard | `cargo test -p application --test alpha_smoke` | 2 passed | ✓ PASS |
| SC5 byte-identity across 5 modes | `cargo test -p application --test canonical_display_identity` | 6 passed | ✓ PASS |
| SC1 400-row golden + SC2 concatenated | `cargo test -p application --test search_goldens` | 3 passed | ✓ PASS |
| 5,000-substring normalization parity | `cargo test -p quran-search --test search_parity` | 3 passed | ✓ PASS |
| SC4 counting + multi-analysis | `cargo test -p application --test counting` | 9 passed | ✓ PASS |
| SC3 family + license gate | `cargo test -p application --test morphology_import` | 15 passed | ✓ PASS |
| SC3 curated family golden | `cargo test -p application --test family_goldens` | 1 passed | ✓ PASS |
| Doctor 19-check soak | `cargo test -p application --test doctor_indexes` | 7 passed | ✓ PASS |
| Performance budget gate | `cargo test -p application --test search_latency` | 2 passed | ✓ PASS |
| Attributed tool surface | `cargo test -p application --test quran_tools` + `cargo test -p tool-registry` | 9 + 6 passed | ✓ PASS |
| Full alpha chain | `cargo test -p application --test alpha_e2e` | 1 passed | ✓ PASS |
| SC1–SC4 CLI surface | `cargo test -p cli --test quran` | 14 passed | ✓ PASS |
| SC3/SC4 HTTP surface | `cargo test -p server` | 22 passed | ✓ PASS |
| Count SQL storage | `cargo test -p storage-sqlite --test quran` + `cargo test -p storage --lib quran` | 11 + 4 passed | ✓ PASS |
| License evidence validation | `cargo test -p quran-morphology --lib license_evidence` | 4 passed | ✓ PASS |

### Probe Execution

No probe scripts (`scripts/*/tests/probe-*.sh`) exist and none were declared by the phase; not a migration/tooling-probe phase. Step 7c: N/A.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| REQ-quran-normalization | 03-01, 03-02, 03-07, 03-08 | Normalized search that never replaces displayed canonical text; indexed search forms; user-controlled normalization; concatenated-word search (PRD §8) | ✓ SATISFIED (automatable) | SC1 (100 normalized rows + 5000-substring parity), SC2 (349-row concatenated golden + CLI snapshot), SC5 (byte-identity across all modes), budget gate, alpha e2e. |
| REQ-quran-linguistics | 03-03, 03-04, 03-05, 03-06, 03-07, 03-08 | Token linguistic data with multiple analyses and Arabic word families (PRD §9) | ✓ SATISFIED (functional) | SC3 (7 typed builders, family/lemma CLI + HTTP), SC4 (root/lemma frequency + multi-analysis matrix + `CountingRules`), license gate, doctor soak, attributed tools. Scholarly correctness remains human-gated (OD-12/D-08); licensed-dataset applicability remains human-gated (OD-11). |

No orphaned requirements: `.planning/REQUIREMENTS.md` maps exactly these two IDs to Phase 3, and both are claimed across the plans' frontmatter (4 + 6 declarations). `REQUIREMENTS.md` currently marks both `Complete`.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| — | — | Debt markers (`TBD`/`FIXME`/`XXX`) in phase-modified files | none | None found (debt-marker gate passes). |
| `crates/application/src/quran_morphology.rs` | 1900-1910 | Family-relation dedup recorded before `pair_filter` (03-REVIEW.md WR-01) | ⚠️ Warning | Can drop a valid relation when a token pair shares several keys; latent (golden passes), no production caller beyond tests. |
| `crates/application/src/quran_tools.rs` | 184-215 | `quran.search` attributes results to the *active* edition even when another edition is searched (WR-02) | ⚠️ Warning | Internally inconsistent envelope for a non-active `edition` param; defeats traceability for that selector. |
| `crates/application/src/quran_cli.rs` | 2788-2801 | `derive_license_status` maps any non-empty `spdx_id` to `OpenLicense` (WR-03) | ⚠️ Warning | Infers a license status from an arbitrary string — deviates from plan 03-06 prohibition T-03-23; the redistribution requirement still fail-closes. Needs human disposition (see human items). |
| `crates/application/src/quran_lexicon_api.rs`, `quran_cli.rs`, `tool-registry/src/lib.rs` | 41-48 / 3154-3158 / 444-450 | Selectors validated after `trim()` but forwarded untrimmed (03-REVIEW.md IN-02) | ⚠️ Warning | A whitespace-padded selector can return a success with an empty relation list (not the "no active dataset" empty-200 the tests rule out). |

All four are already recorded in `03-REVIEW.md` (0 critical, 3 warning, 4 info) and remain
unfixed in the committed tree. None is a stub, a missing artifact, or a broken key link.

### Declared Prohibitions Disposition

The plans declare `must_haves.prohibitions` (all tagged `flagged-unverified`). Directly
enforced ones are verified by the tests below; the remaining judgment-tier prohibitions are
routed to human review (never silently green):

- Enforced and **verified**: no displayed-text replacement (SC5 byte-identity), no `is_correct`/`winner`/`selected` column or merged analysis (alpha_e2e banned-substring assertion), `CountingRules` on every numeric output (`assert_rules_complete`), doctor never mutates (`doctor_indexes_is_read_only`), no activation without redistribution grant (`license_gate_rejects`), canonical never re-tokenized (concatenated matches over derived skeletons).
- **Flagged for human review**: "no owner gate marked resolved" (OD-11/OD-12 remain 🔴 — verified as still open), "synthetic data never presented as ground truth" (goldens stay `pending-linguist`/`synthetic_test_only`), and "no importer may infer a license status" (violated by WR-03 — see human items).

### External Test-Tree Notes (not phase-3 regressions)

- `cargo test -p cli --test doctor_json` → `audit_verify_rejects_corrupt_chain_without_modifying_database` FAILS (`tampered_sequences` Null vs 2). The assertion concerns the audit-chain tamper detector (`crates/application/src/audit_bridge.rs`, `db.rs`, `cli/src/doctor.rs`) and `doctor_json.rs` was last touched by a Phase-1 audit-test commit; no phase-3 plan modified those files. Characterized as external/pre-existing, **not** a phase-3 must-have.
- `serve_hosts_the_worker_and_shuts_down_joined` passes standalone (confirmed here, 4.71 s); the full-suite flake is load-induced, not a phase-3 regression.

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

7. **Disposition the license-status inference (WR-03).**
   **Test:** Decide whether `derive_license_status` inferring `OpenLicense` from any non-empty `spdx_id` is acceptable or must require an explicit `--license-status` / allowlist.
   **Expected:** Accepted as documented risk, or fixed so no status is inferred (plan 03-06 prohibition T-03-23).
   **Why human:** Flagged must-NOT violation at warning level; the fail-closed redistribution requirement still holds.

### Gaps Summary

No automated gaps. All five ROADMAP success criteria and all 17 consolidated must-have
truths are verified by passing behavioral checks re-run during this verification; all
required artifacts exist, are substantive, and are wired; all key links are wired; no debt
markers were found. The phase is **human_needed** solely because both phase requirements
carry human-only ratification gates (OD-11, OD-12/D-08) and their counting/scope corollaries,
plus one warning-level declared-prohibition deviation (WR-03) that needs an owner/reviewer
call. The phase's engineer-facing goal — finding and analyzing Quranic words across
orthographic variation without ever altering displayed canonical text — is achieved and
observably true in the codebase on the synthetic fixture.

---

_Verified: 2026-09-28T07:51:32Z_
_Verifier: the agent (gsd-verifier)_
