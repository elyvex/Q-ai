# Data Model: Storage Repository Contracts

**Feature**: `027-storage-repository-contracts` | **Date**: 2026-09-18

Documents the existing row/trait shapes (spec Key Entities, verified against code). No new persisted state — this model pins what the contracts carry so conformance tests can assert it.

## Entity 1 — Capability ports (`storage/src/lib.rs`)

| Item | Shape |
|---|---|
| `DbBackend` | enum `SQLite` (default) / `Postgres` (Phase 12+) |
| `DbHealth` | `{healthy, backend, schema_version, message}` via `ok()` / `fail()` constructors |
| `Database: Send + Sync` | `read() → ReadTx`, `write() → UnitOfWork`, `health() → DbHealth`, `schema_version()`, `backend()` |
| `ReadTx: Send` | `schema_version()`, `query_one`, `query_list` (stub-unavailable default); read-only by construction |
| `UnitOfWork: Send` | Seven accessors (`sources`, `provenance`, `audit`, `jobs`, `settings`, `outbox`, `quran`) + `commit(self: Box<Self>)` / `rollback(self: Box<Self>)`; all repos share one connection/transaction |

## Entity 2 — Source governance rows (`repository.rs`)

`SourceRow`, `SourceVersionRow` (lifecycle states incl. `Indexing`/`Active`/`Deprecated`), `StateTransitionRow` (transition events), `PrincipalRow` (interim principals), `ApprovalRow` (human approvals). `transition_state(id, from, to)` rejects unexpected-`from`.

## Entity 3 — Provenance / review rows

`ProvenanceRecord` (layer A–E, subject URN, attribution, trust, verification, confidence); `ReviewRecord` (queue, evidence, state, decision). Methods: insert/get/`list_by_subject`, `record_review`.

## Entity 4 — Audit chain rows

`AuditEvent` (sequence, actor, action, subject, outcome, before/after, chain hashes); `ChainVerificationResult` (`valid`, expected next sequence/hash, `gaps`). Methods: append-only `append`, `list_by_subject`, `list_by_sequence`, `latest_sequence`, `verify_chain`.

## Entity 5 — Job rows

`JobRecord` (kind, payload, idempotency key, state, priority, attempts, lease owner/expiry, checkpoint, cancel flag). Lease methods: `claim`/`claim_next` (Queued/Interrupted/Checkpointed-due only), `heartbeat` (owner-held only), `reap_expired_leases` (exactly expired ids).

## Entity 6 — Outbox / generation / tombstone rows

`GenerationRow` (scope, monotonic number — never regresses, reason); `NewOutboxEvent` + `OutboxEventRow` (scope, target generation, operation, subject URN, idempotency key, state, lease, attempts; duplicate `(operation, key)` → `Conflict`); `TombstoneRow` (subject URN, reason, RFC 3339 `effective_at`, `propagation_state`, `Pending` written before visibility change).

## Entity 7 — Quran corpus rows (`quran.rs`)

`QuranEditionRow` (identity, script/riwayah/qiraah, hashes, status); `ActiveEditionRow` (pointer + generation + approving identity); `SurahRow`/`AyahRow`/`TokenRow`/`SeparatorRow`/`DivisionRow`; run-scoped `quran_stg_*` mirrors + `ImportRunRow`; `ValidationReportRow`/`DifferenceReportRow`; `CitationRow`; `TranslationEditionRow`/`TranslationPassageRow`, `WordGlossRow` (attributed, never canonical); `NormalizationRuleRow`/`NormalizationProfileRow` (append-only); `TokenFormRow`/`AyahFormRow`/`SkeletonRow` (Layer-D derived); `IndexPointerRow`/`IndexBuildRunRow`; `SearchCacheRow` (keyed payload + generation + LRU bytes).

Canonical mutators (only): `activate_edition(run_id, edition_id, activated_by, approval_id, activated_at) → generation`; `rollback_edition(slug, version, …) → generation`; human-gated `set_edition_status`.

## Entity 8 — Settings rows

`SettingRow` (key, value JSON, origin, updated_at/by); `get`/`set`/`list`.

## Entity 9 — `StorageError` + Diagnostic (`error.rs`)

Nine codes `QAI-DB-0001…0009` (see [contracts/error-codes.md](contracts/error-codes.md)); `code()`, human `remedy()`, `Diagnostic` (`summary`, `cause_chain`, `next_command`, `is_retryable`), `render_human`/`render_json` carrying identical code.

## Validation rules (cross-entity)

- Stub default on any unimplemented method → `StorageUnavailable`, never panic/I/O.
- `transition_state` unexpected-`from` → `Conflict`/`ConstraintViolation`, never silent.
- Seeded normalization rows rewritten with existing key → append-only trigger rejects (`ImmutableSourceVersion`/`Conflict`).
- `cache_get` corrupt payload → caller-side miss; eviction without read-modify-write races.
- `query_one` missing row → `Ok(None)`; `query_list` unimplemented → stub `StorageUnavailable`.
- Lock contention → `StorageBusy` with bounded retry, never hang.
