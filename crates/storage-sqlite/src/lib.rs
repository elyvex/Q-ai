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
use sha2::{Digest, Sha256};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous, SqliteRow};
use sqlx::{Pool, Row, Sqlite, Transaction};
use std::path::Path;
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
use crate::migrate::{discover_migrations, sha256_file};

type SqlTx = Transaction<'static, Sqlite>;

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
            write_options = write_options.read_only(false);
        }

        let write_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(write_options)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;

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
            .map_err(|_| StorageError::StorageUnavailable)?;

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

    /// Apply all pending migrations and return the highest applied version.
    pub async fn apply_migrations(db_path: &str) -> Result<u32, StorageError> {
        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Full)
            .foreign_keys(true)
            .busy_timeout(std::time::Duration::from_millis(5000));

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS schema_migrations (
                version      INTEGER PRIMARY KEY,
                name         TEXT    NOT NULL,
                checksum     TEXT    NOT NULL,
                applied_at   TEXT    NOT NULL,
                applied_by   TEXT    NOT NULL,
                duration_ms  INTEGER NOT NULL
            )"#,
        )
        .execute(&pool)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;

        let applied_rows = sqlx::query("SELECT version, checksum FROM schema_migrations")
            .fetch_all(&pool)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;

        let mut applied: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
        for row in applied_rows {
            let version: i64 = row.get("version");
            let checksum: String = row.get("checksum");
            applied.insert(version as u32, checksum);
        }

        // migrations/sqlite lives at <repo>/migrations/sqlite; this crate is at
        // <repo>/crates/storage-sqlite, so two parents up from CARGO_MANIFEST_DIR.
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let migrations_dir = manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .map(|repo| repo.join("migrations").join("sqlite"))
            .unwrap_or_else(|_| manifest_dir.join("migrations").join("sqlite"));

        let discovered = discover_migrations(&migrations_dir)
            .map_err(|_| StorageError::StorageUnavailable)?;

        let mut highest_version = 0u32;

        for (version, (up_path, _down)) in discovered {
            highest_version = highest_version.max(version);

            if let Some(existing) = applied.get(&version) {
                let current = sha256_file(&up_path).map_err(|_| StorageError::StorageUnavailable)?;
                if existing != &current {
                    return Err(StorageError::MigrationChecksumMismatch { version });
                }
                continue;
            }

            let sql = std::fs::read_to_string(&up_path).map_err(|_| StorageError::StorageUnavailable)?;
            let checksum = sha256_file(&up_path).map_err(|_| StorageError::StorageUnavailable)?;
            let name = up_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();

            let start = std::time::Instant::now();
            let mut tx = pool.begin().await.map_err(|_| StorageError::StorageUnavailable)?;

            sqlx::query(&sql)
                .execute(&mut *tx)
                .await
                .map_err(|_| StorageError::StorageUnavailable)?;

            let applied_at = time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default();
            let applied_by = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
            let duration_ms = start.elapsed().as_millis() as i64;

            sqlx::query(
                "INSERT INTO schema_migrations (version, name, checksum, applied_at, applied_by, duration_ms) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(version as i64)
            .bind(&name)
            .bind(&checksum)
            .bind(&applied_at)
            .bind(&applied_by)
            .bind(duration_ms)
            .execute(&mut *tx)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;

            tx.commit().await.map_err(|_| StorageError::StorageUnavailable)?;
        }

        Ok(highest_version)
    }

    /// Backup the database file using VACUUM INTO.
    pub async fn backup(db_path: &str, dest_path: &str) -> Result<(), StorageError> {
        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(false)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Full)
            .foreign_keys(true)
            .busy_timeout(std::time::Duration::from_millis(5000));

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;

        let sql = format!("VACUUM INTO '{}'", dest_path.replace('\'', "''"));
        sqlx::query(&sql)
            .execute(&pool)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;

        Ok(())
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

/// Write transaction (unit of work) for SQLite.
///
/// The transaction is held behind a `tokio::sync::Mutex` so repository
/// traits with `&self` read methods can also access it. It implements all
/// repository traits itself.
pub struct SqliteUnitOfWork {
    tx: tokio::sync::Mutex<SqlTx>,
    schema_version: u32,
}

impl SqliteUnitOfWork {
    async fn new(pool: Pool<Sqlite>, schema_version: u32) -> Result<Self, StorageError> {
        let tx = pool.begin().await.map_err(|_| StorageError::StorageUnavailable)?;
        Ok(Self {
            tx: tokio::sync::Mutex::new(tx),
            schema_version,
        })
    }

    async fn tx(&self) -> tokio::sync::MutexGuard<'_, SqlTx> {
        self.tx.lock().await
    }
}

#[async_trait]
impl UnitOfWork for SqliteUnitOfWork {
    fn sources(&mut self) -> &mut dyn SourceRepository {
        self
    }

    fn provenance(&mut self) -> &mut dyn ProvenanceRepository {
        self
    }

    fn audit(&mut self) -> &mut dyn AuditRepository {
        self
    }

    fn jobs(&mut self) -> &mut dyn JobRepository {
        self
    }

    fn settings(&mut self) -> &mut dyn SettingsRepository {
        self
    }

    async fn commit(self: Box<Self>) -> Result<(), StorageError> {
        self.tx.into_inner().commit().await.map_err(|_| StorageError::StorageUnavailable)
    }

    async fn rollback(self: Box<Self>) -> Result<(), StorageError> {
        self.tx.into_inner().rollback().await.map_err(|_| StorageError::StorageUnavailable)
    }
}

// ============================================================================
// Source Repository
// ============================================================================

#[async_trait]
impl SourceRepository for SqliteUnitOfWork {
    async fn get(&self, id: &str) -> Result<Option<SourceRow>, StorageError> {
        let mut tx = self.tx().await;
        let row = sqlx::query(
            "SELECT id, title, content_type, language, created_at FROM sources WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
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

    async fn insert_version(&mut self, version: SourceVersionRow) -> Result<(), StorageError> {
        let mut tx = self.tx().await;
        sqlx::query(
            r#"INSERT INTO source_versions (
                id, source_id, version, schema_version, state, trust_level,
                license_status, license_json, manifest_blob_id, manifest_hash,
                content_hash, source_urls, publication_date, imported_at,
                validated_at, approved_at, approved_by, activated_at,
                deprecated_at, quarantine_reason, validation_report, notes, created_at
            ) VALUES (?, ?, ?, 1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(&version.id)
        .bind(&version.source_id)
        .bind(&version.version)
        .bind(&version.state)
        .bind(&version.trust_level)
        .bind(&version.license_status)
        .bind("{}")
        .bind(&version.manifest_blob_id)
        .bind(Option::<String>::None) // manifest_hash
        .bind(&version.content_hash)
        .bind("[]")
        .bind(Option::<String>::None) // publication_date
        .bind(Option::<String>::None) // imported_at
        .bind(Option::<String>::None) // validated_at
        .bind(Option::<String>::None) // approved_at
        .bind(Option::<String>::None) // approved_by
        .bind(Option::<String>::None) // activated_at
        .bind(Option::<String>::None) // deprecated_at
        .bind(Option::<String>::None) // quarantine_reason
        .bind(Option::<String>::None) // validation_report
        .bind(Option::<String>::None) // notes
        .bind(now())
        .execute(&mut *tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;

        Ok(())
    }

    async fn transition_state(
        &mut self,
        source_version_id: &str,
        from: &str,
        to: &str,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx().await;
        sqlx::query("UPDATE source_versions SET state = ? WHERE id = ? AND state = ?")
            .bind(to)
            .bind(source_version_id)
            .bind(from)
            .execute(&mut *tx)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(())
    }

    async fn list_versions(&self, source_id: &str) -> Result<Vec<SourceVersionRow>, StorageError> {
        let mut tx = self.tx().await;
        let rows = sqlx::query(
            "SELECT id, source_id, version, state, trust_level, license_status, content_hash, manifest_blob_id
             FROM source_versions WHERE source_id = ? ORDER BY created_at",
        )
        .bind(source_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;

        Ok(rows.into_iter().map(|r| SourceVersionRow {
            id: r.get("id"),
            source_id: r.get("source_id"),
            version: r.get("version"),
            state: r.get("state"),
            trust_level: r.get("trust_level"),
            license_status: r.get("license_status"),
            content_hash: r.get("content_hash"),
            manifest_blob_id: r.get("manifest_blob_id"),
        }).collect())
    }

    async fn record_transition(
        &mut self,
        transition: StateTransitionRow,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx().await;
        sqlx::query(
            "INSERT INTO source_state_transitions (id, source_version_id, from_state, to_state, actor_id, reason, occurred_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&transition.id)
        .bind(&transition.source_version_id)
        .bind(&transition.from_state)
        .bind(&transition.to_state)
        .bind(&transition.actor_id)
        .bind(&transition.reason)
        .bind(&transition.occurred_at)
        .execute(&mut *tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(())
    }
}

// ============================================================================
// Provenance Repository
// ============================================================================

#[async_trait]
impl ProvenanceRepository for SqliteUnitOfWork {
    async fn insert(&mut self, record: ProvenanceRecord) -> Result<(), StorageError> {
        let mut tx = self.tx().await;
        sqlx::query(
            r#"INSERT INTO provenance_records (
                id, layer, subject_urn, attribution_kind, attribution_json,
                source_version_id, location_json, quoted_text_hash, trust_level,
                verification_status, confidence, versions_json, created_at,
                created_by, reviewed_by, reviewed_at, review_note, superseded_by
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(&record.id)
        .bind(&record.layer)
        .bind(&record.subject_urn)
        .bind(&record.attribution_kind)
        .bind(&record.attribution_json)
        .bind(&record.source_version_id)
        .bind(Option::<String>::None) // location_json
        .bind(Option::<String>::None) // quoted_text_hash
        .bind(&record.trust_level)
        .bind(&record.verification_status)
        .bind(record.confidence)
        .bind(&record.versions_json)
        .bind(now())
        .bind(&record.created_by)
        .bind(Option::<String>::None) // reviewed_by
        .bind(Option::<String>::None) // reviewed_at
        .bind(Option::<String>::None) // review_note
        .bind(Option::<String>::None) // superseded_by
        .execute(&mut *tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(())
    }

    async fn get(&self, id: &str) -> Result<Option<ProvenanceRecord>, StorageError> {
        let mut tx = self.tx().await;
        let row = sqlx::query(
            r#"SELECT id, layer, subject_urn, attribution_kind, attribution_json,
                      source_version_id, location_json, quoted_text_hash, trust_level,
                      verification_status, confidence, versions_json, created_at,
                      created_by, reviewed_by, reviewed_at, review_note, superseded_by
               FROM provenance_records WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;

        Ok(row.map(|r| map_provenance_row(r)))
    }

    async fn list_by_subject(
        &self,
        subject_urn: &str,
    ) -> Result<Vec<ProvenanceRecord>, StorageError> {
        let mut tx = self.tx().await;
        let rows = sqlx::query(
            r#"SELECT id, layer, subject_urn, attribution_kind, attribution_json,
                      source_version_id, location_json, quoted_text_hash, trust_level,
                      verification_status, confidence, versions_json, created_at,
                      created_by, reviewed_by, reviewed_at, review_note, superseded_by
               FROM provenance_records WHERE subject_urn = ? ORDER BY created_at"#,
        )
        .bind(subject_urn)
        .fetch_all(&mut *tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;

        Ok(rows.into_iter().map(map_provenance_row).collect())
    }

    async fn record_review(&mut self, review: ReviewRecord) -> Result<(), StorageError> {
        let mut tx = self.tx().await;
        sqlx::query(
            r#"INSERT INTO review_queue (id, provenance_id, queue, evidence_json, state, decided_by, decided_at, decision_note, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
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
        .execute(&mut *tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(())
    }
}

fn map_provenance_row(r: SqliteRow) -> ProvenanceRecord {
    ProvenanceRecord {
        id: r.get("id"),
        layer: r.get("layer"),
        subject_urn: r.get("subject_urn"),
        attribution_kind: r.get("attribution_kind"),
        attribution_json: r.get("attribution_json"),
        source_version_id: r.get("source_version_id"),
        location_json: r.get("location_json"),
        quoted_text_hash: r.get("quoted_text_hash"),
        trust_level: r.get("trust_level"),
        verification_status: r.get("verification_status"),
        confidence: r.get("confidence"),
        versions_json: r.get("versions_json"),
        created_at: r.get("created_at"),
        created_by: r.get("created_by"),
        reviewed_by: r.get("reviewed_by"),
        reviewed_at: r.get("reviewed_at"),
        review_note: r.get("review_note"),
        superseded_by: r.get("superseded_by"),
    }
}

// ============================================================================
// Audit Repository
// ============================================================================

/// Compute the hash-chain value for an audit event.
///
/// `prev` is the chain hash of the previous event (empty for the genesis event).
/// The canonical payload covers every user-supplied field of the event.
fn chain_hash_for(prev: &str, e: &AuditEvent) -> String {
    let canonical = format!(
        "{prev}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        e.sequence,
        e.occurred_at,
        e.actor_kind,
        e.actor_id.as_deref().unwrap_or(""),
        e.action,
        e.subject_urn,
        e.outcome,
        e.reason.as_deref().unwrap_or(""),
        e.before_json.as_deref().unwrap_or(""),
        e.after_json.as_deref().unwrap_or(""),
        e.request_id.as_deref().unwrap_or(""),
    );
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[async_trait]
impl AuditRepository for SqliteUnitOfWork {
    async fn append(&mut self, event: AuditEvent) -> Result<(), StorageError> {
        let mut tx = self.tx().await;

        let prev: String = sqlx::query("SELECT chain_hash FROM audit_events ORDER BY sequence DESC LIMIT 1")
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?
            .map(|r| r.get("chain_hash"))
            .unwrap_or_default();

        let chain_hash = chain_hash_for(&prev, &event);

        sqlx::query(
            r#"INSERT INTO audit_events
               (id, sequence, occurred_at, actor_kind, actor_id, action, subject_urn, outcome,
                reason, before_json, after_json, request_id, prev_chain_hash, chain_hash)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
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
        .bind(&prev)
        .bind(&chain_hash)
        .execute(&mut *tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;

        Ok(())
    }

    async fn list_by_subject(
        &self,
        subject_urn: &str,
    ) -> Result<Vec<AuditEvent>, StorageError> {
        let mut tx = self.tx().await;
        let rows = sqlx::query(
            r#"SELECT id, sequence, occurred_at, actor_kind, actor_id, action, subject_urn, outcome,
                      reason, before_json, after_json, request_id, prev_chain_hash, chain_hash
               FROM audit_events WHERE subject_urn = ? ORDER BY sequence"#,
        )
        .bind(subject_urn)
        .fetch_all(&mut *tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;

        Ok(rows.into_iter().map(map_audit_row).collect())
    }

    async fn verify_chain(&self) -> Result<ChainVerificationResult, StorageError> {
        let mut tx = self.tx().await;
        let rows = sqlx::query("SELECT id, sequence, occurred_at, actor_kind, actor_id, action, subject_urn, outcome, reason, before_json, after_json, request_id, prev_chain_hash, chain_hash FROM audit_events ORDER BY sequence")
            .fetch_all(&mut *tx)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;

        let events: Vec<AuditEvent> = rows.into_iter().map(map_audit_row).collect();

        let mut valid = true;
        let mut gaps: Vec<u64> = Vec::new();
        let mut prev_chain = String::new();
        let mut expected_seq = 1u64;

        for e in &events {
            if e.sequence != expected_seq {
                gaps.push(expected_seq);
                valid = false;
                expected_seq = e.sequence;
            }

            if e.prev_chain_hash != prev_chain {
                valid = false;
            }

            let recomputed = chain_hash_for(&e.prev_chain_hash, e);
            if recomputed != e.chain_hash {
                valid = false;
            }

            prev_chain = e.chain_hash.clone();
            expected_seq += 1;
        }

        Ok(ChainVerificationResult {
            valid,
            expected_next_sequence: expected_seq,
            expected_next_hash: prev_chain,
            gaps,
        })
    }
}

fn map_audit_row(r: SqliteRow) -> AuditEvent {
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

// ============================================================================
// Job Repository
// ============================================================================

#[async_trait]
impl JobRepository for SqliteUnitOfWork {
    async fn enqueue(&mut self, job: JobRecord) -> Result<(), StorageError> {
        let mut tx = self.tx().await;
        sqlx::query(
            r#"INSERT INTO jobs (
                id, kind, payload_json, idempotency_key, state, priority, attempts,
                max_attempts, available_at, lease_owner, lease_expires_at,
                checkpoint_json, progress_json, cancel_requested, parent_job_id,
                versions_json, error_code, error_json, created_at, started_at,
                finished_at, created_by
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(&job.id)
        .bind(&job.kind)
        .bind(&job.payload_json)
        .bind(&job.idempotency_key)
        .bind(&job.state)
        .bind(job.priority)
        .bind(job.attempts as i64)
        .bind(job.max_attempts as i64)
        .bind(&job.available_at)
        .bind(&job.lease_owner)
        .bind(&job.lease_expires_at)
        .bind(&job.checkpoint_json)
        .bind(Option::<String>::None) // progress_json
        .bind(job.cancel_requested as i64)
        .bind(Option::<String>::None) // parent_job_id
        .bind("{}") // versions_json
        .bind(Option::<String>::None) // error_code
        .bind(Option::<String>::None) // error_json
        .bind(now())
        .bind(Option::<String>::None) // started_at
        .bind(Option::<String>::None) // finished_at
        .bind(&job.created_by)
        .execute(&mut *tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(())
    }

    async fn claim(
        &mut self,
        job_id: &str,
        owner: &str,
    ) -> Result<Option<JobRecord>, StorageError> {
        let mut tx = self.tx().await;
        let rows = sqlx::query(
            r#"UPDATE jobs
               SET state = 'Leased', lease_owner = ?, lease_expires_at = ?
               WHERE id = ? AND state = 'Queued' AND available_at <= ?
               RETURNING id, kind, payload_json, idempotency_key, state, priority,
                         attempts, max_attempts, available_at, lease_owner,
                         lease_expires_at, checkpoint_json, cancel_requested, created_by"#,
        )
        .bind(owner)
        .bind(now_plus(Duration::minutes(30)))
        .bind(job_id)
        .bind(now())
        .fetch_all(&mut *tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;

        Ok(rows.into_iter().next().map(map_job_row))
    }

    async fn finish(
        &mut self,
        job_id: &str,
        state: &str,
        result: Option<String>,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx().await;
        sqlx::query("UPDATE jobs SET state = ?, finished_at = ?, checkpoint_json = ? WHERE id = ?")
            .bind(state)
            .bind(now())
            .bind(&result)
            .bind(job_id)
            .execute(&mut *tx)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(())
    }

    async fn cancel(&mut self, job_id: &str) -> Result<(), StorageError> {
        let mut tx = self.tx().await;
        sqlx::query("UPDATE jobs SET cancel_requested = 1, state = 'Cancelled' WHERE id = ?")
            .bind(job_id)
            .execute(&mut *tx)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(())
    }

    async fn checkpoint(
        &mut self,
        job_id: &str,
        progress: Option<String>,
        checkpoint: Option<String>,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx().await;
        sqlx::query(
            "UPDATE jobs SET checkpoint_json = ?, progress_json = ?, state = 'Checkpointed' WHERE id = ?",
        )
        .bind(&checkpoint)
        .bind(&progress)
        .bind(job_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(())
    }

    async fn reap_expired_leases(&mut self) -> Result<Vec<String>, StorageError> {
        let mut tx = self.tx().await;
        let rows = sqlx::query(
            "SELECT id FROM jobs WHERE state = 'Leased' AND lease_expires_at < ?",
        )
        .bind(now())
        .fetch_all(&mut *tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;

        let mut ids = Vec::new();
        for row in rows {
            let id: String = row.get("id");
            sqlx::query(
                "UPDATE jobs SET state = 'Queued', lease_owner = NULL, lease_expires_at = NULL WHERE id = ?",
            )
            .bind(&id)
            .execute(&mut *tx)
            .await
            .map_err(|_| StorageError::StorageUnavailable)?;
            ids.push(id);
        }

        Ok(ids)
    }
}

fn map_job_row(r: SqliteRow) -> JobRecord {
    JobRecord {
        id: r.get("id"),
        kind: r.get("kind"),
        payload_json: r.get("payload_json"),
        idempotency_key: r.get("idempotency_key"),
        state: r.get("state"),
        priority: r.get("priority"),
        attempts: r.get::<i64, _>("attempts") as u32,
        max_attempts: r.get::<i64, _>("max_attempts") as u32,
        available_at: r.get("available_at"),
        lease_owner: r.get("lease_owner"),
        lease_expires_at: r.get("lease_expires_at"),
        checkpoint_json: r.get("checkpoint_json"),
        cancel_requested: r.get::<i64, _>("cancel_requested") != 0,
        created_by: r.get("created_by"),
    }
}

// ============================================================================
// Settings Repository
// ============================================================================

#[async_trait]
impl SettingsRepository for SqliteUnitOfWork {
    async fn get(&self, key: &str) -> Result<Option<SettingRow>, StorageError> {
        let mut tx = self.tx().await;
        let row = sqlx::query(
            "SELECT key, value_json, origin, updated_at, updated_by FROM settings WHERE key = ?",
        )
        .bind(key)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;

        Ok(row.map(|r| map_setting_row(r)))
    }

    async fn set(
        &mut self,
        key: &str,
        value_json: &str,
        origin: &str,
        updated_by: Option<&str>,
    ) -> Result<(), StorageError> {
        let mut tx = self.tx().await;
        sqlx::query(
            r#"INSERT INTO settings (key, value_json, origin, updated_at, updated_by)
               VALUES (?, ?, ?, ?, ?)
               ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json,
                   origin = excluded.origin, updated_at = excluded.updated_at,
                   updated_by = excluded.updated_by"#,
        )
        .bind(key)
        .bind(value_json)
        .bind(origin)
        .bind(now())
        .bind(updated_by)
        .execute(&mut *tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<SettingRow>, StorageError> {
        let mut tx = self.tx().await;
        let rows = sqlx::query(
            "SELECT key, value_json, origin, updated_at, updated_by FROM settings ORDER BY key",
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(|_| StorageError::StorageUnavailable)?;

        Ok(rows.into_iter().map(map_setting_row).collect())
    }
}

fn map_setting_row(r: SqliteRow) -> SettingRow {
    SettingRow {
        key: r.get("key"),
        value_json: r.get("value_json"),
        origin: r.get("origin"),
        updated_at: r.get("updated_at"),
        updated_by: r.get("updated_by"),
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn now() -> String {
    format_rfc3339(time::OffsetDateTime::now_utc())
}

fn now_plus(d: time::Duration) -> String {
    format_rfc3339(time::OffsetDateTime::now_utc() + d)
}

fn format_rfc3339(t: time::OffsetDateTime) -> String {
    t.format(&time::format_description::well_known::Rfc3339).unwrap_or_default()
}

const _: () = {
    // Keep `SqliteRow` import used even if a mapping helper changes later.
    fn _assert_sqlite_row(_r: &SqliteRow) {}
};

// ============================================================================
// Tests
// ============================================================================

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

    #[tokio::test]
    async fn test_migrate() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_migrate.db");
        let db_path = path.to_str().unwrap();

        let version = SqliteDatabase::apply_migrations(db_path).await.unwrap();
        assert!(version > 0, "should apply at least one migration");

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(db_path)
                    .create_if_missing(false)
                    .read_only(true),
            )
            .await
            .unwrap();

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM schema_migrations")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(count.0 > 0, "schema_migrations should have rows");
        assert_eq!(count.0 as u32, version, "applied count matches highest version");

        // Spot-check a table from a later migration exists.
        let found: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'sources'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(found.0, 1, "sources table should exist");
    }

    #[tokio::test]
    async fn test_backup() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_backup.db");
        let backup_path = dir.path().join("test_backup_copy.db");

        let db = SqliteDatabase::new(db_path.to_str().unwrap(), 1, false).await.unwrap();

        let mut uow = db.write().await.unwrap();
        uow.settings()
            .set("test_key", r#"{"val": 42}"#, "test", Some("tester"))
            .await
            .unwrap();
        uow.commit().await.unwrap();

        let dest = backup_path.to_str().unwrap();
        SqliteDatabase::backup(db_path.to_str().unwrap(), dest).await.unwrap();

        assert!(backup_path.exists(), "backup file should exist");
        let metadata = std::fs::metadata(&backup_path).unwrap();
        assert!(metadata.len() > 0, "backup file should not be empty");

        // The backup must be a valid, readable SQLite database containing the data.
        let backup_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(dest)
                    .create_if_missing(false)
                    .read_only(true),
            )
            .await
            .unwrap();

        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM settings WHERE key = 'test_key'")
            .fetch_one(&backup_pool)
            .await
            .unwrap();
        assert_eq!(count, 1, "backup should contain inserted setting");
    }
}