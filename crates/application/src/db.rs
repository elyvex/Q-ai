//! Database orchestration for the CLI: migrate, status, verify, backup, probe.
//!
//! Keeps `storage-sqlite` behind the application boundary (arch-check: the CLI
//! may depend on `application` but not on `storage-sqlite` directly).

use std::path::Path;

use config::Config;
use storage::Database as _;
use storage::error::StorageError;
use storage_sqlite::{SqliteDatabase, migrate};

/// Re-exported so the CLI can map catalog errors to exit codes without
/// taking a direct dependency on `storage` (arch-check boundary).
pub use storage::error::StorageError as CatalogError;

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

/// Restore from a backup after verifying its integrity and migration checksums.
///
/// The backup (produced by `VACUUM INTO`) is a complete, standalone snapshot, so
/// copying it into place is safe — unlike copying a live WAL database (ADR-0001 §7).
/// The current database is moved aside as `<path>.pre-restore` rather than deleted.
pub async fn restore_database(
    cfg: &Config,
    backup: &str,
    migrations_dir: &Path,
) -> Result<(), StorageError> {
    // 1. The backup must exist and be a valid, checksum-consistent database.
    let report = migrate::verify_checksums(backup, migrations_dir).await?;
    if !report.valid {
        return Err(StorageError::MigrationChecksumMismatch {
            version: report.mismatches.first().copied().unwrap_or(0),
        });
    }
    let probe = SqliteDatabase::open_read_only(backup).await?;
    let integrity = probe.probe_scalar("PRAGMA integrity_check").await.ok().flatten();
    if !integrity.as_deref().map(|s| s.eq_ignore_ascii_case("ok")).unwrap_or(false) {
        return Err(StorageError::StorageUnavailable);
    }
    drop(probe);

    // 2. Swap in the verified snapshot.
    let target = &cfg.storage.sqlite.path;
    if Path::new(target).exists() {
        let aside = format!("{target}.pre-restore");
        std::fs::rename(target, &aside).map_err(|_| StorageError::StorageUnavailable)?;
    }
    // Remove stale WAL/SHM from the previous database.
    let _ = std::fs::remove_file(format!("{target}-wal"));
    let _ = std::fs::remove_file(format!("{target}-shm"));
    std::fs::copy(backup, target).map_err(|_| StorageError::StorageUnavailable)?;
    Ok(())
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

// ─── Read-only catalog listings (P0-T50 / FU-10) ─────────────────────
//
// All helpers open the database read-only and never write. Rows are fetched
// as `json_object(...)` single-column strings through the `ReadTx`
// abstraction (no new storage-trait methods, no SQL interpolation of user
// input: `show` filters in Rust), parsed, redacted, and returned as JSON.

const CATALOG_LIMIT: usize = 1000;

fn parse_catalog_rows(rows: Vec<String>) -> Result<Vec<serde_json::Value>, StorageError> {
    rows.into_iter()
        .map(|row| serde_json::from_str(&row).map_err(|_| StorageError::StorageUnavailable))
        .collect()
}

fn catalog_page(mut items: Vec<serde_json::Value>) -> serde_json::Value {
    let truncated = items.len() > CATALOG_LIMIT;
    items.truncate(CATALOG_LIMIT);
    let mut page = serde_json::json!({"items": items, "truncated": truncated});
    domain::redaction::redact_json_value(&mut page);
    page
}

/// List all sources (id, title, content_type, language, created_at).
pub async fn list_sources(path: &str) -> Result<serde_json::Value, StorageError> {
    let db = SqliteDatabase::open_read_only(path).await?;
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT json_object('id', id, 'title', title, 'content_type', content_type, \
         'language', language, 'created_at', created_at) \
         FROM sources ORDER BY id LIMIT 1001",
    )
    .fetch_all(db.read_pool())
    .await
    .map_err(|_| StorageError::StorageUnavailable)?;
    Ok(catalog_page(parse_catalog_rows(rows)?))
}

/// Show one source with its versions. Versions expose only review-relevant
/// metadata (no manifest blobs).
pub async fn get_source(path: &str, id: &str) -> Result<serde_json::Value, StorageError> {
    let db = SqliteDatabase::open_read_only(path).await?;
    let row = sqlx::query_scalar::<_, String>(
        "SELECT json_object('id', id, 'title', title, 'content_type', content_type, \
         'language', language, 'created_at', created_at) \
         FROM sources WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(db.read_pool())
    .await
    .map_err(|_| StorageError::StorageUnavailable)?
    .ok_or_else(|| StorageError::NotFound { urn: format!("source:{id}") })?;
    let mut source: serde_json::Value =
        serde_json::from_str(&row).map_err(|_| StorageError::StorageUnavailable)?;
    let version_rows = sqlx::query_scalar::<_, String>(
        "SELECT json_object('id', id, 'version', version, 'state', state, \
         'trust_level', trust_level, 'license_status', license_status, \
         'content_hash', content_hash) \
         FROM source_versions WHERE source_id = ? ORDER BY version LIMIT 1001",
    )
    .bind(id)
    .fetch_all(db.read_pool())
    .await
    .map_err(|_| StorageError::StorageUnavailable)?;
    let mut versions = parse_catalog_rows(version_rows)?;
    for version in &mut versions {
        domain::redaction::redact_json_value(version);
    }
    source["versions"] = serde_json::Value::Array(versions);
    domain::redaction::redact_json_value(&mut source);
    Ok(source)
}

/// List all jobs (metadata only; payloads excluded from listings).
pub async fn list_jobs(path: &str) -> Result<serde_json::Value, StorageError> {
    let db = SqliteDatabase::open_read_only(path).await?;
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT json_object('id', id, 'kind', kind, 'state', state, \
         'priority', priority, 'attempts', attempts, 'max_attempts', max_attempts, \
         'available_at', available_at, 'cancel_requested', cancel_requested) \
         FROM jobs ORDER BY id LIMIT 1001",
    )
    .fetch_all(db.read_pool())
    .await
    .map_err(|_| StorageError::StorageUnavailable)?;
    Ok(catalog_page(parse_catalog_rows(rows)?))
}

/// Redact a JSON-encoded string column: parse it, apply key-level redaction
/// to the nested value, and re-serialize. String payloads such as
/// `{"password":"..."}` evade free-text credential scrubbing (no `=`/`:` separator
/// after the key), so nested parsing is required. Falls back to free-text
/// scrubbing when the column is not valid JSON.
fn redact_json_text_column(raw: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(mut value) => {
            domain::redaction::redact_json_value(&mut value);
            serde_json::to_string(&value).unwrap_or_else(|_| raw.to_string())
        }
        Err(_) => domain::redaction::redact_text(raw).into_owned(),
    }
}

/// Show one job, including its (redacted) payload.
pub async fn get_job(path: &str, id: &str) -> Result<serde_json::Value, StorageError> {
    let db = SqliteDatabase::open_read_only(path).await?;
    let row = sqlx::query_scalar::<_, String>(
        "SELECT json_object('id', id, 'kind', kind, 'state', state, \
         'priority', priority, 'attempts', attempts, 'max_attempts', max_attempts, \
         'available_at', available_at, 'cancel_requested', cancel_requested, \
         'payload_json', payload_json, 'idempotency_key', idempotency_key, \
         'lease_owner', lease_owner, 'checkpoint_json', checkpoint_json, \
         'created_by', created_by) \
         FROM jobs WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(db.read_pool())
    .await
    .map_err(|_| StorageError::StorageUnavailable)?
    .ok_or_else(|| StorageError::NotFound { urn: format!("job:{id}") })?;
    let mut job: serde_json::Value =
        serde_json::from_str(&row).map_err(|_| StorageError::StorageUnavailable)?;
    for column in ["payload_json", "checkpoint_json"] {
        if let Some(raw) = job.get(column).and_then(|value| value.as_str()) {
            job[column] = serde_json::Value::String(redact_json_text_column(raw));
        }
    }
    domain::redaction::redact_json_value(&mut job);
    Ok(job)
}

/// List audit events (envelope only; before/after payloads excluded).
pub async fn list_audit_events(path: &str) -> Result<serde_json::Value, StorageError> {
    let db = SqliteDatabase::open_read_only(path).await?;
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT json_object('id', id, 'sequence', sequence, 'occurred_at', occurred_at, \
         'actor_kind', actor_kind, 'action', action, 'outcome', outcome, \
         'subject_urn', subject_urn) \
         FROM audit_events ORDER BY sequence LIMIT 1001",
    )
    .fetch_all(db.read_pool())
    .await
    .map_err(|_| StorageError::StorageUnavailable)?;
    Ok(catalog_page(parse_catalog_rows(rows)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn migrations_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite")
    }

    /// Highest numbered `NNNN_*.up.sql` present, i.e. the schema version a
    /// full migration run lands on. Derived from disk so adding a migration
    /// never breaks these tests.
    fn latest_migration_version() -> u32 {
        let mut latest = 0u32;
        for entry in std::fs::read_dir(migrations_dir()).unwrap().flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".up.sql") {
                continue;
            }
            let digits: String = name.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(number) = digits.parse::<u32>() {
                latest = latest.max(number);
            }
        }
        latest
    }

    #[tokio::test]
    async fn migrate_then_probe() {
        let dir = tempdir().unwrap();
        let mut cfg = Config::default();
        cfg.app.data_dir = dir.path().display().to_string();
        cfg.storage.sqlite.path = dir.path().join("qai.db").display().to_string();

        let version = migrate_database(&cfg, &migrations_dir()).await.unwrap();
        assert_eq!(version, latest_migration_version());

        let probe = probe_database(&cfg).await;
        assert!(probe.reachable);
        assert!(probe.integrity_ok);
        assert!(probe.foreign_keys_on);
        assert_eq!(probe.schema_version, latest_migration_version());
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
        assert_eq!(status.latest_on_disk, latest_migration_version());

        let dest = dir.path().join("bk.db");
        backup_database(&cfg, dest.to_str().unwrap()).await.unwrap();
        assert!(dest.exists());
    }

    #[tokio::test]
    async fn restore_verifies_and_swaps_in_a_backup() {
        let dir = tempdir().unwrap();
        let mut cfg = Config::default();
        cfg.storage.sqlite.path = dir.path().join("qai.db").display().to_string();
        migrate_database(&cfg, &migrations_dir()).await.unwrap();

        let backup = dir.path().join("bk.db");
        backup_database(&cfg, backup.to_str().unwrap()).await.unwrap();

        // Corrupt the live database, then restore the verified snapshot.
        std::fs::write(&cfg.storage.sqlite.path, b"not a database").unwrap();
        restore_database(&cfg, backup.to_str().unwrap(), &migrations_dir()).await.unwrap();

        let probe = probe_database(&cfg).await;
        assert!(probe.reachable && probe.integrity_ok);
    }

    #[tokio::test]
    async fn restore_rejects_a_missing_backup() {
        let dir = tempdir().unwrap();
        let mut cfg = Config::default();
        cfg.storage.sqlite.path = dir.path().join("qai.db").display().to_string();
        let missing = dir.path().join("nope.db");
        let err = restore_database(&cfg, missing.to_str().unwrap(), &migrations_dir()).await;
        assert!(err.is_err());
    }

    const CATALOG_SENTINEL: &str = "SENTINEL_9f3c__DO_NOT_LEAK";

    async fn seed_catalog(path: &str) {
        use storage::Database as _;
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(path))
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO sources (id, title, content_type, language, created_at, updated_at) \
             VALUES ('src-1', 'Seed Source', 'quran_edition', 'ar', \
             '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO source_versions (id, source_id, version, schema_version, state, \
             trust_level, license_status, license_json, content_hash, created_at) \
             VALUES ('ver-1', 'src-1', '1.0.0', 1, 'Staged', 'ImportedUnverified', \
             'OpenLicense', '{}', 'sha256:aa', '2026-01-01T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .unwrap();
        let payload = format!(r#"{{"n":1,"password":"{CATALOG_SENTINEL}"}}"#);
        sqlx::query(
            "INSERT INTO jobs (id, kind, payload_json, state, priority, attempts, \
             max_attempts, available_at, created_at) \
             VALUES ('job-1', 'system.noop_test', ?, 'Queued', 0, 0, 5, \
             '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        )
        .bind(&payload)
        .execute(&pool)
        .await
        .unwrap();
        pool.close().await;

        let db = storage_sqlite::SqliteDatabase::new(path, 1, true).await.unwrap();
        let mut uow = db.write().await.unwrap();
        let event = audit::AuditEvent {
            id: "00000000-0000-4000-8000-000000000001".parse().unwrap(),
            sequence: 0,
            occurred_at: domain::Timestamp::from_ymd_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            actor: audit::Actor::System { name: "catalog-test".into() },
            action: audit::AuditAction::SourceStaged,
            subject: domain::SubjectRef("urn:qai:test:catalog".into()),
            outcome: audit::AuditOutcome::Allowed,
            reason: None,
            before: None,
            after: None,
            request_id: None,
            prev_chain_hash: domain::ContentHash {
                algorithm: domain::HashAlgorithm::Sha256,
                hex: "00".repeat(32),
            },
            chain_hash: domain::ContentHash {
                algorithm: domain::HashAlgorithm::Sha256,
                hex: "ff".repeat(32),
            },
        };
        crate::audit_bridge::append_audit_event(&mut *uow, event).await.unwrap();
        uow.commit().await.unwrap();
    }

    #[tokio::test]
    async fn catalog_lists_seed_rows_and_show_round_trips() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("qai.db");
        let path = path.to_str().unwrap();
        let mut cfg = Config::default();
        cfg.storage.sqlite.path = path.to_string();
        migrate_database(&cfg, &migrations_dir()).await.unwrap();

        let empty_sources = list_sources(path).await.unwrap();
        assert_eq!(empty_sources["items"].as_array().unwrap().len(), 0);
        assert_eq!(empty_sources["truncated"], false);
        assert!(get_source(path, "missing").await.is_err());
        assert!(get_job(path, "missing").await.is_err());

        seed_catalog(path).await;

        let sources = list_sources(path).await.unwrap();
        let items = sources["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["id"], "src-1");
        assert_eq!(items[0]["title"], "Seed Source");
        assert!(items[0].get("license_json").is_none(), "listings omit blobs");

        let source = get_source(path, "src-1").await.unwrap();
        assert_eq!(source["id"], "src-1");
        let versions = source["versions"].as_array().unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0]["version"], "1.0.0");
        assert_eq!(versions[0]["state"], "Staged");
        assert!(versions[0].get("source_id").is_none());

        let jobs = list_jobs(path).await.unwrap();
        let items = jobs["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["id"], "job-1");
        assert!(items[0].get("payload_json").is_none(), "listings omit payloads");

        let job = get_job(path, "job-1").await.unwrap();
        assert_eq!(job["kind"], "system.noop_test");
        let rendered = serde_json::to_string(&job).unwrap();
        assert!(!rendered.contains(CATALOG_SENTINEL), "payload secrets redacted");
        assert!(rendered.contains(domain::redaction::REDACTED_MARKER));

        let events = list_audit_events(path).await.unwrap();
        let items = events["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["action"], "source_staged");
        assert!(items[0].get("before_json").is_none());
        assert!(items[0].get("after_json").is_none());
    }

    #[tokio::test]
    async fn catalog_helpers_never_modify_the_database() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("qai.db");
        let path = path.to_str().unwrap();
        let mut cfg = Config::default();
        cfg.storage.sqlite.path = path.to_string();
        migrate_database(&cfg, &migrations_dir()).await.unwrap();
        seed_catalog(path).await;

        // Logical content comparison (not raw bytes: pager flushes from the
        // seed pool's background close can land after the snapshot).
        async fn dump(path: &str) -> Vec<String> {
            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(path))
                .await
                .unwrap();
            let mut out = Vec::new();
            for (table, order) in [
                ("sources", "id"),
                ("source_versions", "id"),
                ("jobs", "id"),
                ("audit_events", "sequence"),
            ] {
                // Whole-row dump (ordered) so INSERTs, DELETEs, and UPDATEs
                // are all visible to the comparison below.
                let cols: String = sqlx::query_scalar::<_, String>(&format!(
                    "SELECT group_concat(name, ',') FROM pragma_table_info('{table}')"
                ))
                .fetch_one(&pool)
                .await
                .unwrap();
                let concat = cols
                    .split(',')
                    .map(|column| format!("COALESCE(quote({column}), 'nil')"))
                    .collect::<Vec<_>>()
                    .join(" || '|' || ");
                let full: Vec<String> = sqlx::query_scalar::<_, String>(&format!(
                    "SELECT {concat} FROM {table} ORDER BY {order}"
                ))
                .fetch_all(&pool)
                .await
                .unwrap();
                out.push(format!("{table}:{}", full.join(",")));
            }
            let counts: Vec<i64> = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM sources UNION ALL SELECT COUNT(*) FROM source_versions \
                 UNION ALL SELECT COUNT(*) FROM jobs UNION ALL SELECT COUNT(*) FROM audit_events",
            )
            .fetch_all(&pool)
            .await
            .unwrap();
            out.push(format!("counts:{counts:?}"));
            pool.close().await;
            out
        }
        let before = dump(path).await;
        list_sources(path).await.unwrap();
        get_source(path, "src-1").await.unwrap();
        list_jobs(path).await.unwrap();
        get_job(path, "job-1").await.unwrap();
        list_audit_events(path).await.unwrap();
        assert!(get_source(path, "missing").await.is_err());
        assert_eq!(before, dump(path).await);
    }
}
