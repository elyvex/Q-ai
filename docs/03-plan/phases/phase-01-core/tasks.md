# Phase 1 — Task Board

**Phase:** P1 — Canonical Quran Core
**Source:** `plan.md` §8 (Work Breakdown Structure)
**Total tasks:** 60 sprint tasks + 3 swimlane decisions
**Status legend:** ☐ Not Started · ◐ In Progress · ⊘ Blocked · ☑ Done (→ log in `done.md`)
**Roles:** **BE** backend/Rust · **DATA** corpus/data engineering · **QA** test/verification ·
**DOC** docs/ADRs · **EDIT** editorial/legal reviewer (part-time)
**Est** = engineer-days (ed)

> Completion protocol: a task is ☑ only when it satisfies the Definition of Done
> (`acceptance.md` §3) — not when the code compiles and not when the PR merges. On completion,
> append an entry to `done.md` with the date, owner, PR, and evidence link.

> ⚠️ **Read `README.md` §9 before scheduling.** The task rows sum to **131.0 ed**, not the
> plan's stated **82 ed**, and Sprint 1.2 is **42.0 ed**. Both need an owner decision before
> Sprint 1.1 opens. See §9 below.

---

## 1. Swimlane X — External-Lead-Time Decisions (start immediately, run in parallel)

These are **not** sprint deliverables and are **not** on Phase 1's critical path. They are
tracked here because each has an external lead time engineering cannot compress later, and
because "not on the critical path" is exactly why they get silently dropped.

| ID | Decision | ADR | Needed by | Owner | Decision-open date | Status |
|---|---|---|---|---|---|---|
| P1-X01 | Quran text dataset selection & licensing — open the ADR **now**; start vendor/license correspondence in parallel with Sprint 1.0 | ADR-0101 | **Before Sprint 1.1** | _unassigned_ | _TBD_ | ☐ |
| P1-X02 | Engage and name the qualified editorial reviewer who signs off that the text matches a recognized printed muṣḥaf | ADR-0101 | Before P1-T55 (Sprint 1.5) | _unassigned_ | _TBD_ | ☐ |
| P1-X03 | Reference corpus selection + comparison procedure + sign-off | ADR-0114 | Before P1-T26 (Sprint 1.2) | _unassigned_ | _TBD_ | ☐ |
| P1-X04 | Morphology dataset selection & licensing (carried from Phase 0 swimlane X — still open) | ADR-0203 | Start of Phase 2 | _unassigned_ | _TBD_ | ☐ |
| P1-X05 | Normalization rule catalog + **linguist engagement** (carried from Phase 0 swimlane X — still open) | ADR-0204 | Start of Phase 2 | _unassigned_ | _TBD_ | ☐ |

**Action required at the first standup:** assign an owner and a decision-open date to every row,
independent of whether that owner has sprint work. Do not close Sprint 1.0 with any cell reading
`_unassigned_`. If ADR-0101 cannot name a licensed dataset, decide the **fallback** (ship a
public-domain test fixture; user supplies the real edition via `qai quran import`) — see
`README.md` §8.

---

## 2. Sprint 1.0 — Data & Decisions (runs in parallel with Phase 0 Sprint 0.5) — 11.5 ed

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P1-T01 | Survey candidate Quran datasets: provenance, script, license, numbering | ADR-0101 | — | 3.0 | DATA | ☐ |
| P1-T02 | Legal review of redistribution rights; decide bundle vs user-supplied | ADR-0101 | T01 | 2.0 | EDIT | ☐ |
| P1-T03 | Select reference corpus + define comparison procedure and sign-off | ADR-0114 | T01 | 1.5 | EDIT | ☐ |
| P1-T04 | Write ADR-0101 / 0104 / 0110 | ADR | T01–T03 | 2.0 | DOC | ◐ |
| P1-T05 | Build `test-edition-min` + `adversarial/*` fixtures | D1.13 | T04 | 3.0 | QA | ☑ |

> **T05 is not optional even if a real dataset is licensed.** The `test-edition-min` fixture
> (structurally valid, ~5 surahs, public-domain-safe) keeps CI fast and lets the importer and
> validator land before the real corpus does. The 16 `adversarial/*` corpora are the acceptance
> corpus for AC-P1-04 and must each be engineered to trip one **specific** rule.

**Sprint exit:** ADR-0101 and ADR-0114 are `Accepted` (or explicitly deferred with a recorded
fallback decision), and the fixture set exists.

---

## 3. Sprint 1.1 — Domain & Addressing (Week 1–2) — 19.5 ed

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P1-T06 | `quran-core`: newtypes, `Script`, `NumberingScheme`, enums | D1.1 | Phase 0 done | 1.5 | BE | ☑ |
| P1-T07 | `quran-core`: edition/surah/ayah/segment/token structs | D1.1 | T06 | 2.0 | BE | ☑ |
| P1-T08 | Reference grammar implementation: parser (zero-alloc) + serializer | D1.1 | T06 | 3.0 | BE | ☑ |
| P1-T09 | Reference golden-set tests (300 cases incl. malformed) | D1.13 | T08 | 2.0 | QA | ☑ |
| P1-T10 | `QuranQuotation` + constructor visibility guard + tests | D1.9 | T07 | 1.5 | BE | ☑ |
| P1-T11 | ADR-0102 / 0103 / 0105 | ADR | T08 | 1.5 | DOC | ☑ |
| P1-T12 | Migrations `0010`–`0012` + triggers + constraint tests | D1.5 | T07 | 3.0 | BE | ☑ |
| P1-T13 | Repository layer: editions, surahs, ayahs, tokens, divisions (read paths) | D1.5 | T12 | 3.0 | BE | ☑ |
| P1-T14 | Immutability test suite (trigger + API-level guards) | D1.13 | T12 | 2.0 | QA | ☑ |

> **T06/T07 are the frozen vocabulary.** `quran-core` may depend only on `{domain, serde,
> thiserror}` (plus Unicode crates for grapheme handling) — no async, no I/O, no `sqlx`.
> Extend `xtask/allowlist.toml` before adding path edges or `arch-check` will (correctly) fail.
>
> **T08 freezes the reference grammar (ADR-0102).** Changing it after citations are stored
> breaks every stored citation. Land the 300-case golden set (T09) **before** declaring the
> grammar frozen; `parse(serialize(ref)) == ref` must hold for every variant (AC-P1-13).
>
> **T12 encodes the insert-only guarantee in SQL.** The triggers `QAI-QUR-0001…0005` are the
> last line of defence behind Phase 0's `CanonicalWriter`/`ApprovalToken`. They must be tested
> from raw SQL (T14), not just through the repository.

**Sprint exit:** `quran-core` is a real crate, the reference grammar round-trips all golden
cases, migrations `0010`–`0012` apply cleanly and checksum-verify, and canonical rows reject
`UPDATE`/`DELETE`.

---

## 4. Sprint 1.2 — Import & Validation (Week 2–3) — 42.0 ed ⚠️ overloaded

⚠️ **42.0 ed is ~3× a normal sprint.** See `README.md` §9.2. Agree the 1.2a/1.2b split below
before opening the sprint.

### 4.1 Sprint 1.2a — Format, adapters, tokenizer, hashing (≈ 19.0 ed)

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P1-T15 | Intermediate format schema + JSON Schema + serde types | D1.2 | T07 | 2.0 | BE | ☑ |
| P1-T16 | Adapter trait + adapter for the chosen dataset (per ADR-0101) | D1.2 | T15 | 3.0 | DATA | ☑ |
| P1-T17 | Second adapter (different shape) to prove extensibility | D1.2 | T16 | 1.5 | DATA | ☑ |
| P1-T18 | Register `quran_edition_v1` in the Phase-0 validator registry | D1.4 | T15 | 1.0 | BE | ☑ |
| P1-T19 | Validator rules QV-001…QV-012 | D1.4 | T18 | 3.5 | BE | ☑ |
| P1-T20 | Validator rules QV-013…QV-028 | D1.4 | T19 | 3.5 | BE | ☑ |
| P1-T21 | Unicode auditor (blocks, forbidden points, normalization check, grapheme utils) | D1.4 | T18 | 2.5 | BE | ☑ |
| P1-T22 | Tokenizer (whitespace-preserving) + separator capture + offset computation | D1.3 | T15 | 3.0 | BE | ☑ |
| P1-T23 | Hashing: `text_hash`, `structure_hash`, `token_order_hash` per ADR-0108 | D1.3 | T22 | 2.0 | BE | ☑ |
| P1-T24 | Migrations `0014_staging`, `0015_validation` | D1.5 | T12 | 1.5 | BE | ☑ |

### 4.2 Sprint 1.2b — Importer, activation, differ, verification (≈ 23.0 ed)

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P1-T25 | `quran.import` job with 13 checkpoints, cancellation, resume | D1.3 | T22–T24 | 4.0 | BE | ☑ |
| P1-T26 | Round-trip verifier + reference-corpus comparator | D1.4 | T23 | 2.5 | BE | ◐ |
| P1-T27 | Edition differ (char-level, per ADR-0109) + `DifferenceReport` | D1.3 | T23 | 2.5 | BE | ☑ |
| P1-T28 | Activation transaction + rollback + `corpus_generation` | D1.3 | T25, T27 | 2.5 | BE | ☑ |
| P1-T29 | Crash-at-each-checkpoint test matrix (13 cases) | D1.13 | T25, T28 | 2.5 | QA | ☑ |
| P1-T30 | Adversarial-corpus rejection tests (16 fixtures × correct rule id) | D1.13 | T20, T25 | 2.5 | QA | ☑ |
| P1-T31 | ADR-0106 / 0107 / 0108 / 0109 | ADR | T23, T28 | 2.0 | DOC | ☑ |

> **T25 is the §34 pipeline, exactly:** claim → verify hashes → detect format/select adapter →
> parse (unwrap `Untrusted<T>` here) → Unicode audit → structural validation → tokenize +
> separators + offsets → hash → write to `quran_stg_*` → round-trip verify → reference compare →
> difference report → `Staged` + approval request. Checkpoints are resumable and the job is
> idempotent on `(source_version_id, adapter_version, parser_version)`.
>
> **T25/T28 must never be collapsed.** The importer holds no `ApprovalToken` and literally
> cannot activate (I5/I7). Activation is a separate human command with one transaction ending
> in the pointer flip. A crash mid-import leaves `Staged`.
>
> **T26 depends on P1-X03** (reference corpus + procedure). If the reference corpus is not
> configured, QV-015 is skipped rather than passed silently — and ADR-0114 must say so.
>
> **T29 (crash matrix) is the AC-P1-10 evidence.** Simulated process kill at each of the 13
> checkpoints; the active edition is unchanged in all 13 cases and resume completes correctly.
>
> **T30 (adversarial rejection) is the AC-P1-04 evidence.** Every fixture must be rejected with
> the **specific** expected rule id, not merely "some validation error".

**Sprint exit:** `qai quran import` reaches `Staged` with zero `Fatal` findings, all 16
adversarial corpora are rejected with the right codes, and activation requires human approval.

---

## 5. Sprint 1.3 — Reader, Translations, API (Week 4) — 21.5 ed

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P1-T32 | `QuranReader` implementation: get_ayah/get_ayahs/get_tokens | D1.6 | T13 | 2.5 | BE | ☑ |
| P1-T33 | `get_context` with boundary logic + caps | D1.6 | T32 | 2.0 | BE | ☑ |
| P1-T34 | Division lookups (juz/hizb/rub/manzil/page/ruku/sajdah) | D1.6 | T13 | 1.5 | BE | ☑ |
| P1-T35 | Caching layer keyed by corpus generation + consistency tests | D1.6 | T32 | 2.0 | BE | ☑ |
| P1-T36 | Migration `0013` + translation import job + alignment validation | D1.4 | T12 | 2.5 | BE | ☑ |
| P1-T37 | `AyahView` / `AttributedTranslation` types + principle-5 guard tests | D1.6 | T36 | 1.5 | BE | ☑ |
| P1-T38 | Word-gloss dataset import (optional path) | D1.4 | T36 | 1.5 | DATA | ☐ |
| P1-T39 | API v1 handlers + response envelope + ETag/caching | D1.7 | T32–T37 | 3.0 | BE | ☐ |
| P1-T40 | API OpenAPI spec + contract tests + error-body conformance | D1.7 | T39 | 2.0 | BE | ☐ |
| P1-T41 | Performance benchmarks + threshold gates (table in D1.6) | D1.6 | T35 | 2.0 | QA | ☑ |
| P1-T42 | ADR-0112 / 0113 | ADR | T35, T37 | 1.0 | DOC | ◐ |

> **T33 must respect the declared boundary and `max_ayahs`.** Context is retrieved by
> **canonical structure** (surah/juz/ruku/page), never arbitrary chunking (§11.4). Random
> `(before, after, boundary)` must never cross the boundary or exceed the cap (property test,
> `acceptance.md` §2.4).
>
> **T35 is a correctness task, not a performance task.** The cache is keyed by
> `(edition_id, version, corpus_generation, ref, options_hash)` and is invalidated wholesale
> when `corpus_generation` changes. A cache-consistency test asserts no stale text is served
> after activating a new version (ADR-0113).
>
> **T37 enforces principle 5 at the type level.** There is no variant of `AyahView` in which a
> translation can occupy `canonical`, and `AttributedTranslation` has no constructor without a
> non-empty `translator` and an `edition_ref`. This complements the `translation_editions`
> CHECK constraint.

**Sprint exit:** `QuranReader` meets the D1.6 latency table, the API v1 envelope is stable and
ETagged, and translations are structurally incapable of masquerading as canonical text.

---

## 6. Sprint 1.4 — Tools, Citations, CLI, Doctor (Week 5) — 21.5 ed

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P1-T43 | `ToolResult` contract + `ReproducibilityData` + checksum computation | D1.8 | T32 | 2.5 | BE | ☑ |
| P1-T44 | Minimal tool registry + `quran.get_ayah` + `quran.get_context` | D1.8 | T43 | 2.0 | BE | ☑ |
| P1-T45 | Tool contract conformance tests + no-fabrication tests | D1.13 | T44 | 1.5 | QA | ☑ |
| P1-T46 | `citations` crate: `Citation`, resolver, `verify_quotation`, persistence | D1.9 | T32, T15 | 3.0 | BE | ☑ |
| P1-T47 | Deep-link format + resolver endpoint + round-trip tests | D1.9 | T46 | 1.5 | BE | ☑ |
| P1-T48 | CLI `quran get/context/surah/division/resolve` + `--json` | D1.10 | T32 | 2.5 | BE | ☑ |
| P1-T49 | CLI `quran edition/import/validate/diff/activate/rollback` | D1.10 | T28 | 2.5 | BE | ☑ |
| P1-T50 | CLI snapshot tests incl. RTL/Arabic terminal output sanity | D1.13 | T48, T49 | 1.5 | QA | ☑ |
| P1-T51 | `doctor --quran` checks (19 checks) + `--deep` mode | D1.11 | T23, T13 | 3.0 | BE | ☑ |
| P1-T52 | `doctor --quran --json` schema + CI consumption | D1.11 | T51 | 1.0 | BE | ☑ |
| P1-T53 | ADR-0111 | ADR | T47 | 0.5 | DOC | ☐ |

> **T43/T44 define the tool contract once** so every later tool conforms (§12). The first two
> tools are `ReadOnly` (§29) and are exercised by both the CLI and the API to prove the contract
> works from all interfaces before agents exist.
>
> **T45 is the "no fabrication" gate.** Requesting a non-existent reference must return a typed
> error, never a synthesized ayah. `QuranQuotation::new` is visibility-restricted so no code
> path outside the corpus repository can construct one from a string literal.
>
> **T51 `--deep`** runs full-corpus hash recomputation and full token round-trip; target
> complete in under 30 seconds for a standard edition. `doctor` remains strictly read-only
> (Phase 0 AC-P0-14) — `--quran` must not weaken that.

**Sprint exit:** tools, citations, CLI, and doctor are wired to the reader; the no-fabrication
suite is green; `doctor --quran --deep` passes on the fixture edition.

---

## 7. Sprint 1.5 — Debug Reader, Hardening, Exit (Week 6) — 15.0 ed

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P1-T54 | Debug reader page (RTL, web font, ayah markers) | D1.12 | T39 | 1.5 | BE | ☐ |
| P1-T55 | Editorial review pass: reviewer verifies sampled text against printed muṣḥaf; record `verified_by` | AC | T54 | 3.0 | EDIT | ☐ |
| P1-T56 | Golden-set expansion to all §5.2 edge cases | D1.13 | T55 | 2.0 | QA | ☐ |
| P1-T57 | Property-test suite (§5.4) | D1.13 | T32 | 2.0 | QA | ☑ |
| P1-T58 | Full-corpus soak: import → validate → activate → 10k random lookups → `doctor --deep` | D1.13 | all | 2.0 | QA | ☐ |
| P1-T59 | Docs: corpus architecture, import runbook, rollback runbook, citation spec, adapter authoring guide | D1.14 | all | 3.0 | DOC | ☑ |
| P1-T60 | Phase-1 exit gate review + handoff to Phase 2 | — | all | 1.5 | all | ☐ |

> **T54 is explicitly not the Phase-4 UI.** One server-rendered page at
> `/debug/read/{edition}/{surah}`, labelled "debug view", no persistence, excluded from product
> navigation. Rationale: catching a mangled dataset in Week 3 instead of Week 20 is worth
> 1.5 ed.
>
> **T55 is the human sign-off** behind AC-P1-01. The reviewer is named in `verified_by`; the
> sample and comparison method are recorded. This task cannot be done by the implementer.
>
> **T60 gate:** cannot close until every AC in `acceptance.md` passes and the recorded
> exit-gate ritual is archived.

---

## 8. Board Rollup

| Sprint | Scope | Tasks | Est (ed) | Done | Status |
|---|---|---|---|---|---|
| X — External-lead-time decisions | 5 | — | — | 0 | ☐ Not Started |
| 1.0 — Data & Decisions | 5 | 11.5 | 0 | ☐ Not Started |
| 1.1 — Domain & Addressing | 9 | 19.5 | 0 | ☐ Not Started |
| 1.2 — Import & Validation | 17 | 42.0 ⚠️ | 0 | ☐ Not Started |
| 1.3 — Reader, Translations, API | 11 | 21.5 | 0 | ☐ Not Started |
| 1.4 — Tools, Citations, CLI, Doctor | 11 | 21.5 | 0 | ☐ Not Started |
| 1.5 — Debug Reader, Hardening, Exit | 7 | 15.0 | 0 | ☐ Not Started |
| **Total** | **60 + 5** | **131.0** | **0** | **0%** |

By role: **BE ≈ 81.0 ed** · **QA ≈ 23.0 ed** · **DOC ≈ 10.0 ed** · **DATA ≈ 9.0 ed** ·
**EDIT ≈ 6.5 ed** · shared/all 1.5 ed.

> The plan's own total is **82 ed** (`plan.md` §8 header). The rows above sum to **131.0 ed** —
> a **~49 ed (≈1.6×) discrepancy**. This is the same class of error Phase 0 surfaced
> (77.5 stated vs 113.5 summed). It is recorded here, not silently propagated; resolve per
> `README.md` §9.1 before Sprint 1.1 opens.

---

## 9. Sequencing & Estimate Corrections — Resolve Before Sprint 1.1

Three issues in `plan.md`'s WBS are recorded here rather than propagated. None is a reason to
compress an estimate; each needs an explicit owner decision.

### 9.1 The estimate total does not reconcile

| Sprint | Σ task estimates |
|---|---|
| 1.0 | 11.5 ed |
| 1.1 | 19.5 ed |
| 1.2 | 42.0 ed |
| 1.3 | 21.5 ed |
| 1.4 | 21.5 ed |
| 1.5 | 15.0 ed |
| **Total** | **131.0 ed** |

`plan.md` §8 states "Total ≈ 82 ed ⇒ ~6 weeks with 3 engineers". The rows sum to **131.0 ed**
and, at a realistic 15 ed/week, that is **~8.7 weeks**. **Owner decision required**
(`README.md` §9.1): accept a longer duration, cut scope explicitly, or add a fourth engineer.
The retrofit-impossible deliverables — D1.1 (`quran-core`), D1.4 (validator), D1.5
(migrations + triggers), D1.6 (`QuranReader`), D1.13 (integrity suite) — must not absorb a cut.

### 9.2 Sprint 1.2 is overloaded

42.0 ed in one sprint against a ~15 ed/week team is ~3×. Recommended split (already reflected
in §4):

- **1.2a** — format, adapters, tokenizer, hashing, staging migration (T15–T24) ≈ 19.0 ed;
- **1.2b** — validator rules, importer, activation, differ, verification (T19–T20, T25–T31)
  ≈ 23.0 ed.

QA tasks T29/T30 follow 1.2b. Do not fold T25 (importer) and T28 (activation) into one task to
save time — the separation is invariant I5.

### 9.3 Forward dependencies to watch

| Task | Sprint | Depends on | That task / owner | Resolution |
|---|---|---|---|---|
| P1-T26 reference comparator | 1.2b | P1-X03 reference corpus | Swimlane X (EDIT) | Confirm corpus + procedure before 1.2b opens; else QV-015 is skipped and recorded in ADR-0114 |
| P1-T55 editorial sign-off | 1.5 | P1-X02 named reviewer | Swimlane X (EDIT) | Engage reviewer before Sprint 1.3; reviewer must not be the implementer |
| P1-T18 validator registration | 1.2a | Phase-0 `StructureValidator` registry | Phase 0 (done) | Confirm the registry API is stable when Sprint 1.2a starts |

---

## 10. Definition of Done (per task)

A task is complete only when it satisfies `acceptance.md` §3.1 in full — implemented behind an
interface, unit + property tests, real-SQLite integration where persistence is involved, typed
`Diagnostic` errors with remedy + next command, observability spans/metrics, docs, validated
config, cancellation for long operations, secret redaction, provenance + audit for every
mutation, schema version recorded, deny-by-default checks, failure-recovery tested, and
`fmt`/`clippy -D warnings`/`test`/`deny` green.

See `done.md` for the append-only ledger and entry format.
