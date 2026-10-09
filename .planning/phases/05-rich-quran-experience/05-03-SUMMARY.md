---
phase: 05-rich-quran-experience
plan: 03
subsystem: testing
tags: [parity, cross-surface, research-checksum, trycmd, D-16, SC3, tool-contract]

# Dependency graph
requires:
  - phase: 05-rich-quran-experience plan 01
    provides: dispatch_registered_tool table, quran tool verb, generic POST /api/v1/quran/tool/{name} route, research_checksum contract
  - phase: 05-rich-quran-experience plan 02
    provides: "[cli] workspace + external allow entries this plan's dev-deps reuse"
provides:
  - Repeatable 12-tool × 3-leg parity gate (service ⟷ HTTP ⟷ CLI, byte-identical payload + identical checksum)
  - Timing-free graph checksum (duration_ms scrubbed from checksum inputs)
  - CLI checksum snapshot (parity_s0/s1.trycmd + quran_parity_snapshots runner)
affects: [05-07, phase-10-agent-runtime]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
actuals:
  tokens: 7505
  tasks: 2
  commits: 2

# Tech tracking
tech-stack:
  added: []
  patterns: [in-process seeded fixture shared by all legs, canonical-JSON normalization excluding wall-clock fields, negative-control + table-coverage gate assertions]

key-files:
  created: [crates/cli/tests/parity_cross_surface.rs, crates/cli/tests/quran/parity_s0.trycmd, crates/cli/tests/quran/parity_s1.trycmd]
  modified: [crates/cli/Cargo.toml, Cargo.lock, crates/cli/tests/quran.rs, crates/application/src/quran_graph_tools.rs, xtask/allowlist.toml]

key-decisions:
  - "Seed the parity DB in-process with application functions (import/activate/forms/index/morphology/graph) instead of CLI-driven host flow: no serve process, no flake, and the CLI leg points --data-dir at the same directory"
  - "Graph research_checksum no longer covers presentational duration_ms (scrubbed in the shared envelope helper); displayed payloads untouched"
  - "Normalize-then-compare on canonical JSON bytes; checksums compared as hex strings, not recomputed"
  - "Snapshot pins deterministic checksum hexes literally and wildcards only edition_id + execution_time_ms"

patterns-established:
  - "Parity legs: dispatch_registered_tool (service) ⟷ router(state).oneshot POST /api/v1/quran/tool/{name} (HTTP) ⟷ CARGO_BIN_EXE_qai quran tool (CLI), all from one FIXTURES row"
  - "trycmd import-boundary split: parity_s0 (migrate+import enqueue) + parity_s1 (activate+tool), host syncs between segments"

requirements-completed: [REQ-quran-research-tools, REQ-quran-result-contract]

# Coverage metadata (#1602)
coverage:
  - id: D1
    description: "Service vs HTTP legs agree (normalized bytes + checksum) for all 12 tools"
    requirement: "REQ-quran-research-tools"
    verification:
      - kind: integration
        ref: "cargo test -p cli --test parity_cross_surface service_and_http_legs_match"
        status: pass
    human_judgment: false
  - id: D2
    description: "CLI leg agrees (normalized bytes + checksum) for all 12 tools"
    requirement: "REQ-quran-research-tools"
    verification:
      - kind: integration
        ref: "cargo test -p cli --test parity_cross_surface cli_leg_matches"
        status: pass
    human_judgment: false
  - id: D3
    description: "Checksum field pinned in CLI JSON snapshot output"
    requirement: "REQ-quran-result-contract"
    verification:
      - kind: integration
        ref: "cargo test -p cli --test quran quran_parity_snapshots"
        status: pass
    human_judgment: false
  - id: D4
    description: "Empty query is a typed error on every leg, never an empty result"
    requirement: "REQ-quran-result-contract"
    verification:
      - kind: integration
        ref: "cargo test -p cli --test parity_cross_surface empty_query_is_typed_error_on_every_leg"
        status: pass
    human_judgment: false

# Metrics
duration: ~6h wall (incl. two server restarts, one stray-process cleanup, manual snapshot capture)
completed: 2026-10-09
status: complete
---

# Phase 05 Plan 03 Summary

**SC3/D-16 proven repeatable: all 12 tools produce byte-identical payloads and identical checksums across service, HTTP, and CLI legs.**

## Performance

- **Duration:** ~6h wall (interrupted by server restarts; snapshot captured via a manual serve run)
- **Started:** 2026-10-09
- **Completed:** 2026-10-09
- **Tasks:** 2 completed
- **Files modified:** 8

## Accomplishments

- `parity_cross_surface.rs`: one seeded fixture drives all 12 `TOOL_NAMES` through the shared dispatcher,
  the generic typed-tool HTTP route, and the built `qai` binary — normalized bytes and `research_checksum`
  identical on every leg, plus table-coverage and negative-control assertions and an empty-query typed-error test
- Root-caused and fixed the 05-01 caveat: graph `research_checksum` covered presentational `duration_ms`,
  making cross-leg checksum equality impossible — now scrubbed from checksum inputs in the shared envelope helper
- `parity_s0`/`parity_s1.trycmd` + `quran_parity_snapshots` runner pin the checksum field in CLI JSON output
  (deterministic hexes literal, only `edition_id`/`execution_time_ms` wildcarded)

## Task Commits

Each task was committed atomically:

1. **Task 1: Parity harness — FIXTURES table, service vs HTTP legs** - `d8942d0` (feat)
2. **Task 2: CLI leg comparison + snapshot** - `310fd02` (feat)

**Plan metadata:** `HEAD` (docs: this file)

## Files Created/Modified

- `crates/cli/tests/parity_cross_surface.rs` - FIXTURES (12 rows), in-process seed (import→graph projection),
  `normalize`, three leg helpers, 5 tests
- `crates/cli/tests/quran/parity_s0.trycmd` - migrate + import-enqueue segment (new file; trycmd has no wait
  primitive, so the import boundary needs the split)
- `crates/cli/tests/quran/parity_s1.trycmd` - activate + `quran tool quran.get_ayah --json` checksum snapshot
- `crates/cli/tests/quran.rs` - `quran_parity_snapshots` runner (s0 → host sync → s1)
- `crates/cli/Cargo.toml` + `Cargo.lock` - dev-deps for the harness (application-adjacent path crates + axum/tower/serde)
- `crates/application/src/quran_graph_tools.rs` - `scrub_timing` + checksumming over the scrubbed clone
- `xtask/allowlist.toml` - `[cli]` workspace allow += `storage`/`storage-sqlite`/`quran-corpus`
- `crates/cli/src/quran.rs` - one-line foreign fmt fix (carried, not authored)

## Decisions Made

- In-process seeding over CLI-driven host flow (deterministic, fast, no serve process in the gate).
- Timing scrubbed from checksum inputs only; displayed `duration_ms`/`execution_time_ms` unchanged.
- `http-body-util` dropped from the dev-dep plan: `axum::body::to_bytes` already covers body collection, so no
  new external crate enters the tree.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Graph checksums could never match across legs (05-01 caveat realized)**
- **Found during:** Task 1 (first gate run; `duration_ms` inside checksummed results)
- **Issue:** The shared graph `envelope` checksummed `results` verbatim including presentational `duration_ms`,
  so identical calls produced different checksums per invocation — the plan's central must-have was unachievable.
- **Fix:** `scrub_timing` in `quran_graph_tools.rs` removes `duration_ms` keys from the checksum-input clone
  only. No pinned checksum values existed in tests (only non-emptiness/determinism assertions); non-graph tools
  never embedded timing in results (verified by scan), so the fix is graph-scoped.

**2. [Rule 2 - Missing] Plan-named `http-body-util` is not a workspace dep**
- **Found during:** Task 1 (manifest parse failure)
- **Issue:** Adding it would introduce a new external crate for one body-collection call.
- **Fix:** Dropped; `axum::body::to_bytes` (already a main dep) collects oneshot bodies.

**3. [Rule 2 - Missing] `ToolError` lives in `tools`, not `tool-registry`**
- **Found during:** Task 1 (E0603)
- **Issue:** Test matched `tool_registry::ToolError`.
- **Fix:** Added `tools` dev-dep (already `[cli]`-allowlisted) and matched `tools::ToolError`.

**4. [Rule 1 - Bug] Non-UUID run_id rejected by storage constraint**
- **Found during:** Task 1 (first gate run: `edition id 'parity-run' is not a UUID`)
- **Issue:** Import run_id doubles as an edition id with a UUID constraint.
- **Fix:** UUID run_id.

**5. [Rule 3 - Blocking] Snapshot needed exact `--json` output before the runner existed**
- **Found during:** Task 2
- **Issue:** Hand-writing 85 lines of JSON blind.
- **Fix:** Manual serve→import→activate→tool capture run, snapshot written from real output with deterministic
  hexes pinned literally; stray serve process killed afterwards. Extra `parity_s0` segment file beyond the
  plan's file list (import boundary requires it).

## Verification Results

- `cargo test -p cli --test parity_cross_surface` — 5/5 pass (12 tools × service/HTTP/CLI + coverage + negative
  control + empty-query edges)
- `cargo test -p cli --test quran quran_parity_snapshots` — pass (4/4 trycmd cases)
- `cargo xtask arch-check` — OK, no forbidden edges
