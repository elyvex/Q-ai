# Implementation Plan: Storage Repository Contracts

**Branch**: `027-storage-repository-contracts` | **Date**: 2026-09-18 | **Spec**: [spec.md](spec.md)

**Input**: Reverse specification of the implemented `storage` crate contracts from `/specs/027-storage-repository-contracts/spec.md`

**Note**: This is a reverse-spec plan: the contracts are implemented. The work planned here is verification, conformance hardening, and spec-disjointness — not new storage design. No new crates, no new migrations, no engine changes.

## Summary

Verify and pin the `storage` crate's trait boundary (seven `Send + Sync` repository traits behind `Database`/`ReadTx`/`UnitOfWork`), the single-gated Quran canonical/staging lifecycle, the atomic outbox/generation/tombstone workflows, and the nine stable `QAI-DB-nnnn` error codes — and prove the trait/implementation split against `storage-sqlite` plus disjointness from spec 004. Work items are conformance tests (fake-`UnitOfWork` atomicity, stub-default, code round-trip), `arch-check` layering proof, and contract documentation.

## Technical Context

**Language/Version**: Rust 1.97.1, edition 2024, workspace resolver 2 (`rust-toolchain.toml` pinned)

**Primary Dependencies**: Existing only — `domain` (sole domain dependency), `serde`, `thiserror`, `async-trait`; `storage-sqlite` (conformance subject, specified in 004); `application::db` (trait consumer). No new dependencies.

**Storage**: No schema changes. SQLite backend is the conformance reference; trait surface must stay backend-agnostic (Postgres-portable).

**Testing**: `cargo test -p storage --lib` (row round-trip, backend display, health, `all_codes_unique`, `retryable_variants`); new fake-`UnitOfWork` atomicity tests for `workflows.rs` helpers; `storage-sqlite` integration suites as conformance evidence; `cargo run -p xtask -- arch-check` for layering (`storage` → `domain` only, no `sqlx` leakage); `migrate-check` untouched (no migrations).

**Target Platform**: Local-first; contracts are platform-independent.

**Project Type**: Library-crate contract verification inside the existing Cargo workspace (no new crates).

**Performance Goals**: No performance changes; `StorageBusy` bounded-retry behavior asserted, never hangs.

**Constraints**: Trait methods keep stub defaults (`StorageUnavailable`, never panic/I/O); no SQL/driver types cross into `storage` signatures; `ReadTx` stays read-only; canonical writes only via `activate_edition`/`rollback_edition`/`set_edition_status`; generations monotonic per scope; tombstone-before-visibility ordering in deactivation; duplicate idempotency keys → `Conflict` safe no-op.

**Scale/Scope**: 5 files (`lib.rs`, `repository.rs`, `quran.rs`, `workflows.rs`, `error.rs`); 7 repository traits; 4 workflow helpers; 9 error codes; 0 migrations; 0 new crates.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Canonical Text Integrity**: PASS — plan asserts the single gated write path (`activate_edition`/`rollback_edition` + human-gated `set_edition_status`); no new canonical-write surface; insert-only/trigger-guarded properties verified, not redesigned.
- **II. Layered Trust and Provenance**: PASS — staging → validation → approval → activation → audit ordering pinned by contract tests; Layer A–E row separation asserted (glosses/translations/derived forms never canonical).
- **III. Traceability and Reproducibility**: PASS — provenance/audit repository contracts (`record_review`, `verify_chain` with gap reporting) covered by conformance tasks.
- **IV. Scholarly Honesty**: PASS — no synthesis or grading logic in scope; attribution rows carried verbatim.
- **V. Test-First and Quality Gates**: PASS — Red-Green-Refactor; full gate set; fake-`UnitOfWork` tests written first.
- **VI. Local-First Security, Deny-by-Default**: PASS — no new network/auth surface; `ReadTx` read-only discipline asserted.
- **VII. Simplicity and Architecture Discipline**: PASS — no new crates; `storage` → `domain` layering enforced by `arch-check`; SQLite-adjacent contracts; no migrations (append-only `0001`–`0016` untouched).
- **VIII. Multi-Edition Governance**: PASS — edition/version/slug separation in row types asserted; no cross-edition merge path in trait surface.

*Post-design re-check (Phase 1): no new violations — design documents existing contracts and verification only.*

## Project Structure

### Documentation (this feature)

```text
specs/027-storage-repository-contracts/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
│   ├── repository-surface.md  # Trait method inventory per family
│   └── error-codes.md         # QAI-DB-0001…0009 table + rendering rules
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/
├── storage/src/{lib.rs,repository.rs,quran.rs,workflows.rs,error.rs}  # VERIFY ONLY (tests may be added inline)
├── storage-sqlite/{src,tests}/     # conformance evidence, no contract changes
├── application/src/db.rs           # trait-boundary usage reference, unchanged
└── xtask/src/arch.rs               # layering proof, unchanged
```

**Structure Decision**: No source layout changes. Verification lives in `storage`'s existing unit tests plus new fake-based workflow tests; conformance evidence reuses `storage-sqlite` integration suites. The `application → storage-traits → storage-sqlite` layering is asserted, not altered.

## Complexity Tracking

> No constitution violations; nothing to justify. This section intentionally left empty.
