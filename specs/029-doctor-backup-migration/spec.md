# Feature Specification: Doctor, Backup/Restore, and Checksumed Migration Discipline

**Feature Branch**: `029-doctor-backup-migration`

**Created**: 2026-09-18

**Status**: Draft

**Input**: User description: "Reverse specification 029 — pin the implemented doctor / backup-restore / migration behavior as code-is-truth: 26 Phase-0 doctor checks plus 19 Quran corpus checks with read-only guarantee, VACUUM INTO backup/restore procedure, checksummed append-only migrations 0001–0016, and migrate-check / arch-check gates."

> Gap note (code-is-truth): README.md "Current state" claims checksummed migrations and backups, a diagnostic CLI, and `0001`–`0016` coverage, and the quick-start presents `qai db migrate` + `qai doctor --quran --deep --json` as the demo path, but no prior spec pins the check IDs, the read-only guarantee, the VACUUM INTO procedure, or the gate semantics. This reverse spec pins the implemented behavior in `crates/cli/src/doctor.rs`, `crates/application/src/quran_doctor.rs`, `crates/application/src/db.rs`, `crates/storage-sqlite/src/migrate.rs`, `migrations/sqlite/` + `checksums.json`, and `xtask/src/migrate.rs` + `xtask/src/arch.rs`. Where README prose and code disagree, code wins.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Run read-only system doctor and act on failures (Priority: P1)

An operator runs `qai doctor` (human text) or `qai doctor --json` and gets the full 26-check registry, each result carrying `Pass | Warn | Fail | Skipped` plus a remedy and a next command whenever the check is not passing. A failing check drives a non-zero exit; passing checks carry no remedy. Nothing the command does mutates the database.

**Why this priority**: This is the primary operability surface for every deployment: configuration, database health, jobs, audit, sources, filesystem, security, outbox, tombstones, and local-only posture in one read-only pass. Without pinned check IDs and the read-only guarantee, operators cannot trust or script the output.

**Independent Test**: Can be fully tested by seeding a healthy migrated database with default config, running `qai doctor` and `qai doctor --json`, and asserting all 26 check IDs are present, the JSON validates against `docs/schemas/doctor.v1.schema.json` shape `{ checks: [{ id, status, summary, remedy, next_command }] }` with `status ∈ { pass, warn, fail, skipped }`, and the database file is byte-for-byte unchanged afterwards. Delivers a trustworthy health report.

**Acceptance Scenarios**:

1. **Given** a migrated database and default loopback config, **When** the operator runs `qai doctor`, **Then** all 26 Phase-0 checks (`configuration.valid`, `configuration.file_permissions`, `data_dir.writable`, `database.reachable`, `database.migration_current`, `database.integrity_check`, `database.foreign_keys_enabled`, `secrets.backend_available`, `secrets.refs_resolvable`, `jobs.worker_health`, `jobs.interrupted_runs`, `jobs.dead_letter_count`, `audit.chain_valid`, `sources.manifest_schema_valid`, `sources.orphaned_versions`, `sources.missing_files`, `sources.license_unknown_count`, `sources.multiple_active_versions`, `filesystem.object_store_writable`, `security.bind_address_safe`, `security.tls_policy_consistent`, `observability.subscriber_installed`, `outbox.backlog_age`, `outbox.undispatched_count`, `tombstones.unpropagated_count`, `local_only_fallbacks`) are emitted with `Pass` where healthy and the process exits zero.
2. **Given** an unreachable database, **When** the operator runs `qai doctor`, **Then** `database.reachable` is `Fail` with remedy "run migrations to create the database" and next `qai db migrate`, all database-dependent checks report `Skipped`, and the exit code is the validation failure code.
3. **Given** a non-loopback bind with TLS disabled, **When** the operator runs `qai doctor`, **Then** `security.tls_policy_consistent` is `Fail` with remedy "set server.tls=required or bind to loopback" and next `qai config validate`.
4. **Given** any doctor run, **When** the database file is hashed before and after, **Then** the bytes are identical (read-only guarantee; `probe_database` opens read-only, `doctor_is_read_only` test semantics).

---

### User Story 2 - Verify Quran corpus integrity with sampled or deep scan (Priority: P1)

A corpus steward runs `qai doctor --quran` (sampled token round-trip) or `qai doctor --quran --deep` (full-corpus round-trip) and gets the 19 `quran.*` checks merged into the same single JSON document as the Phase-0 registry, plus merged human text. Hash checks always recompute fully; only the token round-trip sampling changes with `--deep`.

**Why this priority**: Canonical text integrity is non-negotiable (constitution I): recomputed `text_hash` / `structure_hash` / `token_order_hash`, counts, identifiers, Unicode form, division coverage, basmala policy, provenance, layering, alignment, attribution, staging orphans, generation, reference-corpus posture, and license must be verifiable without an LLM and without mutating data.

**Independent Test**: Can be fully tested by importing and activating the synthetic `test-edition-min` fixture, running `qai doctor --quran --json` and `qai doctor --quran --deep --json`, and asserting all 19 `quran.*` IDs are present in one top-level `checks` array and that `--deep` upgrades `quran.token_roundtrip` from the 3-ayah sample (first/middle/last) to a full-corpus scan. Delivers corpus trust independent of the system checks.

**Acceptance Scenarios**:

1. **Given** an active edition with intact rows, **When** the steward runs `qai doctor --quran`, **Then** the 19 checks (`quran.edition_active`, `quran.edition_checksum`, `quran.structure_hash`, `quran.token_order_hash`, `quran.surah_count`, `quran.ayah_count`, `quran.ayah_identifiers`, `quran.token_roundtrip`, `quran.unicode_form`, `quran.division_coverage`, `quran.basmala_policy`, `quran.provenance_complete`, `quran.metadata_layering`, `quran.translations_aligned`, `quran.translation_attribution`, `quran.staging_orphans`, `quran.corpus_generation`, `quran.reference_corpus`, `quran.license_status`) are emitted and merged into the single doctor JSON document (never two concatenated documents).
2. **Given** no active edition, **When** the steward runs `qai doctor --quran`, **Then** `quran.edition_active` is `Fail` ("no active Arabic edition", next `qai quran import --help`) and no further corpus checks run.
3. **Given** a stored `text_hash` that differs from recomputation over stored rows, **When** the steward runs the Quran doctor, **Then** `quran.edition_checksum` is `Fail` with remedy "re-import the edition; do not edit canonical rows" and next `qai quran validate`.
4. **Given** any Quran doctor run, **When** the unit of work completes, **Then** it is rolled back and no write method is invoked (read-only by construction; `--deep` only widens the round-trip sample).

---

### User Story 3 - Back up and restore the database safely (Priority: P2)

An operator takes a consistent snapshot with `qai db backup <path>` and, after corruption or a bad migration, restores it with the verified swap-in procedure that preserves the current file aside instead of deleting it.

**Why this priority**: SQLite WAL databases cannot be backed up with a raw file copy (tears an active WAL — ADR-0001 §7). The backup/restore path is the only supported disaster-recovery story for the local-first deployment.

**Independent Test**: Can be fully tested by migrating a scratch database, inserting rows, running backup to a new path, verifying the backup passes checksum + `integrity_check`, then restoring it over a target and confirming the target matches and the previous target was preserved as `<path>.pre-restore`. Delivers disaster recovery independent of doctor and migration authoring.

**Acceptance Scenarios**:

1. **Given** a live database, **When** the operator runs backup to a non-existent destination, **Then** a consistent standalone snapshot is produced via `VACUUM INTO` on a single read-only connection, and the snapshot verifies (checksums valid, `PRAGMA integrity_check = ok`).
2. **Given** a backup destination that already exists, **When** the operator runs backup, **Then** the command fails with a conflict error and the existing file is left untouched (never silently overwritten).
3. **Given** a backup file with checksum drift or a failing `integrity_check`, **When** the operator attempts restore, **Then** restore aborts before touching the live database (checksum mismatch error, or storage-unavailable on integrity failure).
4. **Given** a valid verified backup and an existing live database, **When** the operator restores, **Then** the live file is renamed to `<path>.pre-restore`, stale `-wal`/`-shm` sidecars are removed, and the backup bytes are copied into place (copy, not a live-WAL copy).

---

### User Story 4 - Evolve schema only through checksummed append-only migrations (Priority: P2)

A developer adds schema change `0017_*` (or an operator applies pending migrations with `qai db migrate`), and the system guarantees ordering, checksum stability, idempotent apply, and pre-write drift detection, with `qai db status` / `qai db verify` exposing current state.

**Why this priority**: Forward-only, checksummed, contiguous migrations (`0001`–`0016` today) are the architecture-discipline backbone (constitution VII): canonical tables are insert-only/forward-only, and editing history is a hard failure, not a warning.

**Independent Test**: Can be fully tested by applying `migrations/sqlite/` to a fresh database twice (same resulting version, second run a no-op), editing one applied `.up.sql` byte and re-running apply or verify (checksum-mismatch failure naming the version), and deleting one version from the sequence and running `migrate-check` (contiguity failure). Delivers schema evolution safety independent of backup and doctor.

**Acceptance Scenarios**:

1. **Given** a fresh database and on-disk versions `0001`–`0016`, **When** migrations are applied, **Then** all pending `NNNN_*.up.sql` files apply in version order, each records `sha256:<hex>` + timestamp + `applied_by=qai` + duration in `schema_migrations`, and the returned version is the on-disk maximum (16 today).
2. **Given** an already-applied migration whose file content changed by even one byte, **When** apply or verify runs, **Then** it aborts with a checksum-mismatch failure for that version before any write (applied-file edits are forbidden; new changes require a new appended version).
3. **Given** a version gap on disk (e.g. `0001` then `0003`), **When** `cargo xtask migrate-check` runs, **Then** it fails the contiguity rule (versions must be unique, contiguous from 1, monotonic) and names the expected version.
4. **Given** an applied database, **When** the operator runs `qai db status` and `qai db verify`, **Then** status reports applied version vs on-disk latest plus the pending list and a current flag, and verify reports `{ valid, mismatches[], missing_on_disk[] }` without writing.

---

### User Story 5 - Preview repairs and hold quality gates before merge (Priority: P3)

An operator previews what would be fixed with `qai doctor --repair-preview` (plan only, zero mutation), and CI blocks merges that break dependency layering or migration discipline via `cargo xtask arch-check` and `cargo xtask migrate-check`.

**Why this priority**: Constitution V/VI/VII require deny-by-default, doctor-never-mutates, and green gates before merge. Repair preview gives operators a safe next step; the two xtask gates make layering and migration rules enforceable rather than conventional.

**Independent Test**: Can be fully tested by running doctor on a database with warnings/failures with `--repair-preview` (lists actionable `Warn|Fail` IDs with would/then lines, database bytes unchanged, exit zero) and by feeding the arch gate a synthetic metadata graph with a forbidden `domain -> cli` edge plus a gapped-migration directory to the migrate gate (both fail with the documented error codes). Delivers safe operations and merge protection independent of the other stories.

**Acceptance Scenarios**:

1. **Given** doctor results containing `Warn`/`Fail` entries, **When** the operator passes `--repair-preview`, **Then** only the plan is printed (`repair-preview (no changes will be made)` with `would: <remedy>` / `then: <next_command>` per actionable check), nothing is mutated, and the exit is zero even with failures present.
2. **Given** a workspace graph containing an edge not in `xtask/allowlist.toml` (e.g. `domain -> cli`), **When** `cargo xtask arch-check` runs, **Then** it fails closed listing each forbidden edge, including for crates absent from the allowlist (unrecognized crates may depend on no workspace crate).
3. **Given** the current tree, **When** the full gate set runs (`cargo fmt --check`, `cargo check`, `cargo clippy -D warnings`, `cargo test --workspace`, `arch-check`, `migrate-check`), **Then** all gates pass before merge per constitution V, and the CLI smoke path (`qai db migrate/status/verify/backup`, `qai doctor`) stays green.

---

### Edge Cases

- What happens when the database file does not exist or is not a valid SQLite file? `database.reachable` is `Fail`; all database-dependent checks become `Skipped` with next `qai db migrate`; probe never errors (returns `DbProbe::default` with the error string recorded).
- How does the system handle a schema older than the on-disk latest? `database.migration_current` is `Fail` (`schema v<N> < latest v<M>`, next `qai db migrate`); `latest_migration_version` is computed from `NNNN_*.up.sql` discovery on `migrations/sqlite`.
- What happens when `PRAGMA integrity_check` does not return exactly `ok`? `database.integrity_check` is `Fail` with remedy "restore from a known-good backup" and next `qai db backup <path>` (note the code-truth asymmetry: the remedy points at backup while recovery requires restore; pinned as-is).
- How does backup handle WAL sidecars and existing destinations? Backup reads through a single read-only connection (`VACUUM INTO ?`) so WAL state is folded into the snapshot; a pre-existing destination aborts with `Conflict` instead of overwriting.
- How does restore handle a live database with `-wal`/`-shm` sidecars? The live file is renamed to `<path>.pre-restore` (never deleted), sidecars are removed, then the verified snapshot is copied into place.
- What happens when an applied migration file is missing from disk at verify time? Its version is recorded as a mismatch (verify treats missing-on-disk applied versions as mismatches; `missing_on_disk` separately lists on-disk versions never applied).
- How does the system handle down migrations? Only `revert_last_migration` consumes `.down.sql` (present for `0001`–`0006`, absent for `0007`–`0016`); it deletes the bookkeeping row first (the down SQL may drop the bookkeeping table), then executes the down SQL; reverting with nothing applied returns `None`; reverting a version without a down file is a migration-required error.
- What happens when `--quran` is combined with `--json`? Exactly one JSON document is emitted (Phase-0 `checks` array extended with the `quran.*` array) that preserves the `doctor.v1` schema shape; redaction scrubs both JSON and human text at the emission boundary without altering keys or nesting.
- How do the writability checks avoid violating read-only semantics? `data_dir.writable` and `filesystem.object_store_writable` write and immediately delete a `.qai-probe` file under the configured directories (filesystem probe only — the database itself is never written); `configuration.file_permissions` only reports (Phase 0 recommends 0600, never enforces).
- What happens when secrets use a non-`env` backend or an unknown backend string? `keychain`/`encrypted_file` yield `Warn` (OS support required, next `qai secret list`); unknown strings yield `Fail`; `secrets.refs_resolvable` is existence-only (never prints values) and `Skipped` for non-`env` backends.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST expose 26 Phase-0 doctor checks with the exact stable IDs `configuration.valid`, `configuration.file_permissions`, `data_dir.writable`, `database.reachable`, `database.migration_current`, `database.integrity_check`, `database.foreign_keys_enabled`, `secrets.backend_available`, `secrets.refs_resolvable`, `jobs.worker_health`, `jobs.interrupted_runs`, `jobs.dead_letter_count`, `audit.chain_valid`, `sources.manifest_schema_valid`, `sources.orphaned_versions`, `sources.missing_files`, `sources.license_unknown_count`, `sources.multiple_active_versions`, `filesystem.object_store_writable`, `security.bind_address_safe`, `security.tls_policy_consistent`, `observability.subscriber_installed`, `outbox.backlog_age`, `outbox.undispatched_count`, `tombstones.unpropagated_count`, `local_only_fallbacks`.
- **FR-002**: System MUST expose 19 Quran corpus checks with the exact stable IDs `quran.edition_active`, `quran.edition_checksum`, `quran.structure_hash`, `quran.token_order_hash`, `quran.surah_count`, `quran.ayah_count`, `quran.ayah_identifiers`, `quran.token_roundtrip`, `quran.unicode_form`, `quran.division_coverage`, `quran.basmala_policy`, `quran.provenance_complete`, `quran.metadata_layering`, `quran.translations_aligned`, `quran.translation_attribution`, `quran.staging_orphans`, `quran.corpus_generation`, `quran.reference_corpus`, `quran.license_status`.
- **FR-003**: Doctor MUST be read-only against the database: health probing opens the database read-only and never errors (returns a default probe with the error recorded on failure); Quran checks run inside a single unit of work that is always rolled back with no write method invoked; running checks MUST leave the database file byte-for-byte identical.
- **FR-004**: Every doctor check result MUST carry `id`, `status` (`pass | warn | fail | skipped`), `summary`, `remedy`, and `next_command`; every non-`Pass` result MUST include both a remedy and a next command; `Pass` results MUST carry no remedy or next command.
- **FR-005**: Doctor JSON output MUST be a single document shaped `{ checks: [{ id, status, summary, remedy, next_command }] }` conforming to `docs/schemas/doctor.v1.schema.json`; `--quran` MUST merge Quran checks into the same `checks` array (never emit two concatenated documents); `--quran --deep` MUST upgrade only `quran.token_roundtrip` from the first/middle/last sample to a full-corpus scan while hashes always fully recompute.
- **FR-006**: Doctor MUST exit zero when no check is `Fail` and with the validation-failure code when any check is `Fail`; `--repair-preview` MUST print only the actionable (`Warn`/`Fail`) plan with `would:`/`then:` lines, mutate nothing, and exit zero.
- **FR-007**: Doctor emission MUST scrub credential-shaped values from both JSON and human text at the print boundary while preserving keys, nesting, and schema shape (redact-then-print).
- **FR-008**: System MUST implement backup as `VACUUM INTO ?` executed on a single read-only connection (never a raw file copy of a live WAL database) and MUST fail with a conflict error when the destination already exists, leaving it untouched.
- **FR-009**: System MUST implement restore as verify-then-swap: checksum-verify the backup against `migrations/sqlite`, require `PRAGMA integrity_check = ok` on the backup opened read-only, rename any existing live database to `<path>.pre-restore` (never delete), remove stale `-wal`/`-shm` sidecars, and copy the snapshot into place.
- **FR-010**: System MUST apply migrations by discovering `NNNN_name.up.sql` (+ optional `NNNN_name.down.sql`) ordered by version, checksum-verifying every already-applied migration before any write, applying pending versions in order, and recording `version, name, sha256:<hex> checksum, applied_at (RFC3339), applied_by=qai, duration_ms` in `schema_migrations` (creating the table if the first migration did not); re-apply MUST be idempotent and return the maximum version.
- **FR-011**: System MUST treat editing any applied migration file as a hard checksum-mismatch failure naming the version, both at apply time (before any write) and at verify time; `verify_checksums` MUST be read-only and report `{ valid, mismatches[], missing_on_disk[] }` where applied-but-absent-on-disk versions count as mismatches.
- **FR-012**: On-disk migrations MUST be unique, contiguous from 1, and monotonic; `cargo xtask migrate-check` MUST fail gapped/non-numeric/misnamed sequences and checksum drift against the published `migrations/sqlite/checksums.json` manifest (newly appended, not-yet-recorded files are allowed); the manifest MUST record `sha256:<hex>` per migration file for all of `0001`–`0016` (22 file keys: up+down for `0001`–`0006`, up-only for `0007`–`0016`).
- **FR-013**: `qai db status` MUST report applied version (read-only open), on-disk latest, the pending version list, and a current flag; `qai db verify` MUST surface the checksum report without writing.
- **FR-014**: `cargo xtask arch-check` MUST enforce the `xtask/allowlist.toml` dependency-direction contract fail-closed: a workspace crate may depend only on its listed workspace crates, and a crate absent from the allowlist may depend on no workspace crate; the CLI MUST reach storage only through `application::db` (never depend on `storage-sqlite` directly).
- **FR-015**: `database.migration_current` MUST compare the probed schema version against `latest_migration_version(migrations/sqlite)` and report `Fail` with next `qai db migrate` when behind, `Pass` when current, and `Skipped` when the database is unreachable.
- **FR-016**: Security checks MUST implement loopback-default semantics: `security.bind_address_safe` passes on `127.0.0.1 | ::1 | localhost | 127.*` and warns otherwise; `security.tls_policy_consistent` fails on non-loopback bind with `tls=disabled` and passes otherwise; `local_only_fallbacks` passes with network egress disabled and warns when enabled.
- **FR-017**: Operational-count checks MUST implement the coded thresholds: `jobs.worker_health` warns when any job is `Running`; `jobs.interrupted_runs` / `jobs.dead_letter_count` warn when nonzero; `outbox.backlog_age` warns only when pending events exist and the oldest is older than 3600s; `outbox.undispatched_count` and `tombstones.unpropagated_count` warn when nonzero; `sources.license_unknown_count` warns when nonzero; `sources.multiple_active_versions` fails when nonzero; `audit.chain_valid` and `sources.orphaned_versions` pass structurally (event count reported; orphans impossible via FK `ON DELETE RESTRICT`); `sources.manifest_schema_valid` and `sources.missing_files` are `Skipped` in Phase 0.

### Key Entities

- **CheckResult / QuranDoctorCheck**: A single diagnostic outcome with stable `id`, `status` (`Pass | Warn | Fail | Skipped`), human `summary`, optional `remedy`, and optional `next_command`; JSON renders `status` lowercase and nulls for absent remedy/command; the merged doctor document is `{ checks: [...] }` only.
- **DbProbe**: A read-only health snapshot (`reachable`, `error`, `schema_version`, `integrity_ok`, `foreign_keys_on`, running/interrupted/dead-lettered job counts, outbox pending count + oldest-pending age, unpropagated tombstones, audit event count, unknown-license count, multiple-active-versions count) gathered without mutation and without surfacing errors to callers.
- **MigrationFile / schema_migrations row**: An on-disk `NNNN_name.up.sql` (+ optional `.down.sql`) version unit and its applied record (`version` primary key, `name` stem, `sha256:<hex>` checksum, `applied_at`, `applied_by`, `duration_ms`); the file bytes are the checksum source of truth.
- **ChecksumReport / MigrationStatus**: Verify outcome (`valid`, `mismatches[]`, `missing_on_disk[]`) and status outcome (applied version, on-disk latest, pending list, current flag); both are read-only views over `schema_migrations` plus on-disk discovery.
- **Backup snapshot**: A standalone SQLite file produced by `VACUUM INTO` (WAL-folded, safe to copy) as opposed to a raw copy of a live WAL database; restore swaps a verified snapshot into place while preserving the predecessor as `<path>.pre-restore`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An operator can run the doctor on a healthy deployment and receive all 45 pinned check IDs (26 system + 19 with `--quran`) with zero failures in under one minute, and the database file hash is unchanged after the run.
- **SC-002**: An operator diagnosing a broken deployment (missing database, stale schema, corrupt integrity, insecure bind) receives the correct `Fail`/`Skipped` pattern with an actionable remedy and next command for every non-passing check, and the process exit distinguishes clean (zero) from action-required (non-zero) runs.
- **SC-003**: An operator can back up a populated database to a new path and restore it onto a target such that the restored database passes checksum verification and integrity check, the pre-restore file is preserved, and a backup to an existing path is refused without data loss.
- **SC-004**: A developer attempting to edit any applied migration file sees apply/verify fail naming the drifted version before any schema write, while a fresh apply of versions 1–16 followed by a re-apply returns the same version with no additional writes.
- **SC-005**: CI rejects a forbidden dependency edge and a gapped or drifted migration set within the standard gate run, while `--repair-preview` on a degraded system prints the full actionable plan with zero database mutation.

## Assumptions

- Reverse-spec scope: behavior is pinned from the 2026-09-18 tree (`crates/cli/src/doctor.rs`, `crates/application/src/quran_doctor.rs`, `crates/application/src/db.rs`, `crates/storage-sqlite/src/migrate.rs`, `migrations/sqlite/0001`–`0016` + `checksums.json`, `xtask/src/migrate.rs`, `xtask/src/arch.rs`); future migrations append `0017+` with up (+ optional down) files and a manifest entry under the same rules.
- Constitution V, VI, VII apply unchanged: red-green-refactor with the full gate set green before merge (V); local-only defaults, deny-by-default tools, loopback bind, fail-closed guards, redaction at emission, `unsafe_code = forbid`, doctor-never-mutates (VI); YAGNI, `application`-routed layering via `arch-check`, contiguous append-only checksummed migrations, forward-only canonical tables, FTS5 as the full-text backend (VII).
- The bundled `test-edition-min` corpus is synthetic test data: fixture success exercises the doctor/migration/backup paths but is never editorial approval of a real corpus, and `quran.reference_corpus` warns until a real reference corpus plus sign-off procedure (ADR-0114) exists.
- `quran.license_status` warns on `Unknown`/empty license state per ADR-0101; translation checks require named translators and edition-aligned translation rows; canonical rows are never edited in place (re-import + approval-gated activation instead).
- Filesystem writability probes (`.qai-probe` create-then-delete) are the only writes doctor-adjacent code performs, and only against configured directories — never against the database.
