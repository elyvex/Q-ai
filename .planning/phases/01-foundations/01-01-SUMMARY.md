---
phase: 01-foundations
plan: 01
subsystem: cli
tags: [rust, clap, sqlite, sqlx, audit-chain, doctor, config-precedence, migration]

# Dependency graph
requires:
  - phase: none
    provides: greenfield foundation slice — no prior phase output consumed
provides:
  - Real-binary foundation acceptance suite (`crates/cli/tests/foundation.rs`, 15 cases)
  - Loaded `(Config, OriginMap)` dispatch envelope with redacted, origin-bearing output
  - Read-only `DatabaseReadiness` + existing-database guards (`application::db`, `quran_cli::open_db`)
  - Typed audit diagnostic contract (`QAI-AUD-0003/0004/0005`) shared by `qai audit verify` and doctor
  - Metadata-only doctor probes + persisted-verifier `audit.chain_valid`
  - `scripts/verify-phase1-preservation.sh` live-worktree preservation verifier
affects: [01-02 (same-UoW audit sequencing consumes the verifier), 01-03 (job queue readiness), 01-04 (serve/enqueue boundary), 01-05 (final gates), TASK-001 closure]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
# Same estimateTokens scale (chars/4 over the realized diff), never a harness token count.
actuals:
  tokens: 30150    # chars/4 summed over the 17 plan-task commit diffs (git diff h~1 h)
  tasks: 3         # tasks completed
  commits: 17      # plan-task commits (3 + 5 + 9); see ## Task Commits
plan_head_before: 641b0c2~1

# Tech tracking
tech-stack:
  added: []
  patterns: [real-binary acceptance via CARGO_BIN_EXE_qai, existing-database guard before read paths, single persisted-verifier authority for audit readiness, metadata-only filesystem probes via symlink_metadata]

key-files:
  created: [crates/cli/tests/foundation.rs, scripts/verify-phase1-preservation.sh, docs/04-tasks/active/TASK-001-foundation-gap-closure.md]
  modified: [crates/cli/src/lib.rs, crates/cli/src/doctor.rs, crates/config/src/lib.rs, crates/application/src/db.rs, crates/application/src/audit_bridge.rs, crates/application/src/quran_cli.rs, crates/audit/src/lib.rs, crates/testkit/tests/config_precedence.rs, docs/04-tasks/active/TASK-001-foundation-gap-closure.md]

key-decisions:
  - "Reuse Config::load, OriginMap, open_read_only, verify_persisted_audit, and the Clap tree — no parallel config/diagnostic/storage service (D-01..D-04)"
  - "seed_synthetic_chain lives in application (not the CLI test) to respect the arch-check edge boundary; documented pub test-support, not operator surface"
  - "Serve-host and SearchApiService::open creating-constructors left untouched — later-phase domains, out of the foundation seven-group scope"
  - "Concurrent phase-3 EXCLUDE-file removal documented, not repaired — never fabricate another session's planning state"

patterns-established:
  - "Real-binary contract tests: every operator-visible behavior asserted through CARGO_BIN_EXE_qai in human and --json modes"
  - "Guard-before-open: existence/readiness check precedes any SQLite constructor on ordinary paths; creation confined to qai db migrate"
  - "One diagnostic renderer + centralized exit mapping for persisted audit failures (exit_code::VALIDATION)"

requirements-completed: [REQ-product-vision, REQ-product-principles, REQ-goals-non-goals, REQ-engineering-baseline, REQ-storage-architecture, REQ-cli-api, REQ-architecture-principles-quality]

# Coverage metadata (#1602) — one entry per shipped deliverable. Drives DETERMINISTIC UAT routing in verify-work.
coverage:
  - id: D1
    description: "Fresh-workspace operator tracer: help groups, config origins, read-only doctor, explicit migrate, empty-chain verify"
    requirement: "REQ-cli-api"
    verification:
      - kind: e2e
        ref: "crates/cli/tests/foundation.rs#fresh_workspace_operator_readiness + help_lists_stable_foundation_groups + doctor_on_fresh_workspace_is_read_only_and_names_migrate + explicit_migration_creates_schema_then_empty_audit_chain_verifies"
        status: pass
    human_judgment: false
  - id: D2
    description: "Effective configuration inspection: CLI>env>file>defaults precedence, dotted-key origins, redaction, keyed get, typed validate diagnostics"
    requirement: "REQ-product-principles"
    verification:
      - kind: integration
        ref: "crates/testkit/tests/config_precedence (10 tests)"
        status: pass
      - kind: e2e
        ref: "crates/cli/tests/foundation.rs#effective_config_reports_file_and_env_origin_without_leaking_secrets + config_get_returns_the_requested_value_with_its_origin + config_validate_*_reports_same_fields_in_both_modes"
        status: pass
    human_judgment: false
  - id: D3
    description: "Metadata-only doctor and existing-database guards: no-side-effect diagnosis, missing/pending/checksum/current readiness, guard on ordinary reads"
    requirement: "REQ-storage-architecture"
    verification:
      - kind: e2e
        ref: "crates/cli/tests/foundation.rs#ordinary_read_on_missing_database_names_migrate_without_creating_state + doctor_after_migration_consumes_the_persisted_audit_verifier"
        status: pass
      - kind: unit
        ref: "cargo test -p application --lib db (12 tests)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Persisted audit verification as single authority: tampered and gap chains fail identically in human and JSON with stable code/remedy/next/affected-sequences"
    requirement: "REQ-product-principles"
    verification:
      - kind: e2e
        ref: "crates/cli/tests/foundation.rs#audit_verify_tampered_human_and_json + audit_verify_gap_human_and_json"
        status: pass
    human_judgment: false

# Metrics
duration: 45min
completed: 2026-09-26
status: complete
---

# Phase 01 Plan 01: Foundation Gap-Closure Tracer Summary

**Real-binary `qai` foundation slice: seven-group help, origin-bearing redacted config, read-only doctor, explicit `qai db migrate`, and persisted-verifier audit — 15 acceptance cases green, arch/migrate gates green.**

## Performance

- **Duration:** 45min (continuation session; full plan spans the interrupted session + this run)
- **Started:** 2026-09-26T10:30:00Z (approx, continuation resume)
- **Completed:** 2026-09-26T14:00:00Z (approx)
- **Tasks:** 3 (01-01-01 tracer, 01-01-02 effective config, 01-01-03 doctor/readiness/audit)
- **Files modified:** 9 (3 created, 6+ modified incl. evidence record)

## Accomplishments

- Fresh-workspace operator path builds and runs end to end: `--help` shows exactly
  `config, db, doctor, job, audit, secret, source`; `config show --json` reports
  loaded file/env values with `ValueOrigin` per leaf and redacts secrets pre-emission (D-07, D-08).
- Doctor is metadata-only (`symlink_metadata`/permission inspection, no `create_dir_all`
  or probe files) and consumes `verify_persisted_audit` instead of row counts (D-06, D-12).
- Ordinary application reads go through the existing-database guard: missing DB returns
  typed `StorageError::MigrationRequired` → exit 3 with the `qai db migrate` remedy,
  never an implicit creation (D-05, T-01-MIG).
- `qai audit verify` and doctor share one typed diagnostic renderer and the centralized
  `exit_code::VALIDATION` mapping: `QAI-AUD-0003/0004/0005` carry stable code, non-empty
  remedy, exact `qai audit verify` next command, and affected sequences in human and JSON (D-12).
- `cargo test -p cli --test foundation` 15/15, `application --lib db` 12/12,
  `config --lib` 32/32, `testkit config_precedence` 10/10, `cli --test catalog` 2/2,
  `arch-check` OK, `migrate-check` OK (21 migrations, checksums stable).
- TASK-001 record carries per-task commands and D-01…D-12 evidence; closure remains
  owned by 01-05-02 per the task record.

## Task Commits

Each task was committed atomically (interrupted-session work verified, not redone):

1. **Task 01-01-01: Activate TASK-001 and prove the fresh-workspace operator flow**
   - `de1a505` (feat scripts: preservation verifier)
   - `641b0c2` (docs activation: TASK-001 record + README entry)
   - `0ac0432` (feat tracer: dispatch envelope, DbProbe verifier result, metadata-only doctor, foundation tests)
2. **Task 01-01-02: Complete effective configuration inspection**
   - `d8af909` (test config: origin totality), `9b953a8` (feat config: total OriginMap),
     `e9c5ae1` (feat cli: explain + redacted keyed lookups), `86c516e` (test cli: precedence/validation contracts),
     `72c5d74` (docs: 01-01-02 evidence)
3. **Task 01-01-03: Enforce metadata-only doctor and existing-database guards**
   - `63cc207`, `d842244` (audit recovery remedies + contract tests),
     `68b0967`, `193e8a7`, `444294f`, `973b62d`, `5f3a0f3`, `32f8b56`
     (typed invalid-chain diagnostics, shared renderer, audit-chain diagnosis, checksum check, readiness states, symlink_metadata probes)
   - `b0d11bc` (feat: open_db guard + approval_flow mapping, tampered/gap human+JSON tests, ordinary-read guard test, seed helper, TASK-001 evidence, fmt)

**Plan metadata:** this SUMMARY + STATE/ROADMAP updates (final docs commit, see completion message).

## Files Created/Modified

- `crates/cli/tests/foundation.rs` — created; 15 real-binary acceptance cases (help, config-origin/redaction, doctor no-side-effect, migrate, persisted-verifier doctor, tampered/gap human+JSON, ordinary-read guard, CLI-over-env adjacency)
- `scripts/verify-phase1-preservation.sh` — created; dynamic capture/check preservation verifier
- `docs/04-tasks/active/TASK-001-foundation-gap-closure.md` — created; active record with per-task evidence rows (01-01-03 row filled this run)
- `crates/cli/src/lib.rs` — loaded `(Config, OriginMap)` dispatch; config explain/get/validate envelopes; audit diagnostic renderer + VALIDATION exit mapping
- `crates/cli/src/doctor.rs` — metadata-only probes; `audit.chain_valid` from persisted report; checksum_current check
- `crates/config/src/lib.rs` — OriginMap total over every effective leaf
- `crates/application/src/db.rs` — `DatabaseReadiness` missing/unreadable/pending/checksum-mismatch/current states with exact remedies
- `crates/application/src/audit_bridge.rs` — `verify_persisted_audit` carried into DbProbe; `seed_synthetic_chain` test-support writer
- `crates/application/src/quran_cli.rs` — `open_db` existing-database guard + `open_error_output` mapping applied to all ordinary reads and `approval_flow`
- `crates/audit/src/lib.rs` — `QAI-AUD-0003/0004/0005` codes, stable remedies, `qai audit verify` next command
- `crates/testkit/tests/config_precedence.rs` — precedence/redaction/origin matrix

## Decisions Made

- Reuse-only implementation per D-01…D-04: `Config::load`, `OriginMap`, `open_read_only`,
  checksummed migration runner, `verify_persisted_audit`, existing Clap tree. No parallel service created.
- `seed_synthetic_chain` placed in `application` (pub, documented test-support) because the
  CLI integration-test target may not gain `audit`/`domain`/`storage-sqlite` edges (`arch-check`
  boundary). Not operator surface; writes only to caller-supplied paths via the production writer.
- Serve-host `open_reader` error path and `SearchApiService::open` creating-constructor left
  unchanged: serve is the long-lived host domain (01-04) and search is a later-phase domain,
  both outside the foundation seven-group scope. Deferred, not diverged.
- Concurrent phase-3 planning session removed its own untracked
  `03-DISCUSS-CHECKPOINT.json` mid-run; recorded as external interference (see Deviations),
  not repaired — fabricating another session's planning state would be worse than the flag.
- `cargo fmt -p application -p cli` applied to task-owned files (prior session committed
  unformatted hunks); diffs are whitespace-only reflows outside the task's semantic hunks.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `approval_flow` mapped a missing database to INTERNAL instead of the guard diagnostic**
- **Found during:** Task 01-01-03 (guard-completeness sweep of all `open_db` call sites)
- **Issue:** Every ordinary read used `open_error_output` (exit 3 + migrate remedy), but the
  mutation-path `approval_flow` still mapped `open_db` failure to `exit::INTERNAL` with no remedy.
- **Fix:** `.map_err(open_error_output)` — one-line change reusing the task's mapping.
- **Files modified:** `crates/application/src/quran_cli.rs`
- **Verification:** `cargo test -p cli --test foundation` 15/15 (incl. new ordinary-read guard test)
- **Committed in:** `b0d11bc` (part of task commit)

**2. [Rule 2 - Missing Critical] No acceptance test proved ordinary reads refuse implicit creation**
- **Found during:** Task 01-01-03 (acceptance-criteria audit: "ordinary application reads use the
  existing-database guard and return a typed migration-required diagnostic")
- **Issue:** The guard existed in working-tree code but no test exercised an ordinary command
  against a missing database.
- **Fix:** Added `ordinary_read_on_missing_database_names_migrate_without_creating_state`
  (`qai quran get 2:255` on a fresh dir → exit 3, names `qai db migrate`, no `qai.db` created).
- **Files modified:** `crates/cli/tests/foundation.rs`
- **Verification:** passes first run against the guard; full suite 15/15
- **Committed in:** `b0d11bc` (part of task commit)

### External interference (not a deviation — no plan change)

- `sh scripts/verify-phase1-preservation.sh check --task 01-01-03` exits 1 with a single error:
  `EXCLUDE path removed: .planning/phases/03-quran-search-linguistics/03-DISCUSS-CHECKPOINT.json`.
  The file was never git-tracked, was present-as-untracked at capture time, and was removed by the
  concurrent phase-3 planning session (its `03-DISCUSSION-LOG.md` now exists). This task touched only
  the six `crates/*` files plus the TASK-001 record; manual hunk audit confirms every pre-existing
  byte/hunk in touched files is preserved (non-additive diffs are rustfmt reflows or the task-owned
  guard conversion). No `stash`/`reset`/`checkout`/`clean` was used. The flag is recorded here for the
  01-05-03 final gate rather than repaired.

---

**Total deviations:** 2 auto-fixed (1 bug, 1 missing critical)
**Impact on plan:** Both fixes are on the task's own guard seam and required by the acceptance criteria. No scope creep; serve/search creating-paths explicitly deferred as out-of-scope.

## Issues Encountered

- Prior executor's uncommitted work (`quran_cli.rs` guard, `audit_bridge.rs` seed helper, both
  audit-verify tests) was preserved verbatim per the resume directive — verified via `git diff`
  before any edit, built upon, never reverted.
- Dispatch memo reported 01-01-02 commits `9b953a8/e9c5ae1` (confirmed) and claimed the two
  audit-verify tests were "MISSING — zero matches": they were present-but-uncommitted in the
  working tree and now pass. No rework needed beyond the guard-completeness sweep.
- `cargo fmt --check` failures in task-owned files from the interrupted session normalized with
  `cargo fmt -p application -p cli`; full test suite re-run green afterwards.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 01-02 (same-UoW sequencing consumes `verify_persisted_audit` and the `DatabaseReadiness` seam).
- Watch items for 01-05-02/01-05-03: the phase-3 EXCLUDE-file preservation flag above; serve/search
  creating-constructors deferred to their owning phases; `seed_synthetic_chain` pub test-support
  documented for the final scope audit.
- No blockers.

## Self-Check: PASSED

- `crates/cli/tests/foundation.rs`, `scripts/verify-phase1-preservation.sh`,
  `docs/04-tasks/active/TASK-001-foundation-gap-closure.md` all FOUND.
- Commits `641b0c2`, `0ac0432`, `9b953a8`, `e9c5ae1`, `b0d11bc` (plus intermediates listed above) all FOUND via `git show`.
- No stub patterns (`TODO|FIXME|placeholder|unimplemented`) in the plan diff.
- No new network/auth/schema trust-boundary surface outside the plan `<threat_model>` — no Threat Flags section.

---
*Phase: 01-foundations*
*Completed: 2026-09-26*
