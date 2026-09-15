# Feature Specification: Global Redaction Tracing Layer + Secret-Leak Sentinel Suite

**Feature Branch**: `001-redaction-hardening`

**Created**: 2026-09-15

**Status**: Draft

**Input**: User description: "P0-T16 — Global redaction tracing layer + secret-leak sentinel suite (D0.5, D0.16). Extend the existing sentinel suite (type-level Secret<T> redaction + audit-level redaction, AC-P0-05) to log emission, error formatting, `config show`, and doctor JSON outputs, and add a global tracing redaction layer. Follow-up FU-01."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Secrets never reach log output (Priority: P1)

An operator runs Q-ai with a real secret configured (API key, provider token).
Every log line emitted through the application's central logging
pipeline — including spans, events, and formatted fields — is scrubbed of secret material before
it reaches any appender, file, or exporter, so a leaked log file alone can
never disclose a credential.

**Why this priority**: Logs are the most routinely persisted, copied, and
shipped-off-host diagnostic artifact. A single unredacted secret in logs
defeats the `Secret<T>` type-level redaction already in place. This is the
named deliverable of P0-T16 and the core of constitution principle VI.

**Independent Test**: Plant a unique sentinel string as a secret value,
exercise code paths that log while the secret is in scope (config load,
job execution, error paths), capture all emitted log output, and assert the
sentinel appears in zero bytes while non-secret fields remain intact.

**Acceptance Scenarios**:

1. **Given** a secret value is held in memory and code logs the surrounding
   context at any level, **When** the full captured log output is searched,
   **Then** the secret string appears zero times and a redaction marker
   appears in its place.
2. **Given** structured log fields containing a mix of secret and non-secret
   values, **When** the output is inspected, **Then** non-secret fields
   (ids, durations, counts) are preserved byte-for-byte.

---

### User Story 2 - Secrets never leak through error rendering (Priority: P2)

A failure occurs while a secret is in scope (bad provider key, failed
connection using a credentialed URL, validation error echoing config).
The human-readable and JSON renderings of diagnostics and errors shown to
the user or written to output files contain no secret material.

**Why this priority**: Errors routinely echo the offending input or
configuration for debuggability, which is exactly how secrets escape via
support tickets, pasted terminal output, and log files.

**Independent Test**: Plant a sentinel secret, trigger representative error
paths that include tainted context (config errors, connection errors,
validation errors), render each error in both human and JSON forms, and
assert zero sentinel bytes in every rendering.

**Acceptance Scenarios**:

1. **Given** an error whose context includes a secret value, **When** it is
   rendered for terminal display, **Then** the sentinel appears zero times.
2. **Given** the same error, **When** it is rendered as JSON, **Then** the
   sentinel appears zero times and the surrounding structure still parses.

---

### User Story 3 - Secrets never leak through CLI inspection outputs (Priority: P3)

An operator runs `config show` or `doctor --json` to inspect or share
system state. Both outputs redact secret values while still showing that
the corresponding settings exist and where each value came from.

**Why this priority**: These commands exist to be copied into bug reports
and runbooks. They must stay safe to share without a manual scrub pass.

**Independent Test**: Configure a sentinel secret, run each command,
capture stdout, and assert zero sentinel bytes while origin/scope metadata
for the redacted settings is still present.

**Acceptance Scenarios**:

1. **Given** a secret is set via any supported source, **When** the operator
   runs the config inspection command, **Then** the output shows the key and
   its origin with the value redacted and the sentinel appears zero times.
2. **Given** a secret is configured, **When** the operator runs the doctor
   JSON report, **Then** the document validates against its schema, the
   secret appears zero times, and all health checks still report.

### Edge Cases

- A secret value that coincides with a common word or a short string:
  redaction must be scoped to actual secret holdings/keys, not substring
  matches against arbitrary text, so normal output is not mangled.
- Nested structures (secret inside a map inside a list inside an event
  field): redaction must recurse to arbitrary depth.
- Secrets introduced after startup (rotation, per-command overrides) are
  covered by the same layer without restart.
- Performance: the redaction pass must not measurably change log
  throughput characteristics for secret-free hot paths (no per-event
  allocation blowup on the common path).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST scrub secret material from all tracing/log
  output via a single global layer that applies regardless of which
  crate or call site emitted the event.
- **FR-002**: System MUST scrub secret material from human-readable error
  and diagnostic renderings.
- **FR-003**: System MUST scrub secret material from JSON error and
  diagnostic renderings while keeping the documents structurally valid.
- **FR-004**: The config inspection command MUST display secret keys and
  their value origins with values redacted.
- **FR-005**: The doctor JSON report MUST contain zero secret bytes and
  MUST still validate against its published schema.
- **FR-006**: The sentinel suite MUST plant a unique sentinel secret and
  assert its absence (zero bytes) across log emission, error formatting
  (human + JSON), config inspection, and doctor JSON outputs.
- **FR-007**: The sentinel suite MUST run as part of the standard
  workspace test suite so regressions fail the normal quality gates.
- **FR-008**: Redaction MUST preserve non-secret surrounding data
  byte-for-byte (ids, counts, origins, structure).
- **FR-009**: Redacted positions MUST carry an explicit redaction marker
  rather than silently dropping the field where the format allows it.

### Key Entities *(include if feature involves data)*

- **Sentinel secret**: A unique, recognizable test credential planted
  during the suite; stands in for any real secret. Attribute: value
  (unique per run or fixed distinctive constant).
- **Redaction policy**: The definition of what counts as secret material
  (held secret values; known secret-bearing keys such as password,
  api_key, token) and what marker replaces it.
- **Emission surface**: Any sink rendering in-memory state to bytes
  outside the process boundary under test — log output, error text/JSON,
  CLI stdout, report documents.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A planted sentinel secret appears in zero bytes of captured
  output across all four surfaces (logs, error text, error JSON, CLI
  inspection outputs) in a full suite run.
- **SC-002**: 100% of the pre-existing secret-leak tests keep passing
  unchanged (type-level and audit-level redaction unbroken).
- **SC-003**: The full workspace quality gates (format, lint, tests,
  architecture check, migration check) pass with the feature merged.
- **SC-004**: A reviewer can follow the suite and identify, for each
  surface, where redaction is enforced, in under 15 minutes of reading.

## Assumptions

- The existing type-level secret wrapper and audit key-denylist
  redaction remain the foundation; this feature extends coverage to new
  surfaces rather than replacing them.
- The application routes diagnostics through a single central
  logging/diagnostics pipeline, which is where the global layer attaches.
- The existing sentinel constant style (distinctive, greppable,
  never a real credential) is reused for the new tests.
- Secret-bearing keys follow the existing audit denylist naming
  (password, api_key, secret, token, and equivalents); new key names
  introduced later are a follow-up, not this feature.
- CLI commands under test already support machine-readable output and a
  non-interactive mode suitable for output capture.
