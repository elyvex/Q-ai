# Phase 3 (dir: phase-04) — Completion Ledger — Quran Knowledge Graph

**Phase:** PRD Phase 3 — Quran Graph (directory numbering unchanged — see `README.md`)
**Status:** Planned — 0 / 30 tasks (28 required + 2 optional spikes) · 0 / 28 acceptance criteria
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
| M0 — Contracts & gates | 2 | 0 | 3.5 | — | ☐ |
| M1 — Foundations | 5 | 0 | 13.5 | — | ☐ |
| M2 — Structural projection | 2 | 0 | 6.0 | — | ☐ |
| M3 — Bounded traversal | 4 | 0 | 10.5 | — | ☐ |
| M4 — Annotations/concepts | 4 | 0 | 11.5 | — | ☐ |
| M5 — Linguistic/root family | 3 | 0 | 7.5 | — | ⊘ blocked |
| M6 — Interfaces/export/viz | 5 | 0 | 12.0 | — | ☐ |
| M7 — Operations/exit | 3 | 0 | 6.5 | — | ☐ |
| Spikes (optional) | 2 | 0 | 3.5 | — | ☐ |
| **Total** | **30 (28 required + 2 optional)** | **0** | **71.0 required + 3.5 optional = 74.5** | **—** | **0%** |

| Artifact class | Complete | Total |
|---|---|---|
| Deliverables (D4.1–D4.9) | 0 | 9 (7 required + 2 optional: D4.7/D4.8) |
| Acceptance criteria (AC-P4-01…28) | 0 | 28 |
| ADRs accepted | 0 | 4 planned decisions (0202, 0702 Phase-3 subset, safety limits, export formats); schema versioning belongs to TASK-413 |
| Migrations applied | 0 | est. 4–6 (allocate from live tree) |
| Required test suites green | 0 | see acceptance.md §2 |

Track actual vs estimate from the first completed task and re-baseline M2–M7 after the M1
spike. Under-recorded actuals are how the next phase inherits a wrong capacity model.

---

## 2. Completed Tasks

_None completed yet._

---

## 3. Blocked / Deferred

| Task | Reason | Unblocks when | Owner |
|---|---|---|---|
| TASK-405/406/407 (M5) | Phase-2 morphology datasets, alignment, and analysis policy not implemented (`crates/quran-morphology` placeholder; P2 sprints 2.4–2.5 at 0 tasks) | ADR-0203-licensed dataset + Phase-2 sprints 2.4–2.5 land | _unassigned_ (P4-X04) |
| TASK-411/412 (spikes) | Optional accelerators; phase exit does not depend on them | M3 conformance suite exists | _unassigned_ |

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
