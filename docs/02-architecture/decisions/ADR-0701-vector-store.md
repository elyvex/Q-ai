# ADR-0701 — Vector Store: sqlite-vec Default, LanceDB/Qdrant Adapters

- Status: Proposed
- Phase: 7 — Multi-RAG and Smart Routing
- Date: 2026-09-06
- Depends on: ADR-0001
- Related decisions: ADR-0201, ADR-0202, ADR-0702
- Requirements: PRD §§6.4, 18–20, 25.8, 25.12, 31–32, 41–42, 62, 73, 75–76

## Context

Q-ai needs semantic retrieval for structure-aware RAG while preserving offline,
zero-config operation.

The default vector backend should not require a service. Larger local corpora
need an embedded growth path, and server deployments need a supported remote
backend.

Embedding models, vector dimensions, distance metrics, filtering capabilities,
and approximate-search behavior differ. A common interface must not conceal
these differences or weaken access controls.

Canonical Quran lookup and exact lexical research must remain independent of
embeddings and vector availability.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| sqlite-vec | Embedded, compact integration with SQLite, no additional service | Exact-scan cost grows with corpus size and dimensions; native extension packaging |
| LanceDB | Embedded columnar/vector storage and ANN options | Separate artifacts and additional dependency footprint |
| Qdrant | Dedicated vector service, filtering, operational scalability | Requires a service or explicitly configured remote deployment |
| PostgreSQL + pgvector | Consolidates server relational and vector storage | Not the zero-config SQLite profile |
| Server-only vector default | Mature remote infrastructure | Violates local-first and offline defaults |

## Decision

Use `sqlite-vec` as the default local vector backend.

Provide optional LanceDB and Qdrant adapters behind a backend-neutral
`VectorStore` port:

- `sqlite-vec`: default local profile.
- LanceDB: opt-in larger local deployments.
- Qdrant: opt-in server or explicitly configured remote deployments.

Adapter selection is configuration-driven. There is no automatic migration
to a remote backend because a local query is slow or a backend is unavailable.

ANN algorithms and capabilities are discovered from the pinned backend
version; they are not assumed from a product name.

### 1. Interface Contract

The `VectorStore` port supports:

- Capability discovery.
- Collection/projection creation and inspection.
- Idempotent generation-scoped upsert.
- Typed filtered nearest-neighbor query.
- Deletion and deletion verification.
- Build finalization and readable-snapshot inspection.
- Projection retirement.

A query includes:

- Embedding-space identity.
- Query vector.
- Selected source snapshot and projection.
- Authorization and metadata filters.
- Top-K and candidate/effort limits.
- Exact or approximate mode where supported.
- Deadline and cancellation context.

Results include stable retrieval-unit IDs, source versions, vector-space
identity, raw distance or similarity, metric interpretation, and build identity.

Backend-native record handles and query objects do not escape the adapter.

### 2. Embedding-Space Identity

An embedding space is defined by more than dimension count.

Its identity includes:

- Provider or local runtime.
- Model identity and revision.
- Artifact digest when available.
- Input preprocessing and prompt/template version.
- Dimensions and numeric representation.
- Distance metric.
- Vector normalization policy.
- Language or task configuration where relevant.

Chunker and parser versions are recorded as projection dependencies.

Vectors from different embedding spaces must not be mixed merely because
their dimensions match. A query vector must match the target space.

This ADR does not select the Arabic embedding model or reranker; those require
their own evaluations and decisions.

### 3. Validation

Before storage or search:

- Verify the expected dimension.
- Reject non-finite numeric values.
- Apply the configured normalization policy.
- Reject invalid vectors, including zero-norm vectors where the chosen metric
  requires a nonzero norm.
- Verify source version, retrieval-unit ID, and content hash.
- Verify the requested metric is supported.

Changing dimensions, model identity, preprocessing, or metric creates a new
embedding space and projection. Do not reinterpret existing vectors in place.

### 4. sqlite-vec Integration

Store the default local vector projection in the same SQLite database file
when supported by the validated build.

Authoritative metadata uses ordinary relational tables. Vector search data
uses adapter-owned extension tables and mappings.

Native integration requirements:

- Pin compatible SQLite, sqlx, and sqlite-vec versions.
- Verify that extension registration uses the same SQLite library instance
  as the sqlx SQLite driver.
- Prefer packaged, trusted registration over runtime loading from arbitrary
  filesystem paths.
- Initialize every connection that uses extension functionality.
- Confine unsafe/native integration to the storage adapter.
- Test supported operating systems and release packaging.

The presence of an extension does not imply sqlx can register it without
adapter-specific integration. A Phase-7 integration spike is a release gate.

Do not introduce a conflicting second bundled SQLite library merely to obtain
an extension-loading convenience API.

If vector initialization fails, report a typed capability failure. Canonical
and lexical features continue to operate; semantic search is explicitly
unavailable. Do not silently send requests remotely.

### 5. Vector Records and Source Boundaries

Every vector maps to a relational retrieval unit containing:

- Corpus, source, edition, and source version.
- Document, passage, and chunk identity where applicable.
- Reversible source location.
- Content hash.
- Parser/chunker configuration.
- Computational provenance and creation time.

Quran embeddings use explicitly defined units such as an ayah or bounded ayah
window. They must not erase canonical boundaries.

Returned vector payload text is not an authoritative quotation. Retrieve and
validate source text through relational repositories before citation or display.

### 6. Filtering and Authorization

Mandatory authorization is part of retrieval semantics, not an optional
postprocessing feature.

Adapters must either:

- Enforce the required filters during vector search; or
- Use an equivalent safe strategy, such as authorized partitions or an exact
  search over an authorized ID set.

If the backend cannot enforce the required policy, fail closed or use an
explicitly supported safe fallback.

Fetching an unrestricted top-K and then removing unauthorized hits is not
equivalent to filtered top-K and must not be presented as such.

Revalidate current authorization, source status, and tombstones before results
leave the trusted retrieval layer. Never expose unauthorized payloads, counts,
or diagnostics to users, agents, or models.

### 7. Scoring and Retrieval Semantics

The adapter identifies whether a value is distance or similarity and whether
higher or lower is better.

The retrieval layer, not the vector backend, owns hybrid fusion and reranking.

- Preserve raw scores for debugging.
- Do not average incomparable scores without an explicit calibrated policy.
- Record exact versus approximate search.
- Record ANN parameters and backend version.
- Use stable domain-ID tie-breaking where feasible.
- Do not describe vector similarity as factual confidence.

Exact search supplies an evaluation baseline. Approximate search requires
versioned recall and latency evaluation.

### 8. Consistency and Lifecycle

All vector backends follow ADR-0702.

Even when sqlite-vec shares the relational file:

- Commit authoritative changes and outbox events first.
- Build the vector projection through retry-safe jobs.
- Publish only a verified readable generation.
- Verify deletion propagation.

A worker may atomically update local vector rows and its local checkpoint
within SQLite, but domain correctness must not depend on a transaction
spanning authoritative writes and every projection backend.

## Accuracy and Religious-Source Implications

- Semantic similarity is a computational suggestion, not proof of a quotation,
  root relationship, scholarly interpretation, or religious equivalence.
- Quran, translation, tafsir, and hadith evidence retain separate source labels.
- Exact Quran lookup takes precedence over semantic retrieval.
- Cross-scripture parallels remain computational unless supported by attributed
  scholarship.
- ANN may omit relevant neighbors and must not power exhaustive numerical claims.

Embedding outputs are Layer D artifacts with model and input-version provenance.

## Licensing Implications

sqlite-vec is commonly distributed under MIT/Apache-2.0 terms; the open-source
LanceDB and Qdrant projects use Apache-2.0. Verify the selected distributions,
pinned versions, dependencies, and service terms before release.

Embedding-model licenses and hosted-provider terms are separate decisions.
Check whether source licensing and privacy policy permit transmission to a
provider and storage or redistribution of derived artifacts.

## Security Implications

- Treat native extension packaging as a supply-chain boundary.
- Never accept user-selected extension binaries.
- Protect local vector files and backups.
- Require explicit configuration before sending vectors or metadata remotely.
- Treat embeddings as potentially sensitive derived data.
- Use scoped credentials, TLS where required, and sanitized endpoint handling.
- Apply request limits, timeouts, cancellation, and bounded batches.
- Enforce source deletion across vectors, payloads, caches, and retired builds.

Backend fallback must respect the initiating user's data-handling policy.

## Operational Implications

sqlite-vec favors simplicity rather than a universal scale target.
Exact-search cost depends on vector count, dimension, filtering, hardware,
concurrency, and latency requirements.

Choose a growth backend using measured:

- Search latency and memory.
- Ingestion and deletion throughput.
- Filter selectivity.
- Backup and rebuild time.
- ANN recall.
- Operational complexity.

Do not promise a fixed vector-count threshold without benchmark evidence.

Exact reproducibility may require retained embedding artifacts. Regenerating
through an unpinned or nondeterministic provider may produce different vectors.
Such regeneration creates a new identified artifact/build rather than silently
claiming an identical replay.

## Migration Strategy

For a backend change:

1. Capture a source and embedding-space snapshot.
2. Export retained vectors or regenerate them from pinned dependencies.
3. Build a shadow destination projection.
4. Verify IDs, dimensions, hashes, filters, and deletion behavior.
5. Compare exact results or measured ANN recall as appropriate.
6. Publish the new release through ADR-0702.
7. Retain the previous valid projection for rollback.

A backend migration need not re-embed if the embedding space and retained
vectors remain compatible.

## Reversal Cost

Low to medium for storage-only changes when vectors are retained.
High when embeddings must be regenerated or the embedding space changes.

Remote-provider cost, model availability, and reproducibility constraints may
dominate the storage migration cost.

## Acceptance Criteria

- The default semantic-search profile requires no vector service.
- sqlx/sqlite-vec integration passes supported-platform packaging tests.
- All adapters pass common upsert, query, filtering, and deletion tests.
- Model or dimension mismatches fail explicitly.
- Restricted data cannot reach unauthorized users, agents, or models.
- Backend failure does not break canonical lookup or lexical search.
- ANN adapters publish versioned recall benchmarks.
- Deletion is verified, not merely acknowledged by an API call.
- Backend migration preserves source addressing and embedding-space identity.
