# Research: 001-redaction-hardening

**Date**: 2026-09-15 | **Spec**: [spec.md](spec.md) | **Method**: direct code
inspection (graft map + targeted reads of `observability`, `config`,
`audit`, `domain::diagnostic`, `cli`, `application`, `xtask` arch rules)

## Context

P0-T16 requires a global tracing redaction layer plus sentinel-suite
coverage for four surfaces: log emission, error formatting (human + JSON),
`config show`, and doctor JSON. FU-01 records that the current suite
(`crates/testkit/tests/secret_leak.rs`, 3 tests) covers type-level
`Secret<T>` redaction and audit-level redaction only. Constitution v1.0.0
principles V (quality gates) and VI (deny-by-default, `Secret<T>`,
`unsafe_code = "forbid"`) constrain the design.

## Findings (verified in-tree)

- F1. `Secret<T>` (`crates/config/src/secret.rs`) redacts `Debug`
  (`Secret(***)`), `Display` (`***`), and `Serialize` (`"***"`), exposes
  via `Deref`/`expose()`, and zeroizes on drop. Typed secrets are already
  safe in any formatting pipeline; only explicitly exposed raw strings
  escape.
- F2. `audit::redact_audit_value` (`crates/audit/src/lib.rs:236`) redacts
  JSON by secret-named keys AND by value substrings (`secret`, `password`,
  `api_key`, `token`, `credential`), marker `"***REDACTED***"`.
- F3. The subscriber is built in exactly one place,
  `observability::init(Format)` (`crates/observability/src/lib.rs:60`),
  as a `tracing_subscriber::fmt()` subscriber (Text compact or JSON) on
  stderr, set as global default. No redaction exists. The sole production
  caller is `application::run` (`crates/application/src/lib.rs:36`), which
  holds `cfg: Config` — threading a flag is trivial.
- F4. `logging.redact_secrets: bool` exists in `LoggingConfig`
  (`crates/config/src/lib.rs:100`, default `true`) with loader support,
  but NOTHING reads it — a dead flag awaiting exactly this feature.
- F5. `Config` holds NO secret values (backend names, paths, numerics
  only); provider keys live in `SecretStore` backends (env / keychain /
  encrypted file), never in `Config`. Real leak vectors in
  `config show` / doctor JSON are credential-bearing URLs/paths
  (e.g. `user:pass@host` userinfo, secret-looking path segments).
- F6. `handle_config_show` (`crates/cli/src/lib.rs:350`) prints
  `Config` Debug or pretty JSON — a redact-then-print choke point.
- F7. `doctor_report` (`crates/cli/src/doctor.rs:717`) builds a JSON doc
  validated against a schema with `additionalProperties: false`
  (`xtask/src/schema.rs:187`) — redaction MUST NOT alter structure.
- F8. `domain::Diagnostic` (`crates/domain/src/diagnostic.rs:63`)
  interpolates free-form `String` fields raw into both renderers. A secret
  appears only if a caller interpolates an exposed value.
- F9. Layering (`xtask/allowlist.toml`): `domain` allows nothing (leaf);
  `observability` allows nothing; `cli` allows `application, config,
  observability, server` (NOT `domain`); `audit` allows `domain,
  storage`; `testkit` allows `domain, config, storage, audit, sources,
  jobs, provenance, observability`. `domain` already depends on
  `serde_json`.
- F10. `observability` has the OTLP exporter behind the `otlp` feature;
  telemetry is off by default with a content-field denylist (AC-P0-18).

## Decisions

- **D1: Shared helper lives in `domain::redaction`.**
  Rationale: `domain` is the dependency leaf (F9) with `serde_json`
  already available (F9) — usable by `domain::Diagnostic` itself,
  `audit` (already allowed), `application` (already allowed), and
  `observability` (one allowlist line). Single source of truth for the
  key matcher + marker ends the audit/telemetry matcher drift (F2 vs
  `is_forbidden_field`, which solve different problems and must NOT be
  merged).
  Alternatives considered: helper in `observability` (rejected —
  `domain::Diagnostic` and `audit` cannot reach it without two allowlist
  changes, wrong direction); per-crate duplication (rejected —
  perpetuates exactly the drift this feature closes).
- **D2: One justified allowlist amendment — `observability += domain`.**
  Rationale: leaf-ward edge, zero cycle risk; recorded with justification
  in `plan.md` Constitution Check per constitution principle VII
  (violations fixed or ADR-justified, never silent). `cli` consumes
  redaction via `application` re-export (composition root, no `cli`
  change); `audit` delegates internally (no `audit` change); `testkit`
  already covered (no change).
- **D3: Redaction contract (three rules, no value registry).**
  Rule A — secret-named keys/fields (reuse audit key matcher:
  `secret|password|api_key|token|credential`, case-insensitive) → whole
  value replaced with marker. Applies to JSON values and tracing fields.
  Rule B — free-text `key[:=]value` patterns (`api_key=sk-…`,
  `password: …`) → value portion replaced. Catches interpolated
  credentials while leaving benign prose ("approval token issued")
  intact; the audit-style bare-substring match is deliberately NOT used
  on free text (`token` would mangle "ApprovalToken" prose).
  Rule C — URL userinfo (`scheme://…@`) → credentials replaced. Covers
  the genuine config/doctor vector (F5).
  Rejected: process-global secret-value registry — fights
  `ZeroizeOnDrop` (registry would retain copies), adds global mutable
  state to the hot logging path, and promises unkeepable security.
  Explicit `.expose()` inside a format string remains a code-review
  concern, documented in `quickstart.md`.
- **D4: Marker stays `"***REDACTED***"`** for JSON/structured output
  (existing audit contract, F2); `Secret<T>` keeps its own `"***"` /
  `Secret(***)` forms (F1) — no churn to shipped contracts. The suite
  asserts sentinel absence everywhere and marker presence where the
  format allows.
- **D5: Wire the dead `logging.redact_secrets` flag (default true) as the
  layer kill-switch.** Rationale: the schema already promises it (F4);
  additive `init_with_options` (or options struct) avoids breaking the
  existing `init(Format)` callers/tests; `application::run` threads
  `cfg.logging.redact_secrets` (F3).
- **D6: Tracing integration = redacting `FormatFields` wrapper on the fmt
  layer (Text + JSON).** Rationale:最小 blast radius — one construction
  site (F3), no event re-emission machinery, applies to every event/span
  field by Rule A. OTLP span-field scrubbing is an explicit follow-up
  (spec Assumptions; telemetry off-by-default + export denylist mitigate).
- **D7: `config show` / doctor JSON route through the shared helper
  (redact-then-print).** Rationale: choke-point fix (F6, F7), structure
  preserved for the schema gate; suite covers the render path with a
  sentinel-bearing document plus the live commands.

## Consequences for design artifacts

- `data-model.md`: redaction policy (matcher, marker, flag), emission
  surfaces, sentinel entity; rule table A/B/C.
- `contracts/redaction-api.md`: `domain::redaction` function signatures,
  `observability` init options, `application` re-export, `audit`
  delegation note.
- `quickstart.md`: suite invocation, manual sentinel drill, `.expose()`
  review guidance, allowlist-amendment verification (`arch-check`).
- All NEEDS CLARIFICATION resolved — none remain.
