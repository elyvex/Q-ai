---
phase: 04-quran-graph
plan: 04
subsystem: api
tags: [rust, axum, knowledge-graph, quran-graph, http, agent-tools, parity, explainability, openapi]

# Dependency graph
requires:
  - phase: 04-quran-graph
    provides: 04-03 read services (GraphBackend trait, Explanation payload), CLI read tree with JSON shapes, export services
provides:
  - five versioned graph HTTP read routes with ETag plus contract tests plus OpenAPI
  - five typed graph agent tools with attributed ToolResult envelopes plus registry growth 7 to 12
  - cross-surface parity suite (CLI vs HTTP vs tools) plus explainability contract suite
  - algorithm/confidence attribution on asserted provenance
affects: [04-05-doctor-export, phase-05-ui, phase-10-agent-runtime]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
# Same estimateTokens scale (chars/4 over the realized diff), never a harness token count.
actuals:
  tokens: 44154
  tasks: 3
  commits: 3

# Tech tracking
tech-stack:
  added: []
  patterns: [code-string status mapping so server never names the quran-graph enum, per-request projection open for serve wiring, primitive-surface vocabulary with application-side folding, delegating parity backend]

key-files:
  created:
    - crates/server/tests/graph.rs
    - crates/application/src/quran_graph_tools.rs
    - crates/application/tests/graph_tools.rs
    - crates/application/tests/graph_explain.rs
  modified:
    - crates/server/src/api.rs
    - docs/08-api/quran-v1-openapi.json
    - crates/tool-registry/src/lib.rs
    - crates/application/src/quran_graph_api.rs
    - crates/application/src/lib.rs
    - crates/application/src/quran_tools.rs
    - crates/server/tests/api.rs
    - crates/cli/src/lib.rs
    - crates/application/tests/graph_paths.rs

key-decisions:
  - "HTTP error statuses match on QAI-GRAPH/QAI-MORPH code strings (the search_error_status precedent) so server gains no quran-graph edge; arch-check stays green with zero allowlist changes"
  - "FileGraphBackend opens the active structural projection per request: rebuilds are picked up live and a missing build is a per-request 404, never a serve-time failure on fresh databases"
  - "Surfaces speak primitives (BudgetPatch, direction/mode/kind strings); application folds them into QueryBudgets/EdgeFilter/Pattern with pre-flight inside read_options, so boundary values behave identically on CLI, HTTP, and tools"
  - "Asserted provenance carries algorithm, algorithm_version, and confidence (additive, defaulted); Eq dropped from the four explanation types since f64 is not Eq and no code bounds on it"
  - "Parity compares canonicalized shared projections (duration stripped) rather than byte equality, because CLI adds quotations/manifest/stale and tools wrap in ToolResult by design"

patterns-established:
  - "Surface-boundary vocabulary: GraphApiError::rejected/unknown_node/access_denied constructors let surfaces without a quran-graph edge stay typed"
  - "Delegating parity backend: the explain suite fakes GraphBackend over the live service, proving trait fidelity while comparing serialized legs"
  - "Projection-pinned reproducibility: source_versions carries graph_projection plus graph_builder alongside the edition"

requirements-completed: [REQ-quran-graph]

# Coverage metadata (#1602) — one entry per shipped deliverable. Drives DETERMINISTIC UAT routing in verify-work.
coverage:
  - id: D1
    description: "Five graph HTTP read routes return enveloped results with ETag support, 422 pre-flight, 404 unknown, 200-plus-truncated partials, 404 unavailable dataset, and no mutation route"
    requirement: "REQ-quran-graph"
    verification:
      - kind: integration
        ref: "crates/server/tests/graph.rs (10 tests)"
        status: pass
    human_judgment: false
  - id: D2
    description: "OpenAPI document contains the five graph paths with Envelope plus Diagnostic schema refs in the same change as the routes"
    requirement: "REQ-quran-graph"
    verification:
      - kind: integration
        ref: "crates/server/tests/api.rs#openapi_spec_covers_every_route"
        status: pass
    human_judgment: false
  - id: D3
    description: "Twelve registered tool names with SemVer 1.0.0 graph version consts; every graph tool result carries the full ToolResult envelope with projection-pinned reproducibility; errors preserve QAI-GRAPH codes; suggestions labeled pending with algorithm attribution"
    requirement: "REQ-quran-graph"
    verification:
      - kind: integration
        ref: "crates/application/tests/graph_tools.rs (5 tests)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Cross-surface parity: canonicalized CLI, HTTP, and tool payloads agree on neighbors, all path modes, subgraph, and patterns; full explainability fields on every leg; unanimous truncated-plus-reason on the negative case"
    requirement: "REQ-quran-graph"
    verification:
      - kind: integration
        ref: "crates/application/tests/graph_explain.rs (4 tests)"
        status: pass
    human_judgment: false

# Metrics
duration: 2h 10m
completed: 2026-09-29
status: complete
---

# Phase 04 Plan 04: Surfaces Plus Parity Summary

**Versioned graph HTTP read routes, typed agent tools with projection-pinned reproducibility, and a proven three-surface parity plus explainability contract — the graph is identically drivable from CLI, API, and agents**

## Performance

- **Duration:** 2h 10m (approx, wall-clock with heavy machine load)
- **Started:** 2026-09-29T06:00:00Z (approx)
- **Completed:** 2026-09-29T08:10:00Z (approx)
- **Tasks:** 3
- **Files modified:** 13 (4 created + 9 modified)

## Accomplishments

- Five POST routes under `/api/v1/quran/graph` (neighbors, path with reachability/shortest/paths modes, subgraph, pattern, root-family) thin over the read services: enveloped results, ETag plus conditional-GET through the existing contract, 422 on pre-flight, 404 on unknown node/projection/dataset, 200 plus truncated reason on partials, 403 on denied scopes
- OpenAPI extended in the same change (five paths, Envelope 200 plus Diagnostic 403/404/422); the api.rs route-coverage test pins the new paths
- Registry grows 7 to 12 tools with per-tool SemVer 1.0.0 consts and input validation; `GraphToolBackend` returns service-identical payloads with pinned canonical references, graph/dataset analysis sources, deterministic projection-pinned checksums, and pending-suggestion warnings with algorithm attribution
- Parity suite proves canonicalized CLI/HTTP/tool agreement on neighbors, all path modes, subgraph, and patterns over one shared fixture (duration excluded), full D-14 explainability on every leg, and unanimous truncated-plus-reason reporting
- No mutation route and no mutation tool anywhere under the graph surface (D-11 fence pinned by route and registry tests)

## Task Commits

Each task was committed atomically:

1. **Task 1: HTTP graph read routes with OpenAPI** - `918c978` (feat)
2. **Task 2: Typed graph tools with attributed envelopes** - `d9afca4` (feat)
3. **Task 3: Cross-surface parity and explainability contract** - `b330014` (feat)

## Files Created/Modified

- `crates/server/src/api.rs` - Graph section: five handlers plus bodies, code-string error mapping, projection-pinning meta with ETag envelope, route registration, `graph` field on AppState
- `crates/server/tests/graph.rs` - 10 route-contract cases (envelope, ETag/304, modes, 422 boundaries, 404 unknown/dataset, truncated 200s, 403 distinguishability, no-mutation fence)
- `docs/08-api/quran-v1-openapi.json` - Five graph paths with Envelope plus Diagnostic refs (pure-addition diff)
- `crates/tool-registry/src/lib.rs` - Five version consts, five param structs, five backend trait methods with defaults, five registry methods, TOOL_NAMES 7 to 12, BackendMeta projection fields
- `crates/application/src/quran_graph_tools.rs` - GraphToolBackend over the read trait with attributed envelopes, pinned references, suggestion warnings
- `crates/application/src/lib.rs` - Module wiring for quran_graph_tools
- `crates/application/tests/graph_tools.rs` - 5 cases (names/versions, attributed service-identical envelopes, error codes, pending labeling)
- `crates/application/tests/graph_explain.rs` - 4 cases (neighbors/paths/subgraph-pattern parity with explainability asserts, truncated unanimity)
- `crates/application/src/quran_graph_api.rs` - Surface vocabulary (BudgetPatch, read_options, PathMode, pattern steps, snapshot meta, FileGraphBackend), GraphApiError diagnostics plus boundary constructors, algorithm/confidence on asserted provenance
- `crates/application/src/quran_tools.rs` - BackendMeta projection fields (empty: reader tools serve no projection)
- `crates/server/tests/api.rs` - DisabledGraph stub for the new AppState field, OpenAPI coverage-list extension, BackendMeta fields
- `crates/cli/src/lib.rs` - Serve wiring for FileGraphBackend (per-request open, never a serve-time failure)
- `crates/application/tests/graph_paths.rs` - Non-exhaustive Asserted match for the additive provenance fields (compat only)

## Decisions Made

- Code-string status mapping (the `search_error_status` precedent) instead of a server-to-quran-graph dependency: zero Cargo/allowlist changes, arch-check green throughout
- Per-request projection open (`FileGraphBackend`) instead of a pinned serve-time handle: always reads the current build and keeps `qai serve` working on fresh databases
- Primitive surface vocabulary with application-side folding (`read_options` runs budgets pre-flight): HTTP/tool boundary values validate identically to CLI without new workspace edges
- Additive explanation fields (`algorithm`, `algorithm_version`, `confidence` with serde defaults) plus dropping unused `Eq` (f64 is not `Eq`): old fixtures still parse, no snapshot churn
- Parity as canonicalized shared-projection comparison (duration stripped) rather than byte equality: CLI and tool wrappers add surface-specific keys by design

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Plan file list omitted required touch points**
- **Found during:** Task 1 (AppState construction would not compile)
- **Issue:** Adding the `graph` backend to AppState breaks `crates/cli/src/lib.rs` (serve wiring) and `crates/server/tests/api.rs` (test state); the surface helpers need an application home (`quran_graph_api.rs`) since server must not name quran-graph types
- **Fix:** Wired `FileGraphBackend::structural` into serve, added a rejecting `DisabledGraph` stub to api.rs tests, put `BudgetPatch`/`read_options`/`parse_path_mode`/`parse_pattern_steps`/`snapshot_meta` plus the `GraphApiError` Diagnostic impl in application
- **Files modified:** crates/cli/src/lib.rs, crates/server/tests/api.rs, crates/application/src/quran_graph_api.rs
- **Verification:** `cargo test -p server` (22 + 10 pass), arch-check clean with zero allowlist changes
- **Committed in:** 918c978 (Task 1 commit)

**2. [Rule 2 - Missing Critical] ETag needs an edition read context the trait did not expose**
- **Found during:** Task 1 (etag_for returns None without text_hash)
- **Issue:** Graph outputs carry snapshot identity but no edition text hash, so `ok_envelope` could never emit an ETag and the acceptance criterion (ETag support) was unimplementable
- **Fix:** Added `GraphBackend::snapshot_meta` (active-edition slug/version/id/hash/generation, degrading to empty on a fresh DB, never fabricated) implemented by both the service and the file backend
- **Files modified:** crates/application/src/quran_graph_api.rs, crates/server/src/api.rs, crates/server/tests/graph.rs
- **Verification:** ETag plus 304 round-trip pinned in graph.rs
- **Committed in:** 918c978 (Task 1 commit)

**3. [Rule 2 - Missing Critical] Confidence missing from the explainability payload**
- **Found during:** Task 3 prep (D-14 names confidence; the payload had none)
- **Issue:** `ProvenanceExplanation::Asserted` carried no confidence, so the plan's explainability assertion (confidence on every result) could not be written
- **Fix:** Added `confidence: Option<f64>` (plus `algorithm`/`algorithm_version` for T-04-12 warnings) from the authority record with serde defaults; dropped `Eq` from the four explanation types (f64 is not Eq; no code bounds on it)
- **Files modified:** crates/application/src/quran_graph_api.rs, crates/application/tests/graph_paths.rs (non-exhaustive match compat)
- **Verification:** graph_explain asserts confidence key plus the 0.5 suggestion value on every leg; graph_paths 11 pass unchanged
- **Committed in:** d9afca4/b330014 (fields in Task 2, compat in Task 3)

**4. [Rule 1 - Bug] Test-side budget and reference expectation errors**
- **Found during:** Tasks 2/3 (first graph_tools and graph_explain runs)
- **Issue:** Direct-service comparisons used `ReadOptions::default()` (6 hops) while tools/CLI apply parity defaults (neighbors 1, paths 4); a refs assertion wrongly required every neighbor ref to be a quran: URI though non-ayah nodes travel as stable IDs by design; reachability/pattern legs were asserted with edge expectations their designs exclude
- **Fix:** Tests fold identical explicit budgets on every leg and assert per-mode explainability (reachability: no edges by design; pattern NEXT-steps: structural only; shortest tie-break: no asserted guarantee)
- **Files modified:** crates/application/tests/graph_tools.rs, crates/application/tests/graph_explain.rs
- **Verification:** Production behavior confirmed correct in each case (applied-filters diffs showed the intended defaults); suites green after correction
- **Committed in:** d9afca4/b330014 (respective test files)

**5. [Rule 1 - Bug] tool-registry test helper placed inside the impl block**
- **Found during:** Task 2 verification (`cargo test -p tool-registry` failed to compile)
- **Issue:** `graph_attributed` defined inside `impl QuranBackend for FakeBackend` is rejected (E0407) and uncallable bare (E0425)
- **Fix:** Moved to a free function in the tests module next to `attributed`
- **Files modified:** crates/tool-registry/src/lib.rs
- **Verification:** `cargo test -p tool-registry` (6 pass)
- **Committed in:** d9afca4 (Task 2 commit)

---

**Total deviations:** 5 auto-fixed (2 bugs, 2 missing-critical, 1 blocking)
**Impact on plan:** All auto-fixes preserve the plan's intent while respecting harder constraints (arch-check edges, ETag contract, D-14 field list). No scope creep; zero new dependencies.

## Issues Encountered

- Machine load averaged 100-380 during execution (unrelated repo builds), pushing full-suite compiles to 17 minutes; verification ran serially with long timeouts, all green at close-out
- `cargo test` filter lesson: `cargo test -p server graph` matches test NAMES (hence `graph_*` naming), not the file — the suite file is `tests/graph.rs` but discovery is by name filter
- Two self-inflicted commit-grouping mistakes (amend sweeping up the next task's files) were caught by `git status` before proceeding and restructured into the three clean per-task commits above

## Known Stubs

None - no placeholder values, TODO markers, or unwired surfaces introduced. The `dispute` service operation still has no CLI verb (carried from 04-02); file-backed CLI reads still assemble explanations without authority rows (pre-existing, reported as UnknownAssertion gaps by design).

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: error-code-status-mapping | crates/server/src/api.rs | T-04-11 pinned: 422 pre-flight (0002/0003), 404 unknown (0001/0004/0007) plus unavailable dataset (QAI-MORPH-0004), 403 denied (0006), 500 build/storage — unknown vs truncated vs denied distinguishable, codes never leak hidden IDs |
| threat_flag: suggestion-labeling | crates/application/src/quran_graph_tools.rs | T-04-12 pinned: pending layer-D edges labeled in tool warnings with algorithm attribution, never in verified sets |
| threat_flag: etag-reuse | crates/server/src/api.rs | T-04-13 pinned: ETag plus 304 through the existing ok_envelope/etag_for contract; no new caching semantics |

T-04-SC accepted (zero new dependencies — verified: no Cargo.toml or allowlist changes in this plan).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 04-05 (doctor/export): HTTP plus tools surfaces are proven identical to the CLI, so doctor-repair and export surfaces can be verified the same three-surface way
- Ready for Phase 5 UI: OpenAPI graph paths plus the Envelope/ToolResult contracts are the machine-readable upstream the UI consumes; reproducibility blocks pin builds for result-contract checksums
- Watch items: `FileGraphBackend` opens per request (fine at research scale; pool or pin if profiling flags it); path modes expand direction-agnostic by traverse design (requested filters recorded, neighbor-scoped reads honor them); ETag degrades to absent on a fresh database (same as search responses)

---

*Phase: 04-quran-graph*
*Completed: 2026-09-29*

## Self-Check: PASSED

All 6 created/modified plan files exist on disk; all 3 task commits resolve in git log (`918c978`, `d9afca4`, `b330014`); no unintended deletions (`git diff --diff-filter=D` clean across the plan range); plan verification set re-ran green at close-out (server 4+22+10, tool-registry 6, graph_tools 5, graph_explain 4, graph_paths 11, arch-check clean, clippy `-D warnings` clean, rustfmt clean on all plan files).
