# Feature Specification: config layered loader

**Feature Branch**: `003-config-layered`

**Created**: 2026-09-17

**Status**: Draft

**Input**: Module-level specification of the implemented `config` crate (crates/config/src/{lib,origin,secret,secret_store}.rs), derived from code truth on 2026-09-17. Reverse specification of what the crate does today. Code is the source of truth over plan docs.

**Constitution compliance**: `.specify/memory/constitution.md` v1.0.2, Principles V (gates), VI (`Secret<T>`, validation, local-first), VII (ADR-0004 layered config).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Layered configuration with provenance (Priority: P1)

An operator sets defaults in code, overrides via TOML file, environment (`QAI__SECTION__KEY`), and CLI flags. Every effective value records where it came from (`ValueOrigin`), so `config show` can explain each value's provenance.

**Why this priority**: Precedence disputes are a classic misconfiguration source. Per-key origin tracking makes the effective config auditable.

**Independent Test**: `cargo test -p config --lib` green (32 passed on 2026-09-17), including `toml_merges_every_section`, `env_overrides_every_section`, `cli_overrides_every_section`, and origin-map assertions per section.

**Acceptance Scenarios**:

1. **Given** defaults plus a TOML file plus `QAI__SERVER__PORT` plus a CLI override, **When** loaded, **Then** CLI wins over env, env over file, file over defaults — and `OriginMap` reports `cli`/`env`/`file`/`default` per key.
2. **Given** a TOML value referencing `${app.data_dir}`, **When** interpolation resolves, **Then** derived paths (sqlite path, object root, log file) inherit the data dir; cycles yield `InterpolationCycle`, never infinite recursion.

---

### User Story 2 - Secrets stay secret (Priority: P1)

Credential values live only inside `Secret<T>` (whose `Debug` prints `Secret(***)`), resolve through `SecretRef` (`backend:key`) backends, and are never persisted to SQLite as values.

**Why this priority**: Constitution Principle VI. A secret printed by `{:?}` or stored in a settings row is a defect.

**Independent Test**: `test_secret_redaction` + `test_secret_serialize` green; testkit sentinel suite covers the render boundary.

**Acceptance Scenarios**:

1. **Given** `Secret::new("my-password")`, **When** debug-formatted or serialized, **Then** output contains zero password bytes.
2. **Given** the env backend, **When** resolving a ref, **Then** it reads `QAI_SECRET_<KEY>` read-only (`put`/`delete` return `Unsupported`); the keychain backend reports `Unsupported` until its OS dependency is added.

---

### User Story 3 - Unsafe binds rejected at load (Priority: P2)

A server bind to a non-loopback address without TLS (or an explicit auth override) fails validation, as do unknown TLS modes — at load time, before anything listens.

**Why this priority**: Local-first default with deny-by-default network posture (Principle VI; `is_loopback` gate).

**Independent Test**: `test_validate_bind_0_0_0_0_without_tls`, `validate_rejects_unknown_tls_mode`, `validate_rejects_non_loopback_without_tls_or_auth_override` green.

**Acceptance Scenarios**:

1. **Given** `bind = "0.0.0.0"` with TLS disabled and no auth override, **When** validated, **Then** load fails with `ConfigError::Validation`.
2. **Given** the same bind with TLS required (or loopback), **When** validated, **Then** load succeeds.

### Edge Cases

- Missing config file is not an error when no path is given (defaults apply); an explicit unreadable path yields `FileRead` with path + source.
- Malformed TOML yields `Parse`; missing required keys yield `MissingKey`.
- `SecretRef::parse` rejects inputs without a `backend:key` shape (`InvalidRef`).
- Encrypted-file backend failures surface as `SecretError` without leaking key material.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Crate MUST expose `Config` with nine sections — `app{mode,data_dir,locale}`, `server{bind,port,tls,require_auth_outside_localhost,max_request_bytes,request_timeout_ms,concurrency_limit}`, `storage{backend,sqlite{path,journal_mode,synchronous,busy_timeout_ms,foreign_keys,max_connections,read_only_pool},objects{backend,root,max_file_bytes}}`, `secrets{backend,encrypted_file_path}`, `jobs{workers,poll_interval_ms,lease_seconds,max_attempts,backoff_*}`, `logging{level,format,file,redact_secrets}`, `telemetry{enabled,otlp_endpoint,metrics_enabled}`, `security{allow_network_egress,domain_allowlist,ssrf_block_private_ranges,max_download_bytes,max_archive_*,follow_symlinks}`, `policy{tool_execution_default,command_execution_enabled,canonical_write_requires_approval}` (`crates/config/src/lib.rs` L36–141).
- **FR-002**: Crate MUST merge CLI > env (`QAI__` prefix, `__` separators) > TOML file > compiled defaults, recording per-key `ValueOrigin::{Default, File{path,line}, Env(var), Cli(flag)}` in `OriginMap` (`merge_toml`, `merge_env`/`set_env_value`, `merge_cli`/`set_cli_value`, `mark_defaults`).
- **FR-003**: Crate MUST resolve `${...}` interpolation after merging, detecting cycles (`resolve_interpolation`, `resolve_string`, `resolve_var`).
- **FR-004**: Crate MUST validate at load: loopback-only bypass (non-loopback bind requires TLS or explicit auth override), closed TLS-mode set, plus section-level sanity (`validate`, `is_loopback`).
- **FR-005**: Crate MUST provide `Secret<T>` with redacting `Debug`/serialization; secret values MUST never be written to SQLite (only `SecretRef` persists) (`secret.rs`, `secret_store.rs` L1–11 contract).
- **FR-006**: Crate MUST provide the `SecretStore` trait (`get`/`put`/`delete`/`list_refs`, `Send + Sync`) with `EnvSecretStore` (read-only), `EncryptedFileSecretStore` (XChaCha20-Poly1305 JSON file), `KeychainSecretStore` (reports `Unsupported` until its OS dependency lands), plus `SecretStores` aggregate and `backend_by_name`.

### Key Entities

- **Config**: Nine-section effective configuration; serializable; validated at load and on change.
- **ValueOrigin / OriginMap**: Per-key provenance of every effective value (default, file+path, env var, CLI flag).
- **Secret\<T\>**: Credential wrapper; redacts on `Debug`/display paths.
- **SecretRef / SecretStore**: `backend:key` reference plus pluggable async backends; values never touch SQLite.

### Error Surface

Code truth on 2026-09-17 — **no `QAI-CFG-` Diagnostic codes exist in code**:

- `ConfigError::{FileRead{path,source}, Parse(toml::de::Error), Validation(String), InterpolationCycle(String), MissingKey(String)}` — plain `thiserror`, no `Diagnostic` impl (`crates/config/src/lib.rs` L20–31).
- `SecretError::{InvalidRef, NotFound, Unsupported, ...}` (`crates/config/src/secret_store.rs` L32+).
- **Doc drift recorded**: `QAI-CFG-` appears in ADR-0004, ADR-0010, and phase plans, but `rg QAI-CFG crates/` returns zero hits. Downstream crates surface config failures under their own namespaces. Do not document `QAI-CFG-` as implemented; owner to either implement the namespace or correct the ADRs.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo test -p config --lib` passes with zero failures (32 passed on 2026-09-17).
- **SC-002**: Every config section is covered by all three merge-source tests (TOML/env/CLI) — no section is default-only by accident.
- **SC-003**: A non-loopback bind without TLS/auth is rejected at load in 100% of attempts (no listen-before-validate path exists).
- **SC-004**: Zero secret bytes in `Debug`/serialized `Secret<T>` output (unit + sentinel suites green).

## Assumptions

- Env prefix is `QAI__` with `__` section separators (per `merge_env`); secret values additionally use `QAI_SECRET_<KEY>` via the env secret backend.
- Telemetry stays opt-in (`enabled: false` default path); OTLP span-field scrubbing gap is owned by observability, not this crate.
- Keychain support is intentionally stubbed (`Unsupported`) pending an OS-dependency decision — not a regression.
- No canonical Quran text, no model dependencies, no Tantivy in this crate (invariants I1/I2/I8 not applicable beyond `Secret<T>` hygiene).
