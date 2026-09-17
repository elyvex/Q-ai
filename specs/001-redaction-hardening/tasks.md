# Tasks: Global Redaction Tracing Layer + Secret-Leak Sentinel Suite

**Input**: Design documents from `/specs/001-redaction-hardening/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/redaction-api.md, quickstart.md

**Tests**: Tests are included per spec (FR-006/007 mandate a sentinel suite). Test-first ordering is enforced within each story.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story. US1 (P1) is MVP.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Rust workspace. Crate-relative paths under `crates/`. Tests live in
`crates/*/tests/` or in-crate `#[cfg(test)]` modules.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: New module + allowlist amendment that every story depends on.

- [x] T001 [P] Add `observability → domain` line to `xtask/allowlist.toml` (justified in plan.md Complexity Tracking; record as `workspace = { allow = ["domain"] }`)
- [x] T002 [P] Add `domain` path dependency to `crates/observability/Cargo.toml`
- [x] T003 Create `crates/domain/src/redaction.rs` module with `pub const REDACTED_MARKER`, `pub fn is_secret_key(name: &str) -> bool`, `pub fn redact_json_value(value: &mut serde_json::Value) -> usize`, `pub fn redact_text(input: &str) -> Cow<'_, str>` per contracts/redaction-api.md §1
- [x] T004 Add `pub mod redaction;` to `crates/domain/src/lib.rs`
- [x] T005 [P] Write unit tests in `crates/domain/src/redaction.rs` covering Rule A (secret-named keys → marker), Rule B (free-text `key[:=]value`), Rule C (URL userinfo), nested recursion, and the borrowed-original zero-alloc hot path

**Checkpoint**: `cargo test -p domain redaction` green; `cargo run -p xtask -- arch-check` green with the new allowlist line.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented.

- [x] T006 Refactor `crates/audit/src/lib.rs::redact_audit_value` to delegate to `domain::redaction::redact_json_value` (behavior-preserving; existing `redaction_strips_secrets` test must pass unchanged) per contracts §2
- [x] T007 Add `InitOptions { format: Format, redact_secrets: bool }` struct and `pub fn init_with_options(opts: InitOptions) -> Shutdown` to `crates/observability/src/lib.rs`; keep `pub fn init(format: Format)` delegating with `redact_secrets: true` per contracts §4
- [x] T008 Add redacting `FormatFields` wrapper to the fmt layer in `crates/observability/src/lib.rs` applying Rule A to recorded field names when `redact_secrets` is true (Text + JSON) per contracts §4 — DELIVERED AS `RedactingWriter` + `redact_log_fields` (see sync note S1)
- [x] T009 [P] Add `pub use domain::redaction;` re-export to `crates/application/src/lib.rs` per contracts §5
- [x] T010 [P] Thread the flag in `crates/application/src/lib.rs::run` via `init_with_options(InitOptions { format: Format::Text, redact_secrets: cfg.logging.redact_secrets })` per contracts §5

**Checkpoint**: Foundation ready — all four emission surfaces can now route through shared redaction. `cargo test -p audit` green; `cargo test -p observability` green.

---

## Phase 3: User Story 1 - Global Log Redaction Layer (Priority: P1) MVP

**Goal**: Secrets never reach log output via the central tracing/logging pipeline.

**Independent Test**: `sentinel_absent_from_traced_fields` + `sentinel_absent_from_secret_typed_event_values` pass in `crates/testkit/tests/secret_leak.rs`.

### Tests for User Story 1 (test-first per FR-006)

- [x] T011 [P] [US1] Write `sentinel_absent_from_traced_fields` test in `crates/testkit/tests/secret_leak.rs` — emit events with secret-named fields holding the sentinel via a test writer, capture output, assert zero bytes; verify FAILS before layer exists
- [x] T012 [P] [US1] Write `sentinel_absent_from_secret_typed_event_values` test in `crates/testkit/tests/secret_leak.rs` — `Secret::new(SENTINEL)` logged via Debug/Display, assert zero bytes (regression guard for F1); verify FAILS before layer exists

### Implementation for User Story 1

- [x] T013 [US1] Build the redacting `FormatFields` wrapper referenced by T008 into the fmt subscriber in `crates/observability/src/lib.rs` (depends on T005 helper + T007/T008 contracts) — DELIVERED AS `RedactingWriter` (see sync note S1)
- [x] T014 [US1] Verify both US1 tests pass: `cargo test -p testkit --test secret_leak`

**Checkpoint**: User Story 1 independently functional — sentinel bytes absent from all traced output. MVP deliverable.

---

## Phase 4: User Story 2 - Error Rendering Redaction (Priority: P2)

**Goal**: Secrets never leak through human-readable or JSON error/diagnostic renderings.

**Independent Test**: `sentinel_key_value_pairs_scrubbed_from_free_text` passes, covering rendered diagnostics.

### Tests for User Story 2 (test-first per FR-006)

- [x] T015 [P] [US2] Write `sentinel_key_value_pairs_scrubbed_from_free_text` test in `crates/testkit/tests/secret_leak.rs` — Rule B shapes (`api_key={SENTINEL}`, `password: {SENTINEL}`) through `redact_text` AND through rendered `domain::Diagnostic` (human + JSON); zero bytes; benign "approval token issued" preserved; verify FAILS before renderers scrub

### Implementation for User Story 2

- [x] T016 [US2] Apply `redact_text` to each free-text field (message, location, affected_resource, remedy, next_command) at render time in `crates/domain/src/diagnostic.rs::render_human` and `render_json` per contracts §3 (stored struct NOT mutated; existing renderer tests unchanged)
- [x] T017 [US2] Verify US2 test passes: `cargo test -p testkit --test secret_leak sentinel_key_value_pairs_scrubbed_from_free_text` and `cargo test -p domain diagnostic` unchanged

**Checkpoint**: User Stories 1 AND 2 both work independently — errors scrub credential patterns in both output forms.

---

## Phase 5: User Story 3 - CLI Inspection Outputs (Priority: P3)

**Goal**: Secrets never leak through `config show` or `doctor --json`.

**Independent Test**: `sentinel_userinfo_scrubbed` + `sentinel_absent_from_config_show_and_doctor_json` pass.

### Tests for User Story 3 (test-first per FR-006)

- [x] T018 [P] [US3] Write `sentinel_userinfo_scrubbed` test in `crates/testkit/tests/secret_leak.rs` — Rule C URL shapes through helper + doctor/config render paths; zero bytes
- [x] T019 [P] [US3] Write `sentinel_absent_from_config_show_and_doctor_json` test in `crates/testkit/tests/secret_leak.rs` — render-path coverage for US3 incl. schema validation of the doctor document — HELPER-LEVEL ONLY (see sync note S2)
- [x] T020 [P] [US3] Write `redaction_switch_off_passes_values_through` test in `crates/testkit/tests/secret_leak.rs` — `redact_secrets=false` omits the layer (documents escape hatch; asserts opt-out works)

### Implementation for User Story 3

- [x] T021 [US3] Add redact-then-print to `handle_config_show` in `crates/cli/src/lib.rs` — serialize Config → `redact_json_value` → print (JSON); `redact_text` over Debug output (text) per contracts §5 (depends on T009 re-export) — LANDED 2026-09-17 via `render_config_show` helper (`Config` → value → scrub → print; Debug → `redact_text`)
- [x] T022 [US3] Add redact-then-print to `doctor_report` in `crates/cli/src/doctor.rs` — JSON doc scrubbed before assembly; schema shape preserved per contracts §5 — LANDED 2026-09-17 (scrub merged doc + human text in `doctor_report`; same boundary applied to `run_checks` JSON/human print branches)
- [x] T023 [US3] Verify all US3 tests pass: `cargo test -p testkit --test secret_leak sentinel_userinfo_scrubbed sentinel_absent_from_config_show_and_doctor_json redaction_switch_off_passes_values_through` — PASS plus new CLI-level tests `config_show_scrubs_credential_shaped_values` + `doctor_report_scrubs_credential_shaped_config_values` (`cargo test -p cli --lib` 16/16; doctor test asserts pre-scrub echo so it cannot go vacuous)

**Checkpoint**: All three user stories independently functional.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories.

- [ ] T024 Run full gate sweep: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo run -p xtask -- arch-check`, `cargo run -p xtask -- migrate-check`, `qai db status && qai doctor` (SC-003) — OPEN; partial evidence 2026-09-17 (see sync note S3)
- [x] T025 Add `//!` module docs to `crates/domain/src/redaction.rs` summarizing Rules A/B/C and the borrow-on-clean contract
- [x] T026 [P] Update `docs/03-plan/phases/phase-00-foundation/tasks.md` — flip P0-T16 to ☑, record exit-gate ritual evidence for AC-P0-05 — DONE (P0-T16 already ☑ in the foundation board)
- [x] T027 [P] Update `CHANGELOG.md` under the next release: redaction hardening entry — DONE (observability `InitOptions` + `RedactingWriter` entry present)
- [x] T028 Archive OTLP span-field scrubbing as a recorded follow-up in `docs/05-followups/open-questions.md` (spec Assumptions — deferred per D6) — DONE (OTLP span-field scrubbing entry present)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately.
- **Foundational (Phase 2)**: Depends on Setup (T003/T004 helper + T001/T002 allowlist). BLOCKS all user stories.
- **User Stories (Phase 3–5)**: All depend on Foundational. US1 → US2 → US3 priority order for MVP; US2 and US3 can parallelize after US1 shares the helper.
- **Polish (Phase 6)**: Depends on all desired user stories being complete.

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2). No dependencies on other stories.
- **User Story 2 (P2)**: Can start after Foundational. Integrates with shared helper (T003) and US1's observability layer but independently testable.
- **User Story 3 (P3)**: Can start after Foundational. Integrates via application re-export (T009) but independently testable.

### Within Each Story

- Tests (if included) MUST be written and FAIL before implementation (T011/T012 → T013; T015 → T016; T018/T019/T020 → T021/T022).
- Models before services; core before integration.

### Parallel Opportunities

- T001, T002, T005 [P] in Setup can run together.
- T009, T010 [P] in Foundational can run together (after T006/T007/T008).
- T011, T012 [P] US1 tests together; T018, T019, T020 [P] US3 tests together.
- T026, T027 [P] in Polish together.
- US1 → US2 → US3 can proceed in priority order or partially in parallel once Phase 2 lands.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1 (T011–T014)
4. **STOP and validate**: run US1 sentinel tests; log redaction is MVP.
5. Then US2 (P2) and US3 (P3) at operator discretion.

### Incremental Delivery

1. Setup + Foundational → helper + layer wired.
2. US1 → log redaction MVP.
3. US2 → diagnostic renderers scrub.
4. US3 → CLI inspection outputs safe.
5. Polish → gates + docs + rollup.

---

## Notes

- [P] tasks = different files, no dependencies.
- [Story] label maps task to specific user story for traceability.
- Each user story independently completable and testable.
- Verify tests fail before implementing.
- Commit after each task or logical group; stop at any checkpoint to validate the story independently.
- Avoid: vague tasks, same-file conflicts, cross-story dependencies that break independence.

---

## Sync note (2026-09-17, spec-kit bootstrap reconciliation)

Source-of-truth hierarchy applied (code > tests > ledgers > docs).
`cargo test -p testkit --test secret_leak` 9/9 green and
`cargo fmt --all -- --check` clean verified this date.

- **S1 — mechanism deviation (T008/T013):** plan and contracts specify a
  redacting `FormatFields` wrapper applying Rule A to recorded tracing
  field names. Shipped code (`crates/observability/src/lib.rs`) uses a
  `RedactingWriter` wrapping stderr plus `pub fn redact_log_fields`,
  which scans formatted output for `key=value` pairs over
  `api_key|apikey|api-key|password|secret|token|credential` (`=` only;
  no `:` shape, no structured JSON field-name path). US1 story goal
  (secrets scrubbed from log emission) is met through this mechanism;
  the `FormatFields` design was superseded. Owner to ratify or redirect.
- **S2 — US3 CLOSED 2026-09-17 (T021/T022/T023):** `render_config_show`
  (`crates/cli/src/lib.rs`) routes JSON through `redact_json_value` and
  Debug text through `redact_text` via `application::redaction` (no new
  workspace edge); `doctor_report` scrubs the merged doc + human text and
  `run_checks` scrubs its JSON/human print branches (same file, same
  boundary). CLI-level sentinel tests added in-crate
  (`config_show_scrubs_credential_shaped_values`,
  `doctor_report_scrubs_credential_shaped_config_values`, the latter
  asserting pre-scrub echo so it cannot go vacuous); `cargo test -p cli
  --lib` 16/16 green. Live smoke on a fresh `QAI_DATA_DIR`: `config show
  --json` parses with all 9 top-level keys, `doctor --json` is a single
  `{"checks": [...]}` doc (26 checks). Known warts recorded, not fixed
  here: (a) Rule A `contains`-matching replaces the whole `secrets`
  subtree with the marker in `--json` mode (backend name hidden;
  fail-closed; live output shows `"secrets": "***REDACTED***"`), while
  text mode keeps it visible — JSON/text asymmetry; (b) `handle_config_get`
  still prints raw Debug (outside T021 scope); (c) `run_quran_checks` is a
  dead public API with unscrubbed prints (no in-tree callers) — scrub on
  first live use. Owner follow-ups for matcher precision, not blockers.
- **S3 — T024 partial evidence 2026-09-17:** `cargo fmt --all -- --check`
  clean, `cargo clippy --workspace --all-targets -- -D warnings` exit 0,
  `cargo xtask arch-check` OK, `cargo xtask migrate-check` OK (16,
  checksums stable), `cargo test -p cli --lib` 16/16,
  `cargo test -p testkit --test secret_leak` 9/9,
  `cargo test -p cli --test doctor_json` 1/1, live `db migrate` +
  `config show` (json+text) + `doctor --json` smoke on fresh temp dir.
  NOT run: full `cargo test --workspace` (concurrent-writer churn in
  `testkit`/phase-0 files plus the known flaky `jobs` timing test make a
  workspace-wide claim unreliable from this session), `qai db status`,
  `qai doctor` text mode. A transient `testkit` lib-test compile error
  appeared mid-session from another writer's half-landed edit and cleared
  on their next edit — not caused by, and untouched by, this session.
