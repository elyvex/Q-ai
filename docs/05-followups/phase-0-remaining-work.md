# Phase 0 — Remaining Work Log

> **Written:** 2026-09-12
> **Updated:** 2026-09-14 (final)
> **Purpose:** Detailed breakdown of everything remaining for an AI agent to complete Phase 0.
> **Repo:** /Users/ali/dev/rust/Q-ai
> **Plan:** docs/03-plan/phases/phase-00-foundation/plan.md
> **Tasks:** docs/03-plan/phases/phase-00-foundation/tasks.md
> **Acceptance:** docs/03-plan/phases/phase-00-foundation/acceptance.md
> **Evidence:** docs/05-followups/done.md

---

## 0. Completion Status (2026-09-14, final)

**Gate:** `cargo test --workspace` = **267 passing / 0 failing**; `cargo xtask ci` (9 steps)
green; `fmt`, `clippy -D warnings`, `arch-check`, `migrate-check`, `adr-lint` all green;
coverage gate met (`cargo llvm-cov` + `xtask coverage-gate`); `qai` binary smoke-tested.

### DONE

- **§A** storage-sqlite real repositories + `migrate.rs` (`apply_migrations`,
  `verify_checksums`, `revert_last_migration`, `backup` via `VACUUM INTO`). *(T17–T20)*
- **§B** CLI binary (`crates/cli/src/main.rs`, `[[bin]] name = "qai"`), wired through
  `application`. *(T48–T51)*
- **§C** Doctor engine: 26-check registry, read-only DB probe, remedies + next commands,
  `--json` (schema-validated), `--repair-preview`. *(T52–T54)*
- **§D** Security guards: archive / net (SSRF) / input / sanitize. *(T43–T45)*
- **§E** Testkit fixtures + suites (secret-leak, path/archive/SSRF, config precedence,
  telemetry privacy, error codes). *(T57)*
- **§F** Jobs: full recovery suite + **in-process worker pool** (registry, retry/backoff,
  dead-lettering, cancellation); provenance/audit DB-level integrity; sources preconditions +
  manifest signing (**real ed25519**) + ingest + genealogy; telemetry denylist; workspace
  error-code uniqueness + **universal `Diagnostic`** conformance. *(T26/T27, T30, T31, T36–T39)*
- **§G** Outbox / generations / tombstones: domain types, `OutboxRepository`,
  transaction-scoped workflows, SQLite impl, monotonic allocator, relay, consistency tests
  (commit-bounds, idempotency, 50-writer monotonicity, tombstone-before-visibility). *(T61–T63,
  T65–T67)*
- **§H** ADR-0000 (project architecture) written; ADR-0001 status corrected; `adr-lint` guards
  all 12 Phase-0 ADRs. Secret-store trait + `SecretRef` with a real env backend. *(T14 partial)*
- **§I** 5 runbooks, `CONTRIBUTING.md`, `examples/config/default.toml`, `hashing-spec.md`. *(T58)*
- **§J** `migrations/sqlite/checksums.json` (up **and** down files). *(T19)*
- **§L** Application composition root + `run()`/`db` module. *(T37-adjacent)*
- **§M** `done.md` (AC verification), `docs/plans/handoff-p0-to-p1.md`, task-board sync.

### PARTIAL

- None. All 26 acceptance criteria are verified (see `done.md`). The Linux/Windows legs of
  the 3-OS matrix run in CI; locally they are covered by cross-target type-checks.

### REMAINING (process)

- **Exit-gate ritual** — the recorded live walkthrough (AC-P0-03/05/06/08/11/14/16) on a
  clean machine.

### BLOCKED

- None.

### Closed since the previous revision

- **AC-P0-01** `cargo xtask ci` (9 steps) passes locally; CI workflow includes the 3-OS
  matrix, `doctor`, `msrv`, `deny`, `schemas`, and `coverage` jobs.
- **AC-P0-22** coverage measured with `cargo llvm-cov` and enforced by `xtask coverage-gate`:
  domain 95.4% / provenance 86.4% / audit 90.2% / sources 88.7% (≥85); config 93.4% /
  jobs 81.2% / storage-sqlite 88.7% (≥75).

---



## 1. Current Workspace Status

### Compilation & Tests

**Workspace compiles cleanly:** `cargo check --workspace` succeeds with zero errors (warnings only from unused code in stub crates).

**Crate status:**

| Crate | Lines | Status | Tests | Notes |
|-------|-------|--------|-------|-------|
| `domain` | ~450 | ✅ Complete | 22 | IDs, types, hashing, security guards, licensing |
| `config` | ~1000 | ✅ Complete | 12 | Layered loader, ValueOrigin, Secret<T>, validation |
| `storage` | ~335 | ✅ Complete | 2 | Traits only: Database, ReadTx, UnitOfWork, 5 repo traits |
| `storage-sqlite` | ~392 | 🔶 Partial | 1 | SQLite pools + pragmas work. All 5 repo impls are stubs returning `StorageUnavailable` |
| `jobs` | ~364 | 🔶 Partial | 4 | Types complete (JobRecord, JobState, JobHandler, JobContext). JobStore wraps repo but repo is still stub |
| `provenance` | ~255 | 🔶 Partial | 5 | Record model, Attribution, ApprovalToken, CanonicalWriter. Repo trait has no real impl |
| `audit` | ~296 | 🔶 Partial | 4 | AuditEvent, hash chain writer/verifier, redaction. Repo trait has no real impl |
| `sources` | ~460 | 🔶 Partial | 6 | StateMachine, ManifestParser, GenealogyResolver complete. Repo trait has no real impl |
| `observability` | ~260 | ✅ Complete | 6 | Tracing subscriber, span conventions, metric catalog |
| `cli` | ~580 | 🔶 Partial | 6 | Full clap command tree + exit codes + doctor stub. **Missing:** `src/main.rs` binary entry point |
| `server` | ~158 | ✅ Complete | 5 | Hand-rolled tokio TCP health server (/healthz, /readyz, 404/405) |
| `testkit` | 3 | ❌ Stub | 0 | Just a doc comment — no test harness, no fixtures |
| `application` | 4 | ❌ Stub | 0 | Not started |

**Total tests passing:** 76 across 13 crates (all pass on `cargo test --workspace`).

**Migrations on disk:** 6 files in `migrations/sqlite/` (0001–0006). No `checksums.json` manifest yet.

---

## 2. What Needs Implementation

### A. Storage-Sqlite: Real Repository Implementations (T17–T20)

All 5 repository trait methods currently return `Err(StorageError::StorageUnavailable)`. Each needs a real SQL implementation.

**Pattern:** All repos are implemented on `SqliteUnitOfWork` which owns a `tokio::sync::Mutex<SqlTx>`. Access the connection via `self.tx.lock().await`. Use `sqlx::query(...).fetch_optional(&mut *tx).await` for queries and `.execute(&mut *tx).await` for mutations.

**Files to create:** `crates/storage-sqlite/src/repos/` directory with:
- `sources.rs` — implement `SourceRepository` for `SqliteUnitOfWork`
- `provenance.rs` — implement `ProvenanceRepository`
- `audit.rs` — implement `AuditRepository`
- `jobs.rs` — implement `JobRepository`
- `settings.rs` — implement `SettingsRepository`

**Row types are defined in** `crates/storage/src/repository.rs` (SourceRow, SourceVersionRow, etc.) — populate these from sqlx Row results.

**Migration apply function:** Create `crates/storage-sqlite/src/migrate.rs`:
```rust
pub async fn apply_migrations(db_path: &str, migrations_dir: &Path) -> Result<u32, StorageError>
```
Must: create `schema_migrations` table if missing, apply pending `.up.sql` files in order, compute sha256 checksums, record them. Add `sha2 = { workspace = true }` to Cargo.toml.

**Backup function:**
```rust
pub async fn backup(db_path: &str, dest: &str) -> Result<(), StorageError>
```
Must use `VACUUM INTO` — NOT `fs::copy` (ADR-0001 §7).

**Non-negotiable:** write pool `max_connections = 1` (serialized). Read pool `query_only = ON`. No `unsafe` anywhere (workspace forbids it).

---

### B. CLI Binary Entry Point (T48–T51)

The CLI crate has `pub fn dispatch(cli: Cli) -> i32` in lib.rs but no `src/main.rs` binary. Need to create:

**`crates/cli/src/main.rs`:**
```rust
fn main() {
    use clap::Parser;
    let cli = cli::Cli::parse();
    let code = cli::dispatch(cli);
    std::process::exit(code);
}
```

**`crates/cli/Cargo.toml` needs `[bin]` section** or just a `[[bin]]` entry:
```toml
[[bin]]
name = "qai"
path = "src/main.rs"
```

Also need to add `storage-sqlite` as a dependency so `db migrate` can call the real migration apply function.

---

### C. Doctor Engine (T52–T54)

`crates/cli/src/doctor.rs` has `run_checks()` which runs 4 checks. Plan D0.14 requires **14 checks**:

**Already implemented:**
- `configuration.valid` ✅
- `data_dir.writable` ✅
- `security.bind_address_safe` ✅
- `security.tls_policy_consistent` ✅

**Still needed (each is a function returning `CheckResult`):**
- `database.reachable` — open a read connection to the SQLite DB
- `database.migration_current` — compare applied versions vs disk migrations
- `database.integrity_check` — run `PRAGMA integrity_check` (read-only)
- `database.foreign_keys_enabled` — `PRAGMA foreign_keys` check
- `secrets.backend_available` — check configured secret backend exists
- `secrets.refs_resolvable` — verify all secret refs resolve (existence only, no values)
- `jobs.worker_health` — check no jobs stuck in `Running` with expired leases
- `jobs.interrupted_runs` — count `Interrupted` state jobs
- `audit.chain_valid` — run audit verifier
- `sources.manifest_schema_valid` — validate any loaded manifest schemas
- `filesystem.object_store_writable` — probe write to configured object store root
- `observability.subscriber_installed` — check tracing subscriber is set up

**Hard rules:**
- `doctor` opens DB **read-only** via the read-only pool
- Every check emits `Pass | Warn | Fail | Skipped` **plus a remedy and next command**
- `--repair-preview` prints the plan only; no mutations (Phase 1+ executes repairs)
- `--json` output must validate against `docs/schemas/doctor.v1.schema.json`

---

### D. Security Baseline (T43–T46)

`crates/domain/src/security.rs` exists with:
- `canonicalize_and_contain()` ✅
- `is_private_ip()` ✅
- `Limits` struct with `check_download()`, `check_expansion()`, `check_entry_count()` ✅
- `Untrusted<T>` wrapper ✅
- `PolicyDecision` + `default_denying()` ✅
- `SecurityError` with 6 variants ✅

**Still needed:**
- **SSRF guard** (`security::net`): DNS-resolve-then-check pattern. Takes a resolved IP list, checks each against `is_private_ip`. Must block redirects to blocked ranges. Domain allowlist check.
- **Archive safety guard** (`security::archive`): zip-slip block, nested-archive depth cap, no symlink/device entries. The `Limits` struct exists but no `check_archive_entry()` that validates individual archive entries.
- **Input validation** (`security::input`): UTF-8 validation, control-char policy, max field lengths, JSON depth cap.
- **HTML/text sanitization** (`security::sanitize`): Strip scripts/handlers/`javascript:` URIs.
- **Taint marking** is done (`Untrusted<T>`) but needs integration tests proving `.into_inner()` is only used in sanitizer modules.
- **Tests needed** (~20 scenarios): traversal payloads ("../../etc/passwd", "a/../../b", nested, backslash), symlink escape (create tmpdir + symlink), limits edge, private IP matrix (127.0.0.1, 10.0.0.1, 172.16.0.1, 192.168.0.1, 169.254.0.1, fe80::, fc00::, ::1, 8.8.8.8), PolicyDecision defaults.

---

### E. Testkit Crate (T57)

`crates/testkit/` is a 3-line stub. Needs to be a real test fixture crate:

**Cargo.toml deps** (already partially set up):
```toml
[dependencies]
domain = { path = "../domain" }
config = { path = "../config" }
storage = { path = "../storage" }
audit = { path = "../audit" }
sources = { path = "../sources" }
jobs = { path = "../jobs" }
tempfile = { workspace = true }

[dev-dependencies]
serde_json = { workspace = true }
```

**Fixtures to implement in `src/lib.rs`:**
- `temp_dir() -> tempfile::TempDir`
- `temp_db_path(dir) -> String`
- `sample_config() -> config::Config`
- `sample_source_version() -> sources::SourceVersion`
- `sample_job_record() -> storage::repository::JobRecord`
- `MockAuditRepo` struct implementing `audit::AuditRepository` with in-memory `Vec<AuditEvent>`

---

### F. Integration Test Suites (D0.16 — security/integrity/recovery)

Create `crates/testkit/src/lib.rs` with `#[cfg(test)]` modules (test files under `crates/testkit/tests/` also acceptable):

**Required test suites** (per plan §8.1):

1. **`tests/security/secret_leak.rs`** — put sentinel value into every Secret backend, then assert sentinel appears in ZERO bytes of: log output, error messages, CLI output, doctor JSON, audit row serialization. This is a permanent CI gate.

2. **`tests/security/path_guard.rs`** — 40+ payloads against `canonicalize_and_contain`: "../../etc/passwd", "a/../../b", symlink escapes, absolute paths, backslash paths, URL-encoded variants.

3. **`tests/security/archive_guard.rs`** — zip-slip, bomb, symlink-entry, nested-depth all rejected.

4. **`tests/security/ssrf_guard.rs`** — loopback/private/link-local/DNS-rebind/redirect-to-private rejected.

5. **`tests/integrity/canonical_guard.rs`** — Canonical provenance cannot be updated/deleted; writes require `ApprovalToken`; `CanonicalChangeRequest` missing any field is rejected.

6. **`tests/integrity/audit_chain.rs`** — Chain verifies end-to-end; tampered row detected; update/delete aborts.

7. **`tests/sources/state_machine.rs`** — All illegal transitions rejected; `Approved` requires hash+license+report+approver; only one `Active` per source.

8. **`tests/recovery/jobs.rs`** — Lease expiry, crash → `Interrupted`, resume from checkpoint, no double side effect.

9. **`tests/config/precedence.rs`** — Full CLI>Env>File>Defaults matrix; invalid configs rejected with actionable errors.

10. **`tests/cli/*.trycmd`** — Human + `--json` output snapshots for every Phase-0 command.

11. **`tests/db/migrations.rs`** — Fresh migrate, idempotent re-run, checksum mismatch fails, down-migrations restore schema.

---

### G. Outbox, Generations, Tombstones (T61–T67)

These are D0.18 — the relational primitives from ADR-0702.

**What exists:** Migration `0006_outbox_generations_tombstones.up.sql` is on disk with the correct schema. Domain types `CorpusGeneration`, `OutboxEvent`, `Tombstone` need to be added to `crates/domain/src/generation.rs` (or a new module).

**What needs implementing:**
- `CorpusGeneration` struct + allocator that guarantees monotonic allocation in the same write transaction
- `OutboxRepository` trait + wiring into `sources`/`provenance` write paths (every relevant commit inserts an outbox row in the same transaction)
- Generic outbox-relay job (`system.outbox_relay`) that claims events and marks `Dispatched`
- `Tombstone` model + wiring into source deactivation/rollback
- Doctor checks: `outbox.backlog_age`, `outbox.dead_letter_count`, `generations.monotonicity`, `tombstones.unpropagated_count`
- Cross-store consistency test suite (4 tests in plan §8.3):
  - `commit_bounds_outbox.rs` — fault injection proves crash between commit is impossible
  - `outbox_idempotency.rs` — duplicate enqueue with same key yields one event
  - `generation_monotonicity.rs` — number never regresses under 50 concurrent writers
  - `tombstone_before_visibility.rs` — tombstone written before state change visible

**Dependencies:** T64 depends on T37 (job worker pool), T66 depends on T52 (doctor engine).

---

### H. ADRs — Write Missing (T34, T47)

8 ADRs exist: `ADR-0000`, `ADR-0001`, `ADR-0002`, `ADR-0006`, `ADR-0012`, `ADR-0201`, `ADR-0202`, `ADR-0702`.
5 ADRs still missing from the Phase 0 plan:

| ADR | Title | What to write |
|-----|-------|---------------|
| `ADR-0003` | Durable job system | DB-backed leased queue, no external broker; tradeoff analysis |
| `ADR-0004` | Config & precedence | CLI>Env>File>Defaults, interpolation, env mapping |
| `ADR-0005` | Secret storage | OS keychain + encrypted file + env; tradeoff per OS |
| `ADR-0007` | Manifest format & signing | ed25519 detached over canonical JSON; unsigned policy default |
| `ADR-0008` | Provenance representation | One universal table vs per-domain tables; typed attribution |
| `ADR-0009` | Audit integrity | Hash chain + append-only triggers |
| `ADR-0010` | Error taxonomy | QAI-XXX-nnnn namespaces, CLI exit codes 0–70 |
| `ADR-0011` | Observability stack | tracing + metrics, OTLP opt-in, privacy denylist |

Each must use the §48 template with sections: Context, Options, Decision, Accuracy implications, Religious-source implications, Licensing implications, Security implications, Operational implications, Migration strategy, Reversal cost, Status: Accepted.

**Location:** `docs/02-architecture/decisions/ADR-nnn-title.md`

---

### I. Documentation (D0.17, T58)

**Architecture docs** (under `docs/architecture/`):
- `crate-map.md` ✅ (exists)
- `data-layers.md` ✅ (exists)
- `source-lifecycle.md` ✅ (exists)
- `error-codes.md` ✅ (exists)

**Still needed:**
- `hashing-spec.md` — canonical_json_bytes definition, SHA-256 algorithm tag format, NFC policy

**Runbooks** (under `docs/runbooks/` — directory exists but is EMPTY):
- `backup-restore.md` — trigger, steps (VACUUM INTO), verify, rollback risk
- `interrupted-job-recovery.md` — detect Interrupted state, reclaim expired leases, resume from checkpoint
- `audit-chain-break.md` — detect via `qai audit verify`, remediation steps
- `migration-checksum-mismatch.md` — detect via `qai db verify`, recovery procedure
- `quarantine-handling.md` — when to quarantine, how to review, re-approval flow

Each ~20 lines.

**CONTRIBUTING.md** (repo root — does NOT exist):
- PR checklist mapping to Definition of Done (plan §11)
- Error code registry rules
- Secret redaction requirements
- Migration append-only policy

---

### J. Migration Checksums Manifest (T19)

`migrations/sqlite/checksums.json` does not exist. This file records `sha256:<hex>` for every migration file. The `xtask migrate-check` CI gate reads it to verify no applied migration was edited.

**Create it** by running the checksum computation over all 6 `.up.sql` files. Format:
```json
{
  "0001_core.up.sql": "sha256:abcdef...",
  "0002_sources.up.sql": "sha256:...",
  ...
}
```

---

### K. ADR Numbering Reconciliation (T59b)

Per plan §6, ADR files should use the phase-coded scheme:
- `ADR-0001` — relational store (file already named correctly)
- `ADR-0002` — migration strategy (file already exists)
- Verify no filename/content mismatches remain

---

### L. Application Crate

`crates/application/` is a 4-line stub. Per plan §3, this is the orchestration crate that wires domain + storage + config + provenance + audit + jobs + sources. Phase 0 scope: create a minimal `lib.rs` that re-exports the key traits and types from its dependencies, plus a `run()` function skeleton for future CLI/serve integration.

---

### M. Acceptance Criteria Verification (T60)

All 26 ACs in `acceptance.md` need verification. Currently **0/26 verified**. Each needs:
1. An automated test or script that proves it
2. Evidence (test path, CI run URL)
3. Entry in `done.md`

**Which ACs are closest to passing:**
- AC-P0-02 (arch-check mutation test) — just needs the negative test script
- AC-P0-04 (config precedence matrix) — config tests partially cover this
- AC-P0-06 (canonical immutability) — provenance crate tests cover invariants
- AC-P0-08 (state machine transitions) — sources tests cover all transitions
- AC-P0-11 (job chaos tests) — NOT implemented yet (needs testkit fixtures)
- AC-P0-14 (doctor read-only) — NOT implemented (doctor doesn't have DB access yet)
- AC-P0-17 (error code uniqueness) — domain has uniqueness tests for QAI-DOM codes, but no workspace-wide test

---

## 3. Execution Priority Order

**Suggested implementation order (by dependency and impact):**

1. **Storage-sqlite repos** (T17-T20) — everything downstream depends on real DB operations
2. **CLI binary + db migrate** (T48-T50) — enables `qai db migrate` and `qai doctor`
3. **Doctor engine full checks** (T52-T54) — enables AC-P0-14
4. **Testkit fixtures** (T57) — enables integration test suites
5. **Integration tests** (D0.16 suites) — enables verification of 10+ ACs
6. **Security guards** (T43-T46) — SSRF, archive, input validation, tests
7. **Outbox/generations/tombstones** (T61-T67) — relational primitives
8. **ADRs** (T34, T47) — documentation completion
9. **Runbooks + CONTRIBUTING.md** (T58) — docs completion
10. **Migration checksums** (T19) — CI gate completion
11. **Application crate wiring** — orchestration skeleton
12. **AC verification + done.md** (T60) — final exit gate

---

## 4. Key Constraints for Continuing Agent

- **Workspace forbids `unsafe`** (`unsafe_code = "forbid"` in Cargo.toml `[workspace.lints.rust]`)
- **Architecture enforcement:** `cargo xtask arch-check` verifies dependency directions per `xtask/allowlist.toml`. Currently `cli` may depend on: `application`, `config`, `observability`, `server`. `storage-sqlite` may depend on: `domain`, `storage`.
- **No SQLite raw file copy for backup** — must use `VACUUM INTO` (ADR-0001 §7)
- **Migrations are append-only** — never edit existing `.up.sql` files; add new ones
- **All errors implement Diagnostic trait** with remedy + next_command fields
- **Secrets are never stored in SQLite** in Phase 0 (env/OS keychain/encrypted file only)
- **`qai doctor` must be read-only** — never writes to the database
- **Locale:** all timestamps UTC RFC3339; all hashes `sha256:<hex>` lowercase
- **SQLx compile-time checks** require `DATABASE_URL` env var; for CI/testing use `sqlx::query()` (non-macro) to avoid compile-time checks
