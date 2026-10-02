# Progress Status

**Updated:** 2026-10-02
**Active phase:** Phase 3.5 — Residual Closure (INSERTED) — `.planning/phases/03.5-residual-closure/` — closing (Wave 4/4)
**Phase-state authority:** `.planning/STATE.md` + per-phase SUMMARY
**Task-granularity authority:** `docs/03-plan/phases/*/tasks.md`
**Evidence authority:** per-phase `done.md` ledgers

## Summary

Phase 3.5 (Residual Closure) is closing. The legacy boards were reconciled
against the live tree (plan 03.5-01): Phase 2 now reads 78/114 ☑ (68%), with
◐ rows covering partial/synthetic/owner-gated work. All four 03.5 plans are
complete (reconciliation + docs, counting/index-job, harness/conformance/
goldens/soak, hardening/gates); the W1–W10 evidence matrix below names the
proving command for every workstream, and all 14 owner decisions remain open.

## Gate status

| Gate | Command | Status |
|---|---|---|
| Format | `cargo fmt --all -- --check` | ✅ clean on the verified task surfaces |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` | ✅ clean for the verified new crates; full workspace rerun pending |
| Targeted tests | `cargo test -p quran-normalization -p quran-search -p quran-morphology -p quran-graph` and Phase-2 application suites | ✅ green in the reconciliation pass |
| Architecture | `cargo run -p xtask -- arch-check` | ✅ OK |
| Migrations | `cargo run -p xtask -- migrate-check` | ✅ OK; 22 migrations, checksums stable |
| Coverage | `cargo run -p xtask -- coverage-gate <lcov.info>` | ✅ `citations` floor aligned to the published 85% (`cargo llvm-cov -p citations` = 99.5% lines) |
| Dependency audit | `cargo deny check` | ⏭️ skipped-with-reason: `cargo-deny` is **not installed** in this local environment; the deny gate is enforced by the CI `deny` job (`.github/workflows/ci.yml`), never a local pass |
| Full workspace | `cargo test --workspace` | ⚠️ not claimed in this pass; existing suite is large and the tree has concurrent work |

## Crate / phase status

| Area | Status | Notes |
|---|---|---|
| Phase 0 foundation | ✅ implementation landed | P0-T56 container runtime remains ◐ because the Docker daemon is unavailable |
| Canonical Quran core | ✅ implementation landed | licensed full-corpus/editorial gates remain open |
| `quran-normalization` | ✅ 11 tasks ☑ / 1 ◐ | T21 awaits linguist sign-off |
| `quran-search` | ✅ core/search hardening landed | T53/T55/T56 remain ◐ pending full-corpus, linguist, and ADR gates |
| `quran-morphology` | ◐ mechanics landed | synthetic adapters/import/activation/tools; licensed dataset, linguist, API, and review gates remain |
| Counting/discovery | ◐ core services + CLI landed | API parity, doctor/evaluation, full soak, and multi-analysis modes remain |
| `quran-graph` | ◐ foundations only | TASK-401/403/404/409/413/414/416/420/424/427 are partial; SQLite/application/CLI/API/ops remain open |
| Server / observability | ✅ core services | OTLP exporter-boundary scrubbing and job scheduling precision fixes landed |

## Current totals

- **Phase 2:** 65 / 114 tasks ☑ (57%); ◐ rows are partial/synthetic/owner-gated; 0 / 50 ACs and 0 / 14 ADRs are formally accepted. Board reconciled 2026-09-29 (plan 03.5-01).
- **Graph phase:** 0 / 30 ☑; 10 ◐; 3 ⊘; M5 (TASK-405/406/407) remains blocked by licensed morphology evidence and the missing graph root-family projection. See `docs/03-plan/phases/phase-04-quran-graph/tasks.md` for task-level status.
- **Phase 0 residual:** P0-T56 remains ◐; clean-machine/container runtime verification is still open.
- **Phase 3.5:** closing (W1–W10 evidence matrix above; Waves 1–4 complete, full gate running) — boards reconciled, docs + SECURITY.md written, counting parity + index job + harness + conformance + goldens + soak landed, Phase-1 hardening + perf/coverage gates landed.

## Next work

1. Close the remaining Phase-2 partials with real data, linguist/owner review, API parity, and full-corpus gates.
2. Implement SQLite graph persistence and application/CLI/API/doctor integration before claiming graph milestones.
3. Run the clean-machine Phase-0 exit ritual when the Docker daemon is available.
4. Keep owner decisions in `docs/05-followups/decisions-needed.md`; agents must not invent approvals.

## Phase 1 closure — TASK-001 foundation gap closure, 2026-09-28

- **TASK-001-foundation-gap-closure: Status: Completed** (plan 01-05-02;
  record at `docs/04-tasks/completed/TASK-001-foundation-gap-closure.md`;
  owner sign-off NOT claimed).
- **Plans:** 01-01, 01-02, 01-03, 01-04, 01-05 — evidence per plan in the
  closed record §Completion and `.planning/phases/01-foundations/01-VALIDATION.md`.
- **Criteria:** C1, C2, C3, C4, C5 — each maps to a repeatable command (help +
  foundation suite; config precedence; same-UoW audit + `audit verify`; job
  lifecycle + host suites; `arch-check` + `xtask ci`).
- **Decisions:** D-01, D-02, D-03, D-04, D-05, D-06, D-07, D-08, D-09, D-10,
  D-11, D-12, D-13, D-14, D-15, D-16.
- **Gate:** clippy/arch/migrate green, all Phase 1 suites green; `cargo fmt`
  and `cargo test --workspace` red only on quoted foreign concurrent-session
  files (untouched); `cargo-deny` unavailable locally (CI-enforced). Re-run
  the full gate when the tree is quiet.
- **Boundary:** brownfield gap closure; no new migration/package/broker;
  deferred/owner-gated items stay open. Phase 2 remains the active phase;
  unrelated gates above are unchanged.

## Phase 3.5 evidence matrix (W1–W10)

Per-workstream criterion, implementation files, and the repeatable command
that proves it. All commands run from the repo root. Owner-gated scope is
marked, never claimed.

| WS | Criterion satisfied | Implementation files | Repeatable proof | State |
|---|---|---|---|---|
| W1 truth restoration | Every board row matches the tree; flipped rows carry `done.md` evidence; the two false headers corrected; OD rows untouched | `docs/03-plan/phases/*/tasks.md` + `done.md`, `docs/03-plan/roadmap.md`, `current-plan.md`, `AGENTS.md`, `.agent/*` | `for f in docs/03-plan/phases/*/tasks.md; do rg -c '☑\|◐\|⊘\|☐' "$f"; done` | closed |
| W2 documentation set | Six P2-T113 operator documents + `SECURITY.md` written from the live implementation, draft ADRs labeled draft | `docs/07-technical/quran-{normalization-spec,profile-catalog,search-cookbook,counting-rules}.md`, `docs/10-operations/quran-reindex-runbook.md`, `SECURITY.md` | `rg -o 'L[0-8]' docs/07-technical/quran-profile-catalog.md \| sort -u \| wc -l` (= 9); `rg -o 'QAI-IDX-0101' docs/10-operations/quran-reindex-runbook.md \| wc -l` (≥ 1) | closed |
| W3 index reconcile job | Read-only `quran.index.verify` on the durable substrate; drift verdict = doctor's computation; sampled report states size/rate; empty DB Skipped-with-remedy | `crates/application/src/quran_index.rs`, `job_queue.rs`, `tests/index_verify.rs`, `crates/cli/tests/quran/index_verify.trycmd` | `cargo test -p application --test index_verify` (6/6); `cargo test -p cli --test quran quran_index_verify` (3/3 cases) | closed |
| W4 counting surface | All 13 counting services reachable from CLI + HTTP with fail-closed parity; every numeric report carries `CountingRules` under `rules` | `crates/application/src/quran_counting.rs`, `quran_lexicon_api.rs`, `quran_cli.rs`, `crates/cli/src/quran.rs`, `crates/server/src/api.rs`, `counting_graph_s3/s4.trycmd` | `cargo test -p application --test counting`; `cargo test -p cli --test quran quran_counting_graph` (26/26 cases); `rg -c 'route("/api/v1/quran/count' crates/server/src/api.rs` (= 13) | closed |
| W5 goldens at size | Family set ≥ 120 (154, all 7 kinds) + root/lemma set at 500; every row `pending-linguist` + `synthetic_test_only`, resolving through the real service | `fixtures/quran/lexicon/families/curated.jsonl`, `fixtures/quran/lexicon/root-lemma-goldens.jsonl`, `crates/application/tests/family_goldens.rs` | `cargo test -p application --test family_goldens` (2/2) | closed at fixture scale; linguistic correctness stays OD-12 |
| W6 evaluation harness | Versioned datasets + metric set + `Unavailable`-with-remedy + tolerance baseline; gates mechanical regressions only | `crates/evaluation/src/{dataset,metrics,report}.rs`, `tests/harness.rs`, `fixtures/quran/evaluation/*` | `cargo test -p evaluation` (14/14) | closed; not a linguistic verdict (ADRs Draft) |
| W7 tool conformance | All 12 registered tools pass one shared contract (envelope, reproducibility, attribution, trace, typed-unavailable, truncation) | `crates/tool-registry/tests/conformance.rs`, `crates/application/tests/quran_tools.rs` | `cargo test -p tool-registry` (16/16); `cargo test -p application --test quran_tools` (9/9) | closed |
| W8 soak + measurement | 50k deterministic queries green with per-query invariants; cold-rebuild budget pinned; fixture bound never loosened | `crates/application/tests/soak_full.rs`, `fixtures/quran/performance/budgets.json`, `crates/application/tests/search_latency.rs` | `cargo test -p application --test soak_full` (2/2, ~30 min wall) | closed at fixture scale; full corpus + p99 stay OD-11 |
| W9 Phase-1 hardening | WR-05/06/07/08/09/10/11/12/13 + IN-06/07 fixed with tests; capability items (WR-01..04, IN-01..05/08/09) deferred to Phase 12 | `audit_bridge.rs`, `cli/src/lib.rs`, `quran.rs`, `cli/src/doctor.rs`, `application::{db,job_queue,quran_cli}.rs`, `jobs/src/queue.rs`, `scripts/verify-phase1-*.sh`, `phase-3.5-deferrals.md` §7 | `cargo test -p application --test phase1_foundation` (8/8); `cargo test -p audit -p provenance -p jobs`; `cargo test -p cli --test foundation audit_verify` + `--test doctor_json` (3/3); `sh scripts/verify-phase1-records.sh` (owned checks pass; companion still flags foreign drift) | closed; capability residue → Phase 12 |
| W10 gates + close | Median/p95 perf gate (same 5 ms bound); citations floor 85 = published 85; this matrix; ODs all open | `crates/application/tests/quran_reader.rs`, `xtask/src/coverage.rs`, `docs/05-followups/{followups,decisions-needed}.md`, this file | `cargo test -p application --test quran_reader` incl. 4× `lookup_performance_smoke`; `cargo llvm-cov -p citations` (99.5% lines); `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace` (see gate table) | closed except: walkthrough half open (needs a human), deny gate CI-enforced, full-workspace result recorded below |

## Phase 3 closure — Quran Search & Linguistics (GSD plans 03-01…03-08), 2026-09-28

- **Status: implemented and phased; alpha proven on the synthetic fixture.**
  Phase 3 (roadmap) is the legacy `phase-02-rag` board by another number
  (D-04): evidence-driven brownfield gap closure, not a licensed-data or
  linguist-ratified release.
- **Success criteria (each backed by a named repeatable check):**
  - **SC1** normalized/exact search with explainability (I9 trace) and the
    normalization rule-doc correction — plans 03-01, 03-02.
  - **SC2** concatenated (spaceless) search: independent oracle + ≥120-case
    golden fixture + segmentation-tiling runner + CLI snapshot — plan 03-02.
  - **SC3** morphology: lemma/root/analyses/word family with all typed relation
    builders, CLI `family`/`lemma` and versioned HTTP routes — plans 03-04,
    03-05.
  - **SC4** frequency / distribution / co-occurrence plus real root/lemma
    frequency and selectable multi-analysis modes, every numeric report carrying
    `CountingRules` — plan 03-03 (CLI/HTTP surfaces 03-05).
  - **SC5** displayed canonical text never modified by normalization: per-hit
    canonical-display identity pin (I8) — plan 03-01.
  - Closing exit evidence: license/attribution activation gate (03-06),
    `doctor --indexes` end-to-end soak + performance budgets + attributed tool
    registry (03-07).
- **Alpha (D-03):** `cargo test -p application --test alpha_e2e` walks
  normalize → forms → index → all five search modes → token/root/lemma/family →
  frequency/distribution/co-occurrence on the synthetic fixture, asserting every
  result contract (trace + canonical span on hits; dataset attribution on
  lexicon results; `CountingRules` on every numeric report).
- **Owner gates remain BLOCKED (🔴, never a silent pass):**
  - **OD-11** (morphology dataset selection & licensing, ADR-0203) — 🔴
    unanswered. SC3/SC4 are satisfied behaviorally on the `synthetic_test_only`
    lexicon only; QAC ships via the user-supplied import path (Option B) until
    `licenses/qac/` is captured. Record: `docs/05-followups/phase-03-owner-gates.md`.
  - **OD-12 / D-08** (normalization catalog + named linguist) — 🔴 unanswered.
    ADR-0204/0205/0210/0215 stay Draft; goldens keep
    `reviewed_by: pending-linguist`. Record:
    `docs/05-followups/phase-03-owner-gates.md`.
- **Deferred (recorded, not dropped):** UI/result-contract research checksum →
  Phase 5; counting/discovery tail (`hapax_search`, collocation, interval,
  first/last-occurrence, `unusual_usage`, `near_duplicate_passages`,
  `missing_expected_form`, numeric-report tool) → later phases; legacy
  `phase-02-rag` tail → follow-ups; transliteration/L8, graph, additional
  qira'at/editions, multi-RAG/comparative scripture. Record:
  `docs/05-followups/phase-03-deferrals.md`.
- **Gate:** `cargo test -p application --test alpha_e2e`,
  `cargo test -p cli --test quran`, `cargo test --workspace`, and
  `xtask arch-check` / `migrate-check`; the pre-existing
  `cli --test doctor_json` audit-tamper failure remains for its own owner.
