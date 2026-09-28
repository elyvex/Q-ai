---
phase: 01-foundations
plan: 05
subsystem: build-gate
tags: [rust, xtask, arch-check, dependency-policy, supply-chain, task-lifecycle, evidence-records]

# Dependency graph
requires:
  - phase: 01-04
    provides: serve-owned worker host, enqueue-only import, host-backed corpus flows, TASK-001 evidence rows 01-04-01…03
  - phase: 01-01
    provides: preservation companion verifier, active TASK-001 record, persisted-verifier audit authority
provides:
  - Explicit per-crate registry/git external dependency policy with inline mutation coverage (`xtask/src/arch.rs`, `xtask/allowlist.toml`)
  - Closed TASK-001 record (`docs/04-tasks/completed/`) with consistent active/completed indexes
  - Completed Phase 1 validation matrix + record-specific closure checker (`01-VALIDATION.md`, `scripts/verify-phase1-records.sh`)
  - Consistent rollup/status/changelog closure entries
affects: [phase-02+, verifier review, TASK-001 consumers]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
# Same estimateTokens scale (chars/4 over the realized diff), never a harness token count.
actuals:
  tokens: 22349    # chars/4 over the 5 plan-task commit diffs (89396 chars)
  tasks: 3         # tasks completed
  commits: 5       # plan-task commits (2 + 2 + 1); SUMMARY + tracking commits separate
plan_head_before: 27409d1

# Tech tracking
tech-stack:
  added: []
  patterns: [source-kind classification of cargo metadata edges, per-crate per-class external allow sets with fail-closed defaults, record-specific token assertions with no cross-record substitution, frontmatter-scoped flag checks]

key-files:
  created: [scripts/verify-phase1-records.sh]
  modified: [xtask/src/arch.rs, xtask/allowlist.toml, docs/04-tasks/completed/TASK-001-foundation-gap-closure.md, docs/04-tasks/active/README.md, docs/04-tasks/completed/README.md, docs/04-tasks/active/TASK-001-foundation-gap-closure.md, .planning/phases/01-foundations/01-VALIDATION.md, docs/06-progress/task-done-rollup.md, docs/06-progress/status.md, CHANGELOG.md]

key-decisions:
  - "External policy derived from live cargo metadata --all-features (23 crates), all git allows empty — no package added, installed, or upgraded (D-02, CONSTRAINT-01)"
  - "Violation messages carry crate/target/source-kind/source with sorted deterministic output; path message byte-identical (D-03)"
  - "Pre-move gate reds quarantined by attribution (foreign files quoted, untouched) instead of blocking Phase 1 closure or touching foreign work (D-01, D-02)"
  - "Active-index entry removed wholesale (2 lines) with diff proof rather than leaving a dangling fragment the literal grep-v proof would produce"
  - "Validation flags set true on matrix coverage with the foreign-red environment recorded alongside — not concealed, not a false pass (D-01, D-03)"

patterns-established:
  - "External-edge gate: classify path/registry/git/unknown at the metadata seam; unknown schemes fail closed; dedupe repeat (name, source) entries"
  - "Record-specific closure checks: require_tokens names exactly one file per assertion; newest-section scoping via header-anchored awk; flags scoped to YAML frontmatter"
  - "Shared-tree gate discipline: quote foreign reds with file:line attribution, re-prove owned scope, never format/fix another session's files"

requirements-completed: [REQ-product-vision, REQ-product-principles, REQ-goals-non-goals, REQ-engineering-baseline, REQ-storage-architecture, REQ-cli-api, REQ-architecture-principles-quality]

# Coverage metadata (#1602) — one entry per shipped deliverable. Drives DETERMINISTIC UAT routing in verify-work.
coverage:
  - id: D1
    description: "Registry/git external dependency enforcement: per-crate allow sets, fail-closed classification, inline mutation tests, green arch-check on live metadata"
    requirement: "REQ-architecture-principles-quality"
    verification:
      - kind: unit
        ref: "cargo test -p xtask (25 passed incl. 6 external-policy mutation cases)"
        status: pass
      - kind: integration
        ref: "cargo run -q -p xtask -- arch-check (OK on live workspace metadata)"
        status: pass
    human_judgment: false
  - id: D2
    description: "TASK-001 lifecycle closure: active-to-completed move with 0/1 index counts, Status Completed, dated evidence section"
    requirement: "REQ-product-vision"
    verification:
      - kind: other
        ref: "test -f completed path && test ! -e active path && active count 0 && completed count 1 && grep Status Completed && git diff --check"
        status: pass
    human_judgment: false
  - id: D3
    description: "Record-specific closure checker independently verifying validation/completed-task/rollup/status/changelog contracts plus lifecycle agreement"
    requirement: "REQ-engineering-baseline"
    verification:
      - kind: other
        ref: "sh -n scripts/verify-phase1-records.sh (clean); pre-move RED 23 FAILs/exit 1; post-move all record assertions pass"
        status: pass
    human_judgment: false
  - id: D4
    description: "Completed validation matrix, rollup, status, and changelog telling one evidence-backed closure story"
    requirement: "REQ-product-principles"
    verification:
      - kind: other
        ref: "record checker post-move: validation/completed-task/rollup-newest/status-closure/changelog-unreleased token groups all ok"
        status: pass
    human_judgment: false
  - id: D5
    description: "Whole-tree green final gate (fmt + full workspace tests incl. foreign-owned targets)"
    requirement: "REQ-engineering-baseline"
    verification: []
    human_judgment: true
    rationale: "Tree-wide fmt and quran_identity reds are owned by concurrent sessions (quoted in TASK-001 §Completion); only the owning sessions can clear them. Verifier must re-run cargo fmt --all -- --check && cargo test --workspace on a quiet tree."

# Metrics
duration: 150min
completed: 2026-09-28
status: complete
---

# Phase 01 Plan 05: External Policy Enforcement and TASK-001 Closure Summary

**`xtask arch-check` now rejects forbidden registry/git edges under an explicit per-crate policy derived from live metadata (25/25 tests), and TASK-001 is closed with a record-specific checker plus consistent validation/rollup/status/changelog evidence — with two foreign-owned gate reds quoted and untouched.**

## Performance

- **Duration:** ~150min (incl. full-workspace gate runs, background evidence refresh, and concurrent-session interference triage)
- **Started:** 2026-09-28T05:00:00Z (approx)
- **Completed:** 2026-09-28 (task-3 commit 5dddba8; SUMMARY after)
- **Tasks:** 3 (01-05-01 external policy, 01-05-02 task closure, 01-05-03 final records)
- **Files modified:** 10 (1 created, 9 modified incl. lifecycle move)

## Accomplishments

- `xtask::arch` classifies every dependency edge (`SourceKind::Path/Registry/
  Git/Unknown`); path edges check `workspace.allow`, registry/git check
  per-crate `external.*.allow`, unknown schemes fail closed; diagnostics are
  sorted deterministic with the path message byte-identical (D-02, D-03,
  T-05-ARCH/T-05-EXTERNAL/T-05-SUPPLYCHAIN).
- `xtask/allowlist.toml` gains a per-crate FND-07 block for all 23 crates with
  external deps, derived from `cargo metadata --all-features`; all git allows
  empty (no git-source dep exists); no package added/installed/upgraded;
  `arch-check` green on live metadata with the CI step unchanged.
- Inline mutation coverage (6 new cases): allowed/forbidden registry with exact
  message pins, allowed/forbidden git (registry-allow ≠ git-allow), unknown
  crate path+registry+git fail-closed, mixed path/external isolation (exactly
  1 violation), unknown-scheme fail-closed; path mutation + server OD-14 pin
  retained. `cargo test -p xtask` 25/25, fmt/clippy clean.
- `scripts/verify-phase1-records.sh` (POSIX, dependency-free): `require_tokens`
  checks each token in exactly one designated record (no cross-record
  substitution), newest-section scoping via header-anchored awk, completion
  flags scoped to validation frontmatter, lifecycle path/index counts, and
  preservation-companion delegation. Pre-move RED run: 23 FAILs/exit 1 as
  designed; post-move: every record assertion passes.
- Sole active→completed handoff done post-evidence: task file moved, active
  index count 0, completed index count 1, `Status: Completed`, dated
  §Completion section (plans 01-01…01-05, C1–C5→commands, gate outcomes,
  D-01…D-16, PROBE-01…10, 4 prohibitions, 27 threats, reuse ledger,
  non-expansion boundary, owner-gated items). Rollup/status/changelog updated
  append-only in 01-05-03; validation matrix complete with
  `nyquist_compliant`/`wave_0_complete` true and a recorded §Execution Results.
- TASK-001 evidence rows 01-05-01/02 filled; 01-05-03 outcomes live in
  validation §Execution Results and the three ledgers.

## Task Commits

Each task was committed atomically (both TDD tasks ran RED `test` → GREEN `feat`):

1. **Task 01-05-01: Enforce registry and git dependency policy** (tracer-adjacent, TDD)
   - `92ebdaa` (test: 6 external-policy mutation cases — RED: no `registry_allow`/`git_allow`)
   - `dee0c5c` (feat: SourceKind classification, per-crate policy, FND-07 allowlist block, TASK-001 row — GREEN: 25/25 + arch-check OK)
2. **Task 01-05-02: Close the cross-phase task record** (TDD)
   - `e68c88f` (test: records checker + pre-move RED 23 FAILs/exit 1)
   - `934af8e` (feat: active→completed move, 0/1 indexes, Status Completed, §Completion with gate evidence)
3. **Task 01-05-03: Final gate and validation/progress records**
   - `5dddba8` (docs: validation complete + §Execution Results, rollup/status/changelog entries)

**Plan metadata:** this SUMMARY + STATE/ROADMAP updates (final tracking commit).

## Files Created/Modified

- `xtask/src/arch.rs` — `SourceKind`, `ExternalRule`/`ExternalAllow`, `registry_allow`/`git_allow`, kind-aware `violations()` with sorted diagnostics, 6 new mutation tests
- `xtask/allowlist.toml` — FND-07 external block (23 crates, git empty) + semantics header
- `scripts/verify-phase1-records.sh` — created; record-specific POSIX closure checker with preservation delegation
- `docs/04-tasks/completed/TASK-001-foundation-gap-closure.md` — moved from active/; Status Completed; 01-05-01/02 rows; §Completion evidence section
- `docs/04-tasks/active/README.md` — TASK-001 entry removed (count 0)
- `docs/04-tasks/completed/README.md` — one dated TASK-001 entry (count 1)
- `.planning/phases/01-foundations/01-VALIDATION.md` — complete; 14 rows green; flags true; §Execution Results
- `docs/06-progress/task-done-rollup.md` — newest-first Phase 1 entry
- `docs/06-progress/status.md` — dated `## Phase 1 closure` section (additive)
- `CHANGELOG.md` — Unreleased brownfield entry with non-expansion boundary

## Decisions Made

- Policy derived from live metadata (not hand-written): guarantees current edges stay green with zero package churn; any new external dep must be named first (fail closed).
- Foreign reds quarantined by attribution, not repaired: formatting or fixing another session's files would violate scope and preservation rules; the closure records quote file:line evidence and point the verifier at a quiet-tree re-run.
- Index entry removed wholesale (both physical lines) with a diff proof: the plan's literal grep-v proof would leave a dangling sentence fragment; the adaptation keeps the index truthful while proving no unrelated byte changed.
- Flags set true on matrix coverage with the environment recorded alongside: `nyquist_compliant`/`wave_0_complete` assert the fourteen task references are covered (true), while §Execution Results discloses the two environmental reds — neither a concealment nor a false pass.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Violation message rendered a doubled kind word**
- **Found during:** Task 01-05-01 (first GREEN run: 2 exact-message assertions failed)
- **Issue:** Format `"{} ({} {})"` with kind `"registry"` produced `"domain -> tokio (registry registry+...) "`.
- **Fix:** Kind labels now carry the word (`"registry source"`, `"git source"`, `"unknown source"`).
- **Files modified:** `xtask/src/arch.rs`
- **Verification:** `cargo test -p xtask` 25/25
- **Committed in:** `dee0c5c` (part of task commit)

**2. [Rule 3 - Blocking] Pre-move gate red on foreign unformatted files**
- **Found during:** Task 01-05-02 (final-gate capture: `cargo fmt --all -- --check` exit 1)
- **Issue:** Diffs confined to concurrent-session untracked `crates/application/tests/alpha_smoke.rs` and `canonical_display_identity.rs`; fixing them would edit another session's work (out of scope, prohibited).
- **Fix:** Recorded the exact failing files in TASK-001 §Completion; re-proved owned scope (`cargo fmt -p xtask --check` clean, clippy workspace green, all Phase 1 suites green, arch/migrate green); proceeded with the move under explicit documentation.
- **Files modified:** none (evidence text only, in the moved record)
- **Verification:** scoped gate list above, each re-run live
- **Committed in:** `934af8e` (evidence in §Completion)

**3. [Rule 3 - Blocking] `cargo test --workspace` red on a foreign-owned target**
- **Found during:** Task 01-05-02 (`application --test quran_identity`, 2 `NotStaged` failures)
- **Issue:** Caused by a concurrent session's uncommitted +189-line `quran_cli.rs`/`quran.rs` rewrite (later committed with the failures persisting); Phase 1 touches nothing in `application/`.
- **Fix:** Same quarantine as (2): quoted in §Completion with counts; full Phase 1 quick suite re-run green (config 32/32, precedence 10/10, foundation 15/15, phase1_foundation 6/6, jobs 37/37, outbox 5/5, integrity 3/3, jobs-audit 5/5, recovery 13/13, host 3/3, quran_import 16/16, cli quran 14/14, xtask 25/25, cli jobs 4/4, catalog 2/2).
- **Files modified:** none
- **Verification:** scoped suite above, re-run live 2026-09-28
- **Committed in:** `934af8e` (evidence), `5dddba8` (validation §Execution Results)

**4. [Rule 3 - Blocking] Record checker flags passed vacuously pre-closure**
- **Found during:** Task 01-05-02 (RED run: `validation.flags` passed on a prose mention)
- **Issue:** The literal `nyquist_compliant: true`/`wave_0_complete: true` strings appear in the draft's sign-off bullet, so whole-file token checks passed before the flags were actually set.
- **Fix:** Flags are now asserted against the YAML frontmatter slice only (awk between the `---` markers).
- **Files modified:** `scripts/verify-phase1-records.sh`
- **Verification:** RED re-run shows `validation.flags` failing pre-closure; passes post-01-05-03
- **Committed in:** `e68c88f` (part of task commit)

**5. [Rule 2 - Missing Critical] Index proof adapted to keep the active index truthful**
- **Found during:** Task 01-05-02 (snapshot proof: the TASK-001 entry spans 2 physical lines)
- **Issue:** Removing only the token-bearing line (the plan's literal proof) leaves a dangling "`01-01…01-05); active until…`" fragment under Active tasks.
- **Fix:** Removed the whole entry and proved via `diff` that only those lines differ; completed side gains exactly one line. Acceptance intent (no unrelated entry touched) preserved and proven.
- **Files modified:** `docs/04-tasks/active/README.md`
- **Verification:** snapshot diffs + counts 0/1 in the commit record
- **Committed in:** `934af8e` (part of task commit)

---

**Total deviations:** 5 auto-fixed (1 bug, 3 blocking, 1 missing-critical/proof adaptation)
**Impact on plan:** All required for correctness or for honest closure on a shared tree. No scope creep: no package, broker, migration, or foreign-file edit; no deferred item absorbed.

## Issues Encountered

- Shared-tree churn throughout: 03-05 family/lexicon commits landed mid-plan; a second session held +189 uncommitted `quran_cli.rs` lines during the first workspace run; `.planning/STATE.md`, `ROADMAP.md`, `WINDOWS.md`, `state.json` were concurrently modified (later cleared). Nothing foreign was staged, edited, stashed, or cleaned; `git status` was checked before every commit.
- `cargo-deny` not installed locally: recorded as unavailable (CI-enforced), per the validation matrix's tool-limitation rule — not converted into a pass.
- The `xtask ci` wrapper cannot pass while tree-wide fmt is foreign-red; its runnable sub-steps (clippy, test scope, arch, migrate, adr-lint, schemas, doctor-schema path) are evidenced individually.
- The preservation companion (`check --all` / `--task`) cannot pass while another session's `.planning/milestone.lock` drifts from its 01-01-01 capture; the drift is foreign, the file untouched, and none of this plan's files are in the capture manifest (verified via manifest grep).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 1 is closed as a brownfield gap closure with D-01…D-16 evidenced; TASK-001 lives at `docs/04-tasks/completed/`.
- Verifier actions on a quiet tree: `cargo fmt --all -- --check`, `cargo test --workspace`, `cargo run -p xtask -- ci`, `sh scripts/verify-phase1-preservation.sh check --all`, `sh scripts/verify-phase1-records.sh` — the last must fully PASS (its only current FAIL is the foreign preservation delegation).
- Watch items: `quran_identity` 2 failures are owned by the concurrent Quran-CLI session (NotStaged on import path); `alpha_smoke.rs`/`canonical_display_identity.rs` need `cargo fmt` by their owner; `sha256:[..]` manifest lines (01-04 note) still await phase-2 rule-work settlement.
- Owner gates stay open: dataset license (ADR-0101), reviewer acceptance, remote deployment, P0-T56 container ritual, linguist goldens. No blocker for Phase 2 implementation work.

## Self-Check: PASSED

- `scripts/verify-phase1-records.sh`, `xtask/src/arch.rs`, `xtask/allowlist.toml`,
  `docs/04-tasks/completed/TASK-001-foundation-gap-closure.md`,
  `.planning/phases/01-foundations/01-VALIDATION.md` all FOUND.
- Commits `92ebdaa`, `dee0c5c`, `e68c88f`, `934af8e`, `5dddba8` all FOUND via `git show`.
- Record checker post-move: every per-record token group ok (only the foreign-blocked preservation delegation fails).
- No stub patterns (`TODO|FIXME|placeholder|unimplemented|todo!`) in the plan diff.
- No new network/auth/schema trust-boundary surface outside the plan `<threat_model>` (T-05-ARCH/EXTERNAL/SUPPLYCHAIN/STATUS/SCOPE/INFO all mitigated in-plan) — no Threat Flags section.

---
*Phase: 01-foundations*
*Completed: 2026-09-28*
