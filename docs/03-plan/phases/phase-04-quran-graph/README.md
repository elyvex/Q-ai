# Phase 3 (dir: phase-04) — Quran Knowledge Graph

Status: **Planned** — planning artifacts authored 2026-09-17 from code + requirements
inspection; no graph implementation delivered. This directory carries PRD **Phase 3**
numbering internally as P4 (`TASK-4xx`, `D4.x`, `AC-P4-xx`); the PRD roadmap places Quran
graph work at Phase 3 and numbering reconciliation remains open — do not renumber by hand
(agent briefing §14.5).

Depends on: Phase 1 canonical corpus + Phase 2 morphology datasets.
Morphology gates **M5 and full phase exit**, but not the start of structural work.
Required dataset licensing and linguist-reviewed evidence cannot remain pending at full exit.

The proposed backend policy below follows ADR-0202; ADR-0202 and the Phase-3 subset of
ADR-0702 remain **Proposed**, with ratification tracked by P4-X01/P4-X02. Safety limits
and export formats are the other two planned ADR decisions (P4-X03/P4-X05).
Schema versioning belongs to TASK-413, not a fifth ADR.

## Planning artifacts

| File | Contents |
|---|---|
| [`plan.md`](plan.md) | Full implementation plan: findings, scope, crate ownership, milestones M0–M7, risks |
| [`tasks.md`](tasks.md) | Task board: TASK-401–412 (original) + TASK-413–430 (new work packages), swimlane X, estimates |
| [`acceptance.md`](acceptance.md) | AC-P4-01…28 matrix, AC/task ownership, required test suites, DoD, exit gate |
| [`done.md`](done.md) | Append-only completion ledger (no completed tasks; planning provenance recorded) |

**Totals:** 30 tasks (TASK-401–430): 28 required / 71 engineer-days, plus 2 optional /
3.5 engineer-days; 74.5 engineer-days overall. M1 contains 5 tasks / 13.5 engineer-days.
All 28 acceptance criteria remain unverified.

## Objective

Build the typed Quran knowledge graph behind the backend-neutral
`GraphStore` port (ADR-0202), including the required **word-root graph**
connecting ayahs that share an Arabic root (e.g., ق و ل → قال / قل / قالوا /
قلتم / يقول), plus the other graph families listed in ADR-0202 §8.

## Backend policy (from ADR-0202)

- Default backend: SQLite adjacency tables with **batched frontier traversal**.
  Bounded recursive CTEs are an optional optimization only where in-expansion budgets
  and equivalent semantics can be proved; delivering two traversal engines is not required.
- Optional derived backends behind the same port:
  - **CozoDB** — evaluate as an accelerator for recursive workloads (TASK-411).
  - **SQLite graph extension** (e.g., `sqlite-graph`) — evaluate as an accelerator
    only if the spike supports a go decision (TASK-412).
  - Recheck current releases, licensing, maintenance, supported capabilities, scale, and
    safety in each spike; this plan assumes no verified current backend maturity.
- SQLite remains the single source of truth in every configuration; all graph
  backends are rebuildable projections with generation stamps.
- No backend query language (SQL/Cypher/Datalog) escapes the `GraphStore` port.

## Deliverables

| ID | Deliverable |
|---|---|
| D4.1 | `GraphStore` port + SQLite adjacency adapter with batched frontier traversal and bounded budgets; bounded CTE optimization optional |
| D4.2 | Structural graph projection (edition/surah/ayah/token/division membership and endpoints) |
| D4.3 | **Word-root graph projection** (root/lemma/word-form/token/ayah) |
| D4.4 | CLI/API/typed-tool read operations: neighbors, path, subgraph, pattern, root-family with ranked ayah results; CLI-only build management |
| D4.5 | Authoritative annotations/review, concept and named-entity projections, bounded export, basic local visualization |
| D4.6 | Backend conformance test suite (every activated adapter must pass) |
| D4.7 | **Optional:** CozoDB adapter spike + benchmark against the default SQLite adapter (go/no-go) |
| D4.8 | **Optional:** SQLite-graph-extension spike + benchmark (go/no-go) |
| D4.9 | Per-graph manifests, read-only doctor, explicit repair, worker execution/progress, cancellation, tombstone reconciliation, GC/restore, performance budgets |

## Scope summary (see plan.md §2–3 for full lists)

**In scope:** structural + linguistic + annotated projections, bounded
neighbors/paths/subgraphs/patterns, authoritative annotations with review
history, CLI/HTTP/tool read parity, Graph JSON export, basic local
visualization, per-projection manifests + read-only doctor checks and explicit repair.
Build management is CLI-only; no new mutation agent tools or HTTP mutation routes are required.

**Out of scope:** hadith/tafsir/isnad ingestion (Phases 5–8), LLM-generated
relationships (I2 — canonical path stays model-free), production auth/RBAC
(Phase 11), full Phase-4 explorer, backend query-language exposure.

**Critical path:** M0 → M1 → M2 → M3 → M6 → M7; M4 branches after M1; M5
(word-root) waits on Phase-2 morphology. The first usable vertical slice is
build → inspect → bounded neighbors → canonical quotation.
