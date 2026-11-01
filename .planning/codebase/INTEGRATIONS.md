---
last_mapped_commit: 6891d7b
last_mapped_at: 2026-11-01
---
# External Integrations

**Analysis Date:** 2026-11-01

## APIs & External Services

**LLM / embedding providers:**

- None integrated. The `crates/llm/`, `crates/embeddings/`, `crates/reranking/`, `crates/model-router/`, `crates/mcp/` crates remain placeholders (empty; `//! Phase N` doc only per workspace `Cargo.toml` comment). `crates/config/src/secret_store.rs` already anticipates provider keys (`secret://env/openai/default` used in tests/docs), but no provider SDK or HTTP call exists in product code. `docs/01-requirements/requirements.md` and phase plans describe future local-first + opt-in remote providers — not yet built.

**HTTP clients:**

- reqwest 0.12.28 — declared in workspace `[workspace.dependencies]` (`Cargo.toml`) with `default-features = false` (no default TLS stack). The only compiled usage path is the opt-in OTLP trace exporter (`opentelemetry-otlp` with `reqwest-client` feature in `crates/observability/src/otlp.rs`). Product-code dataset fetching / provider calls do not exist yet.
  - SDK/Client: `reqwest` (workspace)
  - Auth: n/a

**Dataset / source ingestion:**

- Local files only. Corpus import adapters read CSV and JSON dataset files from disk (`crates/quran-corpus/src/adapters.rs`, `crates/quran-corpus/src/import.rs`, `crates/quran-corpus/src/format.rs`); authenticity via Ed25519 signatures (`ed25519-dalek` in `crates/sources/`) and SHA-256 hashes (`crates/quran-corpus/src/hashing.rs`). No remote fetch: `security.allow_network_egress = false` by default (`crates/config/src/lib.rs`, `examples/config/default.toml`), with SSRF private-range blocking, download/archive size caps, and `follow_symlinks = false` as defense-in-depth when egress is ever enabled.

**Frontend → backend (same-origin API surface):**

- The `web/` SPA talks to the same binary over same-origin `fetch` — no separate API host, no API keys, no CORS. Typed client in `web/src/api/client.ts` mirrors the server `Envelope<T>`/`Meta` contract (`get`/`post` helpers carry the server's typed `error.code` via `ApiRequestError`): `fetchAyahs`, `resolveReference`, `fetchCitation`, `runTool(name, params)` (generic `POST /api/v1/quran/tool/{name}` dispatcher), `ragQuery`/`ragDebug`, plus research (`search`, `family`, `rootFrequency`, `lemmaFrequency`, `frequency`, `distribution`, `cooccurrence`) and graph (`graphNeighbors`, `graphPath`) helpers.
- 13 SPA routes in `web/src/router.tsx` (`/`, `/search`, `/research`, `/words`, `/tools`, `/citations`, `/hadith`, `/isnad`, `/tafsir`, `/scripture`, `/graph`, `/rag-debug`, `/explorer`; deep links `/read/<edition>/<surah>:<ayah>?citation=<id>`), each backed by a view in `web/src/views/` calling the client above. `ExplorerView` dispatches arbitrary requests against the local engine.

## HTTP Route Inventory (all served by `qai serve`)

**Health / meta (`crates/server/src/api.rs` `router()`):**

- `GET /healthz` → `"ok"`; `GET /readyz` → `"ready"`; `GET /api/v1/meta` → `{name: "Q-ai", version}` envelope

**Quran reader (GET, `crates/server/src/api.rs`):**

- `GET /api/v1/quran/editions[?status&language]`, `GET /api/v1/quran/editions/{slug}`, `GET /api/v1/quran/surahs[?edition]`, `GET /api/v1/quran/surahs/{number}[?edition]`
- `GET /api/v1/quran/ayahs/{reference}[?edition&translations&glosses&tokens]` (via `ToolRegistry::get_ayah`; ETag + `If-None-Match → 304`, `Content-Language: ar`)
- `GET /api/v1/quran/context/{reference}[?edition&before&after&boundary&max_ayahs]` (`surah|juz|ruku|page|none`)
- `GET /api/v1/quran/divisions/{kind}/{number}[?edition]`, `GET /api/v1/quran/tokens/{reference}[?edition]`, `GET /api/v1/quran/resolve?ref=…`, `GET /api/v1/quran/citations/{id}` (hard-fail verdicts never served as 200 via `citations::require_exact`)

**Generic tool dispatcher:**

- `POST /api/v1/quran/tool/{name}` (`tool_handler` in `crates/server/src/api.rs`) — serves every registered tool through the single `application::quran_tools::dispatch_registered_tool` table the CLI `quran tool` verb shares (checksums cannot diverge). Deny-by-default gates: mutating isnad tools (`QAI-ISNAD-0326`) and mutating RAG tools (`QAI-RAG-0326`) are refused unless `policy.tool_execution_default = "allow"`.

**Search — five lexical modes (POST, SSE-capable):**

- `POST /api/v1/quran/search/exact|normalized|phrase|concatenated|regex` — same `application::quran_search_api::SearchBackend` services the CLI uses; `Accept: text/event-stream` switches to `hit` + terminal `totals` SSE events. Regex is DFA-only (budget-capped `timeout_ms`, default 3000 / max 10000, `X-Principal` attributed).

**Lexicon / counts (POST):**

- `POST /api/v1/quran/family`, `POST /api/v1/quran/count/root-frequency|lemma-frequency` (SC3/SC4 word-family and root/lemma frequency)
- `POST /api/v1/quran/count/frequency|distribution|occurrences|hapax|cooccurrence|collocation|numeric-report|missing-form|near-duplicates|interval|unusual-usage` (D-10 count instruments; thin wrappers over the same `LexiconBackend` services, default profile `L3.diacritics`)

**Graph reads (POST, read-only):**

- `POST /api/v1/quran/graph/neighbors|path|subgraph|pattern|root-family` — thin over `application::quran_graph_api` services with shared budgets, pre-flight validation, and projection-pinned reproducibility blocks. No mutation route exists under `/api/v1/quran/graph`.

**Hadith / tafsir / verse links:**

- `GET /api/v1/hadith/{collection}/{source_number}`, `POST /api/v1/hadith/search`, `GET /api/v1/hadith/{collection}/{source_number}/gradings`
- `GET /api/v1/tafsir/{work_slug}`, `POST /api/v1/tafsir/search`
- `GET /api/v1/verse/{surah}/{ayah}/links` (Phase 6 SC3 verse-link service, optional `VerseLinkService` on `AppState`)

**Normalization:**

- `POST /api/v1/quran/normalization/preview` (stateless, no DB; byte-identical trace to `qai quran normalize`), `GET /api/v1/quran/normalization/profiles` (built-in ladder catalog)

**Scripture + cross-domain research (`crates/server/src/routes/scripture.rs`, merged into the main router):**

- `GET /api/v1/scripture/editions`, `GET|POST /api/v1/scripture/search`, `GET /api/v1/scripture/{edition}/{book}/{chapter}/{verse}`, `GET /api/v1/scripture/translations/{verse_id}`, `GET|POST /api/v1/scripture/parallel`, `GET|POST /api/v1/scripture/research`
- Aliases: `GET|POST /api/v1/cross-domain/research` and `GET|POST /api/v1/research/cross-domain` (same handler)

**Debug / static:**

- `GET /debug/read/{edition}/{surah}` (non-product engineering preview) + `GET /debug/assets/reader.woff2` (only on `router_with_debug_font`)
- All non-API paths → embedded SPA (`crates/server/src/webassets.rs` `spa_fallback`); unknown `/api/*` paths 404 and never fall through to `index.html`

## Tool Registry (cross-crate boundary)

**Canonical registry (`crates/application/src/quran_tools.rs` `dispatch_registered_tool` + `crates/tool-registry/`, `crates/tools/`, `crates/tool-sdk/`):**

- Reader tools (via `ReaderToolBackend` over `QuranReaderService`): `quran.get_ayah`, `quran.get_context`, `quran.search`, `quran.root`, `quran.lemma`, `quran.morphology`, `quran.family`
- Graph tools (via `GraphToolBackend` opened on the same handle): `quran.graph_neighbors`, `quran.graph_path`, `quran.graph_subgraph`, `quran.graph_pattern`, `quran.graph_root_family`
- Phase 6: `hadith.get`, `hadith.search`, `hadith.gradings`, `tafsir.get`, `tafsir.search`, `isnad.narrator`, `isnad.rijal`, `isnad.trace`, `isnad.identity_propose|accept|reject|dispute|supersede`
- Phase 8 RAG: `rag.create_project` (`rag.create` alias), `rag.import`, `rag.query`, `rag.debug`, `rag.source_status|activate|deactivate|tombstone`
- Phase 9 scripture: `scripture.get`, `scripture.search`, `scripture.translations`, `scripture.editions`, `scripture.parallel_passages`
- Every result carries edition identity, canonical references, reproducibility records, and `research_checksum` (`sha256:<hex>`, projected — never recomputed — into the HTTP `meta` by `crates/server/src/api.rs`)
- Backend traits consumed by the server: `QuranApiBackend`, `SearchBackend` (`application::quran_search_api`), `LexiconBackend` (`application::quran_lexicon_api`), `GraphBackend` (`application::quran_graph_api`), optional `VerseLinkService` — all defined in `crates/application/src/`, implemented over `QuranReaderService`

## Data Storage

**Databases:**

- SQLite (only backend; Postgres is a named future option — `storage.backend = "sqlite" # sqlite | postgres (Phase 12)` in `examples/config/default.toml` — with no implementation)
  - Connection: SQLite file path from config (`storage.sqlite.path`, default `${app.data_dir}/qai.db`; env `QAI_STORAGE__SQLITE__PATH`; `QAI__STORAGE__SQLITE__PATH` nested mapping) — see `crates/config/src/lib.rs`, `.env.example`
  - Client: `sqlx 0.8.6` `SqlitePool` with split write pool (single serialized connection, `synchronous = FULL`) + read pool (`query_only = ON`), WAL journal mode, FK enforcement, 5 s busy timeout (`crates/storage-sqlite/src/lib.rs`)
  - Schema: versioned migrations `migrations/sqlite/0001_*.up.sql` … `0044_verse_links_scripture_source.up.sql` (+ `.down.sql` for 0001–0006) with `checksums.json` integrity manifest, applied/verified by `crates/storage-sqlite/src/migrate.rs` and `cargo xtask migrate-check`. Latest generations cover scripture books/verses/translations/search-forms (0039–0043) and the verse-links scripture source (0044); Phase 8 RAG generations 0035–0038 (traits, rag core)
  - Full-text search: SQLite FTS5 virtual tables (`ayah_fts` + `fts5vocab` term dictionary) in per-generation directories (`<root>/gen-<N>/index.db`) — `crates/quran-search/src/fts5.rs`; DFA-only regex over the term dictionary via `regex-automata`
  - Trait abstraction: `crates/storage/src/` (`Database`, `UnitOfWork`, repository traits incl. `OutboxRepository`, hadith/tafsir/scripture/isnad/RAG repos); SQLite impl in `crates/storage-sqlite/src/` (`SqliteQuranRepository` in `crates/storage-sqlite/src/quran.rs`)

**File Storage:**

- Local filesystem object store only: `${app.data_dir}/objects` (`storage.objects.root`, `backend = "filesystem"`, 512 MB `max_file_bytes`) — `examples/config/default.toml`, `crates/config/src/lib.rs` (`ObjectsConfig`). No S3/GCS/Azure integration. Encrypted secret file at `${app.data_dir}/secrets.age` (`crates/config/src/secret_store.rs`).

**Caching:**

- In-process LRU cache only: `lru 0.12.5` canonical lookup cache in `crates/application/` (ADR-0113). HTTP layer uses `ETag` (`"<text_hash>:<corpus_generation>"`) + `Cache-Control: public, max-age=3600` + conditional `If-None-Match → 304` in `crates/server/src/api.rs`. No Redis/Memcached.

## Authentication & Identity

**Auth Provider:**

- None (no external IdP, no OAuth/OIDC, no API keys for the HTTP surface). Local-first single-user model.
  - Implementation: `principals` table (`local_user | service | system` kinds) in `migrations/sqlite/0001_core.up.sql`; loopback-only bind enforcement in `crates/server/src/lib.rs` (`loopback_addr` rejects non-loopback with `ServerError::NonLoopback`); `require_auth_outside_localhost` + `tls` config validation in `crates/config/src/lib.rs` (`Config::validate`); deny-by-default tool execution (`policy.tool_execution_default = "deny"`, `command_execution_enabled = false`, `canonical_write_requires_approval = true`). API-key/OAuth auth is a future server-mode concern, not implemented.

## Monitoring & Observability

**Error Tracking:**

- None (no Sentry/Honeybadger/Bugsnag integration or dependency).

**Logs:**

- `tracing` + `tracing-subscriber` to stderr (text default, JSON optional; `RUST_LOG`-driven `EnvFilter`; optional file in `logging.file`) with a `RedactingWriter` that scrubs `api_key/password/secret/token/credential` key=value pairs (`crates/observability/src/lib.rs`; `redact_secrets = true` default in `crates/config/src/lib.rs`). No remote log shipper.

**Metrics:**

- In-process `metrics 0.24.6` catalog only (`crates/observability/src/metrics.rs`: `qai_jobs_*`, `qai_db_query_duration_seconds`, `qai_db_pool_in_use`, `qai_audit_events_total`, `qai_config_reloads_total`, `qai_errors_total`, `qai_doctor_checks_total`). No Prometheus exporter / StatsD / Datadog sink wired.

**Traces:**

- Opt-in OTLP/HTTP trace export only, off by default (`telemetry.enabled = false`, PRD §39). Requires both config (`telemetry.otlp_endpoint`, e.g. `http://localhost:4318`) and the `otlp` cargo feature (`crates/observability/src/otlp.rs`: `build_provider`/`init_otlp`, service name `qai`); content-bearing fields (`query_text`, `prompt_text`, `document_content`, `research_question`, `model_response`) are denylisted by the telemetry privacy gate (`crates/observability/src/telemetry.rs`). No collector is bundled; compatible with any OTLP/HTTP endpoint (e.g. local Jaeger/Tempo) when the operator opts in.

## CI/CD & Deployment

**Hosting:**

- Self-hosted / local-first. No cloud provider integration (no AWS/GCP/Azure SDKs). Distribution is the `qai` binary via distroless container (`Dockerfile`) or `docker-compose.yml` (network-less, read-only, `qai-data` volume) or direct `cargo build --locked --release -p cli`. The same binary serves the embedded SPA (`web/dist` via `rust-embed` in `crates/server/src/webassets.rs`); note the Docker build does not run `npm run build`, so an image built from a clean tree serves the API with the SPA 404 path until `web/dist` is built and the image rebuilt.

**CI Pipeline:**

- GitHub Actions (`.github/workflows/ci.yml`): `deny` (cargo-deny advisories/bans/licenses/sources), `lint` (`cargo fmt --check`, `clippy -D warnings`), `arch` (`cargo xtask arch-check` + `migrate-check`), `test` (3-OS matrix: ubuntu/macos/windows, `cargo test --workspace`), `schemas` (`cargo xtask gen-schema` + git-diff check on `docs/schemas/`), `coverage` (`cargo-llvm-cov` + `xtask coverage-gate`). Uses `actions/checkout@v4`, `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`, `embarkstudios/cargo-deny-action@v2`, `actions/upload-artifact@v4`. No release/publish job (all crates `publish = false`). The web toolchain (`npm test` / `npm run build`) is not currently gated in CI — frontend verification is local-only.

## Environment Configuration

**Required env vars:**

- None strictly required — everything has built-in defaults (`Config::default` in `crates/config/src/lib.rs`). Effective configuration sources, in precedence order:
  - `QAI_DATA_DIR` (shorthand honored by `crates/cli/src/lib.rs`) and `QAI__…` nested mapping (e.g. `QAI__SERVER__PORT`, `QAI__STORAGE__SQLITE__PATH`, `QAI__TELEMETRY__ENABLED`) — see `crates/config/src/lib.rs` (`merge_env`) and `.env.example`
  - `QAI_SECRET_<KEY>` for secret values via the `env` secret backend (e.g. future provider keys) — `crates/config/src/secret_store.rs` (`EnvSecretStore`, `SecretRef::env_var`)
  - `QAI_APP_MODE`, `QAI_LOCALE`, `QAI_SERVER_BIND`, `QAI_SERVER_PORT`, `QAI_STORAGE__SQLITE__PATH`, `QAI_LOG_LEVEL`, `QAI_TELEMETRY_ENABLED`, `QAI_SECRETS__BACKEND` — legacy flat names in `.env.example`
  - `RUST_LOG` — tracing filter (`crates/observability/src/lib.rs`)
  - Secret-file passphrase env var (operator-chosen, read via `EncryptedFileSecretStore::from_env` in `crates/config/src/secret_store.rs`)
  - OS keychain (service `qai`, account `<segments joined by />`) only with the `keychain` feature (`keyring` crate, `crates/config/src/secret_store.rs`)

**Secrets location:**

- Never in SQLite (only `secret://<backend>/<path>` references persist). Values live in: process environment (`QAI_SECRET_*`), XChaCha20-Poly1305 encrypted file (`${app.data_dir}/secrets.age`), or OS keychain (macOS Keychain / Secret Service / Windows Credential Manager via `keyring`, feature-gated). See `crates/config/src/secret_store.rs` and `docs/02-architecture/decisions/ADR-0005-secret-storage.md`.

## Webhooks & Callbacks

**Incoming:**

- None. The HTTP surface in `crates/server/src/api.rs` + `crates/server/src/routes/scripture.rs` is read-only product routes (`/healthz`, `/readyz`, `/api/v1/meta`, `/api/v1/quran/*` editions/surahs/ayahs/context/divisions/tokens/resolve/citations/normalization/search/family/count/graph, hadith/tafsir/verse-link/scripture routes, generic `POST /api/v1/quran/tool/{name}`) plus the non-product `/debug/read/{edition}/{surah}` engineering preview. No webhook receivers, no signature-verification middleware.

**Outgoing:**

- None. No webhook dispatch, no event callbacks, no notification channels. Durable cross-crate side effects use the SQLite outbox table (`migrations/sqlite/0006_outbox_generations_tombstones.up.sql`, `OutboxRepository` in `crates/storage/src/`), consumed internally by the `crates/jobs` worker loop — not a network integration.

---

*Integration audit: 2026-11-01*
