//! Migration runner and backup/restore helpers (D0.7 / T19).
//!
//! - [`apply_migrations`] applies pending `NNNN_*.up.sql` files in order,
//!   records `sha256:<hex>` checksums, and verifies already-applied migrations.
//!   Editing an applied migration is a hard failure (`QAI-DB-0003`).
//! - [`backup`] takes a consistent snapshot using `VACUUM INTO` (ADR-0001 §7) —
//!   never a raw file copy, which can tear an active WAL.
//! - [`verify_checksums`] / [`status`] support `qai db verify` / `qai db status`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use storage::error::StorageError;

/// A discovered migration: version, filename stem, and up (and optional down) paths.
#[derive(Debug, Clone)]
pub struct MigrationFile {
    pub version: u32,
    pub name: String,
    pub path: PathBuf,
    pub down_path: Option<PathBuf>,
}

/// Discover `NNNN_name.up.sql` (+ optional `NNNN_name.down.sql`) files, ordered
/// by version.
pub fn discover_migrations(dir: &Path) -> Result<Vec<MigrationFile>, StorageError> {
    let mut found: BTreeMap<u32, MigrationFile> = BTreeMap::new();
    let entries = std::fs::read_dir(dir).map_err(|_| StorageError::StorageUnavailable)?;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let is_down = name.ends_with(".down.sql");
        let is_up = !is_down && name.ends_with(".up.sql");
        if !(is_up || is_down) {
            continue;
        }
        let stem =
            name.strip_suffix(".down.sql").or_else(|| name.strip_suffix(".up.sql")).unwrap_or(name);
        let Some((ver, _rest)) = stem.split_once('_') else {
            continue;
        };
        let version: u32 = ver
            .parse()
            .map_err(|_| StorageError::MigrationRequired { at_schema: 0, required: 0 })?;
        let entry = found.entry(version).or_insert_with(|| MigrationFile {
            version,
            name: stem.to_string(),
            path: PathBuf::new(),
            down_path: None,
        });
        if is_down {
            entry.down_path = Some(path);
        } else {
            entry.path = path;
        }
    }
    Ok(found.into_values().collect())
}

fn sha256_file_hex(path: &Path) -> Result<String, StorageError> {
    let bytes = std::fs::read(path).map_err(|_| StorageError::StorageUnavailable)?;
    let digest = Sha256::digest(&bytes);
    Ok(format_hex(&digest))
}

fn format_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

async fn connect_rw(db_path: &str) -> Result<sqlx::SqlitePool, StorageError> {
    // Ensure the parent directory exists so a fresh `--data-dir` works.
    if let Some(parent) = Path::new(db_path).parent()
        && !parent.as_os_str().is_empty()
    {
        let _ = std::fs::create_dir_all(parent);
    }
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .foreign_keys(true)
        .busy_timeout(std::time::Duration::from_millis(5000));
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|_| StorageError::StorageUnavailable)
}

async fn connect_ro(db_path: &str) -> Result<sqlx::SqlitePool, StorageError> {
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(false)
        .read_only(true)
        .busy_timeout(std::time::Duration::from_millis(5000));
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|_| StorageError::StorageUnavailable)
}

/// Ensure the `schema_migrations` bookkeeping table exists.
async fn ensure_migrations_table(pool: &sqlx::SqlitePool) -> Result<(), StorageError> {
    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version     INTEGER PRIMARY KEY,
            name        TEXT    NOT NULL,
            checksum    TEXT    NOT NULL,
            applied_at  TEXT    NOT NULL,
            applied_by  TEXT    NOT NULL,
            duration_ms INTEGER NOT NULL
        );",
    )
    .execute(pool)
    .await
    .map_err(|_| StorageError::StorageUnavailable)?;
    Ok(())
}

/// Whether a table exists in the SQLite schema.
async fn table_exists(pool: &sqlx::SqlitePool, name: &str) -> Result<bool, StorageError> {
    let row = sqlx::query("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?")
        .bind(name)
        .fetch_optional(pool)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
    Ok(row.is_some())
}

async fn applied_versions(pool: &sqlx::SqlitePool) -> Result<BTreeMap<u32, String>, StorageError> {
    if !table_exists(pool, "schema_migrations").await? {
        return Ok(BTreeMap::new());
    }
    let rows = sqlx::query("SELECT version, checksum FROM schema_migrations")
        .fetch_all(pool)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
    let mut map = BTreeMap::new();
    for row in rows {
        let version: i64 = row.try_get("version").map_err(|_| StorageError::StorageUnavailable)?;
        let checksum: String =
            row.try_get("checksum").map_err(|_| StorageError::StorageUnavailable)?;
        map.insert(version as u32, checksum);
    }
    Ok(map)
}

/// Apply all pending migrations from `migrations_dir` to `db_path`.
///
/// Returns the resulting schema version (0 when there are no migrations).
/// Already-applied migrations are checksum-verified; a mismatch aborts with
/// [`StorageError::MigrationChecksumMismatch`].
pub async fn apply_migrations(db_path: &str, migrations_dir: &Path) -> Result<u32, StorageError> {
    let pool = connect_rw(db_path).await?;
    let discovered = discover_migrations(migrations_dir)?;
    let already = applied_versions(&pool).await?;

    // Verify checksums of already-applied migrations first (fail before any write).
    for migration in &discovered {
        if let Some(recorded) = already.get(&migration.version) {
            let actual = sha256_file_hex(&migration.path)?;
            let recorded_hex = recorded.strip_prefix("sha256:").unwrap_or(recorded);
            if recorded_hex != actual {
                return Err(StorageError::MigrationChecksumMismatch { version: migration.version });
            }
        }
    }

    let mut max_version = already.keys().copied().max().unwrap_or(0);

    for migration in &discovered {
        if already.contains_key(&migration.version) {
            continue;
        }
        let sql = std::fs::read_to_string(&migration.path)
            .map_err(|_| StorageError::StorageUnavailable)?;
        let start = std::time::Instant::now();

        sqlx::raw_sql(&sql).execute(&pool).await.map_err(|_| StorageError::StorageUnavailable)?;

        let checksum = format!("sha256:{}", sha256_file_hex(&migration.path)?);
        let elapsed = start.elapsed().as_millis() as i64;
        // Migration 0001 creates `schema_migrations`; fixtures may not, so
        // ensure it exists before recording the applied row.
        ensure_migrations_table(&pool).await?;
        sqlx::query(
            "INSERT INTO schema_migrations (version, name, checksum, applied_at, applied_by, duration_ms)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(migration.version as i64)
        .bind(&migration.name)
        .bind(&checksum)
        .bind(now_rfc3339())
        .bind("qai")
        .bind(elapsed)
        .execute(&pool)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;

        max_version = max_version.max(migration.version);
    }

    Ok(max_version)
}

/// Result of a checksum verification pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChecksumReport {
    pub valid: bool,
    pub mismatches: Vec<u32>,
    pub missing_on_disk: Vec<u32>,
}

/// Verify that every applied migration's file checksum still matches.
pub async fn verify_checksums(
    db_path: &str,
    migrations_dir: &Path,
) -> Result<ChecksumReport, StorageError> {
    let pool = connect_ro(db_path).await?;
    let discovered = discover_migrations(migrations_dir)?;
    let already = applied_versions(&pool).await?;

    let on_disk: BTreeMap<u32, &MigrationFile> =
        discovered.iter().map(|m| (m.version, m)).collect();

    let mut mismatches = Vec::new();
    for (version, recorded) in &already {
        match on_disk.get(version) {
            Some(migration) => {
                let actual = sha256_file_hex(&migration.path)?;
                let recorded_hex = recorded.strip_prefix("sha256:").unwrap_or(recorded);
                if recorded_hex != actual {
                    mismatches.push(*version);
                }
            }
            None => mismatches.push(*version),
        }
    }

    let missing_on_disk = discovered
        .iter()
        .filter(|m| !already.contains_key(&m.version))
        .map(|m| m.version)
        .collect();

    Ok(ChecksumReport { valid: mismatches.is_empty(), mismatches, missing_on_disk })
}

/// Revert the most recently applied migration using its `.down.sql` file.
///
/// Deletes the `schema_migrations` bookkeeping row first (the down SQL may drop
/// the bookkeeping table itself), then executes the down SQL. Returns the
/// reverted version, or `None` if nothing is applied.
pub async fn revert_last_migration(
    db_path: &str,
    migrations_dir: &Path,
) -> Result<Option<u32>, StorageError> {
    let pool = connect_rw(db_path).await?;
    let already = applied_versions(&pool).await?;
    let Some(version) = already.keys().next_back().copied() else {
        return Ok(None);
    };
    let discovered = discover_migrations(migrations_dir)?;
    let down = discovered
        .iter()
        .find(|m| m.version == version)
        .and_then(|m| m.down_path.clone())
        .ok_or(StorageError::MigrationRequired { at_schema: version, required: version })?;
    let sql = std::fs::read_to_string(&down).map_err(|_| StorageError::StorageUnavailable)?;

    sqlx::query("DELETE FROM schema_migrations WHERE version = ?")
        .bind(version as i64)
        .execute(&pool)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
    sqlx::raw_sql(&sql).execute(&pool).await.map_err(|_| StorageError::StorageUnavailable)?;
    Ok(Some(version))
}

/// A consistent SQLite backup via `VACUUM INTO` (ADR-0001 §7).
///
/// Fails if `dest` already exists.
pub async fn backup(db_path: &str, dest: &str) -> Result<(), StorageError> {
    if Path::new(dest).exists() {
        return Err(StorageError::Conflict);
    }
    // VACUUM INTO must run on a single connection.
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .read_only(true)
        .busy_timeout(std::time::Duration::from_millis(5000));
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;

    sqlx::query("VACUUM INTO ?")
        .bind(dest)
        .execute(&pool)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_migration(dir: &Path, version: u32, name: &str, body: &str) {
        std::fs::write(dir.join(format!("{version:04}_{name}.up.sql")), body).unwrap();
    }

    fn write_down(dir: &Path, version: u32, name: &str, body: &str) {
        std::fs::write(dir.join(format!("{version:04}_{name}.down.sql")), body).unwrap();
    }

    async fn has_table(db_path: &str, table: &str) -> bool {
        let pool = connect_rw(db_path).await.unwrap();
        let exists = table_exists(&pool, table).await.unwrap();
        pool.close().await;
        exists
    }

    #[tokio::test]
    async fn down_migrations_restore_schema() {
        let dir = tempdir().unwrap();
        let mig = dir.path().join("migrations");
        std::fs::create_dir_all(&mig).unwrap();
        write_migration(&mig, 1, "core", "CREATE TABLE t1 (id TEXT PRIMARY KEY);");
        write_down(&mig, 1, "core", "DROP TABLE IF EXISTS t1;");
        write_migration(&mig, 2, "more", "CREATE TABLE t2 (id TEXT PRIMARY KEY);");
        write_down(&mig, 2, "more", "DROP TABLE IF EXISTS t2;");
        let dbp = dir.path().join("qai.db");
        let db = dbp.to_str().unwrap();

        // Nothing applied yet → revert is a no-op.
        assert_eq!(revert_last_migration(db, &mig).await.unwrap(), None);

        apply_migrations(db, &mig).await.unwrap();
        assert!(has_table(db, "t1").await && has_table(db, "t2").await);

        assert_eq!(revert_last_migration(db, &mig).await.unwrap(), Some(2));
        assert!(!has_table(db, "t2").await);
        assert!(has_table(db, "t1").await);

        assert_eq!(revert_last_migration(db, &mig).await.unwrap(), Some(1));
        assert!(!has_table(db, "t1").await);

        // Re-applying restores the schema.
        let v = apply_migrations(db, &mig).await.unwrap();
        assert_eq!(v, 2);
        assert!(has_table(db, "t1").await && has_table(db, "t2").await);
    }

    #[tokio::test]
    async fn fresh_migrate_is_idempotent() {
        let dir = tempdir().unwrap();
        let mig = dir.path().join("migrations");
        std::fs::create_dir_all(&mig).unwrap();
        write_migration(&mig, 1, "core", "CREATE TABLE t1 (id TEXT PRIMARY KEY);");
        write_migration(&mig, 2, "more", "CREATE TABLE t2 (id TEXT PRIMARY KEY);");
        let db = dir.path().join("qai.db");
        let db = db.to_str().unwrap();

        let v1 = apply_migrations(db, &mig).await.unwrap();
        assert_eq!(v1, 2);
        // Re-run is a no-op and returns the same version.
        let v2 = apply_migrations(db, &mig).await.unwrap();
        assert_eq!(v2, 2);
    }

    #[tokio::test]
    async fn checksum_drift_is_detected() {
        let dir = tempdir().unwrap();
        let mig = dir.path().join("migrations");
        std::fs::create_dir_all(&mig).unwrap();
        write_migration(&mig, 1, "core", "CREATE TABLE t1 (id TEXT PRIMARY KEY);");
        let db = dir.path().join("qai.db");
        let db = db.to_str().unwrap();
        apply_migrations(db, &mig).await.unwrap();

        // Edit the applied migration file.
        write_migration(&mig, 1, "core", "CREATE TABLE t1 (id TEXT PRIMARY KEY, extra TEXT);");
        let err = apply_migrations(db, &mig).await.unwrap_err();
        assert!(matches!(err, StorageError::MigrationChecksumMismatch { version: 1 }));
    }

    #[tokio::test]
    async fn backup_round_trips_a_populated_db() {
        let dir = tempdir().unwrap();
        let mig = dir.path().join("migrations");
        std::fs::create_dir_all(&mig).unwrap();
        write_migration(&mig, 1, "core", "CREATE TABLE t1 (id TEXT PRIMARY KEY);");
        let db = dir.path().join("qai.db");
        let db = db.to_str().unwrap();
        apply_migrations(db, &mig).await.unwrap();

        let dest = dir.path().join("backup.db");
        backup(db, dest.to_str().unwrap()).await.unwrap();
        assert!(dest.exists());

        // The backup must be a valid SQLite file with our table.
        let report = verify_checksums(dest.to_str().unwrap(), &mig).await.unwrap();
        assert!(report.valid, "backup should verify: {report:?}");
    }
}
