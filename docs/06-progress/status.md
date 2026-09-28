# Progress Status

**Updated:** 2026-09-24
**Active phase:** Phase 2 — Quran search, normalization, morphology, and counting
**Related boards:** `docs/03-plan/phases/phase-02-rag/tasks.md` · `docs/03-plan/phases/phase-04-quran-graph/tasks.md`

## Summary

The repository has moved beyond the Phase-0 chassis: canonical Quran import/read services,
normalization, FTS5 search, morphology staging/activation services, counting/discovery
services, and in-memory graph foundations are implemented. The phase boards are the source
of truth for completion; partial work is recorded as ◐ rather than counted as ☑.

## Gate status

| Gate | Command | Status |
|---|---|---|
| Format | `cargo fmt --all -- --check` | ✅ clean on the verified task surfaces |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` | ✅ clean for the verified new crates; full workspace rerun pending |
| Targeted tests | `cargo test -p quran-normalization -p quran-search -p quran-morphology -p quran-graph` and Phase-2 application suites | ✅ green in the reconciliation pass |
| Architecture | `cargo run -p xtask -- arch-check` | ✅ OK |
| Migrations | `cargo run -p xtask -- migrate-check` | ✅ OK; 19 migrations |
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

- **Phase 2:** 58 / 114 tasks ☑ (51%); ◐ rows are partial/synthetic/owner-gated; 0 / 50 ACs and 0 / 14 ADRs are formally accepted.
- **Graph phase:** 0 / 30 ☑; 10 ◐; M5 (TASK-405/406/407) remains blocked by licensed morphology evidence and the missing graph root-family projection.
- **Phase 0 residual:** P0-T56 remains ◐; clean-machine/container runtime verification is still open.

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
