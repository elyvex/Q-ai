# Agent Instructions

These instructions apply to all agents working on Q-ai.

## Core Rules

1. Read `AGENTS.md` before doing anything.
2. Follow the reading order in `AGENTS.md` > "Before starting work".
3. Every task you pick up must map to a TASK-ID under `docs/04-tasks/active/`.
4. When done, move the task file from `active/` to `completed/`.
5. Update the rollup in `docs/06-progress/task-done-rollup.md`.
6. Record unresolved questions in `docs/05-followups/open-questions.md`.
7. Record architectural decisions in `docs/02-architecture/decisions/` as ADRs.
8. Never invent requirements not present in `docs/01-requirements/requirements.md`.

## Document Flow

- **Planning** belongs in `docs/03-plan/`.
- **Work items** belong in `docs/04-tasks/`.
- **Unresolved questions** belong in `docs/05-followups/`.
- **Completed-work summaries** belong in `docs/06-progress/`.
- **Deep technical knowledge** belongs in `docs/07-technical/`.

Do not mix these concerns.

## Before You Finish

Run the checklist in `.agent/checklists/after-coding.md` and `.agent/checklists/before-commit.md`.
