---
phase: 01-foundations
plan: 02
subsystem: database
tags: [rust, sqlite, sqlx, unit-of-work, audit-chain, outbox, provenance, fault-injection]

# Dependency graph
requires:
  - phase: 01-01
    provides: persisted-verifier audit authority (`verify_persisted_audit`), existing-database guards, real-binary foundation suite
provides:
  - `AuditedMutation` application seam (one shared-UoW composition for domain + provenance + outbox + chained audit)
  - All authoritative Quran lifecycle paths routed through the audited composition (approval, verification, staging, activation, rollback, deprecate, ensure_source_version)
  - Real-SQLite failure-matrix rollback evidence (6 boundaries) + tamper/recovery evidence
  - Same-UoW audit commit/rollback cases at the storage boundary
affects: [01-03 (job lifecycle audit consumes the same-UoW composition pattern), 01-04 (serve/enqueue boundary), 01-05 (final gates, TASK-001 closure)]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
# Same estimateTokens scale (chars/4 over the realized diff), never a harness token count.
actuals:
  tokens: 14731    # chars/4 over the 2 plan-task commit diffs (git diff 8d05b06~1 70f8083 = 58926 chars)
  tasks: 2         # tasks completed
  commits: 2       # plan-task commits (8d05b06 + 70f8083); SUMMARY + tracking commits separate
plan_head_before: 8d05b06~1

# Tech tracking
tech-stack:
  added: []
  patterns: [single application audited-mutation seam over the existing UnitOfWork, fault-injection via SQL triggers and failing repository/UoW seams, same-UoW audit commit/rollback assertions at the storage boundary]

key-files:
  created: [crates/application/tests/phase1_foundation.rs]
  modified: [crates/application/src/audit_bridge.rs, crates/storage/src/workflows.rs, crates/application/src/quran.rs, crates/application/src/quran_cli.rs, crates/storage-sqlite/tests/commit_bounds_outbox.rs, crates/storage-sqlite/tests/integrity_audit.rs, docs/04-tasks/active/TASK-001-foundation-gap-closure.md]

key-decisions:
  - "One `AuditedMutation` application seam over the existing UnitOfWork — no second transaction manager, no new event hash format, no parallel audit writer (D-01, D-10)"
  - "storage::workflows stays backend-neutral; audit enforcement lives in application wrappers so direct callers cannot silently commit partial lifecycle changes (D-09, D-10)"
  - "Importer staging completes inside the audited transaction (SourceStaged event) while canonical activation stays approval-gated and importer staging-only (D-04, D-09)"
  - "Read-only diagnostics and derived-cache rebuilds stage zero audit events unless authoritative state changes (D-09)"

patterns-established:
  - "Audited composition order: authoritative repository write → required provenance/outbox writes → one chained audit event → caller commits; every error path returns before commit"
  - "Failure-matrix coverage per write boundary: domain, provenance, outbox alloc/enqueue, audit append, commit — each asserts zero partial rows and unchanged prior state"

requirements-completed: [REQ-product-principles, REQ-engineering-baseline, REQ-storage-architecture, REQ-cli-api]

# Coverage metadata (#1602) — one entry per shipped deliverable. Drives DETERMINISTIC UAT routing in verify-work.
coverage:
  - id: D1
    description: "AuditedMutation seam: every authoritative lifecycle mutation stages domain + provenance + outbox + one chained audit event in one UnitOfWork, proven by real-SQLite commit + rollback cases"
    requirement: "REQ-product-principles"
    verification:
      - kind: integration
        ref: "crates/application/tests/phase1_foundation.rs#audited_mutation_commits_domain_provenance_outbox_and_audit_together + audited_mutation_rolls_back_when_audit_append_fails + audited_mutation_records_approval_with_its_grant_event"
        status: pass
    human_judgment: false
  - id: D2
    description: "Atomic rollback at every write boundary: six-boundary failure matrix leaves zero partial rows with prior state unchanged"
    requirement: "REQ-engineering-baseline"
    verification:
      - kind: integration
        ref: "crates/application/tests/phase1_foundation.rs#audited_mutation_failure_matrix_rolls_back_every_boundary"
        status: pass
      - kind: integration
        ref: "crates/storage-sqlite/tests/commit_bounds_outbox.rs#audited_activation_commits_domain_outbox_and_audit_together + audited_activation_rolled_back_discards_outbox_and_audit"
        status: pass
    human_judgment: false
  - id: D3
    description: "Approval-gated canonical activation preserved: importer staging-only, missing/denied/mismatched approvals rejected"
    requirement: "REQ-product-principles"
    verification:
      - kind: integration
        ref: "crates/application/tests/quran_import.rs#activation_service_requires_a_granted_approval"
        status: pass
    human_judgment: false
  - id: D4
    description: "Persisted audit recovery guidance: tampered chain reports the offending sequence with the `qai audit verify` remedy without mutating the DB"
    requirement: "REQ-cli-api"
    verification:
      - kind: integration
        ref: "crates/application/tests/phase1_foundation.rs#tampered_chain_reports_offending_sequence_with_verify_remedy"
        status: pass
      - kind: integration
        ref: "crates/storage-sqlite/tests/integrity_audit.rs#tampered_sequence_is_identifiable_from_storage_reads"
        status: pass
    human_judgment: false
  - id: D5
    description: "Read-only and derived-cache paths stage no audit events unless authoritative state changes"
    requirement: "REQ-storage-architecture"
    verification:
      - kind: integration
        ref: "crates/application/tests/phase1_foundation.rs#read_only_paths_and_derived_rebuilds_stage_no_audit_events"
        status: pass
    human_judgment: false

# Metrics
duration: 20min
completed: 2026-09-27
status: complete
---

# Phase 01 Plan 02: Audited Durable Mutations Summary

**One shared-UnitOfWork `AuditedMutation` seam routes every authoritative Quran lifecycle path (approval, staging, activation, rollback, verification) through domain → provenance/outbox → chained-audit ordering, with a 6-boundary real-SQLite failure matrix proving zero partial rows.**

## Performance

- **Duration:** 20min (close-out run: verify + SUMMARY + tracking; implementation by the prior interrupted session)
- **Started:** 2026-09-27T05:30:00Z (approx, close-out resume)
- **Completed:** 2026-09-27T05:50:00Z (approx)
- **Tasks:** 2 (01-02-01 audited composition, 01-02-02 failure matrix + recovery proof)
- **Files modified:** 8 (1 created, 7 modified incl. evidence record)

## Accomplishments

- `AuditedMutation` in `application::audit_bridge` sequences authoritative writes on the
  caller's existing `&mut dyn storage::UnitOfWork`: domain write first, then required
  provenance/outbox writes, then one hash-chained audit event; caller commits, every
  error path returns before commit (D-09, D-10). No new crate, broker, compensating
  transaction, or hash format.
- All authoritative paths routed through the composition: `record_approval` (row +
  `ApprovalGranted` in the same tx), `record_edition_verification` (`SourceApproved`),
  `activate/rollback/deprecate_edition` (shared `audit_activation` leg),
  `QuranImportHandler` staging completion (`SourceStaged` inside the final
  staging/result transaction, importer still staging-only), `ensure_source_version`
  (`SourceImported`). Approval negatives (missing/denied/subject-mismatch) unchanged.
- Real-SQLite failure matrix (`audited_mutation_failure_matrix_rolls_back_every_boundary`)
  fails each of six boundaries independently (domain NotFound, provenance duplicate-PK,
  generation trigger, outbox idempotency Conflict, audit trigger, no-commit drop) and
  asserts zero partial domain/provenance/audit/outbox rows with prior state unchanged.
- Same-UoW audit commit/rollback proven at the storage boundary
  (`commit_bounds_outbox` +2 cases); tampered chains report the exact offending
  sequence with the `qai audit verify` remedy without mutating the DB
  (`phase1_foundation` tamper case + `integrity_audit` storage case, D-12).
- Read-only paths and full derived-forms rebuilds stage zero audit events and leave
  canonical rows identical (D-09).
- Plan `<verify>` re-run live in this close-out: `phase1_foundation` 6/6,
  `commit_bounds_outbox` 5/5, `integrity_audit` 3/3, `application --lib` 40/40,
  `cli --test foundation` 15/15, `arch-check` OK, `migrate-check` OK (21 migrations).
- TASK-001 record carries both 01-02 evidence rows (commands + observed counts +
  D-09/D-10/D-12 citations); TASK-001 stays Active — closure owned by 01-05-02.

## Task Commits

Each task was committed atomically (prior-session work verified, not redone):

1. **Task 01-02-01: Compose audited durable mutations** — `8d05b06` (feat: AuditedMutation
   seam, audited wrappers, quran.rs/quran_cli.rs routing, phase1_foundation tests, TASK-001 row)
2. **Task 01-02-02: Prove atomic rollback and recovery guidance** — `70f8083` (test:
   failure matrix, commit_bounds_outbox same-UoW cases, integrity_audit tamper case, TASK-001 row)

**Plan metadata:** this SUMMARY + STATE/ROADMAP updates (final tracking commit, see completion message).

## Files Created/Modified

- `crates/application/tests/phase1_foundation.rs` — created; 6 real-SQLite acceptance cases
  (audited commit, audit-failure rollback, approval+grant, 6-boundary failure matrix,
  tamper+remedy, read-only/derived no-audit)
- `crates/application/src/audit_bridge.rs` — `AuditedMutation` composition (+140 lines)
- `crates/storage/src/workflows.rs` — backend-neutral contract kept; audit-contract docs
  + call-through requirements (D-09/D-10)
- `crates/application/src/quran.rs` — approval/verification/activation/rollback/import
  paths routed through the composition
- `crates/application/src/quran_cli.rs` — `ensure_source_version` audited wiring
- `crates/storage-sqlite/tests/commit_bounds_outbox.rs` — same-UoW audit commit/rollback (+77)
- `crates/storage-sqlite/tests/integrity_audit.rs` — tampered-sequence identification (+53)
- `docs/04-tasks/active/TASK-001-foundation-gap-closure.md` — 01-02-01 + 01-02-02 evidence rows

## Decisions Made

- Reuse-only seam per D-01…D-04: existing `UnitOfWork`, `append_audit_event` hash recipe,
  `ApprovalToken`/`CanonicalWriter` gate, checked-in migrations. Nothing replaced.
- Audit enforcement placed in application wrappers (not inside `storage::workflows`) so the
  storage layer stays backend-neutral while direct callers cannot bypass the audit contract.
- Importer staging audited in-transaction without granting activation: the composition covers
  staging completion; the no-canonical-activation and no-approval-mint rules are untouched.
- Read-only/derived paths explicitly excluded from the audited set with a negative test,
  per D-09's authoritative-state criterion.

## Deviations from Plan

None - plan executed exactly as written (implementation by the prior session; this run
verified the committed code against every must-have truth and re-ran the full `<verify>`
chain green before writing this SUMMARY).

## Issues Encountered

- Prior executor was interrupted after committing both task implementations but before
  writing SUMMARY.md. Both commits (`8d05b06`, `70f8083`) were verified via `git show
  --stat` and their contents spot-checked against the plan's must-have truths; no
  reimplementation was needed. TASK-001 evidence rows were already complete — no top-up required.
- `git status --porcelain` shows only untracked `.planning/milestone.lock` from another
  session; left untouched per the dispatch directive.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 01-03 (durable job checkpoints/retry/cancel + lifecycle audit, which consumes
  the same-UoW composition pattern; D-11 job lifecycle stays delegated to 01-03).
- No blockers. TASK-001 remains Active; closure owned by 01-05-02.

## Self-Check: PASSED

- `crates/application/tests/phase1_foundation.rs`, `crates/application/src/audit_bridge.rs`,
  `docs/04-tasks/active/TASK-001-foundation-gap-closure.md` all FOUND.
- Commits `8d05b06`, `70f8083` FOUND via `git show --stat`.
- No stub patterns (`TODO|FIXME|placeholder|unimplemented|todo!`) in the plan diff.
- No new network/auth/schema trust-boundary surface outside the plan `<threat_model>` — no Threat Flags section.

---
*Phase: 01-foundations*
*Completed: 2026-09-27*
