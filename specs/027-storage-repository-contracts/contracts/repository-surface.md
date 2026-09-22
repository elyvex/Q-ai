# Contract: Repository Trait Surface

**Feature**: `027-storage-repository-contracts` | **Date**: 2026-09-18

Method inventory per trait family. Behavior specified in [spec.md](../spec.md) FR-001–FR-014; row shapes in [data-model.md](../data-model.md). Implementation (SQL, pools, transactions) is specified in `specs/004-storage-sqlite/spec.md` and MUST NOT be duplicated here.

## `Database` / `ReadTx` / `UnitOfWork` (`storage/src/lib.rs`)

- `Database`: `read()`, `write()`, `health()`, `schema_version()`, `backend()`
- `ReadTx`: `schema_version()`, `query_one`, `query_list`
- `UnitOfWork`: `sources()`, `provenance()`, `audit()`, `jobs()`, `settings()`, `outbox()`, `quran()`, `commit()`, `rollback()`

## `SourceRepository` / `ProvenanceRepository` / `AuditRepository` / `SettingsRepository`

- Sources: `insert_source/version`, `transition_state(from, to)` (guarded), `list_versions`, `record_transition`, `insert_approval`/`get_approval`, `upsert_principal`
- Provenance: insert/get/`list_by_subject`, `record_review`
- Audit: `append` (append-only), `list_by_subject`, `list_by_sequence`, `latest_sequence`, `verify_chain`
- Settings: `get`/`set` (origin + actor), `list`

## `JobRepository` / `OutboxRepository`

- Jobs: `enqueue`, `claim`, `claim_next`, `reschedule`, `get`, `heartbeat`, `count_by_state`, `finish`, `cancel`, `checkpoint`, `reap_expired_leases`
- Outbox: `allocate_generation` (monotonic per scope) + `current_generation`, `enqueue` (idempotency-keyed), `claim_pending`, `mark_dispatched`, `mark_failed`, `list_by_state`, `insert_tombstone`, `list_pending_tombstones`

## `QuranRepository` (canonical/staging/reads + attributed/derived datasets)

- Staging (run-scoped): `insert_import_run`, `insert_stg_edition/surah/ayah/token/separator/division`, `count_stg_ayahs`, `list_stg_*` (deterministic order), `count_staging_orphans`, `clear_staging`, `delete_import_run`
- Canonical mutators (only): `activate_edition`, `rollback_edition`, `set_edition_status`
- Reads: `get_edition`, `get_active`, `get_ayah`, `get_ayah_by_global`, `list_ayahs_range`, `get_tokens`, `get_separators`, `list_divisions`, `count_ayahs/tokens`
- Attributed/derived (never canonical): validation/difference reports, citations, translations, word glosses, normalization catalog, Layer-D forms/skeletons, index pointers/build runs, search cache

## Workflow helpers (`storage/src/workflows.rs`)

- `record_source_activation`, `record_source_deactivation` (tombstone-first), `record_provenance_write`, `relay_outbox_once` — all caller-`UnitOfWork`-scoped; Phase-0 relay records dispatch only
