---
description: "Task list for feature implementation"
---

# Tasks: Storage Repository Contracts (027)

**Input**: Design documents from `/specs/027-storage-repository-contracts/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: This is a reverse-spec/verification feature (plan.md): the contracts are implemented; all tasks are conformance or negative tests plus gate runs. Test tasks are the substance of this feature and are explicitly included, per plan.md Testing section.

**Organization**: Tasks grouped by user story (US1–US5 from spec.md). All work lives in `crates/storage/` (tests added inline) with `storage-sqlite` and `xtask` providing evidence — no contract changes, no new crates, no migrations.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to

## Path Conventions

```text
crates/storage/src/{lib,repository,quran,workflows,error}.rs   # tests added inline
crates/storage-sqlite/{src,tests}/                              # conformance evidence
crates/xtask/src/                                               # arch-check
```

## Phase 1: Setup

- [X] T001 Verify working tree state: `cargo check -p storage`, `cargo test -p storage --lib`, `cargo run -p xtask -- arch-check` all green before adding tests (record baseline counts)
- [X] T002 [P] Read current tests in `crates/storage/src/{error,lib,repository}.rs` to confirm existing coverage (`all_codes_unique`, `retryable_variants`, row round-trip, backend display, health) and identify gaps vs quickstart.md scenarios

## Phase 2: Foundational

- [X] T003 Add a minimal in-memory fake `UnitOfWork`/repository test harness in `crates/storage/src/workflows.rs` tests module (shared by US1/US3): stub trait impls recording call order, injectable failure into any step, verifying commit semantics without SQLite

## Phase 3: User Story 1 — Typed Send+Sync repository contracts behind one atomic UnitOfWork (P1) 🎯 MVP

**Goal**: Prove the trait boundary is backend-free and stub defaults fail closed.

**Independent Test**: `cargo test -p storage --lib` + `cargo run -p xtask -- arch-check` green.

### Implementation for User Story 1

- [X] T004 [US1] Add test in `crates/storage/src/lib.rs`: stub-default `Repository` method call returns `Err(StorageUnavailable)` (`QAI-DB-0009`), never panics, no I/O (US1-AC2; edge cases: `ReadTx::query_list` stub unavailable, `query_one` missing row → `Ok(None)`)
- [X] T005 [US1] Add fake-`UnitOfWork` test in `crates/storage/src/workflows.rs`: `workflows.rs` helpers drive compiles and run with no SQLx/pool/row types in signatures (US1-AC1)
- [X] T006 [US1] Run `cargo run -p xtask -- arch-check` and record green output in task doc: `storage` → `domain` only, no `sqlx` leakage (SC-001)

**Checkpoint**: Trait boundary provable without backend. MVP complete.

## Phase 4: User Story 3 — Activation/deactivation workflows with atomic outbox (P1)

**Goal**: Prove workflow helper ordering and single-commit atomicity with a fake UnitOfWork.

**Independent Test**: Fake-UnitOfWork tests assert call order; mid-workflow failure leaves zero partial state.

### Implementation for User Story 3

- [X] T007 [US3] Add fake-UnitOfWork tests in `crates/storage/src/workflows.rs` for `record_source_activation`: assert call order generation → enqueue `source_activated:{source_version_id}` → `transition_state(Indexing → Active)`; rollback (injected failure) leaves zero partial state (FR-007)
- [X] T008 [US3] Add fake-UnitOfWork tests for `record_source_deactivation`: assert tombstone `Pending` written BEFORE generation allocation, `source_deactivated` enqueue, then `transition_state(Active → Deprecated)`; mid-failure → zero partial state, no `Deprecated` without tombstone (FR-008; edge case: tombstone-write-fail rollback)
- [X] T009 [US3] Add fake-UnitOfWork tests for `record_provenance_write` and `relay_outbox_once`: provenance insert → generation → `provenance_written` (key bound to generation id); relay claims up to `limit`, marks exactly the claimed set dispatched, zero pending or `limit=0` → empty vec, dispatch-only semantics (FR-009, FR-010)
- [X] T010 [US3] Add duplicate-idempotency-key test in `crates/storage/src/workflows.rs`: duplicate `(operation, idempotency_key)` enqueue yields `Conflict` treated as idempotent no-op at call site (edge case)

## Phase 5: User Story 2 — Quran canonical/staging lifecycle with single gated write path (P1)

**Goal**: Prove no row-level canonical-write method exists outside the three mutators, and staging/lifecycle contract shape.

**Independent Test**: `cargo test -p storage --lib` green + negative trait-surface test; `storage-sqlite` integration suites already cover SQLite conformance.

### Implementation for User Story 2

- [X] T011 [P] [US2] Add a fake `QuranRepository` test in `crates/storage/src/quran.rs` verifying the caller order stage → validate → activate and that staged reads are deterministic (list_stg_* ordering pins) and run-scoped (FR-006)
- [X] T012 [US2] Add a negative trait-surface test/audit in `crates/storage/src/quran.rs` (or a doc-pinned audit note in the task doc): enumerate all mutator methods and prove no canonical-write method exists outside `activate_edition`, `rollback_edition`, `set_edition_status` (FR-005; SC-003)
- [X] T013 [US2] Run `storage-sqlite` quran staging/activation integration tests as conformance evidence and record result in the task doc

## Phase 6: User Story 4 — Jobs, audit, provenance, settings contracts (P2)

**Goal**: Contract-level fake tests for lease semantics, audit chain verification, storage traits.

**Independent Test**: fake verifies `claim_next` state filter, `heartbeat` non-holder failure, `reap_expired_leases` exact ids, `verify_chain` gap reporting.

### Implementation for User Story 6

- [X] T014 [US4] Add fake-impl tests in `crates/storage/src/repository.rs` for JOB lease semantics: `claim_next` only returns Queued/Interrupted/Checkpointed-due; `heartbeat` by non-holder → false; `reap_expired_leases` returns exactly expired ids, unexpired untouched (FR-012)
- [X] T015 [P] [US4] Add fake-impl test in `crates/storage/src/repository.rs` for AUDIT `verify_chain`: valid + expected next sequence/hash; tampering lists exact gap sequences (US4-AC2)
- [X] T016 [P] [US4] Add fake-impl test in `crates/storage/src/repository.rs` for SOURCE `transition_state` unexpected-`from` → rejected (`Conflict`/`ConstraintViolation`), never silently applied (edge case)

## Phase 7: User Story 5 — Stable QAI-DB error codes (P3)

**Goal**: Confirm rendering round-trips and JSON rendering shape.

**Independent Test**: `all_codes_unique`, `retryable_variants` green; review `render_json` shape matches spec field list.

### Implementation for User Story 5

- [X] T017 [P] [US5] Extend tests in `crates/storage/src/error.rs`: render_json carries code/summary/why/location/remedy/next_command/retryable; retryability = exactly {Conflict, StorageBusy, StorageUnavailable}; `code()`-remedy consistency per contracts/error-codes.md (FR-013; SC-004)

## Phase 8: Polish & Verification

- [ ] T018 Run quickstart.md validation: all 6 scenarios (arch-check, stub-default, workflow atomicity, single-gated canonical path, error-code uniformity, disjointness with 004) — document outcomes in the task doc
- [ ] T019 Add contract-only tests to workspace gates: `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
- [ ] T020 Storage crate tests don't disrupt storage-sqlite's integration suites; `migration files 0001–0016` untouched (`ls migrations/sqlite/ | wc -l` unchanged)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — START immediately
- **Foundational (Phase 2)**: Depends on Setup
- **US1 (Phase 3)**: Depends on Phase 2
- **US3 (Phase 4)**: Depends on Phase 3
- **US2 (Phase 5)**: Depends on Phase 2 (Quran trait tests independent of US1/US3 fakes, but sequential for review clarity)
- **US4 (Phase 6)**: Depends on Phase 2
- **US5 (Phase 7)**: Independent of US3/US4/US2 — depends on Phase 2
- **Polish (Phase 8)**: Depends on all prior phases

### User Story Dependencies

- All stories depend only on the Phase 2 fake harness (no cross-story code dependencies)
- US1 → US3 in order only because US3 reuses US1's fake harness

### Parallel Opportunities

- T002, T003 are parallel-safe (read-only)
- T007–T010 sequential (same file `workflows.rs`)
- T011–T013 sequential (same file `quran.rs`), parallel with US4 tasks
- T014 + T015 + T016 + T017 parallelizable across different test modules/files
- T018–T020 sequential in final validation

---

## Parallel Example: User Story 1

```bash
# T005 and T006-A run when T004 done:
Task: "Add fake-UnitOfWork test in crates/storage/src/workflows.rs"
Task: "Run cargo run -p xtask -- arch-check"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1
4. STOP and VALIDATE: run `cargo test -p storage --lib` + `arch-check`

### Incremental Delivery

1. Setup + Foundational → base
2. Add US1 → validate trait boundary
3. Add US3 → validate atomicity fake tests
4. Add US2/US4/US5 → validate lifecycle/lease/error contract tests
5. Polish → run quickstart.md scenarios, workspace gates

---

## Notes

- Contract implementations already exist — this is verification-only. Do NOT change trait signatures, row shapes, error variants, or workflows.
- Test lib additions live inline in existing `#[cfg(test)]` modules; do not create a separate `tests/` dir in `storage`.
- `storage-sqlite` integration suites and `xtask arch-check` are evidence-only consumers; no modifications.
- If a conformance test reveals a mismatch with spec.md, STOP — record the gap in `docs/05-followups/` and do not patch implementation without a spec note.
