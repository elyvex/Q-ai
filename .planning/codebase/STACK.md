---
last_mapped_commit: 68b7ea6c1aacd1476d154550b2a6e9574614b6af
last_mapped_at: 2026-09-22
---
# Technology Stack

**Analysis Date:** 2026-09-22

## Languages

**Primary:**

- Rust 1.97.1 (pinned via `rust-toolchain.toml`, channel `stable`-pinned `1.97.1` with `rustfmt` + `clippy` components) — edition 2024 across every crate in `Cargo.toml` workspace members and each `crates/*/Cargo.toml`
- SQL (SQLite dialect: DDL + FTS5 virtual tables) — migrations in `migrations/sqlite/*.up.sql` / `*.down.sql`, queries via `sqlx` in `crates/storage-sqlite/src/` and `crates/quran-search/src/fts5.rs`

**Secondary:**

- TOML — user configuration (`examples/config/default.toml`, parsed in `crates/config/src/lib.rs`)
- HTML (server-rendered debug reader only) — `crates/server/src/api.rs` (`debug_reader_handler`)
- Shell/YAML — CI and container glue (`.github/workflows/ci.yml`, `Dockerfile`, `docker-compose.yml`)

## Runtime

**Environment:**

- Tokio 1.53.1 multi-thread async runtime (`tokio` workspace dep with `rt-multi-thread, macros, time, fs, sync, signal, net, io-util` features) — used by `crates/server`, `crates/cli`, `crates/storage-sqlite`, `crates/jobs`, `crates/sources`
- Single binary `qai` (`[[bin]] name = "qai"` in `crates/cli/Cargo.toml`, `path = "src/main.rs"`); library + binary split only in `crates/cli` (`src/main.rs` + `src/lib.rs`)

**Package Manager:**

- Cargo with workspace resolver `"2"` (`Cargo.toml`)
- Lockfile: present (`Cargo.lock`, ~85 KB, `--locked` enforced in `Dockerfile` build step)

## Frameworks

**Core:**

- axum 0.8.9 — HTTP server in `crates/server/src/api.rs` (`router()`, `serve()`) and `crates/server/src/lib.rs` (loopback bind guard)
- tower-http 0.6.11 (`trace`, `limit`, `timeout` features) + tower 0.5.3 (`limit`) — middleware layers in `crates/server/src/api.rs` (`TraceLayer`, `RequestBodyLimitLayer`, `TimeoutLayer`, `ConcurrencyLimitLayer`)
- clap 4.6.6 (`derive, env, wrap_help` features) — CLI definition in `crates/cli` (`src/main.rs`, `src/lib.rs`)
- sqlx 0.8.6 (`runtime-tokio, sqlite` features, `default-features = false` — no Postgres/MySQL drivers compiled) — all persistence in `crates/storage-sqlite/src/lib.rs`, `crates/storage-sqlite/src/migrate.rs`, `crates/storage-sqlite/src/quran.rs`, FTS5 index in `crates/quran-search/src/fts5.rs`

**Testing:**

- Rust built-in `#[test]` / `#[tokio::test]` harness — used in every implemented crate
- proptest 1.11.0 — property tests in `crates/quran-core`, `crates/quran-corpus`, `crates/quran-search`, `crates/application`, `crates/domain`
- trycmd 0.15.11 — CLI snapshot tests in `crates/cli` (dev-dependency)
- tempfile 3.27.0 — isolated DB/config fixtures in tests across `crates/storage-sqlite`, `crates/config`, `crates/server`, `crates/application`, `crates/testkit`
- insta 1.x — declared in workspace `[workspace.dependencies]` (`Cargo.toml`) but not yet locked/used (absent from `Cargo.lock`)
- cargo-llvm-cov + `xtask coverage-gate` — coverage gate in `.github/workflows/ci.yml` (domain / provenance / audit / security ≥ 85%)

**Build/Dev:**

- xtask pattern via `.cargo/config.toml` alias (`xtask = "run --quiet --package xtask --"`) — implementation in `crates/xtask/` (`arch-check`, `migrate-check`, `gen-schema`, `coverage-gate`); extra dep `dirs 5.0.1`
- cargo-deny with `deny.toml` (advisories `deny`, unknown registry/git `deny`, GPL/AGPL/LGPL `deny`, allowlist MIT/Apache-2.0/BSD/ISC/Unicode-DFS-2016/Zlib/MPL-2.0) — enforced in `.github/workflows/ci.yml` (`deny` job, `--all-features`)
- rustfmt (stable-only options in `rustfmt.toml`: edition 2024, 4 spaces, `reorder_imports`, Unix newlines) + clippy (`-D warnings` in CI, `unsafe_code = "forbid"` workspace lint in `Cargo.toml`, extra `doc-valid-idents` in `clippy.toml`)
- Docker multi-stage build (`Dockerfile`: `rust:1.97.1-bookworm` → `gcr.io/distroless/cc-debian12:nonroot`, UID 65532)

## Key Dependencies

**Critical:**

- serde 1.0.229 + serde_json 1.x — envelope/DTO serialization everywhere (`crates/domain`, `crates/server/src/api.rs`, `crates/config/src/lib.rs`)
- thiserror 1.0.69 and 2.0.20 (both locked; workspace declares `"2"`) + anyhow 1.0.104 (xtask only) — error handling convention
- async-trait 0.1.92 — async repository/backend traits (`crates/storage/src/`, `crates/server/src/api.rs` `QuranApiBackend`, `crates/config/src/secret_store.rs` `SecretStore`)
- uuid 1.26.0 (`v4, serde`) + time 0.3.55 (`serde, formatting, parsing, macros`) — IDs and timestamps in `crates/domain`, `crates/jobs`, `crates/provenance`, `crates/sources`, `crates/storage-sqlite`
- tracing 0.1.44 + tracing-subscriber 0.3.23 (`env-filter, json, fmt`) — structured logging in `crates/observability/src/lib.rs` (text/JSON formats, secret-redacting writer, `RUST_LOG` filter)
- metrics 0.24.6 — in-process metric catalog in `crates/observability/src/metrics.rs` (`qai_jobs_*`, `qai_db_*`, `qai_audit_events_total`, `qai_errors_total`, `qai_doctor_checks_total`)

**Infrastructure:**

- Cryptography: sha2 0.10.9 (content hashes, secret-file key derivation in `crates/config/src/secret_store.rs`, corpus hashing in `crates/quran-corpus/src/hashing.rs`), ed25519-dalek 2.2.0 (dataset signature verification in `crates/sources/`), chacha20poly1305 0.10.1 (`getrandom` feature; XChaCha20-Poly1305 encrypted secret file in `crates/config/src/secret_store.rs`), base64 0.22.1, zeroize 1.9.0 (`zeroize_derive`; `Secret<T>` redaction in `crates/config/src/secret.rs`)
- Text/linguistics: unicode-segmentation 1.13.3 (grapheme-cluster `char_count` per ADR-0104 in `crates/quran-core/src/text.rs`), unicode-normalization 0.1.25 (NFC/NFD auditor in `crates/quran-corpus/src/unicode.rs`, rules in `crates/quran-normalization/src/rules/n16.rs`), regex-automata 0.4.18 (DFA-only pattern search, no backtracking, in `crates/quran-search/`), csv 1.4.0 (second dataset adapter in `crates/quran-corpus/src/adapters.rs`, `crates/quran-corpus/src/import.rs`), similar 2.7.0 (char-level edition diff in `crates/quran-corpus/src/differ.rs`), lru 0.12.5 (canonical lookup cache in `crates/application/`)
- HTTP client: reqwest 0.12.28 (`json` feature, `default-features = false` — rustls/TLS stack excluded) — declared in workspace deps; direct call sites live behind the opt-in OTLP exporter (`opentelemetry-otlp` with `reqwest-client` feature, see `crates/observability/src/otlp.rs`). No product-code HTTP fetching exists yet; network egress is deny-by-default (`security.allow_network_egress = false` in `crates/config/src/lib.rs`)
- OS secrets (optional): keyring 3.6.3 — optional dep of `crates/config`, enabled only via `keychain` feature (`crates/config/src/secret_store.rs` `KeychainSecretStore`; stub returns `Unsupported` without the feature)
- Observability export (optional): opentelemetry 0.27.1 + opentelemetry_sdk 0.27.1 (`rt-tokio`) + opentelemetry-otlp 0.27.0 (`http-proto, reqwest-client`) + tracing-opentelemetry 0.28.0 — all optional deps of `crates/observability`, enabled only via the `otlp` feature (`crates/observability/src/otlp.rs`)
- Config parsing: toml 0.8.23 — file layer in `crates/config/src/lib.rs`

## Configuration

**Environment:**

- Four-layer precedence implemented in `crates/config/src/lib.rs` (`Config::load`): built-in `Default` < TOML file < `QAI__`-prefixed env (`QAI__SERVER__PORT` style, double-underscore nesting) < CLI `--dotted.key value` overrides; every value tracked in `OriginMap` (`crates/config/src/origin.rs`)
- Template (placeholders only, safe to read): `.env.example` (`QAI_APP_MODE`, `QAI_DATA_DIR`, `QAI_SERVER_BIND/PORT`, `QAI_STORAGE__SQLITE__PATH`, `QAI_LOG_LEVEL`, `QAI_TELEMETRY_ENABLED`, `QAI_SECRETS__BACKEND`, `QAI_SECRET_` prefix). Never commit a real `.env` (gitignored; see `.gitignore`)
- Secrets are `secret://<backend>/<path>` references, never values (`crates/config/src/secret_store.rs`): `env` backend reads `QAI_SECRET_<KEY>` (read-only), `encrypted_file` backend is an XChaCha20-Poly1305 JSON map at `${app.data_dir}/secrets.age` keyed by passphrase from env, `keychain` backend is OS-native behind the `keychain` feature. `Secret<T>` redacts `Display`/`Debug`/serialization (`crates/config/src/secret.rs`)
- Key interpolation: `${app.data_dir}` in `examples/config/default.toml` and `crates/config/src/lib.rs` (`resolve_interpolation`, cycle-guarded at 50 iterations)

**Build:**

- `Cargo.toml` (workspace: resolver, centralized `[workspace.dependencies]`, `unsafe_code = "forbid"` lints)
- `rust-toolchain.toml` (toolchain pin), `rustfmt.toml`, `clippy.toml`, `deny.toml`
- `.cargo/config.toml` (xtask alias), `xtask/allowlist.toml` (dependency allowlist for arch-check)
- `Dockerfile`, `docker-compose.yml`, `.dockerignore`
- `migrations/sqlite/checksums.json` (migration integrity manifest, verified by `crates/storage-sqlite/src/migrate.rs` and `cargo xtask migrate-check`)

## Platform Requirements

**Development:**

- Rust 1.97.1 stable with `rustfmt` + `clippy` components (`rust-toolchain.toml`); SQLite (bundled via `sqlx`, no separate server); Linux/macOS/Windows all tested in CI (`.github/workflows/ci.yml` test matrix: `ubuntu-latest, macos-latest, windows-latest`); `llvm-tools-preview` + `cargo-llvm-cov` for the coverage job

**Production:**

- Deployment target: single static binary `qai` on distroless `cc-debian12:nonroot` as UID 65532 (`Dockerfile`), `QAI_DATA_DIR=/data`, default command `qai version`
- `docker-compose.yml`: `network_mode: none` (no network), `read_only: true`, `cap_drop: ALL`, `no-new-privileges`, 64 MB `/tmp` tmpfs, named volume `qai-data` at `/data`, serves `serve --bind 127.0.0.1:8737`
- Server binds loopback only — enforced in code (`crates/server/src/lib.rs` `loopback_addr`, `NonLoopback` error) and config validation (`crates/config/src/lib.rs` `Config::validate` rejects non-loopback binds without `tls = "required"`); note there is no TLS terminator in the dependency tree (no rustls/native-tls), so `tls = "required"` outside localhost is a deployment precondition, not an in-process feature
- Storage: local SQLite file (`${app.data_dir}/qai.db`, WAL, `synchronous = full`, FK on, 8-connection pool with read-only replica pool) + filesystem object store (`${app.data_dir}/objects`, 512 MB max file) — see `examples/config/default.toml` and `crates/storage-sqlite/src/lib.rs`

---

*Stack analysis: 2026-09-22*
