---
last_mapped_commit: 6891d7bc64bc469a10e9a28faf81023fa11cb9ae
last_mapped_at: 2026-11-01
---
<!-- refreshed: 2026-11-01 -->

# Architecture

**Analysis Date:** 2026-11-01

## System Overview

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                         Edge / Interface Layer                           │
├──────────────┬───────────────┬──────────────────┬───────────────────────┤
│  CLI (`qai`) │  HTTP Server  │   Web GUI (Vite) │  TUI cockpit          │
│ `crates/cli/`│ `crates/server/` │ `web/src/`    │  `crates/tui/`        │
│ +9 subcmds   │ Axum API v1   │ `router.tsx` +   │  ratatui over         │
│              │ + webassets   │ 15 views         │  application services │
└──────┬───────┴───────┬───────┴────────┬─────────┴───────────┬───────────┘
       │               │                │                     │
       ▼               ▼                ▼                     ▼
┌─────────────────────────────────────────────────────────────────────────┐
│              Composition Root: `application`                             │
│         `crates/application/src/`                                        │
│  `quran_*.rs` (reader/tools/forms/index/normalize/search/counting/      │
│   lexicon/morphology/graph_*) `hadith_*` `isnad_*` `tafsir_*`            │
│  `scripture_*` `rag_*` `verse_link` `cross_domain` `knowledge_domain`    │
│  `disagreement` `quran_doctor*` `db` `audit_bridge` `job_queue`          │
└─────────────────────────────────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  Domain Engines (pure + pipeline, one per corpus)                       │
│  Quran: `quran-core` → `quran-corpus` → `quran-normalization`            │
│         `quran-morphology` → `quran-search` → `quran-graph`              │
│  Hadith: `hadith-core` + `hadith-ingestion` → `isnad-graph`             │
│  Tafsir: `tafsir` · Scripture: `scripture` · Links: `verse_link`         │
│  Contract: `citations` · `tools` / `tool-registry` (30+ read-only tools)│
└─────────────────────────────────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  Multi-RAG Platform (local, deterministic, no network)                  │
│  `rag` (projects/sources/docs) → `retrieval` port (`memvid` backend)    │
│  `embeddings` (`local-hash`) · `reranking` (`score`)                     │
└─────────────────────────────────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────────────────────────┐
│        Foundation Services (Phase 0 provenance core)                     │
│  `domain` ← `storage` (traits only)                                      │
│  `provenance` `audit` `jobs` `sources` `config` `observability`          │
│  `evaluation` (mechanical regression harness, P2-T108)                   │
└─────────────────────────────────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  Storage Adapter + SQLite                                               │
│  `crates/storage-sqlite/` ──→ `migrations/sqlite/` (0001–0044)          │
│  (dual pools, WAL, `VACUUM INTO` backup, canonical-write fence 0021)    │
└─────────────────────────────────────────────────────────────────────────┘
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
| `jobs` | Durable DB-backed leased job queue: types, `JobStore`, worker pool, registry | `crates/jobs/src/queue.rs` |
| `sources` | Manifest parser, lifecycle state machine, genealogy registry, upstream tracking | `crates/sources/src/lib.rs` |
| `config` | Layered loader (CLI > env > file > defaults), `ValueOrigin`, `Secret<T>` | `crates/config/src/lib.rs` |
| `observability` | tracing subscriber, spans, metric catalog, opt-in OTLP | `crates/observability/src/lib.rs` |
| `quran-corpus` | Adapters, tokenizer, Unicode auditor, QV validator, differ, 13-checkpoint importer | `crates/quran-corpus/src/import.rs` |
| `quran-normalization` | Rules N01–N22, profiles L0–L8, `SpanMap`, pipeline, traces (derived only) | `crates/quran-normalization/src/lib.rs` |
| `quran-morphology` | Pure lexicon types: adapters (CSV/JSON), alignment, MV validation, families, tagset | `crates/quran-morphology/src/lib.rs` |
| `quran-search` | `FullTextIndex` port + FTS5 backend, tokenizers, search services, cache | `crates/quran-search/src/lib.rs` |
| `quran-graph` | Knowledge-graph **port**: model, `GraphStore` trait, budgets, patterns, structural builder, `MemGraphStore` reference backend, Graph JSON export | `crates/quran-graph/src/lib.rs` |
| `hadith-core` | Hadith domain types: collection, record (matn/isnad split), per-scholar gradings (never merged) | `crates/hadith-core/src/lib.rs` |
| `hadith-ingestion` | Hadith parser + validator (source numbering preserved verbatim) | `crates/hadith-ingestion/src/lib.rs` |
| `isnad-graph` | Narrator/isnad graph: `NarratorId`, narrator records, rijal sources; re-exports shared `quran-graph` vocabulary | `crates/isnad-graph/src/lib.rs` |
| `tafsir` | Tafsir domain types: works (slug/author/dimensions) + records linked via `verse_links` | `crates/tafsir/src/lib.rs` |
| `scripture` | Comparative-scripture types: editions, books, verses, references, translations, views | `crates/scripture/src/lib.rs` |
| `rag` | Multi-RAG core types: `RagProject`/`RagSource`/`RagDocument`, config, hybrid merge, tombstones | `crates/rag/src/lib.rs` |
| `retrieval` | Backend-agnostic `RetrievalBackend` port (`ingest`/`search`/`commit`); `memvid` `.mv2` backend behind feature flag | `crates/retrieval/src/lib.rs` |
| `embeddings` | `EmbeddingProvider` port; deterministic `LocalHashEmbedding` default (no network) | `crates/embeddings/src/lib.rs` |
| `reranking` | `Reranker` port; `ScoreReranker` (score-desc, stable id tie-break) | `crates/reranking/src/lib.rs` |
| `evaluation` | Mechanical regression harness: versioned datasets, metrics, reports (`Unavailable`, never `0`, when data absent) | `crates/evaluation/src/lib.rs` |
| `citations` | Citation resolver, `StoredCitation`, deep links/URNs, quotation verification | `crates/citations/src/lib.rs` |
| `tools` / `tool-registry` | `ToolResult`/`ReproducibilityData` contract; 30+ read-only tools (`quran.*`, `hadith.*`, `tafsir.*`, `scripture.*`, `rag.*`) over backend traits | `crates/tool-registry/src/lib.rs` |
| `application` | Composition root: wires all layers, owns DB handle, Quran/Hadith/Isnad/Tafsir/Scripture/RAG services, graph SQLite adapters, audit bridge | `crates/application/src/lib.rs` |
| `server` | Axum API v1 + health/debug endpoints, loopback-only serve, `QuranApiBackend` trait, scripture sub-router, `webassets` static hosting | `crates/server/src/api.rs` |
| `cli` | `qai` binary, clap command tree (`quran`, `hadith`, `isnad`, `tafsir`, `scripture`, `rag`, `links`), 45-check doctor | `crates/cli/src/lib.rs` |
| `tui` | Ratatui operations cockpit over shared `application` services (never shells out, never speaks HTTP); terminal-text sanitizer | `crates/tui/src/lib.rs` |
| `web` | Vite + React 19 research workspace: hash-less path router, 15 views, typed `/api/v1` client mirroring the server `Envelope<T>` | `web/src/router.tsx` |
| `testkit` | Dev-only fixtures + security/config suites (never ships in a binary) | `crates/testkit/src/lib.rs` |
| `xtask` | Build gates: `arch-check`, `migrate-check`, `ci`, schema/ADR tooling | `xtask/src/main.rs` |
| Phase-N placeholders | `graph`, `ingestion`, `llm`, `model-router`, `agent-domain`, `agent-runtime`, `agency`, `conversations`, `tool-sdk`, `tool-sandbox`, `policy`, `approvals`, `workflows`, `memory`, `mcp`, `api` — still `//! Phase N` empty crates | `crates/<name>/src/lib.rs` |

## Pattern Overview

**Overall:** Layered modular monolith with ports-and-adapters (hexagonal) storage, per-corpus domain engines, and a single composition root. The web GUI and TUI are additional thin edges beside CLI/HTTP.

**Key Characteristics:**

- Pure domain cores (`domain`, `quran-core`, `quran-normalization` logic, `quran-morphology` types, `hadith-core`, `tafsir`, `scripture` types) have no I/O, no async runtime, no model/vector/network dependencies — enforced by `cargo xtask arch-check` against `xtask/allowlist.toml`. `quran-morphology` must never gain `llm`/`embeddings`/`retrieval`/vector deps (AC-P2-36).
- Persistence is trait-defined in `crates/storage/` (`crates/storage/src/repository.rs`, `crates/storage/src/quran.rs`) with exactly one adapter (`crates/storage-sqlite/`). `cli`/`server` must not touch `storage-sqlite` directly; `crates/application/src/db.rs` keeps the backend behind the application boundary.
- Downstream crates depend on traits, not concretes: `server` serves `QuranApiBackend` (`crates/server/src/api.rs`), `tool-registry` serves `QuranBackend` + per-domain backends (`crates/tool-registry/src/lib.rs`), `citations` reads through `CitationSource` (`crates/citations/src/lib.rs`), `quran-graph` exposes only typed `GraphStore` operations (no backend query language crosses the boundary — pinned by test). Contract tests run against fakes with no database.
- Canonical Quran text is structural and approval-gated, never a RAG chunk: staging tables → human-gated activation → generation-stamped canonical rows. The importer holds no `ApprovalToken` and literally cannot activate (I5/I7). Migration `0021_canonical_write_fence.up.sql` adds a DB-level fence; canonical tables are insert-only/forward-only with triggers.
- Normalization/search/morphology/graph form a derived layer: they read canonical text and write separate derived tables/indexes/projections, never mutate canonical rows (I8, re-verified by MV-018/QV-028 after every build).
- Multi-corpus symmetry: hadith, tafsir, and scripture each follow the Quran corpus pattern — source-aware records, verbatim source numbering, staged import, per-domain search forms (`0026_hadith_search_forms`, `0029_tafsir_search_forms`, `0043_scripture_search_forms`), per-domain reader/search/tools triplets in `application` (`hadith_reader.rs`/`hadith_search.rs`/`hadith_tools.rs`, etc.).
- No-fabrication is load-bearing across every surface: unknown verse/translation/citation surfaces as `EditionNotFound`/`AyahNotFound`/`Mismatch`/`LocationNotFound`, never a synthesized fallback. Computational verse links are always labelled `"uncertain"` until an owner confirms (`crates/application/src/verse_link.rs`, D-07). The evaluation harness reports `Unavailable` with a remedy, never `0` (`crates/evaluation/src/lib.rs`).
- RAG is local-first and deterministic: `local-hash` embeddings, `score` reranking, `memvid` file backend behind a feature flag — no network in the retrieval path.

## Layers

**Pure domain (`domain`, `quran-core`, `hadith-core`, `tafsir`, `scripture`, `quran-morphology` types, `isnad-graph` types):**

- Purpose: Shared vocabulary and invariants with zero side effects.
- Location: `crates/domain/src/`, `crates/quran-core/src/`, `crates/hadith-core/src/`, `crates/tafsir/src/`, `crates/scripture/src/`, `crates/quran-morphology/src/`, `crates/isnad-graph/src/lib.rs`
- Contains: Newtype IDs (`crates/domain/src/ids.rs`), `Diagnostic` taxonomy (`crates/domain/src/diagnostic.rs`), hashing (`crates/domain/src/hashing.rs`), security guards (`crates/domain/src/security*.rs`), reference grammar (`crates/quran-core/src/reference/`), `QuranQuotation` (`crates/quran-core/src/quotation.rs`), hadith collections/records/gradings (`crates/hadith-core/src/collection.rs`, `record.rs`, `grading.rs`), tafsir works/records (`crates/tafsir/src/work.rs`, `record.rs`), scripture books/editions/verses/references (`crates/scripture/src/`), morphology dataset/family/tagset (`crates/quran-morphology/src/dataset.rs`, `family.rs`, `tagset.rs`).
- Depends on: `serde`, `thiserror`, `time`, `uuid` (+ `unicode-segmentation` for `quran-core`) only.
- Used by: Every other crate.

**Storage port + adapter (`storage`, `storage-sqlite`):**

- Purpose: Define persistence contracts; provide the single SQLite implementation.
- Location: `crates/storage/src/`, `crates/storage-sqlite/src/`
- Contains: `Database`/`ReadTx`/`UnitOfWork` (`crates/storage/src/lib.rs`), Quran rows (`crates/storage/src/quran.rs`), all repository traits (`crates/storage/src/repository.rs`); sqlx pools, shared-transaction `UnitOfWork`, migration runner (`crates/storage-sqlite/src/migrate.rs`), Quran repo impl (`crates/storage-sqlite/src/quran.rs`).
- Depends on: `storage` → `domain` only; `storage-sqlite` → `storage`, `domain`, `quran-core`.
- Used by: `application` exclusively at runtime (edges reach storage only through `application` re-exports or backend traits).

**Foundation services (`provenance`, `audit`, `jobs`, `sources`, `config`, `observability`, `evaluation`):**

- Purpose: Cross-cutting platform: trust, durability, configuration, telemetry, mechanical regression gating.
- Location: `crates/provenance/src/lib.rs`, `crates/audit/src/lib.rs`, `crates/jobs/src/`, `crates/sources/src/`, `crates/config/src/`, `crates/observability/src/`, `crates/evaluation/src/`
- Contains: `ApprovalToken`/`CanonicalWriter` (`crates/provenance/src/lib.rs`), hash-chain writer/verifier (`crates/audit/src/lib.rs`), `JobStore`/worker pool/registry (`crates/jobs/src/queue.rs`, `crates/jobs/src/worker.rs`, `crates/jobs/src/registry.rs`), manifest lifecycle + upstream tracking (`crates/sources/src/registry.rs`, `crates/sources/src/upstream.rs`), `Secret<T>` (`crates/config/src/secret.rs`), tracing/metrics/OTLP (`crates/observability/src/telemetry.rs`, `crates/observability/src/metrics.rs`, `crates/observability/src/otlp.rs`), versioned datasets + metric computation + report rendering (`crates/evaluation/src/dataset.rs`, `metrics.rs`, `report.rs`).
- Depends on: `domain` + `storage` (ports) + `observability` for `jobs`; see `xtask/allowlist.toml` for the exact edge set.
- Used by: `application` services and the corpus pipelines.

**Quran engine (`quran-core` → `quran-corpus` → derived layer):**

- Purpose: Lossless structured canonical corpus plus reproducible derived indexes, lexicon, and graph projections.
- Location: `crates/quran-core/src/`, `crates/quran-corpus/src/`, `crates/quran-normalization/src/`, `crates/quran-morphology/src/`, `crates/quran-search/src/`, `crates/quran-graph/src/`
- Contains: Adapters (`crates/quran-corpus/src/adapters.rs`), Unicode auditor (`crates/quran-corpus/src/unicode.rs`), tokenizer (`crates/quran-corpus/src/tokenize.rs`), QV-001…QV-028 validator (`crates/quran-corpus/src/validation.rs`), differ (`crates/quran-corpus/src/differ.rs`), importer (`crates/quran-corpus/src/import.rs`); normalization rules/pipeline/span (`crates/quran-normalization/src/rules/`, `crates/quran-normalization/src/pipeline.rs`, `crates/quran-normalization/src/span.rs`, `crates/quran-normalization/src/profile.rs`); morphology adapters/alignment/validation/family (`crates/quran-morphology/src/adapter_csv.rs`, `adapter_json.rs`, `align.rs`, `validate.rs`, `family.rs`); FTS5 backend/tokenizers/services (`crates/quran-search/src/fts5.rs`, `crates/quran-search/src/tokenizer.rs`, `crates/quran-search/src/index.rs`, `crates/quran-search/src/regex.rs`, `crates/quran-search/src/highlight.rs`); graph port + reference backend + traversal + patterns + structural builder + export (`crates/quran-graph/src/store.rs`, `mem.rs`, `traverse.rs`, `pattern.rs`, `structural.rs`, `export.rs`, `model.rs`).
- Depends on: `quran-core`/`domain`/`sources`/`storage` (`quran-corpus`); `domain` only (`quran-normalization`, `quran-morphology`); `quran-normalization`/`quran-core`/`domain`/`storage` (`quran-search`); `domain` + port-only (`quran-graph`). No llm/embeddings/retrieval/vector deps anywhere on this path (invariants I2/I8).
- Used by: `application` Quran services, `citations`, `tools`/`tool-registry`, `server`, `web` (via HTTP).

**Hadith / isnad / tafsir / scripture engines:**

- Purpose: Replicate the Quran engine's source-aware, approval-shaped discipline for the other three corpora plus the narrator graph.
- Location: `crates/hadith-core/src/`, `crates/hadith-ingestion/src/`, `crates/isnad-graph/src/lib.rs`, `crates/tafsir/src/`, `crates/scripture/src/`
- Contains: Collection/Tradition ids (`crates/hadith-core/src/collection.rs`), matn/isnad records with verbatim source numbering (`crates/hadith-core/src/record.rs`), per-scholar gradings with review decisions (`crates/hadith-core/src/grading.rs`), hadith parser/validator (`crates/hadith-ingestion/src/parser.rs`, `validate.rs`); `NarratorId`/`NarratorRecord`/rijal layer over the shared graph vocabulary (`crates/isnad-graph/src/lib.rs`); tafsir works/records (`crates/tafsir/src/work.rs`, `record.rs`); scripture books/editions/verses/references/translations/views (`crates/scripture/src/book.rs`, `edition.rs`, `verse.rs`, `reference.rs`, `translation.rs`, `view.rs`).
- Depends on: `domain` (+ `quran-graph` vocabulary for `isnad-graph`) only.
- Used by: `application` per-domain services (`hadith_*`, `isnad_*`, `tafsir_*`, `scripture_*`), `tool-registry` domain tools, `server` domain routes, `web` domain views.

**Knowledge graph (`quran-graph` port + `application` SQLite adapters):**

- Purpose: Typed, budgeted, authorization-scoped graph queries over frozen structural projections — never free-form graph queries.
- Location: Port in `crates/quran-graph/src/`; production adapters in `crates/application/src/quran_graph_store.rs` (pinned-projection snapshot read-view, byte-identical semantics to the reference backend by construction), `quran_graph_build.rs` (durable staging), `quran_graph_api.rs`, `quran_graph_annotations.rs` (assertion authority with review gate), `quran_graph_export.rs`, `quran_graph_tools.rs`, `quran_graph_doctor.rs`; isnad side in `crates/application/src/isnad_graph_store.rs`, `isnad_reader.rs`, `isnad_tools.rs`.
- Contains: `GraphStore` port (node resolution, neighbors, bounded paths/subgraphs/patterns, build staging, capability discovery — `crates/quran-graph/src/store.rs`), `QueryBudgets` pre-flight checks, `AuthzScope` + tombstone filtering, typed allowlisted `Pattern` queries (`crates/quran-graph/src/pattern.rs`), pure structural projection builder with stable IDs (`crates/quran-graph/src/structural.rs`, `STRUCTURAL_PROJECTION_ID`), Graph JSON export (`crates/quran-graph/src/export.rs`).
- Depends on: `domain` only at the port; adapters additionally on `storage`/`storage-sqlite`.
- Used by: `application` graph services, graph tools (`quran.graph_neighbors/path/subgraph/pattern/root_family`), `server` `/api/v1/quran/graph/*` routes, `web/src/views/GraphView.tsx`.

**Multi-RAG (`rag`, `retrieval`, `embeddings`, `reranking` + `application` glue):**

- Purpose: Project-scoped retrieval over user books with tombstone-aware consistency — local and reproducible, no remote models.
- Location: `crates/rag/src/` (`config.rs`, `project.rs`, `hybrid.rs`, `tombstone.rs`), `crates/retrieval/src/` (`document.rs`, `hit.rs`, `memvid_backend.rs`), `crates/embeddings/src/lib.rs`, `crates/reranking/src/lib.rs`; glue in `crates/application/src/rag_reader.rs`, `rag_search.rs`, `rag_tools.rs`.
- Contains: `RagProject`/`RagSource`/`SourceStatus` (active/deactivated/tombstoned) descriptors, hybrid merge, tombstone filtering; `RetrievalBackend` port with `memvid` `.mv2` file backend (feature-gated); `LocalHashEmbedding` (deterministic SHA-256 expansion, default dim 128); `ScoreReranker`; tools `rag.create_project`/`rag.import`/`rag.query`/`rag.debug`/`rag.source_*` (see `crates/tool-registry/src/lib.rs:2315`).
- Depends on: `domain` (+ `sha2`); no network crates.
- Used by: `application` RAG services, `cross_domain` Books leg, `server` tool-dispatch route, `web/src/views/RagDebugView.tsx` (route-debug surface).

**Contract / tool layer (`citations`, `tools`, `tool-registry`):**

- Purpose: Make fabrication unrepresentable: typed quotations, verified citations, reproducible tool results — now spanning all five knowledge domains.
- Location: `crates/citations/src/lib.rs`, `crates/tools/src/lib.rs`, `crates/tool-registry/src/lib.rs`
- Contains: `QuotationVerdict` + `resolve`/`verify_quotation` (`crates/citations/src/lib.rs`); `ToolResult` + `ReproducibilityData` + `research_checksum` (`crates/tools/src/lib.rs`); `QuranBackend` + per-domain backends and ~30 tool impls: `quran.get_ayah`/`get_context`/`search`/`root`/`lemma`/`morphology`/`family`/`graph_*` (5), `hadith.get`/`search`/`gradings`, `tafsir.get`/`search`, `scripture.get`/`search`/`translations`/`editions`/`parallel_passages`, `rag.create_project`/`import`/`query`/`debug`/`source_*` (`crates/tool-registry/src/lib.rs:995`, `:2315`).
- Depends on: `quran-core` + `domain` (and `tools` for the registry).
- Used by: `application` (`crates/application/src/quran_tools.rs`, `hadith_tools.rs`, `tafsir_tools.rs`, `scripture_tools.rs`, `rag_tools.rs`, `isnad_tools.rs`, `quran_graph_tools.rs`) and `server` (`crates/server/src/api.rs` tool-dispatch endpoint + `QuranApiBackend`).

**Composition root (`application`):**

- Purpose: The only place that wires concrete backends to domain logic; owns the DB handle.
- Location: `crates/application/src/`
- Contains: Bootstrap `run()` (`crates/application/src/lib.rs`), DB orchestration (`crates/application/src/db.rs`), import handler + activation/rollback (`crates/application/src/quran.rs`), CLI command impls (`crates/application/src/quran_cli.rs`), reader with generation-keyed LRU cache (`crates/application/src/quran_reader.rs`), forms/index/normalize/search/counting/lexicon/morphology services (`crates/application/src/quran_forms.rs`, `quran_index.rs`, `quran_normalize.rs`, `quran_search.rs`, `quran_search_cache.rs`, `quran_search_api.rs`, `quran_counting.rs`, `quran_lexicon_api.rs`, `quran_morphology.rs`), graph services (`quran_graph_*.rs`), per-domain triplets (`hadith_reader/search/tools`, `isnad_reader/tools/graph_store`, `tafsir_reader/search/tools`, `scripture_reader/search/tools`, `rag_reader/search/tools`), cross-corpus glue (`verse_link.rs`, `knowledge_domain.rs`, `cross_domain.rs`, `disagreement.rs`), tool backend impls (`quran_tools.rs` et al.), doctor data (`quran_doctor.rs`, `quran_doctor_indexes.rs`), audit bridge, job queue glue.
- Depends on: `domain`, `storage`, `storage-sqlite`, `provenance`, `audit`, `jobs`, `sources`, `config`, `observability`, all corpus/engine crates (see `xtask/allowlist.toml`).
- Used by: `cli`, `server`, `tui` (edges only).

**Edges (`cli`, `server`, `web`, `tui`, `xtask`):**

- Purpose: User-facing surfaces and build-time gates; thin by design.
- Location: `crates/cli/src/`, `crates/server/src/`, `web/src/`, `crates/tui/src/`, `xtask/src/`
- Contains: clap tree + `dispatch()` (`crates/cli/src/lib.rs`), per-domain subcommands (`crates/cli/src/quran.rs`, `hadith.rs`, `isnad.rs`, `tafsir.rs`, `scripture.rs`, `rag.rs`, `links.rs`), doctor registry (`crates/cli/src/doctor.rs`), exit codes (`crates/cli/src/exit_code.rs`), binary entry (`crates/cli/src/main.rs`); Axum routes + envelope (`crates/server/src/api.rs`), scripture sub-router (`crates/server/src/routes/scripture.rs`), static web hosting (`crates/server/src/webassets.rs`), loopback guard (`crates/server/src/lib.rs`); path router + 15 views + typed API client (`web/src/router.tsx`, `web/src/views/`, `web/src/api/client.ts`); ratatui app/palette/screens (`crates/tui/src/app.rs`, `palette.rs`, `screens/`); arch/migrate/schema/CI gates (`xtask/src/`).
- Depends on: `cli` → `application`, `config`, `observability`, `server`; `server` → `application`, `config`, `observability`, `tool-registry`, `tools`, `storage`, `citations`, `quran-core`; `tui` → `application` services only; `web` → HTTP API only (no Rust dependency); `xtask` → nothing (workspace).
- Used by: End users (`qai` binary, browser, terminal UI) and CI.

## Data Flow

### Primary Request Path

1. CLI parse + dispatch (`crates/cli/src/main.rs` → `crates/cli/src/lib.rs` `dispatch()`): global flags (`--config`, `--data-dir`, `--json`, `--yes`) load layered `Config`; subcommand routes to `handle_db`, doctor runners, `handle_quran`, or domain handlers (`hadith.rs`, `isnad.rs`, `tafsir.rs`, `scripture.rs`, `rag.rs`, `links.rs`), or `serve`.
2. Application service executes (`crates/application/src/quran_cli.rs` + per-domain readers): opens `SqliteDatabase` by path, runs the reader/importer/activation logic, returns `CommandOutput { exit, human, json }` with exit codes mirroring `crates/cli/src/exit_code.rs` (0/1/2/3/4/5/6/7/70).
3. Storage access via port (`crates/storage/src/lib.rs` `Database` trait → `crates/storage-sqlite/src/lib.rs` `SqliteDatabase`): reads go through `ReadTx`, writes through a shared-transaction `UnitOfWork` so every repo's writes commit atomically.

### Canonical Import → Activation Flow

1. Adapter parse (`crates/quran-corpus/src/adapters.rs` `EditionAdapter`; JSON + CSV adapters reproduce the same manifest): source file validated against `docs/schemas/quran-edition-source.v1.schema.json`.
2. Unicode audit + tokenize + hash (`crates/quran-corpus/src/unicode.rs`, `crates/quran-corpus/src/tokenize.rs`, `crates/quran-corpus/src/hashing.rs`): NFC policy, forbidden code points, whitespace-preserving tokens, frozen `text_hash`/`structure_hash`/`token_order_hash` (ADR-0108).
3. Validation QV-001…QV-028 (`crates/quran-corpus/src/validation.rs`): counts, identifiers, Unicode, token order, checksums, round-trip; 16 adversarial fixtures in `fixtures/quran/adversarial/` each reject with their rule id.
4. Staged import, 13 checkpoints (`crates/quran-corpus/src/import.rs:run_import`, driven as `quran.import` job by `crates/application/src/quran.rs:QuranImportHandler`): writes go to `quran_stg_*` tables only (`migrations/sqlite/0011_quran_staging.up.sql`); ends at `ApprovalRequested` — the importer holds no `ApprovalToken` and cannot activate.
5. Human-gated activation (`crates/application/src/quran.rs:activate_edition` / `rollback_edition`, `--yes` required): same-transaction audit event (`crates/application/src/audit_bridge.rs`), atomic singleton pointer flip in `quran_active_edition` + `corpus_generation` bump, char-level differ report persisted (`crates/quran-corpus/src/differ.rs`).

### Canonical Read Flow (CLI, HTTP, Web, TUI share it)

1. Reference parse (`crates/quran-core/src/reference/`): frozen grammar (ADR-0102) parses `1:1`, ranges, divisions; `resolve`/`canonical_form`/`serialize` produce stable addressing.
2. Reader lookup (`crates/application/src/quran_reader.rs:QuranReader`): exact ayah/range/surah/division/token expansion with structure-bounded context caps; generation-keyed `lru` cache keyed by `(edition_id, version, corpus_generation, ref, options)` wholesale-invalidated on activation (ADR-0113).
3. Quotation/citation envelope (`crates/quran-core/src/quotation.rs`, `crates/citations/src/lib.rs`): every `QuranQuotation` carries edition + version + hash (I6); resolver returns `ExactMatch`/…/`Mismatch`/`LocationNotFound`/`EditionNotFound`/`AccessDenied` verdicts.
4. Surface rendering: CLI via `crates/application/src/quran_cli.rs` (human text vs `--json`); HTTP via `crates/server/src/api.rs` stable `Envelope { api_version, data, meta }` with `ETag = (text_hash, corpus_generation)`, `Cache-Control`, `Content-Language`, `Diagnostic` error bodies; web via `web/src/api/client.ts` `fetchAyahs`/`resolveReference`/`fetchCitation`/`runTool` into `ReadingView`/`CitationView`; TUI via shared service handles (`crates/tui/src/app.rs:TuiServices`).

### Derived Normalization → Search Flow

1. Forms rebuild (`crates/application/src/quran_forms.rs`, `qai quran forms rebuild <slug@version>`): applies normalization profiles L0–L8 (`crates/quran-normalization/src/profile.rs`) and rules N01–N22 (`crates/quran-normalization/src/rules/`) to produce token/ayah forms + skeletons in `migrations/sqlite/0014_quran_forms.up.sql` tables; MV-018 verifier asserts canonical text unchanged before and after (I8).
2. Index build (`crates/application/src/quran_index.rs`, `qai quran index rebuild|verify`): `FullTextIndex` port (`crates/quran-search/src/index.rs`) with FTS5 backend (`crates/quran-search/src/fts5.rs`) and `ar_*` tokenizers (`crates/quran-search/src/tokenizer.rs`); every index records the `corpus_generation` it was built from (`migrations/sqlite/0015_quran_indexes.up.sql`).
3. Query (`crates/application/src/quran_search.rs`, `crates/quran-search/src/model.rs`): exact / normalized / phrase / concatenated + cross-ayah windows, DFA-only bounded regex (`crates/quran-search/src/regex.rs`, I16), filters, totals, highlighting (`crates/quran-search/src/highlight.rs`), generation-keyed result cache (`crates/application/src/quran_search_cache.rs`, `migrations/sqlite/0016_quran_search_cache.up.sql`).

### Morphology → Lexicon Flow

1. Dataset import (`crates/application/src/quran_morphology.rs`, `0017_quran_lexicon` + `0018_morphology_staging`): CSV/JSON adapters (`crates/quran-morphology/src/adapter_csv.rs`, `adapter_json.rs`) parse the intermediate format; policy + license gates (`policy.rs`, `license.rs`) run before anything persists; alignment (`align.rs`) maps analyses to canonical tokens; MV validation (`validate.rs`) plus `compare.rs` verdicts guard the build.
2. Lexicon API (`crates/application/src/quran_lexicon_api.rs`): root/lemma/morphology/family lookups plus five counting instruments (`crates/application/src/quran_counting.rs`) over frozen counting rules (ADR-0211); served as `quran.root`/`lemma`/`morphology`/`family` tools, `…/count/*` HTTP routes, and `web/src/views/WordsView.tsx`.
3. The test lexicon under `fixtures/quran/lexicon/` is synthetic and labelled (`synthetic_test_only`) — never ground truth; linguist sign-off on tagset/roots/families is pending and the code never claims otherwise.

### Knowledge-Graph Build → Query Flow

1. Structural projection build (`crates/quran-graph/src/structural.rs:build_structural` + `crates/application/src/quran_graph_build.rs`): pure builder maps editions→surahs→ayahs→divisions→tokens to stable IDs; durable staging validates the same rejection rules as the reference backend; activation pins one `graph_projections` row (`migrations/sqlite/0019_quran_graph.up.sql`, fixed by `0022_quran_graph_fix.up.sql`).
2. Query (`crates/application/src/quran_graph_store.rs:SqliteGraphStore` + `quran_graph_api.rs`): loads the pinned projection snapshot through parameterized SQL and serves the `GraphStore` port — neighbors/path/subgraph/typed patterns — with byte-identical budget, authz, tombstone, cancellation, and ordering semantics to `MemGraphStore` by construction. `QueryBudgets::check` pre-flights every query (ADR-0217); export is Graph JSON only (ADR-0218).
3. Annotations (`crates/application/src/quran_graph_annotations.rs`): assertion authority with the same owner-gate shape as verse links — `confirmed` requires reviewer + `decided_at`; served as `quran.graph_*` tools, `/api/v1/quran/graph/*` routes, and `web/src/views/GraphView.tsx`.
4. Isnad side (`crates/application/src/isnad_graph_store.rs`, `isnad_reader.rs`, `isnad_tools.rs`; migrations `0031_narrators`–`0034_narrator_identity_assertions`): narrator trace/rijal over the shared graph vocabulary; served via `qai isnad …`, `/api/v1/…` isnad routes, and `web/src/views/IsnadView.tsx`.

### Multi-RAG Ingest → Query Flow

1. Project setup (`crates/application/src/rag_tools.rs`, `qai rag …`): `rag.create_project` + source registration; config (`crates/rag/src/config.rs`) selects embedding/reranker/router per project.
2. Import (`rag.import`): documents chunked into `RagDocument`s (`0036_rag_documents`), embedded with the project's provider (default `local-hash`), stored in `0037_rag_embeddings` with source rows in `0038_rag_sources`; consistency with the canonical stores follows ADR-0702.
3. Query (`rag.query` via `crates/application/src/rag_search.rs`): lexical/semantic retrieval through the `RetrievalBackend` port, hybrid merge (`crates/rag/src/hybrid.rs`), `ScoreReranker`, tombstone filtering (`crates/rag/src/tombstone.rs`) — tombstoned sources never surface. `rag.debug` returns the route decision alongside hits for `web/src/views/RagDebugView.tsx`.
4. Cross-domain research (`crates/application/src/knowledge_domain.rs`, `cross_domain.rs`): fans one `CrossDomainQuery` out to Quran/Hadith/Tafsir/Scripture/Books searchers plus the RAG books store; results keep domain + source + confidence tags and are never collapsed into consensus; `scripture/research` and `/api/v1/research/cross-domain` serve it (`crates/server/src/routes/scripture.rs`).

### Tool-Dispatch Flow (the agentic runtime contract)

1. Registry (`crates/tool-registry/src/lib.rs:ToolRegistry`): every callable tool is a versioned (`SemVer`), `ReadOnly`, JSON-in/JSON-out entry with typed params and a `ToolResult` carrying `ReproducibilityData` + a single-source `research_checksum` (`crates/tools/src/lib.rs`, never recomputed in the HTTP layer).
2. Dispatch: `POST /api/v1/quran/tool/{name}` (`crates/server/src/api.rs:tool_handler`) → registry → application backend impl (`quran_tools.rs`, `hadith_tools.rs`, `isnad_tools.rs`, `tafsir_tools.rs`, `scripture_tools.rs`, `rag_tools.rs`, `quran_graph_tools.rs`). The web `ToolsView` (`web/src/views/ToolsView.tsx`) renders a typed form, checksum, and CLI equivalent per tool; `ExplorerView` dispatches arbitrary requests against the local engine.
3. Agents do not exist yet: `agent-domain`, `agent-runtime`, `agency`, `conversations`, `tool-sdk`, `tool-sandbox`, `policy`, `approvals`, `workflows`, `memory`, `mcp`, `model-router`, `llm` remain empty Phase-7 placeholders. The registry is the runtime's current boundary — tools prove the §12 contract from all interfaces before agents arrive.

**State Management:**

- SQLite is the sole stateful store (WAL, `foreign_keys=ON`, `busy_timeout`, dual pools: write `max_connections=1`, read `query_only`) — see `crates/storage-sqlite/src/lib.rs` and ADR-0001.
- Schema is append-only and checksummed (`migrations/sqlite/*.up.sql` + `migrations/sqlite/checksums.json`, ADR-0002; now v44). Canonical tables are forward-only (deactivation, not deletion); caches/derived indexes/graph projections/RAG embeddings are generation-stamped so stale reads are impossible by construction.
- In-process state is limited to the reader LRU cache (`crates/application/src/quran_reader.rs`), search cache (`crates/application/src/quran_search_cache.rs`), pinned graph-projection snapshot (`crates/application/src/quran_graph_store.rs`), and the job worker pool (`crates/jobs/src/worker.rs`). No global mutable singletons outside these explicitly scoped caches.

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
- Pattern: `CanonicalWriter` trait takes `&ApprovalToken`; importer path never mints one; activation records a hash-chained audit event in the same transaction; migration `0021` enforces the fence at the DB level.

**`Database` / `ReadTx` / `UnitOfWork` + repository traits:**

- Purpose: Persistence port that keeps domain logic backend-agnostic and PostgreSQL-portable.
- Examples: `crates/storage/src/lib.rs`, `crates/storage/src/repository.rs`, `crates/storage/src/quran.rs`, `crates/storage-sqlite/src/lib.rs`
- Pattern: `async-trait` ports; one shared `SqlTx` behind `Arc<tokio::sync::Mutex<_>>` so a `UnitOfWork` commit is atomic across all repos (outbox invariant).

**`JobHandler` / durable queue:**

- Purpose: Cancellable, resumable, idempotent background work without an external broker (ADR-0003).
- Examples: `crates/jobs/src/lib.rs`, `crates/jobs/src/queue.rs`, `crates/jobs/src/worker.rs`, `crates/jobs/src/registry.rs`, `crates/application/src/job_queue.rs`, `crates/application/src/quran.rs:QuranImportHandler`
- Pattern: `kind` + `payload_schema` + `is_idempotent` + `run(ctx, payload)`; idempotency key `source_version:adapter:parser` (`crates/application/src/quran.rs:import_idempotency_key`).

**`NormalizationPipeline` / `SpanMap` / `NormalizationTrace`:**

- Purpose: Reproducible derived text with exact back-mapping to canonical offsets.
- Examples: `crates/quran-normalization/src/pipeline.rs`, `crates/quran-normalization/src/span.rs`, `crates/quran-normalization/src/trace.rs`, `crates/quran-normalization/src/profile.rs`
- Pattern: Ordered rule application (N01–N22) under a profile (L0–L8); every step emits a bidirectional `SpanMap` (I10, property-tested) and a `NormalizationTrace` recording the exact rule set (I9).

**`FullTextIndex` / `ToolResult`:**

- Purpose: Backend-independent search semantics; reproducible agent-tool contract.
- Examples: `crates/quran-search/src/index.rs`, `crates/quran-search/src/model.rs`, `crates/tools/src/lib.rs`, `crates/tool-registry/src/lib.rs`
- Pattern: `FtsQuery`/`FtsResults`/`IndexManifest` types over the port; `ToolResult` + `ReproducibilityData` + single-source `research_checksum` on every tool call.

**`GraphStore` / `QueryBudgets` / typed `Pattern` (knowledge graph):**

- Purpose: Typed, bounded, authorization-scoped graph access with no backend query language leaking across the port.
- Examples: `crates/quran-graph/src/store.rs`, `crates/quran-graph/src/model.rs`, `crates/quran-graph/src/pattern.rs`, `crates/quran-graph/src/traverse.rs`, `crates/quran-graph/src/structural.rs`, `crates/quran-graph/src/export.rs`
- Pattern: `MemGraphStore` reference backend + conformance suite; `SqliteGraphStore` snapshot adapter passes the same suite; `QueryBudgets::check` pre-flight, per-batch cancellation, stable-ID ordering, typed incompletes; structural builder emits stable IDs (`ayah_stable_id`, `token_stable_id`, …) under `STRUCTURAL_PROJECTION_ID`.

**`RetrievalBackend` / `EmbeddingProvider` / `Reranker` (multi-RAG):**

- Purpose: Swappable retrieval pieces with deterministic local defaults and no network.
- Examples: `crates/retrieval/src/lib.rs`, `crates/retrieval/src/memvid_backend.rs`, `crates/embeddings/src/lib.rs`, `crates/reranking/src/lib.rs`, `crates/rag/src/hybrid.rs`, `crates/rag/src/tombstone.rs`
- Pattern: Port traits with recorded backend names for reproducibility; `local-hash` embeddings, `score` reranking, `.mv2` file backend behind the `memvid` feature.

**`VerseLink` + owner gate / `KnowledgeDomain` / cross-domain aggregator:**

- Purpose: Computational matches are always labelled, never certain; cross-domain results are never collapsed into consensus.
- Examples: `crates/application/src/verse_link.rs`, `crates/application/src/knowledge_domain.rs`, `crates/application/src/cross_domain.rs`, `crates/application/src/disagreement.rs`
- Pattern: Confidence bands (high/medium/low) as display hints only; `confirmed` requires reviewer + `decided_at` (Rust + DB CHECK on migration `0030`); five `KnowledgeDomain`s (Quran, Hadith, Tafsir, Scripture, Books) with explicit opt-out.

**`Diagnostic` / `StorageError`:**

- Purpose: Unique-code typed errors mapped to public CLI exit codes.
- Examples: `crates/domain/src/diagnostic.rs`, `crates/quran-core/src/error.rs`, `crates/quran-corpus/src/error.rs`, `crates/quran-normalization/src/error.rs`, `crates/quran-search/src/error.rs`, `crates/quran-graph/src/error.rs`, `crates/quran-morphology/src/error.rs`, `crates/retrieval/src/error.rs`, `crates/storage/src/error.rs`, `crates/cli/src/exit_code.rs`
- Pattern: `QAI-<NS>-NNNN` codes unique workspace-wide (test-enforced); exit codes 0/1/2/3/4/5/6/7/70 are public API.

## Entry Points

**`qai` binary:**

- Location: `crates/cli/src/main.rs`
- Triggers: Any `./target/debug/qai …` invocation; `dispatch()` in `crates/cli/src/lib.rs`.
- Responsibilities: Parse clap tree (`Status|Version|Doctor|Config|Db|Secret|Source|Job|Audit|Serve|Completions|Quran|Hadith|Isnad|Tafsir|Scripture|Rag|Links|Graph`), load layered config, route to `application` services, map results to process exit codes.

**`application::run` bootstrap:**

- Location: `crates/application/src/lib.rs:run`
- Triggers: `qai serve` and any command needing a live backend.
- Responsibilities: Validate config, init observability with secret redaction, open `SqliteDatabase`, health-check storage, report `RunResult { health }`.

**HTTP serve:**

- Location: `crates/server/src/lib.rs:loopback_addr`, routes in `crates/server/src/api.rs` + `crates/server/src/routes/scripture.rs`, static web hosting in `crates/server/src/webassets.rs`; dispatched from `Commands::Serve`.
- Triggers: `./target/debug/qai serve --bind 127.0.0.1:8737`.
- Responsibilities: Bind axum `TcpListener` (non-loopback refused with exit 4); serve `/healthz`, `/readyz`, `/api/v1/meta`, Quran read endpoints per `docs/08-api/quran-v1-openapi.json`, normalization `preview`/`profiles`, five search modes + concatenated + regex, lexicon/morphology/family/count routes, graph routes, hadith/tafsir/verse-link/isnad routes, scripture/cross-domain research routes, tool-dispatch `POST /api/v1/quran/tool/{name}`, `/debug/read/{edition}/{surah}`, and the built web GUI.

**Web GUI:**

- Location: `web/src/App.tsx` → `web/src/router.tsx` (path router over `window.location`), views in `web/src/views/`, typed client in `web/src/api/client.ts`.
- Triggers: Browser navigation to `/`, `/read/<edition>/<ref>`, `/search`, `/research`, `/words`, `/tools`, `/citations`, `/hadith`, `/isnad`, `/tafsir`, `/scripture`, `/graph`, `/rag-debug`, `/explorer`.
- Responsibilities: Reader (deep-link driven, never constructs verses locally), five lexical search modes, nine research instruments, morphology/lexicon/counts, typed tool forms with checksums, citation workbench with local tamper simulation (labelled local), hadith/isnad/tafsir/scripture browsers, graph explorer, RAG-debug surface, generic API explorer. Fail-closed empty states everywhere (e.g. reader renders the remedy `qai db migrate && qai quran catalog … && qai quran import …` instead of fabricating text).

**TUI cockpit:**

- Location: `crates/tui/src/lib.rs:run`, `crates/tui/src/app.rs:App/TuiServices`, screens in `crates/tui/src/screens/`, command palette in `crates/tui/src/palette.rs`.
- Triggers: `qai tui` (or equivalent wiring through `dispatch()`).
- Responsibilities: Terminal dashboard over the same `application` service handles as `serve`; untrusted dataset text passes through `sanitize_terminal_text` (terminal analogue of the server's `escape_html`).

**`quran.import` job handler:**

- Location: `crates/application/src/quran.rs:QuranImportHandler`
- Triggers: `qai quran import <manifest>` (runs inline) or a queued `quran.import` job.
- Responsibilities: Drive the 13-checkpoint pipeline to `Staged`; never activate.

**Doctor checks:**

- Location: `crates/cli/src/doctor.rs:run_checks` / `run_quran_checks`, data from `crates/application/src/quran_doctor.rs`, `quran_doctor_indexes.rs`, and `crates/application/src/db.rs:DbProbe`.
- Triggers: `./target/debug/qai doctor --quran --deep [--json]`.
- Responsibilities: Read-only diagnostics (`Pass|Warn|Fail|Skipped` + remedy + `next_command`); never mutates data.

## Architectural Constraints

- **Threading:** tokio `rt-multi-thread` (`Cargo.toml` workspace deps); SQLite write pool is a single serialized connection, read pool is `query_only`; the reader notes a dedicated read-pool path as future work — concurrent readers currently serialize behind the single-writer pool (`crates/application/src/quran_reader.rs` docs).
- **Global state:** No module-level mutable singletons. Scoped shared state only: reader `LruCache` behind `Mutex` (`crates/application/src/quran_reader.rs`), search cache (`crates/application/src/quran_search_cache.rs`), pinned graph-projection snapshot (`crates/application/src/quran_graph_store.rs`), shared write `SqlTx` behind `Arc<tokio::sync::Mutex<_>>` (`crates/storage-sqlite/src/lib.rs`), job worker pool (`crates/jobs/src/worker.rs`).
- **Circular imports:** None. Dependency direction is machine-enforced: `cargo xtask arch-check` vs `xtask/allowlist.toml`; a crate absent from the allowlist may have zero workspace deps (fail-closed for new/placeholder crates).
- **Purity fence:** `domain` → `(serde, thiserror, time, uuid)` only, no I/O/async (`xtask/allowlist.toml:[domain]`); `quran-core` adds only `unicode-*`; `quran-corpus`/`quran-normalization`/`quran-morphology`/`quran-search`/`quran-graph` never gain llm/embeddings/retrieval/vector deps (I2/I8, AC-P2-36).
- **Canonical-write fence:** Canonical rows writable only through `CanonicalWriter` + `ApprovalToken` (`crates/provenance/src/lib.rs`), now also enforced by trigger in `migrations/sqlite/0021_canonical_write_fence.up.sql`; staging tables (`quran_stg_*`, morphology staging `0018`) have no immutability triggers and cascade-delete per run; canonical tables are insert-only/forward-only.
- **Graph query fence:** No backend query language crosses the `quran-graph` port (test-pinned); all traversal is budgeted (`QueryBudgets`), allowlisted (`EDGE_VOCABULARY`, typed `Pattern`s), authz-scoped, tombstone-filtered, and cancellable (ADR-0202/0217).
- **Uncertainty fence:** Computational verse links, isnad matches, and graph assertions are labelled `"uncertain"` until owner-confirmed (reviewer + `decided_at`, DB CHECK on `0030`/`0034`); evaluation reports `Unavailable`, never `0`.
- **Placeholder fence:** 16 remaining placeholders (`graph`, `ingestion`, `llm`, `model-router`, `agent-domain`, `agent-runtime`, `agency`, `conversations`, `tool-sdk`, `tool-sandbox`, `policy`, `approvals`, `workflows`, `memory`, `mcp`, `api`) contain only a `//! Phase N` doc and must stay empty until their phase starts. Former placeholders now implemented: `quran-morphology`, `quran-graph`, `hadith-core`, `hadith-ingestion`, `isnad-graph`, `tafsir`, `scripture`, `retrieval`, `rag`, `embeddings`, `reranking`, `evaluation`, `tui` (note: `Cargo.toml` member comments still say "Placeholder crates" — stale comment, members list is authoritative).
- **Network fence:** Local-first, deny-by-default; `serve` binds loopback only (`crates/server/src/lib.rs:loopback_addr`); telemetry/OTLP is opt-in (`crates/observability/src/otlp.rs`); embeddings/retrieval run local deterministic providers — no network in the RAG path; internet content is untrusted until validated; `domain` security guards cover path/archive/net/input/sanitize (`crates/domain/src/security*.rs`); terminal renders sanitize control sequences (`crates/tui/src/lib.rs:sanitize_terminal_text`).

## Anti-Patterns

### Reaching past the composition root

**What happens:** An edge crate (`server`, `cli`, `tui`) imports `storage-sqlite` or concrete reader internals directly instead of going through `application` services/re-exports or a backend trait.
**Why it's wrong:** It hard-codes the backend choice in two places, defeats the `storage` port (PostgreSQL portability), and makes contract tests need a database. The `server` allowlist entry documents this exact smell (`xtask/allowlist.toml:[server]` — `storage` + `tools` allowed only for `ReaderBackend`/`ToolError`, with a note to prefer `application` re-exports).
**Do this instead:** Add the operation to `crates/application/src/` (e.g. `crates/application/src/quran_reader.rs`, `crates/application/src/quran_cli.rs`, `crates/application/src/db.rs`) and call it from the edge; depend on traits (`crates/server/src/api.rs:QuranApiBackend`, `crates/tool-registry/src/lib.rs:QuranBackend`, `crates/citations/src/lib.rs:CitationSource`) with fakes in tests. The TUI already models this: `TuiServices` holds `Arc<dyn …>` handles built by the shared composition path (`crates/tui/src/app.rs`).

### Mutating canonical state without approval

**What happens:** New code writes `quran_editions`/surahs/ayahs/tokens directly (or extends the importer to flip the active pointer) instead of staging + human-gated activation.
**Why it's wrong:** It violates I1/I5/I7, bypasses the audit trail, and breaks the byte-equality corpus gate everything downstream inherits (search, citations, tafsir links, graph projections, RAG consistency per ADR-0702).
**Do this instead:** Follow `crates/quran-corpus/src/import.rs:run_import` (stage only, ends at `ApprovalRequested`) plus `crates/application/src/quran.rs:activate_edition` (requires `ApprovalToken` for the exact `quran-edition:{slug}@{version}` URN, same-transaction audit via `crates/application/src/audit_bridge.rs`).

### Leaking backend query language through the graph port

**What happens:** New graph features accept raw query strings (SQL/Cypher/Gremlin fragments) or return backend handles instead of typed `GraphNode`/`GraphEdge`/`GraphPath` values.
**Why it's wrong:** It defeats the port's backend-swappability (SQLite today, other stores later), bypasses `QueryBudgets`/authz/tombstone filtering, and breaks the byte-identical-semantics guarantee between `MemGraphStore` and `SqliteGraphStore`.
**Do this instead:** Extend the typed surface in `crates/quran-graph/src/` (`model.rs` node/edge kinds, `pattern.rs` allowlisted steps, `store.rs` port methods) and implement both backends; the conformance suite plus the no-backend-language test pin the boundary.

### Presenting computational matches as certainty

**What happens:** A verse link, isnad inference, graph assertion, or eval metric is rendered as confirmed/scholarly consensus, or a missing dataset renders as `0`.
**Why it's wrong:** It violates D-07 and the project's core stance (exact text before interpretation; no false consensus — `docs/00-overview/project-overview.md`). A `0` metric reads as a real measurement.
**Do this instead:** Keep the `"uncertain"` label until owner confirmation (`crates/application/src/verse_link.rs`), per-scholar gradings unmerged (`crates/hadith-core/src/grading.rs`), and `Unavailable` + remedy from the eval harness (`crates/evaluation/src/lib.rs`); the web marks simulations as local (`CitationsWorkbenchView`, tamper simulator).

## Error Handling

**Strategy:** Typed `Diagnostic` errors with unique `QAI-<NS>-NNNN` codes (test-enforced uniqueness), mapped to stable public CLI exit codes and HTTP `Diagnostic` bodies. The web preserves server error codes (`web/src/api/client.ts:ApiRequestError` keeps `code` + `remedy` beside the summary).

**Patterns:**

- Domain/phase `Diagnostic` types per crate (`crates/domain/src/diagnostic.rs`, `crates/quran-core/src/error.rs`, `crates/quran-corpus/src/error.rs`, `crates/quran-normalization/src/error.rs`, `crates/quran-search/src/error.rs`, `crates/quran-graph/src/error.rs`, `crates/quran-morphology/src/error.rs`, `crates/retrieval/src/error.rs`, `crates/storage/src/error.rs`) with `code()` → exit-code mapping in `crates/cli/src/exit_code.rs` and `crates/application/src/quran_cli.rs:exit`.
- `thiserror` for structured errors, `anyhow` at the `xtask` boundary; `Result<T, StorageError>` on the storage port; `ReaderError` preserves reference-grammar `01xx` codes (`crates/application/src/quran_reader.rs`).
- No-fabrication as types: unknown verse/translation/citation surfaces as `EditionNotFound`/`AyahNotFound`/`TranslationNotFound`/`Mismatch`/`LocationNotFound`, never a synthesized fallback — enforced identically in CLI, HTTP, web (`ApiRequestError`), and TUI surfaces.

## Cross-Cutting Concerns

**Logging:** `tracing` + `tracing-subscriber` with secret redaction, structured spans, and a metrics catalog (`crates/observability/src/lib.rs`, `crates/observability/src/telemetry.rs`, `crates/observability/src/metrics.rs`); OTLP export is opt-in only (`crates/observability/src/otlp.rs`); secrets render as `***` via `Secret<T>` (`crates/config/src/secret.rs`) and `crates/domain/src/redaction.rs`.
**Validation:** Layered — manifest schema (`docs/schemas/quran-edition-source.v1.schema.json`), Unicode auditor (`crates/quran-corpus/src/unicode.rs`), QV-001…QV-028 corpus validator (`crates/quran-corpus/src/validation.rs`), morphology policy/license/MV gates (`crates/quran-morphology/src/policy.rs`, `license.rs`, `validate.rs`), graph rejection rules shared by staging and reference backend, config validation at load and on change (`crates/config/src/lib.rs`), payload schemas on job handlers (`crates/application/src/quran.rs:payload_schema`), DFA-only regex bounds (`crates/quran-search/src/regex.rs`).
**Authentication:** No auth provider (local-first single-operator model). Identity primitives exist for provenance (`principals` table in `migrations/sqlite/0001_core.up.sql`, `Actor` in `crates/audit/src/lib.rs`, `PrincipalId` in `crates/domain/src/ids.rs`); `serve` enforces loopback-only binding as its access control (`crates/server/src/lib.rs`).

---

*Architecture analysis: 2026-11-01*
