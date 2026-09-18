# Feature Specification: Durable Job System

**Feature Branch**: `026-durable-job-system`

**Created**: 2026-09-18

**Status**: Draft

**Input**: User description: "durable job system: module-level reverse specification of the implemented jobs crate (crates/jobs/src/lib.rs, queue.rs, worker.rs, registry.rs, ADR-0003, migrations 0004_jobs), derived from code truth. Document JobRecord/JobState/JobHandler/JobContext, InMemoryJobQueue claim/lease semantics, worker retry/cancel/idempotency, and open follow-ups in docs/05-followups/open-questions.md (P0-T39/T40 timestamp/plus-subsecond issues). Code is source of truth. Gap: no spec; status.md lists AC-P0-11/12 chaos/cancellation still open."

## User Scenarios & Testing *(mandatory)*

This is a reverse specification: it documents what the `jobs` crate does today (code truth as of 2026-09-18), not a proposal. Code is the source of truth over plan docs.

### User Story 1 - Run a background job to success (Priority: P1)

A developer enqueues a registered job kind (e.g. `system.noop_test`, `audit.verify_chain`) and a worker claims it, runs the handler, and marks it `Succeeded` with an optional result summary.

**Why this priority**: Every background workload (integrity scans, hashing, manifest validation, outbox relay) depends on enqueue → claim → run → finish working.

**Independent Test**: Can be fully tested by enqueueing a noop job into an empty queue, running one worker `run_once`, and observing `Succeeded` with the job record in a terminal state. Delivers a working durable-execution loop.

**Acceptance Scenarios**:

1. **Given** an empty queue and a registered handler, **When** a job is enqueued and the worker runs once, **Then** the outcome is `Succeeded` and the stored record reaches a terminal state.
2. **Given** a job whose `available_at` is in the future, **When** a worker claims, **Then** the job is not picked up until its scheduled time passes.

---

### User Story 2 - Retry with backoff, then dead-letter (Priority: P2)

A developer runs a job whose handler keeps failing; the worker reschedules it with growing delay and, after exhausting attempts, moves it to the dead-letter state instead of retrying forever.

**Why this priority**: Failure handling is what makes the queue durable rather than lossy; unbounded retry would wedge the system.

**Independent Test**: Can be fully tested by registering an always-failing handler with `max_attempts = N`, draining the queue, and observing `Retried` outcomes followed by a final `DeadLettered` outcome. Delivers bounded failure semantics.

**Acceptance Scenarios**:

1. **Given** a failing job with attempts remaining, **When** the worker processes it, **Then** the outcome is `Retried` with a recorded next delay and the job becomes claimable again after that delay.
2. **Given** a failing job with no attempts remaining, **When** the worker processes it, **Then** the outcome is `DeadLettered` and the job is not retried further.

---

### User Story 3 - Cancel a running job (Priority: P2)

An operator requests cancellation of a running job; the worker's watchdog observes the request and the job ends `Cancelled` rather than `Succeeded`.

**Why this priority**: Long jobs (scans, hashing) must be stoppable without killing the process.

**Independent Test**: Can be fully tested by flagging cancellation on a slow handler mid-run and observing a `Cancelled` outcome. Delivers cooperative stop.

**Acceptance Scenarios**:

1. **Given** a running job with cancellation requested, **When** the worker's check observes it, **Then** the job finishes as `Cancelled`.
2. **Given** a queued job that is cancelled before claim, **When** a worker later scans, **Then** the job never runs to success.

---

### User Story 4 - Recover from worker crash via lease expiry (Priority: P3)

After a worker crash, jobs left in `Running` with expired leases are reaped to `Interrupted` and become claimable again by a live worker.

**Why this priority**: Crash recovery is the durability guarantee; without it a crash leaks jobs permanently.

**Independent Test**: Can be fully tested by expiring a running job's lease, calling `recover_interrupted`, and observing the job reclaimed and re-run. Delivers at-least-once recovery.

**Acceptance Scenarios**:

1. **Given** a `Running` job whose lease has expired, **When** lease reaping runs, **Then** the job transitions to `Interrupted` and is returned as reclaimed.
2. **Given** an `Interrupted` job past its availability time, **When** a worker claims, **Then** the job is picked up for re-execution.

---

### User Story 5 - Unknown job kinds are contained, not dropped (Priority: P3)

A job arrives with a kind that has no registered handler; the worker marks it dead-lettered with the kind recorded instead of silently discarding it.

**Why this priority**: Silent drops hide deployment skew (producer newer than worker); explicit dead-letter makes it visible.

**Independent Test**: Can be fully tested by enqueueing an unregistered kind and observing an `UnknownKind` outcome. Delivers safe handling of version skew.

**Acceptance Scenarios**:

1. **Given** a queued job with an unregistered kind, **When** the worker processes it, **Then** the outcome names the unknown kind and the job does not run any handler.

---

### Edge Cases

- What happens when two workers race to claim the same job? Only one claim succeeds; the loser gets a claim failure, never double execution ownership.
- What happens when the same idempotency key is enqueued twice? The second enqueue is rejected as a replay.
- What happens when a handler records checkpoints and then crashes? The checkpoint value is persisted and available for resume on reclaim.
- What happens when a job payload does not match the handler's declared schema? The handler reports failure and the normal retry/dead-letter policy applies.
- What happens when `reschedule` is called with a sub-second delay? Current behavior truncates sub-second precision (known limitation, see Assumptions); zero-delay rescheduling is used by tests.
- What happens when cancellation timing races handler completion? Cancellation is cooperative and timing-dependent; exact cancellation latency is an open acceptance item (AC-P0-11/12).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST persist jobs with identity, kind, payload, idempotency key, state, priority, attempt counters, scheduling time, lease ownership/expiry, checkpoint data, and cancellation flag.
- **FR-002**: System MUST support the lifecycle states Queued, Leased, Running, Checkpointed, Succeeded, Failed, Cancelled, Interrupted, and DeadLettered, with only legal transitions permitted (enforced by storage CHECK constraints and the state machine).
- **FR-003**: System MUST offer claim-based leasing: a worker claims the next available job for a named owner with a lease duration; only Queued, Interrupted, and Checkpointed jobs past their availability time are claimable.
- **FR-004**: System MUST support lease heartbeat renewal by the holding owner and lease reaping that moves expired `Running` jobs to `Interrupted`.
- **FR-005**: System MUST support progress reporting and checkpoint persistence during execution, with the latest checkpoint readable after the run for resume.
- **FR-006**: System MUST retry failed jobs with exponential backoff (base, cap, jitter fraction) up to a maximum attempt count, then dead-letter them.
- **FR-007**: System MUST support cooperative cancellation: an external cancel request is observable by the handler via a pollable flag and a snapshot check, and a cancelled job ends `Cancelled`.
- **FR-008**: System MUST route each job kind to its registered handler; jobs with no registered handler MUST end explicitly (unknown-kind/dead-letter) rather than being silently dropped.
- **FR-009**: System MUST reject duplicate enqueue under the same idempotency key as a replay failure.
- **FR-010**: System MUST expose stable diagnostic codes for job failures (`QAI-JOB-0001` not-found through `QAI-JOB-0007` storage), marking storage and claim-conflict failures retryable.
- **FR-011**: System MUST ship the Phase-0 job-kind set (`system.integrity_scan`, `system.vacuum`, `system.noop_test`, `system.outbox_relay`, `source.validate_manifest`, `source.compute_hashes`, `audit.verify_chain`) with handlers registered by kind string.
- **FR-012**: System MUST record job lifecycle events durably (jobs plus job-event history; production backend: SQLite tables from migration `0004_jobs`).
- **FR-013**: Handlers MUST declare their kind string, a payload-shape schema, and whether they are idempotent (safe to retry).
- **FR-014**: The worker MUST recover interrupted (lease-expired) jobs before/while draining, and report how many were reclaimed.

### Key Entities *(include if feature involves data)*

- **JobRecord**: The durable unit of work — id, kind, payload, idempotency key, state, priority, attempts/max-attempts, availability time, lease owner/expiry, checkpoint data, cancellation flag, creator.
- **JobState**: Lifecycle position (Queued → Leased → Running → Checkpointed → Succeeded, with Failed/Cancelled/Interrupted/DeadLettered branches); only legal transitions allowed.
- **JobHandler**: Behavior contract per kind — kind string, payload schema, idempotency declaration, run logic receiving context plus payload and returning success/result.
- **JobContext**: Per-execution handle given to a handler — the job record, cancellation observation (snapshot plus shared flag), progress/checkpoint reporting, tracing span, lease deadline.
- **JobOutcome**: Handler result — success flag plus optional result summary.
- **JobQueue**: Scheduling contract — enqueue, claim-next with lease, heartbeat, checkpoint, finish, reschedule with delay, cancel-request check, fetch, reap-expired, pending count; two backends (in-memory for tests/deterministic runs, SQLite-backed for production).
- **Worker / WorkerConfig / WorkerOutcome**: Drain engine — lease, backoff base/max/jitter, max attempts, watchdog poll interval; per-job outcome Idle / Succeeded / Retried (with next delay) / DeadLettered / Cancelled / UnknownKind.
- **HandlerRegistry**: Kind-to-handler map; lookup, kind listing, emptiness check.
- **JobError**: Failure taxonomy with stable `QAI-JOB-nnnn` codes (not-found, invalid-state, idempotency replay, claim conflict, cancelled, max-attempts exceeded, storage).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer can enqueue a registered job and see it succeed through one worker pass in under a minute of wall-clock time in a local environment.
- **SC-002**: An always-failing job with a fixed attempt budget ends dead-lettered after exactly that many attempts — no silent drops, no infinite retry (verifiable by draining the queue and counting outcomes).
- **SC-003**: A cancellation requested mid-run stops the job and leaves it `Cancelled` in at least 9 of 10 trials under test conditions (cooperative timing acknowledged as an open acceptance item for production timing).
- **SC-004**: After a simulated worker crash (lease left expired), 100% of stranded `Running` jobs are reclaimed to a re-runnable state by the recovery pass.
- **SC-005**: Every job failure surfaces a stable `QAI-JOB-nnnn` code that identifies the failure class without exposing secrets.

## Assumptions

- This is a reverse specification of implemented behavior; where this text and the code disagree, the code (`crates/jobs/src/`, `crates/application/src/job_queue.rs`, `migrations/sqlite/0004_jobs*`) governs.
- The in-memory queue defines the scheduling contract used by tests; the production SQLite-backed queue is expected to honor the same contract, with wall-clock and concurrency behavior validated separately.
- Known limitations carried forward, not fixed by this spec: sub-second reschedule delays lose precision (timestamp helper truncates to whole seconds); production SQLite scheduling and wall-clock lease behavior need separate validation; lease-recovery retry policy needs separate validation (see `docs/05-followups/open-questions.md`, Jobs scheduling P0-T39/T40).
- Phase-0 exit items AC-P0-11/12 (real crash/lease chaos suite and cancellation timing) and the worker pool/chaos tests noted in `docs/06-progress/status.md` remain open and are out of scope for this spec.
- Handler payload schemas are JSON-shape declarations, not a separate schema-registry service.
- The `system.noop_test` kind exists to exercise the machinery and carries no production effect.
