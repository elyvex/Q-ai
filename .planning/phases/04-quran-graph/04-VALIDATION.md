---
phase: "4"
slug: "quran-graph"
# status lifecycle: draft (seeded by plan-phase) → validated (set by validate-phase §6)
# audit-milestone §5.5 distinguishes NOT-VALIDATED (draft) from PARTIAL (validated + nyquist_compliant: false) (#2117)
status: validated
nyquist_compliant: true
wave_0_complete: true
created: "2026-09-28"
validated: "2026-10-13"
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Source: `04-RESEARCH.md` §Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (unit + integration) + trycmd (CLI snapshots) + axum test routers (HTTP contract) |
| **Config file** | Cargo workspace; `crates/cli/tests/quran.rs` runner; `xtask` gates (`arch-check`, `migrate-check`, `ci`) |
| **Quick run command** | `cargo test -p quran-graph` |
| **Full suite command** | `cargo xtask ci` (9-step gate per legacy acceptance.md) |
| **Estimated runtime** | ~120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p quran-graph` + affected crate tests + `cargo fmt --check` + `clippy -D warnings`
- **After every plan wave:** Run above + `cargo xtask arch-check` + `cargo xtask migrate-check`
- **Before `/gsd-verify-work`:** Full suite must be green + recorded non-implementer live walkthrough
- **Max feedback latency:** 300 seconds

---

## Per-Task Verification Map

> Refreshed 2026-10-13 (validate-phase re-audit, State A). All 5 plans complete;
> every task below re-ran green in this audit's process. Single phase
> requirement REQ-quran-graph is covered by all rows; manual-only residues are
> visual-legibility judgments (see Manual-Only), not functional gaps.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 04-01-T1 | 01 (tracer) | 1 | REQ-quran-graph SC1 | — | Structural build w/ fenced publish; neighbors link pinned canonical refs, never graph text | integration + e2e | `cargo test -p application --test graph_neighbors` + `cargo test -p cli --test quran quran_graph_snapshots` | ✅ | ✅ green |
| 04-01-T2 | 01 (tracer) | 1 | REQ-quran-graph | — | Extended vocabulary (TRANSLATES/RELATED_TO/SUPPORTED_BY/DISPUTED_BY); Disputed effective, tombstoned hidden; synthetic-labeled fixtures | unit + integration | `cargo test -p quran-graph --test conformance` (19 × Mem+SQLite) | ✅ | ✅ green |
| 04-01-T3 | 01 (tracer) | 1 | REQ-quran-graph | — | Budget-exhaustion pins (fanout/node/edge truncation + reasons), cycle termination, authz counts, cancel incompletes, QAI-GRAPH-0002 pre-flight | integration | `cargo test -p quran-graph` (49: 22 lib + 19 conformance + 2 no_query + 1 structural + 5 traversal) | ✅ | ✅ green |
| 04-02-T1 | 02 (annotations) | 2 | REQ-quran-graph | T-04-04, T-04-05 | Suggest→accept/reject/dispute/correct lifecycle; single-tx authority writes; tombstone invisibility in traversal + export bytes; no mutation beyond CLI | integration | `cargo test -p application --test graph_review` (8) | ✅ | ✅ green |
| 04-02-T2 | 02 (annotations) | 2 | REQ-quran-graph | T-04-06 | Word-root gated on active attributed dataset (typed UnavailableDataset otherwise); annotated builder refs-only TRANSLATES; AC-P4-03 rebuild preserves authority | integration | `cargo test -p application --test graph_build` (6) | ✅ | ✅ green |
| 04-02-T3 | 02 (annotations) | 2 | REQ-quran-graph | — | Review CLI verbs with reviewer + timestamp, typed NOT_FOUND/VALIDATION, audit emission | e2e | `cargo test -p cli --test quran quran_graph_snapshots` | ✅ | ✅ green |
| 04-03-T1 | 03 (reads) | 3 | REQ-quran-graph SC2 | T-04-07 | Reachability/shortest/up-to-K over reference traversal; no-path only when complete; K pre-flight; per-edge provenance | integration | `cargo test -p application --test graph_paths` (11) | ✅ | ✅ green |
| 04-03-T2 | 03 (reads) | 3 | REQ-quran-graph SC1–SC2 | T-04-08, T-04-09 | Full CLI read tree (Path modes, Subgraph, Pattern, Export) w/ budget flags, verbatim truncation, pattern allowlist | e2e | `crates/cli/tests/quran/graph_s2.trycmd` via `cargo test -p cli --test quran` | ✅ | ✅ green |
| 04-03-T3 | 03 (reads) | 3 | REQ-quran-graph SC3 | T-04-10 | Graph JSON v1 w/ co-traveling assertions under policy filter; DOT/SVG static rendering w/ truncation banner | integration + e2e | `cargo test -p application --test graph_export` (5) + snapshots | ✅ | ✅ green |
| 04-04-T1 | 04 (surfaces) | 4 | REQ-quran-graph SC4 | T-04-11, T-04-13 | 5 versioned HTTP read routes: enveloped, ETag/304, 422 pre-flight, 404 unknown, 200+truncated, 403 denied; no mutation route; OpenAPI in same change | integration | `cargo test -p server --test graph` (10) + `openapi_spec_covers_every_route` | ✅ | ✅ green |
| 04-04-T2 | 04 (surfaces) | 4 | REQ-quran-graph SC4 | T-04-12 | 5 typed agent tools (registry 7→12), service-identical attributed envelopes, QAI-GRAPH codes, pending-suggestion labeling | integration | `cargo test -p application --test graph_tools` (6) | ✅ | ✅ green |
| 04-04-T3 | 04 (surfaces) | 4 | REQ-quran-graph SC4 | — | CLI≈HTTP≈tools canonicalized parity; Explanation on every leg; unanimous truncated-plus-reason | integration | `cargo test -p application --test graph_explain` (4) | ✅ | ✅ green |
| 04-05-T1 | 05 (doctor) | 5 | REQ-quran-graph (operability) | — | 6 read-only `quran.graph.*` checks in one rolled-back UnitOfWork; byte-identical immutability proof | integration | `cargo test -p application --test graph_doctor` (10: 6 checks + 4 repair) | ✅ | ✅ green |
| 04-05-T2 | 05 (doctor) | 5 | REQ-quran-graph (operability) | T-04-15 | Confirmed repair (quarantine-dangling, retention-gated tombstone-gc, authority-preserving rebuild); one audit event per repair; CLI confirm-gated | integration | `cargo test -p application --test graph_doctor repair` (4) | ✅ | ✅ green |
| 04-05-T3 | 05 (closure) | 5 | REQ-quran-graph | — | Owner-gate ledger (OD-11/OD-12/P4-X01/X02/X03/X05 BLOCKED w/ closers) + deferral ledger + D-01 SC1–SC4 evidence matrix; full phase gate green | other + full gate | `cargo test -p quran-graph` (49) + 8 app suites (52) + snapshots + `cargo xtask arch-check` + `cargo xtask migrate-check` | ✅ | ✅ green |
| 04-06-T1 | 06 (gap closure G-04-SC1) | 1 | REQ-quran-graph SC1 | T-04-17, T-04-18 | Projection selector (default structural) resolved in one place; concept/entity neighborhood reachable on CLI/HTTP/tool with identical canonicalized output; unknown family is typed `UnknownProjection` (404/NOT_FOUND); pending-scholar attribution preserved, default read scope unchanged | integration + e2e | `cargo test -p application --test graph_neighbors concept_neighborhood_is_reachable_only_through_the_annotated_projection` + `cargo test -p server --test graph graph_neighbors_route_threads_projection_selector` + `cargo test -p cli --test quran quran_graph_snapshots` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky — all rows ✅ green, re-run 2026-10-13.*

---

## Wave 0 Requirements

- [x] Backend-generic conformance harness (`MemGraphStore` → parameterize over `SQLiteGraphStore`) — `crates/quran-graph/tests/conformance.rs`, 19 tests × 2 backends green
- [x] `crates/application/tests/graph_*.rs` integration files (neighbors/paths/subgraph/pattern/review/build/export/doctor) — 8 suites, 52 tests green
- [x] `crates/cli/tests/quran/graph_s1.trycmd` + `graph_s2.trycmd` (+ runner entries) — `quran_graph_snapshots` green
- [x] `crates/server/tests/` graph contract tests + `docs/08-api/quran-v1-openapi.json` graph route additions — 10/10 + `openapi_spec_covers_every_route` green
- [x] `fixtures/quran/graph/concept-seed-v1.json` + annotation golden fixtures (synthetic-labeled) — validated by conformance run
- [x] Migration `0022_quran_graph_fix` + `checksums.json` update (UNIQUE fix, disputed status, assertion scoping) — `migrate-check` green (26 migrations stable)

*Wave 0 complete: all harness + fixture items shipped across plans 04-01–04-05; existing `quran-graph` unit tests (22) run green.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Non-implementer live walkthrough of neighbor view → path → export | REQ-quran-graph SC1–SC4 | Usability/legibility judgment a script cannot make | Operator builds projection from synthetic fixture, opens neighbor view, finds a path, exports subgraph, confirms provenance legible |
| Static SVG/DOT rendering legibility | CONTEXT.md D-13 | Visual judgment | Render a neighborhood subgraph; confirm nodes/edges/provenance readable |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 300s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-10-13 (stale-verification refresh; 15/15 tasks green, 0 functional gaps, 2 visual-legibility manual-only)

## Validation Audit 2026-10-13

| Metric | Count |
|---|---|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |
