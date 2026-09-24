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

- **Phase 2:** 56 / 114 tasks ☑ (49%); ◐ rows are partial/synthetic/owner-gated; 0 / 50 ACs and 0 / 14 ADRs are formally accepted.
- **Graph phase:** 0 / 30 ☑; 10 ◐; M5 (TASK-405/406/407) remains blocked by licensed morphology evidence and the missing graph root-family projection.
- **Phase 0 residual:** P0-T56 remains ◐; clean-machine/container runtime verification is still open.

## Next work

1. Close the remaining Phase-2 partials with real data, linguist/owner review, API parity, and full-corpus gates.
2. Implement SQLite graph persistence and application/CLI/API/doctor integration before claiming graph milestones.
3. Run the clean-machine Phase-0 exit ritual when the Docker daemon is available.
4. Keep owner decisions in `docs/05-followups/decisions-needed.md`; agents must not invent approvals.
