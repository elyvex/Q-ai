# Feature Specification: storage-sqlite backend

**Feature Branch**: `004-storage-sqlite`

**Created**: 2026-09-17

**Status**: Draft

**Input**: Module-level specification of the implemented `storage-sqlite` crate (crates/storage-sqlite/src/{lib,migrate,quran}.rs) plus the `QAI-DB-` error codes it maps (defined in `storage`, crates/storage/src/error.rs), derived from code truth on 2026-09-17. Reverse specification of what exists today. Code is the source of truth over plan docs.

**Constitution compliance**: `.specify/memory/constitution.md` v1.0.2, Principles I (insert-only canonical tables, forward-only deactivation), III (reproducibility via generations/checksums), V (gates), VII (SQLite + append-only checksummed migrations, `VACUUM INTO` backups).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Durable SQLite persistence behind traits (Priority: P1)

Application code talks to `Database`/`ReadTx`/`UnitOfWork` traits (defined in `storage`); this crate provides the real SQLite implementation with a write pool (`max_connections=1`), a read-only pool (`query_only`), WAL, `foreign_keys=ON`, and busy timeouts — so concurrent readers never block the single writer and vice versa.

**Why this priority**: Every other subsystem (sources, jobs, audit, quran corpus, search cache) persists through this backend. Wrong pooling or missing FK enforcement corrupts everything downstream.

**Independent Test**: `cargo test -p storage-sqlite --lib` green (10 passed on 2026-09-17) plus 13 integration suites (`crates/storage-sqlite/tests/`); round-trips cover jobs enqueue/claim, source versions, settings upsert, audit append+verify.

**Acceptance Scenarios**:

1. **Given** a fresh data dir, **When** migrations apply and a job is enqueued then claimed, **Then** claim/lease semantics hold and the round trip is observable in tests.
2. **Given** a path that does not exist, **When** opened via `open_read_only`, **Then** no file is created (read-only open never creates).

---

### User Story 2 - Append-only checksummed migrations (Priority: P1)

Schema evolves only through `migrations/sqlite/NNNN_*.up.sql` files with SHA-256 checksums recorded in `checksums.json`. Applying is idempotent; editing an applied migration is detected as drift and fails; rollback uses the paired `.down.sql`.

**Why this priority**: Constitution Principle VII / invariant: never edit an applied migration. Silent schema drift would invalidate all hash-chained evidence.

**Independent Test**: `fresh_migrate_is_idempotent`, `checksum_drift_is_detected`, `down_migrations_restore_schema` green; `cargo run -p xtask -- migrate-check` OK (16 migrations ordered, checksums stable).

**Acceptance Scenarios**:

1. **Given** a database migrated to v16, **When** migrations are applied again, **Then** zero migrations run and the call succeeds (idempotent).
2. **Given** an applied migration file modified on disk, **When** checksums are verified, **Then** verification fails with `QAI-DB-0007` (checksum mismatch).

---

### User Story 3 - Safe backups (Priority: P2)

Operators back up a live database with `VACUUM INTO`, producing a consistent snapshot without copying a live journal — never a raw filesystem copy.

**Why this priority**: `fs::copy` of a live SQLite file yields corrupt backups. `VACUUM INTO` is the only supported path (ADR-0001).

**Independent Test**: `backup_round_trips_a_populated_db` green; `qai db backup` smoke works on a temp `QAI_DATA_DIR`.

**Acceptance Scenarios**:

1. **Given** a populated database, **When** backed up via `backup()`, **Then** the destination opens cleanly with identical row counts.

### Edge Cases

- `STORAGE_BUSY` (`QAI-DB-0005`) surfaces lock contention instead of hanging forever (busy timeout configured).
- `MIGRATION_REQUIRED` (`QAI-DB-0006`) when code meets an older schema; opening never auto-migrates silently — migration is an explicit step.
- `IMMUTABLE_SOURCE_VERSION` (`QAI-DB-0003`) when a write targets append-only canonical/versioned rows.
- `IDEMPOTENCY_KEY_REPLAY` (`QAI-DB-0008`) on job/outbox double-submit with the same key.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Crate MUST provide `SqliteDatabase` with dual pools (write `max_connections=1`; read `query_only`), `new()` (read-write), `open_read_only()` (never creates), `read()` → `ReadTx`, `write()` → `UnitOfWork`, `health()` → `DbHealth`, `schema_version()`, `backend()`, plus `probe_scalar`/`count` diagnostics (`crates/storage-sqlite/src/lib.rs` L52–190).
- **FR-002**: Crate MUST implement all seven repository families in one `SqliteUnitOfWork` transaction: sources (incl. versions, transitions, approvals, principals), provenance (+ review queue), audit (append + hash-chain verify), jobs (enqueue/claim/claim_next/reschedule/finish/cancel/checkpoint/heartbeat/lease-reap), settings, outbox (generations/allocators/events/tombstones), quran (editions, structure, divisions, translations, staging, validation, forms, indexes, search cache) (`lib.rs` L263–299, `quran.rs`).
- **FR-003**: Crate MUST run migrations via `migrate::{discover_migrations, apply_migrations, verify_checksums, revert_last_migration}` over `migrations/sqlite/` with `schema_migrations` ledger + `checksums.json` comparison (`crates/storage-sqlite/src/migrate.rs` L29–262).
- **FR-004**: Crate MUST back up exclusively via `VACUUM INTO` (`migrate::backup`, L298–319); raw file copies are not a supported path.
- **FR-005**: Crate MUST map `sqlx::Error` to `StorageError` (`map_sqlx_error`), preserving the `QAI-DB-nnnn` codes defined in `storage` (`crates/storage/src/error.rs` L11–28).
- **FR-006**: Crate MUST hide behind the `application` boundary (no direct `storage-sqlite` dependency from `cli`/`server`); enforced by `arch-check`.

### Key Entities

- **SqliteDatabase / SqliteReadTx / SqliteUnitOfWork**: Connection management, read transactions, and multi-repository write transactions with commit/rollback.
- ***Row types**: `SourceRow`, `SourceVersionRow`, `ProvenanceRecord`, `AuditEvent`, `JobRecord`, `SettingRow`, `GenerationRow`, `OutboxEventRow`, `TombstoneRow`, plus the full Quran family (`QuranEditionRow`, `AyahRow`, `TokenRow`, `DivisionRow`, `TranslationPassageRow`, `TokenFormRow`, `SearchCacheRow`, … per `storage/src/quran.rs` + `repository.rs`).
- **MigrationFile / ChecksumReport**: Discovered `NNNN_name.up.sql` units and drift-verification reports.

### Error Surface

`QAI-DB-0001` conflict · `0002` not found · `0003` immutable source version (editing applied/canonical rows) · `0004` constraint violation · `0005` storage busy · `0006` migration required · `0007` migration checksum mismatch · `0008` idempotency-key replay · `0009` storage unavailable (`crates/storage/src/error.rs` L14–28; `StorageError::code()`).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo test -p storage-sqlite --lib` passes with zero failures (10 passed on 2026-09-17) and all 13 integration suites pass.
- **SC-002**: `cargo run -p xtask -- migrate-check` reports 16 ordered migrations with stable checksums.
- **SC-003**: Modifying any applied migration file is detected by `verify_checksums` in 100% of cases (drift test green).
- **SC-004**: Backup of a populated DB restores with identical row counts (round-trip test green).

## Assumptions

- SQLite with WAL, `foreign_keys=ON`, busy timeout per ADR-0001; PostgreSQL-portable SQL discipline so a future port stays feasible.
- Canonical tables are insert-only via triggers and forward-only (deactivation rows, never deletion); every derived/cache row records its `corpus_generation`.
- Schema is at v16 (`0001_core`…`0016_quran_search_cache`); plan tables showing `0010–0015`/`0020–0025` numbering are stale — allocate new numbers from `migrations/sqlite/`, never from plan docs.
- No model/vector dependencies in this crate (invariant I2); FTS5 indexes live here as SQL only, served through `quran-search` semantics.
