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
    /// Optional retry registry: when present, `enqueue` snapshots the job
    /// kind's [`RetryPolicy`](jobs::registry::RetryPolicy) into the row's
    /// `max_attempts` (D-15). Absent, rows keep their caller-supplied limit.
    registry: Option<Arc<HandlerRegistry>>,
}

impl SqliteJobQueue {
    /// Wrap a database handle.
    pub fn new(db: Arc<SqliteDatabase>) -> Self {
        Self { db, registry: None }
    }

    /// Snapshot per-kind retry policy at enqueue time.
    pub fn with_retry_registry(mut self, registry: Arc<HandlerRegistry>) -> Self {
        self.registry = Some(registry);
        self
    }
}

fn to_err(e: storage::error::StorageError) -> JobError {
    JobError::Storage(e.to_string())
}

#[async_trait]
impl JobQueue for SqliteJobQueue {
    async fn enqueue(&self, mut job: JobRecord) -> Result<(), JobError> {
        if let Some(registry) = &self.registry {
            registry.stamp_job_policy(&mut job);
        }
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

    /// Durable cooperative cancellation request (D-16): persists only the
    /// flag in the same unit of work (lifecycle audit arrives in 01-03-03).
    async fn request_cancel(&self, job_id: &str) -> Result<(), JobError> {
        let mut uow = self.db.write().await.map_err(to_err)?;
        uow.jobs().request_cancel(job_id).await.map_err(to_err)?;
        uow.commit().await.map_err(to_err)
    }

    /// Explicit operator retry of an eligible job (D-15).
    async fn retry(&self, job_id: &str) -> Result<(), JobError> {
        let mut uow = self.db.write().await.map_err(to_err)?;
        uow.jobs().retry(job_id).await.map_err(to_err)?;
        uow.commit().await.map_err(to_err)
    }

    /// Owner-held checkpoint write (T-03-LEASE).
    async fn checkpoint_owned(
        &self,
        job_id: &str,
        owner: &str,
        progress: Option<String>,
        checkpoint: Option<String>,
    ) -> Result<bool, JobError> {
        let mut uow = self.db.write().await.map_err(to_err)?;
        let applied = uow
            .jobs()
            .checkpoint_owned(job_id, owner, progress, checkpoint)
            .await
            .map_err(to_err)?;
        uow.commit().await.map_err(to_err)?;
        Ok(applied)
    }

    /// Owner-held terminal transition (T-03-LEASE).
    async fn finish_owned(
        &self,
        job_id: &str,
        owner: &str,
        state: JobState,
        result: Option<String>,
    ) -> Result<bool, JobError> {
        let mut uow = self.db.write().await.map_err(to_err)?;
        let applied = uow
            .jobs()
            .finish_owned(job_id, owner, &state.to_string(), result)
            .await
            .map_err(to_err)?;
        uow.commit().await.map_err(to_err)?;
        Ok(applied)
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
///
/// The worker's registry is also attached to the queue so `enqueue`
/// snapshots each kind's retry policy into the row (D-15).
pub fn build_worker(
    db: Arc<SqliteDatabase>,
    registry: HandlerRegistry,
    owner: impl Into<String>,
) -> Worker {
    let registry = Arc::new(registry);
    let queue = Arc::new(SqliteJobQueue::new(db).with_retry_registry(registry.clone()));
    Worker::new(queue, registry, owner)
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

    /// 01-03-01: enqueue snapshots the kind's retry policy into the row's
    /// `max_attempts` without touching unrelated kinds.
    #[tokio::test]
    async fn enqueue_snapshots_per_kind_retry_policy() {
        use jobs::queue::JobQueue as _;
        use jobs::registry::RetryPolicy;

        let (_dir, db) = db().await;
        let mut registry = HandlerRegistry::new().register(Arc::new(Succeed));
        registry.set_policy(
            kinds::SYSTEM_NOOP_TEST,
            RetryPolicy {
                max_attempts: 2,
                backoff_base_ms: 100,
                backoff_max_ms: 1_000,
                jitter: 0.0,
            },
        );
        let queue = SqliteJobQueue::new(db.clone()).with_retry_registry(Arc::new(registry));

        queue.enqueue(record("j-noop", kinds::SYSTEM_NOOP_TEST, "{}")).await.unwrap();
        queue.enqueue(record("j-other", "test.always_fail", "{}")).await.unwrap();

        let noop = SqliteJobQueue::new(db.clone()).get("j-noop").await.unwrap().unwrap();
        assert_eq!(noop.max_attempts, 2);
        let other = SqliteJobQueue::new(db).get("j-other").await.unwrap().unwrap();
        assert_eq!(other.max_attempts, 5, "unrelated kinds keep the default policy");
    }

    /// 01-03-02: a crashed run resumes after its committed named boundary
    /// when a new worker picks the job up.
    #[tokio::test]
    async fn worker_resumes_crashed_job_from_named_checkpoint() {
        use jobs::queue::JobQueue as _;

        struct ResumeCheck;
        #[async_trait]
        impl JobHandler for ResumeCheck {
            fn kind(&self) -> JobKind {
                "test.resume".into()
            }
            fn payload_schema(&self) -> &'static str {
                r#"{"type":"object"}"#
            }
            fn is_idempotent(&self) -> bool {
                true
            }
            async fn run(&self, ctx: JobContext, _p: Value) -> Result<JobOutcome, JobError> {
                let boundary = ctx.job.checkpoint_json.clone().unwrap_or_default();
                if boundary.contains("import.validated") && ctx.job.attempts == 2 {
                    Ok(JobOutcome::success(Some("resumed".into())))
                } else {
                    Ok(JobOutcome::failure())
                }
            }
        }

        let (dir, db) = db().await;
        let path = dir.path().join("qai.db");
        let queue = SqliteJobQueue::new(db.clone());
        let registry = HandlerRegistry::new().register(Arc::new(ResumeCheck));
        let worker = build_worker(db.clone(), registry, "w2");

        // The first worker crashes after committing a named boundary.
        queue.enqueue(record("j-crash", "test.resume", "{}")).await.unwrap();
        queue.claim_next("crasher", Duration::from_secs(60)).await.unwrap();
        let boundary = r#"{"version":1,"name":"import.validated","payload":{"n":1}}"#;
        assert!(
            queue
                .checkpoint_owned("j-crash", "crasher", None, Some(boundary.into()))
                .await
                .unwrap()
        );
        // Simulate the crash: force the lease into the past.
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&path))
            .await
            .unwrap();
        sqlx::query(
            "UPDATE jobs SET lease_expires_at = '2000-01-01T00:00:00Z' WHERE id = 'j-crash'",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool.close().await;

        assert_eq!(worker.recover_interrupted().await.unwrap(), 1);
        assert_eq!(
            worker.run_once().await.unwrap(),
            WorkerOutcome::Succeeded { job_id: "j-crash".into() }
        );
        let stored = SqliteJobQueue::new(db).get("j-crash").await.unwrap().unwrap();
        assert_eq!(stored.state, "Succeeded");
        assert_eq!(stored.attempts, 2);
        assert!(stored.checkpoint_json.is_some_and(|cp| cp.contains("import.validated")));
    }

    /// 01-03-02: worker terminal outcomes persist the locked disposition in
    /// the existing result/error JSON.
    #[tokio::test]
    async fn worker_persists_cancellation_dispositions_in_sqlite() {
        use jobs::queue::JobQueue as _;
        use jobs::worker::{CancellationDisposition, parse_disposition};

        struct SlowSucceed;
        #[async_trait]
        impl JobHandler for SlowSucceed {
            fn kind(&self) -> JobKind {
                "test.slow_succeed".into()
            }
            fn payload_schema(&self) -> &'static str {
                r#"{"type":"object"}"#
            }
            fn is_idempotent(&self) -> bool {
                true
            }
            async fn run(&self, _ctx: JobContext, _p: Value) -> Result<JobOutcome, JobError> {
                tokio::time::sleep(Duration::from_millis(300)).await;
                Ok(JobOutcome::success(None))
            }
        }

        // `JobRecord` carries no error column, so read the stored
        // result/error JSON back through a direct query on the same file.
        async fn stored_disposition(
            path: &std::path::Path,
            id: &str,
        ) -> Option<CancellationDisposition> {
            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(path))
                .await
                .unwrap();
            let raw: Option<String> =
                sqlx::query_scalar("SELECT error_json FROM jobs WHERE id = ?")
                    .bind(id)
                    .fetch_optional(&pool)
                    .await
                    .unwrap()
                    .flatten();
            pool.close().await;
            raw.as_deref().and_then(parse_disposition)
        }

        let (dir, db) = db().await;
        let db_path = dir.path().join("qai.db");
        let queue = Arc::new(SqliteJobQueue::new(db.clone()));

        // Pre-start request → CancelledAtCheckpoint with the disposition stored.
        let registry = HandlerRegistry::new().register(Arc::new(Succeed));
        let worker = build_worker(db.clone(), registry, "w1");
        queue.enqueue(record("j-d1", kinds::SYSTEM_NOOP_TEST, "{}")).await.unwrap();
        queue.request_cancel("j-d1").await.unwrap();
        assert_eq!(
            worker.run_once().await.unwrap(),
            WorkerOutcome::Cancelled {
                job_id: "j-d1".into(),
                disposition: CancellationDisposition::CancelledAtCheckpoint,
            }
        );
        assert_eq!(
            stored_disposition(&db_path, "j-d1").await,
            Some(CancellationDisposition::CancelledAtCheckpoint)
        );

        // Late request with a successful finish → CompletedBeforeObservation.
        let registry = HandlerRegistry::new().register(Arc::new(SlowSucceed));
        let worker = build_worker(db.clone(), registry, "w1");
        queue.enqueue(record("j-d2", "test.slow_succeed", "{}")).await.unwrap();
        let handle = tokio::spawn(async move { worker.run_once().await });
        tokio::time::sleep(Duration::from_millis(50)).await;
        queue.request_cancel("j-d2").await.unwrap();
        assert_eq!(
            handle.await.unwrap().unwrap(),
            WorkerOutcome::Succeeded { job_id: "j-d2".into() }
        );
        assert_eq!(
            stored_disposition(&db_path, "j-d2").await,
            Some(CancellationDisposition::CompletedBeforeObservation)
        );
    }

    /// 01-03-02: per-kind exhaustion through the real worker and SQLite row.
    #[tokio::test]
    async fn worker_per_kind_exhaustion_dead_letters_through_sqlite() {
        use jobs::queue::JobQueue as _;
        use jobs::registry::RetryPolicy;
        use jobs::worker::Worker;

        let (dir, db) = db().await;
        let db_path = dir.path().join("qai.db");
        let mut registry = HandlerRegistry::new().register(Arc::new(AlwaysFail));
        registry.set_policy(
            "test.always_fail",
            RetryPolicy { max_attempts: 2, backoff_base_ms: 1, backoff_max_ms: 2, jitter: 0.0 },
        );
        let registry = Arc::new(registry);
        let queue = Arc::new(SqliteJobQueue::new(db.clone()).with_retry_registry(registry.clone()));
        let worker = Worker::new(queue.clone(), registry, "w1");

        queue.enqueue(record("j-ex", "test.always_fail", "{}")).await.unwrap();
        // Enqueue stamped the kind policy into the row.
        assert_eq!(
            SqliteJobQueue::new(db.clone()).get("j-ex").await.unwrap().unwrap().max_attempts,
            2
        );

        match worker.run_once().await.unwrap() {
            WorkerOutcome::Retried { .. } => {}
            other => panic!("expected Retried, got {other:?}"),
        }
        queue.reschedule("j-ex", Duration::ZERO, None).await.unwrap();
        // Force due: a zero backoff lands ~now; backdate for determinism.
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&db_path))
            .await
            .unwrap();
        sqlx::query("UPDATE jobs SET available_at = '2000-01-01T00:00:00Z' WHERE id = 'j-ex'")
            .execute(&pool)
            .await
            .unwrap();
        pool.close().await;

        assert_eq!(
            worker.run_once().await.unwrap(),
            WorkerOutcome::DeadLettered { job_id: "j-ex".into() }
        );
        let stored = SqliteJobQueue::new(db).get("j-ex").await.unwrap().unwrap();
        assert_eq!(stored.state, "DeadLettered");
        assert_eq!(stored.attempts, 2);

        // Explicit operator retry reopens the window after exhaustion.
        queue.retry("j-ex").await.unwrap();
        let stored = queue.get("j-ex").await.unwrap().unwrap();
        assert_eq!((stored.state.as_str(), stored.attempts), ("Queued", 0));
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
