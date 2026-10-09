---
phase: 05-rich-quran-experience
plan: 07
subsystem: infra
tags: [ci-gate, source-scan, owner-gates, validation-map, roadmap, SC5-backstop]

# Dependency graph
requires:
  - phase: 05-rich-quran-experience plans 01–06, 08, 09
    provides: every deliverable this plan wires into CI, scans, and records
provides:
  - 11-step `cargo xtask ci` (SPA build + parity enforced pre-merge)
  - Committed layer-separation/prohibition source scan
  - BLOCKED owner gates with closing actions; filled validation map; roadmap plan list
affects: [phase-06-hadith-tafsir and later (CI shape, scan backstop)]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
actuals:
  tokens: 2200
  tasks: 3
  commits: 2

# Tech tracking
tech-stack:
  added: []
  patterns: [CI step insertion with renumber discipline, invariant-based source scans with self-checks, owner-gate re-recording without answering]

key-files:
  created: [crates/application/tests/layer_separation.rs]
  modified: [xtask/src/ci.rs, docs/05-followups/decisions-needed.md, .planning/phases/05-rich-quran-experience/05-VALIDATION.md, .planning/ROADMAP.md]

key-decisions:
  - "SPA build precedes the workspace test in CI (populated embed for all Rust tests); parity runs explicitly right after the workspace test"
  - "Scan predicates assert invariants (field-shaped props, token classes, markers), never brittle counts"
  - "Phase overview stays unchecked until verify-work; only the nine plan rows are marked"

patterns-established:
  - "Environmental skips (missing web/dist) follow the OTLP-gate precedent: notice + early return, never silent pass"
  - "Concurrent-session fmt drift is carried, not reverted, and noted"

requirements-completed: [REQ-quran-display, REQ-quran-research-tools, REQ-quran-result-contract, REQ-tui-cli]

# Coverage metadata (#1602)
coverage:
  - id: D1
    description: "cargo xtask ci runs SPA build + parity as numbered steps (11 total) and passes end-to-end"
    requirement: "REQ-quran-result-contract"
    verification:
      - kind: integration
        ref: "cargo xtask ci"
        status: pass
    human_judgment: false
  - id: D2
    description: "Layer-separation/prohibition source scan passes on the tree and catches violating fixtures"
    requirement: "REQ-quran-display"
    verification:
      - kind: unit
        ref: "cargo test -p application --test layer_separation (7/7)"
        status: pass
    human_judgment: false
  - id: D3
    description: "OD-04/OD-11/OD-12 recorded BLOCKED with closing actions; validation map filled; roadmap lists all nine plans"
    requirement: "REQ-tui-cli"
    verification:
      - kind: other
        ref: "plan Task 3 grep chain (OD + 9 plans + 11 steps + 8 map rows)"
        status: pass
    human_judgment: false

# Metrics
duration: ~2h wall
completed: 2026-10-09
status: complete
---

# Phase 05 Plan 07 Summary

**Phase gate closes: CI enforces SPA build + parity across 11 steps, a committed source scan backstops SC5 and the prohibitions, owner gates stay open and recorded.**

## Performance

- **Duration:** ~2h wall
- **Started:** 2026-10-09
- **Completed:** 2026-10-09
- **Tasks:** 3 completed
- **Files modified:** 5

## Accomplishments

- `xtask ci` renumbered 9→11: SPA build (`npm ci && npm run build`, fails loudly without Node) at step 3,
  explicit parity gate at step 6; dev-loop proxy documented in the step comment; full gate green
- `layer_separation.rs`: 7 scan tests (canonical-slot props, token classes, all-three token families,
  no-innerHTML, no-remote-URL/font, TUI purity) each with inline self-checks proving non-vacuity
- OD-04/OD-11/OD-12 re-recorded BLOCKED with exact closing actions + Phase-5 shipped defaults; none answered
- Validation map: all 8 rows name owning plans + concrete tasks, all green with ran-command evidence;
  Wave 0 all checked; `wave_0_complete: true`; `nyquist_compliant` stays false for verify-work
- Build-artifact policy recorded (`05-VALIDATION.md`); ROADMAP lists all nine plans complete (overview
  unchecked pending verification)

## Task Commits

1. **Task 1: CI wiring** + **Task 2: Source scan** - `35216e8` (feat, shared gate file + scan),
   plus `f98f028` (clippy let-chain fix) and `6736afc` (tree-wide `chore(fmt)` the gate required)
2. **Task 3: Gates/map/roadmap** - docs commit (this run)

**Plan metadata:** `HEAD` (docs: this file)

## Files Created/Modified

- `xtask/src/ci.rs` - SPA build (3/11) + parity (6/11) steps, renumbered labels, dev-loop comment
- `crates/application/tests/layer_separation.rs` - 7 invariant scans + predicate self-checks
- `docs/05-followups/decisions-needed.md` - Phase-5 BLOCKED records for OD-04/OD-11/OD-12
- `.planning/phases/05-rich-quran-experience/05-VALIDATION.md` - filled map, Wave 0, artifact policy
- `.planning/ROADMAP.md` - nine plans checked (scoped edits only)

## Decisions Made

- No new dependencies (T-05-SC).
- `nyquist_compliant` deliberately left false; approval stays pending — verify-work owns both.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Scan flagged the palette's own `Command::new` constructor**
- **Found during:** Task 2 (first scan run)
- **Issue:** The `Command::new` subprocess marker matched `PaletteCommand`'s constructor.
- **Fix:** Marker narrowed to path-qualified `process::Command` with a documented ceiling note.

**2. [Rule 1 - Bug] Convoluted `@font-face` block-walk indexing**
- **Found during:** Task 2 (self-review)
- **Issue:** Index arithmetic was fragile.
- **Fix:** Rewrote as a simple find-`}` slice loop.

**3. [Rule 2 - Missing] `///` vs `//!` doc-comment mix rejected by rustc (E0753)**
- **Found during:** Task 2 (first compile)
- **Issue:** Inner-doc marker used at file top.
- **Fix:** Uniform `//!` header.

**4. [Rule 3 - Blocking] `cargo xtask ci` failed at step 1 on pre-existing fmt drift**
- **Found during:** Task 1 verify (full gate run)
- **Issue:** `crates/tools`, `crates/server/src/api.rs:551`, and other files carried committed fmt drift
  predating this phase; the gate stops at step 1.
- **Fix:** `cargo fmt --all` (formatting-only, verified by diff review) committed as `chore(fmt)`; the gate
  then runs green. A stray `git add -A` momentarily staged the unrelated `crates/graphify-out/` noise —
  caught on commit review, unstaged, never committed. Pre-existing `api.rs:551` drift fixed as part of it.

## Verification Results

- `cargo xtask ci` — green end-to-end (11 steps; see gate output)
- `cargo test -p application --test layer_separation` — 7/7 pass
- `cargo test -p tools research_checksum` + `cargo test -p tool-registry` — green (map evidence)
- Plan Task 3 grep chain — pass
