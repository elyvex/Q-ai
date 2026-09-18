# Feature Specification: application composition root

**Feature Branch**: `009-application-root`

**Created**: 2026-09-17

**Status**: Draft

**Input**: Module-level specification of the implemented `application` crate (crates/application/src/{lib,db,job_queue,audit_bridge,quran,quran_reader,quran_cli,quran_doctor,quran_forms,quran_index,quran_normalize,quran_search,quran_search_cache,quran_tools}.rs), derived from code truth on 2026-09-17. Reverse specification of what exists today. Code is the source of truth over plan docs.

**Constitution compliance**: `.specify/memory/constitution.md` v1.0.2, Principles I (I5/I7: human-gated `--yes` activation with same-transaction audit + atomic pointer flip), III (ToolResult + ReproducibilityData, generation-keyed caches), V (gates), VII (composition root — cross-crate access routes through here; `storage-sqlite` hides behind this boundary).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - One bootstrap for every entry point (Priority: P1)

CLI, server, and tools all boot through `run(cfg: Config) -> RunResult`: config in, telemetry subscriber (with `redact_secrets` flag), SQLite backend health-checked, job queue and audit bridge wired. No entry point assembles its own stack.

**Why this priority**: A single composition root prevents divergent wiring (e.g. a server that forgets redaction or health checks).

**Independent Test**: `cargo test -p application --lib` green (28 passed on 2026-09-17) including `run_health_checks_the_sqlite_backend`; full gate sweep per CONTRIBUTING.

**Acceptance Scenarios**:

1. **Given** a valid `Config`, **When** `run` executes, **Then** logging initializes with the configured `redact_secrets` value and the backend reports healthy before serving anything.
2. **Given** an unhealthy backend, **When** `run` executes, **Then** bootstrap fails fast instead of serving reads against a broken store.

---

### User Story 2 - Human-gated activation with atomic flip (Priority: P1)

Edition activation requires explicit human approval (`--yes`), runs the pointer flip + generation bump + audit event in one transaction, and rolls back symmetrically. The importer (in `quran-corpus`) can never activate — it holds no token.

**Why this priority**: Invariants I5/I7. Auto-activation would let partial or unreviewed imports become canonical truth.

**Independent Test**: CLI acceptance flow (`read_flow.trycmd`: migrate→import→activate→reads→v2→validate→diff→rollback) green; activation/rollback services covered in `quran.rs`/`quran_cli.rs`.

**Acceptance Scenarios**:

1. **Given** a staged edition at `ApprovalRequested` without `--yes`, **When** activation is attempted, **Then** it is refused (exit 4 policy family).
2. **Given** `--yes` with a valid staged edition, **When** activated, **Then** the `quran_active_edition` singleton flips, generation bumps, audit records, and caches keyed to the old generation go stale (never served).

---

### User Story 3 - Read tools that cannot fabricate (Priority: P1)

`quran.get_ayah` and `quran.get_context` (via `ReaderToolBackend` → `ToolRegistry`) serve only canonical retrieval through `QuranReaderService`, returning `ToolResult` + `ReproducibilityData`. Typed errors make fabrication unrepresentable: there is no code path that emits verse text from anything but the store.

**Why this priority**: Invariants I2/I6 + PRD §12 tool result contract. A tool that could emit model-memory text as quotation would break the core promise.

**Independent Test**: `tool_get_ayah`/`tool_get_context` tests green; `to_tool_error` maps every `ReaderError` to a namespaced `ToolError` (no unmapped arm).

**Acceptance Scenarios**:

1. **Given** `quran.get_ayah(2:255)`, **When** executed, **Then** the result carries edition + version + hash, normalization rules, and reproducibility data.
2. **Given** a missing location/edition, **When** executed, **Then** the tool returns `QAI-QUR-0306/0307`-family errors, never invented text.

### Edge Cases

- Reader context is structure-bounded with caps (`ContextSpec`/`boundary surah`): requests cannot fan out to unbounded windows.
- Generation-keyed LRU reader cache + search cache (`quran_search_cache.rs`, migration `0016` shape per DEV-08): cross-generation reads are impossible by key construction.
- `persist_citation` stores citations resolvable later (`resolve_stored` verdicts incl. Mismatch/LocationNotFound/EditionNotFound/AccessDenied).
- Doctor quran checks (`quran_doctor.rs`) are read-only; mutations never run inside doctor (Constitution VII).
- Normalization preview vs. rebuild paths share the pipeline but only rebuild persists (`quran_normalize.rs` vs `quran_forms.rs`).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Crate MUST provide async `run(cfg)` bootstrap wiring config → observability (`InitOptions{redact_secrets}`) → `SqliteDatabase` (+ health) → job queue → audit bridge → `RunResult` (`lib.rs` L32–63, `db.rs`, `job_queue.rs`, `audit_bridge.rs`).
- **FR-002**: Crate MUST provide `QuranReaderService` (`QuranReader` trait) with ayah/range/surah/division/token expansion, structure-bounded context with caps, and generation-keyed LRU caching (`quran_reader.rs` L296–348+).
- **FR-003**: Crate MUST gate activation/rollback on human approval with single-transaction pointer flip + generation bump + audit, plus char-level differ evidence (`quran.rs`, `quran_cli.rs`).
- **FR-004**: Crate MUST expose `ReaderToolBackend`/`registry`/`resolver`, `tool_get_ayah`/`tool_get_context`, `to_tool_error`, `persist_citation`, and the `ReaderCitationSource` bridge into `citations` (`quran_tools.rs` L22–232).
- **FR-005**: Crate MUST provide quran services for forms rebuild (+MV-018), index rebuild/verify (+atomic activation), normalization preview, search services, search-cache management, and read-only doctor checks (`quran_forms`, `quran_index`, `quran_normalize`, `quran_search`, `quran_search_cache`, `quran_doctor`).
- **FR-006**: Crate MUST remain the sole bridge over `storage-sqlite` for `cli`/`server` (arch-check boundary).

### Key Entities

- **QuranReaderService / ResolvedEdition / EditionFilter**: Canonical read service with edition resolution and filtering.
- **ReaderToolBackend / ToolRegistry**: `quran.get_ayah` + `quran.get_context` tool surface with reproducibility envelopes.
- **RunResult**: Bootstrap outcome (backend health + wired subsystems).

### Error Surface

`QAI-QUR-0303` reader resolution failures · `0306` location not found · `0307` edition not found · `0310` tool backend unimplemented/stub · `0311/0312` tool invalid-input/backend (`tools`) · `0321/0322` citation invalid-reference/backend (`citations`). All mapped through `to_tool_error` with no unmapped arms.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo test -p application --lib` passes with zero failures (28 passed on 2026-09-17).
- **SC-002**: CLI acceptance flow (import→activate→reads→v2→diff→rollback, incl. error exits 5/6) stays green end to end.
- **SC-003**: Activation without approval is refused in 100% of attempts; every activation emits its audit event in the same transaction.
- **SC-004**: Every tool result carries the §12 contract fields; no tool path emits non-retrieved verse text (by construction + tests).

## Assumptions

- `storage` traits + `storage-sqlite` backend are the only persistence; `application` owns all cross-crate orchestration (server/CLI never touch SQLite directly).
- `citations` resolver verdicts (ExactMatch…Mismatch/LocationNotFound/EditionNotFound/AccessDenied) and deep-link/URN formats are consumed here, specified with the read surface.
- Jobs worker pool/chaos suites pending (P0 AC-P0-11/12 follow-ups) do not block the read/activate paths specified here.
- No search HTTP/SSE exposure yet — services exist at the Rust API; `server` exposes only the read/normalization surface documented in spec 011.
