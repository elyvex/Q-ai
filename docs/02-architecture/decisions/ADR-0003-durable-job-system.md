# ADR-0003 — Durable Job System: DB-Backed Leased Queue (No External Broker)

- Status: Accepted
- Phase: 0 — Foundations
- Date: 2026-09-11
- Related decisions: ADR-0001, ADR-0009
- Requirements: PRD §§41, 54, 76, 85

## Context

Q-ai must run long-running background work (corpus import, integrity scans,
audit-chain verification, vacuum) reliably across power loss, process crashes,
and machine restarts. A local-first install cannot assume a message broker, and
adding one (RabbitMQ, Kafka) contradicts the zero-config local-first goal.

The queue must support idempotent processing, cancellation, checkpoint/resume,
prioritization, and crash recovery without any external dependency.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| **DB-backed leased queue (SQLite)** | No broker to install; transactions give atomicity; crash recovery via lease expiry; works identically on local and server | Single-writer bottleneck; throughput capped by SQLite |
| Redis-backed queue (BRPOP/XPENDING) | Low latency, native priority, atomic claims | Requires a Redis process; not local-first |
| Dedicated broker (RabbitMQ/Kafka) | High throughput, DLQ semantics, horizontal scale | Operational overhead, separate failure domain, violates zero-config local-first |
| File-based queue (e.g. sled/actix-web-mp) | Embedded; no server | No transactional coupling to the authoritative store; recovery semantics weaker |

## Decision

Use a **DB-backed leased queue** in SQLite via the `jobs` crate. Jobs, leases,
heartbeats, checkpoints, and cancellation all go through the same write pool
serialized by the single connection. No external broker exists in local mode.

Key design elements:

- **Lease-based claiming**: a worker sets `lease_owner` + `lease_expires_at`
  atomically in a `SELECT … FOR UPDATE` style claim (SQLite `UPDATE … WHERE
  state = 'Queued'` returning rows). Expired leases are reclaimed on startup scan
  (`Running` → `Interrupted`).
- **Idempotency**: `UNIQUE(kind, idempotency_key)`; handlers declare
  `is_idempotent()` — non-idempotent handlers are never auto-retried.
- **Checkpoint/resume**: `checkpoint_json` is replayed to the handler on resume.
- **Cancellation**: `cancel_requested` flag polled by handlers via
  `CancellationToken` inside `JobContext`.
- **State machine**: `Queued → Leased → Running → Checkpointed → Succeeded |
  Failed | Cancelled | Interrupted | DeadLettered`.
- **Outbox coupling**: every job that changes projection-relevant state must
  also write an outbox row (ADR-0702 §2) in the same transaction.

## Accuracy and Religious-Source Implications

- None directly. However, job failures during canonical text processing must not
  leave partial results visible; the checkpoint mechanism and atomic activation
  rules (§85) prevent half-applied canonical changes from becoming visible to
  downstream RAG or search.
- Dead-lettered jobs that touched canonical rows are surfaced to `qai doctor`
  for manual review before any canonical reprocessing.

## Licensing Implications

None beyond the Rust workspace license policy (MIT/Apache-2.0). The queue uses
only `sqlx`, `tokio`, and `uuid` from the workspace allowlist.

## Security Implications

- Job payloads are validated against per-kind JSON Schema before execution,
  preventing injection through the queue.
- `created_by: PrincipalId` is always recorded so audit trails identify which
  principal enqueued each job.
- Cancel/cancel-all endpoints require explicit `canonical:cancel` permission
  (deny-by-default).

## Operational Implications

- Workers are horizontally scalable in server mode (multiple workers can claim
  leases concurrently — SQLite serializes writes).
- `qai job list`, `show`, `cancel`, `retry` cover 90% of day-to-day ops.
- The `system.outbox_relay` job reuses the same lease mechanism for outbox
  dispatch.

## Migration Strategy

Zero-data migration on startup: the `jobs` and `job_events` tables are created
by `0004_jobs.up.sql` and populated only by enqueue operations after that.

## Reversal Cost

Medium. The schema is embedded; removing it requires a migration that drops
tables. Running job IDs are stable UUIDs; losing the job history is acceptable
with `qai db backup` but would be lost on a fresh DB rebuild.

## Acceptance Criteria

- Duplicate enqueue with the same `(kind, idempotency_key)` yields exactly one execution.
- `SIGKILL`-ing a worker → job becomes `Interrupted` → resumes from checkpoint without repeating completed stages.
- `qai job cancel` stops a long-running job within 2 seconds.
- `cargo xtask ci` green with the chaos test suite (kill worker, expire lease, duplicate enqueue).
