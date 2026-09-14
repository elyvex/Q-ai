# Runbook — Migration Checksum Mismatch

**When to use:** a startup or `qai db verify` fails with `QAI-DB-0003`
(`MigrationChecksumMismatch`).

## Trigger
- The checksum recorded in `schema_migrations` does not match the on-disk `.up.sql` file, or
  `cargo xtask migrate-check` fails with `QAI-DB-0003`.

## Why it happens
Migrations are **append-only**. Editing an already-applied `.up.sql` changes its checksum and is
forbidden — it silently diverges databases that already ran the old version.

## Steps
1. Identify the file: the error names the migration version.
2. `git log -- migrations/sqlite/NNNN_*.up.sql` (or `git blame`) to see whether the file was edited
   after it was applied.
3. **Preferred fix:** revert the file to its committed content. The checksum then matches again.
4. If the change was intentional, do **not** edit the applied file. Add a **new** migration
   (`NNNN+1_*.up.sql`) that performs the change, then regenerate `migrations/sqlite/checksums.json`.
5. Re-run `cargo xtask migrate-check` and `qai db verify`.

## Hotfix note (risk R7)
A documented `qai db repair-checksum --yes` may be used only with an explicit, audited decision
(ADR-0002). Never use it to paper over an uncommitted edit.

## Verify
- `cargo xtask migrate-check` prints `checksums stable`.
- `qai db verify` exits 0.
