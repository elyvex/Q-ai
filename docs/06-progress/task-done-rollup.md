# Task Done Rollup

> Completed tasks across all phases. Newest first.

## Phase 1 — Canonical Quran Core (in progress)

### Session: 2026-09-14 — execution plan + M0/M1

**Plan**
- `docs/03-plan/phases/phase-01-core/execution-plan.md` written (M0–M10, files/deps/
  allowlist/tests per increment, blocker fallbacks B1–B5, risk mitigations).
- Owner decisions flagged in `done.md` §7 (OWN-01 dataset, OWN-02 reference corpus,
  OWN-03 estimate gap, OWN-04 axum provisional, OWN-05 Phase-0 exit discrepancy).

**quran-core (new crate)**
- P1-T06 — newtypes, enums, slug grammar, grapheme-count helper.
- P1-T07 — edition/surah/ayah/segment/token structs (+ per-row `provenance`, matching
  the authoritative DDL/QV-025).
- P1-T10 — `QuranQuotation` + I6/principle-5 constructor guards.
- P1-T08/T09 — hand-written reference parser + serializer; 331-case golden set
  (`fixtures/quran/golden/references.jsonl`); round-trip + never-panics proptests.
  AC-P1-12/13 automated-green (exit ritual pending).
- `cargo test -p quran-core`: 29 unit + 6 integration passing. Pinned deps respect I2
  (`domain, serde, thiserror, unicode-segmentation` only).

**quran-corpus (new crate)**
- P1-T15 — `qai.quran.edition` v1 types + JSON Schema doc.
- P1-T16/T17 — `EditionAdapter` trait with JSON and CSV adapters; CSV reproduces the
  JSON manifest's ayahs exactly.
- P1-T05 — synthetic `test-edition-min` + 16 adversarial fixtures (all schema-valid;
  targeted at specific QV ids in M5).
- P1-T21 — Unicode auditor (normalization form, forbidden points, expected blocks).
- P1-T22 — whitespace-preserving tokenizer + exact separators + grapheme/byte
  offsets, with a losslessness proptest.
- P1-T23 — frozen `text_hash` / `structure_hash` / `token_order_hash` recipes.
- P1-T18 — `quran_edition_v1` registered in a new (additive) `sources::ValidatorRegistry`;
  `QuranEditionValidator` bridges registry calls to the detailed validator.
- P1-T19/T20 — all QV-001…QV-028 implemented (content rules in `validate_edition`;
  pipeline-state rules split honestly: QV-013/014 helpers now, QV-015 skip-if-unconfigured,
  QV-024/025/026 with the importer, QV-028 vacuous in v1).
- P1-T30 — 16/16 adversarial fixtures rejected with their specific rule ids.
  AC-P1-04 automated-green (exit ritual pending).
- P1-T12/T24 — Quran migrations `0007`–`0012` with insert-only triggers and checksums.
- P1-T13 — `QuranRepository` plus SQLite staging, activation/rollback, canonical reads,
  reports, citations, and translations.
- P1-T14 — raw-SQL trigger suite plus staging/activation/rollback behavior. AC-P1-08
  automated-green (exit ritual pending).

**Tooling**
- M0 — allowlist entries for `quran-core`, `quran-corpus`, `citations`, `tools`,
  `tool-registry`; extended `storage-sqlite`, `application`. `arch-check` converted to a
  name-keyed map (fail-closed preserved) with a new regression test.

### Verified
- `cargo test --workspace`: 291 passing, 0 failing.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo fmt --all`: clean.
- `cargo run -p xtask -- arch-check`: OK.
- `cargo run -p xtask -- migrate-check`: OK.

## Phase 0 — Foundations & Provenance

### Session: Phase 0 completion push

**Storage (`storage-sqlite`)**
- T18 — Dual SQLite pools (write `max_connections=1`, read `query_only`), WAL,
  `foreign_keys=ON`, `busy_timeout`. `SqliteDatabase::open_read_only` for doctor.
- T19 — Migration runner: `apply_migrations` (append-only, checksummed, idempotent),
  `verify_checksums`, `backup` via `VACUUM INTO` (never `fs::copy`).
- T17/T23/T26/T36 (partial) — Real repository implementations for sources, provenance,
  audit, jobs, settings over a shared `Arc<Mutex<Transaction>>`; commit atomically persists
  every repo's writes. Added `SourceRepository::insert_source`.
- Migration checksums manifest `migrations/sqlite/checksums.json` (T19 CI gate).

**Domain / security (`domain`)**
- T43 — `security_archive`: zip-slip, symlink-entry, bomb, entry-count, nested-depth guards.
- T44 — `security_net`: resolve-then-check SSRF guard + domain allowlist (fail-closed).
- T45 — `security_input` (field length, control chars, JSON depth) and `security_sanitize`
  (scripts, event handlers, dangerous URI schemes).

**CLI (`cli`)**
- T48/T49/T50 — `qai` binary entry point (`src/main.rs`), `db` command group wired to real
  migrations (migrate/status/verify/plan/backup/restore), config-aware loading.
- T52/T53/T54 — Doctor engine with a 26-check registry (config, DB, secrets, jobs, audit,
  sources, filesystem, security, observability, outbox/tombstones), read-only DB probe,
  remedies + next commands on every non-pass, and `--repair-preview` (no mutation).

**Application (`application`)**
- Orchestration/composition root: `run()` bootstrap, `db` module (migrate/status/verify/
  backup/probe) keeping `storage-sqlite` behind the application boundary.

**Testkit (`testkit`) — D0.16**
- T57 — Fixtures (temp dir/db, sample config/TOML/source version/job/provenance) and
  `MockAuditRepo`.
- Integration suites: `secret_leak`, `security_path_guard` (40+ payloads),
  `security_archive_guard`, `security_ssrf_guard`, `config_precedence`.

**Tooling**
- `xtask/allowlist.toml` — reconciled allowlist with actual dependency edges (testkit,
  application composition root, `storage-sqlite` serde rename fix).

**Docs — D0.17**
- T58 — 5 runbooks (`docs/runbooks/`), `CONTRIBUTING.md` with the DoD checklist, error-code
  registry rules, secret-redaction requirements, and append-only migration policy.

### Verified
- `cargo test --workspace`: 139 passing, 0 failing.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo fmt --all -- --check`: clean.
- `cargo run -p xtask -- arch-check`: OK.
- `cargo run -p xtask -- migrate-check`: OK.
- `qai db migrate | status --json | verify | backup` and `qai doctor [--json|--repair-preview]`
  smoke-tested against a live SQLite database.

### Still open (not claimed complete)
- True `SIGKILL` job chaos suite (AC-P0-11) and 2-second cancellation timing (AC-P0-12).
- Full audit chain recomputation verifier (`tests/integrity/audit_chain.rs`) beyond the
  structural chain check.
- Outbox/generations/tombstones domain types + allocators (T61–T67) and consistency suites.
- Keychain/age-encrypted secret backends.
- Coverage gates (AC-P0-22) and the recorded exit-gate ritual.
