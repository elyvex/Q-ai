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
| 01-01-02 | FND-01 effective config | `cargo test -p testkit --test config_precedence && cargo test -p cli --test foundation -- --nocapture` | 10/10 + 12/12 green 2026-09-26: CLI>env>file>defaults with `ValueOrigin` per leaf (`config_precedence`: defaults/file/CLI/missing/malformed/origin-totality/redaction; `foundation`: get/validate/explain/`--data-dir` CLI-origin adjacency); `config show --json` carries dotted-key origins; secrets redacted pre-emission; every non-passing validation emits the same stable code, non-empty remedy, and runnable `next_command` in human and JSON (D-07) |
| 01-01-03 | FND-02/FND-03 doctor + readiness + audit | `cargo test -p cli --test foundation -- --nocapture && cargo test -p application --lib db` | 15/15 + 12/12 green 2026-09-26: metadata-only doctor (`symlink_metadata` probes, no `create_dir_all`/probe files; D-06/T-01-DOCTOR); existing-database guard in `quran_cli::open_db` + `approval_flow` returns typed `MigrationRequired` (exit 3, `qai db migrate` remedy) without creating state (D-05/T-01-MIG, incl. new `ordinary_read_on_missing_database_names_migrate_without_creating_state`); persisted `verify_persisted_audit` is the single authority for `qai audit verify` and doctor `audit.chain_valid` — new `audit_verify_tampered_human_and_json` (QAI-AUD-0005, seq 2) and `audit_verify_gap_human_and_json` (QAI-AUD-0004, seq 3) assert same code/remedy/`qai audit verify` next command/affected sequences and `exit_code::VALIDATION` in human and JSON (D-12/T-01-AUDIT); `arch-check` + `migrate-check` green |
| 01-02-01 | FND-04 same-UoW | `cargo test -p application --test phase1_foundation audited_mutation -- --nocapture && cargo test -p application --lib -- --nocapture` | 4/4 + 40/40 green 2026-09-26: `AuditedMutation` (`application::audit_bridge`) stages domain writes → required provenance/outbox writes → one chained `SourceStaged/SourceActivated/SourceApproved/ApprovalGranted/SourceImported` audit event in the caller's `UnitOfWork`; caller commits (D-09/D-10). Routed: `record_approval` (row + `ApprovalGranted` same tx), `record_edition_verification` (`SourceApproved`), `activate/rollback/deprecate_edition` (shared `audit_activation` leg), `QuranImportHandler` staging completion (`SourceStaged` via composition, importer still staging-only, `ApprovalToken` untouched), `ensure_source_version` (`SourceImported`). `storage::workflows` stays backend-neutral with D-09/D-10 audit-contract docs; `audited_source_activation/deactivation/provenance_write` are the application wrappers. Approval negatives (missing/denied/mismatched) unchanged, covered by `quran_import::activation_service_requires_a_granted_approval`. New `phase1_foundation.rs`: commit case proves domain+provenance+2×outbox+2 chained events visible together with `verify_persisted_audit` valid (D-12); audit-failure case proves full rollback with prior state unchanged |
| 01-02-02 | FND-04 failure matrix | `cargo test -p application --test phase1_foundation && cargo test -p storage-sqlite --test commit_bounds_outbox --test integrity_audit` | 6/6 + 5/5 + 3/3 green 2026-09-26: `audited_mutation_failure_matrix_rolls_back_every_boundary` proves all six boundaries (domain NotFound, provenance duplicate-PK, generation trigger, outbox idempotency Conflict, audit trigger, no-commit drop) leave zero partial rows with prior state unchanged (D-10, T-02-ATOMICITY/T-02-PROVENANCE/T-02-SQL bound params). `commit_bounds_outbox::{audited_activation_commits_domain_outbox_and_audit_together, audited_activation_rolled_back_discards_outbox_and_audit}` proves same-UoW audit commit/rollback at the storage boundary. `integrity_audit::tampered_sequence_is_identifiable_from_storage_reads` proves a broken link fails verification with offending sequence 3 identifiable (T-02-RECOVERY); `phase1_foundation::tampered_chain_reports_offending_sequence_with_verify_remedy` proves `verify_persisted_audit.tampered_sequences == [3]` → QAI-AUD-0005 + non-empty remedy + `qai audit verify` next command without mutating the DB (D-12). `read_only_paths_and_derived_rebuilds_stage_no_audit_events` proves reads + a full derived forms rebuild (writes derived rows + Layer-D provenance) stage zero audit events and leave canonical rows identical (D-09) |
| 01-03-01 | FND-05 checkpoints/retry/cancel | `cargo test -p jobs --lib && cargo test -p storage-sqlite --test recovery_jobs && cargo test -p application --lib job_queue` | 30/30 + 9/9 + 5/5 green 2026-09-27: `RetryPolicy{max_attempts,backoff_base_ms,backoff_max_ms,jitter}` on `HandlerRegistry` (`policy_for`/`has_policy`/`set_policy`/`register_with_policy`/`stamp_job_policy`); `JobContext::checkpoint_named` stores versioned `{"version":1,"name","payload"}` (`NamedCheckpoint::parse` rejects legacy raw values; `latest_checkpoint` preserved; `take_pending_checkpoint` drains for the worker watchdog in 01-03-02); `JobQueue` + `storage::JobRepository` gain `request_cancel` (flag-only on non-terminal, lease preserved), `retry` (Failed/DeadLettered/Interrupted → Queued/attempts 0/due now/lease cleared, `checkpoint_json` preserved; others `InvalidState`/`NotFound` without mutation), `checkpoint_owned`/`finish_owned` (Running + lease-holder only, `COALESCE` never clears a boundary, others affect zero rows); SQLite implements all four with owner predicates (`cancel` now delegates to `request_cancel`); `SqliteJobQueue` implements the queue ops and stamps policy at enqueue when built via `build_worker`/`with_retry_registry` (D-14/D-15, T-03-LEASE/T-03-RETRY/T-03-CHECKPOINT). No migration touched |
| 01-03-02 | FND-05 worker outcomes | `cargo test -p jobs --lib -- --nocapture && cargo test -p storage-sqlite --test recovery_jobs && cargo test -p application --lib job_queue` | 37/37 + 13/13 + 8/8 green 2026-09-27: `Worker::run_once` keeps claim → validate → pre-start observation → handler/watchdog → terminal order; watchdog persists each `checkpoint_named` boundary immediately via owner-checked `checkpoint_owned` (drain + flush, `COALESCE` never clears); `CancellationDisposition{CancelledAtCheckpoint,CompletedBeforeObservation,MissedBoundary}` persisted in result/error JSON (`disposition_json`/`parse_disposition`) and reported via `WorkerOutcome::Cancelled{disposition}` (pre-start → AtCheckpoint; success-despite-request → Succeeded + CompletedBeforeObservation stored; failure + request + boundary → AtCheckpoint; failure + request + no boundary → MissedBoundary, never a false claim); retry resolves `policy_for(kind)` when explicit else the enqueue snapshot capped by host config, deterministic per-kind backoff, exhaustion → inspectable DeadLettered, non-idempotent never auto-retried (prohibition kept); terminals use `finish_owned` with lease-loss warn; `run_until_idle` documented long-lived-host-only (D-13/01-04). Real-SQLite: crash-with-pending-cancel keeps flag + boundary, per-kind exhaustion → inspectable + explicit retry reopens, boundary dispositions stored, stale owner finalized nothing (D-11/D-14/D-15/D-16, T-03-LEASE/T-03-CONCURRENCY) |
| 01-03-03 | FND-05 lifecycle audit + `qai job` | `cargo test -p application --lib job_queue && cargo test -p storage-sqlite --test phase1_jobs_audit --test recovery_jobs && cargo test -p cli --test jobs` | 9/9 + 5/5 + 13/13 + 4/4 green 2026-09-27: stable `AuditAction::{JobEnqueued,JobLeased,JobCompleted,JobFailed,JobRetried,JobCancellationRequested,JobCancelled}`; every `SqliteJobQueue` lifecycle op stages its event via `AuditedMutation` in the same UoW (`Actor::Job`, allowlisted id/kind/state/attempts/disposition, subject `urn:qai:job:<id>`; checkpoint/heartbeat stay unaudited per D-11); `retry_job`/`request_job_cancel` operator controls return the `get_job` inspection envelope (NotFound→5, Ineligible→6, missing DB→3 with `qai db migrate` remedy) with same-UoW operator audit (`Actor::System{qai-cli}`); `qai job retry/cancel` replace the old cancel refusal (catalog test drops the refused-cancel case); `get_job` adds redacted `error_json` + parsed `disposition`, redacts payload/checkpoint/error pre-emission (`***REDACTED***`, sentinel never leaks). `phase1_jobs_audit.rs` (5): atomic commit, rollback-on-audit-failure, ordered allowlisted payloads, cancel/retry rows, no rows for checkpoint/heartbeat. `cli/tests/jobs.rs` (4, real binary): show envelope + disposition + NOT_FOUND, retry eligible/conflict/missing, cancel request preserves lease with no false claim, combined-output redaction. Application `lifecycle_transitions_emit_ordered_audited_events` proves enqueue→lease→complete / request→lease→cancel / fail→retry ordering with job-actor attribution (operator retry is `qai-cli`) and `verify_persisted_audit` valid (D-12). `jobs --lib` 37/37, `cli catalog` 2/2, `arch-check` + `migrate-check` green (D-09/D-10/D-11/D-15/D-16, T-03-LEASE/T-03-RETRY/T-03-SECRET) |
| 01-04-01 | FND-06 host | `cargo test -p application --test phase1_jobs_host -- --nocapture && cargo test -p cli --test quran -- --nocapture` | phase1_jobs_host 3/3 + jobs --lib 37/37 green 2026-09-28: `Worker::run_until_shutdown(watch::Receiver<bool>)` (recover, claim-one-at-a-time, bounded idle poll, shutdown-mid-handler persists an owner-safe `request_cancel` and awaits the durable terminal instead of aborting; shared per-job path with `run_once`) + `application::job_queue::{build_default_registry (QuranImport/FormsRebuild/IndexBuild/MorphologyImport on one Arc<SqliteDatabase>), open_host_database (existence backstop, never creates), run_worker_host}` + `qai serve` readiness gate (`database_readiness::Current` else exit 3/70 with remedy+next, never migrates) and one lifetime `select!` over the server future vs SIGINT/SIGTERM that signals the worker, joins it, then returns (D-13/D-14/D-16, T-04-HOST/T-04-SHUTDOWN/T-04-LEASE/T-04-SCOPE). Host test (real SQLite): default registry completeness, seeded `quran.import` → Succeeded with 14 staged + `verify_persisted_audit` valid, pre-start cancel → Cancelled/`cancelled_at_checkpoint` via `get_job`, in-flight race → truthful Succeeded-or-Cancelled (never a false claim), idle shutdown claims 0, late job stays Queued (no detached task). CLI test (real `qai serve` child, isolated loopback port, `/readyz`): seeded import → Succeeded + `validate` passes, SIGINT → exit 0, port released, late job Queued; missing DB → exit 3 naming `qai db migrate` with no file created; non-loopback → exit 4. Full `quran` target 10/12: all import snapshots still green (import still sync in this task); 2 pre-existing `index rebuild` manifest-hash drifts (search/normalize) fail — manifest code untouched here, owned by 01-04-03 |
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
