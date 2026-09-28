---
phase: "4"
slug: "quran-graph"
# status lifecycle: draft (seeded by plan-phase) → validated (set by validate-phase §6)
# audit-milestone §5.5 distinguishes NOT-VALIDATED (draft) from PARTIAL (validated + nyquist_compliant: false) (#2117)
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-28"
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

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 4-tbd | 01 (tracer) | 1 | REQ-quran-graph SC1 | — | Neighbor view links to pinned canonical refs; no canonical text in graph records | integration + trycmd + HTTP contract + tool test | `cargo test -p application --test graph_neighbors` | ❌ W0 | ⬜ pending |
| 4-tbd | 03 (reads) | 3 | REQ-quran-graph SC2 | — | Paths with per-edge provenance; no-path only when complete | integration + conformance extension | `cargo test -p quran-graph --test conformance` | ❌ W0 | ⬜ pending |
| 4-tbd | 03 (reads) | 3 | REQ-quran-graph SC3 | — | Graph JSON v1 with assertions + truncation banner; policy-filtered | unit + policy-leak test | `cargo test -p application --test graph_export` | ❌ W0 | ⬜ pending |
| 4-tbd | 04 (surfaces) | 4 | REQ-quran-graph SC4 | — | Explainability on every result, all surfaces | contract test | `cargo test -p application --test graph_explain` | ❌ W0 | ⬜ pending |
| 4-tbd | 05 (hardening) | 5 | REQ-quran-graph (builds/review/doctor) | — | Staged build + fenced publish + tombstone invisibility + read-only doctor | fault-injection + state-machine + immutability | `cargo test -p application --test graph_build` + `graph_review` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Backend-generic conformance harness (`MemGraphStore` → parameterize over `SQLiteGraphStore`)
- [ ] `crates/application/tests/graph_*.rs` integration files (neighbors/paths/subgraph/pattern/review/build/export/doctor)
- [ ] `crates/cli/tests/quran/graph_s1.trycmd` + `graph_s2.trycmd` (+ runner entries)
- [ ] `crates/server/tests/` graph contract tests + `docs/08-api/quran-v1-openapi.json` graph route additions
- [ ] `fixtures/quran/graph/concept-seed-v1.json` + annotation golden fixtures (synthetic-labeled)
- [ ] Migration `0022_quran_graph_fix` + `checksums.json` update (UNIQUE fix, disputed status, assertion scoping)

*Wave 0 installs the harness + fixtures above; existing `quran-graph` unit tests (38) already run.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Non-implementer live walkthrough of neighbor view → path → export | REQ-quran-graph SC1–SC4 | Usability/legibility judgment a script cannot make | Operator builds projection from synthetic fixture, opens neighbor view, finds a path, exports subgraph, confirms provenance legible |
| Static SVG/DOT rendering legibility | CONTEXT.md D-13 | Visual judgment | Render a neighborhood subgraph; confirm nodes/edges/provenance readable |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 300s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
