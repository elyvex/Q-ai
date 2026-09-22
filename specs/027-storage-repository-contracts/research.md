# Research: Storage Repository Contracts

**Feature**: `027-storage-repository-contracts` | **Date**: 2026-09-18

Reverse-spec planning: every claim below was verified against the tree (files read 2026-09-18). No `NEEDS CLARIFICATION` remains.

## Decision 1 — Trait surface: seven `Send + Sync` families behind `Database`/`ReadTx`/`UnitOfWork`

- **Decision**: The contract is `Database: Send + Sync` (`read() → ReadTx`, `write() → UnitOfWork`, `health()`, `schema_version()`, `backend()`), `ReadTx: Send` (read-only: `schema_version`, `query_one`, `query_list` with stub-unavailable default), `UnitOfWork: Send` (exactly seven accessors — sources, provenance, audit, jobs, settings, outbox, quran — plus `commit`/`rollback`), and seven `Send + Sync` `async_trait` repository traits, each method defaulting to `Err(StorageUnavailable)`.
- **Rationale**: Verified in code — `storage/src/lib.rs:29-116` (`DbBackend`, `DbHealth`, `Database`, `ReadTx`, `UnitOfWork`) and trait declarations at `repository.rs:24/150/211/277/388/429` + `quran.rs:432`. Stub defaults let the contract crate compile and test without a backend and make unimplemented calls fail closed with `QAI-DB-0009` instead of panicking.
- **Alternatives considered**: Documenting the SQLite implementation's concrete types as the contract — rejected: leaks driver types into application signatures and kills backend portability (SC-001). That implementation detail stays in spec 004.

## Decision 2 — Single gated canonical-write path; staging mirrors are run-scoped

- **Decision**: `QuranRepository` exposes `quran_stg_*` staging mirrors keyed by import-run id, deterministic staged reads, and exactly three canonical mutators (`activate_edition`, `rollback_edition`, human-gated `set_edition_status`) — no row-level canonical insert. Activation moves staging → canonical, flips the active pointer, bumps `corpus_generation` by one, consumes staging, and records approving identity, atomically.
- **Rationale**: Verified trait shape (`quran.rs:432-979` per spec pins). This is the code-level enforcement of constitution I/II (insert-only canonical, approval-gated activation, no self-activating import).
- **Alternatives considered**: A direct canonical-insert trait method for "simplicity" — rejected: would bypass approval, hashing, and audit; the absence of such a method is itself a contract property to pin with a negative test (fake verifies no canonical-write method exists outside the three).

## Decision 3 — Outbox atomicity by construction through caller-provided `UnitOfWork`

- **Decision**: The four `workflows.rs` helpers (`record_source_activation:29`, `record_source_deactivation:53`, `record_provenance_write:91`, `relay_outbox_once:118`) take the caller's `UnitOfWork`, so authoritative change + generation allocation + outbox row share one transaction. Ordering is fixed: activation allocates → enqueues (`source_activated:{source_version_id}`) → transitions `Indexing → Active`; deactivation writes the `Pending` tombstone first → allocates → enqueues → transitions `Active → Deprecated`.
- **Rationale**: Verified signatures; matches ADR-0001 §6 / ADR-0702 §3 (generation + outbox event in the same commit) and ADR-0702 §9 (tombstone-before-visibility). Unit-testable with an in-memory fake — no SQLite needed for ordering/atomicity proof.
- **Alternatives considered**: Helpers opening their own transactions — rejected: would allow split-commit (generation without event, `Deprecated` without tombstone), violating SC-002.

## Decision 4 — Nine stable error codes with closed rendering

- **Decision**: `QAI-DB-0001…0009` (Conflict, NotFound, ImmutableSourceVersion, ConstraintViolation, StorageBusy, MigrationRequired, MigrationChecksumMismatch, IdempotencyKeyReplay, StorageUnavailable) with `code()`, `remedy()`, `Diagnostic` (`summary`, `cause_chain`, `next_command`, `is_retryable` true only for 0001/0005/0009), `render_human`/`render_json`.
- **Rationale**: Verified constants at `error.rs:14-22` and the `QAI-DB-nnnn` namespace doc. Closed code set lets CLI exits, logs, and JSON stay consistent across backends; `map_sqlx_error` (004's side) keeps driver errors from crossing the boundary.
- **Alternatives considered**: Open-ended error strings — rejected: unmappable by CLI/server, breaks SC-004.

## Decision 5 — Disjointness rule with spec 004

- **Decision**: Every statement here names a `storage`-crate trait/workflow/error item; pools, pragmas, SQL text, migration files, and `VACUUM INTO` stay in `specs/004-storage-sqlite/spec.md` (SC-005). Conformance evidence may cite `storage-sqlite` test names without re-specifying implementation.
- **Rationale**: Prevents spec drift where two documents claim the same behavior. The seam is crisp: traits here, SQL there.
- **Alternatives considered**: Merging into 004 — rejected: 004 is dated 2026-09-17 and implementation-scoped; the contract layer deserves its own lifecycle.

## Open items

None. All Technical Context entries grounded in verified code.
