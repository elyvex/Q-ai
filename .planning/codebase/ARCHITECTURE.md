---
last_mapped_commit: 68b7ea6c1aacd1476d154550b2a6e9574614b6af
last_mapped_at: 2026-09-22
---
<!-- refreshed: 2026-09-22 -->

# Architecture

**Analysis Date:** 2026-09-22

## System Overview

```text
┌─────────────────────────────────────────────────────────────┐
│                    Edge / Interface Layer                    │
├──────────────────┬──────────────────┬───────────────────────┤
│   CLI (`qai`)    │  HTTP Server     │   xtask (build gates) │
│ `crates/cli/`    │ `crates/server/` │   `xtask/`            │
└────────┬─────────┴────────┬─────────┴──────────┬────────────┘
         │                  │                     │
         ▼                  ▼                     ▼
┌─────────────────────────────────────────────────────────────┐
│              Composition Root: `application`                 │
│         `crates/application/src/`                            │
│  `quran_cli.rs` `quran_reader.rs` `quran.rs` `quran_index.rs`│
│  `quran_search.rs` `quran_forms.rs` `quran_normalize.rs`     │
│  `quran_tools.rs` `quran_doctor.rs` `db.rs` `audit_bridge.rs`│
└─────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│              Quran Engine (pure + pipeline)                  │
│  `crates/quran-core/` → `crates/quran-corpus/`               │
│  `crates/quran-normalization/`  `crates/quran-search/`       │
│  `crates/citations/`  `crates/tools/` `crates/tool-registry/`│
└─────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│        Foundation Services (Phase 0 provenance core)         │
│  `crates/domain/` ← `crates/storage/` (traits only)          │
│  `crates/provenance/` `crates/audit/` `crates/jobs/`         │
│  `crates/sources/` `crates/config/` `crates/observability/`  │
└─────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│  Storage Adapter + SQLite                                    │
│  `crates/storage-sqlite/` ──→ `migrations/sqlite/`           │
│  (dual pools, WAL, `VACUUM INTO` backup)                     │
└─────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

| Component | Responsibility | File |
|-----------|----------------|------|
| `domain` | Pure vocabulary: IDs, primitives, hashing, licensing, security guards, `Diagnostic` | `crates/domain/src/lib.rs` |
| `quran-core` | Pure Quran domain: reference grammar, newtypes, enums, `QuranQuotation`, views | `crates/quran-core/src/lib.rs` |
| `storage` | Persistence ports: `Database`, `ReadTx`, `UnitOfWork`, repository traits, `StorageError` | `crates/storage/src/lib.rs` |
| `storage-sqlite` | Sole storage adapter: sqlx/SQLite impl, migration runner, dual pools, backup | `crates/storage-sqlite/src/lib.rs` |
| `provenance` | `ProvenanceRecord`, `ApprovalToken`, `CanonicalWriter` gate for canonical writes | `crates/provenance/src/lib.rs` |
| `audit` | Append-only hash-chained audit log writer/verifier, redaction | `crates/audit/src/lib.rs` |
| `jobs` | Durable DB-backed leased job queue: types, `JobStore`, worker pool | `crates/jobs/src/lib.rs` |
| `sources` | Manifest parser, lifecycle state machine, genealogy registry | `crates/sources/src/lib.rs` |
| `config` | Layered loader (CLI > env > file > defaults), `ValueOrigin`, `Secret<T>` | `crates/config/src/lib.rs` |
| `observability` | tracing subscriber, spans, metric catalog, opt-in OTLP | `crates/observability/src/lib.rs` |
| `quran-corpus` | Adapters, tokenizer, Unicode auditor, QV validator, differ, 13-checkpoint importer | `crates/quran-corpus/src/import.rs` |
| `quran-normalization` | Rules N01–N22, profiles L0–L8, `SpanMap`, pipeline, traces (derived only) | `crates/quran-normalization/src/lib.rs` |
| `quran-search` | `FullTextIndex` port + FTS5 backend, tokenizers, search services, cache | `crates/quran-search/src/lib.rs` |
| `citations` | Citation resolver, `StoredCitation`, deep links/URNs, quotation verification | `crates/citations/src/lib.rs` |
| `tools` / `tool-registry` | `ToolResult`/`ReproducibilityData` contract; `quran.get_ayah`/`quran.get_context` | `crates/tools/src/lib.rs` |
| `application` | Composition root: wires all layers, owns DB handle, Quran services, audit bridge | `crates/application/src/lib.rs` |
| `server` | Axum API v1 + health endpoints, loopback-only serve, `QuranApiBackend` trait | `crates/server/src/lib.rs` |
| `cli` | `qai` binary, clap command tree, dispatch, 45-check doctor | `crates/cli/src/lib.rs` |
| `testkit` | Dev-only fixtures + security/config suites (never ships in a binary) | `crates/testkit/src/lib.rs` |
| `xtask` | Build gates: `arch-check`, `migrate-check`, `ci`, schema/ADR tooling | `xtask/src/main.rs` |

## Pattern Overview

**Overall:** Layered modular monolith with ports-and-adapters (hexagonal) storage and a single composition root.

**Key Characteristics:**

- Pure domain cores (`domain`, `quran-core`, `quran-normalization` logic) have no I/O, no async runtime, no model/vector dependencies — enforced by `cargo xtask arch-check` against `xtask/allowlist.toml`.
- Persistence is trait-defined in `crates/storage/` (`crates/storage/src/repository.rs`, `crates/storage/src/quran.rs`) with exactly one adapter (`crates/storage-sqlite/`). `cli`/`server` must not touch `storage-sqlite` directly; `crates/application/src/db.rs` keeps the backend behind the application boundary.
- Downstream crates depend on traits, not concretes: `server` serves `QuranApiBackend` (`crates/server/src/api.rs`), `tool-registry` serves `QuranBackend` (`crates/tool-registry/src/lib.rs`), `citations` reads through `CitationSource` (`crates/citations/src/lib.rs`). Contract tests run against fakes with no database.
- Canonical Quran text is structural and approval-gated, never a RAG chunk: staging tables → human-gated activation → generation-stamped canonical rows. The importer holds no `ApprovalToken` and literally cannot activate (I5/I7).
- Normalization/search form a derived layer: they read canonical text and write separate derived tables/indexes, never mutate canonical rows (I8, re-verified by MV-018/QV-028 after every build).

## Layers

**Pure domain (`domain`, `quran-core`):**

- Purpose: Shared vocabulary and invariants with zero side effects.
- Location: `crates/domain/src/`, `crates/quran-core/src/`
- Contains: Newtype IDs (`crates/domain/src/ids.rs`), `Diagnostic` taxonomy (`crates/domain/src/diagnostic.rs`), hashing (`crates/domain/src/hashing.rs`), security guards (`crates/domain/src/security*.rs`), reference grammar (`crates/quran-core/src/reference/`), `QuranQuotation` (`crates/quran-core/src/quotation.rs`), views (`crates/quran-core/src/view.rs`).
- Depends on: `serde`, `thiserror`, `time`, `uuid` (+ `unicode-segmentation` for `quran-core`) only.
- Used by: Every other crate.

**Storage port + adapter (`storage`, `storage-sqlite`):**

- Purpose: Define persistence contracts; provide the single SQLite implementation.
- Location: `crates/storage/src/`, `crates/storage-sqlite/src/`
- Contains: `Database`/`ReadTx`/`UnitOfWork` (`crates/storage/src/lib.rs`), Quran rows (`crates/storage/src/quran.rs`), all repository traits (`crates/storage/src/repository.rs`); sqlx pools, shared-transaction `UnitOfWork`, migration runner (`crates/storage-sqlite/src/migrate.rs`), Quran repo impl (`crates/storage-sqlite/src/quran.rs`).
- Depends on: `storage` → `domain` only; `storage-sqlite` → `storage`, `domain`, `quran-core`.
- Used by: `application` exclusively at runtime (edges reach storage only through `application` re-exports or backend traits).

**Foundation services (`provenance`, `audit`, `jobs`, `sources`, `config`, `observability`):**

- Purpose: Cross-cutting Phase-0 platform: trust, durability, configuration, telemetry.
- Location: `crates/provenance/src/lib.rs`, `crates/audit/src/lib.rs`, `crates/jobs/src/`, `crates/sources/src/`, `crates/config/src/`, `crates/observability/src/`
- Contains: `ApprovalToken`/`CanonicalWriter` (`crates/provenance/src/lib.rs`), hash-chain writer/verifier (`crates/audit/src/lib.rs`), `JobStore`/worker pool (`crates/jobs/src/queue.rs`, `crates/jobs/src/worker.rs`), manifest lifecycle (`crates/sources/src/registry.rs`), `Secret<T>` (`crates/config/src/secret.rs`), tracing/metrics/OTLP (`crates/observability/src/telemetry.rs`, `crates/observability/src/metrics.rs`, `crates/observability/src/otlp.rs`).
- Depends on: `domain` + `storage` (ports) + `observability` for `jobs`; see `xtask/allowlist.toml` for the exact edge set.
- Used by: `application` services and the Quran pipeline.

**Quran engine (`quran-core` → `quran-corpus` → derived layer):**

- Purpose: Lossless structured canonical corpus plus reproducible derived indexes.
- Location: `crates/quran-core/src/`, `crates/quran-corpus/src/`, `crates/quran-normalization/src/`, `crates/quran-search/src/`
- Contains: Adapters (`crates/quran-corpus/src/adapters.rs`), Unicode auditor (`crates/quran-corpus/src/unicode.rs`), tokenizer (`crates/quran-corpus/src/tokenize.rs`), validator QV-001…QV-028 (`crates/quran-corpus/src/validation.rs`), differ (`crates/quran-corpus/src/differ.rs`), importer (`crates/quran-corpus/src/import.rs`); normalization rules/pipeline/span (`crates/quran-normalization/src/rules/`, `crates/quran-normalization/src/pipeline.rs`, `crates/quran-normalization/src/span.rs`, `crates/quran-normalization/src/profile.rs`); FTS5 backend/tokenizers/services (`crates/quran-search/src/fts5.rs`, `crates/quran-search/src/tokenizer.rs`, `crates/quran-search/src/index.rs`, `crates/quran-search/src/regex.rs`, `crates/quran-search/src/highlight.rs`).
- Depends on: `quran-core`/`domain`/`sources`/`storage` (`quran-corpus`); `domain` only (`quran-normalization`); `quran-normalization`/`quran-core`/`domain`/`storage` (`quran-search`). No llm/embeddings/retrieval/vector deps anywhere on this path (invariants I2/I8).
- Used by: `application` Quran services, `citations`, `tools`/`tool-registry`, `server`.

**Contract / tool layer (`citations`, `tools`, `tool-registry`):**

- Purpose: Make fabrication unrepresentable: typed quotations, verified citations, reproducible tool results.
- Location: `crates/citations/src/lib.rs`, `crates/tools/src/lib.rs`, `crates/tool-registry/src/lib.rs`
- Contains: `QuotationVerdict` + `resolve`/`verify_quotation` (`crates/citations/src/lib.rs`); `ToolResult` + `ReproducibilityData` (`crates/tools/src/lib.rs`); `quran.get_ayah`/`quran.get_context` + `QuranBackend` trait (`crates/tool-registry/src/lib.rs`).
- Depends on: `quran-core` + `domain` (and `tools` for the registry).
- Used by: `application` (`crates/application/src/quran_tools.rs`) and `server` (`crates/server/src/api.rs` uses `ToolRegistry` for ayah/context endpoints).

**Composition root (`application`):**

- Purpose: The only place that wires concrete backends to domain logic; owns the DB handle.
- Location: `crates/application/src/`
- Contains: Bootstrap `run()` (`crates/application/src/lib.rs`), DB orchestration (`crates/application/src/db.rs`), import handler + activation/rollback (`crates/application/src/quran.rs`), CLI command impls (`crates/application/src/quran_cli.rs`), reader with generation-keyed LRU cache (`crates/application/src/quran_reader.rs`), forms/index/normalize/search services (`crates/application/src/quran_forms.rs`, `crates/application/src/quran_index.rs`, `crates/application/src/quran_normalize.rs`, `crates/application/src/quran_search.rs`, `crates/application/src/quran_search_cache.rs`), tool backend impl (`crates/application/src/quran_tools.rs`), doctor data (`crates/application/src/quran_doctor.rs`), audit bridge (`crates/application/src/audit_bridge.rs`), job queue glue (`crates/application/src/job_queue.rs`).
- Depends on: `domain`, `storage`, `storage-sqlite`, `provenance`, `audit`, `jobs`, `sources`, `config`, `observability`, `quran-core`, `quran-corpus`, `quran-normalization`, `quran-search`, `citations`, `tools`, `tool-registry` (see `xtask/allowlist.toml`).
- Used by: `cli` and `server` (edges only).

**Edges (`cli`, `server`, `xtask`):**

- Purpose: User-facing surfaces and build-time gates; thin by design.
- Location: `crates/cli/src/`, `crates/server/src/`, `xtask/src/`
- Contains: clap tree + `dispatch()` (`crates/cli/src/lib.rs`), `qai quran …` mapping (`crates/cli/src/quran.rs`), doctor registry (`crates/cli/src/doctor.rs`), exit codes (`crates/cli/src/exit_code.rs`), binary entry (`crates/cli/src/main.rs`); Axum routes + envelope (`crates/server/src/api.rs`), loopback guard (`crates/server/src/lib.rs`); arch/migrate/schema/CI gates (`xtask/src/arch.rs`, `xtask/src/migrate.rs`, `xtask/src/ci.rs`, `xtask/src/schema.rs`, `xtask/src/adr.rs`, `xtask/src/coverage.rs`).
- Depends on: `cli` → `application`, `config`, `observability`, `server`; `server` → `application`, `config`, `observability`, `tool-registry`, `tools`, `storage`, `citations`, `quran-core`; `xtask` → nothing (workspace).
- Used by: End users (`qai` binary, HTTP clients) and CI.

## Data Flow

### Primary Request Path

1. CLI parse + dispatch (`crates/cli/src/main.rs` → `crates/cli/src/lib.rs:211` `dispatch()`): global flags (`--config`, `--data-dir`, `--json`, `--yes`) load layered `Config`; subcommand routes to `handle_db`, `run_checks`/`run_quran_checks`, `handle_quran`, or `serve`.
2. Application service executes (`crates/application/src/quran_cli.rs`): opens `SqliteDatabase` by path, runs the reader/importer/activation logic, returns `CommandOutput { exit, human, json }` with exit codes mirroring `crates/cli/src/exit_code.rs` (0/1/2/3/4/5/6/7/70).
3. Storage access via port (`crates/storage/src/lib.rs` `Database` trait → `crates/storage-sqlite/src/lib.rs` `SqliteDatabase`): reads go through `ReadTx`, writes through a shared-transaction `UnitOfWork` so every repo's writes commit atomically.

### Canonical Import → Activation Flow

1. Adapter parse (`crates/quran-corpus/src/adapters.rs` `EditionAdapter`; JSON + CSV adapters reproduce the same manifest): source file validated against `docs/schemas/quran-edition-source.v1.schema.json`.
2. Unicode audit + tokenize + hash (`crates/quran-corpus/src/unicode.rs`, `crates/quran-corpus/src/tokenize.rs`, `crates/quran-corpus/src/hashing.rs`): NFC policy, forbidden code points, whitespace-preserving tokens, frozen `text_hash`/`structure_hash`/`token_order_hash` (ADR-0108).
3. Validation QV-001…QV-028 (`crates/quran-corpus/src/validation.rs`): counts, identifiers, Unicode, token order, checksums, round-trip; 16 adversarial fixtures in `fixtures/quran/adversarial/` each reject with their rule id.
4. Staged import, 13 checkpoints (`crates/quran-corpus/src/import.rs:run_import`, driven as `quran.import` job by `crates/application/src/quran.rs:QuranImportHandler`): writes go to `quran_stg_*` tables only (`migrations/sqlite/0011_quran_staging.up.sql`); ends at `ApprovalRequested` — the importer holds no `ApprovalToken` and cannot activate.
5. Human-gated activation (`crates/application/src/quran.rs:activate_edition` / `rollback_edition`, `--yes` required): same-transaction audit event (`crates/application/src/audit_bridge.rs`), atomic singleton pointer flip in `quran_active_edition` + `corpus_generation` bump, char-level differ report persisted (`crates/quran-corpus/src/differ.rs`).

### Canonical Read Flow (CLI and HTTP share it)

1. Reference parse (`crates/quran-core/src/reference/`): frozen grammar (ADR-0102) parses `1:1`, ranges, divisions; `resolve`/`canonical_form`/`serialize` produce stable addressing.
2. Reader lookup (`crates/application/src/quran_reader.rs:QuranReader`): exact ayah/range/surah/division/token expansion with structure-bounded context caps; generation-keyed `lru` cache keyed by `(edition_id, version, corpus_generation, ref, options)` wholesale-invalidated on activation (ADR-0113).
3. Quotation/citation envelope (`crates/quran-core/src/quotation.rs`, `crates/citations/src/lib.rs`): every `QuranQuotation` carries edition + version + hash (I6); resolver returns `ExactMatch/…/Mismatch/LocationNotFound/EditionNotFound/AccessDenied` verdicts.
4. Surface rendering: CLI via `crates/application/src/quran_cli.rs` (human text vs `--json`); HTTP via `crates/server/src/api.rs` stable `Envelope { api_version, data, meta }` with `ETag = (text_hash, corpus_generation)`, `Cache-Control`, `Content-Language`, `Diagnostic` error bodies.

### Derived Normalization → Search Flow

1. Forms rebuild (`crates/application/src/quran_forms.rs`, `qai quran forms rebuild <slug@version>`): applies normalization profiles L0–L8 (`crates/quran-normalization/src/profile.rs`) and rules N01–N22 (`crates/quran-normalization/src/rules/`) to produce token/ayah forms + skeletons in `migrations/sqlite/0014_quran_forms.up.sql` tables; MV-018 verifier asserts canonical text unchanged before and after (I8).
2. Index build (`crates/application/src/quran_index.rs`, `qai quran index rebuild|verify`): `FullTextIndex` port (`crates/quran-search/src/index.rs`) with FTS5 backend (`crates/quran-search/src/fts5.rs`) and `ar_*` tokenizers (`crates/quran-search/src/tokenizer.rs`); every index records the `corpus_generation` it was built from (`migrations/sqlite/0015_quran_indexes.up.sql`).
3. Query (`crates/application/src/quran_search.rs`, `crates/quran-search/src/model.rs`): exact / normalized / phrase / concatenated + cross-ayah windows, DFA-only bounded regex (`crates/quran-search/src/regex.rs`, I16), filters, totals, highlighting (`crates/quran-search/src/highlight.rs`), generation-keyed result cache (`crates/application/src/quran_search_cache.rs`, `migrations/sqlite/0016_quran_search_cache.up.sql`). Search HTTP/SSE and `qai quran search` CLI are not yet exposed — the service layer exists but has no edge wiring.

**State Management:**

- SQLite is the sole stateful store (WAL, `foreign_keys=ON`, `busy_timeout`, dual pools: write `max_connections=1`, read `query_only`) — see `crates/storage-sqlite/src/lib.rs` and ADR-0001.
- Schema is append-only and checksummed (`migrations/sqlite/*.up.sql` + `migrations/sqlite/checksums.json`, ADR-0002); canonical tables are forward-only (deactivation, not deletion); caches/derived indexes are generation-stamped so stale reads are impossible by construction.
- In-process state is limited to the reader LRU cache (`crates/application/src/quran_reader.rs`), search cache (`crates/application/src/quran_search_cache.rs`), and the job worker pool (`crates/jobs/src/worker.rs`). No global mutable singletons outside these explicitly scoped caches.

## Key Abstractions

**`QuranRef` / reference grammar:**

- Purpose: The stable addressing scheme for every verse, range, division, and token in the system.
- Examples: `crates/quran-core/src/reference/`, `crates/quran-core/src/numbers.rs`, `crates/quran-core/src/enums.rs`
- Pattern: Frozen parser + serializer with proptest round-trip (`parse`/`serialize`/`canonical_form`/`resolve`); golden set `fixtures/quran/golden/references.jsonl` (331 cases).

**`QuranQuotation` / citations:**

- Purpose: The only type allowed to carry quoted canonical text; makes fabrication a type error.
- Examples: `crates/quran-core/src/quotation.rs`, `crates/citations/src/lib.rs`
- Pattern: Constructor-enforced edition + version + hash (I6); resolver verifies text match and returns a closed `QuotationVerdict` enum.

**`ProvenanceRecord` / `ApprovalToken` / `CanonicalWriter`:**

- Purpose: Human-gated trust root: no canonical mutation without a granted approval for the exact subject URN.
- Examples: `crates/provenance/src/lib.rs`, `crates/application/src/quran.rs`, `crates/application/src/audit_bridge.rs`
- Pattern: `CanonicalWriter` trait takes `&ApprovalToken`; importer path never mints one; activation records a hash-chained audit event in the same transaction.

**`Database` / `ReadTx` / `UnitOfWork` + repository traits:**

- Purpose: Persistence port that keeps domain logic backend-agnostic and PostgreSQL-portable.
- Examples: `crates/storage/src/lib.rs`, `crates/storage/src/repository.rs`, `crates/storage/src/quran.rs`, `crates/storage-sqlite/src/lib.rs`
- Pattern: `async-trait` ports; one shared `SqlTx` behind `Arc<tokio::sync::Mutex<_>>` so a `UnitOfWork` commit is atomic across all repos (outbox invariant).

**`JobHandler` / durable queue:**

- Purpose: Cancellable, resumable, idempotent background work without an external broker (ADR-0003).
- Examples: `crates/jobs/src/lib.rs`, `crates/jobs/src/queue.rs`, `crates/jobs/src/worker.rs`, `crates/application/src/job_queue.rs`, `crates/application/src/quran.rs:QuranImportHandler`
- Pattern: `kind` + `payload_schema` + `is_idempotent` + `run(ctx, payload)`; idempotency key `source_version:adapter:parser` (`crates/application/src/quran.rs:import_idempotency_key`).

**`NormalizationPipeline` / `SpanMap` / `NormalizationTrace`:**

- Purpose: Reproducible derived text with exact back-mapping to canonical offsets.
- Examples: `crates/quran-normalization/src/pipeline.rs`, `crates/quran-normalization/src/span.rs`, `crates/quran-normalization/src/trace.rs`, `crates/quran-normalization/src/profile.rs`
- Pattern: Ordered rule application (N01–N22) under a profile (L0–L8); every step emits a bidirectional `SpanMap` (I10, property-tested) and a `NormalizationTrace` recording the exact rule set (I9).

**`FullTextIndex` / `ToolResult`:**

- Purpose: Backend-independent search semantics; reproducible agent-tool contract.
- Examples: `crates/quran-search/src/index.rs`, `crates/quran-search/src/model.rs`, `crates/tools/src/lib.rs`, `crates/tool-registry/src/lib.rs`
- Pattern: `FtsQuery`/`FtsResults`/`IndexManifest` types over the port; `ToolResult` + `ReproducibilityData` checksums on every tool call.

**`Diagnostic` / `StorageError`:**

- Purpose: Unique-code typed errors mapped to public CLI exit codes.
- Examples: `crates/domain/src/diagnostic.rs`, `crates/quran-core/src/error.rs`, `crates/storage/src/error.rs`, `crates/cli/src/exit_code.rs`
- Pattern: `QAI-<NS>-NNNN` codes unique workspace-wide (test-enforced); exit codes 0/1/2/3/4/5/6/7/70 are public API.

## Entry Points

**`qai` binary:**

- Location: `crates/cli/src/main.rs`
- Triggers: Any `./target/debug/qai …` invocation; `dispatch()` in `crates/cli/src/lib.rs:211`.
- Responsibilities: Parse clap tree (`Status|Version|Doctor|Config|Db|Secret|Source|Job|Audit|Serve|Completions|Quran`), load layered config, route to `application` services, map results to process exit codes.

**`application::run` bootstrap:**

- Location: `crates/application/src/lib.rs:run`
- Triggers: `qai serve` and any command needing a live backend.
- Responsibilities: Validate config, init observability with secret redaction, open `SqliteDatabase`, health-check storage, report `RunResult { health }`.

**HTTP serve:**

- Location: `crates/server/src/lib.rs:loopback_addr`, `crates/server/src/api.rs` (routes), dispatched from `crates/cli/src/lib.rs:301` `Commands::Serve`.
- Triggers: `./target/debug/qai serve --bind 127.0.0.1:8737`.
- Responsibilities: Bind axum `TcpListener` (non-loopback refused with exit 4); serve `/healthz`, `/readyz`, `/api/v1/meta`, Quran read endpoints per `docs/08-api/quran-v1-openapi.json`, normalization `preview`/`profiles`, `/debug/read/{edition}/{surah}`.

**`quran.import` job handler:**

- Location: `crates/application/src/quran.rs:QuranImportHandler`
- Triggers: `qai quran import <manifest>` (runs inline) or a queued `quran.import` job.
- Responsibilities: Drive the 13-checkpoint pipeline to `Staged`; never activate.

**Doctor checks:**

- Location: `crates/cli/src/doctor.rs:run_checks` / `run_quran_checks`, data from `crates/application/src/quran_doctor.rs` and `crates/application/src/db.rs:DbProbe`.
- Triggers: `./target/debug/qai doctor --quran --deep [--json]`.
- Responsibilities: Read-only diagnostics (`Pass|Warn|Fail|Skipped` + remedy + `next_command`); never mutates data.

## Architectural Constraints

- **Threading:** tokio `rt-multi-thread` (`Cargo.toml` workspace deps); SQLite write pool is a single serialized connection, read pool is `query_only`; the reader notes a dedicated read-pool path as future work — concurrent readers currently serialize behind the single-writer pool (`crates/application/src/quran_reader.rs` docs).
- **Global state:** No module-level mutable singletons. Scoped shared state only: reader `LruCache` behind `Mutex` (`crates/application/src/quran_reader.rs`), search cache (`crates/application/src/quran_search_cache.rs`), shared write `SqlTx` behind `Arc<tokio::sync::Mutex<_>>` (`crates/storage-sqlite/src/lib.rs`), job worker pool (`crates/jobs/src/worker.rs`).
- **Circular imports:** None. Dependency direction is machine-enforced: `cargo xtask arch-check` vs `xtask/allowlist.toml`; a crate absent from the allowlist may have zero workspace deps (fail-closed for new/placeholder crates).
- **Purity fence:** `domain` → `(serde, thiserror, time, uuid)` only, no I/O/async (`xtask/allowlist.toml:[domain]`); `quran-core` adds only `unicode-*`; `quran-corpus`/`quran-normalization`/`quran-search` never gain llm/embeddings/retrieval/vector deps (I2/I8, AC-P2-36).
- **Canonical-write fence:** Canonical rows writable only through `CanonicalWriter` + `ApprovalToken` (`crates/provenance/src/lib.rs`); staging tables (`quran_stg_*`) have no immutability triggers and cascade-delete per run; canonical tables are insert-only/forward-only with triggers.
- **Placeholder fence:** ~30 placeholder crates (`graph`, `quran-graph`, `hadith-*`, `tafsir`, `scripture`, `ingestion`, `retrieval`, `rag`, `embeddings`, `reranking`, `llm`, `model-router`, `agent-*`, `agency`, `conversations`, `tool-sdk`, `tool-sandbox`, `policy`, `approvals`, `workflows`, `memory`, `mcp`, `evaluation`, `api`, `tui`, `quran-morphology`, `isnad-graph`) contain only a `//! Phase N` doc and must stay empty until their phase starts.
- **Network fence:** Local-first, deny-by-default; `serve` binds loopback only (`crates/server/src/lib.rs:loopback_addr`); telemetry/OTLP is opt-in (`crates/observability/src/otlp.rs`); internet content is untrusted until validated; `domain` security guards (`crates/domain/src/security*.rs`) cover path/archive/net/input/sanitize.

## Anti-Patterns

### Reaching past the composition root

**What happens:** An edge crate (`server`, `cli`) imports `storage-sqlite` or concrete reader internals directly instead of going through `application` services/re-exports or a backend trait.
**Why it's wrong:** It hard-codes the backend choice in two places, defeats the `storage` port (PostgreSQL portability), and makes contract tests need a database. The `server` allowlist entry documents this exact smell (`xtask/allowlist.toml:[server]` — `storage` + `tools` allowed only for `ReaderBackend`/`ToolError`, with a note to prefer `application` re-exports).
**Do this instead:** Add the operation to `crates/application/src/` (e.g. `crates/application/src/quran_reader.rs`, `crates/application/src/quran_cli.rs`, `crates/application/src/db.rs`) and call it from the edge; depend on traits (`crates/server/src/api.rs:QuranApiBackend`, `crates/tool-registry/src/lib.rs:QuranBackend`, `crates/citations/src/lib.rs:CitationSource`) with fakes in tests.

### Mutating canonical state without approval

**What happens:** New code writes `quran_editions`/surahs/ayahs/tokens directly (or extends the importer to flip the active pointer) instead of staging + human-gated activation.
**Why it's wrong:** It violates I1/I5/I7, bypasses the audit trail, and breaks the byte-equality corpus gate everything downstream inherits (search, citations, tafsir links).
**Do this instead:** Follow `crates/quran-corpus/src/import.rs:run_import` (stage only, ends at `ApprovalRequested`) plus `crates/application/src/quran.rs:activate_edition` (requires `ApprovalToken` for the exact `quran-edition:{slug}@{version}` URN, same-transaction audit via `crates/application/src/audit_bridge.rs`).

## Error Handling

**Strategy:** Typed `Diagnostic` errors with unique `QAI-<NS>-NNNN` codes (test-enforced uniqueness), mapped to stable public CLI exit codes and HTTP `Diagnostic` bodies.

**Patterns:**

- Domain/phase `Diagnostic` types per crate (`crates/domain/src/diagnostic.rs`, `crates/quran-core/src/error.rs`, `crates/quran-corpus/src/error.rs`, `crates/quran-normalization/src/error.rs`, `crates/quran-search/src/error.rs`, `crates/storage/src/error.rs`) with `code()` → exit-code mapping in `crates/cli/src/exit_code.rs` and `crates/application/src/quran_cli.rs:exit`.
- `thiserror` for structured errors, `anyhow` at the `xtask` boundary; `Result<T, StorageError>` on the storage port; `ReaderError` preserves reference-grammar `01xx` codes (`crates/application/src/quran_reader.rs`).
- No-fabrication as types: unknown verse/translation/citation surfaces as `EditionNotFound`/`AyahNotFound`/`TranslationNotFound`/`Mismatch`/`LocationNotFound`, never a synthesized fallback.

## Cross-Cutting Concerns

**Logging:** `tracing` + `tracing-subscriber` with secret redaction, structured spans, and a metrics catalog (`crates/observability/src/lib.rs`, `crates/observability/src/telemetry.rs`, `crates/observability/src/metrics.rs`); OTLP export is opt-in only (`crates/observability/src/otlp.rs`); secrets render as `***` via `Secret<T>` (`crates/config/src/secret.rs`) and `crates/domain/src/redaction.rs`.
**Validation:** Layered — manifest schema (`docs/schemas/quran-edition-source.v1.schema.json`), Unicode auditor (`crates/quran-corpus/src/unicode.rs`), QV-001…QV-028 corpus validator (`crates/quran-corpus/src/validation.rs`), config validation at load and on change (`crates/config/src/lib.rs`), payload schemas on job handlers (`crates/application/src/quran.rs:payload_schema`), DFA-only regex bounds (`crates/quran-search/src/regex.rs`).
**Authentication:** No auth provider (local-first single-operator model). Identity primitives exist for provenance (`principals` table in `migrations/sqlite/0001_core.up.sql`, `Actor` in `crates/audit/src/lib.rs`, `PrincipalId` in `crates/domain/src/ids.rs`); `serve` enforces loopback-only binding as its access control (`crates/server/src/lib.rs`).

---

*Architecture analysis: 2026-09-22*
