---
phase: 04-quran-graph
plan: 02
subsystem: database
tags: [rust, sqlite, knowledge-graph, quran-graph, review-lifecycle, word-root, annotations, trycmd]

# Dependency graph
requires:
  - phase: 04-quran-graph
    provides: 04-01 tracer slice (SqliteGraphStore, structural builder, migration 0022, seed/golden fixtures)
provides:
  - quran_graph_annotations service (propose/suggest/accept/reject/dispute/correct with single-transaction authority writes)
  - word-root builder (active-dataset-gated, dataset-attributed edges)
  - annotated builder (concept-seed-v1 links plus refs-only TRANSLATES coverage)
  - clear_projection_adjacency plus effective-assertion re-derivation (AC-P4-03)
  - Review verb group (CLI management surface with snapshot coverage)
  - GraphError::UnknownAssertion (QAI-GRAPH-0007)
affects: [04-03-read-surfaces, 04-04-parity, 04-05-doctor-export]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
# Same estimateTokens scale (chars/4 over the realized diff), never a harness token count.
actuals:
  tokens: 48062
  tasks: 3
  commits: 3

# Tech tracking
tech-stack:
  added: []
  patterns: [single-transaction authority write over raw sqlx pool, attribution-assertion-per-build, effective-assertion re-derivation on rebuild, cli-thin-wrapper-over-service]

key-files:
  created:
    - crates/application/src/quran_graph_annotations.rs
    - crates/application/tests/graph_review.rs
    - crates/application/tests/graph_build.rs
  modified:
    - crates/application/src/quran_graph_build.rs
    - crates/application/src/quran_cli.rs
    - crates/application/src/lib.rs
    - crates/cli/src/quran.rs
    - crates/cli/tests/quran.rs
    - crates/quran-graph/src/error.rs
    - crates/audit/src/lib.rs
    - fixtures/quran/graph/concept-seed-v1.json

key-decisions:
  - "Annotation writes use one raw sqlx transaction (assertion+provenance+audit+outbox) because graph tables have no storage seam; audit linkage and row shapes mirror storage-sqlite/audit_bridge exactly and the chain verifies under the production checker"
  - "Assertion authority links the active build row else the family's latest row; a family with no build row at all is a typed rejection (build first), never an orphan — the 0022 FK is retained"
  - "Provenance principal FKs name the recording operator (invoked_by); scholarly attribution names the reviewer/author in the assertion row, attribution JSON, and audit event"
  - "Rebuild re-derives adjacency for effective assertions scoped by (projection_id, edition_id); structural rebuilds do not re-derive (deterministic corpus family)"
  - "Review E2E lives in the host-backed Rust runner post-build, not in graph_s1.trycmd, because trycmd segments execute pre-activation and authority linkage requires a built projection"
  - "TRANSLATES edges are ayah-to-translation-edition coverage refs; the builder reads translation editions, never passages (ADR-0112)"

patterns-established:
  - "Attribution assertion per build: one pending layer-B import-kind row per word-root/annotated build that every derived edge references"
  - "Content-addressed conflict reuse: identical claim under a taken ID returns the existing row (no new side rows); a different claim is a typed rejection"
  - "Pre-serialization allowlist (visible_export_sets): structural plus effective-attributed edges travel; tombstoned IDs never reach export bytes"

requirements-completed: [REQ-quran-graph]

# Coverage metadata (#1602) — one entry per shipped deliverable. Drives DETERMINISTIC UAT routing in verify-work.
coverage:
  - id: D1
    description: "Review lifecycle service: suggest-then-accept/reject, correct-creates-new-with-supersedes, tombstone invisibility in traversal and export bytes, disputed stays effective"
    requirement: "REQ-quran-graph"
    verification:
      - kind: integration
        ref: "crates/application/tests/graph_review.rs (8 tests)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Word-root builder from the active synthetic dataset with typed UnavailableDataset otherwise; annotated builder from concept seed with refs-only translation edges; AC-P4-03 delete-plus-rebuild preservation"
    requirement: "REQ-quran-graph"
    verification:
      - kind: integration
        ref: "crates/application/tests/graph_build.rs (6 tests)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Review CLI verbs (propose/suggest/accept/correct) with reviewer plus timestamp in snapshots, typed NOT_FOUND/VALIDATION errors, and audit emission on the host-backed database"
    requirement: "REQ-quran-graph"
    verification:
      - kind: e2e
        ref: "crates/cli/tests/quran.rs#quran_graph_snapshots"
        status: pass
    human_judgment: false
  - id: D4
    description: "No HTTP mutation route and no agent-tool entry for review mutations (D-11 scope fence)"
    requirement: "REQ-quran-graph"
    verification:
      - kind: unit
        ref: "crates/application/tests/graph_review.rs#no_review_mutation_beyond_cli_management"
        status: pass
    human_judgment: false

# Metrics
duration: 1h 10m
completed: 2026-09-29
status: complete
---

# Phase 04 Plan 02: Review Lifecycle Plus Builders Summary

**Assertion authority with propose/suggest/accept/reject/correct and tombstone semantics, dataset-gated word-root and seed-driven annotated builders with AC-P4-03 rebuild preservation, and audited CLI review verbs — all proofed by 15 integration/e2e tests**

## Performance

- **Duration:** 1h 10m (approx)
- **Started:** 2026-09-28T23:40:00+03:30 (approx)
- **Completed:** 2026-09-29T00:47:00+03:30
- **Tasks:** 3
- **Files modified:** 11 (3 created in plan scope + 8 modified)

## Accomplishments

- Review lifecycle over the 0022 schema: manual propose (layer B, seven-field PRD-10.3 provenance), algorithmic suggest (layer D with algorithm/version/confidence), accept/reject/dispute/correct with reviewer plus timestamp; correct inserts a NEW accepted row with `supersedes_id` while the old row keeps its identity as Superseded
- Every write commits assertion plus provenance plus audit plus outbox in ONE transaction with content-addressed conflict reuse; the audit chain verifies under the production checker
- Tombstoned (rejected/superseded) rows vanish from traversal and export bytes while retained for audit; disputed rows stay effective; empty queue and all-tombstoned report complete-empty
- Word-root projection builds only from the active attributed morphology dataset (typed `UnavailableDataset` QAI-MORPH-0004 otherwise); competing analyses coexist; every edge references the pending dataset-attribution assertion
- Annotated projection consumes concept-seed-v1.json links into concept/entity nodes plus `MENTIONS_CONCEPT`/`REFERS_TO` and refs-only `TRANSLATES` coverage edges against registered translation editions (passage text provably absent from projection bytes)
- Delete-plus-rebuild preserves every authority and review-history row and re-derives accepted edges into the fresh build (AC-P4-03)
- CLI `quran graph review` group (propose/suggest/accept/correct) with reviewer plus timestamp in human output, full records in JSON, NOT_FOUND for unknown ids, VALIDATION for CHECK violations, and audit emission on the host-backed database; OD-11/OD-12 stay recorded BLOCKED with closing references

## Task Commits

Each task was committed atomically:

1. **Task 1: Assertion authority and review state machine service** - `267d1a7` (feat)
2. **Task 2: Word-root and annotated builders with rebuild preservation** - `2da0edf` (feat)
3. **Task 3: CLI review management verbs with audit** - `8bc6301` (feat)

## Files Created/Modified

- `crates/application/src/quran_graph_annotations.rs` - Review lifecycle service: propose/suggest/decide/correct, review queue plus history reads, single-transaction authority writer, pre-serialization allowlist
- `crates/application/tests/graph_review.rs` - 8 lifecycle integration tests plus the D-11 no-mutation-beyond-CLI pin
- `crates/application/src/quran_graph_build.rs` - Word-root collector/builder/publisher, annotated seed parser/builder/publisher, shared reserve/stage/verify/flip helpers, adjacency wipe plus effective-assertion re-derivation
- `crates/application/tests/graph_build.rs` - 6 builder integration tests with OD-11/OD-12 BLOCKED pins and the AC-P4-03 proof
- `crates/application/src/quran_cli.rs` - Review option structs plus four thin command wrappers; `UnknownAssertion` maps to NOT_FOUND
- `crates/application/src/lib.rs` - Module wiring for the annotations service
- `crates/cli/src/quran.rs` - `ReviewAction` subcommands plus dispatch (CLI-only, no HTTP/tools)
- `crates/cli/tests/quran.rs` - Review E2E segment inside `quran_graph_snapshots` with audit-list assertions
- `crates/quran-graph/src/error.rs` - `GraphError::UnknownAssertion` (QAI-GRAPH-0007, appended, never renumbered)
- `crates/audit/src/lib.rs` - `GraphAssertionProposed` plus `GraphAssertionDecided` audit actions
- `fixtures/quran/graph/concept-seed-v1.json` - Backward-compatible `links` array (5 curated seed edges); seed validation test still green

## Decisions Made

- Single raw-sqlx-transaction authority writes (no storage seam for graph tables): audit linkage and provenance/outbox row shapes mirror `storage-sqlite`/`audit_bridge` exactly, verified by `verify_persisted_audit` in the test
- Authority rows link the active build row else the family's latest row; annotation against a never-built family is a typed rejection naming the remedy (the 0022 build-row FK is retained, not bypassed)
- Provenance principal FKs (`created_by`/`reviewed_by`) name the recording operator; scholarly attribution (reviewer/author) travels in the assertion row, attribution JSON, and audit event — documented on the input types
- Re-derivation scopes by (`projection_id`, `edition_id`) using the canonical edition id both builders pin; the CLI defaults `--edition` to the active edition id for the same reason
- Review CLI E2E lives in the host-backed Rust runner after build (not in `graph_s1.trycmd`): trycmd segments execute pre-activation at the import boundary, and authority linkage requires a built projection
- `TRANSLATES` edges are ayah-to-translation-edition coverage references; the builder lists translation editions and never reads passages, so refs-only holds by construction (ADR-0112)
- `#[allow(clippy::large_enum_variant)]` on `GraphAction` follows the existing `Commands` precedent; variant boxing would fight clap's subcommand derive

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Authority rows require a build row for the 0022 FK**
- **Found during:** Task 1 (first `graph_review` run: `FOREIGN KEY constraint failed`)
- **Issue:** `graph_assertions.projection_row_id` is `NOT NULL REFERENCES graph_projections(id)`; inserting `''` violates the retained staging-linkage FK
- **Fix:** `authority_build_row` resolves the active row else the family's latest row (any status); a never-built family returns a typed rejection naming the build-first remedy
- **Files modified:** crates/application/src/quran_graph_annotations.rs
- **Verification:** `cargo test -p application --test graph_review` (8 pass)
- **Committed in:** 267d1a7 (Task 1 commit)

**2. [Rule 2 - Missing Critical] Provenance writes must satisfy the 0003 CHECK domain plus principal FKs**
- **Found during:** Task 1 (provenance FK failure, then CHECK failures)
- **Issue:** `provenance_records` restricts layer/attribution-kind/verification-status to closed lists and FKs `created_by`/`reviewed_by` to principals; invented labels fail the write
- **Fix:** Inputs carry `invoked_by` (ensured via `ensure_principal` before the write tx); layers map to `scholarly_annotation`/`computational_annotation`, kinds to `scholar`/`computational`, statuses to `unverified`/`human_verified`/`rejected`/`needs_review`
- **Files modified:** crates/application/src/quran_graph_annotations.rs, crates/application/tests/graph_review.rs
- **Verification:** graph_review 8 pass incl. chain-validity pin
- **Committed in:** 267d1a7 (Task 1 commit)

**3. [Rule 1 - Bug] Content comparison excluded volatile claim fields**
- **Found during:** Task 1 (identical re-propose reported "different claim")
- **Issue:** The claim embeds `created_at`, so byte comparison always differs across writes
- **Fix:** Reuse compares the stable triple plus evidence, location, and attribution; volatile timestamp/status excluded
- **Files modified:** crates/application/src/quran_graph_annotations.rs
- **Verification:** `unknown_assertion_is_not_found_and_bad_inputs_are_validation` reuse case
- **Committed in:** 267d1a7 (Task 1 commit)

**4. [Rule 2 - Missing Critical] Concept seed carried no linkable edges**
- **Found during:** Task 2 (annotated builder had nodes but no edge source)
- **Issue:** `concept-seed-v1.json` listed concepts/persons/places with no ayah linkage, so "consuming the seed into edges" was unimplementable without inventing linkage
- **Fix:** Added a backward-compatible top-level `links` array (5 curated edges with evidence plus source locations); the 04-01 seed validator still passes unchanged
- **Files modified:** fixtures/quran/graph/concept-seed-v1.json, crates/application/src/quran_graph_build.rs
- **Verification:** `cargo test -p quran-graph --test conformance seed_fixtures_validate` plus graph_build 6 pass
- **Committed in:** 2da0edf (Task 2 commit)

**5. [Rule 1 - Bug] AC-P4-03 survival is subset, not set equality**
- **Found during:** Task 2 (republish mints a fresh attribution row)
- **Issue:** Asserting authority set equality across republish fails by design: the new build legitimately adds its own attribution assertion
- **Fix:** Test asserts every pre-existing (id, decision) pair survives with its decision (superset check), not equality
- **Files modified:** crates/application/tests/graph_build.rs
- **Verification:** `rebuild_preserves_authority_and_review_history_ac_p4_03`
- **Committed in:** 2da0edf (Task 2 commit)

**6. [Rule 3 - Blocking] Review CLI E2E runs post-build in the Rust runner, not in graph_s1.trycmd**
- **Found during:** Task 3 (trycmd segments execute at the import boundary, pre-activation)
- **Issue:** The plan's trycmd review segment would run before any build exists, so every propose would hit the Rule-1 typed rejection; `activate` cannot move into trycmd (needs `--yes` on a terminal)
- **Fix:** Review verbs are covered in `quran_graph_snapshots` via `qai_out` on the same host-backed database after build, including `qai audit list` assertions; `graph_s1.trycmd` keeps the import segment
- **Files modified:** crates/cli/tests/quran.rs
- **Verification:** `cargo test -p cli --test quran quran_graph_snapshots`
- **Committed in:** 8bc6301 (Task 3 commit)

**7. [Rule 2 - Missing Critical] New crates need error-code and audit-action vocabulary**
- **Found during:** Tasks 1/3 (unknown ids had no typed error; review audit had no fitting action)
- **Issue:** `GraphError` had no assertion-not-found variant and `AuditAction` had no review actions; reusing neighbors/actions would mislabel surfaces
- **Fix:** Appended `GraphError::UnknownAssertion` (QAI-GRAPH-0007, codes never renumbered) plus `GraphAssertionProposed`/`GraphAssertionDecided` audit actions (additive, no exhaustive matches exist)
- **Files modified:** crates/quran-graph/src/error.rs, crates/audit/src/lib.rs, crates/application/src/quran_cli.rs
- **Verification:** QAI-GRAPH-0007 pin in graph_review; audit-list assertions in quran_graph_snapshots; full quran-graph plus audit suites green
- **Committed in:** 267d1a7 and 8bc6301

---

**Total deviations:** 7 auto-fixed (2 bugs, 3 missing-critical, 2 blocking)
**Impact on plan:** All auto-fixes preserve the plan's intent while respecting harder constraints (0022 FK, 0003 CHECK domain, trycmd execution order, arch-check). No scope creep; zero new dependencies.

## Issues Encountered

- `cargo fmt -- <files>` reformats the whole workspace, not the listed files: it touched three unrelated Phase-3 test files. Reverted immediately with `git checkout`; all later formatting used `rustfmt --edition 2024` on plan files only (verified pure-addition diffs)
- Clippy `large_enum_variant` fired on `GraphAction` after adding the Review group: resolved with `#[allow]` following the existing `Commands` precedent rather than boxing against clap's subcommand derive

## Known Stubs

None - no placeholder values, TODO markers, or unwired surfaces introduced. The `dispute` service operation has no CLI verb yet (accept/reject/correct cover the plan's CLI scope); it is fully wired at the service layer for 04-03/04-05 surfaces.

## Threat Flags

None - all new write surfaces were in the plan's threat model: T-04-04 (layer-D CHECK plus reviewer/timestamp, pending-labeled suggestions) pinned by graph_review cases; T-04-05 (`is_effective` during expansion plus `visible_export_sets` pre-serialization, byte-absence asserted) pinned; T-04-06 (active-dataset gate, synthetic labeling, no root invention) pinned; T-04-SC accepted (zero new dependencies).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 04-03 (export/read surfaces): `visible_export_sets` is the pre-serialization policy filter to consume; attribution assertions travel with edges; review queue/history reads are the management backing
- Ready for 04-04/04-05: re-derivation helper and `clear_projection_adjacency` define the rebuild contract; OD-11/OD-12 stay BLOCKED with test-name and CLI-help markers pointing at decisions-needed.md
- Watch items: authority `edition_id` scope uses the canonical edition row id (CLI defaults `--edition` to active); structural rebuilds intentionally do not re-derive annotation edges; the `dispute` op awaits a CLI verb if 04-05 wants snapshot coverage for it

---
*Phase: 04-quran-graph*
*Completed: 2026-09-29*

## Self-Check: PASSED

All 11 plan files exist on disk; all 3 task commits resolve in git log
(`267d1a7`, `2da0edf`, `8bc6301`); plan verification set re-ran green at
close-out (graph_review 8, graph_build 6, quran_graph_snapshots 1,
quran-graph 49, audit unit suites, arch-check clean, clippy `-D warnings`
clean on all touched crates, rustfmt clean on all plan files).
