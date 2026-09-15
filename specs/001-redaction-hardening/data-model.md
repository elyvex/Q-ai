# Data Model: 001-redaction-hardening

**Spec**: [spec.md](spec.md) | **Research**: [research.md](research.md)

## Entities

### 1. Redaction policy

The definition of what counts as secret material and what replaces it.
Conceptually global and static (compiled-in matcher + marker) with one
runtime switch.

- `secret_key_matcher`: case-insensitive key/field-name match —
  contains `secret | password | api_key | token | credential`
  (reuses the audit matcher semantics; single source of truth after D1).
- `marker`: `"***REDACTED***"` for JSON/structured/text replacement
  (existing audit contract; `Secret<T>` keeps its own forms).
- `redact_secrets_enabled`: runtime switch sourced from the existing
  `logging.redact_secrets` config flag (default true). When false, the
  tracing layer is omitted and render-time helpers pass values through
  (escape hatch for debugging; default-on preserves deny-by-default).
- Relationships: applied by every emission surface; exercised by the
  sentinel suite.

### 2. Emission surface

Any sink that renders in-memory state to bytes outside the process
boundary under test.

| Surface | Render path (verified) | Redaction point |
|---|---|---|
| Log output (text) | `observability::init` fmt compact → stderr | Redacting `FormatFields` wrapper (Rule A on field names) |
| Log output (JSON) | `observability::init` fmt json → stderr | Same wrapper, JSON field values |
| Error text | `Diagnostic::render_human` / `Display` | Per-field helper on message/location/resource/remedy/command |
| Error JSON | `Diagnostic::render_json` | Per-field helper, structure unchanged |
| Config inspection | `handle_config_show` Debug / pretty JSON | Redact-then-print via shared helper |
| Doctor report | `doctor_report` JSON doc (+ human text) | Redact-then-print; schema shape preserved |

### 3. Sentinel secret (test-only)

A unique, recognizable test credential (existing style:
`SENTINEL_9f3c__DO_NOT_LEAK`-class constant) planted by the suite.

- Planted: under secret-named keys/fields, inside `key=value` free text,
  inside URL userinfo, inside `Secret<T>` wrappers.
- Asserted: zero bytes in captured output per surface (SC-001);
  non-secret siblings preserved byte-for-byte (FR-008).
- Never a real credential; never leaves the test process.

## Redaction rule table (normative)

| Rule | Input shape | Action | Example |
|---|---|---|---|
| A — secret-named key/field | JSON key or tracing field matching the key matcher | Replace whole value with marker | `{"api_key":"sk-1"}` → `{"api_key":"***REDACTED***"}` |
| B — `key[:=]value` in free text | `secret\|password\|api_key\|token\|credential` followed by `:`/`=` and a non-space value | Replace value portion with marker | `login failed for api_key=sk-1` → `login failed for api_key=***REDACTED***` |
| C — URL userinfo | `scheme://…@` | Replace userinfo with marker | `https://u:p@host/db` → `https://***REDACTED***@host/db` |

Rules apply recursively to nested objects/arrays (Rule A/C) and across
the full string for free text (Rule B). Benign prose without these shapes
(e.g. "approval token issued") passes through untouched.

## State transitions

None — redaction is a pure, stateless transformation. The only stateful
element is the `redact_secrets_enabled` switch, set once at startup from
config and never mutated thereafter (no hot-reload in this feature).
