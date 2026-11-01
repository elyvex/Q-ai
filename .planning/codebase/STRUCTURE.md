---
last_mapped_commit: 6891d7bc64bc469a10e9a28faf81023fa11cb9ae
last_mapped_at: 2026-11-01
---
# Codebase Structure

**Analysis Date:** 2026-11-01

## Directory Layout

```
Q-ai/
├── crates/             # Rust workspace members (all backend code lives here)
│   ├── application/    # Composition root — Quran/Hadith/Tafsir/Scripture/RAG/graph services
│   ├── domain/         # Pure shared vocabulary (no I/O)
│   ├── quran-core/     # Pure Quran domain (grammar, types, quotations)
│   ├── quran-corpus/   # Adapters, tokenizer, validator, differ, importer
│   ├── quran-normalization/ # Derived normalization rules/pipeline
│   ├── quran-morphology/    # Lexicon types: adapters, alignment, families, tagset
│   ├── quran-search/   # FTS5 backend + search services
│   ├── quran-graph/    # Knowledge-graph port + reference backend (NOT storage)
│   ├── hadith-core/    # Hadith domain types (collection/record/grading)
│   ├── hadith-ingestion/    # Hadith parser + validator
│   ├── isnad-graph/    # Narrator/isnad graph types over shared graph vocabulary
│   ├── tafsir/         # Tafsir works + records
│   ├── scripture/      # Comparative-scripture editions/books/verses
│   ├── rag/            # Multi-RAG projects/sources/documents, hybrid, tombstones
│   ├── retrieval/      # RetrievalBackend port (+ memvid backend, feature-gated)
│   ├── embeddings/     # EmbeddingProvider port (local-hash default)
│   ├── reranking/      # Reranker port (score default)
│   ├── evaluation/     # Mechanical regression harness (datasets/metrics/report)
│   ├── citations/      # Citation resolver + quotation verification
│   ├── tools/          # ToolResult contract
│   ├── tool-registry/  # ~30 versioned read-only tools, all five domains
│   ├── storage/        # Persistence traits (ports)
│   ├── storage-sqlite/ # Sole storage adapter (sqlx/SQLite)
│   ├── provenance/     # ApprovalToken / CanonicalWriter
│   ├── audit/          # Hash-chained audit log
│   ├── jobs/           # Durable leased job queue
│   ├── sources/        # Manifests + lifecycle state machine
│   ├── config/         # Layered config + Secret<T>
│   ├── observability/  # tracing / metrics / opt-in OTLP
│   ├── server/         # Axum HTTP API v1 (+ scripture routes, webassets)
│   ├── cli/            # `qai` binary + per-domain subcommands + doctor
│   ├── tui/            # Ratatui operations cockpit over application services
│   ├── testkit/        # Dev-only fixtures (never ships)
│   └── <~16 placeholders>/ # Empty `//! Phase N` crates (agent stack, llm, mcp, …)
├── web/                # Vite + React 19 research workspace (served by server/webassets)
│   ├── src/router.tsx   # Path router: 15 routes + deep-link parsing
│   ├── src/views/       # One view per route (Reader, Search, Graph, …)
│   ├── src/api/         # Typed /api/v1 client (mirrors Envelope<T>)
│   └── src/components/  # App shell, tool ledger, citation copy, states, UI kit
├── xtask/              # Build gates: arch-check, migrate-check, ci, schema
├── migrations/sqlite/  # Append-only checksummed SQL (0001–0044 + checksums.json)
├── fixtures/quran/     # Synthetic test editions, adversarial + golden sets (+ lexicon/)
├── docs/               # Numbered doc tree 00–10 + decisions, schemas, plans
├── graft/              # Code knowledge-graph index (INDEX.md + per-scope nodes)
├── tests/              # Workspace-level stub (README only; real tests per crate)
├── src/                # Root stub (README only; real code is in crates/)
├── scripts/            # README only (operational scripts live in docs runbooks)
├── examples/config/    # Example configuration files
├── specs/              # Spec-kit state (.spekkit-state.json at root)
├── .cargo/config.toml  # `cargo xtask <cmd>` alias
├── Cargo.toml          # Workspace manifest (members + centralized deps)
├── rust-toolchain.toml # Pinned toolchain 1.97.1
├── clippy.toml / rustfmt.toml / deny.toml # Lint/format/supply-chain config
├── Dockerfile / docker-compose.yml / .dockerignore # Container deployment
└── .planning/codebase/ # This codebase map (generated, committed)
```

## Directory Purposes

**`crates/`:**

- Purpose: The entire Rust workspace; every member is a separate crate with its own `Cargo.toml` + `src/`.
- Contains: Phase 0–9 implemented crates (foundation, Quran engine + derived layer, hadith/isnad/tafsir/scripture, RAG stack, tools, server/cli/tui), `testkit`, `xtask` (as member), and ~16 empty placeholder crates reserving agent-stack names.
- Key files: `Cargo.toml` (workspace root manifest), `crates/application/src/lib.rs`, `crates/cli/src/main.rs`

**`crates/application/`:**

- Purpose: Composition root — the only crate allowed to wire concrete backends together.
- Contains: Quran services (`quran*.rs`: cli/reader/forms/index/normalize/search/counting/lexicon/morphology/search_api/graph_*), per-domain triplets (`hadith_reader/search/tools`, `isnad_reader/tools/graph_store`, `tafsir_reader/search/tools`, `scripture_reader/search/tools`, `rag_reader/search/tools`), cross-corpus glue (`verse_link.rs`, `knowledge_domain.rs`, `cross_domain.rs`, `disagreement.rs`), DB orchestration (`db.rs`), audit bridge, job queue glue.
- Key files: `crates/application/src/lib.rs`, `crates/application/src/quran.rs`, `crates/application/src/quran_cli.rs`, `crates/application/src/quran_reader.rs`, `crates/application/src/db.rs`

**`crates/domain/` + `crates/quran-core/` + sibling domain crates:**

- Purpose: Pure, side-effect-free vocabulary layers.
- Contains: IDs, primitives, hashing, licensing, security guards, `Diagnostic` (`crates/domain/src/`); reference grammar, enums, editions, structure, quotations, views (`crates/quran-core/src/`, incl. `crates/quran-core/src/reference/`); hadith collection/record/grading (`crates/hadith-core/src/`); tafsir work/record (`crates/tafsir/src/`); scripture book/edition/verse/reference/translation/view (`crates/scripture/src/`); morphology dataset/family/tagset/policy (`crates/quran-morphology/src/`); narrator/isnad types over shared graph vocabulary (`crates/isnad-graph/src/lib.rs`).
- Key files: `crates/domain/src/lib.rs`, `crates/domain/src/diagnostic.rs`, `crates/quran-core/src/reference/`, `crates/quran-core/src/quotation.rs`

**`crates/quran-corpus/` + `crates/quran-normalization/` + `crates/quran-morphology/` + `crates/quran-search/` + `crates/quran-graph/`:**

- Purpose: Canonical pipeline (adapters → audit → tokenize → validate → import → activate) and the derived normalization/morphology/search/graph layer.
- Contains: `adapters.rs`, `unicode.rs`, `tokenize.rs`, `hashing.rs`, `validation.rs`, `differ.rs`, `import.rs` (`quran-corpus`); `rules/`, `pipeline.rs`, `span.rs`, `profile.rs`, `trace.rs` (`quran-normalization`); `adapter_csv.rs`, `adapter_json.rs`, `align.rs`, `validate.rs`, `compare.rs`, `family.rs`, `tagset.rs`, `policy.rs`, `license.rs` (`quran-morphology`); `fts5.rs`, `tokenizer.rs`, `index.rs`, `model.rs`, `regex.rs`, `highlight.rs`, `trigram.rs`, `skeleton.rs` (`quran-search`); `model.rs`, `store.rs`, `mem.rs`, `traverse.rs`, `pattern.rs`, `structural.rs`, `export.rs` (`quran-graph`).
- Key files: `crates/quran-corpus/src/import.rs`, `crates/quran-corpus/src/validation.rs`, `crates/quran-search/src/fts5.rs`, `crates/quran-graph/src/store.rs`

**`crates/rag/` + `crates/retrieval/` + `crates/embeddings/` + `crates/reranking/`:**

- Purpose: Local-first multi-RAG platform: project-scoped retrieval with deterministic providers.
- Contains: `config.rs`, `project.rs`, `hybrid.rs`, `tombstone.rs` (`rag`); `document.rs`, `hit.rs`, `memvid_backend.rs` (`retrieval`); `EmbeddingProvider` + `LocalHashEmbedding` (`embeddings`); `Reranker` + `ScoreReranker` (`reranking`).
- Key files: `crates/rag/src/lib.rs`, `crates/retrieval/src/lib.rs`, `crates/application/src/rag_search.rs`

**`crates/storage/` + `crates/storage-sqlite/`:**

- Purpose: Persistence port (traits) + sole SQLite adapter.
- Location: traits in `crates/storage/src/`; impl in `crates/storage-sqlite/src/`.
- Contains: `lib.rs` (`Database`/`ReadTx`/`UnitOfWork`), `repository.rs`, `quran.rs`, `workflows.rs`, `error.rs` (`storage`); `lib.rs` (pools), `migrate.rs`, `quran.rs` (`storage-sqlite`).
- Key files: `crates/storage/src/repository.rs`, `crates/storage-sqlite/src/migrate.rs`

**`crates/cli/` + `crates/server/` + `crates/tui/`:**

- Purpose: Thin edge surfaces; no business logic beyond routing/rendering.
- Contains: clap tree + `dispatch()` + per-domain subcommand files + `doctor.rs` + `exit_code.rs` + `main.rs` (`cli`); `api.rs` (routes + envelope) + `routes/scripture.rs` + `webassets.rs` + `lib.rs` (loopback guard) (`server`); `app.rs` + `palette.rs` + `screens/` + `scripture.rs` (`tui`).
- Key files: `crates/cli/src/lib.rs`, `crates/cli/src/doctor.rs`, `crates/server/src/api.rs`

**`web/`:**

- Purpose: Vite + React 19 research workspace; talks to the server's HTTP API only (no Rust dependency); built assets are served by `crates/server/src/webassets.rs`.
- Contains: Path router (`src/router.tsx`), 15 route views (`src/views/`), typed API client (`src/api/client.ts`), layout/components (`src/components/`), domain helpers (`src/lib/`), display-mode control (`src/display/`), design tokens (`src/tokens/`), tests co-located (`*.test.tsx(x)`), Vite + vitest + Tailwind v4 config.
- Key files: `web/src/router.tsx`, `web/src/App.tsx`, `web/src/api/client.ts`, `web/vite.config.ts`

**`xtask/`:**

- Purpose: Build-time enforcement and developer tooling (invoked as `cargo xtask <cmd>`).
- Contains: `arch.rs` (dependency-direction gate), `migrate.rs` (checksum gate), `ci.rs` (9-step gate), `schema.rs`, `adr.rs`, `coverage.rs`.
- Key files: `xtask/src/main.rs`, `xtask/src/arch.rs`, `xtask/allowlist.toml`

**`migrations/sqlite/`:**

- Purpose: Append-only, checksummed schema evolution; workspace at schema v44.
- Contains: `0001_core` … `0044_verse_links_scripture_source` `.up.sql` files (+ `.down.sql` for 0001–0006 only), `checksums.json`. Quran stack: 0007–0022 (editions, structure, divisions, translations, staging, validation, normalization, forms, indexes, search cache, lexicon, morphology staging, graph ×2, edition identity, write fence, graph fix). Other corpora: 0023–0034 (hadith ×4, tafsir ×3, verse_links, narrators/rijal/isnad ×4). RAG: 0035–0038 (projects, documents, embeddings, sources). Scripture: 0039–0044 (editions, books, verses, translations, search forms, verse-link source).
- Key files: `migrations/sqlite/0007_quran_editions.up.sql`, `migrations/sqlite/0011_quran_staging.up.sql`, `migrations/sqlite/0021_canonical_write_fence.up.sql`, `migrations/sqlite/checksums.json`

**`fixtures/quran/`:**

- Purpose: Synthetic (never real scripture) test data for plumbing, adversarial, and golden tests — plus the synthetic labelled lexicon.
- Contains: `test-edition-min/` (+ `test-edition-min-v2.json`), `test-translation-min.json`, `test-gloss-min.json`, `adversarial/` (×16, one per QV rule), `golden/references.jsonl` (331 reference cases), `lexicon/` (synthetic `synthetic_test_only` morphology data).
- Key files: `fixtures/quran/test-edition-min/manifest.json`, `fixtures/quran/golden/references.jsonl`

**`docs/`:**

- Purpose: Numbered documentation tree; reading order matters (see `AGENTS.md`).
- Contains: `00-overview/` (incl. `agent-briefing.md`), `01-requirements/requirements.md` (PRD), `02-architecture/decisions/` (ADRs incl. `crate-map.md`; Quran engine 0101–0114, derived layer 0201–0218, RAG 0301, vector-store/cross-store 0701–0702), `03-plan/phases/` (phase boards), `05-followups/`, `06-progress/`, `07-technical/`, `08-api/quran-v1-openapi.json`, `09-testing/`, `10-operations/`, `schemas/`, plus `archive/`, `plans/`, `runbooks/`.
- Key files: `docs/00-overview/agent-briefing.md`, `docs/02-architecture/decisions/crate-map.md`, `docs/08-api/quran-v1-openapi.json`

**`graft/`:**

- Purpose: Prebuilt code knowledge-graph index for agent orientation (not runtime code).
- Contains: `INDEX.md` (node list) + per-scope node files (e.g. `graft/crates/…`, `graft/xtask/…`).
- Key files: `graft/INDEX.md`

## Key File Locations

**Entry Points:**

- `crates/cli/src/main.rs`: `qai` binary `main()` — parses clap, calls `dispatch()`, exits with code.
- `crates/cli/src/lib.rs`: `Cli`/`Commands` clap tree + `dispatch()` router for every subcommand.
- `crates/application/src/lib.rs`: `run()` bootstrap — config validation, observability init, DB health check.
- `crates/server/src/lib.rs`: `loopback_addr()` bind guard; routes live in `crates/server/src/api.rs` + `crates/server/src/routes/scripture.rs`; static web assets in `crates/server/src/webassets.rs`.
- `web/src/App.tsx` → `web/src/router.tsx`: web entry; path router rendering one of 15 views.
- `crates/tui/src/lib.rs:run`: TUI entry over shared `application` services.
- `xtask/src/main.rs`: `cargo xtask <cmd>` command implementations.

**Configuration:**

- `Cargo.toml`: Workspace manifest — member list, workspace lints (`unsafe_code = "forbid"`), centralized `[workspace.dependencies]`.
- `rust-toolchain.toml`: Pinned toolchain (`1.97.1` + `rustfmt`/`clippy`).
- `.cargo/config.toml`: `xtask = "run --quiet --package xtask --"` alias.
- `xtask/allowlist.toml`: Dependency-direction rules enforced by `arch-check` (fail-closed).
- `crates/config/src/lib.rs`: Layered loader (CLI > env > file > defaults) + validation.
- `web/vite.config.ts`, `web/tsconfig*.json`, `web/vitest.config.ts`, `web/package.json`: frontend build/test config (React 19, Tailwind v4, vitest).
- `clippy.toml`, `rustfmt.toml`, `deny.toml`: Lint, format, and supply-chain policy.
- `.env.example`: Documents required env vars (existence only; never commit real `.env`).

**Core Logic:**

- `crates/quran-corpus/src/import.rs`: 13-checkpoint staged importer (`run_import`).
- `crates/application/src/quran.rs`: `QuranImportHandler` + approval-gated `activate_edition`/`rollback_edition`.
- `crates/application/src/quran_reader.rs`: `QuranReader` deterministic lookup + LRU cache.
- `crates/quran-normalization/src/pipeline.rs`: Normalization pipeline over profiles/rules.
- `crates/quran-search/src/index.rs` + `crates/quran-search/src/fts5.rs`: `FullTextIndex` port + FTS5 adapter.
- `crates/quran-morphology/src/`: Lexicon adapters/alignment/validation/families; service in `crates/application/src/quran_morphology.rs`, API in `quran_lexicon_api.rs`, counts in `quran_counting.rs`.
- `crates/quran-graph/src/store.rs` + `crates/application/src/quran_graph_store.rs`: graph port + SQLite snapshot adapter.
- `crates/citations/src/lib.rs`: Citation resolver + quotation verification.
- `crates/provenance/src/lib.rs`: `ApprovalToken` + `CanonicalWriter` trust root.
- `crates/tool-registry/src/lib.rs`: All ~30 versioned tool impls + backend traits.

**Testing:**

- Per-crate integration suites: `crates/storage-sqlite/tests/` (13 suites), `crates/application/tests/`, `crates/cli/tests/quran/read_flow.trycmd` (trycmd acceptance), `crates/server/tests/` (API contract vs fakes), `crates/testkit/tests/` (security/config).
- Unit + property tests inline: `crates/*/src/` (`#[cfg(test)]` modules, proptest for grammar/tokenizer/span-map).
- Frontend tests co-located: `web/src/**/*.test.tsx(x)` via vitest (`web/vitest.config.ts`); e.g. `web/src/router.test.tsx`, `web/src/views/*View.test.tsx`.
- Fixtures: `fixtures/quran/` (see above); schemas: `docs/schemas/`.

## Naming Conventions

**Files:**

- `snake_case.rs`, one concept per file: `quran_reader.rs`, `quran_search_cache.rs`, `audit_bridge.rs`, `secret_store.rs`, `security_archive.rs`. Quran subsystems prefix with `quran_*` inside `application` (`quran.rs`, `quran_cli.rs`, `quran_doctor.rs`, `quran_forms.rs`, `quran_index.rs`, `quran_normalize.rs`, `quran_reader.rs`, `quran_search.rs`, `quran_tools.rs`); per-domain triplets follow the same shape (`hadith_reader.rs`/`hadith_search.rs`/`hadith_tools.rs`, `tafsir_*`, `scripture_*`, `rag_*`, `isnad_*`); graph services use `quran_graph_*`.
- Migrations: `NNNN_name.up.sql` zero-padded sequence (`0001_core.up.sql` … `0044_verse_links_scripture_source.up.sql`); `.down.sql` exists only for 0001–0006.
- Tests: `crates/<name>/tests/*.rs` integration suites; `*.trycmd` CLI acceptance (`crates/cli/tests/quran/read_flow.trycmd`); `fixtures/quran/adversarial/*`, `golden/references.jsonl` data files; frontend `*.test.tsx(x)` co-located with the view/component.
- Docs/tasks: `TASK-nnn-slug.md`, `phase-NN-slug/` dirs, `ADR-nnnn-title.md` under `docs/02-architecture/decisions/`, `YYYY-MM-DD-session-nnn.md` follow-ups.
- Web: `PascalCase.tsx` views/components (`ReadingView.tsx`, `AyahColumn.tsx`), `camelCase.ts` lib/client (`client.ts`, `tool-result.ts`); router exports `DIRECTORY`/`ROUTES` tables plus `parseReadPath`/`parseCitationId`.

**Directories:**

- `kebab-case` crate names matching the directory: `crates/quran-normalization/`, `crates/storage-sqlite/`, `crates/tool-registry/`, `crates/hadith-ingestion/`, `crates/isnad-graph/`. Module dirs mirror concepts: `crates/quran-core/src/reference/`, `crates/quran-normalization/src/rules/`, `docs/00-overview/` … `docs/10-operations/`, `web/src/views/`, `web/src/components/`.

## Where to Add New Code

**New Feature:**

- Primary code: `crates/application/src/` (new `quran_<area>.rs` service module, or `<domain>_reader/search/tools.rs` triplet for a new corpus, wired through `crates/application/src/lib.rs` and exposed via `crates/application/src/quran_cli.rs` free functions). Pure logic goes one layer down (`crates/quran-corpus/`, `crates/quran-normalization/`, `crates/quran-morphology/`, `crates/quran-search/`, `crates/quran-graph/`, or a foundation crate) so `application` stays orchestration-only.
- HTTP surface: handler in `crates/server/src/api.rs` (or `crates/server/src/routes/` for a new domain sub-router), always behind a backend trait with fakes in `crates/server/tests/`.
- Web surface: new view in `web/src/views/<Name>View.tsx` (+ co-located `<Name>View.test.tsx`), route entry in `ROUTES`/`DIRECTORY` plus a branch in `Router` (`web/src/router.tsx`), data via `web/src/api/client.ts`; shared logic in `web/src/lib/`, shared UI in `web/src/components/`.
- Tests: unit tests inline in the new module + integration suite under `crates/application/tests/` (run serial `-- --test-threads=1` if parallel flakes); CLI acceptance in `crates/cli/tests/quran/*.trycmd` when user-visible; contract tests vs fakes for HTTP.

**New Component/Module:**

- Implementation: new file in the owning crate's `src/` (e.g. `crates/quran-search/src/<thing>.rs`), re-exported from that crate's `lib.rs`. New Quran services follow the `quran_*` prefix in `crates/application/src/`. Never add a new workspace crate without registering its allowed edges in `xtask/allowlist.toml` (unlisted crates must have zero workspace deps) and adding it to `Cargo.toml [workspace] members`.
- Persistence: new repository trait in `crates/storage/src/` (+ row types), implementation in `crates/storage-sqlite/src/`; new schema change is a new `migrations/sqlite/NNNN_*.up.sql` (next free number from the directory, never from plan tables) + `checksums.json` entry — never edit an applied migration.
- New tool: versioned `SemVer` const + backend-trait method + registry impl in `crates/tool-registry/src/lib.rs`, application backend in the domain's `*_tools.rs`, dispatch coverage via the existing `POST /api/v1/quran/tool/{name}` route, typed form entry in `web/src/views/ToolsView.tsx`.

**Utilities:**

- Shared helpers: `crates/domain/src/` for pure types/guards (IDs, `Diagnostic`, hashing, `security_*`); `crates/testkit/src/` for dev-only fixtures shared across test suites (never a runtime dependency).

## Major Feature Map

**Canonical Quran engine:** reference grammar (`crates/quran-core/src/reference/`) → adapters/validation/import (`crates/quran-corpus/src/`) → activation (`crates/application/src/quran.rs`) → reader (`crates/application/src/quran_reader.rs`) → CLI `qai quran …` (`crates/cli/src/quran.rs`) / HTTP `/api/v1/quran/*` (`crates/server/src/api.rs`) / `ReadingView` + `ResearchView` + `SearchView` (`web/src/views/`).

**Derived text (normalization/search/morphology/counts):** rules/profiles (`crates/quran-normalization/src/`) → forms (`crates/application/src/quran_forms.rs`) → FTS5 index (`crates/application/src/quran_index.rs`) → search API (`crates/application/src/quran_search_api.rs` + `quran_search.rs`) → lexicon/counts (`quran_lexicon_api.rs` + `quran_counting.rs`, morphology service `quran_morphology.rs`) → `WordsView` (`web/src/views/WordsView.tsx`).

**Knowledge graph:** port (`crates/quran-graph/src/`) → build/staging (`crates/application/src/quran_graph_build.rs`) → SQLite snapshot store (`quran_graph_store.rs`) → annotations authority (`quran_graph_annotations.rs`) → export/doctor (`quran_graph_export.rs`, `quran_graph_doctor.rs`) → `qai graph …` + `/api/v1/quran/graph/*` + `GraphView` (`web/src/views/GraphView.tsx`).

**Hadith + isnad:** domain (`crates/hadith-core/src/`) → ingest (`crates/hadith-ingestion/src/`) → reader/search/tools (`crates/application/src/hadith_*.rs`) → narrator graph (`crates/isnad-graph/src/lib.rs` + `crates/application/src/isnad_*.rs`) → `qai hadith …` / `qai isnad …` (`crates/cli/src/hadith.rs`, `isnad.rs`) / hadith + gradings + isnad HTTP routes / `HadithView` + `IsnadView` (`web/src/views/`).

**Tafsir:** domain (`crates/tafsir/src/`) → reader/search/tools (`crates/application/src/tafsir_*.rs`) → `qai tafsir …` (`crates/cli/src/tafsir.rs`) / `/api/v1/tafsir/*` / `TafsirView` (`web/src/views/TafsirView.tsx`); record↔verse joins via `verse_link` + migration `0030`.

**Scripture + cross-domain:** domain (`crates/scripture/src/`) → reader/search/tools (`crates/application/src/scripture_*.rs`) → `qai scripture …` (`crates/cli/src/scripture.rs`) / sub-router (`crates/server/src/routes/scripture.rs`: editions/get/search/translations/parallel/research + cross-domain aliases) / `ScriptureView` (`web/src/views/ScriptureView.tsx`); aggregation in `knowledge_domain.rs` + `cross_domain.rs` (+ `disagreement.rs`).

**Multi-RAG:** types/config (`crates/rag/src/`) + ports (`retrieval`, `embeddings`, `reranking`) → reader/search/tools (`crates/application/src/rag_*.rs`) → `qai rag …` (`crates/cli/src/rag.rs`) / `rag.*` tool dispatch / `RagDebugView` (`web/src/views/RagDebugView.tsx`).

**Agentic tool runtime:** contract (`crates/tools/src/lib.rs`) → registry + backend traits (`crates/tool-registry/src/lib.rs`) → per-domain backends (`crates/application/src/*_tools.rs`, `quran_graph_tools.rs`) → `POST /api/v1/quran/tool/{name}` → `ToolsView` + `ExplorerView` (`web/src/views/ToolsView.tsx`, `ExplorerView.tsx`); agent crates themselves still placeholders.

## Special Directories

**`target/`:**

- Purpose: Cargo build artifacts.
- Generated: Yes
- Committed: No (gitignored).

**`web/dist/` + `web/node_modules/`:**

- Purpose: Vite build output + npm dependencies.
- Generated: Yes
- Committed: No (gitignored).

**`graft/` + `graphify-out/` + `.trail/`:**

- Purpose: Agent code-graph indexes for orientation (not source of truth).
- Generated: Yes (refresh with `graft build` / trail tooling after large changes)
- Committed: Partially (`graft/` yes; generated caches vary — check `.gitignore`)

**`.planning/`:**

- Purpose: GSD planning state + this codebase map.
- Generated: Partially (agent-written, human-reviewed)
- Committed: Yes

**`fixtures/quran/`:**

- Purpose: Synthetic test corpus (fake Arabic-like text for plumbing — never present as scripture) + synthetic labelled lexicon.
- Generated: No (hand-maintained test data)
- Committed: Yes

**`src/`, `tests/`, `scripts/` (repo root):**

- Purpose: Stubs containing only `README.md`; real code lives in `crates/`, real tests per crate, real runbooks in `docs/10-operations/` and `docs/runbooks/`.
- Generated: No
- Committed: Yes

**Placeholder crates (`crates/llm/`, `crates/mcp/`, `crates/agency/`, `crates/policy/`, …):**

- Purpose: Reserved names for future phases; each holds only a `//! Phase N` doc comment in `src/lib.rs`.
- Generated: No
- Committed: Yes — and must stay empty until their phase starts (implementing early breaks the scope fence and `arch-check`).

---

*Structure analysis: 2026-11-01*
