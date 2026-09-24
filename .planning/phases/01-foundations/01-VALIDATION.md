---
phase: "1"
slug: "foundations"
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-24"
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` / `#[tokio::test]`, `trycmd`, `tempfile`, and focused `proptest` coverage |
| **Config file** | None dedicated; Cargo workspace and existing crate test targets |
| **Quick run command** | `cargo test -p config --lib && cargo test -p testkit --test config_precedence && cargo test -p cli --test catalog && cargo test -p jobs --lib && cargo test -p storage-sqlite --test recovery_jobs --test outbox_idempotency --test integrity_audit` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | Measure locally; use focused commands after each task and the full suite at wave/phase gates |

---

## Sampling Rate

- **After every task commit:** Run the focused test command for the touched seam; when Rust files change, also run `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings`.
- **After every plan wave:** Run the quick suite plus `cargo run -q -p xtask -- arch-check && cargo run -q -p xtask -- migrate-check`.
- **Before `/gsd-verify-work`:** Run `cargo test --workspace` and `cargo run -p xtask -- ci`; all five Phase 1 success-criterion evidence commands must be green.
- **Max feedback latency:** Prefer the narrowest focused crate test for task feedback; no watch-mode flags.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | REQ-product-vision, REQ-cli-api | — | Stable foundation groups remain visible without pulling later features into Phase 1 | CLI | `cargo test -p cli --test catalog` | Existing | pending |
| 01-01-02 | 01 | 1 | REQ-product-principles, REQ-engineering-baseline | T1-CONFIG | Effective config shows redacted values and origins; malformed input yields a remedy | unit/CLI | `cargo test -p config --lib && cargo test -p testkit --test config_precedence` | Existing | pending |
| 01-01-03 | 01 | 1 | REQ-storage-architecture, REQ-cli-api | T1-DOCTOR | Doctor is metadata/read-only and pending migrations require explicit `qai db migrate` | integration/CLI | `cargo test -p storage-sqlite --test recovery_jobs && cargo test -p cli --test catalog` | Existing | pending |
| 01-02-01 | 02 | 2 | REQ-product-principles, REQ-storage-architecture | T1-AUDIT | Authoritative writes commit state, provenance, audit, and outbox atomically; failure rolls back | integration | `cargo test -p storage-sqlite --test integrity_audit` | Existing | pending |
| 01-02-02 | 02 | 2 | REQ-product-principles, REQ-engineering-baseline | T1-AUDIT | Doctor delegates to persisted audit verification and reports tampering/gaps | integration/CLI | `cargo test -p application audit_bridge::tests::persisted_verification_checks_hashes_links_and_gaps` | Existing | pending |
| 01-03-01 | 03 | 3 | REQ-engineering-baseline, REQ-storage-architecture, REQ-cli-api | T1-JOBS | Long-lived host owns workers; one-shot commands enqueue without silently running them | unit/integration/CLI | `cargo test -p jobs --lib && cargo test -p application --lib job_queue` | Existing | pending |
| 01-03-02 | 03 | 3 | REQ-product-principles, REQ-storage-architecture | T1-JOBS | Named checkpoints survive lease expiry; retry/backoff and cooperative cancel outcomes are auditable | integration | `cargo test -p storage-sqlite --test recovery_jobs --test outbox_idempotency` | Existing | pending |
| 01-04-01 | 04 | 4 | REQ-architecture-principles-quality | T1-ARCH | Forbidden registry/git edges fail `arch-check`; allowed existing edges remain green | unit/build | `cargo test -p xtask && cargo run -q -p xtask -- arch-check` | Existing | pending |

*Status: pending · green · red · flaky*

---

## Wave 0 Requirements

- [ ] Extend the existing CLI harness (prefer `crates/cli/tests/catalog.rs` / trycmd cases) to cover the seven stable groups, effective config origins, explicit migration remedies, job retry/cancel, and audit verification output.
- [ ] Extend existing application/storage tests for audited mutation rollback, lifecycle audit events, named checkpoints, and cancellation outcomes; do not create parallel test frameworks.
- [ ] Add an `xtask` synthetic external-dependency mutation fixture or test for registry/git edge enforcement.
- [ ] Extend the existing `testkit` fixture boundary only where shared config/data/audit seeds reduce duplication.
- [x] Framework install: **none** — Rust, Cargo, Tokio, SQLx, Clap, trycmd, tempfile, and existing test targets are present.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Final five-criterion evidence review | All Phase 1 requirements | Cross-artifact acceptance judgment is goal-backward and should be checked by the verifier after automated gates | Review each evidence-matrix row, linked repeatable command, and source/test path; reject criteria supported only by prose. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Full suite and `xtask ci` green before phase verification
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
