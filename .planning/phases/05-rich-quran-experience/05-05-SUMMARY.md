---
phase: 05-rich-quran-experience
plan: 05
subsystem: ui
tags: [tui, cockpit-screens, testbackend, doctor, jobs, reader, search, graph, rag-debug, annotations]

# Dependency graph
requires:
  - phase: 05-rich-quran-experience plan 02
    provides: App shell, TuiServices, palette, dashboard, sanitizer this inventory builds on
provides:
  - Full cockpit screen inventory (ops/reader/search/graph/annotations/RAG-debug) with live 5s refresh
  - TuiServices.db_path carried from the qai tui arm
affects: [05-07]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
actuals:
  tokens: 8997
  tasks: 3
  commits: 1

# Tech tracking
tech-stack:
  added: [storage (tui)]
  patterns: [view-model screens with documented mapping rules, dispatch-based refresh over typed tools, honest-empty snapshots]

key-files:
  created: [crates/tui/src/screens/ops.rs, crates/tui/src/screens/reader.rs, crates/tui/src/screens/search.rs, crates/tui/src/screens/graph.rs, crates/tui/src/screens/annotation_review.rs, crates/tui/src/screens/rag_debug.rs, crates/tui/tests/screens.rs]
  modified: [crates/tui/src/screens/mod.rs, crates/tui/src/app.rs, crates/tui/Cargo.toml, crates/cli/src/lib.rs, xtask/allowlist.toml]

key-decisions:
  - "Screens take tui-local view models, not service types: keeps snapshots light, avoids four new crate deps, and documents the canonical-slot mapping rule at the single App call site"
  - "Live refresh reuses dispatch_registered_tool (typed tools over shared handles) instead of backend traits: no arg-struct imports, results navigated as JSON"
  - "Search has no stored query yet (query input is future work): screen renders an honest no-query hint, never a fabricated search"
  - "Structural projection id is the frozen quran-structural-v1 value for the annotation queue; edition id comes from live snapshot_meta"

patterns-established:
  - "Screen honesty: skipped/warn/truncated/unavailable render verbatim; failed refreshes keep the previous snapshot"
  - "Canonical-slot invariant by construction: reader strings originate only from AyahView.canonical at App::refresh_reader"

requirements-completed: [REQ-tui-cli, REQ-quran-research-tools]

# Coverage metadata (#1602)
coverage:
  - id: D1
    description: "Ops screens render doctor/jobs/index status with honest skipped/warn states"
    requirement: "REQ-tui-cli"
    verification:
      - kind: unit
        ref: "crates/tui/tests/screens.rs (render_doctor, render_jobs, render_index_status)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Reader/search screens render canonical Arabic and service-ordered sanitized hits"
    requirement: "REQ-tui-cli"
    verification:
      - kind: unit
        ref: "crates/tui/tests/screens.rs (render_reader, render_search)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Graph/annotation/RAG screens render verbatim truncation, pending queue, typed unavailable"
    requirement: "REQ-tui-cli"
    verification:
      - kind: unit
        ref: "crates/tui/tests/screens.rs (graph_nav, annotation_review, rag_debug)"
        status: pass
    human_judgment: false

# Metrics
duration: ~4h wall (incl. one server restart, six API lookups, one scope pivot)
completed: 2026-10-09
status: complete
---

# Phase 05 Plan 05 Summary

**Cockpit screen inventory ships: ops, reader, search, graph, annotations, and RAG-debug screens render live service state with honest empty/truncation/unavailable states.**

## Performance

- **Duration:** ~4h wall
- **Started:** 2026-10-09
- **Completed:** 2026-10-09
- **Tasks:** 3 completed (one shared commit — see deviations)
- **Files modified:** 12

## Accomplishments

- Six screen modules with view-model APIs: `render_doctor`/`render_jobs`/`render_index_status`,
  `render_reader`, `render_search`, `render_graph_nav`, `render_annotation_review`, `render_rag_debug`
- `App` refreshes every snapshot every 5s through shared services (doctor checks, jobs list, index snapshot,
  selected ayah via typed tool, graph neighborhood, pending-review queue); failures keep prior snapshots
- 8/8 `TestBackend` snapshots green: skipped-as-SKIP, drift-as-WARN, verbatim truncation, pending queue,
  typed RAG-unavailable, order-preserving sanitized search, control-char-safe reader
- `qai tui` arm carries `db_path` into `TuiServices`; only `storage` added as a new tui dep (dyn-Database
  coercion); arch-check/clippy/fmt clean; no subprocess/HTTP in `crates/tui`

## Task Commits

Tasks 1–3 share one commit (shared App scaffolding is inseparable — see deviations):

1. **Task 1: Ops screens** + **Task 2: Reader/search** + **Task 3: Graph/annotations/RAG** - `f386710` (feat)

**Plan metadata:** `HEAD` (docs: this file)

## Files Created/Modified

- `crates/tui/src/screens/ops.rs` - `JobRow`, `IndexStatus`, `render_doctor`/`render_jobs`/`render_index_status`
  (PASS/WARN/FAIL/SKIP labels; drift-as-WARN)
- `crates/tui/src/screens/reader.rs` - `render_reader` (canonical region + translation regions, all sanitized)
- `crates/tui/src/screens/search.rs` - `SearchRow`, `render_search` (service order, sanitized)
- `crates/tui/src/screens/graph.rs` - `GraphNavView`, `render_graph_nav` (verbatim truncation block)
- `crates/tui/src/screens/annotation_review.rs` - `AnnotationRow`, `render_annotation_review`
- `crates/tui/src/screens/rag_debug.rs` - `RagState::Unavailable`, `render_rag_debug` (no retrieval path)
- `crates/tui/src/screens/mod.rs` - module registry + re-exports (`Screen` enum unchanged)
- `crates/tui/src/app.rs` - `ReaderView`, snapshots, 5s `refresh()` (doctor/jobs/index/reader/graph/annotations),
  full screen dispatch, `TuiServices.db_path`
- `crates/tui/tests/screens.rs` - 8 snapshot tests incl. crafted control-char hits and truncation-verbatim
- `crates/tui/Cargo.toml` - `storage` dep; `xtask/allowlist.toml` - `[tui]` allow += `storage`
- `crates/cli/src/lib.rs` - `db_path` carried into `TuiServices`

## Decisions Made

- View-model screens over service-typed signatures (see key-decisions); canonical-slot invariant enforced at the
  single `refresh_reader` mapping site, documented on `ReaderView`.
- Dispatch-based refresh (`dispatch_registered_tool` + JSON navigation) over backend-trait calls: avoids
  importing `NormalizedArgs`/`SearchParams`/`ReadOptions` arg shapes and any new external dep.
- Search query input deferred (honest no-query hint); annotation queue uses the frozen structural projection id
  with the live edition id from `snapshot_meta`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing] Doctor/index services need storage types the plan never wires**
- **Found during:** Task 1 (`run_quran_checks` takes `&dyn storage::Database`, index loader takes
  `&SqliteDatabase`, jobs/annotation feeds take `db_path`)
- **Issue:** `TuiServices` carried no database access; honest ops screens are impossible without it.
- **Fix:** Added `db_path: String` to `TuiServices` (cli arm sets it) and the `storage` dep for the
  dyn-coercion; `load_snapshot` needs no named type so `storage-sqlite` stays out. Same-commit allowlist entry.

**2. [Process] Tasks 1–3 collapsed into one commit**
- **Found during:** Task 3 (all three tasks' screens share the App refresh/dispatch scaffolding and one
  `screens/mod.rs` registry; no per-task commit compiles alone)
- **Issue:** Strict per-task commits would require writing then reverting shared scaffolding twice.
- **Fix:** One cohesive commit; per-task VERIFY test names all run green independently
  (`cargo test -p tui render_doctor render_jobs render_index_status`, `render_reader render_search sanitize`,
  `render_graph_nav render_rag_debug`).

**3. [Rule 1 - Bug] `gen` is a reserved identifier in edition 2024**
- **Found during:** Task 1 (compile error in new code)
- **Issue:** Local `gen` binding rejected.
- **Fix:** Renamed to `generation`.

**4. [Rule 2 - Missing] `Assertion` fields differ from the plan's assumption**
- **Found during:** Task 3 (`assertion_kind`/`claim_json`/`decision: String` don't exist; decision is an enum)
- **Issue:** Plan-assumed field names are wrong.
- **Fix:** Map `id`/`kind`/`decision`/`claim[summary]` with `{:?}`-lowercased enums.

## Verification Results

- `cargo test -p tui` — 12/12 pass (4 shell + 8 screens)
- `cargo xtask arch-check` — OK, no forbidden edges
- `cargo clippy -p tui -p cli --all-targets` — clean; `cargo fmt --check` — clean
- `rg 'process::Command|reqwest|hyper' crates/tui/src` — no hits (D-12)
