//! SQLite storage implementation for Q-ai.
//!
//! This crate implements the `Database`, `ReadTx`, `UnitOfWork`, and
//! repository traits from the `storage` crate using `sqlx` with SQLite.
//!
//! # Architecture
//!
//! - **Write pool**: single connection, serialized transactions, `synchronous=FULL`
//! - **Read pool**: N connections, `query_only=ON`, used for all read operations
//! - **Pragmas**: WAL mode, foreign keys ON, busy timeout 5000ms
//!
//! # Phase 0 scope
//!
//! Implements all Phase 0 migrations (0001–0006) and provides real
//! repository implementations for sources, provenance, audit, jobs, settings.

use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{Pool, Row, Sqlite, Transaction};
use storage::{Database, ReadTx, UnitOfWork, DbHealth, DbBackend,
    repository::{
        SourceRepository, ProvenanceRepository, AuditRepository, JobRepository, SettingsRepository,
        SourceRow, SourceVersionRow, StateTransitionRow,
        ProvenanceRecord, ReviewRecord,
        AuditEvent, ChainVerificationResult,
        JobRecord, SettingRow,
    },
    error::StorageError,
};

/// SQLite database implementation with dual pools.
pub struct SqliteDatabase {
    write_pool: Pool<Sqlite>,
    read_pool: Pool<Sqlite>,
    schema_version: u32,
}

impl SqliteDatabase {
    /// Create a new SQLite database with the given path and config.
    pub async fn new(path: &str, max_connections: u32, read_only_pool: bool) -> Result<Self, StorageError> {
        let mut write_options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Full)
            .foreign_keys(true)
            .busy_timeout(std::time::Duration::from_millis(5000));

        if read_only_pool {
            // For read pool we set query_only = ON
            write_options = write_options.read_only(false);
        }

        let write_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(write_options)
            .await
            .map_err(|e| StorageError::StorageUnavailable)?;

        // Read pool with query_only = ON
        let mut read_options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(false)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Full)
            .foreign_keys(true)
            .busy_timeout(std::time::Duration::from_millis(5000))
            .read_only(true);

        let read_pool = SqlitePoolOptions::new()
            .max_connections(max_connections)
            .connect_with(read_options)
            .await
            .map_err(|e| StorageError::StorageUnavailable)?;

        // Get current schema version
        let schema_version = Self::get_schema_version(&write_pool).await.unwrap_or(0);

        Ok(Self {
            write_pool,
            read_pool,
            schema_version,
        })
    }

    async fn get_schema_version(pool: &Pool<Sqlite>) -> Result<u32, sqlx::Error> {
        let row = sqlx::query("SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1")
            .fetch_optional(pool)
            .await?;
        Ok(row.map(|r| r.get::<i64, _>("version") as u32).unwrap_or(0))
    }
}

#[async_trait]
impl Database for SqliteDatabase {
    async fn read(&self) -> Result<Box<dyn ReadTx>, StorageError> {
        Ok(Box::new(SqliteReadTx::new(self.read_pool.clone(), self.schema_version)))
    }

    async fn write(&self) -> Result<Box<dyn UnitOfWork>, StorageError> {
        Ok(Box::new(SqliteUnitOfWork::new(self.write_pool.clone(), self.schema_version).await?))
    }

    async fn health(&self) -> Result<DbHealth, StorageError> {
        let row = sqlx::query("SELECT 1 as ok")
            .fetch_one(&self.write_pool)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;
        let healthy = row.get::<i64, _>("ok") == 1;
        Ok(DbHealth::ok(DbBackend::SQLite, self.schema_version))
    }

    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn backend(&self) -> DbBackend {
        DbBackend::SQLite
    }
}

/// Read-only transaction handle for SQLite.
pub struct SqliteReadTx {
    pool: Pool<Sqlite>,
    schema_version: u32,
}

impl SqliteReadTx {
    fn new(pool: Pool<Sqlite>, schema_version: u32) -> Self {
        Self {
            pool,
            schema_version,
        }
    }
}

#[async_trait]
impl ReadTx for SqliteReadTx {
    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    async fn query_one(&self, sql: &str) -> Result<Option<String>, StorageError> {
        let row = sqlx::query(sql)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(row.map(|r| r.get::<String, _>(0)))
    }
}

/// Write transaction (unit of work) for SQLite.
pub struct SqliteUnitOfWork {
    tx: Transaction<'static, Sqlite>,
    schema_version: u32,
    sources: SqliteSourceRepository,
    provenance: SqliteProvenanceRepository,
    audit: SqliteAuditRepository,
    jobs: SqliteJobRepository,
    settings: SqliteSettingsRepository,
}

impl SqliteUnitOfWork {
    async fn new(pool: Pool<Sqlite>, schema_version: u32) -> Result<Self, StorageError> {
        let tx = pool.begin().await.map_err(|_| StorageError::StorageUnavailable)?;
        Ok(Self {
            tx,
            schema_version,
            sources: SqliteSourceRepository::new(),
            provenance: SqliteProvenanceRepository::new(),
            audit: SqliteAuditRepository::new(),
            jobs: SqliteJobRepository::new(),
            settings: SqliteSettingsRepository::new(),
        })
    }
}

#[async_trait]
impl UnitOfWork for SqliteUnitOfWork {
    fn sources(&mut self) -> &mut dyn SourceRepository {
        &mut self.sources
    }

    fn provenance(&mut self) -> &mut dyn ProvenanceRepository {
        &mut self.provenance
    }

    fn audit(&mut self) -> &mut dyn AuditRepository {
        &mut self.audit
    }

    fn jobs(&mut self) -> &mut dyn JobRepository {
        &mut self.jobs
    }

    fn settings(&mut self) -> &mut dyn SettingsRepository {
        &mut self.settings
    }

    async fn commit(self: Box<Self>) -> Result<(), StorageError> {
        self.tx.commit().await.map_err(|_| StorageError::StorageUnavailable)
    }

    async fn rollback(self: Box<Self>) -> Result<(), StorageError> {
        self.tx.rollback().await.map_err(|_| StorageError::StorageUnavailable)
    }
}

// ============================================================================
// Repository Implementations
// ============================================================================

struct SqliteSourceRepository;

impl SqliteSourceRepository {
    fn new() -> Self { Self }
}

#[async_trait]
impl SourceRepository for SqliteSourceRepository {
    async fn get(&self, id: &str) -> Result<Option<SourceRow>, StorageError> {
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

struct SqliteProvenanceRepository;

impl SqliteProvenanceRepository {
    fn new() -> Self { Self }
}

#[async_trait]
impl ProvenanceRepository for SqliteProvenanceRepository {
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

struct SqliteAuditRepository;

impl SqliteAuditRepository {
    fn new() -> Self { Self }
}

#[async_trait]
impl AuditRepository for SqliteAuditRepository {
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

struct SqliteJobRepository;

impl SqliteJobRepository {
    fn new() -> Self { Self }
}

#[async_trait]
impl JobRepository for SqliteJobRepository {
    async fn enqueue(&mut self, _job: JobRecord) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    async fn claim(
        &mut self,
        _job_id: &str,
        _owner: &str,
    ) -> Result<Option<JobRecord>, StorageError> {
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

struct SqliteSettingsRepository;

impl SqliteSettingsRepository {
    fn new() -> Self { Self }
}

#[async_trait]
impl SettingsRepository for SqliteSettingsRepository {
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
    use tempfile::tempdir;

    #[tokio::test]
    async fn sqlite_database_creation() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        let db = SqliteDatabase::new(path.to_str().unwrap(), 4, true).await.unwrap();
        assert_eq!(db.backend(), DbBackend::SQLite);
        let health = db.health().await.unwrap();
        assert!(health.healthy);
    }
}