# ADR-0202 — Graph Store Abstraction: Relational Adjacency + Bounded CTE First

- Status: Proposed
- Phase: 3 — Quran Graph
- Date: 2026-09-06
- Depends on: ADR-0001
- Related decisions: ADR-0201, ADR-0702
- Requirements: PRD §§6, 9–10, 11.5, 15, 25.12–25.13, 32.4, 40, 75–76, 81

## Context

Q-ai requires a first-class typed knowledge graph for Quran structure,
linguistic relationships, concepts, scholarly annotations, and later hadith
and isnad research.

The local MVP must not require a dedicated graph database.

Graph results must preserve provenance, uncertainty, edition identity, and
the exact path used to produce a result. Traversal must be resource-bounded
and authorization-aware.

Some graph information is deterministic and rebuildable; other information
is original scholarly or user work. These categories must not be confused.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| Relational adjacency + bounded recursive CTEs | No additional service; natural provenance joins; SQLite/PostgreSQL path | Recursive queries require careful limits and indexing |
| Embedded graph engine | Graph-oriented execution without a remote service | Additional dependency, packaging, and migration complexity |
| PostgreSQL graph extension | Server-side graph capabilities near relational data | Not suitable as the local default; extension dependence |
| Dedicated graph database | Rich traversal tooling and graph query languages | Operational overhead; unnecessary for local MVP |
| Graph only in memory | Simple small-fixture traversal | Startup, memory, persistence, and concurrency limitations |

## Decision

Define a backend-neutral `GraphStore` port and implement it initially with
relational adjacency tables and bounded recursive CTEs.

Use application-controlled, batched frontier traversal where it provides more
reliable expansion limits than a single recursive query.

Do not expose unrestricted SQL, Cypher, or another backend query language
through the public graph API.

### 1. Authority Versus Projection

Authoritative relational data includes:

- Canonical corpus structure.
- Dataset-supplied linguistic relationships.
- Scholar-authored and user-created graph assertions.
- Computational suggestions and their provenance.
- Review decisions, corrections, rejections, and evidence.

The graph retrieval projection includes:

- Materialized nodes and adjacency edges.
- Deterministic structural relationships.
- Traversal indexes and denormalized filter fields.

Rebuilding the graph projection must not discard human annotations or review
history. "Graph is rebuildable" means the traversal representation is
rebuildable from authoritative assertions and source versions.

A write through a graph annotation service first records an authoritative
assertion, audit event, and outbox event in a relational transaction.
It does not write only to a disposable adjacency table.

### 2. GraphStore Contract

The port supports typed operations for:

- Node resolution.
- Neighbors.
- Bounded paths.
- Bounded subgraphs.
- Bounded graph patterns.
- Projection build, inspection, and deletion verification.
- Capability discovery.

Every query carries:

- Selected corpus/source snapshot.
- Authorization context.
- Start nodes and, where applicable, end nodes.
- Allowed node and edge types.
- Direction.
- Provenance, source, confidence, and verification filters.
- Explicit traversal and result budgets.
- Deadline and cancellation context.

Backend-specific handles and query languages remain private.

### 3. Relational Representation

Use versioned relational tables conceptually equivalent to:

    graph_nodes
      node_id
      node_type
      authority_reference
      source_version_id
      projection_id
      snapshot_id

    graph_edges
      edge_id
      source_node_id
      target_node_id
      edge_type
      assertion_id
      provenance_kind
      projection_id
      snapshot_id

Authoritative assertion and provenance records contain, where applicable:

- Source ID and exact source location.
- Author, scholar, user, dataset, or algorithm.
- Algorithm/dataset version.
- Confidence and its interpretation.
- Verification status.
- Creation time.
- Reviewer, decision, and review time.
- Supporting evidence references.

Deterministic structural edges carry builder and input-version provenance.
They are distinguishable from interpretive assertions.

Use foreign keys and indexes supporting:

- Source node + edge type.
- Target node + edge type.
- Snapshot/projection scope.
- Assertion lookup.
- Common source and verification filters.

Frequently queried typed properties should use explicit columns or normalized
tables. JSON may hold extension properties but must not replace integrity
constraints or the core query model.

### 4. Identity and Multiple Assertions

Graph identities use stable domain IDs, not backend rowids.

Do not make `(source_node, edge_type, target_node)` universally unique:
multiple scholars or datasets may independently assert the same relationship
with different evidence or confidence.

Use stable assertion-aware edge identities. Deduplication for presentation
must preserve all underlying attribution.

Never silently merge:

- Different Quran editions.
- Competing morphological analyses.
- Uncertain narrator identities.
- Contradictory scholarly interpretations.
- Sourced parallels and computational similarity suggestions.

### 5. Structural and Linguistic Edges

Generate structural edges from canonical records without modifying them.

Promote Phase-2 relational root, lemma, and analysis records into graph nodes
while preserving dataset identity and alignment.

Do not eagerly materialize every pair of tokens sharing a root. Prefer
traversal through a root node unless a measured workload justifies a versioned
materialized relationship.

This avoids quadratic edge growth and keeps word-family provenance explicit.

### 6. Bounded Query Execution

Every traversal has enforced limits for:

- Maximum depth.
- Expanded nodes and edges.
- Returned nodes, edges, and paths.
- Pattern size.
- Per-query memory or frontier size.
- Wall-clock duration.

Limits are validated before execution and enforced while running.

A final SQL `LIMIT` is not an adequate traversal budget: recursive work may
already have expanded far beyond the returned result count.

Recursive CTE implementations must include depth constraints, cycle handling,
and backend cancellation. For queries where expansion cannot be bounded
reliably inside SQL, use batched frontier expansion with counters.

SQLite interruption/progress support and PostgreSQL statement cancellation or
timeouts belong inside their respective adapters.

### 7. Path Semantics

The request explicitly selects the supported path mode, for example:

- Reachability within N hops.
- One minimum-hop path under the requested filters.
- Up to K bounded paths.
- Bounded pattern matches.

Do not imply that all paths were enumerated or that a shortest path was proven
when a budget stopped execution before proof was possible.

Results include:

- Start and end nodes.
- Ordered nodes and edges for each path.
- Edge types and provenance.
- Applied filters.
- Execution duration and expansion counts.
- Snapshot/build identity.
- Completion status and truncation reason.

An interrupted or bounded-out search returns an explicit incomplete result
or typed error. It must not report "no path exists" when it only means
"no path found within this budget."

## Accuracy and Religious-Source Implications

- A graph connection is not, by itself, a scholarly conclusion.
- Structural, dataset-supplied, scholarly, user, and computational edges
  remain visibly distinct.
- Confidence is attributed evidence metadata, not a universal truth score.
- Computational suggestions do not become verified edges without an explicit
  review decision.
- Disputed claims and uncertain identities remain separate and inspectable.
- Canonical quotations resolve through the relational source store, not graph
  property copies.

Do not automatically combine path confidence values into theological or
historical certainty.

## Licensing Implications

The initial backend adds no dedicated graph-engine license.

Graph assertions may derive from licensed datasets or scholarly works.
Preserve attribution, source location, and export restrictions for individual
assertions and their supporting evidence.

A graph export is not automatically exempt from source licensing.

## Security Implications

Authorization must apply to:

- Starting nodes.
- Every expanded node.
- Every traversed edge.
- Supporting provenance.
- Returned paths, counts, and exports.

It is insufficient to filter only final nodes: a hidden intermediate node
could otherwise reveal a restricted relationship.

Typed patterns use allowlisted predicates and parameterized values.
Reject unsupported operations rather than forwarding raw query text.

Graph budgets protect against accidental and adversarial expansion.

## Operational Implications

Benefits:

- No additional service for the local MVP.
- Transactional storage of original annotations and review history.
- Straightforward provenance and source joins.
- A PostgreSQL adjacency implementation can reuse the logical model.

Costs:

- Deep or highly connected traversals may become expensive.
- Query planning and indexes require workload-specific testing.
- Backend implementations need equivalent path and truncation semantics.

Benchmark representative Quran graphs first, then later isnad and cross-source
graphs. Do not select a dedicated graph database solely on total edge count.

## Migration Strategy

When measured graph workloads exceed the relational backend's practical limits:

1. Select a replacement adapter through a separate ADR.
2. Export authoritative nodes, assertions, and provenance by stable ID.
3. Build a shadow graph projection.
4. Validate identity, attribution, authorization, and path semantics.
5. Run result-equivalence and bounded-execution tests.
6. Publish the replacement projection through ADR-0702.
7. Retain the relational authority and prior projection for rollback.

The new graph backend remains a projection, not a new authority for original
scholarly or user annotations.

## Reversal Cost

Low to medium for storage replacement; potentially high for query-semantic
changes.

Typed patterns, stable identifiers, relational assertion ownership, and
backend conformance tests reduce migration cost. Backend-native query-language
exposure would increase it and is therefore excluded.

## Acceptance Criteria

- The Phase-3 graph runs without a graph database service.
- Deleting and rebuilding projections preserves annotations and review history.
- Neighbor, path, subgraph, and pattern queries enforce budgets.
- Cyclic and high-degree fixtures terminate within configured limits.
- Incomplete traversal is distinguishable from an empty complete result.
- Every non-structural edge resolves to attribution and evidence.
- Unauthorized intermediate nodes cannot appear in or influence disclosed paths.
- Phase-2 linguistic identities survive promotion into graph nodes.
- Canonical lookup continues to work when the graph projection is unavailable.
