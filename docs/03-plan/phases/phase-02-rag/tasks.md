# Phase 2 — Task Board

**Phase:** P2 — Quran Search, Arabic Normalization, Morphology & Word Families
**Source:** `plan.md` §15 (Work Breakdown Structure)
**Total tasks:** 114 sprint tasks + 5 swimlane decisions
**Status legend:** ☐ Not Started · ◐ In Progress · ⊘ Blocked · ☑ Done (→ log in `done.md`)
**Roles:** **BE** backend/Rust · **SRCH** search/index specialist · **LING** Arabic linguist ·
**DATA** corpus/data engineering · **QA** test/verification · **DOC** docs/ADRs
**Est** = engineer-days (ed)

> Completion protocol: a task is ☑ only when it satisfies the Definition of Done
> (`acceptance.md` §3) — not when the code compiles and not when the PR merges. On completion,
> append an entry to `done.md` with the date, owner, PR, and evidence link.
>
> ⚠️ **Read `README.md` §9 before scheduling.** The task rows sum to **278.0 ed**, not the
> plan's stated **≈112 ed**, and Sprints 2.3–2.6 are each **42–51 ed**. All need an owner
> decision before Sprint 2.1 opens. See §9 below.

---

## 1. Swimlane X — External-Lead-Time Decisions (start in Phase-1 Sprint 1.5, run in parallel)

These are **not** sprint deliverables and are **not** on Phase 2's critical path. They are
tracked here because each has an external lead time engineering cannot compress later, and
because "not on the critical path" is exactly why they get silently dropped.

| ID | Decision | ADR | Needed by | Owner | Decision-open date | Status |
|---|---|---|---|---|---|---|
| P2-X01 | Morphology dataset selection & licensing — open the ADR **now**; start vendor/license correspondence in parallel with Phase-1 Sprint 1.5 | ADR-0203 | **Start of Phase 2** | _unassigned_ | _TBD_ | ☐ |
| P2-X02 | Engage and name the qualified Arabic linguist (0.4 FTE) who authors/reviews the rule catalog, tagset, root convention, and golden sets | ADR-0204 | **Sprint 2.0** | _unassigned_ | _TBD_ | ☐ |
| P2-X03 | Unified tagset + root-convention inputs (dataset-native tag analysis) | ADR-0210/0215 | Before P2-T62 (Sprint 2.4) | _unassigned_ | _TBD_ | ☐ |
| P2-X04 | Counting-rule semantics + numeric-report policy (linguistic judgment on ambiguous counts) | ADR-0211 | Before P2-T94 (Sprint 2.6) | _unassigned_ | _TBD_ | ☐ |
| P2-X05 | Transliteration standard comparison (decision only, implementation Phase 4) | ADR-0206 (Reserved) | Phase 4 | _unassigned_ | _TBD_ | ☐ |

**Action required at the first standup:** assign an owner and a decision-open date to every row,
independent of whether that owner has sprint work. Do not close Sprint 2.0 with any cell reading
`_unassigned_`. If ADR-0203 cannot name a licensed dataset, decide the **fallback** (ship the
public-domain test lexicon; user supplies the real dataset via `qai quran morphology import`)
— see `README.md` §8.

---

## 2. Sprint 2.0 — Dataset & Linguistic Decisions (parallel with Phase 1 Sprint 1.5) — 30.5 ed

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P2-T01 | Survey morphology datasets: coverage, depth, tokenization, license | D2.6 | — | 3.0 | DATA | ☐ |
| P2-T02 | Legal review of morphology dataset redistribution | D2.6 | T01 | 1.5 | DOC | ☐ |
| P2-T03 | Define alignment strategy + write alignment spec | D2.6 | T01 | 2.0 | DATA | ☐ |
| P2-T04 | Author normalization rule catalog with full code-point mapping tables | D2.1 | — | 4.0 | LING | ☐ |
| P2-T05 | Review each rule for linguistic correctness + document losses | D2.1 | T04 | 2.5 | LING | ☐ |
| P2-T06 | Define profile ladder L0–L8 and versioning policy | D2.1 | T04 | 1.5 | LING+BE | ☐ |
| P2-T07 | Define unified tagset + dataset tag mapping | D2.6 | T01 | 2.5 | LING | ☐ |
| P2-T08 | Define root convention + unification policy | D2.6 | T01 | 1.5 | LING | ☐ |
| P2-T09 | Define counting-rule semantics + numeric-report policy | D2.9 | T04 | 2.0 | LING+BE | ☐ |
| P2-T10 | Write ADR-0203/0204/0205/0210/0211/0215; record ADR-0206 as reserved | D2.13 | T02–T09 | 3.0 | DOC | ☐ |
| P2-T11 | Build normalization golden set (2,000 input→output pairs per profile) | D2.13 | T05 | 4.0 | LING+QA | ☐ |
| P2-T12 | Build root/lemma golden set (500 curated cases) | D2.13 | T08 | 3.0 | LING+QA | ☐ |

> **T04/T05 are the linguistic freeze.** Every rule ships with a published mapping table and
> a documented loss statement (ADR-0204). Implementation (T16/T17) must not start until the
> linguist has reviewed the catalog — an unreviewed fold is a silent recall bug.
>
> **T11/T12 are linguist-signed fixtures.** Changing an expected value later requires a
> linguist review recorded in the PR — the same discipline as canonical text changes.

**Sprint exit:** ADR-0203/0204/0205/0210/0211/0215 are `Accepted` (or explicitly deferred
with a recorded fallback), ADR-0206 is `Reserved`, and both golden sets exist with
`reviewed_by` + `reviewed_at` headers.

---

## 3. Sprint 2.1 — Normalization Engine (Week 1–2) — 28.5 ed

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P2-T13 | `quran-normalization` crate skeleton, `RuleId`, `NormalizationRule` trait | D2.1 | P1 done | 1.5 | BE | ☐ |
| P2-T14 | `SpanMap`: segments, compose, to_canonical, to_derived | D2.1 | T13 | 3.5 | BE | ☐ |
| P2-T15 | `SpanMap` property tests (5 properties × all ayahs × all profiles) | D2.13 | T14 | 3.0 | QA | ☐ |
| P2-T16 | Implement deterministic rules N01–N17 | D2.1 | T14 | 5.0 | BE | ☐ |
| P2-T17 | Implement heuristic rules N18–N22 with `RuleKind::Heuristic` tagging | D2.1 | T16 | 2.5 | BE | ☐ |
| P2-T18 | `NormalizationPipeline` + profile registry + immutability enforcement | D2.1 | T16 | 2.5 | BE | ☐ |
| P2-T19 | Migration `0020_quran_normalization` + profile/rule seeding | D2.10 | T18 | 1.5 | BE | ☑ |
| P2-T20 | `NormalizationTrace` type + no-default-constructor guard | D2.1 | T18 | 1.0 | BE | ☐ |
| P2-T21 | Golden-set test harness; all 2,000 pairs green | D2.13 | T11,T18 | 2.5 | QA | ☐ |
| P2-T22 | Idempotency + associativity + fuzz (no panic on any Unicode input) | D2.13 | T18 | 2.0 | QA | ☐ |
| P2-T23 | `qai quran normalize --explain` + `--list-profiles` + `--show-rule` | D2.12 | T18 | 2.0 | BE | ☑ |
| P2-T24 | `POST /normalization/preview` + `GET /normalization/profiles` | D2.11 | T23 | 1.5 | BE | ☑ |

> **T14 is the highest-risk pure-logic item in the phase.** `to_canonical(to_derived(r)) ⊇ r`,
> grapheme-boundary safety, composition associativity, L0 identity, and re-normalization
> containment must hold for every ayah × every profile (AC-P2-04).
>
> **T18 freezes the profile ladder.** Profiles are append-only; a changed rule list requires
> a new version, a full rebuild, and a drift report. Never edited in place.
>
> **T20 enforces I9 at the type level.** There is no constructor for a search hit without a
> trace — the same pattern Phase 1 used for `QuranQuotation`.

**Sprint exit:** `quran-normalization` passes all 2,000 golden pairs, all 5 SpanMap
properties, and the fuzz battery; `normalize --explain` teaches the transformation.

---

## 4. Sprint 2.2 — Derived Forms & FTS Foundation (Week 2–3) — 33.5 ed

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P2-T25 | Migration `0021_quran_forms` | D2.2 | T19 | 1.5 | BE | ☑ |
| P2-T26 | `quran.forms.rebuild` job: token + ayah forms, all indexed profiles | D2.2 | T25,T18 | 3.0 | BE | ☑ |
| P2-T27 | Skeleton builder (ayah + 3-ayah windows) + span maps | D2.4 | T26 | 2.5 | BE | ☑ |
| P2-T28 | MV-018 canonical-unchanged verifier wired into every build job | D2.10 | T26 | 1.5 | BE | ☑ |
| P2-T29 | `FullTextIndex` trait + `IndexManifest` + `FtsQuery`/`SearchOpts` types | D2.3 | T13 | 2.5 | SRCH | ☑ |
| P2-T30 | Tantivy backend: schema, writer, reader, commit stamps | D2.3 | T29 | 3.5 | SRCH | ☑ |
| P2-T31 | Custom `ar_*` tokenizers wired to `NormalizationPipeline` (shared query/index path) | D2.3 | T30,T18 | 3.0 | SRCH | ☑ |
| P2-T32 | Query/index tokenizer-parity test (5,000 random substrings) | D2.13 | T31 | 1.5 | QA | ☑ |
| P2-T33 | Migration `0023_quran_indexes` + `index_pointers` + build-run tracking | D2.10 | T25 | 1.5 | BE | ☑ |
| P2-T34 | `quran.index.build` job: staging dir → verify → atomic pointer flip | D2.10 | T30,T33 | 3.0 | SRCH | ☑ |
| P2-T35 | Index generation retention, `gc`, single-step rollback | D2.10 | T34 | 1.5 | SRCH | ☐ |
| P2-T36 | Trigram skeleton posting index + build job | D2.4 | T27 | 3.0 | SRCH | ☐ |
| P2-T37 | Index build crash/cancel matrix (kill at each stage; active pointer unchanged) | D2.13 | T34 | 2.0 | QA | ☐ |
| P2-T38 | Cold-rebuild benchmark + CI threshold gate (< 6 min total) | D2.10 | T34,T36 | 1.5 | QA | ☐ |
| P2-T39 | ADR-0201/0208/0213 | D2.13 | T31,T34 | 2.0 | DOC | ☐ |

> **T28 is the mechanical guarantee of I8.** Every build job re-runs MV-018; a hash drift
> fails the job. No index build may alter canonical tables.
>
> **T31 de-risks tokenizer drift (R6).** Tokenizers are thin wrappers over the tested
> pipeline; the 5,000-substring parity test (T32) is a hard CI gate.
>
> **T34 mirrors Phase-1 activation.** Build into `gen-<N>/`, verify, flip the pointer in one
> SQLite transaction, retain the previous generation. A partial index can never serve queries.

**Sprint exit:** both Tantivy indexes + skeleton postings build atomically, verify, and
cold-rebuild in < 6 minutes; crash at any stage leaves the active pointer unchanged.

---

## 5. Sprint 2.3 — Search Tools (Week 3–4) — 42.0 ed ⚠️ overloaded

⚠️ **42.0 ed is ~3× a normal sprint.** See `README.md` §9.2. Agree the 2.3a/2.3b split
before opening the sprint.

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P2-T40 | `SearchHit`, `ScoreExplain`, unified result assembly + canonical-span attach | D2.5 | T14,T30 | 2.5 | BE | ☑ |
| P2-T41 | `quran.search_exact` (+ zero-result normalization hint) | D2.5 | T40 | 2.0 | SRCH | ☑ |
| P2-T42 | `quran.search_normalized` incl. ad-hoc rule sets + `explain` | D2.5 | T41 | 3.0 | SRCH | ☑ |
| P2-T43 | `quran.search_phrase` (ordered/near/unordered, slop) | D2.5 | T41 | 2.5 | SRCH | ☑ |
| P2-T44 | `quran.search_concatenated`: candidate gen → verify → segmentation explanation | D2.4 | T36,T40 | 4.5 | SRCH | ☑ |
| P2-T45 | Cross-ayah window dedup + `spans_ayah_boundary` labeling | D2.4 | T44 | 2.0 | SRCH | ☑ |
| P2-T46 | `quran.search_regex` with DFA engine + all I16 guards + rate limit | D2.5 | T41 | 3.0 | SRCH | ☑ |
| P2-T47 | Exact `total_matches` counting path (separate from ranked search) | D2.5 | T41 | 1.5 | SRCH | ☑ |
| P2-T48 | Filters: surah/juz/page/revelation-place/global-range | D2.5 | T41 | 2.0 | BE | ☑ |
| P2-T49 | Highlighting: canonical char ranges → display markers | D2.5 | T40 | 2.0 | BE | ☑ |
| P2-T50 | Result cache (`0025`) + generation invalidation + LRU cap | D2.10 | T40 | 2.0 | BE | ☑ |
| P2-T51 | Search API endpoints + SSE streaming variant | D2.11 | T41–T46 | 3.0 | BE | ☐ |
| P2-T52 | CLI search command group with all flags + `--json` | D2.12 | T41–T46 | 2.5 | BE | ☐ |
| P2-T53 | Search golden-set suite (400 queries × expected reference sets) | D2.13 | T44,T46 | 4.0 | QA | ☐ |
| P2-T54 | Regex/DoS abuse suite (pathological patterns, timeout, limits) | D2.13 | T46 | 2.0 | QA | ☐ |
| P2-T55 | Search latency benchmarks + CI gates (table §17.1) | D2.13 | T44 | 2.0 | QA | ☐ |
| P2-T56 | ADR-0207/0212/0214 | D2.13 | T44,T46,T50 | 1.5 | DOC | ☐ |

> **T41 never silently folds.** An exact query with foreign code points returns zero results
> **plus a warning** naming the normalized profile to use.
>
> **T44 is the most distinctive feature and the worst naive complexity.** Skeleton + trigram
> candidates + exact verification bounds the work; every hit carries `segmentation` and
> `spans_*` flags so a cross-verse fragment is never presented as one verse.
>
> **T46 is DoS safety (I16).** `regex-automata` DFA only (no backtracking), 1 MiB / 4 MiB
> size limits, 512-char patterns, 3 s budget, no leading `.*`, per-principal rate limits,
> agent-policy gating.

**Sprint exit:** all five search tools return traced, spanned, cited hits; the 400-query
golden suite and the 15-pattern abuse suite are green.

---

## 6. Sprint 2.4 — Morphology Import & Lexicons (Week 5–6) — 45.0 ed ⚠️ overloaded

⚠️ **45.0 ed is ~3× a normal sprint.** See `README.md` §9.2. Agree the 2.4a/2.4b split
before opening the sprint.

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P2-T57 | Migrations `0022_quran_lexicon`, `0024_morphology_staging` | D2.6 | T25 | 2.0 | BE | ☐ |
| P2-T58 | Intermediate morphology format + JSON Schema + serde types | D2.6 | T57 | 2.0 | DATA | ☐ |
| P2-T59 | Adapter for the chosen dataset (per ADR-0203) | D2.6 | T58 | 4.0 | DATA | ☐ |
| P2-T60 | Second adapter (different shape) proving extensibility | D2.6 | T59 | 2.0 | DATA | ☐ |
| P2-T61 | Alignment engine: `DirectKey` + `AlignmentTable` + unmatched reporting | D2.6 | T59 | 4.0 | DATA | ☐ |
| P2-T62 | Unified tagset mapper (native tags preserved verbatim) | D2.6 | T58,T07 | 2.5 | BE | ☐ |
| P2-T63 | Validation rules MV-001…MV-018 | D2.6 | T61,T62 | 4.0 | BE | ☐ |
| P2-T64 | Lexicon builder: roots, lemmas, stems, counts | D2.6 | T63 | 3.0 | BE | ☐ |
| P2-T65 | Cross-dataset root unification as `review_queue` suggestions (never merge) | D2.8 | T64 | 2.5 | BE | ☐ |
| P2-T66 | `quran.morphology.import` job (12 checkpoints, cancel, resume) | D2.6 | T63,T64 | 3.5 | BE | ☐ |
| P2-T67 | `quran.morphology.activate` (approval + pointer flip + enqueue FTS rebuild) | D2.6 | T66 | 2.0 | BE | ☐ |
| P2-T68 | Coverage + unmatched-token reports; approval threshold gate | D2.6 | T61 | 2.0 | BE | ☐ |
| P2-T69 | Dataset version differ (`morphology diff`) | D2.12 | T66 | 2.0 | BE | ☐ |
| P2-T70 | Layer B/D provenance writing for every analysis/root/lemma row | D2.6 | T63 | 2.0 | BE | ☐ |
| P2-T71 | Adversarial morphology fixtures (18 faults → correct MV rule ids) | D2.13 | T63 | 3.0 | QA | ☐ |
| P2-T72 | Import crash/cancel matrix (12 checkpoints) | D2.13 | T66 | 2.0 | QA | ☐ |
| P2-T73 | Populate FTS lexicon fields (roots/lemmas/stems/pos/patterns) | D2.3 | T67 | 2.0 | SRCH | ☐ |
| P2-T74 | ADR-0209 | D2.13 | T63 | 0.5 | DOC | ☐ |

> **T61 is the highest-risk data step.** `DirectKey` requires a 100% key match; anything else
> needs a hashed, auditable `AlignmentTable` with per-surah unmatched reporting. Under no
> circumstance may alignment modify `quran_tokens`.
>
> **T63/T71 are the AC-P2-15 evidence.** All 18 adversarial fixtures must be rejected with the
> **specific** expected MV rule id, not merely "some validation error".
>
> **T66/T67 must never be collapsed.** The importer holds no approval capability and literally
> cannot activate. Activation is a separate human command that enqueues an FTS rebuild — never
> mutating lexicon fields in place.

**Sprint exit:** `quran.morphology.import` reaches `Staged` with zero `Fatal` findings, all
18 adversarial datasets are rejected with the right codes, and activation requires human
approval.

---

## 7. Sprint 2.5 — Morphology & Family Tools (Week 6–7) — 47.5 ed ⚠️ overloaded

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P2-T75 | `AnalysisPolicy` + `analysis_sources` + suppression reporting | D2.7 | T64 | 2.0 | BE | ☐ |
| P2-T76 | `quran.morphology` tool | D2.7 | T75 | 2.5 | BE | ☐ |
| P2-T77 | `quran.morphology_compare` with agreement verdicts, no resolution field | D2.7 | T76 | 2.5 | BE | ☐ |
| P2-T78 | `quran.root_search` (convention resolution, grouping, occurrences) | D2.7 | T64 | 3.0 | BE | ☐ |
| P2-T79 | `quran.lemma_search` | D2.7 | T64 | 1.5 | BE | ☐ |
| P2-T80 | `quran.pattern_search` + capability-unavailable error path | D2.7 | T64 | 2.0 | BE | ☐ |
| P2-T81 | `quran.affix_search` (dataset backend + `L7` heuristic backend, labeled) | D2.7 | T17,T64 | 2.5 | BE | ☐ |
| P2-T82 | Root/lemma browse endpoints + CLI `root list` | D2.11 | T78 | 1.5 | BE | ☐ |
| P2-T83 | `FamilyRelation` taxonomy + `word_family_relations` table wiring | D2.8 | T57 | 2.0 | BE | ☐ |
| P2-T84 | Family resolution algorithm (all 5 input paths) | D2.8 | T78,T83 | 3.5 | BE | ☐ |
| P2-T85 | Relation builders: same-form/lemma/stem/root, derived, inflectional, affix | D2.8 | T84 | 3.5 | BE | ☐ |
| P2-T86 | Per-member `explanation` generator (differing-feature diffing) | D2.8 | T85 | 2.5 | BE | ☐ |
| P2-T87 | Computational-suggestion path: opt-in, confidence floor, mandatory labels | D2.8 | T85 | 2.0 | BE | ☐ |
| P2-T88 | `review_queue` promotion flow: suggestion → `ScholarVerified` with evidence | D2.8 | T87,T65 | 2.5 | BE | ☐ |
| P2-T89 | Morphology/family API endpoints | D2.11 | T76–T87 | 2.5 | BE | ☐ |
| P2-T90 | CLI morphology/root/lemma/family/pattern/affix commands | D2.12 | T76–T87 | 3.0 | BE | ☐ |
| P2-T91 | Root/lemma golden-set suite (500 cases) | D2.13 | T12,T78 | 3.0 | QA | ☐ |
| P2-T92 | Family-relation golden suite (120 curated families, reviewed by LING) | D2.13 | T85 | 3.5 | LING+QA | ☐ |
| P2-T93 | Multi-analysis non-merge tests (no authoritative flag; suppression visible) | D2.13 | T77 | 2.0 | QA | ☐ |

> **T77 has no "winner" field.** Conflicting analyses are presented side-by-side with
> attribution; verdicts are `Identical | CompatibleVariant | Conflicting | OnlyInOne`.
> A synthesis mode is a review failure.
>
> **T81 always says which backend answered.** Dataset morphemes → `Attested (dataset: …)`;
> L7 rules → `Heuristic (pattern-based)`. The label is part of the result, not the docs.
>
> **T86 has no default constructor without an explanation.** Every family member carries a
> human-readable string built from relation + dataset + differing features.

**Sprint exit:** morphology and family tools return attributed, explained results; the
500-case root/lemma suite and the 120-family curated set are green with linguist sign-off.

---

## 8. Sprint 2.6 — Counting, Discovery, Doctor, Evaluation (Week 7–8) — 51.0 ed ⚠️ overloaded

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P2-T94 | `CountingRules` type + serialization + mandatory-field enforcement | D2.9 | T75 | 2.0 | BE | ☐ |
| P2-T95 | `quran.frequency` (exact SQL aggregation, all multi-analysis modes) | D2.9 | T94 | 2.5 | BE | ☐ |
| P2-T96 | `quran.distribution` + partition provenance + disagreement warnings | D2.9 | T95 | 2.5 | BE | ☐ |
| P2-T97 | `quran.cooccurrence` (token/ayah/segment windows, cross-ayah flags) | D2.9 | T95 | 2.5 | BE | ☐ |
| P2-T98 | `quran.collocation` (PMI + LLR + t-score, min-count floor) | D2.9 | T97 | 2.5 | BE | ☐ |
| P2-T99 | `quran.first_last_occurrence`, `quran.interval_analysis` + disclaimer | D2.9 | T95 | 2.0 | BE | ☐ |
| P2-T100 | `quran.numeric_report` + checksum + no-interpretation policy | D2.9 | T94 | 2.5 | BE | ☐ |
| P2-T101 | `quran.hapax_search`, `quran.unusual_usage` | D2.9 | T95 | 2.0 | BE | ☐ |
| P2-T102 | `quran.near_duplicate_passages` (MinHash + exact verify + aligned spans) | D2.9 | T27 | 3.0 | SRCH | ☐ |
| P2-T103 | `quran.missing_expected_form` + mandatory disclaimer | D2.9 | T85 | 2.0 | BE | ☐ |
| P2-T104 | Counting/discovery API endpoints + CLI commands | D2.11 | T95–T103 | 3.0 | BE | ☐ |
| P2-T105 | `doctor` Phase-2 checks (19 checks) incl. `quran.search.smoke` | D2.13 | T34,T67 | 3.5 | BE | ☐ |
| P2-T106 | Index-drift reporting with precise input diff + `QAI-IDX-0101` warnings | D2.10 | T105 | 2.0 | BE | ☐ |
| P2-T107 | Nightly reconciliation job (`quran.index.verify`, 1 % sample, MV-018) | D2.10 | T105 | 2.5 | BE | ☐ |
| P2-T108 | Evaluation harness: metric definitions, versioned datasets, gates | D2.13 | T53,T91 | 3.5 | QA | ☐ |
| P2-T109 | Counting-rules determinism tests (same rules ⇒ same number, always) | D2.13 | T95 | 1.5 | QA | ☐ |
| P2-T110 | Tool-contract conformance for all 22 Phase-2 tools | D2.13 | T104 | 2.5 | QA | ☐ |
| P2-T111 | Full soak: rebuild all indexes → 50k randomized queries → doctor → reconcile | D2.13 | all | 2.5 | QA | ☐ |
| P2-T112 | ADR-0216 + ADR index update | D2.13 | T18 | 0.5 | DOC | ☐ |
| P2-T113 | Docs: normalization spec, profile catalog, search cookbook, morphology adapter guide, counting-rules explainer, reindex runbook | D2.13 | all | 4.0 | DOC | ☐ |
| P2-T114 | Phase-2 exit gate review + handoff to Phase 3 | — | all | 2.0 | all | ☐ |

> **T95 counts from SQL, never from FTS term frequencies.** Counts are exact
> (`COUNT(*)` over the lexicon join) because tokenizer versions can drift.
>
> **T100/T99/T103 carry mandatory copy.** `numeric_report` contains no interpretive
> commentary; `interval_analysis` and `missing_expected_form` emit their fixed disclaimers
> verbatim (snapshot-tested, AC-P2-29).
>
> **T105 `quran.search.smoke` is the cheapest early-warning system in the phase:** 12 canned
> queries with known expected references; any tokenizer or profile regression fails loudly.
>
> **T114 gate:** cannot close until every AC in `acceptance.md` passes and the recorded
> exit-gate ritual is archived.

---

## 9. Board Rollup

| Sprint | Scope | Tasks | Est (ed) | Done | Status |
|---|---|---|---|---|---|
| X — External-lead-time decisions | 5 | — | 0 | ☐ Not Started |
| 2.0 — Dataset & Linguistic Decisions | 12 | 30.5 | 0 | ☐ Not Started |
| 2.1 — Normalization Engine | 12 | 28.5 | 0 | ☐ Not Started |
| 2.2 — Derived Forms & FTS Foundation | 15 | 33.5 | 0 | ☐ Not Started |
| 2.3 — Search Tools | 17 | 42.0 ⚠️ | 0 | ☐ Not Started |
| 2.4 — Morphology Import & Lexicons | 18 | 45.0 ⚠️ | 0 | ☐ Not Started |
| 2.5 — Morphology & Family Tools | 19 | 47.5 ⚠️ | 0 | ☐ Not Started |
| 2.6 — Counting, Discovery, Doctor, Evaluation | 21 | 51.0 ⚠️ | 0 | ☐ Not Started |
| **Total** | **114 + 5** | **278.0** | **0** | **0%** |

By role: **BE ≈ 141.0 ed** · **SRCH ≈ 40.0 ed** · **QA ≈ 40.5 ed** · **DATA ≈ 17.0 ed** ·
**DOC ≈ 13.0 ed** · **LING ≈ 10.5 ed** · **LING+BE ≈ 3.5 ed** · **LING+QA ≈ 10.5 ed** ·
shared/all 2.0 ed.

> The plan's own total is **≈112 ed** (`plan.md` §15 header). The rows above sum to
> **278.0 ed** — a **~166 ed (≈2.5×) discrepancy**. This is the same class of error Phase 0
> surfaced (77.5 stated vs 113.5 summed) and Phase 1 surfaced (82 stated vs 131.0 summed).
> It is recorded here, not silently propagated; resolve per `README.md` §9.1 before
> Sprint 2.1 opens.

---

## 10. Sequencing & Estimate Corrections — Resolve Before Sprint 2.1

Three issues in `plan.md`'s WBS are recorded here rather than propagated. None is a reason to
compress an estimate; each needs an explicit owner decision.

### 10.1 The estimate total does not reconcile

| Sprint | Σ task estimates |
|---|---|
| 2.0 | 30.5 ed |
| 2.1 | 28.5 ed |
| 2.2 | 33.5 ed |
| 2.3 | 42.0 ed |
| 2.4 | 45.0 ed |
| 2.5 | 47.5 ed |
| 2.6 | 51.0 ed |
| **Total** | **278.0 ed** |

`plan.md` §15 states "Total ≈ 112 ed ⇒ ~8 weeks with 3 engineers + 0.4 FTE Arabic
linguist". The rows sum to **278.0 ed** and, at a realistic 15 ed/week, that is
**~18.5 weeks**. **Owner decision required** (`README.md` §9.1): accept a longer duration,
cut scope explicitly, or add engineers. The retrofit-impossible deliverables — D2.1
(normalization + SpanMap), D2.3 (FTS + tokenizer parity), D2.6 (import + MV-001…018 +
alignment), D2.13 (integrity/eval suites) — must not absorb a cut.

### 10.2 Sprints 2.3–2.6 are each overloaded

42–51 ed per sprint against a ~15 ed/week team is ~3×. Recommended splits (already
reflected in §§5–6 for 2.3/2.4; extend the same pattern to 2.5/2.6):

- **2.3a** — search core + result assembly (T40–T43, T47–T50);
- **2.3b** — concatenated + regex + hardening (T44–T46, T53–T55);
- **2.4a** — import + alignment + validation (T57–T63, T71–T72);
- **2.4b** — lexicons + activation + FTS lexicon fields (T64–T70, T73–T74);
- **2.5a** — morphology tools (T75–T82); **2.5b** — family engine (T83–T90);
- **2.6a** — counting/discovery (T94–T104); **2.6b** — doctor/eval/soak/docs (T105–T114).

QA suites follow their build halves. Do not fold T66 (import) and T67 (activation) into
one task to save time — the separation is invariant I5 extended to datasets.

### 10.3 Forward dependencies to watch

| Task | Sprint | Depends on | That task / owner | Resolution |
|---|---|---|---|---|
| P2-T21 golden harness | 2.1 | P2-T11 normalization golden set | Sprint 2.0 (LING+QA) | Confirm linguist-signed set lands before 2.1 opens |
| P2-T31 shared pipeline | 2.2 | P2-T18 pipeline + profiles | Sprint 2.1 (BE) | Confirm profile ladder frozen; no in-place edits |
| P2-T61 alignment engine | 2.4 | P2-X01 dataset + P2-T59 adapter | Swimlane X (DATA) | Confirm dataset + shape before 2.4 opens; else fallback lexicon per ADR-0203 |
| P2-T62 tagset mapper | 2.4 | P2-T07 unified tagset | Sprint 2.0 (LING) | Confirm tagset + mapping before 2.4 opens |
| P2-T91/T92 eval suites | 2.5 | P2-T12 root/lemma set, linguist review | Sprint 2.0 + LING | Engage linguist before Sprint 2.3; curated families need review lead time |
| P2-T105 doctor checks | 2.6 | P2-T34 index build, P2-T67 activation | Sprints 2.2/2.4 | Confirm manifest + generation-stamping APIs stable when 2.6 starts |

---

## 11. Definition of Done (per task)

A task is complete only when it satisfies `acceptance.md` §3.1 in full — implemented behind an
interface, unit + property tests, real-SQLite/Tantivy integration where persistence is
involved, typed `Diagnostic` errors with remedy + next command, observability spans/metrics,
docs, validated config, cancellation for long operations, secret redaction, provenance + audit
for every mutation, generation + version stamps recorded, deny-by-default checks,
failure-recovery tested, heuristic/computational labeling present where applicable, and
`fmt`/`clippy -D warnings`/`test`/`deny` green.

See `done.md` for the append-only ledger and entry format.
