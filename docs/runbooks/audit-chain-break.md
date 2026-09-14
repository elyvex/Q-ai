# Runbook — Audit Chain Break

**When to use:** `qai audit verify` reports a chain mismatch, a gap, or `doctor` flags
`audit.chain_valid` as failing.

## Trigger
- `qai audit verify` returns non-zero, or a nightly `audit.verify_chain` job fails.

## Why it matters
The audit log is append-only and hash-chained: every row commits to all prior rows. A break means
either (a) the database was edited outside Q-ai, or (b) corruption occurred. It is **not** repaired
by rewriting rows.

## Steps
1. Identify the first bad sequence:
   `qai audit list --since <ts>` and compare `prev_chain_hash` to the prior row's `chain_hash`.
2. Capture evidence: export the surrounding rows (`qai audit list --json`) before any restore.
3. Determine scope — a single row or a contiguous region.
4. If corruption: restore from the most recent verified backup (`backup-restore.md`) and replay
   the audit events recorded since; do not patch hashes by hand.
5. If external edit: preserve the tampered database as forensic evidence and open an incident.

## Prevention
- `UPDATE`/`DELETE` on `audit_events` is blocked by SQL triggers (`QAI-AUD-0001/0002`).
- Chain writes are serialized through the single write pool, so concurrent writers cannot gap the
  sequence.

## Verify
- `qai audit verify` exits 0 and reports `valid: true`.
