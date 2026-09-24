---
phase: "2"
slug: "canonical-quran-core"
# status lifecycle: draft (seeded by plan-phase) → validated (set by validate-phase §6)
# audit-milestone §5.5 distinguishes NOT-VALIDATED (draft) from PARTIAL (validated + nyquist_compliant: false) (#2117)
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-09-25"
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Seeded by plan-phase from `02-RESEARCH.md` § Validation Architecture. Per-task rows
> are populated by `validate-phase` after planning.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` / `#[tokio::test]`, `proptest` (properties), `trycmd` (real CLI snapshots), `tempfile` (isolated real-SQLite state) |
| **Config file** | none — the Cargo workspace + existing test targets are the configuration |
| **Quick run command** | `cargo test -p quran-core -p quran-corpus -p citations -p provenance` |
| **Integration command** | `cargo test -p storage-sqlite --test quran && cargo test -p application --test quran_import --test quran_reader --test quran_translation --test quran_verification --test quran_doctor --test quran_tools` |
| **Surface command** | `cargo test -p cli --test quran --test catalog --test doctor_json && cargo test -p server --test api` |
| **Full suite command** | `cargo test --workspace` |
| **Full pre-merge gate** | `cargo run -p xtask -- ci` (incl. `fmt`, `clippy -D warnings`, `arch-check`, `migrate-check`, `gen-schema` diff) |
| **Estimated runtime** | full workspace suite ~ minutes (Rust); focused targets seconds |

---

## Sampling Rate

- **After every task commit:** the focused target for the touched seam, plus `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` when Rust files change.
- **After every plan wave:** the canonical quick suite + `cargo run -q -p xtask -- arch-check && cargo run -q -p xtask -- migrate-check`.
- **Before `/gsd-verify-work`:** `cargo test --workspace` and `cargo run -p xtask -- ci` green, plus the five success-criterion commands recorded with their output as the phase's evidence-of-record (D-11).
- **Max feedback latency:** focused targets (seconds); full suite (minutes).

**Known baseline exception (NOT this phase's scope):** `cargo test -p cli --test quran` reports 3 passed / 2 failed — `quran_search_snapshots` (`search.trycmd`) and `quran_normalize_snapshots` (`normalize.trycmd`). Those belong to the legacy search/normalization phase (roadmap Phase 3). Do not attribute their failure to Phase 2; re-check on a quiet worktree before any phase-gate read.

---

## Per-Task Verification Map

Seeded at criterion level. `validate-phase` expands to one row per task after planning.

| Criterion | Plan | Wave | Requirement | Test type | Automated command | File Exists | Status |
|-----------|------|------|-------------|-----------|-------------------|-------------|--------|
| SC-1 import → activation → rollback | TBD | TBD | REQ-quran-corpus | integration + CLI snapshot | `cargo test -p application --test quran_import`; `cargo test -p storage-sqlite --test quran`; `cargo test -p cli --test quran` | ✅ (rollback-rejection: ❌ W0 — QC-01) | ⬜ pending |
| SC-2 byte-exact lookup + pinned edition | TBD | TBD | REQ-quran-corpus | integration + property + golden | `cargo test -p application --test quran_reader`; `cargo test -p quran-core --test reference_grammar` | ✅ (operator hash surface: ❌ W0 — QC-02) | ⬜ pending |
| SC-3 six integrity families | TBD | TBD | REQ-ingestion-validation-eval | unit + integration + snapshot | `cargo test -p quran-corpus`; `cargo test -p application --test quran_doctor`; `cargo test -p cli --test doctor_json` | ✅ (reference operator path: ❌ W0 — QC-03/QC-04) | ⬜ pending |
| SC-4 canonical-write fence | TBD | TBD | REQ-quran-corpus | integration + source-scan unit | `cargo test -p storage-sqlite --test quran`; `cargo test -p storage --lib quran`; `cargo test -p application --test quran_import` | ✅ for 5 statements; ❌ W0 for 7 untriggered tables — QC-05 | ⬜ pending |
| SC-5 `verify_quotation` hard failure | TBD | TBD | REQ-quran-corpus | unit + integration | `cargo test -p citations`; `cargo test -p application --test quran_tools`; `cargo test -p server --test api` | ✅ library; ❌ W0 production path — QC-07 | ⬜ pending |
| REQ-data-separation-layers (translations) | TBD | TBD | REQ-data-separation-layers | integration | `cargo test -p application --test quran_translation` | ✅; `text_hash` gap — QC-08 | ⬜ pending |
| Purity fence (no LLM/vector on canonical path) | TBD | TBD | REQ-quran-corpus | build gate | `cargo run -q -p xtask -- arch-check` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/application/tests/quran_import.rs` — rollback rejection coverage (missing/denied/mismatched approval → pointer unchanged) + rollback cache-invalidation case. [QC-01]
- [ ] `crates/storage-sqlite/tests/quran.rs` — extend `canonical_triggers_abort_raw_writes_with_codes` to one row per canonical table after QC-05's migration; extend `canonical_tables_declare_the_trigger_set` to enumerate the full expected trigger set.
- [ ] `crates/cli/tests/quran/` — a trycmd case for the integrity/verify surface (D-11, QC-03) and the quotation hard-failure exit code (QC-07).
- [ ] `fixtures/quran/` — the richer synthetic edition (D-08) plus a companion synthetic reference + pinned `reference_text_hash`, and a golden hash file for the new fixture. [QC-13]
- [ ] A committed corpus-integrity report artifact (D-11) — none exists today.
- [ ] Framework install: **none** — Rust, Cargo, SQLx, trycmd, tempfile, proptest, and all listed test targets already exist.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Canonical dataset identity + license (OD-01) | REQ-quran-corpus | Human/editor decision — no agent may invent dataset, license, or reviewer | Owner supplies dataset identity + license evidence; record in ADR-0101 and close OD-01 |
| Named editorial reviewer sign-off (OD-02) | REQ-quran-corpus | Human/editor judgment on religious-source correctness | Owner names reviewer + sample + method; record in the edition's `verified_by` |
| Independent reference-corpus identity (OD-03) | REQ-ingestion-validation-eval | Human/source decision; note `spqrxi/quranchecksum` is hash-only and cannot alone satisfy a byte-exact QV-015 comparison | Owner ratifies corpus identity + procedure (text-bearing vs hash-only) per ADR-0114 |

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency acceptable (focused targets seconds)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
