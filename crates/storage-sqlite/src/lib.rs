//! SQLite storage implementation for Q-ai.
//!
//! Implements `Database`, `ReadTx`, `UnitOfWork`, and the repository traits
//! from the `storage` crate using `sqlx` with SQLite (D0.6 / T17–T20).
//!
//! # Architecture
//!
//! - **Write pool**: single connection, serialized transactions, `synchronous=FULL`.
//! - **Read pool**: N connections, `query_only=ON`, used for read operations.
//! - **Pragmas**: WAL, foreign keys ON, busy timeout 5000 ms.
//!
//! All five repositories share one `Transaction` through an
//! `Arc<tokio::sync::Mutex<..>>`, so a `UnitOfWork` commit atomically persists
//! every repo's writes (the foundation of the outbox invariant, D0.18).

pub mod migrate;
pub(crate) mod quran;

use std::sync::Arc;

use async_trait::async_trait;
use quran::SqliteQuranRepository;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{Pool, Row, Sqlite, Transaction};
use storage::{
    Database, DbBackend, DbHealth, ReadTx, UnitOfWork,
    error::StorageError,
    quran::QuranRepository,
    repository::{
        ApprovalRow, AuditEvent, AuditRepository, ChainVerificationResult, GenerationRow, JobRecord,
        JobRepository, NewOutboxEvent, OutboxEventRow, OutboxRepository, ProvenanceRecord,
        ProvenanceRepository, ReviewRecord, SettingRow, SettingsRepository, SourceRepository,
        SourceRow, SourceVersionRow, StateTransitionRow, TombstoneRow,
    },
};
use tokio::sync::Mutex;

/// The concrete SQLite transaction type used across all repositories.
pub(crate) type SqlTx = Transaction<'static, Sqlite>;
/// Shared handle to the write transaction.
pub(crate) type SharedTx = Arc<Mutex<SqlTx>>;

fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

// ─── Database ───────────────────────────────────────────────────────────

/// SQLite database implementation with dual pools.
pub struct SqliteDatabase {
    write_pool: Pool<Sqlite>,
    read_pool: Pool<Sqlite>,
    schema_version: u32,
}

impl SqliteDatabase {
    /// Open (or create) a SQLite database with the given pool configuration.
    pub async fn new(
        path: &str,
        max_connections: u32,
        _read_only_pool: bool,
    ) -> Result<Self, StorageError> {
        // Ensure the parent directory exists so a fresh `--data-dir` works.
        if let Some(parent) = std::path::Path::new(path).parent()
            && !parent.as_os_str().is_empty()
        {
            let _ = std::fs::create_dir_all(parent);
        }
        let write_options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Full)
            .foreign_keys(true)
            .busy_timeout(std::time::Duration::from_millis(5000));

        let write_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(write_options)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;

        let read_options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(false)
            .foreign_keys(true)
            .busy_timeout(std::time::Duration::from_millis(5000))
            .read_only(true);

        let read_pool = SqlitePoolOptions::new()
            .max_connections(max_connections.max(1))
            .connect_with(read_options)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;

        let schema_version = Self::get_schema_version(&write_pool).await.unwrap_or(0);

        Ok(Self { write_pool, read_pool, schema_version })
    }

    /// Open an existing database **read-only** (used by `qai doctor`, AC-P0-14).
    ///
    /// Fails if the file does not exist; never creates or writes.
    pub async fn open_read_only(path: &str) -> Result<Self, StorageError> {
        let read_options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(false)
            .foreign_keys(true)
            .busy_timeout(std::time::Duration::from_millis(5000))
            .read_only(true);

        let read_pool = SqlitePoolOptions::new()
            .max_connections(4)
            .connect_with(read_options)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;

        let schema_version = Self::get_schema_version(&read_pool).await.unwrap_or(0);

        Ok(Self { write_pool: read_pool.clone(), read_pool, schema_version })
    }

    async fn get_schema_version(pool: &Pool<Sqlite>) -> Result<u32, sqlx::Error> {
        let row =
            sqlx::query("SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1")
                .fetch_optional(pool)
                .await?;
        Ok(row.map(|r| r.get::<i64, _>("version") as u32).unwrap_or(0))
    }

    /// Run a read-only scalar probe against the database.
    pub async fn probe_scalar(&self, sql: &str) -> Result<Option<String>, StorageError> {
        let row = sqlx::query(sql)
            .fetch_optional(&self.read_pool)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(row.map(|r| r.get::<String, _>(0)))
    }

    /// Count rows matching a read-only query returning a single integer.
    pub async fn count(&self, sql: &str) -> Result<i64, StorageError> {
        let row = sqlx::query(sql)
            .fetch_optional(&self.read_pool)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(row.map(|r| r.get::<i64, _>(0)).unwrap_or(0))
    }

    /// The underlying read-only pool (for doctor checks).
    pub fn read_pool(&self) -> &Pool<Sqlite> {
        &self.read_pool
    }
}

#[async_trait]
impl Database for SqliteDatabase {
    async fn read(&self) -> Result<Box<dyn ReadTx>, StorageError> {
        Ok(Box::new(SqliteReadTx::new(self.read_pool.clone(), self.schema_version)))
    }

    async fn write(&self) -> Result<Box<dyn UnitOfWork>, StorageError> {
        Ok(Box::new(SqliteUnitOfWork::new(self.write_pool.clone()).await?))
    }

    async fn health(&self) -> Result<DbHealth, StorageError> {
        let row = sqlx::query("SELECT 1 as ok")
            .fetch_one(&self.write_pool)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;
        let healthy = row.get::<i64, _>("ok") == 1;
        if healthy {
            Ok(DbHealth::ok(DbBackend::SQLite, self.schema_version))
        } else {
            Ok(DbHealth::fail(
                DbBackend::SQLite,
                self.schema_version,
                "probe returned unexpected value".into(),
            ))
        }
    }

    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn backend(&self) -> DbBackend {
        DbBackend::SQLite
    }
}

// ─── ReadTx ─────────────────────────────────────────────────────────────

/// Read-only transaction handle for SQLite.
pub struct SqliteReadTx {
    pool: Pool<Sqlite>,
    schema_version: u32,
}

impl SqliteReadTx {
    fn new(pool: Pool<Sqlite>, schema_version: u32) -> Self {
        Self { pool, schema_version }
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

// ─── UnitOfWork ─────────────────────────────────────────────────────────

/// Write transaction (unit of work) for SQLite.
pub struct SqliteUnitOfWork {
    tx: SharedTx,
    sources: SqliteSourceRepository,
    provenance: SqliteProvenanceRepository,
    audit: SqliteAuditRepository,
    jobs: SqliteJobRepository,
    settings: SqliteSettingsRepository,
    outbox: SqliteOutboxRepository,
    quran: SqliteQuranRepository,
}

impl SqliteUnitOfWork {
    async fn new(pool: Pool<Sqlite>) -> Result<Self, StorageError> {
        let tx = pool.begin().await.map_err(|_| StorageError::StorageUnavailable)?;
        let shared: SharedTx = Arc::new(Mutex::new(tx));
        Ok(Self {
            tx: shared.clone(),
            sources: SqliteSourceRepository::new(shared.clone()),
            provenance: SqliteProvenanceRepository::new(shared.clone()),
            audit: SqliteAuditRepository::new(shared.clone()),
            jobs: SqliteJobRepository::new(shared.clone()),
            settings: SqliteSettingsRepository::new(shared.clone()),
            outbox: SqliteOutboxRepository::new(shared.clone()),
            quran: SqliteQuranRepository::new(shared),
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

    fn outbox(&mut self) -> &mut dyn OutboxRepository {
        &mut self.outbox
    }

    fn quran(&mut self) -> &mut dyn QuranRepository {
        &mut self.quran
    }

    async fn commit(self: Box<Self>) -> Result<(), StorageError> {
        let Self { tx, sources, provenance, audit, jobs, settings, outbox, quran } = *self;
        // Drop the repository Arc clones so `tx` is the sole owner.
        drop((sources, provenance, audit, jobs, settings, outbox, quran));
        let mutex = Arc::try_unwrap(tx).map_err(|_| StorageError::StorageBusy)?;
        let txn = mutex.into_inner();
        txn.commit().await.map_err(|_| StorageError::StorageUnavailable)
    }

    async fn rollback(self: Box<Self>) -> Result<(), StorageError> {
        let Self { tx, sources, provenance, audit, jobs, settings, outbox, quran } = *self;
        drop((sources, provenance, audit, jobs, settings, outbox, quran));
        let mutex = Arc::try_unwrap(tx).map_err(|_| StorageError::StorageBusy)?;
        let txn = mutex.into_inner();
        txn.rollback().await.map_err(|_| StorageError::StorageUnavailable)
    }
}

// ─── Source repository ──────────────────────────────────────────────────

pub(crate) struct SqliteSourceRepository {
    tx: SharedTx,
}

impl SqliteSourceRepository {
    pub(crate) fn new(tx: SharedTx) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl SourceRepository for SqliteSourceRepository {
    async fn get(&self, id: &str) -> Result<Option<SourceRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query(
            "SELECT id, title, content_type, language, created_at FROM sources WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(row.map(|r| SourceRow {
            id: r.get("id"),
            title: r.get("title"),
            content_type: r.get("content_type"),
            language: r.get("language"),
            created_at: r.get("created_at"),
        }))
    }

    async fn insert_source(&mut self, source: SourceRow) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO sources
                (id, title, alternate_titles, content_type, authors, identifiers, language,
                 created_at, updated_at)
             VALUES (?, ?, '[]', ?, '[]', '{}', ?, ?, ?)",
        )
        .bind(&source.id)
        .bind(&source.title)
        .bind(&source.content_type)
        .bind(&source.language)
        .bind(&source.created_at)
        .bind(&source.created_at)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn insert_version(&mut self, version: SourceVersionRow) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO source_versions
                (id, source_id, version, schema_version, state, trust_level, license_status,
                 license_json, manifest_blob_id, content_hash, source_urls, created_at)
             VALUES (?, ?, ?, 1, ?, ?, ?, '{}', ?, ?, '[]', ?)",
        )
        .bind(&version.id)
        .bind(&version.source_id)
        .bind(&version.version)
        .bind(&version.state)
        .bind(&version.trust_level)
        .bind(&version.license_status)
        .bind(&version.manifest_blob_id)
        .bind(&version.content_hash)
        .bind(now_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn transition_state(
        &mut self,
        source_version_id: &str,
        from: &str,
        to: &str,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query("UPDATE source_versions SET state = ? WHERE id = ? AND state = ?")
            .bind(to)
            .bind(source_version_id)
            .bind(from)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound {
                urn: format!("source_version:{source_version_id}"),
            });
        }
        Ok(())
    }

    async fn list_versions(&self, source_id: &str) -> Result<Vec<SourceVersionRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT id, source_id, version, state, trust_level, license_status, content_hash,
                    manifest_blob_id
             FROM source_versions WHERE source_id = ? ORDER BY version ASC",
        )
        .bind(source_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(rows
            .into_iter()
            .map(|r| SourceVersionRow {
                id: r.get("id"),
                source_id: r.get("source_id"),
                version: r.get("version"),
                state: r.get("state"),
                trust_level: r.get("trust_level"),
                license_status: r.get("license_status"),
                content_hash: r.get("content_hash"),
                manifest_blob_id: r.get("manifest_blob_id"),
            })
            .collect())
    }

    async fn record_transition(
        &mut self,
        transition: StateTransitionRow,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO source_state_transitions
                (id, source_version_id, from_state, to_state, actor_id, reason, occurred_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&transition.id)
        .bind(&transition.source_version_id)
        .bind(&transition.from_state)
        .bind(&transition.to_state)
        .bind(&transition.actor_id)
        .bind(&transition.reason)
        .bind(&transition.occurred_at)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn insert_approval(&mut self, approval: ApprovalRow) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO approvals
                (id, subject_urn, kind, requested_by, decided_by, decision,
                 request_payload, decision_note, requested_at, decided_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&approval.id)
        .bind(&approval.subject_urn)
        .bind(&approval.kind)
        .bind(&approval.requested_by)
        .bind(&approval.decided_by)
        .bind(&approval.decision)
        .bind(&approval.request_payload)
        .bind(&approval.decision_note)
        .bind(&approval.requested_at)
        .bind(&approval.decided_at)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn get_approval(&self, id: &str) -> Result<Option<ApprovalRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query("SELECT * FROM approvals WHERE id = ?")
            .bind(id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(row.map(|r| ApprovalRow {
            id: r.get("id"),
            subject_urn: r.get("subject_urn"),
            kind: r.get("kind"),
            requested_by: r.get("requested_by"),
            decided_by: r.get("decided_by"),
            decision: r.get("decision"),
            request_payload: r.get("request_payload"),
            decision_note: r.get("decision_note"),
            requested_at: r.get("requested_at"),
            decided_at: r.get("decided_at"),
        }))
    }
}

// ─── Provenance repository ──────────────────────────────────────────────

pub(crate) struct SqliteProvenanceRepository {
    tx: SharedTx,
}

impl SqliteProvenanceRepository {
    pub(crate) fn new(tx: SharedTx) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl ProvenanceRepository for SqliteProvenanceRepository {
    async fn insert(&mut self, record: ProvenanceRecord) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO provenance_records
                (id, layer, subject_urn, attribution_kind, attribution_json, source_version_id,
                 trust_level, verification_status, confidence, versions_json, created_at, created_by)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&record.id)
        .bind(&record.layer)
        .bind(&record.subject_urn)
        .bind(&record.attribution_kind)
        .bind(&record.attribution_json)
        .bind(&record.source_version_id)
        .bind(&record.trust_level)
        .bind(&record.verification_status)
        .bind(record.confidence)
        .bind(&record.versions_json)
        .bind(now_rfc3339())
        .bind(&record.created_by)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn get(&self, id: &str) -> Result<Option<ProvenanceRecord>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query(
            "SELECT id, layer, subject_urn, attribution_kind, attribution_json, source_version_id,
                    trust_level, verification_status, confidence, versions_json, created_by
             FROM provenance_records WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(row.map(map_provenance_row))
    }

    async fn list_by_subject(
        &self,
        subject_urn: &str,
    ) -> Result<Vec<ProvenanceRecord>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT id, layer, subject_urn, attribution_kind, attribution_json, source_version_id,
                    trust_level, verification_status, confidence, versions_json, created_by
             FROM provenance_records WHERE subject_urn = ? ORDER BY created_at ASC",
        )
        .bind(subject_urn)
        .fetch_all(&mut **tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(rows.into_iter().map(map_provenance_row).collect())
    }

    async fn record_review(&mut self, review: ReviewRecord) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO review_queue
                (id, provenance_id, queue, evidence_json, state, decided_by, decided_at,
                 decision_note, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&review.id)
        .bind(&review.provenance_id)
        .bind(&review.queue)
        .bind(&review.evidence_json)
        .bind(&review.state)
        .bind(&review.decided_by)
        .bind(&review.decided_at)
        .bind(&review.decision_note)
        .bind(&review.created_at)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }
}

fn map_provenance_row(r: sqlx::sqlite::SqliteRow) -> ProvenanceRecord {
    ProvenanceRecord {
        id: r.get("id"),
        layer: r.get("layer"),
        subject_urn: r.get("subject_urn"),
        attribution_kind: r.get("attribution_kind"),
        attribution_json: r.get("attribution_json"),
        source_version_id: r.get("source_version_id"),
        trust_level: r.get("trust_level"),
        verification_status: r.get("verification_status"),
        confidence: r.get("confidence"),
        versions_json: r.get("versions_json"),
        created_by: r.get("created_by"),
    }
}

// ─── Audit repository ───────────────────────────────────────────────────

pub(crate) struct SqliteAuditRepository {
    tx: SharedTx,
}

impl SqliteAuditRepository {
    pub(crate) fn new(tx: SharedTx) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl AuditRepository for SqliteAuditRepository {
    async fn append(&mut self, event: AuditEvent) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO audit_events
                (id, sequence, occurred_at, actor_kind, actor_id, action, subject_urn, outcome,
                 reason, before_json, after_json, request_id, prev_chain_hash, chain_hash)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&event.id)
        .bind(event.sequence as i64)
        .bind(&event.occurred_at)
        .bind(&event.actor_kind)
        .bind(&event.actor_id)
        .bind(&event.action)
        .bind(&event.subject_urn)
        .bind(&event.outcome)
        .bind(&event.reason)
        .bind(&event.before_json)
        .bind(&event.after_json)
        .bind(&event.request_id)
        .bind(&event.prev_chain_hash)
        .bind(&event.chain_hash)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn list_by_subject(&self, subject_urn: &str) -> Result<Vec<AuditEvent>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT id, sequence, occurred_at, actor_kind, actor_id, action, subject_urn, outcome,
                    reason, before_json, after_json, request_id, prev_chain_hash, chain_hash
             FROM audit_events WHERE subject_urn = ? ORDER BY sequence ASC",
        )
        .bind(subject_urn)
        .fetch_all(&mut **tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(rows.into_iter().map(map_audit_row).collect())
    }

    async fn list_by_sequence(
        &self,
        from: u64,
        to: Option<u64>,
    ) -> Result<Vec<AuditEvent>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = match to {
            Some(to) => sqlx::query(
                "SELECT id, sequence, occurred_at, actor_kind, actor_id, action, subject_urn,
                        outcome, reason, before_json, after_json, request_id, prev_chain_hash,
                        chain_hash
                 FROM audit_events WHERE sequence BETWEEN ? AND ? ORDER BY sequence ASC",
            )
            .bind(from as i64)
            .bind(to as i64)
            .fetch_all(&mut **tx)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?,
            None => sqlx::query(
                "SELECT id, sequence, occurred_at, actor_kind, actor_id, action, subject_urn,
                        outcome, reason, before_json, after_json, request_id, prev_chain_hash,
                        chain_hash
                 FROM audit_events WHERE sequence >= ? ORDER BY sequence ASC",
            )
            .bind(from as i64)
            .fetch_all(&mut **tx)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?,
        };
        Ok(rows.into_iter().map(map_audit_row).collect())
    }

    async fn latest_sequence(&self) -> Result<u64, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query("SELECT MAX(sequence) AS max_seq FROM audit_events")
            .fetch_one(&mut **tx)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(row.get::<Option<i64>, _>("max_seq").unwrap_or(0).max(0) as u64)
    }

    async fn verify_chain(&self) -> Result<ChainVerificationResult, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT id, sequence, occurred_at, actor_kind, actor_id, action, subject_urn, outcome,
                    reason, before_json, after_json, request_id, prev_chain_hash, chain_hash
             FROM audit_events ORDER BY sequence ASC",
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;

        let events: Vec<AuditEvent> = rows.into_iter().map(map_audit_row).collect();

        // Structural verification: sequences are contiguous from 1 and each
        // event's prev_chain_hash equals the previous event's chain_hash.
        // (Full hash recomputation lives in `audit::AuditVerifier`, which owns
        // the canonical algorithm.)
        let mut gaps = Vec::new();
        let mut valid = true;
        let mut expected_seq: u64 = 1;
        let mut prev_hash = "00".repeat(32);
        for ev in &events {
            if ev.sequence != expected_seq {
                gaps.push(ev.sequence);
                valid = false;
            }
            if ev.prev_chain_hash != prev_hash {
                valid = false;
            }
            prev_hash = ev.chain_hash.clone();
            expected_seq = ev.sequence + 1;
        }

        Ok(ChainVerificationResult {
            valid,
            expected_next_sequence: expected_seq,
            expected_next_hash: prev_hash,
            gaps,
        })
    }
}

fn map_audit_row(r: sqlx::sqlite::SqliteRow) -> AuditEvent {
    AuditEvent {
        id: r.get("id"),
        sequence: r.get::<i64, _>("sequence") as u64,
        occurred_at: r.get("occurred_at"),
        actor_kind: r.get("actor_kind"),
        actor_id: r.get("actor_id"),
        action: r.get("action"),
        subject_urn: r.get("subject_urn"),
        outcome: r.get("outcome"),
        reason: r.get("reason"),
        before_json: r.get("before_json"),
        after_json: r.get("after_json"),
        request_id: r.get("request_id"),
        prev_chain_hash: r.get("prev_chain_hash"),
        chain_hash: r.get("chain_hash"),
    }
}

// ─── Job repository ─────────────────────────────────────────────────────

pub(crate) struct SqliteJobRepository {
    tx: SharedTx,
}

impl SqliteJobRepository {
    pub(crate) fn new(tx: SharedTx) -> Self {
        Self { tx }
    }
}

fn map_job_row(r: sqlx::sqlite::SqliteRow) -> JobRecord {
    JobRecord {
        id: r.get("id"),
        kind: r.get("kind"),
        payload_json: r.get("payload_json"),
        idempotency_key: r.get("idempotency_key"),
        state: r.get("state"),
        priority: r.get::<i64, _>("priority") as i32,
        attempts: r.get::<i64, _>("attempts") as u32,
        max_attempts: r.get::<i64, _>("max_attempts") as u32,
        available_at: r.get("available_at"),
        lease_owner: r.get("lease_owner"),
        lease_expires_at: r.get("lease_expires_at"),
        checkpoint_json: r.get("checkpoint_json"),
        cancel_requested: r.get::<i64, _>("cancel_requested") != 0,
        created_by: r.get::<Option<String>, _>("created_by").unwrap_or_default(),
    }
}

#[async_trait]
impl JobRepository for SqliteJobRepository {
    async fn enqueue(&mut self, job: JobRecord) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO jobs
                (id, kind, payload_json, idempotency_key, state, priority, attempts, max_attempts,
                 available_at, lease_owner, lease_expires_at, checkpoint_json, cancel_requested,
                 created_by, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&job.id)
        .bind(&job.kind)
        .bind(&job.payload_json)
        .bind(&job.idempotency_key)
        .bind(&job.state)
        .bind(job.priority as i64)
        .bind(job.attempts as i64)
        .bind(job.max_attempts as i64)
        .bind(&job.available_at)
        .bind(&job.lease_owner)
        .bind(&job.lease_expires_at)
        .bind(&job.checkpoint_json)
        .bind(job.cancel_requested as i64)
        .bind(if job.created_by.is_empty() { None } else { Some(job.created_by.clone()) })
        .bind(now_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn claim(
        &mut self,
        job_id: &str,
        owner: &str,
    ) -> Result<Option<JobRecord>, StorageError> {
        let mut tx = self.tx.lock().await;
        let lease_expires = (time::OffsetDateTime::now_utc() + time::Duration::minutes(5))
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
        let result = sqlx::query(
            "UPDATE jobs
             SET state = 'Running', lease_owner = ?, lease_expires_at = ?,
                 attempts = attempts + 1, started_at = COALESCE(started_at, ?)
             WHERE id = ? AND state IN ('Queued', 'Interrupted', 'Checkpointed')",
        )
        .bind(owner)
        .bind(&lease_expires)
        .bind(now_rfc3339())
        .bind(job_id)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        let row = sqlx::query(
            "SELECT id, kind, payload_json, idempotency_key, state, priority, attempts,
                    max_attempts, available_at, lease_owner, lease_expires_at, checkpoint_json,
                    cancel_requested, created_by
             FROM jobs WHERE id = ?",
        )
        .bind(job_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(row.map(map_job_row))
    }

    async fn claim_next(
        &mut self,
        owner: &str,
        lease_seconds: u64,
    ) -> Result<Option<JobRecord>, StorageError> {
        let mut tx = self.tx.lock().await;
        let now = now_rfc3339();
        let lease_expires = (time::OffsetDateTime::now_utc()
            + time::Duration::seconds(lease_seconds as i64))
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();
        // Select the highest-priority due job, then claim it atomically.
        let candidate = sqlx::query(
            "SELECT id FROM jobs
             WHERE state IN ('Queued','Interrupted','Checkpointed') AND available_at <= ?
             ORDER BY priority DESC, available_at ASC LIMIT 1",
        )
        .bind(&now)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        let Some(row) = candidate else {
            return Ok(None);
        };
        let id: String = row.get("id");
        let result = sqlx::query(
            "UPDATE jobs
             SET state = 'Running', lease_owner = ?, lease_expires_at = ?,
                 attempts = attempts + 1, started_at = COALESCE(started_at, ?)
             WHERE id = ? AND state IN ('Queued','Interrupted','Checkpointed')",
        )
        .bind(owner)
        .bind(&lease_expires)
        .bind(now_rfc3339())
        .bind(&id)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        let row = sqlx::query(
            "SELECT id, kind, payload_json, idempotency_key, state, priority, attempts,
                    max_attempts, available_at, lease_owner, lease_expires_at, checkpoint_json,
                    cancel_requested, created_by
             FROM jobs WHERE id = ?",
        )
        .bind(&id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(row.map(map_job_row))
    }

    async fn reschedule(
        &mut self,
        job_id: &str,
        delay_seconds: u64,
        reason: Option<String>,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        let available_at = (time::OffsetDateTime::now_utc()
            + time::Duration::seconds(delay_seconds as i64))
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();
        let affected = sqlx::query(
            "UPDATE jobs
             SET state = 'Queued', available_at = ?, lease_owner = NULL, lease_expires_at = NULL,
                 error_json = ?
             WHERE id = ?",
        )
        .bind(&available_at)
        .bind(reason)
        .bind(job_id)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        if affected.rows_affected() == 0 {
            return Err(StorageError::NotFound { urn: format!("job:{job_id}") });
        }
        Ok(())
    }

    async fn get(&self, job_id: &str) -> Result<Option<JobRecord>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query(
            "SELECT id, kind, payload_json, idempotency_key, state, priority, attempts,
                    max_attempts, available_at, lease_owner, lease_expires_at, checkpoint_json,
                    cancel_requested, created_by
             FROM jobs WHERE id = ?",
        )
        .bind(job_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(row.map(map_job_row))
    }

    async fn finish(
        &mut self,
        job_id: &str,
        state: &str,
        result: Option<String>,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        let affected = sqlx::query(
            "UPDATE jobs SET state = ?, finished_at = ?, error_json = ?
             WHERE id = ?",
        )
        .bind(state)
        .bind(now_rfc3339())
        .bind(result)
        .bind(job_id)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        if affected.rows_affected() == 0 {
            return Err(StorageError::NotFound { urn: format!("job:{job_id}") });
        }
        Ok(())
    }

    async fn cancel(&mut self, job_id: &str) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        let affected = sqlx::query("UPDATE jobs SET cancel_requested = 1 WHERE id = ?")
            .bind(job_id)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        if affected.rows_affected() == 0 {
            return Err(StorageError::NotFound { urn: format!("job:{job_id}") });
        }
        Ok(())
    }

    async fn checkpoint(
        &mut self,
        job_id: &str,
        progress: Option<String>,
        checkpoint: Option<String>,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query("UPDATE jobs SET progress_json = ?, checkpoint_json = ? WHERE id = ?")
            .bind(progress)
            .bind(checkpoint)
            .bind(job_id)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn heartbeat(
        &mut self,
        job_id: &str,
        owner: &str,
        lease_seconds: u64,
    ) -> Result<bool, StorageError> {
        let mut tx = self.tx.lock().await;
        let lease_expires = (time::OffsetDateTime::now_utc()
            + time::Duration::seconds(lease_seconds as i64))
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();
        let result = sqlx::query(
            "UPDATE jobs SET lease_expires_at = ?
             WHERE id = ? AND lease_owner = ? AND state = 'Running'",
        )
        .bind(&lease_expires)
        .bind(job_id)
        .bind(owner)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn count_by_state(&self, state: &str) -> Result<i64, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query("SELECT COUNT(*) AS n FROM jobs WHERE state = ?")
            .bind(state)
            .fetch_one(&mut **tx)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(row.get::<i64, _>("n"))
    }

    async fn reap_expired_leases(&mut self) -> Result<Vec<String>, StorageError> {
        let mut tx = self.tx.lock().await;
        let now = now_rfc3339();
        let rows = sqlx::query(
            "SELECT id FROM jobs
             WHERE state = 'Running' AND lease_expires_at IS NOT NULL AND lease_expires_at < ?",
        )
        .bind(&now)
        .fetch_all(&mut **tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        let ids: Vec<String> = rows.into_iter().map(|r| r.get("id")).collect();
        if !ids.is_empty() {
            sqlx::query(
                "UPDATE jobs SET state = 'Interrupted', lease_owner = NULL, lease_expires_at = NULL
                 WHERE state = 'Running' AND lease_expires_at IS NOT NULL AND lease_expires_at < ?",
            )
            .bind(&now)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        }
        Ok(ids)
    }
}

// ─── Settings repository ────────────────────────────────────────────────

pub(crate) struct SqliteSettingsRepository {
    tx: SharedTx,
}

impl SqliteSettingsRepository {
    pub(crate) fn new(tx: SharedTx) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl SettingsRepository for SqliteSettingsRepository {
    async fn get(&self, key: &str) -> Result<Option<SettingRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query(
            "SELECT key, value_json, origin, updated_at, updated_by FROM settings WHERE key = ?",
        )
        .bind(key)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(row.map(|r| SettingRow {
            key: r.get("key"),
            value_json: r.get("value_json"),
            origin: r.get("origin"),
            updated_at: r.get("updated_at"),
            updated_by: r.get("updated_by"),
        }))
    }

    async fn set(
        &mut self,
        key: &str,
        value_json: &str,
        origin: &str,
        updated_by: Option<&str>,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO settings (key, value_json, origin, updated_at, updated_by)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(key) DO UPDATE SET
                value_json = excluded.value_json,
                origin = excluded.origin,
                updated_at = excluded.updated_at,
                updated_by = excluded.updated_by",
        )
        .bind(key)
        .bind(value_json)
        .bind(origin)
        .bind(now_rfc3339())
        .bind(updated_by)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<SettingRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT key, value_json, origin, updated_at, updated_by FROM settings ORDER BY key ASC",
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(rows
            .into_iter()
            .map(|r| SettingRow {
                key: r.get("key"),
                value_json: r.get("value_json"),
                origin: r.get("origin"),
                updated_at: r.get("updated_at"),
                updated_by: r.get("updated_by"),
            })
            .collect())
    }
}

// ─── Outbox / generations / tombstones ──────────────────────────────────

pub(crate) struct SqliteOutboxRepository {
    tx: SharedTx,
}

impl SqliteOutboxRepository {
    pub(crate) fn new(tx: SharedTx) -> Self {
        Self { tx }
    }
}

fn map_generation_row(r: sqlx::sqlite::SqliteRow) -> GenerationRow {
    GenerationRow {
        id: r.get("id"),
        scope: r.get("scope"),
        number: r.get::<i64, _>("number") as u64,
        reason: r.get("reason"),
        created_at: r.get("created_at"),
    }
}

fn map_outbox_row(r: sqlx::sqlite::SqliteRow) -> OutboxEventRow {
    OutboxEventRow {
        id: r.get("id"),
        scope: r.get("scope"),
        target_generation: r.get("target_generation"),
        operation: r.get("operation"),
        subject_urn: r.get("subject_urn"),
        idempotency_key: r.get("idempotency_key"),
        payload_json: r.get("payload_json"),
        state: r.get("state"),
        lease_owner: r.get("lease_owner"),
        lease_expires_at: r.get("lease_expires_at"),
        attempts: r.get::<i64, _>("attempts") as u32,
        created_at: r.get("created_at"),
        dispatched_at: r.get("dispatched_at"),
    }
}

fn map_tombstone_row(r: sqlx::sqlite::SqliteRow) -> TombstoneRow {
    TombstoneRow {
        id: r.get("id"),
        subject_urn: r.get("subject_urn"),
        reason: r.get("reason"),
        effective_at: r.get("effective_at"),
        created_by: r.get("created_by"),
        propagation_state: r.get("propagation_state"),
    }
}

const OUTBOX_COLUMNS: &str = "id, scope, target_generation, operation, subject_urn, \
     idempotency_key, payload_json, state, lease_owner, lease_expires_at, attempts, \
     created_at, dispatched_at";

#[async_trait]
impl OutboxRepository for SqliteOutboxRepository {
    async fn allocate_generation(
        &mut self,
        scope: &str,
        reason: &str,
    ) -> Result<GenerationRow, StorageError> {
        let mut tx = self.tx.lock().await;
        // The single write connection serializes this read-then-insert, so the
        // number is monotonic. (PostgreSQL equivalent: `SELECT ... FOR UPDATE`.)
        let current = sqlx::query(
            "SELECT number FROM corpus_generations WHERE scope = ? ORDER BY number DESC LIMIT 1",
        )
        .bind(scope)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        let next: i64 = current.map(|r| r.get::<i64, _>("number")).unwrap_or(0) + 1;

        let id = domain::CorpusGenerationId::new().to_string();
        let created_at = now_rfc3339();
        sqlx::query(
            "INSERT INTO corpus_generations (id, scope, number, reason, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(scope)
        .bind(next)
        .bind(reason)
        .bind(&created_at)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(GenerationRow {
            id,
            scope: scope.to_string(),
            number: next as u64,
            reason: reason.to_string(),
            created_at,
        })
    }

    async fn current_generation(&self, scope: &str) -> Result<Option<GenerationRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query(
            "SELECT id, scope, number, reason, created_at FROM corpus_generations
             WHERE scope = ? ORDER BY number DESC LIMIT 1",
        )
        .bind(scope)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(row.map(map_generation_row))
    }

    async fn enqueue(&mut self, event: NewOutboxEvent) -> Result<String, StorageError> {
        let mut tx = self.tx.lock().await;
        let id = domain::OutboxEventId::new().to_string();
        sqlx::query(
            "INSERT INTO outbox_events
                (id, scope, target_generation, operation, subject_urn, idempotency_key,
                 payload_json, state, attempts, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, 'Pending', 0, ?)",
        )
        .bind(&id)
        .bind(&event.scope)
        .bind(&event.target_generation)
        .bind(&event.operation)
        .bind(&event.subject_urn)
        .bind(&event.idempotency_key)
        .bind(&event.payload_json)
        .bind(now_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(id)
    }

    async fn claim_pending(
        &mut self,
        owner: &str,
        limit: u32,
    ) -> Result<Vec<OutboxEventRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(&format!(
            "SELECT {OUTBOX_COLUMNS} FROM outbox_events WHERE state = 'Pending' \
             ORDER BY created_at ASC LIMIT ?"
        ))
        .bind(limit as i64)
        .fetch_all(&mut **tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        let claimed: Vec<OutboxEventRow> = rows.into_iter().map(map_outbox_row).collect();
        if claimed.is_empty() {
            return Ok(claimed);
        }
        let lease_expires = (time::OffsetDateTime::now_utc() + time::Duration::minutes(5))
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
        for event in &claimed {
            sqlx::query(
                "UPDATE outbox_events
                 SET state = 'Claimed', lease_owner = ?, lease_expires_at = ?,
                     attempts = attempts + 1
                 WHERE id = ? AND state = 'Pending'",
            )
            .bind(owner)
            .bind(&lease_expires)
            .bind(&event.id)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        }
        Ok(claimed
            .into_iter()
            .map(|mut e| {
                e.state = "Claimed".to_string();
                e.lease_owner = Some(owner.to_string());
                e.lease_expires_at = Some(lease_expires.clone());
                e.attempts += 1;
                e
            })
            .collect())
    }

    async fn mark_dispatched(&mut self, id: &str) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "UPDATE outbox_events
             SET state = 'Dispatched', dispatched_at = ?, lease_owner = NULL, lease_expires_at = NULL
             WHERE id = ?",
        )
        .bind(now_rfc3339())
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn mark_failed(&mut self, id: &str, reason: &str) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "UPDATE outbox_events SET state = 'Failed', payload_json = json_set(payload_json, '$.failure_reason', ?)
             WHERE id = ?",
        )
        .bind(reason)
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn list_by_state(&self, state: &str) -> Result<Vec<OutboxEventRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(&format!(
            "SELECT {OUTBOX_COLUMNS} FROM outbox_events WHERE state = ? ORDER BY created_at ASC"
        ))
        .bind(state)
        .fetch_all(&mut **tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(rows.into_iter().map(map_outbox_row).collect())
    }

    async fn insert_tombstone(&mut self, tombstone: TombstoneRow) -> Result<(), StorageError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "INSERT INTO tombstones
                (id, subject_urn, reason, effective_at, created_by, propagation_state)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&tombstone.id)
        .bind(&tombstone.subject_urn)
        .bind(&tombstone.reason)
        .bind(&tombstone.effective_at)
        .bind(&tombstone.created_by)
        .bind(&tombstone.propagation_state)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn list_pending_tombstones(&self) -> Result<Vec<TombstoneRow>, StorageError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query(
            "SELECT id, subject_urn, reason, effective_at, created_by, propagation_state
             FROM tombstones WHERE propagation_state = 'Pending' ORDER BY effective_at ASC",
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(rows.into_iter().map(map_tombstone_row).collect())
    }
}

// ─── Error mapping ──────────────────────────────────────────────────────

/// Map a `sqlx::Error` to the fixed `StorageError` taxonomy (D0.6).
pub(crate) fn map_sqlx_error(err: sqlx::Error) -> StorageError {
    match &err {
        sqlx::Error::Database(db_err) => {
            let code = db_err.code().unwrap_or_default();
            let msg = db_err.message().to_string();
            if code == "2067" || code == "1555" || msg.contains("UNIQUE") {
                StorageError::Conflict
            } else if msg.contains("QAI-QUR-") {
                StorageError::ConstraintViolation { message: msg }
            } else if msg.contains("QAI-PROV") || msg.contains("immutable") {
                StorageError::ImmutableSourceVersion
            } else if msg.contains("CHECK")
                || msg.contains("FOREIGN KEY")
                || msg.contains("NOT NULL")
            {
                StorageError::ConstraintViolation { message: msg }
            } else {
                StorageError::StorageUnavailable
            }
        }
        sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed => StorageError::StorageBusy,
        sqlx::Error::RowNotFound => StorageError::NotFound { urn: "row".into() },
        _ => StorageError::StorageUnavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use storage::repository::SourceVersionRow as _SVRow;
    use tempfile::tempdir;

    async fn migrated_db(dir: &Path) -> SqliteDatabase {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
        let db_path = dir.join("qai.db");
        migrate::apply_migrations(db_path.to_str().unwrap(), &repo_root).await.unwrap();
        // Seed a principal for FK targets (jobs.created_by, settings.updated_by).
        let seed = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new().filename(&db_path).foreign_keys(true),
            )
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO principals (id, kind, display_name, created_at)
             VALUES ('principal', 'local_user', 'Test', ?)",
        )
        .bind(now_rfc3339())
        .execute(&seed)
        .await
        .unwrap();
        seed.close().await;
        SqliteDatabase::new(db_path.to_str().unwrap(), 4, true).await.unwrap()
    }

    #[tokio::test]
    async fn sqlite_database_health() {
        let dir = tempdir().unwrap();
        let db = migrated_db(dir.path()).await;
        assert_eq!(db.backend(), DbBackend::SQLite);
        let health = db.health().await.unwrap();
        assert!(health.healthy);
        assert_eq!(health.schema_version, 12);
    }

    #[tokio::test]
    async fn read_only_open_never_creates() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing.db");
        let err = SqliteDatabase::open_read_only(missing.to_str().unwrap()).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn job_enqueue_and_claim_round_trip() {
        let dir = tempdir().unwrap();
        let db = migrated_db(dir.path()).await;

        let job = JobRecord {
            id: "job-1".into(),
            kind: "system.noop_test".into(),
            payload_json: "{}".into(),
            idempotency_key: Some("k1".into()),
            state: "Queued".into(),
            priority: 0,
            attempts: 0,
            max_attempts: 5,
            available_at: now_rfc3339(),
            lease_owner: None,
            lease_expires_at: None,
            checkpoint_json: None,
            cancel_requested: false,
            created_by: "principal".into(),
        };
        let mut uow = db.write().await.unwrap();
        uow.jobs().enqueue(job.clone()).await.unwrap();
        let claimed = uow.jobs().claim("job-1", "worker-1").await.unwrap();
        assert!(claimed.is_some());
        assert_eq!(claimed.unwrap().state, "Running");
        uow.commit().await.unwrap();

        // Duplicate enqueue with the same idempotency key conflicts.
        let mut uow = db.write().await.unwrap();
        let dup = uow.jobs().enqueue(job).await;
        assert!(matches!(dup, Err(StorageError::Conflict)));
        uow.rollback().await.unwrap();
    }

    #[tokio::test]
    async fn source_version_insert_and_list() {
        let dir = tempdir().unwrap();
        let db = migrated_db(dir.path()).await;
        let mut uow = db.write().await.unwrap();

        // Insert the parent source first (FK).
        uow.sources()
            .insert_source(SourceRow {
                id: "src-1".into(),
                title: "Test".into(),
                content_type: "quran_edition".into(),
                language: Some("ar".into()),
                created_at: now_rfc3339(),
            })
            .await
            .unwrap();

        uow.sources()
            .insert_version(_SVRow {
                id: "ver-1".into(),
                source_id: "src-1".into(),
                version: "1.0.0".into(),
                state: "Staged".into(),
                trust_level: "ImportedUnverified".into(),
                license_status: "OpenLicense".into(),
                content_hash: Some("sha256:".to_string() + &"aa".repeat(32)),
                manifest_blob_id: None,
            })
            .await
            .unwrap();
        let versions = uow.sources().list_versions("src-1").await.unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].state, "Staged");
        uow.commit().await.unwrap();
    }

    #[tokio::test]
    async fn settings_upsert_round_trip() {
        let dir = tempdir().unwrap();
        let db = migrated_db(dir.path()).await;
        let mut uow = db.write().await.unwrap();
        uow.settings().set("logging.level", "\"debug\"", "cli", None).await.unwrap();
        let got = uow.settings().get("logging.level").await.unwrap().unwrap();
        assert_eq!(got.value_json, "\"debug\"");
        assert_eq!(got.origin, "cli");
        uow.commit().await.unwrap();
    }

    #[tokio::test]
    async fn audit_append_and_verify() {
        let dir = tempdir().unwrap();
        let db = migrated_db(dir.path()).await;
        let mut uow = db.write().await.unwrap();
        let zero = "00".repeat(32);
        uow.audit()
            .append(AuditEvent {
                id: "a-1".into(),
                sequence: 1,
                occurred_at: now_rfc3339(),
                actor_kind: "system".into(),
                actor_id: None,
                action: "config_change".into(),
                subject_urn: "urn:qai:config".into(),
                outcome: "allowed".into(),
                reason: None,
                before_json: None,
                after_json: None,
                request_id: None,
                prev_chain_hash: zero.clone(),
                chain_hash: "aa".repeat(32),
            })
            .await
            .unwrap();
        let report = uow.audit().verify_chain().await.unwrap();
        assert!(report.valid, "chain should verify: {report:?}");
        uow.commit().await.unwrap();
    }
}
