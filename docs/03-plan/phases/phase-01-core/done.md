# Phase 1 — Completion Ledger

**Phase:** P1 — Canonical Quran Core
**Status:** 🔴 Not Started — 0 / 60 tasks · 0 / 21 acceptance criteria · 0 / 14 ADRs · 0 / 6 migrations
**Started:** _not started_
**Completed:** —

---

## How To Use This File

This is an **append-only ledger**. It records what was actually completed, when, by whom, and
with what evidence.

**Rules**

1. **Append only.** Never edit or delete an existing entry. If an entry was wrong, append a
   correction in §6 referencing the original.
2. **Evidence is mandatory.** Every entry links a PR, CI run, test path, or recording. An entry
   without evidence is not a completion record.
3. **DoD before ☑.** A task moves to done only when it satisfies the Definition of Done
   (`acceptance.md` §3.1) — not when it compiles or when the PR merges.
4. **Mirror the board.** When you append here, flip the status in `tasks.md` or
   `acceptance.md` in the same commit. The two must never disagree.
5. **Record deviations.** If the delivered thing differs from `plan.md`, log it in §5 with the
   reason. Silent drift is how a canonical-text phase stops being canonical.

**Entry format**

```
### P1-Tnn — <task title>
- **Deliverable:** D1.x
- **Completed:** YYYY-MM-DD
- **Owner:** <name> (<role>)
- **PR / commit:** <link>
- **Evidence:** <CI run, test path, snapshot, or recording>
- **DoD:** ✅ all items / ⚠️ exceptions: <list with justification>
- **Notes:** <deviations, follow-ups, TODOs filed>
```

---

## 1. Progress Summary

| Sprint | Tasks | Done | Est (ed) | Actual (ed) | Status |
|---|---|---|---|---|---|
| X — External-lead-time decisions | 5 | 0 | — | — | ☐ |
| 1.0 — Data & Decisions | 5 | 0 | 11.5 | — | ☐ |
| 1.1 — Domain & Addressing | 9 | 0 | 19.5 | — | ☐ |
| 1.2 — Import & Validation | 17 | 0 | 42.0 | — | ☐ |
| 1.3 — Reader, Translations, API | 11 | 0 | 21.5 | — | ☐ |
| 1.4 — Tools, Citations, CLI, Doctor | 11 | 0 | 21.5 | — | ☐ |
| 1.5 — Debug Reader, Hardening, Exit | 7 | 0 | 15.0 | — | ☐ |
| **Total** | **60 + 5** | **0** | **131.0** | **—** | **0%** |

| Artifact class | Complete | Total |
|---|---|---|
| Deliverables (D1.1–D1.14) | 0 | 14 |
| Acceptance criteria (AC-P1-01…21) | 0 | 21 |
| ADRs accepted | 0 | 14 |
| Migrations applied (`0010`–`0015`) | 0 | 6 |
| Required test suites green | 0 | 15 |
| D1.14 documents published | 0 | 5 |

Track **actual vs. estimate** from the first completed task. `tasks.md` §9.1 flags a **~49 ed
discrepancy** between the plan's stated 82 ed and its own task rows (131.0 ed); actuals recorded
here are the only way to learn which number was closer. Record actuals even when they exceed the
estimate — an under-recorded sprint is how the next phase inherits a wrong capacity model.

---

## 2. Completed Tasks

_No tasks completed yet._

**Entry format (repeat per task)**

```
### P1-Tnn — <task title>
- **Deliverable:** D1.x
- **Completed:** YYYY-MM-DD
- **Owner:** <name> (<role>)
- **PR / commit:** <link>
- **Evidence:** <CI run, test path, snapshot, or recording>
- **DoD:** ✅ all items / ⚠️ exceptions: <list with justification>
- **Notes:** <deviations, follow-ups, TODOs filed>
```

### Sprint 1.0 — Data & Decisions

### Sprint 1.1 — Domain & Addressing

### Sprint 1.2 — Import & Validation

### Sprint 1.3 — Reader, Translations, API

### Sprint 1.4 — Tools, Citations, CLI, Doctor

### Sprint 1.5 — Debug Reader, Hardening, Exit

---

## 3. Verified Acceptance Criteria

_None verified yet._

**Entry format**

```
### AC-P1-nn — <criterion short name>
- **Verified:** YYYY-MM-DD
- **Verified by:** <reviewer name> (must not be the implementer for ritual ACs)
- **Method:** <test path / scripted check / live walkthrough>
- **Evidence:** <CI run URL, artifact, or recording timestamp>
- **Result:** ☑ Pass
- **Notes:** <caveats, re-verification triggers>
```

Live-walkthrough ACs (§5 ritual: AC-P1-02, 03, 05/06, 07, 08, 04, 10, 11, 12) additionally
require the recording link and the reviewer's name, and must be verified by someone other than
the implementer.

| ID | Criterion | Verified | By | Evidence |
|---|---|---|---|---|
| AC-P1-01 | ADR-0101 accepted with licensed, editorially signed-off edition | — | — | — |
| AC-P1-02 | 🎥 Import → `Staged`, zero `Fatal` findings | — | — | — |
| AC-P1-03 | 🎥 Import cannot activate; human approval required | — | — | — |
| AC-P1-04 | 🎥 All QV-001…028 implemented; 16 adversarial fixtures reject with specific ids | — | — | — |
| AC-P1-05 | 🎥 Every ayah reconstructs byte-for-byte | — | — | — |
| AC-P1-06 | 🎥 Every token offset matches its surface | — | — | — |
| AC-P1-07 | 🎥 Recomputed hashes match import-time values | — | — | — |
| AC-P1-08 | 🎥 UPDATE/DELETE on canonical rows aborts with coded errors | — | — | — |
| AC-P1-09 | No public API writes canonical rows without `CanonicalChangeSession` | — | — | — |
| AC-P1-10 | 🎥 Crash at each of 13 checkpoints leaves active edition unchanged; retry resumes | — | — | — |
| AC-P1-11 | 🎥 Cancel removes `quran_stg_*` rows and records cancellation | — | — | — |
| AC-P1-12 | 🎥 300 golden references parse; malformed inputs return coded errors, never panic | — | — | — |
| AC-P1-13 | `parse(serialize(ref)) == ref` for all variants | — | — | — |
| AC-P1-14 | Quran read API v1 envelope, meta, ETag, content-language, error body | — | — | — |
| AC-P1-15 | Tool §12 contract conformance + deterministic checksum + no fabrication | — | — | — |
| AC-P1-16 | CLI conventions + snapshots incl. RTL sanity | — | — | — |
| AC-P1-17 | `doctor --quran` 19 checks, read-only, `--deep` < 30 s, JSON schema | — | — | — |
| AC-P1-18 | Debug reader RTL, labelled, no persistence, excluded from nav | — | — | — |
| AC-P1-19 | No stale text served after activation (generation-keyed cache) | — | — | — |
| AC-P1-20 | 14 ADRs accepted §48-complete; 5 D1.14 docs published | — | — | — |
| AC-P1-21 | Citation resolver verdicts + persisted citation re-verification | — | — | — |

---

## 4. Accepted ADRs

_None accepted yet._

**Entry format**

```
### ADR-01nn — <title>
- **Status:** Accepted
- **Accepted:** YYYY-MM-DD
- **Author / reviewers:** <names>
- **File:** adr/ADR-01nn-<slug>.md
- **§48 fields present:** Context · Options · Decision · Accuracy · Religious-source ·
  Licensing · Security · Operational · Migration strategy · Reversal cost
- **Decision summary:** <one or two sentences>
- **Constrains:** <deliverables / later phases>
```

| ADR | Title | Blocking for | Status |
|---|---|---|---|
| ADR-0101 | Initial Quran dataset, script, riwayah, and license | **everything** | ☐ |
| ADR-0102 | Quran addressing scheme & reference grammar | D1.1 | ☐ |
| ADR-0103 | Verse-numbering scheme handling and alternate-numbering strategy | D1.1, D1.5 | ☐ |
| ADR-0104 | Unicode policy (normalization, allowed blocks, forbidden code points, graphemes) | D1.4 | ☐ |
| ADR-0105 | Canonical tokenization rule (whitespace-preserving surface tokenization) | D1.3 | ☐ |
| ADR-0106 | Canonical text storage layout (row-per-ayah vs blob+index) | D1.5 | ☐ |
| ADR-0107 | Atomic activation & rollback (staging + pointer flip + generation counter) | D1.3 | ☐ |
| ADR-0108 | Corpus hashing scheme (`text_hash` / `structure_hash` / `token_order_hash`) | D1.3, D1.4 | ☐ |
| ADR-0109 | Edition difference algorithm for canonical text | D1.3 | ☐ |
| ADR-0110 | Basmala representation policy | D1.1 | ☐ |
| ADR-0111 | Citation identity and deep-link URL/URN format | D1.9 | ☐ |
| ADR-0112 | Translation alignment and attribution model | D1.4 | ☐ |
| ADR-0113 | Canonical lookup caching & invalidation | D1.6 | ☐ |
| ADR-0114 | Reference-corpus comparison procedure and who signs off | D1.4 | ☐ |

> **Religious-source and licensing sections are substantive for Phase 1**, not boilerplate.
> ADR-0101, 0104, 0105, 0110, and 0114 directly determine whether text or metadata can be
> corrupted, mis-numbered, or presented as something it is not; the ADR lint (Phase 0
> AC-P0-19) fails if the reasoning is omitted rather than written down.

---

## 5. Deviations From Plan

_None recorded yet._

Log anything delivered differently from `plan.md`. Deviations are expected and fine —
**undocumented** deviations are the problem, because Phase 2 inherits this corpus assuming the
plan describes it.

**Entry format**

```
### DEV-nn — <short title>
- **Date:** YYYY-MM-DD
- **Plan reference:** plan.md §x / D1.y / P1-Tnn
- **Planned:** <what the plan said>
- **Delivered:** <what was actually built>
- **Reason:** <why>
- **Scope impact:** <deliverables / ACs affected>
- **Phase-2 impact:** <what the handoff doc must say>
- **Approved by:** <name>
```

Deviations requiring **explicit sign-off** because they touch retrofit-impossible guarantees:
any change to the hashing recipe (ADR-0108), the reference grammar (ADR-0102), canonical
immutability/triggers (D1.5), the importer→activation separation (D1.3, I5), the token
round-trip guarantee (D1.3, AC-P1-05), or translation attribution (D1.6, principle 5).

---

## 6. Corrections

_None._

**Entry format**

```
### COR-nn — correction to <entry ID>
- **Date:** YYYY-MM-DD
- **Original entry:** <ID and date>
- **What was wrong:** <description>
- **Correct record:** <description>
- **Cause:** <how the wrong record happened>
```

---

## 7. Deferred Items & Follow-Ups

_None recorded yet._

Anything intentionally not done in Phase 1 that is **not** already in the out-of-scope list
(`README.md` §4.2). Every row needs a named owner and a target phase — an unowned deferral is a
silent scope leak into Phase 2.

| ID | Item | Reason deferred | Target phase | Owner | Logged |
|---|---|---|---|---|---|
| — | — | — | — | — | — |

Carried into `docs/plans/handoff-p1-to-p2.md` by task P1-T60.

---

## 8. Phase Closure

| Gate | Requirement | Evidence | Signed off by | Date |
|---|---|---|---|---|
| All 60 tasks done | `tasks.md` fully ☑ | | | |
| All 13 plan criteria verified | `acceptance.md` §1.1–1.4 | | | |
| All 8 supporting criteria verified | `acceptance.md` §1.6 | | | |
| Coverage gates met | `acceptance.md` §4 | | | |
| 15 required suites green | `acceptance.md` §3.2 | | | |
| 14 ADRs accepted, §48-complete | §4 above | | | |
| 6 migrations applied & checksummed | `migrations/sqlite/` | | | |
| Editorial sign-off recorded | P1-T55 (`verified_by`) | | | |
| Exit-gate ritual recorded | `acceptance.md` §5 (9 live steps) | | | |
| Handoff doc published | `docs/plans/handoff-p1-to-p2.md` | | | |
| Swimlane X decisions owned & open | P1-X01…X05 | | | |
| Deviations documented | §5 above | | | |
| Deferrals owned | §7 above | | | |

**Phase 1 accepted:** _pending_
**Phase 2 unblocked:** _pending_

> Closure requires the **Swimlane X** row. ADR-0101 (dataset licensing) and ADR-0114 (reference
> corpus sign-off) have external lead times engineering cannot compress; ADR-0203/0204
> (morphology, normalization + linguist) are needed at the start of Phase 2. A green Phase 1
> with any of these unowned means Phase 2 starts stalled on a decision that was visible from
> week 1. Recording this as a closure gate is the only reliable defence.
