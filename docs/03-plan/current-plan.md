# Current Plan — Active Phase Pointer

**Updated:** 2026-09-29
**Active phase:** Phase 3.5 — Residual Closure (INSERTED) — `.planning/phases/03.5-residual-closure/`

- **Phase-state authority:** `.planning/STATE.md` + per-phase SUMMARY files.
- **Task-granularity authority:** `docs/03-plan/phases/*/tasks.md` boards.
- **Evidence authority:** per-phase `done.md` ledgers.
- Phase 2 board reconciled 2026-09-29 (plan 03.5-01): 65/114 ☑ (57%). See
  `docs/03-plan/phases/phase-02-rag/tasks.md` for task-level status.
- Phase-4 graph board: 0 / 30 ☑, 10 ◐, 3 ⊘. See
  `docs/03-plan/phases/phase-04-quran-graph/tasks.md` for task-level status.
- Phase-0 residual: P0-T56 stays ◐ — `Dockerfile`/`docker-compose.yml` exist but
  the Docker daemon is unavailable, so runtime verification is still open
  (see `docs/05-followups/open-questions.md`).
- Owner-gated items live in `docs/05-followups/decisions-needed.md` (OD-01…OD-14,
  all 🔴) — agents must not close them; record recommendations only.
- Phase numbering is legacy: `phase-02-rag` holds search/linguistics,
  `phase-03-server` is an empty placeholder (server code lives in
  `crates/server`), and the graph proposal lives under `phase-04-quran-graph`.
  Do not renumber directories by hand.
