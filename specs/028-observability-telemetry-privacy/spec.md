# Feature Specification: Observability Telemetry Privacy

**Feature Branch**: `028-observability-telemetry-privacy`

**Created**: 2026-09-18

**Status**: Draft

**Input**: User description: "Reverse specification of the implemented observability stack: tracing subscriber setup (text/JSON, RUST_LOG filter), span conventions, metric catalog, telemetry content denylist, RedactingWriter stderr scrubbing, off-by-default OTLP export, AC-P0-18, and the known OTLP span-field scrubbing gap."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Structured logs with redaction on by default (Priority: P1)

An operator starts Q-ai and gets a working tracing subscriber immediately: human-readable text or JSON-structured logs on stderr, filtered by `RUST_LOG`, with secret `key=value` patterns scrubbed before they reach the destination. Redaction is on unless explicitly disabled for debugging.

**Why this priority**: Every run must be debuggable local-first with no setup, and must never persist a credential to a log file. This is the default path all callers use (`init`), and the deny-by-default posture required by constitution section VI.

**Independent Test**: Initialize with `Format::Text` and with `Format::Json`, emit events containing `api_key=<sentinel>` alongside `status=ok`, capture stderr bytes, and assert zero sentinel bytes, presence of `***REDACTED***`, and intact `status=ok`. Repeat with `InitOptions::default_off_for_debug()` to confirm the escape hatch omits scrubbing.

**Acceptance Scenarios**:

1. **Given** default initialization, **When** an event containing `token=<value>` is emitted, **Then** stderr contains `token=***REDACTED***` and zero bytes of the original value.
2. **Given** `Format::Json` initialization, **When** an event is emitted, **Then** the output is JSON-structured and secret `key=value` patterns are still scrubbed.
3. **Given** `RUST_LOG` is set, **When** the subscriber starts, **Then** only events passing the env filter are emitted.

---

### User Story 2 - Telemetry stays off and content fields never export (Priority: P1)

A privacy reviewer enables telemetry explicitly and verifies that content-bearing fields (query text, prompt text, document content, research questions, model responses) can never leave the process via telemetry payloads, satisfying AC-P0-18.

**Why this priority**: Exporting a research question or model response to a collector is a privacy breach even when the user opted into telemetry. The compile-time denylist plus off-by-default gate is the enforcement mechanism (PRD §39, ADR-0011).

**Independent Test**: Build a payload containing all five forbidden fields plus `job_id` and `duration_ms`, run the sanitizer, and assert exactly 5 keys removed with safe fields byte-identical. Assert each forbidden name and its variants (`query`, `Prompt`, `research_questions`) are rejected by the predicate while `job_id` and `duration_ms` are allowed.

**Acceptance Scenarios**:

1. **Given** telemetry is not explicitly enabled, **When** the application runs, **Then** no OTLP exporter is installed and no telemetry payload leaves the process.
2. **Given** a telemetry payload containing `query_text`, `prompt_text`, `document_content`, `research_question`, and `model_response`, **When** it is sanitized, **Then** all five keys are removed, the removal count is 5, and `job_id` / `duration_ms` survive unchanged.
3. **Given** a nested payload with a forbidden key at depth, **When** it is sanitized, **Then** the nested key is removed, its siblings survive, and the count reflects only removed keys.

---

### User Story 3 - Stable spans and metrics for debugging and dashboards (Priority: P2)

A developer instruments a subsystem (storage, job, source) with the standard span fields and named metric helpers, and an operator builds dashboards against metric names that are guaranteed stable.

**Why this priority**: Cross-subsystem traces and Phase 12 Grafana dashboards only work if span field names and metric names are a stable contract, not ad-hoc strings per call site.

**Independent Test**: Emit spans using only the published `qai.*` field names, call each metric helper once, and assert the nine catalog instruments exist with the documented label sets and no registration panic.

**Acceptance Scenarios**:

1. **Given** a span is emitted, **When** its fields are inspected, **Then** field names are drawn from the ten published constants (`qai.component`, `qai.operation`, `qai.job_id`, `qai.run_id`, `qai.source_id`, `qai.source_version`, `qai.principal_id`, `qai.workspace_id`, `qai.duration_ms`, `qai.outcome`).
2. **Given** application startup, **When** the metric catalog initializes twice, **Then** registration is idempotent and all nine instruments (`qai_jobs_enqueued_total`, `qai_jobs_completed_total`, `qai_job_duration_seconds`, `qai_db_query_duration_seconds`, `qai_db_pool_in_use`, `qai_audit_events_total`, `qai_config_reloads_total`, `qai_errors_total`, `qai_doctor_checks_total`) are available with their documented label keys.

---

### User Story 4 - Opt-in OTLP export to a collector (Priority: P2)

An operator who explicitly opts into telemetry exports traces over OTLP/HTTP to a configured collector endpoint, with a guard that flushes and shuts down the pipeline on drop.

**Why this priority**: Local-first deployments need no network export; the few operators with a collector need a single explicit call that wires the OpenTelemetry bridge correctly.

**Independent Test**: Call the provider builder with a malformed endpoint (assert error string) and with a valid local endpoint (assert provider builds and shuts down cleanly); call the installer and assert a global subscriber is set and a shutdown guard is returned.

**Acceptance Scenarios**:

1. **Given** the `otlp` feature is compiled in and telemetry is explicitly enabled, **When** a valid OTLP/HTTP endpoint is provided, **Then** a batch-exporting tracer provider with `service.name="qai"` and tracer name `qai` is built and installed as the global subscriber.
2. **Given** a malformed endpoint string, **When** the provider is built, **Then** construction fails with an error and no subscriber is installed.
3. **Given** an installed OTLP guard, **When** it is dropped, **Then** the tracer provider shuts down (flush attempted, no panic).

---

### User Story 5 - Known OTLP secret-scrub gap is bounded and disclosed (Priority: P3)

A security auditor reads the spec and understands exactly which path is NOT yet scrubbed for secrets: OTLP span fields bypass the stderr `RedactingWriter`, so secret `key=value` patterns in span fields can reach the OTLP backend unscrubbed. The content-field denylist still applies; secret-key scrubbing on the OTLP path is an open follow-up.

**Why this priority**: Shipping with an undisclosed bypass would misrepresent the privacy posture. Documenting the boundary (stderr scrubbed, OTLP span fields not yet scrubbed, content fields always gated) lets operators make an informed opt-in decision.

**Independent Test**: Inspect the export pipeline and confirm span export does not pass through the stderr writer; confirm the content denylist still gates export payloads; confirm the follow-up (span processor or OTLP-specific field hook when `redact_secrets=true`) is recorded.

**Acceptance Scenarios**:

1. **Given** redaction is enabled, **When** a span field contains a secret `key=value` pattern and telemetry is exported via OTLP, **Then** the current behavior (unscrubbed on the OTLP path) is documented as a known gap rather than claimed as covered.
2. **Given** the same export, **When** the payload contains a denylisted content field, **Then** that field is still stripped before export.

---

### Edge Cases

- Empty secret value (`token=` with no value): the scrubber leaves the empty pair untouched rather than inserting a marker into unrelated text.
- Secret key matching is case-insensitive (`API_KEY=`, `Token=`) and covers the seven recognized keys (`api_key`, `apikey`, `api-key`, `password`, `secret`, `token`, `credential`); value termination stops at space, comma, semicolon, newline, carriage return, `}`, `]`, `"`, `'`, so surrounding structured data is preserved.
- Non-UTF8 log bytes are handled lossily without panic; the writer reports the original buffer length (not the scrubbed length) to satisfy the `io::Write` contract.
- `set_global_default` failure (a subscriber already installed, e.g. in tests) is ignored rather than panicking, so `init` never crashes the process.
- `Shutdown` guard drop is currently a no-op placeholder (flush happens via the subscriber's own mechanism); callers must still hold the guard for the documented pattern.
- `is_forbidden_field` substring rules are intentionally broad for `query` and `prompt` (any field name containing them is denied) and phrase-scoped for `document_content`, `research_question`, `model_response`; safe operational fields (`job_id`, `duration_ms`) are never matched.
- Metric label values are caller-supplied strings (`kind`, `outcome`, `op`, `pool`, `action`, `code`, `check`, `status`); metric names and label keys are fixed by the catalog — no ad-hoc instrument names exist outside it.
- OTLP module is compiled only with the `otlp` feature; without it there is no export path at all.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST install a tracing subscriber on startup via `init` (defaults: text format, redaction on) or `init_with_options`, honoring `RUST_LOG` through `EnvFilter::from_default_env`.
- **FR-002**: System MUST support both `Format::Text` (compact, with target/file/line) and `Format::Json` (JSON, with target/file/line) subscriber layouts.
- **FR-003**: System MUST route all stderr log writes through `RedactingWriter` when `redact_secrets=true` (the default), applying Rule A `key=value` scrubbing with the `***REDACTED***` marker; the `default_off_for_debug()` escape hatch MUST be the only path that omits the writer.
- **FR-004**: System MUST implement Rule A scrubbing in `redact_log_fields` over exactly the seven recognized keys (`api_key`, `apikey`, `api-key`, `password`, `secret`, `token`, `credential`), case-insensitively, preserving non-secret surrounding data and leaving empty values untouched.
- **FR-005**: System MUST publish exactly the ten `qai.*` span field-name constants (`component`, `operation`, `job_id`, `run_id`, `source_id`, `source_version`, `principal_id`, `workspace_id`, `duration_ms`, `outcome`) as the span convention.
- **FR-006**: System MUST provide the nine-instrument metric catalog with fixed names, kinds, and label keys (counters `qai_jobs_enqueued_total{kind}`, `qai_jobs_completed_total{kind,outcome}`, `qai_audit_events_total{action}`, `qai_config_reloads_total{outcome}`, `qai_errors_total{code}`, `qai_doctor_checks_total{check,status}`; histograms `qai_job_duration_seconds{kind}`, `qai_db_query_duration_seconds{op}`; gauge `qai_db_pool_in_use{pool}`), registered idempotently via `init()` with one convenience wrapper per instrument.
- **FR-007**: System MUST keep telemetry export off by default: no OTLP exporter is installed unless telemetry is explicitly enabled and the `otlp` feature is compiled in.
- **FR-008**: System MUST deny export of the five content-bearing fields (`query_text`, `prompt_text`, `document_content`, `research_question`, `model_response`) via `is_forbidden_field` (exact plus documented substring variants) and MUST provide recursive `sanitize_telemetry_value` that strips denylisted keys at any depth and returns the removal count (AC-P0-18).
- **FR-009**: System MUST provide opt-in OTLP/HTTP export via `build_provider` (batch exporter, Tokio runtime, `service.name="qai"`, error on malformed endpoint) and `init_otlp` (bridges the `qai` tracer into the global subscriber, returns an `OtlpGuard` that shuts the provider down on drop).
- **FR-010**: System MUST document the known gap that OTLP span-field export bypasses the stderr `RedactingWriter`, so secret `key=value` patterns in span fields are currently unscrubbed on the OTLP path; content-field denylisting still applies.
- **FR-011**: Stderr secret-material scrubbing beyond Rule A field patterns (global tracing layer, error rendering, CLI inspection outputs, sentinel suite) is owned by `specs/001-redaction-hardening/spec.md` and MUST NOT be re-specified here; this spec references that document for those surfaces.

### Key Entities *(include if feature involves data)*

- **Subscriber / InitOptions**: The global tracing installation; attributes: format (text or JSON), `redact_secrets` flag (default true), env-filter source (`RUST_LOG`), stderr destination, `Shutdown` guard lifetime.
- **RedactingWriter**: The stderr write interceptor; attribute: Rule A pattern set (seven secret keys) and `***REDACTED***` marker; writes scrubbed bytes while reporting the original length.
- **Span convention**: The ten `qai.*` field names carried on spans for component, operation, job/run/source identity, principal/workspace accountability, duration, and outcome.
- **Metric catalog**: The nine named instruments with fixed label keys; treated as a stable API (renames/removals are breaking changes).
- **Telemetry denylist**: The five forbidden content fields plus substring variants enforced by `is_forbidden_field`; recursive payload sanitizer returning removal counts.
- **OTLP exporter**: The opt-in pipeline (`build_provider` / `init_otlp` / `OtlpGuard`); attributes: endpoint URL, batch exporter, `service.name="qai"` resource, shutdown-on-drop semantics. Known limitation: span-field secret scrubbing bypass (open follow-up).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: With default initialization, a planted sentinel in `api_key=` / `token=` positions appears in zero bytes of captured stderr across text and JSON formats, replaced by the redaction marker, while adjacent non-secret fields survive unchanged.
- **SC-002**: Telemetry payloads containing all five forbidden content fields are sanitized to zero remaining forbidden keys with a removal count of 5, and nested forbidden keys are stripped without disturbing siblings (AC-P0-18 demonstrable in under 5 minutes).
- **SC-003**: All nine metric instruments and all ten span field constants are usable from a clean build with zero registration panics and zero ad-hoc instrument names outside the catalog.
- **SC-004**: A reviewer can state the exact privacy boundary after one reading: what is scrubbed on stderr, what is denied from telemetry, what OTLP export requires (explicit enable + feature + endpoint), and what the documented OTLP span-field gap is.

## Assumptions

- Code is truth: this spec reverses the implementation in `crates/observability/src/{lib,metrics,telemetry,otlp}.rs` as of 2026-09-18; where prose and code disagree, the code governs.
- Stderr secret scrubbing (Rule A and beyond), error/CLI surfaces, and the sentinel-leak suite are owned by `specs/001-redaction-hardening/spec.md`; this spec covers only the observability-local Rule A writer hook plus the OTLP/metrics/telemetry surfaces and references that document instead of duplicating it.
- Telemetry configuration (`telemetry.enabled=false` default, blank OTLP endpoint default, endpoint-without-enable validation error, TLS outside localhost, `logging.level/format/file` knobs, `qai doctor observability.subscriber_installed`) follows ADR-0011; local config validation details live outside this spec.
- Known open gap (from `docs/05-followups/open-questions.md`, P0-T16 follow-up): OTLP span export bypasses `RedactingWriter`; span-field scrubbing for the OTLP path when `redact_secrets=true` (span processor or OTLP-specific field hook) is deferred to Phase 3 or later and is explicitly out of scope here.
- Metric names are a stable API per ADR-0011 migration strategy: additions allowed, renames/removals are breaking.
- Constitution section VI (local-first, deny-by-default, loopback default, fail-closed guards) is the governing principle; no new network, storage, or permission surface is introduced by this spec.
