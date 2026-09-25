---
phase: 02-canonical-quran-core
plan: 04
subsystem: database
tags: [quran, immutability, triggers, migration, provenance, approval-token, canonical-writer, rollback, caching]

requires:
  - phase: 01-foundations
    provides: append-only migration runner + checksum ledger, approval-gated activation/rollback transaction inside one UnitOfWork, generation-keyed reader cache, SQLite integration harness
  - phase: 02-canonical-quran-core
    provides: plan 02-01's 0020 edition identity/default columns and the persisted ApprovalRow fetch path on the activation surface

provides:
  - migration 0021 — BEFORE UPDATE/DELETE coded aborts for quran_surahs, quran_token_separators, quran_segments, quran_divisions, translation_editions, translation_passages, word_glosses, plus DELETE on quran_editions and immutability of the 0020 identity/default columns (QAI-QUR-0006…0021)
  - provenance::ApprovalToken::from_approval_row plus a crate-private raw constructor (no public forge path)
  - provenance::ApprovalGate implementing CanonicalWriter, with subject-bound begin/commit/abort sessions
  - application::{activate_edition, rollback_edition, record_edition_verification, deprecate_edition} opening and committing an ApprovalGate session inside the existing single UnitOfWork
  - crates/quran-corpus/tests/import_path.rs — importer code-path audit (no token, no canonical write, terminal state Staged)
  - rollback rejection + rollback cache-invalidation coverage (QC-01)
  - docs/07-technical/quran-canonical-core-decisions.md — quran_segments scope, trigger-code map, wire-not-retire rationale

affects: [02-07 phase evidence of record, phase 3 morphology (canonical segments)]

actuals:
  tokens: 15137    # chars/4 over the realized diff (60546 chars, code + docs)
  tasks: 3
  commits: 8       # MEASURED: git rev-list --count d52024e..HEAD (task 2 was committed across six commits by the resumed harness)
  plan_head_before: d52024edf4940a8645e525c13812d2ac7920320c

tech-stack:
  added: []
  patterns:
    - "Canonical insert-only fence: one BEFORE UPDATE and one BEFORE DELETE RAISE(ABORT) per table with a stable QAI-QUR-<NS>-NNNN code, appended as a new migration"
    - "Mint-not-construct: ApprovalToken is only reachable through from_approval_row, which requires a persisted granted approval row with a non-empty subject"
    - "Gate-bracketed publication: open a CanonicalWriter session (subject-bound) before the storage mutator and commit it after, inside the same UnitOfWork"
    - "Source-audit test: strip comments from importer source and assert forbidden canonical-write/approval-token vocabulary is absent"

key-files:
  created:
    - migrations/sqlite/0021_canonical_write_fence.up.sql
    - crates/quran-corpus/tests/import_path.rs
    - docs/07-technical/quran-canonical-core-decisions.md
  modified:
    - migrations/sqlite/checksums.json
    - crates/storage-sqlite/tests/quran.rs
    - crates/provenance/src/lib.rs
    - crates/application/src/quran.rs
    - crates/application/src/quran_cli.rs
    - crates/application/tests/quran_import.rs
    - crates/application/tests/quran_reader.rs

key-decisions:
  - "Migration 0021 is append-only and additive: 0007–0020 stay byte-identical; translation rows are insert-only published artifacts, so a future status transition needs a new forward migration, never an edit."
  - "Canonical quran_segments stays empty in Phase 2 (reserved for the morphology phase); the frozen activation copy list is unchanged and a future phase adds a forward migration plus a copy step."
  - "The CanonicalWriter gate is wired, not retired: ADR-0000 locks canonical immutability as a type-level property and .agent/coding-rules.md states the invariant, so the type cannot be dead code."
  - "ApprovalToken::new is crate-private; the only public mint path is from_approval_row (decision == 'approved' and non-empty subject_urn), proven behaviourally and by a source scan."
  - "Rollback is approval-gated exactly like activation; a mismatched subject is rejected before the storage mutator runs and the rejected paths leave pointer, canonical rows, and corpus_generation untouched."

patterns-established:
  - "Coded trigger taxonomy: new fence tables/verbs take the next unused QAI-QUR code; existing codes are never reused or renumbered (ADR-0010)."
  - "A second BEFORE UPDATE OF trigger fences new identity/default columns without DROP/recreating the existing 0007 trigger."
  - "Rejection-proof tests snapshot the active pointer, canonical row count, and corpus_generation and assert equality after each refused mutation."

requirements-completed: [REQ-quran-corpus, REQ-ingestion-validation-eval]

coverage:
  - id: D1
    description: "Append-only migration 0021 fences every previously untriggered canonical table and verb with a unique coded abort, plus DELETE on quran_editions and immutability of the 0020 identity/default columns."
    requirement: "REQ-quran-corpus"
    verification:
      - kind: integration
        ref: "crates/storage-sqlite/tests/quran.rs#canonical_triggers_abort_raw_writes_with_codes"
        status: pass
      - kind: integration
        ref: "crates/storage-sqlite/tests/quran.rs#canonical_tables_declare_the_trigger_set"
        status: pass
      - kind: other
        ref: "cargo run -q -p xtask -- migrate-check (21 migrations ordered; checksums stable)"
        status: pass
      - kind: unit
        ref: "cargo test -p storage --lib quran (gated mutator surface unchanged)"
        status: pass
    human_judgment: false
  - id: D2
    description: "ApprovalToken is only mintable from a persisted granted approval row and names its exact subject; ApprovalGate implements CanonicalWriter and gates activate/rollback/verify/deprecate inside the existing single UnitOfWork."
    requirement: "REQ-ingestion-validation-eval"
    verification:
      - kind: unit
        ref: "crates/provenance/src/lib.rs#approved_row_mints_a_token_bound_to_its_subject"
        status: pass
      - kind: unit
        ref: "crates/provenance/src/lib.rs#denied_approval_row_yields_the_not_granted_error"
        status: pass
      - kind: unit
        ref: "crates/provenance/src/lib.rs#subject_less_approval_row_yields_the_missing_subject_error"
        status: pass
      - kind: unit
        ref: "crates/provenance/src/lib.rs#no_public_constructor_bypasses_the_row_check"
        status: pass
      - kind: unit
        ref: "crates/provenance/src/lib.rs#approval_gate_rejects_a_mismatched_subject_before_any_write"
        status: pass
      - kind: unit
        ref: "crates/provenance/src/lib.rs#approval_gate_tracks_open_committed_and_aborted_sessions"
        status: pass
      - kind: integration
        ref: "crates/application/tests/quran_import.rs#activation_service_requires_a_granted_approval"
        status: pass
      - kind: integration
        ref: "crates/application/tests/quran_verification.rs#verification_requires_a_granted_approval_for_the_exact_urn"
        status: pass
    human_judgment: false
  - id: D3
    description: "The importer has no code path to canonical tables: no ApprovalToken/CanonicalWriter reference, no activation/rollback call, no canonical-table INSERT, and a terminal run state of Staged."
    requirement: "REQ-ingestion-validation-eval"
    verification:
      - kind: integration
        ref: "crates/quran-corpus/tests/import_path.rs#importer_holds_no_approval_token_or_canonical_writer"
        status: pass
      - kind: integration
        ref: "crates/quran-corpus/tests/import_path.rs#importer_never_calls_an_activation_or_rollback_mutator"
        status: pass
      - kind: integration
        ref: "crates/quran-corpus/tests/import_path.rs#importer_never_inserts_into_a_canonical_table"
        status: pass
      - kind: integration
        ref: "crates/quran-corpus/tests/import_path.rs#importer_only_uses_staging_writes_and_ends_staged"
        status: pass
    human_judgment: false
  - id: D4
    description: "Rollback rejection is proven (missing/denied/mismatched approval and rollback-to-active leave pointer, canonical rows, and generation unchanged), rollback invalidates the reader cache (no stale v2 text, generation strictly increases), and the quran_segments scope plus trigger-code map are recorded."
    requirement: "REQ-quran-corpus"
    verification:
      - kind: integration
        ref: "crates/application/tests/quran_import.rs#rejected_rollbacks_leave_canonical_state_untouched"
        status: pass
      - kind: integration
        ref: "crates/application/tests/quran_reader.rs#cache_serves_no_stale_text_after_rollback"
        status: pass
      - kind: other
        ref: "test -s docs/07-technical/quran-canonical-core-decisions.md"
        status: pass
    human_judgment: false

duration: 30 min
completed: 2026-09-25
status: complete
---

# Phase 02 Plan 04: Canonical-write fence completion Summary

**Migration 0021 fences every canonical table with coded aborts, `ApprovalToken` is only mintable from a persisted granted approval row and gates every publication through a real `ApprovalGate`, the importer is audited to have no canonical path, and rollback rejection plus cache invalidation are proven.**

## Performance

- **Duration:** ~30 min (resumed after a server restart; task 1 was already committed, task 2 in-flight work adopted)
- **Started:** 2026-09-25T13:41:35Z (task 1 commit clock; this executor resumed after restart)
- **Completed:** 2026-09-25T14:11:47Z
- **Tasks:** 3
- **Files modified:** 11 (3 created, 8 modified)

## Accomplishments

- Completed the SQL fence (D-13a): migration `0021` adds `QAI-QUR-0006`…`0021`, one coded `BEFORE UPDATE`/`BEFORE DELETE` abort per previously untriggered canonical table and verb, a DELETE trigger on `quran_editions`, and a second identity/default-column immutability trigger. `0007`–`0020` are byte-identical and `migrate-check` reports all 21 in order.
- Made the type-level gate real (D-13b, QC-06): `ApprovalToken::new` is crate-private and the only public mint path is `from_approval_row`, which requires a persisted row with `decision = "approved"` and a non-empty `subject_urn`. `ApprovalGate` implements `CanonicalWriter` and is called on `activate_edition`, `rollback_edition`, `record_edition_verification`, and `deprecate_edition`, all inside the existing single `UnitOfWork` — so a subject mismatch aborts before any canonical write.
- Audited the importer (D-13c, QC-05/QC-06): `import_path.rs` strips comments and proves `import.rs` never references an `ApprovalToken`/`CanonicalWriter`, never calls an activation/rollback mutator, never issues a canonical-table INSERT, and ends at run state `Staged`.
- Closed the uncovered rollback branch (QC-01): missing/denied/subject-mismatched approvals and rollback-to-active each leave the active pointer, canonical rows, and `corpus_generation` unchanged; a v1 → v2 → rollback sequence strictly increases the generation and serves v1 text again with no stale v2 result.
- Recorded the decisions: `docs/07-technical/quran-canonical-core-decisions.md` holds the `quran_segments` staging-only scope, the full `0021` trigger-code map, and the wire-not-retire rationale.

## Task Commits

Each task was committed atomically:

1. **Task 1: Complete the insert-only trigger fence as an append-only migration** - `eefe10e` (feat) — committed in the prior (pre-restart) session
2. **Task 2: Make the approval token non-forgeable and give CanonicalWriter a real implementor** - `3c49a61` (provenance), `ca970e5` (application gate), `3b3ce40` (CLI exit mapping), `2197c8f` (importer audit), `6f6ab34` (wire-vs-retire doc comment)
3. **Task 3: Rollback rejection, cache invalidation, and the quran_segments scope decision** - `6e1eea6` (test)

**Plan metadata:** committed with this SUMMARY (docs: complete plan)

_Note: Task 2's in-flight work was committed by the resumed harness before this executor's first commit (see Deviations)._

## Files Created/Modified

- `migrations/sqlite/0021_canonical_write_fence.up.sql` - append-only trigger set for the remaining canonical tables/verbs plus the 0020 identity/default columns
- `migrations/sqlite/checksums.json` - one new checksum entry for 0021
- `crates/storage-sqlite/tests/quran.rs` - per-table coded abort assertions and the enumerated trigger set
- `crates/provenance/src/lib.rs` - crate-private token constructor, `from_approval_row`, `authorises`, `ApprovalGate`, typed errors `QAI-PROV-0010…0013`
- `crates/application/src/quran.rs` - `check_approval` returns the approval row + token; publication paths bracket the storage mutator with a gate session
- `crates/application/src/quran_cli.rs` - map `ActivationError::Provenance` to the policy exit code
- `crates/quran-corpus/tests/import_path.rs` - importer code-path audit
- `crates/application/tests/quran_import.rs` - `rejected_rollbacks_leave_canonical_state_untouched`
- `crates/application/tests/quran_reader.rs` - `cache_serves_no_stale_text_after_rollback`
- `docs/07-technical/quran-canonical-core-decisions.md` - segments scope, trigger-code map, wire-not-retire rationale

## Decisions Made

- Canonical `quran_segments` stays empty in Phase 2; activation's copy list is untouched and a future morphology phase adds a forward migration plus a copy step.
- Translation rows are fenced insert-only because they are published artifacts; a future status transition is a new forward migration.
- The `CanonicalWriter`/`ApprovalToken` gate is wired rather than retired, because ADR-0000 and `.agent/coding-rules.md` state the invariant as a locked, type-level property.
- The raw token constructor is crate-private; only a persisted granted approval row mints a token, and a subject mismatch is rejected before the storage mutator.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `map_activation_error` became non-exhaustive after adding `ActivationError::Provenance`**
- **Found during:** Task 2 verification (compile)
- **Issue:** Adding the `Provenance(String)` variant to `ActivationError` made the `match &error` in `crates/application/src/quran_cli.rs` non-exhaustive (`E0004`), blocking the whole `application` crate.
- **Fix:** Mapped `E::Provenance(_)` to `exit::POLICY` alongside the other approval-gate failures.
- **Files modified:** crates/application/src/quran_cli.rs
- **Verification:** `cargo check -p provenance -p application -p quran-corpus --tests` clean; `cargo test -p application --test quran_import --test quran_verification` green.
- **Committed in:** `3b3ce40`

**2. [Rule 1 - Bug] Duplicate `Self::EmptyReviewer` match arm in `ActivationError::code()`**
- **Found during:** Task 2 verification (compile warning)
- **Issue:** A merge artifact left `Self::EmptyReviewer => 306` twice, producing an `unreachable_patterns` warning that would fail `clippy -D warnings`.
- **Fix:** Removed the duplicate arm.
- **Files modified:** crates/application/src/quran.rs
- **Verification:** Warning gone; crate compiles clean.
- **Committed in:** `ca970e5`

### Process Deviation

**3. Task 2 in-flight work was committed by the resumed harness, not by this executor**
- **Found during:** Adoption of the interrupted working tree
- **Issue:** The plan was interrupted by a server restart with Task 2 changes uncommitted (`crates/provenance/src/lib.rs`, `crates/application/src/quran.rs`, the untracked `import_path.rs`). Before this executor's first commit, those changes (and this executor's first compile fixes) were committed across six commits (`3c49a61`, `ca970e5`, `3b3ce40`, `2197c8f`, `6b13b80`, `6f6ab34`) with non-GSD scope-free messages.
- **Impact:** Task 2's work is fully committed and verified, but not under a single `feat(02-04):` subject. No code was changed to compensate; this executor verified the committed content by re-running the full Task 2 suite. Task 3 was committed atomically as `test(02-04): …`.

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug) + 1 process deviation
**Impact on plan:** Both auto-fixes were required for the crate to compile and stay clippy-clean; the process deviation is a recording artifact of the interrupted session. No scope creep: the frozen activation/rollback transaction, hash recipes, and validator were not rebuilt.

## Issues Encountered

- The plan's `<output>` line names `02-03-SUMMARY.md`; this executor wrote `02-04-SUMMARY.md` as instructed (the plan file number is authoritative). Noted, not corrected in the plan file.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Criterion 4 ("canonical tables reject all non-approved writes; importer has no code path to canonical tables") now has executable evidence on all three D-13 layers.
- Criterion 1's uncovered rollback-rejection and rollback-invalidation branches are covered; plan 02-07 can cite these tests in `docs/06-progress/phase-02-evidence.md`.
- `REQ-quran-corpus` and `REQ-ingestion-validation-eval` are shared with sibling plans (02-03/02-05/02-06/02-07) that have no SUMMARY yet, so the shared-ID gate keeps them pending.
- Owner gates OD-01/OD-02/OD-03 remain open and recorded, not closed.

## Self-Check: PASSED

- Key files exist: `migrations/sqlite/0021_canonical_write_fence.up.sql`, `crates/quran-corpus/tests/import_path.rs`, `docs/07-technical/quran-canonical-core-decisions.md`.
- Task commits exist: `eefe10e`, `3c49a61`, `ca970e5`, `3b3ce40`, `2197c8f`, `6f6ab34`, `6e1eea6`.
- Plan-level verification re-run green: `cargo test -p storage-sqlite --test quran` (10 passed), `cargo test -p storage --lib quran` (4 passed), `cargo test -p provenance --lib` (10 passed), `cargo test -p quran-corpus --test import_path` (4 passed), `cargo test -p application --test quran_import` (15 passed), `cargo test -p application --test quran_verification` (3 passed), `cargo test -p application --test quran_reader` (13 passed), `cargo run -q -p xtask -- migrate-check` (21 ordered, OK), `cargo run -q -p xtask -- arch-check` (OK), `0007–0020` diff empty.

---

*Phase: 02-canonical-quran-core*
*Completed: 2026-09-25*
