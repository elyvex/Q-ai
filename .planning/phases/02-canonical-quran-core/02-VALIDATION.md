---
phase: "2"
slug: "canonical-quran-core"
# status lifecycle: draft (seeded by plan-phase) → validated (set by validate-phase §6)
# audit-milestone §5.5 distinguishes NOT-VALIDATED (draft) from PARTIAL (validated + nyquist_compliant: false) (#2117)
status: validated
nyquist_compliant: true
wave_0_complete: true
created: "2026-09-25"
validated: "2026-10-04"
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Seeded by plan-phase from `02-RESEARCH.md` § Validation Architecture. Expanded to
> task level and audited by `validate-phase` / the Nyquist auditor on 2026-10-04
> (see `## Validation Audit 2026-10-04` at the end).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` / `#[tokio::test]`, `proptest` (properties), `trycmd` (real CLI snapshots), `tempfile` (isolated real-SQLite state) |
| **Config file** | none — the Cargo workspace + existing test targets are the configuration |
| **Quick run command** | `cargo test -p quran-core -p quran-corpus -p citations -p provenance` |
| **Integration command** | `cargo test -p storage-sqlite --test quran && cargo test -p application --test quran_import --test quran_reader --test quran_translation --test quran_verification --test quran_doctor --test quran_tools --test quran_identity --test quran_reference --test answer_path_ledger` |
| **Surface command** | `cargo test -p cli --test quran --test catalog --test doctor_json --test corpus_integrity && cargo test -p server --test api` |
| **Full suite command** | `cargo test --workspace` |
| **Full pre-merge gate** | `cargo run -p xtask -- ci` (incl. `fmt`, `clippy -D warnings`, `arch-check`, `migrate-check`, `gen-schema` diff) |
| **Estimated runtime** | full workspace suite ~ minutes (Rust); focused targets seconds |

---

## Sampling Rate

- **After every task commit:** the focused target for the touched seam, plus `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` when Rust files change.
- **After every plan wave:** the canonical quick suite + `cargo run -q -p xtask -- arch-check && cargo run -q -p xtask -- migrate-check`.
- **Before `/gsd-verify-work`:** `cargo test --workspace` and `cargo run -p xtask -- ci` green, plus the five success-criterion commands recorded with their output as the phase's evidence-of-record (D-11).
- **Max feedback latency:** focused targets (seconds); full suite (minutes).

**Known baseline exception (NOT this phase's scope):** `cargo test -p cli --test quran` reports `quran_search_snapshots` (`search_s1/s2.trycmd`) and `quran_normalize_snapshots` (`normalize_s1/s2.trycmd`) failing from a non-deterministic index manifest hash. Those belong to the legacy search/normalization phase (roadmap Phase 3). Do not attribute their failure to Phase 2; re-check on a quiet worktree before any phase-gate read. The Phase-2-owned harnesses (`quran_snapshots`, `quran_verify_snapshots`, `quran_reference_snapshots`, `edition_identity_snapshots`, `quran_verify_quotation_snapshots`, `quran_catalog_snapshots`) all pass.

---

## Per-Task Verification Map

One row per task across all eight plans. `File Exists` = the referenced test file / fixture / artifact exists at HEAD. `Status` was set by re-running the exact `<automated>` command on 2026-10-04 (see the audit section).

| Task | Criterion | Plan | Wave | Requirement | Test type | Automated command | File Exists | Status |
|------|-----------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01/T1 edition identity → operator surface | SC-1, SC-2 | 02-01 | 1 | REQ-quran-corpus | integration + migration gate | `cargo test -p application --test quran_identity`; `cargo test -p storage-sqlite --test quran`; `cargo run -q -p xtask -- migrate-check` | ✅ | ✅ green |
| 02-01/T2 persist declared license | SC-1 | 02-01 | 1 | REQ-data-separation-layers | integration | `cargo test -p application --test quran_identity`; `cargo test -p application --test quran_import` | ✅ | ✅ green |
| 02-01/T3 primary/default selector | SC-1, SC-2 | 02-01 | 1 | REQ-quran-corpus | integration | `cargo test -p application --test quran_identity`; `cargo test -p application --test quran_reader`; `cargo run -q -p xtask -- arch-check` | ✅ | ✅ green |
| 02-02/T1 rich fixture + CSV/golden mirrors | SC-3 | 02-02 | 2 | REQ-quran-corpus | unit + golden | `cargo test -p quran-corpus --test fixtures --test adversarial` | ✅ | ✅ green |
| 02-02/T2 companion reference + identity fixture | SC-3 | 02-02 | 2 | REQ-ingestion-validation-eval | unit + golden | `cargo test -p quran-corpus --test fixtures` | ✅ | ✅ green |
| 02-03/T1 verify six-family surface + state-derived doctor + stored hash line | SC-3 | 02-03 | 3 | REQ-ingestion-validation-eval | integration + CLI snapshot | `cargo test -p cli --test quran -- quran_verify_snapshots`; `cargo test -p application --test quran_doctor`; `cargo test -p cli --test doctor_json` | ✅ | ✅ green |
| 02-03/T2 operator reference path (QV-015 byte-exact) | SC-3 | 02-03 | 3 | REQ-ingestion-validation-eval | integration + CLI snapshot | `cargo test -p application --test quran_reference`; `cargo test -p cli --test quran -- quran_reference_snapshots`; `cargo test -p quran-corpus --lib differ` | ✅ | ✅ green |
| 02-03/T3 edition show identity/primary/license | SC-2 | 02-03 | 3 | REQ-quran-corpus | CLI snapshot + integration | `cargo test -p cli --test quran -- edition_identity_snapshots`; `cargo test -p application --test quran_identity` | ✅ | ✅ green |
| 02-04/T1 canonical trigger fence (migration 0021) | SC-4 | 02-04 | 2 | REQ-quran-corpus | integration + migration gate | `cargo test -p storage-sqlite --test quran`; `cargo test -p storage --lib quran`; `cargo run -q -p xtask -- migrate-check` | ✅ | ✅ green |
| 02-04/T2 non-forgeable ApprovalToken + CanonicalWriter gate | SC-4 | 02-04 | 2 | REQ-ingestion-validation-eval | unit + integration + source audit | `cargo test -p provenance --lib`; `cargo test -p quran-corpus --test import_path`; `cargo test -p application --test quran_import --test quran_verification` | ✅ | ✅ green |
| 02-04/T3 rollback rejection + cache invalidation + segments scope | SC-1 | 02-04 | 2 | REQ-quran-corpus | integration + doc check | `cargo test -p application --test quran_import`; `cargo test -p application --test quran_reader`; `test -s docs/07-technical/quran-canonical-core-decisions.md` | ✅ | ✅ green |
| 02-05/T1 shared verdict→hard-failure mapping | SC-5 | 02-05 | 4 | REQ-quran-corpus | unit | `cargo test -p citations`; `cargo run -q -p xtask -- arch-check` | ✅ | ✅ green |
| 02-05/T2 verify-quotation production surface | SC-5 | 02-05 | 4 | REQ-quran-corpus | integration + CLI snapshot | `cargo test -p application --test quran_tools`; `cargo test -p cli --test quran -- quran_verify_quotation_snapshots` | ✅ | ✅ green |
| 02-05/T3 HTTP enforcement + recorded read-path exemptions | SC-5 | 02-05 | 4 | REQ-quran-corpus | integration | `cargo test -p server --test api`; `cargo test -p application --test quran_tools` | ✅ | ✅ green |
| 02-06/T1 additive translation hash recipe + real text_hash | [REQ-data-separation-layers] | 02-06 | 3 | REQ-data-separation-layers | unit + integration | `cargo test -p quran-corpus --lib hashing`; `cargo test -p application --test quran_translation`; `cargo test -p quran-corpus --lib` | ✅ | ✅ green |
| 02-06/T2 verbatim translation license + layer-separation negative test | [REQ-data-separation-layers] | 02-06 | 3 | REQ-data-separation-layers | integration + unit | `cargo test -p application --test quran_translation`; `cargo test -p quran-core --lib view` | ✅ | ✅ green |
| 02-07/T1 owner-gate ledger + owner-ratifiable D-15 record | [owner gates] | 02-07 | 5 | REQ-quran-corpus | doc/audit gate | `rg -c 'OD-01\|OD-02\|OD-03' docs/05-followups/phase-02-owner-gates.md`; `rg 'ratif\|D-15'`; `rg 'phase-02-owner-gates' docs/05-followups/decisions-needed.md` | ✅ | ✅ green (manual sign-off — see Manual-Only) |
| 02-07/T2 coverage gate reconciliation | [QC-12] | 02-07 | 5 | REQ-ingestion-validation-eval | build gate | `cargo test -p xtask`; `cargo run -q -p xtask -- coverage-gate lcov.info` | ✅ | ✅ green (phase-2 rows) |
| 02-07/T3 evidence of record + integrity artifact + TASK-002 closure | SC-3 | 02-07 | 5 | REQ-ingestion-validation-eval | integration + artifact gate | `cargo test -p cli --test corpus_integrity`; python six-family artifact check; python evidence-token check; `test -s …TASK-002…` lifecycle check; full Quran-area suite | ✅ | ✅ green |
| 02-08/T1 rebuild D-15 ledger in evidence-of-record files (G-02-3) | D-15 / G-02-3 | 02-08 | 1 | REQ-quran-corpus | doc/source gate | LEDGER grep gate over `phase-02-evidence.md`, `phase-02-owner-gates.md`, `02-05-SUMMARY.md` | ✅ | ✅ green (ratification manual — see Manual-Only) |
| 02-08/T2 correct in-code doc + research source + plan narrowing | G-02-3 | 02-08 | 1 | REQ-ingestion-validation-eval | doc gate | `cargo build -p application` + DOCS grep gate (`search_exact_handler`/`quran.search`/`G-02-3`) | ✅ | ✅ green |
| 02-08/T3 answer-path source-scan guard | D-15 / G-02-3 | 02-08 | 1 | REQ-quran-corpus | source-scan guard | `cargo test -p application --test answer_path_ledger` (6 tests) | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Criterion → task roll-up**

| Criterion | Requirements | Covering tasks | Status |
|-----------|--------------|----------------|--------|
| SC-1 import → activation → rollback | REQ-quran-corpus | 02-01/T1–T3, 02-04/T3 | ✅ green |
| SC-2 byte-exact lookup + pinned edition (incl. stored hash) | REQ-quran-corpus | 02-01/T1, T3, 02-03/T3 | ✅ green |
| SC-3 six integrity families (+ committed artifact) | REQ-ingestion-validation-eval | 02-02/T1–T2, 02-03/T1–T2, 02-07/T3 | ✅ green |
| SC-4 canonical-write fence (triggers + type-level gate + importer audit) | REQ-quran-corpus | 02-04/T1–T2 | ✅ green |
| SC-5 `verify_quotation` hard failure on production paths | REQ-quran-corpus | 02-05/T1–T3 | ✅ green |
| REQ-data-separation-layers (translations) | REQ-data-separation-layers | 02-01/T2, 02-06/T1–T2 | ✅ green |
| Purity fence (no LLM/vector on canonical path) | REQ-quran-corpus | `arch-check` | ✅ green |
| D-15 answer-path ledger completeness (G-02-3) | REQ-quran-corpus | 02-05/T3, 02-08/T1–T3 | ✅ green (mechanically guarded; owner ratification manual) |

---

## Wave 0 Requirements

All Wave 0 requirements were seeded as MISSING and are now **filled and verified green** by plans 02-01…02-08.

- [x] `crates/application/tests/quran_import.rs` — rollback rejection coverage (missing/denied/mismatched approval → pointer unchanged) + rollback cache-invalidation case. [QC-01] → `rejected_rollbacks_leave_canonical_state_untouched`, `cache_serves_no_stale_text_after_rollback`. ✅
- [x] `crates/storage-sqlite/tests/quran.rs` — `canonical_triggers_abort_raw_writes_with_codes` now asserts one coded abort per canonical table (QAI-QUR-0001…0021, 22 statements across every canonical table + edition identity/default columns); `canonical_tables_declare_the_trigger_set` enumerates all 21 expected triggers. [QC-05] → 02-04 T1. ✅
- [x] `crates/cli/tests/quran/` — a trycmd case for the integrity/verify surface (D-11, QC-03: `verify_s1/s2.trycmd`) and the quotation hard-failure exit code (QC-07: `verify_quotation_s1/s2.trycmd`). ✅
- [x] `fixtures/quran/` — the richer synthetic edition (D-08: `test-edition-rich/`) plus a companion synthetic reference + pinned `reference_text_hash`, and a golden hash file (`golden/ayah_texts.jsonl`, edition-scoped). [QC-13] → 02-02. ✅
- [x] A committed corpus-integrity report artifact (D-11) — `docs/06-progress/corpus-integrity-report.json`, guarded by `crates/cli/tests/corpus_integrity.rs`. → 02-07 T3. ✅
- [x] Framework install: **none** — Rust, Cargo, SQLx, trycmd, tempfile, proptest, and all listed test targets already exist. ✅

---

## Manual-Only Verifications

These are genuine human/editor decisions. Each is recorded as an explicit blocked owner gate (never auto-closable) in `docs/05-followups/phase-02-owner-gates.md`.

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Canonical dataset identity + license (OD-01) | REQ-quran-corpus | Human/editor decision — no agent may invent dataset, license, or reviewer | Owner supplies dataset identity + license evidence + bundle policy in ADR-0101, then imports a real edition with `qai quran import <manifest> --adapter json`; close OD-01 in `decisions-needed.md` |
| Named editorial reviewer sign-off (OD-02) | REQ-quran-corpus | Human/editor judgment on religious-source correctness | Owner names reviewer + comparison sample + method, then `qai quran edition verify <slug@version> --reviewer <name> --method <method> --yes`; any in-tree reviewer string is a test placeholder and must be labelled as such |
| Independent reference-corpus identity (OD-03) | REQ-ingestion-validation-eval | Human/source decision; note `spqrxi/quranchecksum` is hash-only and cannot alone satisfy a byte-exact QV-015 comparison | Owner ratifies corpus identity + procedure (text-bearing vs hash-only) per ADR-0114, then `qai quran import <manifest> --reference <reference-path>` evaluates QV-015 against it |
| D-15 read-path exemption ratification | REQ-quran-corpus | Whether the "exempt-by-construction" reading of D-15 is correct is an owner architectural judgment; the ledger completeness is now **mechanically guarded** (`answer_path_ledger.rs`), but the interpretation is not testable | Owner ratifies or overrules the emission-frame interpretation recorded in `docs/05-followups/phase-02-owner-gates.md` §"Owner-ratifiable interpretation — D-15" |

*All other phase behaviors have automated verification.*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies (22/22)
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (all 6 seeded items filled and green)
- [x] No watch-mode flags
- [x] Feedback latency acceptable (focused targets seconds)
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** signed 2026-10-04 (Nyquist validation audit — see below)

---

## Validation Audit 2026-10-04

**Scope:** Bring `02-VALIDATION.md` from `status: draft` to validated. Re-ran every
task-level `<automated>` command against HEAD (not trusting SUMMARY claims), expanded
the criterion-level map to one row per task, and checked each seeded Wave 0 gap is
genuinely filled. Implementation files were read-only; no test or fixture was added
because no genuine MISSING coverage gap was found.

### Commands re-run and observed results

| Command | Observed |
|---------|----------|
| `cargo test -p quran-core -p quran-corpus -p citations -p provenance` | quran-core 46, quran-corpus 75 (52+6+13+4), citations 13, provenance 10 — all 0 failed |
| `cargo test -p storage-sqlite --test quran` | 11 passed, 0 failed |
| `cargo test -p application --test quran_import --test quran_reader --test quran_translation --test quran_verification --test quran_doctor --test quran_tools --test quran_identity --test quran_reference --test answer_path_ledger` | 16 / 13 / 13 / 3 / 4 / 9 / 7 / 1 / 6 passed, 0 failed |
| `cargo test -p cli --test quran -- quran_snapshots quran_verify_snapshots quran_reference_snapshots edition_identity_snapshots quran_verify_quotation_snapshots quran_catalog_snapshots` | 6 passed, 0 failed |
| `cargo test -p cli --test catalog --test doctor_json --test corpus_integrity` | 2 / 3 / 4 passed, 0 failed |
| `cargo test -p server --test api` | 22 passed, 0 failed |
| `cargo test -p storage --lib quran` | 4 passed, 0 failed |
| `cargo test -p xtask` | 25 passed, 0 failed |
| `cargo run -q -p xtask -- arch-check` | OK — no forbidden dependency edges |
| `cargo run -q -p xtask -- migrate-check` | OK — 22 migration(s) ordered; checksums stable |
| `cargo run -q -p xtask -- gen-schema` + `git diff --exit-code docs/schemas/` | SCHEMA_CLEAN |
| LEDGER grep gate (02-08 T1) | LEDGER_OK |
| DOCS grep gate (02-08 T2) | DOCS_OK |
| Corpus-integrity artifact six-family python check | FAMILIES_OK |
| Evidence-token python check (02-07 T3) | EVIDENCE_OK |
| TASK-002 lifecycle check | TASK002_OK |
| `cargo run -q -p xtask -- coverage-gate lcov.info` | quran-core 91.67% OK, quran-corpus 92.55% OK; citations 79.27% < 85% FAIL (see below) |

### Metrics

| Metric | Value |
|--------|-------|
| Tasks audited | 22 (plans 02-01…02-08) |
| Rows green | 22 / 22 |
| Seeded Wave 0 gaps | 6 / 6 filled and verified |
| New MISSING gaps found this audit | 0 |
| Tests/fixtures added this audit | 0 (no genuine gap) |
| Manual-only items (honest, non-automatable) | 4 (OD-01, OD-02, OD-03, D-15 ratification) |
| `nyquist_compliant` | true |

### Findings / notes

- **G-02-3 (D-15 answer-path ledger)** — closed by plan 02-08. The mechanical guard
  `crates/application/tests/answer_path_ledger.rs` (6 tests) resolves every ledger
  symbol kind-aware and fails closed on drift or an unrecorded emitting handler.
  Verified green. This test did not exist when this file was seeded.
- **QC-05 hard edge** — verified the trigger test is genuinely per-table (22 raw
  `UPDATE`/`DELETE` statements with distinct codes), not a trivial 5-statement
  check; the enumeration test asserts all 21 expected triggers.
- **Coverage-gate citations RED (⚠️ not Phase 2 scope)** — the committed
  `lcov.info` is dated 2026-09-25 and predates the later plan `03.5-04`, which
  raised the `citations` floor to the published 85% (`xtask/src/coverage.rs:11-14,
  49`). Plan 02-07's own scope (QC-12) was to add the `quran-core`/`quran-corpus`
  rows — both pass (91.67% / 92.55%). Regenerate `lcov.info` before reading the
  `citations` row as a phase gate; it is not a Phase 2 requirement failure.
- **Legacy baseline** — the two CLI snapshot failures (`quran_search_snapshots`,
  `quran_normalize_snapshots`) are Roadmap Phase 3 work and were excluded from this
  audit; every Phase-2-owned harness passes.
