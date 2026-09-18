# Phase 3 (dir: phase-04) — Acceptance Criteria & Exit Gate

**Phase:** PRD Phase 3 — Quran Graph (directory numbering unchanged — see `README.md`)
**Source:** `plan.md` §6 (acceptance highlights), ADR-0202 §Acceptance,
ADR-0702 lines 496–512, PRD `requirements.md` (§10, §10.3–10.6, §43:3880–3892, §54, §75,
§76, §81, §85, §93)
**Ownership:** AC→task mapping in §2b cross-checked against the current `tasks.md` board
(TASK-401–430; IDs frozen)
**Plan criteria:** 28 (AC-P4-01 … AC-P4-28)
**Gate owner:** unassigned (assign at M0 exit)
**Status:** 0 / 28 verified

**Status legend:** ☐ Not verified · ◐ Partially verified · ✗ Failed · ☑ Verified

> **Verification rule:** an AC is ☑ only when an **automated test or scripted check** proves
> it and the evidence artifact (CI run URL, test path, or recording) is recorded in the
> Evidence column and appended to `done.md`. The exit ritual additionally requires a
> **recorded live walkthrough** by a reviewer who is not the implementer.
>
> **Morphology note:** AC-P4-19 and AC-P4-20 — plus the linguistic (root/lemma) portion of
> AC-P4-21 — are ⊘-gated on Phase-2 morphology (P4-X04). Structural and non-linguistic
> interface work can start independently. The phase cannot exit with required evidence missing: pending editorial
> dependencies (dataset licensing, linguist-reviewed goldens) are not compatible with a
> full exit. If ADR-0203 datasets cannot land in time, that is an **explicit owner decision
> to descope or slip the phase**, recorded here — not a silent AC relaxation and not a
> partial exit presented as a full one.

---

## 1. Acceptance Criteria

### 1.1 Integrity & immutability (canonical safety — blocking)

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P4-01** | Every build leaves canonical integrity intact: recomputed `text_hash`, `structure_hash`, `token_order_hash` unchanged before and after, and structural membership/endpoint correctness holds (every structural edge's endpoints exist; membership sets match canonical structure; extends MV-018/QV-028 discipline to graph builds) | Hash + membership/endpoint verification in build validator + test | D4.2 | | ☐ |
| **AC-P4-02** | Graph payloads carry references + hashes only; no second copy of canonical Arabic text in `graph_nodes`/`graph_edges`; quotations resolve through the reader | Schema check + code review + test | D4.2, D4.3 | | ☐ |
| **AC-P4-03** | Deleting and rebuilding any projection preserves authoritative assertions, evidence links, and review history | Delete+rebuild test on populated fixture | D4.2, D4.5 | | ☐ |
| **AC-P4-04** | Canonical lookup (reader/CLI/API) works with every graph projection missing or deleted | Negative integration test | D4.2 | | ☐ |
| **AC-P4-05** | Different editions and competing analyses are never silently merged: same `(source, predicate, target)` supports multiple attributed assertions; each retains distinct identity | Multi-assertion test + schema review | D4.1, D4.3, D4.5 | | ☐ |

> AC-P4-01 implements I8/I14 for graphs; AC-P4-05 implements ADR-0202 §4 and I3/I11 by
> analogy. A rebuild that loses one review row fails this phase — no "the projection is
> rebuildable anyway" excuse.

### 1.2 Build lifecycle, publication, consistency

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P4-06** | Partial, cancelled, or stale builds never replace a verified publication: publication is fenced CAS against the expected current release, and a recovered worker supersedes an expired one | Publication-race + crash fault-injection tests | D4.2, D4.9 | | ☐ |
| **AC-P4-07** | Killing the builder at any batch boundary leaves the published projection unchanged; restart resumes from durable staged progress (not in-memory checkpoints) | Crash matrix over batch checkpoints | D4.2 | | ☐ |
| **AC-P4-08** | Duplicate delivery and crash-after-commit-before-ack are idempotent: no duplicate logical records | Outbox fan-out idempotency tests | D4.9 | | ☐ |
| **AC-P4-09** | Snapshot reads never mix revisions: changing active edition/review/form revision between reads within one build does not alter captured inputs; input snapshot identity records source + dependency versions + builder version | Snapshot-consistency test with mid-read mutation | D4.1, D4.9 | | ☐ |
| **AC-P4-10** | Identical pinned inputs produce identical semantic graph content (memberships, edge identities, provenance); timing/run metadata excluded from equality | Determinism tests, insertion-order variation | D4.2, D4.3 | | ☐ |
| **AC-P4-11** | Graph mutations commit assertion + provenance + revision + audit + outbox atomically (one UoW); injected failure between any pair leaves nothing partially committed | Atomicity fault-injection tests | D4.5 | | ☐ |

> AC-P4-06/07 implement ADR-0702 acceptance items 1–5 (no lost intent, idempotent
> delivery, recovery, fencing, partial builds never activate). AC-P4-09 is the PRD §85
> guarantee adapted to graphs.

### 1.2b Bounded queries & authorization (ADR-0202 §6–7)

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P4-12** | Every neighbor/path/subgraph/pattern query enforces depth, expansion, frontier, result, pattern-size, and wall-clock budgets **during expansion** — a final `LIMIT` alone is insufficient | Budget-exhaustion tests with expansion counters | D4.1 | | ☐ |
| **AC-P4-13** | Cyclic and high-degree fixtures terminate within configured limits; incomplete traversal is explicitly distinguishable from an empty complete result (typed status + truncation reason, never "no path exists") | Conformance suite fixtures | D4.1, D4.6 | | ☐ |
| **AC-P4-14** | Access restrictions and tombstones apply to **every expanded node, traversed edge, and evidence dependency** — hidden intermediates cannot influence disclosed paths, counts, or exports | Authz-filtered-intermediate conformance tests | D4.1, D4.5 | | ☐ |

### 1.3 Provenance & annotations

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P4-15** | Every **non-structural** edge resolves to full attribution + evidence: source ID + location, author/algorithm, version, confidence, verification status, created_at (PRD §10.3) | Schema completeness test over all edge families | D4.5 | | ☐ |
| **AC-P4-16** | Structural edges carry deterministic builder + input-version provenance, distinguishable from interpretive assertions in every result and export | Representation test + export inspection | D4.2 | | ☐ |
| **AC-P4-17** | Review workflow: computational suggestions require explicit human accept/reject/correct; accepted suggestions retain reviewer + decision + timestamp; suggestions can never self-promote to verified | Workflow state-machine tests | D4.5 | | ☐ |
| **AC-P4-18** | Graph results explain themselves: start/end nodes, traversed path, edge types + provenance, applied filters, snapshot/build identity, completion status (PRD §10.5) | Contract test over all query types | D4.1 | | ☐ |

### 1.4 Linguistic graph (⊘ gated on Phase-2 morphology)

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P4-19** | Root/lemma identities originate only in attributed, versioned morphology datasets; the graph never invents/guesses roots; normalization does not create roots | Code review + negative test (no analyzer → typed error, not invented edges) | D4.3 | | ☐ |
| **AC-P4-20** | Root-family query for reviewed fixtures (ق و ل، ك ت ب، ع ل م) returns the expected ayah families with dataset attribution and separate competing analyses | Linguist-reviewed golden fixtures | D4.3, D4.4 | | ☐ |
| **AC-P4-21** | `qai doctor` detects per-graph drift independently — including morphology dataset version change — and reports each projection's staleness separately | Drift-injection test per projection | D4.9 | | ☐ |

### 1.5 Interfaces, export, visualization, ops

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P4-22** | CLI, HTTP API, and typed tools expose consistent **read** semantics for the same core operations — neighbors, path, subgraph, pattern, root-family — with consistent budgets and diagnostics; build lifecycle management is CLI-only, and this phase requires no new mutation agent tools and no new HTTP mutation routes | CLI trycmd + HTTP contract + tool tests over shared fixtures (read ops) | D4.4, D4.9 | | ☐ |
| **AC-P4-23** | Graph JSON export (GraphML where practical) contains identities, paths, attribution, dependency versions, and explicit truncation; exports respect tombstones/access policy; no unbounded dumps | Export tests + policy-leak tests | D4.5 | | ☐ |
| **AC-P4-24** | Basic local visualization supports bounded exploration, evidence inspection, provenance-category labels, RTL Arabic, and an accessible non-graph alternative; canonical reading never depends on it | Browser/accessibility checks + reader-independence test | D4.5 | | ☐ |

> AC-P4-24 is deliberately minimal (Phase-3 "visualization" per requirements.md:3733/3891).
> The interactive Phase-4 explorer remains Phase-4 scope; browser tests are **new
> infrastructure** and part of this phase's verification work, not assumed.

### 1.6 Operations: doctor, tombstones, performance, migrations

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P4-25** | `qai doctor` graph checks are strictly **read-only**; repairs require a separately invoked explicit CLI command with confirmation and audit — doctor never mutates as a side effect of checking | Read-only connection + before/after persisted-state comparison on drifted fixtures; explicit repair command tests | D4.9 | | ☐ |
| **AC-P4-26** | Tombstone reconciliation prevents stale publications, replay, rebuild or restore from resurrecting deleted content or revoked access; GC preserves authoritative assertions, evidence, review history and pinned readers while reclaiming eligible projections; restored snapshots reconcile current tombstones before becoming readable | Tombstone reconciliation + GC crash/retry + old-backup restore/rebuild tests, with pinned readers and revoked evidence | D4.9 | | ☐ |
| **AC-P4-27** | Representative-scale builds and queries meet recorded performance budgets; query cancellation stops backend expansion and build cancellation at batch boundaries preserves prior publication; builds execute through registered job workers with durable progress and observable progress events | Representative benchmark + query/build cancellation tests + worker registration/execution and progress-event contract tests | D4.1, D4.9 | | ☐ |
| **AC-P4-28** | Graph schema upgrades commit DDL, data changes and migration-ledger entries atomically per migration; failed/interrupted migrations preserve the prior committed state and retry safely; successful upgrades preserve existing data | Migration fault-injection + rollback/retry + populated-database upgrade tests | D4.1 | | ☐ |

> AC-P4-25/26/27 correspond to TASK-429/430 scope (read-only doctor, GC/restore safety,
> performance/cancellation/progress). AC-P4-28 hardens TASK-416's migration work; the
> structural membership/endpoint correctness it would otherwise duplicate is covered by
> AC-P4-01.

---

## 2. Required test suites (must be green before exit)

| Suite | Proves | Location (planned) |
|---|---|---|
| Backend conformance (`TASK-409`) | AC-P4-12/13/14 for every adapter | `crates/graph/tests/conformance/` |
| Snapshot & revision consistency | AC-P4-09 | `crates/application/tests/graph_snapshot.rs` |
| Publication races & crash matrix | AC-P4-06/07 | `crates/application/tests/graph_build.rs` |
| Determinism & identity | AC-P4-05/10 | `crates/quran-graph/tests/determinism.rs` |
| Atomicity fault injection | AC-P4-11 | `crates/storage-sqlite/tests/` |
| Migration recovery & upgrade atomicity | AC-P4-28; M1 exit; upgrade safety | `crates/storage-sqlite/tests/` |
| Structural membership/endpoint correctness | AC-P4-01 (membership + endpoint validation) | `crates/quran-graph/tests/` |
| CLI trycmd (graph) | AC-P4-22 | `crates/cli/tests/graph/` |
| HTTP contract + OpenAPI (read ops) | AC-P4-22/23 | `crates/server/tests/` |
| Tool contracts (read ops) | AC-P4-22 | `crates/application/tests/graph_tools.rs` |
| Doctor immutability + read-only guarantee | AC-P4-21, AC-P4-25 | `crates/cli/src/doctor.rs` tests |
| Tombstone GC/restore round-trip | AC-P4-26 | `crates/storage-sqlite/tests/` + application |
| Performance/cancellation/progress events | AC-P4-27 | `crates/jobs/tests/` + runbook measurements |
| Export policy-leak | AC-P4-23 | `crates/server/tests/` / application |
| Visualization browser/a11y | AC-P4-24 | new M6 test infra |
| Canonical reference-only payloads, rebuild preservation, reader independence | AC-P4-02/03/04 | `crates/quran-graph/tests/` + application integration tests |
| Durable outbox fan-out and replay | AC-P4-08 | `crates/application/tests/` + storage integration tests |
| Attribution, structural provenance, human review | AC-P4-15/16/17 | `crates/quran-graph/tests/` + application integration tests |
| Explainable query contracts | AC-P4-18 | `crates/graph/tests/conformance/` + transport contract tests |
| Dataset-qualified roots and reviewed family goldens | AC-P4-19/20 | `crates/quran-graph/tests/` + application integration tests |

---

## 2b. AC → task ownership (current board)

Every AC must map to an owning task on the `tasks.md` board. Preserve existing task IDs;
if scope moves between tasks, update this mapping. Individual owners remain unassigned.

| AC | Owning task(s) | Notes |
|---|---|---|
| AC-P4-01 | TASK-404, TASK-414, TASK-419 | hash, membership, endpoint and edge-vocabulary validation |
| AC-P4-02 | TASK-413, TASK-402 | schema constraint + adapter enforcement |
| AC-P4-03 | TASK-404, TASK-419 | delete+rebuild preservation |
| AC-P4-04 | TASK-404 | negative integration test |
| AC-P4-05 | TASK-413, TASK-408 | multi-assertion schema + concept edges |
| AC-P4-06 | TASK-417, TASK-418 | fenced CAS publication + lease fencing |
| AC-P4-07 | TASK-417, TASK-419 | reservation + durable staged batches |
| AC-P4-08 | TASK-418 | outbox fan-out idempotency |
| AC-P4-09 | TASK-415, TASK-416 | true read txns + durable snapshots |
| AC-P4-10 | TASK-404, TASK-419 | determinism tests |
| AC-P4-11 | TASK-421, TASK-416 | assertion-store atomic UoW |
| AC-P4-12 | TASK-401, TASK-402, TASK-403, TASK-420 | port, frontier and typed-pattern budget enforcement |
| AC-P4-13 | TASK-402, TASK-409 | conformance fixtures |
| AC-P4-14 | TASK-409 | authz-filtered intermediates |
| AC-P4-15 | TASK-405, TASK-408, TASK-421, TASK-422, TASK-423 | attribution + evidence on linguistic, concept, annotation and entity edges |
| AC-P4-16 | TASK-404, TASK-419 | structural provenance representation |
| AC-P4-17 | TASK-422 | review workflow state machine |
| AC-P4-18 | TASK-401, TASK-426 | self-describing results via port + HTTP |
| AC-P4-19 | TASK-405 | dataset-sourced identities only (⊘ Phase-2) |
| AC-P4-20 | TASK-405, TASK-407 | golden fixtures (⊘ Phase-2) |
| AC-P4-21 | TASK-407, TASK-410 | per-graph drift incl. morphology portion (⊘ Phase-2 for linguistic part) |
| AC-P4-22 | TASK-406, TASK-424, TASK-425, TASK-426 | read-parity CLI/tools/HTTP including root-family |
| AC-P4-23 | TASK-427 | export correctness |
| AC-P4-24 | TASK-428 | minimal visualization |
| AC-P4-25 | TASK-429 | read-only doctor + explicit repair |
| AC-P4-26 | TASK-429 | tombstone GC/restore safety |
| AC-P4-27 | TASK-402, TASK-419, TASK-424, TASK-429, TASK-430 | query/build cancellation, worker execution, performance budgets and progress |
| AC-P4-28 | TASK-416 | migration atomicity |

---

## 3. Definition of Done (per task)

A task is ☑ only when **all** of the following hold:

1. Implementation merged with `fmt`, `clippy -D warnings`, targeted tests green.
2. `cargo xtask arch-check` and `cargo xtask migrate-check` pass (allowlist + migrations
   updated in the same change when needed).
3. Tests added/updated prove the behavior; fault-injection for lifecycle tasks.
4. Board status flipped and `done.md` entry appended with evidence in the same commit.
5. New error codes registered (never renumbered); new public commands/routes documented.
6. Invariants I1–I16 spot-checked for the touched surface (esp. I1, I2, I5, I8, I14, I16).

---

## 4. Exit gate

- All AC-P4-01…28 ☑ with recorded evidence (AC-P4-19/20 and the linguistic portion of
  AC-P4-21 require the Phase-2 dependency to be resolved or an explicitly recorded owner
  decision to descope — the latter is a gate change, not an omission; missing required
  evidence is never compatible with a full exit).
- Full gate: `cargo xtask ci` (9-step) green.
- Live walkthrough recorded by a **non-implementer** reviewer.
- `phase-04-quran-graph/done.md` rollup complete; `task-done-rollup.md` updated;
  `CHANGELOG.md` entry; open follow-ups filed in `docs/05-followups/`.
- ADRs accepted: 0202, 0702 (Phase-3 subset), query-safety limits, export formats —
  exactly these four; schema versioning is TASK-413 work, not an additional invented ADR.
  None of these are accepted by default: they remain **Planned/Proposed** until their
  swimlane owners (P4-X01/02/03/05) ratify them.
- Editorial/owner items that remain open (dataset licensing, linguist-reviewed goldens) are
  listed as pending with named owners — they cannot be simulated or closed by an agent.
