# ADR-0001 — Relational Store: SQLite + sqlx, PostgreSQL-Portable SQL

- Status: Proposed
- Phase: 0 — Foundations
- Date: 2026-09-06
- Related decisions: ADR-0201, ADR-0202, ADR-0701, ADR-0702
- Requirements: PRD §§6–7, 25.12–25.13, 32.1, 33, 41, 55–56, 75–76, 82, 85

## Context

Q-ai needs a transactional, local-first store for canonical source text,
source versions, structured corpora, provenance, annotations, workspaces,
configuration, jobs, and audit records.

The default installation must work offline without a database daemon.
Canonical text must remain immutable within an approved source version.
Server deployments must eventually support PostgreSQL without rewriting
domain logic.

Full-text, vector, and graph retrieval have different indexing requirements.
Their optimized representations must not become independent sources of truth.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| SQLite + sqlx | Embedded, transactional, mature, Rust async integration, PostgreSQL driver available | Single-writer constraint; backend-specific SQL still requires care |
| SQLite + rusqlite | Direct SQLite API and extension integration | SQLite-specific application integration; synchronous API |
| PostgreSQL from the beginning | Strong concurrency, server access controls, operational tooling | Requires a service; weakens zero-config local deployment |
| Embedded PostgreSQL distribution | PostgreSQL semantics locally | Packaging and lifecycle complexity; not genuinely lightweight |
| ORM-first persistence | Convenient entity mapping and CRUD | Does not remove SQL differences; may obscure corpus constraints and graph queries |

## Decision

Use SQLite as the default authoritative relational database, accessed through
sqlx-backed storage adapters.

Design schemas and repository contracts for PostgreSQL portability. Implement
PostgreSQL as a separate adapter when server deployment requires it.

SQLite is the local implementation of the authoritative relational store,
not an authority hard-coded into the domain model.

### 1. Authoritative Data

The relational store owns:

- Sources, editions, immutable source versions, and content hashes.
- Quran hierarchy, canonical text, token order, and stable addressing.
- Hadith, tafsir, scripture, document, and chunk records.
- Dataset-supplied linguistic analyses and their alignment records.
- Scholarly, user, and computational annotations with explicit trust layers.
- Graph assertions, review decisions, and provenance.
- Users, access policies, workspaces, configuration, jobs, and audit events.
- Projection manifests, generation metadata, outbox events, and tombstones.

Large original files and retained computational artifacts may live in managed
file or object storage. Their identity, hash, version, location, and retention
policy must be recorded relationally.

Full-text indexes, vector indexes, and materialized graph traversal structures
are rebuildable projections.

### 2. Storage Boundary

Domain and application code use typed storage ports, not sqlx directly.

The relational boundary consists of:

- A `Database` port for connection-independent capabilities.
- Typed repositories for domain reads and writes.
- A `UnitOfWork` for atomic changes spanning repositories.
- A restricted maintenance interface for migrations, backup, and recovery.

Transactions must use one underlying connection for their entire lifetime.
A repository participating in a unit of work must not silently acquire another
connection and commit independently.

Concrete pools, transactions, SQL rows, and driver errors remain inside storage
adapters. Expose typed application errors such as:

- `Conflict`
- `NotFound`
- `ImmutableSourceVersion`
- `ConstraintViolation`
- `StorageBusy`
- `MigrationRequired`
- `StorageUnavailable`

Do not expose unrestricted SQL through ordinary domain ports or agent tools.

### 3. SQLite Configuration

The normal writable local profile uses:

    journal_mode = WAL
    synchronous = FULL
    foreign_keys = ON

Operational requirements:

- Verify WAL mode during database initialization.
- Apply connection-local settings, especially foreign-key enforcement, to
  every connection.
- Configure a bounded busy timeout and bounded application retries.
- Keep write transactions short.
- Perform parsing, normalization, embedding generation, and network requests
  outside database transactions.
- Bound pool size and writer concurrency.
- Monitor WAL growth and checkpoint progress.
- Use supported local filesystems with working locking semantics; do not
  treat a database on an arbitrary network share as a supported server profile.

WAL improves reader/writer concurrency but does not remove SQLite's
single-writer limitation.

Durability settings do not replace checksums, backups, filesystem checks, or
corpus-integrity validation.

### 4. Canonical Immutability

Approved canonical text and structural identity are append-only by version.

Enforcement includes:

- Foreign keys and uniqueness constraints for edition-aware addressing.
- Database triggers rejecting updates or deletes to protected canonical rows.
- Repository APIs that create new versions instead of editing approved ones.
- Separate lifecycle metadata for activation, deprecation, and tombstones.
- Audited approval and activation workflows.

Corrections create a new source version, including validation and a difference
report. Changing the active version does not rewrite historical text.

Ordinary application code and agents cannot bypass immutability protections.
Any legally required physical purge is a separate privileged maintenance
workflow, not a canonical-edit API.

### 5. PostgreSQL Portability

"PostgreSQL-portable" means equivalent behavior behind the same contracts,
not one SQL string or migration file that works everywhere.

Use these conventions:

- Application-generated stable identifiers; never expose SQLite rowids.
- Explicit column types, nullability, constraints, and ordering.
- Unicode-preserving text storage; no implicit search normalization.
- Application-defined timestamp and Boolean mappings for each backend.
- Parameterized queries.
- Standard joins, aggregates, and recursive CTEs where practical.
- Explicit conflict targets for upserts.
- No `INSERT OR REPLACE` for authoritative rows.
- No dependency on SQLite's permissive typing or implicit type coercions.
- No linguistic correctness dependency on database locale or collation.

Backend-specific placeholders, locking, triggers, JSON operations, extension
DDL, and query optimizations belong in adapters.

Use backend-specific migration directories with corresponding logical schema
versions. Do not assume sqlx's generic connection facilities translate SQL
dialects or validate both schemas automatically.

### 6. Transactions and Jobs

A transaction that changes projection-relevant authoritative state must also:

1. Record the relevant revision or generation change.
2. Insert a durable outbox event.
3. Record required audit information.

The state change and event commit together.

Projection workers run after commit and follow ADR-0702. Do not hold a
relational transaction open while writing Tantivy, LanceDB, or Qdrant.

### 7. Backups

Database backup must use a supported SQLite-consistent mechanism, such as its
online backup API or another validated snapshot procedure.

Copying only the live main database file while ignoring its WAL is not a
supported backup strategy.

A recoverable backup includes:

- A consistent relational snapshot.
- Referenced source manifests and retained source files.
- Authoritative annotations and review history.
- Required configuration and audit data.
- An inventory and hashes for associated artifacts.

Derived indexes may be backed up for faster recovery, but restore must work
without them. Restored projections are validated against relational manifests
before use.

## Accuracy and Religious-Source Implications

- Canonical lookup reads only the authoritative relational corpus.
- Quran quotations never come from a vector payload or search-index snippet.
- Edition, recitation, numbering scheme, and source version remain explicit.
- Translations and annotations cannot overwrite canonical Arabic.
- Multiple morphological analyses and scholarly disagreements remain separate.
- Database comparison behavior cannot silently define Arabic normalization.

Canonical immutability and exact addressing are correctness constraints, not
merely repository conventions.

## Licensing Implications

SQLite is distributed as public-domain software; sqlx is commonly distributed
under MIT/Apache-2.0 terms. Verify the exact pinned dependencies and notices
during dependency review.

Software licensing does not grant rights to distribute a corpus.
Source licenses, attribution, export restrictions, and version-specific
license changes remain authoritative metadata.

## Security Implications

- Bind Q-ai locally by default.
- Restrict permissions on the data directory and backups.
- Use read-only connections or roles for read-only execution paths.
- Separate importer, migration, maintenance, and ordinary query capabilities.
- Parameterize all data values; allowlist dynamic identifiers.
- Never expose extension loading or unrestricted SQL to agents.
- Enforce authorization in repositories and again at retrieval boundaries.

SQLite does not provide PostgreSQL-style per-user server roles. Local security
therefore also depends on process isolation, filesystem permissions, and
application authorization.

## Operational Implications

Benefits:

- No database service for local use.
- Transactional provenance, jobs, and source management.
- Straightforward offline backup and restore.
- A clear path to PostgreSQL.

Costs:

- Writes must be scheduled around a single writer.
- Long readers can delay WAL checkpointing.
- Two backend implementations require explicit compatibility tests.
- Triggers and migrations require careful operational review.

## Migration Strategy

For SQLite-to-PostgreSQL migration:

1. Enter maintenance mode or otherwise freeze authoritative writes.
2. Take and verify a recoverable backup.
3. Create the PostgreSQL schema using PostgreSQL migrations.
4. Transfer authoritative rows while preserving IDs, versions, and hashes.
5. Validate counts, constraints, canonical hashes, token order, and citations.
6. Transfer projection manifests and rebuild or validate derived stores.
7. Switch the configured relational adapter.
8. Retain the original SQLite backup for rollback.

Do not use uncontrolled dual writes during migration.

## Reversal Cost

Medium.

Repository boundaries reduce application changes, but schema conversion,
transaction semantics, triggers, migrations, and operational tooling still
require work. Stable domain identifiers and source hashes must survive any
replacement.

## Acceptance Criteria

- A fresh local installation starts without a database daemon.
- All pooled connections enforce foreign keys.
- Protected canonical updates and deletes fail.
- An authoritative write and its outbox event commit or roll back together.
- Migration and backup/restore tests preserve canonical hashes and addressing.
- Domain crates do not depend on sqlx or concrete database adapters.
- Storage tests cover contention, interruption, rollback, and disk failures.
- Portable repository behavior is tested against PostgreSQL before the
  PostgreSQL adapter is declared supported.
