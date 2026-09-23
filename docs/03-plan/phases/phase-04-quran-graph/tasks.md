# Phase 3 (dir: phase-04) — Task Board — Quran Knowledge Graph

**Phase:** PRD Phase 3 — Quran Graph (directory numbering unchanged; see README.md introduction)
**Source:** `plan.md` §5 (milestones) · ADR-0202 · ADR-0702 (Phase-3 subset)
**Status legend:** ☐ Not Started · ◐ In Progress · ⊘ Blocked · ☑ Done (→ log in `done.md`)
**Est** = engineer-days (ed) — **estimates are provisional until the M1 spike;** do not
compress silently (trap 10). No swimlane owner is assigned yet; owner assignment is an M0
exit requirement.

> Completion protocol: a task is ☑ only when it satisfies `acceptance.md` DoD (§3) and an
> evidence entry is appended to `done.md` in the same commit. Gates (fmt, clippy
> `-D warnings`, tests, arch-check, migrate-check) run per task, not per milestone.

> **Code-provenance note (2026-09-24):** `crates/quran-graph` (2,883 lines) and
> `crates/quran-morphology` (2,534 lines) landed in-tree across 81 unpushed
> local commits while this board still read 0/30 — code is ahead of its
> paperwork. Verified this date: `cargo test -p quran-graph` 34/34 green
> (lib 20 + conformance 6 + no-query-language 2 + structural 1 + traversal 5),
> `cargo test -p quran-morphology` 52/52 green, clippy `-D warnings` clean,
> `cargo fmt --check` clean, `cargo xtask arch-check` OK, `cargo xtask
> migrate-check` OK (19 migrations, incl. graph + morphology + lexicon tables).
> Four tasks move ☐ → ◐ below on that evidence; **none is ☑** (M0 decisions
> unmade, SQLite adapter / application wiring / CLI / HTTP / doctor absent,
> DoD + `done.md` evidence entries pending with the owning session).
> M5 stays ⊘ (no licensed morphology dataset; Phase-2 Sprints 2.4–2.5 at 0 tasks).

---

## 1. Swimlane X — external decisions (start immediately, run in parallel)

| ID | Decision | ADR | Needed by | Owner | Status |
|---|---|---|---|---|---|
| P4-X01 | Ratify ADR-0202 (Proposed → Accepted) | ADR-0202 | Before M1 | _unassigned_ | ☐ |
| P4-X02 | Accept Phase-3 subset of ADR-0702 (snapshot/publication/fencing contract) | ADR-0702 | Before M1 | _unassigned_ | ☐ |
| P4-X03 | Graph query safety limits ADR (budget defaults, exhaustion semantics) | new ADR | Before M3 | _unassigned_ | ☐ |
| P4-X04 | Confirm Phase-2 morphology dataset/ADR-0203 landing — gates M5 and full phase exit, not structural engineering | ADR-0203 | Before M5 | _unassigned_ | ☐ |
| P4-X05 | Export format decision (Graph JSON mandatory; GraphML scope) | new ADR | Before M6 | _unassigned_ | ☐ |

---

## 2. M0 — Contracts & gates

| ID | Task | Deliv. | Depends | Est | Status |
|---|---|---|---|---|---|
| TASK-413 | Schema versioning; stable node/assertion identity; endpoint pairs; direction; assertion-aware duplicate identity; local authorization/review permissions, effective tombstones and historical-edition policy | D4.1 | P4-X01, P4-X02 | 2.0 | ☐ |
| TASK-414 | Edge-vocabulary reconciliation (`PRECEDES`/`FOLLOWS` ↔ `NEXT` alias policy); requirement-to-task matrix; allowlist + manifest updates for graph crates | D4.1 | — | 1.5 | ☐ |

**M0 exit:** accepted decisions; allowlist green (`cargo xtask arch-check`); every upstream
blocker named with an owner.

## 3. M1 — Foundations: port, snapshots, lifecycle

| ID | Task | Deliv. | Depends | Est | Status |
|---|---|---|---|---|---|
| TASK-401 | `GraphStore` port: node resolution, neighbors, bounded paths/subgraphs/patterns, build/inspect/delete-verify, capability discovery; snapshot+authz+budgets+deadline on every query; incomplete-vs-empty; no backend query language leaks | D4.1 | TASK-413 | 3.0 | ☐ |
| TASK-415 | True read transactions (replace pooled `ReadTx`) + bounded bulk canonical reads; restart-safe durable dependency snapshots | D4.1 | TASK-401 | 3.0 | ☐ |
| TASK-416 | Migrations: assertion authority, snapshots, build catalog, adjacency, delivery ledger (numbers from live tree; atomic DDL + ledger insert); harden migration runner transactionality if needed | D4.1 | TASK-413 | 2.5 | ☐ |
| TASK-417 | Atomic build reservation + fenced CAS publication; revalidate complete dependency snapshots before pointer changes; preserve prior release on stale worker completion | D4.9 | TASK-415, TASK-416 | 2.0 | ☐ |
| TASK-418 | Bridge Quran activation/rollback and relevant source changes to atomic revision+audit+outbox intent; durable fan-out, per-target delivery, expired-claim recovery and lease fencing | D4.9 | TASK-416 | 3.0 | ☐ |

**M1 exit:** fault-injection tests show no partial commits; expired worker cannot publish;
snapshot reads never mix revisions. Re-estimate M2–M7 from actuals here.

## 4. M2 — Structural projection (first vertical slice)

| ID | Task | Deliv. | Depends | Est | Status |
|---|---|---|---|---|---|
| TASK-404 | Structural builder: edition/surah/ayah/token/division nodes; `CONTAINS`, `NEXT` with explicit surah-boundary behavior; input-version provenance; rebuild preserves annotations/review history; canonical lookup works with projection missing | D4.2 | TASK-415, TASK-416, TASK-417 | 3.0 | ☐ |
| TASK-419 | Build lifecycle implementation: capture inputs → reserve build → staged resumable batches (durable progress, not in-memory checkpoints) → validate hashes/membership → atomic publication → retain previous verified build | D4.2 | TASK-404 | 3.0 | ☐ |

**M2 exit:** deterministic structural graph; zero dangling edges; cancellation preserves
prior publication. M2 provides internal build/inspection; the first query slice adds
TASK-402: build → inspect → bounded neighbors → canonical quotation.

## 5. M3 — Bounded traversal + conformance

| ID | Task | Deliv. | Depends | Est | Status |
|---|---|---|---|---|---|
| TASK-402 | SQLite adjacency adapter: indexed parameterized node/neighbor reads per ADR-0202 §3; cycle handling, interruption, effective authorization/tombstone/evidence filtering during expansion | D4.1 | TASK-404, P4-X03 | 3.0 | ☐ |
| TASK-403 | Batched frontier traversal for bounded subgraphs, reachability, minimum-hop and up-to-K paths; counters, deterministic ordering, cancellation and explicit incomplete results; optional CTE equivalence if implemented | D4.1 | TASK-402 | 2.5 | ☐ |
| TASK-409 | Backend-agnostic conformance suite: neighbors/paths/subgraph/pattern, budgets, authz-filtered intermediates, incomplete-result semantics; draft harness with TASK-401, complete after query implementations; new adapters must pass before activation | D4.6 | TASK-403, TASK-420 | 3.0 | ☐ |
| TASK-420 | Typed pattern queries: allowlisted predicates, parameterized values, pattern-size budget; reject unsupported operations (no raw query text) | D4.1 | TASK-403 | 2.0 | ☐ |

**M3 exit:** cyclic + high-degree fixtures terminate within limits; hidden intermediates
cannot influence disclosed paths or counts; budget exhaustion ≠ "no path".

## 6. M4 — Annotations, concepts, entities

| ID | Task | Deliv. | Depends | Est | Status |
|---|---|---|---|---|---|
| TASK-408 | Concept/topic projection (`MENTIONS_CONCEPT`) from annotated datasets; interpretive edges carry attribution + evidence; contradictory claims never merged | D4.5 | TASK-409, TASK-421 | 3.0 | ☐ |
| TASK-421 | Authoritative assertion store: immutable/versioned records, evidence links, review decisions, supersession — separate from adjacency; lossless provenance persistence (fix storage-DTO gaps: source location, reviewer, supersession) | D4.5 | TASK-416 | 3.5 | ☐ |
| TASK-422 | Annotation service: create scholar/user edges (`CITES`, `CONTRASTS_WITH`, `PARALLELS`, `EXPLAINS`, …); accept/reject/correct/supersede workflow; reviewer+decision+timestamp retained; computational suggestions representable, never auto-verified | D4.5 | TASK-421 | 3.0 | ☐ |
| TASK-423 | Named-entity annotation support (entity nodes + `REFERS_TO`/`MENTIONS`), attribution required, evidence visible before acceptance | D4.5 | TASK-421 | 2.0 | ☐ |

**M4 exit:** every non-structural edge resolves to attribution + evidence; rebuild
preserves annotations + review history; assertion+audit+revision+outbox commit atomically.

## 7. M5 — Linguistic projections + root family — ⊘ blocked on Phase 2 morphology

| ID | Task | Deliv. | Depends | Est | Status |
|---|---|---|---|---|---|
| TASK-405 | Word-root projection: promote Phase-2 root/lemma/analysis records preserving dataset identity/alignment; hub topology (no ayah×ayah); competing analyses separate | D4.3 | TASK-409, P4-X04, Phase-2 sprints 2.4–2.5 | 3.0 | ⊘ |
| TASK-406 | Root-family tool/CLI: ranked ayahs with provenance, dataset versions, expansion counts; typed budgets; explicit truncation; results cite attribution | D4.4 | TASK-405 | 2.5 | ⊘ |
| TASK-407 | Linguist-reviewed golden fixtures (ق و ل، ك ت ب، ع ل م); doctor root/lemma drift checks; generation stamps | D4.9 | TASK-405 | 2.0 | ⊘ |

**M5 exit:** competing analyses never merge; dataset-version change trips drift; reviewed
families pass. Synthetic fixtures prove mechanics only — never linguistic completeness.

## 8. M6 — Interfaces, export, visualization

| ID | Task | Deliv. | Depends | Est | Status |
|---|---|---|---|---|---|
| TASK-424 | CLI: `qai graph build/inspect/neighbors/path/subgraph/pattern/root-family/export`; explicit build management and confirmed repairs stay CLI-only; thin service dispatch and cancellation | D4.4 | TASK-419, TASK-409, TASK-406, TASK-427 | 2.5 | ☐ |
| TASK-425 | Read-only typed tools `quran.graph_neighbors/path/subgraph/pattern`, root-family and entity-timeline adapters; reproducibility binds projection/build/dataset versions, query, policy and budgets; no build-mutation tool | D4.4 | TASK-409, TASK-406, TASK-423 | 2.0 | ☐ |
| TASK-426 | Read-only HTTP neighbors/path/subgraph/pattern/root-family/entity-timeline and projection inspection routes; missing/stale/incomplete/refused semantics; cache validators bind query+projection+deps+budgets+visibility | D4.4 | TASK-419, TASK-409, TASK-406, TASK-423 | 2.5 | ☐ |
| TASK-427 | Application Graph JSON export service (identities, paths, attribution, versions, truncation); GraphML where practical; bounded output respects tombstones/authz | D4.5 | TASK-409, TASK-408, TASK-423, P4-X05 | 2.0 | ☐ |
| TASK-428 | Basic local visualization: bounded exploration, evidence inspection, provenance labels, RTL, accessible non-graph alternative, reader navigation | D4.5 | TASK-426, TASK-422, TASK-427 | 3.0 | ☐ |

**M6 exit:** CLI/API/tool parity; exports cannot leak restricted evidence; visualization
exposes provenance and limits.

## 9. M7 — Operations, recovery, exit

| ID | Task | Deliv. | Depends | Est | Status |
|---|---|---|---|---|---|
| TASK-410 | Per-graph manifests: projection ID, source/builder versions, snapshot; `qai doctor` per-graph drift; rebuild command wiring | D4.9 | TASK-419 | 2.0 | ☐ |
| TASK-429 | Worker registration; read-only doctor and separately confirmed/audited CLI repair; effective tombstone reconciliation and propagation ledger; GC preserves authority and pinned readers; restore/replay cannot resurrect revoked content | D4.9 | TASK-418, TASK-419, TASK-421 | 2.5 | ☐ |
| TASK-430 | Performance measurements on representative graphs; cancellation tests; progress events; operations runbook | D4.9 | TASK-429 | 2.0 | ☐ |

**M7 exit:** recovery verified; runbook recorded; editorial gates pending with named owners.

## 10. Optional backend spikes (go/no-go; do not block phase exit)

| ID | Task | Deliv. | Depends | Est | Status |
|---|---|---|---|---|---|
| TASK-411 | CozoDB adapter spike + benchmark vs SQLite-CTE; license audit (MPL-2.0); explicit go/no-go; Datalog never leaks | D4.7 | TASK-409 | 2.0 | ☐ |
| TASK-412 | sqlite-graph extension spike + benchmark; maturity bar >1k nodes / stable releases; Cypher never leaks | D4.8 | TASK-409 | 1.5 | ☐ |

---

## 11. Rollup

| Milestone | Tasks | Est (ed) |
|---|---|---|
| M0 | 2 | 3.5 |
| M1 | 5 | 13.5 |
| M2 | 2 | 6.0 |
| M3 | 4 | 10.5 |
| M4 | 4 | 11.5 |
| M5 (blocked) | 3 | 7.5 |
| M6 | 5 | 12.0 |
| M7 | 3 | 6.5 |
| Spikes (optional) | 2 | 3.5 |
| **Required work** | **28** | **71.0** |
| **All work including optional spikes** | **30** | **74.5** |

These are provisional effort sums, not elapsed-time or critical-path estimates. They
exclude external approval and dataset waiting time. Re-baseline after M1. Interface
scaffolding can start against fake backends earlier; the dependencies above gate task
completion, including real morphology and annotation integration.

Task IDs here are **TASK-4xx** per the original TASK-401 convention. Any new task must take
the next unused number and be appended with a dated note — never inserted silently.
