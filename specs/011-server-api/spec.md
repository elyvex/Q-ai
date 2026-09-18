# Feature Specification: server loopback API

**Feature Branch**: `011-server-api`

**Created**: 2026-09-17

**Status**: Draft

**Input**: Module-level specification of the implemented `server` crate (crates/server/src/{lib,api}.rs: axum API v1 + health server + debug reader), derived from code truth on 2026-09-17. Reverse specification of what exists today. Code is the source of truth over plan docs.

**Constitution compliance**: `.specify/memory/constitution.md` v1.0.2, Principles III (envelope + meta + ETag + reproducibility fields on every response), VI (loopback-only serve; non-loopback refused), VII (composition root via application; HTML-escaped debug output).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Health and readiness for supervisors (Priority: P1)

Orchestrators poll `/healthz` (liveness) and `/readyz` (readiness, backend-checked) to supervise the process. Both are cheap, unauthenticated, and loopback-only like everything else on this server.

**Why this priority**: Supervision without readiness checks routes traffic to a server whose backend is down.

**Independent Test**: `cargo test -p server --lib` green (4 passed on 2026-09-17); live `serve --bind 127.0.0.1:8737` answers both endpoints.

**Acceptance Scenarios**:

1. **Given** a running server with a healthy backend, **When** `GET /readyz`, **Then** 200.
2. **Given** a backend failure, **When** `GET /readyz`, **Then** non-200 (never a false-ready 200).

---

### User Story 2 - Quran read API v1 (Priority: P1)

Clients read editions, surahs, ayahs, context windows, divisions, tokens, resolved references, and stored citations under `/api/v1/quran/…`, plus normalization `preview` (POST) and `profiles` (GET). Every success is an `Envelope{data, meta}` with `Meta` (edition, version, hashes, normalization rules, reproducibility data), ETag, and `Content-Language`; every failure is a Diagnostic `ErrorBody` with the mapped HTTP status (`tool_status`).

**Why this priority**: This is the machine-readable counterpart to the CLI reads — same reader services, HTTP envelope, no new semantics.

**Independent Test**: `envelope_meta_serializes_with_required_keys` + `tool_status_mapping` green; OpenAPI contract `docs/08-api/quran-v1-openapi.json`.

**Acceptance Scenarios**:

1. **Given** `GET /api/v1/quran/ayahs/2:255`, **When** served, **Then** the envelope carries canonical text with edition + version + hash in `meta`.
2. **Given** an unknown edition, **When** requested, **Then** the status maps from the tool error (`QAI-QUR-0307` family → 404-class) with a Diagnostic body — never invented text.

---

### User Story 3 - Loopback-only with a labelled debug reader (Priority: P2)

The server binds loopback only; non-loopback binds are refused (exit 4 at the CLI layer). A `/debug/read/{edition}/{surah}` RTL-labelled reader aids development, with HTML-escaped output and an optional bundled woff2 font endpoint.

**Why this priority**: Principle VI. An unauthenticated research API must never silently bind to a public interface.

**Independent Test**: `serve_defaults_to_loopback` (cli) green; `debug_escape_html_neutralizes_markup` green.

**Acceptance Scenarios**:

1. **Given** `--bind 0.0.0.0:8737` without TLS/auth posture, **When** serve starts, **Then** it refuses (exit 4) instead of listening publicly.
2. **Given** adversarial markup in debug-rendered text, **When** served, **Then** it is neutralized by `escape_html`.

### Edge Cases

- **No search endpoints exist** (no HTTP/SSE search) — search stays at the Rust service API; documenting search routes would be fabrication.
- Normalization `preview` is stateless computation; `profiles` lists the frozen L0–L8 ladder (no mutation endpoint).
- Citation endpoints resolve stored citations with the full verdict set (incl. Mismatch/AccessDenied → mapped statuses).
- `QuranApiBackend` trait (`ReaderBackend` impl) keeps axum handlers decoupled from the reader service for tests.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Crate MUST serve `GET /healthz` + `GET /readyz` (readiness backend-gated) over axum + tower-http (`lib.rs`, `api.rs router()` L799–842, `serve()` L845–851).
- **FR-002**: Crate MUST serve API v1 (`API_VERSION = "v1"`): editions, edition-by-slug, surahs, surah, ayahs, context, divisions, tokens, resolve, citations, normalization preview/profiles, debug reader (+font) — all through `QuranApiBackend` (`api.rs` L46–756, `ReaderBackend` L854–1061).
- **FR-003**: Crate MUST wrap successes in `Envelope{data, meta: Meta}` (`empty_meta`/`meta_from_tool`), emit ETags (`etag_for`), set `Content-Language`, and render failures as Diagnostic `ErrorBody`/`ErrorDetail` with `tool_status`-mapped codes (`api.rs` L86–271).
- **FR-004**: Crate MUST refuse non-loopback binds without approved posture (exit 4 via CLI serve path) and HTML-escape debug output (`escape_html`, L772).
- **FR-005**: Crate MUST add no model/embedding/vector path; all data flows from `application` reader services (arch-check boundary).

### Key Entities

- **Envelope\<T\> / Meta / EditionMeta**: Data + reproducibility metadata (edition, version, hashes, rules) on every response.
- **ErrorBody / ErrorDetail**: Diagnostic failure bodies with mapped HTTP statuses.
- **QuranApiBackend / ReaderBackend / AppState**: Handler/backend seam, canonical-backed implementation, shared state.
- **PreviewRequest**: Stateless normalization preview input.

### Error Surface

`QAI-QUR-0306` location not found · `0307` edition not found · `0308/0309` context/division errors · `0310` unimplemented · `0322` citation backend (observed in `crates/server/src` 2026-09-17), mapped to HTTP statuses via `tool_status` (test-pinned). Failures never synthesize verse text.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo test -p server --lib` passes with zero failures (4 passed on 2026-09-17).
- **SC-002**: Live loopback serve answers `/healthz`, `/readyz`, and `/api/v1/meta` from inside its network namespace.
- **SC-003**: Every v1 success response validates against the envelope contract (required meta keys present; ETag emitted).
- **SC-004**: Non-loopback binds refused in 100% of unapproved attempts; debug output neutralizes markup (escape test green).

## Assumptions

- Container/Compose validation (daemon unavailable 2026-09-17) is an open follow-up: image build, UID 65532, volume ownership, and in-namespace endpoint checks must be re-run once the daemon is available — not claimed here.
- Debug-reader font licensing is an owner decision (P1 blocker list); the font endpoint serves bundled bytes only when provided.
- Streaming API, MCP, agent tools, and search endpoints are future surfaces — explicitly absent, not implied.
- OpenAPI file `docs/08-api/quran-v1-openapi.json` mirrors this router; drift between them is a defect to file, not to silently resolve here.
