---
last_mapped_commit: 6891d7b
last_mapped_at: 2026-11-01
---
# Technology Stack

**Analysis Date:** 2026-11-01

## Languages

**Primary:**

- Rust 1.97.1 (pinned via `rust-toolchain.toml`, channel `1.97.1` with `rustfmt` + `clippy` components) — edition 2024 across every crate in `Cargo.toml` workspace members and each `crates/*/Cargo.toml`
- TypeScript ~6.0.2 (strict, `verbatimModuleSyntax`, `erasableSyntaxOnly`, `noUnusedLocals/Parameters`) — entire `web/src/` SPA (`web/tsconfig.app.json`, `web/tsconfig.json`, `web/tsconfig.node.json`)
- SQL (SQLite dialect: DDL + FTS5 virtual tables) — migrations in `migrations/sqlite/*.up.sql` / `*.down.sql` (44 generations, `0001`…`0044`), queries via `sqlx` in `crates/storage-sqlite/src/` and `crates/quran-search/src/fts5.rs`
- CSS via Tailwind v4 (`@tailwindcss/vite` plugin, design tokens in `web/src/index.css` / `web/src/tokens/`) — no separate CSS framework

**Secondary:**

- TOML — user configuration (`examples/config/default.toml`, parsed in `crates/config/src/lib.rs`)
- HTML (server-rendered debug reader only) — `crates/server/src/api.rs` (`debug_reader_handler`)
- Shell/YAML — CI and container glue (`.github/workflows/ci.yml`, `Dockerfile`, `docker-compose.yml`)

## Runtime

**Environment:**

- Tokio 1.53.1 multi-thread async runtime (`tokio` workspace dep with `rt-multi-thread, macros, time, fs, sync, signal, net, io-util` features) — used by `crates/server`, `crates/cli`, `crates/storage-sqlite`, `crates/jobs`, `crates/sources`
- Node 24 (observed `v24.18.0`) + Vite 8.3.3 dev server / build — `web/` only (`web/package.json` scripts `dev`, `build`, `preview`)
- Browsers (evergreen, ES2023 target) — compiled SPA output in `web/dist/` (gitignored build artifact), embedded into the binary at compile time
- Single binary `qai` (`[[bin]] name = "qai"` in `crates/cli/Cargo.toml`, `path = "src/main.rs"`); library + binary split only in `crates/cli` (`src/main.rs` + `src/lib.rs`)

**Package Manager:**

- Cargo with workspace resolver `"2"` (`Cargo.toml`); 56 workspace members (`crates/*` + `xtask`)
- Lockfile: present (`Cargo.lock`, enforced with `--locked` in `Dockerfile` build step and CI-spirit `cargo build --locked --release -p cli --bin qai`)
- npm with lockfile: present (`web/package-lock.json`); `web/` is `private: true`, `type: module`, version `0.0.0`

## Frameworks

**Core:**

- axum 0.8.9 — HTTP server in `crates/server/src/api.rs` (`router()`, `router_with_debug_font()`) plus `crates/server/src/routes/scripture.rs` (scripture + cross-domain research router, merged via `.merge()`), `crates/server/src/lib.rs` (loopback bind guard), `crates/server/src/webassets.rs` (SPA fallback)
- tower-http 0.6.11 (`trace`, `limit`, `timeout` features) + tower 0.5.3 (`limit`) — middleware layers in `crates/server/src/api.rs` (`TraceLayer`, `RequestBodyLimitLayer` 1 MiB, `TimeoutLayer` 30 s → `408`, `ConcurrencyLimitLayer` 128)
- rust-embed 8.13.0 (`mime-guess` via `mime_guess 2`) — compile-time embed of `web/dist` in `crates/server/src/webassets.rs` (`SpaAssets`, `spa_fallback`; `allow_missing` so plain `cargo build` works before `npm run build`)
- ratatui 0.30.2 — TUI cockpit in `crates/tui/` (deps on `application`, `config`, `domain`, `quran-core`, `storage`, `tool-registry`, `tools`)
- clap 4.6.6 (`derive, env, wrap_help` features) — CLI definition in `crates/cli` (`src/main.rs`, `src/lib.rs`; verbs in `src/quran.rs`, `src/hadith.rs`, `src/tafsir.rs`, `src/isnad.rs`, `src/rag.rs`, `src/scripture.rs`, `src/links.rs`, `src/doctor.rs`)
- sqlx 0.8.6 (`runtime-tokio, sqlite` features, `default-features = false` — no Postgres/MySQL drivers compiled) — all persistence in `crates/storage-sqlite/src/lib.rs`, `crates/storage-sqlite/src/migrate.rs`, `crates/storage-sqlite/src/quran.rs`, FTS5 index in `crates/quran-search/src/fts5.rs`
- React 19.3.0 + React DOM 19.3.0 — SPA in `web/src/` (`App.tsx`, `main.tsx`, `router.tsx`, 13 views in `web/src/views/`)
- Tailwind CSS 4.3.3 via `@tailwindcss/vite` 4.3.3 — styling (`web/vite.config.ts` plugins `[react(), tailwindcss()]`, `base: '/'` for same-origin serving)
- shadcn/ui (`shadcn ^4.23.0`, `components.json`) + Base UI (`@base-ui/react ^1.9.0`) + `cmdk`, `class-variance-authority`, `lucide-react ^1.62.0`, `sonner`, `next-themes`, `tw-animate-css` — component library in `web/src/components/` (`ui/`, `layout/`, `command/`)

**Testing:**

- Rust built-in `#[test]` / `#[tokio::test]` harness — used in every implemented crate
- proptest 1.11.0 — property tests in `crates/quran-core`, `crates/quran-corpus`, `crates/quran-search`, `crates/application`, `crates/domain`
- trycmd 0.15.11 — CLI snapshot tests in `crates/cli` (dev-dependency)
- tempfile 3.27.0 — isolated DB/config fixtures in tests across `crates/storage-sqlite`, `crates/config`, `crates/server`, `crates/application`, `crates/testkit`
- insta 1.x — declared in workspace `[workspace.dependencies]` (`Cargo.toml`) but not yet locked/used (absent from `Cargo.lock`)
- cargo-llvm-cov + `xtask coverage-gate` — coverage gate in `.github/workflows/ci.yml` (domain / provenance / audit / security ≥ 85%)
- Vitest 5.0.3 + jsdom 30.1.2 + Testing Library (`@testing-library/react 16.3.3`, `@testing-library/dom 10.4.2`) — frontend tests (`web/vitest.config.ts`: `environment: jsdom`, `include: ['src/**/*.test.{ts,tsx}']`; run via `npm test` → `vitest run`); co-located `*.test.tsx` beside views/components (`router.test.tsx`, `ReadingView.test.tsx`, `ToolsView.test.tsx`, `ToolLedger.test.tsx`, …)
- TypeScript build gate — `npm run build` runs `tsc -b && vite build`, so type errors fail the web build

**Build/Dev:**

- xtask pattern via `.cargo/config.toml` alias (`xtask = "run --quiet --package xtask --"`) — implementation in `crates/xtask/` (`arch-check`, `migrate-check`, `gen-schema`, `coverage-gate`); extra dep `dirs 5.0.1`
- cargo-deny with `deny.toml` (advisories `deny`, unknown registry/git `deny`, GPL/AGPL/LGPL `deny`, allowlist MIT/Apache-2.0/BSD/ISC/Unicode-DFS-2016/Zlib/MPL-2.0) — enforced in `.github/workflows/ci.yml` (`deny` job, `--all-features`)
- rustfmt (stable-only options in `rustfmt.toml`: edition 2024, 4 spaces, `reorder_imports`, Unix newlines) + clippy (`-D warnings` in CI, `unsafe_code = "forbid"` workspace lint in `Cargo.toml`, extra `doc-valid-idents` in `clippy.toml`)
- oxlint 1.81.0 — JS/TS linter (devDependency in `web/package.json`)
- `@vitejs/plugin-react` 6.1.2 — React Fast Refresh for Vite dev/build/test
- Docker multi-stage build (`Dockerfile`: `rust:1.97.1-bookworm` → `gcr.io/distroless/cc-debian12:nonroot`, UID 65532) — note: the Rust stage does NOT build `web/dist`; the SPA must be built separately (`npm run build` in `web/`) before the binary embeds it, otherwise `spa_fallback` 404s with "SPA not built"

## Key Dependencies

**Critical:**

- serde 1.0.229 + serde_json 1.x — envelope/DTO serialization everywhere (`crates/domain`, `crates/server/src/api.rs`, `crates/config/src/lib.rs`, `web/src/api/client.ts` mirrors the `Envelope<T>`/`Meta` contract)
- thiserror 1.0.69 and 2.0.20 (both locked; workspace declares `"2"`) + anyhow 1.0.104 (xtask only) — error handling convention
- async-trait 0.1.92 — async repository/backend traits (`crates/storage/src/`, `crates/server/src/api.rs` `QuranApiBackend`, `crates/config/src/secret_store.rs` `SecretStore`)
- uuid 1.26.0 (`v4, serde`) + time 0.3.55 (`serde, formatting, parsing, macros`) — IDs and timestamps in `crates/domain`, `crates/jobs`, `crates/provenance`, `crates/sources`, `crates/storage-sqlite`
- tracing 0.1.44 + tracing-subscriber 0.23 (`env-filter, json, fmt`) — structured logging in `crates/observability/src/lib.rs` (text/JSON formats, secret-redacting writer, `RUST_LOG` filter)
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
- `web/package.json` + `web/vite.config.ts` + `web/vitest.config.ts` + `web/tsconfig*.json` + `web/components.json` (`shadcn` design-system config)
- `Dockerfile`, `docker-compose.yml`, `.dockerignore`
- `migrations/sqlite/checksums.json` (migration integrity manifest, verified by `crates/storage-sqlite/src/migrate.rs` and `cargo xtask migrate-check`)

**Run / verify commands:**

```bash
cargo build --locked --release -p cli --bin qai   # production binary
cargo test --workspace                             # full Rust test suite
cargo fmt --all -- --check                         # formatting gate (CI lint job)
cargo clippy --workspace --all-targets -- -D warnings  # lint gate (CI lint job)
cargo run -p xtask -- arch-check                   # workspace edge audit (CI arch job)
cargo run -p xtask -- migrate-check                # migration checksum audit (CI arch job)
cargo run -p xtask -- gen-schema                   # regenerate docs/schemas (CI schemas job)
cargo llvm-cov --workspace --lcov --output-path lcov.info  # coverage (CI coverage job)
cargo run -q -p xtask -- coverage-gate lcov.info   # enforce ≥85% on domain/provenance/audit/security
cd web && npm run dev      # Vite dev server
cd web && npm test         # vitest run (jsdom)
cd web && npm run build    # tsc -b && vite build → web/dist (must precede cargo build for SPA embed)
```

## Platform Requirements

**Development:**

- Rust 1.97.1 stable with `rustfmt` + `clippy` components (`rust-toolchain.toml`); SQLite (bundled via `sqlx`, no separate server); Linux/macOS/Windows all tested in CI (`.github/workflows/ci.yml` test matrix: `ubuntu-latest, macos-latest, windows-latest`); `llvm-tools-preview` + `cargo-llvm-cov` for the coverage job
- Node 24 + npm for `web/` (Vite 8.3.3, TypeScript ~6.0.2, Vitest 5.0.3); install via `npm install` in `web/` (lockfile `web/package-lock.json`); web path alias `@/*` → `web/src/*` shared by `tsconfig.app.json` and `vitest.config.ts`

**Production:**

- Deployment target: single static binary `qai` on distroless `cc-debian12:nonroot` as UID 65532 (`Dockerfile`), `QAI_DATA_DIR=/data`, default command `qai version`
- `docker-compose.yml`: `network_mode: none` (no network), `read_only: true`, `cap_drop: ALL`, `no-new-privileges`, 64 MB `/tmp` tmpfs, named volume `qai-data` at `/data`, serves `serve --bind 127.0.0.1:8737`
- Web UI is served same-origin by the same binary: embedded `web/dist` under every non-API path with SPA fallback (`crates/server/src/webassets.rs`); API paths (`/api/*`) never fall through to the SPA (typed 404 instead)
- Server binds loopback only — enforced in code (`crates/server/src/lib.rs` `loopback_addr`, `NonLoopback` error) and config validation (`crates/config/src/lib.rs` `Config::validate` rejects non-loopback binds without `tls = "required"`); note there is no TLS terminator in the dependency tree (no rustls/native-tls), so `tls = "required"` outside localhost is a deployment precondition, not an in-process feature
- Storage: local SQLite file (`${app.data_dir}/qai.db`, WAL, `synchronous = full`, FK on, 8-connection pool with read-only replica pool) + filesystem object store (`${app.data_dir}/objects`, 512 MB max file) — see `examples/config/default.toml` and `crates/storage-sqlite/src/lib.rs`

---

*Stack analysis: 2026-11-01*
