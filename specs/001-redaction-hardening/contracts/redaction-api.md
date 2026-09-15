# Contracts: 001-redaction-hardening

**Spec**: [spec.md](../spec.md) | **Data model**: [data-model.md](../data-model.md)

No wire protocol or HTTP surface changes. Contracts are Rust API
boundaries between workspace crates. All signatures are normative for
`/speckit-tasks` file paths; bodies belong to implementation.

## 1. `domain::redaction` (new module, `crates/domain/src/redaction.rs`)

Single source of truth (research D1). Pure functions, no I/O, no
workspace dependencies (`unsafe_code = "forbid"` inherited).

```rust
/// Replacement marker for redacted values (existing audit contract).
pub const REDACTED_MARKER: &str = "***REDACTED***";

/// Rule A: does this key/field name denote secret material?
/// Case-insensitive contains: secret | password | api_key | token | credential.
pub fn is_secret_key(name: &str) -> bool;

/// Rules A+C: redact a JSON value in place; returns number of redactions.
/// Objects: secret-named keys → marker (no recursion into replaced value).
/// Strings: Rule C userinfo scrub. Arrays/objects recurse.
pub fn redact_json_value(value: &mut serde_json::Value) -> usize;

/// Rule B (+C): redact credential patterns inside free text.
/// Returns the redacted string (borrowed when unchanged).
pub fn redact_text(input: &str) -> std::borrow::Cow<'_, str>;
```

Postconditions: `redact_json_value` never alters structure (keys,
nesting, non-secret leaves byte-identical); `redact_text` on
secret-free input returns the borrowed original (zero-alloc hot path).

## 2. `audit` delegation (`crates/audit/src/lib.rs`)

`redact_audit_value` keeps its signature and marker; its body delegates
to `domain::redaction::redact_json_value` (behavior-preserving refactor;
existing `redaction_strips_secrets` test must pass unchanged).

```rust
pub fn redact_audit_value(value: &mut serde_json::Value) // unchanged signature
```

## 3. `domain::Diagnostic` renderers (`crates/domain/src/diagnostic.rs`)

`render_human` and `render_json` apply `redact_text` to each free-text
field (message, location, affected_resource, remedy, next_command)
at render time. Stored struct data is NOT mutated. Existing renderer
tests must pass unchanged (sample data contains no credential shapes).

## 4. `observability` init options (`crates/observability/src/lib.rs`)

Additive API (existing `init(Format)` keeps behavior: redaction ON,
preserving deny-by-default for current callers).

```rust
/// Options for subscriber construction.
pub struct InitOptions {
    pub format: Format,
    pub redact_secrets: bool, // default: true
}

pub fn init_with_options(opts: InitOptions) -> Shutdown;
pub fn init(format: Format) -> Shutdown; // delegates with redact_secrets: true
```

The fmt layer (Text + JSON) is built with a redacting `FormatFields`
wrapper applying Rule A to recorded field names when
`redact_secrets` is true. New workspace edge `observability → domain`
requires the one-line `xtask/allowlist.toml` amendment (research D2).

## 5. `application` composition (`crates/application/src/lib.rs`)

```rust
pub use domain::redaction; // re-export for CLI surfaces (no new cli edge)
```

`run(cfg)` threads the flag: `init_with_options(InitOptions {
format: Format::Text, redact_secrets: cfg.logging.redact_secrets })`.

`config show` and `doctor` render paths (owned by `cli`, served via
`application`) apply redact-then-print through this re-export:
serialize → `redact_json_value` → print (JSON); `redact_text` over
Debug output (text). Doctor schema shape unchanged.

## 6. Sentinel suite (`crates/testkit/tests/secret_leak.rs`, extended)

New tests (names normative, bodies in implementation):

- `sentinel_absent_from_traced_fields` — events with secret-named
  fields holding the sentinel, captured via test writer; zero bytes.
- `sentinel_absent_from_secret_typed_event_values` — `Secret::new(SENTINEL)`
  logged via Debug/Display; zero bytes (regression guard for F1).
- `sentinel_key_value_pairs_scrubbed_from_free_text` — Rule B shapes
  (`api_key={SENTINEL}`) through `redact_text` AND through rendered
  diagnostics; zero bytes; benign "approval token issued" preserved.
- `sentinel_userinfo_scrubbed` — Rule C URL shapes through helper +
  doctor/config render paths; zero bytes.
- `sentinel_absent_from_config_show_and_doctor_json` — render-path
  coverage for US3 incl. schema validation of the doctor document.
- `redaction_switch_off_passes_values_through` — `redact_secrets=false`
  omits the layer (documents the escape hatch; asserts opt-out works).

All tests run under `cargo test --workspace` (FR-007); no new harness.
