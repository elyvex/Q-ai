---
last_mapped_commit: 68b7ea6c1aacd1476d154550b2a6e9574614b6af
last_mapped_at: 2026-09-22
---
# Testing Patterns

**Analysis Date:** 2026-09-22

## Test Framework

**Runner:**

- `cargo test` (Rust 1.97.1 stable, edition 2024). No JS-style runner; the workspace is pure Cargo.
- Config: workspace `Cargo.toml` (`resolver = "2"`, shared `[workspace.dependencies]` pinning `proptest 1`, `tempfile 3`, `insta 1`, `trycmd 0.15`, `tokio 1`, `sqlx 0.8`)
- Async runtime in tests: `tokio` with `#[tokio::test]` (multi-thread where noted); single-thread via `tokio::runtime::Builder::new_current_thread()` when driving the real binary synchronously

**Assertion Library:**

- Standard `assert!` / `assert_eq!` / `assert_ne!` / `assert!(matches!(...))`, plus `prop_assert!` / `prop_assert_eq!` inside `proptest!` blocks

**Run Commands:**

```bash
cargo test --workspace                  # Full suite (CI step 3/9)
cargo test -p <crate>                   # Single crate (unit + integration)
cargo test -p cli --test quran          # trycmd snapshot suites only
cargo test -p storage-sqlite --test quran  # Real-SQLite persistence acceptance
cargo run -p xtask -- ci                # Full 9-step pre-merge gate (fmt, clippy, test, deny, arch-check, migrate-check, adr-lint, gen-schema diff, doctor schema)
cargo xtask coverage-gate <lcov.info>   # Per-crate coverage thresholds (needs cargo llvm-cov report first)
```

## Test File Organization

**Location:**

- Unit tests: inline `#[cfg(test)] mod tests` at the bottom of the source file (`crates/domain/src/primitives.rs:320-432`, `crates/storage/src/error.rs:110-223`, `crates/config/src/lib.rs:1018-1349`)
- Property tests: inline `proptest!` blocks in `src/` AND standalone files in `crates/*/tests/` (e.g. `crates/domain/tests/primitives.rs`, `crates/quran-core/tests/reference_grammar.rs`)
- Integration tests: `crates/<name>/tests/*.rs`, one file per concern (`crates/storage-sqlite/tests/quran.rs`, `crates/server/tests/api.rs`, `crates/application/tests/quran_reader.rs`, `crates/quran-corpus/tests/fixtures.rs`, `crates/testkit/tests/secret_leak.rs`)
- CLI snapshots: `crates/cli/tests/quran/*.trycmd` driven by thin harnesses (`crates/cli/tests/quran.rs`, `crates/cli/tests/catalog.rs`, `crates/cli/tests/doctor_json.rs`)

**Naming:**

- Test files are `snake_case` topic names: `primitives.rs`, `quran.rs`, `api.rs`, `secret_leak.rs`, `diagnostics.rs`, `error_codes.rs`, `reference_grammar.rs`, `backup_restore.rs`, `generation_monotonicity.rs`
- Test fns are `snake_case` sentences: `canonical_triggers_abort_raw_writes_with_codes`, `golden_ayah_texts_match_the_imported_fixture`, `doctor_quran_json_is_a_single_merged_document`
- trycmd cases: `read_flow.trycmd` (end-to-end reading + edition lifecycle), `normalize.trycmd` (normalization introspection), `catalog.trycmd` (upstream catalog)

**Structure:**

```
crates/<name>/
├── src/
│   └── *.rs                  # inline #[cfg(test)] mod tests + proptest! blocks
├── tests/
│   ├── <topic>.rs            # #[test] / #[tokio::test] integration suites
│   └── quran/*.trycmd        # CLI snapshot cases (cli crate only)
fixtures/quran/
├── test-edition-min/         # manifest.json + ayahs.csv (14 ayahs, 5 surahs)
├── test-edition-min-v2.json  # second version for diff/rollback flows
├── test-translation-min.json # attributed translation import
├── test-gloss-min.json       # word-gloss import
├── golden/                   # ayah_texts.jsonl, references.jsonl (≥300 cases)
└── adversarial/<name>/       # 16 malformed manifests (missing_ayah, nfd_text, …)
         └── hash_mismatch/   # data.json + declared-sha256.txt package
```

## Test Structure

**Suite Organization:**

```rust
//! Phase 1 — persistence acceptance: AC-P1-08/09 (P1-T12–T14, T24).
//!
//! Against real SQLite in a tempdir, never a mock:
//! - insert-only triggers abort raw `UPDATE`/`DELETE` with `QAI-QUR-0001…0005`;
//! ... (contract bullets)

use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

async fn migrated_db() -> (tempfile::TempDir, SqliteDatabase, String) { /* ... */ }

#[tokio::test]
async fn canonical_triggers_abort_raw_writes_with_codes() {
    let (_dir, db, path) = migrated_db().await;
    // ... arrange via stage_run + activate_edition, act with raw SQL, assert codes
}
```

- Every integration file opens with a `//!` header citing the acceptance criteria and task IDs it pins (`crates/storage-sqlite/tests/quran.rs:1-12`, `crates/server/tests/api.rs:1-4`, `crates/quran-corpus/tests/fixtures.rs:1-5`)
- Shared arrange logic goes in `async fn` / plain `fn` helpers at file top (`migrated_db`, `stage_run`, `edition_row`, `test_view`, `test_state`, `serve_once`, `body_json`); assertions stay inline in the test

**Patterns:**

- Setup pattern: build a tempdir, apply real migrations, seed minimal rows, return handles — see `migrated_db()` in `crates/storage-sqlite/tests/quran.rs:31-95` (inserts `principals`, `sources`, `source_versions`, `provenance_records`, `approvals` rows so activation preconditions hold)
- Teardown pattern: hold the `TempDir` binding for the test lifetime (`let (_dir, db, path) = ...`); drop order cleans up. No explicit teardown fns
- Assertion pattern: `assert!` with a context message showing both sides (`assert!(message.contains(code), "{sql} must abort with {code}, got: {message}")`); JSON assertions parse to `serde_json::Value` and index keys (`assert_eq!(value["code"], "QAI-QUR-0307")`)

## Mocking

**Framework:** No mocking framework. Hand-written fakes (`FakeBackend`, `FakeApi`, `MockAuditRepo`) plus the shared `testkit` fixture crate. `proptest` strategies generate inputs; `tempfile` provides isolation.

**Patterns:**

```rust
// In-memory fake behind the real trait (server/tests/api.rs:50-88)
struct FakeBackend;

#[async_trait::async_trait]
impl QuranBackend for FakeBackend {
    async fn backend_get_ayah(
        &self,
        reference: &quran_core::QuranRef,
        _options: &quran_core::AyahOptions,
    ) -> Result<(quran_core::AyahView, BackendMeta), ToolError> {
        match reference {
            quran_core::QuranRef::Ayah { surah, .. } if surah.get() == 99 => {
                Err(ToolError::Backend { code: "QAI-QUR-0307".into(), detail: "ayah not found: 99:1".into() })
            }
            _ => Ok((test_view(), test_meta())),
        }
    }
    // ...
}
```

```rust
// Deterministic fixtures (crates/testkit/src/lib.rs)
pub fn fixture_id<T>(index: u64) -> T;              // stable UUID per index
pub fn sample_config(dir: &TempDir) -> config::Config;
pub fn sample_source_version() -> SourceVersion;
pub fn sample_job_record_at(clock: &FixtureClock, index: u64) -> JobRecord;
pub struct FixtureClock { /* fixed 2026-01-01T00:00:00Z, set() to advance */ }
pub struct MockAuditRepo { /* Vec-backed AuditRepository with tamper() */ }
```

**What to Mock:**

- Trait backends that would need a database or network: `QuranBackend` / `QuranApiBackend` fakes in `crates/server/tests/api.rs` (contract shape, ETag/304, error bodies — no DB)
- The audit repository via `testkit::MockAuditRepo` (in-memory `Vec` with `tamper(sequence)` for chain-break tests)
- Time via `testkit::FixtureClock` (default `2026-01-01T00:00:00Z`); IDs via `testkit::fixture_id(index)` for replay-stable UUIDs

**What NOT to Mock:**

- Persistence: integration tests use **real SQLite** in a tempdir, never mocks (`CONTRIBUTING.md` checklist; `crates/storage-sqlite/tests/*.rs`, `crates/application/tests/*.rs` all call `migrate_database` / `apply_migrations` for real)
- Migrations: always applied from `migrations/sqlite/` (never hand-created schema)
- The CLI binary: trycmd drives the real `qai` binary via `env!("CARGO_BIN_EXE_qai")` with `QAI_DATA_DIR` pointed at a fresh tempdir (`crates/cli/tests/quran.rs:11-20`); each trycmd file gets its own database because files run in parallel
- Secrets: use the `SENTINEL_9f3c__DO_NOT_LEAK` value and assert zero bytes in output rather than mocking redaction (`crates/testkit/tests/secret_leak.rs`)

## Fixtures and Factories

**Test Data:**

```rust
// Golden-file loop (crates/quran-core/tests/reference_grammar.rs:21-38)
const GOLDEN: &str = include_str!("../../../fixtures/quran/golden/references.jsonl");

fn golden_rows() -> Vec<(String, serde_json::Value)> { /* line-numbered JSON parse */ }

#[test]
fn golden_valid_cases_parse_and_serialize() {
    for (where_, row) in golden_rows() {
        let Some(expected) = row.get("canonical").and_then(serde_json::Value::as_str) else { continue };
        let parsed = parse(input).unwrap_or_else(|err| panic!("{where_} `{input}` failed: {err}"));
        assert_eq!(serialize(&parsed), expected, "{where_} `{input}`");
    }
}
```

```rust
// Edition fixture via include_str! + dual adapters (crates/quran-corpus/tests/fixtures.rs:10-11)
const MIN_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const MIN_CSV: &str = include_str!("../../../fixtures/quran/test-edition-min/ayahs.csv");
```

**Location:**

- Shared builders: `crates/testkit/src/lib.rs` (`temp_dir`, `temp_db_path`, `sample_config`, `sample_toml_config`, `sample_source_version`, `sample_source_file`, `sample_job_record`, `sample_provenance_record`, `MockAuditRepo`, `FixtureClock`, `fixture_id`) — dev-only crate, must never be a runtime dependency
- Golden truth: `fixtures/quran/golden/ayah_texts.jsonl`, `fixtures/quran/golden/references.jsonl` (≥300 grammar cases; valid rows assert `parse → serialize`, invalid rows assert the exact `QAI-QUR-01xx` code)
- Minimal corpus: `fixtures/quran/test-edition-min/` (+ `ayahs.csv` for CSV-adapter parity), `test-edition-min-v2.json`, `test-translation-min.json`, `test-gloss-min.json`
- Adversarial inputs: `fixtures/quran/adversarial/<16 names>/manifest.json` — each must be *format-valid* (parse OK) and fail later in the validator with its specific rule id (`all_sixteen_adversarial_manifests_are_format_valid` in `crates/quran-corpus/tests/fixtures.rs:147-177`)
- Upstream mirrors: `fixtures/upstream/` (large read-only vendor snapshots, not hand-editable)

## Coverage

**Requirements:** Enforced by `cargo xtask coverage-gate <lcov.info>` (`xtask/src/coverage.rs`):

- `domain`, `provenance`, `audit`, `sources`: **≥ 85%** line coverage
- `config`, `jobs`, `storage-sqlite`: **≥ 75%**
- `cli`, `server`: smoke + snapshot only, no numeric gate

**View Coverage:**

```bash
cargo llvm-cov --workspace --lcov --output-path lcov.info   # produce the report
cargo run -p xtask -- coverage-gate lcov.info               # enforce thresholds
```

The gate parser (`parse_lcov` / `coverage_by_crate` / `violations`) is pure and self-tested against synthetic LCOV in `xtask/src/coverage.rs:118-172`; a crate with no records counts as 100% (vacuous pass).

## Test Types

**Unit Tests:**

- Scope: single-type invariants beside the implementation (`#[cfg(test)] mod tests`): round-trips (`ids_are_distinct_types`, `all_ids_roundtrip` in `crates/domain/src/ids.rs`), code/remedy tables (`code_and_remedy_match_contract_table` in `crates/storage/src/error.rs`), config merge layers (`toml_merges_every_section`, `env_overrides_every_section`, `cli_overrides_every_section` in `crates/config/src/lib.rs`), exit-code mappers (`crates/cli/src/exit_code.rs:43-72`)
- Approach: table-driven `vec![...]` cases with one assert per row; exhaustive enum coverage via `all_errors()` helpers

**Integration Tests:**

- Scope: cross-crate behavior against real infrastructure. Examples: `crates/storage-sqlite/tests/quran.rs` (trigger aborts with `QAI-QUR-0001…0005`, activation bumps `corpus_generation`, rollback restores prior version), `crates/application/tests/*.rs` (`quran_import`, `quran_reader`, `index_build`, `forms_rebuild`, `search_cache`), `crates/server/tests/api.rs` (envelope shape, ETag/304, diagnostic error bodies, OpenAPI spec coverage), `crates/cli/tests/doctor_json.rs` (audit-verify rejects corrupt chains with exit 3 without modifying the DB)
- Approach: `#[tokio::test]` + tempdir + real migrations; raw `sqlx` for fault injection (drop trigger, rewrite `chain_hash`, checkpoint WAL) then assert the product detects it

**E2E Tests:**

- Framework: `trycmd` 0.15 snapshot suites + the 9-step `cargo xtask ci` gate (step 9 builds the real binary, migrates a scratch DB, and validates `doctor --json` against `docs/schemas/doctor.v1.schema.json`)
- Canonical flow (`crates/cli/tests/quran/read_flow.trycmd`): `db migrate → quran import → activate --yes → get/resolve/surah/context/division → translation import → gloss import → validate → activate v2 → diff → rollback → hashes → error exits 5/6`, with `[..]` wildcards for volatile values (hashes pinned where deterministic)
- Permanent sentinel suites (run in every phase): `crates/testkit/tests/secret_leak.rs` (zero sentinel bytes across `Secret` reprs, audit redaction, tracing layer, `config show`/`doctor --json` paths), `crates/testkit/tests/diagnostics.rs` (every error type renders stable human/JSON), `crates/testkit/tests/error_codes.rs` (`QAI-<NS>-<nnnn>` well-formedness + cross-namespace uniqueness)

## Common Patterns

**Async Testing:**

```rust
// Preferred: attribute macro (crates/server/tests/api.rs, crates/storage-sqlite/tests/quran.rs)
#[tokio::test]
async fn activation_moves_rows_flips_pointer_and_bumps_generation() {
    let (_dir, db, _path) = migrated_db().await;
    let mut uow = db.write().await.unwrap();
    // ...
    uow.commit().await.unwrap();
}

// Bridging sync contexts (binary-driving tests, crates/cli/tests/doctor_json.rs:29)
let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
runtime.block_on(application::db::migrate_database(&cfg, &migrations)).unwrap();
```

**Error Testing:**

```rust
// Typed-variant assertion (crates/storage-sqlite/tests/quran.rs:324)
assert!(matches!(err, storage::StorageError::ConstraintViolation { .. }));

// Stable-code assertion (crates/quran-core/tests/reference_grammar.rs:63-65)
match parse(input) {
    Ok(parsed) => panic!("{where_} `{input}` parsed unexpectedly to {parsed:?}"),
    Err(err) => assert_eq!(err.code().to_string(), expected, "{where_} `{input}`"),
}

// Raw-SQL trigger assertion (crates/storage-sqlite/tests/quran.rs:247-257)
let err = sqlx::query(sql).execute(&pool).await.unwrap_err();
assert!(message.contains(code), "{sql} must abort with {code}, got: {message}");

// CLI exit-code assertion lives in the trycmd file, not Rust:
// ```console
// $ qai quran activate test-edition-min@0.1.0 --yes
// ? 6
// error: no staged edition test-edition-min@0.1.0
// ```
```

**Property Testing (`proptest 1`):**

```rust
// Round-trip + invariant style (crates/domain/tests/primitives.rs:17-27)
proptest! {
    #[test]
    fn semver_display_parse_roundtrip(v in semver_str()) {
        let parsed: SemVer = v.parse().unwrap();
        let shown = parsed.to_string();
        let reparsed: SemVer = shown.parse().unwrap();
        prop_assert_eq!(parsed, reparsed);
        prop_assert_eq!(shown, v);
    }
}
// Grammar fuzzer (crates/quran-core/tests/reference_grammar.rs:150-156)
proptest! {
    #[test]
    fn parser_never_panics_and_errors_are_coded(input in "\\PC*") {
        if let Err(err) = parse(&input) {
            prop_assert!(rendered.starts_with("QAI-QUR-01"), "unexpected code {rendered}");
        }
    }
}
```

Where used: `crates/domain/tests/primitives.rs` (semver/UUID/confidence/language/timestamp), `crates/domain/src/hashing.rs`, `crates/quran-corpus/src/tokenize.rs`, `crates/quran-normalization/src/span.rs` + `tests/deterministic_rules.rs`, `crates/quran-core/tests/reference_grammar.rs` (arbitrary `QuranRef` round-trip + never-panics fuzz), `crates/application/tests/quran_reader.rs` (manual `TestRunner` boundary sampling).

---

*Testing analysis: 2026-09-22*
