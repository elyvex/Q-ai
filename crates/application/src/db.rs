//! Database orchestration for the CLI: migrate, status, verify, backup, probe.
//!
//! Keeps `storage-sqlite` behind the application boundary (arch-check: the CLI
//! may depend on `application` but not on `storage-sqlite` directly).

use std::path::Path;

use config::Config;
use storage::Database as _;
use storage::error::StorageError;
use storage_sqlite::{SqliteDatabase, migrate};

/// Outcome of `qai db status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationStatus {
    pub applied_version: u32,
    pub latest_on_disk: u32,
    pub pending: Vec<u32>,
    pub current: bool,
}

/// A read-only snapshot of database health used by `qai doctor`.
#[derive(Debug, Clone, Default)]
pub struct DbProbe {
    pub reachable: bool,
    pub error: Option<String>,
    pub schema_version: u32,
    pub integrity_ok: bool,
    pub foreign_keys_on: bool,
    pub running_jobs: i64,
    pub interrupted_jobs: i64,
    pub dead_lettered_jobs: i64,
    pub outbox_pending: i64,
    pub outbox_oldest_pending_seconds: i64,
    pub tombstones_unpropagated: i64,
    pub audit_events: i64,
    pub license_unknown_count: i64,
    pub multiple_active_versions: i64,
}

/// The highest migration version present on disk (0 if none).
pub fn latest_migration_version(migrations_dir: &Path) -> u32 {
    migrate::discover_migrations(migrations_dir)
        .map(|list| list.iter().map(|m| m.version).max().unwrap_or(0))
        .unwrap_or(0)
}

/// Apply pending migrations. Returns the resulting schema version.
pub async fn migrate_database(cfg: &Config, migrations_dir: &Path) -> Result<u32, StorageError> {
    migrate::apply_migrations(&cfg.storage.sqlite.path, migrations_dir).await
}

/// Verify applied-migration checksums without writing.
pub async fn verify_migrations(
    cfg: &Config,
    migrations_dir: &Path,
) -> Result<migrate::ChecksumReport, StorageError> {
    migrate::verify_checksums(&cfg.storage.sqlite.path, migrations_dir).await
}

/// Report migration status (applied vs on-disk latest).
pub async fn migration_status(
    cfg: &Config,
    migrations_dir: &Path,
) -> Result<MigrationStatus, StorageError> {
    let discovered = migrate::discover_migrations(migrations_dir)?;
    let latest_on_disk = discovered.iter().map(|m| m.version).max().unwrap_or(0);
    let db = SqliteDatabase::open_read_only(&cfg.storage.sqlite.path).await?;
    let applied_version = db.schema_version();
    let applied_count = db.count("SELECT COUNT(*) FROM schema_migrations").await.unwrap_or(0);
    // Pending = on-disk versions not yet applied (by count, versions are contiguous).
    let pending: Vec<u32> =
        discovered.iter().filter(|m| m.version > applied_version).map(|m| m.version).collect();
    Ok(MigrationStatus {
        applied_version,
        latest_on_disk,
        pending,
        current: applied_version >= latest_on_disk || applied_count == 0 && latest_on_disk == 0,
    })
}

/// Take a consistent backup (`VACUUM INTO`).
pub async fn backup_database(cfg: &Config, dest: &str) -> Result<(), StorageError> {
    migrate::backup(&cfg.storage.sqlite.path, dest).await
}

/// Probe database health read-only. Never mutates; never errors.
pub async fn probe_database(cfg: &Config) -> DbProbe {
    let mut probe = DbProbe::default();
    let db = match SqliteDatabase::open_read_only(&cfg.storage.sqlite.path).await {
        Ok(db) => db,
        Err(e) => {
            probe.error = Some(format!("{e}"));
            return probe;
        }
    };
    probe.reachable = true;
    probe.schema_version = db.schema_version();

    probe.integrity_ok = db
        .probe_scalar("PRAGMA integrity_check")
        .await
        .ok()
        .flatten()
        .map(|s| s.eq_ignore_ascii_case("ok"))
        .unwrap_or(false);

    probe.foreign_keys_on = db.count("PRAGMA foreign_keys").await.unwrap_or(0) == 1;

    let table_exists = |name: &str| {
        format!("SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = '{name}'")
    };
    if db.count(&table_exists("jobs")).await.unwrap_or(0) > 0 {
        probe.running_jobs =
            db.count("SELECT COUNT(*) FROM jobs WHERE state = 'Running'").await.unwrap_or(0);
        probe.interrupted_jobs =
            db.count("SELECT COUNT(*) FROM jobs WHERE state = 'Interrupted'").await.unwrap_or(0);
        probe.dead_lettered_jobs =
            db.count("SELECT COUNT(*) FROM jobs WHERE state = 'DeadLettered'").await.unwrap_or(0);
    }
    if db.count(&table_exists("outbox_events")).await.unwrap_or(0) > 0 {
        probe.outbox_pending = db
            .count("SELECT COUNT(*) FROM outbox_events WHERE state = 'Pending'")
            .await
            .unwrap_or(0);
        probe.outbox_oldest_pending_seconds = db
            .count(
                "SELECT CAST((julianday('now') - julianday(MIN(created_at))) * 86400 AS INTEGER)
                 FROM outbox_events WHERE state = 'Pending'",
            )
            .await
            .unwrap_or(0);
    }
    if db.count(&table_exists("tombstones")).await.unwrap_or(0) > 0 {
        probe.tombstones_unpropagated = db
            .count("SELECT COUNT(*) FROM tombstones WHERE propagation_state = 'Pending'")
            .await
            .unwrap_or(0);
    }
    if db.count(&table_exists("source_versions")).await.unwrap_or(0) > 0 {
        probe.license_unknown_count = db
            .count("SELECT COUNT(*) FROM source_versions WHERE license_status = 'Unknown'")
            .await
            .unwrap_or(0);
        probe.multiple_active_versions = db
            .count(
                "SELECT COUNT(*) FROM (
                    SELECT source_id FROM source_versions WHERE state = 'Active'
                    GROUP BY source_id HAVING COUNT(*) > 1
                 )",
            )
            .await
            .unwrap_or(0);
    }
    if db.count(&table_exists("audit_events")).await.unwrap_or(0) > 0 {
        probe.audit_events = db.count("SELECT COUNT(*) FROM audit_events").await.unwrap_or(0);
    }
    probe
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn migrations_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite")
    }

    #[tokio::test]
    async fn migrate_then_probe() {
        let dir = tempdir().unwrap();
        let mut cfg = Config::default();
        cfg.app.data_dir = dir.path().display().to_string();
        cfg.storage.sqlite.path = dir.path().join("qai.db").display().to_string();

        let version = migrate_database(&cfg, &migrations_dir()).await.unwrap();
        assert_eq!(version, 6);

        let probe = probe_database(&cfg).await;
        assert!(probe.reachable);
        assert!(probe.integrity_ok);
        assert!(probe.foreign_keys_on);
        assert_eq!(probe.schema_version, 6);
    }

    #[tokio::test]
    async fn probe_missing_db_is_unreachable_without_panicking() {
        let dir = tempdir().unwrap();
        let mut cfg = Config::default();
        cfg.storage.sqlite.path = dir.path().join("nope.db").display().to_string();
        let probe = probe_database(&cfg).await;
        assert!(!probe.reachable);
        assert!(probe.error.is_some());
    }

    #[tokio::test]
    async fn status_and_backup_round_trip() {
        let dir = tempdir().unwrap();
        let mut cfg = Config::default();
        cfg.storage.sqlite.path = dir.path().join("qai.db").display().to_string();
        migrate_database(&cfg, &migrations_dir()).await.unwrap();

        let status = migration_status(&cfg, &migrations_dir()).await.unwrap();
        assert!(status.current);
        assert_eq!(status.latest_on_disk, 6);

        let dest = dir.path().join("bk.db");
        backup_database(&cfg, dest.to_str().unwrap()).await.unwrap();
        assert!(dest.exists());
    }
}
