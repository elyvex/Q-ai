# Phase 2 — Completion Ledger

**Phase:** P2 — Quran Search, Arabic Normalization, Morphology & Word Families
**Status:** 🟡 In Progress — 5 / 114 tasks · 0 / 50 acceptance criteria · 0 / 14 ADRs · 2 / 6 migrations
**Started:** 2026-09-14
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
   reason. Silent drift is how a search phase stops being verifiable.

**Entry format**

```
### P2-Tnn — <task title>
- **Deliverable:** D2.x
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
| 2.0 — Dataset & Linguistic Decisions | 12 | 0 | 30.5 | — | ☐ |
| 2.1 — Normalization Engine | 12 | 3 | 28.5 | — | ☐ |
| 2.2 — Derived Forms & FTS Foundation | 15 | 2 | 33.5 | — | ☐ |
| 2.3 — Search Tools | 17 | 0 | 42.0 | — | ☐ |
| 2.4 — Morphology Import & Lexicons | 18 | 0 | 45.0 | — | ☐ |
| 2.5 — Morphology & Family Tools | 19 | 0 | 47.5 | — | ☐ |
| 2.6 — Counting, Discovery, Doctor, Evaluation | 21 | 0 | 51.0 | — | ☐ |
| **Total** | **114 + 5** | **5** | **278.0** | **—** | **4%** |

| Artifact class | Complete | Total |
|---|---|---|
| Deliverables (D2.1–D2.13) | 0 | 13 |
| Acceptance criteria (AC-P2-01…50) | 0 | 50 |
| ADRs accepted (+ 2 reserved) | 0 | 14 + 2 |
| Migrations applied (`0013`–`0018` per DEV-04) | 2 | 6 |
| Required test suites green | 0 | 17 |
| D2.13 documents published | 0 | 6 |

Track **actual vs. estimate** from the first completed task. `tasks.md` §10.1 flags a
**~166 ed discrepancy** between the plan's stated ≈112 ed and its own task rows (278.0 ed);
actuals recorded here are the only way to learn which number was closer. Record actuals even
when they exceed the estimate — an under-recorded sprint is how the next phase inherits a
wrong capacity model.

---

## 2. Completed Tasks

_None completed yet._

**Entry format (repeat per task)**

```
### P2-Tnn — <task title>
- **Deliverable:** D2.x
- **Completed:** YYYY-MM-DD
- **Owner:** <name> (<role>)
- **PR / commit:** <link>
- **Evidence:** <CI run, test path, snapshot, or recording>
- **DoD:** ✅ all items / ⚠️ exceptions: <list with justification>
- **Notes:** <deviations, follow-ups, TODOs filed>
```

### Sprint 2.0 — Dataset & Linguistic Decisions

### Sprint 2.1 — Normalization Engine

### P2-T19 — Migration `0013_quran_normalization` + profile/rule seeding
- **Deliverable:** D2.10
- **Completed:** 2026-09-15
- **Owner:** agent (BE)
- **PR / commit:** working tree; landed via owner commits (see `git log -- migrations/sqlite/0013* crates/storage*/src/quran.rs`)
- **Evidence:** `migrations/sqlite/0013_quran_normalization.up.sql`; `cargo run -p xtask -- migrate-check` (13 ordered, checksums stable); `crates/application/tests/normalization_seed.rs::seed_matches_code`, `::profile_lifecycle_and_append_only_trigger` (real SQLite: 22 rules + 9 profiles equal code, trigger rejects rewrites with QAI-NORM-0003)
- **DoD:** ✅ all items / layers: derived-only tables, no canonical touch; typed `StorageError` paths; `arch-check` unaffected (no new workspace edges in storage crates)
- **Notes:** DEV-04 (numbering `0013`, not `0020`); DEV-06 (trigger reports QAI-NORM-0003, not QAI-NORM-0001). Required `read_flow.trycmd` + `sqlite_database_health` version bumps 12→13 (fallout, same commit scope).

### P2-T23 — `qai quran normalize --explain` + `--list-profiles` + `--show-rule`
- **Deliverable:** D2.12
- **Completed:** 2026-09-15
- **Owner:** agent (BE)
- **PR / commit:** working tree; landed via owner commits (see `git log -- crates/cli/src/quran.rs crates/application/src/quran_cli.rs crates/application/src/quran_normalize.rs`)
- **Evidence:** `crates/cli/tests/quran/normalize.trycmd` (10 snapshot cases: list/show/explain/adhoc/usage + reserved-rule note), `cargo test -p cli --test quran` green; `crates/application/src/quran_normalize.rs` unit tests (spec parsing, preview trace)
- **DoD:** ✅ all items / defines AC-P2-38 evidence (rule-by-rule `--explain` with heuristic flags); exit codes per CLI contract (0/2/5/70)
- **Notes:** definitions served from seeded rows (proves seed per call), implementations from code; reserved N23/N24 handled without an implementation. Harness split to one temp DB per trycmd file (parallel-safety fix).

### P2-T24 — `POST /normalization/preview` + `GET /normalization/profiles`
- **Deliverable:** D2.11
- **Completed:** 2026-09-15
- **Owner:** agent (BE)
- **PR / commit:** working tree; landed via owner commits (see `git log -- crates/server/src/api.rs`)
- **Evidence:** `crates/server/tests/api.rs::normalization_preview_matches_cli_pipeline` (incl. AC-P2-39 byte-identical trace vs the CLI pipeline path), `::normalization_profiles_lists_ladder`; OpenAPI entries in `docs/08-api/quran-v1-openapi.json` (coverage test extended)
- **DoD:** ✅ all items / envelope + `QAI-NORM-*` error bodies; no new workspace edges (server renders via `application::quran_normalize` re-exports)
- **Notes:** none.

### Sprint 2.2 — Derived Forms & FTS Foundation

### P2-T25 — Migration `0014_quran_forms`
- **Deliverable:** D2.2
- **Completed:** 2026-09-15
- **Owner:** agent (BE)
- **PR / commit:** working tree; landed via owner commits (see `git log -- migrations/sqlite/0014* crates/storage*/src/quran.rs`)
- **Evidence:** `migrations/sqlite/0014_quran_forms.up.sql`; `cargo run -p xtask -- migrate-check` (14 ordered, checksums stable); `crates/storage-sqlite/tests/quran_forms.rs` (real SQLite: empty reads, orphan writes rejected on canonical FK, edition delete total)
- **DoD:** ✅ all items / derived-only tables with provenance+generation stamps; canonical untouched; FK (not discipline) enforces I8
- **Notes:** span maps deliberately NOT stored (recomputed from canonical text through the shared pipeline — R6 by construction; recorded in migration header, ADR-0208 keeps FTS-side scope). Windows surah-scoped by CHECK.

### P2-T29 — `FullTextIndex` trait + `IndexManifest` + `FtsQuery`/`SearchOpts` types
- **Deliverable:** D2.3
- **Completed:** 2026-09-15
- **Owner:** agent (SRCH)
- **PR / commit:** working tree; landed via owner commits (see `git log -- crates/quran-search/`)
- **Evidence:** `crates/quran-search/src/{error,index,model}.rs`; `cargo test -p quran-search` (error codes, opts ceilings, query/manifest JSON round-trips); `cargo run -p xtask -- arch-check` (no llm/embeddings/retrieval/vector edges)
- **DoD:** ✅ all items / backend-agnostic port (FTS5 now, Tantivy/OpenSearch named only); exact-count `count()` separate from ranked `search()`; `QAI-IDX-0101` reserved for drift
- **Notes:** `QAI-IDX-*` error namespace opened (0001–0004 + 0101). FTS5 adapter (P2-T30) and tokenizers (P2-T31) are separate tasks.

### Sprint 2.3 — Search Tools

### Sprint 2.4 — Morphology Import & Lexicons

### Sprint 2.5 — Morphology & Family Tools

### Sprint 2.6 — Counting, Discovery, Doctor, Evaluation

---

## 3. Verified Acceptance Criteria

_None verified yet._

**Entry format**

```
### AC-P2-nn — <criterion short name>
- **Verified:** YYYY-MM-DD
- **Verified by:** <reviewer name> (must not be the implementer for ritual ACs)
- **Method:** <test path / scripted check / live walkthrough>
- **Evidence:** <CI run URL, artifact, or recording timestamp>
- **Result:** ☑ Pass
- **Notes:** <caveats, re-verification triggers>
```

Live-walkthrough ACs (§5 ritual: AC-P2-05, 07, 08, 09, 17, 22, 26, 28, 31, 33, 50)
additionally require the recording link and the reviewers' names (Arabic linguist + Phase-1
editorial reviewer), and must be verified by someone other than the implementer.

| ID | Criterion | Verified | By | Evidence |
|---|---|---|---|---|
| AC-P2-01 | ADR-0203 accepted with licensed dataset or documented fallback | — | — | — |
| AC-P2-02 | ADR-0204/0205 accepted with mapping tables + loss docs | — | — | — |
| AC-P2-03 | All 2,000 normalization golden pairs pass | — | — | — |
| AC-P2-04 | SpanMap 5 properties hold; offset fidelity 1.00 | — | — | — |
| AC-P2-05 | 🎥 Canonical text byte-identical after all builds/imports | — | — | — |
| AC-P2-06 | No SearchHit without NormalizationTrace | — | — | — |
| AC-P2-07 | 🎥 `الرحمن` search returns the full expected set | — | — | — |
| AC-P2-08 | 🎥 `بسمالله` search returns 1:1 with segmentation | — | — | — |
| AC-P2-09 | 🎥 Persian-keyboard query: L5 match + L0 zero-result warning | — | — | — |
| AC-P2-10 | Profile monotonicity holds for every golden query | — | — | — |
| AC-P2-11 | Every hit maps to exact canonical ranges (re-normalization check) | — | — | — |
| AC-P2-12 | Citation resolver validates every hit; failures never returned | — | — | — |
| AC-P2-13 | Regex DoS suite: 15 pathological patterns bounded + rate-limited | — | — | — |
| AC-P2-14 | All 400 search golden queries pass incl. must_not_contain | — | — | — |
| AC-P2-15 | 18 adversarial morphology fixtures reject with specific MV ids | — | — | — |
| AC-P2-16 | Alignment never modifies tokens; unmatched ratio gates approval | — | — | — |
| AC-P2-17 | 🎥 No authoritative column; competing analyses all returned | — | — | — |
| AC-P2-18 | Compare returns verdicts only; no resolution field | — | — | — |
| AC-P2-19 | Suppression always counted under every AnalysisPolicy | — | — | — |
| AC-P2-20 | Every linguistic row has Layer B/D provenance | — | — | — |
| AC-P2-21 | Layer D rows carry algorithm/version/confidence; verified-gate holds | — | — | — |
| AC-P2-22 | 🎥 Cross-dataset roots linked as suggestions, never merged | — | — | — |
| AC-P2-23 | 300 root + 200 lemma cases at gated precision/recall | — | — | — |
| AC-P2-24 | Segmentation faithfulness ≥ 0.995 | — | — | — |
| AC-P2-25 | Family members grouped + explained; curated set ≥ 0.95 | — | — | — |
| AC-P2-26 | 🎥 Suggestions off by default with mandatory label | — | — | — |
| AC-P2-27 | Suggestion → ScholarVerified only via review queue | — | — | — |
| AC-P2-28 | 🎥 CountingRules complete; determinism holds | — | — | — |
| AC-P2-29 | No-interpretation policy + verbatim disclaimers | — | — | — |
| AC-P2-30 | Hapax results state their profile (rule-relativity proven) | — | — | — |
| AC-P2-31 | 🎥 Atomic builds: kill at any stage, previous generation serves | — | — | — |
| AC-P2-32 | `index rebuild --all` from sources; cold rebuild < 6 min | — | — | — |
| AC-P2-33 | 🎥 Version bumps produce exact drift reports + QAI-IDX-0101 warnings | — | — | — |
| AC-P2-34 | Drift never auto-repaired; doctor read-only | — | — | — |
| AC-P2-35 | No stale cache served across generation bump | — | — | — |
| AC-P2-36 | No llm/embeddings/retrieval/vector dep in the three crates | — | — | — |
| AC-P2-37 | All 22 tools: §12 contract, ReadOnly, checksum determinism | — | — | — |
| AC-P2-38 | `normalize --explain` shows rule-by-rule trace + heuristic flags | — | — | — |
| AC-P2-39 | Preview endpoint returns the same trace from the same code path | — | — | — |
| AC-P2-40 | 5,000-substring tokenizer parity holds | — | — | — |
| AC-P2-41 | 19 doctor checks implemented; smoke fails loudly on regression | — | — | — |
| AC-P2-42 | Nightly reconciliation verifies counts, sample, keys, orphans, MV-018 | — | — | — |
| AC-P2-43 | Latency targets met, CI-gated at 20% tolerance | — | — | — |
| AC-P2-44 | Hard accuracy gates pass; runs versioned and diffed | — | — | — |
| AC-P2-45 | Two morphology adapters prove extensibility | — | — | — |
| AC-P2-46 | Linguist sign-off on all three golden/curated sets | — | — | — |
| AC-P2-47 | 14 ADRs accepted §48-complete; ADR-0206 Reserved | — | — | — |
| AC-P2-48 | 6 D2.13 docs published | — | — | — |
| AC-P2-49 | Full soak: 50k queries, zero integrity findings/panics/hash changes | — | — | — |
| AC-P2-50 | 🎥 MVP linguistic workflow end-to-end via CLI and API | — | — | — |

---

## 4. Accepted ADRs

_None accepted yet._

**Entry format**

```
### ADR-02nn — <title>
- **Status:** Accepted / Reserved
- **Accepted:** YYYY-MM-DD
- **Author / reviewers:** <names>
- **File:** adr/ADR-02nn-<slug>.md
- **§48 fields present:** Context · Options · Decision · Accuracy · Religious-source ·
  Licensing · Security · Operational · Migration strategy · Reversal cost
- **Decision summary:** <one or two sentences>
- **Constrains:** <deliverables / later phases>
```

| ADR | Title | Blocking for | Status |
|---|---|---|---|
| ADR-0201 | Full-text engine: Tantivy + custom Arabic tokenizers | §4 | ☐ |
| ADR-0202 | *(reserved for Phase 3 graph store — not written here)* | — | Reserved |
| ADR-0203 | Initial Quran morphology dataset, license, alignment, attribution | **everything morphological** | ☐ |
| ADR-0204 | Arabic normalization rule catalog, mapping tables, rule ordering | §3.2 | ☐ |
| ADR-0205 | Normalization profile ladder, versioning, immutability | §3.3 | ☐ |
| ADR-0206 | Transliteration standard *(reserved; decision deferred to Phase 4)* | §3.2 (N23) | Reserved |
| ADR-0207 | Concatenated-search architecture (skeleton + trigram + verify) | §4.4 | ☐ |
| ADR-0208 | Offset-mapping representation (`SpanMap`) and storage format | §3.4 | ☐ |
| ADR-0209 | Multi-analysis representation; no authoritative flag | §6.1 | ☐ |
| ADR-0210 | Root convention + cross-dataset unification as suggestions | §6.2 | ☐ |
| ADR-0211 | Counting rules, multi-analysis semantics, numeric-report policy | §8.1 | ☐ |
| ADR-0212 | Regex/pattern-search resource limits and engine choice | §4.3 | ☐ |
| ADR-0213 | Index generation stamping, activation, retention, drift policy | §9.3 | ☐ |
| ADR-0214 | Search result caching and invalidation keying | §13 | ☐ |
| ADR-0215 | Unified morphological tagset + dataset tag mapping | §6.1 | ☐ |
| ADR-0216 | Fuzzy-search policy (experimental, off by default) | §3.3 (L8) | ☐ |

> **Religious-source and licensing sections are substantive for Phase 2**, not boilerplate.
> ADR-0203, 0204, 0205, 0209, 0210, and 0211 directly determine whether linguistic data can be
> misattributed, silently merged, or presented as scholarship; the ADR lint fails if the
> reasoning is omitted rather than written down.

---

## 5. Deviations From Plan

### DEV-06 — Append-only triggers report QAI-NORM-0003, not QAI-NORM-0001
- **Date:** 2026-09-15
- **Plan reference:** README §7 (`0020` trigger `QAI-NORM-0001`) / P2-T19
- **Planned:** append-only trigger named `QAI-NORM-0001`
- **Delivered:** four triggers (`trg_normalization_{rules,profiles}_no_{update,delete}`) raising `QAI-NORM-0003`
- **Reason:** `QAI-NORM-0001` is already the `UnknownRule` error code; reusing it for immutability violations would conflate two failure modes. `QAI-NORM-0003` (`ProfileImmutable`) matches the violation semantics.
- **Scope impact:** error-code documentation only; enforcement identical
- **Phase-3 impact:** none; handoff records the code mapping
- **Approved by:** agent (owner to ratify)

Log anything delivered differently from `plan.md`. Deviations are expected and fine —
**undocumented** deviations are the problem, because Phase 3 inherits these indexes and
lexicons assuming the plan describes them.

**Entry format**

```
### DEV-nn — <short title>
- **Date:** YYYY-MM-DD
- **Plan reference:** plan.md §x / D2.y / P2-Tnn
- **Planned:** <what the plan said>
- **Delivered:** <what was actually built>
- **Reason:** <why>
- **Scope impact:** <deliverables / ACs affected>
- **Phase-3 impact:** <what the handoff doc must say>
- **Approved by:** <name>
```

Deviations requiring **explicit sign-off** because they touch retrofit-impossible guarantees:
any change to the rule catalog or profile ordering (ADR-0204/0205), the SpanMap
representation (ADR-0208), the multi-analysis non-merge rule (I11, ADR-0209), the
never-generate-roots rule (§2.3), the canonical-unchanged guarantee (I8, MV-018), or the
counting-rules transparency (I15, ADR-0211).

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

_Awaiting owner assignment at the first standup — seeded with the known Phase-2 deferrals._

Anything intentionally not done in Phase 2 that is **not** already in the out-of-scope list
(`README.md` §4.2). Every row needs a named owner and a target phase — an unowned deferral is a
silent scope leak into Phase 3.

| ID | Item | Reason deferred | Target phase | Owner | Logged |
|---|---|---|---|---|---|
| OWN-01 | ADR-0203 dataset/license/reviewer (P2-X01) unresolved; engineering proceeds on the public-domain test lexicon; ADR-0203 stays DRAFT | legal/data act an agent cannot make | Phase 2 / swimlane X | _unassigned_ | 2026-09-14 |
| OWN-02 | Linguist engagement 0.4 FTE (P2-X02) unresolved; T04/T05/T11/T12/T92 cannot be signed off without a named linguist | hiring act an agent cannot make | Phase 2 / swimlane X | _unassigned_ | 2026-09-14 |
| OWN-03 | Estimate gap ≈112 ed (stated) vs 278.0 ed (summed); proceeds incrementally with the 2.3a/2.3b + 2.4a/2.4b splits; no silent compression | owner capacity/scope decision | Phase 2 scheduling | _unassigned_ | 2026-09-14 |
| OWN-04 | FTS backend adopted provisionally as Tantivy per plan §4.2; ADR-0201 to ratify or redirect to the FTS5 fallback before P2-T30 | owner to ratify | Phase 2 (before P2-T30) | _unassigned_ | 2026-09-14 |
| OWN-05 | Migration numbers `0020`–`0025` assume no intervening migrations land first; renumber contiguously if Phase 1 lands more, record mapping here | numbering contingency | Phase 2 (before P2-T19) | _unassigned_ | 2026-09-14 |

Carried into `docs/plans/handoff-p2-to-p3.md` by task P2-T114.

---

## 8. Phase Closure

| Gate | Requirement | Evidence | Signed off by | Date |
|---|---|---|---|---|
| All 114 tasks done | `tasks.md` fully ☑ | | | |
| All 50 plan criteria verified | `acceptance.md` §1.1–1.9 | | | |
| Coverage gates met | `acceptance.md` §4 | | | |
| 17 required suites green | `acceptance.md` §3.2 | | | |
| Latency targets met | `acceptance.md` §2.4 | | | |
| Hard accuracy gates pass | `acceptance.md` §2.5 | | | |
| 14 ADRs accepted + 2 reserved, §48-complete | §4 above | | | |
| 6 migrations applied & checksummed | `migrations/sqlite/` | | | |
| Linguist sign-off recorded | P2-T11/T12/T92 (`docs/reviews/`) | | | |
| Exit-gate ritual recorded | `acceptance.md` §5 (11 live steps) | | | |
| Handoff doc published | `docs/plans/handoff-p2-to-p3.md` | | | |
| Swimlane X decisions owned & open | P2-X01…X05 | | | |
| Deviations documented | §5 above | | | |
| Deferrals owned | §7 above | | | |

**Phase 2 accepted:** _pending_
**Phase 3 unblocked:** _pending_

> Closure requires the **Swimlane X** row. ADR-0203 (dataset licensing) has an external lead
> time engineering cannot compress; ADR-0204/0205 (rule catalog + linguist) gate every
> golden set; ADR-0210/0211/0215 gate the import and counting work. A green Phase 2 with any
> of these unowned means Phase 3 starts stalled on a decision that was visible from week 1.
> Recording this as a closure gate is the only reliable defence.
