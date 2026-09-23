# `docs/04-tasks/` — cross-phase task index

> **System of record:** per-phase boards under `docs/03-plan/phases/*/tasks.md`
> plus their `done.md` ledgers. This directory exists because `AGENTS.md` and
> `.agent/` workflow docs require a `docs/04-tasks/active/` location; it is an
> **index, not a second tracker**. Never record status here that disagrees with
> a phase board — update the board and link it instead.

- `active/` — one stub per cross-phase or orphan work item that has no home on
  a phase board, each pointing at the owning board/task ID.
- `completed/` — closed stubs, moved here on completion with date + evidence.

Workflow-text reconciliation (naming the phase boards canonical in
`AGENTS.md`/`.agent/instructions.md`/`.agent/workflow.md`) is still open —
see FU-DOC-02 in `docs/05-followups/followups.md`.
