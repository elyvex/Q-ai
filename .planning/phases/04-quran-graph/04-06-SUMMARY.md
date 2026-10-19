---
phase: 04-quran-graph
plan: 06
subsystem: api
tags: [quran-graph, projection, read-scope, cli, http, tool-registry, openapi, trycmd]

# Dependency graph
requires:
  - phase: 04-quran-graph
    provides: "GraphApiService::open_for_projection, publish_annotated_build, the annotated (quran-annotated-v1) build, and the neighbors read surfaces pinned to the structural family"
provides:
  - "ReadOptions.projection — the single projection selector field on the graph read service"
  - "FileGraphBackend::service(projection) — the one resolution point that maps the selector through GraphApiService::open_for_projection"
  - "`--projection` on `qai quran graph neighbors` and `graph export` (CLI), with `--projection` + `--file` as a typed USAGE error"
  - "`projection` on the HTTP neighbours request body and on the quran.graph_neighbors tool param (tool v1.1.0)"
  - "Concept/entity neighbor reachability coverage + corrected SC1 evidence row and a §7 deferral for the still-structural reads"
affects: [04-quran-graph, 05, verify-work, multi-rag]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
actuals:
  tokens: 11660
  tasks: 3
  commits: 4
# verify-work's commit-claim reconciliation greps `^commits:` at column 0 (#3968/#4670),
# so this flat mirror of actuals.commits is what that check reads. Keep the two in sync.
commits: 4
plan_head_before: f12509f3f2e61d558ced5f336168e20837992da5
plan_head_after: b069e14e60ec2552fc133da722c2544444d466be

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "One selector, one resolution point: every read surface funnels the projection through ReadOptions.projection → FileGraphBackend::service → open_for_projection"
    - "Typed unknown-family failure at open time (GraphError::UnknownProjection / QAI-GRAPH-0001 → HTTP 404 / CLI NOT_FOUND) instead of an empty success"

key-files:
  created: []
  modified:
    - crates/application/src/quran_graph_api.rs
    - crates/application/src/quran_cli.rs
    - crates/application/src/quran_graph_tools.rs
    - crates/cli/src/quran.rs
    - crates/server/src/api.rs
    - crates/tool-registry/src/lib.rs
    - crates/application/tests/graph_neighbors.rs
    - crates/application/tests/graph_explain.rs
    - crates/application/tests/graph_tools.rs
    - crates/server/tests/graph.rs
    - crates/tool-registry/tests/conformance.rs
    - crates/cli/tests/quran/graph_s2.trycmd
    - docs/08-api/quran-v1-openapi.json
    - docs/05-followups/phase-04-deferrals.md
    - .planning/phases/04-quran-graph/04-VALIDATION.md

key-decisions:
  - "ReadOptions.projection is the single selector field; resolved in exactly one place by FileGraphBackend::service(projection) via GraphApiService::open_for_projection — no per-surface resolution logic to drift."
  - "GRAPH_NEIGHBORS_TOOL_VERSION bumped 1.0.0 → 1.1.0 (additive optional input); path/subgraph/pattern/root-family versions unchanged because those verbs stay structural-only this cycle."
  - "Only `neighbors` and `export` gained the selector. `path`, `subgraph`, `pattern`, and `inspect` intentionally remain structural-only, recorded as a new explicit deferral (§7 of phase-04-deferrals.md) so the boundary is stated rather than silently omitted."
  - "An unknown or unbuilt projection family fails at open time (QAI-GRAPH-0001, CLI exit NOT_FOUND, HTTP 404), never as an empty neighbor set."

patterns-established:
  - "Selector threading: an optional string crosses CLI/HTTP/tool into one read-options field, so adding a future read verb means wiring one field, not a new resolution path."

requirements-completed: [REQ-quran-graph]

# Coverage metadata (#1602)
coverage:
  - id: D1
    description: "The graph read service carries a projection selector (ReadOptions.projection) resolved in one place through GraphApiService::open_for_projection, defaulting to the structural family."
    requirement: "REQ-quran-graph"
    verification:
      - kind: integration
        ref: "cargo test -p application --test graph_explain"
        status: pass
      - kind: integration
        ref: "cargo test -p application --test graph_neighbors"
        status: pass
    human_judgment: false
  - id: D2
    description: "`qai quran graph neighbors|export --db --projection <family>` selects the read family; `--projection` combined with `--file` is a typed USAGE error; selector-less reads are byte-identical to before."
    requirement: "REQ-quran-graph"
    verification:
      - kind: e2e
        ref: "cargo test -p cli --test quran quran_graph_snapshots (tests/quran/graph_s2.trycmd explicit-structural + QAI-GRAPH-0001 segments)"
        status: pass
      - kind: integration
        ref: "cargo test -p application --test graph_neighbors concept_neighborhood_is_reachable_only_through_the_annotated_projection (CLI leg)"
        status: pass
    human_judgment: false
  - id: D3
    description: "An HTTP POST to /api/v1/quran/graph/neighbors with `projection` reaches the backend as options.projection, and the OpenAPI neighbours request body documents the property."
    requirement: "REQ-quran-graph"
    verification:
      - kind: integration
        ref: "cargo test -p server --test graph graph_neighbors_route_threads_projection_selector"
        status: pass
      - kind: other
        ref: "python3 assertion that 'projection' exists in the neighbours request-body schema properties (SURFACES_OK)"
        status: pass
    human_judgment: false
  - id: D4
    description: "The quran.graph_neighbors tool accepts `projection` and forwards it through the shared read-options field; GRAPH_NEIGHBORS_TOOL_VERSION is 1.1.0 and only that param gained the field."
    requirement: "REQ-quran-graph"
    verification:
      - kind: unit
        ref: "cargo test -p tool-registry"
        status: pass
      - kind: integration
        ref: "cargo test -p application --test graph_tools"
        status: pass
    human_judgment: false
  - id: D5
    description: "The D-01 SC1 evidence row names the selector and the concept-neighborhood check and states which read operations remain structural-only; a VALIDATION row (04-06-T1) maps the truth to its tests."
    requirement: "REQ-quran-graph"
    verification:
      - kind: other
        ref: "docs/05-followups/phase-04-deferrals.md SC1 row + §7 deferral; .planning/phases/04-quran-graph/04-VALIDATION.md row 04-06-T1"
        status: pass
    human_judgment: true
    rationale: "Whether the ledger wording honestly represents the remaining structural-only surface is an editorial judgment about closure evidence, not something a test asserts."

# Metrics
duration: 36min
completed: 2026-10-16
status: complete
---

# Phase 4 Plan 06: Projection Selector for Graph Reads Summary

**A projection selector (`--projection` / body+tool `projection` field → `ReadOptions.projection` → one `open_for_projection` resolution point) makes annotated concept/entity neighborhoods reachable on CLI, HTTP, and the tool while the default structural read scope stays byte-identical.**

## Performance

- **Duration:** 36 min
- **Started:** 2026-10-16T11:03:44Z
- **Completed:** 2026-10-16T11:39:46Z
- **Tasks:** 3 / 3
- **Files modified:** 15

## Accomplishments

- `ReadOptions.projection` added and resolved in exactly one place: `FileGraphBackend::service(projection)` calls `GraphApiService::open_for_projection(db_path, projection.unwrap_or(STRUCTURAL_PROJECTION_ID))`; every `GraphBackend` impl that goes through `service()` passes `args.options.projection.as_deref()`. `root_family` (lexicon-gated) and `snapshot_meta` (build metadata, not a projection) stay projection-free by design, and `GraphApiService::open` is unchanged as the structural shorthand.
- CLI surfaces: a shared `graph_projection(raw)` helper (mirroring `review_projection`) defaults to `STRUCTURAL_PROJECTION_ID`; `neighbors_graph_db`/`export_db` open via `open_for_projection`; `--projection` added to `Neighbors` and `Export`, and `--projection` + `--file` returns a typed `exit::USAGE` with `--projection applies only to --db` before dispatch.
- HTTP + tool surfaces: `GraphNeighborsBody.projection` (`#[serde(default)]`) is set onto `options` in `graph_neighbors_handler` — the existing `graph_error_status` mapping already turns `QAI-GRAPH-0001` into 404, so an unknown family needed no new error plumbing. `GraphNeighborsParams.projection` added, `GRAPH_NEIGHBORS_TOOL_VERSION` → `1.1.0`, and `backend_graph_neighbors` forwards it. OpenAPI documents the property.
- Concept-neighborhood reachability proven end to end by `concept_neighborhood_is_reachable_only_through_the_annotated_projection`: the structural default read of `concept:mercy` is a typed `NodeNotFound`; the annotated read is non-empty with `projection_id == quran-annotated-v1`, seed attribution assertion ids on every edge, and `pending-scholar` attribution on the concept/entity nodes; the file-backend, tool, and CLI surfaces all match after canonicalization (excluding `duration_ms`).
- CLI snapshots extended with two segments: the explicit `--projection quran-structural-v1` output byte-identical to the selector-less baseline, and `quran-not-built-v1` asserting the typed `QAI-GRAPH-0001` NOT_FOUND error (exit 5).
- Closure evidence corrected: the SC1 ledger row now names the selector, the check commands, and the deliberately structural-only reads; a new §7 deferral records `path`/`subgraph`/`pattern`/`inspect` as an explicit boundary; `04-06-T1` added to the validation table with `T-04-17`/`T-04-18` refs.

## Task Commits

Each task was committed atomically:

1. **Task 1: Projection selector on the read service and CLI neighbor/export verbs** — `b4ad0e2` (feat)
2. **Task 2: Projection selector on the HTTP neighbors route and the quran.graph_neighbors tool, plus OpenAPI** — `cc6702b` (feat)
3. **Task 3: Concept-neighborhood integration coverage, CLI snapshot, and accurate closure evidence** — `d4bf736` (test)
4. **Formatting follow-up on the Task 2/3 test files** — `b069e14` (style)

**Plan metadata:** `<filled by the final docs commit>` (docs: complete plan)

## Files Created/Modified

- `crates/application/src/quran_graph_api.rs` — `ReadOptions.projection`; `FileGraphBackend::service(projection)` resolving through `open_for_projection`.
- `crates/application/src/quran_cli.rs` — `graph_projection` helper; `neighbors_graph_db`/`export_db` projection params; `cmd_graph_neighbors`/`cmd_graph_export` trailing param + the `--file` USAGE guard.
- `crates/application/src/quran_graph_tools.rs` — `backend_graph_neighbors` forwards `params.projection`.
- `crates/cli/src/quran.rs` — `--projection` on `Neighbors`/`Export` and their dispatch arms.
- `crates/server/src/api.rs` — `GraphNeighborsBody.projection` → `options.projection`.
- `crates/tool-registry/src/lib.rs` — `GraphNeighborsParams.projection`; version 1.1.0.
- `crates/application/tests/graph_neighbors.rs` — the concept-neighborhood reachability test and its `canonical`/`annotated_options` helpers.
- `crates/application/tests/graph_explain.rs`, `crates/application/tests/graph_tools.rs`, `crates/tool-registry/tests/conformance.rs` — literal `GraphNeighborsParams`/`cmd_graph_neighbors` sites updated with the new trailing `None`.
- `crates/server/tests/graph.rs` — `FakeGraph::neighbors` annotated arm, `annotated_neighbors_json()` helper, and `graph_neighbors_route_threads_projection_selector`.
- `crates/cli/tests/quran/graph_s2.trycmd` — explicit-structural-selector and unknown-family snapshot segments.
- `docs/08-api/quran-v1-openapi.json` — `projection` property on the neighbours request body.
- `docs/05-followups/phase-04-deferrals.md` — corrected SC1 row + §7 structural-only deferral.
- `.planning/phases/04-quran-graph/04-VALIDATION.md` — `04-06-T1` row.

## Decisions Made

- **One resolution point.** The selector lives on `ReadOptions` and is consumed only by `FileGraphBackend::service`, so CLI, HTTP, and tool surfaces cannot drift in how a family string is resolved; `GraphApiService::open` stays the structural shorthand rather than gaining a parallel code path.
- **Additive tool version.** `GRAPH_NEIGHBORS_TOOL_VERSION` 1.0.0 → 1.1.0 because `projection` is an optional input; the other graph verb versions are untouched since they gained no field.
- **Selector scope held to SC1's need.** Only `neighbors` and `export` accept a family; `path`/`subgraph`/`pattern`/`inspect` remain structural-only and are recorded in §7 of the deferral ledger as an explicit boundary (a general multi-projection read surface would carry its own parity/visibility contract and is a feature, not this gap).
- **Typed failure, never empty success.** Unknown/unbuilt families fail at open time via `open_for_projection`'s active-build lookup (`QAI-GRAPH-0001` → HTTP 404 / CLI NOT_FOUND), asserted by a CLI snapshot segment rather than left to the default path.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] OpenAPI property inserted without re-serializing the whole document**
- **Found during:** Task 2
- **Issue:** Adding the `projection` property by round-tripping the OpenAPI JSON through a default `json.load`/`json.dump` escaped an existing em-dash elsewhere in the file, producing a large unrelated diff.
- **Fix:** Wrote the property via a targeted Python edit using `ensure_ascii=False`, keeping the diff to the four intended lines.
- **Files modified:** `docs/08-api/quran-v1-openapi.json`
- **Verification:** `git diff` shows only the `projection` property addition; the OpenAPI schema assertion passes.
- **Committed in:** `cc6702b` (Task 2 commit)

**2. [Rule 1 - Bug] trycmd non-zero-exit assertion placed correctly**
- **Found during:** Task 3
- **Issue:** The unknown-projection segment initially asserted `? 5` at the end of the output block, which trycmd ignores; the segment passed without actually asserting the exit code.
- **Fix:** Moved the `? 5` marker to the line immediately after the `$ cmd` line, which is where trycmd reads the expected status.
- **Files modified:** `crates/cli/tests/quran/graph_s2.trycmd`
- **Verification:** `cargo test -p cli --test quran quran_graph_snapshots` — all 12 segments green, including the `QAI-GRAPH-0001` segment.
- **Committed in:** `d4bf736` (Task 3 commit)

**3. [Rule 1 - Bug] `cargo fmt` reformat after the Task 2/3 commits**
- **Found during:** Post-task verification
- **Issue:** `cargo fmt --check` flagged the newly added test call sites and assertions in `graph_explain.rs` / `server/tests/graph.rs`.
- **Fix:** Applied `cargo fmt` and committed the reformat separately so the task commits stay attributable.
- **Files modified:** `crates/application/tests/graph_explain.rs`, `crates/server/tests/graph.rs`
- **Verification:** `cargo fmt --check` exits 0.
- **Committed in:** `b069e14` (style follow-up)

---

**Total deviations:** 3 auto-fixed (2 bugs, 1 blocking)
**Impact on plan:** No scope creep — each fix was required for the plan's own verification to be trustworthy (a snapshot that asserted nothing, a formatter gate, a clean diff). No new dependencies.

## Issues Encountered

- **`cargo clippy -- -D warnings` cannot pass in this workspace, for reasons outside this plan.** `cargo clippy -p application -p server -p tool-registry -p cli --all-targets -- -D warnings` fails in `crates/domain` (`crates/domain/src/ids.rs:58` `clippy::crate_in_macro_def`) before the plan's crates are even reached; the working tree also carries pre-existing lints in `crates/application/src/verse_link.rs` (`:217` `too_many_arguments`, `:373` `collapsible_if`) from unrelated in-progress phase-06 work. None of these files is touched by this plan. The clippy line of the plan's `<verification>` block is recorded as **not met — pre-existing and out of scope**; every other verification line is green. See "Deferred Issues" below.

## Deferred Issues

| Issue | Location | Status |
|---|---|---|
| `clippy::crate_in_macro_def` | `crates/domain/src/ids.rs:58` | Pre-existing; blocks workspace-wide `-D warnings`. Not fixed (out of scope). |
| `clippy::too_many_arguments`, `clippy::collapsible_if` | `crates/application/src/verse_link.rs:217`, `:373` | Pre-existing phase-06 WIP in an unrelated file. Not fixed (out of scope; file deliberately untouched). |
| `.planning/phases/04-quran-graph/04-VERIFICATION.md` modified in the working tree | not staged/committed by this plan | Verifier-authored `gaps_found` report from `2026-10-16T10:43:42Z`, the input this plan closes. Left uncommitted intentionally — it is the verifier's artifact, not this plan's. |
| Projection selector absent on `graph path` / `subgraph` / `pattern` / `inspect` | throughout | Intentional scope boundary, recorded as §7 in `docs/05-followups/phase-04-deferrals.md`. |

## Known Stubs

None — every field added by this plan is wired end to end (selector → `ReadOptions` → backend → payload), and no placeholder or hardcoded-empty value was introduced.

## Threat Flags

None — no new security-relevant surface beyond the plan's `<threat_model>`. The selector adds no widening path: it resolves through `open_for_projection`'s active-build lookup, and expansion still uses the same `AuthzScope` and pre-serialization allowlist for every family (T-04-17); the default is pinned byte-identical by the explicit-structural snapshot segment and the `--file` USAGE guard (T-04-18).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- SC1's gap is closed on the code and evidence side: a caller can reach a concept/entity neighborhood from the CLI (`--projection`), HTTP (`projection` body field), and the agent runtime (`quran.graph_neighbors` v1.1.0), with the default structural read unchanged.
- Re-running `/gsd-verify-work` for phase 04 should now see the concept-neighborhood evidence it flagged as missing and can re-score SC1.
- Carried forward, not blockers: the workspace-wide clippy warnings above, and the structural-only read verbs recorded in the deferral ledger.

---
*Phase: 04-quran-graph*
*Completed: 2026-10-16*

## Self-Check: PASSED

- All 15 plan files + this SUMMARY exist on disk.
- All 4 plan commits (`b4ad0e2`, `cc6702b`, `d4bf736`, `b069e14`) are ancestors of HEAD.
- `commits: 4` measured via `git rev-list --count f12509f3f2e61d558ced5f336168e20837992da5..b069e14` (the 4 task/fix commits; the metadata commit that carries this file is separate, per the plan-commit-ledger contract).
