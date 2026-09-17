# Phase 4 — Quran Knowledge Graph

Status: Proposed (planning); no graph implementation delivered (reviewed 2026-09-17)
Depends on: Phase 1 (canonical corpus) + Phase 2 morphology datasets/FTS

`crates/graph`, `crates/quran-graph` and `crates/isnad-graph` remain placeholders.
The backend policy and deliverables below are proposals, not installed adapters or
available CLI commands. Phase-2 morphology datasets are not yet implemented.

This directory uses Phase 4 numbering while the PRD roadmap places Quran graph work
in Phase 3; numbering reconciliation remains open. See the
[project README](../../../../README.md#phase-tracking) for current implementation status.

## Objective

Build the typed Quran knowledge graph behind the backend-neutral
`GraphStore` port (ADR-0202), including the required **word-root graph**
connecting ayahs that share an Arabic root (e.g., ق و ل → قال / قل / قالوا /
قلتم / يقول), plus the other graph families listed in ADR-0202 §8.

## Backend policy (from ADR-0202)

- Default backend: SQLite adjacency tables + bounded recursive CTEs /
  batched frontier traversal.
- Optional derived backends behind the same port:
  - **CozoDB** — Rust, Datalog recursion, MVCC; evaluate as accelerator for
    transitive/recursive workloads.
  - **SQLite graph extension** (e.g., `sqlite-graph`) — Cypher inside SQLite;
    evaluate only when mature (currently alpha, ~1k-node tested).
- SQLite remains the single source of truth in every configuration; all graph
  backends are rebuildable projections with generation stamps.
- No backend query language (SQL/Cypher/Datalog) escapes the `GraphStore` port.

## Deliverables

| ID | Deliverable |
|---|---|
| D4.1 | `GraphStore` port + SQLite-CTE adapter, bounded budgets |
| D4.2 | Structural graph projection (surah/ayah/token hierarchy) |
| D4.3 | **Word-root graph projection** (root/lemma/word-form/token/ayah) |
| D4.4 | Root-family traversal tool (`qai graph root-family ق و ل`) with ranked ayah results |
| D4.5 | Concept graph (annotated, Phase-2+ datasets) |
| D4.6 | Backend conformance test suite (all adapters must pass) |
| D4.7 | CozoDB adapter spike + benchmark vs SQLite-CTE (go/no-go) |
| D4.8 | SQLite-graph-extension spike + benchmark (go/no-go) |
| D4.9 | Per-graph manifests, `qai doctor` drift checks |
