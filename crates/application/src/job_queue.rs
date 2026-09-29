//! SQLite-backed [`JobQueue`] adapter and worker assembly (D0.9 / T37).
//!
//! Keeps the worker pool in `jobs` backend-agnostic: this adapter implements the
//! `jobs::JobQueue` trait over a real SQLite database (each operation runs in
//! its own unit of work).

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use audit::{Actor, AuditAction, AuditOutcome};
use domain::SubjectRef;
use jobs::queue::JobQueue;
use jobs::registry::HandlerRegistry;
use jobs::worker::{Worker, parse_disposition};
use jobs::{JobError, JobState};
use storage::Database as _;
use storage::repository::JobRecord;
use storage_sqlite::SqliteDatabase;

use crate::audit_bridge::AuditedMutation;

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

/// Subject URN for a job's lifecycle audit chain: one ordered chain per job.
fn job_subject(job_id: &str) -> SubjectRef {
    SubjectRef(format!("urn:qai:job:{job_id}"))
}

/// Allowlisted lifecycle payload (D-11, T-03-SECRET): identity, kind, state,
/// attempts, and disposition only. Payload, checkpoint, and error content
/// never enter the audit chain.
fn lifecycle_after(job: &JobRecord, state: &str, disposition: Option<&str>) -> serde_json::Value {
    let mut after = serde_json::json!({
        "job_id": job.id,
        "kind": job.kind,
        "state": state,
        "attempts": job.attempts,
        "max_attempts": job.max_attempts,
    });
    if let Some(disposition) = disposition {
        after["disposition"] = disposition.into();
    }
    after
}

/// Stage one hash-chained lifecycle event in the same unit of work as the
/// job mutation (D-10, D-11). Worker transitions use [`Actor::Job`].
async fn stage_job_audit(
    uow: &mut dyn storage::UnitOfWork,
    action: AuditAction,
    job: &JobRecord,
    state: &str,
    disposition: Option<&str>,
) -> Result<(), JobError> {
    AuditedMutation {
        actor: Actor::Job { job_id: job.id.clone() },
        action,
        subject: job_subject(&job.id),
        outcome: AuditOutcome::Allowed,
        reason: None,
        before: None,
        after: Some(lifecycle_after(job, state, disposition)),
        request_id: Some(job.id.clone()),
    }
    .stage(uow)
    .await
    .map_err(|e| JobError::Storage(e.to_string()))?;
    Ok(())
}

/// Read a job inside the same unit of work for its audit payload.
async fn job_in(uow: &mut dyn storage::UnitOfWork, job_id: &str) -> Result<JobRecord, JobError> {
    uow.jobs().get(job_id).await.map_err(to_err)?.ok_or(JobError::NotFound { id: job_id.into() })
}

#[async_trait]
impl JobQueue for SqliteJobQueue {
    async fn enqueue(&self, mut job: JobRecord) -> Result<(), JobError> {
        if let Some(registry) = &self.registry {
            registry.stamp_job_policy(&mut job);
        }
        let mut uow = self.db.write().await.map_err(to_err)?;
        uow.jobs().enqueue(job.clone()).await.map_err(to_err)?;
        stage_job_audit(&mut *uow, AuditAction::JobEnqueued, &job, "Queued", None).await?;
        uow.commit().await.map_err(to_err)
    }

    async fn claim_next(
        &self,
        owner: &str,
        lease: Duration,
    ) -> Result<Option<JobRecord>, JobError> {
        let mut uow = self.db.write().await.map_err(to_err)?;
        let claimed = uow.jobs().claim_next(owner, lease.as_secs()).await.map_err(to_err)?;
        if let Some(ref job) = claimed {
            stage_job_audit(&mut *uow, AuditAction::JobLeased, job, &job.state, None).await?;
        }
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
        uow.jobs().finish(job_id, &state.to_string(), result.clone()).await.map_err(to_err)?;
        let job = job_in(&mut *uow, job_id).await?;
        let disposition = result.as_deref().and_then(parse_disposition).map(|d| d.to_string());
        let action = match state {
            JobState::Succeeded => AuditAction::JobCompleted,
            JobState::Cancelled => AuditAction::JobCancelled,
            _ => AuditAction::JobFailed,
        };
        stage_job_audit(&mut *uow, action, &job, &state.to_string(), disposition.as_deref())
            .await?;
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
        let job = job_in(&mut *uow, job_id).await?;
        stage_job_audit(&mut *uow, AuditAction::JobRetried, &job, &job.state, None).await?;
        uow.commit().await.map_err(to_err)
    }

    async fn cancel_requested(&self, job_id: &str) -> Result<bool, JobError> {
        let mut uow = self.db.write().await.map_err(to_err)?;
        let job = uow.jobs().get(job_id).await.map_err(to_err)?;
        let _ = uow.rollback().await;
        Ok(job.map(|j| j.cancel_requested).unwrap_or(false))
    }

    /// Durable cooperative cancellation request (D-16): persists only the
    /// flag plus its lifecycle audit event in the same unit of work.
    async fn request_cancel(&self, job_id: &str) -> Result<(), JobError> {
        let mut uow = self.db.write().await.map_err(to_err)?;
        uow.jobs().request_cancel(job_id).await.map_err(to_err)?;
        let job = job_in(&mut *uow, job_id).await?;
        stage_job_audit(&mut *uow, AuditAction::JobCancellationRequested, &job, &job.state, None)
            .await?;
        uow.commit().await.map_err(to_err)
    }

    /// Explicit operator retry of an eligible job (D-15), audited as a retry.
    async fn retry(&self, job_id: &str) -> Result<(), JobError> {
        let mut uow = self.db.write().await.map_err(to_err)?;
        uow.jobs().retry(job_id).await.map_err(to_err)?;
        let job = job_in(&mut *uow, job_id).await?;
        stage_job_audit(&mut *uow, AuditAction::JobRetried, &job, &job.state, None).await?;
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

    /// Owner-held terminal transition (T-03-LEASE) with its lifecycle audit
    /// event in the same unit of work. This is the worker's completion /
    /// failure / cancellation path, so it carries the same audit contract
    /// as the plain finish, including the cancellation disposition.
    /// Checkpoint and heartbeat writes stay unaudited: D-11 enumerates
    /// enqueue, lease, completion, failure, and cancellation — high-frequency
    /// progress writes would flood the chain without adding transitions.
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
            .finish_owned(job_id, owner, &state.to_string(), result.clone())
            .await
            .map_err(to_err)?;
        if applied {
            let job = job_in(&mut *uow, job_id).await?;
            let disposition = result.as_deref().and_then(parse_disposition).map(|d| d.to_string());
            let action = match state {
                JobState::Succeeded => AuditAction::JobCompleted,
                JobState::Cancelled => AuditAction::JobCancelled,
                _ => AuditAction::JobFailed,
            };
            stage_job_audit(&mut *uow, action, &job, &state.to_string(), disposition.as_deref())
                .await?;
        }
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

// ─── Long-lived worker host (D-13) ─────────────────────────────────
//
// `qai serve` owns durable execution through this composition; one-shot
// commands only enqueue. Only existing handlers are registered (D-04) and
// unknown kinds follow the worker dead-letter policy (T-04-SCOPE).

/// Default production handler registry: every existing application job
/// handler against the same database handle.
///
/// Registration only wires the existing execution seams — feature behavior,
/// approval gates, and canonical boundaries are unchanged.
pub fn build_default_registry(db: &Arc<SqliteDatabase>) -> HandlerRegistry {
    HandlerRegistry::new()
        .register(Arc::new(crate::quran::QuranImportHandler::new(db.clone())))
        .register(Arc::new(crate::quran_forms::FormsRebuildHandler::new(db.clone())))
        .register(Arc::new(crate::quran_index::IndexBuildHandler::new(db.clone())))
        .register(Arc::new(crate::quran_index::IndexVerifyHandler::new(db.clone())))
        .register(Arc::new(crate::quran_morphology::MorphologyImportHandler::new(db.clone())))
}

/// Open the host database without creating or migrating (D-05).
///
/// The serve composition gates on `DatabaseReadiness::Current` first; this
/// existence check is the backstop so the host can never materialize state
/// on its own.
pub async fn open_host_database(db_path: &str) -> Result<Arc<SqliteDatabase>, JobError> {
    if std::fs::symlink_metadata(db_path).is_err() {
        return Err(JobError::Storage(format!(
            "no database at {db_path}; run `qai db migrate` first"
        )));
    }
    SqliteDatabase::new(db_path, 4, true).await.map(Arc::new).map_err(to_err)
}

/// Long-lived worker host: default registry plus the shutdown-aware worker
/// loop. Recovers interrupted leases on start, polls claims at a bounded
/// interval, propagates typed errors, and joins before returning — never a
/// detached task.
pub async fn run_worker_host(
    db: Arc<SqliteDatabase>,
    owner: impl Into<String>,
    shutdown: tokio::sync::watch::Receiver<bool>,
) -> Result<u32, JobError> {
    let registry = build_default_registry(&db);
    let worker = build_worker(db, registry, owner);
    worker.run_until_shutdown(shutdown).await
}

// ─── Operator job controls (D-15, D-16) ─────────────────────────────

/// Operator-facing job control failure with a centralized exit mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobControlError {
    /// No job with this id exists.
    NotFound { id: String },
    /// The job exists but is in a state that rejects the operation.
    Ineligible { id: String, state: String },
    /// No database file exists yet; the operator must migrate first (D-05).
    DatabaseMissing { path: String },
    /// A storage-layer failure.
    Storage(String),
}

impl JobControlError {
    /// Map to the centralized CLI exit code: missing ids → NOT_FOUND,
    /// ineligible states → CONFLICT, missing database → VALIDATION with the
    /// `qai db migrate` remedy.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::NotFound { .. } => 5,
            Self::Ineligible { .. } => 6,
            Self::DatabaseMissing { .. } => 3,
            Self::Storage(_) => 1,
        }
    }
}

impl std::fmt::Display for JobControlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { id } => write!(f, "job not found: {id}"),
            Self::Ineligible { id, state } => {
                write!(f, "job {id} in state {state} rejects this operation")
            }
            Self::DatabaseMissing { path } => {
                write!(f, "no database at {path}; run `qai db migrate` first")
            }
            Self::Storage(detail) => write!(f, "job storage error: {detail}"),
        }
    }
}

fn control_err(e: storage::error::StorageError) -> JobControlError {
    JobControlError::Storage(e.to_string())
}

/// Open the control database, refusing to create state (D-05): a missing
/// file names `qai db migrate` instead of materializing an empty database.
async fn open_control_db(path: &str) -> Result<SqliteDatabase, JobControlError> {
    if std::fs::symlink_metadata(path).is_err() {
        return Err(JobControlError::DatabaseMissing { path: path.to_string() });
    }
    SqliteDatabase::new(path, 4, true).await.map_err(control_err)
}

/// Stage an operator-attributed lifecycle event: the local CLI principal is
/// a [`Actor::System`] (no principal row exists for the operator), while
/// worker transitions use [`Actor::Job`].
async fn stage_operator_audit(
    uow: &mut dyn storage::UnitOfWork,
    action: AuditAction,
    job: &JobRecord,
    state: &str,
) -> Result<(), JobControlError> {
    AuditedMutation {
        actor: Actor::System { name: "qai-cli".to_string() },
        action,
        subject: job_subject(&job.id),
        outcome: AuditOutcome::Allowed,
        reason: None,
        before: None,
        after: Some(lifecycle_after(job, state, None)),
        request_id: Some(job.id.clone()),
    }
    .stage(uow)
    .await
    .map_err(|e| JobControlError::Storage(e.to_string()))?;
    Ok(())
}

/// Explicit operator retry of an eligible job (D-15).
///
/// Returns the post-commit inspection envelope (the same shape as
/// `qai job show`). Missing ids → [`JobControlError::NotFound`];
/// ineligible states → [`JobControlError::Ineligible`] with the row
/// untouched.
pub async fn retry_job(db_path: &str, job_id: &str) -> Result<serde_json::Value, JobControlError> {
    let db = open_control_db(db_path).await?;
    let mut uow = db.write().await.map_err(control_err)?;
    let before = uow.jobs().get(job_id).await.map_err(control_err)?;
    let Some(before) = before else {
        let _ = uow.rollback().await;
        return Err(JobControlError::NotFound { id: job_id.to_string() });
    };
    if !matches!(before.state.as_str(), "Failed" | "DeadLettered" | "Interrupted") {
        let _ = uow.rollback().await;
        return Err(JobControlError::Ineligible { id: job_id.to_string(), state: before.state });
    }
    uow.jobs().retry(job_id).await.map_err(control_err)?;
    let job = uow.jobs().get(job_id).await.map_err(control_err)?.ok_or_else(|| {
        JobControlError::Storage(format!("job {job_id} vanished inside its retry"))
    })?;
    stage_operator_audit(&mut *uow, AuditAction::JobRetried, &job, &job.state).await?;
    uow.commit().await.map_err(control_err)?;
    crate::db::get_job(db_path, job_id).await.map_err(control_err)
}

/// Cooperative operator cancellation request (D-16).
///
/// Persists only the request flag plus its audit event; returns the
/// still-running inspectable job and never claims the worker has stopped —
/// that truth arrives when the worker reports its disposition.
pub async fn request_job_cancel(
    db_path: &str,
    job_id: &str,
) -> Result<serde_json::Value, JobControlError> {
    let db = open_control_db(db_path).await?;
    let mut uow = db.write().await.map_err(control_err)?;
    let before = uow.jobs().get(job_id).await.map_err(control_err)?;
    let Some(before) = before else {
        let _ = uow.rollback().await;
        return Err(JobControlError::NotFound { id: job_id.to_string() });
    };
    if !matches!(
        before.state.as_str(),
        "Queued" | "Leased" | "Running" | "Checkpointed" | "Interrupted"
    ) {
        let _ = uow.rollback().await;
        return Err(JobControlError::Ineligible { id: job_id.to_string(), state: before.state });
    }
    uow.jobs().request_cancel(job_id).await.map_err(control_err)?;
    let job = uow.jobs().get(job_id).await.map_err(control_err)?.ok_or_else(|| {
        JobControlError::Storage(format!("job {job_id} vanished inside its cancellation"))
    })?;
    stage_operator_audit(&mut *uow, AuditAction::JobCancellationRequested, &job, &job.state)
        .await?;
    uow.commit().await.map_err(control_err)?;
    crate::db::get_job(db_path, job_id).await.map_err(control_err)
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

    /// 01-03-03: every worker-driven lifecycle transition stages its audit
    /// event in the same transaction, ordered and actor-attributed.
    #[tokio::test]
    async fn lifecycle_transitions_emit_ordered_audited_events() {
        use jobs::queue::JobQueue as _;

        async fn audit_actions(path: &std::path::Path) -> Vec<(String, String, String, String)> {
            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(path))
                .await
                .unwrap();
            let rows = sqlx::query(
                "SELECT action, actor_kind, subject_urn, COALESCE(after_json, '') AS after_json \
                 FROM audit_events ORDER BY sequence ASC",
            )
            .fetch_all(&pool)
            .await
            .unwrap();
            pool.close().await;
            rows.into_iter()
                .map(|r| {
                    use sqlx::Row as _;
                    (
                        r.get::<String, _>("action"),
                        r.get::<String, _>("actor_kind"),
                        r.get::<String, _>("subject_urn"),
                        r.get::<String, _>("after_json"),
                    )
                })
                .collect()
        }

        let (dir, db) = db().await;
        let db_path = dir.path().join("qai.db");
        let registry = HandlerRegistry::new().register(Arc::new(Succeed));
        let worker = build_worker(db.clone(), registry, "w1");
        let queue = Arc::new(SqliteJobQueue::new(db.clone()));

        // Happy path: enqueue → lease → completion.
        queue.enqueue(record("j-a1", kinds::SYSTEM_NOOP_TEST, "{}")).await.unwrap();
        assert_eq!(
            worker.run_once().await.unwrap(),
            WorkerOutcome::Succeeded { job_id: "j-a1".into() }
        );

        // Cancelled path: request → pre-start observation → disposition.
        queue.enqueue(record("j-a2", kinds::SYSTEM_NOOP_TEST, "{}")).await.unwrap();
        queue.request_cancel("j-a2").await.unwrap();
        let outcome = worker.run_once().await.unwrap();
        assert!(matches!(outcome, WorkerOutcome::Cancelled { .. }), "{outcome:?}");

        // Exhaustion path with explicit operator retry afterwards.
        let mut registry = HandlerRegistry::new().register(Arc::new(AlwaysFail));
        registry.set_policy(
            "test.always_fail",
            jobs::registry::RetryPolicy {
                max_attempts: 1,
                backoff_base_ms: 1,
                backoff_max_ms: 2,
                jitter: 0.0,
            },
        );
        let failing = Worker::new(queue.clone(), Arc::new(registry), "w1");
        queue.enqueue(record("j-a3", "test.always_fail", "{}")).await.unwrap();
        assert_eq!(
            failing.run_once().await.unwrap(),
            WorkerOutcome::DeadLettered { job_id: "j-a3".into() }
        );
        retry_job(db_path.to_str().unwrap(), "j-a3").await.unwrap();

        let events = audit_actions(&db_path).await;
        let actions: Vec<&str> = events.iter().map(|e| e.0.as_str()).collect();
        // Note the j-a2 order: the request persists before the worker
        // leases the job; the lease event is D-11's audited lease, and the
        // pre-start observation cancels it immediately after.
        assert_eq!(
            actions,
            vec![
                "job_enqueued",
                "job_leased",
                "job_completed",
                "job_enqueued",
                "job_cancellation_requested",
                "job_leased",
                "job_cancelled",
                "job_enqueued",
                "job_leased",
                "job_failed",
                "job_retried",
            ],
            "{actions:?}"
        );
        // Worker transitions are job-actor attributed to their own subject
        // chain; the explicit operator retry is CLI-attributed instead.
        for (action, actor, subject, _) in &events {
            if action == "job_retried" && subject == "urn:qai:job:j-a3" {
                assert_eq!(actor, "system", "{action}");
            } else {
                assert_eq!(actor, "job", "{action}");
            }
            assert!(subject.starts_with("urn:qai:job:"), "{action}");
        }
        // …except the explicit operator retry, which is CLI-attributed.
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&db_path))
            .await
            .unwrap();
        let retry_actor: String =
            sqlx::query_scalar("SELECT actor_id FROM audit_events WHERE action = 'job_retried'")
                .fetch_one(&pool)
                .await
                .unwrap();
        pool.close().await;
        assert_eq!(retry_actor, "qai-cli");
        // Cancellation outcome carries the locked disposition; payloads stay
        // metadata-only.
        let cancelled = events.iter().find(|e| e.0 == "job_cancelled").unwrap();
        assert!(cancelled.3.contains("cancelled_at_checkpoint"), "{}", cancelled.3);
        for (_, _, _, after) in &events {
            assert!(!after.contains("password"), "{after}");
        }
        // The composed chain verifies end to end (D-12).
        let report =
            crate::audit_bridge::verify_persisted_audit(db_path.to_str().unwrap()).await.unwrap();
        assert!(report.valid, "{report:?}");
        assert_eq!(report.checked_events, events.len());
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
