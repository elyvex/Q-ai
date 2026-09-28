---
phase: 04-quran-graph
plan: 03
subsystem: api
tags: [rust, sqlite, knowledge-graph, quran-graph, read-services, export, trycmd, explainability]

# Dependency graph
requires:
  - phase: 04-quran-graph
    provides: 04-01 tracer slice (SqliteGraphStore, structural builder, migration 0022) plus 04-02 annotation authority, review queue, word-root builders, and visible_export_sets
provides:
  - quran_graph_api read services (neighbors, reachability, shortest, up-to-K, subgraph, pattern, root-family with full explainability payload)
  - full CLI read tree (Path modes, Subgraph, Pattern, Export --db/--format, budget flags) with graph_s2 snapshots
  - quran_graph_export service (Graph JSON v1 assembly under policy filter, DOT/SVG static rendering)
  - graph_paths.rs (11 read-regression cases) and graph_export.rs (5 export-regression cases)
affects: [04-04-parity, 04-05-doctor-export, phase-05-ui]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
# Same estimateTokens scale (chars/4 over the realized diff), never a harness token count.
actuals:
  tokens: 37734
  tasks: 3
  commits: 3

# Tech tracking
tech-stack:
  added: []
  patterns: [async-backend-trait-over-sync-port, explanation-on-every-result, pre-serialization-allowlist-composition, plain-text-static-rendering]

key-files:
  created:
    - crates/application/src/quran_graph_api.rs
    - crates/application/src/quran_graph_export.rs
    - crates/application/tests/graph_paths.rs
    - crates/application/tests/graph_export.rs
    - crates/cli/tests/quran/graph_s2.trycmd
  modified:
    - crates/application/src/lib.rs
    - crates/application/src/quran_cli.rs
    - crates/application/src/quran_graph_store.rs
    - crates/cli/src/quran.rs
    - crates/cli/tests/quran.rs
    - crates/quran-graph/src/mem.rs

key-decisions:
  - "Read services are a concrete GraphApiService over SqliteGraphStore behind an async GraphBackend trait (quran_search_api shape): no new traversal algorithm; path modes reuse traverse::up_to_k_paths and the port ops, so budget/authz/cancel/ordering semantics hold by construction"
  - "No-path-requires-completeness lives in the result types (ReachabilityOutput.reachable is Option, ShortestOutput/PathsOutput expose no_path_proven): truncated searches report unknown, never absence"
  - "K beyond max_paths is a pre-flight BudgetExceeded at the service (T-04-07), and the CLI defaults --paths to the max-paths budget so the pin holds end-to-end"
  - "Subgraph and Pattern are db-only verbs (RootFamily precedent); Path and Export are file-or-db; every read verb threads QueryBudgets from flags with zero clamping"
  - "File-backed reads assemble the same Explanation contract via shared pub helpers (explain_edge_with, describe_authz, validate_k, rank_family_ayahs) so CLI/HTTP/tools cannot drift"
  - "Export assembles through visible_export_sets plus export_json_with_notice; restricted scopes pre-filter the authority sets before composition (T-04-10)"
  - "DOT/SVG are plain-text emission with zero new deps; SVG labels are stable IDs (refs-only) with an RTL document root for later Arabic-script labels; coordinates are presentational only"

patterns-established:
  - "Explanation-on-every-result: snapshot identity, per-edge structural-vs-assertion provenance, applied filters with authz descriptor (never hidden IDs), completion, duration ms"
  - "Pre-serialization allowlist composition: effective-plus-scoped assertions pre-filtered, then visible_export_sets, then export_json_with_notice"
  - "Budget flags with no clamping: None keeps defaults, explicit zeros fail pre-flight"

requirements-completed: [REQ-quran-graph]

# Coverage metadata (#1602) — one entry per shipped deliverable. Drives DETERMINISTIC UAT routing in verify-work.
coverage:
  - id: D1
    description: "Read services return explained results for all path modes, subgraph, patterns, and root-family over SQLite"
    requirement: "REQ-quran-graph"
    verification:
      - kind: integration
        ref: "crates/application/tests/graph_paths.rs (11 tests)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Full CLI read tree (neighbors, three path modes, subgraph, pattern, export, root-family) with budget flags and verbatim truncation"
    requirement: "REQ-quran-graph"
    verification:
      - kind: e2e
        ref: "crates/cli/tests/quran.rs#quran_graph_snapshots"
        status: pass
      - kind: e2e
        ref: "crates/cli/tests/quran/graph_s2.trycmd (8 segments)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Graph JSON v1 export carries co-traveling assertions under policy filter with DOT/SVG static rendering"
    requirement: "REQ-quran-graph"
    verification:
      - kind: integration
        ref: "crates/application/tests/graph_export.rs (5 tests)"
        status: pass
      - kind: e2e
        ref: "crates/cli/tests/quran.rs#quran_graph_snapshots (export/dot/svg assertions)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Typed CLI errors render stable codes with correct exits (unknown node 5, pre-flight 3, unavailable word-root 5 with QAI-MORPH-0004)"
    requirement: "REQ-quran-graph"
    verification:
      - kind: e2e
        ref: "crates/cli/tests/quran.rs#quran_graph_snapshots (exit-code assertions)"
        status: pass
    human_judgment: false

# Metrics
duration: 49min
completed: 2026-09-28
status: complete
---

# Phase 04 Plan 03: Reads Plus Export Summary

**Explained graph reads (all path modes, subgraph, patterns, root-family) plus policy-filtered Graph JSON export with DOT/SVG rendering, all reachable from a completed CLI read tree with snapshot coverage**

## Performance

- **Duration:** 49 min
- **Started:** 2026-09-28T21:24:20Z
- **Completed:** 2026-09-28T22:13:43Z
- **Tasks:** 3
- **Files modified:** 11 (5 created + 6 modified)

## Accomplishments

- Read service (`quran_graph_api.rs`) over the SQLite projection: neighbors, two-node reachability, shortest min-hop path, up-to-K ranked paths, bounded subgraph, typed patterns, and lexicon-gated root-family — every result carrying start/end nodes, ordered node IDs, per-edge structural-vs-assertion provenance, applied filters, snapshot identity, completion, and duration ms
- Full CLI read tree: Path gains file-or-db plus `--mode reachability|shortest|paths` with `--paths` K; new Subgraph (`--seed`) and Pattern (`--seed`, `--step EDGE[:Kind]`) verbs; Export gains `--db` plus `--format json|dot|svg`; every verb threads QueryBudgets from flags with zero clamping and prints the truncation marker with the verbatim reason
- Export service (`quran_graph_export.rs`): load pinned sets, assemble through `visible_export_sets` plus `export_json_with_notice`, DOT plus layered-by-hop SVG as plain-text emission with truncation banners
- `graph_s2.trycmd` (8 read segments) plus extended `quran_graph_snapshots` assertions: all path modes, subgraph, pattern, export JSON/DOT/SVG files, verbatim truncation, and exit codes (unknown node 5 with QAI-GRAPH-0004, pre-flight 3 with QAI-GRAPH-0002, unavailable word-root 5 with QAI-MORPH-0004, unknown mode/format 2)

## Task Commits

Each task was committed atomically:

1. **Task 1: Read services with full explainability payload** - `3a293df` (feat)
2. **Task 2: CLI read verbs and graph_s2 snapshots** - `f81d7b8` (feat)
3. **Task 3: Graph JSON export with policy filter and static rendering** - `70eaa30` (feat)

## Files Created/Modified

- `crates/application/src/quran_graph_api.rs` - GraphBackend trait, GraphApiService, typed args, Explanation payload, shared helpers (explain_edge_with, describe_authz, validate_k, rank_family_ayahs)
- `crates/application/src/quran_graph_export.rs` - load_projection_sets, assemble_export, export_document, hop_layers, render_dot, render_svg
- `crates/application/tests/graph_paths.rs` - 11 read cases (path modes, provenance, pattern rejection, seed skipping, attribution, completeness, K pre-flight, truncation precision)
- `crates/application/tests/graph_export.rs` - 5 export cases (travel-with-edges, byte-absence, truncation notice, one-step document, DOT/SVG smoke)
- `crates/cli/tests/quran/graph_s2.trycmd` - 8 read segments (neighbors, 3 path modes, subgraph, pattern, export, verbatim truncation)
- `crates/application/src/lib.rs` - Module wiring for the two new services
- `crates/application/src/quran_cli.rs` - Budget flags, mode selector, Subgraph/Pattern verbs, Export formats, coded error rendering, shared ranking
- `crates/application/src/quran_graph_store.rs` - `export_sets()` accessor over the pinned snapshot
- `crates/cli/src/quran.rs` - GraphBudgetFlags, extended Path/Export, new Subgraph/Pattern variants, dispatch
- `crates/cli/tests/quran.rs` - graph_s2 segment run plus JSON/exit-code/render assertions
- `crates/quran-graph/src/mem.rs` - `MemGraphStore::dump()` (deterministic full-set listing)

## Decisions Made

- Service is concrete over SqliteGraphStore behind an async GraphBackend trait (the quran_search_api shape, faked by 04-04 surfaces); no traversal algorithm was reimplemented — path modes reuse traverse helpers and port ops
- Completeness is a type property: `reachable: Option<bool>`, `no_path_proven()` true only for complete-empty; min_hops semantics pin the unknown-source (NodeNotFound) vs unknown-destination (proven absence) distinction
- Subgraph/Pattern are db-only verbs following the RootFamily precedent (the plan's "db selection" reading); Path/Export stay file-or-db with identical mode shapes on both
- File-backed reads assemble the same Explanation via shared pub helpers, so fixture/debug output and SQLite output share one contract
- RootFamily keeps direct `root_search` (no graph-build dependency) with ranking factored into the shared `rank_family_ayahs` helper, so CLI and service results cannot drift
- SVG labels are stable IDs only (refs-only invariant wins over literal "RTL labels"); the document root declares `direction="rtl"` for later Arabic-script labels, and coordinates are never asserted
- CLI graph errors now render through `render_human()` (code plus remedy plus next command); morphology errors carry `[QAI-MORPH-0004]` in the message

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] min_hops never errors on unknown destinations**
- **Found during:** Task 1 (first `graph_paths` run)
- **Issue:** The test expected NodeNotFound for an unknown destination, but min_hops BFS simply never arrives (proven absence); only unknown sources error via neighbors
- **Fix:** Test pins both behaviors: unknown destination is proven `Some(false)`, unknown source is NodeNotFound
- **Files modified:** crates/application/tests/graph_paths.rs
- **Verification:** `cargo test -p application --test graph_paths` (11 pass)
- **Committed in:** 3a293df (Task 1 commit)

**2. [Rule 1 - Bug] Fifth path via ayah:1:3 in the parallel-edges pin**
- **Found during:** Task 1 (parallel-assertions case found 5 paths, not 4)
- **Issue:** A 3-hop structural detour (ayah:1:1 → surah:1 → ayah:1:3 → ayah:1:2) also ranks; the exact-count assertion was over-specific
- **Fix:** Assert exactly 3 one-hop routes (NEXT plus both PARALLELS) plus the 2-hop containment route, with per-edge provenance on each
- **Files modified:** crates/application/tests/graph_paths.rs
- **Verification:** `cargo test -p application --test graph_paths` (11 pass)
- **Committed in:** 3a293df (Task 1 commit)

**3. [Rule 2 - Missing Critical] MemGraphStore needed a full-set listing for export**
- **Found during:** Task 2 (CLI export --db design)
- **Issue:** The port has no list-all operation, so a full-projection export was unimplementable without it
- **Fix:** Added `MemGraphStore::dump()` (deterministic order, deduped) plus `SqliteGraphStore::export_sets()`; traversal filtering stays in the port operations
- **Files modified:** crates/quran-graph/src/mem.rs, crates/application/src/quran_graph_store.rs
- **Verification:** graph_export byte-absence cases plus full CLI suite green
- **Committed in:** f81d7b8 (Task 2 commit)

**4. [Rule 2 - Missing Critical] CLI export format wiring lives in Task 3**
- **Found during:** Task 3 (DOT/SVG service needed its operator surface)
- **Issue:** The plan's Task 3 files omit the CLI, but "both written by the CLI export verb to files" requires `--format`/`--seed` on the Export verb plus file assertions in the CLI runner
- **Fix:** Extended the Export verb (`--format json|dot|svg`, `--seed`) and added DOT/SVG file assertions to `quran_graph_snapshots`; JSON output kept byte-identical to Task 2
- **Files modified:** crates/application/src/quran_cli.rs, crates/cli/src/quran.rs, crates/cli/tests/quran.rs
- **Verification:** `cargo test -p cli --test quran quran_graph_snapshots` green
- **Committed in:** 70eaa30 (Task 3 commit)

**5. [Rule 3 - Blocking] Root-family seed analyses shared one ayah**
- **Found during:** Task 1 (both seed analyses sat in ayah 1:1, so per-ayah dedup correctly returned one row)
- **Issue:** Test expectation of two ranked ayahs contradicted the seed
- **Fix:** Moved the second analysis to ayah 1:2 (plus the lemma rows the analysis FK requires)
- **Files modified:** crates/application/tests/graph_paths.rs
- **Verification:** `cargo test -p application --test graph_paths` (11 pass)
- **Committed in:** 3a293df (Task 1 commit)

---

**Total deviations:** 5 auto-fixed (3 bugs, 2 missing-critical)
**Impact on plan:** All auto-fixes preserve the plan's intent while respecting harder constraints (port shape, FK domains, BFS semantics). No scope creep; zero new dependencies.

## Issues Encountered

- `GraphApiService::open` requires the structural projection, so the root-family positive test publishes a structural build first even though root-family reads only the lexicon — test-only ordering, not a service limitation worth a second constructor
- `cargo fmt --check` reports drift in three untouched Phase-3 test files (alpha_smoke.rs, canonical_display_identity.rs, quran_identity.rs) — pre-existing, left alone per scope boundary (same drift 04-01 logged)
- A stray pre-existing `stash@{0}` ("WIP on main" from another session) was observed and left untouched

## Known Stubs

None - no placeholder values, TODO markers, or unwired surfaces introduced (byte-scan of the plan diff is clean). The `dispute` service operation still has no CLI verb (carried from 04-02); the file-backed export path ships zero assertions by construction (fixture/debug only).

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: dos-budget-preflight | crates/application/src/quran_graph_api.rs | T-04-07 pinned: K beyond max_paths is a pre-flight error; graph_paths exhaustion cases plus CLI `--paths 11` exit-3 pin |
| threat_flag: pattern-allowlist | crates/application/src/quran_cli.rs | T-04-08 pinned: `--step EDGE[:Kind]` parsing plus validate_pattern before I/O; PRECEDES/raw-text/malformed-kind exit-3 pins |
| threat_flag: truncation-honesty | crates/application/src/quran_cli.rs | T-04-09 pinned: marker plus verbatim reason in human, truncated plus incomplete_reason in JSON, Option/unknown types refuse no-path claims |
| threat_flag: export-policy-filter | crates/application/src/quran_graph_export.rs | T-04-10 pinned: effective-plus-scoped pre-filter composed with visible_export_sets; restricted/tombstoned byte-absence cases |

T-04-SC accepted (zero new dependencies — verified in the task commits).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 04-04 (parity): GraphBackend is the fake seam for HTTP/tool contract tests; CLI JSON shapes (mode/result/truncated/incomplete_reason/explanation) are the parity targets; export DOT/SVG files are the local-inspection baseline
- Ready for 04-05 (doctor/export): export_document plus hop_layers are the doctor-surface building blocks; pre-existing fmt drift and the foreign stash entry are logged above for visibility
- Watch items: path-family modes expand over EdgeFilter::any by traverse design (the requested filter is carried for neighbor-scoped reads and recorded effective in explanations); Export --seed defaults to the first node; SVG coordinates must never be asserted

---
*Phase: 04-quran-graph*
*Completed: 2026-09-28*

## Self-Check: PASSED

All 5 created files exist on disk; all 3 task commits resolve in git log
(`3a293df`, `f81d7b8`, `70eaa30`); plan verification set re-ran green at
close-out (graph_paths 11, graph_export 5, quran_graph_snapshots 1 with 8
graph_s2 segments, arch-check clean, clippy `-D warnings` clean on
application/cli/quran-graph, rustfmt clean on all plan files).
