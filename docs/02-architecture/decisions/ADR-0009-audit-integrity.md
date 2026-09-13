# ADR-0009 — Audit Integrity: Hash Chain + Append-Only Triggers

- Status: Accepted
- Phase: 0 — Foundations
- Date: 2026-09-11
- Related decisions: ADR-0001, ADR-0006
- Requirements: PRD §§82, 91

## Context

Q-ai must produce an immutable, tamper-evident log of every mutation: config
changes, source imports, approvals, canonical change sessions, job lifecycle,
and doctor repairs. The log must survive power loss and remain verifiable
offline. SQLite lacks built-in audit capabilities, so the integrity must be
enforced by triggers and a content hash chain.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| **Hash chain + append-only triggers** | Tamper-evident; any edit breaks the chain; verifiable offline; no external dependency | Read-side verification is O(n); cannot redact (by design) |
| WORM storage (append-only filesystem) | OS-level immutability | OS-specific; not portable; no per-record integrity |
| External audit service | Strongest prevention | Network dependency; defeats local-first; single point of failure |
| Merkle tree over audit rows | Efficient partial verification | Complex; rebalancing on deletion is infeasible (and deletion is forbidden) |

## Decision

Implement an **append-only `audit_events` table** with two enforcement layers:

1. **Trigger layer**: `BEFORE UPDATE` → `RAISE(ABORT, 'QAI-AUD-0001: audit log is append-only')`; same for `DELETE` (`QAI-AUD-0002`). These fire even if a process has `UPDATE` permission, protecting against bugs in application code.
2. **Hash chain layer**: each row's `chain_hash = H(prev_chain_hash || canonical_json(event_without_chain_hash))` (SHA-256 over ADR-0006 canonical JSON). `qai audit verify` walks the chain and recomputes every hash; any tampered row is detected.

Design rules:

- `sequence` is a gapless, monotonically increasing `INTEGER UNIQUE`.
- `actor` distinguishes `Principal | System | Job | Agent(future)`.
- `before`/`after` are allowlisted-key JSON (redacted; free-form payload keys are
  rejected by the audit writer schema).
- The audit writer runs all values through the global redaction layer — a
  permanent CI gate (`secret_leak.rs`) verifies a sentinel never appears in any
  audit row.
- The `audit.verify_chain` job runs nightly (Phase 1+); in Phase 0 it is
  invokable via `qai audit verify`.

## Accuracy and Religious-Source Implications

- Audit integrity underpins the credibility of every claim Q-ai makes about
  source origin and canonical integrity. A broken chain must visibly fail open
  (report FAIL) rather than silently degrade.
- Religious sources are never quoted without a traceable audit path; the hash
  chain enables a user to verify that a quoted verse has not been substituted
  since import (chain integrity + `quoted_text_hash` in provenance).

## Licensing Implications

SHA-256 via `sha2` crate (MIT/Apache-2.0). No impact.

## Security Implications

- Append-only + trigger enforcement means even a compromised process cannot
  silently edit history.
- `prev_chain_hash` links in the chain make selective deletion detectable
  (sequence gaps + hash mismatch).
- Audit records are included in backup/restore verification (AC-P0-21).

## Operational Implications

- `qai audit list`, `qai audit verify` (reads only — no writes) cover the
  operator surface.
- Verification scales linearly; nightly re-verification in Phase 1+ runs during
  low-traffic windows.
- Chain hashes are stored as `TEXT "sha256:<hex>"` per ADR-0006.

## Migration Strategy

Created by `0005_audit.up.sql`. Existing audit logs from Phase 1+ are verified
by `qai audit verify` on first run; any gap is a hard FAIL for the operator.

## Reversal Cost

Very high. Removing the chain would require rewriting every `chain_hash` and
`prev_chain_hash` column, invalidating every existing chain. By design this is
irreversible.

## Acceptance Criteria

- `qai audit verify` detects a manually tampered row (insert in `tests/integrity/audit_chain.rs`).
- `UPDATE`/`DELETE` on `audit_events` are aborted by triggers.
- A sentinel secret never appears in any audit row (CI gate).
