# Agent Workflow

## 1. Read Context

1. `AGENTS.md` (root)
2. `.agent/instructions.md`
3. `docs/00-overview/project-overview.md`
4. `docs/01-requirements/requirements.md`
5. `docs/03-plan/current-plan.md`
6. Active phase `plan.md` under `docs/03-plan/phases/`
7. `docs/06-progress/status.md`
8. `docs/05-followups/open-questions.md`

## 2. Plan

- Work items live in `docs/04-tasks/active/`.
- A task not listed in `docs/04-tasks/` must be created there before work starts.
- Each task must have a TASK-ID in its filename.

## 3. Implement

- Follow `.agent/coding-rules.md`.
- Run tests and checks before declaring done.
- Follow the checklist in `.agent/checklists/before-coding.md` and `.agent/checklists/after-coding.md`.

## 4. Complete

- Move the task file from `docs/04-tasks/active/` to `docs/04-tasks/completed/`.
- Update `docs/06-progress/task-done-rollup.md`.
- Update `docs/06-progress/status.md`.
- Update `CHANGELOG.md` when appropriate.

## 5. Follow Up

- Anything unresolved goes to `docs/05-followups/open-questions.md`.
- Session notes go to `docs/05-followups/agent-followups/YYYY-MM-DD-session-nnn.md`.
