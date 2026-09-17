# Q-ai

A local-first research platform for the Quran, hadith, Islamic literature, and comparative scripture, built in Rust.

Q-ai puts exact, attributable source text before generated interpretation. The current implementation is a **developer-stage canonical Quran engine with a CLI, a loopback HTTP API, and an emerging normalization/search layer**—not yet the complete research product described in the [PRD](docs/01-requirements/requirements.md).

## Current state

**Reviewed: 2026-09-17.** Implementation progress and formal phase acceptance are tracked separately.

| Area | Implemented today | Remaining / limitations |
|---|---|---|
| Foundations | SQLite repositories, checksummed migrations and backups, provenance, hash-chained audit, jobs, configuration, secret backends, security guards, observability, diagnostic CLI | Phase-0 sign-off and remaining acceptance evidence are pending |
| Canonical Quran core | Edition/version models, reference parsing, JSON/CSV adapters, validation, lossless tokenization, hashing, staged import, approval-gated activation/rollback, exact lookup and bounded context | Approved dataset, licensing/editorial decisions, independent reference-corpus verification and full-edition acceptance runs remain open |
| Reading and citations | Attributed translations and word glosses, edition-pinned quotations, citation resolution, CLI reads, HTTP read API and a labelled RTL debug reader | No production Web GUI or TUI; debug-reader font/licensing work remains open |
| Normalization and search | Versioned normalization rules/profiles and offset traces; derived forms; SQLite FTS5 index builds; Rust search services for exact, normalized, phrase, concatenated/cross-ayah and bounded regex search | Search-query CLI/HTTP/SSE wiring, morphology, word families, linguistic review and phase-wide hardening remain unfinished |
| Broader research platform | Workspace placeholders and design documents | Knowledge graphs, hadith/tafsir/scripture ingestion, multi-RAG, model providers, agent runtime, MCP, full Web/TUI and multi-user deployment are planned, not available |

The bundled `test-edition-min` dataset is **synthetic test data, not Quran text and not suitable for religious research**. A successful fixture import or test run is not editorial approval of a real corpus.

### Phase tracking

| Phase | State | Evidence |
|---|---|---|
| P0 — Foundations & Provenance | Implementation complete; formal sign-off pending | [Completion ledger](docs/03-plan/phases/phase-00-foundation/done.md), [acceptance](docs/03-plan/phases/phase-00-foundation/acceptance.md) |
| P1 — Canonical Quran Core | In progress; implementation and fixture verification substantially landed, source decisions and exit gates open | [Status](docs/03-plan/phases/phase-01-core/STATUS.md), [tasks](docs/03-plan/phases/phase-01-core/tasks.md) |
| P2 — Quran Search, Normalization & Linguistics | In progress; normalization, indexing and search-service core landed; no phase exit acceptance recorded | [Completion ledger](docs/03-plan/phases/phase-02-rag/done.md), [execution plan](docs/03-plan/phases/phase-02-rag/execution-plan.md) |
| Later work | Planning / placeholders | [Knowledge-graph proposal](docs/03-plan/phases/phase-04-quran-graph/README.md), [PRD](docs/01-requirements/requirements.md) |

Phase directory names retain legacy numbering: `phase-02-rag` now contains search/linguistics, `phase-03-server` is a placeholder, and the graph proposal lives under `phase-04-quran-graph`. The PRD roadmaps and these paths are not fully reconciled. `current-plan.md` is empty and the global progress status is older than the phase ledgers; use the linked task boards and completion evidence for current work.

## Quick start

Run from the repository root with rustup and a native Rust build toolchain. [rust-toolchain.toml](rust-toolchain.toml) pins **Rust 1.97.1** with rustfmt and Clippy. No LLM, vector database, external database server, or remote model credentials are needed for this demo.

```bash
cargo build -p cli --bin qai
./target/debug/qai --help
```

Use a fresh, dedicated directory so the synthetic edition cannot replace an existing research corpus. The following commands create a local SQLite database and explicitly approve activation of the test fixture:

```bash
export QAI_DATA_DIR="$(mktemp -d)"
./target/debug/qai db migrate
./target/debug/qai quran import fixtures/quran/test-edition-min/manifest.json
./target/debug/qai quran activate test-edition-min@0.1.0 --yes
./target/debug/qai quran get 1:1 --json
./target/debug/qai quran context 2:1 --before 1 --after 1
./target/debug/qai quran edition show test-edition-min@0.1.0 --statistics --hashes
./target/debug/qai doctor --quran --deep --json
```

Keep the same `QAI_DATA_DIR` in subsequent shells to reuse this database. Doctor is read-only and can report warnings or failures for incomplete configuration or missing real-corpus verification; it is not a release-certification command.

### Normalization and indexing

After the demo import and activation:

```bash
./target/debug/qai quran normalize --list-profiles
./target/debug/qai quran normalize --show-rule N06
./target/debug/qai quran forms rebuild test-edition-min@0.1.0
./target/debug/qai quran index rebuild
./target/debug/qai quran index verify
```

The current full-text backend is **SQLite FTS5**, not the Tantivy adapter proposed in the phase plan. Search services exist in Rust, but `qai quran search` and search HTTP endpoints are not yet exposed. Top-level `secret`, `source`, `job`, `audit`, and `completions` commands are also placeholders; their presence in `--help` does not imply working workflows.

### Local API

With the same initialized data directory:

```bash
./target/debug/qai serve --bind 127.0.0.1:8737
```

In another terminal:

```bash
curl http://127.0.0.1:8737/api/v1/quran/editions
curl http://127.0.0.1:8737/api/v1/quran/ayahs/1:1
```

The server provides `/healthz`, `/readyz`, Quran read/citation routes, normalization preview/profile routes, and `/debug/read/test-edition-min@0.1.0/1`. It rejects non-loopback binding; this is not a production multi-user service. See the [Quran v1 OpenAPI document](docs/08-api/quran-v1-openapi.json) and [route implementation](crates/server/src/api.rs).

## Architecture and integrity

- **Canonical data is structured:** editions, surahs, ayahs, segments and tokens—not only retrieval chunks.
- **Canonical text and derived data stay separate:** normalization operates on derived forms, while quotations retain edition/version/hash identity.
- **Imports do not activate themselves:** staging, validation, explicit approval, audit and an atomic active-edition switch separate ingestion from visibility.
- **Lookup is deterministic and model-free:** canonical reading does not depend on an LLM or vector store.
- **Translations and annotations retain attribution:** they must not be presented as the original Quran or as unqualified scholarly consensus.
- **Local-first defaults:** telemetry is opt-in. Exporter redaction gaps and other unresolved decisions are tracked in [open questions](docs/05-followups/open-questions.md); do not treat the current build as production-hardened.

| Location | Role |
|---|---|
| `crates/domain`, `config`, `provenance`, `audit`, `sources`, `jobs`, `observability` | Foundation types and services |
| `crates/storage`, `storage-sqlite` | Repository contracts and SQLite implementation |
| `crates/quran-core`, `quran-corpus`, `citations` | Canonical models, import/validation and citation identity |
| `crates/quran-normalization`, `quran-search` | Derived normalization, indexing and search |
| `crates/application`, `tools`, `tool-registry` | Composition, use cases and deterministic tool contracts |
| `crates/cli`, `server` | `qai` executable and local HTTP interface |
| `crates/testkit`, `crates/*/tests`, `fixtures` | Shared fixtures, integration/property/CLI tests and synthetic datasets |
| `migrations/sqlite` | Append-only schema migrations, currently `0001`–`0016` |
| `xtask` | Architecture, migration, schema, coverage and CI tooling |

Other workspace crates may be empty placeholders. The root `src/`, `scripts/`, and `tests/` directories are not the implementation, tooling, or test entry points.

## Development and verification

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask arch-check
cargo xtask migrate-check
```

The extended `cargo xtask ci` gate also uses dependency-audit tooling; inspect [xtask](xtask/src/ci.rs) and the [contribution guide](CONTRIBUTING.md) before running it. Historical passing test counts in phase documents are evidence for those sessions, not a guarantee about the current working tree.

See [CONTRIBUTING.md](CONTRIBUTING.md) for integrity, migration and review requirements, and [AGENTS.md](AGENTS.md) for agent workflow rules.

## Documentation

- [Product requirements / PRD](docs/01-requirements/requirements.md)
- [Architecture decisions](docs/02-architecture/decisions/)
- [Phase plans and work boards](docs/03-plan/phases/)
- [Task completion rollup](docs/06-progress/task-done-rollup.md)
- [Open follow-ups](docs/05-followups/followups.md)
- [Corpus architecture](docs/07-technical/quran-corpus-architecture.md)
- [Adapter authoring](docs/07-technical/quran-adapter-authoring.md)
- [Citation specification](docs/07-technical/quran-citation-spec.md)
- [Import runbook](docs/10-operations/quran-import-runbook.md)
- [Rollback runbook](docs/10-operations/quran-rollback-runbook.md)

The numbered `docs/` tree runs from `00-overview` through `10-operations`; some overview and planning files remain scaffolds. Plans describe intended behavior; code, tests and recorded acceptance evidence determine what is available now.
