---
phase: 03-quran-search-linguistics
plan: 03
subsystem: quran-linguistics
tags: [rust, sqlite, quran-lexicon, counting-rules, multi-analysis, root-frequency, lemma-frequency, cli, tdd]

# Dependency graph
requires:
  - phase: 02-canonical-quran-core
    provides: quran_datasets/quran_roots/quran_lemmas/quran_token_analyses (migration 0017), morphology import + approval-gated activation, analyses_for_root/analyses_for_lemma, CountingRules + QAI-CNT-* codes
  - phase: 03-quran-search-linguistics
    provides: 03-01 alpha-smoke tracer and the SC5 per-hit canonical pin (wave 1)
provides:
  - "Six lexicon aggregation trait methods (count_analyses_for_root/lemma (+by_surah), count_distinct_tokens_for_root/lemma) with StorageUnavailable defaults and exact, fully-bound SQLite impls"
  - "Real root_frequency/lemma_frequency over the active lexicon with a complete CountingRules block (active <slug>@<version> + requested MultiAnalysisHandling) and a summing per-surah breakdown"
  - "rules_for_with(profile, version, window, datasets, mode) — datasets and multi-analysis handling caller-selectable; rules_for delegates with [\"stored-forms\"]/SingleSource"
  - "CLI: qai quran count root-frequency / lemma-frequency (+ --mode) and the qai quran freq surface-form alias (D-10)"
  - "Three-distinct-count multi-analysis mode matrix asserted on the synthetic lexicon (SingleSource / AllAnalyses / OneVotePerToken)"
affects: [03-04, 03-05, 03-06, 03-07, 03-08, phase-04-graph, phase-05-result-contract]

# Actuals (#2632) — same estimateTokens scale as the plan's estimate (chars/4 over the realized diff)
actuals:
  tokens: 10569
  tasks: 3
  commits: 4
plan_head_before: 0afc4d1bd273e8af4cf240b39a969e02247dedc6

tech-stack:
  added: []
  patterns:
    - "Exact lexicon aggregation in SQL: COUNT(*) over the analysis join; COUNT(DISTINCT (surah,ayah,token_position)) via a grouped subquery (SQLite rejects the row-value COUNT(DISTINCT (a,b,c)) form)"
    - "Mode-relative counting: the requested MultiAnalysisHandling selects which rows are one occurrence AND is recorded verbatim in CountingRules; identical rules reproduce identical count + checksum"
    - "Suppression is never silent: SingleSource records excluded ambiguous tokens in CountingRules.exclusions (ADR-0209)"
    - "Typed fail-closed surface: no active lexicon ⇒ CountingError::UnavailableDataset (QAI-CNT-0005), never an empty report and never guessed data"

key-files:
  created:
    - .planning/phases/03-quran-search-linguistics/03-03-RED-EVIDENCE.json
  modified:
    - crates/storage/src/quran.rs
    - crates/storage-sqlite/src/quran.rs
    - crates/application/src/quran_counting.rs
    - crates/application/tests/counting.rs
    - crates/application/src/quran_cli.rs
    - crates/cli/src/quran.rs
    - crates/cli/tests/quran/counting_graph_s2.trycmd
    - .planning/phases/03-quran-search-linguistics/deferred-items.md

key-decisions:
  - "Multi-analysis counting semantics are code-implemented but NOT linguist-ratified (OD-12 BLOCKED): SingleSource = a token counted once only when exactly one of its analyses matches; AllAnalyses = one vote per analysis row; OneVotePerToken = one vote per matching token. Asserted behaviourally on the synthetic lexicon only; competing analyses never elect an authoritative winner (I11/ADR-0209)."
  - "SingleSource records the excluded ambiguous-token count in CountingRules.exclusions so suppression is reported, not silent (ADR-0209)."
  - "Lexicon aggregation is exact SQL with typed .bind() parameters only; the distinct-token count uses a GROUP BY subquery because SQLite rejects the plan's COUNT(DISTINCT (a,b,c)) row-value form as a misuse."
  - "root/lemma targets are dataset keys (normalized forms as stored) and are not re-normalized through the profile; the profile is recorded in the rules block as the stated counting convention."
  - "The qai quran freq alias refuses a non-default --mode with a typed usage error rather than silently ignoring it, because surface-form frequency is single-source only."

patterns-established:
  - "Mode-selectable counting: function takes (target, profile, mode); the mode both drives the SQL aggregation and is embedded in the rules block"
  - "Storage answers counts without loading rows (COUNT(*) / grouped subquery); only the token-dedup and single-source modes iterate matching analysis rows for their breakdown"
  - "TDD in Rust: scaffold the (profile, mode) signatures in the RED commit so the target tests compile and fail on assertions, then implement the bodies in GREEN"

requirements-completed: [REQ-quran-linguistics]

# Coverage metadata (#1602)
coverage:
  - id: D1
    description: "Six lexicon aggregation methods on the quran storage trait (count_analyses_for_root/lemma, *_by_surah, count_distinct_tokens_for_root/lemma) with StorageUnavailable defaults and SQLite impls that bind every parameter (no string-concatenated SQL); exact COUNT(*) and grouped-subquery COUNT(DISTINCT token) joins over quran_token_analyses → quran_roots/quran_lemmas."
    requirement: REQ-quran-linguistics
    verification:
      - kind: integration
        ref: "cargo test -p storage-sqlite --test quran"
        status: pass
      - kind: unit
        ref: "cargo test -p storage --lib quran"
        status: pass
    human_judgment: false
  - id: D2
    description: "Real root_frequency/lemma_frequency over the active lexicon: resolve <slug>@<version>, aggregate in one db.write() UoW with rollback(), and assemble FrequencyReport { target, rules, count, by_surah, checksum }; no-dataset path stays CountingError::UnavailableDataset (QAI-CNT-0005)."
    requirement: REQ-quran-linguistics
    verification:
      - kind: integration
        ref: "cargo test -p application --test counting -- root_lemma_frequency multi_analysis_modes --nocapture"
        status: pass
      - kind: integration
        ref: "cargo test -p application --test counting"
        status: pass
    human_judgment: false
  - id: D3
    description: "Multi-analysis matrix: SingleSource / AllAnalyses / OneVotePerToken produce three visibly different counts (1 / 127 / 64 on the mixed synthetic lexicon), each reports its own handling mode, the by-surah breakdown sums to the total, and identical rules reproduce identical count + checksum."
    requirement: REQ-quran-linguistics
    verification:
      - kind: integration
        ref: "cargo test -p application --test counting -- multi_analysis_modes"
        status: pass
    human_judgment: true
    rationale: "The mode semantics satisfy the plan's behavioural contract but are an unresolved, flagged assumption: the counting convention is not linguist-ratified (OD-12 BLOCKED, ADR-0211 Draft). A human/linguist must ratify what 'one occurrence' means before these counts are treated as scholarly ground truth."
  - id: D4
    description: "CLI surface (D-10/D-13): qai quran count root-frequency / lemma-frequency with --mode, and the qai quran freq alias; no active lexicon yields the typed UnavailableDataset as a policy/not-found error (exit 5) and an unknown mode is a usage error (exit 2)."
    requirement: REQ-quran-linguistics
    verification:
      - kind: integration
        ref: "cargo test -p cli --test quran"
        status: pass
      - kind: other
        ref: "cargo run -q -p xtask -- migrate-check && cargo run -q -p xtask -- arch-check"
        status: pass
    human_judgment: false

# Metrics
duration: 23 min
completed: 2026-09-28
status: complete
---

# Phase 3 Plan 03: Root/Lemma Frequency + Selectable Multi-Analysis Summary

**`qai quran count root-frequency`/`lemma-frequency` now compute exact SQL counts over the active lexicon — with the active `<slug>@<version>` and a caller-selectable multi-analysis mode in every rules block — closing SC4's "for any root or lemma" gap and making `AllAnalyses`/`OneVotePerToken` real instead of dormant.**

## Performance

- **Duration:** 23 min
- **Started:** 2026-09-28T01:44:12Z
- **Completed:** 2026-09-28T02:06:52Z
- **Tasks:** 3/3
- **Files modified:** 8 (7 source/test/planning edits + 1 RED-evidence record created)

## Accomplishments

- **SC4/G-01 closed:** `root_frequency`/`lemma_frequency` no longer hard-stub to `UnavailableDataset`. They resolve the active dataset, aggregate exact SQL counts (never FTS frequencies), and return a `FrequencyReport` whose `rules.datasets` names the active `<slug>@<version>`.
- **G-07 closed:** `AllAnalyses` and `OneVotePerToken` are now produced. `rules_for_with(profile, version, window, datasets, mode)` lets the caller choose both the dataset list and the multi-analysis handling; `rules_for` delegates with the legacy `["stored-forms"]`/`SingleSource` defaults.
- **Mode relativity proven:** on a mixed synthetic lexicon (one single-analysis token + 63 two-analysis tokens sharing one root), the three modes give `SingleSource = 1`, `OneVotePerToken = 64`, `AllAnalyses = 127` — three visibly different counts, each with its own `multi_analysis_handling`, a summing per-surah breakdown, and deterministic checksums.
- **Typed fail-closed preserved:** with no active lexicon both functions return `CountingError::UnavailableDataset` / `QAI-CNT-0005` (asserted in `morphology_gated_targets_unavailable`), and the CLI surfaces it as exit 5 — never an empty report, never guessed data.
- **CLI surface (D-10/D-13):** `qai quran count root-frequency <root> --mode …`, `qai quran count lemma-frequency <lemma> --mode …`, and the `qai quran freq <text>` surface-form alias, all snapshot-tested.
- **Storage port extended:** six aggregation methods with `StorageUnavailable` defaults on the `quran` trait and exact, fully-bound SQLite implementations.

## Task Commits

Each task was committed atomically (Task 2 is the TDD task: RED → GREEN):

1. **Task 1: Storage aggregation methods for root/lemma analysis counts (G-01)** - `1fd0b14` (feat)
2. **Task 2 (RED): add failing root/lemma frequency + multi-analysis mode tests** - `8ef0201` (test)
3. **Task 2 (GREEN): implement root/lemma frequency + selectable modes** - `8502d59` (feat)
4. **Task 3: CLI surface for root/lemma frequency + freq alias** - `6d79a65` (feat)

**Plan metadata:** (this commit) (docs: complete plan)

_Note: Task 2 produced the mandated RED commit (`test(03-03): …`) before the GREEN commit (`feat(03-03): …`); no REFACTOR commit was needed._

## TDD Gate Compliance

| Gate | Commit | Verdict |
|------|--------|---------|
| RED | `8ef0201` `test(03-03): add failing root/lemma frequency and multi-analysis mode tests` | `RED_EVIDENCE_OK` — both target tests failed on assertions (`assert!(root.is_ok(), …)` / `assert!(single.is_ok(), …)`), exit 101; record at `.planning/phases/03-quran-search-linguistics/03-03-RED-EVIDENCE.json` |
| GREEN | `8502d59` `feat(03-03): implement root/lemma frequency with selectable multi-analysis modes` | pass — `cargo test -p application --test counting` 9/9 |
| REFACTOR | — | not needed (no cleanup applied) |

## Files Created/Modified

- `crates/storage/src/quran.rs` — six lexicon aggregation trait methods with `StorageUnavailable` default bodies.
- `crates/storage-sqlite/src/quran.rs` — the six SQLite impls (typed binds; analysis counts and grouped-subquery distinct-token counts over the root/lemma joins).
- `crates/application/src/quran_counting.rs` — real `root_frequency`/`lemma_frequency`, `rules_for_with` (+`resolve_profile`, `active_dataset_id`, `lexicon_target`, mode aggregation helpers), updated `MultiAnalysisHandling` docs.
- `crates/application/tests/counting.rs` — `root_lemma_frequency`, `multi_analysis_modes`, the mixed-lexicon harness, and the updated 4-arg no-dataset calls.
- `crates/application/src/quran_cli.rs` — `cmd_count_root_frequency`, `cmd_count_lemma_frequency`, `cmd_freq`, and the `--mode` parser.
- `crates/cli/src/quran.rs` — `CountAction::RootFrequency`/`LemmaFrequency`, `QuranAction::Freq`, and dispatch wiring.
- `crates/cli/tests/quran/counting_graph_s2.trycmd` — typed no-dataset snapshots for root/lemma frequency, the `freq` alias, and the unknown-mode usage error.
- `.planning/phases/03-quran-search-linguistics/03-03-RED-EVIDENCE.json` — the verified RED evidence record.
- `.planning/phases/03-quran-search-linguistics/deferred-items.md` — pre-existing `rustfmt` drift logged (out of scope).

## Decisions Made

- **Mode semantics (flagged assumption):** `SingleSource` counts a token once only when exactly one of its analyses matches; `AllAnalyses` counts every matching analysis row; `OneVotePerToken` counts every matching token once. This makes the three modes pairwise distinct on the plan's prescribed "two competing analyses" shape, satisfying the must_have contract. It is code-implemented but **not linguist-ratified** (OD-12 BLOCKED / ADR-0211 Draft); the tests assert it on the synthetic lexicon only.
- **Suppression is reported:** `SingleSource` puts the excluded ambiguous-token count in `CountingRules.exclusions` (ADR-0209: suppression never silent).
- **SQLite row-value workaround:** `COUNT(DISTINCT (a.surah, a.ayah, a.token_position))` is rejected by SQLite ("row value misused"), so the exact equivalent `COUNT(*)` over a `GROUP BY` subquery is used.
- **Targets are dataset keys:** root/lemma inputs are trimmed and used as stored normalized forms (matching `root_search`/`lemma_search`); the profile is recorded in the rules block rather than re-normalizing the key.
- **`freq` alias:** refuses a non-default `--mode` with a typed usage error instead of silently ignoring mode, since surface-form frequency is single-source.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] The plan's CLI snapshot target `counting_graph.trycmd` does not exist**
- **Found during:** Task 3 (CLI surface)
- **Issue:** `files_modified` named `crates/cli/tests/quran/counting_graph.trycmd`, but the real (host-backed, split) counting/graph snapshots are `counting_graph_s1.trycmd` / `counting_graph_s2.trycmd`.
- **Fix:** Added the root/lemma-frequency, `freq`, and unknown-mode blocks to `counting_graph_s2.trycmd` (the segment where the edition is active and morphology reads run), mirroring the existing `? 5` styled block. Same correction pattern 03-02 recorded for `search.trycmd` → `search_s2.trycmd`.
- **Files modified:** `crates/cli/tests/quran/counting_graph_s2.trycmd`
- **Verification:** `cargo test -p cli --test quran` → 13/13 passed.
- **Committed in:** `6d79a65` (Task 3 commit)

**2. [Rule 1 - Bug] SQLite rejects the plan's `COUNT(DISTINCT (a,b,c))` row-value form**
- **Found during:** Task 1 (storage aggregation)
- **Issue:** The plan specified `COUNT(DISTINCT (a.surah, a.ayah, a.token_position))` for the distinct-token method; SQLite fails with `row value misused` on that syntax.
- **Fix:** Used the exact equivalent `SELECT COUNT(*) FROM (… GROUP BY a.surah, a.ayah, a.token_position)`, with an inline comment recording the rationale.
- **Files modified:** `crates/storage-sqlite/src/quran.rs`
- **Verification:** `cargo test -p storage-sqlite --test quran` and the Task-2 service tests (which assert token-level counts) pass.
- **Committed in:** `1fd0b14` (Task 1 commit)

**3. [Rule 1 - Bug] Service signature gained `profile` + `mode` (plan's behaviour snippet showed a 2-arg call)**
- **Found during:** Task 2 (RED phase)
- **Issue:** The behavior block illustrated `root_frequency(&db, "كتب")`, but the plan requires a caller-selected `MultiAnalysisHandling` and a `profile`, and Task 3's handler is `cmd_count_root_frequency(db_path, root, profile, mode)`. A 2-arg service cannot carry the mode.
- **Fix:** Signature is `root_frequency(db, root, profile, mode)` / `lemma_frequency(db, lemma, profile, mode)`; the existing no-dataset test and the CLI were updated accordingly.
- **Files modified:** `crates/application/src/quran_counting.rs`, `crates/application/tests/counting.rs`, `crates/application/src/quran_cli.rs`, `crates/cli/src/quran.rs`
- **Verification:** all three verify commands green.
- **Committed in:** `8ef0201` (RED scaffold) / `8502d59` (GREEN impl)

---

**Total deviations:** 3 auto-fixed (1 blocking, 2 bug).
**Impact on plan:** All three were required for correct/compilable behaviour. The mode semantics remain a flagged, unresolved assumption (OD-12) as the plan itself records.

## Issues Encountered

- **Pre-existing `rustfmt` drift (out of scope):** `cargo fmt -p application` reformats two committed 03-01 files (`alpha_smoke.rs`, `canonical_display_identity.rs`). They were reverted untouched and logged in `deferred-items.md`; plan 03-03's own files are rustfmt-clean under the edition-2024 config.
- **`gsd_run` is a shell function** not present in a fresh executor shell; the tools were invoked directly via `node …/gsd-tools.cjs` for the RED-evidence check.
- **Concurrent single-tree editing:** the tree is shared (`use_worktrees: false`); only the plan's declared files were staged. A concurrent session's `?? .planning/milestone.lock` was left untouched.

## User Setup Required

None — no external service configuration required.

## Known Stubs

None — both stub bodies were replaced with real implementations, the no-dataset path is a typed error (not an empty report), and no placeholder/hardcoded-empty value flows to any output.

## Threat Flags

None — the plan's threat register (T-03-10…T-03-13) covers this surface, and no new network endpoint, auth path, file-access pattern, or trust-boundary schema change was introduced. New SQL binds every parameter (`T-03-10`); every numeric report embeds `CountingRules` with `datasets` + `multi_analysis_handling` (`T-03-11`/`T-03-12`); aggregation stays scalar/indexed (`T-03-13`).

## Owner Gates (BLOCKED — not silently passed)

- **OD-12 (normalization catalog + named linguist, ADR-0204/0205)** — BLOCKED. The multi-analysis counting convention implemented here (`SingleSource`/`AllAnalyses`/`OneVotePerToken`) is code-asserted on the **synthetic** lexicon only and is **not linguist-ratified**; ADR-0211 stays Draft. Closing step: name a qualified Arabic linguist and record sign-off in `docs/reviews/`, then flip ADR-0204/0205 (and ratify ADR-0211 counting semantics).
- **OD-11 (morphology dataset selection & licensing, ADR-0203)** — BLOCKED. Root/lemma frequency is exercised on `synthetic_test_only` data, never scholarly ground truth. Closing step: capture the QAC license to `licenses/qac/` per `licenses/README.md`; if `redistribution_allowed: true` with all capture fields, ratify ADR-0203 Option A and set the license-matrix entry; otherwise keep Option B (user-supplied import). Record in `docs/05-followups/owner-decisions.md`.

## Next Phase Readiness

- SC4 is now satisfiable end-to-end with an active (synthetic) lexicon: activate, run `qai quran count root-frequency <root>`, get a deterministic count with a complete rules block; the mode matrix proves counts are rule-relative.
- Phase-4/family and Phase-5 consumers can rely on `FrequencyReport.rules.datasets` naming the active `<slug>@<version>` and on mode-relative checksums.
- Open concerns: the counting convention needs linguist ratification (OD-12); no licensed lexicon is active (OD-11) so counts remain synthetic-only in this phase.
- Verification snapshot: `cargo test -p storage --lib quran` (4), `cargo test -p storage-sqlite --test quran` (11), `cargo test -p application --test counting` (9), `cargo test -p cli --test quran` (13), `xtask migrate-check` and `xtask arch-check` all green.

## Self-Check: PASSED

- FOUND: `crates/storage/src/quran.rs`
- FOUND: `crates/storage-sqlite/src/quran.rs`
- FOUND: `crates/application/src/quran_counting.rs`
- FOUND: `crates/application/tests/counting.rs`
- FOUND: `crates/application/src/quran_cli.rs`
- FOUND: `crates/cli/src/quran.rs`
- FOUND: `crates/cli/tests/quran/counting_graph_s2.trycmd`
- FOUND: `.planning/phases/03-quran-search-linguistics/03-03-RED-EVIDENCE.json`
- FOUND commit: `1fd0b14` (Task 1), `8ef0201` (Task 2 RED), `8502d59` (Task 2 GREEN), `6d79a65` (Task 3)
