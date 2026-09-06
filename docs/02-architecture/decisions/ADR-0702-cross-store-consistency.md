# ADR-0702 — Cross-Store Consistency, Generation Stamping, and Reconciliation

- Status: Proposed
- Phase: 7 — Multi-RAG; foundational subset required from Phases 0–3
- Date: 2026-09-06
- Depends on: ADR-0001
- Applies to: ADR-0201, ADR-0202, ADR-0701
- Requirements: PRD §§12.1, 34, 39, 41, 50, 54, 73, 75–76, 82, 85, 93

## Context

Q-ai combines an authoritative relational store with independently committed
full-text, graph, and vector projections.

A crash can occur after a source change but before indexing, after a projection
write but before job acknowledgment, or during activation. Separate stores
cannot be assumed to participate in one transaction.

The system must prevent partial activation, stale-source confusion, deleted
content resurfacing, and loss of original annotations.

A generation number alone is insufficient: two indexes may carry the same
number while depending on different normalization rules, datasets, models, or
source-version selections.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| Synchronous dual writes | Simple happy path | Partial failures create drift; no atomicity across stores |
| Distributed transactions / two-phase commit | Strong theoretical coordination | Not supported uniformly; excessive complexity for local engines |
| Best-effort jobs without durable publication metadata | Easy initial implementation | Cannot reliably prove completeness or recover activation |
| Relational authority + transactional outbox + versioned projections | Local-first, retry-safe, rebuildable, explicit failure handling | Requires manifests, workers, activation coordination, and reconciliation |

## Decision

Use:

1. One authoritative relational state.
2. A transactional outbox.
3. At-least-once, idempotent projection jobs.
4. Versioned dependency snapshots and projection manifests.
5. Atomic relational publication of complete release manifests.
6. Continuous drift detection and explicit repair.
7. Tombstones with verified propagation.

Do not use cross-store transactions.

The ADR number reflects Phase-7 integration, not permission to defer basic
consistency until Phase 7:

- Phase 0 establishes outbox, revisions, jobs, and tombstones.
- Phase 2 applies generation manifests and safe full-text publication.
- Phase 3 applies the same contract to graph projections.
- Phase 7 coordinates multi-store releases and vector reconciliation.

### 1. Authority and Ownership

The relational store is authoritative for:

- Source versions and activation state.
- Canonical and structured content.
- Original scholarly/user annotations and computational provenance.
- Source access and licensing policy.
- Desired projection state.
- Job intent, release manifests, and tombstones.

Managed source files and retained artifacts are authoritative inputs when
referenced by relational manifests and verified hashes.

Projection stores never decide independently which source version is active.

A projection rebuild must not delete original graph annotations, review
history, or other authoritative records.

### 2. Generation and Snapshot Model

Use distinct identifiers for distinct purposes.

#### Corpus generation

`corpus_generation` is a monotonic revision within a defined corpus scope.

Allocate it transactionally when projection-relevant authoritative state
changes. A rollback to older source content still creates a new generation;
generation numbers never move backward.

Serialize generation allocation through relational locking or an equivalent
backend-specific transaction mechanism. Do not infer committed ordering from
wall-clock timestamps or the maximum allocated job ID.

#### Dependency snapshot

A dependency snapshot captures the exact inputs required to build a projection:

- Corpus scopes and their generations.
- Selected source IDs, versions, and hashes.
- Selected annotation/dataset revisions.
- Ordered normalization profiles and versions.
- Parser, chunker, tokenizer, and graph-builder versions as applicable.
- Embedding-space identity and retained artifact hashes as applicable.

Capture a consistent snapshot using a relational read transaction and
versioned input records, or materialize the required input manifest.
A manifest must not claim historical revisions that the builder can no longer
read consistently.

#### Projection build

`build_id` identifies one concrete build attempt or immutable output set.

Multiple builds may target the same dependency snapshot. They are not assumed
byte-identical merely because their target generation matches.

#### Published release

A release manifest references a compatible set of verified projection builds
and the authoritative source/dependency snapshot they serve.

A query pins one release, rather than independently asking every backend for
its own "latest" index.

### 3. Projection Manifest

Every projection manifest records at least:

    manifest_schema_version
    projection_id
    projection_kind
    backend_type
    backend_version
    build_id
    corpus_scope
    corpus_generation
    dependency_snapshot_id
    dependency_snapshot_hash
    source_versions
    rule_set_version
    dataset_version
    producer_versions
    projection_schema_version
    backend_configuration_hash
    embedding_space_id
    physical_locator
    state
    expected_record_count
    actual_record_count
    validation_summary
    created_at
    completed_at

For multi-corpus projections, record a sorted scope-to-generation map.

`rule_set_version` and `dataset_version` identify the relevant version set or
its canonical digest, not an ambiguous single global string. Use an explicit
not-applicable value when a dependency category does not apply.

The manifest schema defines canonical serialization and hash computation.

Backend-specific schema/configuration hashes may legitimately differ across
stores. Compatibility is determined from the declared shared source and
dependency requirements, not by requiring every manifest field to be equal.

### 4. Transactional Outbox

A projection-relevant write follows this sequence in one relational transaction:

1. Validate permissions, source state, and applicable approval.
2. Write authoritative records or lifecycle changes.
3. Allocate the affected revision/generation.
4. Record durable outbox events.
5. Record required audit events.
6. Commit.

An outbox event identifies:

- Event ID and operation.
- Corpus scope and target generation.
- Affected entity/source version.
- Input revision or content hash.
- Required projection targets.
- Idempotency key.
- Creation and retry metadata.

Dispatch happens after commit.

The database commit succeeds independently of projection availability. The
resulting state may be pending indexing, but must never be falsely reported
as fully indexed or activated.

### 5. Worker Semantics

Delivery is at least once. Exactly-once execution is not assumed.

Workers must support:

- Idempotent writes using stable versioned record keys.
- Bounded retries with backoff and jitter.
- Durable checkpoints.
- Leases and interrupted-job recovery.
- Cancellation.
- Failed/dead-letter state with actionable diagnostics.

Use generation-specific destinations and a single effective writer or fencing
mechanism per destination. An expired worker must not overwrite a newer build
or publish an obsolete release.

Jobs operate on captured input versions. They must not read "whatever is
current" and label the output as an earlier generation.

If events are coalesced, the replacement job must build the complete newer
snapshot. Skipped work cannot be hidden by advancing a high-water mark.

A consumed-event checkpoint is not proof that a projection is complete,
durable, or searchable.

### 6. Build, Verify, and Publish

The lifecycle is:

    Pending → Building → Verifying → Ready → Published → Retired

Failures and cancellation enter explicit terminal or resumable states and do
not replace the active release.

Publication proceeds as follows:

1. Capture the target dependency snapshot.
2. Build each required projection in a non-active destination.
3. Commit/flush backend output using its supported durability mechanism.
4. Confirm the output is readable under its recorded locator.
5. Validate source IDs, revisions, content hashes, expected membership,
   tombstones, and required semantic checks.
6. Mark verified builds `Ready` in relational metadata.
7. In one relational transaction:
   - Recheck approval and activation preconditions.
   - Verify all required builds belong to the compatible target snapshot.
   - Compare the expected current release to prevent a stale coordinator
     from overwriting a newer publication.
   - Publish the new release and corresponding source activation state.
   - Record the activation audit event.
8. Retire old builds only after retention and active-reader rules allow it.

Physical preparation happens before relational publication. A crash before
step 7 leaves unused output, not an active partial release.

There is no filesystem/database distributed commit. The relational pointer is
the publication authority; backend artifacts must already be ready.

### 7. Incremental Builds

Incremental work is encouraged, but published views must remain stable.

An adapter may implement stable snapshots using:

- Separate directories or collections.
- Versioned rows filtered by snapshot.
- Backend snapshots.
- Copy-on-write structures.
- Immutable segments referenced by a new manifest.

A build does not need to duplicate unchanged physical data. It does need a
verifiable logical membership set for the captured snapshot.

Do not mutate an active shared destination in ways that expose half of a new
generation while retaining the old manifest label.

An adapter without safe snapshot semantics must use a shadow rebuild or an
explicit maintenance window, not silently weaken the publication contract.

### 8. Read Consistency

Canonical lookup reads the authoritative source version directly and never
depends on full-text, graph, or vector health.

Projection-dependent requests:

1. Resolve and pin a release manifest.
2. Determine required projections for the query plan.
3. Check their declared compatibility and availability.
4. Query those exact builds.
5. Hydrate source records by stable ID and version.
6. Revalidate current authorization, source restrictions, and tombstones.
7. Return build and source versions in reproducibility metadata.

Default strict mode does not silently mix incompatible source snapshots.

An explicitly requested degraded mode may omit an unavailable retrieval
channel and continue with compatible channels. It must report:

- Which channel was omitted.
- Why it was omitted.
- The release and source versions used.
- Any effect on completeness or ranking.

Version pinning does not override current access revocation or removal policy.
If a pinned version is no longer legally or operationally accessible, replay
fails explicitly rather than substituting another source version.

### 9. Tombstones and Deletion

Deletion or deactivation begins with a relational transaction that records:

- The tombstone or lifecycle transition.
- A new relevant revision/generation.
- Outbox work for affected projections and caches.
- The required audit event.

Current policy blocks retrieval immediately, even while physical cleanup is
pending.

Propagation covers:

- Full-text documents.
- Vector records and payloads.
- Graph nodes and affected edges.
- Parsed/chunk representations subject to removal policy.
- Caches, previews, and temporary artifacts.
- Retained and retired projection builds where required.

Canonical versions are normally deprecated or deactivated, not destructively
edited. Historical availability follows authorization, retention, and legal
policy.

For physical deletion:

- Record per-target acknowledgment.
- Verify absence using backend-supported reads or enumeration.
- Keep failures visible and retryable.
- Prevent older jobs from resurrecting removed content.
- Do not discard tombstones before the replay/retention horizon is safe.

Backend acknowledgment is not equivalent to verified absence.

Physical erasure from backups, storage media, and remote service retention is
a separate operational/legal concern. Do not claim secure erasure merely
because an index delete succeeded.

### 10. Reconciliation

Reconciliation compares desired relational state with actual backend state.

Checks include:

- Missing or unreadable projections.
- Source/dependency version drift.
- Missing expected records.
- Unexpected orphan records.
- Incorrect IDs, hashes, dimensions, or schema.
- Unpropagated tombstones.
- Incomplete builds and abandoned staging output.
- Interrupted jobs and expired leases.
- Published manifests pointing to unavailable artifacts.

Record counts alone do not prove consistency.

Use:

- Fast manifest and health checks at startup and periodically.
- Scoped membership/hash checks after updates.
- Deeper scheduled scans.
- Full validation during rebuild, restore, and migration.

Repair options include requeueing idempotent work, rebuilding a projection,
retiring an invalid build, and cleaning verified orphan artifacts.

Never repair canonical text automatically to make it agree with an index.

### 11. Doctor and Repair Controls

`qai doctor --indexes` and `qai doctor --json` are read-only diagnostics.

`qai doctor --repair-preview` produces an explicit proposed repair plan.

The plan identifies:

- Affected sources and projections.
- Detected mismatch.
- Proposed action.
- Expected data and service impact.
- Required permissions.
- Whether network/model use or cost is involved.

Repair is a separate authorized action. Confirmation is required when it
changes indexes, metadata, activation, or user data.

`qai index rebuild --all` rebuilds enabled projections from retained,
authorized authoritative inputs. It must report unavailable models,
missing artifacts, or licensing constraints instead of inventing replacements.

Automatic retries of already authorized jobs are distinct from a diagnostic
command initiating a new repair.

### 12. Reproducibility

Research provenance includes:

- Release and projection build IDs.
- Source and edition versions.
- Dependency snapshot hash.
- Normalization and dataset versions.
- Embedding-space and retained vector artifact identity.
- Graph-builder, tokenizer, parser, and chunker versions.
- Retrieval, ANN, fusion, and reranking configuration.
- Tool versions and stable ordering policy.

Deterministic tools must reproduce their semantic output for the same
accessible pinned inputs and implementation contract.

Do not promise byte-identical engine files, identical timing, or deterministic
ANN/model output without evidence.

If regeneration changes embeddings or another nondeterministic artifact,
record a new artifact/build identity and disclose the difference.

## Accuracy and Religious-Source Implications

- A query cannot silently combine an old morphology dataset with a new corpus
  edition while claiming one consistent result.
- Canonical quotations are always verified against an identified source version.
- Original scholarly assertions and review decisions survive index rebuilds.
- Deactivated or restricted material cannot reappear through stale indexes.
- Rollback changes active selection, never historical canonical content.
- Reconciliation does not promote computational suggestions to verified claims.

## Licensing Implications

Rebuild permission is not the same as redistribution permission.

Generation manifests preserve source-license identity and relevant changes.
Rebuilds, remote embedding calls, exports, retained snapshots, and deletion
must comply with current source and provider policies.

If a license change prevents historical replay, report that limitation
explicitly.

## Security Implications

- Authorization and tombstones override stale projection payloads.
- Repair, activation, deletion, and garbage collection require explicit
  capabilities and audit records.
- Manifest and artifact paths are validated.
- Hashes detect corruption but do not authenticate maliciously replaced data;
  signed manifests or another trust mechanism are needed where authenticity
  is required.
- Diagnostics redact restricted source details and secrets.
- Retired snapshots and caches receive the same access protections as active
  data.

## Operational Implications

Benefits:

- Crash-safe publication without distributed transactions.
- Rebuildable projections and observable consistency.
- Safe backend replacement and rollback.
- Explicit completeness and degradation behavior.

Costs:

- Additional metadata, jobs, manifests, and validation work.
- Temporary storage overhead for shadow builds.
- Indexing lag between authoritative commit and searchable publication.
- Retention and garbage-collection coordination.
- Backend-specific snapshot and deletion-verification implementations.

Monitor generation lag, oldest pending event, build failures, retry counts,
tombstone age, missing artifacts, reconciliation mismatches, and disk usage.

## Migration Strategy

For existing unstamped projections:

1. Inventory authoritative sources and current backend data.
2. Introduce relational generations, outbox, and manifest tables.
3. Treat legacy projections as unverified.
4. Capture a dependency snapshot.
5. Rebuild or fully validate shadow projections.
6. Publish a verified release.
7. Retire legacy data after rollback and retention requirements are met.

Version the manifest schema. Readers must reject unsupported manifests rather
than guess their meaning.

## Reversal Cost

High.

This protocol defines observable consistency, deletion, activation, and
reproducibility semantics across the application.

Individual backends remain replaceable. Removing generation-aware publication
or the transactional outbox would weaken product guarantees and requires a
new ADR with equivalent protections.

## Acceptance Criteria

- A crash between relational commit and dispatch loses no indexing intent.
- Duplicate delivery does not create duplicate logical records.
- A crash after backend commit but before acknowledgment is safely recoverable.
- Expired workers cannot publish over newer releases.
- Partial builds never become active.
- Queries pin compatible source/dependency snapshots.
- Tombstoned or access-revoked content is immediately blocked from disclosure.
- Late jobs cannot resurrect deleted records.
- Reconciliation detects version drift, missing records, and orphans.
- Doctor diagnostics do not mutate data.
- Rebuilds preserve canonical hashes, original annotations, and citations.
- Backup restore can reconstruct projections without relying on their backups.
- Fault-injection tests cover every build and activation transition.
- Multi-store outages produce explicit errors or declared degraded execution,
  never silent source substitution.
