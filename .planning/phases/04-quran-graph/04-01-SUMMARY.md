---
phase: 04-quran-graph
plan: 01
subsystem: database
tags: [rust, sqlite, knowledge-graph, quran-graph, traversal, conformance, trycmd]

# Dependency graph
requires:
  - phase: 02-canonical-quran-core
    provides: active-edition pointer, canonical reader, verify_canonical_quotation pin
  - phase: 03-quran-search-linguistics
    provides: host-backed trycmd segment pattern, UnitOfWork read conventions
provides:
  - 0022_quran_graph_fix migration (multi-assertion key, disputed state, authority scope, edge attrs)
  - SqliteGraphStore (application-layer GraphStore read-view over the pinned active projection)
  - structural builder with single-transaction fenced publish + generation stamp
  - SQLite-backed CLI build/inspect/neighbors with per-hit pinned canonical quotations
  - TRANSLATES/RELATED_TO/SUPPORTED_BY/DISPUTED_BY vocabulary + AssertionDecision::Disputed
  - backend-generic conformance harness (Mem + SQLite rows) with budget-exhaustion pins
  - concept-seed-v1.json + annotation-goldens.json fixtures
affects: [04-02-annotations, 04-03-read-surfaces, 04-04-parity, 04-05-doctor-export]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
# Same estimateTokens scale (chars/4 over the realized diff), never a harness token count.
actuals:
  tokens: 33432
  tasks: 3
  commits: 3

# Tech tracking
tech-stack:
  added: []
  patterns: [snapshot-on-open sync-port adapter, single-transaction staged build with fenced publish, backend-generic conformance matrix]

key-files:
  created:
    - migrations/sqlite/0022_quran_graph_fix.up.sql
    - crates/application/src/quran_graph_store.rs
    - crates/application/src/quran_graph_build.rs
    - crates/application/tests/graph_neighbors.rs
    - crates/cli/tests/quran/graph_s1.trycmd
    - fixtures/quran/graph/concept-seed-v1.json
    - fixtures/quran/graph/annotation-goldens.json
  modified:
    - migrations/sqlite/checksums.json
    - crates/application/src/lib.rs
    - crates/application/src/quran_cli.rs
    - crates/cli/src/quran.rs
    - crates/cli/tests/quran.rs
    - crates/quran-graph/src/model.rs
    - crates/quran-graph/tests/conformance.rs
    - crates/quran-graph/Cargo.toml
    - xtask/allowlist.toml
    - Cargo.lock

key-decisions:
  - "SqliteGraphStore is a snapshot-on-open read-view delegating traversal to the reference expansion logic: the sync GraphStore port cannot perform async SQL per query without blocking an executor, so behavioral parity holds by construction"
  - "Conformance SQLite rows required dev-only quran-graph test edges to application/storage-sqlite plus sqlx/tempfile/tokio (testkit precedent in the allowlist); production port direction application -> quran-graph is unchanged"
  - "Migration 0022 also adds graph_edges.attrs_json so structural input-version provenance round-trips (refs-only, never canonical text)"
  - "crates/graph stays reserved with zero Phase 4 code (RESEARCH open question 3, decided)"
  - "Graph error to CLI exit mapping: NodeNotFound/UnknownProjection -> NOT_FOUND, BudgetExceeded/PatternRejected -> VALIDATION, AuthzDenied -> POLICY, BuildFailed -> INTERNAL"

patterns-established:
  - "Snapshot adapter: pin one projection build row, load via parameterized SQL, delegate sync queries to reference logic"
  - "Single-transaction build: reserve (building) -> stage batches -> verify (counts, no-dangling, canonical-unchanged) -> flip active + supersede previous in one commit"
  - "Backend-generic conformance: pure-data fixtures seeded into both backends, backend-name println for --nocapture listings"

requirements-completed: [REQ-quran-graph]

# Coverage metadata (#1602) — one entry per shipped deliverable. Drives DETERMINISTIC UAT routing in verify-work.
coverage:
  - id: D1
    description: "0022_quran_graph_fix migration applies cleanly with stable checksums"
    requirement: "REQ-quran-graph"
    verification:
      - kind: other
        ref: "cargo xtask migrate-check"
        status: pass
    human_judgment: false
  - id: D2
    description: "SqliteGraphStore opens the active structural projection with manifest generation stamp"
    requirement: "REQ-quran-graph"
    verification:
      - kind: integration
        ref: "crates/application/tests/graph_neighbors.rs#structural_build_inspect_neighbors_canonical_quotation"
        status: pass
    human_judgment: false
  - id: D3
    description: "Bounded neighbors around a verse with per-hit pinned canonical quotations re-verifying byte-identical"
    requirement: "REQ-quran-graph"
    verification:
      - kind: integration
        ref: "crates/application/tests/graph_neighbors.rs#structural_build_inspect_neighbors_canonical_quotation"
        status: pass
      - kind: e2e
        ref: "crates/cli/tests/quran.rs#quran_graph_snapshots"
        status: pass
    human_judgment: false
  - id: D4
    description: "Same node pair carries multiple attributed assertions without constraint violation; tombstoned rows stay hidden"
    requirement: "REQ-quran-graph"
    verification:
      - kind: integration
        ref: "crates/application/tests/graph_neighbors.rs#one_triple_carries_multiple_attributed_assertions"
        status: pass
    human_judgment: false
  - id: D5
    description: "CLI build persists to SQLite; inspect/neighbors --db work; selection and validation errors stay typed"
    requirement: "REQ-quran-graph"
    verification:
      - kind: e2e
        ref: "crates/cli/tests/quran.rs#quran_graph_snapshots"
        status: pass
    human_judgment: false
  - id: D6
    description: "Extended edge vocabulary (TRANSLATES/RELATED_TO/SUPPORTED_BY/DISPUTED_BY) with predecessor/successor aliases still rejected; Disputed is effective, never tombstoned"
    requirement: "REQ-quran-graph"
    verification:
      - kind: unit
        ref: "crates/quran-graph/src/model.rs#vocabulary_has_expected_entries + tombstone_semantics"
        status: pass
    human_judgment: false
  - id: D7
    description: "Backend-generic conformance harness: identical triangle/authz/tombstone/budget fixtures pass on Mem and SQLite"
    requirement: "REQ-quran-graph"
    verification:
      - kind: integration
        ref: "cargo test -p quran-graph --test conformance"
        status: pass
    human_judgment: false
  - id: D8
    description: "Budget-exhaustion pins: fanout/node/edge truncation with reasons, cycle termination, authz counts, cancel incompletes, per-field pre-flight with QAI-GRAPH-0002"
    requirement: "REQ-quran-graph"
    verification:
      - kind: integration
        ref: "cargo test -p quran-graph --test conformance (sqlite_* rows)"
        status: pass
    human_judgment: false
  - id: D9
    description: "Seed fixtures validate as versioned synthetic-labeled JSON with full review lifecycle"
    requirement: "REQ-quran-graph"
    verification:
      - kind: integration
        ref: "crates/quran-graph/tests/conformance.rs#seed_fixtures_validate"
        status: pass
    human_judgment: false

# Metrics
duration: 2h 25m
completed: 2026-09-28
status: complete
---

# Phase 04 Plan 01: Quran Graph Tracer Slice Summary

**SQLite-backed structural projection end-to-end: fenced build from the active edition, manifest inspect with generation stamp, bounded neighbors with per-hit pinned canonical quotations, multi-assertion triples, and a backend-generic conformance suite proving SQLite parity**

## Performance

- **Duration:** 2h 25m
- **Started:** 2026-09-28T21:05:00+03:30 (approx)
- **Completed:** 2026-09-28T23:30:24+03:30
- **Tasks:** 3
- **Files modified:** 17 (7 created in plan scope + 10 modified, incl. Cargo.lock)

## Accomplishments

- Structural projection builds from the synthetic fixture into SQLite and publishes through a single-transaction fenced flip with the generation stamp read from the active edition row
- Manifest inspects with `projection_id quran-structural-v1` and `corpus_generation` matching the active edition; neighbors open around `ayah:1:1` bounded, deterministic, and complete under default budgets
- Every ayah hit links to a pinned canonical ref (`quran:test-edition-min@0.1.0:s:a`) that re-verifies byte-identical through `verify_canonical_quotation` (ExactMatch, resolver-read hash)
- Same triple carries two accepted attributions plus one hidden tombstoned rejection with no constraint violation (NULL-safe partial unique indexes)
- Conformance suite (19 tests) runs identical fixtures on Mem and SQLite backends; budget-exhaustion, authz-count, cycle, cancel, and per-field pre-flight pins all green on both
- Vocabulary extended per D-04/D-05/D-07; `Disputed` authorizes traversal while rejected/superseded hide; predecessor/successor aliases still rejected
- Seed fixtures carry version, test-min scope, synthetic-only labeling, per-entry attribution, the full review lifecycle, and explicit OD-11/OD-12 BLOCKED plus P4-X01..X05 pending-ratification headers
- `crates/graph` untouched (reserved, zero Phase 4 code)

## Task Commits

Each task was committed atomically:

1. **Task 1: End-to-end structural build, inspect, neighbors, canonical quotation** - `4b03aed` (feat)
2. **Task 2: Vocabulary extension, disputed status, conformance parity, seed fixtures** - `4347709` (feat)
3. **Task 3: SQLite adapter parity hardening and budget-exhaustion pins** - `0631739` (test)

## Files Created/Modified

- `migrations/sqlite/0022_quran_graph_fix.up.sql` - Forward-only graph schema fix: multi-assertion partial unique indexes, disputed CHECK, authority scope columns, edge attrs_json
- `migrations/sqlite/checksums.json` - 0022 checksum entry
- `crates/application/src/quran_graph_store.rs` - SqliteGraphStore snapshot read-view + GraphStore delegation
- `crates/application/src/quran_graph_build.rs` - Structural input collection + staged-batch build with fenced publish
- `crates/application/src/lib.rs` - Module wiring for the two new services
- `crates/application/src/quran_cli.rs` - Build persists to SQLite; inspect/neighbors gain --db with reader-resolved quotations; typed graph error mapping
- `crates/application/tests/graph_neighbors.rs` - Tracer integration proof (build/inspect/neighbors/quotation/multi-assertion/tombstone)
- `crates/cli/src/quran.rs` - GraphAction Inspect/Neighbors gain optional --file plus --db; dispatch threads db_path
- `crates/cli/tests/quran.rs` - quran_graph_snapshots e2e (host-backed import, then activate/build/inspect/neighbors assertions)
- `crates/cli/tests/quran/graph_s1.trycmd` - Migrate + import segment for the graph snapshot test
- `crates/quran-graph/src/model.rs` - Four new predicates (len 15->19); AssertionDecision::Disputed (effective, never tombstoned)
- `crates/quran-graph/tests/conformance.rs` - Backend-generic harness (8 original + 11 new tests on both backends)
- `crates/quran-graph/Cargo.toml` - Dev-only conformance SQLite wiring
- `fixtures/quran/graph/concept-seed-v1.json` - Versioned synthetic concept/person/place seed
- `fixtures/quran/graph/annotation-goldens.json` - Full review-lifecycle golden assertions
- `xtask/allowlist.toml` - Dev-only quran-graph test edges with justification comments
- `Cargo.lock` - New dev-dependency edges recorded

## Decisions Made

- Snapshot-on-open adapter with delegation to the reference expansion logic, because the sync GraphStore port cannot perform async SQL per query without blocking an executor; parity holds by construction rather than by parallel implementation
- Dev-only allowlist edges (quran-graph test target to application/storage-sqlite, sqlx/tempfile/tokio) following the testkit precedent comment convention; production port direction application -> quran-graph unchanged and arch-check green
- 0022 adds `graph_edges.attrs_json` so structural input-version provenance round-trips through the projection (Rule 2: refs-only provenance would otherwise be lost)
- `crates/graph` reserved with zero Phase 4 code; all Phase 4 work in quran-graph + application
- CLI error mapping: NodeNotFound/UnknownProjection -> exit 5, BudgetExceeded/PatternRejected -> exit 3, AuthzDenied -> exit 4, BuildFailed -> exit 70
- Stale-projection signal is advisory in CLI output (`stale` flag + human note), never a failure; quotation failure is fail-closed internal, never fabricated
- Pre-set cancel flag approximates mid-expansion cancellation in tests (exercises the same per-batch gate); true mid-flight injection is not synchronously expressible against the sync port

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Snapshot+delegate adapter instead of batched per-hop SQL**
- **Found during:** Task 1 (SqliteGraphStore implementation)
- **Issue:** The plan prescribes batched single-hop SELECTs during expansion, but the GraphStore port's query methods are synchronous while SQL access is async; blocking an executor inside a query method is unsound, and changing the port shape is forbidden by task 3
- **Fix:** Pin one projection row, load nodes/edges/assertions once through parameterized SQL (edges ordered by edge, dst), delegate all five query methods to the reference expansion logic — pre-flight, per-batch cancel, in-expansion authz, stable ordering, and typed incompletes hold by construction
- **Files modified:** crates/application/src/quran_graph_store.rs
- **Verification:** Backend-generic conformance (19 tests x 2 backends) plus graph_neighbors integration, all green
- **Committed in:** 4b03aed (Task 1 commit)

**2. [Rule 2 - Missing Critical] graph_edges.attrs_json added in 0022**
- **Found during:** Task 1 (migration authoring)
- **Issue:** Migration 0019 has no attrs column on graph_edges, so structural input-version provenance (builder, edition, input version) would be silently dropped by the projection round-trip
- **Fix:** Added `attrs_json TEXT NOT NULL DEFAULT '{}'` to the rebuilt edges table; builder writes provenance, store reads it back
- **Files modified:** migrations/sqlite/0022_quran_graph_fix.up.sql
- **Verification:** migrate-check green; stored structural edges carry input_version through reopen
- **Committed in:** 4b03aed (Task 1 commit)

**3. [Rule 3 - Blocking] Dev-only allowlist edges for the conformance SQLite rows**
- **Found during:** Task 2 (conformance parameterization)
- **Issue:** The plan requires SQLite backend construction inside `cargo test -p quran-graph --test conformance`, but arch-check cannot distinguish dev-dependencies (cargo metadata lists all kinds), and the adapter cannot move into quran-graph without a layering inversion
- **Fix:** Dev-dependencies (application, storage-sqlite, sqlx, tempfile, tokio) plus allowlist entries scoped with dev-only justification comments (testkit precedent); production direction unchanged
- **Files modified:** crates/quran-graph/Cargo.toml, xtask/allowlist.toml, Cargo.lock
- **Verification:** arch-check green; task-3 acceptance directions hold (no storage-sqlite -> quran-graph edge, adapter edge stays application -> quran-graph)
- **Committed in:** 4347709 (Task 2 commit)

**4. [Rule 2 - Missing Critical] Typed graph error to CLI exit mapping**
- **Found during:** Task 1 (CLI --db verbs)
- **Issue:** The file-backed verbs mapped every graph error (including budget violations) to NOT_FOUND, which would render pre-flight rejections and authz denials as misleading statuses
- **Fix:** graph_error_exit maps NodeNotFound/UnknownProjection -> 5, BudgetExceeded/PatternRejected -> 3, AuthzDenied -> 4, BuildFailed -> 70, applied to file and db paths
- **Files modified:** crates/application/src/quran_cli.rs
- **Verification:** quran_graph_snapshots pins exit 5 for unknown nodes and exit 3 for hops=0
- **Committed in:** 4b03aed (Task 1 commit)

**5. [Rule 1 - Bug] Cycle test expectation corrected for direction-agnostic expansion**
- **Found during:** Task 3 (cycle pin)
- **Issue:** My test asserted one path on a 3-cycle, but EdgeFilter::any() expands both directions, so the one-hop backward traversal is a second valid shortest-first path
- **Fix:** Assert both paths in shortest-first stable order instead of weakening the fixture
- **Files modified:** crates/quran-graph/tests/conformance.rs
- **Verification:** sqlite_cycles_terminate_within_hops green on both backends
- **Committed in:** 0631739 (Task 3 commit)

Minor notes (not rule-classed): the `disputed -> Disputed` store mapping landed in the task-2 commit (one task early, since the variant only exists from task 2); the task-3 commit is test-only because delegation needed no store change; pre-existing rustfmt drift in three unrelated test files was reverted untouched; the `supersedes_id` self-FK in 0022 names the post-rename table so it resolves after `ALTER ... RENAME TO`.

---

**Total deviations:** 5 auto-fixed (1 bug, 2 missing-critical, 2 blocking)
**Impact on plan:** All auto-fixes preserve the plan's intent while respecting harder constraints (sync port shape, arch-check, CLI exit-code contract). No scope creep; no new production dependencies.

## Issues Encountered

- `cargo xtask arch-check` hung once under rapid successive cargo invocations (lock contention); a standalone retry passed immediately — transient, not a code issue
- Full `cargo xtask ci` stays the phase gate in plan 04-05 per the plan; this plan ran its own verification set (quran-graph suite, graph_neighbors, quran_graph_snapshots, migrate-check, arch-check, fmt, clippy)

## Known Stubs

None - no placeholder values, TODO markers, or unwired surfaces introduced. The `disputed` decision path is fully wired (model variant, migration CHECK, store mapping, conformance pins, golden fixture row).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 04-02 (annotations): SqliteGraphStore, fenced builder, authority scope columns, disputed state, and seed/golden fixtures are the exact inputs the annotation builder and review verbs need
- Watch items for later plans: batched frontier SQL remains an allowed optimization behind SqliteGraphStore if projections outgrow snapshot loads; native timeout_ms enforcement is cancel-flag-equivalent in both backends today; stale-projection drift detection belongs to the 04-05 doctor verb
- Pre-existing `cargo fmt --check` drift in three unrelated Phase-3 test files was left untouched (out of scope, logged here for visibility)

---
*Phase: 04-quran-graph*
*Completed: 2026-09-28*

## Self-Check: PASSED

All 7 created files exist on disk; all 3 task commits resolve in git log;
`crates/graph` untouched; plan verification set re-ran green at close-out
(quran-graph 49 tests, graph_neighbors 2, quran_graph_snapshots 1,
migrate-check 22 migrations, arch-check clean).
