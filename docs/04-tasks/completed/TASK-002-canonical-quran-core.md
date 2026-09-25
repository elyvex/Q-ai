# TASK-002 — Canonical Quran Core (Phase 2)

- **Task ID:** `TASK-002-canonical-quran-core`
- **Status:** Completed
- **Phase:** `02-canonical-quran-core` (roadmap Phase 2)
- **Owner:** Engineering (GSD plans 02-01…02-07)
- **Phase contract:** `.planning/ROADMAP.md` §Phase 2 + `.planning/REQUIREMENTS.md`
  (`REQ-quran-corpus`, `REQ-data-separation-layers`, `REQ-ingestion-validation-eval`)
- **Closure owned by:** plan 02-07 — this record is created and moved active→completed
  only after the gate evidence and the phase evidence document exist.

## Purpose

Deliver and prove one validated Quran edition that is importable, addressable, and
provably immutable: staging → validation → atomic activation with rollback; byte-exact
`surah:ayah` lookup with a pinned edition reference; the six corpus-integrity families;
canonical tables that reject all non-approved writes with an importer that has no path
to them; and `verify_quotation` hard-failure enforcement on every answer path that
accepts an externally supplied quotation.

## Implementation units

| Plan | Title | Status |
|---|---|---|
| 02-01 | Edition upstream identity, declared license, and primary/default designation | complete |
| 02-02 | Richer synthetic fixture, companion synthetic reference, identity fixture | complete |
| 02-03 | Operator integrity evidence (`qai quran verify`), reference path, identity snapshot | complete |
| 02-04 | Canonical-write fence completion, approval token, importer audit, rollback rejection | complete |
| 02-05 | Quotation hard-failure wiring (shared mapping, CLI verb, HTTP enforcement) | complete |
| 02-06 | Translation-layer completion (additive hash, verbatim license, negative test) | complete |
| 02-07 | Owner gates, coverage reconciliation, evidence of record, task closure | complete |

## Acceptance criteria → roadmap success criteria

| AC | Roadmap success criterion | Evidence |
|---|---|---|
| AC-1 | Operator import through staging → validation → atomic activation with rollback | `crates/application/tests/quran_import.rs`; `crates/storage-sqlite/tests/quran.rs`; `crates/cli/tests/quran/read_flow.trycmd` |
| AC-2 | Byte-exact `surah:ayah` lookup with a pinned edition reference | `crates/application/tests/quran_reader.rs`; `crates/quran-core/tests/reference_grammar.rs`; `read_flow.trycmd` |
| AC-3 | Corpus integrity checks (counts, addressing, Unicode, checksums, round-trip, reference comparison) pass | `docs/06-progress/corpus-integrity-report.json`; `crates/cli/tests/corpus_integrity.rs`; `qai quran verify` |
| AC-4 | Canonical tables reject all non-approved writes; importer has no code path to canonical tables | `migrations/sqlite/0021_canonical_write_fence.up.sql`; `crates/storage-sqlite/tests/quran.rs`; `crates/quran-corpus/tests/import_path.rs`; `crates/provenance/src/lib.rs` |
| AC-5 | Every quotation verifies via `verify_quotation`; mismatch is a hard failure | `crates/cli/tests/quran/verify_quotation.trycmd`; `crates/citations/src/lib.rs`; `crates/application/src/quran_tools.rs`; `crates/server/src/api.rs` |
| AC-6 | `REQ-data-separation-layers`: a translation can never reach a canonical slot | `crates/application/tests/quran_translation.rs#translation_cannot_reach_a_canonical_slot` |

## Owner gates (block real canonical activation)

`OD-01` (dataset identity + license), `OD-02` (named reviewer + `verified_by`), and
`OD-03` (independent reference corpus + signer) remain 🔴. They are recorded — never
closed by an agent — in `docs/05-followups/phase-02-owner-gates.md`, with the closing
action and command for each.

## Evidence of record

- `docs/06-progress/corpus-integrity-report.json` — committed six-family integrity artifact.
- `docs/06-progress/phase-02-evidence.md` — criterion → command → observed result, the
  six spec-less edge-probe dispositions, and the D-15 read-path exemption.
- `docs/05-followups/phase-02-owner-gates.md` — blocked owner gates plus the coverage shortfall.

## Completion — 2026-09-25

**Status: Completed.** Closed by plan 02-07 after the gate evidence and the phase
evidence document existed. The task is complete against the acceptance criteria
above; **real canonical activation remains blocked** by the owner gates
OD-01/OD-02/OD-03.

Recorded command outcomes (observed on this tree, 2026-09-25):

| Command | Observed |
|---|---|
| `cargo test -p application --test quran_import --test quran_reader --test quran_translation --test quran_verification --test quran_doctor --test quran_tools` | 54 passed, 0 failed |
| `cargo test -p storage-sqlite --test quran` | 10 passed, 0 failed |
| `cargo test -p cli --test quran --test doctor_json` | quran 7 passed / 2 failed (documented legacy baseline: `search.trycmd` / `normalize.trycmd`); doctor_json 2 passed |
| `cargo test -p server --test api` | 18 passed, 0 failed |
| `cargo test -p cli --test corpus_integrity` | 4 passed, 0 failed |
| `cargo run -q -p xtask -- coverage-gate lcov.info` | OK — quran-core 91.67% (min 90%), quran-corpus 92.55% (min 90%), citations 79.27% (min 79%; 85% floor shortfall recorded) |
| `cargo run -q -p xtask -- arch-check` | OK — no forbidden dependency edges |
| `cargo run -q -p xtask -- migrate-check` | OK — 21 migration(s) ordered; checksums stable |

The two red snapshots belong to the legacy search/normalization phase (this
roadmap's Phase 3) and must be re-checked on a quiet worktree before any phase-gate
read; see `docs/06-progress/phase-02-evidence.md` §4. No owner gate is closed by
this record.

---

