# Quickstart: Storage Repository Contracts Validation

**Feature**: `027-storage-repository-contracts` | **Date**: 2026-09-18

Validation scenarios proving the contracts hold. No new behavior is built — every scenario asserts existing code against the spec. Details in [spec.md](spec.md); row shapes in [data-model.md](data-model.md).

## Prerequisites

```bash
cargo test -p storage --lib
cargo run -p xtask -- arch-check
```

## Scenario 1 — Trait boundary is backend-free (SC-001)

```bash
cargo run -p xtask -- arch-check
```

- **Expect**: `storage` → `domain` only; no `sqlx`, pool, or row types in application-callable signatures; `application::db` compiles against traits only.

## Scenario 2 — Stub defaults fail closed (US1-AC2, edge cases)

```bash
cargo test -p storage --lib
```

- **Expect**: calling any unimplemented repository method on stub defaults returns `QAI-DB-0009`, never panics, never I/O; `all_codes_unique` and `retryable_variants` green.

## Scenario 3 — Workflow atomicity with a fake UnitOfWork (SC-002)

- **Expect** (fake-`UnitOfWork` tests): mid-workflow failure in `record_source_activation` / `record_source_deactivation` / `record_provenance_write` leaves zero partial state — no generation without its event, no `Deprecated` without its tombstone; `relay_outbox_once` claims then dispatches exactly the claimed set; duplicate idempotency keys are safe no-ops.

## Scenario 4 — Single gated canonical path (SC-003)

- **Expect**: trait-surface review (or negative test) shows no canonical-write method outside `activate_edition` / `rollback_edition` / `set_edition_status`; activation flips pointer +1 generation, consumes staging, records approving identity — exercised via `storage-sqlite` staging/activation suites and `qai quran import/activate` smoke.

## Scenario 5 — Error-code uniformity (SC-004)

- **Expect**: each of the nine codes round-trips through `code()`, `render_human`, `render_json` with documented remedy/retryability; misuse matrix (unexpected-`from` transition, duplicate enqueue, corrupt cache row → miss, lock contention → `StorageBusy`) behaves per data-model validation rules.

## Scenario 6 — Disjointness with 004 (SC-005)

- **Expect**: every contract statement names a `storage`-crate item; no pools/SQL/migration-file claims duplicated from `specs/004-storage-sqlite/spec.md`.
