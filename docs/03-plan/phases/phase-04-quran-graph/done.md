# Phase 3 (dir: phase-04) — Completion Ledger — Quran Knowledge Graph

**Phase:** PRD Phase 3 — Quran Graph (directory numbering unchanged — see `README.md`)
**Status:** In progress — 0 / 30 ☑ (10 ◐ code-landed-partial, 3 ⊘ M5 blocked) · 0 / 28 acceptance criteria
**Started:** —
**Completed:** —

---

## How To Use This File

This is an **append-only ledger**. It records what was actually completed, when, by whom,
and with what evidence.

**Rules**

1. **Append only.** Never edit or delete an existing entry. If an entry was wrong, append a
   correction in §6 referencing the original.
2. **Evidence is mandatory.** Every entry links a PR, CI run, test path, or recording. An
   entry without evidence is not a completion record.
3. **DoD before ☑.** A task moves to done only when it satisfies the Definition of Done
   (`acceptance.md` §3) — not when it compiles or when the PR merges.
4. **Mirror the board.** When you append here, flip the status in `tasks.md` or
   `acceptance.md` in the same commit. The two must never disagree.
5. **Record deviations.** If the delivered thing differs from `plan.md`, log it in §5 with
   the reason. Silent drift is how an integrity phase stops being verifiable.

**Entry format**

```
### TASK-4nn — <task title>
- **Deliverable:** D4.x
- **Completed:** YYYY-MM-DD
- **Owner:** <name> (<role>)
- **PR / commit:** <link>
- **Evidence:** <CI run, test path, snapshot, or recording>
- **DoD:** all items / exceptions: <list with justification>
- **Notes:** <deviations, follow-ups, TODOs filed>
```

---

## 1. Progress Summary

| Milestone | Tasks | Done | Est (ed) | Actual (ed) | Status |
|---|---|---|---|---|---|
| X — External decisions (swimlane, not tasks) | 5 items | 0 | — | — | ☐ |
| M0 — Contracts & gates | 2 | 0 ☑ / 2 ◐ (TASK-413/414) | 3.5 | — | ◐ |
| M1 — Foundations | 5 | 0 ☑ / 2 ◐ (TASK-401/416) | 13.5 | — | ◐ code ahead of board |
| M2 — Structural projection | 2 | 0 ☑ / 1 ◐ (TASK-404) | 6.0 | — | ◐ |
| M3 — Bounded traversal | 4 | 0 ☑ / 3 ◐ (TASK-403/409/420) | 10.5 | — | ◐ |
| M4 — Annotations/concepts | 4 | 0 | 11.5 | — | ☐ |
| M5 — Linguistic/root family | 3 | 0 | 7.5 | — | ⊘ blocked |
| M6 — Interfaces/export/viz | 5 | 0 ☑ / 2 ◐ (TASK-424/T427) | 12.0 | — | ◐ |
| M7 — Operations/exit | 3 | 0 | 6.5 | — | ☐ |
| Spikes (optional) | 2 | 0 | 3.5 | — | ☐ |
| **Total** | **30 (28 required + 2 optional)** | **0 ☑ / 10 ◐** | **71.0 required + 3.5 optional = 74.5** | **—** | **0% ☑; 10 partial** |

| Artifact class | Complete | Total |
|---|---|---|
| Deliverables (D4.1–D4.9) | 0 | 9 (7 required + 2 optional: D4.7/D4.8) |
| Acceptance criteria (AC-P4-01…28) | 0 | 28 |
| ADRs accepted | 0 | 4 required (0202, 0702 Phase-3 subset, safety limits, export formats); schema versioning belongs to TASK-413. Drafts landed 2026-09-24: ADR-0217 (query limits), ADR-0218 (export formats) — both Proposed, owner ratification outstanding (P4-X03/P4-X05 ◐) |
| Migrations applied | 1 | 4–6 (only `0019_quran_graph` adjacency/catalog skeleton; assertion authority, dependency snapshots, build catalog and delivery ledger still missing — TASK-416) |
| Required test suites green | 0 | see acceptance.md §2 |

Track actual vs estimate from the first completed task and re-baseline M2–M7 after the M1
spike. Under-recorded actuals are how the next phase inherits a wrong capacity model.

---

## 2. Completed Tasks

_None completed yet (0 ☑). Partial work landed without task closure is tracked
in `tasks.md` (◐ rows) and summarised here:_

- **TASK-401 ◐** — `crates/quran-graph/src/store.rs` (240 lines) `GraphStore`
  port: node resolution, neighbors, bounded paths/subgraphs/patterns,
  build-staging/inspect/capabilities, `AuthzScope`, `EdgeFilter`, `Direction`,
  cancellation helper. Reference backend `src/mem.rs` (574 lines).
  No-query-language guard: `tests/no_query_language.rs` (2 tests).
- **TASK-403 ◐** — `src/traverse.rs` (441 lines): bounded subgraph, reachability,
  min-hop, up-to-K paths with expansion counters, deterministic ordering,
  cancellation, typed incomplete results. Tests: `tests/traversal.rs` (5, incl.
  cyclic + high-degree star termination).
- **TASK-404 ◐** — `src/structural.rs` (333 lines): edition/surah/ayah/token/
  division nodes, `CONTAINS`/`NEXT` with explicit surah-boundary behavior,
  stable IDs, input-version provenance, no canonical text embedded
  (`structural::tests::no_canonical_text_embedded`). Golden fixture:
  `fixtures/quran/graph/mini-structural.json` via
  `tests/structural.rs::mini_corpus_golden_node_and_edge_sets`.
  Durable resumable build lifecycle / fenced publication / annotation
  preservation: **not implemented** (TASK-415/416/417/419).
- **TASK-409 ◐** — `tests/conformance.rs` (8 tests): neighbors/paths/subgraph/
  pattern, budget exhaustion ≠ empty, authz-filtered intermediates, build
  inspect + capabilities, plus (added 2026-09-24 with ADR-0217) out-of-range
  budgets rejected at every port entry point and authz applied during pattern
  expansion. Draft harness: only `MemGraphStore` has passed it.
- **TASK-420 ◐** — `src/pattern.rs` (307 lines): typed allowlisted predicates,
  parameterized values, `PATTERN_SIZE_BUDGET`, unsupported-operation rejection.
- **TASK-427 ◐** — `src/export.rs` (179 lines): Graph JSON export with
  identities/attribution/versions plus `ExportNotice` truncation, tombstone and
  authz filtering; plus two ADR-0218 tests added 2026-09-24 (interpretive edges
  travel with their assertion/evidence record; a restricted assertion is absent
  from the serialized bytes, not merely from the edge list). GraphML +
  application-level export service: not implemented.
- **TASK-424 ◐** — CLI dispatch in `crates/application/src/quran_cli.rs` and
  `crates/cli/src/quran.rs` now covers structural build/inspect, neighbors, path,
  root-family, and Graph JSON export over a file-backed in-memory projection.
  Subgraph/pattern commands, durable build management, repair confirmation, and
  production persistence remain absent.
- **P4-X03 / P4-X05 ◐ (decision drafts, 2026-09-24)** —
  `docs/02-architecture/decisions/ADR-0217-graph-query-limits.md` records the
  budget defaults, validation ranges and "truncated ≠ empty" exhaustion
  semantics that `QueryBudgets` already implements;
  `ADR-0218-graph-export-formats.md` records `quran-graph-json-v1` as the
  mandatory lossless format (assertions + truncation + policy filtering) and
  GraphML as a declared-lossy derived view. Both are **Proposed**: the
  ratification itself is owner work (P4-X03 before M3, P4-X05 before M6) and no
  agent may flip either status.
- **TASK-413 ◐** — stable node IDs (`edition:…`, `surah:…`, `ayah:…`,
  `token:…`), the allowlisted edge vocabulary, `AuthzScope` visibility, and
  effective-tombstone filtering landed in `quran-graph`. Schema-version
  constants, assertion-aware duplicate identity, review permissions, and the
  historical-edition policy remain unmade (M0 owner decisions).
- **TASK-414 ◐** — the 15-entry `EDGE_VOCABULARY` names `NEXT` canonically and
  rejects `PRECEDES`/`FOLLOWS` (`tests/no_query_language.rs`, staging validates
  predicates). The requirement-to-task matrix and manifest notes remain.
- **TASK-416 ◐** — `migrations/sqlite/0019_quran_graph.up.sql` creates
  `graph_projections`, `graph_build_progress`, `graph_nodes`, `graph_edges`, and
  `graph_assertions` (assertion authority, build catalog, adjacency) with
  checksum pinning (`migrate-check` 19 ordered, stable). Dependency snapshots,
  the durable delivery ledger, and migration-runner transaction hardening
  remain.

Gates run 2026-09-24 for the above: `cargo test -p quran-graph` 38/38 green
(lib 22 + conformance 8 + no-query-language 2 + structural 1 + traversal 5),
`cargo clippy -p quran-graph --all-targets -- -D warnings` clean,
`cargo fmt --check -p quran-graph` clean, `cargo xtask arch-check` OK,
`cargo xtask migrate-check` OK (19 migrations, incl. `0019_quran_graph`).
Commits: `5a32a15` (crate), `d4ed983`/`093d225`/`496903c`/`b8baad0` (tests),
`27d796d` (refactor), `8380acd` (golden fixture). None is pushed (`main` is
~70+ commits ahead of `origin/main` as of this entry; the exact count moves as
concurrent sessions commit).

---

## 3. Blocked / Deferred

| Task | Reason | Unblocks when | Owner |
|---|---|---|---|
| TASK-405/406/407 (M5) | **Correction 2026-09-24:** the code is no longer a placeholder — `crates/quran-morphology` has 2,534 lines (adapters JSON/CSV, align, compare, dataset, family, policy, tagset, validate) with 52/52 tests green, plus ADRs 0209–0215 recorded. The Phase-2 board now records morphology mechanics as 8 ☑ / several ◐ (T57–T74), but the *evidence* gate remains: no ADR-0203-licensed production dataset, no linguist-reviewed goldens, and no graph root-family projection. Mechanics may be exercised on synthetic fixtures only; no graph M5 task closes. | ADR-0203-licensed dataset + linguist sign-off + graph root-family projection | _unassigned_ (P4-X04) |
| TASK-411/412 (spikes) | Optional accelerators; phase exit does not depend on them | M3 conformance suite stable after a real adapter ships | _unassigned_ |
| TASK-401/403/404/409/420/424/427 → ☑ | Implemented only as pure `quran-graph`/in-memory or file-backed CLI groundwork; no SQLite adapter, durable application lifecycle, HTTP/tools, doctor, or full interface parity | SQLite adapter + application services + `done.md` evidence per task | _unassigned_ |

---

## 4. Acceptance Criteria Status

_Tracked in `acceptance.md` — 0 / 28 verified. Keep the two files in sync._

Required morphology evidence gates M5 and full phase exit, not structural work starting.
Pending required editorial evidence is a blocker, not a completed exit. All ADR decisions
remain planned/proposed until explicitly accepted; no implementation completion is recorded.

---

## 5. Deviations from plan

_None recorded yet._

---

## 6. Corrections

_None recorded yet._

---

## 7. Planning provenance

- **2026-09-17:** Plan authored from code + requirements inspection (static analysis; gates
  not run at planning time). Key inputs: PRD `requirements.md:3724–3733`, `:3880–3892`,
  `:529–667`, `:789–807`, `:6208–6227`; ADR-0202 (Proposed); ADR-0702 Phase-3 subset
  (Proposed); code findings — placeholder graph crates, pooled `ReadTx`
  (storage-sqlite/src/lib.rs:195), separate Quran pointer generation
  (storage-sqlite/src/quran.rs:682), consumer-less outbox relay
  (storage/src/workflows.rs:112), index-builder publication gaps
  (application/src/quran_index.rs:262–519), failure-only worker checkpoints
  (jobs/src/worker.rs:128), debug-reader-only web surface (server/src/api.rs:694).
- **2026-09-17:** TASK-413–430 allocated (next free numbers after TASK-401–412).
  Original TASK-401–412 IDs preserved.
