# ADR-0201 — Full-Text Engine: Tantivy + Custom Arabic Tokenizer

- Status: Proposed
- Phase: 2 — Quran Search and Linguistics
- Date: 2026-09-06
- Depends on: ADR-0001
- Related decisions: ADR-0203, ADR-0204, ADR-0206, ADR-0702
- Requirements: PRD §§8–9, 11–12, 19, 32.2, 36, 40, 46–47, 75–76, 85

## Context

Q-ai requires offline lexical retrieval with Arabic-aware tokenization,
explicit normalization profiles, BM25 ranking, phrase and proximity search,
and source-aware metadata filtering.

Canonical Quran text must remain unchanged. Search must explain how normalized
matches map back to canonical tokens and text spans.

Generic Arabic analyzers may conflate orthographic choices, remove meaningful
distinctions, or perform stemming that users did not request. Quran morphology
also requires attributed datasets and multiple analyses rather than a hidden
single-analysis stemmer.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| Tantivy + custom tokenizer | Embedded, Rust implementation, BM25, positional indexing, configurable schema | Separate index directory; custom Arabic and lifecycle integration |
| SQLite FTS5 | Integrated with SQLite; operationally compact | Custom Arabic tokenization requires additional integration; less direct fit for the intended Rust search architecture |
| OpenSearch | Mature server search and operational features | Requires a service; unsuitable as the zero-config default |
| Relational scans and indexes only | Simple and useful for exact structured queries | Insufficient as the general ranked full-text engine |

## Decision

Use Tantivy behind the `FullTextIndex` abstraction.

Implement a custom Arabic tokenization adapter that consumes Q-ai's versioned
normalization and canonical-alignment model.

Tantivy is a derived retrieval engine, not a canonical text store and not the
owner of Arabic normalization rules.

### 1. Responsibility Boundaries

Responsibilities are divided as follows:

- `quran-normalization`: ordered normalization rules, profile versions,
  derived forms, span maps, and normalization traces.
- `quran-morphology`: dataset alignment and attributed analyses.
- `quran-search`: typed search semantics, query planning, verification,
  canonical hydration, and explanations.
- Tantivy adapter: index schema, token streams, candidate retrieval,
  positional queries, scoring, filtering, and index lifecycle.

The `FullTextIndex` port supports:

- Backend capability discovery.
- Generation-scoped build and update operations.
- Typed lexical queries.
- Stable-ID results with match evidence and score metadata.
- Inspection, deletion verification, and manifest access.

Tantivy-specific query objects, document addresses, and schema types must not
escape the adapter.

### 2. Field and Document Design

Phase 2 uses ayah-oriented documents with canonical token positions. Additional
document types may support token-level or explicitly bounded window searches.

Each indexed record includes:

- Stable retrieval-unit ID.
- Corpus, source, edition, and source-version identifiers.
- Canonical location and source content hash.
- Projection identity and generation information.
- Language, domain, and applicable access-control metadata.
- Separate searchable fields for enabled normalization profiles.
- Positional data where phrase or proximity semantics require it.

Canonical, normalized, root, and lemma fields are separate. A query must
explicitly select its semantics.

A keyword field can support whole-value equality, but is not a substitute for
canonical substring verification or positional phrase search.

Translation and commentary documents retain their own source identities.
They are never indexed as if they were canonical Quran Arabic.

### 3. Arabic Tokenization Contract

Document tokenization follows canonical token alignment for the selected
edition. It must not create a competing canonical token sequence.

For every emitted token, retain sufficient information to identify:

- The canonical token or tokens represented.
- Position and any position gap.
- Byte offsets in the indexed text representation.
- The normalization profile and version.
- The mapping back to canonical spans.

Query and document normalization use the same profile definitions.

Requirements:

- Handle Arabic combining marks and Quranic annotation marks explicitly.
- Keep hamza/alif, ya/alif-maqsura, ta-marbuta/ha, and Persian/Arabic folding
  independently configurable.
- Do not enable broad normalization in exact mode.
- Do not apply default stemming or stop-word removal to Quran exact or
  phrase fields.
- Preserve positions when normalization produces an empty token.
- Reject malformed offsets and never slice UTF-8 inside a code point.

Span maps may be one-to-many or many-to-one. Normalization is lossy, so the
system must not pretend normalized text can reconstruct the original.
Canonical display always comes from stored source text.

ADR-0204 owns the actual rule catalog and profile definitions. This ADR does
not silently choose those mappings.

### 4. Phrase, Proximity, and Concatenated Search

Define phrase semantics in Q-ai's typed query model:

- Exact ordered phrase.
- Ordered proximity with an explicit gap bound.
- Unordered proximity with an explicit window bound.
- Explicit boundary policy.

Ordinary ayah search does not match across ayah boundaries. Cross-boundary
search requires an explicitly selected window representation that preserves
the original boundaries.

Use Tantivy positional operations where they implement the requested semantics.
Otherwise retrieve candidates and verify them against relational token/form
data. Unsupported semantics must not silently degrade to bag-of-words search.

Concatenated search uses versioned joined-form or skeleton candidates, followed
by segmentation-aware verification. Removing spaces must not erase the
explanation of which canonical tokens matched.

Resource-bounded n-gram or other supporting indexes may be used. They remain
derived projections under the same generation contract.

### 5. Exact Search and Canonical Lookup

`quran.get_ayah` and exact verse navigation bypass Tantivy entirely.

Strict exact-text search verifies matches against the selected canonical
source version. A Tantivy candidate plan is permitted only if it is complete
for the requested exact-search semantics.

When token indexing cannot guarantee completeness, use a bounded relational
or canonical-text search path rather than silently missing valid matches.

BM25 scores do not establish exact textual equality.

### 6. Morphological Search

Root and lemma identities originate in attributed relational datasets.
Tantivy may accelerate retrieval but must not invent linguistic analyses.

Do not flatten alternative analyses into positional combinations that imply
an unattested analysis. Verify candidate matches against the relevant dataset,
analysis identity, and alignment.

If no morphology dataset is active:

- Return an actionable `NoDatasetActive` result for dataset-dependent tools.
- Do not substitute guessed roots or lemmas.
- Keep computational analyses explicitly in Layer D with provenance and
  review status.

Transliteration and phonetic fields may be reserved, but remain disabled until
their separate decision and implementation are available. Fuzzy spelling
search is experimental and off by default for Phase 2.

### 7. Result Contract

A lexical result contains:

- Stable canonical references and source versions.
- Retrieval-unit ID and content hash.
- Field and normalization profile used.
- Canonical token/span matches.
- Segmentation or normalization explanation.
- Analysis attribution where applicable.
- BM25 or other score with its meaning identified.
- Index build identity and reproducibility metadata.
- Warnings, including incomplete or degraded execution.

Search-index text and snippets are not authoritative quotations. Hydrate and
validate displayed quotations through the canonical repository.

### 8. Index Lifecycle

Adopt ADR-0702's consistency contract from the first Phase-2 index:

- Build into a non-active generation.
- Use a captured source/dependency snapshot.
- Commit and make the generation durably readable.
- Validate IDs, versions, counts, and required content hashes.
- Publish by changing the relational active-release pointer.
- Never replace a healthy active index with a partial build.

A Tantivy writer commit alone is not Q-ai source activation.

Use bounded worker pools for indexing and CPU-heavy search work. Propagate
deadlines and cancellation without blocking the async runtime.

## Accuracy and Religious-Source Implications

- Search normalization never changes displayed Quran text.
- Exact, normalized, morphological, fuzzy, and semantic matches remain distinct.
- BM25 relevance is not confidence, scholarly authority, or theological weight.
- Root and lemma matches retain dataset attribution and competing analyses.
- Numeric counts use explicit deterministic counting rules, not ranked hit
  counts from a truncated search page.

Golden tests must cover Uthmani marks, combining characters, normalization
collisions, attached forms, repeated words, positional gaps, and edition
boundaries.

## Licensing Implications

Tantivy is distributed under the MIT license; verify pinned dependencies and
required notices.

Indexing a corpus does not remove its licensing restrictions. Stored snippets,
exports, cached results, and morphology data remain subject to source licenses.

## Security Implications

- Compile typed query input; do not expose arbitrary backend query syntax.
- Bound query length, clause count, expansions, regex complexity, results,
  memory, and execution time.
- Apply source and authorization filters during retrieval.
- Revalidate current authorization and tombstones before releasing results.
- Do not leak unauthorized snippets, facets, or counts.
- Validate index paths and prevent path traversal or symlink escape.

Untrusted source text is data, never an instruction to the application or model.

## Operational Implications

Benefits:

- Embedded lexical search without a server.
- Explicit Arabic search behavior.
- Reusable full-text infrastructure for later corpora.

Costs:

- A separate directory and writer lifecycle.
- Custom tokenizer and span-map maintenance.
- Rebuilds for incompatible schema or tokenizer changes.
- Corpus-dependent BM25 statistics and backend-version-sensitive ranking.

Deterministic replay requires pinned inputs, index/scoring configuration,
compatible engine versions, and stable tie-breaking by domain ID.

## Migration Strategy

For tokenizer, schema, or backend changes:

1. Create a new versioned projection configuration.
2. Build a shadow index from authoritative source versions.
3. Run golden-query, citation, authorization, and performance tests.
4. Compare result changes explicitly.
5. Publish the new release atomically.
6. Retain the prior valid release for rollback.

Other full-text backends implement the same semantic contract. Capability gaps
must be explicit; an adapter swap cannot silently change phrase semantics.

## Reversal Cost

Medium.

The index is disposable, but Arabic tokenization, schema mapping, query
semantics, and ranking evaluation must be reimplemented or adapted.
Canonical text and linguistic provenance do not require conversion.

## Acceptance Criteria

- Phase-2 lexical search works fully offline without an LLM or vector store.
- Indexing and searching never modify canonical text.
- Query and document normalization pass shared golden tests.
- Highlight spans resolve to valid canonical UTF-8 boundaries.
- Phrase, proximity, and concatenated matches expose verified token mappings.
- Missing morphology never produces guessed dataset results.
- Unauthorized records cannot appear in hits, snippets, facets, or counts.
- Interrupted builds never replace the active index.
- Rebuilds from the same pinned inputs reproduce deterministic match sets.
