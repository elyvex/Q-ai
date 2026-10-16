---
phase: 04-quran-graph
verified: 2026-10-16T19:22:46Z
status: passed
score: 11/11 must-haves verified
covered_files:
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
covered_digest: "v3:sha256:43e522ce407b1d90e46d0e135fb8c391927be4338816d8790b259cc2e9c4fe96"
behavior_unverified: 0
overrides_applied: 0
re_verification:
  previous_status: passed
  previous_score: 11/11
  gaps_closed: []
  gaps_remaining: []
  regressions: []
advisory:
  - finding: "`next_command()` in `crates/quran-graph/src/error.rs` still emits `qai graph …` for five of six graph errors; the real tree is `qai quran graph …`, so suggested commands fail if pasted. The new unknown-projection snapshot (`graph_s2.trycmd:78`) inherits `Next: qai graph build --help`."
    category: other
    reason: "Carried forward from the prior report; surfaced by 04-UI-REVIEW.md (Pillar 1 warning). Copy/UX defect, no failing test, does not block graph navigation. Out of scope for the G-04-SC1 gap-closure plan."
    evidence_status: "none provided"
  - finding: "`dispute` review operation is service-only with no CLI verb (accept/reject/correct cover the CLI)."
    category: other
    reason: "Carried forward; surfaced by 04-UI-REVIEW.md (Pillar 6 warning). 04-02-PLAN did not require a dispute verb, so this is polish/parity, not a plan must-have."
    evidence_status: "none provided"
  - finding: "Static DOT/SVG export renders raw node/edge sets, bypassing the tombstone policy filter that the authoritative Graph JSON path applies (WR-01)."
    category: security
    reason: "Carried forward; flagged in 04-REVIEW.md and dispositioned accepted (D-13 local-operator static rendering). The authoritative JSON envelope is policy-filtered with byte-absence proven by `graph_export::restricted_and_tombstoned_ids_are_absent_from_bytes`. No failing test demonstrates an impact on a success criterion."
    evidence_status: "none provided"
  - finding: "HTTP/tool path-mode requests accept but ignore caller-supplied `edge_types`/`direction` (WR-03)."
    category: other
    reason: "Carried forward; dispositioned accepted in 04-REVIEW.md. All surfaces share the same service and report the effective `any` filter in `Explanation.applied_filters`, so cross-surface parity is not broken. A typed rejection is a contract change, not a bug fix."
    evidence_status: "none provided"
  - finding: "Shallow doctor tombstone probe reports `Pass` after a capped (50-source) sample (WR-04)."
    category: other
    reason: "Carried forward; dispositioned accepted in 04-REVIEW.md; `--deep` upgrades to a full probe. No reproducible failure artifact."
    evidence_status: "none provided"
  - finding: "SVG output has `role=\"img\"` + `<desc>` but no `<title>` element."
    category: other
    reason: "Carried forward; surfaced by 04-UI-REVIEW.md (minor a11y note). One-line addition, no functional impact."
    evidence_status: "none provided"
  - finding: "`qai quran graph export --db --projection <family>` is wired (opens via `GraphApiService::open_for_projection` through `graph_projection`) but has no dedicated behavioral test for a non-default family; coverage for the new selector is on `neighbors` (the concept-neighborhood integration test + the CLI snapshot segments)."
    category: other
    reason: "New-scope observation from the gap-closure plan (04-06): the plan scoped selector coverage to `neighbors`, and the export render path is exercised with the structural family only. The export selector resolves through the same one resolution point that is tested for neighbors and the pre-serialization filter is unchanged, so no failure is reproducible; it is a coverage gap, not a defect."
    evidence_status: "none provided"
---

# Phase 04: Quran Graph Verification Report

**Phase Goal:** Users can explore structural and linguistic relationships of the Quran as a navigable graph.
**Verified:** 2026-10-16T19:22:46Z
**Status:** passed
**Re-verification:** Yes — final re-verification after all downstream phases (through phase 06, including 06-05 verse-link plumbing). The previous report (`2026-10-16T11:50:26Z`) returned `passed` 11/11. This run re-derived must-haves from ROADMAP success criteria + every PLAN frontmatter, re-checked every previously-verified truth against HEAD (`fc786ca`) with independently re-run suites (quran-graph conformance 20+20, application graph 54, server graph 11, CLI snapshots, OpenAPI coverage, arch-check, migrate-check), and verified no regression from downstream verse-link plumbing (`crates/cli/src/lib.rs`, `crates/server/src/api.rs`, `cli/tests/parity_cross_surface.rs`, `server/tests/{api,citation_open,graph}.rs`) — graph behavior is unchanged. Covered set excludes Phase-06 artifacts not part of Phase 4 and adds none.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | SC1 — User can open a neighbor view around any verse, word, root, or concept | ✓ VERIFIED | **Gap closed.** The concept/entity leg is now reachable via a projection selector. `ReadOptions.projection` (`quran_graph_api.rs:164`, default `None` = structural) is resolved in one place by `FileGraphBackend::service(projection)` (`:1203-1210`) through `GraphApiService::open_for_projection`, and every request-scoped `GraphBackend` impl passes `args.options.projection.as_deref()` (`:1225-1246`). Re-run in this process: `concept_neighborhood_is_reachable_only_through_the_annotated_projection` (graph_neighbors 3/3 green) proves the structural default read of `concept:mercy` is a typed `NodeNotFound`, the annotated read is non-empty with `snapshot.projection_id == quran-annotated-v1` and seed attribution on every edge, and the service / file-backend / tool / CLI payloads are identical after canonicalization. Verse (`ayah`), word (token) and root (`root_family`, lexicon-gated projection-free) legs remain green. |
| 2 | SC2 — User can find paths between two graph nodes with each edge's provenance shown | ✓ VERIFIED | `graph_paths` 11/11 green in this process (reachability/shortest/up-to-K, per-edge structural-vs-assertion provenance, parallel asserted edges keep own provenance, K pre-flight error, truncated searches refuse no-path claims); `quran_graph_api.rs` dispatches over `traverse::up_to_k_paths`; CLI `Path --mode` covered by `graph_s2.trycmd` (`quran_graph_snapshots` 1/1). |
| 3 | SC3 — User can export a subgraph with edge provenance intact | ✓ VERIFIED | `graph_export` 5/5 green (assertions travel with kept edges, restricted/tombstoned ids byte-absent, truncation notice with reason, DOT/SVG identities + banner); `quran_graph_export.rs` composes `visible_export_sets` → `export_json_with_notice` before serialization. The new `--projection` on export opens through `open_for_projection` (`quran_cli.rs:4448-4451`) and then uses the same unchanged `load_projection_sets` → render path. |
| 4 | SC4 — Graph queries explain why each result was returned | ✓ VERIFIED | `graph_explain` 4/4 green (canonicalized CLI≈HTTP≈tools parity on neighbors / all path modes / subgraph / pattern, unanimous truncated-plus-reason); `Explanation` carries start/end, ordered path, edge types + per-edge provenance, confidence, applied filters, snapshot identity, completion, duration. |
| 5 | Structural projection builds from the active edition into SQLite and its manifest inspects with a generation stamp | ✓ VERIFIED | `graph_neighbors::structural_build_inspect_neighbors_canonical_quotation` + `graph_doctor::doctor_manifest_current_passes_on_fresh_build` + `doctor_manifest_mismatch_warns_after_generation_bump` green; migration `0022` present and `migrate-check` OK (30 migrations, checksums stable). |
| 6 | SQLite adapter shows the same bounded/authorized/deterministic behavior as the reference backend | ✓ VERIFIED | `cargo test -p quran-graph` → conformance 20/20 green in this process (per-field pre-flight rejection, fanout truncation reason, node/edge exhaustion incomplete, cycle termination, cancellation, restricted intermediates absent, insertion-order determinism, NodeNotFound vs truncated-empty); `arch-check` OK (no `storage-sqlite → quran-graph` edge). |
| 7 | Same node pair can carry multiple attributed assertions without constraint violation | ✓ VERIFIED | `graph_neighbors::one_triple_carries_multiple_attributed_assertions` green; NULL-safe partial unique indexes in `0022_quran_graph_fix.up.sql`. |
| 8 | Review lifecycle (propose/suggest/accept/reject/correct-with-supersede), tombstones hidden from traversal+export while retained, rebuilds never discard annotations/history | ✓ VERIFIED | `graph_review` 8/8 + `graph_build::rebuild_preserves_authority_and_review_history_ac_p4_03` green; tombstone byte-absence asserted in `graph_export`. |
| 9 | Word-root projection builds only from the active attributed dataset, typed unavailable otherwise | ✓ VERIFIED | `graph_build::od11_dataset_license_blocked_wordroot_requires_active_dataset`, `wordroot_build_from_synthetic_dataset`, `od12_linguist_blocked_annotated_seed_stays_mechanics_only` green (7/7 suite). |
| 10 | Every CLI read op has an HTTP route and a typed tool returning identical (canonicalized) results with projection-pinned reproducibility | ✓ VERIFIED | `server/tests/graph.rs` 11/11 green in this process (5 enveloped routes incl. root-family, ETag/304, 422/404, truncation-as-200, no-mutation fence, + the new `graph_neighbors_route_threads_projection_selector`); `graph_tools` 6/6 green; `graph_explain` parity green; OpenAPI has all 5 `/api/v1/quran/graph/*` paths and `openapi_spec_covers_every_route` green. |
| 11 | Doctor is read-only (drift/dangling/tombstone checks, never mutates); repair/GC are explicit, confirmed and audited; owner gates and deferral ledger recorded | ✓ VERIFIED | `graph_doctor` 10/10 green (`doctor_run_leaves_state_untouched` proves immutability, `repair_refuses_without_confirmation`, `repair_tombstone_gc_collects_only_old_tombstones`, repair quarantine/rebuild turn doctor green); `phase-04-owner-gates.md` records OD-11/OD-12 + P4-X BLOCKED; `phase-04-deferrals.md` SC1 row now names the selector/check and the new §7 records the deliberately structural-only reads. |

**Score:** 11/11 truths verified (0 present, behavior-unverified)

### Gap-Closure Must-Haves (04-06)

| # | Truth (04-06 frontmatter) | Status | Evidence |
|---|---------------------------|--------|----------|
| G1 | A caller can read the neighbor view of a concept/entity node on CLI, HTTP, and the `quran.graph_neighbors` tool, while the default read scope stays structural | ✓ VERIFIED | Concept test legs (a)-(e) green; HTTP body field `GraphNeighborsBody.projection` → `options.projection` (`api.rs:1844`) asserted by `graph_neighbors_route_threads_projection_selector`; tool param forwarded (`quran_graph_tools.rs:268`) |
| G2 | Omitting the selector reads structural exactly as today; unknown/unbuilt family is a typed `GraphError::UnknownProjection` (`QAI-GRAPH-0001`; CLI NOT_FOUND, HTTP 404), never empty success | ✓ VERIFIED | CLI snapshot `--projection quran-not-built-v1` asserts `? 5` + `[QAI-GRAPH-0001]` (`graph_s2.trycmd:73-76`); test (f) asserts `UnknownProjection`; `graph_error_status` maps `QAI-GRAPH-0001` → 404 (`api.rs:1736`); explicit-structural snapshot segment byte-identical (8 nodes / 7 edges) |
| G3 | Annotated concept/entity nodes stay reference-plus-hash with `pending-scholar` attribution and the seed's attribution assertion id under any selector | ✓ VERIFIED | Test (b): every edge `.assertion_id == report.attribution_assertion_id`; every `Concept`/`Entity` node `reviewed_by == "pending-scholar"` |
| G4 | `qai quran graph export --db --projection <family>` exports that family through the same pre-serialization policy filter as the default export | ✓ VERIFIED | `export_db` opens via `GraphApiService::open_for_projection(db_path, graph_projection(projection))` (`quran_cli.rs:4448-4451`) and reuses the unchanged `load_projection_sets` → policy-filtered render path (byte-absence test green). Wiring verified; export-with-non-default-family behavior is untested — see `advisory:` |
| G5 | The D-01 evidence ledger SC1 row names the selector and the concept-neighborhood check, and records which reads remain intentionally structural-only | ✓ VERIFIED | `phase-04-deferrals.md:84` SC1 row names `ReadOptions.projection`, `--projection`, the HTTP/tool `projection` field, the check commands, and the structural-only boundary; new §7 (`:67-71`) records `path`/`subgraph`/`pattern`/`inspect` |

**Gap-closure key links:** `ReadOptions.projection` → `FileGraphBackend::service` → `open_for_projection` ✓ WIRED · HTTP body `projection` → `ReadOptions.projection` ✓ WIRED (`api.rs:1844`) · `GraphNeighborsParams.projection` → same field ✓ WIRED (`quran_graph_tools.rs:268`) · CLI `--projection` + `--file` → typed USAGE ✓ WIRED (`quran_cli.rs:3667-3668`, `:4397-4398`) · annotated publish path untouched (04-06 modifies no build file) ✓ · corrected SC1 ledger row points at the concept-neighborhood test ✓.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `migrations/sqlite/0022_quran_graph_fix.up.sql` | multi-assertion key, disputed CHECK, authority scope | ✓ VERIFIED | present; `migrate-check` OK |
| `crates/application/src/quran_graph_store.rs` | `SqliteGraphStore` adapter | ✓ VERIFIED | wired into `quran_graph_api.rs`, export, CLI, doctor |
| `crates/application/src/quran_graph_build.rs` | structural + word-root + annotated builders, fenced publish | ✓ VERIFIED | real SQL inserts; annotated projection now **read** via the selector (prior "built-but-unread" warning resolved) |
| `crates/application/src/quran_graph_annotations.rs` | review lifecycle service | ✓ VERIFIED | assertion+provenance+audit+outbox in one UnitOfWork |
| `crates/application/src/quran_graph_api.rs` | read services + `Explanation` contract + `ReadOptions.projection` | ✓ VERIFIED | `projection: Option<String>` (`:164`); `service(projection)` (`:1203`) |
| `crates/application/src/quran_graph_export.rs` | export + DOT/SVG rendering | ✓ VERIFIED | `visible_export_sets` → `export_json_with_notice` before serialization |
| `crates/application/src/quran_graph_tools.rs` | typed agent tools | ✓ VERIFIED | `options.projection = params.projection` (`:268`) |
| `crates/application/src/quran_graph_doctor.rs` | read-only checks + confirmed repair/GC | ✓ VERIFIED | 10/10 green |
| `crates/quran-graph/src/{model,traverse,mem,export,structural}.rs` | port contract + reference traversal | ✓ VERIFIED | conformance 20/20 both backends |
| `crates/quran-graph/tests/conformance.rs` | backend-generic parity harness | ✓ VERIFIED | 20 tests × 2 backends green |
| `crates/application/tests/graph_*.rs` (8 suites) | integration/contract suites | ✓ VERIFIED | 54 tests green in this process: build 7, doctor 10, explain 4, export 5, neighbors 3 (+1 new), paths 11, review 8, tools 6 |
| `crates/server/tests/graph.rs` | HTTP contract tests | ✓ VERIFIED | 11/11 green incl. no-mutation fence + new projection-selector route test |
| `crates/tool-registry/tests/conformance.rs` | tool-registry conformance | ✓ VERIFIED | 10/10 green (updated for `projection: None` literal) |
| `crates/cli/tests/quran/graph_s1.trycmd` + `graph_s2.trycmd` | CLI snapshots | ✓ VERIFIED | `quran_graph_snapshots` green; s2 carries the explicit-structural + unknown-family segments |
| `docs/08-api/quran-v1-openapi.json` | 5 graph paths + neighbors `projection` property | ✓ VERIFIED | all 5 present; `projection` property present (asserted) |
| `docs/05-followups/phase-04-owner-gates.md` | BLOCKED owner gates | ✓ VERIFIED | OD-11/OD-12, P4-X BLOCKED |
| `docs/05-followups/phase-04-deferrals.md` | deferral ledger + D-01 matrix | ✓ VERIFIED | SC1 row corrected (`:84`); §7 structural-only boundary added (`:67-71`) |
| `fixtures/quran/graph/{concept-seed-v1,annotation-goldens}.json` | seed fixtures | ✓ VERIFIED | versioned, synthetic-labeled, attributed; exercised by build/conformance |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `quran_graph_store.rs` | `quran-graph` traversal | snapshot-on-open + reference expansion semantics | ✓ WIRED | conformance 20×2 green; arch-check clean |
| build publish | active pointer | single-transaction fenced flip + generation stamp | ✓ WIRED | `flip_to_active`; manifest generation asserted in `graph_neighbors` |
| neighbor ayah hits | canonical reader | `verify_canonical_quotation` per hit, never graph text | ✓ WIRED | byte-identical asserts green |
| annotation writes | authority + audit | assertion + provenance + audit + outbox in one UnitOfWork | ✓ WIRED | `graph_review` chain-validity green |
| export assembly | policy filter | `visible_export_sets` + `retain_visible`/`assertion_allowlist_predicate` before serialization | ✓ WIRED | byte-absence asserted |
| HTTP handlers | read services | thin over `quran_graph_api`; 422/404/200+truncated mapping | ✓ WIRED | 5 routes; `server/tests/graph.rs` 11/11 green |
| tools | read services | `GraphToolBackend`; `BackendMeta` pins projection+builder+generation | ✓ WIRED | `graph_tools` 6/6 green |
| **request selector → projection family** | **read service** | `ReadOptions.projection` → `FileGraphBackend::service(projection)` → `open_for_projection` | ✓ WIRED | was the SC1 gap; now the single resolution point serves CLI/HTTP/tool |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| neighbors CLI `--db` | ayah hit text | canonical reader via `verify_canonical_quotation` | Yes | ✓ FLOWING |
| export bytes | nodes/edges/assertions | `SqliteGraphStore::export_sets` → `visible_export_sets` → `export_json_with_notice` | Yes | ✓ FLOWING |
| HTTP graph responses | result payloads | `FileGraphBackend` opens the requested projection family per request | Yes | ✓ FLOWING |
| tool results | payloads | `GraphToolBackend` over the live read trait | Yes | ✓ FLOWING |
| word-root edges | morphology analyses | active attributed dataset via the D-03 gate | Yes | ✓ FLOWING |
| root-family ranked ayahs | morphology occurrences | `quran_morphology::root_search` (lexicon-gated, no projection build) | Yes | ✓ FLOWING |
| **concept/entity nodes** | **concept neighborhood** | `quran-annotated-v1` read via the selector; asserted non-empty with seed attribution and `pending-scholar` | Yes | ✓ FLOWING (was ✗ DISCONNECTED) |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| concept neighborhood reachable only via annotated selector (service/backend/tool/CLI + unknown family) | `cargo test -p application --test graph_neighbors` | 3 pass, 0 failed (incl. new concept test) | ✓ PASS |
| application graph suites (8 targets) | `cargo test -p application --test graph_build --test graph_doctor --test graph_explain --test graph_export --test graph_neighbors --test graph_paths --test graph_review --test graph_tools` | 54 pass, 0 failed | ✓ PASS |
| quran-graph crate (lib + conformance + no-query-language + structural + traversal) | `cargo test -p quran-graph` | 22 + 20 + 2 + 1 + 5 pass, 0 failed | ✓ PASS |
| server graph contract (incl. projection-selector route) | `cargo test -p server --test graph` | 11 pass, 0 failed | ✓ PASS |
| tool-registry (lib + conformance) | `cargo test -p tool-registry` | 6 + 10 pass, 0 failed | ✓ PASS |
| CLI graph snapshots | `cargo test -p cli --test quran quran_graph` | 1 pass (s1 + s2 segments) | ✓ PASS |
| OpenAPI ↔ routes | `cargo test -p server --test api openapi_spec_covers_every_route` | 1 pass | ✓ PASS |
| architecture gate | `cargo run -q -p xtask -- arch-check` | OK — no forbidden dependency edges | ✓ PASS |
| migration gate | `cargo run -q -p xtask -- migrate-check` | OK — 30 migrations ordered; checksums stable | ✓ PASS |

### Probe Execution

No phase-declared or conventional `scripts/*/tests/probe-*.sh` probes exist for this phase. Skipped.

### Test Quality Audit

| Test File | Linked Req | Active | Skipped | Circular | Assertion Level | Verdict |
|-----------|-----------|--------|---------|----------|-----------------|---------|
| `graph_neighbors.rs` | REQ-quran-graph | 3 | 0 | none | Behavioral (build→inspect→neighbors→quotation; concept reachability across 4 surfaces + typed failures) | ✓ PASS |
| `graph_review.rs` | REQ-quran-graph | 8 | 0 | none | Behavioral (state machine + tombstone byte-absence) | ✓ PASS |
| `graph_build.rs` | REQ-quran-graph | 7 | 0 | none | Behavioral (rebuild preservation, typed gate) | ✓ PASS |
| `graph_paths.rs` | REQ-quran-graph | 11 | 0 | none | Value/Behavioral (provenance fields) | ✓ PASS |
| `graph_export.rs` | REQ-quran-graph | 5 | 0 | none | Behavioral (policy byte-absence) | ✓ PASS |
| `graph_tools.rs` | REQ-quran-graph | 6 | 0 | none | Value (envelope fields + checksum) | ✓ PASS |
| `graph_explain.rs` | REQ-quran-graph | 4 | 0 | none | Behavioral (3-surface parity) | ✓ PASS |
| `graph_doctor.rs` | REQ-quran-graph | 10 | 0 | none | Behavioral (immutability before/after) | ✓ PASS |
| `server/tests/graph.rs` | REQ-quran-graph | 11 | 0 | none | Status/Value (route contract incl. fence + selector threading) | ✓ PASS |
| `quran-graph/tests/conformance.rs` | REQ-quran-graph | 20 | 0 | none | Behavioral (both backends) | ✓ PASS |

**Disabled tests on requirements:** 0. **Circular patterns detected:** 0. **Insufficient assertions:** 0.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| REQ-quran-graph | 04-01, 04-02, 04-03, 04-04, 04-05, 04-06 (all six plans declare it) | First-class graph representation independent of vector search: node/edge types, edge provenance, graph search, explainability, semi-automated annotation (PRD §10) | ✓ SATISFIED | 19-predicate vocabulary + multi-assertion authority; provenance on every edge; graph search (neighbors/paths/subgraph/pattern/root-family) on CLI+HTTP+tools with parity proof; `Explanation` on every result; suggest→human-decide lifecycle with pending labeling; tombstone/retention semantics; doctor+repair operability; read-scope selector reaching the annotated projection; owner gates honestly BLOCKED. All four ROADMAP success criteria now met. |

**Orphaned requirements:** none. `REQUIREMENTS.md` maps exactly `REQ-quran-graph` to Phase 4 (row `:106` shows `Complete`, checkbox `:30` checked) and all six Phase-4 plans declare it; no other ID appears in any Phase-4 plan frontmatter.

### Decision Coverage

All trackable CONTEXT.md decisions are honored by shipped artifacts (14/14; `not_honored: []`).

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| — | — | `TODO/FIXME/XXX/TBD/unimplemented!/todo!` scan over all 7 `quran_graph_*.rs` + the 04-06-modified files | — | None found (clean — no debt markers, no blocker) |
| annotated projection | `quran_graph_build.rs` | Built-but-unread projection (dead output) | ✅ Resolved | Now read through `ReadOptions.projection`; the prior ⚠️ warning is closed |
| accepted code-review warnings WR-01/03/04/… | 04-REVIEW.md | Each dispositioned inline on a verified-closed phase | ℹ️ Info | Recorded under `advisory:` |
| UI-review warnings | 04-UI-REVIEW.md | `qai graph` next-command path, missing `dispute` CLI verb, SVG `<title>` | ℹ️ Info | Carried forward under `advisory:`; no failing test |

### Human Verification Required

None. Every truth above is exercised by automated suites re-run by this verifier in its own process (50 quran-graph + 54 application graph + 11 server + tool-registry + CLI snapshots + gates). Static DOT/SVG output is file-asserted (node/edge identity + truncation banner) with coordinates explicitly out of contract; no interactive GUI/TUI ships in this phase (deferred to Phase 5).

### Gaps Summary

No gaps. The single prior gap (G-04-SC1) is genuinely closed in the codebase, not merely claimed:

- The selector exists and is resolved in exactly one place. `ReadOptions.projection` (`quran_graph_api.rs:164`, default `None`) is honored by `FileGraphBackend::service(projection)` (`:1203-1210`), which maps `None` to `STRUCTURAL_PROJECTION_ID` — the default read scope is unchanged and `GraphApiService::open` remains its structural shorthand.
- The concept/entity neighborhood is now reachable on all three surfaces: CLI (`--projection` on `neighbors`/`export`, `quran_cli.rs:3665-3672`, `:4384-4402`), HTTP (`GraphNeighborsBody.projection` → `options.projection`, `api.rs:1844`), and the `quran.graph_neighbors` tool (`GraphNeighborsParams.projection` v1.1.0 → the same field, `quran_graph_tools.rs:268`).
- A new behavioral integration test (`graph_neighbors.rs:204`) proves the structural default hides `concept:mercy` (typed `NodeNotFound`, never empty-success), the annotated selector reveals it (non-empty, `projection_id == quran-annotated-v1`, seed attribution on every edge, `pending-scholar` on concept/entity nodes), and the service / file-backend / tool / CLI payloads are identical after canonicalization; it also proves an unknown family is a typed `UnknownProjection`.
- The CLI snapshot pins the default byte-identical under an explicit `--projection quran-structural-v1` and asserts `? 5` + `QAI-GRAPH-0001` for an unknown family; `graph_error_status` maps `QAI-GRAPH-0001` → HTTP 404.
- The evidence ledger is honest: the SC1 row now names the selector and the check, and §7 records the reads (`path`/`subgraph`/`pattern`/`inspect`) that remain intentionally structural-only this cycle.

No regressions in the previously-verified truths: all eight application graph suites, the 50 quran-graph tests (conformance on both backends), the 11 server contract tests, tool-registry, CLI snapshots, OpenAPI coverage, `arch-check`, and `migrate-check` are green in this process. The only suite deltas are additive (`graph_neighbors` 2→3, `server/graph` 10→11). One coverage-only advisory remains (export-selector behavior not directly tested for a non-default family); it is not a defect and does not block the goal.

---

_Verified: 2026-10-16T19:22:46Z_
_Verifier: the agent (gsd-verifier)_
