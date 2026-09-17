# Phase 2 — Quran Search, Normalization & Linguistics

**Phase ID:** P2
**Phase name:** Quran Search, Arabic Normalization, Morphology & Word Families
**Plan version:** 1.0.0
**PRD baseline:** Q-ai PRD v0.3.2
**PRD traceability:** §8 · §9 · §11.1 · §11.2 · §11.3 · §11.6 · §11.7 · §12 · §12.1 ·
§19.1 · §25.4 · §25.6 · §26 · §27.1 · §32.2 · §35.1 · §36.1 · §40 · §41 · §43 (Phase 2) ·
§44.1 · §46 · §47 · §50 · §56 · §67 · §76 · §88 (items 3–4) · §89
**Depends on:** Phase 0 — all deliverables · Phase 1 — all deliverables
(see `../phase-01-core/done.md` §7 for the handoff assets)
**Blocks:** Phase 3 (graph — needs roots/lemmas as nodes) · Phase 4 (Web search &
word inspector) · Phase 7 (hybrid retrieval reuses the FTS layer) · Phase 9
(linguistic research tools for agents)
**Target duration (plan):** 8 calendar weeks ≈ 22–24 engineer-weeks,
3 engineers + part-time Arabic linguist (0.4 FTE)
**Task-board estimate:** 278.0 ed across 114 tasks → see §9
**Status:** In progress; normalization/indexing/search core implemented, exit gates open (reviewed 2026-09-17)

The [completion ledger](done.md) and [execution plan](execution-plan.md) record the
normalization pipeline, profiles and offset traces, derived-form rebuilds, FTS5 index
builds, and exact/normalized/phrase/concatenated/cross-ayah/regex search services.
Normalization and index-management CLI commands and normalization preview HTTP routes
exist; search-query CLI/HTTP/SSE wiring, morphology, families, linguistic review and
phase-wide hardening remain unfinished. No phase exit acceptance is recorded.

The current backend is **SQLite FTS5**, not the Tantivy backend specified in the original
plan (DEV-05). Scope/deliverable tables below retain that design target; they are not a
claim that every feature is implemented. Dataset/licensing and linguist decisions remain
open, and P1's outstanding source/acceptance gates still apply. The legacy directory
name `phase-02-rag` does not describe this phase: multi-RAG is later work.

**Companion documents**

| File | Purpose |
|---|---|
| `plan.md` | Full Phase-2 plan (source of truth — do not edit without an ADR) |
| `tasks.md` | Work Breakdown Structure as a live task board (P2-T01…T114, swimlane X) |
| `acceptance.md` | Phase-2 exit gate: AC-P2-01…50 + supporting gates and required test suites |
| `done.md` | Append-only completion ledger (tasks, ACs, ADRs, deviations, deferrals) |

---

## 1. How To Read This Phase

- **D2.x** = Deliverable (a shippable, testable unit; synthesis for this phase —
  `plan.md` numbers sections, not deliverables — see §5).
- **P2-Tnn** = Task (a work item on the board, `tasks.md`).
- **AC-P2-nn** = Acceptance Criterion (phase exit gate, `acceptance.md`).
- **ADR-02nn** = Architecture Decision Record required inside this phase.
- **I8…I16** = the non-negotiable invariants introduced here (§3).
- **MV-nnn** = a morphology-validation rule (`plan.md` §6.4).
- **QV-028 / MV-018** = the canonical-unchanged verifier re-run after every build.

Every task lists `Deliverable`, `Depends`, `Estimate (engineer-days)`, and `Owner role`.
Nothing in Phase 2 may weaken a Phase-0 or Phase-1 guarantee; it may only build on them.
In particular: no task may modify canonical tables, re-tokenize canonical text, invent a
second reference grammar, or add an LLM/vector dependency to the deterministic path.

---

## 2. Phase Objective

Turn the canonical corpus of Phase 1 into a **searchable, linguistically explorable,
fully explainable research engine** — while guaranteeing that not one byte of canonical
text changes.

### The Phase-2 promise (PRD §8, §8.3, §9.2, §46)

> A user can type Arabic with any combination of missing diacritics, missing spaces,
> wrong hamza forms, or Persian code points, and Q-ai finds the canonical passage —
> then *shows exactly which normalization rules and segmentation made the match*, links
> every hit to a pinned canonical reference, exposes every root/lemma/morphological
> analysis **with its attribution and confidence**, never merges competing analyses, and
> never presents a computational suggestion as verified scholarship.

This is a load-bearing promise. If Phase 2 ships a search layer that *usually* finds the
right passage but cannot explain itself, every later graph edge, frequency claim, and
agent research report inherits an unverifiable step that no downstream tooling can audit.
The gate is therefore explainability + byte-level verifiability, not recall alone.

---

## 3. Non-Negotiable Invariants Introduced Here

| # | Invariant | Enforcement | Where tested |
|---|---|---|---|
| **I8** | Normalization never mutates canonical text; it only produces **derived** search copies | canonical tables untouched by index jobs; MV-018 re-verified after every build; separate physical tables/index dirs | `acceptance.md` integrity suite; AC-P2-05 |
| **I9** | Every search result reports the exact ordered rule set applied | `NormalizationTrace` is a required field of every match; no constructor without it | type test; AC-P2-06 |
| **I10** | Every match maps back to exact canonical character offsets | bidirectional `SpanMap`, property-tested | 5 SpanMap properties; AC-P2-04/11 |
| **I11** | Multiple morphological analyses coexist; none is silently authoritative | `morphology_analyses` has no `is_correct` column; preference is a per-request policy recorded in output | schema test; AC-P2-17/18/19 |
| **I12** | Machine-derived linguistic data is Layer D with algorithm, version, confidence, verification status | DB CHECK + display labels | doctor `quran.morphology.provenance`; AC-P2-20/21 |
| **I13** | Word-family relations distinguish exact-form / lemma / stem / root / computational / scholar-verified | closed `FamilyRelation` enum + provenance | family suite; AC-P2-25/26 |
| **I14** | Derived indexes are reproducible from `(source_version, rule_set_version, dataset_version)` and record their `corpus_generation` | index manifests + doctor staleness checks | drift suite; AC-P2-33 |
| **I15** | Numeric reports state their exact counting rules and never assert numerological significance | `CountingRules` required in output; fixed disclaimer for speculative patterns | determinism suite; AC-P2-28/29 |
| **I16** | Regex and pattern search are resource-bounded and cannot be used for DoS | compiled-size cap, step budget, timeout, no-backtracking engine | DoS suite; AC-P2-13 |

---

## 4. Scope Fence

### 4.1 In scope (`plan.md` §2.1)

| # | Item | PRD |
|---|---|---|
| 1 | Versioned Arabic normalization rule engine (N01–N24) with offset mapping | §8.1, §8.2 |
| 2 | Derived search forms per token, ayah, and surah (12 forms) | §8.1 |
| 3 | Full-text index (Arabic-aware) with exact + normalized + phrase fields | §32.2 |
| 4 | Letter-skeleton index for concatenated / space-insensitive search | §8.3 |
| 5 | `quran.search_exact`, `search_normalized`, `search_concatenated`, `search_phrase`, `search_regex` | §11.1 |
| 6 | Root, lemma, and stem lexicons with dataset attribution | §9.1, §43 P2 |
| 7 | Morphological analyses (multi-dataset, multi-analysis) + segmentation | §9.1, §9.2 |
| 8 | `quran.root_search`, `lemma_search`, `morphology`, `morphology_compare`, `pattern_search`, `affix_search` | §11.2 |
| 9 | Word-family engine (`quran.word_family`) with typed relations | §9.3, §11.2 |
| 10 | Frequency, distribution, co-occurrence, collocation, interval, first/last-occurrence tools | §11.3 |
| 11 | Discovery tools: `unusual_usage`, `hapax_search`, `near_duplicate_passages`, `missing_expected_form` | §11.7 |
| 12 | Numeric report tool with explicit counting rules | §11.6 |
| 13 | Full-text index abstraction (`FullTextIndex` trait) + Tantivy implementation | §32.2 |
| 14 | Index build/rebuild jobs, index manifests, generation stamping, reconciliation | §76, §41 |
| 15 | Quran search API v1 (remaining §27.1 endpoints) | §27.1 |
| 16 | CLI: `qai quran search/root/lemma/family/morphology/freq/index` | §26, §25.4 |
| 17 | `qai doctor --quran --indexes` (normalized/root/lemma/morphology index drift) | §50 |
| 18 | Linguistic provenance: every root/lemma/analysis attributed, versioned, confidence-scored | §6.4, §9.2 |
| 19 | Evaluation harness with versioned golden sets and metric gates | §36.1, §47 |
| 20 | Search result explainability payload (rules, segmentation, offsets, scores) | §8.3, §12 |

### 4.2 Out of scope — do not build these in Phase 2 (`plan.md` §2.2)

| Excluded | Lands in |
|---|---|
| Graph nodes/edges/paths (Phase 2 produces roots/lemmas as *relational lexicon rows*) | **Phase 3** |
| Semantic / vector / embedding search (Phase 2 is lexical + morphological only) | **Phase 7** |
| Web UI search page and word inspector (Phase 2 ships API + CLI only) | **Phase 4** |
| Rhetorical/structural tools (`repetition_analysis`, `parallel_structure`, `rhyme_analysis`, `address_shift`, `discourse_links`, `pronoun_reference`) — except `near_duplicate_passages`, which is purely lexical and ships here | **Phase 3** |
| Tafsir/hadith/scripture search (`FullTextIndex` trait designed for reuse, only Quran indexes built) | **Phase 5 / 8** |
| Transliteration search and phonetic approximation (schema fields reserved, rule stubs present, disabled; ADR-0206 deferred) | **Phase 4** |
| Fuzzy/edit-distance spelling search L8 (ships *experimental, off by default* behind a config flag) | full quality in **Phase 4** |
| Cross-encoder reranking, query decomposition, multi-query retrieval | **Phase 7** |

### 4.3 Deliberate constraints adopted now

1. **Derived data never touches canonical storage.** Index and lexicon builds read
   canonical rows and write only to derived tables/index dirs; MV-018 re-hashes canonical
   tables in the same job (I8).
2. **Profiles are append-only.** Changing a profile's rule list requires a new version, a
   full index rebuild, and a doctor drift report — never an in-place edit.
3. **L7/L8 results are always labeled heuristic/experimental.** The UI/CLI must render
   "matched using heuristic affix stripping" (§8.3); true morphological search is the
   lexicon path, not L7.
4. **The query path and the index path share one `NormalizationPipeline` instance.**
   A query and a document can never disagree; the 5,000-substring parity test enforces it.
5. **Drift is a warning, never an auto-repair.** Queries against a drifted index still work
   but carry `QAI-IDX-0101` staleness warnings; `doctor` is read-only and only suggests
   `qai quran morphology reindex`.

---

## 5. Deliverable Index

`plan.md` numbers sections, not deliverables. The D2.x IDs below are a synthesis for
tracking only — each maps to the cited plan section, which remains authoritative.

| ID | Deliverable | Plan | Primary crates | ACs |
|---|---|---|---|---|
| D2.1 | `quran-normalization` — rules N01–N24, pipeline, profiles, `NormalizationTrace` | §3 | `quran-normalization` | 02, 03, 06, 38, 40 |
| D2.2 | Derived forms + skeletons (`quran_token_forms`, `quran_ayah_forms`, `quran_skeletons`) | §3.5, §4.4 | `quran-corpus`, `storage-sqlite` | 05, 30 |
| D2.3 | `FullTextIndex` trait + Tantivy backend + custom `ar_*` tokenizers | §4.1, §4.2 | `quran-search` | 31, 32, 40 |
| D2.4 | Skeleton + trigram concatenated search with segmentation explanation | §4.4, §5.3 | `quran-search` | 08, 09, 10 |
| D2.5 | Search tools: exact / normalized / phrase / concatenated / regex | §5 | `tools`, `quran-search` | 07–14, 37 |
| D2.6 | Morphology import pipeline + MV-001…018 + alignment engine + lexicons | §6.1–6.4 | `quran-morphology` | 01, 15, 16, 20, 24, 45 |
| D2.7 | Morphology tools: `morphology`, `morphology_compare`, `root_search`, `lemma_search`, `pattern_search`, `affix_search` | §6.5 | `tools`, `quran-morphology` | 18, 19, 23 |
| D2.8 | Word-family engine (`FamilyRelation`, resolution, explanations, suggestions) | §7 | `quran-morphology` | 22, 25, 26, 27 |
| D2.9 | Counting & discovery tools (frequency → missing-forms, 13 tools) + `CountingRules` | §8 | `tools`, `quran-search` | 28, 29, 30 |
| D2.10 | Index lifecycle: build/rebuild/verify/gc, manifests, drift, reconciliation, cache | §9, §13 | `quran-search`, `jobs` | 31–35, 42 |
| D2.11 | Quran search API v1 + SSE streaming + normalization preview | §10 | `server`, `api` | 37, 39 |
| D2.12 | CLI search/linguistics/counting/normalize/dataset/index command groups | §11 | `cli` | 38, 50 |
| D2.13 | `qai doctor --indexes` (19 checks) + eval harness + golden/property suites | §12, §16, §17 | `cli::doctor`, `testkit` | 41–44, 46–50 |

---

## 6. Architecture & Dependency Rule

```text
                    ┌──────────────────────┐
                    │ quran-normalization  │  rules, pipeline, profiles,
                    │ (pure logic, no I/O) │  SpanMap, traces
                    └──────────┬───────────┘
                               │  shared pipeline instance
              ┌────────────────▼───────────────┐
              │          quran-search          │  FTS trait, Tantivy backend,
              │ (index + query, no LLM/embed)  │  skeleton/trigram, counting
              └────────────────┬───────────────┘
                               │  lexicon joins for roots/lemmas
              ┌────────────────▼───────────────┐
              │        quran-morphology        │  import, alignment, MV-*,
              │ (lexicons, analyses, families) │  family engine, CountingRules
              └────────────────┬───────────────┘
        ┌──────────────────────┼──────────────────────┐
        │                      │                      │
  ┌─────▼──────┐      ┌────────▼────────┐    ┌────────▼────────┐
  │ storage-   │      │  application    │    │  server / api   │
  │ sqlite     │      │ (services+jobs) │    │ (search v1+SSE) │
  └────────────┘      └────────┬────────┘    └────────┬────────┘
                               │                      │
                        ┌──────▼──────────────────────▼──────┐
                        │                 cli                │
                        │  quran search/…  ·  doctor --indexes│
                        └────────────────────────────────────┘
```

Intended dependency edges (to be added to `xtask/allowlist.toml`):

```text
quran-normalization -> domain                        (serde, thiserror, unicode-* only; NO async, NO sqlx)
quran-search        -> quran-normalization, quran-core, domain, storage
                                                     (NO llm/embeddings/retrieval/vector-store)
quran-morphology    -> quran-normalization, quran-core, domain, sources, storage
                                                     (NO llm/embeddings/retrieval/vector-store)
storage-sqlite      -> storage, domain, quran-core   (forms/lexicon/index-pointer repos)
application         -> quran-normalization, quran-search, quran-morphology, …
cli                 -> application, config, observability, server
server              -> application, config, observability
```

**Hard rule (I8/I12, extends Phase-1 I2):** `quran-normalization`, `quran-search`, and
`quran-morphology` must have no dependency on `llm`, `embeddings`, `retrieval`, or any
vector-store crate. `cargo xtask arch-check` enforces this (AC-P2-36); adding those edges
is a gate failure and must not be merged.

---

## 7. Migration Status

Implemented files under `migrations/sqlite/` are `0013_quran_normalization.up.sql`,
`0014_quran_forms.up.sql`, `0015_quran_indexes.up.sql` and
`0016_quran_search_cache.up.sql`. The normalization append-only trigger uses
`QAI-NORM-0003`. Lexicon and morphology-staging migrations remain planned.
See `done.md` DEV-04/06/07/08 for numbering and schema deviations.

The following is the original design table, not the installed schema:

| Planned file | Contents |
|---|---|
| `0020_quran_normalization.up.sql` | `normalization_rules`, `normalization_profiles` + append-only trigger `QAI-NORM-0001` |
| `0021_quran_forms.up.sql` | `quran_token_forms`, `quran_ayah_forms`, `quran_skeletons` + lookup indexes |
| `0022_quran_lexicon.up.sql` | `morphology_datasets`, `quran_roots`, `quran_lemmas`, `quran_token_analyses`, `quran_morphemes`, `quran_derivations`, `word_family_relations` |
| `0023_quran_indexes.up.sql` | `index_pointers`, `index_build_runs` |
| `0024_quran_morphology_staging.up.sql` | `morph_stg_*` mirrors + `import_run_id`, no immutability triggers, cascade from `morphology_import_runs` |
| `0025_quran_search_cache.up.sql` | `search_result_cache` (generation-keyed, LRU-capped at 128 MiB) |

Migrations are **append-only and checksummed** (Phase 0 D0.7). Phase-1 migrations end at
`0012`; implemented Phase-2 migrations continue at `0013`–`0016`. Allocate future numbers
from the actual migration directory, not the original table, and never renumber an
applied migration.

---

## 8. Data-Source Prerequisite — The Hard Gate (ADR-0203)

Phase 2 **cannot ship morphology** without a licensed dataset. This is a *legal + data*
task, not a coding task, and **must start in Phase-1 Sprint 1.5** (tracked as swimlane
**P2-X01…X03** in `tasks.md` §1, deliberately outside the sprint line).

**ADR-0203 (Initial Quran morphology dataset)** must record:

- candidate datasets — coverage (all 77k+ tokens?), depth (POS, features, lemma, root,
  stem, pattern), license/redistribution rights (§38);
- **alignment strategy**: how dataset token indices map to the Phase-1
  `(edition, surah, ayah, position)` key; a disagreeing tokenization is bridged by an
  explicit, auditable alignment table — *never* by re-tokenizing canonical text;
- **attribution string** shown in the UI for every analysis from that dataset;
- whether the dataset supplies **one** or **multiple** analyses per token (I11 works either way);
- root normalization convention (bare letters vs. separators, hamza treatment).

**Fallback if no dataset can be bundled.** Phase 2 ships the full multi-analysis schema,
importer, alignment validator, and tools; a **public-domain test lexicon** covering the
fixture edition powers the tests; users run `qai quran morphology import <manifest>`.
Root/lemma search degrades to a typed "no dataset active" error naming the missing
capability — never to guessed data. Decided **in ADR-0203**, not improvised on the deadline.

**Non-negotiable:** Q-ai must **never** generate roots or lemmas with an LLM and store them
as if dataset-supplied. A computational analyzer writes Layer D rows with
`Attribution::Computational`, confidence, and `verification_status = needs_review` only.

> ⚠️ The linguist (0.4 FTE) is the longest-lead item after the dataset licence.
> Sourcing/booking a qualified Arabic linguist — not the writing — is the bottleneck.
> Open ADR-0204 work and golden-set authoring in Sprint 2.0 or the sign-offs (AC-P2-46)
> stall the whole phase.

---

## 9. Scheduling Reality Check — Read Before Committing To Dates

### 9.1 The task rows do not reconcile with the stated total

The plan's WBS header says **"Total ≈ 112 ed ⇒ ~8 weeks with 3 engineers + 0.4 FTE
Arabic linguist."** Summing the individual task estimates gives **278.0 ed**, a
**~166 ed gap** (~2.5×). At a realistic 15 ed/week for a 3-engineer team that is
**~18.5 weeks**, not 8. (Phase 1 had the same class of error: 82 stated vs 131.0 summed.)

| Sprint | Scope | Σ task estimates |
|---|---|---|
| 2.0 | Dataset & Linguistic Decisions (parallel with Phase 1 Sprint 1.5) | 30.5 ed |
| 2.1 | Normalization Engine (Week 1–2) | 28.5 ed |
| 2.2 | Derived Forms & FTS Foundation (Week 2–3) | 33.5 ed |
| 2.3 | Search Tools (Week 3–4) | 42.0 ed |
| 2.4 | Morphology Import & Lexicons (Week 5–6) | 45.0 ed |
| 2.5 | Morphology & Family Tools (Week 6–7) | 47.5 ed |
| 2.6 | Counting, Discovery, Doctor, Evaluation (Week 7–8) | 51.0 ed |
| **Total** | **114 tasks** | **278.0 ed** |

By role: **BE ≈ 141.0** · **SRCH ≈ 40.0** · **QA ≈ 40.5** · **DATA ≈ 17.0** ·
**DOC ≈ 13.0** · **LING ≈ 10.5** · **LING+BE ≈ 3.5** · **LING+QA ≈ 10.5** · shared/all 2.0.

Pick one before Sprint 2.1 starts:

- **(a)** Accept ~18–19 weeks and revise the target duration; or
- **(b)** Hold 8 weeks and cut scope explicitly — the first cuts should be D2.9 discovery
  depth (P2-T101…T103), D2.10 cache (P2-T50), and D2.13 doc depth (P2-T113); or
- **(c)** Add engineers: BE is 141 ed (≈ half the phase) and SRCH owns the critical
  index/search path — a second BE and a dedicated SRCH for Sprints 2.2–2.5.

Do **not** resolve this by silently compressing estimates. **D2.1 (normalization +
SpanMap), D2.3 (FTS + tokenizer parity), D2.6 (import + MV-001…018 + alignment), and
D2.13 (integrity/eval suites) are retrofit-impossible** and must not absorb the cut:
once a match cannot be mapped back to canonical offsets, or an analysis can be stored
without attribution, the guarantee is gone.

### 9.2 Sprints 2.3–2.6 are each overloaded

Every build sprint from 2.3 onward exceeds a 15 ed/week team by ~3×, and 2.4/2.5 contain
the two highest-risk items in the phase (the 12-checkpoint morphology import and the
family-resolution algorithm). Recommended split, to be agreed before Sprint 2.3 opens:

- **2.3a — Search core** (P2-T40…T43, T47…T50; exact/normalized/phrase + result assembly)
- **2.3b — Concatenated + regex + hardening** (P2-T44…T46, T53…T55; skeleton verify, DFA guards)
- **2.4a — Import + alignment + validation** (P2-T57…T63, T71…T72)
- **2.4b — Lexicons + activation + FTS lexicon fields** (P2-T64…T70, T73…T74)

The QA suites (P2-T53…T55, T71…T72, T91…T93) follow their build halves as dedicated
hardening windows.

### 9.3 Cross-phase decisions with external lead times

Tracked as swimlane **P2-X01…X05**. These are **not** on Phase 2's own critical path,
which is exactly why they get deprioritized — and each has a lead time engineering cannot
compress.

| Decision | ADR | Needed by | Bottleneck |
|---|---|---|---|
| Morphology dataset selection & licensing | ADR-0203 | **Start of Phase 2 (Sprint 2.0)** | Licensing review, dataset-shape evaluation |
| Linguist engagement (0.4 FTE) + rule-catalog review | ADR-0204/0205 | **Sprint 2.0** | Sourcing/booking a qualified Arabic linguist |
| Unified tagset + root convention | ADR-0210/0215 | Before P2-T62 (Sprint 2.4) | Dataset-native tag analysis |
| Counting-rule semantics | ADR-0211 | Before P2-T94 (Sprint 2.6) | Linguistic judgment on ambiguous counts |
| Transliteration standard (reserved) | ADR-0206 | Phase 4 | Standards comparison, not implementation |

**Action:** in the **first standup**, assign an owner and a "decision-open" date to each
row, independent of sprint task assignments and independent of whether that owner has
other Phase-2 work.

---

## 10. Quick Start

Build and initialize the synthetic demo using the [project quick start](../../../../README.md#quick-start),
then run these commands from the repository root with the same `QAI_DATA_DIR`:

```bash
./target/debug/qai quran normalize --list-profiles
./target/debug/qai quran normalize --show-rule N06
./target/debug/qai quran forms rebuild test-edition-min@0.1.0
./target/debug/qai quran index rebuild
./target/debug/qai quran index verify
```

`qai quran search`, `root`, `morphology`, `family`, and `doctor --indexes` are planned,
not current commands. Index options are `--index` and (for rebuild) `--edition`;
`--all` and `--force` are not implemented. Search is currently a Rust service surface.
See the [project verification commands](../../../../README.md#development-and-verification)
for build and test gates.

**Error-code namespaces:** `QAI-NORM-nnnn` (normalization), `QAI-IDX-nnnn` (indexes;
`QAI-IDX-0101` = stale-index warning). Reference-grammar and canonical-table codes stay
`QAI-QUR-*` from Phase 1. Codes are public API.

**CLI exit codes** are inherited from Phase 0 (D0.13) and must not diverge: `0` ok · `1`
generic · `2` usage · `3` validation failed · `4` denied by policy · `5` not found · `6`
conflict/state · `7` cancelled · `70` internal.

---

## 11. ADRs Required In This Phase

| ADR | Title | Blocking | Notes |
|---|---|---|---|
| ADR-0201 | Full-text engine: Tantivy + custom Arabic tokenizers | §4 | Rust-native, offline, positional phrase support vs. writing tokenizers ourselves |
| ADR-0202 | *(reserved for Phase 3 graph store — not written here)* | — | Reserved |
| ADR-0203 | Initial Quran morphology dataset, license, alignment strategy, attribution string | **Everything morphological** | Coverage/quality vs. redistribution rights; bundled vs. user-supplied |
| ADR-0204 | Arabic normalization rule catalog, mapping tables, and rule ordering | §3.2 | Recall vs. precision; each fold loses a distinction permanently |
| ADR-0205 | Normalization profile ladder, versioning, and immutability policy | §3.3 | Simplicity vs. user control; profile explosion risk |
| ADR-0206 | Transliteration standard *(reserved; decision recorded, implementation Phase 4)* | §3.2 (N23) | ALA-LC vs. DIN vs. Buckwalter vs. IJMES |
| ADR-0207 | Concatenated-search architecture: skeleton + trigram candidates + verification | §4.4 | Index size and window duplication vs. recall across boundaries |
| ADR-0208 | Offset-mapping representation (`SpanMap`) and storage format | §3.4 | Storage cost vs. exact highlight/citation fidelity |
| ADR-0209 | Multi-analysis morphology representation; no authoritative flag | §6.1 | UI complexity vs. §9.2 compliance (non-negotiable) |
| ADR-0210 | Root convention normalization and cross-dataset root unification as reviewable suggestions | §6.2 | Convenience of merging vs. never silently merging |
| ADR-0211 | Counting rules, multi-analysis counting semantics, and numeric-report policy | §8.1 | Multiple defensible counts vs. one "official" number (we choose transparency) |
| ADR-0212 | Regex and pattern-search resource limits and engine choice | §4.3 | Expressiveness vs. DoS safety |
| ADR-0213 | Index generation stamping, atomic activation, retention, and drift policy | §9.3 | Disk usage vs. instant rollback |
| ADR-0214 | Search result caching and invalidation keying | §13 | Latency vs. staleness risk (correctness wins) |
| ADR-0215 | Unified morphological tagset and mapping from dataset-native tags | §6.1 | Interoperability vs. information loss (native tags always preserved verbatim) |
| ADR-0216 | Fuzzy-search policy: experimental, off by default, explicit labeling | §3.3 (L8) | Discoverability vs. false confidence |

Every ADR uses the Phase-0 §48 template, including **Accuracy implications**,
**Religious-source implications**, and **Licensing implications** (enforced by the ADR
lint). For Phase-2 ADRs the religious-source section is often substantive — ADR-0203,
0204, 0205, 0209, 0210, and 0211 directly determine whether linguistic data can be
misattributed, silently merged, or presented as scholarship.

ADR numbering follows the phase-coded scheme (`ADR-02nn` = Phase 2). Writing an ADR with
a bare `ADR-000n` id is a lint failure. ADR-0202 and ADR-0206 are recorded as `Reserved`,
not `Accepted`.

---

## 12. Exit Gate

Phase 2 is complete when **all criteria in `acceptance.md` pass**: the 50 plan-defined
ACs (AC-P2-01…50), the required integrity/property suites, and the coverage floors. The
exit-gate ritual (`acceptance.md` §5) must be recorded live on a clean machine by reviewers
who are not the implementers (the Arabic linguist + the Phase-1 editorial reviewer).

**Handoff artifact:** `docs/plans/handoff-p2-to-p3.md` (task P2-T114), containing the
frozen profile catalog with versions, the active morphology dataset and its attribution
string, the list of Layer D rows awaiting review, the deferred-tool list from §4.2 with
target phases, and every place Phase 3 must record `graph_version` alongside the existing
version stamps.

---

## 13. Top Risks

| # | Risk | L | I | Mitigation |
|---|---|---|---|---|
| R1 | **No licensable morphology dataset** — root/lemma/family become empty shells | Med | **Critical** | Swimlane X, ADR-0203 fallback decided in advance; typed "dataset unavailable" errors, never guessed data; search half ships regardless |
| R2 | **Normalization rules linguistically wrong**, silently degrading recall/precision | High | **Critical** | Linguist authors/reviews catalog before implementation; published mapping tables + loss statements; 2,000-pair golden set; monotonicity property test |
| R3 | **Dataset tokenization disagrees** with Phase-1 surface tokenization | High | **Critical** | Explicit hashed `AlignmentTable`, never inferred; unmatched ratio gates approval; re-tokenizing canonical text architecturally impossible |
| R4 | **SpanMap bugs** produce wrong highlights and unverifiable citations | High | High | 5 properties × every ayah × every profile; citation resolver re-verifies every hit; 92% coverage gate on normalization crate |
| R5 | **Concatenated search** misses cross-boundary matches or explodes combinatorially | High | High | Skeleton + trigram candidates + exact verify; fixed-cost 3-ayah windows; 120-case golden set; p99 ≤ 150 ms gate |
| R6 | **Query/index tokenizer drift** produces unexplainable misses | Med | High | One shared pipeline instance; 5,000-substring parity hard gate; `search.smoke` runtime check |
| R7 | **Frequency counts treated as absolute** (numerology pressure) | High | Med | Required `CountingRules`; explicit multi-analysis handling; no-interpretation policy; fixed disclaimers |
| R8 | **Pressure to pick "the correct" analysis** for a cleaner UI | High | **Critical** | Schema-level enforcement (no such column); echoed `AnalysisPolicy` + suppression counts; AC-P2-17/18/19 |
| R9 | **Cross-dataset root unification quietly merges** distinct roots | Med | High | Review-queue suggestions only; nullable Layer-D link; non-merge integrity test |
| R10 | **Estimate gap** (§9.1) turns into a crunch that cuts D2.1/D2.3/D2.6/D2.13 | High | High | Owner decision before Sprint 2.1; cut discovery/cache/doc depth first, never the integrity work |
