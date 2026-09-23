# Current Plan — Active Phase Pointer

**Updated:** 2026-09-24
**Active phase:** Phase 2 — Quran search/linguistics (`docs/03-plan/phases/phase-02-rag/`)

- Search core is implemented: P2-T51/T52 (search API + CLI) completed 2026-09-23
  (see `docs/06-progress/task-done-rollup.md`). Next: P2-T53–T56 (goldens, DoS
  suite, latency gates, ADRs). Morphology Sprints 2.4–2.5 are untouched (0 tasks),
  which keeps Phase-4 M5 (TASK-405/406/407) blocked.
- Phase-4 graph foundations (port, traversal, structural projection, memory
  backend, conformance) exist in `crates/quran-graph` with green tests, but the
  Phase-4 board (`docs/03-plan/phases/phase-04-quran-graph/tasks.md`) still
  reads 0/30 — code is ahead of its task paperwork; do not treat the board as
  the implementation state. See that phase's `done.md` §3 for the blocker list.
- Phase-0 residual: P0-T56 stays ◐ — `Dockerfile`/`docker-compose.yml` exist but
  the Docker daemon is unavailable, so runtime verification is still open
  (see `docs/05-followups/open-questions.md`).
- Owner-gated items live in `docs/05-followups/decisions-needed.md` (OD-01…OD-14,
  all 🔴) — agents must not close them; record recommendations only.
- Phase numbering is legacy: `phase-02-rag` holds search/linguistics,
  `phase-03-server` is an empty placeholder (server code lives in
  `crates/server`), and the graph proposal lives under `phase-04-quran-graph`.
  Do not renumber directories by hand.
