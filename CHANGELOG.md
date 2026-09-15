# Changelog

All notable changes to Q-ai are documented here.

## [Unreleased]

### Added — Phase 2 Quran Search, Normalization & Linguistics (in progress)

- **quran-normalization** (new crate, M1a): `QAI-NORM-*` errors with the
  Phase-2 `Diagnostic` contract, `RuleId` N01–N24 catalog identity with
  heuristic/reserved flags, the `NormalizationRule` trait, `NormalizedText`,
  and the bidirectional composable `SpanMap` (I10) with unit + proptest
  coverage (identity, composition associativity, round-trip containment).
- **docs**: `docs/03-plan/phases/phase-02-rag/execution-plan.md` — reconciled
  M1a..M7 plan (migration mapping `0013`–`0018`, FTS5-first fallback DEV-05).
- **xtask**: `arch-check` allowlist gains `[quran-normalization]`
  (`domain` only; no llm/embeddings/retrieval/vector-store edges).

### Added — Phase 1 Canonical Quran Core (in progress)
- **quran-corpus**: 13-checkpoint importer driver, char-level differ, frozen
  hashing recipes; `QAI-QUR-02xx` diagnostics.
- **application**: `quran.import` job handler, approval-gated activation/rollback,
  hash-chained audit bridge, deterministic `QuranReader` + generation-keyed cache.
- **storage**: `QuranRepository` (staging, atomic activation, reads, reports,
  citations, translations), approval rows, audit sequence queries.
- **docs**: ADR-0101…0114 (0101/0114 + 0111/0112 drafts pending owners/deliverables).


- **quran-core** (new crate): pure domain — `SurahNumber`/`AyahNumber`/`TokenPosition`
  newtypes, edition/enum vocabulary, `QuranEdition`/`EditionStatistics`,
  `Surah`/`Ayah`/`Segment`/`Token`, `QuranQuotation` (constructor requires edition
  identity + version + hash; translations need a named translator), and `QAI-QUR-*`
  errors implementing the Phase-1 `Diagnostic` contract.
- **xtask**: `arch-check` allowlist is now name-keyed (`#[serde(flatten)]`) so Phase-1
  crates register edges in `allowlist.toml` without gate changes; fail-closed preserved
  with a new regression test.
- **quran-corpus** (new crate): `qai.quran.edition` v1 intermediate format + JSON/CSV
  adapters (`EditionAdapter` trait) with `QAI-QUR-02xx` errors.
- **docs**: `docs/schemas/quran-edition-source.v1.schema.json` for the edition format.
- **fixtures**: synthetic `test-edition-min` edition (5 surahs / 14 ayahs, nonsense
  Arabic-shaped text), 16 adversarial corpora, 331-case reference golden set.
- **storage**: `QuranRepository` covering staging, atomic activation/rollback,
  canonical reads, validation/difference reports, citations, and translations.
- **storage-sqlite**: Quran migrations `0007`–`0012`, insert-only canonical triggers,
  and repository implementation.

### Added — Phase 0 foundations

- **storage-sqlite**: real repository implementations for sources, provenance, audit,
  jobs, and settings over a shared write transaction; `SqliteDatabase::open_read_only`
  and read-only health probes.
- **storage-sqlite**: checksummed, append-only migration runner (`apply_migrations`),
  `verify_checksums`, and `VACUUM INTO` backups (never a raw file copy).
- **storage-sqlite**: `migrations/sqlite/checksums.json` manifest for the CI append-only gate.
- **domain**: security guards — `security_archive` (zip-slip/bomb/symlink/depth),
  `security_net` (SSRF resolve-then-check + domain allowlist), `security_input`
  (length/control/JSON-depth), `security_sanitize` (scripts/handlers/URI schemes).
- **application**: composition root with `run()` bootstrap and a `db` module for
  migrate/status/verify/backup/probe.
- **cli**: `qai` binary, `db` command group backed by real migrations, and a 26-check
  read-only `doctor` with remedies, next commands, `--json`, and `--repair-preview`.
- **testkit**: fixtures and integration suites for secret-leak, path/archive/SSRF guards,
  and config precedence.
- **docs**: 5 runbooks, `CONTRIBUTING.md` with the Definition of Done checklist.
- **outbox / generations / tombstones** (D0.18): `CorpusGeneration`/`CorpusScope`,
  `OutboxEvent`, `Tombstone` domain types; `OutboxRepository` wired into `UnitOfWork`;
  transaction-scoped workflows; monotonic generation allocator; lease-based relay;
  doctor checks; consistency tests (commit-bounds, idempotency, 50-writer monotonicity,
  tombstone-before-visibility).
- **config**: `SecretStore` trait + `SecretRef` with a real env backend; keychain and
  age-encrypted-file backends abstracted (return `Unsupported` until their crates land).
- **storage-sqlite**: down-migration files (0001–0006) + `revert_last_migration`.
- **application/cli**: verified backup restore with `.pre-restore` swap; `qai db restore --yes`.
- **observability**: telemetry denylist (query/prompt/document/research/model fields).
- **sources**: `ManifestParser::ingest_local` (Staged) with tamper detection and the
  unsigned policy; `SourceError` gains stable `QAI-SRC-nnnn` codes.
- **xtask**: `coverage-gate` (per-crate thresholds, CI-wired) and `adr-lint` (AC-P0-19).
- **jobs**: in-process worker pool — `JobQueue` trait + `InMemoryJobQueue`, `HandlerRegistry`,
  and a `Worker` with cancellation/heartbeat, retry/backoff, dead-lettering, and minimal
  JSON-Schema payload validation; `application::job_queue` wires it to SQLite.
- **sources**: real **ed25519** manifest signature verification (`ed25519-dalek`) with a
  tamper/wrong-key test.
- **config**: `SecretStore` backends — env, **XChaCha20-Poly1305** encrypted file, and OS
  keychain behind the `keychain` feature.
- **observability**: opt-in OTLP trace exporter behind the `otlp` feature (T42).
- **storage**: `Diagnostic` now has rendering defaults; `JobError`/`AuditError`/
  `ProvenanceError`/`SourceError` implement it with stable `QAI-*` codes.
- **xtask**: `validate` subcommand (self-contained JSON-Schema subset validator) and a
  9-step `ci` gate that also runs `adr-lint` and validates `qai doctor --json` against
  `docs/schemas/doctor.v1.schema.json`.
- **ci**: 3-OS test matrix (`--all-features` on Linux), plus `doctor`, `msrv`, and
  enforced `coverage` jobs; ADR lint added to the arch job.
- **coverage**: `cargo llvm-cov --workspace` enforced by `xtask coverage-gate`
  (domain 95.4% / provenance 86.4% / audit 90.2% / sources 88.7% / config 93.4% /
  jobs 81.2% / storage-sqlite 88.7%).
- **docs**: ADR-0000 (project architecture), ADR-0301 (RAG strategy, Proposed),
  `examples/config/default.toml`, `docs/plans/handoff-p0-to-p1.md`,
  `docs/05-followups/done.md` (AC verification).

### Changed

- `storage`: repository traits are now `Send + Sync` (async `&self` methods).
- `audit`: the chain writer links each event to the previous hash; the verifier recomputes
  every hash to detect tampered rows and sequence gaps.
- `domain`: diagnostic codes render as `QAI-<NS>-<nnnn>`.
- `xtask`: `arch-check` allowlist reconciled with actual dependency edges; the
  `storage-sqlite` table rename is honored; migrate-check accepts the `sha256:` prefix.

### Fixed

- Workspace `clippy -D warnings` and `cargo fmt --check` are clean.
- `qai db migrate --data-dir <new>` now creates the data directory instead of failing.
- `sources`: `Approved` requires a human approver identity (PRD §22.3).
