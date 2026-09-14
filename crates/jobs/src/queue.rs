//! Job queue abstraction for the worker pool (D0.9 / T37).
//!
//! The [`Worker`](crate::worker::Worker) operates against this trait, so the
//! same scheduling logic runs against the in-memory backend (tests) and the
//! SQLite-backed queue (production, via `application::jobs::SqliteJobQueue`).

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use storage::repository::JobRecord;

use crate::{JobError, JobState};

/// A durable, lease-based job queue.
#[async_trait]
pub trait JobQueue: Send + Sync {
    /// Enqueue a new job.
    async fn enqueue(&self, job: JobRecord) -> Result<(), JobError>;

    /// Claim the next available job for `owner`, leasing it for `lease`.
    ///
    /// Considers `Queued`, `Interrupted`, and `Checkpointed` jobs whose
    /// `available_at` has passed.
    async fn claim_next(
        &self,
        owner: &str,
        lease: Duration,
    ) -> Result<Option<JobRecord>, JobError>;

    /// Renew the lease if `owner` still holds it. Returns whether it is held.
    async fn heartbeat(
        &self,
        job_id: &str,
        owner: &str,
        lease: Duration,
    ) -> Result<bool, JobError>;

    /// Record a progress update and/or checkpoint.
    async fn checkpoint(
        &self,
        job_id: &str,
        progress: Option<String>,
        checkpoint: Option<String>,
    ) -> Result<(), JobError>;

    /// Transition a job to a terminal/recorded state.
    async fn finish(
        &self,
        job_id: &str,
        state: JobState,
        result: Option<String>,
    ) -> Result<(), JobError>;

    /// Re-queue a job after `delay` (retry/backoff).
    async fn reschedule(
        &self,
        job_id: &str,
        delay: Duration,
        reason: Option<String>,
    ) -> Result<(), JobError>;

    /// Whether cancellation has been requested.
    async fn cancel_requested(&self, job_id: &str) -> Result<bool, JobError>;

    /// Fetch a job by id.
    async fn get(&self, job_id: &str) -> Result<Option<JobRecord>, JobError>;

    /// Reap expired leases (`Running` → `Interrupted`), returning the ids.
    async fn reap_expired(&self) -> Result<Vec<String>, JobError>;

    /// Number of jobs waiting to run.
    async fn queued_count(&self) -> Result<u64, JobError>;
}

fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

fn plus(d: Duration) -> String {
    let at = time::OffsetDateTime::now_utc() + time::Duration::seconds(d.as_secs() as i64);
    at.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

/// An in-memory [`JobQueue`] for tests and local/deterministic runs.
#[derive(Default)]
pub struct InMemoryJobQueue {
    jobs: Mutex<HashMap<String, JobRecord>>,
    order: Mutex<Vec<String>>,
}

impl InMemoryJobQueue {
    /// Create an empty queue.
    pub fn new() -> Self {
        Self::default()
    }

    fn with_job<R>(&self, id: &str, f: impl FnOnce(&mut JobRecord) -> R) -> Option<R> {
        let mut jobs = self.jobs.lock().unwrap();
        jobs.get_mut(id).map(f)
    }
}

#[async_trait]
impl JobQueue for InMemoryJobQueue {
    async fn enqueue(&self, job: JobRecord) -> Result<(), JobError> {
        let mut order = self.order.lock().unwrap();
        let mut jobs = self.jobs.lock().unwrap();
        if jobs.contains_key(&job.id) {
            return Err(JobError::IdempotencyKeyReplay { key: job.id.clone() });
        }
        order.push(job.id.clone());
        jobs.insert(job.id.clone(), job);
        Ok(())
    }

    async fn claim_next(
        &self,
        owner: &str,
        lease: Duration,
    ) -> Result<Option<JobRecord>, JobError> {
        let now = now_rfc3339();
        let order = self.order.lock().unwrap().clone();
        let mut jobs = self.jobs.lock().unwrap();
        for id in order {
            let Some(job) = jobs.get_mut(&id) else { continue };
            let claimable = matches!(
                job.state.as_str(),
                "Queued" | "Interrupted" | "Checkpointed"
            ) && job.available_at <= now;
            if claimable {
                job.state = "Running".to_string();
                job.lease_owner = Some(owner.to_string());
                job.lease_expires_at = Some(plus(lease));
                job.attempts += 1;
                return Ok(Some(job.clone()));
            }
        }
        Ok(None)
    }

    async fn heartbeat(
        &self,
        job_id: &str,
        owner: &str,
        lease: Duration,
    ) -> Result<bool, JobError> {
        let held = self
            .with_job(job_id, |job| {
                if job.lease_owner.as_deref() == Some(owner) && job.state == "Running" {
                    job.lease_expires_at = Some(plus(lease));
                    true
                } else {
                    false
                }
            })
            .unwrap_or(false);
        Ok(held)
    }

    async fn checkpoint(
        &self,
        job_id: &str,
        progress: Option<String>,
        checkpoint: Option<String>,
    ) -> Result<(), JobError> {
        self.with_job(job_id, |job| {
            if progress.is_some() {
                job.checkpoint_json = checkpoint.or_else(|| job.checkpoint_json.clone());
            } else if checkpoint.is_some() {
                job.checkpoint_json = checkpoint;
            }
        });
        Ok(())
    }

    async fn finish(
        &self,
        job_id: &str,
        state: JobState,
        _result: Option<String>,
    ) -> Result<(), JobError> {
        self.with_job(job_id, |job| {
            job.state = state.to_string();
            job.lease_owner = None;
            job.lease_expires_at = None;
        });
        Ok(())
    }

    async fn reschedule(
        &self,
        job_id: &str,
        delay: Duration,
        _reason: Option<String>,
    ) -> Result<(), JobError> {
        self.with_job(job_id, |job| {
            job.state = JobState::Queued.to_string();
            job.lease_owner = None;
            job.lease_expires_at = None;
            job.available_at = plus(delay);
        });
        Ok(())
    }

    async fn cancel_requested(&self, job_id: &str) -> Result<bool, JobError> {
        Ok(self.with_job(job_id, |job| job.cancel_requested).unwrap_or(false))
    }

    async fn get(&self, job_id: &str) -> Result<Option<JobRecord>, JobError> {
        Ok(self.jobs.lock().unwrap().get(job_id).cloned())
    }

    async fn reap_expired(&self) -> Result<Vec<String>, JobError> {
        let now = now_rfc3339();
        let order = self.order.lock().unwrap().clone();
        let mut jobs = self.jobs.lock().unwrap();
        let mut reaped = Vec::new();
        for id in order {
            if let Some(job) = jobs.get_mut(&id)
                && job.state == "Running"
                && job.lease_expires_at.as_deref().is_some_and(|t| t < now.as_str())
            {
                job.state = "Interrupted".to_string();
                job.lease_owner = None;
                job.lease_expires_at = None;
                reaped.push(id);
            }
        }
        Ok(reaped)
    }

    async fn queued_count(&self) -> Result<u64, JobError> {
        Ok(self
            .jobs
            .lock()
            .unwrap()
            .values()
            .filter(|j| j.state == "Queued")
            .count() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::record;

    #[tokio::test]
    async fn claim_marks_running_and_increments_attempts() {
        let q = InMemoryJobQueue::new();
        q.enqueue(record("j1", "system.noop_test", "Queued")).await.unwrap();
        let claimed = q.claim_next("w1", Duration::from_secs(60)).await.unwrap().unwrap();
        assert_eq!(claimed.state, "Running");
        assert_eq!(claimed.attempts, 1);
        assert_eq!(claimed.lease_owner.as_deref(), Some("w1"));
        // A second worker cannot claim it.
        assert!(q.claim_next("w2", Duration::from_secs(60)).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn heartbeat_only_for_the_holder() {
        let q = InMemoryJobQueue::new();
        q.enqueue(record("j2", "system.noop_test", "Queued")).await.unwrap();
        q.claim_next("w1", Duration::from_secs(60)).await.unwrap();
        assert!(q.heartbeat("j2", "w1", Duration::from_secs(60)).await.unwrap());
        assert!(!q.heartbeat("j2", "w2", Duration::from_secs(60)).await.unwrap());
    }
}
