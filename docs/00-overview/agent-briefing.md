# Q-ai Agent Briefing

**Audience:** an AI coding agent that will develop, verify, and orchestrate work on this repository.
**Reviewed:** 2026-09-17. Code, tests, and recorded acceptance evidence are the source of truth;
plans describe intent. When this file and a phase ledger disagree, trust the ledger, then fix this file.

Start every session with §1. Use §15 as your orchestration checklist.

---

## 1. Session startup (do this first, every time)

1. Read `AGENTS.md` (workflow, naming, completion flow).
2. Read this briefing (`docs/00-overview/agent-briefing.md`).
3. Check `git status --short` and `git log --oneline -5` — **concurrent writers are common in this
   tree** (see §14.8). Know what is dirty before you touch anything.
4. Read the active phase board: `docs/03-plan/phases/phase-01-core/tasks.md` /
   `phase-02-rag/tasks.md`, plus `STATUS.md` / `done.md` for evidence.
5. Check `docs/05-followups/open-questions.md` and `docs/05-followups/followups.md` for open gates.
6. Orient with graft before grepping: `graft map` cold, `graft ask "<question>" --source` to locate,
   `graft grep "<literal>"` when you need every occurrence, `graft skeleton <file>` for an API
   surface, `graft callers <symbol> --depth 2` before changing any signature.

Useful index files: `graft/INDEX.md` (code graph), `docs/02-architecture/decisions/crate-map.md`,
`README.md` (verified runnable demo), `CONTRIBUTING.md` (gates).

---

## 2. What Q-ai is

**Q-ai** is a local-first research platform for the Quran, hadith collections, Islamic literature,
and comparative scripture (Torah, Tanakh, New Testament). PRD: `docs/01-requirements/requirements.md`
(v0.3.2, Draft/Living Document, ~7000 lines).

- **Language:** Rust (workspace, `resolver = "2"`, toolchain pinned to **1.97.1** in
  `rust-toolchain.toml`; `unsafe_code = "forbid"` workspace-wide).
- **Interfaces:** Web GUI, TUI, CLI (`qai`), REST API, streaming API, agent tools, MCP — all powered
  by one core engine.
- **Deployment:** local-first + server. Telemetry is opt-in; local data stays local unless a remote
  provider is explicitly enabled.
- **Architecture:** Canonical Quran Engine + Knowledge Graph + Multi-RAG + Agentic Tool Runtime.

Knowledge domains (`./agent/project-context.md`): (1) Quran — canonical text, morphology,
translations, tafsir, graph; (2) Hadith — Shia collections, isnads, narrators, gradings, graph;
(3) Islamic literature — theology, fiqh, history, rijal, sira, dictionaries; (4) comparative
scripture; (5) user collections.

The load-bearing idea: **the Quran is not a bag of RAG chunks.** It has a dedicated lossless,
structured, hash-verified corpus engine with stable addressing. Everything downstream (search,
graph edges, tafsir links, agent reports, citations) inherits its correctness — so the corpus
gate is byte-equality, not plausibility. Hadith/tafsir/scripture may use RAG, FTS, and graphs
according to their structure.

Current reality (2026-09-17): a **developer-stage canonical Quran engine** — CLI + loopback HTTP
API + emerging normalization/search layer. Web GUI, TUI, graphs, hadith, multi-RAG, LLM
providers, and agent runtime are planned, not built. Details in §13.

---

## 3. Non-negotiable principles

From PRD §2 (19 principles) and `.agent/coding-rules.md`. Violating these is a defect, not a
trade-off:

1. Exact text before generated interpretation.
2. Every factual claim traceable (provenance + citations).
3. Quran stored structurally, not only as RAG chunks.
4. Canonical text separated from translations and annotations.
5. Translations never presented as the original Quran (enforced: `translation_editions.translator`
   `NOT NULL` + CHECK, `AyahView` has no translation-as-canonical variant).
6. Disputed claims labeled disputed; schools represented without silent merging.
7. Hadith grading attributed to a source/scholar.
8. Search normalization never modifies displayed canonical text (derived copies only).
9. Imported internet content is untrusted until validated.
10. No model may fabricate a verse, hadith, chain, grading, or citation (typed no-fabrication errors;
    canonical path has zero model dependencies — compile-time property via `arch-check`, I2).
11. Answers distinguish quotation vs. source summary vs. AI analysis.
12. Research reproducible (hashes, generations, manifests, checksums).
13. Local-first; deny-by-default permissions/network/filesystem/side effects.
14. Religious conclusions never falsely presented as scholarly consensus; Q-ai assists research, does
    not issue rulings or claim religious authority.

Structural enforcements: canonical rows writable **only** through `CanonicalWriter` + `ApprovalToken`
(`crates/provenance`); importer holds no token and cannot activate; `domain` has no I/O or async
runtime; nothing that mutates data runs inside `doctor`; secrets live only in `Secret<T>`.

---

## 4. PRD map (how to navigate `requirements.md`)

| PRD area | Sections | Notes |
|---|---|---|
| Vision, principles, goals, non-goals | §1–§4 | Non-goals rule out fatwas, sectarian endorsements, foundation models |
| Quran engine | §7 (model, hierarchy, addressing §7.4), §13, §35.1 (validation) | Core of P1 |
| Search/linguistics | §8–§9, §11–§12 | Core of P2; §11.4 tool contract, §12.1 reproducibility |
| Sources/editions | §22 (manifests), §38–§39 (licensing, pinning), §82 | Source state machine |
| Jobs | §34, §41, §54 | Durable, leased, cancellable |
| Integrity | §46, §56, §93 | Trust layers, hash chains |
| CLI/API/doctor | §25–§27, §50 | Exit codes, `--json`, read-only doctor |
| Security | §25.13, §37, §74, §84, §92 | Taint, SSRF, archive guards |
| Provenance/audit | §39, §76, §85 | Generations, approval-gated activation |
| Acceptance/DoD | §58 | PRD-level definition of done |
| Crate inventory | §87 | All workspace crates incl. placeholders |
| Roadmap | §43 **+ a second phase list** | ⚠️ Two divergent roadmaps in one file (§43 = 11 phases; later list = 13 phases with different P10–P12). Do not renumber phases on your own; record the conflict as a follow-up |

---

## 5. Architecture

### 5.1 Layering (enforced by `cargo xtask arch-check` vs `xtask/allowlist.toml`)

```text
domain  ←  application  →  audit / provenance / jobs / sources
  │              │                       │
  │        quran-core → quran-corpus → citations / storage-sqlite / server / cli
  │              │         │    │
  │     quran-normalization / quran-search (derived layer; never touches canonical rows)
  └──────────────┴─────────┴──→ storage (traits) + storage-sqlite ──→ observability
                                            │
                                   cli / server (edges only)
```

Rules: `domain` → (serde, thiserror, time, uuid) only, no I/O/async. `quran-core` stays pure
(domain, serde, thiserror, unicode-*; **no** llm/embeddings/retrieval/vector crates — invariant I2).
`quran-corpus`/`quran-normalization`/`quran-search` likewise never gain model/vector dependencies
(I2/I8). `application` is the composition root. `storage-sqlite` hides behind the application
boundary. Placeholders must stay empty until their phase starts.

### 5.2 Decisive ADRs (`docs/02-architecture/decisions/`)

- **ADR-0001** SQLite + sqlx, PostgreSQL-portable SQL; dual pools (write `max_connections=1`,
  read `query_only`), WAL, `foreign_keys=ON`, `busy_timeout`. Backup via `VACUUM INTO`, never
  `fs::copy`.
- **ADR-0002** Migrations append-only + checksummed (`migrations/sqlite/NNNN_*.up.sql`,
  `checksums.json`); editing an applied migration is a coded failure (`QAI-DB-0003`).
- **ADR-0003** Durable DB-backed leased job queue, no external broker; cancellable, resumable,
  idempotent.
- **ADR-0004/0005** Layered config (CLI > env > file > defaults) with `ValueOrigin`; secrets in
  `Secret<T>` (env / keychain / age file), redacted everywhere (`***`), never in SQLite as values.
- **ADR-0006/0108** SHA-256 + canonical JSON + NFC policy; frozen `text_hash`/`structure_hash`/
  `token_order_hash` recipes with algorithm tags.
- **ADR-0007** Source manifests v1, ed25519 detached signatures over canonical JSON.
- **ADR-0008/0009** Universal `ProvenanceRecord` + `ApprovalToken`/`CanonicalWriter`; append-only
  hash-chained audit log.
- **ADR-0010** Typed `Diagnostic` errors, unique code registry, public CLI exit codes
  (0/1/2/3/4/5/6/7/70).
- **ADR-0011** tracing + metrics catalog, OTLP opt-in only (known gap: span-field scrubbing, §14.7).
- **ADR-0102–0114** Quran addressing grammar, numbering, Unicode, tokenization, storage layout,
  activation, hashing, diffing, basmala, citations, translations, caching, reference corpus.
  ADR-0101 (dataset/license) and ADR-0114 (reference corpus) are still **Draft** — the hard gates.
- **ADR-0203** Quran morphology dataset (Draft, created 2026-09-18): edition-relative
  morphology + alignment; provider/licence unselected. Multi-edition architecture and
  verified upstream facts: `docs/02-architecture/upstream-sources.md`,
  `docs/05-followups/owner-decisions.md`.
- **ADR-0201** Full-text engine: plan says Tantivy; **shipped FTS5** (DEV-05, offline Cargo cache had
  no Tantivy). Treat FTS5 as current truth; ratification is an owner decision.
- **ADR-0202** Graph store (Proposed): SQLite adjacency + bounded CTEs default; CozoDB and
  sqlite-graph as spikes. SQLite stays source of truth; backends are rebuildable projections.

---

## 6. Codebase map

Workspace root `Cargo.toml` lists all members. Toolchain: `rustc 1.97.1`, `cargo xtask <cmd>` alias in
`.cargo/config.toml`.

| Crate | Role | Key files |
|---|---|---|
| `domain` | IDs, types, hashing, licensing, security guards (path/archive/net/input/sanitize) | `src/security*.rs`, `src/hashing.rs`, `src/diagnostic.rs` |
| `config` | Layered loader, `ValueOrigin`, `Secret<T>`, validation | `src/lib.rs`, `src/secret*.rs` |
| `storage` | Repository traits + `StorageError` | `src/quran.rs`, `src/repository.rs` |
| `storage-sqlite` | Real repos, migration runner, checksums, backup | `src/lib.rs`, `src/migrate.rs`, `tests/` (13 suites) |
| `provenance` | Record model, `ApprovalToken`, `CanonicalWriter` | `src/lib.rs` |
| `audit` | Hash-chain writer/verifier, redaction | `src/lib.rs` |
| `sources` | Manifest parser, lifecycle state machine, genealogy | `src/lib.rs`, `src/registry.rs` |
| `jobs` | Types, `JobStore`, worker pool | `src/lib.rs`, `src/worker.rs`, `src/queue.rs` |
| `observability` | Subscriber, spans, metric catalog, OTLP | `src/lib.rs`, `src/metrics.rs` |
| `quran-core` | Pure domain: newtypes, enums, reference grammar, `QuranQuotation`, views | `src/reference/`, `src/quotation.rs`, `src/edition.rs` |
| `quran-corpus` | Adapters, tokenizer, Unicode auditor, QV validator, differ, 13-checkpoint importer | `src/import.rs`, `src/validation.rs`, `src/adapters.rs`, `src/tokenize.rs`, `src/hashing.rs`, `src/differ.rs` |
| `quran-normalization` | Rules N01–N22, profiles L0–L8, `SpanMap`, pipeline, traces | `src/rules/`, `src/pipeline.rs`, `src/span.rs`, `src/profile.rs` |
| `quran-search` | FTS5 backend, tokenizers, search services, highlighting, cache | `src/fts5.rs`, `src/tokenizer.rs`, `src/index.rs`, `src/regex.rs`, `src/highlight.rs` |
| `citations` | Resolver, `StoredCitation`, deep links/URNs | `src/lib.rs` |
| `tools`, `tool-registry` | `ToolResult`/`ReproducibilityData`, `quran.get_ayah`/`quran.get_context` | `tools/src/lib.rs`, `tool-registry/src/lib.rs` |
| `application` | Composition root, `quran_cli`, reader/tools/normalize services, audit bridge | `src/quran_cli.rs`, `src/db.rs` |
| `server` | Axum API v1, debug reader, loopback-only serve | `src/api.rs`, `src/lib.rs` |
| `cli` | `qai` binary, command tree, 45-check doctor | `src/lib.rs`, `src/quran.rs`, `src/doctor.rs` |
| `testkit` | Fixtures + security/config suites | `src/lib.rs`, `tests/` |
| `xtask` | arch/migrate/schema/coverage/adr/ci tooling | `src/*.rs`, `allowlist.toml` |
| Placeholders | `graph`, `quran-graph`, `quran-morphology`, `isnad-graph`, `hadith-*`, `tafsir`, `scripture`, `ingestion`, `retrieval`, `rag`, `embeddings`, `reranking`, `llm`, `model-router`, `agent-*`, `agency`, `conversations`, `tool-sdk`, `tool-sandbox`, `policy`, `approvals`, `workflows`, `memory`, `mcp`, `evaluation`, `api`, `tui` | Empty `//! Phase N` docs — **do not implement outside their phase** |

Non-crate paths: root `src/`, `scripts/`, `tests/` are **stubs with READMEs only** (real code is in
`crates/`). Fixtures: `fixtures/quran/` (`test-edition-min` synthetic, `test-edition-min-v2.json`,
`test-gloss-min.json`, `test-translation-min.json`, `adversarial/` ×16, `golden/references.jsonl`
331 cases). Schemas: `docs/schemas/` (edition source, doctor v1). Migrations: `migrations/sqlite/`.

---

## 7. Data model & migrations (workspace at schema v16)

| Migration | Contents |
|---|---|
| `0001_core` | `schema_migrations`, `principals`, `workspaces`, `settings`, `blobs` |
| `0002_sources` | sources, versions, files, genealogy, state transitions, approvals |
| `0003_provenance` | `provenance_records` (+ immutability triggers), `review_queue` |
| `0004_jobs` | `jobs`, `job_events` |
| `0005_audit` | `audit_events` (+ append-only triggers) |
| `0006_outbox_generations_tombstones` | `corpus_generations`, `outbox_events`, `tombstones` |
| `0007_quran_editions` | `quran_editions` (+ immutability trigger), `quran_active_edition` singleton + `corpus_generation` |
| `0008_quran_structure` | surahs, ayahs, tokens, separators, segments (+ insert-only triggers) |
| `0009_quran_divisions` | juz/hizb/rub/manzil/ruku/page/sajdah + range index |
| `0010_quran_translations` | `translation_editions` (`translator` NOT NULL + CHECK), passages, word glosses |
| `0011_quran_staging` | `quran_stg_*` mirrors + `import_run_id`, no immutability triggers, cascade deletes |
| `0012_quran_validation` | `validation_reports`, `difference_reports`, `citations` |
| `0013_quran_normalization` | rules, profiles (+ append-only trigger raising `QAI-NORM-0003`, not `0001`) |
| `0014_quran_forms` | `quran_token_forms`, `quran_ayah_forms`, `quran_skeletons` |
| `0015_quran_indexes` | `index_pointers`, `index_build_runs` |
| `0016_quran_search_cache` | generation-keyed search cache (shape differs from plan — DEV-08) |

Rules: append-only, checksummed, never edit applied files; canonical tables forward-only
(deactivation, not deletion); every cache/derived index records the `corpus_generation` it was
built from; activation flips the singleton pointer in one transaction.

---

## 8. Canonical Quran pipeline (Phase 1)

1. **Adapters** (`quran-corpus/src/adapters.rs`): `EditionAdapter` trait; JSON + CSV adapters
   (CSV reproduces the JSON manifest exactly). Schema: `docs/schemas/quran-edition-source.v1.schema.json`.
2. **Unicode auditor** (`src/unicode.rs`): normalization form, forbidden code points, expected blocks.
3. **Tokenizer** (`src/tokenize.rs`): whitespace-preserving, exact separators, grapheme/byte offsets;
   proptest `reconstruct(tokens, separators) == text`.
4. **Hashing** (`src/hashing.rs` + ADR-0108): frozen `text_hash` / `structure_hash` / `token_order_hash`.
5. **Validator** (`src/validation.rs`): rules **QV-001…QV-028** (counts, identifiers, Unicode, token
   order, checksums, round-trip, reference comparison). 16 adversarial fixtures each reject with
   their specific rule id. QV-015 (reference-corpus comparison) is a recorded skip until ADR-0114.
6. **Importer** (`src/import.rs::run_import`): **13 checkpoints**, deterministic restart-is-resume,
   cancellation with staging cleanup, `stop_after` dry-run/chaos support. Ends at
   `ApprovalRequested` — it **cannot activate** (holds no `ApprovalToken`).
7. **Activation/rollback** (`application` services): human-gated (`--yes`), same-transaction audit,
   atomic pointer flip + generation bump; `quran_edition_v1` char-level differ + persisted
   difference reports.
8. **Reader** (`application::quran_reader`): ayah/range/surah/division/token expansion,
   structure-bounded context with caps, generation-keyed LRU cache (no stale text), typed errors.
9. **Translations/glosses**: attributed, verse-level (+ optional word-level), structural alignment;
   served only on request, never as canonical.
10. **Citations** (`crates/citations`): resolve/resolve_stored, verdicts
    (ExactMatch/…/Mismatch/LocationNotFound/EditionNotFound/AccessDenied), frozen deep-link/URN
    formats. Every `QuranQuotation` carries edition + version + hash (I6, constructor-enforced).
11. **Tools** (`tool-registry`): `quran.get_ayah`, `quran.get_context` with `ToolResult` +
    `ReproducibilityData`; typed errors make fabrication unrepresentable.

---

## 9. Normalization & search (Phase 2, partial)

- **Rules N01–N22** with offset-accurate `SpanMap` and rule-by-rule `NormalizationTrace`
  (`crates/quran-normalization`). Profiles **L0–L8** append-only (exact → whitespace → marks →
  diacritics → hamza → codepoints → skeleton → heuristic affix).
- **Derived forms**: `qai quran forms rebuild <slug@version>` rebuilds token/ayah forms + skeletons;
  MV-018 verifier asserts canonical text unchanged before and after.
- **Index**: `FullTextIndex` trait + **SQLite FTS5** backend (not Tantivy — DEV-05) with custom
  `ar_*` tokenizers; `qai quran index rebuild/verify`; atomic generation activation.
- **Search services** (Rust API, `crates/quran-search`): exact, normalized, phrase, concatenated +
  cross-ayah windows, DFA-only bounded regex (no backtracking — I16), filters, totals,
  highlighting, generation-keyed cache. **Not yet exposed**: `qai quran search` CLI and search
  HTTP/SSE endpoints do not exist.
- **Not started**: morphology import/lexicons (sprint 2.4–2.5), families, counting/discovery,
  doctor index checks, evaluation/soak, all 14 P2 ADRs, all 50 P2 ACs, linguistic dataset + linguist
  decisions (sprint 2.0).

---

## 10. CLI & API reference (implemented)

Data dir: `QAI_DATA_DIR` (isolate demos in a fresh temp dir). Destructive verbs require `--yes`;
reads support `--json`. Exit codes are public API: 0 ok · 1 generic · 2 usage · 3 validation ·
4 policy · 5 not found · 6 conflict/state · 7 cancelled · 70 internal. Error namespaces: `QAI-CFG-`,
`QAI-SEC-`, `QAI-DB-`, `QAI-JOB-`, `QAI-SRC-`, `QAI-PROV-`, `QAI-AUD-`, `QAI-CLI-`, `QAI-QUR-`,
`QAI-NORM-`, `QAI-IDX-` (unique workspace-wide, test-enforced).

```bash
cargo build -p cli --bin qai
export QAI_DATA_DIR="$(mktemp -d)"
./target/debug/qai db migrate
./target/debug/qai quran import fixtures/quran/test-edition-min/manifest.json
./target/debug/qai quran activate test-edition-min@0.1.0 --yes
./target/debug/qai quran get 1:1 --json
./target/debug/qai quran context 2:1 --before 1 --after 1 --boundary surah
./target/debug/qai quran surah 1 | quran division juz 1 | quran resolve 1:1
./target/debug/qai quran edition show test-edition-min@0.1.0 --statistics --hashes
./target/debug/qai quran validate test-edition-min@0.1.0 --report /tmp/report.json
./target/debug/qai quran diff --edition test-edition-min --from 0.1.0 --to <v2>
./target/debug/qai quran normalize --list-profiles | --show-rule N06
./target/debug/qai quran forms rebuild test-edition-min@0.1.0
./target/debug/qai quran index rebuild | quran index verify
./target/debug/qai doctor --quran --deep [--json]
./target/debug/qai serve --bind 127.0.0.1:8737   # loopback only; non-loopback refused (exit 4)
```

HTTP (`crates/server/src/api.rs`, spec `docs/08-api/quran-v1-openapi.json`): `/healthz`, `/readyz`,
`/api/v1/meta`, `/api/v1/quran/editions[/{slug}]`, `/surahs[?edition]`, `/surahs/{number}`,
`/ayahs/{reference}`, `/context/{reference}`, `/divisions/{kind}/{number}`, `/tokens/{reference}`,
`/resolve`, `/citations/{id}`, normalization `preview` (POST) + `profiles` (GET),
`/debug/read/{edition}/{surah}` (RTL labelled reader). Envelope + `meta` + ETag +
`Content-Language`; errors are Diagnostic bodies. Placeholder top-level commands (`secret`,
`source`, `job`, `audit`, `completions`) print phase stubs — their presence in `--help` means nothing.

---

## 11. Invariants I1–I16 (test these, never weaken them)

| # | Invariant | Enforcement |
|---|---|---|
| I1 | Canonical ayah text immutable per edition version | insert-only tables + triggers + `CanonicalWriter`/`ApprovalToken` |
| I2 | Canonical text never produced by a model | no model deps on canonical path; `arch-check` |
| I3 | Readings/editions never merged | `edition_id` in every key and record; QV-027 |
| I4 | Token order stable, verifiable | unique `(edition,surah,ayah,position)` + order checksum; QV-024 |
| I5 | Partial import never becomes `Active` | separate staging tables + single-tx pointer flip |
| I6 | Every quotation carries edition + version + hash | `QuranQuotation` constructor |
| I7 | Corrections = new version + diff + human approval | change requests + differ |
| I8 | Normalization never mutates canonical text | separate tables; MV-018/QV-028 re-verified after every build |
| I9 | Every hit reports its exact ordered rule set | `NormalizationTrace` required field |
| I10 | Every match maps to canonical character offsets | bidirectional `SpanMap`, property-tested |
| I11 | Competing morphological analyses coexist | no `is_correct` column; per-request preference policy recorded |
| I12 | Machine linguistics is Layer D with algorithm/version/confidence/status | DB CHECK + display labels |
| I13 | Family relations typed (form/lemma/stem/root/computational/verified) | closed enum + provenance |
| I14 | Derived indexes reproducible + generation-stamped | manifests + doctor staleness checks |
| I15 | Numeric reports state counting rules; no numerology claims | `CountingRules` + fixed disclaimer |
| I16 | Regex/pattern search resource-bounded | size caps, step budget, timeout, DFA-only engine |

---

## 12. Testing & verification

- Layout: unit + property tests in `crates/*/src`, integration in `crates/*/tests/`
  (application, storage-sqlite ×13, cli trycmd, server API, testkit security/config).
- Property tests (proptest): reference-grammar round-trip + never-panics, tokenizer losslessness,
  span-map properties. Golden sets: 331-case `references.jsonl`, 16 adversarial fixtures.
- CLI acceptance: `crates/cli/tests/quran/read_flow.trycmd` (migrate→import→activate→reads→
  translations→v2→validate→diff→rollback→hashes→error exits 5/6).
- Gates (also `CONTRIBUTING.md`): `cargo fmt --all -- --check` · `cargo check/clippy --workspace
  --all-targets -- -D warnings` · `cargo test --workspace` (note: `application` tests may need
  `-- --test-threads=1`; a parallel-timeout and a flaky `jobs::worker::retries_then_succeeds` are
  recorded) · `cargo xtask arch-check` · `cargo xtask migrate-check`. Full gate `cargo xtask ci`
  adds deny (warns if uninstalled), adr-lint, gen-schema diff, and a doctor-JSON schema check
  (`xtask/src/ci.rs`, 9 steps).

---

## 13. Current state snapshot (2026-09-17)

| Phase | State | Evidence / blockers |
|---|---|---|
| P0 Foundations & Provenance | Implementation complete; **formal sign-off pending** | `phase-00-foundation/done.md`; AC-P0-05 partial; exit ritual + human sign-off open; T55/T56 deferred |
| P1 Canonical Quran Core | In progress (~77% board; 50☑/3◐/12☐; 19/21 ACs automated-green, rituals pending; 12/14 ADRs) | `phase-01-core/STATUS.md`; **blocked on ADR-0101** (dataset/license/editorial reviewer, Draft), ADR-0114 (reference corpus, Draft), debug-reader font licensing, exit ritual |
| P2 Search/Normalization/Linguistics | In progress (24/114 tasks; M1–M3 core landed; 0/50 ACs, 0/14 ADRs) | `phase-02-rag/done.md`; blocked on ADR-0203 dataset, 0.4 FTE linguist, FTS ratification, search-surface wiring, morphology |
| Graph / server / later | Proposed or placeholder | `phase-04-quran-graph/README.md` (proposal, ADR-0202); `phase-03-server/` empty; PRD roadmap numbering unreconciled |

Owner/editorial decisions no agent can take (dataset licensing, reviewer sign-off, linguist
booking, estimate-vs-schedule choices) are tracked as swimlane-X items, mostly unassigned —
see `docs/05-followups/`. surfacing them to the human is part of orchestration.

---

## 14. Known traps (read before acting)

1. **Synthetic fixture ≠ scripture.** `test-edition-min` is fake Arabic-like text for plumbing.
   Never present it as Quran, never "expand" it into real verses, never invent dataset/ reviewer/
   license facts (P1-T56/T58 constraints). Fixture success ≠ editorial approval.
2. **FTS5, not Tantivy.** Plan text still says Tantivy in places; shipped code is FTS5 (DEV-05).
   Don't add Tantivy (offline cache lacks it) or claim plan numbers as installed.
3. **Migration numbering drift.** Plan tables show `0010–0015` (P1) and `0020–0025` (P2); actual
   files are `0007–0012` and `0013–0016`. Always allocate from `migrations/sqlite/`, never from
   plan tables; never edit applied files.
4. **Trigger/code/shape deviations** (P2 `done.md` DEV-04…08): normalization trigger raises
   `QAI-NORM-0003`; search-cache shape differs. Follow code + ledger, not the plan table.
5. **Phase-naming mismatch.** Dir `phase-02-rag` = search/linguistics (multi-RAG is PRD Phase 7);
   `phase-03-server/` is empty; graph proposal lives in `phase-04-quran-graph/` though the PRD
   puts graph at P3. Don't renumber; link carefully.
6. **Stale scaffolds.** `docs/03-plan/{current-plan,master-plan,milestones,roadmap}.md` are empty;
   `docs/06-progress/status.md` lags the ledgers; P0/P1/P2 `README.md` headers were stale before
   2026-09-17 (now carry reviewed-status headers). Trust `tasks.md`/`done.md`/`STATUS.md`/
   `task-done-rollup.md` in that order.
7. **OTLP redaction gap.** Tracing redaction covers stderr; OTLP span export bypasses it
   (`open-questions.md`). Telemetry stays opt-in; don't log secrets anywhere regardless.
8. **Concurrent-writer hazard.** Multiple agents share this tree. Uncommitted changes (e.g. an
   `ImportOptions.reference` field added to `quran-corpus/src/import.rs` without updating
   `application/tests/quran_import.rs`) can break `cargo test` compilation and `fmt --check`.
   Always run `git status` first; never "fix" чужой half-landed change silently — report it,
   and keep your diff confined to your task.
9. **Flaky/slow tests.** `jobs::worker::retries_then_succeeds` flakes; parallel `application`
   runs have timed out (serial rerun passes). De-flake before any exit ritual; don't claim gates
   you didn't run.
10. **Estimate gaps.** Every phase's task sum exceeds its headline estimate (P0 77.5 vs 113.5;
    P1 82 vs 131; P2 112 vs 278 ed), each with an unassigned owner decision. Never compress
    estimates silently; cut D1.12/debug-reader/OpenAPI-docs depth first, never integrity work.

---

## 15. How to orchestrate work (the agent loop)

1. **Startup:** §1 checklist. Restate the goal, the phase, and the TASK-ID before acting.
2. **Pick up work:** from `tasks.md` boards or `docs/04-tasks/active/`; every implementation task
   needs a TASK-ID (create `TASK-nnn-slug.md` first if missing). Respect `Depends` chains and the
   scope fence (phase READMEs §4 list what is out of scope — e.g. no search in P1, no LLM deps
   anywhere near the deterministic path).
3. **Investigate graft-first:** one `graft ask --source` usually locates the target; `callers
   --depth 2/all` before refactors; `grep` for exhaustiveness. Open files only at the exact
   `file:line` spans graft gives; never re-read whole files to rebuild understanding.
4. **Change minimally:** smallest boundary-preserving edit; new errors get registry codes (never
   renumber); new deps need `xtask/allowlist.toml` entries or `arch-check` fails; new migrations
   are append-only + checksummed (`migrate-check`); config changes validate at load and on change;
   long operations get cancellation/timeouts; mutations record provenance + audit.
5. **Verify like a gate:** fmt → clippy `-D warnings` → targeted tests → `arch-check` →
   `migrate-check` → workspace tests (serial for `application` if parallel flakes) → live CLI
   smoke in a temp `QAI_DATA_DIR` (§10). Quote exact results; never claim unrun gates.
6. **Record everything:** flip the board status + append `done.md` (never rewrite history) +
   update `task-done-rollup.md` + phase progress + `CHANGELOG.md` (user-visible) + follow-ups for
   anything open (owner, ritual, decision). ADRs for architectural choices use the §48 template
   incl. Accuracy / Religious-source / Licensing implications.
7. **Rituals need humans:** exit-gate rituals (clean machine, live, recorded, non-implementer
   reviewer) and editorial sign-offs cannot be performed or simulated by you. Mark them pending
   with a named owner request.
8. **Never:** invent scripture/citations/gradings/datasets/reviewers; merge editions or competing
   analyses; put translations where canonical text belongs; add model/vector deps to the
   deterministic path; mutate data in `doctor`; store secrets in SQLite; weaken an invariant to
   hit a schedule; mark complete with failing gates.

Parallel subagents: split by crate/layer along the §5 diagram, share this briefing + TASK-IDs,
and rebase on `git status` before merging — overlapping files (importer, reader, doctor) collide
fast. After large changes, run `graft build` to refresh the graph.

---

## 16. Command cheat sheet

```bash
graft map | graft ask "<q>" --source | graft grep "<lit>" | graft skeleton <file> | graft callers <sym> --depth 2
cargo build -p cli --bin qai
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace   # or: cargo test -p application -- --test-threads=1
cargo xtask arch-check | migrate-check | ci   # ci = full 9-step gate (xtask/src/ci.rs)
./target/debug/qai --help | db migrate | doctor --quran --deep --json | serve --bind 127.0.0.1:8737
```

Key docs: PRD `docs/01-requirements/requirements.md` · architecture `docs/02-architecture/` ·
upstream sources `docs/02-architecture/upstream-sources.md` ·
owner decisions `docs/05-followups/owner-decisions.md` ·
phases `docs/03-plan/phases/phase-{00-foundation,01-core,02-rag,04-quran-graph}/` · tasks
`docs/04-tasks/` · follow-ups `docs/05-followups/` · progress `docs/06-progress/` · corpus
`docs/07-technical/quran-{corpus-architecture,adapter-authoring,citation-spec}.md` · API
`docs/08-api/quran-v1-openapi.json` · runbooks `docs/10-operations/quran-{import,rollback}-runbook.md`.
