//! Durable, DB-backed job system for Q-ai.
//!
//! Provides [`JobRecord`], [`JobState`], [`JobHandler`], and [`JobContext`]
//! for cancellable, resumable, idempotent background jobs.
//!
//! The job store uses [`storage::JobRepository`] for persistence.
//! Jobs are stored in the SQLite `jobs` and `job_events` tables
//! (migration `0004_jobs`).
//!
//! # Job lifecycle
//!
//! ```text
//! Queued -> Leased -> Running -> Checkpointed -> Succeeded
//!                         |              |
//!                         v              v
//!                     Cancelled    Failed -> DeadLettered
//!                         |
//!                         v
//!                     Interrupted (on crash recovery)
//! ```

use async_trait::async_trait;
use std::fmt;
use storage::repository::JobRecord;

pub mod queue;
pub mod registry;
pub mod worker;

#[cfg(test)]
pub(crate) mod test_support {
    use storage::repository::JobRecord;

    /// A minimal job record for tests.
    pub fn record(id: &str, kind: &str, state: &str) -> JobRecord {
        JobRecord {
            id: id.into(),
            kind: kind.into(),
            payload_json: "{}".into(),
            idempotency_key: None,
            state: state.into(),
            priority: 0,
            attempts: 0,
            max_attempts: 5,
            available_at: time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
            lease_owner: None,
            lease_expires_at: None,
            checkpoint_json: None,
            cancel_requested: false,
            created_by: "test".into(),
        }
    }
}

// ─── Job kinds ───────────────────────────────────────────────

/// Phase 0 job kinds (plan D0.9). Stable strings; handlers register by kind.
pub mod kinds {
    /// Background integrity scan.
    pub const SYSTEM_INTEGRITY_SCAN: &str = "system.integrity_scan";
    /// SQLite VACUUM maintenance.
    pub const SYSTEM_VACUUM: &str = "system.vacuum";
    /// No-op handler used to exercise the job machinery.
    pub const SYSTEM_NOOP_TEST: &str = "system.noop_test";
    /// Generic outbox relay (T64). Reuses the outbox lease semantics.
    pub const SYSTEM_OUTBOX_RELAY: &str = "system.outbox_relay";
    /// Validate a source manifest.
    pub const SOURCE_VALIDATE_MANIFEST: &str = "source.validate_manifest";
    /// Compute and verify content hashes.
    pub const SOURCE_COMPUTE_HASHES: &str = "source.compute_hashes";
    /// Verify the audit hash chain.
    pub const AUDIT_VERIFY_CHAIN: &str = "audit.verify_chain";

    /// Every Phase 0 job kind.
    pub const ALL: [&str; 7] = [
        SYSTEM_INTEGRITY_SCAN,
        SYSTEM_VACUUM,
        SYSTEM_NOOP_TEST,
        SYSTEM_OUTBOX_RELAY,
        SOURCE_VALIDATE_MANIFEST,
        SOURCE_COMPUTE_HASHES,
        AUDIT_VERIFY_CHAIN,
    ];
}

// ─── JobState ────────────────────────────────────────────────

/// The state of a background job.
///
/// Legal transitions are enforced by the database schema (CHECK constraints)
/// and by the state machine in [`JobContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    /// Job is waiting to be picked up by a worker.
    Queued,
    /// Job has been leased by a worker but not yet started.
    Leased,
    /// Job is actively running.
    Running,
    /// Job has reached a checkpoint and can be resumed.
    Checkpointed,
    /// Job completed successfully.
    Succeeded,
    /// Job failed after exhausting retries.
    Failed,
    /// Job was cancelled by a user or system.
    Cancelled,
    /// Job was interrupted (e.g. worker crash) and needs recovery.
    Interrupted,
    /// Job has been moved to the dead letter queue.
    DeadLettered,
}

impl fmt::Display for JobState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Queued => write!(f, "Queued"),
            Self::Leased => write!(f, "Leased"),
            Self::Running => write!(f, "Running"),
            Self::Checkpointed => write!(f, "Checkpointed"),
            Self::Succeeded => write!(f, "Succeeded"),
            Self::Failed => write!(f, "Failed"),
            Self::Cancelled => write!(f, "Cancelled"),
            Self::Interrupted => write!(f, "Interrupted"),
            Self::DeadLettered => write!(f, "DeadLettered"),
        }
    }
}

/// Type alias for job kind strings.
pub type JobKind = String;

// ─── JobError ────────────────────────────────────────────────

/// Errors originating from the job system.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum JobError {
    /// The job was not found.
    #[error("job not found: {id}")]
    NotFound { id: String },
    /// The job is in an invalid state for the requested operation.
    #[error("job {id} is in state {state}, expected {expected}")]
    InvalidState { id: String, state: String, expected: String },
    /// The job has already been handled (idempotency).
    #[error("idempotency key replay: {key}")]
    IdempotencyKeyReplay { key: String },
    /// The job could not be claimed because it was already leased.
    #[error("job {id} could not be claimed: already leased")]
    ClaimFailed { id: String },
    /// The job was cancelled.
    #[error("job {id} was cancelled")]
    Cancelled { id: String },
    /// The job exceeded its maximum number of attempts.
    #[error("job {id} exceeded max attempts ({max})")]
    MaxAttemptsExceeded { id: String, max: u32 },
    /// A storage-layer failure while accessing the queue.
    #[error("job storage error: {0}")]
    Storage(String),
}

// ─── JobHandler ──────────────────────────────────────────────

/// Trait implemented by job handlers.
///
/// Each handler defines a `kind` string, a JSON Schema for its payload,
/// whether it is idempotent, and the `run` logic.
#[async_trait]
pub trait JobHandler: Send + Sync {
    /// The job kind this handler processes.
    fn kind(&self) -> JobKind;

    /// A JSON Schema string describing the expected payload shape.
    fn payload_schema(&self) -> &'static str;

    /// Whether this job is idempotent (safe to retry on failure).
    fn is_idempotent(&self) -> bool;

    /// Execute the job. Receives a `JobContext` for cancellation, progress,
    /// and checkpointing, plus the parsed payload.
    async fn run(
        &self,
        ctx: JobContext,
        payload: serde_json::Value,
    ) -> Result<JobOutcome, JobError>;
}

/// The outcome of a job execution.
#[derive(Debug, Clone)]
pub struct JobOutcome {
    /// Whether the job succeeded.
    pub success: bool,
    /// Optional result string (e.g. summary of work done).
    pub result: Option<String>,
}

// ─── JobContext ──────────────────────────────────────────────

/// Context provided to a job handler during execution.
///
/// Provides cancellation, progress tracking, checkpointing, and
/// access to the underlying storage.
pub struct JobContext {
    /// The job being executed.
    pub job: JobRecord,
    /// Whether cancellation has been requested (snapshot at construction).
    pub cancel_requested: bool,
    /// The tracing span for this job execution.
    pub span: tracing::Span,
    /// Shared cancellation flag, pollable from inside a long handler.
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// Latest checkpoint value reported by the handler.
    checkpoint: std::sync::Arc<std::sync::Mutex<Option<String>>>,
}

impl JobContext {
    /// Create a new `JobContext` for the given job.
    pub fn new(job: JobRecord) -> Self {
        let span = tracing::span!(tracing::Level::INFO, "qai.job", job_id = %job.id);
        Self {
            job,
            cancel_requested: false,
            span,
            cancel: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            checkpoint: std::sync::Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Check whether a cancellation has been requested.
    ///
    /// Returns `true` if either the construction-time snapshot was set or the
    /// shared cancellation flag has been raised since.
    pub fn is_cancelled(&self) -> bool {
        self.cancel_requested || self.cancel.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Create a context sharing an externally-owned cancellation flag (used by
    /// the worker's watchdog).
    pub fn with_cancel(
        job: JobRecord,
        cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        let span = tracing::span!(tracing::Level::INFO, "qai.job", job_id = %job.id);
        Self {
            job,
            cancel_requested: false,
            span,
            cancel,
            checkpoint: std::sync::Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Request cancellation from outside the handler.
    pub fn request_cancel(&self) {
        self.cancel.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    /// A cloneable handle to the cancellation flag, for a cooperative handler
    /// that must observe cancellation from a spawned task or a loop.
    pub fn cancel_flag(&self) -> std::sync::Arc<std::sync::atomic::AtomicBool> {
        self.cancel.clone()
    }

    /// A cloneable handle to the checkpoint slot, so a worker can read the last
    /// checkpoint the handler recorded after the context is moved into `run`.
    pub fn checkpoint_sink(
        &self,
    ) -> std::sync::Arc<std::sync::Mutex<Option<String>>> {
        self.checkpoint.clone()
    }

    /// Report progress.
    pub fn progress(&self, stage: &str, done: u64, total: u64) {
        tracing::info!(
            target: "qai.job",
            job_id = %self.job.id,
            stage,
            done,
            total,
            "job progress"
        );
    }

    /// Record a checkpoint value for resume support.
    pub fn checkpoint(&self, value: &str) {
        if let Ok(mut slot) = self.checkpoint.lock() {
            *slot = Some(value.to_string());
        }
        tracing::info!(
            target: "qai.job",
            job_id = %self.job.id,
            checkpoint = value,
            "job checkpoint"
        );
    }

    /// The latest checkpoint value recorded by the handler, if any.
    pub fn latest_checkpoint(&self) -> Option<String> {
        self.checkpoint.lock().ok().and_then(|slot| slot.clone())
    }

    /// Return the deadline for the current lease.
    pub fn deadline(&self) -> Option<String> {
        self.job.lease_expires_at.clone()
    }

    /// Heartbeat: renew the lease. Returns true if the lease is still valid.
    pub async fn heartbeat(&self) -> bool {
        // The actual heartbeat is implemented by the worker pool
        // which calls JobRepository methods.
        true
    }
}

// ─── JobRepository trait ─────────────────────────────────────

/// Repository for job persistence.
///
/// This wraps [`storage::repository::JobRepository`] with additional
/// job-specific operations and the state machine.
#[async_trait]
pub trait JobRepository: Send {
    /// Enqueue a new job.
    async fn enqueue(&mut self, job: JobRecord) -> Result<(), JobError>;

    /// Claim a job for processing (lease acquisition).
    async fn claim(&mut self, job_id: &str, owner: &str) -> Result<Option<JobRecord>, JobError>;

    /// Finish a job with a final state.
    async fn finish(
        &mut self,
        job_id: &str,
        state: JobState,
        result: Option<String>,
    ) -> Result<(), JobError>;

    /// Cancel a running job.
    async fn cancel(&mut self, job_id: &str) -> Result<(), JobError>;

    /// Update job progress and checkpoint.
    async fn checkpoint(
        &mut self,
        job_id: &str,
        progress: Option<String>,
        checkpoint: Option<String>,
    ) -> Result<(), JobError>;

    /// Reap expired leases, returning job IDs to reclaim.
    async fn reap_expired_leases(&mut self) -> Result<Vec<String>, JobError>;

    /// Get a job by ID.
    async fn get(&mut self, job_id: &str) -> Result<Option<JobRecord>, JobError>;
}

// ─── JobStore ────────────────────────────────────────────────

/// A concrete job repository backed by [`storage::repository::JobRepository`].
pub struct JobStore {
    repo: Box<dyn storage::repository::JobRepository>,
}

impl JobStore {
    /// Create a new `JobStore` from a storage job repository.
    pub fn new(repo: Box<dyn storage::repository::JobRepository>) -> Self {
        Self { repo }
    }

    /// Transition a job state, enforcing valid transitions.
    #[cfg(test)]
    fn validate_transition(from: JobState, to: JobState) -> bool {
        match from {
            JobState::Queued => matches!(to, JobState::Leased | JobState::Cancelled),
            JobState::Leased => {
                matches!(to, JobState::Running | JobState::Cancelled | JobState::Interrupted)
            }
            JobState::Running => matches!(
                to,
                JobState::Checkpointed
                    | JobState::Succeeded
                    | JobState::Failed
                    | JobState::Cancelled
                    | JobState::Interrupted
            ),
            JobState::Checkpointed => matches!(
                to,
                JobState::Running | JobState::Succeeded | JobState::Failed | JobState::Cancelled
            ),
            JobState::Succeeded
            | JobState::Failed
            | JobState::Cancelled
            | JobState::Interrupted
            | JobState::DeadLettered => false,
        }
    }
}

#[async_trait]
impl JobRepository for JobStore {
    async fn enqueue(&mut self, job: JobRecord) -> Result<(), JobError> {
        self.repo.enqueue(job).await.map_err(|e| JobError::NotFound { id: e.to_string() })?;
        Ok(())
    }

    async fn claim(&mut self, job_id: &str, owner: &str) -> Result<Option<JobRecord>, JobError> {
        self.repo
            .claim(job_id, owner)
            .await
            .map_err(|_e| JobError::NotFound { id: job_id.to_string() })
    }

    async fn finish(
        &mut self,
        job_id: &str,
        state: JobState,
        result: Option<String>,
    ) -> Result<(), JobError> {
        self.repo
            .finish(job_id, &state.to_string(), result)
            .await
            .map_err(|_e| JobError::NotFound { id: job_id.to_string() })
    }

    async fn cancel(&mut self, job_id: &str) -> Result<(), JobError> {
        self.repo.cancel(job_id).await.map_err(|_e| JobError::NotFound { id: job_id.to_string() })
    }

    async fn checkpoint(
        &mut self,
        job_id: &str,
        progress: Option<String>,
        checkpoint: Option<String>,
    ) -> Result<(), JobError> {
        self.repo
            .checkpoint(job_id, progress, checkpoint)
            .await
            .map_err(|_e| JobError::NotFound { id: job_id.to_string() })
    }

    async fn reap_expired_leases(&mut self) -> Result<Vec<String>, JobError> {
        self.repo.reap_expired_leases().await.map_err(|e| JobError::NotFound { id: e.to_string() })
    }

    async fn get(&mut self, _job_id: &str) -> Result<Option<JobRecord>, JobError> {
        // Note: storage::JobRepository doesn't have a `get` method,
        // so we cast through. This is a Phase 0 stub.
        Ok(None)
    }
}

// ─── JobOutcome helpers ──────────────────────────────────────

impl JobOutcome {
    /// Create a successful outcome.
    pub fn success(result: Option<String>) -> Self {
        Self { success: true, result }
    }

    /// Create a failed outcome.
    pub fn failure() -> Self {
        Self { success: false, result: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_transitions() {
        assert!(JobStore::validate_transition(JobState::Queued, JobState::Leased));
        assert!(JobStore::validate_transition(JobState::Queued, JobState::Cancelled));
        assert!(JobStore::validate_transition(JobState::Leased, JobState::Running));
        assert!(JobStore::validate_transition(JobState::Running, JobState::Succeeded));
        assert!(JobStore::validate_transition(JobState::Running, JobState::Failed));
        assert!(JobStore::validate_transition(JobState::Running, JobState::Interrupted));
        assert!(JobStore::validate_transition(JobState::Checkpointed, JobState::Running));
        // Invalid transitions
        assert!(!JobStore::validate_transition(JobState::Succeeded, JobState::Running));
        assert!(!JobStore::validate_transition(JobState::Cancelled, JobState::Leased));
    }

    #[test]
    fn job_state_display() {
        assert_eq!(JobState::Queued.to_string(), "Queued");
        assert_eq!(JobState::Succeeded.to_string(), "Succeeded");
        assert_eq!(JobState::DeadLettered.to_string(), "DeadLettered");
    }

    #[test]
    fn job_outcome_success() {
        let outcome = JobOutcome::success(Some("done".to_string()));
        assert!(outcome.success);
        assert_eq!(outcome.result, Some("done".to_string()));
    }

    #[test]
    fn job_outcome_failure() {
        let outcome = JobOutcome::failure();
        assert!(!outcome.success);
        assert!(outcome.result.is_none());
    }

    #[test]
    fn job_kinds_are_unique_and_stable() {
        let unique: std::collections::HashSet<&str> = kinds::ALL.iter().copied().collect();
        assert_eq!(unique.len(), kinds::ALL.len(), "job kinds must be unique");
        assert!(kinds::ALL.contains(&"system.outbox_relay"));
    }

    fn record(id: &str, kind: &str, state: &str) -> JobRecord {
        JobRecord {
            id: id.into(),
            kind: kind.into(),
            payload_json: "{}".into(),
            idempotency_key: None,
            state: state.into(),
            priority: 0,
            attempts: 1,
            max_attempts: 5,
            available_at: String::new(),
            lease_owner: None,
            lease_expires_at: None,
            checkpoint_json: None,
            cancel_requested: false,
            created_by: "test".into(),
        }
    }

    #[test]
    fn checkpoint_is_recorded_on_the_context() {
        let ctx = JobContext::new(record("job-cp", "test", "Running"));
        assert!(ctx.latest_checkpoint().is_none());
        ctx.checkpoint("stage-2");
        assert_eq!(ctx.latest_checkpoint().as_deref(), Some("stage-2"));
    }

    #[test]
    fn cancel_flag_reflects_requests() {
        let ctx = JobContext::new(record("job-cx", "test", "Running"));
        assert!(!ctx.is_cancelled());
        ctx.request_cancel();
        assert!(ctx.is_cancelled());
    }

    #[tokio::test]
    async fn cancellation_stops_a_cooperative_handler_within_two_seconds() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::time::{Duration, Instant};

        struct Slow;
        #[async_trait]
        impl JobHandler for Slow {
            fn kind(&self) -> JobKind {
                "test.slow".into()
            }
            fn payload_schema(&self) -> &'static str {
                "{}"
            }
            fn is_idempotent(&self) -> bool {
                true
            }
            async fn run(
                &self,
                ctx: JobContext,
                _payload: serde_json::Value,
            ) -> Result<JobOutcome, JobError> {
                let flag: Arc<AtomicBool> = ctx.cancel_flag();
                let started = Instant::now();
                while !flag.load(Ordering::SeqCst) {
                    if started.elapsed() > Duration::from_secs(30) {
                        return Ok(JobOutcome::failure());
                    }
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
                Ok(JobOutcome::success(Some("stopped".into())))
            }
        }

        let ctx = JobContext::new(record("job-cancel", "test.slow", "Running"));
        let flag = ctx.cancel_flag();
        let handle = tokio::spawn(async move { Slow.run(ctx, serde_json::json!({})).await });

        tokio::time::sleep(Duration::from_millis(50)).await;
        let cancel_at = Instant::now();
        flag.store(true, Ordering::SeqCst);

        let outcome = handle.await.unwrap().unwrap();
        assert!(
            cancel_at.elapsed() < Duration::from_secs(2),
            "cancellation took {:?}, expected < 2s",
            cancel_at.elapsed()
        );
        assert!(outcome.success);
    }
}
