---
phase: 04-quran-graph
verified: 2026-11-15T15:12:07Z
status: passed
score: 11/11 must-haves verified
covered_files:
  - .planning/phases/04-quran-graph/04-01-PLAN.md
  - .planning/phases/04-quran-graph/04-01-SUMMARY.md
  - .planning/phases/04-quran-graph/04-02-PLAN.md
  - .planning/phases/04-quran-graph/04-02-SUMMARY.md
  - .planning/phases/04-quran-graph/04-03-PLAN.md
  - .planning/phases/04-quran-graph/04-03-SUMMARY.md
  - .planning/phases/04-quran-graph/04-04-PLAN.md
  - .planning/phases/04-quran-graph/04-04-SUMMARY.md
  - .planning/phases/04-quran-graph/04-05-PLAN.md
  - .planning/phases/04-quran-graph/04-05-SUMMARY.md
  - .planning/phases/04-quran-graph/04-06-PLAN.md
  - .planning/phases/04-quran-graph/04-06-SUMMARY.md
  - crates/application/src/quran_cli.rs
  - crates/application/src/quran_graph_annotations.rs
  - crates/application/src/quran_graph_api.rs
  - crates/application/src/quran_graph_build.rs
  - crates/application/src/quran_graph_doctor.rs
  - crates/application/src/quran_graph_export.rs
  - crates/application/src/quran_graph_store.rs
  - crates/application/src/quran_graph_tools.rs
  - crates/application/tests/graph_build.rs
  - crates/application/tests/graph_doctor.rs
  - crates/application/tests/graph_explain.rs
  - crates/application/tests/graph_export.rs
  - crates/application/tests/graph_neighbors.rs
  - crates/application/tests/graph_paths.rs
  - crates/application/tests/graph_review.rs
  - crates/application/tests/graph_tools.rs
  - crates/cli/src/quran.rs
  - crates/cli/tests/quran.rs
  - crates/cli/tests/quran/graph_s1.trycmd
  - crates/cli/tests/quran/graph_s2.trycmd
  - crates/quran-graph/src/export.rs
  - crates/quran-graph/src/mem.rs
  - crates/quran-graph/src/model.rs
  - crates/quran-graph/src/structural.rs
  - crates/quran-graph/src/traverse.rs
  - crates/quran-graph/tests/conformance.rs
  - crates/server/src/api.rs
  - crates/server/tests/graph.rs
  - crates/tool-registry/src/lib.rs
  - crates/tool-registry/tests/conformance.rs
  - docs/05-followups/phase-04-deferrals.md
  - docs/05-followups/phase-04-owner-gates.md
  - docs/08-api/quran-v1-openapi.json
  - fixtures/quran/graph/annotation-goldens.json
  - fixtures/quran/graph/concept-seed-v1.json
  - migrations/sqlite/0022_quran_graph_fix.up.sql
  - migrations/sqlite/checksums.json
covered_digest: "v3:sha256:236e3fb790e1bd24aa803e9740d5d1ab87051d95ff6a6e7e4ebf1aadf0980b58"
behavior_unverified: 0
overrides_applied: 0
re_verification:
  previous_status: passed
  previous_score: 11/11
  gaps_closed: []
  gaps_remaining: []
  regressions: []
advisory:
  - finding: "`next_command()` in `crates/quran-graph/src/error.rs` still emits `qai graph …` for five of six graph errors; the real tree is `qai quran graph …`, so suggested commands fail if pasted. The unknown-projection snapshot (`graph_s2.trycmd:77`) inherits `Next: qai graph build --help`."
    category: other
    reason: "Carried forward and re-confirmed at HEAD (`error.rs:219-228`). Copy/UX defect: no failing test, does not block graph navigation. `crates/quran-graph/src/error.rs` is outside this report's covered set."
    evidence_status: "none provided"
  - finding: "`dispute` review operation is service-only with no CLI verb (accept/reject/correct cover the CLI)."
    category: other
    reason: "Carried forward and re-confirmed (`quran_cli.rs:980` accepts a disputed suggestion; no `dispute` verb in `crates/cli/src/quran.rs`). 04-02-PLAN did not require a dispute verb — polish/parity, not a plan must-have."
    evidence_status: "none provided"
  - finding: "Static DOT/SVG export renders raw node/edge sets, bypassing the tombstone policy filter that the authoritative Graph JSON path applies (WR-01)."
    category: security
    reason: "Carried forward; dispositioned accepted (D-13 local-operator static rendering). The authoritative JSON envelope is policy-filtered with byte-absence proven by `graph_export::restricted_and_tombstoned_ids_are_absent_from_bytes`. No failing test demonstrates impact on a success criterion."
    evidence_status: "none provided"
  - finding: "HTTP path-mode requests accept but ignore caller-supplied `edge_types`/`direction` (WR-03)."
    category: other
    reason: "Carried forward; dispositioned accepted. All surfaces share one service and report the effective filter in `Explanation.applied_filters`, so cross-surface parity is not broken. A typed rejection is a contract change, not a bug fix."
    evidence_status: "none provided"
  - finding: "Shallow doctor tombstone probe can report `Pass` after a capped source sample (`SHALLOW_TOMBSTONE_SRC_CAP`, `quran_graph_doctor.rs:643-659` truncates `srcs` without a bounded marker, unlike the dangling scan's `dangling_bounded`)."
    category: other
    reason: "Carried forward and re-confirmed; `--deep` upgrades to a full probe. No reproducible failure artifact; the dangling scan does surface boundedness."
    evidence_status: "none provided"
  - finding: "SVG output has `role=\"img\"` + `<desc>` but no `<title>` element (`quran_graph_export.rs:238`)."
    category: other
    reason: "Carried forward; minor a11y note. One-line addition, no functional impact."
    evidence_status: "none provided"
  - finding: "`qai quran graph export --db --projection <family>` is wired (opens via `GraphApiService::open_for_projection` through `graph_projection`, `quran_cli.rs:4448-4450`) but no test exports a non-default family; selector coverage sits on `neighbors` and the structural export render."
    category: other
    reason: "Re-confirmed at HEAD: no `projection`/annotated-family case in `graph_export.rs` or `graph_s*.trycmd`. The export selector resolves through the same one resolution point tested for neighbors and the pre-serialization filter is unchanged, so no failure is reproducible — a coverage gap, not a defect."
    evidence_status: "none provided"
---

# Phase 04: Quran Graph Verification Report

**Phase Goal:** Users can explore structural and linguistic relationships of the Quran as a navigable graph.
**Verified:** 2026-11-15T15:12:07Z
**Status:** passed
**Re-verification:** Yes — refresh re-verification. The prior report (`2026-10-16T19:22:46Z`, `passed` 11/11) was fingerprint-stale because phases 05–13 subsequently edited covered inputs (`crates/server/src/api.rs`, `crates/application/src/quran_cli.rs`, `crates/tool-registry/src/lib.rs`, `crates/quran-graph/src/*`). This run re-derived must-haves from the ROADMAP success criteria plus all six PLAN frontmatters, re-checked every truth against HEAD (`67b47f6`), and recomputed the covered-input fingerprint through the CLI. Covered set is unchanged from the prior report; no Phase-05+ artifact is claimed.

**Evidence basis.** Every graph suite result below comes from one `cargo test --workspace --no-fail-fast` run at HEAD (`/tmp/qai-cargo-test.log`: 203 suites, 0 failures; only the standalone 50k `soak_full` soak was stopped by the operator and is re-running separately). This verifier re-ran `arch-check` and `migrate-check` in its own process and re-read every claimed wiring site and test body in the current tree.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | SC1 — User can open a neighbor view around any verse, word, root, or concept | ✓ VERIFIED | Selector exists and resolves in one place: `ReadOptions.projection` (`quran_graph_api.rs:164`, default `None` = structural) → `FileGraphBackend::service(projection)` (`:1203-1206`, `unwrap_or(STRUCTURAL_PROJECTION_ID)`) → `GraphApiService::open_for_projection` (`:740-747`), used by every request-scoped read (`:1225-1246`). `concept_neighborhood_is_reachable_only_through_the_annotated_projection` (`graph_neighbors.rs:204`) green in the HEAD run: structural read of `concept:mercy` is a typed `NodeNotFound`, annotated read is non-empty with `projection_id == quran-annotated-v1`, every edge carries the seed `attribution_assertion_id`, every Concept/Entity node has `reviewed_by == "pending-scholar"`, and service/file-backend/tool/CLI payloads are identical after canonicalization; unknown family is typed `UnknownProjection`. Verse/word/root legs remain green (`structural_build_inspect_neighbors_canonical_quotation`, `root_family_carries_dataset_attribution_or_typed_unavailability`). |
| 2 | SC2 — User can find paths between two graph nodes with each edge's provenance shown | ✓ VERIFIED | `graph_paths` 11/11 green in the HEAD run (reachability, shortest, up-to-K with stable tie-break, per-edge structural-vs-assertion provenance, parallel asserted edges keep own provenance, K-beyond-max pre-flight error, truncated searches refuse no-path claims, pattern-allowlist rejection before I/O, root-family attribution or typed unavailability). CLI `Path --mode` pinned by `graph_s2.trycmd`; 5 HTTP routes registered at `api.rs:2274-2278`; `quran.graph_path` tool green. |
| 3 | SC3 — User can export a subgraph with edge provenance intact | ✓ VERIFIED | `graph_export` 5/5 green (assertions travel with kept edges, restricted/tombstoned ids byte-absent, truncation notice carries reason, document assembles in one step, DOT/SVG identities + banner). `quran_graph_export.rs` composes `visible_export_sets` → `export_json_with_notice` before serialization. Export `--projection` opens through `open_for_projection` (`quran_cli.rs:4448-4450`) and reuses the unchanged filtered render path. |
| 4 | SC4 — Graph queries explain why each result was returned | ✓ VERIFIED | `graph_explain` 4/4 green (canonicalized CLI≈HTTP≈tools parity on neighbors, all path modes, subgraph+pattern, and unanimous truncated-plus-reason). `Explanation` carries start/end, ordered path, edge types, per-edge provenance, confidence, applied filters, snapshot identity, completion, duration. |
| 5 | Structural projection builds from the active edition into SQLite and its manifest inspects with a generation stamp | ✓ VERIFIED | `structural_build_inspect_neighbors_canonical_quotation` (graph_neighbors), `doctor_manifest_current_passes_on_fresh_build` and `doctor_manifest_mismatch_warns_after_generation_bump` (graph_doctor) green; CLI `inspect` snapshot prints `projection … (build …, generation …, status active)` (`quran_cli.rs:3634`); `migrations/sqlite/0022` present and `migrate-check` OK (44 migrations ordered, checksums stable — re-run by this verifier). |
| 6 | SQLite adapter shows the same bounded/authorized/deterministic behavior as the reference backend | ✓ VERIFIED | `quran-graph/tests/conformance.rs` 20/20 green (both backends), including per-field pre-flight rejection, fanout truncation reason, node/edge exhaustion incomplete, cycle termination, cancellation flag, restricted intermediates absent, insertion-order determinism, `NodeNotFound` vs truncated-empty; lib 22/22 and no-query-language 2/2 green. `arch-check` OK — no forbidden edge (re-run by this verifier). |
| 7 | Same node pair can carry multiple attributed assertions without constraint violation | ✓ VERIFIED | `one_triple_carries_multiple_attributed_assertions` (graph_neighbors) green; NULL-safe partial unique indexes in `0022_quran_graph_fix.up.sql` (sha256 recorded in `checksums.json:29`, stable under `migrate-check`). |
| 8 | Review lifecycle (propose/suggest/accept/reject/correct-with-supersede), tombstones hidden from traversal+export while retained, rebuilds never discard annotations/history | ✓ VERIFIED | `graph_review` 8/8 green (`suggest_then_accept_promotes_with_reviewer_and_timestamp`, `suggest_then_reject_tombstones_but_retains_audit`, `correct_inserts_new_row_with_supersedes_chain`, `supported_by_and_disputed_by_are_first_class_with_disputed_effective`, `empty_queue_and_all_tombstoned_report_complete_empty`, golden lifecycle shapes); `graph_build::rebuild_preserves_authority_and_review_history_ac_p4_03` green; tombstone byte-absence asserted in `graph_export`. |
| 9 | Word-root projection builds only from the active attributed dataset, typed unavailable otherwise | ✓ VERIFIED | `graph_build` 7/7 green: `wordroot_build_from_synthetic_dataset`, `od11_dataset_license_blocked_wordroot_requires_active_dataset`, `od12_linguist_blocked_annotated_seed_stays_mechanics_only`; `root_family_carries_dataset_attribution_or_typed_unavailability` + `QAI-MORPH-0004` → CLI 5 / HTTP 404 (`api.rs:1777`). |
| 10 | Every CLI read op has an HTTP route and a typed tool returning identical (canonicalized) results with projection-pinned reproducibility | ✓ VERIFIED | `server/tests/graph.rs` 11/11 green (5 enveloped POST routes, ETag, 422/404/403, truncated-as-200, no-mutation fence, `graph_neighbors_route_threads_projection_selector`); `graph_tools` 6/6 green (5 graph tools registered with version consts, envelopes attributed + service-identical, namespaced codes preserved, pending-suggestion warnings, non-empty research checksum); `openapi_spec_covers_every_route` green; OpenAPI has all 5 `/api/v1/quran/graph/*` paths. Route scope is exactly the five the plan specifies — `export` writes files/format output and is deliberately not an HTTP route. |
| 11 | Doctor is read-only (drift/dangling/tombstone checks, never mutates); repair/GC are explicit, confirmed and audited; owner gates and deferral ledger recorded | ✓ VERIFIED | `graph_doctor` 10/10 green (`doctor_run_leaves_state_untouched`, `repair_refuses_without_confirmation`, `repair_tombstone_gc_collects_only_old_tombstones`, quarantine/rebuild turn doctor green, empty DB → Skipped with build remedy); `phase-04-owner-gates.md` records OD-11/OD-12 🔴 BLOCKED with closing commands; `phase-04-deferrals.md` SC1 row (`:84`) names the selector + check and §7 (`:67-71`) records the intentionally structural-only reads. |

**Score:** 11/11 truths verified (0 behavior-unverified, 0 overrides)

### Plan-level Truth Coverage (24 plan truths → 11 roadmap truths)

| Plan | Plan truths | Mapped to | Status |
|------|-------------|-----------|--------|
| 04-01 | 4 (build+inspect, bounded neighbors + canonical quotation, SQLite parity, multi-assertion triple) | truths 5, 1, 6, 7 | ✓ VERIFIED |
| 04-02 | 4 (review lifecycle, suggest queue, tombstone/rebuild retention, word-root dataset gate) | truth 8, 8, 8, 9 | ✓ VERIFIED |
| 04-03 | 4 (paths with provenance, bounded subgraph/patterns, Graph JSON export, DOT/SVG static render) | truths 2, 2, 3, 3 | ✓ VERIFIED |
| 04-04 | 4 (HTTP route, typed tool + checksum, cross-surface parity, explainability) | truth 10 (all four) | ✓ VERIFIED |
| 04-05 | 4 (read-only doctor, confirmed repair/GC, owner-gate ledger, evidence matrix) | truth 11 (all four) | ✓ VERIFIED |
| 04-06 | 5 gap-closure truths (G1–G5) | truths 1, 10, 8/1, 3, 11 | ✓ VERIFIED |

**Gap-closure (G-04-SC1) detail:** G1 concept/entity neighbor reachable on CLI/HTTP/tool with structural default ✓ · G2 omission reads structural, unknown family is `QAI-GRAPH-0001` (CLI exit 5 per `graph_s2.trycmd:74-76`, HTTP 404 per `api.rs:1772`), never empty success ✓ · G3 reference-plus-hash + `pending-scholar` + seed assertion id under any selector ✓ · G4 export selector wired through the same filtered path ✓ (behavior for a non-default family untested — `advisory:`) · G5 ledger row names selector + check + structural-only boundary ✓.

### Prohibitions (must-NOT)

| Prohibition | Tier | Status | Evidence |
|-------------|------|--------|----------|
| Suggested/computed edges never presented as verified; unverified suggestions stay labeled until a recorded human decision | judgment | ✓ VERIFIED | `graph_suggestion_results_are_labeled_pending_in_warnings`; suggest→accept/reject lifecycle tests; `reviewed_by == "pending-scholar"` asserted on annotated nodes |
| Disputed positions never presented as consensus; disagreement shown with attribution | judgment | ✓ VERIFIED | `supported_by_and_disputed_by_are_first_class_with_disputed_effective`; every edge carries `assertion_id`/provenance |
| Truncated/partial never rendered as absence or negative claim; every partial carries `truncated` + non-empty reason | judgment | ✓ VERIFIED | `truncated_exports_carry_the_incomplete_notice`, `truncated_searches_refuse_no_path_claims`, `graph_parity_truncated_reports_unanimously`, `graph_truncated_results_are_200_with_reason` |
| Selector never widens visibility (same `AuthzScope` + allowlist pre-serialization for every family) | judgment | ✓ VERIFIED | conformance restricted-intermediate/cancellation tests; `restricted_and_tombstoned_ids_are_absent_from_bytes`; no build-path change in 04-06 |
| Pending mechanics-only seed nodes stay `pending-scholar`, never verified scholarship | judgment | ✓ VERIFIED | concept test leg (b) asserts `pending-scholar` on every Concept/Entity node |
| A selector never changes the default read scope | judgment | ✓ VERIFIED | concept test leg (a) typed `NodeNotFound`; explicit `--projection quran-structural-v1` snapshot byte-identical (8 nodes / 7 edges, `graph_s2.trycmd:65-70`) |

No unverified prohibitions.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `migrations/sqlite/0022_quran_graph_fix.up.sql` | multi-assertion key, disputed CHECK, authority scope | ✓ VERIFIED | present, checksum-tracked; `migrate-check` OK |
| `crates/application/src/quran_graph_store.rs` | `SqliteGraphStore` adapter | ✓ VERIFIED | snapshot-on-open; unknown active row → typed `UnknownProjection` (`:199`) |
| `crates/application/src/quran_graph_build.rs` | structural + word-root + annotated builders, fenced publish | ✓ VERIFIED | real SQL inserts; `rebuild_preserves_authority_and_review_history_ac_p4_03` green |
| `crates/application/src/quran_graph_annotations.rs` | review lifecycle service | ✓ VERIFIED | layer `B`/`D` `ProvenanceLayer` (`model.rs:205-250`); assertion+provenance+audit+outbox in one UnitOfWork |
| `crates/application/src/quran_graph_api.rs` | read services + `Explanation` + `ReadOptions.projection` | ✓ VERIFIED | `projection: Option<String>` (`:164`); `service(projection)` (`:1203`) |
| `crates/application/src/quran_graph_export.rs` | export + DOT/SVG rendering | ✓ VERIFIED | `visible_export_sets` → `export_json_with_notice` pre-serialization |
| `crates/application/src/quran_graph_tools.rs` | typed agent tools | ✓ VERIFIED | `options.projection = params.projection` (`:268`) |
| `crates/application/src/quran_graph_doctor.rs` | read-only checks + confirmed repair/GC | ✓ VERIFIED | 10/10 green; shallow caps documented (`:53-55`) |
| `crates/quran-graph/src/{model,traverse,mem,export,structural}.rs` | port contract + reference traversal | ✓ VERIFIED | conformance 20/20 on both backends |
| `crates/application/tests/graph_*.rs` (8 suites) | integration/contract suites | ✓ VERIFIED | 54 tests green at HEAD: build 7, doctor 10, explain 4, export 5, neighbors 3, paths 11, review 8, tools 6 |
| `crates/server/tests/graph.rs` | HTTP contract tests | ✓ VERIFIED | 11/11 green incl. no-mutation fence + selector threading |
| `crates/tool-registry/tests/conformance.rs` | tool-registry conformance | ✓ VERIFIED | 10/10 green |
| `crates/cli/tests/quran/graph_s1.trycmd` + `graph_s2.trycmd` | CLI snapshots | ✓ VERIFIED | `quran_graph_snapshots` green; s2 carries explicit-structural + unknown-family segments |
| `docs/08-api/quran-v1-openapi.json` | 5 graph paths + neighbors `projection` | ✓ VERIFIED | all 5 paths present; `openapi_spec_covers_every_route` green |
| `docs/05-followups/phase-04-owner-gates.md` | BLOCKED owner gates | ✓ VERIFIED | OD-11/OD-12 🔴, P4-X01/X02/X03/X05 recorded with closing commands |
| `docs/05-followups/phase-04-deferrals.md` | deferral ledger + D-01 matrix | ✓ VERIFIED | SC1 row corrected (`:84`); §7 structural-only boundary (`:67-71`); GraphML deferred |
| `fixtures/quran/graph/{concept-seed-v1,annotation-goldens}.json` | seed fixtures | ✓ VERIFIED | versioned, synthetic-labeled, BLOCKED headers, exercised by build/review/conformance |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `quran_graph_store.rs` | `quran-graph` traversal | snapshot-on-open + reference expansion semantics | ✓ WIRED | conformance 20×2 green; `arch-check` clean |
| build publish | active pointer | single-transaction fenced flip + generation stamp | ✓ WIRED | manifest generation asserted in graph_neighbors/cli inspect |
| neighbor ayah hits | canonical reader | `verify_canonical_quotation` per hit | ✓ WIRED | byte-identical asserts green |
| annotation writes | authority + audit | assertion+provenance+audit+outbox in one UnitOfWork | ✓ WIRED | `graph_review` chain-validity green |
| export assembly | policy filter | `visible_export_sets` + retain/allowlist before serialization | ✓ WIRED | two byte-absence tests green |
| HTTP handlers | read services | thin over `quran_graph_api`; 422/404/403/200+truncated | ✓ WIRED | 5 routes (`api.rs:2274-2278`), 11/11 green |
| tools | read services | `GraphToolBackend`; `BackendMeta` pins projection+builder+generation | ✓ WIRED | `graph_tools` 6/6 green |
| request selector | read service | `ReadOptions.projection` → `FileGraphBackend::service` → `open_for_projection` | ✓ WIRED | single resolution point serves CLI/HTTP/tool (`:1203`, `api.rs:1880`, `tools:268`, `quran_cli.rs:3737`) |
| CLI `--projection` | typed USAGE | `--projection` with `--file` rejected | ✓ WIRED | `quran_cli.rs:3667-3668` (neighbors), `:4397-4398` (export) |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| neighbors CLI `--db` | ayah hit text | canonical reader via `verify_canonical_quotation` | Yes | ✓ FLOWING |
| export bytes | nodes/edges/assertions | `SqliteGraphStore::export_sets` → `visible_export_sets` → `export_json_with_notice` | Yes | ✓ FLOWING |
| HTTP graph responses | result payloads | `FileGraphBackend` opens the requested family per request | Yes | ✓ FLOWING |
| tool results | payloads | `GraphToolBackend` over the live read trait | Yes | ✓ FLOWING |
| word-root edges | morphology analyses | active attributed dataset via the D-03 gate | Yes | ✓ FLOWING |
| root-family ranked ayahs | morphology occurrences | `quran_morphology::root_search` (lexicon-gated, no projection build) | Yes | ✓ FLOWING |
| concept/entity nodes | concept neighborhood | `quran-annotated-v1` read via the selector, asserted non-empty with seed attribution | Yes | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| concept neighborhood reachable only via annotated selector (4 surfaces + unknown family) | `cargo test -p application --test graph_neighbors` (HEAD workspace run) | 3 pass, 0 failed | ✓ PASS |
| application graph suites (8 targets) | workspace run, graph_* targets | 54 pass, 0 failed | ✓ PASS |
| quran-graph crate (lib + conformance + no-query-language) | workspace run | 22 + 20 + 2 pass, 0 failed | ✓ PASS |
| server graph contract (incl. selector route) | workspace run, `tests/graph.rs` | 11 pass, 0 failed | ✓ PASS |
| tool-registry (lib + conformance) | workspace run | 10 conformance pass, 0 failed | ✓ PASS |
| CLI graph snapshots | workspace run, `tests/quran.rs` | `quran_graph_snapshots` ok | ✓ PASS |
| OpenAPI ↔ routes | workspace run, `server/tests/api.rs` | `openapi_spec_covers_every_route` ok | ✓ PASS |
| architecture gate | `cargo run -q -p xtask -- arch-check` | OK — no forbidden dependency edges | ✓ PASS (re-run by verifier) |
| migration gate | `cargo run -q -p xtask -- migrate-check` | OK — 44 migrations ordered; checksums stable | ✓ PASS (re-run by verifier) |

### Probe Execution

No phase-declared or conventional `scripts/*/tests/probe-*.sh` probes exist for this phase. Skipped.

### Test Quality Audit

| Test File | Linked Req | Active | Skipped | Circular | Assertion Level | Verdict |
|-----------|-----------|--------|---------|----------|-----------------|---------|
| `graph_neighbors.rs` | REQ-quran-graph | 3 | 0 | none | Behavioral (build→inspect→neighbors→quotation; concept reachability across 4 surfaces + typed failures) | ✓ PASS |
| `graph_review.rs` | REQ-quran-graph, REQ-data-separation-layers | 8 | 0 | none | Behavioral (state machine, layer B/D, tombstone byte-absence) | ✓ PASS |
| `graph_build.rs` | REQ-quran-graph | 7 | 0 | none | Behavioral (rebuild preservation, typed dataset/linguist gates) | ✓ PASS |
| `graph_paths.rs` | REQ-quran-graph | 11 | 0 | none | Value/Behavioral (provenance + truncation semantics) | ✓ PASS |
| `graph_export.rs` | REQ-quran-graph | 5 | 0 | none | Behavioral (policy byte-absence) | ✓ PASS |
| `graph_tools.rs` | REQ-quran-graph, REQ-cli-api | 6 | 0 | none | Value (envelope fields + research checksum) | ✓ PASS |
| `graph_explain.rs` | REQ-quran-graph, REQ-cli-api | 4 | 0 | none | Behavioral (3-surface parity) | ✓ PASS |
| `graph_doctor.rs` | REQ-quran-graph | 10 | 0 | none | Behavioral (immutability before/after) | ✓ PASS |
| `server/tests/graph.rs` | REQ-quran-graph, REQ-cli-api | 11 | 0 | none | Status/Value (route contract + fence + selector) | ✓ PASS |
| `quran-graph/tests/conformance.rs` | REQ-quran-graph | 20 | 0 | none | Behavioral (both backends) | ✓ PASS |

**Disabled tests on requirements:** 0. **Circular patterns detected:** 0. **Insufficient assertions:** 0.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| REQ-quran-graph | 04-01…04-06 (all six declare it) | First-class graph representation independent of vector search: node/edge types, edge provenance, graph search, explainability, semi-automated annotation (PRD §10) | ✓ SATISFIED | 19-predicate vocabulary + multi-assertion authority; provenance on every edge; search (neighbors/paths/subgraph/pattern/root-family) on CLI+HTTP+tools with parity proof; `Explanation` on every result; suggest→human-decide lifecycle with pending labeling; tombstone/retention semantics; doctor+repair operability; read-scope selector reaching the annotated projection; owner gates honestly BLOCKED. All four ROADMAP success criteria met. `REQUIREMENTS.md:30` checked, `:106` Phase 4 `Complete`. |
| REQ-data-separation-layers | Phase-2 owner; extended here (no Phase-4 plan claims it) | Trust layers A (canonical) / B (publisher metadata) / C (scholarly) / D (computational) / E (user+AI notes) | ✓ SATISFIED (extended, no regression) | Graph assertions carry typed `ProvenanceLayer::{B,D}` (`quran-graph/src/model.rs:205-250`; `quran_graph_annotations.rs:287-321` rejects unknown layer strings), corrective rows inherit the old layer (`:133`), and canonical text never comes from graph records (canonical-reader quotation asserts). `REQUIREMENTS.md:19` checked, owned by Phase 2 (`:102`). |
| REQ-cli-api | Phase-1 owner; extended here (no Phase-4 plan claims it) | Executable `qai`; versioned API; cancellable long-running operations | ✓ SATISFIED (extended, no regression) | Graph verbs ship as `qai quran graph …` on the real CLI (`crates/cli/src/quran.rs:740-1060`); 5 versioned POST routes under `/api/v1/quran/graph` (`api.rs:2274-2278`) with Envelope/ETag and OpenAPI in the same change; cancellation threading proven in conformance (`resolve_node/neighbors` take `&cancel`, cancel-flag honored) and budget/timeout pre-flight is typed. `REQUIREMENTS.md:51` checked, owned by Phase 1 (`:99`). |

**Orphaned requirements:** none. `REQUIREMENTS.md` maps exactly `REQ-quran-graph` to Phase 4 (`:106`, `Complete`; checkbox `:30` checked); the other two phase IDs belong to Phases 1/2 and are marked Complete there — Phase 4 only extends them, and both extensions are exercised by green suites. No other ID appears in any Phase-4 plan frontmatter.

### Decision Coverage

All trackable CONTEXT.md decisions are honored by shipped artifacts (14/14; `not_honored: []`). The projection selector is intentionally limited to `neighbors`/`export` (04-06 scope, recorded in `phase-04-deferrals.md` §7); `path`/`subgraph`/`pattern`/`inspect` remain structural-only by explicit decision, not omission.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| all 8 `quran_graph_*.rs`, `quran-graph/src/*.rs`, `quran_cli.rs`, `cli/src/quran.rs`, `server/src/api.rs`, `tool-registry/src/lib.rs` | — | `TODO/FIXME/XXX/TBD/todo!/unimplemented!` | — | None found (clean — no debt markers, no blocker) |
| `quran-graph/src/error.rs` | 219-228 | Misleading `next_command()` (`qai graph …` for 5 of 6 errors) | ℹ️ Info | `advisory:` (outside covered set; no failing test) |
| `quran_graph_export.rs` | 238 | Static SVG lacks `<title>` | ℹ️ Info | `advisory:` a11y polish |
| static DOT/SVG | — | Bypasses tombstone policy filter (WR-01) | ℹ️ Info | Accepted D-13; authoritative JSON is filtered and byte-asserted |

### Human Verification Required

None. Every truth is exercised by automated suites green at HEAD, and this verifier re-ran the two release gates (`arch-check`, `migrate-check`) plus re-read every claimed wiring site and assertion in the current tree. Static DOT/SVG output is file-asserted (node/edge identity + truncation banner) with coordinates explicitly out of contract; no interactive GUI/TUI ships in this phase. The owner gates below are human-only by design and stand open —

**Standing owner gates (not defects, not new gaps):** OD-11 dataset/licensing 🔴, OD-12 linguist ratification 🔴, P4-X01/X02/X03/X05 ratifications outstanding, recorded with exact closing commands in `docs/05-followups/phase-04-owner-gates.md`. Phase 4 is *engineered*, not a licensed-dataset, linguist-ratified, or ADR-ratified release; the word-root projection is proven behaviorally on the synthetic `synthetic_test_only` lexicon only.

### Gaps Summary

No gaps. All 11 truths, 24 plan truths, and 6 prohibitions are verified against HEAD with named green suites, not SUMMARY claims. The previously-recorded G-04-SC1 closure holds in the current tree: one selector field (`ReadOptions.projection`), one resolution point (`FileGraphBackend::service` → `open_for_projection`), three surfaces wired (CLI `--projection` incl. `--file` USAGE guard; HTTP body `projection`; tool param), a typed `UnknownProjection` (`QAI-GRAPH-0001` → CLI exit 5, HTTP 404) instead of empty success, and a behavioral test proving structural-default `NodeNotFound` vs annotated reachability with `pending-scholar` attribution. Downstream phases (05–13) touched covered inputs (`api.rs`, `quran_cli.rs`, `tool-registry`, `crates/quran-graph`), and the full workspace run at HEAD shows no regression: 203 suites green, including every graph suite at its prior count (build 7, doctor 10, explain 4, export 5, neighbors 3, paths 11, review 8, tools 6, server 11, conformance 20, tool-registry conformance 10).

One coverage-only advisory remains (export selector behavior not directly exercised for a non-default family) plus the pre-existing accepted review/UI advisories; none is a reproducible defect and none blocks the goal.

---

_Verified: 2026-11-15T15:12:07Z_
_Verifier: the agent (gsd-verifier)_
