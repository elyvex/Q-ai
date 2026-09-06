Continuing. First the tail of **File 2** (Phase 1) that got cut off, then the complete **File 3** (Phase 2).

---

## File 2 (continued) — `docs/plans/PHASE-1-CANONICAL-QURAN-CORE.md`


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
Continuing `docs/plans/PHASE-2-QURAN-SEARCH-AND-LINGUISTICS.md` from §4.3.

````markdown
**Regex safety (I16):** patterns compile with the `regex-automata` DFA engine (no backtracking
⇒ no catastrophic blowup), with `size_limit = 1 MiB`, `dfa_size_limit = 4 MiB`, max pattern
length 512 chars, and a wall-clock budget of 3 s enforced by a cancellation token checked every
4,096 documents. Additional guards:

- Patterns are only allowed against **indexed normalized fields**, never against a raw scan of
  canonical text.
- `.*` / `.+` at pattern start is rejected with an actionable error suggesting an anchored
  alternative (prevents full-index term-dictionary walks).
- Every regex search records `terms_examined` and `documents_scanned` in the result trace, and a
  soft warning is emitted above configurable thresholds.
- Regex search is rate-limited per principal (default 10/min) and is denied for agent tool
  calls unless the agent's policy explicitly grants `quran.search_regex`.

### 4.4 Concatenated / space-insensitive search (PRD §8.3)

The single most distinctive Phase-2 feature and the one with the worst naive complexity.
Design: **skeleton + trigram candidate generation + verified substring match**.

```text
Query "بسمالله"
   │
   ├─ normalize with L6.skeleton  →  "بسمالله"   (already spaceless)
   │
   ├─ generate character trigrams → [بسم, سما, مال, الل, لله]
   │
   ├─ probe skeleton trigram index (ayah-level, then surah-window level)
   │      → candidate set of ayah ids (and cross-ayah windows)
   │
   ├─ for each candidate: exact substring search in its skeleton string
   │      (memchr/two-way; skeletons are short)
   │
   └─ for each hit: SpanMap → canonical char range → token range
          → verify by re-normalizing the canonical slice
          → emit match with NormalizationTrace + segmentation explanation
```

**Cross-token and cross-ayah matching**

Per §8.3 ("Search across token boundaries"), the skeleton store holds three levels:

| Level | Unit | Purpose |
|---|---|---|
| `ayah` | one skeleton per ayah | most queries |
| `window` | sliding window of 3 consecutive ayahs, stride 1 | phrases straddling an ayah boundary |
| `surah` | virtual, built on demand from ayah skeletons | rare very long queries |

Window matches are **deduplicated** against ayah matches and are labeled
`spans_ayah_boundary = true` so the UI can show the reader that a match crosses a verse break —
never silently presenting a cross-verse fragment as one verse (principle 1, §13.4).

**Segmentation explanation (required by §8.3)**

Every concatenated match returns:

```json
{
  "reference": "quran:hafs-uthmani@1.0.0:1:1",
  "canonical_text": "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ",
  "matched_canonical_char_range": [0, 12],
  "matched_tokens": [1, 2],
  "match_explanation": {
    "profile": "L6.skeleton@1.0.0",
    "rules_applied": ["N01","N11","N16","N04","N14","N03","N05","N02","N07","N06","N08","N10","N13","N09","N15","N12","N17"],
    "contains_heuristic_rules": false,
    "query_skeleton": "بسمالله",
    "matched_skeleton": "بسمالله",
    "segmentation": [
      { "query_part": "بسم",  "canonical_token": 1, "canonical_surface": "بِسْمِ" },
      { "query_part": "الله", "canonical_token": 2, "canonical_surface": "ٱللَّهِ" }
    ],
    "spans_ayah_boundary": false,
    "spans_token_boundary": true
  }
}
```

**Performance target:** p99 < 150 ms for a 3–20 char query over the full corpus, cold cache.
Achieved because the ayah skeleton corpus is ~350 KB of text total; the trigram index is a
compact posting list held in `redb` or a Tantivy keyword field.

---

## 5. Search Tools (PRD §11.1)

All five are `side_effect_class = ReadOnly` (§29), conform to the Phase-1 `ToolResult` contract
(§12), and finally populate the `normalization_rules` field that Phase 1 left empty.

### 5.1 `quran.search_exact`

```jsonc
// input
{
  "text": "ٱلرَّحْمَٰنِ",
  "edition": "hafs-uthmani@1.0.0",     // optional; defaults to Active
  "field": "text_exact",               // text_exact | text_ws
  "match": "substring",                // substring | whole_token | ayah_prefix
  "filters": { "surah": [1,2,3], "juz": null, "revelation_place": null },
  "limit": 100, "offset": 0,
  "order": "canonical"                 // canonical | relevance
}
```

Searches canonical surface text with **no linguistic expansion** (§11.1). Zero normalization
beyond the explicitly requested `L0`/`L1` profile. If the query contains characters absent from
the edition (e.g. a Persian `ک`), the tool returns zero results **plus a warning** suggesting
`quran.search_normalized` with `L5.codepoints` — it never silently folds.

### 5.2 `quran.search_normalized`

```jsonc
{
  "text": "الرحمن",
  "profile": "L3.diacritics",          // or explicit rules[] for full control
  "rules": null,                       // e.g. ["N01","N03","N06"] overrides profile
  "match": "whole_token",
  "filters": {...}, "limit": 100,
  "explain": true
}
```

- Either `profile` **or** `rules` (never both). If `rules` are given, the effective set is
  hashed into an ad-hoc profile id `adhoc:<sha256[..12]>` and the query is served by
  normalizing candidates at query time from the closest superset index, then verifying.
- `explain: true` returns the full `NormalizationTrace` (§3.4) plus per-hit BM25 breakdown.

### 5.3 `quran.search_concatenated`

```jsonc
{
  "text": "بسمالله",
  "allow_cross_ayah": true,
  "max_ayah_span": 3,
  "filters": {...}, "limit": 50
}
```

Per §4.4. Always returns `segmentation` and `spans_*` flags.

### 5.4 `quran.search_phrase`

```jsonc
{
  "text": "الحمد لله رب العالمين",
  "profile": "L3.diacritics",
  "mode": "ordered_exact",   // ordered_exact | ordered_near | unordered_near
  "slop": 0,                 // max intervening tokens for *_near
  "filters": {...}
}
```

Backed by Tantivy positional phrase queries. `unordered_near` becomes a boolean-AND with a
post-filter enforcing a token-distance window computed from stored positions (§11.1).

### 5.5 `quran.search_regex`

```jsonc
{
  "pattern": "^ا?ل?رحم",
  "field": "text_bare",
  "limit": 100,
  "timeout_ms": 3000
}
```

Guarded per I16. Result includes `terms_examined`, `documents_scanned`, `truncated: bool`.

### 5.6 Unified result shape

```rust
pub struct SearchHit {
    pub reference: String,                 // pinned canonical reference
    pub deep_link: String,
    pub quotation: QuranQuotation,         // from Phase 1; carries edition + hash
    pub canonical_span: CanonicalSpan,     // I10
    pub matched_tokens: Vec<u16>,
    pub score: Option<f32>,                // BM25; None for canonical-order tools
    pub score_explain: Option<ScoreExplain>,
    pub explanation: NormalizationTrace,   // I9 — no constructor without it
    pub warnings: Vec<Warning>,
}
```

`ToolResult<Vec<SearchHit>>` additionally carries `total_matches` (exact count, computed by a
separate `count` call — never estimated), `truncated`, and the §12 reproducibility block whose
`normalization_rule_set` is now populated.

---

## 6. Lexicon & Morphology (PRD §9.1, §9.2, §11.2)

### 6.1 Multi-dataset, multi-analysis model (I11)

```rust
pub struct MorphologyDataset {
    pub source_id: SourceId,
    pub slug: String,                  // "quranic-corpus", "camel-morph", …
    pub name: String,
    pub version: SemVer,
    pub source_version_id: SourceVersionId,
    pub license: LicenseRecord,
    pub trust_level: TrustLevel,        // typically ScholarReviewed or PublisherVerified
    pub attribution_display: String,    // exact string shown in the UI (ADR-0203)
    pub aligned_edition_id: EditionId,
    pub alignment_method: AlignmentMethod,
    pub coverage: Coverage,             // tokens_covered / tokens_total, per-surah breakdown
    pub root_convention: RootConvention,
    pub provides_multiple_analyses: bool,
    pub status: DatasetStatus,
}

pub enum AlignmentMethod {
    /// Dataset uses the same (surah, ayah, position) keys — verified 1:1.
    DirectKey,
    /// Dataset tokenization differs; an explicit, auditable alignment table maps
    /// dataset tokens -> Phase-1 token positions. NEVER re-tokenizes canonical text.
    AlignmentTable { table_source_version: SourceVersionId, unmatched_count: u64 },
}

pub struct MorphologicalAnalysis {
    pub id: AnalysisId,
    pub edition_id: EditionId,
    pub surah: SurahNumber, pub ayah: AyahNumber, pub position: u16,
    pub dataset_id: SourceId,
    pub analysis_index: u16,           // 0..n for multiple competing analyses
    pub segments: Vec<Morpheme>,       // prefixes + stem + suffixes, in order
    pub lemma_id: Option<LemmaId>,
    pub root_id: Option<RootId>,
    pub stem: Option<String>,
    pub pattern: Option<String>,       // e.g. فَعَلَ / فَاعِل
    pub features: MorphFeatures,       // POS, person, gender, number, case, mood, voice,
                                       // aspect, state, definiteness, verb_form (I–X)
    pub dependency: Option<DependencyRelation>,
    pub confidence: Option<Confidence>,
    pub verification_status: VerificationStatus,
    pub provenance_id: ProvenanceId,   // Layer B if dataset-supplied, D if computed
}

pub struct Morpheme {
    pub index: u16,
    pub kind: MorphemeKind,            // Prefix | Stem | Suffix
    pub surface: String,               // as it appears in the canonical token
    pub tag: String,                   // dataset-native tag, preserved verbatim
    pub normalized_tag: Option<UnifiedTag>, // mapped to Q-ai's unified tagset (Layer D)
    pub char_range: Option<Range<u32>>,     // within the token surface, when supplied
}
```

**There is no `is_correct`, `is_primary`, or `selected` column** (I11). Selection is a *query-time
policy*:

```rust
pub enum AnalysisPolicy {
    /// Return every analysis from every active dataset (default for research views).
    All,
    /// Prefer a configured dataset, but always report that others exist.
    PreferDataset { dataset: SourceId },
    /// Only analyses from a specific dataset.
    OnlyDataset { dataset: SourceId },
    /// Only human-verified analyses.
    VerifiedOnly,
}
```

Every response containing analyses includes
`analysis_sources: [{dataset, version, attribution_display, analyses_returned, analyses_suppressed}]`
so suppression is always visible (§9.2 "avoid silently selecting one as unquestionably correct").

### 6.2 Root, lemma, stem lexicons

```rust
pub struct Root {
    pub id: RootId,
    pub letters: String,               // normalized per RootConvention, e.g. "رحم"
    pub letters_spaced: String,        // display form "ر ح م"
    pub letter_count: u8,
    pub dataset_id: SourceId,          // roots are dataset-scoped; same letters across
                                       // datasets are linked, not merged
    pub canonical_root_id: Option<RootId>, // cross-dataset unification (Layer D, reviewable)
    pub token_count: u32,              // occurrences in this edition
    pub lemma_count: u32,
    pub provenance_id: ProvenanceId,
}

pub struct Lemma {
    pub id: LemmaId,
    pub form: String,
    pub form_bare: String,
    pub root_id: Option<RootId>,
    pub pos: Option<UnifiedTag>,
    pub gloss: Option<String>,         // attributed, optional
    pub dataset_id: SourceId,
    pub token_count: u32,
    pub provenance_id: ProvenanceId,
}
```

**Cross-dataset root unification is a Layer D suggestion, queued for review** (§10.6 pattern
from Phase 0's `review_queue`). Two datasets that spell a root differently are *linked with a
confidence*, never merged. This prevents the exact failure mode PRD §9.2 warns about.

### 6.3 Import pipeline (`quran.morphology.import` job)

```text
 1  claim source version (Downloaded|Staged)               [checkpoint: claimed]
 2  verify hashes                                          [checkpoint: hashed]
 3  detect format, select adapter                          [checkpoint: adapter]
 4  parse → intermediate morphology format                 [checkpoint: parsed]
 5  ALIGN to Phase-1 tokens (DirectKey or AlignmentTable)   [checkpoint: aligned]
 6  validate (MV-001 … MV-018, §6.4)                        [checkpoint: validated]
 7  build root / lemma / stem lexicons                      [checkpoint: lexicons]
 8  write Layer B/D provenance rows                         [checkpoint: provenance]
 9  stage into morph_stg_* tables                           [checkpoint: staged]
10  coverage report + unmatched-token report                [checkpoint: coverage]
11  diff vs previous dataset version                        [checkpoint: diffed]
12  → Staged; approval request                              [terminal]
```

Activation (`qai quran morphology activate <dataset>@<v>`) is a separate human-approved,
single-transaction pointer flip that also **enqueues** an FTS rebuild for the `roots`, `lemmas`,
`stems`, `pos_tags`, `patterns` fields — never mutating them in place.

**Alignment is the highest-risk step.** Rules:

- `DirectKey` requires a 100 % key match; any mismatch is `Fatal`.
- `AlignmentTable` requires the table to be a declared source file with its own hash; unmatched
  tokens are reported per surah and gate approval above a configurable threshold (default 0.5 %).
- Under **no circumstance** may alignment modify `quran_tokens` (I8, Phase-1 I1).

### 6.4 Morphology validation rules

| Rule | Check | Severity |
|---|---|---|
| MV-001 | Every analysis references an existing `(edition, surah, ayah, position)` | Fatal |
| MV-002 | No analysis references a token outside the active edition | Fatal |
| MV-003 | `analysis_index` values per token are dense `0..n` | Fatal |
| MV-004 | Morpheme surfaces concatenate to the token surface (modulo declared normalization) | Error |
| MV-005 | Exactly one `Stem` morpheme per analysis (or documented exception list) | Error |
| MV-006 | Morpheme `char_range`s, when supplied, are non-overlapping and within the token | Error |
| MV-007 | Every `root_id` / `lemma_id` resolves within the same dataset | Fatal |
| MV-008 | Root letters conform to the declared `RootConvention` | Error |
| MV-009 | POS tags map to the unified tagset, or are recorded as unmapped with a warning | Warning |
| MV-010 | Feature combinations are internally consistent (e.g. no `mood` on a noun) | Warning |
| MV-011 | Coverage ≥ declared threshold; per-surah gaps reported | Error |
| MV-012 | Dataset version, license, and attribution string are all present and non-empty | Fatal |
| MV-013 | Every analysis row has a provenance row at Layer B or D (never A) | Fatal |
| MV-014 | Computationally derived analyses carry algorithm + version + confidence | Fatal |
| MV-015 | No analysis is `human_verified` without a reviewer | Fatal |
| MV-016 | Alignment unmatched-token ratio ≤ threshold | Error |
| MV-017 | Round-trip: staged rows re-serialize to a hash-stable intermediate document | Fatal |
| MV-018 | **Canonical tables are byte-identical before and after the import** (QV-028 re-run) | Fatal |

MV-018 is the mechanical guarantee of I8 and is re-run after *every* index or lexicon build.

### 6.5 Morphology tools

**`quran.morphology`** — token segmentation and all analyses.

```jsonc
// input
{ "reference": "1:1:1", "policy": "all", "include_features": true, "include_dependency": false }
```

Returns, per analysis: dataset attribution, segmentation with per-morpheme surfaces and ranges,
lemma, root, pattern, features, confidence, verification status. Plus a top-level
`analyses_by_dataset` summary and `disagreements` array listing fields where datasets differ
(§9.2 comparison support).

**`quran.morphology_compare`** — explicit side-by-side comparison (§11.2).

```jsonc
{ "reference": "2:255:5", "datasets": ["quranic-corpus","camel-morph"], "fields": ["root","lemma","pos","pattern"] }
```

Output is a matrix with an `agreement` verdict per field
(`Identical | CompatibleVariant | Conflicting | OnlyInOne`), **never a resolution**. Per PRD
§21.4 and §92:21, conflicting analyses are presented side-by-side with attribution; the tool has
no "winner" field and no synthesis mode.

**`quran.root_search`** (§11.2)

```jsonc
{ "root": "ر ح م", "dataset": null, "group_by": "lemma",
  "include_tokens": true, "filters": {...}, "limit": 500 }
```

Accepts spaced, unspaced, or partially normalized root input; resolves through the dataset's
`RootConvention` and reports which convention/dataset matched. Returns occurrence counts, the
lemma breakdown, and (optionally) every token occurrence with canonical references.

**`quran.lemma_search`** — all inflections of a lemma, grouped by surface form with counts.

**`quran.pattern_search`** (§11.2) — search morphological patterns / verb forms
(`فَعَّلَ`, form II, active participle, etc.). Requires a dataset supplying `pattern` or
`verb_form`; otherwise returns a typed "capability unavailable" error naming the missing field —
never a guess.

**`quran.affix_search`** (§11.2) — search by prefix, suffix, attached pronoun, conjunction,
article, or preposition. Two backends, and the result always says which was used:

| Backend | Condition | Label |
|---|---|---|
| Dataset morphemes | a morphology dataset is active | `Attested (dataset: …)` |
| `L7.affix` heuristic | no dataset, or user requests it | `Heuristic (pattern-based)` |

---

## 7. Word Family Engine (PRD §9.3, §11.2)

### 7.1 Relation taxonomy (I13)

```rust
pub enum FamilyRelation {
    /// Identical canonical surface form.
    SameExactForm,
    /// Same normalized form under a stated profile.
    SameNormalizedForm { profile: ProfileId },
    /// Same lemma per a named dataset.
    SameLemma { dataset: SourceId },
    /// Same stem per a named dataset.
    SameStem { dataset: SourceId },
    /// Same root per a named dataset.
    SameRoot { dataset: SourceId },
    /// Derivational relation supplied by a dataset (e.g. verbal noun of a verb).
    DerivedForm { dataset: SourceId, derivation: DerivationKind },
    /// Inflectional variant (number, gender, case, person, tense).
    InflectionalVariant { dataset: SourceId, differing_features: Vec<FeatureName> },
    /// Affix-only difference (prefix/suffix attachment).
    AffixVariant { added: Vec<String>, removed: Vec<String> },
    /// Algorithmically suggested; requires confidence; Layer D.
    ComputationalSuggestion { algorithm: String, version: SemVer, confidence: Confidence },
    /// A human reviewer explicitly affirmed this relation.
    ScholarVerified { reviewer: PrincipalId, note: Option<String>, reviewed_at: Timestamp },
}

pub enum DerivationKind {
    VerbalNoun, ActiveParticiple, PassiveParticiple, Nomen, Adjective,
    Diminutive, Elative, PlaceNoun, TimeNoun, InstrumentNoun, Other(String),
}
```

Additional axes required by §9.3 and surfaced as flags on each family member:
`singular_plural_relation`, `masculine_feminine_relation`, `verb_form` (I–X),
`active_passive_relation`.

**Hard rule:** the UI/CLI must group members **by relation class**, and every
`ComputationalSuggestion` renders with an explicit "computationally suggested — not verified
scholarship" label (§6.5, §13.4, §30). A `ComputationalSuggestion` becomes `ScholarVerified`
only through the Phase-0 `review_queue` with evidence displayed (§10.6).

### 7.2 `quran.word_family`

```jsonc
// input
{
  "input": "رحمة",                    // surface word, or {"root":"رحم"}
  Continuing `docs/plans/PHASE-2-QURAN-SEARCH-AND-LINGUISTICS.md` from §7.2.

````markdown
```jsonc
// input
{
  "input": "رحمة",                       // surface word, OR {"root":"رحم"}, OR {"reference":"1:1:3"}
  "resolve_via": "auto",                 // auto | surface | lemma | root | reference
  "dataset": null,                       // null = all active datasets
  "include_relations": [
    "same_exact_form","same_lemma","same_stem","same_root",
    "derived_form","inflectional_variant","affix_variant"
  ],
  "include_computational_suggestions": false,   // default FALSE (opt-in, §6.5)
  "min_confidence": 0.7,                 // applies to suggestions only
  "include_occurrences": true,
  "max_members": 500,
  "group_by": "relation"                 // relation | lemma | form | frequency
}
```

**Resolution algorithm (fully deterministic, no LLM):**

```text
input
  │
  ├─ if reference → look up token → analyses (per AnalysisPolicy)
  ├─ if root      → resolve root through RootConvention of each active dataset
  └─ if surface   → normalize with L5.codepoints → exact-form lookup
                     ├─ found → collect its analyses
                     └─ not found → try L7.affix → report "resolved via heuristic affix stripping"
                                    └─ still not found → typed error listing near-miss candidates
  │
  ▼
for each resolved (root_id, lemma_id, stem) per dataset:
  ├─ SameRoot        : SELECT tokens WHERE root_id = ?
  ├─ SameLemma       : SELECT tokens WHERE lemma_id = ?
  ├─ SameStem        : SELECT tokens WHERE stem = ?
  ├─ SameExactForm   : SELECT tokens WHERE surface = ?
  ├─ DerivedForm     : dataset-supplied derivation edges (Layer B)
  ├─ InflectionalVariant : same lemma, differing feature vector → diff features
  └─ AffixVariant    : same stem, differing prefix/suffix morphemes
  │
  ▼
group, dedupe by (surface, dataset-agnostic), attach counts + first/last occurrence
  │
  ▼
FamilyResult with per-member relation, dataset attribution, provenance, confidence
```

**Output shape**

```jsonc
{
  "query": { "input": "رحمة", "resolved_as": { "surface": "رَحْمَة", "reference": "2:64:12" } },
  "resolution": {
    "method": "surface_exact",
    "normalization_profile": "L5.codepoints@1.0.0",
    "used_heuristic": false
  },
  "roots": [
    { "letters": "رحم", "letters_spaced": "ر ح م",
      "dataset": "quranic-corpus@1.0.0", "token_count": 339, "lemma_count": 12 }
  ],
  "groups": [
    {
      "relation": "same_root",
      "relation_label": "Same root (dataset: Quranic Corpus 1.0.0)",
      "evidence_class": "dataset_supplied",
      "members": [
        { "surface": "الرَّحْمَٰن", "surface_bare": "الرحمن", "lemma": "رَحْمَٰن",
          "count": 57, "first": "quran:hafs-uthmani@1.0.0:1:1",
          "last": "quran:hafs-uthmani@1.0.0:78:38",
          "verb_form": null, "derivation": "adjective",
          "provenance_id": "…", "verification_status": "unverified" }
      ]
    },
    {
      "relation": "computational_suggestion",
      "relation_label": "Computationally suggested — not verified scholarship",
      "evidence_class": "computational",
      "display_warning": "These relations were generated algorithmically and have not been reviewed by a scholar.",
      "members": [ /* only present when include_computational_suggestions = true */ ]
    }
  ],
  "analysis_sources": [
    { "dataset": "quranic-corpus", "version": "1.0.0",
      "attribution_display": "Quranic Arabic Corpus (v1.0.0)",
      "analyses_returned": 4, "analyses_suppressed": 0 }
  ],
  "disagreements": [
    { "field": "root", "reference": "2:64:12",
      "values": [ { "dataset": "quranic-corpus", "value": "رحم" },
                  { "dataset": "camel-morph",    "value": "رحم" } ],
      "verdict": "identical" }
  ],
  "warnings": []
}
```

**Explainability requirement (§9.3):** every member carries an `explanation` string built from
the relation type + dataset + differing features, e.g.
*"Same lemma رَحِمَ (Quranic Corpus 1.0.0); differs in person (3rd → 1st) and number (sg → pl)."*
No member may be returned without one — enforced by the type having no default constructor.

---

## 8. Frequency, Distribution & Discovery Tools (PRD §11.3, §11.7)

All tools in this section are **deterministic, LLM-free, and reproducible**. Each returns an
explicit `CountingRules` block (I15) so a number can never be misread.

### 8.1 `CountingRules` (required in every numeric output)

```rust
pub struct CountingRules {
    pub unit: CountUnit,                  // Token | Form | Lemma | Root | Ayah | Surah | Character
    pub normalization_profile: Option<ProfileId>,
    pub dataset: Option<SourceId>,        // when counting lemmas/roots
    pub analysis_policy: AnalysisPolicy,  // how multi-analysis tokens were counted
    pub multi_analysis_handling: MultiAnalysisHandling,
    pub includes_pause_marks: bool,
    pub includes_basmala: BasmalaCounting, // Always | Never | PerEditionPolicy
    pub scope: CountScope,                // WholeEdition | Surahs(..) | Juz(..) | Range(..)
    pub edition: EditionRef,
    pub corpus_generation: u64,
    pub definition_note: String,          // human-readable exact definition
}

pub enum MultiAnalysisHandling {
    /// A token with 2 analyses whose roots differ contributes 1 to EACH root. Reported.
    CountOncePerAnalysis,
    /// Token counted once; ambiguous tokens listed separately.
    CountOncePreferredDataset { dataset: SourceId, ambiguous_tokens: u32 },
    /// Only tokens with a single unambiguous analysis are counted.
    UnambiguousOnly { excluded_tokens: u32 },
}
```

**This is the guard against numerology (§11.6).** Two "counts of the word X" that differ are
almost always different `CountingRules`; Q-ai makes that visible instead of arguing.

### 8.2 `quran.frequency`

```jsonc
{ "target": { "kind": "root", "value": "رحم" },     // form | lemma | root | phrase | pattern
  "profile": "L3.diacritics",
  "dataset": "quranic-corpus",
  "multi_analysis": "count_once_per_analysis",
  "scope": { "kind": "whole_edition" },
  "breakdown": ["surah"] }
```

Returns total, per-breakdown counts, `counting_rules`, and `ambiguous_tokens` detail.
Counts are **exact** (never estimated) and computed by SQL aggregation over the lexicon join,
not from FTS term frequencies (which can drift with tokenizer versions).

### 8.3 `quran.distribution`

Distribution by surah, revelation place (Makki/Madani — **Layer B/C metadata, attributed**), juz,
hizb, page, or a custom ayah-range partition. Returns absolute counts, normalized rates
(per 1,000 tokens), and the metadata provenance for the partition scheme.

Per §6.2/§9 the revelation-place classification is *not canonical*; the output carries
`partition_provenance` naming the dataset/scholar for that classification, and a warning when
sources disagree.

### 8.4 `quran.cooccurrence`

```jsonc
{ "a": { "kind": "root", "value": "رحم" },
  "b": { "kind": "root", "value": "غفر" },
  "window": { "unit": "token", "size": 10 },   // token | ayah | segment
  "ordered": false,
  "scope": {...} }
```

Windows are computed over `global_token_index` / `global_ayah_index` from Phase 1 — which is
exactly why those columns exist. Cross-ayah windows are allowed and flagged.

### 8.5 `quran.collocation`

Ranks statistically significant neighbours using **PMI, log-likelihood ratio, and t-score**,
reporting all three plus raw counts. The tool **never** claims significance without stating the
measure, the window, and the counting rules. A minimum-frequency floor (default 3) prevents
rank-1 artifacts, and the floor is reported.

### 8.6 `quran.first_last_occurrence`

Returns first, last, and the full ordered occurrence list in canonical order, each with pinned
references and `global_ayah_index` — enabling reproducible ordering across editions.

### 8.7 `quran.interval_analysis`

Distances between consecutive occurrences, measured in ayahs, tokens, or global indices, with
min/max/mean/median/stddev. Output includes a fixed disclaimer:

> *These are mechanical distance measurements under the stated counting rules. They do not
> constitute a claim of intentional numeric structure.*

### 8.8 `quran.numeric_report` (§11.6)

Produces a reproducible, exportable count report combining several targets under **one** stated
rule set, with a checksum. Hard requirements:

- Every number is accompanied by its `CountingRules`.
- The report includes a `reproducibility.checksum` (§12.1) that a third party can re-run.
- The report **must not** include any interpretive commentary. A `notes` field exists for the
  *user's* own annotation and is labeled as a user note (Layer E).
- If a user requests a target whose count depends on an ambiguous analysis, the report lists the
  ambiguity rather than picking a value.

### 8.9 Discovery tools (§11.7)

| Tool | Definition | Notes |
|---|---|---|
| `quran.unusual_usage` | Rare lemmas/roots/forms/constructions below a frequency threshold, or forms whose POS is atypical for their root | Threshold is an input, reported in output |
| `quran.hapax_search` | Forms or lemmas occurring exactly once **under the stated normalization profile and dataset** | Result count changes with profile — this is stated prominently |
| `quran.near_duplicate_passages` | Passage pairs with high lexical overlap | Uses shingled skeleton hashing (MinHash over 4-grams) + exact verification; returns Jaccard + aligned token spans |
| `quran.missing_expected_form` | Given a root/pattern, list attested vs. unattested forms | Output carries a mandatory disclaimer: *"Absence of a form in this edition is a lexical observation, not a theological conclusion"* (§11.7) |

`quran.near_duplicate_passages` is the only Phase-2 tool with a tunable similarity threshold; it
is labeled `ComputationalAnnotation` in spirit but remains `ReadOnly` because it writes nothing.
Its suggested pairs may be **promoted** to graph `PARALLELS` edges in Phase 3 — only through the
`review_queue`.

### 8.10 Deferred to Phase 3

`quran.semantic_field`, `quran.repetition_analysis`, `quran.parallel_structure`,
`quran.rhyme_analysis`, `quran.address_shift`, `quran.discourse_links`,
`quran.pronoun_reference`, `quran.contrastive_search`, `quran.parallel_verses`.
Rationale: each requires concept/entity nodes, rhetorical annotations, or graph traversal.
Phase 2 ships their *data prerequisites* (roots, lemmas, patterns, positions, skeletons).

---

## 9. Index Lifecycle & Consistency (PRD §76, §41, §50)

### 9.1 Index inventory

| Index id | Backend | Built from | Rebuild cost (target) |
|---|---|---|---|
| `quran.ayah.v1` | Tantivy | canonical ayahs + profiles | < 60 s |
| `quran.token.v1` | Tantivy | canonical tokens + profiles + lexicons | < 120 s |
| `quran.skeleton.v1` | trigram postings (`redb`/Tantivy keyword) | ayah + window skeletons | < 30 s |
| `quran.forms.v1` | SQLite tables | canonical tokens/ayahs × profiles | < 45 s |
| `quran.lexicon.v1` | SQLite tables | morphology dataset | < 90 s |

**Total cold rebuild target: < 6 minutes** for the reference edition on a 4-core laptop.
This is a CI-gated benchmark, because "rebuildable from source" (§41) is meaningless if it takes
hours.

### 9.2 Jobs

```text
quran.index.build      {index_id, generation}      idempotent on (index_id, generation, versions)
quran.index.rebuild    {index_id | "all", force}
quran.index.verify     {index_id}
quran.index.gc         {retain_generations}
quran.morphology.import {source_version_id}
quran.morphology.activate {dataset, version}       (human-approved)
quran.lexicon.rebuild  {dataset}
quran.forms.rebuild    {profiles[]}
```

All follow the Phase-0 job contract: checkpointed, cancellable, resumable, progress-reporting,
dead-lettering. Cancellation deletes the staging generation directory and never touches the
active pointer.

### 9.3 Generation stamping & drift detection (I14)

```sql
CREATE TABLE index_pointers (
  index_id            TEXT PRIMARY KEY,
  active_generation   INTEGER NOT NULL,
  previous_generation INTEGER,
  manifest_json       TEXT NOT NULL,
  manifest_hash       TEXT NOT NULL,
  activated_at        TEXT NOT NULL,
  activated_by        TEXT NOT NULL REFERENCES principals(id)
);
```

`qai doctor --indexes` compares, for each index, the manifest's
`(corpus_generation, edition_version, rule_set_versions, morphology_dataset_versions,
tokenizer_version)` against current live values, and reports precisely which input drifted —
reproducing the PRD §50 example line:

```text
✗ Morphology index differs from source version
    expected morphology dataset: quranic-corpus@1.1.0
    index built from:            quranic-corpus@1.0.0
Suggested action:
  qai quran morphology reindex
```

Drift is a **warning, never an auto-repair** (§50). Queries against a drifted index still work
but every `ToolResult` carries `warnings: [{code: "QAI-IDX-0101", message: "index is stale…"}]`
so a research answer can never silently rest on a stale index.

### 9.4 Reconciliation

A nightly `quran.index.verify` job checks:

- FTS `doc_count` equals the relational count for the active generation.
- A random 1 % sample of FTS docs round-trips to the correct canonical reference and text hash.
- No orphan generations on disk without an `index_pointers` row (and vice versa).
- Lexicon foreign keys all resolve; no dangling `root_id`/`lemma_id`.
- **MV-018 re-run**: canonical tables are byte-identical to their Phase-1 activation hashes.

---

## 10. API Additions (PRD §27.1)

```text
POST /api/v1/quran/search/exact
POST /api/v1/quran/search/normalized
POST /api/v1/quran/search/concatenated
POST /api/v1/quran/search/phrase
POST /api/v1/quran/search/regex
POST /api/v1/quran/search/root
POST /api/v1/quran/search/lemma
POST /api/v1/quran/word-family
POST /api/v1/quran/morphology
POST /api/v1/quran/morphology/compare
POST /api/v1/quran/pattern-search
POST /api/v1/quran/affix-search
POST /api/v1/quran/frequency
POST /api/v1/quran/distribution
POST /api/v1/quran/cooccurrence
POST /api/v1/quran/collocation
POST /api/v1/quran/occurrences
POST /api/v1/quran/interval-analysis
POST /api/v1/quran/numeric-report
POST /api/v1/quran/hapax
POST /api/v1/quran/unusual-usage
POST /api/v1/quran/near-duplicates
POST /api/v1/quran/missing-forms

GET  /api/v1/quran/roots?dataset=&prefix=&limit=        # root index browsing
GET  /api/v1/quran/roots/:letters
GET  /api/v1/quran/lemmas?dataset=&prefix=
GET  /api/v1/quran/normalization/profiles               # list profiles + rule lists
POST /api/v1/quran/normalization/preview                # show what a rule set does to input
GET  /api/v1/quran/morphology/datasets
GET  /api/v1/quran/indexes                              # manifests + drift status
```

`POST /api/v1/quran/normalization/preview` deserves emphasis: it renders the rule-by-rule
transformation of a user's input with the `SpanMap` at each step. It is the single best
debugging and trust-building endpoint in the phase, and it is what the Phase-4 UI will use to
explain "why did this match?".

Streaming: search endpoints support `Accept: text/event-stream` to stream hits as they are
found (§53), with a terminal event carrying totals and the reproducibility block.

---

## 11. CLI Additions (PRD §26, §25.4)

```bash
# Search
qai quran search "الرحمن"                              # defaults to L3.diacritics
qai quran search "ٱلرَّحْمَٰنِ" --exact
qai quran search "الرحمن" --profile L5.codepoints --explain
qai quran search "بسمالله" --concatenated [--cross-ayah]
qai quran search "الحمد لله" --phrase --slop 2 --unordered
qai quran search --regex "^ا?ل?رحم" --field text_bare
qai quran search "الرحمن" --surah 1,2,3 --json

# Linguistics
qai quran root "ر ح م" [--dataset quranic-corpus] [--group-by lemma]
qai quran root list --prefix ر --limit 50
qai quran lemma "رَحِمَ"
qai quran family "رحمة" [--include-suggestions] [--min-confidence 0.8]
qai quran morphology 1:1:1 [--policy all]
qai quran morphology compare 2:255:5 --datasets quranic-corpus,camel-morph
qai quran pattern "فَعَّلَ"
qai quran affix --prefix و --suffix هم

# Counting
qai quran freq --root رحم --breakdown surah
qai quran dist --root رحم --by revelation-place
qai quran cooc --root رحم --root غفر --window 10
qai quran colloc --lemma رَحِمَ --measure llr --min-count 3
qai quran occurrences --root رحم --first --last
qai quran intervals --root رحم
qai quran numeric-report --config report.toml --out report.json
qai quran hapax [--profile L3.diacritics]
qai quran unusual --threshold 2
qai quran near-duplicates --min-jaccard 0.8
qai quran missing-forms --root رحم

# Normalization introspection
qai quran normalize "بِسْمِ ٱللَّهِ" --profile L3.diacritics --explain
qai quran normalize --list-profiles
qai quran normalize --show-rule N06

# Morphology datasets
qai quran morphology dataset list
qai quran morphology import <manifest>
qai quran morphology validate <dataset>@<v>
qai quran morphology activate <dataset>@<v>
qai quran morphology coverage <dataset>@<v>
qai quran morphology diff <dataset> --from 1.0.0 --to 1.1.0

# Indexes
qai quran index list
qai quran index build <index-id>
qai quran index rebuild --all [--force]
qai quran index verify <index-id>
qai quran index gc --retain 1
qai doctor --indexes [--json]
```

`qai quran normalize … --explain` output (human mode) is deliberately a teaching tool:

```text
Input:  بِسْمِ ٱللَّهِ
Profile: L3.diacritics@1.0.0

  N01 whitespace_collapse   بِسْمِ ٱللَّهِ            (no change)
  N11 strip_zero_width      بِسْمِ ٱللَّهِ            (no change)
  N16 nfc                   بِسْمِ ٱللَّهِ            (no change)
  N04 strip_quranic_marks   بِسْمِ ٱللَّهِ            (no change)
  N14 strip_pause_marks     بِسْمِ ٱللَّهِ            (no change)
  N03 strip_harakat         بسم ٱلله                 (removed 6 marks)
  N05 strip_superscript_alef بسم ٱلله                (no change)
  N02 strip_tatweel         بسم ٱلله                 (no change)

Result: بسم ٱلله
Offset map: [0..3]→[0..6]  [4..8]→[7..14]
Heuristic rules used: none
```

---

## 12. `qai doctor` Additions (PRD §50)

```text
QURAN LINGUISTICS
  quran.normalization.profiles_loaded     All declared profiles resolve to known rules
  quran.normalization.idempotency         Idempotent rules verified idempotent
  quran.normalization.spanmap_sanity      Sampled offset round-trips succeed
  quran.forms.current                     Derived forms match corpus_generation + rule versions
  quran.forms.coverage                    Every token/ayah has every indexed form
  quran.canonical_unchanged               MV-018: canonical hashes match Phase-1 activation
  quran.fts.ayah_index                    Present, active generation, doc_count matches
  quran.fts.token_index                   Present, active generation, doc_count matches
  quran.skeleton.index                    Present, trigram postings consistent
  quran.index.drift                       Per-index input-version comparison
  quran.index.orphans                     No on-disk generations without pointers
  quran.morphology.dataset_active         At least one active dataset (or explicit none)
  quran.morphology.alignment              Unmatched-token ratio within threshold
  quran.morphology.coverage               Coverage ≥ declared threshold
  quran.morphology.provenance             Every analysis has Layer B/D provenance
  quran.morphology.unverified_layer_d     Count of Layer D rows awaiting review
  quran.lexicon.integrity                 No dangling root_id/lemma_id
  quran.lexicon.root_unification_queue    Pending cross-dataset root suggestions
  quran.search.smoke                      12 canned queries return expected known hits


markdown
`quran.search.smoke` is a genuinely valuable check: it runs a dozen fixed queries with known
expected references (e.g. `الرحمن` → must include 1:1, 1:3, 2:163; `بسمالله` concatenated → must
include 1:1; root `رحم` → must return the known total) and fails loudly if any regress. It is the
cheapest possible early-warning system for tokenizer or profile drift.

---

## 13. Database Schema (Phase 2 Migrations)

### `0020_quran_normalization.up.sql`

```sql
CREATE TABLE normalization_rules (
  rule_id       TEXT PRIMARY KEY,          -- 'N03'
  name          TEXT NOT NULL,
  version       TEXT NOT NULL,
  kind          TEXT NOT NULL CHECK (kind IN ('deterministic','heuristic')),
  description   TEXT NOT NULL,
  mapping_hash  TEXT NOT NULL,             -- hash of the code-point mapping table
  idempotent    INTEGER NOT NULL CHECK (idempotent IN (0,1))
);

CREATE TABLE normalization_profiles (
  profile_id    TEXT NOT NULL,             -- 'L3.diacritics'
  version       TEXT NOT NULL,
  label         TEXT NOT NULL,
  rule_list     TEXT NOT NULL,             -- ordered JSON array of rule_ids
  rule_list_hash TEXT NOT NULL,
  indexed       INTEGER NOT NULL CHECK (indexed IN (0,1)),
  contains_heuristic INTEGER NOT NULL CHECK (contains_heuristic IN (0,1)),
  created_at    TEXT NOT NULL,
  PRIMARY KEY (profile_id, version)
);

-- Profiles are append-only: a changed rule list requires a NEW version (§3.3).
CREATE TRIGGER trg_profile_immutable
BEFORE UPDATE OF rule_list, rule_list_hash ON normalization_profiles
BEGIN SELECT RAISE(ABORT, 'QAI-NORM-0001: profiles are immutable; publish a new version'); END;
```

### `0021_quran_forms.up.sql`

```sql
CREATE TABLE quran_token_forms (
  edition_id        TEXT    NOT NULL,
  surah             INTEGER NOT NULL,
  ayah              INTEGER NOT NULL,
  position          INTEGER NOT NULL,
  profile_id        TEXT    NOT NULL,
  profile_version   TEXT    NOT NULL,
  form              TEXT    NOT NULL,
  form_hash         TEXT    NOT NULL,
  span_map_json     TEXT    NOT NULL,      -- SpanMap back to canonical token offsets
  corpus_generation INTEGER NOT NULL,
  provenance_id     TEXT    NOT NULL REFERENCES provenance_records(id),
  PRIMARY KEY (edition_id, surah, ayah, position, profile_id, profile_version),
  FOREIGN KEY (edition_id, surah, ayah, position)
    REFERENCES quran_tokens(edition_id, surah, ayah, position),
  FOREIGN KEY (profile_id, profile_version)
    REFERENCES normalization_profiles(profile_id, version)
);

CREATE INDEX ix_tokform_lookup ON quran_token_forms(edition_id, profile_id, form);

CREATE TABLE quran_ayah_forms (
  edition_id        TEXT    NOT NULL,
  surah             INTEGER NOT NULL,
  ayah              INTEGER NOT NULL,
  profile_id        TEXT    NOT NULL,
  profile_version   TEXT    NOT NULL,
  form              TEXT    NOT NULL,
  form_hash         TEXT    NOT NULL,
  span_map_json     TEXT    NOT NULL,
  corpus_generation INTEGER NOT NULL,
  provenance_id     TEXT    NOT NULL REFERENCES provenance_records(id),
  PRIMARY KEY (edition_id, surah, ayah, profile_id, profile_version)
);

CREATE TABLE quran_skeletons (
  edition_id        TEXT    NOT NULL,
  level             TEXT    NOT NULL CHECK (level IN ('ayah','window')),
  key_start_global  INTEGER NOT NULL,      -- global_ayah_index of first ayah
  key_end_global    INTEGER NOT NULL,
  skeleton          TEXT    NOT NULL,
  skeleton_hash     TEXT    NOT NULL,
  span_map_json     TEXT    NOT NULL,
  corpus_generation INTEGER NOT NULL,
  PRIMARY KEY (edition_id, level, key_start_global, key_end_global)
);

CREATE INDEX ix_skeleton_lookup ON quran_skeletons(edition_id, level, skeleton);
```

### `0022_quran_lexicon.up.sql`

```sql
CREATE TABLE morphology_datasets (
  id                    TEXT PRIMARY KEY,
  slug                  TEXT NOT NULL,
  version               TEXT NOT NULL,
  name                  TEXT NOT NULL,
  source_version_id     TEXT NOT NULL REFERENCES source_versions(id),
  aligned_edition_id    TEXT NOT NULL REFERENCES quran_editions(id),
  alignment_method      TEXT NOT NULL CHECK (alignment_method IN ('direct_key','alignment_table')),
  alignment_source_ver  TEXT REFERENCES source_versions(id),
  unmatched_token_count INTEGER NOT NULL DEFAULT 0,
  root_convention       TEXT NOT NULL,
  provides_multiple     INTEGER NOT NULL CHECK (provides_multiple IN (0,1)),
  attribution_display   TEXT NOT NULL CHECK (length(trim(attribution_display)) > 0),
  license_json          TEXT NOT NULL,
  trust_level           TEXT NOT NULL,
  coverage_json         TEXT NOT NULL,
  status                TEXT NOT NULL CHECK (status IN
                          ('Staged','Approved','Active','Deprecated','Quarantined')),
  imported_at           TEXT NOT NULL,
  activated_at          TEXT,
  approval_id           TEXT REFERENCES approvals(id),
  UNIQUE (slug, version),
  CHECK (status NOT IN ('Approved','Active') OR approval_id IS NOT NULL)
);

CREATE TABLE quran_roots (
  id                TEXT PRIMARY KEY,
  dataset_id        TEXT NOT NULL REFERENCES morphology_datasets(id) ON DELETE RESTRICT,
  letters           TEXT NOT NULL,
  letters_spaced    TEXT NOT NULL,
  letter_count      INTEGER NOT NULL,
  token_count       INTEGER NOT NULL DEFAULT 0,
  lemma_count       INTEGER NOT NULL DEFAULT 0,
  canonical_root_id TEXT REFERENCES quran_roots(id),   -- cross-dataset link (Layer D)
  provenance_id     TEXT NOT NULL REFERENCES provenance_records(id),
  UNIQUE (dataset_id, letters)
);

CREATE INDEX ix_roots_letters ON quran_roots(letters);

CREATE TABLE quran_lemmas (
  id            TEXT PRIMARY KEY,
  dataset_id    TEXT NOT NULL REFERENCES morphology_datasets(id) ON DELETE RESTRICT,
  form          TEXT NOT NULL,
  form_bare     TEXT NOT NULL,
  root_id       TEXT REFERENCES quran_roots(id),
  pos           TEXT,
  gloss         TEXT,
  token_count   INTEGER NOT NULL DEFAULT 0,
  provenance_id TEXT NOT NULL REFERENCES provenance_records(id),
  UNIQUE (dataset_id, form, pos)
);

CREATE INDEX ix_lemmas_bare ON quran_lemmas(form_bare);

CREATE TABLE quran_token_analyses (
  id                  TEXT PRIMARY KEY,
  edition_id          TEXT    NOT NULL,
  surah               INTEGER NOT NULL,
  ayah                INTEGER NOT NULL,
  position            INTEGER NOT NULL,
  dataset_id          TEXT    NOT NULL REFERENCES morphology_datasets(id) ON DELETE RESTRICT,
  analysis_index      INTEGER NOT NULL,
  lemma_id            TEXT REFERENCES quran_lemmas(id),
  root_id             TEXT REFERENCES quran_roots(id),
  stem                TEXT,
  pattern             TEXT,
  verb_form           TEXT,
  features_json       TEXT NOT NULL,
  dependency_json     TEXT,
  confidence          REAL CHECK (confidence IS NULL OR (confidence BETWEEN 0.0 AND 1.0)),
  verification_status TEXT NOT NULL,
  provenance_id       TEXT NOT NULL REFERENCES provenance_records(id),
  FOREIGN KEY (edition_id, surah, ayah, position)
    REFERENCES quran_tokens(edition_id, surah, ayah, position),
  UNIQUE (edition_id, surah, ayah, position, dataset_id, analysis_index)
);

CREATE INDEX ix_analyses_root  ON quran_token_analyses(root_id);
CREATE INDEX ix_analyses_lemma ON quran_token_analyses(lemma_id);
CREATE INDEX ix_analyses_token ON quran_token_analyses(edition_id, surah, ayah, position);

CREATE TABLE quran_morphemes (
  analysis_id     TEXT    NOT NULL REFERENCES quran_token_analyses(id) ON DELETE CASCADE,
  morpheme_index  INTEGER NOT NULL,
  kind            TEXT    NOT NULL CHECK (kind IN ('prefix','stem','suffix')),
  surface         TEXT    NOT NULL,
  tag             TEXT    NOT NULL,
  normalized_tag  TEXT,
  char_start      INTEGER,
  char_end        INTEGER,
  PRIMARY KEY (analysis_id, morpheme_index)
);

-- Dataset-supplied derivation edges (Layer B); Phase 3 promotes these to graph edges.
CREATE TABLE quran_derivations (
  id             TEXT PRIMARY KEY,
  dataset_id     TEXT NOT NULL REFERENCES morphology_datasets(id),
  from_lemma_id  TEXT NOT NULL REFERENCES quran_lemmas(id),
  to_lemma_id    TEXT NOT NULL REFERENCES quran_lemmas(id),
  derivation     TEXT NOT NULL,
  confidence     REAL,
  provenance_id  TEXT NOT NULL REFERENCES provenance_records(id),
  UNIQUE (dataset_id, from_lemma_id, to_lemma_id, derivation)
);

-- Human-verified family relations (Layer C), created via review_queue (§10.6).
CREATE TABLE word_family_relations (
  id              TEXT PRIMARY KEY,
  relation        TEXT NOT NULL,
  left_urn        TEXT NOT NULL,
  right_urn       TEXT NOT NULL,
  dataset_id      TEXT REFERENCES morphology_datasets(id),
  evidence_json   TEXT NOT NULL,
  confidence      REAL,
  provenance_id   TEXT NOT NULL REFERENCES provenance_records(id),
  UNIQUE (relation, left_urn, right_urn, dataset_id)
);
```

### `0023_quran_indexes.up.sql`

```sql
CREATE TABLE index_pointers (
  index_id            TEXT PRIMARY KEY,
  active_generation   INTEGER NOT NULL,
  previous_generation INTEGER,
  manifest_json       TEXT NOT NULL,
  manifest_hash       TEXT NOT NULL,
  doc_count           INTEGER NOT NULL,
  activated_at        TEXT NOT NULL,
  activated_by        TEXT NOT NULL REFERENCES principals(id)
);

CREATE TABLE index_build_runs (
  id            TEXT PRIMARY KEY,
  index_id      TEXT NOT NULL,
  generation    INTEGER NOT NULL,
  job_id        TEXT REFERENCES jobs(id),
  state         TEXT NOT NULL CHECK (state IN
                  ('building','verifying','verified','activated','failed','cancelled')),
  manifest_json TEXT,
  doc_count     INTEGER,
  duration_ms   INTEGER,
  error_json    TEXT,
  started_at    TEXT NOT NULL,
  finished_at   TEXT,
  UNIQUE (index_id, generation)
);
```

### `0024_quran_morphology_staging.up.sql`

`morph_stg_*` mirrors of `quran_roots`, `quran_lemmas`, `quran_token_analyses`,
`quran_morphemes`, `quran_derivations`, each with `import_run_id`, no immutability triggers, and
`ON DELETE CASCADE` from `morphology_import_runs` — identical pattern to Phase-1 staging.

### `0025_quran_search_cache.up.sql`

```sql
CREATE TABLE search_result_cache (
  cache_key         TEXT PRIMARY KEY,   -- H(tool, input, profiles, generation, dataset versions)
  tool_name         TEXT NOT NULL,
  corpus_generation INTEGER NOT NULL,
  result_json       TEXT NOT NULL,
  hit_count         INTEGER NOT NULL DEFAULT 0,
  created_at        TEXT NOT NULL,
  last_used_at      TEXT NOT NULL,
  bytes             INTEGER NOT NULL
);

CREATE INDEX ix_cache_gen ON search_result_cache(corpus_generation);
CREATE INDEX ix_cache_lru ON search_result_cache(last_used_at);
```

Cache is invalidated wholesale on `corpus_generation` or any index-manifest change, and is
size-capped (default 128 MiB) with LRU eviction.

---

## 14. ADRs Required In Phase 2

| ADR | Title | Blocking | Key trade-off |
|---|---|---|---|
| ADR-0201 | Full-text engine: Tantivy + custom Arabic tokenizers | §4 | Rust-native, offline, positional phrase support vs. writing tokenizers ourselves |
| ADR-0202 | *(reserved for Phase 3 graph store — not written here)* | — | — |
| ADR-0203 | Initial Quran morphology dataset, license, alignment strategy, attribution string | §6 | Coverage/quality vs. redistribution rights; bundled vs. user-supplied |
| ADR-0204 | Arabic normalization rule catalog, mapping tables, and rule ordering | §3.2 | Recall vs. precision; each fold loses a distinction permanently |
| ADR-0205 | Normalization profile ladder, versioning, and immutability policy | §3.3 | Simplicity vs. user control; profile explosion risk |
| ADR-0206 | Transliteration standard (reserved; decision recorded, implementation Phase 4) | §3.2 (N23) | ALA-LC vs. DIN vs. Buckwalter vs. IJMES |
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

---

## 15. Work Breakdown Structure

Total ≈ **112 ed** ⇒ ~8 weeks with 3 engineers + 0.4 FTE Arabic linguist.
Roles: **BE** backend, **SRCH** search/index specialist, **LING** Arabic linguist,
**DATA** data engineering, **QA**, **DOC**.

### Sprint 2.0 — Dataset & Linguistic Decisions (parallel with Phase 1 Sprint 1.5)

| ID | Task | Deliv. | Dep | Est | Role |
|---|---|---|---|---|---|
| P2-T01 | Survey morphology datasets: coverage, depth, tokenization, license | ADR-0203 | — | 3.0 | DATA |
| P2-T02 | Legal review of morphology dataset redistribution | ADR-0203 | T01 | 1.5 | DOC |
| P2-T03 | Define alignment strategy + write alignment spec | ADR-0203 | T01 | 2.0 | DATA |
| P2-T04 | Author normalization rule catalog with full code-point mapping tables | ADR-0204 | — | 4.0 | LING |
| P2-T05 | Review each rule for linguistic correctness + document losses | ADR-0204 | T04 | 2.5 | LING |
| P2-T06 | Define profile ladder L0–L8 and versioning policy | ADR-0205 | T04 | 1.5 | LING+BE |
| P2-T07 | Define unified tagset + dataset tag mapping | ADR-0215 | T01 | 2.5 | LING |
| P2-T08 | Define root convention + unification policy | ADR-0210 | T01 | 1.5 | LING |
| P2-T09 | Define counting-rule semantics + numeric-report policy | ADR-0211 | T04 | 2.0 | LING+BE |
| P2-T10 | Write ADR-0203/0204/0205/0210/0211/0215; record ADR-0206 as reserved | ADR | T02–T09 | 3.0 | DOC |
| P2-T11 | Build normalization golden set (2,000 input→output pairs per profile) | §16 | T05 | 4.0 | LING+QA |
| P2-T12 | Build root/lemma golden set (500 curated cases) | §16 | T08 | 3.0 | LING+QA |

### Sprint 2.1 — Normalization Engine (Week 1–2)

| ID | Task | Deliv. | Dep | Est | Role |
|---|---|---|---|---|---|
| P2-T13 | `quran-normalization` crate skeleton, `RuleId`, `NormalizationRule` trait | §3.2 | P1 done | 1.5 | BE |
| P2-T14 | `SpanMap`: segments, compose, to_canonical, to_derived | §3.4 | T13 | 3.5 | BE |
| P2-T15 | `SpanMap` property tests (5 properties × all ayahs × all profiles) | §3.4 | T14 | 3.0 | QA |
| P2-T16 | Implement deterministic rules N01–N17 | §3.2 | T14 | 5.0 | BE |
| P2-T17 | Implement heuristic rules N18–N22 with `RuleKind::Heuristic` tagging | §3.2 | T16 | 2.5 | BE |
| P2-T18 | `NormalizationPipeline` + profile registry + immutability enforcement | §3.3 | T16 | 2.5 | BE |
| P2-T19 | Migration `0020_quran_normalization` + profile/rule seeding | §13 | T18 | 1.5 | BE |
| P2-T20 | `NormalizationTrace` type + no-default-constructor guard | I9 | T18 | 1.0 | BE |
| P2-T21 | Golden-set test harness; all 2,000 pairs green | §16 | T11,T18 | 2.5 | QA |
| P2-T22 | Idempotency + associativity + fuzz (no panic on any Unicode input) | §16 | T18 | 2.0 | QA |
| P2-T23 | `qai quran normalize --explain` + `--list-profiles` + `--show-rule` | §11 | T18 | 2.0 | BE |
| P2-T24 | `POST /normalization/preview` + `GET /normalization/profiles` | §10 | T23 | 1.5 | BE |

### Sprint 2.2 — Derived Forms & FTS Foundation (Week 2–3)

| ID | Task | Deliv. | Dep | Est | Role |
|---|---|---|---|---|---|
| P2-T25 | Migration `0021_quran_forms` | §13 | T19 | 1.5 | BE |
| P2-T26 | `quran.forms.rebuild` job: token + ayah forms, all indexed profiles | §9.2 | T25,T18 | 3.0 | BE |
| P2-T27 | Skeleton builder (ayah + 3-ayah windows) + span maps | §4.4 | T26 | 2.5 | BE |
| P2-T28 | MV-018 canonical-unchanged verifier wired into every build job | I8 | T26 | 1.5 | BE |
| P2-T29 | `FullTextIndex` trait + `IndexManifest` + `FtsQuery`/`SearchOpts` types | §4.1 | T13 | 2.5 | SRCH |
| P2-T30 | Tantivy backend: schema, writer, reader, commit stamps | §4.2 | T29 | 3.5 | SRCH |
| P2-T31 | Custom `ar_*` tokenizers wired to `NormalizationPipeline` (shared query/index path) | §4.2 | T30,T18 | 3.0 | SRCH |
| P2-T32 | Query/index tokenizer-parity test (5,000 random substrings) | §16 | T31 | 1.5 | QA |
| P2-T33 | Migration `0023_quran_indexes` + `index_pointers` + build-run tracking | §13 | T25 | 1.5 | BE |
| P2-T34 | `quran.index.build` job: staging dir → verify → atomic pointer flip | §9 | T30,T33 | 3.0 | SRCH |
| P2-T35 | Index generation retention, `gc`, single-step rollback | §9.3 | T34 | 1.5 | SRCH |
| P2-T36 | Trigram skeleton posting index + build job | §4.4 | T27 | 3.0 | SRCH |
| P2-T37 | Index build crash/cancel matrix (kill at each stage; active pointer unchanged) | §16 | T34 | 2.0 | QA |
| P2-T38 | Cold-rebuild benchmark + CI threshold gate (< 6 min total) | §9.1 | T34,T36 | 1.5 | QA |
| P2-T39 | ADR-0201/0208/0213 | ADR | T31,T34 | 2.0 | DOC |

### Sprint 2.3 — Search Tools (Week 3–4)

| ID | Task | Deliv. | Dep | Est | Role |
|---|---|---|---|---|---|
| P2-T40 | `SearchHit`, `ScoreExplain`, unified result assembly + canonical-span attach | §5.6 | T14,T30 | 2.5 | BE |
| P2-T41 | `quran.search_exact` (+ zero-result normalization hint) | §5.1 | T40 | 2.0 | SRCH |
| P2-T42 | `quran.search_normalized` incl. ad-hoc rule sets + `explain` | §5.2 | T41 | 3.0 | SRCH |
| P2-T43 | `quran.search_phrase` (ordered/near/unordered, slop) | §5.4 | T41 | 2.5 | SRCH |
| P2-T44 | `quran.search_concatenated`: candidate gen → verify → segmentation explanation | §5.3 | T36,T40 | 4.5 | SRCH |
| P2-T45 | Cross-ayah window dedup + `spans_ayah_boundary` labeling | §4.4 | T44 | 2.0 | SRCH |
| P2-T46 | `quran.search_regex` with DFA engine + all I16 guards + rate limit | §5.5 | T41 | 3.0 | SRCH |
| P2-T47 | Exact `total_matches` counting path (separate from ranked search) | §5.6 | T41 | 1.5 | SRCH |
| P2-T48 | Filters: surah/juz/page/revelation-place/global-range | §4.3 | T41 | 2.0 | BE |
| P2-T49 | Highlighting: canonical char ranges → display markers | I10 | T40 | 2.0 | BE |
| P2-T50 | Result cache (`0025`) + generation invalidation + LRU cap | §13 | T40 | 2.0 | BE |
| P2-T51 | Search API endpoints + SSE streaming variant | §10 | T41–T46 | 3.0 | BE |
| P2-T52 | CLI search command group with all flags + `--json` | §11 | T41–T46 | 2.5 | BE |
| P2-T53 | Search golden-set suite (400 queries × expected reference sets) | §16 | T44,T46 | 4.0 | QA |
| P2-T54 | Regex/DoS abuse suite (pathological patterns, timeout, limits) | §16 | T46 | 2.0 | QA |
| P2-T55 | Search latency benchmarks + CI gates (table §17.1) | §17 | T44 | 2.0 | QA |
| P2-T56 | ADR-0207/0212/0214 | ADR | T44,T46,T50 | 1.5 | DOC |

### Sprint 2.4 — Morphology Import & Lexicons (Week 5–6)

| ID | Task | Deliv. | Dep | Est | Role |
|---|---|---|---|---|---|
| P2-T57 | Migrations `0022_quran_lexicon`, `0024_morphology_staging` | §13 | T25 | 2.0 | BE |
| P2-T58 | Intermediate morphology format + JSON Schema + serde types | §6.3 | T57 | 2.0 | DATA |
| P2-T59 | Adapter for the chosen dataset (per ADR-0203) | §6.3 | T58 | 4.0 | DATA |
| P2-T60 | Second adapter (different shape) proving extensibility | §6.3 | T59 | 2.0 | DATA |
| P2-T61 | Alignment engine: `DirectKey` + `AlignmentTable` + unmatched reporting | §6.3 | T59 | 4.0 | DATA |
| P2-T62 | Unified tagset mapper (native tags preserved verbatim) | §6.1 | T58,T07 | 2.5 | BE |
| P2-T63 | Validation rules MV-001…MV-018 | §6.4 | T61,T62 | 4.0 | BE |
| P2-T64 | Lexicon builder: roots, lemmas, stems, counts | §6.2 | T63 | 3.0 | BE |
| P2-T65 | Cross-dataset root unification as `review_queue` suggestions (never merge) | §6.2 | T64 | 2.5 | BE |
| P2-T66 | `quran.morphology.import` job (12 checkpoints, cancel, resume) | §6.3 | T63,T64 | 3.5 | BE |
| P2-T67 | `quran.morphology.activate` (approval + pointer flip + enqueue FTS rebuild) | §6.3 | T66 | 2.0 | BE |
| P2-T68 | Coverage + unmatched-token reports; approval threshold gate | §6.3 | T61 | 2.0 | BE |
| P2-T69 | Dataset version differ (`morphology diff`) | §11 | T66 | 2.0 | BE |
| P2-T70 | Layer B/D provenance writing for every analysis/root/lemma row | I12 | T63 | 2.0 | BE |
| P2-T71 | Adversarial morphology fixtures (18 faults → correct MV rule ids) | §16 | T63 | 3.0 | QA |
| P2-T72 | Import crash/cancel matrix (12 checkpoints) | §16 | T66 | 2.0 | QA |
| P2-T73 | Populate FTS lexicon fields (roots/lemmas/stems/pos/patterns) | §4.2 | T67 | 2.0 | SRCH |
| P2-T74 | ADR-0209 | ADR | T63 | 0.5 | DOC |

### Sprint 2.5 — Morphology & Family Tools (Week 6–7)

| ID | Task | Deliv. | Dep | Est | Role |
|---|---|---|---|---|---|
| P2-T75 | `AnalysisPolicy` + `analysis_sources` + suppression reporting | §6.1 | T64 | 2.0 | BE |
| P2-T76 | `quran.morphology` tool | §6.5 | T75 | 2.5 | BE |
| P2-T77 | `quran.morphology_compare` with agreement verdicts, no resolution field | §6.5 | T76 | 2.5 | BE |
| P2-T78 | `quran.root_search` (convention resolution, grouping, occurrences) | §6.5 | T64 | 3.0 | BE |
| P2-T79 | `quran.lemma_search` | §6.5 | T64 | 1.5 | BE |
| P2-T80 | `quran.pattern_search` + capability-unavailable error path | §6.5 | T64 | 2.0 | BE |
| P2-T81 | `quran.affix_search` (dataset backend + `L7` heuristic backend, labeled) | §6.5 | T17,T64 | 2.5 | BE |
| P2-T82 | Root/lemma browse endpoints + CLI `root list` | §10,§11 | T78 | 1.5 | BE |
| P2-T83 | `FamilyRelation` taxonomy + `word_family_relations` table wiring | §7.1 | T57 | 2.0 | BE |
| P2-T84 | Family resolution algorithm (all 5 input paths) | §7.2 | T78,T83 | 3.5 | BE |
| P2-T85 | Relation builders: same-form/lemma/stem/root, derived, inflectional, affix | §7.2 | T84 | 3.5 | BE |
| P2-T86 | Per-member `explanation` generator (differing-feature diffing) | §7.2 | T85 | 2.5 | BE |
| P2-T87 | Computational-suggestion path: opt-in, confidence floor, mandatory labels | §7.1 | T85 | 2.0 | BE |
| P2-T88 | `review_queue` promotion flow: suggestion → `ScholarVerified` with evidence | §7.1 | T87,T65 | 2.5 | BE |
| P2-T89 | Morphology/family API endpoints | §10 | T76–T87 | 2.5 | BE |
| P2-T90 | CLI morphology/root/lemma/family/pattern/affix commands | §11 | T76–T87 | 3.0 | BE |
| P2-T91 | Root/lemma golden-set suite (500 cases) | §16 | T12,T78 | 3.0 | QA |
| P2-T92 | Family-relation golden suite (120 curated families, reviewed by LING) | §16 | T85 | 3.5 | LING+QA |
| P2-T93 | Multi-analysis non-merge tests (no authoritative flag; suppression visible) | §16 | T77 | 2.0 | QA |

### Sprint 2.6 — Counting, Discovery, Doctor, Evaluation (Week 7–8)

| ID | Task | Deliv. | Dep | Est | Role |
|---|---|---|---|---|---|
| P2-T94 | `CountingRules` type + serialization + mandatory-field enforcement | §8.1 | T75 | 2.0 | BE |
| P2-T95 | `quran.frequency` (exact SQL aggregation, all multi-analysis modes) | §8.2 | T94 | 2.5 | BE |
| P2-T96 | `quran.distribution` + partition provenance + disagreement warnings | §8.3 | T95 | 2.5 | BE |
| P2-T97 | `quran.cooccurrence` (token/ayah/segment windows, cross-ayah flags) | §8.4 | T95 | 2.5 | BE |
| P2-T98 | `quran.collocation` (PMI + LLR + t-score, min-count floor) | §8.5 | T97 | 2.5 | BE |
| P2-T99 | `quran.first_last_occurrence`, `quran.interval_analysis` + disclaimer | §8.6–8.7 | T95 | 2.0 | BE |
| P2-T100 | `quran.numeric_report` + checksum + no-interpretation policy | §8.8 | T94 | 2.5 | BE |
| P2-T101 | `quran.hapax_search`, `quran.unusual_usage` | §8.9 | T95 | 2.0 | BE |
| P2-T102 | `quran.near_duplicate_passages` (MinHash + exact verify + aligned spans) | §8.9 | T27 | 3.0 | SRCH |
| P2-T103 | `quran.missing_expected_form` + mandatory disclaimer | §8.9 | T85 | 2.0 | BE |
| P2-T104 | Counting/discovery API endpoints + CLI commands | §10,§11 | T95–T103 | 3.0 | BE |
| P2-T105 | `doctor` Phase-2 checks (19 checks) incl. `quran.search.smoke` | §12 | T34,T67 | 3.5 | BE |
| P2-T106 | Index-drift reporting with precise input diff + `QAI-IDX-0101` warnings | §9.3 | T105 | 2.0 | BE |
| P2-T107 | Nightly reconciliation job (`quran.index.verify`, 1 % sample, MV-018) | §9.4 | T105 | 2.5 | BE |
| P2-T108 | Evaluation harness: metric definitions, versioned datasets, gates | §17.2 | T53,T91 | 3.5 | QA |
| P2-T109 | Counting-rules determinism tests (same rules ⇒ same number, always) | §16 | T95 | 1.5 | QA |
| P2-T110 | Tool-contract conformance for all 22 Phase-2 tools | §16 | T104 | 2.5 | QA |
| P2-T111 | Full soak: rebuild all indexes → 50k randomized queries → doctor → reconcile | §16 | all | 2.5 | QA |
| P2-T112 | ADR-0216 + ADR index update | ADR | T18 | 0.5 | DOC |
| P2-T113 | Docs: normalization spec, profile catalog, search cookbook, morphology adapter guide, counting-rules explainer, reindex runbook | §18 | all | 4.0 | DOC |
| P2-T114 | Phase-2 exit-gate review + handoff to Phase 3 | — | all | 2.0 | all |

---

## 16. Testing Strategy

### 16.1 Golden sets (versioned, in `fixtures/quran/`)

| Set | Size | Content |
|---|---|---|
| `normalization/pairs.jsonl` | 2,000 | `{input, profile, expected_output, expected_rules_applied}` |
| `normalization/spanmaps.jsonl` | 300 | `{ayah_ref, profile, derived_range, expected_canonical_range}` |
| `search/queries.jsonl` | 400 | `{tool, input, expected_references[], expected_total, must_not_contain[]}` |
| `search/concatenated.jsonl` | 120 | Space-free queries incl. cross-ayah and Persian-codepoint cases |
| `search/regex.jsonl` | 60 | Patterns incl. 15 pathological ones expected to be rejected/bounded |
| `lexicon/roots.jsonl` | 300 | `{root, dataset, expected_token_count, expected_lemmas[]}` |
| `lexicon/lemmas.jsonl` | 200 | `{lemma, expected_forms[], expected_count}` |
| `morphology/analyses.jsonl` | 400 | `{reference, dataset, expected_segments[], expected_features}` |
| `families/curated.jsonl` | 120 | Linguist-reviewed families with expected relation classes |
| `counting/reports.jsonl` | 80 | `{target, counting_rules, expected_count}` — the anti-numerology set |
| `adversarial/morphology/*` | 18 | Faulty datasets → expected MV rule id |

**Governance:** golden sets are reviewed and signed off by the Arabic linguist (P2-T11, T12, T92)
and stored with a `reviewed_by` + `reviewed_at` header. Changing an expected value requires a
linguist review recorded in the PR — the same discipline as canonical text changes.

### 16.2 Property tests

| Property | Scope |
|---|---|
| `SpanMap` round-trip containment | every ayah × every profile |
| `SpanMap` composition associativity | random rule chains |
| Rule idempotency (for rules declaring it) | random Unicode strings |
| No rule panics on arbitrary Unicode | fuzz, 10⁶ inputs |
| Normalized-search recall ⊇ exact-search results | every golden query |
| Profile monotonicity: results(Lₙ) ⊆ results(Lₙ₊₁) for the same query | all indexed profiles |
| Concatenated search finds every exact-search hit for space-free queries | golden set |
| Frequency counts equal `COUNT(*)` over the occurrence list from the same rules | all targets |
| Family membership is symmetric for symmetric relations | all families |
| Index doc_count equals relational count | after every build |
| Canonical text hashes unchanged after every build/import | MV-018, every job |

### 16.3 Integrity & safety suites

| Suite | Proves |
|---|---|
| `tests/integrity/canonical_untouched.rs` | Building every index and importing every dataset leaves Phase-1 hashes identical (I8) |
| `tests/integrity/no_authoritative_analysis.rs` | Schema has no `is_correct`/`is_primary`; suppression is always reported (I11) |
| `tests/integrity/layer_d_labeling.rs` | Every computational row has algorithm+version+confidence and cannot be `human_verified` without a reviewer (I12) |
| `tests/integrity/no_llm_dependency.rs` | `quran-normalization`, `quran-search`, `quran-morphology` do not depend on `llm`/`embeddings` |
| `tests/integrity/trace_required.rs` | No `SearchHit` can be constructed without a `NormalizationTrace` (I9) |
| `tests/security/regex_dos.rs` | 15 pathological patterns bounded within limits (I16) |
| `tests/security/rate_limits.rs` | Regex and heavy-scan tools rate-limited per principal |
| `tests/recovery/index_build.rs` | Crash/cancel at each build stage never activates a partial index |
| `tests/recovery/morphology_import.rs` | Crash/cancel at each of 12 checkpoints; resume correctness |
| `tests/consistency/drift.rs` | Bumping any input version produces the exact expected doctor drift report |
| `tests/consistency/cache.rs` | No stale cached result served across a generation bump |

### 16.4 Coverage gates

`quran-normalization` **≥ 92 %** (it is the highest-risk pure-logic crate);
`quran-search` **≥ 85 %**; `quran-morphology` **≥ 85 %**; tools layer **≥ 80 %**.

---

## 17. Performance & Evaluation Targets

### 17.1 Latency (reference: 4-core laptop, cold OS cache, warm index)

| Operation | p50 | p99 | Cap |
|---|---|---|---|
| `search_exact` (single token) | < 5 ms | < 25 ms | 2 s timeout |
| `search_normalized` (indexed profile) | < 8 ms | < 40 ms | 2 s |
| `search_normalized` (ad-hoc rules) | < 60 ms | < 250 ms | 5 s |
| `search_phrase` (3 tokens, slop 0) | < 12 ms | < 60 ms | 2 s |
| `search_concatenated` (3–20 chars) | < 35 ms | < 150 ms | 3 s |
| `search_regex` (anchored) | < 80 ms | < 500 ms | 3 s hard |
| `root_search` (frequent root, 339 hits) | < 20 ms | < 90 ms | 2 s |
| `word_family` (large root, no suggestions) | < 60 ms | < 250 ms | 5 s |
| `morphology` (single token, all datasets) | < 6 ms | < 30 ms | 2 s |
| `frequency` (root, breakdown by surah) | < 40 ms | < 180 ms | 5 s |
| `collocation` (lemma, window 10) | < 250 ms | < 1.2 s | 10 s |
| `near_duplicate_passages` (full corpus) | < 2.5 s | < 8 s | 30 s |
| `normalize --explain` (single string) | < 1 ms | < 5 ms | — |
| Full index rebuild (all indexes) | < 4 min | < 6 min | — |

All are CI-gated benchmarks with a 20 % regression tolerance; exceeding it fails the build.

### 17.2 Accuracy gates (PRD §36.1, §47) — versioned evaluation datasets

| Metric | Target | Gate |
|---|---|---|
| Exact search precision / recall | 1.00 / 1.00 | **Hard** — any failure blocks release |
| Diacritic-insensitive search recall (`L3`) | ≥ 0.99 | Hard |
| Diacritic-insensitive search precision | ≥ 0.95 | Soft (warn) |
| Concatenated phrase search accuracy | ≥ 0.97 | Hard |
| Cross-ayah concatenated recall | ≥ 0.90 | Soft |
| Root-search precision (vs. dataset ground truth) | 1.00 | Hard |
| Root-search recall | ≥ 0.99 | Hard |
| Lemma-search accuracy | ≥ 0.99 | Hard |
| Morphological segmentation accuracy (vs. dataset) | ≥ 0.995 | Hard (this is a faithfulness-of-import metric, not an NLP metric) |
| Word-family explanation correctness (linguist-rated) | ≥ 0.95 | Hard on the 120-family curated set |
| Offset-mapping fidelity | 1.00 | Hard |
| Frequency-count reproducibility | 1.00 | Hard |
| Canonical-text integrity after all builds | 1.00 | Hard, zero tolerance |
| Citation resolution for search hits | 1.00 | Hard |

Evaluation runs are stored with `dataset_version`, `code_version`, `index_manifest_hash`, and
results are diffed against the previous run in CI; a regression on any hard gate blocks merge.

---

## 18. Acceptance Criteria (Phase 2 Exit Gate)

| ID | Criterion | Verification |
|---|---|---|
| AC-P2-01 | ADR-0203 accepted; a licensed morphology dataset is active, **or** the documented user-supplied fallback works end-to-end with the public-domain test lexicon | ADR + scripted run |
| AC-P2-02 | ADR-0204/0205 accepted with complete code-point mapping tables; every rule's linguistic loss is documented | ADR review by linguist |
| AC-P2-03 | All 2,000 normalization golden pairs pass for all indexed profiles | golden suite |
| AC-P2-04 | `SpanMap` satisfies all 5 properties across every ayah × every profile; offset fidelity is 1.00 | property suite |
| AC-P2-05 | **Canonical text is byte-identical** (all Phase-1 hashes match) after building every index and importing every dataset — verified by MV-018 in every job and by `doctor` | integrity suite + doctor |
| AC-P2-06 | No `SearchHit` can exist without a `NormalizationTrace`; every trace lists the ordered rule ids and flags heuristics | type test + snapshot |
| AC-P2-07 | Searching `الرحمن` (no diacritics) returns 1:1, 1:3, 2:163 and the full expected reference set; searching the exact Uthmani form returns the same set | golden query |
| AC-P2-08 | Searching `بسمالله` (no spaces, no diacritics) returns 1:1 with a segmentation explanation mapping each query part to canonical tokens 1 and 2 | golden query |
| AC-P2-09 | A Persian-keyboard query (`ک`/`ی`/`ه` variants) matches the correct Arabic text under `L5.codepoints`, and returns zero results under `L0.exact` **with a warning naming the profile to use** — never a silent fold | golden query pair |
| AC-P2-10 | Profile monotonicity holds: for every golden query, `results(Lₙ) ⊆ results(Lₙ₊₁)` across all indexed profiles | property suite |
| AC-P2-11 | Every search hit maps to exact canonical character and token ranges; slicing the canonical text at that range and re-normalizing reproduces the matched derived substring | property suite |
| AC-P2-12 | The citation resolver validates every search hit; a hit whose quotation fails verification is never returned | citation suite |
| AC-P2-13 | `quran.search_regex` rejects or bounds all 15 pathological patterns within limits; `terms_examined` and `documents_scanned` are reported; the rate limit is enforced | DoS suite |
| AC-P2-14 | All 400 search golden queries pass, including `must_not_contain` assertions (no false positives) | golden suite |
| AC-P2-15 | Morphology import: all 18 adversarial fixtures are rejected with the **specific** expected MV rule id | adversarial suite |
| AC-P2-16 | Alignment never modifies `quran_tokens`; `AlignmentTable` unmatched ratio is reported per surah and gates approval above threshold | integrity + import test |
| AC-P2-17 | The schema contains **no** `is_correct`, `is_primary`, or `selected` column on analyses; a token with competing analyses returns all of them with attribution | schema test + golden case |
| AC-P2-18 | `quran.morphology_compare` returns per-field verdicts (`identical`/`compatible_variant`/`conflicting`/`only_in_one`) and has **no** resolution or synthesis field | contract test + code review |
| AC-P2-19 | `analysis_sources` always reports `analyses_returned` and `analyses_suppressed`; suppression is never silent under any `AnalysisPolicy` | policy matrix test |
| AC-P2-20 | Every root, lemma, analysis, and derived form row has a provenance record at Layer B or D — never Layer A | doctor `quran.morphology.provenance` |
| AC-P2-21 | Every Layer D row carries algorithm, version, and confidence, and cannot reach `human_verified` without a reviewer id | DB CHECK tests |
| AC-P2-22 | Cross-dataset roots with differing spellings are **linked as reviewable suggestions**, never merged; the review queue shows the evidence before acceptance | non-merge test + queue test |
| AC-P2-23 | All 300 root and 200 lemma golden cases pass with precision 1.00 and recall ≥ 0.99 against dataset ground truth | evaluation harness |
| AC-P2-24 | Morphological segmentation matches the source dataset at ≥ 0.995 (import faithfulness) | evaluation harness |
| AC-P2-25 | `quran.word_family` returns members grouped by relation class; every member carries a human-readable explanation; the 120-family curated set scores ≥ 0.95 on linguist review | family suite + linguist sign-off |
| AC-P2-26 | Computational suggestions are **off by default**, require opt-in, carry confidence, and render with the mandatory "not verified scholarship" label in CLI, API, and tool output | default-behavior test + snapshot |
| AC-P2-27 | A suggestion becomes `ScholarVerified` only through the review queue with a recorded reviewer, timestamp, and displayed evidence | promotion-flow test |
| AC-P2-28 | Every numeric output carries a complete `CountingRules` block; two runs with identical rules produce identical numbers; changing `multi_analysis_handling` visibly changes the reported count and the rules block | determinism suite |
| AC-P2-29 | `quran.numeric_report` contains no interpretive commentary; `quran.interval_analysis` and `quran.missing_expected_form` emit their mandatory disclaimers verbatim | snapshot tests |
| AC-P2-30 | `quran.hapax_search` results change with profile and the output states the profile prominently — proving counts are rule-relative, not absolute | golden case |
| AC-P2-31 | Index builds are atomic: killing the build at any stage never activates a partial index; the previous generation continues serving | crash matrix |
| AC-P2-32 | `qai quran index rebuild --all` reconstructs every index from canonical sources + manifests, and a full cold rebuild completes in < 6 minutes | timed run |
| AC-P2-33 | Bumping any input version (corpus generation, profile version, dataset version, tokenizer version) produces the exact expected drift report from `doctor --indexes`, and every affected tool result carries a `QAI-IDX-0101` staleness warning | drift suite |
| AC-P2-34 | Drift is never auto-repaired; `doctor` remains read-only and only suggests `qai quran morphology reindex` | read-only test |
| AC-P2-35 | No stale cached result is served across a generation bump | cache-consistency test |
| AC-P2-36 | `quran-normalization`, `quran-search`, and `quran-morphology` have no dependency on `llm`, `embeddings`, `retrieval`, or any vector store crate | architecture test |
| AC-P2-37 | All 22 Phase-2 tools conform to the §12 contract, are `ReadOnly`, populate `normalization_rules`, and produce identical `reproducibility.checksum` for identical inputs on the same generation | tool conformance suite |
| AC-P2-38 | `qai quran normalize --explain` shows the rule-by-rule transformation with offset maps and flags heuristic rules | snapshot test |
| AC-P2-39 | `POST /api/v1/quran/normalization/preview` returns the same step-by-step trace as the CLI, from the same code path | parity test |
| AC-P2-40 | The query tokenizer and index tokenizer produce identical output for 5,000 random ayah substrings | parity test |
| AC-P2-41 | All 19 Phase-2 `doctor` checks are implemented; `quran.search.smoke` fails loudly on any tokenizer or profile regression | doctor run + mutation test |
| AC-P2-42 | Nightly reconciliation verifies doc counts, a 1 % sampled round-trip, lexicon foreign keys, orphan generations, and MV-018 | nightly job |
| AC-P2-43 | All latency targets in §17.1 are met and gated in CI with ≤ 20 % regression tolerance | benchmark suite |
| AC-P2-44 | All hard accuracy gates in §17.2 pass; evaluation results are versioned and diffed against the previous run | evaluation harness |
| AC-P2-45 | Two morphology adapters exist for structurally different dataset shapes, proving a new dataset needs only an adapter + manifest | adapter tests |
| AC-P2-46 | The linguist has signed off on the normalization golden set, root/lemma golden set, and family curated set; sign-off records are stored in `docs/reviews/` | signed review records |
| AC-P2-47 | ADRs 0201, 0203–0216 are `Accepted` with all §48 fields; ADR-0206 is recorded as `Reserved` with the decision deferred to Phase 4 | ADR lint |
| AC-P2-48 | Docs complete: normalization spec with mapping tables, profile catalog, search cookbook, morphology adapter guide, counting-rules explainer, reindex runbook | doc review checklist |
| AC-P2-49 | Full soak passes: rebuild all indexes → 50,000 randomized queries across all tools → `doctor` → reconciliation, with zero integrity findings, zero panics, and zero canonical-hash changes | nightly soak |
| AC-P2-50 | The MVP linguistic workflow (§44.3) succeeds end-to-end via CLI **and** API: search without diacritics → search without spaces → select a word → inspect lemma/root/morphology/word-family → view all occurrences | scripted acceptance run |

**Exit gate ritual:** a live walkthrough on a clean machine performing AC-P2-05, 07, 08, 09, 17,
22, 26, 28, 31, 33, 50 in sequence, witnessed by the Arabic linguist and the editorial reviewer
from Phase 1.

---

## 19. Risks & Mitigations

| # | Risk | L | I | Mitigation |
|---|---|---|---|---|
| R1 | No licensable morphology dataset; root/lemma/family features become empty shells | Medium | Critical | Sprint 2.0 starts during Phase 1; ADR-0203 pre-commits to the user-supplied fallback with a public-domain test lexicon; tools return typed "dataset unavailable" errors that name the missing capability — never guessed data; the search half of Phase 2 ships regardless |
| R2 | Normalization rules are linguistically wrong, silently degrading recall or precision forever | High | Critical | A qualified Arabic linguist authors and reviews the rule catalog (P2-T04/T05) *before* implementation; every rule has a published mapping table and a documented loss statement; 2,000-pair golden set with linguist sign-off; profile monotonicity property test catches inconsistent ordering |
| R3 | Dataset tokenization disagrees with Phase-1 surface tokenization; alignment becomes guesswork | High | Critical | Alignment is an explicit, hashed, auditable source file (`AlignmentTable`), never inferred; unmatched ratio is reported per surah and gates approval; re-tokenizing canonical text is architecturally impossible (Phase-1 triggers) |
| R4 | `SpanMap` bugs produce wrong highlights and unverifiable citations | High | High | 5 properties tested across every ayah × every profile; the citation resolver independently re-verifies every hit; `--explain` output makes maps human-inspectable; 92 % coverage gate on the normalization crate |
| R5 | Concatenated search either misses cross-boundary matches or explodes combinatorially | High | High | Skeleton + trigram candidate generation with exact verification bounds the work; the 3-ayah window level is a fixed cost (not combinatorial); dedup + `spans_ayah_boundary` labeling; 120-case golden set including cross-ayah cases; p99 gated at 150 ms |
| R6 | Tokenizer drift between query path and index path produces unexplainable misses | Medium | High | Both paths call the same `NormalizationPipeline` instance (T31); a 5,000-substring parity test is a hard CI gate; `search.smoke` catches regressions at runtime |
| R7 | Users treat frequency counts as absolute, then dispute them (numerology pressure) | High | Medium | `CountingRules` is a required, prominently rendered field; `multi_analysis_handling` is explicit; `numeric_report` forbids interpretive commentary; `interval_analysis` carries a fixed disclaimer; `hapax_search` demonstrates rule-relativity by design |
| R8 | Pressure to pick "the correct" morphological analysis for a cleaner UI | High | Critical | I11 is enforced at the schema level (no such column exists); `AnalysisPolicy` is a *query-time* choice always echoed in output with suppression counts; AC-P2-17/18/19 make regression detectable; ADR-0209 records why |
| R9 | Cross-dataset root unification quietly merges distinct roots | Medium | High | Unification is only ever a `review_queue` suggestion with confidence; `canonical_root_id` is nullable and Layer D; non-merge test in the integrity suite |
| R10 | Index rebuild time grows until "rebuildable from source" becomes theoretical | Medium | Medium | < 6 min cold-rebuild is a CI-gated benchmark from Sprint 2.2 onward; batch writers, parallel per-profile builds, and staging directories keep it flat |
| R11 | Regex or collocation tools become a local DoS vector | Medium | Medium | DFA engine (no backtracking), size/step/time budgets, anchor requirement, rate limits, per-principal quotas, agent-policy gating; 15-pattern abuse suite |
| R12 | Result cache serves stale data after a corpus or index change | Medium | High | Cache key includes corpus generation and every index manifest hash; wholesale invalidation on any bump; dedicated cache-consistency test |
| R13 | Scope creep into semantic/vector search or graph features | High | Medium | §2.2 fence is explicit and lists each deferred tool with its target phase; architecture test forbids the embedding/vector dependencies outright |
| R14 | Heuristic affix rules (N18–N21) are mistaken for real morphology | High | High | `RuleKind::Heuristic` is a first-class flag; `contains_heuristic_rules` appears on every trace; `affix_search` labels which backend answered; UI/CLI copy is fixed and snapshot-tested |
| R15 | Linguist unavailable, blocking golden-set sign-off | Medium | High | Linguist engaged at 0.4 FTE from Sprint 2.0; golden-set authoring is front-loaded into Sprint 2.0; sized at ≈ 14 linguist-days total across the phase |
| R16 | Tantivy custom tokenizer complexity underestimated | Medium | Medium | Tokenizers are thin wrappers over the already-tested `NormalizationPipeline`; the parity test de-risks the integration; ADR-0201 records the FTS5 fallback and its cost |

---

## 20. Definition of Done (Phase 2)

In addition to the Phase-0 and Phase-1 DoD, every Phase-2 deliverable requires:

- [ ] Canonical text provably unchanged (MV-018 runs in the same job that produced the artifact)
- [ ] Derived data written only to derived tables/indexes, at Layer B or D, with provenance
- [ ] Every user-visible result carries the ordered normalization rule set actually applied
- [ ] Every match maps back to exact canonical character, byte, and token ranges
- [ ] Heuristic and computational outputs are labeled as such in CLI, API, and tool payloads
- [ ] Competing analyses are preserved and attributed; suppression is counted and reported
- [ ] Every numeric output carries complete, reproducible counting rules
- [ ] Deterministic operations require no LLM (architecture test proves the absence of the dep)
- [ ] Resource limits enforced: result caps, timeouts, regex budgets, rate limits
- [ ] Derived artifacts record `corpus_generation` + all input versions; drift is detectable
- [ ] Index/dataset activation is atomic, approved, audited, and single-step reversible
- [ ] Golden-set and property tests exist and are linguist-reviewed where linguistic judgment applies
- [ ] Latency benchmarks gated in CI
- [ ] `qai doctor` can detect the failure mode this deliverable introduces

---

## 21. Handoff To Phase 3

| Asset | Location | Phase-3 usage |
|---|---|---|
| `quran_roots`, `quran_lemmas` rows | `quran-morphology` | promoted to `Root` / `Lemma` graph nodes (§10.1) |
| `quran_token_analyses`, `quran_morphemes` | `quran-morphology` | `HAS_LEMMA`, `HAS_ROOT`, `HAS_STEM`, `HAS_PREFIX`, `HAS_SUFFIX`, `HAS_ANALYSIS` edges |
| `quran_derivations` (Layer B) | `quran-morphology` | `DERIVED_FROM` edges with dataset provenance |
| `word_family_relations` (Layer C) | `quran-morphology` | `SAME_ROOT_AS`, `SAME_LEMMA_AS` scholar-verified edges |
| `near_duplicate_passages` output | `quran-search` | candidate `PARALLELS` / `SIMILAR_TO` edges → `review_queue` |
| Collocation / co-occurrence stats | `quran-search` | weights and candidate generation for `RELATED_TO` suggestions |
| `NormalizationPipeline` + profiles | `quran-normalization` | node key normalization; graph-query text matching |
| `SpanMap` | `quran-normalization` | mapping graph annotations to canonical spans |
| `FullTextIndex` trait + Tantivy backend | `quran-search` | reused verbatim in Phase 5 (hadith) and Phase 7 (multi-RAG lexical leg) |
| Index generation stamping + drift detection | `quran-search` | graph store adopts the identical manifest pattern |
| `review_queue` promotion flow | `application` | the mechanism for §10.6 semi-automated graph annotation |
| `CountingRules` | `application::tools` | graph metrics and path counts inherit the same discipline |
| Tool contract with populated `normalization_rules` | `application::tools` | graph tools complete the contract with `graph_version` |
| Error namespaces `QAI-NORM-*`, `QAI-IDX-*` | `domain::error` | `QAI-GRAPH-*` reserved next |
| Golden sets + evaluation harness | `fixtures/`, `tests/` | extended with graph path-correctness sets (§36.1) |

**Handoff document:** `docs/plans/handoff-p2-to-p3.md` (task P2-T114), containing the frozen
profile catalog with versions, the active morphology dataset and its attribution string, the
list of Layer D rows awaiting review, the deferred-tool list from §2.2 with target phases, and
every place Phase 3 must record `graph_version` alongside the existing version stamps.

---

## 22. Summary

Phase 2 converts a verified corpus into a research instrument without weakening a single
integrity guarantee. Its defining choices:

1. **Normalization is a versioned, inspectable, reversible transformation** — never an edit.
   Canonical text is read-only input to every index build, and that is mechanically re-verified
   after every job.
2. **Every match explains itself.** The rule set, the segmentation, the offsets, and the score
   are all part of the result, because a search hit a user cannot verify is not research.
3. **Competing scholarship stays competing.** There is no column in which a "correct" analysis
   could be stored, so no future feature can accidentally introduce one.
4. **Machine output is permanently distinguishable from dataset-supplied and scholar-verified
   data**, and can only cross that line through an explicit human decision with displayed
   evidence.
5. **Numbers carry their rules.** This single discipline defuses the most common category of
   dispute in Quranic word-counting.
6. **Nothing here needs an LLM**, and the build system proves it.
