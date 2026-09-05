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

| RuleId | Name | Effect |
|---|---|---|
| `N01` | `whitespace_collapse` | Collapse runs of whitespace to a single U+0020; trim |
| `N02` | `strip