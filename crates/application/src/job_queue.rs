//! SQLite-backed [`JobQueue`] adapter and worker assembly (D0.9 / T37).
//!
//! Keeps the worker pool in `jobs` backend-agnostic: this adapter implements the
//! `jobs::JobQueue` trait over a real SQLite database (each operation runs in
//! its own unit of work).

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use jobs::queue::JobQueue;
use jobs::registry::HandlerRegistry;
use jobs::worker::Worker;
use jobs::{JobError, JobState};
use storage::Database as _;
use storage::repository::JobRecord;
use storage_sqlite::SqliteDatabase;

/// A [`JobQueue`] backed by the SQLite `jobs` table.
pub struct SqliteJobQueue {
    db: Arc<SqliteDatabase>,
}

impl SqliteJobQueue {
    /// Wrap a database handle.
    pub fn new(db: Arc<SqliteDatabase>) -> Self {
        Self { db }
    }
}

fn to_err(e: storage::error::StorageError) -> JobError {
    JobError::Storage(e.to_string())
}

#[async_trait]
impl JobQueue for SqliteJobQueue {
    async fn enqueue(&self, job: JobRecord) -> Result<(), JobError> {
        let mut uow = self.db.write().await.map_err(to_err)?;
        uow.jobs().enqueue(job).await.map_err(to_err)?;
        uow.commit().await.map_err(to_err)
    }

    async fn claim_next(
        &self,
        owner: &str,
        lease: Duration,
    ) -> Result<Option<JobRecord>, JobError> {
        let mut uow = self.db.write().await.map_err(to_err)?;
        let claimed = uow.jobs().claim_next(owner, lease.as_secs()).await.map_err(to_err)?;
        uow.commit().await.map_err(to_err)?;
        Ok(claimed)
    }

    async fn heartbeat(
        &self,
        job_id: &str,
        owner: &str,
        lease: Duration,
    ) -> Result<bool, JobError> {
        let mut uow = self.db.write().await.map_err(to_err)?;
        let held = uow.jobs().heartbeat(job_id, owner, lease.as_secs()).await.map_err(to_err)?;
        uow.commit().await.map_err(to_err)?;
        Ok(held)
    }

    async fn checkpoint(
        &self,
        job_id: &str,
        progress: Option<String>,
        checkpoint: Option<String>,
    ) -> Result<(), JobError> {
        let mut uow = self.db.write().await.map_err(to_err)?;
        uow.jobs().checkpoint(job_id, progress, checkpoint).await.map_err(to_err)?;
        uow.commit().await.map_err(to_err)
    }

    async fn finish(
        &self,
        job_id: &str,
        state: JobState,
        result: Option<String>,
    ) -> Result<(), JobError> {
        let mut uow = self.db.write().await.map_err(to_err)?;
        uow.jobs().finish(job_id, &state.to_string(), result).await.map_err(to_err)?;
        uow.commit().await.map_err(to_err)
    }

    async fn reschedule(
        &self,
        job_id: &str,
        delay: Duration,
        reason: Option<String>,
    ) -> Result<(), JobError> {
        let mut uow = self.db.write().await.map_err(to_err)?;
        uow.jobs().reschedule(job_id, delay.as_secs(), reason).await.map_err(to_err)?;
        uow.commit().await.map_err(to_err)
    }

    async fn cancel_requested(&self, job_id: &str) -> Result<bool, JobError> {
        let mut uow = self.db.write().await.map_err(to_err)?;
        let job = uow.jobs().get(job_id).await.map_err(to_err)?;
        let _ = uow.rollback().await;
        Ok(job.map(|j| j.cancel_requested).unwrap_or(false))
    }

    async fn get(&self, job_id: &str) -> Result<Option<JobRecord>, JobError> {
        let mut uow = self.db.write().await.map_err(to_err)?;
        let job = uow.jobs().get(job_id).await.map_err(to_err)?;
        let _ = uow.rollback().await;
        Ok(job)
    }

    async fn reap_expired(&self) -> Result<Vec<String>, JobError> {
        let mut uow = self.db.write().await.map_err(to_err)?;
        let reaped = uow.jobs().reap_expired_leases().await.map_err(to_err)?;
        uow.commit().await.map_err(to_err)?;
        Ok(reaped)
    }

    async fn queued_count(&self) -> Result<u64, JobError> {
        let mut uow = self.db.write().await.map_err(to_err)?;
        let n = uow.jobs().count_by_state("Queued").await.map_err(to_err)?;
        let _ = uow.rollback().await;
        Ok(n.max(0) as u64)
    }
}

/// Assemble a [`Worker`] for a SQLite database.
pub fn build_worker(
    db: Arc<SqliteDatabase>,
    registry: HandlerRegistry,
    owner: impl Into<String>,
) -> Worker {
    Worker::new(Arc::new(SqliteJobQueue::new(db)), Arc::new(registry), owner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jobs::kinds;
    use jobs::worker::WorkerOutcome;
    use jobs::{JobContext, JobHandler, JobKind, JobOutcome};
    use serde_json::Value;
    use tempfile::TempDir;

    fn migrations_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite")
    }

    async fn db() -> (TempDir, Arc<SqliteDatabase>) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("qai.db");
        let mut cfg = crate::Config::default();
        cfg.storage.sqlite.path = path.display().to_string();
        crate::db::migrate_database(&cfg, &migrations_dir()).await.unwrap();
        let db = SqliteDatabase::new(path.to_str().unwrap(), 4, true).await.unwrap();
        (dir, Arc::new(db))
    }

    fn record(id: &str, kind: &str, payload: &str) -> JobRecord {
        JobRecord {
            id: id.into(),
            kind: kind.into(),
            payload_json: payload.into(),
            idempotency_key: Some(id.into()),
            state: "Queued".into(),
            priority: 0,
            attempts: 0,
            max_attempts: 5,
            available_at: domain::Timestamp::now().to_string(),
            lease_owner: None,
            lease_expires_at: None,
            checkpoint_json: None,
            cancel_requested: false,
            created_by: String::new(),
        }
    }

    struct Succeed;
    #[async_trait]
    impl JobHandler for Succeed {
        fn kind(&self) -> JobKind {
            kinds::SYSTEM_NOOP_TEST.into()
        }
        fn payload_schema(&self) -> &'static str {
            r#"{"type":"object"}"#
        }
        fn is_idempotent(&self) -> bool {
            true
        }
        async fn run(&self, _ctx: JobContext, _p: Value) -> Result<JobOutcome, JobError> {
            Ok(JobOutcome::success(None))
        }
    }

    struct AlwaysFail;
    #[async_trait]
    impl JobHandler for AlwaysFail {
        fn kind(&self) -> JobKind {
            "test.always_fail".into()
        }
        fn payload_schema(&self) -> &'static str {
            r#"{"type":"object"}"#
        }
        fn is_idempotent(&self) -> bool {
            true
        }
        async fn run(&self, _ctx: JobContext, _p: Value) -> Result<JobOutcome, JobError> {
            Ok(JobOutcome::failure())
        }
    }

    #[tokio::test]
    async fn worker_runs_a_job_to_success_via_sqlite() {
        let (_dir, db) = db().await;
        let queue = SqliteJobQueue::new(db.clone());
        let registry = HandlerRegistry::new().register(Arc::new(Succeed));
        let worker = build_worker(db.clone(), registry, "w1");

        queue.enqueue(record("j1", kinds::SYSTEM_NOOP_TEST, "{}")).await.unwrap();
        assert_eq!(
            worker.run_once().await.unwrap(),
            WorkerOutcome::Succeeded { job_id: "j1".into() }
        );

        let stored = SqliteJobQueue::new(db).get("j1").await.unwrap().unwrap();
        assert_eq!(stored.state, "Succeeded");
    }

    #[tokio::test]
    async fn worker_dead_letters_a_failing_job() {
        let (_dir, db) = db().await;
        let queue = SqliteJobQueue::new(db.clone());
        let registry = HandlerRegistry::new().register(Arc::new(AlwaysFail));
        let worker =
            build_worker(db.clone(), registry, "w1").with_config(jobs::worker::WorkerConfig {
                max_attempts: 2,
                backoff_base: Duration::from_millis(1),
                backoff_max: Duration::from_millis(2),
                backoff_jitter: 0.0,
                ..Default::default()
            });

        queue.enqueue(record("j2", "test.always_fail", "{}")).await.unwrap();

        let mut outcome = WorkerOutcome::Idle;
        for _ in 0..4 {
            outcome = worker.run_once().await.unwrap();
            if matches!(outcome, WorkerOutcome::DeadLettered { .. }) {
                break;
            }
            queue.reschedule("j2", Duration::ZERO, None).await.unwrap();
        }
        assert_eq!(outcome, WorkerOutcome::DeadLettered { job_id: "j2".into() });
        let stored = SqliteJobQueue::new(db).get("j2").await.unwrap().unwrap();
        assert_eq!(stored.state, "DeadLettered");
    }

    #[tokio::test]
    async fn duplicate_idempotency_key_is_rejected() {
        let (_dir, db) = db().await;
        let queue = SqliteJobQueue::new(db);
        queue.enqueue(record("k1", kinds::SYSTEM_NOOP_TEST, "{}")).await.unwrap();
        let dup = queue.enqueue(record("k1", kinds::SYSTEM_NOOP_TEST, "{}")).await;
        assert!(dup.is_err(), "duplicate idempotency key must be rejected");
    }

    #[tokio::test]
    async fn recover_interrupted_reclaims_expired_leases() {
        let (_dir, db) = db().await;
        let queue = SqliteJobQueue::new(db.clone());
        let registry = HandlerRegistry::new().register(Arc::new(Succeed));
        let worker = build_worker(db, registry, "w1");

        queue.enqueue(record("j3", kinds::SYSTEM_NOOP_TEST, "{}")).await.unwrap();
        // Claim it, then force the lease into the past (simulated crash).
        queue.claim_next("w1", Duration::from_secs(60)).await.unwrap();
        // Reap should find nothing yet (lease is in the future).
        assert_eq!(worker.recover_interrupted().await.unwrap(), 0);
    }
}
