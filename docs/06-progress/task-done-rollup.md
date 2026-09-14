# Task Done Rollup

> Completed tasks across all phases. Newest first.

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
