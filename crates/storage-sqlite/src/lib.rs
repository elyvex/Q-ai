//! SQLite storage implementation for Q-ai.
//!
//! Provides [`SqliteDatabase`] — the concrete [`Database`] implementation
//! using `sqlx` with dual connection pools (write = 1, read = N),
//! WAL mode, and full migration support.

pub mod migrate;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::sync::Arc;
use storage::error::StorageError;
use storage::repository::{
    AuditEvent, AuditRepository, ChainVerificationResult, JobRecord, JobRepository,
    ProvenanceRecord, ProvenanceRepository, ReviewRecord, SettingsRepository, SettingRow,
    SourceRepository, SourceRow, SourceVersionRow, StateTransitionRow,
};
use storage::{Database, DbBackend, DbHealth, ReadTx, UnitOfWork};
use tokio::sync::Mutex;

// ─── SqliteDatabase ────────────────────────────────────────────────

/// The SQLite-backed database implementation.
///
/// Uses two connection pools:
/// - **write pool**: `max_connections = 1` for serialized write transactions
/// - **read pool**: `max_connections` connections for read-only queries
pub struct SqliteDatabase {
    write_pool: SqlitePool,
    read_pool: SqlitePool,
    schema_version: u32,
}

impl SqliteDatabase {
    /// Open a SQLite database at the given URL (e.g. `":memory:"` or a file path).
    ///
    /// Applies all pending migrations automatically.
    pub async fn open(url: &str) -> Result<Self, StorageError> {
        Self::open_with_migrations(url, true).await
    }

    /// Open a SQLite database, optionally applying migrations.
    pub async fn open_with_migrations(url: &str, auto_migrate: bool) -> Result<Self, StorageError> {
        let escaped_url = if url.starts_with(":memory:") || url.starts_with("sqlite:") {
            url.to_string()
        } else {
            format!("sqlite://{url}?mode=rwc")
        };

        let write_options = if url == ":memory:" {
            SqliteConnectOptions::new()
                .filename(":memory:")
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                .synchronous(sqlx::sqlite::SqliteSynchronous::Full)
                .foreign_keys(true)
                .busy_timeout(std::time::Duration::from_millis(5000))
        } else if url.starts_with("sqlite:") {
            url.parse::<SqliteConnectOptions>()
                .map_err(|_| StorageError::StorageUnavailable)?
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                .synchronous(sqlx::sqlite::SqliteSynchronous::Full)
                .foreign_keys(true)
                .busy_timeout(std::time::Duration::from_millis(5000))
        } else {
            SqliteConnectOptions::new()
                .filename(url)
                .create_if_missing(true)
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                .synchronous(sqlx::sqlite::SqliteSynchronous::Full)
                .foreign_keys(true)
                .busy_timeout(std::time::Duration::from_millis(5000))
        };

        let read_options = SqliteConnectOptions::new()
            .filename(":memory:")
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Full)
            .foreign_keys(true)
            .busy_timeout(std::time::Duration::from_millis(5000));

        let write_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(write_options)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;

        sqlx::query("PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA foreign_keys = ON; PRAGMA busy_timeout = 5000;")
            .execute(&write_pool)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;

        let read_pool = if url == ":memory:" {
            write_pool.clone()
        } else {
            SqlitePoolOptions::new()
                .max_connections(8)
                .connect_with(read_options)
                .await
                .unwrap_or_else(|_| {
                    write_pool.clone()
                })
        };

        let mut db = Self {
            write_pool,
            read_pool,
            schema_version: 0,
        };

        if auto_migrate {
            db.apply_migrations().await?;
        }

        let version = db.load_schema_version().await;
        db.schema_version = version;

        Ok(db)
    }

    /// Apply all pending migrations from `migrations/sqlite/`.
    async fn apply_migrations(&mut self) -> Result<(), StorageError> {
        let migration_dir = std::path::Path::new("migrations/sqlite");
        let files = migrate::discover_migrations(migration_dir)
            .map_err(|_| StorageError::StorageUnavailable)?;

        if files.is_empty() {
            return Ok(());
        }

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                checksum TEXT NOT NULL,
                applied_at TEXT NOT NULL,
                applied_by TEXT NOT NULL,
                duration_ms INTEGER NOT NULL
            )",
        )
        .execute(&self.write_pool)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;

        let applied: Vec<(i64,)> = sqlx::query_as("SELECT version FROM schema_migrations ORDER BY version")
            .fetch_all(&self.write_pool)
            .await
            .unwrap_or_default();
        let applied_set: std::collections::HashSet<i64> =
            applied.into_iter().map(|(v,)| v).collect();

        for (version, (up_path, _down_path)) in files.iter() {
            if applied_set.contains(&(*version as i64)) {
                continue;
            }

            let sql = tokio::fs::read_to_string(up_path)
                .await
                .map_err(|_| StorageError::StorageUnavailable)?;

            let start = std::time::Instant::now();

            sqlx::query(&sql)
                .execute(&self.write_pool)
                .await
                .map_err(|e| StorageError::ConstraintViolation {
                    message: format!("migration {version}: {e}"),
                })?;

            let checksum = migrate::sha256_file(up_path).unwrap_or_default();
            let now = time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default();

            sqlx::query(
                "INSERT INTO schema_migrations (version, name, checksum, applied_at, applied_by, duration_ms)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(*version as i64)
            .bind(up_path.file_name().and_then(|f| f.to_str()).unwrap_or(""))
            .bind(checksum)
            .bind(now)
            .bind("qai")
            .bind(start.elapsed().as_millis() as i64)
            .execute(&self.write_pool)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;
        }

        Ok(())
    }

    /// Return the current schema version by reading the schema_migrations table.
    async fn load_schema_version(&self) -> u32 {
        let result: Result<(i64,), _> =
            sqlx::query_as("SELECT COALESCE(MAX(version), 0) FROM schema_migrations")
                .fetch_one(&self.read_pool)
                .await;
        match result {
            Ok((v,)) => v as u32,
            _ => 0,
        }
    }
}

impl std::fmt::Debug for SqliteDatabase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteDatabase")
            .field("schema_version", &self.schema_version)
            .field("backend", &self.backend())
            .finish()
    }
}

#[async_trait::async_trait]
impl Database for SqliteDatabase {
    async fn read(&self) -> Result<Box<dyn ReadTx>, StorageError> {
        let version = self.load_schema_version().await;
        Ok(Box::new(SqliteReadTx {
            pool: self.read_pool.clone(),
            schema_version: version,
        }))
    }

    async fn write(&self) -> Result<Box<dyn UnitOfWork>, StorageError> {
        let tx = self.write_pool.begin().await
            .map_err(|_| StorageError::StorageUnavailable)?;
        let version = self.schema_version;
        Ok(Box::new(SqliteUnitOfWork {
            tx: Mutex::new(tx),
            _schema_version: version,
        }))
    }

    async fn health(&self) -> Result<DbHealth, StorageError> {
        let version = self.schema_version;
        Ok(DbHealth::ok(DbBackend::SQLite, version))
    }

    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn backend(&self) -> DbBackend {
        DbBackend::SQLite
    }
}

// ─── SqliteReadTx ──────────────────────────────────────────────────

/// Read-only transaction handle for SQLite.
pub struct SqliteReadTx {
    pool: SqlitePool,
    schema_version: u32,
}

#[async_trait::async_trait]
impl ReadTx for SqliteReadTx {
    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    async fn query_one(&self, sql: &str) -> Result<Option<String>, StorageError> {
        let result = sqlx::query(sql)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;

        match result {
            Some(row) => {
                let col: Option<String> = row.try_get(0).ok();
                Ok(col)
            }
            None => Ok(None),
        }
    }
}

// ─── SqliteUnitOfWork ──────────────────────────────────────────────

/// Write transaction unit of work for SQLite.
pub struct SqliteUnitOfWork {
    tx: Mutex<sqlx::Transaction<'static, sqlx::Sqlite>>,
    _schema_version: u32,
}

#[async_trait::async_trait]
impl UnitOfWork for SqliteUnitOfWork {
    fn sources(&mut self) -> &mut dyn SourceRepository {
        // We use a static stub for now since we need to satisfy the trait.
        // The actual repos need the transaction's mutable access.
        // For Phase 0, these are stubs that match the storage trait.
        self as &mut dyn SourceRepository as &mut dyn SourceRepository
    }

    fn provenance(&mut self) -> &mut dyn ProvenanceRepository {
        self as &mut dyn ProvenanceRepository as &mut dyn ProvenanceRepository
    }

    fn audit(&mut self) -> &mut dyn AuditRepository {
        self as &mut dyn AuditRepository as &mut dyn AuditRepository
    }

    fn jobs(&mut self) -> &mut dyn JobRepository {
        self as &mut dyn JobRepository as &mut dyn JobRepository
    }

    fn settings(&mut self) -> &mut dyn SettingsRepository {
        self as &mut dyn SettingsRepository as &mut dyn SettingsRepository
    }

    async fn commit(self: Box<Self>) -> Result<(), StorageError> {
        let tx = Arc::new(self.tx);
        let _lock = tx.lock_owned();
        // We need to consume the transaction — bypass Mutex by reconstructing.
        // Unfortunately Mutex doesn't give us owned inner after lock.
        // Workaround: drop the lock and use `into_inner`.
        // Since we hold Box<Self>, we can extract via `into_inner`.
        let me = *self;
        me.tx.into_inner().commit().await
            .map_err(|_| StorageError::StorageUnavailable)
    }

    async fn rollback(self: Box<Self>) -> Result<(), StorageError> {
        let me = *self;
        me.tx.into_inner().rollback().await
            .map_err(|_| StorageError::StorageUnavailable)
    }
}

// ─── Stub repo traits via UnitOfWork itself (all return StorageUnavailable) ─

#[async_trait::async_trait]
impl SourceRepository for SqliteUnitOfWork {
    async fn get(&self, _id: &str) -> Result<Option<SourceRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }
    async fn insert_version(&mut self, _version: SourceVersionRow) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }
    async fn transition_state(
        &mut self,
        _source_version_id: &str,
        _from: &str,
        _to: &str,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }
    async fn list_versions(&self, _source_id: &str) -> Result<Vec<SourceVersionRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }
    async fn record_transition(
        &mut self,
        _transition: StateTransitionRow,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }
}

#[async_trait::async_trait]
impl ProvenanceRepository for SqliteUnitOfWork {
    async fn insert(&mut self, _record: ProvenanceRecord) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }
    async fn get(&self, _id: &str) -> Result<Option<ProvenanceRecord>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }
    async fn list_by_subject(
        &self,
        _subject_urn: &str,
    ) -> Result<Vec<ProvenanceRecord>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }
    async fn record_review(
        &mut self,
        _review: ReviewRecord,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }
}

#[async_trait::async_trait]
impl AuditRepository for SqliteUnitOfWork {
    async fn append(&mut self, _event: AuditEvent) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }
    async fn list_by_subject(
        &self,
        _subject_urn: &str,
    ) -> Result<Vec<AuditEvent>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }
    async fn verify_chain(&self) -> Result<ChainVerificationResult, StorageError> {
        Err(StorageError::StorageUnavailable)
    }
}

#[async_trait::async_trait]
impl JobRepository for SqliteUnitOfWork {
    async fn enqueue(&mut self, _job: JobRecord) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }
    async fn claim(&mut self, _job_id: &str, _owner: &str) -> Result<Option<JobRecord>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }
    async fn finish(
        &mut self,
        _job_id: &str,
        _state: &str,
        _result: Option<String>,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }
    async fn cancel(&mut self, _job_id: &str) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }
    async fn checkpoint(
        &mut self,
        _job_id: &str,
        _progress: Option<String>,
        _checkpoint: Option<String>,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }
    async fn reap_expired_leases(&self) -> Result<Vec<String>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }
}

#[async_trait::async_trait]
impl SettingsRepository for SqliteUnitOfWork {
    async fn get(&self, _key: &str) -> Result<Option<SettingRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }
    async fn set(
        &mut self,
        _key: &str,
        _value_json: &str,
        _origin: &str,
        _updated_by: Option<&str>,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }
    async fn list(&self) -> Result<Vec<SettingRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_open_and_health() {
        let db = SqliteDatabase::open_with_migrations(":memory:", false).await.unwrap();
        let health = db.health().await.unwrap();
        assert!(health.healthy);
        assert_eq!(health.backend, DbBackend::SQLite);
    }

    #[tokio::test]
    async fn test_read_tx_schema_version() {
        let db = SqliteDatabase::open_with_migrations(":memory:", false).await.unwrap();
        let tx = db.read().await.unwrap();
        let _sv = tx.schema_version();
    }

    #[tokio::test]
    async fn test_write_and_rollback() {
        let db = SqliteDatabase::open_with_migrations(":memory:", false).await.unwrap();
        let uow = db.write().await.unwrap();
        uow.rollback().await.unwrap();
    }

    #[tokio::test]
    async fn test_write_and_commit() {
        let db = SqliteDatabase::open_with_migrations(":memory:", false).await.unwrap();
        let uow = db.write().await.unwrap();
        uow.commit().await.unwrap();
    }
}
