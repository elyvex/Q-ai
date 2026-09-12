# ADR-0002 — Migration Strategy: Append-Only Checksummed SQL, Forward-Only Canonical

- Status: Accepted
- Phase: 0 — Foundations
- Date: 2026-09-11
- Related decisions: ADR-0001, ADR-0009
- Requirements: PRD §§6–7, 25.12, 73, 82, 92:43–46, QAI-DB-0003

## Context

The Q-ai platform requires a database migration framework to evolve the relational schema over time.
Because Q-ai operates in both single-user embedded local-first mode (SQLite) and server environments (eventually PostgreSQL), schema evolution must be reliable, reproducible, and verifiable.

Editing an applied migration file in production can result in silent drift, schema divergence, and corruption of the hash-chains used for audit logging and canonical text provenance.
Furthermore, the canonical Quran text and primary scripture sources are mathematically immutable; down-migrations that delete canonical text present a catastrophic data loss risk.

We need a migration engine that:
1. Validates that applied migration files are append-only and have not been edited (checksum matching).
2. Prevents rollback (down-migrations) of immutable canonical data, enforcing a forward-only policy for Layer A text.
3. Operates seamlessly on SQLite in local deployments.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| **sqlx built-in migrations** | Integrated with the driver, simple, handles basic checksums | Limited custom validation, hard to enforce custom forward-only policies or custom checksum hooks |
| **Custom Migration Runner** | Complete control over checksum verification, custom failure modes (remedy/next command), strict forward-only rules for canonical tables | Development effort, must handle schema locking and file parsing manually |
| **Liquibase / Flyway (Java-based)** | Mature, enterprise-grade, massive feature set | Requires JVM, slow, heavy, external dependency in Rust workspace |
| **Tern / Refinery** | Rust-native third-party migration tools | Adds another external crate; might not map error codes to the Q-ai `Diagnostic` standard easily |

## Decision

We will implement a **Custom Migration Runner** within the `storage-sqlite` crate (D0.7).
This runner will use standard SQL migration files (`.sql`) but wrap the execution in a validation engine that enforces strict append-only and checksum constraints.

Key design specifications:

### 1. Verification and Checksums
- Every migration file has an associated SHA-256 checksum recorded in the database `_qai_migrations` table upon execution.
- Before executing any pending migrations, the runner computes the SHA-256 of all previously applied migration files on disk and compares them with the database records.
- **Strict Failure (QAI-DB-0003):** If any applied migration file's checksum differs, or if an applied file is missing, the runner aborts immediately. It emits a `Diagnostic` error containing a clear remedy (restore from backup or revert file changes) and blocks all database connections.

### 2. Forward-Only Policy for Canonical Tables
- Structural/canonical schema tables (Layer A - e.g., surahs, ayahs, tokens) are marked as immutable at the migration layer.
- While schema definition changes (adding columns to annotations, creating index tables) may occur, deleting or modifying canonical schema elements is strictly forbidden.
- Down-migrations are disabled by default. Schema rollbacks are only allowed for development environments on non-canonical tables. Production migrations are strictly **forward-only**; corrections to canonical data must be applied as a new forward-running migration with a new version increment (PRD §7.3).

### 3. PostgreSQL Portability
- Schema definition files are partitioned by dialet. SQLite migrations live in `migrations/sqlite/`, and future PostgreSQL migrations will live in `migrations/postgres/`.
- The migration metadata table schema remains identical across backends to permit unified reporting by the `qai doctor` CLI.

## Accuracy and Religious-Source Implications

- Forward-only migrations prevent accidental truncation of Quranic or scriptural text.
- Re-running migrations on an already migrated database must be an idempotent, completely safe no-op.
- Any change to canonical database schemas must register an entry in the system audit log.

## Licensing Implications

- The custom runner relies only on standard Rust libraries (`sha2`, `serde`) and `sqlx` (MIT/Apache-2.0).

## Security Implications

- The migration metadata table `_qai_migrations` is protected by triggers or strict application-layer access controls.
- The migration runner executes with elevated privileges compared to normal query connections. Normal connections use a read-only pool or lack write access to `_qai_migrations`.

## Operational Implications

- Schema evolution is deterministic.
- Safe backup-and-restore hooks (using SQLite's `VACUUM INTO` or online backup API) are integrated so that a backup is taken *before* any migration is applied.

## Migration Strategy

When migrating from SQLite to PostgreSQL in production:
1. Ensure the SQLite schema is at the latest migration version.
2. Run the PostgreSQL migration suite up to the corresponding logical schema version.
3. Port tables preserving exact logical keys and version metadata.

## Reversal Cost

High.
Once the database schema evolves, reversing it requires restoring the pre-migration backup or writing a corrective forward migration.

## Acceptance Criteria

- `tests/db/migrations.rs` validates fresh migration, re-run idempotency, and checksum mismatch termination.
- Editing an applied migration file causes a hard, coded `Diagnostic` failure.
- Backup is automatically verified before schema modification.
