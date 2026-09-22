---
last_mapped_commit: 68b7ea6c1aacd1476d154550b2a6e9574614b6af
last_mapped_at: 2026-09-22
---
# Codebase Structure

**Analysis Date:** 2026-09-22

## Directory Layout

```
Q-ai/
├── crates/             # Rust workspace members (all real code lives here)
│   ├── application/    # Composition root — Quran + platform services
│   ├── domain/         # Pure shared vocabulary (no I/O)
│   ├── quran-core/     # Pure Quran domain (grammar, types, quotations)
│   ├── quran-corpus/   # Adapters, tokenizer, validator, differ, importer
│   ├── quran-normalization/ # Derived normalization rules/pipeline
│   ├── quran-search/   # FTS5 backend + search services
│   ├── citations/      # Citation resolver + quotation verification
│   ├── tools/          # ToolResult contract
│   ├── tool-registry/  # quran.get_ayah / quran.get_context
│   ├── storage/        # Persistence traits (ports)
│   ├── storage-sqlite/ # Sole storage adapter (sqlx/SQLite)
│   ├── provenance/     # ApprovalToken / CanonicalWriter
│   ├── audit/          # Hash-chained audit log
│   ├── jobs/           # Durable leased job queue
│   ├── sources/        # Manifests + lifecycle state machine
│   ├── config/         # Layered config + Secret<T>
│   ├── observability/  # tracing / metrics / opt-in OTLP
│   ├── server/         # Axum HTTP API v1
│   ├── cli/            # `qai` binary + doctor
│   ├── testkit/        # Dev-only fixtures (never ships)
│   └── <~30 placeholders>/ # Empty `//! Phase N` crates (graph, llm, rag, …)
├── xtask/              # Build gates: arch-check, migrate-check, ci, schema
├── migrations/sqlite/  # Append-only checksummed SQL (0001–0016 + checksums.json)
├── fixtures/quran/     # Synthetic test editions, adversarial + golden sets
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
- Contains: 13 implemented Phase-0/1/2 crates, `testkit`, `xtask` (as member), and ~30 empty placeholder crates reserving Phase 2–9 names.
- Key files: `Cargo.toml` (workspace root manifest), `crates/application/src/lib.rs`, `crates/cli/src/main.rs`

**`crates/application/`:**

- Purpose: Composition root — the only crate allowed to wire concrete backends together.
- Contains: Quran services (`quran*.rs`), DB orchestration (`db.rs`), audit bridge, job queue glue.
- Key files: `crates/application/src/lib.rs`, `crates/application/src/quran.rs`, `crates/application/src/quran_cli.rs`, `crates/application/src/quran_reader.rs`, `crates/application/src/db.rs`

**`crates/domain/` + `crates/quran-core/`:**

- Purpose: Pure, side-effect-free vocabulary layers.
- Contains: IDs, primitives, hashing, licensing, security guards, `Diagnostic` (`crates/domain/src/`); reference grammar, enums, editions, structure, quotations, views (`crates/quran-core/src/`, incl. `crates/quran-core/src/reference/`).
- Key files: `crates/domain/src/lib.rs`, `crates/domain/src/diagnostic.rs`, `crates/quran-core/src/reference/`, `crates/quran-core/src/quotation.rs`

**`crates/quran-corpus/` + `crates/quran-normalization/` + `crates/quran-search/`:**

- Purpose: Canonical pipeline (adapters → audit → tokenize → validate → import → activate) and the derived normalization/search layer.
- Contains: `adapters.rs`, `unicode.rs`, `tokenize.rs`, `hashing.rs`, `validation.rs`, `differ.rs`, `import.rs` (`quran-corpus`); `rules/`, `pipeline.rs`, `span.rs`, `profile.rs`, `trace.rs` (`quran-normalization`); `fts5.rs`, `tokenizer.rs`, `index.rs`, `model.rs`, `regex.rs`, `highlight.rs` (`quran-search`).
- Key files: `crates/quran-corpus/src/import.rs`, `crates/quran-corpus/src/validation.rs`, `crates/quran-search/src/fts5.rs`

**`crates/storage/` + `crates/storage-sqlite/`:**

- Purpose: Persistence port (traits) + sole SQLite adapter.
- Contains: `lib.rs` (`Database`/`ReadTx`/`UnitOfWork`), `repository.rs`, `quran.rs`, `workflows.rs`, `error.rs` (`storage`); `lib.rs` (pools), `migrate.rs`, `quran.rs` (`storage-sqlite`).
- Key files: `crates/storage/src/repository.rs`, `crates/storage-sqlite/src/migrate.rs`

**`crates/cli/` + `crates/server/`:**

- Purpose: Thin edge surfaces; no business logic beyond routing/rendering.
- Contains: clap tree + `dispatch()` + `quran.rs` + `doctor.rs` + `exit_code.rs` + `main.rs` (`cli`); `api.rs` (routes + envelope) + `lib.rs` (loopback guard) (`server`).
- Key files: `crates/cli/src/lib.rs`, `crates/cli/src/doctor.rs`, `crates/server/src/api.rs`

**`xtask/`:**

- Purpose: Build-time enforcement and developer tooling (invoked as `cargo xtask <cmd>`).
- Contains: `arch.rs` (dependency-direction gate), `migrate.rs` (checksum gate), `ci.rs` (9-step gate), `schema.rs`, `adr.rs`, `coverage.rs`.
- Key files: `xtask/src/main.rs`, `xtask/src/arch.rs`, `xtask/allowlist.toml`

**`migrations/sqlite/`:**

- Purpose: Append-only, checksummed schema evolution; workspace at schema v16.
- Contains: `0001_core` … `0016_quran_search_cache` `.up.sql` files (+ `.down.sql` for 0001–0006 only), `checksums.json`.
- Key files: `migrations/sqlite/0007_quran_editions.up.sql`, `migrations/sqlite/0011_quran_staging.up.sql`, `migrations/sqlite/checksums.json`

**`fixtures/quran/`:**

- Purpose: Synthetic (never real scripture) test data for plumbing, adversarial, and golden tests.
- Contains: `test-edition-min/` (+ `test-edition-min-v2.json`), `test-translation-min.json`, `test-gloss-min.json`, `adversarial/` (×16, one per QV rule), `golden/references.jsonl` (331 reference cases).
- Key files: `fixtures/quran/test-edition-min/manifest.json`, `fixtures/quran/golden/references.jsonl`

**`docs/`:**

- Purpose: Numbered documentation tree; reading order matters (see `AGENTS.md`).
- Contains: `00-overview/` (incl. `agent-briefing.md`), `01-requirements/requirements.md` (PRD), `02-architecture/decisions/` (ADRs incl. `crate-map.md`), `03-plan/phases/` (phase boards), `05-followups/`, `06-progress/`, `07-technical/`, `08-api/quran-v1-openapi.json`, `09-testing/`, `10-operations/`, `schemas/`, plus `archive/`, `plans/`, `runbooks/`.
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
- `crates/server/src/lib.rs`: `loopback_addr()` bind guard + `ServerError`; routes live in `crates/server/src/api.rs`.
- `xtask/src/main.rs`: `cargo xtask <cmd>` command implementations.

**Configuration:**

- `Cargo.toml`: Workspace manifest — member list, workspace lints (`unsafe_code = "forbid"`), centralized `[workspace.dependencies]`.
- `rust-toolchain.toml`: Pinned toolchain (`1.97.1` + `rustfmt`/`clippy`).
- `.cargo/config.toml`: `xtask = "run --quiet --package xtask --"` alias.
- `xtask/allowlist.toml`: Dependency-direction rules enforced by `arch-check` (fail-closed).
- `crates/config/src/lib.rs`: Layered loader (CLI > env > file > defaults) + validation.
- `clippy.toml`, `rustfmt.toml`, `deny.toml`: Lint, format, and supply-chain policy.
- `.env.example`: Documents required env vars (existence only; never commit real `.env`).

**Core Logic:**

- `crates/quran-corpus/src/import.rs`: 13-checkpoint staged importer (`run_import`).
- `crates/application/src/quran.rs`: `QuranImportHandler` + approval-gated `activate_edition`/`rollback_edition`.
- `crates/application/src/quran_reader.rs`: `QuranReader` deterministic lookup + LRU cache.
- `crates/quran-normalization/src/pipeline.rs`: Normalization pipeline over profiles/rules.
- `crates/quran-search/src/index.rs` + `crates/quran-search/src/fts5.rs`: `FullTextIndex` port + FTS5 adapter.
- `crates/citations/src/lib.rs`: Citation resolver + quotation verification.
- `crates/provenance/src/lib.rs`: `ApprovalToken` + `CanonicalWriter` trust root.

**Testing:**

- Per-crate integration suites: `crates/storage-sqlite/tests/` (13 suites), `crates/application/tests/`, `crates/cli/tests/quran/read_flow.trycmd` (trycmd acceptance), `crates/server/tests/` (API contract vs fakes), `crates/testkit/tests/` (security/config).
- Unit + property tests inline: `crates/*/src/` (`#[cfg(test)]` modules, proptest for grammar/tokenizer/span-map).
- Fixtures: `fixtures/quran/` (see above); schemas: `docs/schemas/`.

## Naming Conventions

**Files:**

- `snake_case.rs`, one concept per file: `quran_reader.rs`, `quran_search_cache.rs`, `audit_bridge.rs`, `secret_store.rs`, `security_archive.rs`. Quran subsystems prefix with `quran_` inside `application` (`quran.rs`, `quran_cli.rs`, `quran_doctor.rs`, `quran_forms.rs`, `quran_index.rs`, `quran_normalize.rs`, `quran_reader.rs`, `quran_search.rs`, `quran_tools.rs`).
- Migrations: `NNNN_name.up.sql` zero-padded sequence (`0001_core.up.sql` … `0016_quran_search_cache.up.sql`); `.down.sql` exists only for 0001–0006.
- Tests: `crates/<name>/tests/*.rs` integration suites; `*.trycmd` CLI acceptance (`crates/cli/tests/quran/read_flow.trycmd`); `fixtures/quran/adversarial/*`, `golden/references.jsonl` data files.
- Docs/tasks: `TASK-nnn-slug.md`, `phase-NN-slug/` dirs, `ADR-nnnn-title.md` under `docs/02-architecture/decisions/`, `YYYY-MM-DD-session-nnn.md` follow-ups.

**Directories:**

- `kebab-case` crate names matching the directory: `crates/quran-normalization/`, `crates/storage-sqlite/`, `crates/tool-registry/`. Module dirs mirror concepts: `crates/quran-core/src/reference/`, `crates/quran-normalization/src/rules/`, `docs/00-overview/` … `docs/10-operations/`.

## Where to Add New Code

**New Feature:**

- Primary code: `crates/application/src/` (new `quran_<area>.rs` service module, wired through `crates/application/src/lib.rs` and exposed via `crates/application/src/quran_cli.rs` free functions). Pure logic goes one layer down (`crates/quran-corpus/`, `crates/quran-normalization/`, `crates/quran-search/`, or a foundation crate) so `application` stays orchestration-only.
- Tests: unit tests inline in the new module + integration suite under `crates/application/tests/` (run serial `-- --test-threads=1` if parallel flakes); CLI acceptance in `crates/cli/tests/quran/*.trycmd` when user-visible.

**New Component/Module:**

- Implementation: new file in the owning crate's `src/` (e.g. `crates/quran-search/src/<thing>.rs`), re-exported from that crate's `lib.rs`. New Quran services follow the `quran_*` prefix in `crates/application/src/`. Never add a new workspace crate without registering its allowed edges in `xtask/allowlist.toml` (unlisted crates must have zero workspace deps) and adding it to `Cargo.toml [workspace] members`.
- Persistence: new repository trait in `crates/storage/src/` (+ row types), implementation in `crates/storage-sqlite/src/`; new schema change is a new `migrations/sqlite/NNNN_*.up.sql` (next free number from the directory, never from plan tables) + `checksums.json` entry — never edit an applied migration.

**Utilities:**

- Shared helpers: `crates/domain/src/` for pure types/guards (IDs, `Diagnostic`, hashing, `security_*`); `crates/testkit/src/` for dev-only fixtures shared across test suites (never a runtime dependency).

## Special Directories

**`target/`:**

- Purpose: Cargo build artifacts.
- Generated: Yes
- Committed: No (gitignored).

**`graft/`:**

- Purpose: Agent code-graph index (orientation, not source of truth).
- Generated: Yes (refresh with `graft build` after large changes)
- Committed: Yes

**`.planning/`:**

- Purpose: GSD planning state + this codebase map.
- Generated: Partially (agent-written, human-reviewed)
- Committed: Yes

**`fixtures/quran/`:**

- Purpose: Synthetic test corpus (fake Arabic-like text for plumbing — never present as scripture).
- Generated: No (hand-maintained test data)
- Committed: Yes

**`src/`, `tests/`, `scripts/` (repo root):**

- Purpose: Stubs containing only `README.md`; real code lives in `crates/`, real tests per crate, real runbooks in `docs/10-operations/` and `docs/runbooks/`.
- Generated: No
- Committed: Yes

**Placeholder crates (`crates/graph/`, `crates/llm/`, `crates/rag/`, `crates/tafsir/`, `crates/api/`, `crates/tui/`, …):**

- Purpose: Reserved names for Phases 2–9; each holds only a `//! Phase N` doc comment in `src/lib.rs`.
- Generated: No
- Committed: Yes — and must stay empty until their phase starts (implementing early breaks the scope fence and `arch-check`).

---

*Structure analysis: 2026-09-22*
