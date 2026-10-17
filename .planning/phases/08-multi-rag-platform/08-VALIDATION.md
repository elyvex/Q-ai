---
phase: "8"
slug: "multi-rag-platform"
status: draft
nyquist_compliant: false
wave_0_complete: true
created: "2026-10-17"
---

# Phase 8 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` / `#[tokio::test]` + `trycmd` |
| **Quick run command** | `cargo test -p application --test rag_search -p cli --test rag_parity` |
| **Full suite command** | `cargo test --workspace && cargo xtask arch-check && cargo xtask migrate-check` |
| **Estimated runtime** | ~300 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p <touched crate>`
- **Before `/gsd-verify-work`:** Full suite must be green (incl. `arch-check`, `migrate-check`)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | Status |
|---------|------|------|-------------|-----------|-------------------|--------|
| 08-01 Task 1: migrations + traits | 08-01 | 1 | REQ-multi-rag | unit | `cargo test -p rag -p application` | ✅ green |
| 08-02 Task 1: TOML config + lifecycle | 08-02 | 2 | REQ-multi-rag | unit | `cargo test -p application --test rag_search` | ✅ green |
| 08-03 Task 1: hybrid retrieval RRF | 08-03 | 3 | REQ-multi-rag | unit | `cargo test -p application --test rag_search` | ✅ green |
| 08-04 Task 1: routing transparency | 08-04 | 4 | REQ-multi-rag | unit | `cargo test -p application --test rag_search` | ✅ green |
| 08-05 Task 1: tombstoning | 08-05 | 5 | REQ-multi-rag | unit | `cargo test -p application --test rag_search` | ✅ green |
| 08-06 Task 1: cross-surface parity | 08-06 | 5 | REQ-multi-rag | integration | `cargo test -p cli --test rag_parity` | ✅ green |
| 08-06 Task 3: full phase gate | 08-06 | 5 | REQ-multi-rag | integration | `cargo test --workspace && cargo xtask arch-check && cargo xtask migrate-check` | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Phase Gate Results

- `cargo test --workspace`: green (204 test binaries ok)
- `cargo xtask arch-check`: OK
- `cargo xtask migrate-check`: OK (38 migrations)
