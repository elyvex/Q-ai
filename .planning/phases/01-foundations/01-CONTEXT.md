# Phase 1: Foundations - Context

**Gathered:** 2026-09-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Close evidence-backed gaps in the existing brownfield foundation so a developer can build the workspace, configure it, migrate the local SQLite database explicitly, run stable foundation CLI groups, execute provenance-safe durable state changes, verify the append-only audit chain, and use durable background jobs safely.

This phase reconciles and hardens current capabilities. It does not rebuild working foundation services, roll back later-phase Quran/search/server work, or expand into those phases' capabilities.

</domain>

<decisions>
## Implementation Decisions

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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Contract
- `.planning/ROADMAP.md` §Phase 1 — five foundation success criteria and phase boundary.
- `.planning/REQUIREMENTS.md` — Phase 1 requirements, especially `REQ-product-principles`, `REQ-engineering-baseline`, `REQ-storage-architecture`, and `REQ-cli-api`.
- `.planning/PROJECT.md` — locked decisions, constraints, success metric, and local-first direction.
- `docs/01-requirements/requirements.md` — authoritative living PRD; Phase 0/foundation, configuration, storage, jobs, provenance, audit, CLI, diagnostics, and definition-of-done requirements.

### Locked Architecture Decisions
- `docs/02-architecture/decisions/ADR-0000-project-architecture.md` — layered workspace, single `qai` binary, local-first architecture.
- `docs/02-architecture/decisions/ADR-0001-relational-store.md` — SQLite authority and PostgreSQL-portable contracts.
- `docs/02-architecture/decisions/ADR-0002-migration-strategy.md` — append-only checksummed migrations and forward-only canonical policy.
- `docs/02-architecture/decisions/ADR-0003-durable-job-system.md` — DB-backed leased job system without an external local broker.
- `docs/02-architecture/decisions/ADR-0004-config-precedence.md` — CLI > environment > file > defaults and origin tracking.
- `docs/02-architecture/decisions/ADR-0005-secret-storage.md` — secret references, redacted handling, and secret backends.
- `docs/02-architecture/decisions/ADR-0006-hashing-canonical.md` — canonical JSON and SHA-256 hash contract.
- `docs/02-architecture/decisions/ADR-0007-manifest-format-signing.md` — signed-manifest activation safety.
- `docs/02-architecture/decisions/ADR-0008-provenance-representation.md` — universal provenance record and attribution invariants.
- `docs/02-architecture/decisions/ADR-0009-audit-integrity.md` — append-only hash-chained audit events and verification.
- `docs/02-architecture/decisions/ADR-0010-error-taxonomy.md` — stable diagnostic codes, remedies, and CLI exit mapping.
- `docs/02-architecture/decisions/ADR-0011-observability.md` — tracing/metrics catalog and opt-in telemetry.
- `docs/02-architecture/decisions/ADR-0012-workspace-boundaries.md` — machine-enforced crate dependency boundaries.

### Existing Foundation Evidence
- `.planning/codebase/ARCHITECTURE.md` — current composition root, storage transaction, audit bridge, job queue, and CLI integration patterns.
- `.planning/codebase/CONVENTIONS.md` — typed diagnostics, stable error codes, test placement, and architecture conventions.
- `.planning/codebase/STACK.md` — pinned toolchain, build gates, SQLite runtime, configuration precedence, and test infrastructure.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/config`: layered `Config`, `ValueOrigin`, origin explanation, validation, interpolation, and redacted `Secret<T>` handling.
- `crates/storage` / `crates/storage-sqlite`: `Database`, `ReadTx`, atomic `UnitOfWork`, SQLite pools, checksummed migration runner, repositories, backup, and outbox primitives.
- `crates/provenance` / `crates/audit` / `crates/storage/src/workflows.rs`: provenance persistence and atomic provenance/outbox workflows ready to be extended with audit coverage.
- `crates/jobs`: durable job types, leased `JobStore`, cancellation, checkpoint-capable worker abstractions, and bounded execution primitives.
- `crates/application`: single composition root for config, observability, database, audit bridge, and job-queue wiring.
- `crates/cli`: stable foundation command groups and centralized public exit-code mapping.
- `crates/cli/src/doctor.rs` and `crates/application/src/quran_doctor.rs`: existing read-only diagnostic registry with remedies and next commands.

### Established Patterns
- Layered modular monolith with ports-and-adapters; edges depend on `application`, and concrete storage is not exposed to CLI/server.
- Durable writes use a shared transaction so repository, provenance, audit, and outbox records commit atomically.
- Diagnostics use stable `QAI-*` codes, explicit remedies and next commands, and centralized CLI exit-code mapping.
- Canonical and high-risk writes are approval-gated; importer and ordinary interfaces cannot mint or bypass approval tokens.
- SQLite migrations are append-only and checksum-verified; pending migrations are never silently applied.
- CI enforces formatting, clippy, tests, architecture boundaries, migration integrity, supply-chain policy, and coverage gates.

### Integration Points
- `crates/application/src/db.rs`: database lifecycle and readiness integration.
- `crates/application/src/audit_bridge.rs`: application-level audit/provenance transaction boundary.
- `crates/application/src/job_queue.rs`: queue registration and long-lived worker hosting.
- `crates/application/src/lib.rs`: shared runtime/composition bootstrap.
- `crates/cli/src/lib.rs`: foundation command parsing and dispatch.
- `crates/cli/src/doctor.rs`: first-run readiness and remediation guidance.
- `crates/storage-sqlite/src/migrate.rs`: explicit migration and checksum verification.
- `xtask/src/arch.rs` and `xtask/allowlist.toml`: machine-enforced dependency boundaries.

</code_context>

<specifics>
## Specific Ideas

- Phase planning should begin with a five-row evidence matrix: criterion, existing implementation, repeatable evidence, gap, and planned task if needed.
- Completion should report reused evidence, implemented gaps, and remaining blocked decisions rather than treating all foundation code as new work.
- The operator journey should remain inspect-first and mutation-explicit: diagnose, configure explicitly, migrate explicitly, verify, then operate.

</specifics>

<deferred>
## Deferred Ideas

- Additional Quran search, linguistic, graph, server, and streaming capabilities remain in their roadmap phases, even where partial implementations already exist.
- Remote PostgreSQL/Qdrant deployment, TLS, backup operations beyond foundation primitives, and production management remain in Phase 12.
- The initial Quran dataset and license decision (ADR-0101) remains a Phase 2 input and does not alter the Phase 1 foundation contract.

</deferred>

---

*Phase: 1-Foundations*
*Context gathered: 2026-09-24*
