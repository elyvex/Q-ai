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
```

`quran.search.smoke` is a genuinely valuable check: it runs a dozen fixed queries with known
expected references (e.g. `الرحمن`