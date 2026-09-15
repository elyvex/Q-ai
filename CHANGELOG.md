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
- **quran-normalization** (M1b): all 17 deterministic rules N01–N17 as pure
  total functions with per-rule `SpanMap`s (mapping tables documented per
  rule; N06 standalone-hamza→∅, N15 lam-alef expansion 1→2, N16 full NFC with
  starter-tracked offsets cross-checked against reference NFC), `all_rules()`
  / `by_id()` registry, 41 unit + 7 integration tests (mapping tables,
  idempotency, hull round-trips, fuzz no-panic, basmala→bare/skeleton and
  spaceless-query seed goldens pending linguist review).
- **quran-normalization** (M1b continued): heuristic rules N18–N22 (affix
  strips with ≥2-char guards to fixpoint, N22 repeat-collapse; all
  `RuleKind::Heuristic`), `NormalizationTrace` with derived heuristic flag
  and empty-profile rejection (I9 value type), append-only `ProfileRegistry`
  with the v1 L0–L8 ladder, and the shared `NormalizationPipeline`
  (profile + `adhoc:<sha12>` builds, text+trace always returned together);
  rule versions now typed `domain::SemVer`.
- **quran-normalization** (M1c): migration `0013_quran_normalization`
  (rules + profiles tables, append-only triggers reporting QAI-NORM-0003,
  v1 seeds), `QuranRepository` normalization catalog access + SQLite
  implementation, `application::quran_normalize` services, `qai quran
  normalize` (`--explain`, `--list-profiles`, `--show-rule`, adhoc `--rules`),
  server `POST /api/v1/quran/normalization/preview` + `GET
  /api/v1/quran/normalization/profiles` with byte-identical CLI/API traces
  (AC-P2-39 evidence), OpenAPI entries.
- **quran-search** (new crate, M2): backend-agnostic `FullTextIndex` port
  (`FtsQuery` incl. phrase/boolean/range/regex/all, `SearchOpts` with
  1000-result and 10 s ceilings, `IndexManifest` with generation + version
  stamps, `FtsDoc`/`FtsHit`/`FtsResults` with exact totals) plus the
  `QAI-IDX-*` error contract; FTS5 selected first (DEV-05), Tantivy named
  only as a future adapter.
- **storage**: derived-forms tables `quran_token_forms`, `quran_ayah_forms`,
  `quran_skeletons` (migration `0014`, numbering per DEV-04) with
  FK-to-canonical enforcement, Layer D stamps, and surah-scoped window
  CHECKs; FTS5 availability probe (`fts5_available`) as the M2 entry gate.
- **quran-search** (M2): `skeletons_for_surah` (ayah + surah-scoped 3-ayah
  windows from joined raw texts) and `QAI-IDX-0005 CanonicalChanged`
  (MV-018's fatal code, reused by M4).
- **application** (M2): `quran.forms.rebuild` job (resolve → MV-018 pre →
  load → per-surah build → single tx with MV-018 post → commit;
  content-addressed Layer-D provenance, cancel-safe, idempotent) with
  `FormsRebuildHandler`, `verify_canonical_unchanged` (+ in-transaction
  variant), and `qai quran forms rebuild`.
- **application** (M2): `quran.index.build` job (resolve → MV-018 pre →
  stage → chunked ayah-doc build from stored forms → commit → counted
  manifest → verify → MV-018 post → single-transaction pointer flip with
  run supersede) with `IndexBuildHandler` and `qai quran index
  rebuild/verify`; `0015_quran_indexes` (pointers + build runs, DEV-07
  numbering); `Fts5Index` explicit build generations for retention.
- **quran-search** (M2): `Fts5Index` FTS5 adapter (generation directories,
  transactioned batches, term/phrase/boolean/range/all queries, metadata
  filters, exact counts, DFA-only regex over the term dictionary with I16
  budgets, lifecycle + verify) and the `ar_*` `TokenizerFamily` on the
  shared pipeline (both paths normalize; 5,000-substring parity suite);
  `regex-automata 0.4` workspace dependency (offline-cached).
- **quran-search** (M3): `SearchHit` unified assembly (pinned reference via
  the Phase-1 grammar, validated `QuranQuotation`, exact span + tokens,
  score explanation, mandatory trace, warnings; fail-closed
  `QAI-IDX-0006`), `ScoreExplain`, `Warning` (incl. `QAI-IDX-0101`
  staleness); `CanonicalSpan::byte_range_in` for highlight slicing.
- **application** (M3): `quran.search_exact` (L0/L1, whole-token FTS plus
  substring/prefix scans, Persian zero-result hint, never a silent fold)
  and `quran.search_normalized` (L0–L5 on their FTS fields, L7/L8/adhoc
  verified scans, typed profile-or-rules selector, explain toggles
  relevance + BM25 breakdowns) — both assembling `SearchHit` through the
  single validating path with drift warnings.

### Added — Phase 1 Canonical Quran Core (in progress)
- **cli**: read verbs `quran get/context/surah/division/resolve` and lifecycle verbs
  `quran import/validate/activate/rollback/diff/edition/translation`, every read command
  with `--json`, destructive verbs gated on `--yes`; `quran translation import` now
  writes an attributed provenance row (principle 5) instead of pointing at a principal.
- **cli**: `doctor --quran` runs 19 checks against a read-only database handle with
  `--deep` full-corpus verification; `--quran --json` now emits one merged
  `{"checks":[...]}` document validating against `docs/schemas/doctor.v1.schema.json`
  (previously two concatenated JSON documents).
- **cli (tests)**: `read_flow.trycmd` snapshot suite (trycmd) covering migrate → import →
  activate → RTL reading + provenance → `surah`/`context`/`division`/`resolve` → attributed
  translations → v2 import/validate/diff → activate gen 2 → rollback gen 3 → `hashes` →
  error exits.
- **config**: `QAI_DATA_DIR` now configures the database and object-store paths, not just
  the database file (previously the two diverged).
- **quran-corpus**: 13-checkpoint importer driver, char-level differ, frozen
  hashing recipes; `QAI-QUR-02xx` diagnostics.
- **application**: `quran.import` job handler, approval-gated activation/rollback,
  hash-chained audit bridge, deterministic `QuranReader` + generation-keyed cache.
- **storage**: `QuranRepository` (staging, atomic activation, reads, reports,
  citations, translations), approval rows, audit sequence queries.
- **docs (D1.14)**: five Phase-1 documents published —
  `quran-corpus-architecture.md`, `quran-adapter-authoring.md`,
  `quran-citation-spec.md` under `docs/07-technical/`, and
  `quran-import-runbook.md`, `quran-rollback-runbook.md` under `docs/10-operations/`.
- **docs**: ADR-0101…0114 (0101/0114 + 0111/0112 drafts pending owners/deliverables).
- **arch-check**: `[server]` allowlist now names `storage` + `tools` (both already
  declared deps of `crates/server`); recorded as a layering follow-up (OWN-06) rather
  than a silent loosening.


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
