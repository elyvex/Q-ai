---
phase: 05-rich-quran-experience
plan: 01
subsystem: api
tags: [research-checksum, tool-contract, sha256, canonical-json, parity, quran-tools, openapi]

# Dependency graph
requires:
  - phase: 04-quran-graph
    provides: 12-name ToolRegistry, graph tool backends, versioned HTTP API and CLI surfaces this plan extends
provides:
  - Required payload-covering `research_checksum` on the shared `ToolResult` contract, populated by all 12 registered tools
  - One shared name→tool dispatcher (`dispatch_registered_tool`) backing both `quran tool … --json` and `POST /api/v1/quran/tool/{name}`
  - `Meta.research_checksum` on every HTTP envelope plus OpenAPI/test lock-step for the field and the generic route
affects: [05-02, 05-03, 05-04, 05-05, 05-06, 05-07, 05-08, 05-09, phase-10-agent-runtime]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
actuals:
  tokens: 13650
  tasks: 3
  commits: 3

# Tech tracking
tech-stack:
  added: []
  patterns: [domain-separated checksum over canonical JSON, single shared dispatcher table for cross-surface parity, typed-error-only tool boundary]

key-files:
  created: []
  modified: [crates/tools/src/lib.rs, crates/tool-registry/src/lib.rs, crates/application/src/quran_tools.rs, crates/application/src/quran_graph_tools.rs, crates/application/src/quran_cli.rs, crates/cli/src/quran.rs, crates/server/src/api.rs, docs/08-api/quran-v1-openapi.json]

key-decisions:
  - "Checksum envelope keeps the plan's exact shape (domain + tool@version + params + inputs + payload) via frozen canonical_json_bytes; no finish-constructor, literals carry one checksum call each"
  - "Graph envelope folds projection/dataset pinning into checksum inputs so identical payloads from different builds cannot collide"
  - "CLI --json emits the dispatcher value verbatim (checksum as {algorithm,hex} object); the sha256:<hex> string rendering lives in Meta, matching meta_from_tool"
  - "Bespoke search/lexicon/graph routes keep empty research_checksum and their own identities; only the generic typed-tool route carries the registry checksum"

patterns-established:
  - "Research checksum: tools::research_checksum(tool, version, params, inputs, payload) reused at every ToolResult site with per-tool canonical inputs"
  - "Cross-surface parity by construction: dispatch_registered_tool is the one table CLI and HTTP legs share"
  - "Meta projection rule: HTTP never recomputes checksums, it projects result.research_checksum as sha256:<hex>"

requirements-completed: [REQ-quran-result-contract, REQ-quran-research-tools]

# Coverage metadata (#1602)
coverage:
  - id: D1
    description: "Required payload-covering research_checksum on ToolResult, populated by all 12 tools and projected into Meta"
    requirement: "REQ-quran-result-contract"
    verification:
      - kind: unit
        ref: "crates/tools/src/lib.rs#research_checksum_is_deterministic_payload_and_param_sensitive"
        status: pass
      - kind: unit
        ref: "crates/tools/src/lib.rs#research_checksum_is_insensitive_to_field_insertion_order"
        status: pass
      - kind: integration
        ref: "crates/server/tests/api.rs#get_ayah_research_checksum_matches_service"
        status: pass
    human_judgment: false
  - id: D2
    description: "All 12 tools reachable via quran tool --json and POST /api/v1/quran/tool/{name} through one shared dispatcher with identical checksums"
    requirement: "REQ-quran-research-tools"
    verification:
      - kind: integration
        ref: "crates/application/tests/quran_tools.rs#cmd_tool_surfaces_research_checksum"
        status: pass
      - kind: integration
        ref: "crates/server/tests/api.rs#tool_route_serves_registered_tools_with_matching_checksum"
        status: pass
      - kind: e2e
        ref: "manual serve-backed run: CLI and HTTP legs returned identical checksum ad512ac45743a8ac…"
        status: pass
    human_judgment: false
  - id: D3
    description: "OpenAPI Meta schema and required-keys contract advertise research_checksum and the generic typed-tool route"
    requirement: "REQ-quran-result-contract"
    verification:
      - kind: unit
        ref: "crates/server/tests/api.rs#openapi_spec_covers_every_route"
        status: pass
      - kind: unit
        ref: "crates/server/tests/api.rs#openapi_spec_schemas_resolve_and_cover_json_responses"
        status: pass
    human_judgment: false

# Metrics
duration: 95min
completed: 2026-10-07
status: complete
---

# Phase 05 Plan 01: Research Result Contract Summary

**Required payload-covering `research_checksum` on every tool result, one shared dispatcher serving all 12 tools over CLI and HTTP with identical checksums, and the OpenAPI contract advertising both**

## Performance

- **Duration:** ~95 min
- **Started:** 2026-10-07T (phase execution wave)
- **Completed:** 2026-10-07
- **Tasks:** 3/3
- **Files modified:** 12

## Accomplishments
- `tools::research_checksum` (domain-separated `qai-research-checksum-v1`, frozen canonical-JSON + sha256) is a required `ToolResult` field populated at every production construction site — no site left empty, whole workspace compiles.
- `GET /api/v1/quran/ayahs/{ref}` carries `meta.research_checksum` equal to the service-side value; all five `Meta` sites set the field.
- `dispatch_registered_tool` (one 12-name table) backs `qai quran tool <name> --params … --json` and `POST /api/v1/quran/tool/{name}`; serve-backed run proved byte-identical checksums across CLI and HTTP legs.
- OpenAPI `Meta` requires `research_checksum`, documents the generic route, and the server test enforces both in lock-step.

## Task Commits

Each task was committed atomically:

1. **Task 1: End-to-end research_checksum on quran.get_ayah + migrate every construction site** - `9c6e8df` (feat)
2. **Task 2: Shared typed-tool dispatcher → CLI path and generic HTTP route** - `77dd2d1` (feat)
3. **Task 3: OpenAPI Meta schema and required-keys contract** - `d624013` (docs)

Branch: `agent-05-01-research-checksum` (3 commits on top of `main@d322d38`).

## Files Created/Modified
- `crates/tools/src/lib.rs` - `RESEARCH_CHECKSUM_DOMAIN`, `research_checksum()`, required `ToolResult::research_checksum`, unit tests
- `crates/tool-registry/src/lib.rs` - checksum at `get_ayah`/`get_context` sites + test `attributed()` helper
- `crates/tool-registry/tests/conformance.rs` - migrated the two extra literals the required field forced (`conformable_envelope`, truncated-path backend)
- `crates/application/src/quran_tools.rs` - checksum at search/root/lemma/morphology/family sites + `dispatch_registered_tool`
- `crates/application/src/quran_graph_tools.rs` - checksum in the shared `envelope` helper (all five graph tools, projection-pinned inputs)
- `crates/application/src/quran_cli.rs` - `cmd_tool` verb
- `crates/cli/src/quran.rs` - `QuranAction::Tool` + dispatch arm
- `crates/server/src/api.rs` - `Meta.research_checksum`, all five sites, `tool_handler`, `/api/v1/quran/tool/{name}` route
- `crates/server/tests/api.rs` - get_ayah checksum equality, tool-route test, required-keys + route-family + envelope assertions
- `crates/application/tests/quran_tools.rs` - get_ayah checksum test, `cmd_tool` + 12-name dispatch coverage test
- `crates/application/tests/graph_tools.rs` - graph checksum determinism/sensitivity test
- `docs/08-api/quran-v1-openapi.json` - `Meta.research_checksum` (required) + the generic route path

## Decisions Made
- Kept the plan's exact checksum envelope shape and per-site literals rather than introducing a `finish()` constructor: 10 call sites × one checksum call each was the shorter, clearer diff, and the in-test `attributed()`/`conformable_envelope` helpers already centralize the fakes.
- Graph `envelope` folds `source_versions` (projection id/builder or dataset) into the checksum inputs, so identical payloads served from different builds cannot share a digest.
- CLI `--json` emits the dispatcher value verbatim (`research_checksum` as `{algorithm, hex}` object, consistent with `reproducibility.checksum`); the `sha256:<hex>` string form lives only in `Meta`, rendered by projection. The plan's behavior prose loosely called the CLI value a string — acceptance criteria only require the serialized `ToolResult`, which is what ships.
- `meta_from_search` / `lexicon_meta` / `graph_base_meta` carry empty `research_checksum`: the bespoke routes keep their own identities and are outside the D-16 gate's scope, per plan.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Repaired a mangled function header in quran_tools.rs**
- **Found during:** Task 2 (dispatcher insertion)
- **Issue:** A mis-anchored edit deleted the `pub async fn tool_get_ayah(` header line, leaving a doc comment attached to a bare signature fragment.
- **Fix:** Re-inserted the exact header line; verified with `cargo check -p application`.
- **Files modified:** crates/application/src/quran_tools.rs
- **Verification:** `cargo check -p application` clean, no warnings
- **Committed in:** 77dd2d1 (part of task commit)

**2. [Rule 1 - Bug] Fixed moved-value borrows in the dispatcher error path**
- **Found during:** Task 2 (dispatcher insertion)
- **Issue:** `bad_params(tool, params)` took params by value while `serde_json::from_value(params)` had already moved it — 12 arms would not compile.
- **Fix:** `bad_params` takes the `serde_json::Error` instead and renders it in the detail message.
- **Files modified:** crates/application/src/quran_tools.rs
- **Verification:** `cargo check -p application -p server -p cli` clean
- **Committed in:** 77dd2d1 (part of task commit)

**3. [Rule 3 - Blocking] Migrated two unlisted ToolResult literals in conformance.rs**
- **Found during:** Task 1 (required-field migration)
- **Issue:** The plan's file list omitted `crates/tool-registry/tests/conformance.rs`, but its `conformable_envelope` and truncated-path backend literals fail compilation with the new required field.
- **Fix:** Populated `research_checksum` at both sites with the same helper, same shape as the in-crate `attributed()` fake.
- **Files modified:** crates/tool-registry/tests/conformance.rs
- **Verification:** `cargo check --workspace --all-targets` clean; `cargo test -p tool-registry --test conformance` 10 passed
- **Committed in:** 9c6e8df (part of task commit)

**4. [Rule 3 - Blocking] Reused the pre-existing test POST helper instead of a duplicate**
- **Found during:** Task 2 (server route test)
- **Issue:** Added a 3-tuple `post_json` helper that collided with the file's existing 2-tuple `post_json` wrapper over `post_raw` (E0428 + 20 type errors).
- **Fix:** Deleted the duplicate; rewrote the new test against the existing `post_json`.
- **Files modified:** crates/server/tests/api.rs
- **Verification:** `cargo test -p server --test api tool_route` passes
- **Committed in:** 77dd2d1 (part of task commit)

---

**Total deviations:** 4 auto-fixed (2 bugs, 2 blocking)
**Impact on plan:** All fixes were necessary for compilation or test integrity; no scope creep. No architectural changes (no Rule 4 gate tripped).

## Issues Encountered
- Full `cargo test -p application` exceeds a 10-minute timeout (pre-existing: soak/heavy suites). Verified via targeted suites instead: `quran_tools` (11 passed), `graph_tools` (6 passed), `search_tools` (14 passed), plus `tools`, `tool-registry` (incl. conformance), `server`, and `cli` (incl. all trycmd snapshots) fully green.
- Graph payloads embed wall-clock `duration_ms`, so two graph invocations can theoretically differ in checksum (checksum covers the payload per D-14). Observed stable (0ms) in tests; the D-16 parity gate (later plan) owns timing normalization per RESEARCH Pitfall 8.

## Threat Flags

None — all new surface (generic typed-tool route) matches the plan's threat model: fixed 12-name match, typed deserialization, existing 1MB body limit, typed 4xx mapping (T-05-23 mitigated and tested on both legs).

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- D-16 gate work (plan 05-07 or whichever owns the cross-surface golden gate) can now drive every tool through CLI, HTTP, and service legs with one shared table and compare identical checksums; note the `duration_ms` normalization caveat above for graph payloads.
- `Meta.research_checksum` is empty-string (not absent) on bespoke search/lexicon/graph routes — consumers must treat empty as "not backed by a registered tool result".
- Branch `agent-05-01-research-checksum` is ready to merge; STATE.md/ROADMAP.md updates intentionally left to the orchestrator.

---
*Phase: 05-rich-quran-experience, Plan: 01*
*Completed: 2026-10-07*
