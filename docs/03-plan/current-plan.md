# Current Plan — Active Phase Pointer

**Updated:** 2026-09-24
**Active phase:** Phase 2 — Quran search/linguistics (`docs/03-plan/phases/phase-02-rag/`)

- Search core is implemented: P2-T51/T52 (search API + CLI) completed 2026-09-23
  (see `docs/06-progress/task-done-rollup.md`). The 2026-09-24 reconciliation now
  records 58/114 Phase-2 tasks ☑, plus P2-T35 (single-step index rollback)
  closed later the same day → **59/114 ☑ (52%)**; ◐ rows cover
  partial/synthetic/owner-gated work.
  Licensed morphology evidence, API parity, doctor/evaluation, and exit gates remain open.
- Phase-4 graph foundations (port, traversal, structural projection, memory
  backend, conformance, and a partial file-backed CLI) exist with green tests.
  The Phase-4 board records **0 / 30 ☑, 10 ◐, 3 ⊘**; see that phase's `done.md`
  for the blocker list.
- Phase-0 residual: P0-T56 stays ◐ — `Dockerfile`/`docker-compose.yml` exist but
  the Docker daemon is unavailable, so runtime verification is still open
  (see `docs/05-followups/open-questions.md`).
- Owner-gated items live in `docs/05-followups/decisions-needed.md` (OD-01…OD-14,
  all 🔴) — agents must not close them; record recommendations only.
- Phase numbering is legacy: `phase-02-rag` holds search/linguistics,
  `phase-03-server` is an empty placeholder (server code lives in
  `crates/server`), and the graph proposal lives under `phase-04-quran-graph`.
  Do not renumber directories by hand.
