# Runbook — Backup & Restore

**When to use:** before an upgrade, before a risky migration, or on a schedule.

## Trigger
- Scheduled maintenance, pre-migration, or suspected corruption (see `migration-checksum-mismatch.md`).

## Steps
1. Pick a destination outside the data directory:
   `qai db backup /safe/qai-$(date -u +%Y%m%dT%H%M%SZ).db`
2. Confirm the file exists and is non-empty. `qai db backup` uses `VACUUM INTO`, which takes a
   consistent snapshot even while the WAL is active — **never** copy the live `.db` file by hand.
3. Verify the snapshot:
   `QAI_DB_PATH=/safe/qai-<ts>.db qai db verify`
4. Record the backup path and checksum in your ops log.

## Restore
1. Stop the running `qai serve` / job workers.
2. Move the current database aside (do not delete until verification passes).
3. Copy the snapshot into place as `storage.sqlite.path`.
4. Run `qai db verify` then `qai doctor` before restarting.

## Rollback risk
Restoring discards everything written since the snapshot. There is no automatic merge. If the
snapshot predates applied migrations, `qai db migrate` will re-apply pending ones.

## Verify
- `qai doctor` reports `database.integrity_check` = pass and `database.migration_current` = pass.
