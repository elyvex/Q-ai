---
phase: 03-quran-search-linguistics
plan: 08
subsystem: quran-search-linguistics
tags: [rust, sqlite, alpha, owner-gates, deferrals, documentation, od-11, od-12, d-03, d-08]

# Dependency graph
requires:
  - phase: 03-quran-search-linguistics
    provides: 03-01 the tracer + SC5 canonical-display pin + search services; 03-02 concatenated golden set + segmentation; 03-03 counting + root/lemma frequency + multi-analysis modes; 03-04 family builders + typed affix error; 03-05 CLI/HTTP family + counting surfaces; 03-06 license matrix + activation-rejection gate; 03-07 doctor soak + performance budgets + attributed tool registry
  - phase: 02-canonical-quran-core
    provides: the canonical reader (SC5 comparison source), approval-gated activation, and the Phase-2 owner-gate record pattern
provides:
  - "docs/05-followups/phase-03-owner-gates.md: OD-11 / OD-12 / D-08 recorded as explicit BLOCKED owner gates with exact closing steps (licenses/qac capture; named linguist + ADR-0204/0205/0210/0215 acceptance)"
  - "docs/05-followups/phase-03-deferrals.md: explicit deferral ledger (UI + result-contract checksum, counting tail incl. hapax_search, legacy phase-02-rag tail, transliteration/L8, graph, editions/multi-RAG) + the 03-VALIDATION.md template-state deferral"
  - "crates/application/tests/alpha_e2e.rs: alpha_end_to_end_synthetic — the full D-03 alpha on the synthetic fixture, enforcing every result contract in one run"
  - "docs/06-progress/status.md + task-done-rollup.md: current Phase-3 progress with the remaining BLOCKED gates"
affects: [phase-04-graph, phase-05-result-contract, gsd-verify-work, phase-10-agent-runtime]

# Actuals (#2632) — same estimateTokens scale as the plan's estimate (chars/4 over the realized diff)
actuals:
  tokens: 16800
  tasks: 3
  commits: 3   # plan's own atomic commits: 18c718d, 3033755, 6657a33
  # NOTE (#3968): the ledger window plan_head_before..HEAD measured 5 commits; 2 belong
  # to concurrent workstreams on the shared tree (a127698 docs(01) verification,
  # 07e41ac docs(phase-01) complete). The metadata/state commit is additional.
  plan_head_before: e7ca845b44ef34b53cd6dd3cba8fd0a0d3433780

tech-stack:
  added: []
  patterns:
    - "Alpha evidence as one full-chain test: import/activate -> forms -> index -> morphology import+activate -> all five search modes -> token/root/lemma/family -> frequency/distribution/co-occurrence, asserting the result contract (trace/span/quotation, attribution, CountingRules) at every step"
    - "Owner-gate record follows the Phase-2 agent-uncloseable pattern: status key, what it blocks, exact closing step + command/file, and an explicit statement that decisions-needed.md stays the source of truth"
    - "Deferral ledger records scope deferrals separately from owner-gate blocks, and records the validation-contract template state rather than silently ignoring it"

key-files:
  created:
    - docs/05-followups/phase-03-owner-gates.md
    - docs/05-followups/phase-03-deferrals.md
    - crates/application/tests/alpha_e2e.rs
  modified:
    - docs/06-progress/status.md
    - docs/06-progress/task-done-rollup.md

key-decisions:
  - "OD-11 and OD-12/D-08 are recorded as explicit BLOCKED owner gates with exact closing steps (licenses/qac capture per licenses/README.md; named linguist + ADR-0204/0205/0210/0215 acceptance), never silently passed; decisions-needed.md remains the source of truth and this plan closes nothing (T-03-30)."
  - "The alpha is proven on the synthetic fixture only: the test is explicitly labeled synthetic, and the deferral ledger + owner-gates record the limit, so no synthetic result is presented as scholarly consensus (T-03-31)."
  - "Every numeric alpha report is gated on a complete CountingRules block; lexicon reports (root/lemma frequency) additionally assert the active dataset is named in rules.datasets (I15/SC4, T-03-32)."
  - "The 03-VALIDATION.md per-task map is deferred to /gsd-validate-phase, recorded in the deferral ledger; the authoritative Phase-3 verification map is the <verify> blocks in 03-01..03-08-PLAN.md (recorded, not silently ignored)."
  - "The alpha harness imports an aligned synthetic lexicon with two competing analyses per token and builds all seven typed family relations before reading them, so the SC3 surface is exercised on attributed, no-winner data."

patterns-established:
  - "One-command alpha: `cargo test -p application --test alpha_e2e` demonstrates the whole Phase-1–3 alpha chain with every contract enforced"
  - "Owner-gate + deferral split: blocks (OD-11/OD-12) are recorded in one file, scope deferrals in another; both cite the legacy terminal-state board"

requirements-completed: [REQ-quran-normalization, REQ-quran-linguistics]

# Coverage metadata (#1602)
coverage:
  - id: D1
    description: "OD-11 and OD-12/D-08 are recorded as explicit BLOCKED owner gates with status, what they block, and the exact closing step + command/file (licenses/qac capture; named linguist + ADR flips), stating that decisions-needed.md is the source of truth and nothing is closed."
    requirement: REQ-quran-linguistics
    verification:
      - kind: integration
        ref: "docs/05-followups/phase-03-owner-gates.md (grep OD-11, OD-12, licenses/qac)"
        status: pass
    human_judgment: false
  - id: D2
    description: "The deferred scope is recorded explicitly (UI/Phase 5, counting tail incl. hapax_search, legacy phase-02-rag tail, transliteration/L8, graph, editions/multi-RAG) plus the 03-VALIDATION.md template-state deferral, and status.md/task-done-rollup.md reflect Phase-3 progress with the OD-11/OD-12 blocks."
    requirement: REQ-quran-linguistics
    verification:
      - kind: integration
        ref: "docs/05-followups/phase-03-deferrals.md (grep hapax_search); docs/06-progress/status.md (grep OD-11); git diff status.md/task-done-rollup.md"
        status: pass
    human_judgment: false
  - id: D3
    description: "The full D-03 alpha runs green in one command: normalize -> forms -> index -> all five search modes -> token/root/lemma/family -> frequency/distribution/co-occurrence, with trace + canonical span + byte-identical quotation on every hit, segmentation on concatenated hits, dataset attribution on lexicon results, and a complete CountingRules on every numeric report."
    requirement: REQ-quran-normalization
    verification:
      - kind: integration
        ref: "cargo test -p application --test alpha_e2e#alpha_end_to_end_synthetic (1 pass)"
        status: pass
      - kind: integration
        ref: "cargo test -p cli --test quran (14 pass)"
        status: pass
      - kind: other
        ref: "cargo test --workspace --no-fail-fast (only the pre-existing cli --test doctor_json audit-tamper case fails); xtask arch-check + migrate-check"
        status: pass
    human_judgment: false
  - id: D4
    description: "Linguistic/scholarly correctness of the alpha results and the licensed-dataset applicability of the alpha (real QAC data, linguist-ratified tagset/roots/goldens)."
    requirement: REQ-quran-linguistics
    verification: []
    human_judgment: true
    rationale: "The alpha runs on the synthetic `synthetic_test_only` lexicon and fixture; no result asserts linguistic correctness (OD-12/D-08 BLOCKED, ADR-0204/0205/0210/0215 unratified) and no licensed morphology dataset is bundled (OD-11 BLOCKED, ADR-0203 Option B)."

# Metrics
duration: 50 min
completed: 2026-09-28
status: complete
---

# Phase 3 Plan 08: Owner Gates + Deferral Ledger + Alpha End-to-End Evidence Summary

**The phase closes with OD-11 and OD-12/D-08 recorded as explicit BLOCKED owner gates (with exact closing steps), the deferred scope recorded rather than dropped, and one runnable full-chain alpha test that proves normalize → forms → index → all five search modes → morphology/root/lemma/family → frequency/distribution/co-occurrence on the synthetic fixture with every result contract enforced.**

## Performance

- **Duration:** 50 min
- **Started:** 2026-09-28T06:28:16Z (plan ledger base `e7ca845`)
- **Completed:** 2026-09-28 (SUMMARY written)
- **Tasks:** 3/3
- **Files modified:** 5 (3 created, 2 modified)

## Accomplishments

- **The two owner gates are recorded as explicit BLOCKED items, never a silent pass (D-08/D-02).** `docs/05-followups/phase-03-owner-gates.md` follows the Phase-2 agent-uncloseable pattern: an agent-uncloseable header, a status key, and one section per gate. **OD-11** records status 🔴, what it blocks (real licensed morphology activation; scholarly SC3/SC4 credibility), and the closing step — capture the QAC license to `licenses/qac/{LICENSE.txt,capture.json,attribution.txt}` per `licenses/README.md` (mandatory `source_url`/`capture_date`/`capturer`; then ratify ADR-0203 Option A if `redistribution_allowed: true` with all fields and set the `fixtures/quran/morphology/license-matrix.json` entry permissive, else accept Option B and keep the user-supplied import), recording the decision + date in `owner-decisions.md`. **OD-12/D-08** records status 🔴 and closes only by naming a qualified Arabic linguist (~0.4 FTE), recording sign-off in `docs/reviews/`, then flipping ADR-0204/0205/0210/0215 to Accepted and updating golden headers from `reviewed_by: pending-linguist`. The file states explicitly that `decisions-needed.md` remains the source of truth and that it closes nothing.
- **The deferred scope is recorded explicitly, not silently dropped (D-02/D-14/D-12).** `docs/05-followups/phase-03-deferrals.md` names every CONTEXT.md Deferred Idea and D-14 tool with rationale and target phase — interactive TUI inspector / Web GUI / word-inspector UI / `REQ-quran-result-contract` checksum → Phase 5; collocation, interval, first/last-occurrence, `unusual_usage`, `hapax_search`, `near_duplicate_passages`, `missing_expected_form`, numeric-report tool → later phases; the legacy `phase-02-rag` tail (discovery/documentation depth, eval-harness polish) → follow-ups; transliteration (ADR-0206) and full-quality L8 (ADR-0216); graph nodes/edges → Phase 4; additional qira'at/editions and multi-RAG/comparative-scripture. It also records the **`03-VALIDATION.md` template-state deferral**: the authoritative per-task verification map is the `<verify>` blocks in `03-01…03-08-PLAN.md`, and the per-task map is deferred to `/gsd-validate-phase`.
- **Phase progress artifacts are current.** `docs/06-progress/status.md` gains a surgical Phase-3 closure section (SC1–SC5 → named checks, the alpha bar, and the OD-11/OD-12 blocks) and `docs/06-progress/task-done-rollup.md` gains a newest-first Phase-3 entry. Both edits are purely additive — the concurrent Phase-1 sections were preserved (81 additions, 0 deletions).
- **D-03's alpha bar is proven in one command.** `crates/application/tests/alpha_e2e.rs::alpha_end_to_end_synthetic` walks the whole alpha on `fixtures/quran/test-edition-min` plus an aligned synthetic lexicon (two competing analyses per token, all seven typed family relations built): migrate → import → activate → `rebuild_forms` → `rebuild_index` → morphology import + approval-gated activation with license evidence → `search_exact` / `search_normalized` / `search_phrase` / `search_concatenated` / `search_regex` → `morphology_for_token` → `root_search` / `lemma_search` → `word_family` → `frequency` / `distribution` / `cooccurrence` / `root_frequency` / `lemma_frequency`. Every search hit is asserted to carry the mandatory trace (I9), a non-empty canonical span (I10), a pinned reference, and a quotation byte-identical to the canonical row resolved independently through the reader (SC5); concatenated hits carry a non-empty segmentation (SC2); every lexicon result carries dataset attribution (SC3); every numeric report carries a complete `CountingRules` block, with the lexicon reports naming the active dataset (I15/SC4); and the multi-analysis results never elect a winner.

## Task Commits

Each task was committed atomically:

1. **Task 1: Record OD-11 and OD-12/D-08 as explicit BLOCKED owner gates** — `18c718d` (docs)
2. **Task 2: Record the deferral ledger and update phase progress artifacts** — `3033755` (docs)
3. **Task 3: Alpha end-to-end evidence across every SC surface (D-03)** — `6657a33` (test)

**Plan metadata:** (this commit) `docs(03-08): complete [plan-name] plan`

## Files Created/Modified

- `docs/05-followups/phase-03-owner-gates.md` (new) — the agent-uncloseable OD-11 / OD-12 / D-08 record with status, blocked scope, and exact closing steps + commands.
- `docs/05-followups/phase-03-deferrals.md` (new) — the explicit deferral ledger (UI, counting tail, legacy tail, transliteration/L8, graph, editions/multi-RAG) plus the `03-VALIDATION.md` template-state deferral.
- `crates/application/tests/alpha_e2e.rs` (new) — `alpha_end_to_end_synthetic`, the full-chain alpha with contract assertions (harness `alpha_db`, `lexicon_document`, `morph_import_params`, `assert_search_contract`, `assert_rules_complete`).
- `docs/06-progress/status.md` (modified) — appended the Phase-3 closure section (SC map, alpha bar, OD-11/OD-12 blocks, deferrals, gate).
- `docs/06-progress/task-done-rollup.md` (modified) — newest-first Phase-3 closure entry.

## Decisions Made

- **Owner gates are blocks, not deferrals.** They live in their own record with an explicit "closes nothing" statement; the deferral ledger is reserved for scope deferrals. Both cite the legacy terminal-state board and `decisions-needed.md`.
- **The alpha asserts contract *presence*, not linguistic truth.** Every hit must carry a trace/span/verified quotation and every numeric report a complete `CountingRules`; no assertion claims linguistic correctness (OD-12) or real-data behavior (OD-11). The L0-exact trace legitimately carries no rule applications, so the "ordered rule set non-empty" assertion is applied only where a profile actually applies rules (L3/L6), while the trace label is asserted on every hit.
- **The alpha harness imports a no-winner lexicon.** Two competing analyses per token plus all seven typed family relations give the SC3 surface real, attributed, no-winner data to assert against.
- **`03-VALIDATION.md` stays a seeded template.** Its per-task map is deferred to `/gsd-validate-phase`; the PLAN `<verify>` blocks are the authoritative map. Recorded in the deferral ledger rather than silently ignored.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Import run id must be a UUID because it doubles as the canonical `edition_id`**
- **Found during:** Task 3 (first alpha run)
- **Issue:** The test seeded the import `run_id` as `"run-alpha-e2e"`, but the reader (`QuranReaderService::get_ayah`, used for the SC5 byte-identity check) parses the edition id as a UUID and failed with `edition id 'run-alpha-e2e' is not a UUID`.
- **Fix:** Used the `alpha_smoke` harness' UUID `RUN_ID` (`aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee`) as the import run id.
- **Files modified:** `crates/application/tests/alpha_e2e.rs`
- **Verification:** `cargo test -p application --test alpha_e2e` → 1 pass.
- **Committed in:** `6657a33` (Task 3)

**2. [Rule 1 - Bug] `&Arc<SqliteDatabase>` does not coerce to `&dyn Database`**
- **Found during:** Task 3 (first compile)
- **Issue:** `run_import` and `activate_edition` take `&dyn storage::Database`; passing `&db` where `db: Arc<SqliteDatabase>` failed the trait bound (deref coercion does not unsize to a trait object through `Arc`).
- **Fix:** Passed `&*db` for those two calls (the `alpha_smoke` convention); the generic service calls keep `&db`.
- **Files modified:** `crates/application/tests/alpha_e2e.rs`
- **Verification:** Compiles and the alpha passes.
- **Committed in:** `6657a33` (Task 3)

**3. [Rule 3 - Blocking] rustfmt reformat of the new test file**
- **Found during:** Task 3 (pre-commit hygiene)
- **Issue:** `rustfmt --check` reported formatting diffs in the new file.
- **Fix:** Ran `rustfmt --edition 2024` on the file; re-checked clean.
- **Files modified:** `crates/application/tests/alpha_e2e.rs`
- **Verification:** `rustfmt --edition 2024 --check` clean; test still green.
- **Committed in:** `6657a33` (Task 3)

---

**Total deviations:** 3 auto-fixed (2 bugs, 1 blocking).
**Impact on plan:** No scope creep — no new dependency, no migration, no canonical/hash recipe touched. Every declared `files_modified` entry was touched.

## Issues Encountered

- **Full workspace run is not fully green, by design/known state.** `cargo test --workspace --no-fail-fast` fails on exactly one test result block: the pre-existing `crates/cli --test doctor_json::audit_verify_rejects_corrupt_chain_without_modifying_database` audit-tamper case (Phase-1 territory, proven unrelated in 03-05). All 177 other test-result blocks are ok, including `alpha_e2e` (1) and `cli --test quran` (14). Per the plan's known-failure note, the plan's own suites are the gate.
- **Shared-tree concurrent commits.** Two concurrent workstream commits (`a127698 docs(01) verification`, `07e41ac docs(phase-01) complete`) landed in the ledger window; the raw `rev-list` count is 5 while this plan's own commits are 3. Recorded transparently in `actuals`.

## User Setup Required

None — no external service configuration, no new dependencies, no package installs.

## Known Stubs

None new. The plan's intentional non-executing items are already recorded (and are not code stubs):

- `fixtures/quran/performance/budgets.json` → the `full-corpus` p50/p99 rows are recorded but not executable (OD-11; no licensed corpus).
- `quran.fts.token_index` → `skipped` by design (no `quran.token.v1` backend), pre-existing.

## Threat Flags

None beyond the plan's own register. `T-03-30` (owner gate recorded as resolved) is mitigated: the owner-gates file states it closes nothing, `decisions-needed.md` stays the source of truth, and verification greps both gate ids plus the `licenses/qac` closing path. `T-03-31` (synthetic data presented as scholarly) is mitigated: the alpha test header and the deferral/owner-gate records label the fixture synthetic and assert no linguistic correctness. `T-03-32` (numeric output without rules) is mitigated: `assert_rules_complete` gates every numeric report. `T-03-33` (silent drop of deferred scope) is mitigated: the deferral ledger names every CONTEXT.md Deferred Idea and D-14 tool, grepped for `hapax_search`. `T-03-SC` stays accepted (no packages added).

## Owner Gates (BLOCKED — not silently passed)

- **OD-11 (morphology dataset selection & licensing, ADR-0203)** — BLOCKED. The alpha and every lexicon assertion run on the synthetic `synthetic_test_only` fixture. Closing step: capture the QAC license to `licenses/qac/{LICENSE.txt,capture.json,attribution.txt}` per `licenses/README.md`; ratify ADR-0203 Option A when `redistribution_allowed: true` with all fields, else accept Option B; record in `docs/05-followups/owner-decisions.md` and `docs/05-followups/phase-03-owner-gates.md`.
- **OD-12 / D-08 (normalization catalog + named linguist, ADR-0204/0205/0210/0215)** — BLOCKED. Closing step: name a qualified Arabic linguist (~0.4 FTE), record sign-off in `docs/reviews/`, then flip ADR-0204/0205/0210/0215 to Accepted and update golden headers; record in `docs/05-followups/phase-03-owner-gates.md`.

## Next Phase Readiness

- **Phase 3 is at its engineered finish line:** every success criterion (SC1–SC5) maps to existing code plus a repeatable check; both phase requirements are covered; the two owner gates and the deferred scope are recorded explicitly.
- **`/gsd-verify-work` input:** `cargo test -p application --test alpha_e2e` proves the alpha; `cargo test -p cli --test quran` proves the CLI surfaces; the only workspace failure is the known pre-existing `cli --test doctor_json` audit-tamper case.
- **Open follow-ups:** OD-11/OD-12 remain 🔴 and owner-uncloseable by an agent; the `03-VALIDATION.md` per-task map awaits `/gsd-validate-phase`; the deferred ledger lists every out-of-scope item with its target phase.
- **Phase 4 (Graph)** can build on roots/lemmas via the attributed lexicon services proven here.

---

*Phase: 03-quran-search-linguistics*
*Completed: 2026-09-28*

## Self-Check: PASSED

- All declared/created files exist: `docs/05-followups/phase-03-owner-gates.md`, `docs/05-followups/phase-03-deferrals.md`, `docs/06-progress/status.md`, `docs/06-progress/task-done-rollup.md`, `crates/application/tests/alpha_e2e.rs`.
- All 3 plan task commits exist: `18c718d` (Task 1), `3033755` (Task 2), `6657a33` (Task 3).
- Verification re-run green: `cargo test -p application --test alpha_e2e` (1), `cargo test -p cli --test quran` (14), `cargo test --workspace --no-fail-fast` (only the pre-existing `cli --test doctor_json` audit-tamper failure), `xtask arch-check` OK, `xtask migrate-check` OK (21 migrations).
- Out-of-scope failure left alone and documented: `cli --test doctor_json` (pre-existing audit-tamper, Phase-1 territory).
