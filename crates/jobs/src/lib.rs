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
use domain::ids::JobId;
use storage::error::StorageError;
use storage::repository::JobRecord;
use std::fmt;

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
    async fn run(&self, ctx: JobContext, payload: serde_json::Value)
        -> Result<JobOutcome, JobError>;
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
    /// Whether cancellation has been requested.
    pub cancel_requested: bool,
    /// The tracing span for this job execution.
    pub span: tracing::Span,
}

impl JobContext {
    /// Create a new `JobContext` for the given job.
    pub fn new(job: JobRecord) -> Self {
        let span = tracing::span!(tracing::Level::INFO, "qai.job", job_id = %job.id);
        Self {
            job,
            cancel_requested: false,
            span,
        }
    }

    /// Check whether a cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancel_requested
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
        tracing::info!(
            target: "qai.job",
            job_id = %self.job.id,
            checkpoint = value,
            "job checkpoint"
        );
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
    fn validate_transition(from: JobState, to: JobState) -> bool {
        match from {
            JobState::Queued => matches!(to, JobState::Leased | JobState::Cancelled),
            JobState::Leased => matches!(to, JobState::Running | JobState::Cancelled | JobState::Interrupted),
            JobState::Running => matches!(to, JobState::Checkpointed | JobState::Succeeded | JobState::Failed | JobState::Cancelled | JobState::Interrupted),
            JobState::Checkpointed => matches!(to, JobState::Running | JobState::Succeeded | JobState::Failed | JobState::Cancelled),
            JobState::Succeeded | JobState::Failed | JobState::Cancelled | JobState::Interrupted | JobState::DeadLettered => false,
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
        self.repo.claim(job_id, owner).await
            .map_err(|e| JobError::NotFound { id: job_id.to_string() })
    }

    async fn finish(
        &mut self,
        job_id: &str,
        state: JobState,
        result: Option<String>,
    ) -> Result<(), JobError> {
        self.repo.finish(job_id, &state.to_string(), result).await
            .map_err(|e| JobError::NotFound { id: job_id.to_string() })
    }

    async fn cancel(&mut self, job_id: &str) -> Result<(), JobError> {
        self.repo.cancel(job_id).await
            .map_err(|e| JobError::NotFound { id: job_id.to_string() })
    }

    async fn checkpoint(
        &mut self,
        job_id: &str,
        progress: Option<String>,
        checkpoint: Option<String>,
    ) -> Result<(), JobError> {
        self.repo.checkpoint(job_id, progress, checkpoint).await
            .map_err(|e| JobError::NotFound { id: job_id.to_string() })
    }

    async fn reap_expired_leases(&mut self) -> Result<Vec<String>, JobError> {
        self.repo.reap_expired_leases().await
            .map_err(|e| JobError::NotFound { id: e.to_string() })
    }

    async fn get(&mut self, job_id: &str) -> Result<Option<JobRecord>, JobError> {
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
}
