---
phase: 09-comparative-scripture
plan: 05
subsystem: server/tui/cli surfaces
tags: [scripture, http, tui, cli, parity]
requires:
  - phase: 09-comparative-scripture
    plan: 04
provides:
  - HTTP scripture routes (editions/get/search/translations/parallel/research)
  - TUI scripture panels (browse/parallel/cross-domain)
  - Cross-domain CLI command
  - Cross-surface parity (tool names 22)
affects: [server, tui, cli, tool-registry]
---

# Phase 9 Plan 05: HTTP + TUI + CLI surfaces Summary

## Accomplishments
- Added `crates/server/src/routes/scripture.rs` with edition/get/search/translations/parallel/research routes (incl. cross-domain aliases) and registered in lib.rs router.
- Added TUI scripture panels (browse, parallel passages with uncertain labels, cross-domain) wired into app navigation.
- Finalized TOOL_NAMES to 22 including 5 scripture.* tools; added `qai scripture research --domains` CLI command.
- arch-check passes with scripture allowlist entries.

## Files Created/Modified
- crates/server/src/routes/scripture.rs
- crates/server/src/lib.rs, api.rs
- crates/tui/src/scripture.rs, screens/scripture.rs, app.rs, lib.rs, screens/mod.rs
- crates/cli/src/scripture.rs, lib.rs
- crates/tool-registry/src/lib.rs
- xtask/allowlist.toml

## Decisions Made
- Cross-domain kept as application service + CLI/HTTP (not a typed tool trait) per plan.

## Issues Encountered
- None blocking; warnings only (unused imports).

## Next Step
Ready for 09-06 (owner gates, deferrals, validation map, phase gate).
---
*Phase: 09-comparative-scripture*
*Completed: 2026-10-19*
