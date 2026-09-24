# Phase 1: Foundations - Pattern Map

**Mapped:** 2026-09-24  
**Mode:** brownfield gap closure; reuse the existing Rust workspace, SQLite adapter, configuration loader, audit bridge, and job queue.  
**Files analyzed:** 31 target/conditional files (26 existing seams + 5 focused test/schema additions)  
**Analogs found:** 27 / 31 (4 partial or intentionally new)  
**Tracked-source gate:** every existing analog named below was checked with `git ls-files --error-unmatch`; no gitignored capability or runtime mirror paths are used.

> This map is intentionally seam-oriented. The planner should preserve the existing
> command tree, storage ports, hash-chain implementation, and job queue rather than
> create a parallel foundation service. Rows marked **conditional** are only needed
> if the implementation contract requires a new fixture, schema column, or policy
> representation; they are not permission to expand Phase 1 into a greenfield crate.

## Work-package map

| Work package | Research gaps | Primary responsibility | Shared seams |
|---|---|---|---|
| WP-1 — Operator bootstrap | FND-01, FND-02, FND-03 | Effective `(Config, OriginMap)` at dispatch; explicit migration/readiness; genuinely read-only doctor | `crates/cli/src/lib.rs`, `crates/application/src/db.rs`, `crates/application/src/quran_cli.rs` |
| WP-2 — Audited durable writes | FND-04 | Domain mutation + provenance + audit + outbox in one `UnitOfWork`; persisted audit verifier reused by doctor | `crates/storage/src/workflows.rs`, `crates/application/src/audit_bridge.rs`, `crates/application/src/quran.rs` |
| WP-3 — Durable job lifecycle | FND-05 | Immediate durable checkpoints, per-kind retry, explicit retry/cancel, cooperative terminal outcomes, lifecycle audit | `crates/jobs/src/worker.rs`, `crates/jobs/src/queue.rs`, `crates/storage-sqlite/src/lib.rs` |
| WP-4 — Long-lived worker host | FND-06 | `qai serve` owns worker tasks; one-shot commands enqueue/report only; graceful shutdown | `crates/cli/src/lib.rs`, `crates/application/src/job_queue.rs`, `crates/application/src/quran_cli.rs` |
| WP-5 — Architecture enforcement | FND-07 | Check registry and git dependencies against an explicit external policy; keep CI mutation coverage | `xtask/src/arch.rs`, `xtask/allowlist.toml` |

## File Classification

### WP-1 — Operator bootstrap

| New/Modified File | Role | Data Flow | Status | Closest Analog | Match Quality |
|---|---|---|---|---|---|
| `crates/cli/src/lib.rs` | controller | request-response | modify | `crates/config/src/lib.rs:223-260`; current `dispatch` at `211-353` | exact seam + reusable loader |
| `crates/cli/src/doctor.rs` | utility | file-I/O / diagnostics | modify | `crates/application/src/quran_doctor.rs:83-136`; `storage-sqlite` `open_read_only` at `106-123` | role-match, stronger read-only analog |
| `crates/application/src/db.rs` | service | file-I/O / readiness | modify | same file `migration_status` at `66-84`, `probe_database` at `129-200` | exact |
| `crates/application/src/quran_cli.rs` | controller | request-response | modify | same file `CommandOutput` at `41-65`, `cmd_doctor_quran` at `1414-1457` | exact |
| `crates/cli/tests/catalog.rs` | test | request-response / E2E | modify or reuse | same file real-binary helper at `13-35`, assertions at `38-89` | exact |
| `crates/cli/tests/foundation.rs` | test | request-response / E2E | new (recommended) | `crates/cli/tests/catalog.rs` plus `crates/cli/tests/quran.rs:11-20` | role-match |
| `crates/testkit/tests/config_precedence.rs` | test | transform / request-response | modify | same file cases at `9-74` | exact |
| `crates/testkit/src/lib.rs` | test utility | transform / fixtures | modify if shared helpers are needed | `sample_config` at `76-100`, `sample_job_record` at `151-183` | exact |

### WP-2 — Audited durable writes

| New/Modified File | Role | Data Flow | Status | Closest Analog | Match Quality |
|---|---|---|---|---|---|
| `crates/storage/src/workflows.rs` | service | CRUD / event-driven | modify | same file `record_source_activation` at `27-48`, failure matrix at `472-537` | exact |
| `crates/application/src/audit_bridge.rs` | service | event-driven | modify | same file `append_audit_event` at `200-232`, `verify_persisted_audit` at `242-271` | exact |
| `crates/application/src/quran.rs` | service | CRUD / event-driven | modify | `audit_activation` at `262-294`, `rollback_edition` at `429-467` | exact |
| `crates/application/src/job_queue.rs` | service | event-driven | modify (shared WP-3/4) | same file `SqliteJobQueue` at `19-135` | exact |
| `crates/storage/src/lib.rs` | model / port | CRUD | conditional | `UnitOfWork` trait at `116-146` | exact port seam |
| `crates/storage/src/repository.rs` | model / port | CRUD / event-driven | modify if job/lifecycle signatures change | `JobRepository` at `270-380` | exact |
| `crates/storage-sqlite/src/lib.rs` | config / adapter | CRUD / event-driven | modify (shared WP-3) | shared transaction at `230-307`; job SQL at `818-1037` | exact |
| `crates/application/tests/phase1_foundation.rs` | test | event-driven | new (recommended) | `crates/application/src/audit_bridge.rs:299-343`, `crates/application/tests/common/mod.rs:50-117` | role-match |
| `crates/storage-sqlite/tests/phase1_jobs_audit.rs` | test | CRUD / event-driven | new (recommended) | `commit_bounds_outbox.rs:13-74`, `integrity_audit.rs:28-80` | role-match |
| `crates/storage-sqlite/tests/commit_bounds_outbox.rs` | test | CRUD | modify if adding audit coverage | same file committed/rollback cases at `13-74` | exact |
| `crates/storage-sqlite/tests/integrity_audit.rs` | test | event-driven | modify if adding lifecycle assertions | same file append-only/tamper cases at `28-80` | exact |
| `migrations/sqlite/0020_*.up.sql` | migration | file-I/O / schema | conditional | `0004_jobs.up.sql`, `0005_audit.up.sql`, `storage-sqlite/src/migrate.rs:163-219` | partial; no exact future schema |

### WP-3 — Durable job lifecycle

| New/Modified File | Role | Data Flow | Status | Closest Analog | Match Quality |
|---|---|---|---|---|---|
| `crates/jobs/src/lib.rs` | model / service | event-driven | modify | `JobState` at `90-114`, `JobContext`/`JobRepository` at `225-379` | exact |
| `crates/jobs/src/queue.rs` | service | event-driven | modify | queue port at `16-68`, in-memory transitions at `107-253` | exact |
| `crates/jobs/src/worker.rs` | service | event-driven / worker | modify | `run_once` at `99-173`, `handle_failure` at `187-221` | exact |
| `crates/jobs/src/registry.rs` | service / config | event-driven | modify if policy is registry-backed | `HandlerRegistry` at `11-49` | exact |
| `crates/storage-sqlite/tests/recovery_jobs.rs` | test | event-driven | modify/extend | same file crash/recovery cases at `33-135` | exact |
| `crates/application/src/job_queue.rs` | service | event-driven | modify (shared WP-2/4) | each adapter operation currently opens/commits one UoW at `35-125` | exact |
| `crates/storage-sqlite/src/lib.rs` | adapter | CRUD / event-driven | modify (shared WP-2) | `SqliteJobRepository` methods at `818-1037` | exact |
| `crates/storage/src/repository.rs` | model / port | CRUD / event-driven | modify (shared WP-2) | repository trait at `270-380` | exact |
| `crates/cli/src/lib.rs` | controller | request-response | modify (shared WP-1/4) | `handle_job` at `656-690`, explicit refusal currently | exact |

### WP-4 — Long-lived worker host

| New/Modified File | Role | Data Flow | Status | Closest Analog | Match Quality |
|---|---|---|---|---|---|
| `crates/cli/src/quran.rs` | controller | request-response | modify if command wording/options change | same file `QuranAction` at `12-201` and handler dispatch later in the file | exact |
| `crates/application/src/lib.rs` | provider / composition root | event-driven | modify | `application::run` at `39-66`; server composition in `cli/src/lib.rs:301-345` | role-match |
| `crates/cli/tests/quran.rs` | test | request-response / E2E | modify if host/enqueue cases are added | `run_cases` at `11-20` and existing snapshot entry points at `22-50` | exact |
| `crates/application/src/quran_cli.rs` | controller | request-response | modify (shared WP-1) | `cmd_import` at `569-671`; current `run_import_job` call at `655` | exact |
| `crates/application/src/quran.rs` | service | event-driven | modify (shared WP-2/3) | `run_import_job` at `1124-1169` | exact |
| `crates/application/src/job_queue.rs` | service | event-driven | modify (shared WP-2/3) | `build_worker` at `128-135` | exact |

### WP-5 — Architecture enforcement

| New/Modified File | Role | Data Flow | Status | Closest Analog | Match Quality |
|---|---|---|---|---|---|
| `xtask/src/arch.rs` | utility / build gate | transform | modify | same file `Dependency`, `Allowlist`, `violations` at `17-164`; synthetic tests at `166-331` | exact |
| `xtask/allowlist.toml` | config | transform / build-time | modify | current workspace policy at `1-101`; parser at `xtask/src/arch.rs:47-85` | partial; external policy is new |
| `xtask/src/arch.rs` inline test module | test | transform / build-time | existing inline module, extend | synthetic `package`/`meta` helpers at `xtask/src/arch.rs:180-213` | exact |

## Pattern Assignments

### WP-1 / `crates/cli/src/lib.rs` (controller, request-response)

**Analog:** `crates/config/src/lib.rs:219-260` and `crates/cli/src/lib.rs:211-353`.

**Imports pattern.** Keep the CLI's existing ordering and add only the configuration
origin types needed by the effective-config envelope:

```rust
use clap::{Parser, Subcommand};
use config::{Config, OriginMap, ValueOrigin};
```

**Configuration envelope pattern** (`crates/config/src/lib.rs:223-260`):

```rust
pub fn load(
    config_path: Option<&PathBuf>,
    env_prefix: &str,
    cli_overrides: &BTreeMap<String, String>,
) -> Result<(Self, OriginMap), ConfigError> {
    let mut config: Self = Self::default();
    let mut origins = OriginMap::new();
    mark_defaults(&mut origins);
    // file -> env -> CLI -> interpolation -> validation
    Ok((config, origins))
}
```

Do not reconstruct a default configuration inside `handle_config_show`,
`handle_config_get`, or `handle_config_validate`. The current anti-pattern is
visible at `crates/cli/src/lib.rs:421-450`:

```rust
fn handle_config_show(explain: bool, defaults: bool, json: bool) -> i32 {
    let cfg = Config::default();
    // ... ignores explain, loaded file, and env precedence
}
```

**Dispatch pattern.** The existing `dispatch` is the correct single fan-out point;
replace only the initial `Config` value with a loaded `(Config, OriginMap)` pair and
pass both to handlers that need them. Preserve the stable top-level `Commands`
enum (`config`, `db`, `doctor`, `job`, `audit`, `secret`, `source`) and central exit
mapping from `crates/cli/src/exit_code.rs`.

**Output/redaction pattern** (`crates/cli/src/lib.rs:411-419`):

```rust
if json {
    let mut value = serde_json::to_value(cfg).unwrap_or(serde_json::Value::Null);
    application::redaction::redact_json_value(&mut value);
    serde_json::to_string_pretty(&value).unwrap_or_else(|_| "null".to_string())
} else {
    application::redaction::redact_text(&format!("{cfg:?}")).into_owned()
}
```

Extend the JSON envelope with origins, but redact before printing. Keep the
`ValueOrigin` spelling stable (`default`, `file ...`, `env ...`, `cli ...`) from
`crates/config/src/origin.rs:4-27`.

**Validation/error pattern.** Reuse `Config::validate` and map
`ConfigError::Validation` to `exit_code::VALIDATION`; do not add a new config error
namespace. The existing mapper is `crates/cli/src/exit_code.rs:22-31`.

**Tests.** Extend the redaction test at `crates/cli/src/lib.rs:816-841` and use
`crates/testkit/tests/config_precedence.rs:9-74` for file/env/CLI cases. A CLI test
must exercise a real file plus environment override and assert the effective value,
origin, and secret redaction—not only `Config::default()`.

---

### WP-1 / `crates/cli/src/doctor.rs` (utility, file-I/O)

**Analogs:** `crates/cli/src/doctor.rs:43-99` for the result type and
`crates/application/src/quran_doctor.rs:83-136` for a rollback-based read-only
check flow.

**Check-result pattern:**

```rust
pub struct CheckResult {
    pub id: &'static str,
    pub status: CheckStatus,
    pub summary: String,
    pub remedy: Option<String>,
    pub next_command: Option<String>,
}
```

Every non-passing result must carry a remedy and next command; the existing test
`every_check_emits_a_next_command_when_not_passing` at
`crates/cli/src/doctor.rs:779-787` is the regression contract.

**Read-only pattern.** Reuse `SqliteDatabase::open_read_only`
(`crates/storage-sqlite/src/lib.rs:103-123`) and, when a `UnitOfWork` is needed for
repository reads, explicitly roll it back as `quran_doctor` does at
`crates/application/src/quran_doctor.rs:83-136`.

The current write probes are the gap to replace:

```rust
// crates/cli/src/doctor.rs:237-254
std::fs::create_dir_all(&dir)
    .and_then(|_| std::fs::write(dir.join(".qai-probe"), b"q"))
```

Use metadata/existence/permission inspection only. A fresh data directory must
remain absent after doctor; “not proven writable” is an actionable warning, not
permission to create a probe file.

**Audit authority pattern.** Replace the row-count branch at
`crates/cli/src/doctor.rs:441-458` with `application::audit_bridge::verify_persisted_audit`.
The verifier already checks sequence continuity, previous links, and recomputed
hashes (`crates/application/src/audit_bridge.rs:254-270`). Doctor should render its
`valid`, `gaps`, and `tampered_sequences` values and point to `qai audit verify`.

**JSON/doctor composition.** Preserve the single merged `checks` array produced by
`doctor_report` at `crates/cli/src/doctor.rs:715-753`; redact the complete document
at the emission boundary. Do not print a second standalone audit document.

**Tests.** Add fresh-tempdir no-side-effect coverage next to
`doctor_is_read_only` at `crates/cli/src/doctor.rs:870-891`, and a tampered-chain
case using the fault-injection style in
`crates/application/src/audit_bridge.rs:299-343`.

---

### WP-1 / `crates/application/src/db.rs` (service, file-I/O)

**Analog:** the same file's `migration_status` (`66-84`) and `probe_database`
(`129-200`), with `SqliteDatabase::open_read_only` as the storage primitive.

```rust
pub async fn probe_database(cfg: &Config) -> DbProbe {
    let mut probe = DbProbe::default();
    let db = match SqliteDatabase::open_read_only(&cfg.storage.sqlite.path).await {
        Ok(db) => db,
        Err(e) => {
            probe.error = Some(format!("{e}"));
            return probe;
        }
    };
    // PRAGMA/table probes only; never a creating constructor
    probe
}
```

The existing migration status already distinguishes applied/latest/pending, but
its error path currently does not produce a friendly missing-database outcome.
Add a shared readiness result around this logic rather than string-matching errors
in every CLI handler. It should distinguish: missing database, unreadable file,
pending migrations, checksum mismatch, and healthy/current database.

**Catalog/redaction pattern.** Reuse `catalog_page` and the redacted JSON helpers
at `crates/application/src/db.rs:202-223,289-330`. The same `open_read_only` pattern
is already used by all ordinary listings (`list_sources`, `list_jobs`, `get_job`,
`list_audit_events`), so a new listing helper should follow the same return shape.

**Migration seam.** `migrate_database` is the only explicit creation/mutation
entry point (`crates/application/src/db.rs:52-55`); ordinary commands must not call
it implicitly. The migration runner already verifies applied checksums before
writes (`crates/storage-sqlite/src/migrate.rs:163-219`).

**Tests.** Extend the inline `tempdir` + real-migration tests at
`crates/application/src/db.rs:374-443` with missing, pending, current, and no-side-
effect cases. Use `common::migrations_dir()`-style disk discovery rather than a
hard-coded migration count.

---

### WP-1 / `crates/application/src/quran_cli.rs` (controller, request-response)

**Analog:** `CommandOutput` and `open_reader` at
`crates/application/src/quran_cli.rs:41-65,127-134`, plus the read-only
`cmd_doctor_quran` path at `1414-1457`.

Keep the two-shape output contract:

```rust
pub struct CommandOutput {
    pub exit: i32,
    pub human: String,
    pub json: serde_json::Value,
}
```

The current shared opener is the mutation seam:

```rust
// current: crates/application/src/quran_cli.rs:71-73
async fn open_db(db_path: &str) -> Result<SqliteDatabase, StorageError> {
    SqliteDatabase::new(db_path, 4, true).await
}
```

`SqliteDatabase::new` creates the parent directory and database
(`crates/storage-sqlite/src/lib.rs:59-100`). Replace the ordinary-read call path
with an existing-database/readiness helper, while retaining `new` only behind the
explicit migration/creation flow. Do not change the large command surface
unrelated to the Phase 1 seam; concurrent Phase-2 index work in this file must be
preserved.

**Doctor/import overlap.** `cmd_doctor_quran` already chooses `open_read_only`
and returns a typed `CommandOutput`; use that pattern for any new doctor/readiness
surface. The `cmd_import` dry-run validation at `569-610` is the correct
pre-enqueue validation boundary, but its final call at `655-669` currently invokes
an inline worker (see WP-4).

---

### WP-1 / `crates/cli/tests/catalog.rs` and `crates/cli/tests/foundation.rs` (tests, E2E)

**Analog:** the real-binary helper in `crates/cli/tests/catalog.rs:13-35`:

```rust
fn qai(dir: &std::path::Path, args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_qai"));
    cmd.args(["--data-dir", dir.to_str().unwrap()]);
    cmd.args(args);
    for (key, value) in envs { cmd.env(key, value); }
    cmd.output().expect("run qai")
}
```

Use a fresh `tempfile::tempdir()` per test, migrate only where the case needs a
database, and assert both exit codes and JSON keys. The new `foundation.rs` should
cover the seven stable groups, effective config/origins, missing/pending migration
remedies, no DB creation from ordinary reads, audit verification, and secret/job
redaction. Reuse `crates/cli/tests/quran.rs:11-20` for trycmd setup:

```rust
trycmd::TestCases::new()
    .default_bin_name("qai")
    .env("QAI_DATA_DIR", dir.path().to_str().unwrap())
    .case("tests/foundation/*.trycmd");
```

If the planner prefers one harness instead of a new file, add these cases to
`catalog.rs`; do not create a second ad-hoc CLI runner.

---

### WP-2 / `crates/storage/src/workflows.rs` (service, CRUD/event-driven)

**Analog:** same file `record_source_activation` at `27-48` and its failure matrix
at `472-537`.

```rust
pub async fn record_source_activation(
    uow: &mut dyn UnitOfWork,
    source_version_id: &str,
    scope: &str,
    subject_urn: &str,
) -> Result<String, StorageError> {
    let generation = uow.outbox().allocate_generation(scope, "source_activated").await?;
    uow.outbox().enqueue(/* same event */).await?;
    uow.sources().transition_state(source_version_id, "Indexing", "Active").await?;
    Ok(generation.id)
}
```

The existing helpers deliberately operate through `&mut dyn UnitOfWork` and commit
once at the caller. Keep that boundary. Add audit/provenance/outbox composition at
the application boundary or through an injected callback; do not make the
backend-neutral `storage` workflow depend on `application` or `audit` crates.

Use the existing call-order test as the model for new failure-injection cases:

```rust
for fail in [FailAt::Allocate, FailAt::Enqueue, FailAt::Transition] {
    let err = record_source_activation(&mut uow, "ver-1", "s", "urn:a").await;
    assert!(err.is_err());
    Box::new(uow).rollback().await.unwrap();
}
```

Extend the matrix to cover audit append and outbox failure, and assert that no
domain row, provenance row, audit row, or outbox row survives rollback.

---

### WP-2 / `crates/application/src/audit_bridge.rs` (service, event-driven)

**Analog:** the complete bridge implementation in the same file.

**Same-transaction append pattern** (`200-232`):

```rust
pub async fn append_audit_event(
    uow: &mut dyn storage::UnitOfWork,
    mut event: AuditEvent,
) -> Result<AuditEvent, AuditError> {
    let (sequence, prev_hash) = {
        let bridge = StorageAuditBridge::new(uow.audit());
        let sequence = bridge.latest_sequence().await? + 1;
        let prev_hash = /* prior chain hash or genesis */;
        (sequence, prev_hash)
    };
    event.sequence = sequence;
    event.occurred_at = Timestamp::now();
    event.prev_chain_hash = prev_hash.clone();
    event.chain_hash = HashChainWriter::compute_chain_hash(&prev_hash, &event);
    let row = StorageAuditBridge::to_row(&event)?;
    uow.audit().append(row).await?;
    Ok(event)
}
```

**Persisted verification pattern** (`242-270`): read ordered events from the
read-only database, recompute each chain hash and predecessor link, collect gaps
and tampered sequences, set `valid`, and roll back the snapshot. This is the
authority for both `qai audit verify` and doctor's `audit.chain_valid` check.

**Error/redaction pattern.** Keep action/outcome strings in snake_case, map storage
failures through `AuditError::Storage`, and use the existing `AuditVerificationReport`
shape. Do not introduce a second hash format or a row-count integrity check.

---

### WP-2 / `crates/application/src/quran.rs` (service, CRUD/event-driven)

**Analogs:** `audit_activation` at `262-294`, `activate_edition` at `296-343`,
and `rollback_edition` at `429-467`.

```rust
let mut uow = db.write().await.map_err(ActivationError::storage)?;
check_approval(&mut *uow, approval_id, &expected_subject).await?;
let generation = uow.quran().activate_edition(/* ... */).await?;
audit_activation(
    &mut *uow,
    AuditAction::SourceActivated,
    &expected_subject,
    invoked_by,
    &edition_id,
    generation,
).await?;
uow.commit().await.map_err(ActivationError::storage)?;
```

The importer's staging audit at `70-131` is another same-transaction example. Keep
the approval gate (`check_approval` at `238-260`) and never move approval token
minting into the importer. Any new import/source lifecycle write should use the
same domain mutation → provenance/audit/outbox → one commit order, with rollback
on every failure.

Use `ActivationError`'s existing `code`, `remedy`, and `next_command` implementation
(`196-235`) for new operator-facing errors.

---

### WP-2 / `crates/application/src/job_queue.rs` and `crates/storage-sqlite/src/lib.rs`

**Analogs:** the current one-operation-per-UoW adapter at
`crates/application/src/job_queue.rs:35-125` and the shared transaction at
`crates/storage-sqlite/src/lib.rs:230-307`.

The current adapter pattern is:

```rust
async fn enqueue(&self, job: JobRecord) -> Result<(), JobError> {
    let mut uow = self.db.write().await.map_err(to_err)?;
    uow.jobs().enqueue(job).await.map_err(to_err)?;
    uow.commit().await.map_err(to_err)
}
```

That is the seam to replace for lifecycle audit: enqueue/lease/checkpoint/retry/
cancel/terminal transitions must have a composition point that can append the
required audit event in the same UoW. Keep `SqliteJobQueue` as the production
adapter; do not introduce a second queue implementation.

The shared transaction itself is already correct:

```rust
async fn commit(self: Box<Self>) -> Result<(), StorageError> {
    let Self { tx, sources, provenance, audit, jobs, settings, outbox, quran } = *self;
    drop((sources, provenance, audit, jobs, settings, outbox, quran));
    let mutex = Arc::try_unwrap(tx).map_err(|_| StorageError::StorageBusy)?;
    mutex.into_inner().commit().await
        .map_err(|_| StorageError::StorageUnavailable)
}
```

Every new repository handle must be included in the destructuring/drop list and
must share the same `SharedTx`.

---

### WP-2 / `crates/storage/src/repository.rs` and `crates/storage/src/lib.rs`

**Analogs:** the existing ports, not the SQLite implementation.

`crates/storage/src/lib.rs:116-146` defines the `UnitOfWork` port and explicitly
exposes `sources`, `provenance`, `audit`, `jobs`, `settings`, `outbox`, and
`quran`. Preserve that narrow port. `crates/storage/src/repository.rs:270-380`
defines the current `JobRepository` methods and `JobRecord` fields. If retry policy,
named checkpoints, or lifecycle event data require new fields/methods, change the
port and both in-memory/fake and SQLite implementations together; do not make the
CLI depend on SQL or concrete SQLite types.

For any new `JobRecord` field, update all fixture constructors in
`crates/testkit/src/lib.rs:151-183`, `crates/jobs/src/lib.rs:35-54`, and the
storage/job tests in the same wave.

---

### WP-2 / Tests and migration seams

**Real SQLite test pattern:** `crates/storage-sqlite/tests/commit_bounds_outbox.rs:13-74`
uses `common::fixture()`, one UoW, explicit commit/drop, then queries the file.
Copy this for rollback-on-audit-failure and rollback-on-outbox-failure cases.
`crates/storage-sqlite/tests/integrity_audit.rs:28-80` is the append-only/tamper
pattern for lifecycle assertions.

**Audit tamper pattern:** the application bridge test at
`crates/application/src/audit_bridge.rs:299-343` drops the update trigger, mutates
a row, and asserts `tampered_sequences`. Reuse it for doctor and acceptance tests.

**Migration pattern:** if a durable field cannot fit the existing `jobs` JSON/event
columns, add a new append-only `migrations/sqlite/00NN_*.up.sql` and update
`migrations/sqlite/checksums.json` through the existing migration tooling. Copy the
schema/trigger style from `0004_jobs.up.sql:1-42` and `0005_audit.up.sql:1-26`; do
not edit applied migrations. The exact new migration is intentionally not named in
research and has no exact analog.

---

### WP-3 / `crates/jobs/src/lib.rs` (model/service, event-driven)

**Analogs:** same file `JobState` at `90-114` and `JobContext` at `225-379`.

```rust
pub enum JobState {
    Queued, Leased, Running, Checkpointed, Succeeded,
    Failed, Cancelled, Interrupted, DeadLettered,
}
```

The handler contract is the stable integration seam:

```rust
pub trait JobHandler: Send + Sync {
    fn kind(&self) -> JobKind;
    fn payload_schema(&self) -> &'static str;
    fn is_idempotent(&self) -> bool;
    async fn run(
        &self,
        ctx: JobContext,
        payload: serde_json::Value,
    ) -> Result<JobOutcome, JobError>;
}
```

`JobContext::checkpoint` currently writes only an in-memory mutex
(`crates/jobs/src/lib.rs:292-326`). The new contract must make a named checkpoint
persist immediately through the queue/store at a resumable boundary; preserve
`checkpoint_sink` only as a compatibility view if needed. Add explicit terminal
cancellation outcome data (completed-before-request, cancelled-at-boundary,
missed-boundary) without changing unrelated job kinds.

Use the existing `JobError::code` and `is_retryable` implementation
(`163-187`) for stable `QAI-JOB-*` diagnostics.

---

### WP-3 / `crates/jobs/src/worker.rs` (service, event-driven/worker)

**Analog:** same file `run_once` at `99-173`, `handle_failure` at `187-221`, and
`watchdog` at `224-239`.

Preserve the sequence: claim → validate payload → observe cancellation → run
handler under watchdog → persist terminal state. The current failure branch is the
specific gap:

```rust
// current: only copied on failure, lines 149-170
let recorded = checkpoint_sink.lock().ok().and_then(|slot| slot.clone());
if let Some(cp) = recorded {
    self.queue.checkpoint(&job.id, None, Some(cp)).await?;
}
if !handler.is_idempotent() {
    self.queue.finish(&job.id, JobState::DeadLettered, /* ... */).await?;
    return Ok(WorkerOutcome::DeadLettered { job_id: job.id });
}
self.handle_failure(&job).await
```

Move checkpoint persistence to the handler boundary callback/contract, and use the
job kind's policy for retryability and max attempts instead of only
`WorkerConfig.max_attempts` (`20-45,187-205`). Keep bounded exponential backoff
and deterministic jitter in `backoff` (`207-221`); do not add a new broker or
randomness dependency.

The existing watchdog is the cancellation polling primitive:

```rust
if queue.cancel_requested(&job_id).await.unwrap_or(false) {
    cancel.store(true, Ordering::SeqCst);
}
let _ = queue.heartbeat(&job_id, &owner, lease).await;
```

A cancellation request must be durable before the CLI returns, observed at named
checkpoints, and reflected in a terminal state plus lifecycle audit event.

---

### WP-3 / `crates/jobs/src/queue.rs` and `crates/storage-sqlite/src/lib.rs`

**Analogs:** queue port at `crates/jobs/src/queue.rs:16-68`, in-memory transitions
at `107-253`, and SQLite job SQL at `crates/storage-sqlite/src/lib.rs:818-1037`.

Keep the port backend-neutral. Add only the operations needed for explicit retry,
durable cancellation request, named checkpoint persistence, and inspectable
terminal outcome. The current in-memory queue is a useful behavioral oracle:

```rust
let claimable = matches!(job.state.as_str(),
    "Queued" | "Interrupted" | "Checkpointed")
    && rfc3339_le(&job.available_at, now);
```

The SQLite implementation already has the durable primitives to preserve:
`claim_next` includes `Interrupted`/`Checkpointed` and increments attempts
(`850-939`), `cancel` sets `cancel_requested` (`1010-1021`), and `checkpoint`
updates `progress_json`/`checkpoint_json` (`1023-1037`). Add failure/ownership
checks at the repository boundary; do not clear a lease from a CLI process
without an auditable request.

---

### WP-3 / `crates/application/src/job_queue.rs`, `crates/jobs/src/registry.rs`, and CLI job controls

**Analogs:** `build_worker` at `crates/application/src/job_queue.rs:128-135`,
`HandlerRegistry` at `crates/jobs/src/registry.rs:11-49`, and `handle_job` at
`crates/cli/src/lib.rs:656-690`.

Keep `HandlerRegistry` as the kind-to-handler composition point. If per-kind retry
policy is registry-backed, expose it beside `kind`/`payload_schema` rather than
adding a second handler registry. The current CLI refusal is the explicit seam:

```rust
JobAction::Cancel { .. } => {
    eprintln!("refusing `qai job cancel`: ...");
    exit_code::USAGE
}
```

Replace it with a durable request operation returning one of the stable exit
codes. Add `Retry` to `JobAction` only as a foundation CLI control; use
`storage_error_code` (`crates/cli/src/lib.rs:603-610`) for `NotFound`,
`MigrationRequired`, and checksum mismatches. The JSON shape from
`application::db::get_job` (`274-329`) should expose attempts, max attempts,
state, cancel flag, and redacted checkpoint/payload fields.

---

### WP-3 / `crates/storage-sqlite/tests/recovery_jobs.rs` (test, event-driven)

**Analog:** same file `expired_lease_marks_interrupted_and_resumes_from_checkpoint`
at `52-90` and `cancel_is_durably_recorded` at `125-135`.

Extend these real-SQLite tests rather than replacing them. Add cases for:

- checkpoint visible after a successful first stage and preserved after lease expiry;
- explicit retry resets only the intended attempts/availability fields;
- per-kind max attempts and exhausted failed state;
- cancellation before start, during a checkpoint boundary, and after completion;
- lifecycle audit rows for enqueue, lease, completion, failure, and cancellation;
- ownership-safe heartbeat/cancel behavior.

Keep the real database fixture and SQL fault injection; do not mock SQLite for
these guarantees.

---

### WP-4 / `crates/application/src/quran.rs` and `crates/application/src/quran_cli.rs`

**Analogs:** `run_import_job` at `crates/application/src/quran.rs:1124-1169` and
`cmd_import` at `crates/application/src/quran_cli.rs:569-671`.

The current one-shot path is explicit:

```rust
let worker = Worker::new(queue.clone(), registry, "qai-cli");
worker.run_until_idle().await?;
```

This is the behavior to remove from one-shot command execution. Preserve the
existing `ImportInput`, deterministic idempotency key, payload validation, and
staging semantics. The new command should enqueue a `JobRecord`, return its job
ID and inspectable state, and leave execution to the long-lived host. Do not call
`run_until_idle` from `cmd_import` or from any other one-shot CLI path.

The Quran handler at `crates/application/src/quran.rs:70-131` is still the correct
handler analog: parse/validate input, use `ctx.cancel_flag()`, persist checkpoints
at named boundaries, and commit domain result + provenance/audit in one UoW.

---

### WP-4 / `crates/cli/src/lib.rs`, `crates/application/src/lib.rs`, and `crates/server/src/api.rs`

**Analogs:** the current `Serve` assembly at `crates/cli/src/lib.rs:301-345`,
`application::run` at `crates/application/src/lib.rs:39-66`, and the long-lived
server entry at `crates/server/src/api.rs:1323-1330`.

The current serve path constructs a runtime and reader/search backends, then awaits
the HTTP server; it does not build or supervise a worker. Reuse that composition
shape, but add a worker host with explicit lifetime. The worker loop should reuse
`jobs::Worker::run_once` and the SQLite queue adapter; one-shot commands must only
enqueue.

There is no existing graceful worker-host implementation or signal abstraction in
the repository. The server serve function is only a long-lived-task analog, not a
complete shutdown pattern. Use the pinned Tokio features already present in
`Cargo.toml` (`rt-multi-thread`, `macros`, `time`, `sync`, `signal`, `net`); do not
add a broker or new utility crate. A host should signal tasks, await final
persistence, and then return a stable CLI exit code.

---

### WP-4 / `crates/cli/src/quran.rs` and `crates/cli/tests/quran.rs`

**Analogs:** `QuranAction::Import` at `crates/cli/src/quran.rs:68-78` and
`run_cases` at `crates/cli/tests/quran.rs:11-20`.

Change command wording/JSON only at the CLI boundary; keep all database access in
`application::quran_cli`. A host test should prove: enqueue returns a job ID;
`qai serve` processes it; a one-shot import does not synchronously drain a worker;
and shutdown waits for the worker to finish or persist a safe terminal state.

Use separate temp directories for parallel trycmd cases, as documented at
`crates/cli/tests/quran.rs:3-9`. The existing Phase-2 changes in this file are
unrelated; preserve them.

---

### WP-5 / `xtask/src/arch.rs` (utility/build gate, transform)

**Analog:** same file `Dependency`/`Allowlist`/`violations` at `17-164` and
synthetic mutation tests at `166-331`.

Current data model:

```rust
pub struct Dependency {
    pub name: String,
    pub source: Option<String>, // registry/git sources are represented here
}

pub fn violations(meta: &Metadata, allowlist: &Allowlist) -> Vec<String> {
    // current implementation only collects source.is_none() path edges
}
```

The new policy must classify every dependency source, not only path edges. Keep
`violations` pure and return sorted, human-readable forbidden-edge strings. Use
the existing synthetic JSON helpers:

```rust
fn package(id: &str, name: &str, deps: Vec<(&str, Option<&str>)>) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": name,
        "dependencies": deps.into_iter()
            .map(|(n, src)| serde_json::json!({ "name": n, "source": src }))
            .collect::<Vec<_>>(),
    })
}
```

Add a synthetic registry or git edge that is explicitly denied and assert
`violations` catches it. Keep the current path-edge tests and the
`server_allowlist_matches_od14_decision` test at `299-331`.

**CI seam:** `.github/workflows/ci.yml:47-64` already runs `arch-check` and
`migrate-check`; retain the gate and make the unit/mutation test fail if a
forbidden external edge is accepted. No CI workflow rewrite is implied.

---

### WP-5 / `xtask/allowlist.toml` and inline architecture tests

**Analogs:** current workspace policy at `xtask/allowlist.toml:1-55` and the
parser at `xtask/src/arch.rs:47-85`.

The checked-in file currently reserves `external.allow` but does not consume it:

```toml
# Semantics:
#   workspace.allow   = workspace crates whose EDGE to this crate is legal.
#   external.allow    = (reserved) external crates permitted; unused for now.
#   workspace.deny    = workspace crates that MUST NOT be depended on.
```

Represent the external allow/deny policy explicitly and document whether the rule
is per-crate or global. Keep the existing fail-closed behavior for unknown crates.
The exact external list is intentionally a planner/owner decision; do not infer a
new package or dependency from this phase.

An inline `xtask/src/arch.rs` test is the established seam. Keep the synthetic registry/git cases in that existing module; the project has no separate xtask integration-test target and no new test package is needed.

## Shared Patterns

### 1. Layered composition and imports

**Sources:** `crates/application/src/lib.rs:1-31`, `crates/cli/src/lib.rs:7-15`.

The CLI may depend on `application`, `config`, `observability`, and `server`; it
must not import `storage-sqlite` directly. Put database lifecycle, worker assembly,
and command-specific services in `application`. Keep the existing import order:
std/core, external crates, workspace crates, then `crate`/`super`.

### 2. Effective configuration, origins, and redaction

**Sources:** `crates/config/src/lib.rs:223-260`,
`crates/config/src/origin.rs:4-68`, `crates/cli/src/lib.rs:407-419`.

Load once, preserve `(Config, OriginMap)`, and redact at the output boundary. Do
not create a second file/environment merge in handlers. The effective value and
its origin must travel together through all foundation commands.

### 3. Read-only diagnostics and explicit migration

**Sources:** `crates/storage-sqlite/src/lib.rs:103-123`,
`crates/application/src/db.rs:66-84,129-200`, `crates/application/src/quran_doctor.rs:83-136`.

Use `open_read_only` for probes/listings/verification. If a UoW is used for reads,
roll it back. `qai db migrate` is the only explicit schema mutation path; missing
or pending migrations must produce the exact `qai db migrate` remedy.

### 4. One audited `UnitOfWork`

**Sources:** `crates/storage/src/lib.rs:116-146`,
`crates/storage-sqlite/src/lib.rs:230-307`,
`crates/application/src/audit_bridge.rs:200-232`,
`crates/storage/src/workflows.rs:27-110`.

The canonical order is: authoritative domain write → provenance → required outbox
→ audit event (or an explicitly documented equivalent) → one `commit`. Any error
before commit rolls back the entire UoW. Never “compensate” with a second delete
transaction.

### 5. Diagnostics and exit codes

**Sources:** `crates/storage/src/error.rs:25-107,366-373`,
`crates/domain/src/diagnostic.rs:75-129`, `crates/cli/src/exit_code.rs:1-40`.

Use stable `QAI-*` codes, explicit remedies, and next commands. Map validation,
policy, not-found, conflict, cancellation, and internal failures centrally; do not
print ad-hoc `eprintln!` strings as the only machine-readable contract.

### 6. Durable job contract and idempotency

**Sources:** `crates/jobs/src/lib.rs:192-214,225-379`,
`crates/jobs/src/queue.rs:16-68`, `crates/application/src/quran.rs:35-38,70-131`.

Handlers declare `kind`, schema, idempotency, and run with `JobContext`; queue
claims are lease-based; all retry/cancel/checkpoint operations go through the
queue/repository ports. Keep deterministic idempotency keys and stable kind
strings.

### 7. Real infrastructure tests

**Sources:** `crates/testkit/src/lib.rs:66-100,151-183`,
`crates/storage-sqlite/tests/commit_bounds_outbox.rs:13-74`,
`crates/cli/tests/catalog.rs:13-35`.

Use `tempfile`, real migrations, real SQLite, and the real `qai` binary for
acceptance. Use hand-written fakes only for pure trait behavior. Keep sentinel
secret checks and logical/byte-level no-mutation assertions.

### 8. Architecture and CI

**Sources:** `xtask/src/arch.rs:81-164`, `xtask/allowlist.toml:1-101`,
`.github/workflows/ci.yml:47-64`.

Keep `arch-check` as a pure metadata check with a checked-in allowlist and a
synthetic mutation test. Extend the same gate to external sources; do not replace
it with grep or a second policy system. `cargo-deny` remains a separate supply-chain
gate.

## No Analog Found / Planner Decisions Required

| File or Concern | Role | Data Flow | Why no exact analog | Reuse instead |
|---|---|---|---|---|
| `migrations/sqlite/0020_*.up.sql` (if needed) | migration | file-I/O | The research explicitly leaves schema fields and migration number unchosen; current 0004/0005 are older schemas | `0004_jobs.up.sql`, `0005_audit.up.sql`, `crates/storage-sqlite/src/migrate.rs` |
| New worker-host lifecycle module, if created | provider/service | event-driven | No existing worker supervisor, signal abstraction, or graceful shutdown implementation | `crates/cli/src/lib.rs:301-345`, `crates/server/src/api.rs:1323-1330`, `jobs::Worker::run_once` |
| External dependency allow/deny representation | config/build gate | transform | `xtask/allowlist.toml` reserves `external.allow` but no checked-in rule or parser exists | `xtask/src/arch.rs:105-142` pure `violations` and synthetic metadata builders |
| `xtask/src/arch.rs` inline test module | test | transform | Existing policy tests are inline; the project has no separate xtask integration-test target | Extend the existing synthetic cases at `xtask/src/arch.rs:166-331` |

## Evidence Matrix Support

The five work packages map directly to the five Phase 1 success criteria without
rebuilding later Quran/search/server capabilities:

| Criterion | Reuse target | New acceptance evidence |
|---|---|---|
| Build/help tree | `crates/cli/src/lib.rs:14-122`, existing `catalog` test | `crates/cli/tests/foundation.rs` seven-group help contract |
| CLI > env > file > defaults | `crates/config/src/lib.rs:223-260`, `crates/testkit/tests/config_precedence.rs` | real CLI file/env/origin/redaction case |
| Provenance + append-only audit | `crates/storage/src/workflows.rs`, `crates/application/src/audit_bridge.rs` | failure-injection same-UoW tests and `qai audit verify` |
| Jobs enqueue/lease/checkpoint/cancel | `crates/jobs/src/worker.rs`, `crates/jobs/src/queue.rs`, `crates/storage-sqlite/src/lib.rs` | real SQLite recovery/retry/cancel/lifecycle audit suite |
| Forbidden dependencies fail CI | `xtask/src/arch.rs`, `xtask/allowlist.toml` | synthetic registry-edge mutation plus existing CI command |

## Metadata

**Analog search scope:** `crates/config`, `crates/cli`, `crates/application`,
`crates/jobs`, `crates/storage`, `crates/storage-sqlite`, `crates/audit`,
`crates/provenance`, `crates/server`, `crates/testkit`, `crates/domain`,
`xtask`, `migrations/sqlite`, and their unit/integration tests.  
**Graft usage:** repository map and targeted symbol searches were used before source inspection; current worktree source was read where the graph was projection-limited or concurrent Phase-2 files had changed.  
**Tracked-source verification:** all existing analog paths above returned a non-empty `git ls-files` result.  
**Worktree safety:** no source file was modified during mapping. `crates/application/src/quran_cli.rs` and `crates/cli/src/quran.rs` were observed carrying unrelated Phase-2 changes during inspection; re-check the live worktree before editing and preserve any such hunks.  
**Pattern extraction date:** 2026-09-24.
