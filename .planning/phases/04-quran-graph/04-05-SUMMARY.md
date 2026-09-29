---
phase: 04-quran-graph
plan: 05
subsystem: database
tags: [rust, sqlite, knowledge-graph, quran-graph, doctor, repair, gc, audit, owner-gates, evidence-matrix]

# Dependency graph
requires:
  - phase: 04-quran-graph
    provides: 04-03 read services plus export, 04-04 HTTP/tools/parity (GraphBackend seam, Explanation contract, review lifecycle, fenced builders)
provides:
  - run_quran_graph_checks (six read-only quran.graph.* checks, rolled-back UnitOfWork, byte-identical proof)
  - doctor-repair verb group (quarantine-dangling, tombstone-gc, rebuild-projection; confirmed, audited, retention-gated)
  - quran graph doctor CLI verb rendering the read-only checks
  - graph_doctor.rs (10 cases: 6 checks + 4 repair)
  - phase-04-owner-gates.md (OD-11/OD-12/P4-X01/X02/X03/X05 BLOCKED with closers)
  - phase-04-deferrals.md (deferral ledger + D-01 SC1-SC4 evidence matrix)
affects: [phase-05-ui, phase-10-agent-runtime, phase-12-ops]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
# Same estimateTokens scale (chars/4 over the realized diff), never a harness token count.
actuals:
  tokens: 28160
  tasks: 3
  commits: 4

# Tech tracking
tech-stack:
  added: []
  patterns: [read-only-by-construction doctor over one rolled-back UnitOfWork, confirm-wrapper-owned CLI confirmation with service-side refusal, sql-side retention comparison via julianday, repair-audit on dedicated subject URN]

key-files:
  created:
    - crates/application/src/quran_graph_doctor.rs
    - crates/application/tests/graph_doctor.rs
    - docs/05-followups/phase-04-owner-gates.md
    - docs/05-followups/phase-04-deferrals.md
  modified:
    - crates/application/src/lib.rs
    - crates/application/src/quran_cli.rs
    - crates/cli/src/quran.rs
    - crates/application/src/quran_graph_annotations.rs

key-decisions:
  - "Retention compares inside SQLite (julianday age > retention_days): zero new dependencies, no host/store timestamp format skew, T-04-SC holds with no allowlist change"
  - "CLI confirm() wrapper owns confirmation (--yes/APPROVE); service functions still refuse confirmed=false by construction, so direct callers can never slip through (T-04-15 defense in depth)"
  - "Repair audit reuses the review chain writer under a dedicated subject URN (quran-graph-repair:<operation>) with the DoctorRepairExecuted action — identical chain arithmetic, distinguishable subject"
  - "Quarantine deletes only adjacency rows and records them verbatim in the audit event; GC deletes only rejected/superseded rows past retention plus their invisible edges; rebuild reuses the fenced publish path so authority is preserved by the same AC-P4-03 contract"
  - "Pre-existing cargo fmt drift in three Phase-3 test files left untouched per scope boundary (same drift 04-01/04-03 logged)"

patterns-established:
  - "Doctor-read vs repair-write split: checks open read-only and roll back; repairs open write pools, require confirmation, and emit audit — doctor output stays reproducible from the audit trail"
  - "Before-and-after counts in human summaries, full machine records in JSON, audit sequence in both"

requirements-completed: [REQ-quran-graph]

# Coverage metadata (#1602) — one entry per shipped deliverable. Drives DETERMINISTIC UAT routing in verify-work.
coverage:
  - id: D1
    description: "Read-only graph doctor: six stable quran.graph.* checks (manifest currency, dependency snapshot, dangling edges, tombstone invisibility) with severity-honest Pass/Warn/Fail/Skipped and byte-identical immutability"
    requirement: "REQ-quran-graph"
    verification:
      - kind: integration
        ref: "crates/application/tests/graph_doctor.rs#doctor_manifest_current_passes_on_fresh_build + 5 check cases"
        status: pass
    human_judgment: false
  - id: D2
    description: "Explicit confirmed repair: quarantine-dangling, retention-gated tombstone-gc refusing live rows, authority-preserving rebuild, one audit event per repair, post-repair doctor green"
    requirement: "REQ-quran-graph"
    verification:
      - kind: integration
        ref: "crates/application/tests/graph_doctor.rs#repair_* (4 cases)"
        status: pass
    human_judgment: false
  - id: D3
    description: "CLI doctor verb plus doctor-repair group with confirm-gated dispatch, before-and-after human counts, full-record JSON"
    requirement: "REQ-quran-graph"
    verification:
      - kind: e2e
        ref: "qai quran graph doctor --help + qai quran graph doctor-repair --help (clap wiring smoke)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Owner-gate ledger (OD-11/OD-12/P4-X01/X02/X03/X05 BLOCKED with exact closers) and deferral ledger with D-01 SC1-SC4 evidence matrix"
    requirement: "REQ-quran-graph"
    verification:
      - kind: other
        ref: "test -f docs/05-followups/phase-04-owner-gates.md && rg BLOCKED (13 hits, OD-11/OD-12/P4-X markers present)"
        status: pass
    human_judgment: false
  - id: D5
    description: "Full phase gate: quran-graph suite, all eight application graph suites, CLI snapshots, arch-check, migrate-check, clippy denied-warnings"
    requirement: "REQ-quran-graph"
    verification:
      - kind: integration
        ref: "cargo test -p quran-graph (49) + 8 suites (51) + quran_graph_snapshots (1) + arch-check + migrate-check"
        status: pass
    human_judgment: false

# Metrics
duration: 11h wall-clock incl. prior-attempt gap (active execution ~2h)
completed: 2026-09-29
status: complete
---

# Phase 04 Plan 05: Doctor Plus Closure Summary

**Read-only graph doctor with severity-honest checks, explicit confirmed repair with retention-gated tombstone GC and audit, BLOCKED owner-gate and deferral ledgers with the D-01 evidence matrix, and a green full phase gate — Phase 4 is operable and honestly closed**

## Performance

- **Duration:** ~11h wall-clock (08:35→19:32 +0330; includes a killed prior-attempt gap — Task 1's commit plus uncommitted Task-2 repair code were inherited; active execution ~2h, dominated by serial full-suite compiles)
- **Started:** 2026-09-29T05:05:54Z (first task commit)
- **Completed:** 2026-09-29T16:01:58Z
- **Tasks:** 3
- **Files modified:** 8 (4 created + 4 modified)

## Accomplishments

- Doctor verifies every projection read-only: per-projection manifest checks (generation stamp vs active corpus generation, dataset versions vs active morphology datasets, dependency-snapshot completeness), atomic dangling-edge scans, and tombstone-invisibility probes (rejected/superseded ids retained in authority, invisible in traversal and export bytes); fresh databases report Skipped with the build remedy, stale-but-servable drift is Warn, shallow bounds report a distinct Warn naming `--deep`
- Doctor runs inside one UnitOfWork that always rolls back, invokes no write method, and leaves database bytes identical (table-count + file-byte proof across shallow and deep runs)
- Repair is explicit and audited: `quarantine-dangling` removes dangling adjacency (authority untouched, removed rows recorded verbatim in the audit event), `tombstone-gc` collects only rejected/superseded rows older than the retention gate while refusing pending/accepted/disputed rows by SQL construction, `rebuild-projection` republishes through the fenced path preserving authority; every repair requires confirmation and emits one `doctor_repair_executed` audit event with before-and-after counts
- CLI surface: `quran graph doctor [--deep]` renders the six checks (Fail exits VALIDATION); `quran graph doctor-repair (quarantine-dangling | tombstone-gc [--retention-days 90] | rebuild-projection --projection … [--seed-file …])` wrapped in the established `confirm` (`--yes`/APPROVE) gate
- Owner-gate ledger records OD-11, OD-12, P4-X01, P4-X02, P4-X03, P4-X05 as BLOCKED with exact closing commands/steps; deferral ledger maps tafsir/hadith/isnad → Phases 6–7, explorer/GUI/TUI → Phase 5, CozoDB/sqlite-graph → optional spikes, GraphML → lossy-only, auth/hardening → Phases 11–12, with the D-01 evidence matrix (SC1–SC4 + lifecycle + doctor → files + checks, gaps honestly marked)
- Full gate green: quran-graph 49, eight application graph suites 51 (incl. graph_doctor 10), CLI snapshots 1, arch-check clean with zero allowlist changes, migrate-check 22 stable, clippy `-D warnings` clean, rustfmt clean on all plan files

## Task Commits

Each task was committed atomically:

1. **Task 1: Read-only graph doctor checks** - `2f0feb3` (feat)
2. **Task 2: Explicit repair and tombstone GC commands** - `48d2d7c` (feat)
3. **Task 3: Owner gates, evidence matrix, deferrals, full gate** - `3c2b18c` (docs)

**Plan metadata:** SUMMARY commit follows (docs: complete 04-05 plan).

## Files Created/Modified

- `crates/application/src/quran_graph_doctor.rs` - `run_quran_graph_checks` (6 stable ids, id-ordered output, shallow/deep caps) + confirmed `repair_quarantine_dangling` / `repair_tombstone_gc` / `repair_rebuild_projection` with `RepairReport` before-and-after counts
- `crates/application/tests/graph_doctor.rs` - 10 cases: manifest/drift/dangling/tombstone/empty/immutability + 4 `repair_*` (confirmation refusal, quarantine+green, retention refusal+green, rebuild+green)
- `crates/application/src/quran_cli.rs` - `cmd_graph_doctor` + three `cmd_graph_doctor_repair_*` (human counts + full-record JSON, `LOCAL_PRINCIPAL` operator identity)
- `crates/cli/src/quran.rs` - `GraphAction::Doctor` (read-only, no confirm) + `GraphAction::DoctorRepair` with `DoctorRepairAction` (confirm-wrapped)
- `crates/application/src/quran_graph_annotations.rs` - `append_repair_audit` (`pub(crate)` shared chain writer on the repair subject URN)
- `crates/application/src/lib.rs` - `quran_graph_doctor` module wiring (task-1 commit)
- `docs/05-followups/phase-04-owner-gates.md` - Six BLOCKED gates with closers, agent-uncloseable fence
- `docs/05-followups/phase-04-deferrals.md` - Deferral ledger + D-01 evidence matrix

## Decisions Made

- Retention compares inside SQLite (`julianday('now') - julianday(decided_at) > retention_days`): the review writes store RFC 3339 UTC, so host-side cutoff formatting would risk skew; SQL-side keeps one clock, zero new dependencies, no allowlist change (T-04-SC holds)
- Confirmation is owned by the CLI `confirm()` wrapper (`--yes`/terminal APPROVE, POLICY refusal on non-terminal without `--yes`) while the service still refuses `confirmed=false` — direct service callers can never slip through (T-04-15 defense in depth)
- Repair audit reuses the review chain writer (`append_audit` chain arithmetic unchanged) under a dedicated subject URN with the `DoctorRepairExecuted` action, so repair events are distinguishable and the pre-repair doctor output stays reproducible from the trail
- `ensure_operator` guards the missing-database path *before* the creating constructor, so a typo'd repair path errors instead of materializing an empty database file
- Pre-existing `cargo fmt --check` drift in three Phase-3 test files left untouched per scope boundary (same drift 04-01/04-03 logged; no plan file has drift)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Adopted and repaired the inherited uncommitted Task-2 repair draft**
- **Found during:** Task 2 start (working tree held 412 uncommitted lines of repair code + `time` dep from a killed prior attempt; Task 1 already committed as `2f0feb3`)
- **Issue:** The draft added `time` to `application` (not in `xtask/allowlist.toml`, violating the plan's T-04-SC zero-new-dependencies), left a dead `before` counter (clippy warnings), and `ensure_operator` could create a database file on a missing path
- **Fix:** Replaced the host-side cutoff with SQL `julianday` comparison + display-cutoff query, reverted `Cargo.toml`/`Cargo.lock`, wired `before` into the quarantine summary/records, added the missing-DB guard before the creating constructor
- **Files modified:** crates/application/src/quran_graph_doctor.rs, crates/application/Cargo.toml, Cargo.lock (reverted)
- **Verification:** `cargo test -p application --test graph_doctor repair` (4 pass), arch-check clean with zero allowlist changes
- **Committed in:** 48d2d7c (Task 2 commit)

**2. [Rule 1 - Bug] Missing `.await` on the test-only `backdate_decision` helper call**
- **Found during:** Task 2 (GC test collected 0 rows instead of 1)
- **Issue:** The async helper future was created and dropped unpolled (compiler `unused_must_use` warning) — bisected through four debug probes before the warning text surfaced the cause
- **Fix:** Added the missing `.await`; no production code involved
- **Files modified:** crates/application/tests/graph_doctor.rs
- **Verification:** `repair_tombstone_gc_collects_only_old_tombstones` green; full 10/10 suite green
- **Committed in:** 48d2d7c (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 missing-critical, 1 bug)
**Impact on plan:** Both preserve the plan's intent under harder constraints (arch-check allowlist, async correctness). No scope creep; zero new dependencies; no CLI snapshot additions beyond the plan's service-test proof (clap wiring smoke-tested via `--help`).

## Issues Encountered

- `cargo fmt --check` fails workspace-wide on three pre-existing Phase-3 test files (`alpha_smoke.rs`, `canonical_display_identity.rs`, `quran_identity.rs`) — untouched per scope boundary; all Phase-4/plan files are rustfmt-clean (verified per-file via `rustfmt --edition 2024` + the `Diff in` list)
- `echo ===` separators misbehave in this runner's shell tool (spurious `== not found`); cosmetic only, avoided thereafter
- Heavy machine load pushed single-crate checks to 2–3 min; verification ran serially with long timeouts, all green at close-out

## Known Stubs

None — byte-scan of the plan diff is clean (no TODO/FIXME/placeholder/unimplemented markers). The `dispute` service operation still has no CLI verb (carried from 04-02, out of this plan's scope); file-backed CLI reads still assemble explanations without authority rows (pre-existing, by design).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for Phase 5 UI: OpenAPI graph paths + Envelope/ToolResult contracts are the machine-readable upstream; `doctor`/`doctor-repair` + audit events are the operator runbook primitives; reproducibility blocks pin builds for result-contract checksums
- Ready for Phase 12 ops: `run_quran_graph_checks` is the service fn to surface; retention/GC semantics (rejected/superseded past N days, live rows unreachable) are the policy to schedule
- Watch items: `FileGraphBackend` opens per request (fine at research scale; pool or pin if profiling flags it); ETag degrades to absent on fresh databases; the six owner gates stay BLOCKED until humans close them — `/gsd-verify-work` consumes the evidence matrix above

---

*Phase: 04-quran-graph*
*Completed: 2026-09-29*

## Self-Check: PASSED

All 4 created files exist on disk; all 3 task commits resolve in git log
(`2f0feb3`, `48d2d7c`, `3c2b18c`); no unintended deletions (`git
diff --diff-filter=D` clean across the plan range); plan verification set
re-ran green at close-out (quran-graph 49, application graph suites 51,
quran_graph_snapshots 1, arch-check clean, migrate-check 22 stable,
clippy `-D warnings` clean, rustfmt clean on all plan files; workspace
`fmt --check` drift is pre-existing Phase-3 only).
