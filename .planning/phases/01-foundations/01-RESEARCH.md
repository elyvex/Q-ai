# Phase 1: Foundations - Research

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
### Brownfield Gap Closure
- **D-01:** Treat Phase 1 as an evidence-driven brownfield gap-closure phase: map every Phase 1 success criterion to current code and at least one repeatable check, preserve working implementations, and create implementation work only for missing behavior or weak evidence.
- **D-02:** The `.planning/ROADMAP.md` Phase 1 contract and locked ADRs 0000–0012 govern completion. Existing plans and documentation may supply evidence but must not silently expand the phase.
- **D-03:** Code inspection plus a repeatable automated, CLI-level, or equivalent check is sufficient evidence for an existing criterion. Add dedicated acceptance coverage where equivalent evidence is absent or confidence is weak.
- **D-04:** Reuse existing Quran, search, and server code only as foundation evidence. Do not absorb additional capabilities from those later phases into Phase 1.

### Operator Bootstrap Path
- **D-05:** Keep database migrations explicit. Ordinary commands must report pending migrations and the exact `qai db migrate` remedy rather than auto-mutating the schema.
- **D-06:** Make `qai doctor` the read-only first-run entry point, reporting configuration, data-directory, migration, storage, audit, and job readiness with exact next commands.
- **D-07:** Ordinary configuration inspection defaults to a redacted effective configuration with value origins; `--json` provides the machine-readable equivalent. Secret values are never displayed.
- **D-08:** The stable Phase 1 command-tree contract is the existing top-level foundation groups: `config`, `db`, `doctor`, `job`, `audit`, `secret`, and `source`. Later feature groups are independently versioned surfaces.

### Audit Transaction Coverage
- **D-09:** Audit durable domain state changes: source lifecycle, approvals, activation/rollback, provenance, imports, and job/admin mutations. Read-only diagnostics and derived-cache rebuilds are not audited unless they alter authoritative state.
- **D-10:** For an audited durable mutation, commit the domain mutation, provenance record, audit event, and required outbox event all-or-nothing. Failure to write any required record rolls back the whole operation.
- **D-11:** Persist job lifecycle audit events for enqueue, lease, completion, failure, and cancellation. Each handler's domain-result writes remain atomic with their own provenance/audit requirements.
- **D-12:** Make `qai audit verify` a required foundation check that validates event ordering, row hashes, chain continuity, and provides recovery guidance when verification fails.

### Job Worker Lifecycle
- **D-13:** Execute queued jobs by default inside long-lived hosts such as `qai serve`; one-shot CLI commands may enqueue work but must not silently start workers.
- **D-14:** Handlers persist durable named checkpoints sufficient to resume safely after process crash or lease expiry. Resume continues after the last committed checkpoint and handlers remain idempotent.
- **D-15:** Retry transient failures with bounded exponential backoff plus jitter, using per-kind attempt limits. Exhausted jobs enter an inspectable failed state and require explicit retry.
- **D-16:** Cancellation is cooperative: persist the request, stop at handler checkpoints, finalize safely, and report whether the job was cancelled, completed before observing cancellation, or missed the requested boundary.

### the agent's Discretion
- Exact checkpoint payload schemas, backoff constants, and operator-facing wording may follow existing project conventions so long as the locked behavior above is preserved.

### Deferred Ideas (OUT OF SCOPE)
- Additional Quran search, linguistic, graph, server, and streaming capabilities remain in their roadmap phases, even where partial implementations already exist.
- Remote PostgreSQL/Qdrant deployment, TLS, backup operations beyond foundation primitives, and production management remain in Phase 12.
- The initial Quran dataset and license decision (ADR-0101) remains a Phase 2 input and does not alter the Phase 1 foundation contract.
</user_constraints>

> **Provenance convention.** `[VERIFIED: path:lines]` means the cited source definition was opened in this research session; `[VERIFIED: live probe]` means the behavior was observed by running the real binary or a local tool. `[CITED: URL]` is an official-documentation reference, and `[ASSUMED]` marks a future implementation choice that still needs planner or owner confirmation. The user-constraint block above is copied verbatim from `01-CONTEXT.md` and is treated as locked input rather than an independently asserted research claim. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:15-45]

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REQ-product-vision | Platform scope and the trustworthy Quran-first product direction. | The foundation work stays limited to configuration, storage, jobs, provenance, audit, diagnostics, and the stable CLI surface; it does not absorb Quran/search/server capabilities. [VERIFIED: .planning/ROADMAP.md:30-40] [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:18-22] |
| REQ-product-principles | Exact-text-first, traceability, layer separation, no fabrication, locality, deny-by-default, and no false consensus. | Atomic provenance/audit/outbox coverage, redacted effective configuration, and a read-only doctor directly support traceability, locality, and fail-closed behavior. [VERIFIED: AGENTS.md:111-124] [VERIFIED: .planning/ROADMAP.md:34-40] |
| REQ-goals-non-goals | Complete the foundation without building a foundation model or claiming sectarian authority. | The recommended implementation reuses the existing Rust workspace and SQLite authority; no model, vector, remote-provider, or religious-authority capability is introduced. [VERIFIED: .planning/PROJECT.md:43-50] [VERIFIED: .planning/PROJECT.md:52-58] |
| REQ-engineering-baseline | Async architecture, typed errors, validated configuration, observability, dependency principles, security, testing, and DX. | Existing typed configuration/SQLite/job/audit seams are reused; the gap register targets missing CLI wiring, read-only diagnostics, lifecycle events, cancellation outcomes, and acceptance tests. [VERIFIED: .planning/REQUIREMENTS.md:58-59] [VERIFIED: .planning/codebase/ARCHITECTURE.md:79-89] |
| REQ-storage-architecture | SQLite authority, portable storage contracts, explicit migrations, and durable outbox primitives. | The existing `UnitOfWork`, checksummed migration runner, and SQLite queue are preserved; the plan adds explicit migration readiness and closes atomic audit coverage rather than replacing storage. [VERIFIED: crates/storage-sqlite/src/lib.rs:230-307] [VERIFIED: crates/storage-sqlite/src/migrate.rs:168-219] |
| REQ-cli-api | Executable `qai`, stable command tree, machine-readable output, and cancellable long-running operations. | The current Clap tree and `qai --help` are retained; the missing work is effective configuration/origins, explicit remediation, job cancellation/retry, and a long-lived worker host. [VERIFIED: crates/cli/src/lib.rs:14-122] [VERIFIED: crates/cli/src/lib.rs:210-352] |
| REQ-architecture-principles-quality | Correctness, maintainability, extensibility, and enforced architectural boundaries. | `xtask arch-check` and CI already run, but the checker only evaluates workspace/path edges; external registry restrictions must be made enforceable without broad crate coupling. [VERIFIED: xtask/src/arch.rs:102-164] [VERIFIED: .github/workflows/ci.yml:47-64] |
</phase_requirements>

## Project Constraints (from AGENTS.md)

- **Read the project briefing in order.** Read `AGENTS.md`, `docs/00-overview/project-overview.md`, `docs/03-plan/current-plan.md`, the active phase plan, `docs/06-progress/status.md`, and `docs/05-followups/open-questions.md` before implementation. [VERIFIED: AGENTS.md:44-52]
- **Use the repository's documentation taxonomy.** Keep planning under `docs/03-plan/`, technical knowledge under `docs/07-technical/`, follow-ups under `docs/05-followups/`, and progress under `docs/06-progress/`; do not mix those concerns. [VERIFIED: .agent/instructions.md:16-24]
- **Every implementation task needs a repository task ID and a completed-state update.** Do not mark work complete until tests, lint/checks, acceptance criteria, rollup, follow-ups, and changelog treatment are addressed. [VERIFIED: AGENTS.md:66-74] [VERIFIED: .agent/definition-of-done.md:1-12]
- **Keep the architecture layered.** Domain code must not depend on `api`, `server`, `tui`, `cli`, or a concrete provider; route new cross-crate access through `application`. [VERIFIED: .agent/coding-rules.md:5-13]
- **Preserve safety invariants.** Canonical rows require an approval-gated writer; derived artifacts record their inputs; permissions, network, filesystem, and side effects are deny-by-default; nothing that can modify data runs inside `doctor`; secrets are never logged, embedded in code, or written to plain config. [VERIFIED: .agent/coding-rules.md:13-22]
- **Do not silently expand the phase.** The project overview identifies Quran/search/graph/web/TUI/agent surfaces as later or emerging work, and the Phase 1 context explicitly limits the work to brownfield foundation gaps. [VERIFIED: docs/00-overview/project-overview.md:3-10] [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:7-12]
- **Preserve unrelated worktree changes.** This research session observed concurrent modifications in Phase-2/search/morphology files and a deleted discussion checkpoint; the research commit must stage only `01-RESEARCH.md`. [VERIFIED: live git status, 2026-09-24]

## Summary

This is a brownfield gap-closure phase, not a greenfield foundation build. The repository already contains the Rust workspace, layered configuration loader, SQLite adapter, shared `UnitOfWork`, audit hash-chain bridge, durable job queue, Clap command tree, and architecture/CI gates; targeted tests and live probes confirm that most primitives work. [VERIFIED: Cargo.toml:3-18] [VERIFIED: crates/config/src/lib.rs:219-259] [VERIFIED: crates/storage-sqlite/src/lib.rs:230-307] [VERIFIED: crates/application/src/audit_bridge.rs:200-270] [VERIFIED: crates/jobs/src/queue.rs:16-68] [VERIFIED: xtask/src/arch.rs:144-164]

The material gaps sit at the seams between those primitives. The CLI loads defaults in its handlers instead of using the effective `(Config, OriginMap)` returned by the config crate; a file port of `4242` plus environment port `5151` is printed as the default `8737`, and `config get server.port` ignores the key. [VERIFIED: crates/cli/src/lib.rs:381-405] [VERIFIED: crates/cli/src/lib.rs:421-449] [VERIFIED: live probe: `QAI__SERVER__PORT=5151 ... config show --json` printed `server.port=8737`] The doctor module claims read-only behavior but its data-directory and object-store checks call `create_dir_all` and write `.qai-probe`; its audit check reports PASS based only on the number of rows, while `qai audit verify` correctly reports a tampered sequence. [VERIFIED: crates/cli/src/doctor.rs:1-8] [VERIFIED: crates/cli/src/doctor.rs:237-254] [VERIFIED: crates/cli/src/doctor.rs:528-546] [VERIFIED: crates/cli/src/doctor.rs:441-458] [VERIFIED: live probe: doctor printed `audit.chain_valid` status `pass` after the persisted verifier reported `tampered_sequences: [1]`]

The recommended plan is four bounded work packages: (1) thread effective configuration and origins through the CLI and make doctor/migration readiness genuinely read-only; (2) make every authoritative mutation use one audited unit of work and make doctor delegate to the persisted audit verifier; (3) move default execution to a long-lived `qai serve` worker host, persist named checkpoints, add per-kind retry and explicit retry/cancel outcomes, and audit job lifecycle transitions; and (4) extend `xtask arch-check` to enforce the intended external dependency policy. Existing Quran/search/server code should be touched only where the Phase 1 worker/host decision requires wiring. [ASSUMED: recommended work-package decomposition and the proposed `qai serve` worker-host seam]

**Primary recommendation:** plan a brownfield hardening slice centered on one shared configuration/readiness path, one shared audited `UnitOfWork` boundary, and one long-lived worker lifecycle; do not add a package or rebuild a foundation service that already has passing evidence. [ASSUMED: recommended implementation shape]

**Research confidence:** MEDIUM overall. Internal code, live probes, and test output are high-confidence; current external API guidance is medium-confidence; exact future checkpoint schemas, migration numbering, and ASVS control identifiers remain uncommitted choices. [VERIFIED: targeted test output, 2026-09-24] [CITED: https://tokio.rs/tokio/topics/shutdown] [CITED: https://github.com/OWASP/ASVS]

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Command parsing, stable help, global flags, and exit mapping | Browser / Client (`cli`) | Application | `Cli` owns the top-level Clap tree and dispatch; application services should not own user-facing command syntax. [VERIFIED: crates/cli/src/lib.rs:14-122] [VERIFIED: crates/cli/src/lib.rs:210-352] |
| Effective configuration, precedence, validation, interpolation, and origin metadata | Application / API boundary | Config, CLI | `Config::load` already owns the four-layer merge and returns `OriginMap`; the missing work is to preserve and expose that result rather than reimplement precedence. [VERIFIED: crates/config/src/lib.rs:219-259] [VERIFIED: crates/config/src/origin.rs:4-68] |
| Read-only health and first-run diagnosis | Application | CLI doctor, SQLite adapter | The application builds a read-only probe and the CLI renders remedies; filesystem checks must remain metadata-only. [VERIFIED: crates/application/src/db.rs:26-49] [VERIFIED: crates/application/src/db.rs:128-199] [VERIFIED: crates/cli/src/doctor.rs:178-207] |
| Migration discovery, checksum verification, and explicit migration | Storage adapter | Application, CLI | The SQLite migration runner already owns checksummed forward migrations; the application/CLI should expose readiness and require `qai db migrate`, not invoke it implicitly. [VERIFIED: crates/storage-sqlite/src/migrate.rs:168-219] [VERIFIED: crates/application/src/db.rs:45-84] |
| Durable job queue, leases, retry scheduling, cancellation requests, and checkpoints | Database / Storage | Jobs, Application | SQLite is the durable authority; `jobs` owns the backend-neutral state machine and `application` owns the concrete adapter/worker assembly. [VERIFIED: migrations/sqlite/0004_jobs.up.sql:1-42] [VERIFIED: crates/jobs/src/queue.rs:16-68] [VERIFIED: crates/application/src/job_queue.rs:19-135] |
| Long-lived worker execution | Application / API boundary | Jobs, Tokio | `qai serve` is the existing long-lived host entry point; it should own worker task lifetime while the jobs crate remains backend-neutral. [VERIFIED: crates/cli/src/lib.rs:301-345] [CITED: https://tokio.rs/tokio/topics/shutdown] |
| Domain mutation + provenance + audit + outbox | Database / Storage | Application, Audit, Provenance | The shared SQLite transaction already gives repositories one commit boundary; the application must compose all required writers before committing. [VERIFIED: crates/storage-sqlite/src/lib.rs:230-307] [VERIFIED: crates/storage/src/workflows.rs:27-109] |
| Audit-chain verification and recovery guidance | Audit / Application | Storage, CLI | The audit crate owns hash computation, the bridge maps rows, and the CLI owns the operator-facing verification result. [VERIFIED: crates/application/src/audit_bridge.rs:200-270] [VERIFIED: crates/cli/src/lib.rs:257-286] |
| Workspace and external dependency enforcement | Build / CI (`xtask`) | Cargo metadata, CI | `arch-check` is the machine gate; it must inspect the metadata dependency edges that the project claims to forbid, not only path edges. [VERIFIED: xtask/src/arch.rs:102-164] [VERIFIED: .github/workflows/ci.yml:47-64] |

## Phase 1 Success-Criterion Evidence Matrix

The matrix is intentionally five rows, one per locked Phase 1 success criterion. “Reuse” means the planner should not create a replacement task for working behavior; “harden” means create work only at the demonstrated seam.

| # | Success criterion | Existing implementation | Repeatable evidence | Gap | Planned task if needed |
|---|---|---|---|---|---|
| 1 | Developer can build the workspace and run `qai --help` showing the stable command tree. [VERIFIED: .planning/ROADMAP.md:34-40] | The workspace builds, the `Cli` enum has the foundation groups, and the real help output lists `config`, `db`, `doctor`, `job`, `audit`, `secret`, and `source`. [VERIFIED: crates/cli/src/lib.rs:76-110] [VERIFIED: live probe: `target/debug/qai --help`] | `cargo build -p cli --bin qai`; `target/debug/qai --help`; `cargo test -p cli --test catalog` (2 passed). [VERIFIED: live build/test output, 2026-09-24] | The top-level tree exists, but no dedicated Phase-1 help contract asserts the seven groups independently of later Quran/search surfaces. [VERIFIED: crates/cli/src/lib.rs:76-122] | **FND-01:** add a focused foundation CLI contract test/snapshot and keep later feature groups independently versioned. [ASSUMED: proposed test location] |
| 2 | Operator can configure via CLI > env > file > defaults with validation errors that name the remedy. [VERIFIED: .planning/ROADMAP.md:34-40] | The config crate implements the merge order and validation, and tests pass for defaults, file, CLI, and selected env overrides. [VERIFIED: crates/config/src/lib.rs:219-297] [VERIFIED: crates/testkit/tests/config_precedence.rs:1-74] | `cargo test -p config --lib` (32 passed); `cargo test -p testkit --test config_precedence` (5 passed); live file+env probe printed `server.port=8737` instead of the expected effective value. [VERIFIED: live test/probe output, 2026-09-24] | CLI handlers discard loaded origins, use `Config::default`, ignore `get` keys, and do not pass a complete CLI override map; the current `ConfigAction` block defines `Show`, `Get`, and `Validate` but no `Set` variant; env/CLI setters cover only a subset of `Config` fields and the default origin map is incomplete. [VERIFIED: crates/cli/src/lib.rs:124-145] [VERIFIED: crates/cli/src/lib.rs:381-449] [VERIFIED: crates/config/src/lib.rs:303-325] [VERIFIED: crates/config/src/lib.rs:544-747] [VERIFIED: crates/config/src/lib.rs:751-873] | **FND-02:** thread one loaded effective-config/origin object through dispatch, implement redacted/origin output and keyed lookup, and close the precedence matrix without inventing a new persistence format. [ASSUMED: recommended config API shape] |
| 3 | System records provenance and append-only audit events for every state-changing operation. [VERIFIED: .planning/ROADMAP.md:34-40] | `UnitOfWork` shares one SQLite transaction; the audit bridge computes and appends chained events; activation and persisted verification have tests. [VERIFIED: crates/storage-sqlite/src/lib.rs:230-307] [VERIFIED: crates/application/src/audit_bridge.rs:200-270] [VERIFIED: crates/application/src/audit_bridge.rs:299-343] | `cargo test -p storage-sqlite --test integrity_audit` (2 passed); `cargo test -p application audit_bridge::tests::persisted_verification_checks_hashes_links_and_gaps`; live `qai audit verify` returned `{"valid":false,"tampered_sequences":[1]}` and exit `3`. [VERIFIED: live test/probe output, 2026-09-24] | `record_source_activation` and `record_provenance_write` write outbox/domain records without an audit event; job lifecycle events are not emitted in the reviewed worker path; doctor reports a tampered chain as PASS. [VERIFIED: crates/storage/src/workflows.rs:27-109] [VERIFIED: crates/jobs/src/worker.rs:100-205] [VERIFIED: crates/cli/src/doctor.rs:441-458] | **FND-03:** introduce a reusable audited mutation composition, route every authoritative path through it, and make doctor call the persisted verifier. [ASSUMED: proposed helper name is a planner choice] |
| 4 | Background jobs enqueue, lease, checkpoint, and cancel without an external broker. [VERIFIED: .planning/ROADMAP.md:34-40] | SQLite storage, leases, retry/backoff, recovery, durable cancellation flag, and context cancellation primitives exist; 22 jobs tests and 4 recovery tests pass. [VERIFIED: crates/storage-sqlite/src/lib.rs:818-1037] [VERIFIED: crates/jobs/src/worker.rs:18-220] [VERIFIED: crates/storage-sqlite/tests/recovery_jobs.rs:52-135] | `cargo test -p jobs --lib` (22 passed); `cargo test -p storage-sqlite --test recovery_jobs` (4 passed); `cargo test -p application --lib job_queue` (2 relevant tests passed in the application suite). [VERIFIED: live test output, 2026-09-24] | Context checkpoints are memory-only and are persisted only on failure; the worker has one global attempt policy; the CLI refuses cancel and has no retry; `build_worker` is not hosted by `qai serve`; `quran import` calls `run_until_idle` inline. [VERIFIED: crates/jobs/src/lib.rs:225-325] [VERIFIED: crates/jobs/src/worker.rs:144-205] [VERIFIED: crates/application/src/job_queue.rs:128-135] [VERIFIED: crates/application/src/quran.rs:1124-1169] | **FND-04:** add durable named checkpoint persistence, per-kind policy, explicit retry/cancel commands and outcomes, lifecycle audit events, and a `qai serve` host; make one-shot commands enqueue only. [ASSUMED: recommended worker-host and outcome schema] |
| 5 | `xtask arch-check` passes and CI fails on any forbidden crate dependency. [VERIFIED: .planning/ROADMAP.md:34-40] | The allowlist and pure `violations()` function pass current path/workspace-edge tests, and CI runs `arch-check` and `migrate-check`. [VERIFIED: xtask/src/arch.rs:102-164] [VERIFIED: xtask/allowlist.toml:1-10] [VERIFIED: .github/workflows/ci.yml:47-64] | `cargo run -q -p xtask -- arch-check` printed `arch-check: OK — no forbidden dependency edges.`; `cargo test -p xtask` (19 passed). [VERIFIED: live command output, 2026-09-24] | `Dependency.source` is read, but only `source.is_none()` path edges are checked; the allowlist comments reserve external rules while the implementation ignores registry dependencies. [VERIFIED: xtask/src/arch.rs:36-45] [VERIFIED: xtask/src/arch.rs:117-139] [VERIFIED: xtask/allowlist.toml:3-7] | **FND-05:** add an explicit external dependency allow/deny policy, mutation tests for a forbidden registry edge, and retain the CI gate. [ASSUMED: policy representation and external list are planner choices] |

## Current Evidence to Reuse

Do not rebuild these working seams:

- `Config::load` returns the effective config and `OriginMap` in precedence order, and the config crate has 32 passing unit tests. [VERIFIED: crates/config/src/lib.rs:219-259] [VERIFIED: live test output, 2026-09-24]
- `SqliteUnitOfWork` gives all repositories one shared transaction and explicit `commit`/`rollback` methods. [VERIFIED: crates/storage-sqlite/src/lib.rs:230-307]
- The audit bridge computes sequence, previous hash, and row hash and persists the event in the caller's unit of work; persisted verification detects tampering. [VERIFIED: crates/application/src/audit_bridge.rs:200-270] [VERIFIED: crates/application/src/audit_bridge.rs:299-343]
- The jobs crate has backend-neutral queue/worker abstractions, lease recovery, bounded backoff, cancellation flag polling, and recovery fixtures. [VERIFIED: crates/jobs/src/queue.rs:16-68] [VERIFIED: crates/jobs/src/worker.rs:18-220] [VERIFIED: crates/storage-sqlite/tests/recovery_jobs.rs:52-135]
- The CLI already has stable foundation command enums, centralized exit-code mapping, redacted catalog output, and trycmd/catalog tests. [VERIFIED: crates/cli/src/lib.rs:76-208] [VERIFIED: crates/cli/src/lib.rs:564-610] [VERIFIED: crates/cli/tests/catalog.rs:13-21]
- `arch-check`, `migrate-check`, formatting, clippy, tests, schema checks, and coverage gates are already represented in CI. [VERIFIED: .github/workflows/ci.yml:27-129] [VERIFIED: xtask/src/ci.rs:15-104]

## Scope Boundaries

### In scope

- Foundation CLI groups: `config`, `db`, `doctor`, `job`, `audit`, `secret`, and `source`, plus the minimum `qai serve` host wiring needed by D-13. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:76-80]
- Effective configuration loading, complete origin tracking, redaction, keyed inspection, actionable validation, and explicit migration readiness. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:24-28]
- One auditable transaction boundary for domain mutation, provenance, audit, and required outbox records. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:30-35]
- Durable job enqueue/lease/heartbeat/checkpoint/retry/cancel behavior, lifecycle audit, cooperative cancellation outcomes, and long-lived worker hosting. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:36-40]
- Architecture enforcement for the dependency edges the project claims to govern, with CI regression coverage. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:58-71]

### Out of scope

- New Quran search, linguistic, graph, streaming, GUI, TUI, agent, MCP, RAG, model, embedding, or remote-provider capabilities. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:7-12] [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:121-128]
- Replacing SQLite, introducing an external broker, or implementing PostgreSQL/Qdrant adapters. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:58-63] [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:121-126]
- A new full secret-management product surface. The current `secret set`/`delete` handlers explicitly refuse durable writes; preserve that boundary unless the owner changes D-08/D-09. [VERIFIED: crates/cli/src/lib.rs:692-731] [ASSUMED: preserving this refusal is the recommended Phase 1 boundary]
- Automatic migration, silent worker startup from one-shot commands, or automatic repair of a tampered audit chain. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:24-35] [VERIFIED: crates/cli/src/lib.rs:126-175]

## Standard Stack

### Core

| Library / tool | Version | Purpose | Why standard for this phase |
|---|---:|---|---|
| Rust toolchain | `1.97.1` | Workspace language and pinned compiler. | The repository pins `channel = "1.97.1"` with rustfmt and clippy; reuse it rather than introduce a toolchain change. [VERIFIED: rust-toolchain.toml:1-5] |
| Tokio | `1.53.1` | Async runtime, timers, tasks, signals, and future worker-host orchestration. | It is already the resolved runtime dependency and is used by the server, CLI, storage, and jobs paths. [VERIFIED: Cargo.lock:2738-2752] [VERIFIED: .planning/codebase/STACK.md:24-31] |
| SQLx | `0.8.6` | SQLite pools, transactions, and query execution. | It is the resolved SQLite adapter and already owns the shared `UnitOfWork`; no second persistence abstraction is needed. [VERIFIED: Cargo.lock:2406-2415] [VERIFIED: crates/storage-sqlite/src/lib.rs:23-41] |
| Clap | `4.6.6` | Typed CLI parsing and stable command/help surface. | It is the resolved derive dependency used by the existing `Cli` tree. [VERIFIED: Cargo.lock:394-402] [VERIFIED: crates/cli/src/lib.rs:11-16] |
| trycmd | `0.15.11` | Real-binary CLI snapshots and exit-code assertions. | It is already the CLI dev dependency and the existing harness sets an isolated `QAI_DATA_DIR`. [VERIFIED: Cargo.lock:3073-3089] [VERIFIED: crates/cli/tests/quran.rs:11-20] |
| SQLite | bundled through SQLx | Local authoritative relational store. | The project decision is SQLite authority with a read pool and explicit migrations; no external database service is required. [VERIFIED: .planning/PROJECT.md:52-58] [VERIFIED: crates/storage-sqlite/src/lib.rs:58-100] |

### Supporting

| Library / tool | Version | Purpose | When to use |
|---|---:|---|---|
| `serde` / `serde_json` | workspace `1.x` | Config, CLI JSON, audit payloads, and stable machine output. | Use for envelopes and snapshots; do not serialize raw secrets. [VERIFIED: Cargo.toml:72-74] |
| `thiserror` | workspace `2.x` | Typed crate errors and stable diagnostics. | Use for new errors; preserve centralized CLI exit mapping. [VERIFIED: Cargo.toml:74-75] [VERIFIED: .planning/codebase/CONVENTIONS.md:74-105] |
| `tempfile` | `3.27.0` in the lockfile | Isolated data directories and real SQLite fixtures. | Use in every CLI/integration acceptance test that could create state. [VERIFIED: .planning/codebase/TESTING.md:142-147] |
| `proptest` | workspace `1.x` | Property coverage for config, timestamps, and idempotency where useful. | Use only where a property is more precise than a fixture matrix. [VERIFIED: Cargo.toml:86-90] |
| `cargo-deny` | CI-installed | Advisory, license, and source policy. | Keep as a separate supply-chain gate; do not substitute it for `arch-check`. [VERIFIED: .github/workflows/ci.yml:12-25] |

**Installation:** no new package installation is recommended. Run `cargo build -p cli --bin qai` using the existing lockfile; do not run `cargo add` or upgrade dependencies as part of this phase. [ASSUMED: preserving the locked stack is the recommended scope]

**Version verification:** `cargo tree -p cli --depth 1` resolved Clap `4.6.6`, Tokio `1.53.1`, and SQLx `0.8.6`; the lockfile resolves trycmd `0.15.11`. [VERIFIED: Cargo.lock:394-398] [VERIFIED: Cargo.lock:2406-2410] [VERIFIED: Cargo.lock:2738-2742] [VERIFIED: Cargo.lock:3073-3077] A registry search on 2026-09-24 displayed newer search results for some packages, but the crates.io API publish-date probe timed out; no publish date or upgrade compatibility is claimed, and no upgrade is part of this phase. [VERIFIED: live registry-search/API probe, 2026-09-24] [ASSUMED: do not infer compatibility from a newer registry result]

## Package Legitimacy Audit

No new external package is proposed or required for Phase 1. The package-legitimacy gate is therefore **not applicable to an install action** in this phase; the existing workspace dependencies are reused under `Cargo.lock` and are not reclassified as newly approved packages. [VERIFIED: Cargo.toml:70-120] [VERIFIED: Cargo.lock:390-398] [VERIFIED: Cargo.lock:2406-2410] [VERIFIED: Cargo.lock:2738-2742] [VERIFIED: Cargo.lock:3073-3077]

| Package family | Registry | Resolved version evidence | New install | Disposition |
|---|---|---:|---|---|
| Tokio / SQLx / Clap / trycmd | crates.io | `1.53.1` / `0.8.6` / `4.6.6` / `0.15.11` in `Cargo.lock`. [VERIFIED: Cargo.lock:394-398] [VERIFIED: Cargo.lock:2406-2410] [VERIFIED: Cargo.lock:2738-2742] [VERIFIED: Cargo.lock:3073-3077] | No | Reuse; no new legitimacy decision required. |
| Any new worker/config/CLI dependency | Any | None proposed. [ASSUMED: no new dependency] | No | Do not add without a separate measured need and package audit. |

**Packages removed due to a SLOP verdict:** none; no new package was researched for installation. **Packages flagged as SUS:** none. [ASSUMED: no new package is in scope]

## Architecture Patterns

### System Architecture Diagram

```text
                         operator / CI
                              |
                              v
                 +----------------------------+
                 | qai CLI + Clap dispatch    |
                 | stable foundation groups    |
                 +----------------------------+
                       |             |
          effective    |             | explicit mutation request
          config       v             v
       +--------------------+   +--------------------+
       | Config::load       |   | application service |
       | + OriginMap        |   | (audited boundary)  |
       +--------------------+   +--------------------+
              |                         |
              v                         v
       +--------------------+   +--------------------+
       | qai doctor         |   | UnitOfWork         |
       | metadata/read-only |   | domain + provenance|
       +--------------------+   | + audit + outbox    |
              |                 +--------------------+
              |                         | one commit/rollback
              v                         v
       +----------------------------------------------------+
       | SQLite authority: migrations, jobs, audit, outbox   |
       +----------------------------------------------------+
                                      ^
                                      |
                 +--------------------+--------------------+
                 |                                         |
                 v                                         v
       +--------------------+                    +--------------------+
       | qai serve host      |                    | one-shot command    |
       | worker tasks/signals|                    | enqueue only        |
       +--------------------+                    +--------------------+
                 |
                 v
       +----------------------------------------------------+
       | Worker -> claim/heartbeat/checkpoint/cancel        |
       | handler -> own audited domain transaction          |
       +----------------------------------------------------+
                                      |
                                      v
       +----------------------------------------------------+
       | qai audit verify / recovery guidance                |
       +----------------------------------------------------+
```

The primary decision branches are: missing or pending migrations route to the explicit `qai db migrate` remedy; doctor uses read-only metadata and persisted verification; an audited mutation either commits every required record or rolls back the whole unit; a worker observes a durable cancellation request at a named checkpoint and records a terminal outcome; and a valid audit chain is the only path to a passing audit readiness check. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:24-40] [VERIFIED: crates/cli/src/doctor.rs:122-154] [VERIFIED: crates/application/src/audit_bridge.rs:242-270]

### Recommended Project Structure

```text
crates/
├── config/src/lib.rs                 # complete merge/origin/redaction contract
├── application/src/
│   ├── db.rs                         # read-only readiness and migration status
│   ├── audit_bridge.rs               # persisted verification and audit append
│   ├── job_queue.rs                  # SQLite queue adapter and worker assembly
│   ├── quran.rs                       # audited mutation composition only
│   └── quran_cli.rs                   # enqueue-only one-shot import path
├── cli/src/
│   ├── lib.rs                         # loaded config, stable dispatch, foundation commands
│   ├── doctor.rs                      # metadata-only checks and audit verifier call
│   └── quran.rs                       # host/one-shot boundary wiring
├── jobs/src/
│   ├── lib.rs                         # durable checkpoint/cancellation contracts
│   └── worker.rs                      # per-kind policy and lifecycle transitions
└── storage/src/workflows.rs           # audited domain/provenance/outbox composition
migrations/sqlite/                     # append-only checksummed migration files
xtask/src/arch.rs                       # path + external dependency enforcement
```

The structure is a map to existing modules, not a request to create parallel crates. The current composition root already wires config, storage, provenance, audit, jobs, and observability, while CLI code is intended to depend on application rather than concrete SQLite. [VERIFIED: crates/application/src/lib.rs:1-26] [VERIFIED: .planning/codebase/ARCHITECTURE.md:54-77]

### Pattern 1: Loaded Configuration Envelope

**What:** Load once, preserve the effective value and its origin together, and pass that pair through every command that needs configuration. Do not reconstruct a default `Config` inside a handler.

**When to use:** All `config`, `db`, `doctor`, `job`, `audit`, `source`, `secret`, and `serve` paths. The current library already returns `(Config, OriginMap)` and the CLI currently discards the second value. [VERIFIED: crates/config/src/lib.rs:219-259] [VERIFIED: crates/cli/src/lib.rs:381-405]

**Verified library pattern:**

```rust
// Source: crates/config/src/lib.rs:219-259
pub fn load(
    config_path: Option<&PathBuf>,
    env_prefix: &str,
    cli_overrides: &BTreeMap<String, String>,
) -> Result<(Self, OriginMap), ConfigError> {
    let mut config: Self = Self::default();
    let mut origins = OriginMap::new();
    mark_defaults(&mut origins);
    // file, environment, CLI, interpolation, and validation follow below.
    Ok((config, origins))
}
```

The origin enum currently has the exact variants `Default`, `File { path: String, line: Option<usize> }`, `Env(String)`, and `Cli(String)`. [VERIFIED: crates/config/src/origin.rs:4-11] The implementation should render these values in a stable redacted envelope rather than expose secret values through a second path. [VERIFIED: crates/cli/src/lib.rs:407-419] [CITED: https://docs.rs/clap/4.6.6/clap/_derive/]

### Pattern 2: Read-Only Bootstrap and Explicit Migration

**What:** `qai doctor` should inspect existence, type, permissions, migration status, and integrity without creating or writing. Ordinary commands should use a read/existing-database readiness check and emit a coded pending-migration result with the exact remedy.

**When to use:** First-run operator flow and every command that opens a database, especially `quran get`, source/job catalogs, audit listing, and `serve`.

The current read-only SQLite constructor sets `create_if_missing(false)` and `read_only(true)`, while the normal constructor sets `create_if_missing(true)` and creates the parent directory. [VERIFIED: crates/storage-sqlite/src/lib.rs:58-77] [VERIFIED: crates/storage-sqlite/src/lib.rs:103-122] Use the former for diagnostics and the latter only behind an explicit migration/creation command. [VERIFIED: crates/application/src/db.rs:52-84] [VERIFIED: crates/storage-sqlite/src/migrate.rs:168-219]

**Implementation direction:** add a shared readiness result rather than scattering string matching across CLI handlers. A missing file, unreadable file, pending migrations, checksum mismatch, and healthy database should be distinct operator outcomes. [ASSUMED: recommended readiness result shape; the existing `MigrationStatus` and `DbProbe` are the reusable starting points] [VERIFIED: crates/application/src/db.rs:17-43]

### Pattern 3: One Audited Unit of Work

**What:** Compose the authoritative domain write, its provenance record, its audit event, and any required outbox event through one `UnitOfWork`; call `commit` only after all writes succeed.

**When to use:** Source lifecycle, approvals, activation/rollback, provenance writes, imports, job/admin mutations, and any handler result that changes authoritative state. Read-only doctor and derived-cache rebuilds remain outside the audit requirement unless they alter authoritative state. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:30-35]

The current transaction implementation starts one SQLx transaction and exposes repository handles that share it; commit and rollback drop repository clones before unwrapping the transaction. [VERIFIED: crates/storage-sqlite/src/lib.rs:244-307]

**Verified transaction pattern:**

```rust
// Source: crates/storage-sqlite/src/lib.rs:291-305
async fn commit(self: Box<Self>) -> Result<(), StorageError> {
    let Self { tx, sources, provenance, audit, jobs, settings, outbox, quran } = *self;
    drop((sources, provenance, audit, jobs, settings, outbox, quran));
    let mutex = Arc::try_unwrap(tx).map_err(|_| StorageError::StorageBusy)?;
    let txn = mutex.into_inner();
    txn.commit().await.map_err(|_| StorageError::StorageUnavailable)
}
```

The current workflow helpers demonstrate why the missing audit composition matters: `record_source_activation` allocates an outbox generation, enqueues an event, and transitions the source, but does not append an audit event; `record_provenance_write` similarly inserts provenance and outbox state without audit. [VERIFIED: crates/storage/src/workflows.rs:27-109] The plan should add failure-injection tests at each required write boundary, not only a happy-path integration test. [ASSUMED: recommended failure-injection test matrix]

### Pattern 4: Long-Lived Worker Host with Cooperative Cancellation

**What:** Keep the queue and handler contracts backend-neutral, but host one or more worker loops in the existing `qai serve` process. One-shot commands enqueue and return; they do not call `run_until_idle` implicitly.

**When to use:** Import/index/morphology jobs and future foundation jobs. The current `qai serve` path starts the HTTP server but does not assemble or supervise a worker, while `run_import_job` constructs a worker and drains it inline. [VERIFIED: crates/cli/src/lib.rs:301-345] [VERIFIED: crates/application/src/quran.rs:1124-1169]

Tokio's official shutdown guidance separates detecting shutdown, signaling tasks, and awaiting task completion; cancellation should notify tasks, and the host should wait for cleanup/final persistence. [CITED: https://tokio.rs/tokio/topics/shutdown] Reuse the existing Tokio dependency and the repository's current cancellation flag unless the planner can justify adding a separate utility crate. [VERIFIED: Cargo.toml:79-80] [VERIFIED: crates/jobs/src/lib.rs:257-295] [ASSUMED: no new cancellation utility package is necessary]

**Current lifecycle vocabulary (verbatim):** `Queued`, `Leased`, `Running`, `Checkpointed`, `Succeeded`, `Failed`, `Cancelled`, `Interrupted`, and `DeadLettered`. [VERIFIED: crates/jobs/src/lib.rs:90-113] The implementation may add an explicit cancellation outcome/reason, but its exact serialized names and checkpoint payloads remain an implementation choice. [ASSUMED]

**Checkpoint rule:** persist a named checkpoint immediately when a handler reaches a resumable boundary, not only when the handler fails. The current `JobContext::checkpoint` writes to an in-memory mutex and the worker copies that value to the queue only in the failure branch. [VERIFIED: crates/jobs/src/lib.rs:225-325] [VERIFIED: crates/jobs/src/worker.rs:149-170]

**Cancellation rule:** a CLI cancel request must write `cancel_requested` durably and be auditable; the worker watchdog already polls `cancel_requested`, but the CLI currently refuses the operation and the worker exposes only a generic cancelled outcome. [VERIFIED: crates/storage-sqlite/src/lib.rs:1010-1021] [VERIFIED: crates/jobs/src/worker.rs:224-239] [VERIFIED: crates/cli/src/lib.rs:682-688] [VERIFIED: crates/jobs/src/worker.rs:144-154]

### Pattern 5: Persisted Audit Verification as the Doctor Authority

**What:** Reuse `verify_persisted_audit` for both `qai audit verify` and the doctor's audit readiness result. A row count is not chain validity.

**When to use:** Any operator readiness check, CI acceptance test, or recovery workflow.

The persisted verifier reads ordered events, checks sequence continuity, recomputes the chain hash, records tampered sequences, and rolls back its read snapshot. [VERIFIED: crates/application/src/audit_bridge.rs:242-270] The current doctor check only branches on `probe.audit_events` and reports PASS when rows exist. [VERIFIED: crates/cli/src/doctor.rs:441-458] The live probe demonstrated the false positive: `qai audit verify` returned exit `3` and `tampered_sequences: [1]`, while doctor returned `audit.chain_valid` status `pass`. [VERIFIED: live probe, 2026-09-24]

### Anti-Patterns to Avoid

- **Rebuilding the foundation crates:** Reuse `Config`, `UnitOfWork`, `audit`, `jobs`, and the existing composition root; create tasks only at the demonstrated seams. [VERIFIED: crates/application/src/lib.rs:1-26]
- **Loading defaults in a command handler:** It hides file/env/CLI precedence and makes origin reporting impossible. [VERIFIED: crates/cli/src/lib.rs:421-449]
- **Making doctor green by writing probes:** A diagnostic that creates directories or files violates the locked read-only contract even if it cleans them up. [VERIFIED: crates/cli/src/doctor.rs:1-8] [VERIFIED: crates/cli/src/doctor.rs:237-254]
- **Calling SQLite's creating constructor from ordinary commands:** It turns a read into an implicit bootstrap mutation. [VERIFIED: crates/storage-sqlite/src/lib.rs:58-77] [VERIFIED: crates/application/src/quran_cli.rs:71-73]
- **Treating an audit row count as integrity:** A malformed row still counts; only ordered hash verification is authoritative. [VERIFIED: crates/cli/src/doctor.rs:441-458] [VERIFIED: crates/application/src/audit_bridge.rs:242-270]
- **Persisting checkpoints only after failure:** A process crash loses the in-memory checkpoint and can repeat completed work. [VERIFIED: crates/jobs/src/lib.rs:292-325] [VERIFIED: crates/jobs/src/worker.rs:155-159]
- **Silently starting a worker from a one-shot command:** It violates the operator-visible execution model and makes cancellation/host ownership ambiguous. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:36-40] [VERIFIED: crates/application/src/quran.rs:1159-1160]
- **Adding a new broker or utility crate by default:** The locked architecture uses SQLite and no external local broker; solve the host with existing Tokio/jobs primitives unless evidence justifies otherwise. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:58-63] [VERIFIED: Cargo.toml:79-90]

## Implementation Gap Register

| Gap ID | Exact current seam | Evidence | Prescriptive implementation direction | Required tests |
|---|---|---|---|---|
| FND-01 | `crates/cli/src/lib.rs::loads_or_default`, `handle_config_show`, `handle_config_get`, `handle_config_validate`; `crates/config/src/lib.rs::Config::load` and merge helpers. [VERIFIED: crates/cli/src/lib.rs:381-449] [VERIFIED: crates/config/src/lib.rs:219-259] | The CLI discards the origin map, uses defaults in handlers, ignores `get` keys, and has no effective-config object. [VERIFIED: live probe, 2026-09-24] | Load `(Config, OriginMap)` once at dispatch, pass both to handlers, redact at the output boundary, and cover every field in origin tracking. Do not add a config-file mutation format unless separately approved. [ASSUMED: recommended API] | Unit tests for all precedence layers; CLI JSON/human tests for origins, keyed get, invalid values, and secret redaction. [ASSUMED: proposed test names] |
| FND-02 | `crates/cli/src/doctor.rs::data_dir_writable`, `filesystem_object_store_writable`, `audit_chain_valid`; `crates/application/src/db.rs::probe_database`; `crates/storage-sqlite/src/lib.rs::open_read_only`. [VERIFIED: crates/cli/src/doctor.rs:237-254] [VERIFIED: crates/cli/src/doctor.rs:528-546] [VERIFIED: crates/cli/src/doctor.rs:441-458] | Doctor creates/writes probe files; audit readiness is a row count. [VERIFIED: live probes, 2026-09-24] | Replace write probes with metadata/existence checks and call persisted audit verification; make inability to prove writability a non-mutating warning with a remedy. [ASSUMED: recommended diagnostic semantics] | Fresh temp-dir test asserts no paths/files are created; tampered-audit test asserts doctor fails or warns and points to `qai audit verify`. [ASSUMED: proposed test names] |
| FND-03 | `crates/application/src/db.rs::migration_status`; `crates/cli/src/lib.rs::db_path_for` and ordinary command dispatch; `crates/application/src/quran_cli.rs::open_db`. [VERIFIED: crates/application/src/db.rs:65-84] [VERIFIED: crates/application/src/quran_cli.rs:71-73] | `db status` on a missing DB returned generic `storage unavailable`; `quran get` created `qai.db`. [VERIFIED: live probes, 2026-09-24] | Add a shared existing-DB/readiness guard and map missing/pending migrations to the exact `qai db migrate` remedy; reserve creation for explicit migration. [ASSUMED: recommended error contract] | CLI tests for missing DB, pending migration, current DB, and no side effects; check no DB/WAL is created by ordinary reads. [ASSUMED: proposed test names] |
| FND-04 | `crates/storage/src/workflows.rs::record_source_activation`, `record_provenance_write`; `crates/application/src/audit_bridge.rs::append_audit_event`; `crates/application/src/quran.rs::audit_activation` and staged import path. [VERIFIED: crates/storage/src/workflows.rs:27-109] [VERIFIED: crates/application/src/audit_bridge.rs:200-231] [VERIFIED: crates/application/src/quran.rs:262-342] | Existing UoW/audit primitives are strong, but coverage is fragmented and some workflows omit audit/provenance/outbox combinations. [VERIFIED: crates/storage/src/workflows.rs:27-109] | Define one application-level audited mutation composition; use it for all authoritative state changes and add failure injection at every required write. Keep source import catalog-only unless the owner confirms otherwise. [ASSUMED: recommended composition] | Real-SQLite tests for commit, rollback on audit failure, rollback on outbox failure, and source/import lifecycle event ordering. [ASSUMED: proposed test names] |
| FND-05 | `crates/jobs/src/lib.rs::JobContext::checkpoint`; `crates/jobs/src/worker.rs::run_once`, `handle_failure`; `crates/application/src/job_queue.rs::SqliteJobQueue`; `crates/storage/src/repository.rs::JobRepository`. [VERIFIED: crates/jobs/src/lib.rs:225-325] [VERIFIED: crates/jobs/src/worker.rs:100-205] [VERIFIED: crates/application/src/job_queue.rs:35-135] [VERIFIED: crates/storage/src/repository.rs:277-361] | Checkpoint is memory-only until failure; worker uses a global `WorkerConfig.max_attempts`; storage has `cancel` but the queue/CLI do not expose it; no explicit retry command. [VERIFIED: crates/jobs/src/worker.rs:149-205] [VERIFIED: crates/storage-sqlite/src/lib.rs:1010-1021] [VERIFIED: crates/cli/src/lib.rs:192-200] [VERIFIED: crates/cli/src/lib.rs:656-690] | Persist named checkpoints immediately, resolve policy per job kind, expose `job retry` and `job cancel`, record cancellation outcome, and audit lifecycle transitions. [ASSUMED: exact retry/outcome schemas] | Real-SQLite crash/resume tests, per-kind max-attempt tests, explicit retry tests, cancellation-before/during/after-boundary tests, and audit-event assertions. [ASSUMED: proposed test names] |
| FND-06 | `crates/application/src/job_queue.rs::build_worker`; `crates/cli/src/lib.rs::Serve`; `crates/application/src/quran.rs::run_import_job`; `crates/application/src/quran_cli.rs::cmd_import`. [VERIFIED: crates/application/src/job_queue.rs:128-135] [VERIFIED: crates/cli/src/lib.rs:301-345] [VERIFIED: crates/application/src/quran.rs:1124-1169] [VERIFIED: crates/application/src/quran_cli.rs:569-670] | There is no long-lived worker host in `serve`, and import explicitly drains a worker inline. [VERIFIED: crates/application/src/quran.rs:1159-1160] | Add a host lifecycle in `qai serve`; make one-shot import enqueue/report a job ID, with an explicit operator path to run/observe work. Do not add a new broker. [ASSUMED: proposed host API] | Host lifecycle test, one-shot enqueue-only test, graceful-shutdown test, and integration test that a queued job runs under `qai serve`. [ASSUMED: proposed test names] |
| FND-07 | `xtask/src/arch::violations`; `xtask/allowlist.toml`; `.github/workflows/ci.yml` arch job. [VERIFIED: xtask/src/arch.rs:102-164] [VERIFIED: xtask/allowlist.toml:1-10] [VERIFIED: .github/workflows/ci.yml:47-64] | Only path dependencies are filtered by `source.is_none()`; external policy is reserved but unused. [VERIFIED: xtask/src/arch.rs:117-139] [VERIFIED: xtask/allowlist.toml:3-7] | Parse registry/git dependency sources, define an explicit allowed/denied external policy, and add a synthetic forbidden-registry-edge test. [ASSUMED: policy format] | `cargo test -p xtask`, `cargo run -p xtask -- arch-check`, and a CI mutation test. [VERIFIED: .github/workflows/ci.yml:59-64] |

### Recommended Plan Decomposition

1. **Wave 0 — contracts and fixtures:** add the missing foundation CLI/doctor/job/audit acceptance fixtures and freeze the current migration count/checksum baseline. This is a test/fixture task, not a service rewrite. [ASSUMED: recommended wave ordering]
2. **Wave 1 — operator bootstrap:** implement FND-01 through FND-03; make the inspect → configure → migrate → verify journey pass before enabling mutations. [ASSUMED: recommended dependency ordering]
3. **Wave 2 — authoritative writes:** implement FND-04 and lifecycle audit coverage; add failure-injection tests before exposing new mutation commands. [ASSUMED: recommended dependency ordering]
4. **Wave 3 — worker lifecycle:** implement FND-05 and FND-06; keep the existing SQLite queue and handler contracts, then move default execution to `qai serve`. [ASSUMED: recommended dependency ordering]
5. **Wave 4 — enforcement and phase gate:** implement FND-07, run all targeted/full commands, and update phase/task/rollup/follow-up artifacts only after acceptance evidence is green. [VERIFIED: AGENTS.md:76-108] [ASSUMED: recommended final wave]

## Don't Hand-Roll

| Problem | Don't build | Use instead | Why |
|---|---|---|---|
| Config precedence and origin tracking | A second ad-hoc environment/file merge in each CLI handler. | `config::Config::load`, `OriginMap`, and the existing merge helpers. | The library already returns the effective pair and has precedence tests. [VERIFIED: crates/config/src/lib.rs:219-259] [VERIFIED: crates/testkit/tests/config_precedence.rs:18-49] |
| Atomic multi-repository writes | A bespoke “write then audit” sequence or compensating deletes. | `storage::UnitOfWork` and `SqliteUnitOfWork::commit/rollback`. | One SQLx transaction is already shared by all repository handles. [VERIFIED: crates/storage-sqlite/src/lib.rs:230-307] |
| Audit hashing and verification | A new event hash format or row-count integrity check. | `audit::HashChainWriter`, `AuditVerifier`, and `application::audit_bridge`. | The project has a frozen hash-chain contract and persisted verifier. [VERIFIED: crates/application/src/audit_bridge.rs:200-270] [VERIFIED: migrations/sqlite/0005_audit.up.sql:1-26] |
| Job leasing, recovery, and retry scheduling | A second queue implementation in CLI/application. | `jobs::JobQueue`, `Worker`, and `SqliteJobQueue`. | The existing abstraction already supports in-memory tests and SQLite production operations. [VERIFIED: crates/jobs/src/queue.rs:16-68] [VERIFIED: crates/application/src/job_queue.rs:19-135] |
| CLI parsing/help/exit mapping | Manual argument parsing or per-command help text. | Clap derive plus the centralized `exit_code` module. | The stable command tree already derives from `Cli`/`Subcommand`. [VERIFIED: crates/cli/src/lib.rs:11-16] [VERIFIED: crates/cli/src/lib.rs:40-122] [CITED: https://docs.rs/clap/4.6.6/clap/_derive/] |
| Real CLI acceptance snapshots | Mocking the binary or asserting only library internals. | trycmd with an isolated `QAI_DATA_DIR`. | Existing tests drive the real `qai` binary and assert output/exit behavior. [VERIFIED: crates/cli/tests/quran.rs:11-20] [CITED: https://github.com/assert-rs/trycmd] |
| Migration integrity | Editing applied SQL or silently invoking migrations from reads. | Existing checksummed migration runner and `migrate-check`. | The runner verifies already-applied checksums before writes and the repository has 19 ordered migrations. [VERIFIED: crates/storage-sqlite/src/migrate.rs:168-219] [VERIFIED: live `migrate-check` output, 2026-09-24] |
| Dependency policy | A manual review checklist or grep-only rule. | Extend the pure `xtask::arch::violations` metadata check and keep CI enforcement. | The current checker already has mutation tests for forbidden workspace edges. [VERIFIED: xtask/src/arch.rs:102-164] [VERIFIED: cargo test -p xtask output, 2026-09-24] |

**Key insight:** the hard problems in this phase are composition and observability of existing primitives, not missing cryptographic, database, or parser technology. Adding a new library would increase dependency and architecture surface without closing the demonstrated seams. [VERIFIED: crates/storage-sqlite/src/lib.rs:230-307] [VERIFIED: crates/application/src/audit_bridge.rs:200-270] [VERIFIED: crates/jobs/src/worker.rs:18-220]

## Common Pitfalls

### Pitfall 1: Treating `Config::default()` as the effective config

**What goes wrong:** File and environment values disappear at the CLI boundary; precedence tests pass in the library while the real command prints defaults. [VERIFIED: live probe, 2026-09-24]

**Why it happens:** `loads_or_default` returns only a `Config` in several paths, and handlers independently instantiate `Config::default()`. [VERIFIED: crates/cli/src/lib.rs:381-405] [VERIFIED: crates/cli/src/lib.rs:421-449]

**How to avoid:** Make the loaded `(Config, OriginMap)` a required dispatch input; add a CLI-level precedence test that exercises file + env + global override and checks redaction/origins. [ASSUMED: recommended test shape]

**Warning signs:** output contains `${app.data_dir}`, default port `8737` despite a file/env override, or `--explain` has no origin section. [VERIFIED: live probe, 2026-09-24]

### Pitfall 2: “Read-only” doctor checks that write probes

**What goes wrong:** Running doctor on a fresh data directory creates the data and object directories, violating D-06 and the project rule that doctor must not modify data. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:24-28] [VERIFIED: .agent/coding-rules.md:13-16] [VERIFIED: live probe, 2026-09-24]

**Why it happens:** `data_dir_writable` and `filesystem_object_store_writable` call `create_dir_all` and `write`. [VERIFIED: crates/cli/src/doctor.rs:237-254] [VERIFIED: crates/cli/src/doctor.rs:528-546]

**How to avoid:** Use `symlink_metadata`/permissions/existence checks and report “not verified without mutation” where necessary; never call the creating SQLite constructor from doctor. [ASSUMED: recommended metadata-only implementation]

**Warning signs:** a doctor run on a nonexistent temp path leaves `objects/` or a database behind. [VERIFIED: live probe, 2026-09-24]

### Pitfall 3: Letting ordinary reads create a database

**What goes wrong:** `quran get` on an unmigrated path creates `qai.db` before failing, turning diagnosis into mutation. [VERIFIED: crates/application/src/quran_cli.rs:71-73] [VERIFIED: live probe, 2026-09-24]

**Why it happens:** `open_db` calls `SqliteDatabase::new`, whose options use `create_if_missing(true)` and create the parent directory. [VERIFIED: crates/storage-sqlite/src/lib.rs:58-77]

**How to avoid:** Route ordinary commands through an existing-DB readiness guard; keep `qai db migrate` as the explicit creation/mutation path. [ASSUMED: recommended guard]

**Warning signs:** an error path still leaves a DB, WAL, or object directory. [VERIFIED: live probe, 2026-09-24]

### Pitfall 4: Calling a row count an audit verification

**What goes wrong:** A tampered chain is reported healthy by doctor while the required verifier fails. [VERIFIED: live probe, 2026-09-24]

**Why it happens:** `audit_chain_valid` only checks whether `probe.audit_events` is zero or positive. [VERIFIED: crates/cli/src/doctor.rs:441-458]

**How to avoid:** Reuse `verify_persisted_audit` and propagate its `valid`, `gaps`, and `tampered_sequences` into the doctor result. [VERIFIED: crates/application/src/audit_bridge.rs:234-270]

**Warning signs:** doctor says “structural chain present” without a sequence/hash result. [VERIFIED: crates/cli/src/doctor.rs:453-456]

### Pitfall 5: Splitting a domain mutation from its audit/outbox records

**What goes wrong:** A domain row can commit while its required audit or outbox row is missing, making recovery and trust evidence diverge. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:30-35] [VERIFIED: crates/storage/src/workflows.rs:27-109]

**Why it happens:** Existing helpers compose different subsets, and job adapter methods each open/commit their own unit of work. [VERIFIED: crates/application/src/job_queue.rs:35-125]

**How to avoid:** Define one application-level composition function and inject failure at each required repository call; assert rollback, event order, and idempotency. [ASSUMED: recommended helper/test]

**Warning signs:** a test passes when the audit insert fails or when the outbox enqueue fails after the domain transition. [ASSUMED]

### Pitfall 6: Checkpointing only in memory

**What goes wrong:** A crash after a completed stage but before handler failure loses the checkpoint and repeats work. [VERIFIED: crates/jobs/src/lib.rs:292-325] [VERIFIED: crates/jobs/src/worker.rs:155-159]

**Why it happens:** `JobContext::checkpoint` updates an `Arc<Mutex<Option<String>>>`; the worker reads the sink only in the failure branch. [VERIFIED: crates/jobs/src/lib.rs:310-325] [VERIFIED: crates/jobs/src/worker.rs:149-170]

**How to avoid:** Persist a named checkpoint at each resumable boundary through the queue/store, and make resume tests assert the exact committed boundary. [ASSUMED: recommended checkpoint API]

**Warning signs:** `jobs.checkpoint_json` remains `None` after a successful multi-stage handler. [VERIFIED: crates/jobs/src/lib.rs:240-241]

### Pitfall 7: Applying one retry policy to every job kind

**What goes wrong:** A transient storage retry budget is applied to a non-idempotent or domain-invalid job, or a long import exhausts a global limit unintentionally. [VERIFIED: crates/jobs/src/worker.rs:187-205] [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:39-40]

**Why it happens:** `WorkerConfig` has one `max_attempts` and `handle_failure` compares the claimed job's attempts to that worker-wide value, even though `JobRecord` also carries `max_attempts`. [VERIFIED: crates/jobs/src/worker.rs:18-44] [VERIFIED: crates/jobs/src/worker.rs:187-203] [VERIFIED: crates/storage/src/repository.rs:363-379]

**How to avoid:** Resolve retryability and attempt limits by job kind/handler policy, persist the selected policy or its version, and make explicit retry reset only the intended fields under audit. [ASSUMED: recommended policy design]

**Warning signs:** changing one handler's retry behavior changes every handler using the same worker. [ASSUMED]

### Pitfall 8: Treating cancellation as an immediate terminal state

**What goes wrong:** A handler is marked cancelled before it reaches a safe boundary, or the operator cannot tell whether work completed before observing the request. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:39-40]

**Why it happens:** The current worker polls a boolean and immediately finishes `Cancelled` after the handler returns; the CLI refuses the request entirely. [VERIFIED: crates/jobs/src/worker.rs:144-154] [VERIFIED: crates/cli/src/lib.rs:682-688]

**How to avoid:** Persist the request, let the handler observe it at named checkpoints, and record a terminal outcome plus lifecycle audit event. [ASSUMED: exact outcome names]

**Warning signs:** a running handler's lease is cleared by a CLI process without a corresponding durable request and audit event. [ASSUMED]

### Pitfall 9: Letting `qai serve` and one-shot commands share ambiguous execution ownership

**What goes wrong:** An import appears synchronous, a worker runs in a short-lived process, and shutdown/cancellation semantics cannot be reasoned about. [VERIFIED: crates/application/src/quran.rs:1159-1160] [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:36-40]

**Why it happens:** `run_import_job` constructs and drains a worker inline, while `qai serve` only constructs the API server. [VERIFIED: crates/application/src/quran.rs:1124-1169] [VERIFIED: crates/cli/src/lib.rs:301-345]

**How to avoid:** Put the worker loop in the long-lived host and make one-shot commands enqueue/report only. [ASSUMED: recommended host seam]

**Warning signs:** a command named `import` returns only after the job reaches a terminal state without an explicit host. [VERIFIED: crates/application/src/quran_cli.rs:655-670]

### Pitfall 10: Believing path-only architecture checks cover external crates

**What goes wrong:** A forbidden registry or git dependency can pass `arch-check` because the implementation only evaluates dependencies with `source.is_none()`. [VERIFIED: xtask/src/arch.rs:117-139] [VERIFIED: xtask/allowlist.toml:3-7]

**Why it happens:** The allowlist reserves an external policy but `violations()` builds `allowed_path_deps` and never checks registry/git sources. [VERIFIED: xtask/src/arch.rs:58-78] [VERIFIED: xtask/src/arch.rs:117-139]

**How to avoid:** Define the external policy in the allowlist, parse dependency source/kind, and add a synthetic forbidden registry edge to the xtask tests. [ASSUMED: policy representation]

**Warning signs:** CI runs `arch-check` but a deliberately forbidden external dependency remains green. [VERIFIED: .github/workflows/ci.yml:47-64]

## Code Examples

### Effective configuration load and origin preservation

```rust
// Source: crates/config/src/lib.rs:219-259
let (mut config, mut origins) = Config::load(
    config_path,
    "QAI",
    &cli_overrides,
)?;
```

The current function marks defaults, merges file, environment, and CLI values, resolves interpolation, validates, and returns both values. [VERIFIED: crates/config/src/lib.rs:219-259] The CLI must retain this pair rather than replacing it with `Config::default()`. [VERIFIED: crates/cli/src/lib.rs:381-405] [ASSUMED: recommended call-site refactor]

### Read-only SQLite probe

```rust
// Source: crates/storage-sqlite/src/lib.rs:103-122
pub async fn open_read_only(path: &str) -> Result<Self, StorageError> {
    let read_options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(false)
        .foreign_keys(true)
        .busy_timeout(std::time::Duration::from_millis(5000))
        .read_only(true);
    // connect a read pool; do not create parent directories or database files
    // ...
}
```

This is the correct storage primitive for doctor/readiness. The current `data_dir_writable` and object-store checks bypass it by writing filesystem probes. [VERIFIED: crates/storage-sqlite/src/lib.rs:103-122] [VERIFIED: crates/cli/src/doctor.rs:237-254] [VERIFIED: crates/cli/src/doctor.rs:528-546]

### Same-transaction audit append

```rust
// Source: crates/application/src/audit_bridge.rs:205-231
pub async fn append_audit_event(
    uow: &mut dyn storage::UnitOfWork,
    mut event: AuditEvent,
) -> Result<AuditEvent, AuditError> {
    let (sequence, prev_hash) = {
        let bridge = StorageAuditBridge::new(uow.audit());
        let sequence = bridge.latest_sequence().await? + 1;
        let prev_hash = if sequence == 1 {
            genesis_hash()
        } else {
            bridge
                .list_by_sequence(sequence - 1, Some(sequence - 1))
                .await?
                .into_iter()
                .find(|candidate| candidate.sequence == sequence - 1)
                .map(|candidate| candidate.chain_hash)
                .unwrap_or_else(genesis_hash)
        };
        (sequence, prev_hash)
    };
    event.sequence = sequence;
    event.occurred_at = Timestamp::now();
    event.prev_chain_hash = prev_hash.clone();
    event.chain_hash = HashChainWriter::compute_chain_hash(&prev_hash, &event);
    let row = StorageAuditBridge::to_row(&event)?;
    uow.audit().append(row).await?;
    Ok(event)
}
```

The caller must invoke this before the same `UnitOfWork` commit. [VERIFIED: crates/application/src/audit_bridge.rs:200-231] [VERIFIED: crates/storage-sqlite/src/lib.rs:291-305]

### Worker cancellation observation

```rust
// Source: crates/jobs/src/worker.rs:224-239
async fn watchdog(
    queue: Arc<dyn JobQueue>,
    job_id: String,
    owner: String,
    cancel: Arc<AtomicBool>,
    lease: Duration,
    poll: Duration,
) {
    loop {
        tokio::time::sleep(poll).await;
        if queue.cancel_requested(&job_id).await.unwrap_or(false) {
            cancel.store(true, Ordering::SeqCst);
        }
        let _ = queue.heartbeat(&job_id, &owner, lease).await;
    }
}
```

This is an existing polling primitive, not a complete durable cancellation contract. The Phase 1 implementation must persist the request, record the outcome, and define what happens if the handler finishes before observing the flag. [VERIFIED: crates/jobs/src/worker.rs:224-239] [ASSUMED: required outcome extension]

### Persisted audit verification

```rust
// Source: crates/application/src/audit_bridge.rs:254-270
let mut previous = genesis_hash();
let mut expected_sequence = Some(1);
for event in events {
    if Some(event.sequence) != expected_sequence {
        report.gaps.push(event.sequence);
    }
    if event.prev_chain_hash != previous
        || HashChainWriter::compute_chain_hash(&previous, &event) != event.chain_hash
    {
        report.tampered_sequences.push(event.sequence);
    }
    previous = event.chain_hash;
    expected_sequence = event.sequence.checked_add(1);
}
report.valid = report.gaps.is_empty() && report.tampered_sequences.is_empty();
snapshot.rollback().await?;
```

Doctor should consume this result rather than reimplementing a weaker check. [VERIFIED: crates/application/src/audit_bridge.rs:254-270]

## State of the Art

| Old/current approach | Phase 1 target approach | When changed | Impact |
|---|---|---|---|
| CLI handlers instantiate `Config::default()` and discard origins. [VERIFIED: crates/cli/src/lib.rs:421-449] | Load `(Config, OriginMap)` once, pass it through dispatch, redact at output. [ASSUMED: recommended target] | Phase 1 gap closure | Makes the real operator path match the library precedence contract. |
| Doctor writes `.qai-probe` files and counts audit rows. [VERIFIED: crates/cli/src/doctor.rs:237-254] [VERIFIED: crates/cli/src/doctor.rs:441-458] | Metadata-only readiness plus persisted audit verification. [ASSUMED: recommended target] | Phase 1 gap closure | First-run diagnostics become non-mutating and trustworthy. |
| Normal `SqliteDatabase::new` creates missing paths for ordinary commands. [VERIFIED: crates/storage-sqlite/src/lib.rs:58-77] | Existing-DB guard; explicit `qai db migrate` creates/applies schema. [ASSUMED: recommended target] | Phase 1 gap closure | Prevents hidden bootstrap mutations. |
| One `WorkerConfig.max_attempts` governs the worker. [VERIFIED: crates/jobs/src/worker.rs:18-44] [VERIFIED: crates/jobs/src/worker.rs:187-203] | Per-kind bounded retry policy with explicit retry. [ASSUMED: recommended target] | Phase 1 gap closure | Makes transient/non-transient and domain-invalid behavior inspectable. |
| `quran import` constructs a worker and calls `run_until_idle`. [VERIFIED: crates/application/src/quran.rs:1159-1160] | `qai serve` owns long-lived workers; one-shot commands enqueue only. [ASSUMED: recommended target] | Phase 1 gap closure | Makes cancellation, shutdown, and retry ownership deterministic. |
| `arch-check` checks path edges only. [VERIFIED: xtask/src/arch.rs:117-139] | Check workspace and external dependency sources against explicit policy. [ASSUMED: recommended target] | Phase 1 gap closure | CI can fail on the forbidden dependencies the contract names. |

**Deprecated/outdated patterns for this phase:**

- Do not use ordinary-command auto-migration as a convenience; it contradicts D-05 and the observed side effect. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:24-28] [VERIFIED: live probe, 2026-09-24]
- Do not treat `Config::default()` in a handler as a valid fallback after a file was supplied; it silently discards operator intent. [VERIFIED: crates/cli/src/lib.rs:381-405] [VERIFIED: live probe, 2026-09-24]
- Do not use row count as a substitute for `verify_persisted_audit`; the live tampering probe disproves that shortcut. [VERIFIED: live probe, 2026-09-24]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|---|---|---|
| A1 | A future migration may be named `0020_*`; the exact filename and schema are not created or verified. | Implementation Gap Register, Common Pitfalls | A migration-number collision or schema mismatch; planner must inspect the final migration set before choosing the name. |
| A2 | Exact checkpoint payload schemas, cancellation outcome names, retry reason strings, and operator wording are implementation choices. | Architecture Patterns, FND-05 | CLI/API consumers may need a compatibility decision; keep names centralized and versioned. |
| A3 | A catalog-only `source import` is the narrowest useful Phase 1 implementation; it must not activate canonical data. | Scope Boundaries, FND-04 | If the owner intended full source lifecycle semantics, the task needs a larger acceptance surface. |
| A4 | `secret set`/`secret delete` should remain refused in Phase 1 while reference listing/validation remains available. | Scope Boundaries, Project Constraints | If D-08 is interpreted as requiring secret mutations, the planner must add a separately audited secret-reference workflow rather than silently storing values. |
| A5 | A `qai serve` worker host can be added using existing Tokio/jobs primitives without a new broker or utility package. | Architecture Patterns, FND-06 | A new dependency or server API change may be needed; this is a checkpoint for implementation design. |
| A6 | Proposed test filenames and test names in the gap register are placeholders, not existing paths. | Implementation Gap Register, Validation Architecture | Planner must create them under the project’s snake_case/topic conventions. |
| A7 | Newer registry search results do not justify an upgrade; publish dates and compatibility were not observed because the registry API probe timed out. | Standard Stack | If an upgrade is separately required, run a dedicated compatibility and supply-chain review. |
| A8 | ASVS is used only as a review vocabulary; exact versioned ASVS control identifiers are not treated as locked requirements. | Security Domain, Sources | Security acceptance must still be checked against the project’s locked ADRs and implementation tests. |

## Open Questions

1. **Does Phase 1 require a fully usable `source import`, or only a stable foundation surface?**
   - What we know: D-08 includes `source`; D-09 requires source lifecycle/import audit; the current handler refuses import and points to the later Quran command. [VERIFIED: crates/cli/src/lib.rs:612-652] [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:76-80]
   - What's unclear: whether catalog-only import is sufficient for the Phase 1 acceptance contract.
   - Recommendation: implement only validated catalog/source-version staging and audited lifecycle transitions; do not activate canonical text or absorb the Phase 2 corpus pipeline. [ASSUMED: recommended narrow interpretation]
2. **Should secret mutation commands remain explicitly out of scope?**
   - What we know: the stable group exists, but current set/delete handlers refuse durable writes and the project overview places secret management in a later surface. [VERIFIED: crates/cli/src/lib.rs:86-89] [VERIFIED: crates/cli/src/lib.rs:692-731]
   - What's unclear: whether “stable group” means command presence or mutation capability.
   - Recommendation: preserve reference-only inspection and explicit refusal; any future value mutation needs a separate threat model and audit design. [ASSUMED: recommended interpretation]
3. **What exact per-kind retry policy should be persisted?**
   - What we know: D-15 allows agent discretion for constants; current `WorkerConfig` is global and `JobRecord` has `max_attempts`. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:42-43] [VERIFIED: crates/jobs/src/worker.rs:18-44]
   - What's unclear: the policy table and whether it is config-, registry-, or migration-backed.
   - Recommendation: keep a typed per-kind policy in the jobs/application boundary, expose it in job inspection, and test each kind’s exhausted outcome before locking names. [ASSUMED]
4. **Should the Phase 1 migration be schema-changing or only an enforcement/test change?**
   - What we know: 19 checksummed migrations currently pass `migrate-check`; no `0020_*` file exists. [VERIFIED: migrations/sqlite/checksums.json:2-26] [VERIFIED: live `migrate-check` output, 2026-09-24]
   - What's unclear: whether lifecycle audit/outbox/checkpoint fields require columns or can use existing JSON/event tables.
   - Recommendation: inventory required fields first; add one append-only migration only if a durable field cannot be represented by the current schema. [ASSUMED: migration path remains unchosen]
5. **What should `qai serve` do when the database is unmigrated?**
   - What we know: doctor must report readiness, and ordinary commands must not auto-migrate. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:24-28]
   - What's unclear: whether serve should exit immediately, stay alive for health endpoints, or run a read-only degraded mode.
   - Recommendation: fail startup with the same migration remedy and exit code as other ordinary commands; do not silently migrate. [ASSUMED: recommended operator behavior]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Rust compiler | All workspace compilation | ✓ | `1.97.1` | None; pinned toolchain is installed. [VERIFIED: rust-toolchain.toml:1-5] [VERIFIED: live `rustc --version`] |
| Cargo | Build/test/lockfile | ✓ | `1.97.1` | None. [VERIFIED: live `cargo --version`] |
| SQLite runtime | Migrations, storage, jobs, audit | ✓ | `3.51.0` CLI; SQLx bundled runtime | None; use the existing SQLx adapter. [VERIFIED: live sqlite probe] [VERIFIED: Cargo.lock:2406-2415] |
| `cargo-deny` | Supply-chain CI gate | ✗ locally | — | CI installs/enforces it; local `xtask ci` warns/skips when absent. [VERIFIED: .github/workflows/ci.yml:12-25] [VERIFIED: xtask/src/ci.rs:29-48] |
| `cargo-llvm-cov` | Coverage gate | ✓ | command available | None for this phase. [VERIFIED: live command probe] |
| Docker daemon | Container integration, not Phase 1 | ✗ daemon | client `29.6.1`; socket absent | Out of scope; use local Cargo/SQLite verification. [VERIFIED: live `docker info` probe] [VERIFIED: docs/05-followups/open-questions.md:3-17] |
| External job broker | Durable jobs | Not needed | — | SQLite `jobs` table and lease queue are the locked local design. [VERIFIED: migrations/sqlite/0004_jobs.up.sql:1-42] |

**Missing dependencies with no fallback:** none for the Phase 1 code/test scope. **Missing dependencies with fallback:** `cargo-deny` is locally missing but CI is the enforcing fallback; Docker runtime verification is unavailable but explicitly outside this phase. [VERIFIED: .github/workflows/ci.yml:12-25] [VERIFIED: docs/05-followups/open-questions.md:3-17]

## Validation Architecture

`.planning/config.json` is absent in this repository, so Nyquist validation is treated as enabled for this phase. [VERIFIED: file-not-found probe, 2026-09-24]

### Test Framework

| Property | Value |
|---|---|
| Framework | Rust built-in `#[test]`/`#[tokio::test]`, `trycmd` for the real CLI, `tempfile` for isolated state, and `proptest` where a property is useful. [VERIFIED: .planning/codebase/TESTING.md:9-29] |
| Config file | None dedicated to Phase 1; Cargo workspace and existing test targets are the configuration. [VERIFIED: .planning/codebase/TESTING.md:11-15] |
| Quick run command | `cargo test -p config --lib && cargo test -p testkit --test config_precedence && cargo test -p cli --test catalog && cargo test -p jobs --lib && cargo test -p storage-sqlite --test recovery_jobs --test outbox_idempotency --test integrity_audit` |
| Full suite command | `cargo test --workspace` |
| Formatting/lint gate | `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings` |
| Architecture/migrations | `cargo run -q -p xtask -- arch-check && cargo run -q -p xtask -- migrate-check` |
| Full pre-merge gate | `cargo run -p xtask -- ci` [VERIFIED: .planning/codebase/TESTING.md:21-29] |

### Phase Requirements → Test Map

| Req ID | Behavior | Test type | Automated command | File exists? |
|---|---|---|---|---|
| REQ-product-vision | Foundation CLI remains narrow and the stable groups are visible. | CLI snapshot | `cargo test -p cli --test catalog` plus new foundation help case. [VERIFIED: crates/cli/tests/catalog.rs:13-21] | Existing catalog: ✅; foundation help case: ❌ Wave 0. [ASSUMED: proposed case] |
| REQ-product-principles | State changes carry traceable audit/provenance and secrets are redacted. | unit/integration/security | `cargo test -p storage-sqlite --test integrity_audit`; `cargo test -p testkit --test secret_leak`; new audited-mutation matrix. [VERIFIED: crates/storage-sqlite/tests/integrity_audit.rs:1-20] | Existing audit/secret tests: ✅; mutation matrix: ❌ Wave 0. [ASSUMED] |
| REQ-goals-non-goals | No later Quran/search/server capability is pulled into foundation tasks. | review/CLI contract | Foundation scope test plus `cargo test -p cli --test catalog`; manual diff review. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:7-12] | Existing harness: ✅; scope assertion: ❌ Wave 0. [ASSUMED] |
| REQ-engineering-baseline | Config, doctor, jobs, cancellation, typed errors, tests, and DX work on real paths. | unit/integration/CLI | `cargo test -p config --lib`; `cargo test -p jobs --lib`; new `crates/cli/tests/foundation.rs` or equivalent. [VERIFIED: .planning/codebase/TESTING.md:32-47] | Existing config/jobs: ✅; foundation CLI: ❌ Wave 0. [ASSUMED: proposed path] |
| REQ-storage-architecture | Migrations are explicit/checksummed; writes are atomic; queue is durable without a broker. | integration | `cargo test -p storage-sqlite --test recovery_jobs --test outbox_idempotency --test integrity_audit`; `cargo run -q -p xtask -- migrate-check`. [VERIFIED: crates/storage-sqlite/tests/recovery_jobs.rs:1-135] | Existing: ✅; lifecycle/audit atomicity additions: ❌ Wave 0. [ASSUMED] |
| REQ-cli-api | Foundation commands expose stable help, redacted JSON, explicit remedies, and job controls. | CLI/trycmd | `cargo test -p cli --test catalog`; new foundation trycmd cases and real `qai --help` check. [VERIFIED: crates/cli/tests/quran.rs:11-20] | Existing harness: ✅; missing cases: ❌ Wave 0. [ASSUMED: proposed path] |
| REQ-architecture-principles-quality | Forbidden workspace and external dependencies fail the architecture gate. | unit/build gate | `cargo test -p xtask`; `cargo run -q -p xtask -- arch-check`; synthetic registry-edge mutation. [VERIFIED: xtask/src/arch.rs:166-190] | Existing path-edge tests: ✅; external-edge test: ❌ Wave 0. [ASSUMED] |

### Sampling Rate

- **Per task commit:** the focused test command for the touched seam, plus `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` when Rust files change. [VERIFIED: .agent/definition-of-done.md:1-12]
- **Per wave merge:** the quick suite for the wave and `cargo run -q -p xtask -- arch-check && cargo run -q -p xtask -- migrate-check`. [VERIFIED: .github/workflows/ci.yml:41-64]
- **Phase gate:** `cargo test --workspace`, `cargo run -p xtask -- ci`, and the five success-criterion evidence commands must be green before `/gsd-verify-work`. [VERIFIED: .planning/codebase/TESTING.md:21-29] [VERIFIED: .planning/ROADMAP.md:34-40]

### Wave 0 Gaps

- [ ] `crates/cli/tests/foundation.rs` or an equivalent trycmd harness — covers the seven stable groups, effective config origins, migration remedies, job cancel/retry, and audit verify output. [ASSUMED: proposed path/name]
- [ ] `crates/application/tests/phase1_foundation.rs` or focused existing-suite additions — covers audited mutation rollback, job lifecycle events, named checkpoints, and cancellation outcomes. [ASSUMED: proposed path/name]
- [ ] `crates/storage-sqlite/tests/phase1_jobs_audit.rs` or focused additions — covers same-transaction audit/outbox and durable checkpoint behavior against real SQLite. [ASSUMED: proposed path/name]
- [ ] `xtask` synthetic external-dependency mutation fixture — covers the currently untested registry/git edge policy. [ASSUMED: proposed fixture]
- [ ] `crates/testkit` shared fixture extension — isolated config/data/audit seed and tamper helper; the crate already exists as the dev-only fixture boundary. [VERIFIED: .planning/codebase/ARCHITECTURE.md:73-77] [ASSUMED: proposed helper names]
- [ ] Framework install: **none**; Rust, Cargo, Tokio, SQLx, Clap, trycmd, tempfile, and the existing test targets are present. [VERIFIED: Cargo.toml:70-90] [VERIFIED: live environment probe, 2026-09-24]

## Security Domain

Security enforcement is treated as enabled because `.planning/config.json` is absent. [VERIFIED: file-not-found probe, 2026-09-24] ASVS is used as a review vocabulary, not as a replacement for the project’s locked ADRs. [CITED: https://github.com/OWASP/ASVS] [ASSUMED: exact ASVS 5.0 control IDs are not locked here]

### Applicable ASVS Categories

| ASVS category | Applies | Standard control / evidence |
|---|---|---|
| V2 Authentication | Limited / not a Phase 1 remote-auth deliverable | Keep the local CLI/loopback model; remote auth remains a later server concern. Do not expose a non-loopback bind without the existing TLS/auth policy. [VERIFIED: .planning/PROJECT.md:62-66] [VERIFIED: crates/config/src/lib.rs:262-297] |
| V3 Session Management | No for the local foundation CLI; relevant to future server | Do not add session state to Phase 1; preserve request/lease identity for jobs. [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:121-126] |
| V4 Access Control | Yes | Human approval for canonical changes, local operator identity, lease-owner checks, and deny-by-default side effects. [VERIFIED: .agent/coding-rules.md:10-22] [VERIFIED: crates/application/src/quran.rs:240-259] |
| V5 Input Validation | Yes | Clap parsing, typed config validation, schema/payload validation, bound SQL parameters, and explicit migration checks. [VERIFIED: crates/jobs/src/worker.rs:113-125] [VERIFIED: crates/config/src/lib.rs:262-297] [VERIFIED: crates/application/src/db.rs:225-271] |
| V6 Cryptography | Yes | Reuse the existing SHA-256 audit/content-hash contracts and signed-manifest/secret cryptography; never hand-roll a new hash or cipher for this phase. [VERIFIED: .planning/PROJECT.md:60-66] [VERIFIED: crates/application/src/audit_bridge.rs:21-38] |

### Known Threat Patterns for the Rust/SQLite/CLI Stack

| Pattern | STRIDE | Standard mitigation |
|---|---|---|
| Secret leakage through config/doctor/job/audit output | Information disclosure | Redact structured values and human text at the emission boundary; use `Secret<T>`/secret refs, never raw values. [VERIFIED: crates/cli/src/lib.rs:407-419] [VERIFIED: .agent/coding-rules.md:17-22] |
| Doctor writes files or follows an unsafe path | Tampering / elevation of privilege | Metadata-only checks, canonicalized existing paths, no symlink following by default, and a no-side-effect test. [VERIFIED: crates/config/src/lib.rs:201-208] [VERIFIED: .agent/coding-rules.md:13-16] |
| Audit row tampering or chain-gap blindness | Tampering / repudiation | Append-only triggers, row hash recomputation, sequence checks, and `qai audit verify`; doctor must reuse the verifier. [VERIFIED: migrations/sqlite/0005_audit.up.sql:1-26] [VERIFIED: crates/application/src/audit_bridge.rs:254-270] |
| Migration checksum drift or implicit schema mutation | Tampering | Append-only checksummed migration files, explicit migration command, and `migrate-check`. [VERIFIED: crates/storage-sqlite/src/migrate.rs:168-219] [VERIFIED: .planning/PROJECT.md:60-66] |
| SQL injection through catalog/config/job identifiers | Tampering | Use SQLx bound parameters; current catalog helpers bind user IDs and values. [VERIFIED: crates/application/src/db.rs:241-251] [VERIFIED: crates/storage-sqlite/src/lib.rs:1010-1015] |
| Cancellation race or lease takeover | Elevation of privilege / denial of service | Persist request, check lease ownership, make handlers idempotent, and audit terminal outcomes. [VERIFIED: crates/storage-sqlite/src/lib.rs:850-939] [VERIFIED: .planning/phases/01-foundations/01-CONTEXT.md:36-40] |
| Forbidden dependency enters through registry/git edge | Tampering / supply-chain | Extend `arch-check` to external sources and retain `cargo-deny` as a separate CI gate. [VERIFIED: xtask/src/arch.rs:117-139] [VERIFIED: .github/workflows/ci.yml:12-25] |

## Sources

### Primary (HIGH confidence — opened in this session)

- `crates/config/src/lib.rs:219-873` — precedence, validation, environment/CLI merge, and origin recording.
- `crates/config/src/origin.rs:4-68` — `ValueOrigin` and `OriginMap` definitions.
- `crates/cli/src/lib.rs:14-352,381-449,612-731` — command tree, dispatch, effective-config gap, source/job/secret handlers.
- `crates/cli/src/doctor.rs:1-207,237-300,378-458,528-546` — read-only claim, write probes, migration check, job/audit checks.
- `crates/application/src/db.rs:17-84,128-199` — migration status and read-only probe.
- `crates/storage-sqlite/src/lib.rs:58-122,230-307,818-1037` — constructors, UoW, job persistence, cancel/checkpoint.
- `crates/storage/src/workflows.rs:27-109` — current source/provenance/outbox transaction helpers.
- `crates/application/src/audit_bridge.rs:200-270` — audit append and persisted verification.
- `crates/jobs/src/lib.rs:90-113,225-325` and `crates/jobs/src/worker.rs:18-239` — job states, context, worker retry/cancellation/checkpoint behavior.
- `crates/application/src/job_queue.rs:19-135` and `crates/application/src/quran.rs:1124-1169` — SQLite queue assembly and inline import worker.
- `migrations/sqlite/0004_jobs.up.sql:1-42` and `migrations/sqlite/0005_audit.up.sql:1-26` — current durable schemas and append-only triggers.
- `xtask/src/arch.rs:102-164`, `xtask/allowlist.toml:1-101`, `.github/workflows/ci.yml:47-64` — architecture rule and CI enforcement.
- `Cargo.toml:3-120`, `Cargo.lock:394-398,2406-2410,2738-2742,3073-3077`, `rust-toolchain.toml:1-5` — pinned stack.
- `crates/testkit/tests/config_precedence.rs:1-74`, `crates/storage-sqlite/tests/recovery_jobs.rs:1-135`, `crates/cli/tests/quran.rs:11-20` — existing repeatable evidence.
- `.planning/phases/01-foundations/01-CONTEXT.md:15-128`, `.planning/ROADMAP.md:30-40`, `.planning/REQUIREMENTS.md:11-67` — locked phase scope, success criteria, and requirement mapping.
- `AGENTS.md:44-172` and `.agent/coding-rules.md:1-22` — project constraints.

### Secondary (MEDIUM confidence)

- [Tokio graceful shutdown](https://tokio.rs/tokio/topics/shutdown) — official guidance fetched this session; use for signal → cooperative notification → await completion. [CITED: https://tokio.rs/tokio/topics/shutdown]
- [SQLx 0.8.6 `Transaction`](https://docs.rs/sqlx/0.8.6/sqlx/struct.Transaction.html) — official API reference for explicit commit/rollback semantics. [CITED: https://docs.rs/sqlx/0.8.6/sqlx/struct.Transaction.html]
- [Clap 4.6.6 derive API](https://docs.rs/clap/4.6.6/clap/_derive/) — official derive/command documentation. [CITED: https://docs.rs/clap/4.6.6/clap/_derive/]
- [trycmd documentation](https://github.com/assert-rs/trycmd) — official project documentation for real CLI snapshot cases. [CITED: https://github.com/assert-rs/trycmd]
- Existing `.planning/codebase/*.md` maps — useful orientation, but their header dates are 2026-09-22 and they are not treated as stronger than opened source/current probes. [VERIFIED: .planning/codebase/ARCHITECTURE.md:1-9] [VERIFIED: .planning/codebase/STACK.md:1-7]

### Tertiary (LOW confidence)

- [OWASP ASVS repository](https://github.com/OWASP/ASVS) — web-search-level security vocabulary only; exact control IDs and version-specific applicability require owner/security review. [CITED: https://github.com/OWASP/ASVS]
- Registry search/API probe — current search results were observed, but publish dates were not obtained and no compatibility conclusion is drawn. [VERIFIED: live registry probe, 2026-09-24] [ASSUMED: no upgrade recommendation]

## Metadata

**Confidence breakdown:**

- Standard stack: **HIGH** — toolchain and resolved package versions were read from `rust-toolchain.toml`/`Cargo.lock`, and the binary/tool probes ran. [VERIFIED: rust-toolchain.toml:1-5] [VERIFIED: Cargo.lock:394-398] [VERIFIED: Cargo.lock:2406-2410] [VERIFIED: Cargo.lock:2738-2742] [VERIFIED: Cargo.lock:3073-3077]
- Architecture: **HIGH** — relevant source definitions, migration schemas, allowlist, and CI jobs were opened and live arch/migration checks passed. [VERIFIED: crates/storage-sqlite/src/lib.rs:230-307] [VERIFIED: xtask/src/arch.rs:102-164] [VERIFIED: live arch/migrate output, 2026-09-24]
- Pitfalls: **HIGH** — the principal CLI/doctor/audit gaps were reproduced against the real `qai` binary, not inferred from stale maps. [VERIFIED: live probes, 2026-09-24]
- External API guidance: **MEDIUM** — official Tokio documentation was fetched; SQLx/Clap/trycmd references are official URLs and project patterns, but not all pages were available through the live fetch path in this session. [CITED: https://tokio.rs/tokio/topics/shutdown] [CITED: https://docs.rs/sqlx/0.8.6/sqlx/struct.Transaction.html] [CITED: https://docs.rs/clap/4.6.6/clap/_derive/] [CITED: https://github.com/assert-rs/trycmd]
- Future implementation details: **LOW/MEDIUM** — migration naming, checkpoint schema, source/secret scope, and external policy representation are explicitly marked `[ASSUMED]`.

**Research date:** 2026-09-24
**Valid until:** 2026-10-24 for stable repository findings; re-check external package/API guidance sooner if the lockfile or Phase 1 decisions change.
