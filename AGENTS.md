# AGENTS.md

## Project

**Q-ai** is a research platform for studying the Quran, hadith collections, Islamic literature, and comparative scripture (Torah, Tanakh, New Testament).

- **Primary Language:** Rust
- **Primary Interfaces:** Web GUI, TUI, CLI, API
- **Deployment:** Local-first + Server
- **Architecture:** Canonical Quran Engine + Knowledge Graph + Multi-RAG + Agentic Tool Runtime

Read `docs/01-requirements/requirements.md` for the full PRD.

---

## Project Structure

Documentation lives in a numbered tree under `docs/`. Reading order matters:

```text
00-overview     What is this project?
01-requirements What must it do?
02-architecture How is it designed? (ADRs live in decisions/)
03-plan         What is the plan? (phases + backlog)
04-tasks        What work items exist? (active / blocked / completed)
05-followups    What remains unresolved?
06-progress     What has been done? (rollups, status)
07-technical    How does it work in detail?
08-api          API docs
09-testing      Test strategy and plans
10-operations   Deployment, monitoring, ops
archive         Obsolete plans, decisions, completed phases
```

Agent-critical files at the repo root:

- `AGENTS.md` (this file)
- `README.md`
- `CHANGELOG.md`
- `.agent/` — agent instructions, coding rules, workflow, checklists

---

## Before starting work

1. Read this file.
2. Read `docs/00-overview/project-overview.md`.
3. Read `docs/03-plan/current-plan.md`.
4. Read the active phase plan under `docs/03-plan/phases/`.
5. Read `docs/06-progress/status.md`.
6. Check `docs/05-followups/open-questions.md`.

---

## Naming Conventions

- **Tasks:** `TASK-nnn-slug.md` (e.g., `TASK-001-project-bootstrap.md`)
- **Phases:** `phase-NN-slug/` (e.g., `phase-00-foundation/`)
- **Architecture Decision Records:** `ADR-nnnn-title.md` under `docs/02-architecture/decisions/`
- **Deliverables:** `Dn.n` (e.g., `D0.1`)
- **Acceptance Criteria:** `AC-Pn-nn` (e.g., `AC-P0-01`)
- **Sessions / follow-ups:** `YYYY-MM-DD-session-nnn.md` under `docs/05-followups/agent-followups/`

---

## Planning Rules

- Never modify the master plan without justification.
- Every implementation task must have a TASK-ID.
- Update task status after implementation.
- Record important architectural decisions as ADRs.
- Never mark a task complete until its acceptance criteria are satisfied.

---

## Completion Rules

When completing a task:

1. Implement it.
2. Run tests.
3. Run lint/checks.
4. Update the task document.
5. Update phase progress.
6. Update `docs/06-progress/task-done-rollup.md`.
7. Add important follow-ups to `docs/05-followups/`.
8. Update `CHANGELOG.md` when appropriate.

---

## Completion Flow

```text
Task
  ↓
Implementation
  ↓
Task completed
  ↓
task-done-rollup.md
  ↓
Phase completed
  ↓
phase status
  ↓
Master progress
```

---

## Key Principles from the PRD

- Exact text before generated interpretation.
- Every factual claim must be traceable.
- The Quran is stored structurally, not only as RAG chunks.
- Canonical text must be separated from translations and annotations.
- No model may fabricate a verse, hadith, chain, grading, or citation.
- Answers must distinguish quotation, source summary, and AI analysis.
- Research results should be reproducible.
- Local data remains local unless the user explicitly enables a remote provider.
- Agents and tools use deny-by-default permissions.
- Religious conclusions must not be falsely presented as scholarly consensus.

See `docs/01-requirements/requirements.md` for the full list.

---

## Current Phase

Check `docs/03-plan/current-plan.md` for the active phase and `docs/06-progress/status.md` for the current state.

<!-- graft:start -->
## Graft — repo context graph

This repo is indexed in `graft/`: small linked markdown nodes that explain each
system and carry exact file:line spans, kept in sync with the code through git.

For ANY task here — understanding how something works, finding where code lives,
or scoping a change — get context from the graph before grepping or opening
source files. Re-ask freely (it's cheap) and reuse literal identifiers you
already have (symbol, error string, file name) as the query. New to this repo?
Run `graft map` first — a token-budgeted orientation (dir clusters, hubs,
hotspots), no LLM, no key.

- Run `graft ask "<your question>" --source` → ranked nodes with the relevant
  code spans inlined (each hit's ≤8-line crux by default; `--full` for whole
  definitions when the crux isn't enough). Match the tool to the task shape:
  for understanding or editing, the top node IS the answer — cite its
  `covers:` file:line spans and edit straight from `--source`. For
  exhaustive tasks ("every occurrence / every caller of this pattern"), ranked
  results are top-N, not complete — run `graft grep "<literal>"` instead
  (exhaustive over indexed files, grouped by enclosing symbol), falling back
  to raw `grep -rn` only for unindexed files.
- `graft skeleton <file>` → every definition's signature + span, ~10× cheaper
  than reading the file; use it to skim an API surface.
- `graft callers <symbol>` gives precomputed, exact edges — who calls this.
  Add `--direction out` for what it calls, or `--depth N` to walk
  transitively for the full blast radius. For structural questions, skip
  ranking and use this directly.
- Or browse: `graft/INDEX.md` lists every node; follow the links.
- Monorepos and folders of multiple repos rank fairly across sub-projects —
  hits carry `[scope/]` labels naming which one they're from. Narrow with
  `graft ask "<task>" --in <scope>/` once you know where you're working.

If a returned span is truncated ("+N more lines"), open the file at that exact
range before finalizing. Only open source files when a node genuinely lacks a
needed detail, and then at the exact file:line the node points to — never
re-read whole files.

After big code changes, refresh the graph with `graft build` (deterministic,
no API key, $0).
<!-- graft:end -->
