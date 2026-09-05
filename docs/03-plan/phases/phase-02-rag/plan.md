Continuing. First the tail of **File 2** (Phase 1) that got cut off, then the complete **File 3** (Phase 2).

---

## File 2 (continued) — `docs/plans/PHASE-1-CANONICAL-QURAN-CORE.md`

````markdown
## 9. Acceptance Criteria (Phase 1 Exit Gate) — continued

| ID | Criterion | Verification |
|---|---|---|
| AC-P1-13 | `parse(serialize(ref)) == ref` holds for every reference variant; canonical serialization is always the fully-qualified pinned form | property test |
| AC-P1-14 | `qai quran get 2:255` returns the exact canonical Arabic text, edition slug, edition version, `text_hash`, and a deep link — with no LLM, vector store, or network call in the code path | integration test + architecture test (forbidden crate deps) |
| AC-P1-15 | `get_context` never crosses its declared boundary, never exceeds `max_ayahs`, and returns whole ayahs only (no arbitrary chunking) | property test |
| AC-P1-16 | Division lookups (juz, hizb, rubʿ, manzil, page, ruku, sajdah) return correct ranges for the golden division fixture | golden test |
| AC-P1-17 | Performance targets in D1.6 are met on the reference hardware profile and enforced as CI thresholds | criterion benches |
| AC-P1-18 | After activating a new edition version, no cached value from the previous generation is ever served | cache-consistency test |
| AC-P1-19 | A translation cannot occupy the `canonical` field of `AyahView`; a `TranslationEdition` with an empty translator is rejected at both the type and DB level | type test + DB CHECK test |
| AC-P1-20 | Every translation passage and every structural-metadata field resolves to a non-canonical provenance layer (B/C), verified by `doctor --quran` | doctor check `quran.metadata_layering` |
| AC-P1-21 | API v1 endpoints match the published OpenAPI spec; every response carries `meta.edition`, `meta.corpus_generation`, `meta.canonical_reference`, and `meta.reproducibility.checksum` | contract tests |
| AC-P1-22 | `quran.get_ayah` and `quran.get_context` conform to the §12 tool result contract, are `ReadOnly`, and produce identical `reproducibility.checksum` for identical inputs on the same corpus generation | tool conformance test |
| AC-P1-23 | Requesting a non-existent reference (e.g. `2:300`, `115:1`) returns a typed coded error; no code path can synthesize ayah text | negative tests + `QuranQuotation` constructor visibility test |
| AC-P1-24 | The citation resolver correctly returns `ExactMatch`, `Mismatch{first_difference_at}`, `LocationNotFound`, and `EditionNotFound` for the corresponding fixtures, and persists resolved citations with hash + ingestion version | citation suite |
| AC-P1-25 | `qai quran diff --from A --to B` produces a human-reviewable difference report identifying every changed ayah with character-level ranges | differ test on a mutated fixture edition |
| AC-P1-26 | `qai quran rollback --to <prev>` restores the previous active version atomically, bumps `corpus_generation`, and writes an audit event | rollback test |
| AC-P1-27 | `doctor --quran` implements all 19 checks, runs strictly read-only, and `--deep` completes a full hash + round-trip verification in under 30 s for the reference edition | timed doctor run |
| AC-P1-28 | Two adapters exist for two structurally different dataset shapes, proving a new edition needs only an adapter + manifest (no core change) | adapter tests + code review |
| AC-P1-29 | Editorial reviewer has verified the sampled golden set against a recognized printed muṣḥaf; `verified_by`, `verified_at`, and `verification_method` are populated on the active edition | signed review record in `docs/reviews/` |
| AC-P1-30 | Full soak passes: import → validate → activate → 10,000 randomized lookups → `doctor --quran --deep`, with zero integrity findings and zero cache inconsistencies | soak job in nightly CI |
| AC-P1-31 | All Phase-1 ADRs (0101–0114) are `Accepted` and contain every §48 field | ADR lint |
| AC-P1-32 | Docs complete: corpus architecture, addressing spec, hashing spec, adapter authoring guide, import runbook, rollback runbook, citation spec | doc review checklist |

**Exit gate ritual:** a live walkthrough on a clean machine performing AC-P1-02, 03, 08, 10, 14, 23, 26, 27 in sequence, witnessed by the editorial reviewer.

---

## 10. Risks & Mitigations

| # | Risk | L | I | Mitigation |
|---|---|---|---|---|
| R1 | No suitable Quran dataset can be legally bundled | Medium | High | Sprint 1.0 starts before code; ADR-0101 pre-commits to the user-supplied fallback with a public-domain test fixture; product copy explains the import step |
| R2 | The chosen dataset has undetected textual defects | Medium | Critical | Independent reference-corpus comparison (QV-015) is a *Fatal* rule; named human editorial sign-off (AC-P1-29); adversarial fixtures prove the detector actually detects |
| R3 | Unicode/normalization decisions made hastily, corrupting text or breaking Phase 2 | High | Critical | ADR-0104 written before the importer; forbidden-code-point list explicit; grapheme-cluster-aware offsets from day one; NFC asserted, never applied silently to canonical text |
| R4 | Tokenization choice conflicts with Phase 2 morphological segmentation | High | High | ADR-0105 makes Phase-1 tokenization *surface-only and lossless*; morphological segmentation is a separate, additive layer that references `(surah, ayah, position)` and never rewrites tokens |
| R5 | Reference grammar changes later, invalidating stored citations | Medium | High | Grammar frozen in ADR-0102; canonical serialization is always pinned+fully-qualified; a `reference_grammar_version` is stored on every persisted citation so a future migration is mechanical |
| R6 | Hash definition changes, invalidating every reproducibility checksum | Low | High | ADR-0108 freezes exactly which bytes enter each hash; algorithm tag inside `ContentHash`; documented rehash-migration procedure |
| R7 | Basmala/numbering edge cases produce wrong ayah counts | High | Medium | Explicit `BasmalaPolicy` per surah (never inferred); golden fixtures for 1:1, 9:1, 27:30; QV-020 rule |
| R8 | SQLite row-per-ayah performance insufficient for whole-surah reads | Low | Medium | Covering indexes + `global_ayah_index` range scans; benchmarks gated in CI; ADR-0106 records the blob+index alternative and its migration path |
| R9 | Immutability triggers block legitimate maintenance (e.g. removing a `Removed` edition) | Medium | Low | Documented, audited offline maintenance command in a runbook; never available in normal operation |
| R10 | Scope creep into search because "it's almost free" | High | Medium | §2.2 fence; any search work is rejected in review and moved to Phase 2; the debug reader deliberately has no search box |
| R11 | Translation import turns into a large data-engineering project | Medium | Medium | Phase 1 ships translation *schema + importer + one translation*; additional translations are Phase 4 content work, not Phase 1 engineering |
| R12 | Editorial reviewer unavailable, blocking AC-P1-29 | Medium | High | Reviewer identified and scheduled during Sprint 1.0; sampled review (200 golden ayahs) sized to ≤ 3 engineer-days of reviewer time |

---

## 11. Definition of Done (Phase 1)

In addition to the Phase-0 DoD, every Phase-1 deliverable requires (§58, §46, §93):

- [ ] Cannot modify canonical text accidentally (trigger + type-level guard + test)
- [ ] Corpus-integrity tests exist and run in CI
- [ ] Unicode and normalization tests exist (normalization applies to *derived* data only)
- [ ] Preserves exact source addressing; canonical serialization is pinned
- [ ] Every output carries edition id, edition version, and source version
- [ ] Any generated/derived value is labeled and provenance-tagged (Layer B/C/D)
- [ ] Works without an LLM (architecture test forbids the dependency)
- [ ] Performance limits and result caps enforced
- [ ] Domain, repository, API, tool, and CLI tests all present
- [ ] Source-version tracking, checksum validation, round-trip tests, citation-resolution tests
- [ ] Difference report produced for any version change; rollback tested
- [ ] Derived artifacts record `corpus_generation` and are rebuildable

---

## 12. Handoff To Phase 2

| Asset | Location | Phase-2 usage |
|---|---|---|
| `Token` rows with exact offsets + separators | `quran_tokens`, `quran_token_separators` | input to normalization forms and morphological segmentation |
| `global_ayah_index`, `global_token_index` | `quran_ayahs`, `quran_tokens` | interval analysis, co-occurrence windows, distribution math |
| `QuranRef` grammar + resolver | `quran-core` | search result addressing and highlight ranges |
| `corpus_generation` counter | `quran_active_edition` | index staleness detection (`doctor` morphology/index checks) |
| `ToolResult` + `ReproducibilityData` | `application::tools` | every Phase-2 search tool inherits the contract; `normalization_rules` field finally gets populated |
| Citation resolver + `QuotationVerdict::MatchAfterDeclaredNormalization` | `citations` | verifying normalized-search quotations |
| Validation registry + QV-028 hook | `quran-corpus::validation` | proving index builds never alter canonical text |
| Adapter pattern | `quran-corpus::adapters` | morphology-dataset adapters follow the same shape |
| Golden + adversarial fixtures | `fixtures/quran/` | extended with normalization and root/lemma golden sets |
| Error namespaces `QAI-QUR-*`, reserved `QAI-NORM-*`, `QAI-IDX-*` | `domain::error` | Phase-2 codes |

**Handoff document:** `docs/plans/handoff-p1-to-p2.md` (task P1-T60), including the frozen
Unicode policy, tokenization contract, and a list of every place Phase 2 must record
`normalization_rule_set_version`.
````

---

## File 3 — `docs/plans/PHASE-2-QURAN-SEARCH-AND-LINGUISTICS.md`

````markdown
# Q-ai — Phase 2 Development Plan: Quran Search, Normalization & Linguistics

**Plan Version:** 1.0.0
**PRD Baseline:** Q-ai PRD v0.3.2
**Phase ID:** P2
**Phase Name:** Quran Search, Arabic Normalization, Morphology & Word Families
**PRD Traceability:** §8, §9, §11.1, §11.2, §11.3, §11.7, §12, §12.1, §19.1, §25.6, §27.1, §32.2, §35.1, §36.1, §40, §43 (Phase 2 both variants), §44.1, §46, §47, §50, §56, §67, §88 (items 3–4), §89
**Depends On:** Phase 0 (all), Phase 1 (all)
**Blocks:** Phase 3 (graph — needs roots/lemmas as nodes), Phase 4 (Web search & word inspector), Phase 7 (hybrid retrieval reuses the FTS layer), Phase 9 (linguistic research tools for agents)
**Target Duration:** 8 calendar weeks (≈ 22–24 engineer-weeks, 3 engineers + part-time Arabic linguist)
**Status:** Not Started

---

## 1. Phase Objective

Turn the canonical corpus of Phase 1 into a **searchable, linguistically explorable, fully
explainable research engine** — while guaranteeing that not one byte of canonical text changes.

**The Phase-2 promise (PRD §8, §8.3, §9.2, §46):**

> A user can type Arabic with any combination of missing diacritics, missing spaces, wrong
> hamza forms, or Persian code points, and Q-ai finds the canonical passage — then *shows
> exactly which normalization rules and segmentation made the match*, links every hit to a
> pinned canonical reference, exposes every root/lemma/morphological analysis **with its
> attribution and confidence**, never merges competing analyses, and never presents a
> computational suggestion as verified scholarship.

### 1.1 Non-negotiable invariants introduced here

| # | Invariant | Enforcement | PRD |
|---|---|---|---|
| I8 | Normalization never mutates canonical text; it only produces **derived** search copies | canonical tables untouched by index jobs; QV-028 re-verified after every build; separate physical tables/index dirs | §8, principle 9 |
| I9 | Every search result reports the exact ordered rule set applied | `NormalizationTrace` is a required field of every match; no constructor without it | §8.2, §8.3 |
| I10 | Every match maps back to exact canonical character offsets | bidirectional offset map, property-tested | §8.3, §13.2 |
| I11 | Multiple morphological analyses coexist; none is silently authoritative | `morphology_analyses` has no `is_correct` column; a *preferred dataset* is a per-request/config choice recorded in output | §9.2 |
| I12 | Machine-derived linguistic data is Layer D with algorithm, version, confidence, verification status | DB CHECK from Phase 0 + display labels | §6.4, §9.1 |
| I13 | Word-family relations distinguish exact-form / lemma / stem / root / computational / scholar-verified | closed `FamilyRelation` enum + provenance | §9.3 |
| I14 | Derived indexes are reproducible from `(source_version, rule_set_version, dataset_version)` and record their `corpus_generation` | index manifests + doctor staleness checks | §76, §50 |
| I15 | Numeric reports state their exact counting rules and never assert numerological significance | `CountingRules` required in output; fixed disclaimer for speculative patterns | §11.6 |
| I16 | Regex and pattern search are resource-bounded and cannot be used for DoS | compiled-size cap, step budget, timeout, no backtracking engine | §11.1, §40 |

---

## 2. Scope

### 2.1 In Scope

| # | Item | PRD |
|---|---|---|
| 1 | Versioned Arabic normalization rule engine with offset mapping | §8.1, §8.2 |
| 2 | Derived search forms per token, ayah, and surah (12 forms) | §8.1 |
| 3 | Full-text index (Arabic-aware) with exact + normalized + phrase fields | §32.2 |
| 4 | Letter-skeleton index for **concatenated / space-insensitive** search | §8.3 |
| 5 | `quran.search_exact`, `search_normalized`, `search_concatenated`, `search_phrase`, `search_regex` | §11.1 |
| 6 | Root, lemma, and stem lexicons with dataset attribution | §9.1, §43 P2 |
| 7 | Morphological analyses (multi-dataset, multi-analysis) + segmentation | §9.1, §9.2 |
| 8 | `quran.root_search`, `lemma_search`, `morphology`, `morphology_compare`, `pattern_search`, `affix_search` | §11.2 |
| 9 | Word-family engine (`quran.word_family`) with typed relations | §9.3, §11.2 |
# Part A — Database engine recommendation

Short answer: **one embedded relational spine (SQLite) as the single source of truth, plus purpose-built embedded engines mounted behind traits for full-text, vectors, and graph — all swappable to server-grade backends without touching domain code.**

## A.1 Recommended default profile (`local`, zero-config)

| Layer | Default (zero-config, embedded) | Why |
|---|---|---|
| Relational / canonical / metadata / jobs / audit | **SQLite** (WAL, `synchronous=FULL`, `foreign_keys=ON`) | Single file, no daemon, transactional, mature, perfect for immutable canonical rows + triggers; already assumed by PRD §32.1 |
| Full-text (Arabic-aware, BM25, phrase, proximity) | **Tantivy** (embedded, pure Rust, a directory on disk) | Custom tokenizer required for your normalization forms; BM25 + positional phrase queries; no server; PRD §32.2 already suggests it |
| Vector | **`sqlite-vec`** (default) → **LanceDB** (when corpora grow) | `sqlite-vec` keeps everything in the *same file* = truly zero-config; LanceDB gives real ANN (IVF/HNSW) still file-based |
| Graph | **SQLite adjacency tables + recursive CTEs** | PRD §32.4 explicitly forbids requiring a graph DB for local MVP; typed edges + provenance columns are trivially relational |
| Analytics (frequency, distribution, co-occurrence, collocation) | **SQLite aggregates**, optional **DuckDB** attach for heavy scans | Optional accelerator; never authoritative |
| Cache / offset maps / skeleton blobs | **`redb`** or plain files (optional) | Pure-Rust embedded KV; only if SQLite proves slow |

Everything lives under `~/.local/share/qai/` — one directory, no services, `qai serve` works offline.

## A.2 Recommended server profile (Phase 11/12, opt-in)

| Layer | Server backend |
|---|---|
| Relational | **PostgreSQL** |
| Full-text | Tantivy (still embedded per node) or **OpenSearch** |
| Vector | **Qdrant** (or `pgvector` if you want fewer services) |
| Graph | **Postgres recursive CTE** → **Apache AGE** → **Neo4j** only if graph queries become the bottleneck |

## A.3 The "open hand" — the abstraction contract

Make these four traits the *only* way anything touches storage. This is what actually keeps your hand open:

```rust
pub trait Database      { /* relational, tx, UnitOfWork */ }   // sqlite | postgres
pub trait FullTextIndex { /* index, search, delete, generation */ } // tantivy | fts5 | opensearch
pub trait VectorStore   { /* upsert, query, delete, dims */ }  // sqlite-vec | lance | qdrant | pgvector
pub trait GraphStore    { /* nodes, edges, neighbors, paths, bounded pattern */ } // sqlite-cte | age | kuzu | neo4j
```

Rules that make polyglot persistence safe (this is the part most projects get wrong):

1. **SQLite is the source of truth.** FTS, vector, and graph stores are *derived, rebuildable projections*. `qai index rebuild --all` must restore them from the relational store + source versions.
2. **Every derived store records `corpus_generation` + `rule_set_version` + `dataset_version`** in an index manifest. `qai doctor` compares them and reports drift (PRD §50's `✗ Morphology index differs from source version` is exactly this).
3. **No cross-store transactions.** Write relational first, then enqueue an idempotent reindex job. Reconciliation job detects orphans/tombstones (PRD §76).
4. **Canonical lookup never touches FTS/vector/graph** (PRD §40). Enforced by an architecture test.
5. **Deletion is tombstone + verified propagation**, never a raw delete in one store.

## A.4 Honest trade-offs

| Choice | Give up | Get |
|---|---|---|
| SQLite spine | Single-writer throughput; no network access | Zero config, ACID, trivial backup (`qai db backup`), triggers enforce immutability |
| Tantivy | A separate directory to manage; custom tokenizer work | Real BM25 + phrase + proximity over your normalized Arabic fields, offline |
| `sqlite-vec` default | Brute-force-ish at large N (fine to ~10⁵–10⁶ vectors) | Literally zero extra config; upgrade path to LanceDB/Qdrant is a trait swap |
| Relational graph | No Cypher; you write bounded CTEs | No new dependency; provenance columns per edge are natural; meets PRD §32.4 |

**Candidates to consciously reject for the default:** Neo4j/Kuzu (extra runtime or C++ dep for MVP), Milvus/Weaviate (server-only), sled (unmaintained-ish), embedded Postgres (defeats "light").

## A.5 ADRs to write

- **ADR-0001** Relational store: SQLite + `sqlx`, Postgres-portable SQL *(Phase 0)*
- **ADR-0201** Full-text engine: Tantivy + custom Arabic tokenizer *(Phase 2)*
- **ADR-0202** Graph store abstraction: relational adjacency + bounded CTE first *(Phase 3)*
- **ADR-0701** Vector store: `sqlite-vec` default, LanceDB/Qdrant adapters *(Phase 7)*
- **ADR-0702** Cross-store consistency, generation stamping, reconciliation *(Phase 7)*

---

# Part B — Phase 2 plan (continuation)

This continues `docs/plans/PHASE-2-QURAN-SEARCH-AND-LINGUISTICS.md` from §2.1 item 10.

````markdown
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

### 2.2 Explicitly Out of Scope (Phase 2)

- **Graph nodes/edges/paths** → Phase 3. Phase 2 produces roots/lemmas as *relational lexicon
  rows*; Phase 3 promotes them to graph nodes and adds `SAME_ROOT_AS`, `DERIVED_FROM` edges.
- **Semantic / vector / embedding search** → Phase 7. Phase 2 is lexical + morphological only.
  `quran.semantic_field` is deferred to Phase 3/7 (it needs concept nodes).
- **Web UI search page and word inspector** → Phase 4. Phase 2 ships API + CLI only.
- **Rhetorical/structural tools** (`repetition_analysis`, `parallel_structure`, `rhyme_analysis`,
  `address_shift`, `discourse_links`, `pronoun_reference`) → Phase 3, because they depend on
  graph annotations and entity/concept data. Exception: `near_duplicate_passages` ships here
  because it is purely lexical.
- **Tafsir/hadith/scripture search** → Phase 5/8. The `FullTextIndex` trait is designed for
  reuse but only Quran indexes are built now.
- **Transliteration search and phonetic approximation** → Phase 4 (needs ADR-0206 decision on
  standard; schema field reserved, rule stubs present, disabled by default per §8.1
  "where explicitly enabled").
- **Fuzzy/edit-distance spelling search (L8)** → shipped as *experimental, off by default*
  behind a config flag; full quality work is Phase 4.
- Cross-encoder reranking, query decomposition, multi-query retrieval → Phase 7.

### 2.3 Data-source prerequisite (hard gate, start in Phase 1 Sprint 1.5)

Phase 2 cannot ship morphology without a licensed dataset.

**ADR-0203 (Initial Quran morphology dataset)** must record:

- Candidate datasets, their coverage (all 77k+ tokens?), analysis depth (POS, features,
  lemma, root, stem, pattern), and **license/redistribution rights** (§38).
- **Alignment strategy**: how dataset token indices map to the Phase-1 `(edition, surah, ayah,
  position)` key. Any dataset whose tokenization disagrees with Phase-1 surface tokenization
  must be aligned via an explicit, auditable alignment table — *never* by re-tokenizing
  canonical text.
- **Attribution string** shown in the UI for every analysis from that dataset.
- Whether the dataset supplies **one** analysis per token or **multiple** (I11 must work either way).
- Root normalization convention (bare letters vs. with separators, hamza treatment).

**Fallback if no dataset can be bundled:** Phase 2 ships the full multi-analysis schema,
importer, alignment validator, and tools; a **public-domain test lexicon** covering the
fixture edition powers the tests; users run `qai quran morphology import <manifest>`.
Root/lemma search degrades to "no dataset active" with a clear, actionable error — never to
guessed data. This must be decided in ADR-0203, not improvised.

**Non-negotiable:** Q-ai must **never** generate roots or lemmas with an LLM and store them as
if they were dataset-supplied. If a computational analyzer is used at all, it writes Layer D
rows with `Attribution::Computational`, confidence, and `verification_status = needs_review`
(§6.4, §9.2, §10.6).

---

## 3. Normalization Architecture

### 3.1 Design

```text
canonical text (immutable, Phase 1)
        │  read-only
        ▼
┌──────────────────────────────────────────────────────────────┐
│  NormalizationPipeline (rule_set_id @ version)               │
│                                                              │
│  rule₁ ──► rule₂ ──► … ──► ruleₙ                             │
│    │         │              │                                │
│    └─ SpanMap └─ SpanMap    └─ SpanMap                       │
│                    │                                         │
│           composed ▼                                         │
│        CompositeSpanMap: derived_range ⇄ canonical_range     │
└──────────────────────────────────────────────────────────────┘
        │
        ├──► derived form strings (stored: quran_token_forms, quran_ayah_forms)
        ├──► FTS fields (Tantivy)
        ├──► skeleton strings + n-gram index
        └──► NormalizationTrace (returned with every match)
```

### 3.2 Rule catalog (frozen by ADR-0204)

Each rule has a stable `RuleId`, a version, a pure implementation, and a documented character
mapping table. Rules are **composable and order-significant**; the ordered list *is* the profile.


````markdown
| RuleId | Name | Effect | Notes |
|---|---|---|---|
| `N01` | `whitespace_collapse` | Runs of whitespace → single U+0020; trim ends | Always first. Idempotent |
| `N02` | `strip_tatweel` | Remove U+0640 (ـ) | Pure deletion |
| `N03` | `strip_harakat` | Remove U+064B–U+0652 (fathatan, dammatan, kasratan, fatha, damma, kasra, shadda, sukun) + U+0656, U+0657, U+0658, U+0659, U+065A–U+065F | The core "ignore diacritics" rule |
| `N04` | `strip_quranic_marks` | Remove U+06D6–U+06ED (small high signs, sajdah sign, rub-el-hizb, waqf marks), U+06DD (end of ayah), U+0615, U+0617–U+061A, U+06E5, U+06E6 | Uthmani-specific annotation marks |
| `N05` | `strip_superscript_alef` | Remove U+0670 (ٰ) | Separated from N03 because it changes reading, not just vowelling |
| `N06` | `normalize_hamza_forms` | أ(U+0623) إ(U+0625) آ(U+0622) → ا(U+0627); ؤ(U+0624) → و; ئ(U+0626) → ي; ء(U+0621) → ∅ or ا (sub-option) | Configurable sub-flags; documented mapping table |
| `N07` | `normalize_wasla` | ٱ(U+0671) → ا(U+0627) | Very common in Uthmani text |
| `N08` | `normalize_alif_maqsura` | ى(U+0649) → ي(U+064A) | Direction fixed by ADR-0204 |
| `N09` | `normalize_ta_marbuta` | ة(U+0629) → ه(U+0647) | Optional per §8.1 |
| `N10` | `normalize_persian_codepoints` | ک(U+06A9)→ك; ی(U+06CC)→ي; ه(U+06C1/U+06C0)→ه; گ ژ چ پ preserved but flagged; Persian ي/ك variants folded | Critical for Persian-keyboard users |
| `N11` | `strip_zero_width_and_bidi` | Remove U+200B–U+200F, U+202A–U+202E, U+2066–U+2069, U+FEFF | Security + correctness |
| `N12` | `strip_punctuation` | Remove Arabic and Latin punctuation | Off in exact profiles |
| `N13` | `fold_digits` | ٠–٩ (U+0660–0669), ۰–۹ (U+06F0–06F9) → ASCII 0–9 | For reference/number queries |
| `N14` | `strip_pause_marks` | Remove waqf letters ۖ ۗ ۘ ۙ ۚ ۛ (subset of N04, separately addressable) | Allows "keep marks but drop waqf" |
| `N15` | `expand_presentation_forms` | ﻻ and Arabic Presentation Forms A/B → base sequences (limited NFKC) | Never applied to canonical text |
| `N16` | `nfc` | Canonical composition | Asserted, not silently applied, on canonical (Phase 1); applied to *queries* |
| `N17` | `remove_spaces` | Delete all U+0020 → space-insensitive skeleton | Enables §8.3 concatenated search |
| `N18` | `strip_definite_article` | Leading ال / لل when followed by ≥2 letters | **Heuristic**, labeled `Heuristic` in the trace |
| `N19` | `strip_conjunction_prefix` | Leading و / ف | Heuristic |
| `N20` | `strip_preposition_prefix` | Leading ب / ل / ك | Heuristic |
| `N21` | `strip_pronoun_suffix` | Trailing ه، ها، هم، هن، هما، ك، كم، كن، نا، ي، ني | Heuristic |
| `N22` | `dedupe_repeated_letters` | Collapse ≥3 identical letters to 2 | Experimental, off by default |
| `N23` | `transliterate` | Arabic → Latin per configured standard | **Reserved, Phase 4** (ADR-0206) |
| `N24` | `phonetic_key` | Arabic → phonetic approximation key | **Reserved, experimental**, off by default (§8.1 "where explicitly enabled") |

**Rule implementation contract**

```rust
pub trait NormalizationRule: Send + Sync {
    fn id(&self) -> RuleId;
    fn version(&self) -> SemVer;
    fn description(&self) -> &'static str;
    fn kind(&self) -> RuleKind; // Deterministic | Heuristic

    /// Pure. Must produce a SpanMap so offsets remain reversible (I10).
    fn apply(&self, input: &NormalizedText) -> NormalizedText;

    /// True if applying twice equals applying once. Property-tested.
    fn is_idempotent(&self) -> bool;
}

pub enum RuleKind {
    /// Pure code-point/whitespace transformation with a published mapping table.
    Deterministic,
    /// Pattern-based approximation of morphology. MUST be surfaced to the user
    /// as a heuristic in the NormalizationTrace (§8.3 "explanation of the
    /// normalization and segmentation used").
    Heuristic,
}
```

### 3.3 Normalization profiles (PRD §8.2 strictness ladder)

A **profile** is an ordered, versioned rule list with a stable id. Profiles — not individual
rules — are what users select, what indexes are built from, and what appears in
`reproducibility.normalization_rule_set`.

| Profile id | §8.2 label | Rules (in order) | Indexed? |
|---|---|---|---|
| `L0.exact` | Exact canonical | — (identity) | ✅ field `text_exact` |
| `L1.ws` | Exact after whitespace normalization | N01, N11, N16 | ✅ `text_ws` |
| `L2.marks` | Ignore Quranic marks | L1 + N04, N14 | ✅ `text_marks` |
| `L3.diacritics` | Ignore diacritics | L2 + N03, N05, N02 | ✅ `text_bare` (primary search field) |
| `L4.hamza` | Normalize hamza/alif | L3 + N07, N06, N08 | ✅ `text_hamza` |
| `L5.codepoints` | Normalize Arabic/Persian code points | L4 + N10, N13, N09, N15 | ✅ `text_folded` (most permissive indexed) |
| `L6.skeleton` | Space-insensitive skeleton | L5 + N12, N17 | ✅ separate skeleton store + n-gram index |
| `L7.affix` | Morphological (heuristic affix) search | L5 + N18, N19, N20, N21 | ✅ `text_affix` (token-level only) |
| `L8.fuzzy` | Fuzzy spelling search | L5 + Levenshtein at query time (N22 optional) | ⚠️ query-time only, **experimental, off by default** |

Rules:

- **Profiles are append-only.** Changing a profile's rule list requires a **new version**
  (`L3.diacritics@2.0.0`), a full index rebuild, and a `doctor` drift report. Never edited in place.
- `L7` and `L8` results are always tagged `contains_heuristic_rules = true`, and the UI/CLI must
  render "matched using heuristic affix stripping" (§8.3).
- A **true** morphological search (root/lemma) is *not* `L7`; it is §7's lexicon path. `L7` exists
  only for users who type a surface word with attached particles and have no morphology dataset.

### 3.4 Offset mapping (`SpanMap`)

```rust
/// Bidirectional, composable offset map. Every rule emits one; the pipeline composes them.
pub struct SpanMap {
    /// Monotonic segments: (derived_range, canonical_range, provenance_rule)
    segments: Vec<SpanSegment>,
}

impl SpanMap {
    /// Map a match in derived space back to canonical char + byte offsets (I10).
    pub fn to_canonical(&self, derived: Range<u32>) -> CanonicalSpan;
    pub fn to_derived(&self, canonical: Range<u32>) -> Option<Range<u32>>;
    pub fn compose(self, next: SpanMap) -> SpanMap;
}

pub struct CanonicalSpan {
    pub char_range: Range<u32>,   // grapheme-cluster indices into ayah.text
    pub byte_range: Range<u32>,
    pub token_range: Range<u16>,  // inclusive token positions touched
    pub exact: bool,              // false if the match straddles a deleted region ambiguously
}
```

**Why this matters:** without `SpanMap`, a "found it" result cannot be highlighted in the
canonical Uthmani text, cannot be cited to a character range, and cannot be re-verified by the
citation resolver. `SpanMap` is the single most heavily property-tested component of Phase 2.

**Property tests (all must hold for every ayah × every profile):**

1. `to_canonical(to_derived(r)) ⊇ r` for every canonical range `r` that survives normalization.
2. `to_canonical` output ranges are within `0..ayah.text.len()` and land on grapheme boundaries.
3. Composition is associative: `(a∘b)∘c == a∘(b∘c)`.
4. For identity profile `L0`, `to_canonical` is the identity map.
5. Every derived match's canonical span, when sliced from canonical text and re-normalized with
   the same profile, contains the matched derived substring.

### 3.5 Derived forms (PRD §8.1)

Stored per **token** and per **ayah**. Surah-level forms are computed on demand (concatenating
ayah forms) to keep storage bounded.

| Form | Level | Profile | Storage |
|---|---|---|---|
| `surface_uthmani` | token, ayah | `L0` (canonical, Phase 1) | already in Phase 1 tables |
| `simple` | token, ayah | `L2.marks` | `quran_*_forms` |
| `bare` | token, ayah | `L3.diacritics` | `quran_*_forms` |
| `hamza_folded` | token, ayah | `L4.hamza` | `quran_*_forms` |
| `folded` | token, ayah | `L5.codepoints` | `quran_*_forms` |
| `skeleton` | ayah, surah(virtual) | `L6.skeleton` | `quran_skeletons` + n-gram index |
| `affix_stripped` | token | `L7.affix` | `quran_token_forms` |
| `lemma_form` | token | — (lexicon, §7) | `quran_token_analyses` |
| `root_form` | token | — (lexicon, §7) | `quran_token_analyses` |
| `stem_form` | token | — (lexicon, §7) | `quran_token_analyses` |
| `transliteration` | token, ayah | `L23` reserved | column reserved, NULL in Phase 2 |
| `phonetic` | token | `L24` reserved | column reserved, NULL in Phase 2 |

Every derived-form row records `rule_set_id`, `rule_set_version`, `corpus_generation`, and a
`provenance_id` at Layer D (§6.4) with `Attribution::Computational { algorithm: "normalizer",
version, parameters_hash }`.

---

## 4. Full-Text Index Architecture

### 4.1 Abstraction

```rust
#[async_trait]
pub trait FullTextIndex: Send + Sync {
    fn backend(&self) -> FtsBackend;                 // Tantivy | Fts5 | OpenSearch
    fn manifest(&self) -> IndexManifest;             // generation + versions (I14)

    async fn create(&self, schema: &FtsSchema) -> Result<()>;
    async fn add_batch(&self, docs: Vec<FtsDoc>) -> Result<()>;
    async fn commit(&self) -> Result<CommitStamp>;
    async fn search(&self, q: &FtsQuery, opts: &SearchOpts) -> Result<FtsResults>;
    async fn count(&self, q: &FtsQuery) -> Result<u64>;
    async fn delete_by_generation(&self, gen: u64) -> Result<u64>;
    async fn stats(&self) -> Result<FtsStats>;
    async fn verify(&self) -> Result<FtsIntegrityReport>;
}

pub struct IndexManifest {
    pub index_id: String,                    // "quran.ayah.v1"
    pub schema_version: u32,
    pub corpus_generation: u64,              // from Phase 1
    pub edition_id: EditionId,
    pub edition_version: SemVer,
    pub rule_set_versions: BTreeMap<ProfileId, SemVer>,
    pub tokenizer_version: SemVer,
    pub morphology_dataset_versions: BTreeMap<SourceId, SemVer>,
    pub built_at: Timestamp,
    pub doc_count: u64,
    pub content_hash: ContentHash,           // over the index manifest, for drift detection
}
```

### 4.2 Tantivy implementation (ADR-0201)

**Two indexes** (deliberately not one):

| Index | Doc granularity | Purpose |
|---|---|---|
| `quran.ayah.v1` | one doc per ayah | phrase, proximity, BM25, regex, ayah-level retrieval |
| `quran.token.v1` | one doc per token | affix/root/lemma joins, exact form counting, word-level highlight |

**`quran.ayah.v1` schema**

```rust
// Stored fields
field!("edition_id",  STRING | STORED);
field!("surah",       U64    | STORED | INDEXED | FAST);
field!("ayah",        U64    | STORED | INDEXED | FAST);
field!("global_index",U64    | STORED | INDEXED | FAST);
field!("juz",         U64    | INDEXED | FAST);
field!("page",        U64    | INDEXED | FAST);
field!("revelation",  STRING | INDEXED);          // makki|madani
field!("generation",  U64    | INDEXED | FAST);   // corpus_generation, for atomic swap

// Text fields — one per indexed profile, each with positions for phrase queries
field!("text_exact",  text_opts("ar_exact",  IndexRecordOption::WithFreqsAndPositions) | STORED);
field!("text_ws",     text_opts("ar_ws",     …));
field!("text_marks",  text_opts("ar_marks",  …));
field!("text_bare",   text_opts("ar_bare",   …));   // primary
field!("text_hamza",  text_opts("ar_hamza",  …));
field!("text_folded", text_opts("ar_folded", …));
field!("text_affix",  text_opts("ar_affix",  …));

// Lexicon fields (populated from §7 when a morphology dataset is active)
field!("roots",       text_opts("keyword", WithFreqsAndPositions));
field!("lemmas",      text_opts("keyword", WithFreqsAndPositions));
field!("stems",       text_opts("keyword", WithFreqsAndPositions));
field!("pos_tags",    text_opts("keyword", WithFreqs));
field!("patterns",    text_opts("keyword", WithFreqs));
```

**Custom tokenizers** (`quran-search::tokenizer`): each `ar_*` tokenizer is
`WhitespaceSplitter → ProfileNormalizer(profile) → TokenEmitter(with byte offsets)`.
Critically, the tokenizer **reuses the same `NormalizationPipeline`** as the query path, so a
query and a document can never disagree. This is enforced by a test that normalizes 5,000
random ayah substrings through both paths and asserts equality.

**Atomic index activation (mirrors Phase 1 §D1.3):**

```text
build into  <data_dir>/index/quran.ayah.v1/gen-<N>/     (staging dir)
verify      doc_count, sampled round-trip, manifest hash
flip        write index_pointers row (index_id -> gen-N) in ONE SQLite tx
retain      previous generation until the next successful build (single-step rollback)
GC          `qai index gc` removes generations older than the retained one
```

A partially built index can never serve queries (I5 extended to indexes).

### 4.3 Query model

```rust
pub enum FtsQuery {
    Term      { field: FieldId, term: String },
    Phrase    { field: FieldId, terms: Vec<String>, slop: u32, ordered: bool },
    Boolean   { must: Vec<FtsQuery>, should: Vec<FtsQuery>, must_not: Vec<FtsQuery> },
    Range     { field: FieldId, lo: Option<i64>, hi: Option<i64> },
    Regex     { field: FieldId, pattern: String },   // bounded; see I16
    All,
}

pub struct SearchOpts {
    pub limit: u32,                 // hard cap 1000 (§40 memory-bounded)
    pub offset: u32,
    pub filters: Vec<Filter>,       // surah, juz, page, revelation place, range
    pub order: ResultOrder,         // CanonicalOrder | Relevance | Frequency
    pub highlight: bool,
    pub timeout: Duration,          // default 2 s, hard ceiling 10 s
    pub explain: bool,              // include per-hit scoring breakdown
}
```

**Regex safety (I16):** patterns are compiled with the `regex-automata` DFA engine (no
backtracking ⇒ no catastrophic blowup), with `size_limit` = 1 MiB, `dfa_size_limit` = 4 MiB,
max pattern length 512 chars, and a wall-clock bud