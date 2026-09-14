# Changelog

All notable changes to Q-ai are documented here.

## [Unreleased]

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

### Changed

- `storage`: repository traits are now `Send + Sync` (async `&self` methods).
- `xtask`: `arch-check` allowlist reconciled with actual dependency edges; the
  `storage-sqlite` table rename is honored; migrate-check accepts the `sha256:` prefix.

### Fixed

- Workspace `clippy -D warnings` and `cargo fmt --check` are clean.
