# Tasks: Search-Service Wiring (CLI + Server API + SSE)

**Input**: Design documents from `/specs/030-search-service-wiring/` (spec.md, plan.md, research.md, data-model.md, contracts/cli-search.md, contracts/http-search.md, quickstart.md)

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md — all present. Constitution v1.2.0 applies (V: test-first mandatory for behavior changes — test tasks included on that basis; full gate set per task group).

**Tests**: Included per constitution Principle V (Red-Green-Refactor mandatory). Write each test first, confirm it fails, then implement.

**Organization**: Grouped by user story. US1 (CLI) and US2 (HTTP) are both P1 and independent (different crates/files) — parallelizable after Phase 2.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Exact file paths in every description

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Baseline verification before any wiring work

- [ ] T001 Verify clean baseline: `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`, `cargo test --workspace` all green on the untouched tree
- [ ] T002 [P] Verify gate tooling baseline: `cargo run -p xtask -- arch-check` and `cargo run -p xtask -- migrate-check` pass

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared error-mapping coverage that both surfaces depend on; no service/engine changes

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T003 Extend CLI error mapper `storage_error_code` in crates/cli/src/lib.rs to cover `QAI-IDX-*` / `QAI-NORM-*` search codes → exit codes (usage 2, validation 3, policy 4, not-found 5, conflict 6, internal 70) per data-model.md Entity 4
- [ ] T004 [P] Verify server `Diagnostic → HTTP status` conversion in crates/server/src/api.rs covers search codes (400/404/409/429/5xx) per data-model.md Entity 4; extend only if a code is unmapped

**Checkpoint**: Foundation ready — error mapping proven for both surfaces; user stories can now proceed in parallel

---

## Phase 3: User Story 1 — Search from the terminal (Priority: P1) 🎯 MVP

**Goal**: `qai quran search` verb reaching all five application services with JSON + human output, pagination, and typed errors

**Independent Test**: Run `qai quran search --query <text> --mode <mode> [--profile ...] [--limit N] [--offset M] [--json]` against a built index; exit 0 with paginated contracted hits, no server involved

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T005 [P] [US1] trycmd snapshot cases for `quran search --json` (normalized + exact) in crates/cli/tests/ covering hit shape, total/truncated, generation, trace
- [ ] T006 [P] [US1] trycmd misuse-matrix cases in crates/cli/tests/ (profile+rules → exit 2; unbuilt index → exit 5 with rebuild remedy; unknown profile → non-zero Diagnostic)

### Implementation for User Story 1

- [ ] T007 [P] [US1] Add `Search` variant with all flags (`--tool`, `--query`, `--edition`, `--profile`/`--rules` with `conflicts_with`, `--field`, `--match-mode`, `--phrase-mode`, `--slop`, `--allow-cross-ayah`, `--max-ayah-span`, `--regex-field`, `--pattern`, `--principal`, `--timeout-ms`, filters, `--limit`, `--offset`, `--explain`, `--highlight`) to `QuranAction` in crates/cli/src/quran.rs
- [ ] T008 [P] [US1] Implement `cmd_search` in crates/application/src/quran_cli.rs: assemble transport-neutral SearchRequest, validate (profile+rules exclusivity, ceilings: limit min 1000, timeout clamp 1..=10000), dispatch to the five `quran_search` services, return `CommandOutput` (full section-12 JSON + canonical-text-preserving human rendering)
- [ ] T009 [US1] Wire dispatch arm for `QuranAction::Search` in `handle_quran_async` in crates/cli/src/quran.rs (depends on T007, T008); confirm T005–T006 now pass

**Checkpoint**: US1 fully functional and testable independently — all five tools reachable from the terminal

---

## Phase 4: User Story 2 — Search over HTTP with the section-12 envelope (Priority: P1)

**Goal**: Five paged GET routes returning `Envelope<SearchEnvelope>` with ETag, conditional requests, and Diagnostic errors

**Independent Test**: `curl` the five routes against the loopback server; assert envelope shape, ETag presence, error-status mapping, and hit-set identity with CLI `--json` output — no CLI involvement

### Tests for User Story 2

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T010 [P] [US2] Server route tests in crates/server/tests/ for `GET /api/v1/quran/search/normalized` (envelope shape, ETag + `If-None-Match` short-circuit, meta edition/generation/reproducibility fields)
- [ ] T011 [P] [US2] Server error-mapping tests in crates/server/tests/ (unknown profile → 400; unbuilt index → 404; edition mismatch → 409; all with Diagnostic bodies, never 200-with-empty)

### Implementation for User Story 2

- [ ] T012 [US2] Add five GET routes (`/search/exact`, `/search/normalized`, `/search/phrase`, `/search/concatenated`, `/search/regex`) with query-param structs mirroring the CLI surface to `router()` in crates/server/src/api.rs, behind the existing loopback guard (no guard code changes)
- [ ] T013 [US2] Implement search handlers in crates/server/src/api.rs: param validation (profile|rules exclusivity → 400, ceilings), service dispatch with generation-keyed `cache_lookup`/`cache_store`, `Envelope<SearchEnvelope>` assembly via existing `json_response`/`etag_for`/`Meta` helpers (depends on T012); confirm T010–T011 now pass
- [ ] T014 [US2] CLI↔HTTP parity check: diff identical query/edition/profile/pagination across `qai quran search --json` and the HTTP route (references, spans, ordering, totals identical); record in quickstart validation

**Checkpoint**: US1 AND US2 both work independently; machine-readable search unblocks future GUI/agents

---

## Phase 5: User Story 3 — Normalization profile passthrough (Priority: P2)

**Goal**: Registry (`@version`-pinned) vs adhoc rule selection forwarded unchanged on both surfaces, with the serving `rule_set` reported

**Independent Test**: Same query with different `--profile`/`?profile=` and `--rules`/`?rules=` values yields distinct serving traces and rule sets; both-together rejected before index access

### Tests for User Story 3

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T015 [P] [US3] Profile-pinning tests (CLI trycmd + server test): `--profile L3.diacritics@<version>` serves the pinned version and output `rule_set` names exactly that profile
- [ ] T016 [P] [US3] Adhoc-rules tests (CLI trycmd + server test): `--rules N01,N03` serves with per-candidate verification semantics and adhoc trace; `--profile` + `--rules` together → exit 2 / HTTP 400 without index access

### Implementation for User Story 3

- [ ] T017 [US3] Build parsed selection into `NormalizedProfile::Registry(id, version?) | Adhoc(ids)` in crates/application/src/quran_cli.rs (CLI) and crates/server/src/api.rs (HTTP) with no new normalization logic — pure passthrough to service signatures (depends on T015–T016 passing after wiring)

**Checkpoint**: US3 behaviors verified on both surfaces; wiring adds zero normalization semantics

---

## Phase 6: User Story 4 — Stream large result sets over SSE (Priority: P2)

**Goal**: `GET /api/v1/quran/search/stream` emitting per-hit events plus exactly one terminal summary event

**Independent Test**: `Accept: text/event-stream` client collects events; each hit matches the paged hit schema; final summary carries exact total, truncated, warnings, checksum

### Tests for User Story 4

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T018 [P] [US4] SSE route test in crates/server/tests/: multi-page query yields N hit events in paged schema + exactly one terminal summary event (total, truncated, warnings, checksum)

### Implementation for User Story 4

- [ ] T019 [US4] Add `GET /api/v1/quran/search/stream` SSE handler in crates/server/src/api.rs (axum `Sse`, normalized-search param set, disconnect stops work without above-warning logs); confirm T018 passes

**Checkpoint**: Bounded-memory streaming path works independently of offset pagination

---

## Phase 7: User Story 5 — Bounded regex search through both surfaces (Priority: P3)

**Goal**: DFA-bounded regex over explicitly selected indexed fields via CLI flags and HTTP params, with timeout + per-principal rate-limit budgets and `regex_report` in output

**Independent Test**: Same regex query via CLI and HTTP yields identical `regex_report` content and identical budget rejections

### Tests for User Story 5

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T020 [P] [US5] Regex parity tests (CLI trycmd + server test): `--tool search_regex --regex-field text_bare --pattern <expr>` output includes `regex_report` identical across surfaces; pathological/over-budget pattern → CLI policy exit / HTTP 429/400, never backtracking execution

### Implementation for User Story 5

- [ ] T021 [US5] Thread `regex_field` (restricted to `INDEXED_FIELDS`), `pattern` (up-front `compile_dfa`), `principal` + `RateLimiter`, `timeout_ms` clamp through `cmd_search` in crates/application/src/quran_cli.rs and the regex HTTP handler in crates/server/src/api.rs; surface `principal`/budget params without implementing the Phase-7 policy engine (depends on T020 passing after wiring)

**Checkpoint**: All user stories independently functional; dangerous path carries explicit budgets on both surfaces

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Validation, regression gates, and docs that span all stories

- [ ] T022 [P] Empty-query and filter edge cases: whitespace/marks-only query → zero hits both surfaces; unsatisfiable filters → zero hits not errors; stale generation → mandatory `stale_index` advisory on both surfaces (tests + wiring fixes if gaps found)
- [ ] T023 [P] Non-loopback `--bind` refusal still applies with new routes enabled (manual + test per existing serve guard tests)
- [ ] T024 Run quickstart.md end-to-end (Scenarios 1–6) against a fresh `QAI_DATA_DIR` and record results
- [ ] T025 No-regression gate: engine suites + golden reference set + full gates (`cargo fmt --check`, `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `arch-check`, `migrate-check`) green versus pre-wiring baseline
- [ ] T026 Update README search-wiring limitation note (search CLI/HTTP now exposed) and CHANGELOG.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories
- **User Stories (Phases 3–7)**: All depend on Phase 2; US1/US2 parallelizable (different crates); US3 work overlaps US1/US2 files — sequence after or coordinate; US4/US5 depend on US2 HTTP scaffolding for the server side
- **Polish (Phase 8)**: Depends on all desired stories being complete

### Within Each User Story

- Tests MUST be written and FAIL before implementation (constitution V)
- CLI parse (quran.rs) and execution (quran_cli.rs) are parallelizable [P]; dispatch wiring (T009) depends on both
- Server tests before handlers; handlers before parity checks
- Story checkpoint before moving to next priority

### Parallel Opportunities

- T002 with T001; T004 with T003; T005 with T006; T007 with T008; T010 with T011; T015 with T016; T022 with T023
- US1 (Phase 3) and US2 (Phase 4) can run in parallel after Phase 2 (disjoint files: cli+quran_cli vs server api)
- US4 server work can parallelize with US3 once US2 routes exist

---

## Parallel Example: User Story 1

```bash
# Launch US1 tests together (different files, no dependencies):
Task: "trycmd snapshot cases for quran search --json in crates/cli/tests/"
Task: "trycmd misuse-matrix cases in crates/cli/tests/"

# Launch US1 parse + execution together (different files):
Task: "Add Search variant to QuranAction in crates/cli/src/quran.rs"
Task: "Implement cmd_search in crates/application/src/quran_cli.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T002)
2. Complete Phase 2: Foundational (T003–T004)
3. Complete Phase 3: US1 CLI search (T005–T009)
4. **STOP and VALIDATE**: run quickstart Scenario 1–2 + misuse matrix against built index
5. Demo: terminal search across all five tools

### Incremental Delivery

1. Setup + Foundational → error mapping ready
2. + US1 → terminal search (MVP)
3. + US2 → HTTP search + CLI↔HTTP parity (machine API)
4. + US3 → pinned/adhoc profile fidelity on both surfaces
5. + US4 → SSE streaming for large sets
6. + US5 → budgeted regex on both surfaces
7. + Polish → gates green, docs current

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to its user story for traceability
- No engine changes (`quran_search.rs` called, never modified), no new crates, no migrations — enforced by `arch-check` + `migrate-check` in T025
- Commit after each task or logical group; stop at any checkpoint to validate the story independently
- `checklists/contracts.md` (28 items, reviewer-owned) remains the requirements-quality gate for `/speckit-implement`; this task list does not modify checklist markers
