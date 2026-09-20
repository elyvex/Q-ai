# Feature Specification: storage repository contracts

**Feature Branch**: `027-storage-repository-contracts`

**Created**: 2026-09-18

**Status**: Draft

**Input**: Reverse specification of the implemented `storage` crate contracts (`crates/storage/src/{lib,repository,quran,workflows,error}.rs`) plus the trait-split with `storage-sqlite`, derived from code truth on 2026-09-18. Companion to `specs/004-storage-sqlite/spec.md`: 004 covers the SQLite implementation (pools, migrations, backup); this spec covers the trait contracts the implementation must satisfy (repository families, row shapes, workflow ordering, error codes). Code is the source of truth over plan docs.

**Constitution compliance**: `.specify/memory/constitution.md` v1.2.0, Principle I (insert-only canonical tables, approval-gated activation, no LLM-generated canonical text), Principle II (Layer A–E separation, staging → validation → approval → activation → audit, no self-activating import; glosses/translations/derived forms never canonical), Principle VII (SQLite-adjacent contracts, `storage` → `domain` only, append-only checksummed migrations `0001`–`0016`, forward-only deactivation, FTS5 as SQL only).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Typed Send+Sync repository contracts behind one atomic UnitOfWork (Priority: P1)

Application and domain code talk only to async repository traits (`SourceRepository`, `ProvenanceRepository`, `AuditRepository`, `JobRepository`, `SettingsRepository`, `OutboxRepository`, `QuranRepository`) exposed through a single `UnitOfWork`, itself vended by the `Database` port (`read()` → `ReadTx`, `write()` → `UnitOfWork`, `health()`, `schema_version()`, `backend()`). All repo traits are `Send + Sync` via `async_trait`; every trait method ships a stub default returning `Err(StorageError::StorageUnavailable)` so the contract crate compiles and tests without a backend. All repositories participating in one `UnitOfWork` share one underlying connection/transaction, so a projection-relevant change and its outbox row commit or roll back together (ADR-0001 §6, ADR-0702 §3).

**Why this priority**: This is the seam every subsystem (sources, jobs, audit, Quran corpus, search cache) depends on. If the trait boundary leaks SQL/driver types or allows split-commit across repositories, the outbox invariant and backend portability (SQLite today, PostgreSQL later) collapse.

**Independent Test**: `cargo test -p storage --lib` green (row round-trip, backend display, health, error-code uniqueness, retryability); `cargo run -p xtask -- arch-check` confirms `storage` → `domain` only and no `sqlx` in domain/application-callable surface; a mock `UnitOfWork` can drive `workflows.rs` helpers without SQLite.

**Acceptance Scenarios**:

1. **Given** only the `storage` crate (no `storage-sqlite`), **When** application code is compiled against `Database`/`UnitOfWork`/repository traits, **Then** it builds with no `sqlx`, pool, or row types in its signatures.
2. **Given** a `UnitOfWork` with stub-default repositories, **When** any unimplemented repository method is called, **Then** it returns `QAI-DB-0009` (`StorageUnavailable`) and never panics or performs I/O.

---

### User Story 2 - Quran canonical/staging lifecycle with a single gated write path (Priority: P1)

Importers stage one edition into `quran_stg_*` mirrors keyed by import-run id (`insert_stg_edition/surah/ayah/token/separator/division`, `insert_import_run`, `count_stg_ayahs`, `list_stg_*` in deterministic order), then publish exclusively through `activate_edition` (staging → canonical move + active-pointer flip + `corpus_generation` bump + staging consume, atomically) or `rollback_edition` (pointer flip to an existing slug/version + generation bump, atomically), each recording the approving identity (`activated_by`, `approval_id`, `activated_at`). The trait exposes no row-level canonical insert; `set_edition_status` is the only human-gated lifecycle mutation and identity/hashes stay trigger-guarded. Reads (`get_edition`, `get_active`, `get_ayah`, `get_ayah_by_global`, `list_ayahs_range`, `get_tokens`, `get_separators`, `list_divisions`, `count_ayahs/tokens`) serve the active or pinned edition; translations, word glosses, validation/difference reports, citations, normalization catalog, derived Layer-D forms, index pointers/build runs, and the search-result cache are separate attributed/derived datasets that never overwrite canonical Arabic.

**Why this priority**: Constitution Principle I/II: canonical text is insert-only and never LLM-generated; a second write path would bypass approval, hashing, and audit. Staging-then-atomic-activation is the import safety gate.

**Independent Test**: Exercised against the SQLite backend by the Quran staging/activation suites (`storage-sqlite` integration tests) plus `qai quran import/activate` smoke; at contract level, a fake `QuranRepository` verifies the caller order (stage → validate → activate) and that no canonical-write method exists outside `activate_edition`/`rollback_edition`/`set_edition_status`.

**Acceptance Scenarios**:

1. **Given** a staged run, **When** `activate_edition(run_id, edition_id, activated_by, approval_id, activated_at)` commits, **Then** canonical rows appear, the active pointer flips, the generation bumps by exactly one, staging rows are consumed, and the approving identity is recorded — all atomically.
2. **Given** a previously active edition version, **When** `rollback_edition(slug, version, activated_by, approval_id, activated_at)` commits, **Then** the pointer returns to that version with a new generation and no historical text is rewritten.

---

### User Story 3 - Activation/deactivation workflows with atomic outbox, generation, and tombstone (Priority: P1)

The transaction-scoped helpers in `workflows.rs` make the outbox invariant impossible to violate by construction: `record_source_activation` (allocate generation → enqueue `source_activated` → `transition_state(Indexing → Active)`), `record_source_deactivation` (write tombstone `Pending` first → allocate generation → enqueue `source_deactivated` → `transition_state(Active → Deprecated)`), `record_provenance_write` (insert record → allocate generation → enqueue `provenance_written`), and `relay_outbox_once` (claim up to `limit` for `owner`, mark each dispatched, return relayed count) — all through the caller's `UnitOfWork` so the authoritative change and its outbox row live or die in one transaction. Deactivation writes the tombstone before the state transition (write-before-visibility, ADR-0702 §9); duplicate `(operation, idempotency_key)` enqueue yields `Conflict` (idempotent no-op at the call site); generations are monotonic per scope and never regress.

**Why this priority**: ADR-0001 §6 / ADR-0702 §3: every projection-relevant write must record its generation change and durable outbox event in the same commit. Ordering (tombstone-before-visibility) is what blocks retrieval immediately while physical cleanup is still pending.

**Independent Test**: Unit-testable with an in-memory fake `UnitOfWork`: assert call order and single-commit atomicity for each helper; assert `relay_outbox_once` claims then dispatches exactly the claimed set; Phase-0 relay has no consumers (no FTS/vector/graph) and only records dispatch (T64).

**Acceptance Scenarios**:

1. **Given** a version in `Indexing`, **When** `record_source_activation` runs and the unit of work commits, **Then** a new generation exists, exactly one `source_activated` outbox row exists with key `source_activated:{source_version_id}`, and the version reads `Active` — or none of the three is visible on rollback.
2. **Given** a version in `Active`, **When** `record_source_deactivation` runs, **Then** a `Pending` tombstone exists before the version reads `Deprecated`, a `source_deactivated` outbox row exists, and a reader blocked on the tombstone cannot observe the still-`Active` row without the tombstone being durable.

---

### User Story 4 - Jobs, audit, provenance, and settings lifecycle contracts (Priority: P2)

Background work runs through `JobRepository` (enqueue → `claim`/`claim_next` with lease owner + expiry → `heartbeat` renew → `checkpoint` progress → `finish`/`cancel`/`reschedule` with delay → `reap_expired_leases`); evidence runs through `AuditRepository` (append-only `append`, `list_by_subject`, `list_by_sequence`, `latest_sequence`, `verify_chain` with gap/hash reporting) and `ProvenanceRepository` (insert/get/`list_by_subject`, `record_review`); source governance runs through `SourceRepository` (`insert_source/version`, guarded `transition_state(from, to)`, `list_versions`, `record_transition`, `insert_approval`/`get_approval`, `upsert_principal`); runtime flags run through `SettingsRepository` (`get`/`set` with origin + actor/`list`). `ReadTx` stays read-only (`schema_version`, `query_one`, `query_list` with a stub-unavailable default); `application::db` catalog listings go through read-only opens and `json_object(...)` single-column reads, never through new write-trait methods.

**Why this priority**: Jobs/audit/provenance are the operability and trust backbone (leases prevent double-execution, hash chains make tampering evident, provenance separates Layer A–E). They are P2 only because nothing serves without Stories 1–3 first.

**Independent Test**: `storage-sqlite` integration round-trips (jobs enqueue/claim, audit append+verify, source versions, settings upsert) green; contract-level, a fake verifies lease semantics (`claim_next` only returns Queued/Interrupted/Checkpointed-due, `heartbeat` fails for a non-holder, `reap_expired_leases` returns exactly expired ids).

**Acceptance Scenarios**:

1. **Given** a queued job, **When** two workers race `claim_next` with different owners, **Then** exactly one owner holds the lease and the loser observes no job.
2. **Given** an audit log, **When** `verify_chain` runs, **Then** it reports valid plus expected next sequence/hash, or lists the exact gap sequences on tampering.

---

### User Story 5 - Stable QAI-DB error codes with consistent rendering (Priority: P3)

All nine failure modes carry stable `QAI-DB-nnnn` codes (`0001` Conflict, `0002` NotFound, `0003` ImmutableSourceVersion, `0004` ConstraintViolation, `0005` StorageBusy, `0006` MigrationRequired, `0007` MigrationChecksumMismatch, `0008` IdempotencyKeyReplay, `0009` StorageUnavailable) with `code()`, human `remedy()`, and a `Diagnostic` implementation (`summary`, `cause_chain`, `next_command` for migrate/verify/retry, `is_retryable` true only for Conflict/StorageBusy/StorageUnavailable, stable `render_human`/`render_json`). Backend driver errors never cross the trait boundary; adapters map them to these codes (`map_sqlx_error` in `storage-sqlite`, specified in 004).

**Why this priority**: Stable codes let CLI exit codes, logs, and JSON output stay consistent across backends (SQLite today, PostgreSQL later). It is P3 because it standardizes failures rather than enabling a new flow.

**Independent Test**: `all_codes_unique` and `retryable_variants` unit tests in `error.rs`; snapshot CLI `--output json` error objects for code/remedy/next-command stability.

**Acceptance Scenarios**:

1. **Given** any `StorageError`, **When** rendered via `Diagnostic::render_human` and `render_json`, **Then** both carry the same `QAI-DB-nnnn` code and the JSON object contains code/summary/why/location/remedy/next_command/retryable.
2. **Given** a duplicate `(operation, idempotency_key)` outbox enqueue or job double-submit, **When** the repository rejects it, **Then** the caller observes `Conflict`/`IdempotencyKeyReplay` and treats it as a safe idempotent no-op, not a new mutation.

---

### Edge Cases

- Unimplemented trait method called (stub default): returns `StorageUnavailable` (`QAI-DB-0009`), never panics, never touches I/O.
- `transition_state` from an unexpected `from` state: rejected (`Conflict`/`ConstraintViolation`), never silently applied.
- Duplicate outbox `(operation, idempotency_key)`: yields `Conflict`; call site treats as idempotent no-op.
- Outbox claim with zero pending or `limit = 0`: returns empty vec, marks nothing dispatched.
- `claim`/`heartbeat` by a non-holder: returns `None`/`false`; only the lease owner renews.
- `reap_expired_leases`: returns exactly expired lease ids; unexpired leases untouched.
- Tombstone write fails mid-deactivation: whole `UnitOfWork` rolls back, so no `Deprecated` state is visible without its tombstone.
- `activate_edition` on a missing run/edition or `rollback_edition` to an unknown slug/version: `NotFound`, pointer and generation unchanged.
- Seeded normalization rows rewritten via `insert_normalization_profile` with an existing key: rejected by the append-only trigger (`ImmutableSourceVersion`/`Conflict`), never overwritten.
- `cache_get` on corrupt payload JSON: caller treats as a miss (shape validated in `application`, not in `storage`); `cache_enforce_cap`/`cache_delete_stale` evict without read-modify-write races.
- `ReadTx::query_list` unimplemented: stub returns `StorageUnavailable`; `query_one` missing table/row returns `Ok(None)`, not an error.
- Backend lock contention: surfaces `StorageBusy` (`QAI-DB-0005`) with bounded retry, never hangs forever.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `storage` crate MUST define only contracts (traits, row types, `StorageError`, `DbHealth`/`DbBackend`, workflow helpers) and MUST depend only on `domain` (plus `serde`, `thiserror`, `async-trait`); concrete pools, transactions, SQL rows, and driver errors MUST remain inside backend crates (`crates/storage/src/lib.rs` L1–26; enforced by `cargo xtask arch-check`).
- **FR-002**: All seven repository traits (`SourceRepository`, `ProvenanceRepository`, `AuditRepository`, `JobRepository`, `SettingsRepository`, `OutboxRepository`, `QuranRepository`) MUST be `Send + Sync` `async_trait`s whose every method defaults to `Err(StorageError::StorageUnavailable)` (`repository.rs`, `quran.rs`).
- **FR-003**: `Database` port MUST expose `read() → ReadTx`, `write() → UnitOfWork`, `health() → DbHealth`, `schema_version()`, `backend()` (`SQLite` default, `Postgres` for Phase 12+); `ReadTx: Send` MUST expose `schema_version()`, `query_one`, `query_list` (stub-unavailable default) (`lib.rs` L70–109).
- **FR-004**: `UnitOfWork: Send` MUST expose exactly the seven repository accessors (`sources`, `provenance`, `audit`, `jobs`, `settings`, `outbox`, `quran`) plus `commit(self: Box<Self>)` / `rollback(self: Box<Self>)`, and all participating repositories MUST share one underlying connection/transaction so writes commit atomically (`lib.rs` L110–146; ADR-0001 §§2, 6).
- **FR-005**: `QuranRepository` MUST expose the full canonical/staging/read surface with stringly-typed rows (editions, active pointer, surahs, ayahs, tokens, separators, divisions, `quran_stg_*` mirrors, import runs, validation/difference reports, citations, translations, word glosses, normalization catalog, Layer-D forms/skeletons, index pointers/build runs, search cache) and MUST expose no row-level canonical insert — the only canonical-write paths are `activate_edition` and `rollback_edition` (both recording `activated_by`/`approval_id`/`activated_at` and returning the new generation), plus human-gated `set_edition_status` (`quran.rs` L8–12, L432–979).
- **FR-006**: Staging writes MUST be run-scoped (`insert_stg_*` take `run_id`), staged reads MUST return deterministic order (`list_stg_surahs`/`list_stg_ayahs` by number/`(surah, ayah)`, tokens by position, separators by `after_position`), and lifecycle helpers (`count_stg_ayahs`, `count_staging_orphans`, `clear_staging` keeping the run record, `delete_import_run` cascading) MUST hold (`quran.rs` L443–554).
- **FR-007**: `record_source_activation` MUST allocate a generation for `scope`, enqueue `source_activated` with idempotency key `source_activated:{source_version_id}`, then `transition_state(source_version_id, "Indexing", "Active")` through the same `UnitOfWork`, returning the generation id (`workflows.rs` L33–48).
- **FR-008**: `record_source_deactivation` MUST insert the tombstone (`propagation_state "Pending"`, reason `"deactivated"`, RFC 3339 `effective_at`) before allocating the generation and enqueueing `source_deactivated`, then `transition_state(source_version_id, "Active", "Deprecated")` through the same `UnitOfWork`, returning the tombstone id (`workflows.rs` L53–87; ADR-0702 §9).
- **FR-009**: `record_provenance_write` MUST insert the record, then allocate a generation and enqueue `provenance_written` (idempotency key bound to the generation id) through the same `UnitOfWork` (`workflows.rs` L91–110).
- **FR-010**: `relay_outbox_once` MUST claim up to `limit` pending events for `owner` and mark each claimed event dispatched, returning the relayed count; with no consumers in Phase 0 it MUST record dispatch only, never fabricate delivery (`workflows.rs` L112–129).
- **FR-011**: `OutboxRepository` MUST provide monotonic per-scope `allocate_generation` (never regresses) + `current_generation`, `enqueue` (duplicate `(operation, idempotency_key)` → `Conflict`), lease-based `claim_pending`/`mark_dispatched`/`mark_failed`/`list_by_state`, and `insert_tombstone`/`list_pending_tombstones` (`repository.rs` L421–536).
- **FR-012**: `JobRepository` MUST provide `enqueue`/`claim`/`claim_next` (Queued/Interrupted/Checkpointed-due only)/`reschedule`/`get`/`heartbeat` (owner-held only)/`count_by_state`/`finish`/`cancel`/`checkpoint`/`reap_expired_leases`; `AuditRepository` MUST provide append-only `append` + `list_by_subject`/`list_by_sequence`/`latest_sequence`/`verify_chain`; `SourceRepository` MUST guard `transition_state(source_version_id, from, to)` on expected-`from` (`repository.rs` L24–77, L149–361).
- **FR-013**: `StorageError` MUST carry exactly the nine `QAI-DB-0001…0009` codes with `code()`, human `remedy()`, and the `Diagnostic` surface (`summary`, `cause_chain`, `next_command` for migrate/verify/retry, `is_retryable` true only for Conflict/StorageBusy/StorageUnavailable, `render_human`/`render_json`) (`error.rs` L11–108, L146–298).
- **FR-014**: Trait/implementation split MUST hold: `storage-sqlite` implements all seven families in one shared-`Transaction` `UnitOfWork`; `application::db` keeps `storage-sqlite` behind the application boundary (CLI depends on `application`, never directly on `storage-sqlite`); unrestricted SQL MUST NOT cross the trait boundary into domain ports or agent tools (`storage-sqlite/src/lib.rs` L1–41; `application/src/db.rs` L1–16; ADR-0001 §§2–3).

### Key Entities

- **Database / ReadTx / UnitOfWork**: Capability ports — connection-independent access, read-only handle, and the atomic multi-repository write transaction (`lib.rs` L70–146); `DbBackend` (`SQLite`, `Postgres`), `DbHealth` (healthy/backend/schema_version/message).
- **Source family**: `SourceRow`, `SourceVersionRow`, `StateTransitionRow`, `PrincipalRow`, `ApprovalRow` — catalog, version lifecycle, transition events, interim principals, human approvals (`repository.rs` L79–141).
- **Provenance / Review**: `ProvenanceRecord` (layer, subject URN, attribution, trust, verification, confidence), `ReviewRecord` (queue, evidence, state, decision) (`repository.rs` L175–203).
- **Audit chain**: `AuditEvent` (sequence, actor, action, subject, outcome, before/after, chain hashes), `ChainVerificationResult` (valid, expected next sequence/hash, gaps) (`repository.rs` L242–268).
- **Jobs**: `JobRecord` (kind, payload, idempotency key, state, priority, attempts, lease owner/expiry, checkpoint, cancel flag) (`repository.rs` L363–380).
- **Outbox / generations / tombstones**: `GenerationRow` (scope, monotonic number, reason), `NewOutboxEvent` + `OutboxEventRow` (scope, target generation, operation, subject URN, idempotency key, state, lease, attempts), `TombstoneRow` (subject URN, reason, effective_at, propagation_state) (`repository.rs` L488–536).
- **Quran corpus**: `QuranEditionRow` (identity, script/riwayah/qiraah, hashes, status), `ActiveEditionRow` (edition pointer + corpus generation + approving identity), `SurahRow`/`AyahRow`/`TokenRow`/`SeparatorRow`/`DivisionRow`, `StagedEditionRef`, `ImportRunRow`, `ValidationReportRow`, `DifferenceReportRow`, `CitationRow`, `TranslationEditionRow`/`TranslationPassageRow`, `WordGlossRow` (attributed, never canonical), `NormalizationRuleRow`/`NormalizationProfileRow` (append-only catalog), `TokenFormRow`/`AyahFormRow`/`SkeletonRow` (Layer-D derived), `IndexPointerRow`/`IndexBuildRunRow` (mutable pointer + build lifecycle), `SearchCacheRow` (keyed payload + generation + LRU bytes) (`quran.rs` L20–420, L981–1000).
- **Settings**: `SettingRow` (key, value JSON, origin, updated_at/by) (`repository.rs` L411–419).
- **StorageError + Diagnostic**: Nine `QAI-DB-nnnn` failures with codes, remedies, retryability, and human/JSON rendering (`error.rs` L25–298).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Any conforming backend can be swapped behind `Database`/`UnitOfWork` without changing application call sites — demonstrated by `application::db` compiling against traits only and `arch-check` reporting no `sqlx`/adapter leakage into domain or application signatures.
- **SC-002**: Every projection-relevant write commits its authoritative change and outbox row atomically — verified by fake-`UnitOfWork` tests where a mid-workflow failure leaves zero partial state (no generation without its event, no `Deprecated` without its tombstone).
- **SC-003**: Staged editions activate or roll back exactly once per approval — activation flips the pointer, bumps the generation by one, consumes staging, and records the approving identity, with no second canonical-write path observable in the trait surface.
- **SC-004**: All nine error codes render identically across backends — each code round-trips through `code()`, `render_human`, and `render_json` with its documented remedy and retryability, and duplicate idempotent submits are safe no-ops rather than duplicate mutations.
- **SC-005**: This spec stays disjoint from 004 — every contract statement here names a `storage`-crate trait/workflow/error item, while every implementation statement (pools, pragmas, SQL, migration files, `VACUUM INTO`) remains in `specs/004-storage-sqlite/spec.md`.

## Assumptions

- Code on 2026-09-18 is truth: `crates/storage/src/{lib.rs,repository.rs,quran.rs,workflows.rs,error.rs}` as read; `storage-sqlite` skimmed (`lib.rs` dual pools + shared-transaction `UnitOfWork`, `migrate.rs` discover/apply/verify/backup); `application/src/db.rs` skimmed for trait-boundary usage (read-only opens, `json_object` catalog reads); ADRs skimmed (ADR-0001 relational store + outbox invariant; ADR-0006 SHA-256/NFC canonical hashing, which row `*_hash` string fields are presumed to carry as lowercase hex).
- Gap recorded: `specs/004-storage-sqlite/spec.md` (2026-09-17) specifies the SQLite implementation only; it does not specify the `Send + Sync` trait contracts, row shapes, workflow orderings, or error-code semantics — this spec fills exactly that gap and duplicates no 004 implementation detail.
- Stringly-typed rows are intentional (Phase-0 convention: storage moves bytes, typed domain mapping lives above); `Option`/nullable and `i64`/`u64` shapes follow the code as-is.
- Timestamps are RFC 3339 UTC strings; hash fields are opaque strings at this layer (algorithm/normalization policy owned by `domain`/ADR-0006, not re-specified here).
- `relay_outbox_once` records dispatch only in Phase 0 (no FTS/vector/graph consumers yet); lease durations, retry/backoff policy, and dead-letter thresholds are backend/operator concerns outside this contract spec.
- Constitution v1.2.0 §§I, II, VII apply (insert-only canonical + approval-gated activation, Layer A–E separation with staged attributed imports, `storage` → `domain` layering with append-only checksummed migrations); no open clarifications — informed guesses above stand as assumptions.
