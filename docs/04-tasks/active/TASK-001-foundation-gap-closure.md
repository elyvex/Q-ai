# TASK-001 — Foundation Gap Closure (Phase 1)

- **Task ID:** `TASK-001-foundation-gap-closure`
- **Status:** Active
- **Phase:** `01-foundations` (roadmap Phase 1)
- **Owner:** Engineering (GSD plans 01-01…01-05)
- **Phase contract:** `.planning/ROADMAP.md` §Phase 1 + `.planning/REQUIREMENTS.md`
  (`REQ-product-vision`, `REQ-product-principles`, `REQ-goals-non-goals`,
  `REQ-engineering-baseline`, `REQ-storage-architecture`, `REQ-cli-api`,
  `REQ-architecture-principles-quality`)
- **Closure owned by:** plan **01-05-02 only** — this record is moved
  `active → completed` by 01-05-02 after the captured green pre-move final gate and
  the record checker exist. Plan **01-05-03 owns post-move records** (validation,
  task-done rollup, status, changelog). No other plan may move this file or change
  either index.
- **Created:** 2026-09-26 by plan 01-01 (task 01-01-01) before any production edit,
  after `sh scripts/verify-phase1-preservation.sh capture`.

## Purpose

Close the evidence-backed operator-bootstrap gaps (FND-01, FND-02, FND-03) in the
existing brownfield foundation so a developer can inspect effective configuration and
its origins, diagnose a fresh workspace without creating state, migrate the local
SQLite database explicitly, and verify the header-linked audit chain — without a second
configuration, diagnostic, or storage service.

This is an **evidence-driven brownfield gap-closure** task (D-01). It preserves the
existing working foundation services rather than rebuilding them.

## Five roadmap success criteria (Phase 1)

| ID | Criterion | Owning evidence |
|---|---|---|
| C1 | Developer can build the workspace and run `qai --help` showing the stable command tree. | `crates/cli/tests/foundation.rs` seven-group help contract; `cargo test -p cli --test foundation` |
| C2 | Operator can configure via CLI > env > file > defaults with validation errors that name the remedy. | `crates/testkit/tests/config_precedence.rs`; `crates/cli/tests/foundation.rs` |
| C3 | System records provenance and append-only audit events for every state-changing operation. | 01-02 same-UoW tests; `qai audit verify` |
| C4 | Background jobs enqueue, lease, checkpoint, and cancel without an external broker. | 01-03 real-SQLite queue/recovery tests; `qai job` controls |
| C5 | `xtask arch-check` passes and CI fails on any forbidden crate dependency. | 01-05 external-edge mutation tests; `arch-check` |

## Locked decisions referenced (D-01 … D-16)

D-01 evidence-driven brownfield closure · D-02 roadmap/ADR contract governs ·
D-03 repeatable automated evidence · D-04 reuse later-phase code only as evidence ·
D-05 explicit migrations with `qai db migrate` remedy · D-06 read-only `qai doctor` ·
D-07 redacted effective config with origins + JSON · D-08 stable foundation groups ·
D-09 audit authoritative state changes · D-10 domain/provenance/audit/outbox
all-or-nothing · D-11 job lifecycle audit · D-12 `qai audit verify` authoritative ·
D-13 long-lived hosts run workers · D-14 named durable checkpoints ·
D-15 bounded per-kind retry · D-16 cooperative cancellation outcomes.

## Plan evidence (appended per task while status is Active)

| Plan/task | Gap | Command | Observed |
|---|---|---|---|
| 01-01-01 | FND-01/FND-02/FND-03 tracer | `cargo test -p cli --test foundation -- --nocapture && sh scripts/verify-phase1-preservation.sh check --task 01-01-01` | _pending_ |
| 01-01-02 | FND-01 effective config | `cargo test -p testkit --test config_precedence && cargo test -p cli --test foundation -- --nocapture` | _pending_ |
| 01-01-03 | FND-02/FND-03 doctor + readiness + audit | `cargo test -p cli --test foundation -- --nocapture && cargo test -p application --lib db` | _pending_ |
| 01-02-01 | FND-04 same-UoW | `cargo test -p application --test phase1_foundation audited_mutation -- --nocapture` | _pending_ |
| 01-02-02 | FND-04 failure matrix | `cargo test -p application --test phase1_foundation && cargo test -p storage-sqlite --test commit_bounds_outbox --test integrity_audit` | _pending_ |
| 01-03-01 | FND-05 checkpoints/retry/cancel | `cargo test -p jobs --lib && cargo test -p storage-sqlite --test recovery_jobs` | _pending_ |
| 01-03-02 | FND-05 worker outcomes | `cargo test -p jobs --lib -- --nocapture && cargo test -p storage-sqlite --test recovery_jobs` | _pending_ |
| 01-03-03 | FND-05 lifecycle audit + `qai job` | `cargo test -p application --lib job_queue && cargo test -p storage-sqlite --test phase1_jobs_audit --test recovery_jobs && cargo test -p cli --test jobs` | _pending_ |
| 01-04-01 | FND-06 host | `cargo test -p application --test phase1_jobs_host -- --nocapture && cargo test -p cli --test quran -- --nocapture` | _pending_ |
| 01-04-02 | FND-06 enqueue-only | `cargo test -p application --test quran_import -- --nocapture && cargo test -p cli --test quran enqueue_only_import_is_queued_without_host` | _pending_ |
| 01-04-03 | FND-06 snapshots | `cargo test -p cli --test quran -- --nocapture` | _pending_ |
| 01-05-01 | FND-07 external policy | `cargo test -p xtask && cargo run -q -p xtask -- arch-check` | _pending_ |
| 01-05-02 | closure (sole owner) | `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && ... && sh scripts/verify-phase1-preservation.sh check --task 01-05-02` | _pending_ |
| 01-05-03 | final gate + post-move records | `sh scripts/verify-phase1-preservation.sh check --all && sh scripts/verify-phase1-records.sh && sh scripts/verify-phase1-preservation.sh check --all` | _pending_ |

## Threat predicates

`T-01-CFG`, `T-01-DOCTOR`, `T-01-MIG`, `T-01-AUDIT`, `T-01-SQL` (plan 01-01) and the
01-02…01-05 predicates. Prohibitions: no secret output, no doctor writes, no implicit
migration/worker, no row-count audit pass.

## Probes

`PROBE-01` … `PROBE-10` dispositions are recorded in
`.planning/phases/01-foundations/01-VALIDATION.md` §Spec-Less Probe Ledger; this task
record references them and does not discharge them.

## Notes

- Owner-gated and deferred items (ADR-0101 dataset license, remote deployment) remain
  outside Phase 1 and are not closed by this task.
- Live worktree preservation is proven by `scripts/verify-phase1-preservation.sh`
  (`capture` before edits, `check` after each task and at the final gate).

---
