---
phase: 01-foundations
plan: 03
subsystem: background-jobs
tags: [rust, sqlite, sqlx, job-queue, checkpoint, retry-policy, cancellation, audit-chain, redaction, cli]

# Dependency graph
requires:
  - phase: 01-02
    provides: AuditedMutation same-UoW composition (`application::audit_bridge`) consumed by every job lifecycle transition
  - phase: 01-01
    provides: persisted-verifier audit authority (`verify_persisted_audit`), existing-database guards, centralized CLI exit codes
provides:
  - Named durable checkpoints (`JobContext::checkpoint_named`, versioned JSON) persisted immediately through the queue, surviving expired-lease recovery
  - Per-kind `RetryPolicy` on `HandlerRegistry` with enqueue-time snapshot, bounded deterministic backoff, dead-letter exhaustion, explicit operator retry
  - Cooperative cancellation: persisted request + three locked dispositions (`CancelledAtCheckpoint`, `CompletedBeforeObservation`, `MissedBoundary`), owner-safe throughout
  - Same-UoW lifecycle audit (7 stable actions) + `qai job retry/cancel/show` with redacted inspection and centralized exit mapping
  - Real-SQLite + real-binary acceptance suites (`recovery_jobs`, `phase1_jobs_audit`, `cli/tests/jobs`)
affects: [01-04 (host consumes the durable lifecycle + dispositions), 01-05 (final gates, TASK-001 closure)]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
# Same estimateTokens scale (chars/4 over the realized diff), never a harness token count.
actuals:
  tokens: 36110    # chars/4 over the 3 plan-task commit diffs (git diff 1577681..5ce2a4d = 144441 chars)
  tasks: 3         # tasks completed
  commits: 3       # plan-task commits (d60da35 + 3de124c + 5ce2a4d); SUMMARY + tracking commits separate
plan_head_before: 1577681

# Tech tracking
tech-stack:
  added: []
  patterns: [backend-neutral port + concrete SQLite adapter for every job transition, owner-predicate SQL for lease safety, same-UoW audit staging per lifecycle transition, allowlisted audit payloads with secret redaction pre-emission, real-binary CLI tests via CARGO_BIN_EXE_qai]

key-files:
  created: [crates/storage-sqlite/tests/phase1_jobs_audit.rs, crates/cli/tests/jobs.rs]
  modified: [crates/jobs/src/lib.rs, crates/jobs/src/queue.rs, crates/jobs/src/registry.rs, crates/jobs/src/worker.rs, crates/storage/src/repository.rs, crates/storage-sqlite/src/lib.rs, crates/application/src/job_queue.rs, crates/application/src/db.rs, crates/audit/src/lib.rs, crates/cli/src/lib.rs, crates/storage-sqlite/tests/recovery_jobs.rs, docs/04-tasks/active/TASK-001-foundation-gap-closure.md]

key-decisions:
  - "Immediate checkpoint persistence through the queue (not failure-time read-back) — the committed SQLite row is the resume authority (D-14)"
  - "Per-kind policy with enqueue-time max_attempts snapshot, capped by host config at runtime; non-idempotent jobs never auto-retry (D-15 + prohibition kept)"
  - "Cancellation is request-only from any non-owner; only the lease holder finalizes, with exactly three locked dispositions (D-16 + prohibitions kept)"
  - "Checkpoint/heartbeat writes stage no audit rows — D-11 enumerates enqueue, lease, completion, failure, cancellation; progress writes would flood the chain"
  - "Operator retry/cancel return the same redacted get_job envelope and carry a System{qai-cli}-attributed audit event; worker transitions are Job-attributed"

patterns-established:
  - "Owner-checked transitions: checkpoint_owned/finish_owned/heartbeat require the current lease holder, affect zero rows otherwise; request_cancel never clears ownership"
  - "Disposition round-trip: persisted in result/error JSON via disposition_json/parse_disposition, surfaced through WorkerOutcome and the qai job show envelope"

requirements-completed: [REQ-product-principles, REQ-engineering-baseline, REQ-storage-architecture, REQ-cli-api, REQ-architecture-principles-quality]

# Coverage metadata (#1602) — one entry per shipped deliverable. Drives DETERMINISTIC UAT routing in verify-work.
coverage:
  - id: D1
    description: "Durable named checkpoints: checkpoint_named persists versioned payloads immediately; expired-lease recovery resumes from the last committed boundary"
    requirement: "REQ-storage-architecture"
    verification:
      - kind: unit
        ref: "cargo test -p jobs --lib (37 passed incl. checkpoint/cancellation/backoff cases)"
        status: pass
      - kind: integration
        ref: "crates/storage-sqlite/tests/recovery_jobs.rs#named_checkpoint_is_committed_before_handler_completion + expired_lease_recovery_preserves_the_named_checkpoint + expired_lease_marks_interrupted_and_resumes_from_checkpoint"
        status: pass
    human_judgment: false
  - id: D2
    description: "Per-kind bounded retry: RetryPolicy per handler kind, enqueue snapshot, deterministic backoff, dead-letter exhaustion, explicit retry only for eligible states"
    requirement: "REQ-engineering-baseline"
    verification:
      - kind: integration
        ref: "crates/storage-sqlite/tests/recovery_jobs.rs#explicit_retry_transitions_only_eligible_states + per_kind_exhaustion_leaves_an_inspectable_state_with_explicit_retry"
        status: pass
      - kind: integration
        ref: "crates/cli/tests/jobs.rs#job_retry_moves_only_eligible_jobs"
        status: pass
    human_judgment: false
  - id: D3
    description: "Cooperative cancellation: persisted request preserves live leases; worker reports exactly CancelledAtCheckpoint / CompletedBeforeObservation / MissedBoundary"
    requirement: "REQ-engineering-baseline"
    verification:
      - kind: integration
        ref: "crates/storage-sqlite/tests/recovery_jobs.rs#cancel_request_preserves_owner_and_expiry + cancellation_boundaries_record_truthful_terminal_states + crash_with_pending_cancel_keeps_request_and_boundary + stale_owner_cannot_finalize_after_lease_loss"
        status: pass
      - kind: integration
        ref: "crates/cli/tests/jobs.rs#job_cancel_requests_without_clearing_the_lease"
        status: pass
    human_judgment: false
  - id: D4
    description: "Lifecycle audit atomicity: every enqueue/lease/completion/failure/retry/cancel-request/cancel-outcome stages its chained event in the same UoW with allowlisted payloads"
    requirement: "REQ-product-principles"
    verification:
      - kind: unit
        ref: "cargo test -p application --lib job_queue (9 passed incl. lifecycle_transitions_emit_ordered_audited_events)"
        status: pass
      - kind: integration
        ref: "crates/storage-sqlite/tests/phase1_jobs_audit.rs (5 passed: atomic commit, rollback, ordering, cancel/retry rows, no rows for checkpoint/heartbeat)"
        status: pass
    human_judgment: false
  - id: D5
    description: "qai job controls: retry/cancel/show --json with redacted inspection envelope, centralized NOT_FOUND/CONFLICT/VALIDATION exits, no secret leakage"
    requirement: "REQ-cli-api"
    verification:
      - kind: e2e
        ref: "crates/cli/tests/jobs.rs (4 passed incl. job_show_exposes_the_redacted_inspection_envelope + job_output_never_leaks_secrets)"
        status: pass
      - kind: integration
        ref: "cargo test -p cli --test catalog (2 passed, guarded-command behavior preserved)"
        status: pass
    human_judgment: false

# Metrics
duration: 35min
completed: 2026-09-27
status: complete
---

# Phase 01 Plan 03: Durable Job Checkpoints, Retry, Cancellation, and Lifecycle Audit Summary

**SQLite-leased job queue gains immediately-persisted named checkpoints, per-kind bounded retry with explicit operator retry, cooperative owner-safe cancellation with three locked dispositions, same-transaction lifecycle audit, and real `qai job retry/cancel/show` controls proven by real-SQLite and real-binary suites.**

## Performance

- **Duration:** ~35min (close-out run: verify in-progress work, TASK-001 evidence, task-3 commit, SUMMARY + tracking)
- **Started:** 2026-09-27T21:11:28+0330 (first task commit d60da35; prior-session implementation)
- **Completed:** 2026-09-27 (close-out commit 5ce2a4d + SUMMARY)
- **Tasks:** 3 (01-03-01 contracts, 01-03-02 worker semantics, 01-03-03 audit + CLI controls)
- **Files modified:** 16 (2 created, 14 modified incl. evidence record)

## Accomplishments

- `RetryPolicy{max_attempts,backoff_base_ms,backoff_max_ms,jitter}` lives beside
  `HandlerRegistry` (`policy_for`/`set_policy`/`register_with_policy`/`stamp_job_policy`);
  enqueue snapshots the kind policy into `max_attempts` (D-15). No migration, no broker,
  no second queue — existing `jobs` columns reused, `0004_jobs.up.sql` untouched.
- `JobContext::checkpoint_named(name, payload)` stores versioned
  `{"version":1,"name","payload"}` and persists through the queue immediately; the
  worker watchdog drains and flushes each boundary via owner-checked
  `checkpoint_owned`, so the committed SQLite row — not memory — is the resume
  authority. Expired-lease recovery reclaims `Interrupted`/`Checkpointed` jobs from
  the last committed boundary without repeating it (D-14).
- `request_cancel` persists only the flag on non-terminal jobs and never clears a
  live lease; `retry` moves only Failed/DeadLettered/Interrupted → Queued
  (attempts 0, due now, lease cleared, checkpoint preserved) and rejects everything
  else without mutation; heartbeat/checkpoint/finish reject non-holders with zero
  rows affected (T-03-LEASE, both lease prohibitions kept).
- `Worker::run_once` keeps claim → validate → pre-start observation →
  handler/watchdog → terminal order; `CancellationDisposition` is exactly
  `CancelledAtCheckpoint` / `CompletedBeforeObservation` / `MissedBoundary`,
  persisted in result/error JSON and reported via `WorkerOutcome` (no false
  cancellation claim — prohibition kept; non-idempotent jobs never auto-retried —
  prohibition kept; `run_until_idle` documented host-only for 01-04, D-13).
- Seven stable `AuditAction`s (`job_enqueued, job_leased,
  job_completed, job_failed, job_retried, job_cancellation_requested,
  job_cancelled`) stage via `AuditedMutation` in the same UoW as each job mutation
  (`Actor::Job`, subject `urn:qai:job:<id>`, allowlisted id/kind/state/attempts/
  disposition only). Checkpoint/heartbeat stage no rows per D-11's enumerated set.
- `qai job retry <id>` / `qai job cancel <id>` replace the old cancel refusal;
  both return the redacted `get_job` envelope (attempts, max attempts, state,
  cancel request, redacted checkpoint/error, disposition). Missing id → exit 5,
  ineligible state → exit 6, missing database → exit 3 with the `qai db migrate`
  remedy; happy paths exit 0 with one parseable JSON document. Secrets never
  leak (`***REDACTED***`, sentinel-negative tests).
- Plan `<verify>` re-run live in this close-out: `jobs --lib` 37/37,
  `recovery_jobs` 13/13, `phase1_jobs_audit` 5/5, `application --lib job_queue`
  9/9, `cli --test jobs` 4/4, `cli --test catalog` 2/2, `arch-check` OK,
  `migrate-check` OK (21 migrations, checksums stable).
- TASK-001 record carries all three 01-03 evidence rows (commands + observed
  counts + D-11/D-14/D-15/D-16 citations); TASK-001 stays Active — closure owned
  by 01-05.

## Task Commits

Each task was committed atomically (tasks 1–2 by the prior session, verified via
`git show --stat` and spot-checked; task 3 closed out in this run):

1. **Task 01-03-01: Define durable checkpoint and retry contracts** — `d60da35`
   (feat: RetryPolicy, checkpoint_named, owner-safe request_cancel/retry/
   checkpoint_owned/finish_owned ports, real-SQLite adapter proof, TASK-001 row)
2. **Task 01-03-02: Make worker checkpoints, retries, and cancellation cooperative** — `3de124c`
   (feat: watchdog persistence, three dispositions, per-kind exhaustion, ownership
   cases, TASK-001 row)
3. **Task 01-03-03: Add job controls and lifecycle audit events** — `5ce2a4d`
   (feat: 7 audit actions + same-UoW composition, operator controls with exit
   mapping, job retry/cancel commands, redacted show + disposition, phase1_jobs_audit
   + cli/jobs suites, catalog update, TASK-001 row)

**Plan metadata:** this SUMMARY + STATE/ROADMAP updates (final tracking commit, see completion message).

## Files Created/Modified

- `crates/jobs/src/registry.rs` — `RetryPolicy` + per-kind lookup/stamping
- `crates/jobs/src/lib.rs` — `checkpoint_named`, versioned `NamedCheckpoint`, pending-drain
- `crates/jobs/src/queue.rs` — `request_cancel`/`retry`/owner-safe ops on the `JobQueue` port
- `crates/jobs/src/worker.rs` — watchdog persistence, `CancellationDisposition`,
  `WorkerOutcome` dispositions, per-kind retry, owner-checked terminals
- `crates/storage/src/repository.rs` — backend-neutral explicit retry/cancel/checkpoint ports
- `crates/storage-sqlite/src/lib.rs` — owner-safe SQLite transitions on existing columns
- `crates/application/src/job_queue.rs` — audited `SqliteJobQueue` adapter + operator
  `retry_job`/`request_job_cancel` with `JobControlError` exit mapping
- `crates/application/src/db.rs` — `get_job` redacted envelope + `error_json` + `disposition`
- `crates/audit/src/lib.rs` — 7 stable job lifecycle actions
- `crates/cli/src/lib.rs` — `job retry`/`job cancel` with centralized error rendering
- `crates/cli/tests/catalog.rs` — drops the refused-cancel case (cancel is now real)
- `crates/storage-sqlite/tests/recovery_jobs.rs` — extended adapter/ownership/crash coverage
- `crates/storage-sqlite/tests/phase1_jobs_audit.rs` — created; 5 lifecycle-audit cases
- `crates/cli/tests/jobs.rs` — created; 4 real-binary control/redaction/exit-code cases
- `docs/04-tasks/active/TASK-001-foundation-gap-closure.md` — 01-03-01/02/03 evidence rows

## Decisions Made

- Reuse-only per D-01…D-04: existing `jobs` table, `JobQueue`, `HandlerRegistry`,
  `Worker`, `SqliteJobQueue`, lease recovery, deterministic backoff, audited UoW.
  No broker, second queue, new package, or migration.
- Checkpoint writes go through the queue at record time (immediate persistence)
  rather than being read back only after failure — the prior memory-only gap is
  closed at the seam, not papered over in the worker.
- `finish_owned` (not plain `finish`) is the worker terminal path so lease-loss
  races finalize nothing; `request_cancel` is deliberately ineligible-state-free
  for active states and conflict-only for terminal ones.
- Audit attribution split: worker transitions are `Actor::Job`, explicit operator
  actions are `Actor::System{qai-cli}` — the chain shows who moved the job.
- Checkpoint/heartbeat excluded from the audit chain (high-frequency progress,
  not lifecycle transitions) with a negative test pinning the exclusion.

## Deviations from Plan

None - plan executed as written across the two sessions. The prior executor
completed tasks 1–3 implementation (including the full task-3 working tree:
audited adapter, operator controls, CLI commands, both new test targets, catalog
update) but was interrupted before filling the 01-03-03 TASK-001 evidence row
and committing. This close-out verified every must-have truth by re-running the
full `<verify>` chain green, filled the evidence row from observed counts, and
committed the preserved working tree atomically as the task-3 commit. No
Rule 1–4 fixes were needed — nothing actually failed.

## Issues Encountered

- Prior executor interrupted mid-plan after committing `d60da35` (task 1) and
  `3de124c` (task 2), leaving task-3 work uncommitted in the tree. Both commits
  verified via `git show --stat`; the uncommitted tree was preserved verbatim
  (`git status`/`git diff` first, no stash/reset/clean) and committed as
  `5ce2a4d`. No work lost.
- `git status` shows untracked `.planning/milestone.lock` from another session;
  left untouched per the dispatch directive.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 01-04 (host ownership, D-13): the durable lifecycle, dispositions,
  and audited transitions are the exact seams the host consumes; `run_until_idle`
  is already fenced as host-only.
- No blockers. TASK-001 remains Active; closure owned by 01-05.

## Self-Check: PASSED

- `crates/cli/tests/jobs.rs`, `crates/storage-sqlite/tests/phase1_jobs_audit.rs`,
  `crates/application/src/job_queue.rs`, `crates/cli/src/lib.rs`,
  `docs/04-tasks/active/TASK-001-foundation-gap-closure.md` all FOUND.
- Commits `d60da35`, `3de124c`, `5ce2a4d` FOUND via `git show --stat`.
- No stub patterns (`TODO|FIXME|placeholder|unimplemented|todo!`) in the plan diff.
- No new network/auth/schema trust-boundary surface outside the plan
  `<threat_model>` (T-03-LEASE/T-03-RETRY/T-03-CHECKPOINT/T-03-SECRET/
  T-03-CONCURRENCY all mitigated in-plan) — no Threat Flags section.

---
*Phase: 01-foundations*
*Completed: 2026-09-27*
