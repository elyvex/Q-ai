---
phase: "1"
slug: "foundations"
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-24"
updated: "2026-09-25"
---

# Phase 1 — Validation Strategy

> Planned execution contract for Phase 1 Foundations. This matrix is complete at planning time; every row is still **pending** until its owning executor records an actual result.

## Execution State

The plans define fourteen implementation tasks across five waves. No task is complete merely because it appears in this file. The executor must create the active `TASK-001-foundation-gap-closure` record before the first production edit, run the task-local command in the owning task, and only then append evidence. The active-to-completed move is owned exclusively by `01-05-02`; `01-05-03` records post-move consistency and may rerun gates but cannot move the task.

## Test Infrastructure

| Property | Planned value |
|----------|---------------|
| **Framework** | Existing Rust `#[test]`/`#[tokio::test]`, real `qai` process tests, `trycmd`, `tempfile`, and existing SQLite fixtures |
| **New dependency** | None. Reuse the checked-in `Cargo.lock`; no package legitimacy checkpoint is required because no install task is planned. |
| **Task-local quick command** | Run the exact command in the owning task row before marking that task complete. |
| **Wave command** | Run the task commands for the wave, then `cargo run -q -p xtask -- arch-check` and `cargo run -q -p xtask -- migrate-check`. |
| **Phase quick suite** | `cargo test -p config --lib && cargo test -p testkit --test config_precedence && cargo test -p cli --test foundation -- --nocapture && cargo test -p application --test phase1_foundation && cargo test -p jobs --lib && cargo test -p storage-sqlite --test recovery_jobs --test commit_bounds_outbox --test integrity_audit --test phase1_jobs_audit && cargo test -p application --test phase1_jobs_host && cargo test -p application --test quran_import && cargo test -p cli --test quran -- --nocapture && cargo test -p xtask` |
| **Full suite** | `cargo test --workspace` |
| **Final gate** | `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo run -q -p xtask -- arch-check && cargo run -q -p xtask -- migrate-check && cargo run -p xtask -- ci` |
| **Runtime** | Prefer the narrowest task command; no watch-mode flags. Record unavailable optional tooling as unavailable rather than converting it to a pass. |

## Sampling and Nyquist Rules

- **After every task commit:** run that task's automated command, then run `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` when Rust files changed.
- **After every plan wave:** run the wave's task commands and the architecture/migration gates. A later plan's command cannot substitute for an earlier task's own acceptance run.
- **Before phase verification:** run the final gate, record the real exit outcomes, and perform the independent cross-record check below.
- **Every implementation task:** has a runnable command in this matrix and in its PLAN.md. A test may be expanded in a later task, but the owning task must create and run its own first proof.
- **Real host evidence:** tasks `01-04-01`, `01-04-02`, and `01-04-03` each run the real `qai serve` path or the existing real-binary Quran harness; an application-only worker test is insufficient.
- **Status vocabulary:** `pending`, `green`, `red`, or `flaky`; only an actual command result may change a row. `green` never means a test was merely named in a plan.

## Per-Task Verification Matrix

All fourteen rows are required. The `Target` column identifies the artifact that must exist at execution time; `new` means the task creates it, while `existing` means the task extends an already tracked seam.

| Task ID | Plan | Wave | Requirement coverage | Locked decisions | Threat predicates | Required acceptance behavior | Test type | Automated command | Target | Status |
|---|---|---:|---|---|---|---|---|---|---|---|
| 01-01-01 | 01 | 1 | REQ-product-vision, REQ-engineering-baseline, REQ-cli-api, REQ-architecture-principles-quality | D-01, D-02, D-03, D-04, D-05, D-06, D-08, D-12 | T-01-CFG, T-01-DOCTOR, T-01-MIG, T-01-AUDIT | Real `CARGO_BIN_EXE_qai` fresh-workspace path shows seven groups, origin-bearing config, no doctor-created path, explicit `qai db migrate`, and an empty-chain audit result. | CLI/E2E | `cargo test -p cli --test foundation -- --nocapture` | `crates/cli/tests/foundation.rs` (new), active TASK-001/index | pending |
| 01-01-02 | 01 | 1 | REQ-product-principles, REQ-engineering-baseline, REQ-cli-api | D-01, D-02, D-03, D-07, D-08 | T-01-CFG | Effective file/env/CLI/default precedence, dotted origins, keyed lookup, secret redaction, and identical human/JSON code + remedy + next command for every non-passing validation result. | config/CLI | `cargo test -p testkit --test config_precedence && cargo test -p cli --test foundation -- --nocapture` | `crates/testkit/tests/config_precedence.rs`, `crates/cli/tests/foundation.rs` | pending |
| 01-01-03 | 01 | 1 | REQ-product-principles, REQ-engineering-baseline, REQ-storage-architecture, REQ-cli-api | D-01, D-05, D-06, D-12 | T-01-DOCTOR, T-01-MIG, T-01-AUDIT, T-01-SQL | Fresh, missing, pending, checksum-mismatch, current, restricted, and tampered-audit cases prove metadata-only doctor, no ordinary creating open, and `qai audit verify` recovery guidance. | CLI/integration | `cargo test -p cli --test foundation -- --nocapture && cargo test -p application --lib db` | `crates/application/src/quran_cli.rs`, `crates/cli/tests/foundation.rs` | pending |
| 01-02-01 | 02 | 2 | REQ-product-principles, REQ-engineering-baseline, REQ-storage-architecture, REQ-cli-api | D-01, D-02, D-03, D-09, D-10 | T-02-ATOMICITY, T-02-PROVENANCE, T-02-CANONICAL | One application `AuditedMutation` path commits domain, provenance/outbox, and chained audit records together; an injected failure leaves no partial row and importer staging cannot activate canonical data. | real SQLite/integration | `cargo test -p application --test phase1_foundation audited_mutation -- --nocapture && cargo test -p application --lib -- --nocapture` | `crates/application/tests/phase1_foundation.rs` (new), audited lifecycle seams | pending |
| 01-02-02 | 02 | 2 | REQ-product-principles, REQ-engineering-baseline, REQ-storage-architecture, REQ-cli-api | D-01, D-03, D-09, D-10, D-12 | T-02-ATOMICITY, T-02-RECOVERY, T-02-SQL | Independent real-SQLite fault injection covers every required write boundary, contiguous chain, tamper sequence, and a non-mutating `qai audit verify` remedy. | real SQLite/integration | `cargo test -p application --test phase1_foundation && cargo test -p storage-sqlite --test commit_bounds_outbox --test integrity_audit` | `crates/application/tests/phase1_foundation.rs`, `commit_bounds_outbox.rs`, `integrity_audit.rs` | pending |
| 01-03-01 | 03 | 3 | REQ-engineering-baseline, REQ-storage-architecture, REQ-cli-api | D-01, D-02, D-03, D-14, D-15 | T-03-LEASE, T-03-RETRY, T-03-CHECKPOINT | A task-local contract test proves named checkpoint persistence, per-kind policy isolation, explicit retry eligibility, and owner-safe operations against existing SQLite columns. | jobs contract/real SQLite | `cargo test -p jobs --lib && cargo test -p storage-sqlite --test recovery_jobs` | `crates/jobs/src/lib.rs` inline test, existing recovery target | pending |
| 01-03-02 | 03 | 3 | REQ-product-principles, REQ-engineering-baseline, REQ-storage-architecture, REQ-cli-api | D-01, D-03, D-11, D-14, D-15, D-16 | T-03-LEASE, T-03-RETRY, T-03-CHECKPOINT, T-03-CONCURRENCY | Real SQLite crash/resume, per-kind exhaustion, cancellation before/during/after boundary, and ownership cases run in this task; the three truthful cancellation dispositions are asserted. | worker/real SQLite | `cargo test -p jobs --lib -- --nocapture && cargo test -p storage-sqlite --test recovery_jobs -- --nocapture` | `crates/jobs/src/worker.rs`, `crates/storage-sqlite/tests/recovery_jobs.rs` | pending |
| 01-03-03 | 03 | 3 | REQ-product-principles, REQ-engineering-baseline, REQ-storage-architecture, REQ-cli-api | D-01, D-03, D-11, D-14, D-15, D-16 | T-03-LEASE, T-03-SECRET, T-03-CONCURRENCY | Enqueue, lease, checkpoint, retry, cancellation request/outcome, redacted job inspection, and explicit CLI controls are asserted against real SQLite in this task. | application/CLI/real SQLite | `cargo test -p application --lib job_queue && cargo test -p storage-sqlite --test phase1_jobs_audit --test recovery_jobs` | `crates/storage-sqlite/tests/phase1_jobs_audit.rs` (new), job CLI seam | pending |
| 01-04-01 | 04 | 4 | REQ-product-vision, REQ-product-principles, REQ-goals-non-goals, REQ-engineering-baseline, REQ-storage-architecture, REQ-cli-api, REQ-architecture-principles-quality | D-01, D-02, D-03, D-04, D-05, D-13, D-14, D-16 | T-04-HOST, T-04-SHUTDOWN, T-04-LEASE | A real `qai serve` child on an isolated loopback port claims a queued job, proves readiness, receives shutdown, awaits a durable terminal/checkpoint state, and exits without a detached worker; missing/pending DB reports the migration remedy. | application + real CLI host | `cargo test -p application --test phase1_jobs_host -- --nocapture && cargo test -p cli --test quran -- --nocapture` | `crates/application/tests/phase1_jobs_host.rs` (new), `crates/cli/tests/quran.rs` | pending |
| 01-04-02 | 04 | 4 | REQ-product-vision, REQ-product-principles, REQ-goals-non-goals, REQ-engineering-baseline, REQ-storage-architecture, REQ-cli-api, REQ-architecture-principles-quality | D-01, D-04, D-05, D-09, D-10, D-11, D-13, D-14, D-15, D-16 | T-04-ENQUEUE, T-04-LEASE, T-04-SECRET | The one-shot import returns a durable queued id/state before host processing; a real CLI case observes that response, then observes the host-owned terminal state; no inline worker or canonical activation is introduced. | application + real CLI | `cargo test -p application --test quran_import -- --nocapture && cargo test -p cli --test quran -- --nocapture` | `crates/application/tests/quran_import.rs`, `crates/cli/tests/quran.rs`, queued result seam | pending |
| 01-04-03 | 04 | 4 | REQ-product-vision, REQ-product-principles, REQ-engineering-baseline, REQ-cli-api | D-03, D-04, D-13 | T-04-HOST, T-04-ENQUEUE, T-04-SCOPE | The existing four trycmd suites run against a real `qai serve` child, synchronize persisted terminal state before dependent commands, clean up child/port/data paths, and retain unrelated Quran/search behavior. | real CLI/trycmd | `cargo test -p cli --test quran -- --nocapture` | four existing Quran snapshots and `crates/cli/tests/quran.rs` | pending |
| 01-05-01 | 05 | 5 | REQ-architecture-principles-quality, REQ-engineering-baseline | D-01, D-02, D-03, D-04 | T-05-ARCH, T-05-EXTERNAL, T-05-SUPPLYCHAIN | Inline `xtask/src/arch.rs` tests prove allowed/forbidden registry and git edges plus path fail-closed behavior; current metadata-derived allowances remain green and no package is installed. | build policy | `cargo test -p xtask && cargo run -q -p xtask -- arch-check` | `xtask/src/arch.rs`, `xtask/allowlist.toml` | pending |
| 01-05-02 | 05 | 5 | REQ-product-vision, REQ-product-principles, REQ-goals-non-goals, REQ-engineering-baseline, REQ-storage-architecture, REQ-cli-api, REQ-architecture-principles-quality | D-01, D-02, D-03 | T-05-STATUS, T-05-SCOPE | After a captured green pre-move final gate, the sole owner moves active TASK-001 to completed and updates the completed index; 01-05-03 then appends the newest-first rollup and updates the other progress records. | repository records | `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo run -q -p xtask -- arch-check && cargo run -q -p xtask -- migrate-check && cargo run -p xtask -- ci && test -f docs/04-tasks/completed/TASK-001-foundation-gap-closure.md && test ! -e docs/04-tasks/active/TASK-001-foundation-gap-closure.md && for id in 01-01 01-02 01-03 01-04 01-05 D-01 D-02 D-03 D-04 D-05 D-06 D-07 D-08 D-09 D-10 D-11 D-12 D-13 D-14 D-15 D-16 C1 C2 C3 C4 C5; do rg -q -- "$id" docs/04-tasks/completed/TASK-001-foundation-gap-closure.md || exit 1; done && rg -q 'TASK-001-foundation-gap-closure' docs/04-tasks/completed/README.md && git diff --check -- docs/04-tasks` | active/completed TASK-001, completed README, and pre-move gate evidence | pending |
| 01-05-03 | 05 | 5 | REQ-product-vision, REQ-product-principles, REQ-goals-non-goals, REQ-engineering-baseline, REQ-storage-architecture, REQ-cli-api, REQ-architecture-principles-quality | D-01, D-02, D-03, D-04, D-05, D-06, D-07, D-08, D-09, D-10, D-11, D-12, D-13, D-14, D-15, D-16 | T-01-CFG, T-01-DOCTOR, T-01-MIG, T-01-AUDIT, T-01-SQL, T-02-ATOMICITY, T-02-PROVENANCE, T-02-CANONICAL, T-02-RECOVERY, T-02-SQL, T-03-LEASE, T-03-RETRY, T-03-CHECKPOINT, T-03-SECRET, T-03-CONCURRENCY, T-04-HOST, T-04-SHUTDOWN, T-04-ENQUEUE, T-04-LEASE, T-04-SECRET, T-04-SCOPE, T-05-ARCH, T-05-EXTERNAL, T-05-SUPPLYCHAIN, T-05-STATUS, T-05-SCOPE, T-05-INFO | Rerun the final gate after the already completed TASK-001 move, record actual outcomes, update validation/task-done rollup/status/changelog, and independently verify every plan, decision, requirement, criterion, probe, threat, and active/completed status. | final cross-record gate | `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo run -q -p xtask -- arch-check && cargo run -q -p xtask -- migrate-check && cargo run -p xtask -- ci && test -f docs/04-tasks/completed/TASK-001-foundation-gap-closure.md && test ! -e docs/04-tasks/active/TASK-001-foundation-gap-closure.md && for id in 01-01 01-02 01-03 01-04 01-05 D-01 D-02 D-03 D-04 D-05 D-06 D-07 D-08 D-09 D-10 D-11 D-12 D-13 D-14 D-15 D-16 REQ-product-vision REQ-product-principles REQ-goals-non-goals REQ-engineering-baseline REQ-storage-architecture REQ-cli-api REQ-architecture-principles-quality C1 C2 C3 C4 C5 PROBE-01 PROBE-02 PROBE-03 PROBE-04 PROBE-05 PROBE-06 PROBE-07 PROBE-08 PROBE-09 PROBE-10 T-01-CFG T-01-DOCTOR T-01-MIG T-01-AUDIT T-01-SQL T-02-ATOMICITY T-02-PROVENANCE T-02-CANONICAL T-02-RECOVERY T-02-SQL T-03-LEASE T-03-RETRY T-03-CHECKPOINT T-03-SECRET T-03-CONCURRENCY T-04-HOST T-04-SHUTDOWN T-04-ENQUEUE T-04-LEASE T-04-SECRET T-04-SCOPE T-05-ARCH T-05-EXTERNAL T-05-SUPPLYCHAIN T-05-STATUS T-05-SCOPE T-05-INFO; do rg -q -- "$id" .planning/phases/01-foundations/01-VALIDATION.md docs/04-tasks/completed/TASK-001-foundation-gap-closure.md docs/06-progress/task-done-rollup.md docs/06-progress/status.md CHANGELOG.md || exit 1; done && git diff --check -- .planning/phases/01-foundations/01-VALIDATION.md docs/04-tasks docs/06-progress/task-done-rollup.md docs/06-progress/status.md CHANGELOG.md` | validation, task-done rollup, status, changelog, completed record | pending |

## Wave 0 and Missing-Coverage Ownership

There is no separate Wave 0 implementation task. Each missing test seam is assigned to the first task that changes it, so a later plan cannot be used as a substitute first proof:

| Missing seam | Owning task | Required proof |
|---|---|---|
| Seven-group real-binary help and fresh-workspace operator path | 01-01-01 | `crates/cli/tests/foundation.rs` creates and runs the real `qai` test. |
| Effective origins, keyed get, redaction, and validation diagnostics | 01-01-02 | Its own config precedence plus real CLI matrix, including identical human/JSON remedy fields. |
| Metadata-only doctor, readiness states, and tampered-chain result | 01-01-03 | Its own fresh/pending/current/restricted/tampered cases. |
| Same-UnitOfWork first proof | 01-02-01 | One commit and one rollback at the new audited composition boundary. |
| Failure matrix and recovery guidance | 01-02-02 | Independent real-SQLite fault injection, not only the first proof. |
| Named checkpoint and retry contracts | 01-03-01 | Task-local jobs contract test before worker expansion. |
| Crash, retry, cancellation, and ownership behavior | 01-03-02 | Task-local real-SQLite cases. |
| Lifecycle audit and operator controls | 01-03-03 | Task-local real-SQLite lifecycle-audit/control cases. |
| Real `qai serve` ownership and joined shutdown | 01-04-01 | Real child process plus application host test. |
| Enqueue-only import and later host-owned terminal state | 01-04-02 | Application and real-binary queued response/terminal observation. |
| Host-backed corpus snapshot boundary | 01-04-03 | Existing `quran` target runs all four trycmd suites. |
| Registry/git policy mutation | 01-05-01 | Inline `xtask/src/arch.rs` tests and `arch-check`. |
| TASK-001 closure and final evidence | 01-05-02, 01-05-03 | Pre-move gate, sole move, then post-move cross-record gate. |

## Requirement Coverage

The seven Phase 1 requirement IDs from `.planning/ROADMAP.md` are all represented below and in the plan frontmatter. This table is a cross-plan audit, not a claim that the broad PRD requirement is fully delivered outside Phase 1.

| Requirement ID | Phase-1 boundary covered | Plans/tasks | Repeatable evidence |
|---|---|---|---|
| REQ-product-vision | Trustworthy Quran-first foundation; later Quran/search/server surfaces are not absorbed. | 01-01, 01-04, 01-05 | `cargo test -p cli --test foundation -- --nocapture`; `cargo test -p cli --test quran -- --nocapture`; scope audit. |
| REQ-product-principles | Exact-text-first, traceability, locality, deny-by-default, redaction, no false consensus. | 01-01 through 01-05 | Config sentinel tests, same-UoW failure injection, no-side-effect doctor, enqueue-only tests, and final workspace gates. |
| REQ-goals-non-goals | No foundation model, remote provider, broker, sectarian authority claim, or new corpus capability. | 01-01, 01-04, 01-05 | `arch-check`, no-install/scope audit, and final cross-record check. |
| REQ-engineering-baseline | Typed errors, validated config, cancellation, real tests, formatting, clippy, and evidence-backed DX. | 01-01 through 01-05 | Every task-local command, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo run -p xtask -- ci`. |
| REQ-storage-architecture | SQLite authority, explicit migrations, shared UoW, durable outbox/jobs. | 01-01 through 01-05 | Readiness tests, real SQLite atomicity/recovery tests, and `cargo run -q -p xtask -- migrate-check`. |
| REQ-cli-api | Executable `qai`, stable groups, JSON/remedies, queued jobs, cancellable `qai serve`. | 01-01, 01-03, 01-04, 01-05 | Real binary foundation, job controls, host/import flows, catalog/help tests, and final gate. |
| REQ-architecture-principles-quality | Correctness, maintainability, extensibility, and machine-enforced boundaries. | 01-01 through 01-05 | Inline architecture mutation tests, `arch-check`, `migrate-check`, full workspace gates, and final status records. |

## Five Roadmap Success Criteria

| ID | Exact Phase 1 criterion | Owning evidence | Gate/evidence command |
|---|---|---|---|
| C1 | Developer can build the workspace and run `qai --help` showing the stable command tree. | 01-01-01; 01-05-03 | `cargo build -p cli --bin qai && target/debug/qai --help && cargo test -p cli --test foundation -- --nocapture` |
| C2 | Operator can configure via CLI > env > file > defaults with validation errors that name the remedy. | 01-01-01, 01-01-02; 01-05-03 | `cargo test -p config --lib && cargo test -p testkit --test config_precedence && cargo test -p cli --test foundation -- --nocapture` |
| C3 | System records provenance and append-only audit events for every state-changing operation. | 01-02-01, 01-02-02, 01-03-03; 01-05-03 | `cargo test -p application --test phase1_foundation && cargo test -p storage-sqlite --test commit_bounds_outbox --test integrity_audit && cargo test -p application --lib audit_bridge` |
| C4 | Background jobs enqueue, lease, checkpoint, and cancel without an external broker. | 01-03-01 through 01-04-03; 01-05-03 | `cargo test -p jobs --lib && cargo test -p storage-sqlite --test recovery_jobs --test phase1_jobs_audit && cargo test -p application --test phase1_jobs_host && cargo test -p cli --test quran -- --nocapture` |
| C5 | `xtask arch-check` passes and CI fails on any forbidden crate dependency. | 01-05-01 and 01-05-03 | `cargo test -p xtask && cargo run -q -p xtask -- arch-check && cargo run -q -p xtask -- migrate-check && cargo run -p xtask -- ci` |

C1–C5 are the only Phase 1 roadmap success criteria. A later-phase criterion cannot substitute for one of these rows.

## Locked Decision Coverage: D-01 through D-16

Every decision copied from `01-CONTEXT.md` is implemented by at least one task and has a named evidence path. The decision IDs are cited in the owning plan actions as well as this matrix.

| Decision | Locked behavior | Owning task(s) | Acceptance evidence |
|---|---|---|---|
| D-01 | Evidence-driven brownfield gap closure; preserve working implementations. | 01-01 through 01-05 | Reuse ledgers in all plans plus the final scope/diff audit. |
| D-02 | ROADMAP and ADRs 0000–0012 govern; no silent phase expansion. | 01-01, 01-02, 01-04, 01-05 | `arch-check`, source-audit tables, and final no-expansion status. |
| D-03 | Repeatable automated, CLI-level, or equivalent evidence closes criteria. | 01-01, 01-02, 01-03, 01-04, 01-05 | Fourteen task-local commands, five criterion rows, and final gate. |
| D-04 | Reuse later Quran/search/server code only as evidence; do not expand it. | 01-01, 01-04, 01-05 | Host-boundary-only snapshots, scope audit, and excluded worktree preservation. |
| D-05 | Migrations remain explicit; ordinary commands report the exact `qai db migrate` remedy. | 01-01-01, 01-01-03, 01-04-01, 01-04-02 | Fresh/pending/current/checksum readiness tests and `migrate-check`. |
| D-06 | `qai doctor` is read-only first-run diagnosis with exact next commands. | 01-01-01, 01-01-03 | Fresh-directory no-side-effect and restricted-file cases. |
| D-07 | Effective config is redacted and origin-bearing; JSON is the machine equivalent. | 01-01-02 | File/env/CLI/default matrix, sentinel redaction, and human/JSON comparison. |
| D-08 | Stable top-level foundation groups are config, db, doctor, job, audit, secret, source. | 01-01-01, 01-01-02 | Real help assertion and unchanged command-tree regression. |
| D-09 | Audit source lifecycle, approvals, activation/rollback, provenance, imports, and job/admin mutations. | 01-02-01, 01-02-02, 01-03-03, 01-04-02 | One-UoW composition, lifecycle tests, and importer staging-only negatives. |
| D-10 | Domain, provenance, audit, and required outbox records commit all-or-nothing. | 01-02-01, 01-02-02, 01-03-03, 01-04-02 | Failure injection at every required boundary; no partial rows. |
| D-11 | Job enqueue, lease, completion, failure, and cancellation events are persisted. | 01-03-02, 01-03-03, 01-04-01, 01-04-02 | Real SQLite lifecycle audit rows and host consumption. |
| D-12 | `qai audit verify` is the authority for ordering, hashes, continuity, and recovery guidance. | 01-01-01, 01-01-03, 01-02-02, 01-05-03 | Shared verifier result, tamper/gap tests, and final criterion evidence. |
| D-13 | Long-lived `qai serve` hosts execute workers; one-shot commands enqueue only. | 01-04-01, 01-04-02, 01-04-03 | Real child host, queued response, and host-backed corpus flows. |
| D-14 | Named durable checkpoints support safe idempotent resume. | 01-03-01, 01-03-02, 01-04-01 | Immediate checkpoint visibility and expired-lease recovery. |
| D-15 | Retry is bounded per kind with backoff/jitter; exhaustion requires explicit retry. | 01-03-01, 01-03-02, 01-03-03, 01-04-02 | Per-kind exhaustion and operator retry cases. |
| D-16 | Cancellation is cooperative and reports cancelled-at-boundary, completed-before-observation, or missed-boundary. | 01-03-02, 01-03-03, 01-04-01, 01-04-02 | Boundary timing, ownership, durable request, and joined shutdown tests. |

## Spec-Less Probe Ledger

The ten deterministic report probes are individually retained and dispositioned. `retained-assumption` is intentional: the foundation supplies executable evidence for the named seam while the broader product-domain claim remains outside Phase 1. A probe is not silently converted into a pass.

| Probe ID | Requirement / edge | Disposition | Executable coverage in this phase | Retained boundary |
|---|---|---|---|---|
| PROBE-01 | REQ-product-principles / exact-text-first | retained-assumption | 01-02 importer staging, approval, and canonical-write negatives; 01-04 existing Quran flows remain unchanged outside host synchronization. | Full corpus-domain exactness is broader than foundation tests. |
| PROBE-02 | REQ-product-principles / traceability and no fabrication | retained-assumption | 01-02 same-UoW rollback, sequence/hash, and tamper cases; 01-03 lifecycle audit cases. | Scholarly-claim evaluation is not a Phase 1 deliverable. |
| PROBE-03 | REQ-product-principles / locality | retained-assumption | Local real-binary paths, local SQLite/loopback host, and 01-05 external-edge policy. | Remote modes are not exercised. |
| PROBE-04 | REQ-product-principles / deny-by-default and no-false-consensus | retained-assumption | 01-01 no-side-effect doctor, 01-04 enqueue-only behavior, and final scope audit. | Religious-consensus evaluation remains outside this phase. |
| PROBE-05 | REQ-product-vision | retained-assumption | Quran-first boundary, stable groups, real `qai serve`, and source/scope audit. | The complete multi-domain platform vision is not fully acceptance-tested here. |
| PROBE-06 | REQ-goals-non-goals | retained-assumption | No model/provider/broker/migration/package install task; final dependency and scope checks. | Later corpus, remote, and religious-authority capabilities remain out of scope. |
| PROBE-07 | REQ-engineering-baseline | retained-assumption | Fourteen task-local test runs plus fmt, clippy, full workspace tests, `arch-check`, `migrate-check`, and `xtask ci`. | The complete PRD engineering matrix exceeds this phase. |
| PROBE-08 | REQ-storage-architecture | retained-assumption | Readiness/no-side-effect, atomicity, real-SQLite queue/recovery, and `migrate-check` evidence. | PostgreSQL portability is not exercised. |
| PROBE-09 | REQ-cli-api | retained-assumption | Real `qai` foundation, job controls, enqueue-only import, and `qai serve` host tests. | Later versioned API domains remain outside scope. |
| PROBE-10 | REQ-architecture-principles-quality | retained-assumption | Inline registry/git mutation cases in `xtask/src/arch.rs` and final `arch-check`. | Broader UX/performance requirements are not reimplemented. |

## Threat Predicate Register

The following is the complete union of the threat predicates introduced by plans 01-01 through 01-05. Each has a concrete mitigation and a task-local or final-gate proof; none is left as a prose-only claim.

| Threat ID | Plan/task owner | STRIDE category | Severity | Disposition | Concrete mitigation and evidence |
|---|---|---|---|---|---|
| T-01-CFG | 01-01-01, 01-01-02 | Information disclosure | high | mitigate | Apply redaction before human/JSON emission; sentinel tests in foundation/config targets. |
| T-01-DOCTOR | 01-01-01, 01-01-03 | Tampering / elevation of privilege | high | mitigate | Replace write probes with metadata inspection; assert fresh directory remains empty. |
| T-01-MIG | 01-01-01, 01-01-03 | Tampering | high | mitigate | Reserve creating constructors/migrations for explicit `qai db migrate`; test missing/pending/checksum states. |
| T-01-AUDIT | 01-01-01, 01-01-03, 01-02-02 | Repudiation / tampering | high | mitigate | Share persisted verifier result between audit CLI and doctor; test gaps/tampered sequences. |
| T-01-SQL | 01-01-03 | Tampering | medium | mitigate | Use existing SQLx bound parameters and catalog/readiness queries. |
| T-02-ATOMICITY | 01-02-01, 01-02-02 | Tampering / repudiation | critical | mitigate | Enforce one UoW and fault-inject domain, provenance, outbox, audit, and commit boundaries. |
| T-02-PROVENANCE | 01-02-01 | Tampering / information disclosure | high | mitigate | Require provenance and required outbox records before the single audited commit. |
| T-02-CANONICAL | 01-02-01 | Elevation of privilege | high | mitigate | Preserve approval token and canonical-writer checks; add importer staging-only negatives. |
| T-02-RECOVERY | 01-02-02 | Repudiation | high | mitigate | Recompute sequence, predecessor links, and row hashes; expose `qai audit verify` remedy. |
| T-02-SQL | 01-02-02 | Tampering | medium | mitigate | Keep real-SQLite queries parameterized and repository-mediated. |
| T-03-LEASE | 01-03-01, 01-03-02, 01-03-03 | Elevation of privilege / denial of service | high | mitigate | Require current lease owner for heartbeat/checkpoint/terminal writes; cancellation only requests. |
| T-03-RETRY | 01-03-01, 01-03-02 | Denial of service | high | mitigate | Use bounded per-kind attempts/backoff/jitter, dead-letter exhaustion, and explicit retry. |
| T-03-CHECKPOINT | 01-03-01, 01-03-02 | Tampering / repudiation | high | mitigate | Persist named checkpoints at handler boundaries and recover the last committed value. |
| T-03-SECRET | 01-03-03 | Information disclosure | high | mitigate | Allowlist lifecycle metadata and redact checkpoint/error/payload JSON before emission. |
| T-03-CONCURRENCY | 01-03-02, 01-03-03 | Tampering | medium | mitigate | Preserve atomic claim predicates and real-SQLite double-claim/expiry ownership tests. |
| T-04-HOST | 01-04-01, 01-04-03 | Denial of service / elevation of privilege | high | mitigate | One readiness-gated host, bounded polling, typed error propagation, and joined lifetime. |
| T-04-SHUTDOWN | 01-04-01 | Tampering / repudiation | high | mitigate | Request cooperative cancellation, await durable checkpoint/terminal persistence, and test signal shutdown. |
| T-04-ENQUEUE | 01-04-02, 01-04-03 | Tampering | high | mitigate | Validate before enqueue, bind job parameters, use audited queue seams, and return an inspectable id. |
| T-04-LEASE | 01-04-01, 01-04-02 | Elevation of privilege / denial of service | high | mitigate | Reuse 01-03 owner checks; never clear a lease from one-shot or serve shutdown paths. |
| T-04-SECRET | 01-04-02, 01-04-03 | Information disclosure | high | mitigate | Redact queued/error output and keep raw payloads/checkpoints out of host diagnostics. |
| T-04-SCOPE | 01-04-01, 01-04-03 | Elevation of privilege | medium | mitigate | Register existing handlers only and preserve approval/canonical boundaries. |
| T-05-ARCH | 01-05-01, 01-05-03 | Tampering / elevation of privilege | critical | mitigate | Classify path/registry/git sources against explicit per-crate rules and retain mutation tests. |
| T-05-EXTERNAL | 01-05-01, 01-05-03 | Tampering / supply-chain | high | mitigate | Reject unlisted external source/name pairs and test forbidden registry/git cases. |
| T-05-SUPPLYCHAIN | 01-05-01, 01-05-03 | Tampering | high | mitigate | Add no package/install task; retain cargo-deny as a separate gate. |
| T-05-STATUS | 01-05-02, 01-05-03 | Repudiation | high | mitigate | Capture gate evidence before the sole move and cross-check all task/progress records. |
| T-05-SCOPE | 01-05-02, 01-05-03 | Elevation of privilege | medium | mitigate | Audit all source types and preserve deferred/owner-gated and unrelated worktree paths. |
| T-05-INFO | 01-05-03 | Information disclosure | medium | mitigate | Record command outcomes and metadata, never raw secret-bearing payload output. |

No package-manager install task exists, so a package-legitimacy checkpoint is not applicable. Existing dependencies remain pinned by the current lockfile and are not reclassified as new approvals.

## Prohibitions and Negative Invariants

These predicate checks are separate from the threat IDs and must remain visible in task evidence:

| Predicate | Required negative result | Coverage |
|---|---|---|
| No secret output | Secret sentinel is absent from config, doctor, job, host, and JSON output. | 01-01-02, 01-03-03, 01-04-02. |
| No doctor writes | Fresh data directory, database, WAL, object directory, and probe file remain absent. | 01-01-01, 01-01-03. |
| No implicit migration or worker | Ordinary commands do not create/migrate; one-shot import does not drain a worker. | 01-01-03, 01-04-01, 01-04-02. |
| No row-count audit pass | Gaps/tampered sequences are non-valid and point to `qai audit verify`. | 01-01-03, 01-02-02. |
| No partial durable commit | Failure at any required domain/provenance/outbox/audit boundary leaves no partial row. | 01-02-01, 01-02-02. |
| No compensating delete | A failed transaction is rolled back, not repaired by a second mutation. | 01-02-01, 01-02-02. |
| No importer approval bypass | Importer stages only and cannot mint/bypass an approval token. | 01-02-01, 01-04-02. |
| No automatic non-idempotent retry | Non-idempotent work dead-letters rather than auto-retrying. | 01-03-01, 01-03-02. |
| No unowned lease clearing | Only the current owner finalizes/heartbeats; CLI cancellation sets a request only. | 01-03-02, 01-03-03, 01-04-01. |
| No false cancellation claim | One of the three locked dispositions is persisted and exposed. | 01-03-02, 01-03-03, 01-04-01. |
| No unsafe shutdown | Serve waits for a durable checkpoint/terminal state and joins the worker. | 01-04-01, 01-04-03. |
| No second queue authority or broker | SQLite and the existing job ports remain the only execution path. | 01-03-01, 01-04-01. |
| No raw job payload disclosure | Allowlisted/redacted metadata only in host, CLI, and audit output. | 01-03-03, 01-04-02. |
| No unapproved external edge | Registry/git/path mutation cases fail closed under explicit policy. | 01-05-01. |
| No unverified completion | A criterion/task closes only with a recorded command and evidence link. | 01-05-02, 01-05-03. |
| No unrelated scope edit | Excluded live worktree paths remain byte-for-byte untouched and unstaged. | 01-01 baseline, 01-05-02, 01-05-03. |

## TASK-001 Lifecycle and Cross-Record Contract

| Order | Owner | Required state/action | Automated check |
|---:|---|---|---|
| 1 | 01-01-01 | Create `docs/04-tasks/active/TASK-001-foundation-gap-closure.md` and active index entry before production edits; status is `Active`. | `test -f docs/04-tasks/active/TASK-001-foundation-gap-closure.md` |
| 2 | 01-01 through 01-04 | Append task evidence while the task remains active; do not move or mark completed. | `rg -n 'TASK-001-foundation-gap-closure|D-01|D-16' docs/04-tasks/active/TASK-001-foundation-gap-closure.md` |
| 3 | 01-05-01 | Complete the architecture mutation/gate evidence while active. | `cargo test -p xtask && cargo run -q -p xtask -- arch-check` |
| 4 | 01-05-02 | Run and capture the pre-move final gate; only after green, move active TASK-001 to completed and update the completed task/index records. Leave the task-done rollup untouched. | `test -f docs/04-tasks/completed/TASK-001-foundation-gap-closure.md && test ! -e docs/04-tasks/active/TASK-001-foundation-gap-closure.md` |
| 5 | 01-05-03 | Rerun/record the final gate and cross-record check; update validation, task-done rollup, status, and changelog. Do not move TASK-001 again. | Final-gate command plus the ID loops in the task matrix. |

The completed record must contain, at minimum, the following exact groups: all five plan IDs `01-01` through `01-05`; all seven `REQ-*` IDs; all five `C1`–`C5` criterion IDs; all `D-01` through `D-16`; all ten `PROBE-01` through `PROBE-10` dispositions; all threat IDs in the threat register; the final command outcomes; and one statement that owner-gated/deferred items remain outside Phase 1. The active path must be absent after the move, the completed path present, and the status/rollup/changelog must describe the same completion event. No unrelated worktree path may be staged or edited.

## Source Audit

| Source | IDs/features audited | Plans | Status | Disposition |
|---|---|---|---|---|
| GOAL | Roadmap Phase 1 goal and C1–C5 | 01-01 through 01-05 | COVERED | Every criterion has a task and repeatable command. |
| REQ | REQ-product-vision, REQ-product-principles, REQ-goals-non-goals, REQ-engineering-baseline, REQ-storage-architecture, REQ-cli-api, REQ-architecture-principles-quality | 01-01 through 01-05 | COVERED | All seven IDs appear in plan frontmatter and above. |
| RESEARCH | FND-01 through FND-07, ARCH-01 through ARCH-05, CONSTRAINT-01 through CONSTRAINT-03, and the resolved RQ-01 through RQ-05 section | 01-01 through 01-05 | COVERED | Plan source audits and the decision/probe ledgers preserve each constraint. |
| CONTEXT | D-01 through D-16 and the agent's discretion | 01-01 through 01-05 | COVERED | Each locked decision has an owner task and evidence row. |

Deferred ideas, the Phase-2 dataset/license decision, remote deployment, and later Quran/search/server capabilities are exclusions, not missing Phase 1 work. No item in the four source tables is left unplanned.

## Manual and Owner Boundaries

There are no implementation tasks without automated verification. The following are deliberate owner/external boundaries and must not be mislabeled as automated green evidence:

- Dataset licensing and owner sign-off remain a Phase-2 input; this phase does not select or activate a corpus.
- Remote PostgreSQL/Qdrant/TLS/deployment and later product surfaces remain out of scope.
- A locally unavailable optional tool such as `cargo-deny` must be recorded as unavailable; the required Phase 1 `arch-check`, `migrate-check`, and `xtask ci` outcomes still need their actual results.
- The final verifier may perform a goal-backward review after the automated gate, but it must not close a criterion from prose alone.

## Validation Sign-Off

- [x] Fourteen task rows are present and each has an automated command.
- [x] All five plans and all seven Phase 1 requirement IDs are represented.
- [x] All five roadmap criteria and D-01 through D-16 have task/evidence mappings.
- [x] All ten spec-less probes are named and dispositioned.
- [x] All 27 threat predicates and negative prohibitions have concrete coverage.
- [x] Real `qai serve` evidence is assigned to each host/import/corpus task.
- [x] `xtask` architecture tests are inline in `xtask/src/arch.rs`.
- [x] TASK-001 ownership, pre-move gate, sole move, and post-move consistency checks are explicit.
- [ ] Actual task rows are green after execution.
- [ ] `wave_0_complete: true` and `nyquist_compliant: true` are set only after the final gate and independent cross-record check pass.

**Approval:** pending execution and final verifier review
