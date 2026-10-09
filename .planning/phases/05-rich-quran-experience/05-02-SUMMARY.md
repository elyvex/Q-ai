---
phase: 05-rich-quran-experience
plan: 02
subsystem: ui
tags: [tui, ratatui, crossterm, arch-check, allowlist, cli, cockpit, palette, terminal]

# Dependency graph
requires:
  - phase: 05-rich-quran-experience plan 01
    provides: phase tracer + contract plumbing this vertical builds alongside (no code dependency)
provides:
  - `[tui]` arch-check allowlist entry plus workspace deps (ratatui, rust-embed, mime_guess) unblocking all SPA/TUI/parity work
  - Buildable `crates/tui` shell: event loop, panic-safe restore, palette, dashboard, terminal sanitizer
  - `qai tui` verb wired to the same service constructors as `qai serve`
affects: [05-03, 05-04, 05-05, 05-06, 05-07, 05-08, 05-09]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
actuals:
  tokens: 17942
  tasks: 3
  commits: 6

# Tech tracking
tech-stack:
  added: [ratatui 0.30.2, rust-embed 8, mime_guess 2, async-trait (tui)]
  patterns: [allowlist-with-dependency same-commit rule, TestBackend screen tests, panic-hook terminal restore, poll-thread event pump]

key-files:
  created: [crates/tui/src/app.rs, crates/tui/src/palette.rs, crates/tui/src/screens/mod.rs, crates/tui/src/screens/dashboard.rs, crates/tui/tests/render.rs]
  modified: [xtask/allowlist.toml, Cargo.toml, Cargo.lock, crates/tui/Cargo.toml, crates/tui/src/lib.rs, crates/cli/Cargo.toml, crates/cli/src/lib.rs]

key-decisions:
  - "TUI consumes crossterm only via ratatui::crossterm re-export (no direct crossterm edge); crossterm stays allowlisted so a future direct edge still gates explicitly"
  - "Event input runs on a dedicated poll/read thread over an mpsc channel instead of EventStream, so no futures/streaming dependency enters the tui surface"
  - "Render tests build TuiServices::for_tests() (scratch-file reader + unimplemented trait doubles) instead of growing a storage-sqlite test edge that arch-check would fail closed on"
  - "Palette filter returns borrowed matches; sanitizer keeps \\n/\\t and strips CSI/OSC/charset sequences (terminal analogue of escape_html)"

patterns-established:
  - "TUI data access: Arc<dyn application::…> handles held by TuiServices; screens never touch HTTP or subprocesses"
  - "Same-constructor rule: qai tui builds reader/search/lexicon/graph with the identical calls as the qai serve arm"

requirements-completed: [REQ-tui-cli]

# Coverage metadata (#1602)
coverage:
  - id: D1
    description: "Allowlist + workspace deps admit TUI/SPA/parity legs with arch-check green"
    requirement: "REQ-tui-cli"
    verification:
      - kind: unit
        ref: "cargo xtask arch-check"
        status: pass
    human_judgment: false
  - id: D2
    description: "TUI shell compiles with event loop, panic-safe restore, palette, dashboard, shared-service data access"
    requirement: "REQ-tui-cli"
    verification:
      - kind: unit
        ref: "crates/tui/tests/render.rs (4/4: dashboard buffer, palette filter, sanitizer, quit-on-q)"
        status: pass
    human_judgment: false
  - id: D3
    description: "qai tui exposed and wired to the same services as qai serve"
    requirement: "REQ-tui-cli"
    verification:
      - kind: unit
        ref: "cargo run -q -p cli --bin qai -- tui --help (exit 0, lists tui)"
        status: pass
      - kind: unit
        ref: "rg 'process::Command|reqwest|hyper' crates/tui (no hits)"
        status: pass
    human_judgment: false

# Metrics
duration: ~5h wall (incl. two server restarts, one stale-lock kill, one concurrent-session reconciliation)
completed: 2026-10-09
status: complete
---

# Phase 05 Plan 02 Summary

**TUI foundation ships: arch-check admits the tui surface, the ratatui shell renders a dashboard with panic-safe restore, and `qai tui` drives the shared services.**

## Performance

- **Duration:** ~5h wall (interrupted twice by server restarts; see deviations)
- **Started:** 2026-10-07
- **Completed:** 2026-10-09
- **Tasks:** 3 completed
- **Files modified:** 12

## Accomplishments

- Allowlist + workspace plumbing admits the whole phase surface (`[tui]`, extended `[cli]`/`[server.external]`,
  `ratatui`/`rust-embed`/`mime_guess`) with `arch-check` green
- `crates/tui` builds: `App` event loop over shared services, command palette with substring filter, dashboard
  screen verified through `TestBackend`, `sanitize_terminal_text` for untrusted dataset strings
- `qai tui --help` exits 0; the dispatch arm reuses the `Serve` arm's readiness gate and service constructors,
  reaching the cockpit through direct handles only (no HTTP, no shell-out)

## Task Commits

Each task was committed atomically:

1. **Task 1: Allowlist + workspace dependency plumbing** - `b72f211` (feat)
2. **Task 2: TUI application shell** - `f4b1cca` (deps) + `9794d2a` (test) + `a27fae8` (feat lib root/sanitizer,
   concurrent session) + `c7f97ba` (feat shell/palette/dashboard)
3. **Task 3: `qai tui` verb** - `5482c3f` (feat)

**Plan metadata:** `HEAD` (docs: this file)

## Files Created/Modified

- `xtask/allowlist.toml` - `[tui]` + `[tui.external.*]` blocks; `[cli]` workspace allow +=
  `tui`/`tool-registry`/`tools`/`quran-core`/`domain`; `[cli.external.registry]` +=
  `axum`/`tower`/`http-body-util`/`serde`; `[server.external.registry]` += `rust-embed`/`mime_guess`;
  `[tui.external.registry]` += `async-trait`
- `Cargo.toml` / `Cargo.lock` - workspace deps `ratatui 0.30.2`, `rust-embed 8`, `mime_guess 2`
- `crates/tui/Cargo.toml` - path deps on application/config/domain/observability/quran-core/tool-registry/tools;
  workspace ratatui/serde/serde_json/thiserror/tokio/async-trait; workspace lints
- `crates/tui/src/lib.rs` - Phase-5 crate root; `sanitize_terminal_text` (CSI/OSC/charset + C0/C1 stripping,
  keeps `\n`/`\t`/Arabic)
- `crates/tui/src/app.rs` - `TuiServices`, `App::new/run/on_key`, free `run()` (init→run→restore), panic hook,
  poll-thread event pump, `for_tests()` doubles
- `crates/tui/src/palette.rs` - `Command` + `Palette` inventory/filter (9 cockpit destinations)
- `crates/tui/src/screens/mod.rs` - `Screen` selector enum mirroring the palette
- `crates/tui/src/screens/dashboard.rs` - `render_dashboard` (title, screen list, `qai tui` palette hint)
- `crates/tui/tests/render.rs` - 4 tests: dashboard buffer, palette filter, sanitizer, quit-on-q
- `crates/cli/Cargo.toml` - `tui` path dep
- `crates/cli/src/lib.rs` - `Commands::Tui` variant + dispatch arm (readiness gate + shared constructors)

## Decisions Made

- Crossterm only via `ratatui::crossterm`; event input via a dedicated poll/read thread (no `EventStream`, no new
  streaming dep).
- `TuiServices::for_tests()` keeps render tests free of a `storage-sqlite` edge: scratch-file reader
  (schema-version fallback, never queried) plus `unimplemented!()` trait doubles.
- `[cli]` allow extension ships the full plan-listed superset now so 05-03's parity legs need no second allowlist
  commit.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] gsd-executor subagent dispatch failed 3x at infrastructure level**
- **Found during:** Wave 1 dispatch (before any plan work)
- **Issue:** `subagent` calls failed with `database is locked`, then `aborted` twice across two server restarts;
  no executor ever started on 05-02.
- **Fix:** Fell back to sequential inline execution per the workflow's own fallback rule (Agent genuinely
  unavailable), executing 05-02 directly as orchestrator.

**2. [Rule 1 - Bug] Concurrent session landed 05-02 foundation commits mid-execution**
- **Found during:** Task 2 (foreign `f4b1cca`/`9794d2a`/`a27fae8` on `main`, same author, 41h stale at discovery)
- **Issue:** Parallel implementation of the same plan (deps, render tests, lib root + sanitizer) with a slightly
  different API surface (`Command` vs `PaletteCommand`, `TuiServices::for_tests()`, free `run()`).
- **Fix:** Adopted the committed foundation verbatim (kept their sanitizer, their tests) and implemented the
  missing remainder (palette, screens, app shell, cli wiring) against their API instead of competing with it.

**3. [Rule 3 - Blocking] Partial 05-02 working-tree edits from the killed session**
- **Found during:** Restart recovery (uncommitted `Cargo.toml` + `allowlist.toml` edits, no SUMMARY, no commits)
- **Issue:** Safe-resume gate: production changes with no SUMMARY.
- **Fix:** The edits matched Task 1 exactly and `arch-check` was green, so they were kept and committed as
  `b72f211` rather than reverted; a stray `crates/tui` deletion was restored after proving it was committed
  scaffold, not partial work.

**4. [Rule 2 - Missing] Plan-named constructors lacked exact paths for the test seam**
- **Found during:** Task 2 (trait `SearchBackend` etc. live behind `#[async_trait]`; arg/return types live in
  sibling modules, not the `*_api` modules)
- **Issue:** Test doubles need `async-trait` + cross-module type imports the plan never names.
- **Fix:** Added `async-trait` to tui deps + allowlist (same commit as use) and imported
  `quran_search::{SearchOutput, SearchError}`, `quran_morphology::FamilyMemberView`,
  `quran_counting::FrequencyReport` in `app.rs`. Crate-level edge is still just `application`.

**5. [Rule 1 - Bug] Parallel render tests raced on one scratch DB path**
- **Found during:** Task 2 verification (dashboard test failed only in full-suite runs)
- **Issue:** All `for_tests()` calls shared one pid-keyed temp path; concurrent opens collided.
- **Fix:** Per-call atomic sequence suffix in the scratch filename.

## Verification Results

- `cargo test -p tui` — 4/4 pass (dashboard buffer, palette filter, sanitizer, quit-on-q)
- `cargo xtask arch-check` — OK, no forbidden edges
- `cargo run -q -p cli --bin qai -- tui --help` — exit 0, lists `tui`
- `rg 'process::Command|reqwest|hyper' crates/tui` — no hits (D-12)
- `cargo clippy -p cli -p tui --all-targets` — clean; `cargo fmt --check` — clean
