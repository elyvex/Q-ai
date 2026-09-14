# Runbook — Interrupted Job Recovery

**When to use:** a worker crashed, was `SIGKILL`ed, or a lease expired mid-run.

## Trigger
- `qai doctor` reports `jobs.interrupted_runs` > 0, or `jobs.worker_health` warns about
  `Running` jobs whose leases have expired.

## Detection
1. `qai job list --state Running` — look for rows whose `lease_expires_at` has passed.
2. `qai job list --state Interrupted` — jobs already reclaimed by the startup scan.

## Steps
1. Confirm no live worker still owns the job (check `lease_owner`).
2. Reap expired leases: the worker startup scan moves `Running` → `Interrupted` automatically.
   To force it, restart the job worker pool.
3. Resume: jobs with a `checkpoint` replay from their last checkpoint; completed stages are not
   re-run. Queue a resume with `qai job retry <job-id>` (Phase 1+), or re-enqueue idempotently.
4. If the job was **not** idempotent and partially applied a side effect, treat it as failed and
   inspect the audit trail before retrying.

## Guarantees relied on
- At-least-once delivery with idempotency keys; a duplicate enqueue is a no-op.
- No partial activation: handlers stage then flip a pointer in one transaction.
- Checkpoints are durable; resume never repeats a completed stage.

## Verify
- `qai doctor` shows `jobs.interrupted_runs` = 0 and `jobs.dead_letter_count` = 0.
