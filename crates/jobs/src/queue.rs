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
    async fn claim_next(&self, owner: &str, lease: Duration)
    -> Result<Option<JobRecord>, JobError>;

    /// Renew the lease if `owner` still holds it. Returns whether it is held.
    async fn heartbeat(&self, job_id: &str, owner: &str, lease: Duration)
    -> Result<bool, JobError>;

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

    /// Persist a cooperative cancellation request (D-16).
    ///
    /// Sets only the request flag; never clears the lease owner/expiry, so a
    /// live worker keeps its lease and observes the request at its next
    /// checkpoint. Accepted only for non-terminal jobs
    /// (`Queued`/`Leased`/`Running`/`Checkpointed`/`Interrupted`).
    async fn request_cancel(&self, job_id: &str) -> Result<(), JobError>;

    /// Explicit operator retry after exhaustion (D-15).
    ///
    /// Accepted only for `Failed`/`DeadLettered`/`Interrupted` jobs. Resets
    /// the attempt/availability fields (`Queued`, attempts 0, due now, lease
    /// cleared) while preserving the last committed `checkpoint_json` so the
    /// retried run resumes after its boundary. Automatic retries never touch
    /// non-idempotent jobs; this explicit path is the only way back.
    async fn retry(&self, job_id: &str) -> Result<(), JobError>;

    /// Record a progress update and/or checkpoint when `owner` holds the
    /// current lease (T-03-LEASE).
    ///
    /// Returns whether a row was affected: a caller that does not hold the
    /// lease changes nothing.
    async fn checkpoint_owned(
        &self,
        job_id: &str,
        owner: &str,
        progress: Option<String>,
        checkpoint: Option<String>,
    ) -> Result<bool, JobError>;

    /// Transition a job to a terminal/recorded state when `owner` holds the
    /// current lease (T-03-LEASE). Returns whether a row was affected.
    async fn finish_owned(
        &self,
        job_id: &str,
        owner: &str,
        state: JobState,
        result: Option<String>,
    ) -> Result<bool, JobError>;

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
    let at = time::OffsetDateTime::now_utc()
        + time::Duration::seconds(d.as_secs() as i64)
        + time::Duration::nanoseconds(d.subsec_nanos() as i64);
    at.format(&time::format_description::well_known::Rfc3339).unwrap_or_default()
}

fn rfc3339_le(a: &str, b: &str) -> bool {
    let parse = |s: &str| {
        time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
            .map(|dt| dt.unix_timestamp_nanos())
    };
    match (parse(a), parse(b)) {
        (Ok(a), Ok(b)) => a <= b,
        _ => false,
    }
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

    fn claim_next_at(
        &self,
        owner: &str,
        lease: Duration,
        now: &str,
    ) -> Result<Option<JobRecord>, JobError> {
        let order = self.order.lock().unwrap().clone();
        let mut jobs = self.jobs.lock().unwrap();
        for id in order {
            let Some(job) = jobs.get_mut(&id) else { continue };
            let claimable = matches!(job.state.as_str(), "Queued" | "Interrupted" | "Checkpointed")
                && rfc3339_le(&job.available_at, now);
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

    fn reap_expired_at(&self, now: &str) -> Vec<String> {
        let order = self.order.lock().unwrap().clone();
        let mut jobs = self.jobs.lock().unwrap();
        let mut reaped = Vec::new();
        for id in order {
            if let Some(job) = jobs.get_mut(&id)
                && job.state == "Running"
                && job.lease_expires_at.as_deref().is_some_and(|t| rfc3339_le(t, now))
            {
                job.state = "Interrupted".to_string();
                job.lease_owner = None;
                job.lease_expires_at = None;
                reaped.push(id);
            }
        }
        reaped
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
        self.claim_next_at(owner, lease, &now_rfc3339())
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

    async fn request_cancel(&self, job_id: &str) -> Result<(), JobError> {
        let outcome = self.with_job(job_id, |job| {
            let eligible = matches!(
                job.state.as_str(),
                "Queued" | "Leased" | "Running" | "Checkpointed" | "Interrupted"
            );
            if eligible {
                // Request-only: the live lease (owner/expiry) is preserved so
                // the owning worker observes the request cooperatively.
                job.cancel_requested = true;
            }
            eligible
        });
        match outcome {
            Some(true) => Ok(()),
            Some(false) => {
                let state = self.get(job_id).await?.map(|j| j.state).unwrap_or_default();
                Err(JobError::InvalidState {
                    id: job_id.to_string(),
                    state,
                    expected: "a non-terminal state".to_string(),
                })
            }
            None => Err(JobError::NotFound { id: job_id.to_string() }),
        }
    }

    async fn retry(&self, job_id: &str) -> Result<(), JobError> {
        let outcome = self.with_job(job_id, |job| {
            let eligible = matches!(job.state.as_str(), "Failed" | "DeadLettered" | "Interrupted");
            if eligible {
                // Explicit operator retry: fresh attempt window, due now, no
                // lease — but the last committed checkpoint is preserved so
                // the run resumes after its boundary (D-14).
                job.state = JobState::Queued.to_string();
                job.attempts = 0;
                job.available_at = now_rfc3339();
                job.lease_owner = None;
                job.lease_expires_at = None;
                job.cancel_requested = false;
            }
            eligible
        });
        match outcome {
            Some(true) => Ok(()),
            Some(false) => {
                let state = self.get(job_id).await?.map(|j| j.state).unwrap_or_default();
                Err(JobError::InvalidState {
                    id: job_id.to_string(),
                    state,
                    expected: "Failed, DeadLettered, or Interrupted".to_string(),
                })
            }
            None => Err(JobError::NotFound { id: job_id.to_string() }),
        }
    }

    async fn checkpoint_owned(
        &self,
        job_id: &str,
        owner: &str,
        progress: Option<String>,
        checkpoint: Option<String>,
    ) -> Result<bool, JobError> {
        Ok(self
            .with_job(job_id, |job| {
                let holds = job.state == "Running" && job.lease_owner.as_deref() == Some(owner);
                if holds {
                    // Only provided columns move; a `None` never clears a
                    // committed boundary.
                    if progress.is_some() {
                        job.checkpoint_json = checkpoint.or_else(|| job.checkpoint_json.clone());
                    } else if checkpoint.is_some() {
                        job.checkpoint_json = checkpoint;
                    }
                }
                holds
            })
            .unwrap_or(false))
    }

    async fn finish_owned(
        &self,
        job_id: &str,
        owner: &str,
        state: JobState,
        _result: Option<String>,
    ) -> Result<bool, JobError> {
        Ok(self
            .with_job(job_id, |job| {
                let holds = job.state == "Running" && job.lease_owner.as_deref() == Some(owner);
                if holds {
                    job.state = state.to_string();
                    job.lease_owner = None;
                    job.lease_expires_at = None;
                }
                holds
            })
            .unwrap_or(false))
    }

    async fn get(&self, job_id: &str) -> Result<Option<JobRecord>, JobError> {
        Ok(self.jobs.lock().unwrap().get(job_id).cloned())
    }

    async fn reap_expired(&self) -> Result<Vec<String>, JobError> {
        Ok(self.reap_expired_at(&now_rfc3339()))
    }

    async fn queued_count(&self) -> Result<u64, JobError> {
        Ok(self.jobs.lock().unwrap().values().filter(|j| j.state == "Queued").count() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::record;

    #[test]
    fn timestamp_comparison_handles_mixed_fraction_digits() {
        assert!(rfc3339_le("2026-01-01T00:00:00.123Z", "2026-01-01T00:00:00.123456Z"));
        assert!(!rfc3339_le("2026-01-01T00:00:00.123456Z", "2026-01-01T00:00:00.123Z"));
        assert!(rfc3339_le("2026-01-01T00:00:00.123000Z", "2026-01-01T00:00:00.123Z"));
        assert!(rfc3339_le("2026-01-01T00:00:00.123Z", "2026-01-01T00:00:00.123000Z"));
        assert!(rfc3339_le("2026-01-01T00:00:00.122999Z", "2026-01-01T00:00:00.123Z"));
        assert!(!rfc3339_le("2026-01-01T00:00:00.123001Z", "2026-01-01T00:00:00.123Z"));
        assert!(rfc3339_le("2026-01-01T00:00:00Z", "2026-01-01T00:00:00.000000Z"));
        assert!(rfc3339_le("2026-01-01T00:00:00.000000Z", "2026-01-01T00:00:00Z"));
        assert!(rfc3339_le("2026-01-01T00:00:00.999999Z", "2026-01-01T00:00:01Z"));
        assert!(!rfc3339_le("2026-01-01T00:00:01Z", "2026-01-01T00:00:00.999999Z"));
        assert!(!rfc3339_le("not-a-time", "2026-01-01T00:00:00Z"));
        assert!(!rfc3339_le("2026-01-01T00:00:00Z", "not-a-time"));
    }

    #[test]
    fn plus_preserves_subsecond_delays() {
        // `plus` must not truncate sub-second precision: a 500 ms delay must
        // land ~0.5 s in the future, not at the current second.
        let before = time::OffsetDateTime::now_utc();
        let stamped = plus(Duration::from_millis(500));
        let at =
            time::OffsetDateTime::parse(&stamped, &time::format_description::well_known::Rfc3339)
                .expect("plus must emit parseable RFC3339");
        let delta = (at - before).as_seconds_f64();
        assert!(
            (0.4..1.5).contains(&delta),
            "500 ms delay produced {delta:.3}s offset ({stamped})"
        );

        // Whole-second delays keep working.
        let stamped = plus(Duration::from_secs(60));
        let at =
            time::OffsetDateTime::parse(&stamped, &time::format_description::well_known::Rfc3339)
                .expect("plus must emit parseable RFC3339");
        let delta = (at - before).as_seconds_f64();
        assert!(
            (59.0..61.5).contains(&delta),
            "60 s delay produced {delta:.3}s offset ({stamped})"
        );
    }

    #[tokio::test]
    async fn mixed_precision_timestamps_control_claims_and_reaping() {
        let now = "2026-01-01T00:00:00.123456Z";
        for (timestamp, eligible) in [
            ("2026-01-01T00:00:00.123Z", true),
            ("2026-01-01T00:00:00.123456000Z", true),
            ("2026-01-01T00:00:00.123457Z", false),
            ("not-a-time", false),
        ] {
            let q = InMemoryJobQueue::new();
            let mut job = record("claim", "system.noop_test", "Queued");
            job.available_at = timestamp.into();
            q.enqueue(job).await.unwrap();
            let claimed = q.claim_next_at("worker", Duration::from_secs(60), now).unwrap();
            assert_eq!(claimed.is_some(), eligible, "claim {timestamp}");
            assert_eq!(q.get("claim").await.unwrap().unwrap().attempts, u32::from(eligible));

            let mut job = record("lease", "system.noop_test", "Running");
            job.lease_owner = Some("worker".into());
            job.lease_expires_at = Some(timestamp.into());
            q.enqueue(job).await.unwrap();
            assert_eq!(q.reap_expired_at(now).contains(&"lease".to_string()), eligible);
            let job = q.get("lease").await.unwrap().unwrap();
            assert_eq!(job.state, if eligible { "Interrupted" } else { "Running" });
            assert_eq!(job.lease_owner.is_none(), eligible);
        }
    }

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

    /// 01-03-01 contract: a named checkpoint recorded through the owned
    /// operation is immediately readable from the stored row.
    #[tokio::test]
    async fn named_checkpoint_is_immediately_visible_to_the_next_reader() {
        use crate::NamedCheckpoint;
        let q = InMemoryJobQueue::new();
        q.enqueue(record("j-ncp", "system.noop_test", "Queued")).await.unwrap();
        q.claim_next("w1", Duration::from_secs(60)).await.unwrap();

        let stored =
            NamedCheckpoint::new("stage.done", serde_json::json!({"n": 3})).to_json_string();
        assert!(q.checkpoint_owned("j-ncp", "w1", None, Some(stored.clone())).await.unwrap());
        // Readable from the committed row before the handler continues.
        let row = q.get("j-ncp").await.unwrap().unwrap();
        assert_eq!(row.checkpoint_json.as_deref(), Some(stored.as_str()));
        let named = NamedCheckpoint::parse(row.checkpoint_json.as_deref().unwrap()).unwrap();
        assert_eq!(named.name, "stage.done");
        // A reclaim (Interrupted → Running) carries the boundary forward.
        q.reap_expired_at("2999-01-01T00:00:00Z");
        let row = q.get("j-ncp").await.unwrap().unwrap();
        assert_eq!(row.state, "Interrupted");
        assert_eq!(row.checkpoint_json.as_deref(), Some(stored.as_str()));
    }

    /// 01-03-01 contract: explicit retry eligibility per state.
    #[tokio::test]
    async fn explicit_retry_accepts_only_eligible_states() {
        let q = InMemoryJobQueue::new();
        for (id, state) in
            [("r-failed", "Failed"), ("r-dead", "DeadLettered"), ("r-inter", "Interrupted")]
        {
            let mut job = record(id, "system.noop_test", state);
            job.attempts = 5;
            job.cancel_requested = true;
            job.checkpoint_json = Some(r#"{"version":1,"name":"b","payload":{}}"#.into());
            q.enqueue(job).await.unwrap();
            q.retry(id).await.unwrap();
            let row = q.get(id).await.unwrap().unwrap();
            assert_eq!(row.state, "Queued", "{id}");
            assert_eq!(row.attempts, 0, "{id}: explicit retry resets the attempt window");
            assert!(row.lease_owner.is_none() && row.lease_expires_at.is_none(), "{id}");
            assert!(!row.cancel_requested, "{id}");
            assert!(row.checkpoint_json.is_some(), "{id}: boundary preserved for resume");
        }
        for (id, state) in [("r-queued", "Queued"), ("r-run", "Running"), ("r-ok", "Succeeded")] {
            q.enqueue(record(id, "system.noop_test", state)).await.unwrap();
            let err = q.retry(id).await.unwrap_err();
            assert!(matches!(err, JobError::InvalidState { .. }), "{id}: {err:?}");
            assert_eq!(q.get(id).await.unwrap().unwrap().state, state, "{id} unchanged");
        }
        assert!(matches!(q.retry("missing").await.unwrap_err(), JobError::NotFound { .. }));
    }

    /// 01-03-01 contract: cancellation is request-only and owner-safe.
    #[tokio::test]
    async fn cancel_request_preserves_the_live_lease() {
        let q = InMemoryJobQueue::new();
        q.enqueue(record("j-cx", "system.noop_test", "Queued")).await.unwrap();
        q.claim_next("w1", Duration::from_secs(60)).await.unwrap();

        q.request_cancel("j-cx").await.unwrap();
        assert!(q.cancel_requested("j-cx").await.unwrap());
        let row = q.get("j-cx").await.unwrap().unwrap();
        assert_eq!(row.state, "Running");
        assert_eq!(row.lease_owner.as_deref(), Some("w1"));
        assert!(row.lease_expires_at.is_some(), "live lease expiry preserved");

        // Terminal jobs reject the request without mutation.
        q.enqueue(record("j-done", "system.noop_test", "Succeeded")).await.unwrap();
        let err = q.request_cancel("j-done").await.unwrap_err();
        assert!(matches!(err, JobError::InvalidState { .. }));
        assert!(!q.cancel_requested("j-done").await.unwrap());
        assert!(matches!(
            q.request_cancel("missing").await.unwrap_err(),
            JobError::NotFound { .. }
        ));
    }

    /// 01-03-01 contract: non-owner terminal/checkpoint writes affect zero rows.
    #[tokio::test]
    async fn non_owner_writes_change_no_row() {
        let q = InMemoryJobQueue::new();
        q.enqueue(record("j-own", "system.noop_test", "Queued")).await.unwrap();
        q.claim_next("w1", Duration::from_secs(60)).await.unwrap();

        assert!(!q.heartbeat("j-own", "w2", Duration::from_secs(60)).await.unwrap());
        assert!(!q.checkpoint_owned("j-own", "w2", None, Some("x".into())).await.unwrap());
        assert!(!q.finish_owned("j-own", "w2", JobState::Succeeded, None).await.unwrap());
        let row = q.get("j-own").await.unwrap().unwrap();
        assert_eq!(row.state, "Running");
        assert_eq!(row.lease_owner.as_deref(), Some("w1"));
        assert!(row.checkpoint_json.is_none());

        // The holder's writes still apply.
        assert!(q.checkpoint_owned("j-own", "w1", None, Some("cp".into())).await.unwrap());
        assert!(q.finish_owned("j-own", "w1", JobState::Succeeded, None).await.unwrap());
        assert_eq!(q.get("j-own").await.unwrap().unwrap().state, "Succeeded");
    }
}
