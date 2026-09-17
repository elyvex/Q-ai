//! In-process worker pool (D0.9 / T37–T39).
//!
//! A [`Worker`] claims jobs from a [`JobQueue`], runs the registered handler
//! with cancellation + heartbeat + checkpointing, and applies the retry /
//! backoff / dead-letter policy.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde_json::Value;
use tracing::{info, warn};

use crate::queue::JobQueue;
use crate::registry::HandlerRegistry;
use crate::{JobContext, JobError, JobState};

/// Retry/backoff/lease policy for a worker.
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Lease duration granted on claim and renewed by the heartbeat.
    pub lease: Duration,
    /// Base of the exponential backoff.
    pub backoff_base: Duration,
    /// Cap on the backoff.
    pub backoff_max: Duration,
    /// Jitter fraction (0.0–1.0) applied to the backoff.
    pub backoff_jitter: f64,
    /// Maximum attempts before dead-lettering.
    pub max_attempts: u32,
    /// How often the watchdog checks cancellation and renews the lease.
    pub poll_interval: Duration,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            lease: Duration::from_secs(60),
            backoff_base: Duration::from_millis(500),
            backoff_max: Duration::from_secs(60),
            backoff_jitter: 0.2,
            max_attempts: 5,
            poll_interval: Duration::from_millis(25),
        }
    }
}

/// The result of processing a single job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerOutcome {
    /// Nothing was available to run.
    Idle,
    /// The job succeeded.
    Succeeded { job_id: String },
    /// The job failed and was scheduled for retry.
    Retried { job_id: String, next_delay_ms: u64 },
    /// The job failed and exhausted its attempts.
    DeadLettered { job_id: String },
    /// The job was cancelled.
    Cancelled { job_id: String },
    /// The job kind had no registered handler.
    UnknownKind { job_id: String, kind: String },
}

/// An in-process worker that drains a [`JobQueue`].
pub struct Worker {
    queue: Arc<dyn JobQueue>,
    registry: Arc<HandlerRegistry>,
    owner: String,
    config: WorkerConfig,
}

impl Worker {
    /// Create a worker with the default policy.
    pub fn new(
        queue: Arc<dyn JobQueue>,
        registry: Arc<HandlerRegistry>,
        owner: impl Into<String>,
    ) -> Self {
        Self { queue, registry, owner: owner.into(), config: WorkerConfig::default() }
    }

    /// Override the retry/backoff/lease policy.
    pub fn with_config(mut self, config: WorkerConfig) -> Self {
        self.config = config;
        self
    }

    /// Recover interrupted runs (expired leases) before processing. Returns the
    /// number of jobs reclaimed.
    pub async fn recover_interrupted(&self) -> Result<usize, JobError> {
        let reaped = self.queue.reap_expired().await?;
        if !reaped.is_empty() {
            info!(count = reaped.len(), "reclaimed interrupted jobs");
        }
        Ok(reaped.len())
    }

    /// Claim and process at most one job.
    pub async fn run_once(&self) -> Result<WorkerOutcome, JobError> {
        let Some(job) = self.queue.claim_next(&self.owner, self.config.lease).await? else {
            return Ok(WorkerOutcome::Idle);
        };

        let Some(handler) = self.registry.get(&job.kind) else {
            warn!(job_id = %job.id, kind = %job.kind, "no handler registered; dead-lettering");
            self.queue
                .finish(&job.id, JobState::DeadLettered, Some("unknown job kind".into()))
                .await?;
            return Ok(WorkerOutcome::UnknownKind { job_id: job.id, kind: job.kind });
        };

        // Payload schema validation (fail closed).
        let payload: Value = serde_json::from_str(&job.payload_json).unwrap_or(Value::Null);
        if !validate_payload(handler.payload_schema(), &payload) {
            self.queue
                .finish(&job.id, JobState::DeadLettered, Some("invalid payload".into()))
                .await?;
            return Ok(WorkerOutcome::DeadLettered { job_id: job.id });
        }

        // Already cancelled before we started.
        if self.queue.cancel_requested(&job.id).await? {
            self.queue.finish(&job.id, JobState::Cancelled, None).await?;
            return Ok(WorkerOutcome::Cancelled { job_id: job.id });
        }

        // Watchdog: poll cancellation + renew the lease while the handler runs.
        let cancel = Arc::new(AtomicBool::new(false));
        let ctx = JobContext::with_cancel(job.clone(), cancel.clone());
        let checkpoint_sink = ctx.checkpoint_sink();
        let watchdog = tokio::spawn(watchdog(
            self.queue.clone(),
            job.id.clone(),
            self.owner.clone(),
            cancel.clone(),
            self.config.lease,
            self.config.poll_interval,
        ));

        let result = handler.run(ctx, payload).await;
        watchdog.abort();

        match result {
            Ok(outcome) if outcome.success => {
                self.queue.finish(&job.id, JobState::Succeeded, outcome.result).await?;
                Ok(WorkerOutcome::Succeeded { job_id: job.id })
            }
            Ok(_) | Err(_) => {
                // Cooperative cancellation takes precedence over failure.
                if cancel.load(Ordering::SeqCst) {
                    self.queue.finish(&job.id, JobState::Cancelled, None).await?;
                    return Ok(WorkerOutcome::Cancelled { job_id: job.id });
                }
                // Persist any checkpoint the handler recorded before failing.
                let recorded = checkpoint_sink.lock().ok().and_then(|slot| slot.clone());
                if let Some(cp) = recorded {
                    self.queue.checkpoint(&job.id, None, Some(cp)).await?;
                }
                if !handler.is_idempotent() {
                    self.queue
                        .finish(
                            &job.id,
                            JobState::DeadLettered,
                            Some("non-idempotent job cannot be retried automatically".into()),
                        )
                        .await?;
                    return Ok(WorkerOutcome::DeadLettered { job_id: job.id });
                }
                self.handle_failure(&job).await
            }
        }
    }

    /// Process jobs until the queue is empty. Returns the number processed.
    pub async fn run_until_idle(&self) -> Result<u32, JobError> {
        let mut processed = 0;
        loop {
            match self.run_once().await? {
                WorkerOutcome::Idle => break,
                _ => processed += 1,
            }
        }
        Ok(processed)
    }

    async fn handle_failure(
        &self,
        job: &storage::repository::JobRecord,
    ) -> Result<WorkerOutcome, JobError> {
        let attempts = job.attempts.max(1);
        if attempts >= self.config.max_attempts {
            self.queue
                .finish(&job.id, JobState::DeadLettered, Some("max attempts exceeded".into()))
                .await?;
            Ok(WorkerOutcome::DeadLettered { job_id: job.id.clone() })
        } else {
            let delay = self.backoff(attempts, &job.id);
            self.queue.reschedule(&job.id, delay, Some("retry".into())).await?;
            Ok(WorkerOutcome::Retried {
                job_id: job.id.clone(),
                next_delay_ms: delay.as_millis() as u64,
            })
        }
    }

    /// Exponential backoff with deterministic jitter (no `rand` dependency).
    fn backoff(&self, attempts: u32, job_id: &str) -> Duration {
        let shift = attempts.saturating_sub(1).min(16);
        let exp =
            self.config.backoff_base.saturating_mul(1u32 << shift).min(self.config.backoff_max);
        // Deterministic 0..1 fraction from the job id.
        let mut hash: u64 = 1469598103934665603;
        for b in job_id.bytes() {
            hash ^= b as u64;
            hash = hash.wrapping_mul(1099511628211);
        }
        let frac = (hash % 1000) as f64 / 1000.0;
        let factor = 1.0 + self.config.backoff_jitter * (frac * 2.0 - 1.0);
        Duration::from_secs_f64((exp.as_secs_f64() * factor).max(0.0)).min(self.config.backoff_max)
    }
}

async fn watchdog(
    queue: Arc<dyn JobQueue>,
    job_id: String,
    owner: String,
    cancel: Arc<AtomicBool>,
    lease: Duration,
    poll: Duration,
) {
    loop {
        tokio::time::sleep(poll).await;
        if queue.cancel_requested(&job_id).await.unwrap_or(false) {
            cancel.store(true, Ordering::SeqCst);
        }
        let _ = queue.heartbeat(&job_id, &owner, lease).await;
    }
}

/// Validate a payload against a minimal JSON Schema subset.
///
/// Supports `type` (`object`/`array`/`string`/`number`/`integer`/`boolean`),
/// `required` (property names), and `properties[*].type`. Unknown keywords are
/// ignored. Fails closed: an unparsable schema rejects the payload.
pub fn validate_payload(schema: &str, payload: &Value) -> bool {
    let Ok(schema) = serde_json::from_str::<Value>(schema) else {
        return false;
    };
    validate_against(&schema, payload)
}

fn validate_against(schema: &Value, value: &Value) -> bool {
    if let Some(expected) = schema.get("type").and_then(|t| t.as_str()) {
        let ok = match expected {
            "object" => value.is_object(),
            "array" => value.is_array(),
            "string" => value.is_string(),
            "number" => value.is_number(),
            "integer" => value.is_i64() || value.is_u64(),
            "boolean" => value.is_boolean(),
            "null" => value.is_null(),
            _ => true,
        };
        if !ok {
            return false;
        }
    }
    if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
        let Some(obj) = value.as_object() else {
            return required.is_empty();
        };
        for key in required {
            if let Some(k) = key.as_str()
                && !obj.contains_key(k)
            {
                return false;
            }
        }
    }
    if let (Some(props), Some(obj)) =
        (schema.get("properties").and_then(|p| p.as_object()), value.as_object())
    {
        for (k, subschema) in props {
            if let Some(v) = obj.get(k)
                && !validate_against(subschema, v)
            {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinds;
    use crate::queue::InMemoryJobQueue;
    use crate::registry::HandlerRegistry;
    use crate::test_support::record;
    use crate::{JobContext, JobError, JobHandler, JobKind, JobOutcome};
    use std::sync::atomic::{AtomicU32, Ordering as AtomicOrdering};

    struct Succeed;
    #[async_trait::async_trait]
    impl JobHandler for Succeed {
        fn kind(&self) -> JobKind {
            kinds::SYSTEM_NOOP_TEST.into()
        }
        fn payload_schema(&self) -> &'static str {
            r#"{"type":"object","required":["n"],"properties":{"n":{"type":"integer"}}}"#
        }
        fn is_idempotent(&self) -> bool {
            true
        }
        async fn run(&self, _ctx: JobContext, _p: Value) -> Result<JobOutcome, JobError> {
            Ok(JobOutcome::success(Some("ok".into())))
        }
    }

    struct FailTimes {
        remaining: AtomicU32,
    }
    #[async_trait::async_trait]
    impl JobHandler for FailTimes {
        fn kind(&self) -> JobKind {
            "test.flaky".into()
        }
        fn payload_schema(&self) -> &'static str {
            r#"{"type":"object"}"#
        }
        fn is_idempotent(&self) -> bool {
            true
        }
        async fn run(&self, _ctx: JobContext, _p: Value) -> Result<JobOutcome, JobError> {
            if self.remaining.fetch_sub(1, AtomicOrdering::SeqCst) > 0 {
                Ok(JobOutcome::failure())
            } else {
                Ok(JobOutcome::success(None))
            }
        }
    }

    async fn worker_with(registry: HandlerRegistry) -> (Worker, Arc<InMemoryJobQueue>) {
        let queue = Arc::new(InMemoryJobQueue::new());
        let worker =
            Worker::new(queue.clone(), Arc::new(registry), "w1").with_config(WorkerConfig {
                backoff_base: Duration::from_millis(1),
                backoff_max: Duration::from_millis(5),
                backoff_jitter: 0.0,
                max_attempts: 3,
                poll_interval: Duration::from_millis(5),
                ..Default::default()
            });
        (worker, queue)
    }

    #[tokio::test]
    async fn runs_a_job_to_success() {
        let (worker, queue) = worker_with(HandlerRegistry::new().register(Arc::new(Succeed))).await;
        let mut job = record("j-ok", kinds::SYSTEM_NOOP_TEST, "Queued");
        job.payload_json = r#"{"n":1}"#.into();
        queue.enqueue(job).await.unwrap();

        assert_eq!(
            worker.run_once().await.unwrap(),
            WorkerOutcome::Succeeded { job_id: "j-ok".into() }
        );
        assert_eq!(queue.get("j-ok").await.unwrap().unwrap().state, "Succeeded");
    }

    #[tokio::test]
    async fn invalid_payload_is_dead_lettered() {
        let (worker, queue) = worker_with(HandlerRegistry::new().register(Arc::new(Succeed))).await;
        let mut job = record("j-bad", kinds::SYSTEM_NOOP_TEST, "Queued");
        job.payload_json = r#"{"n":"not-int"}"#.into();
        queue.enqueue(job).await.unwrap();

        assert_eq!(
            worker.run_once().await.unwrap(),
            WorkerOutcome::DeadLettered { job_id: "j-bad".into() }
        );
    }

    #[tokio::test]
    async fn unknown_kind_is_dead_lettered() {
        let (worker, queue) = worker_with(HandlerRegistry::new()).await;
        queue.enqueue(record("j-unknown", "test.nope", "Queued")).await.unwrap();
        match worker.run_once().await.unwrap() {
            WorkerOutcome::UnknownKind { kind, .. } => assert_eq!(kind, "test.nope"),
            other => panic!("expected UnknownKind, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn retries_then_succeeds() {
        let registry =
            HandlerRegistry::new().register(Arc::new(FailTimes { remaining: AtomicU32::new(1) }));
        let (worker, queue) = worker_with(registry).await;
        queue.enqueue(record("j-flaky", "test.flaky", "Queued")).await.unwrap();

        // First attempt fails → retried.
        match worker.run_once().await.unwrap() {
            WorkerOutcome::Retried { .. } => {}
            other => panic!("expected Retried, got {other:?}"),
        }
        // Make it immediately available again and drain.
        queue.reschedule("j-flaky", Duration::ZERO, None).await.unwrap();
        assert_eq!(
            worker.run_once().await.unwrap(),
            WorkerOutcome::Succeeded { job_id: "j-flaky".into() }
        );
    }

    #[tokio::test]
    async fn dead_letters_after_max_attempts() {
        let registry =
            HandlerRegistry::new().register(Arc::new(FailTimes { remaining: AtomicU32::new(100) }));
        let (worker, queue) = worker_with(registry).await;
        queue.enqueue(record("j-fail", "test.flaky", "Queued")).await.unwrap();

        let mut outcome = WorkerOutcome::Idle;
        for _ in 0..5 {
            outcome = worker.run_once().await.unwrap();
            if matches!(outcome, WorkerOutcome::DeadLettered { .. }) {
                break;
            }
            queue.reschedule("j-fail", Duration::ZERO, None).await.unwrap();
        }
        assert_eq!(outcome, WorkerOutcome::DeadLettered { job_id: "j-fail".into() });
    }

    struct NonIdempotentFailure {
        calls: AtomicU32,
        returns_error: bool,
    }

    #[async_trait::async_trait]
    impl JobHandler for NonIdempotentFailure {
        fn kind(&self) -> JobKind {
            "test.non_idempotent".into()
        }
        fn payload_schema(&self) -> &'static str {
            r#"{"type":"object"}"#
        }
        fn is_idempotent(&self) -> bool {
            false
        }
        async fn run(&self, _ctx: JobContext, _p: Value) -> Result<JobOutcome, JobError> {
            self.calls.fetch_add(1, AtomicOrdering::SeqCst);
            if self.returns_error {
                Err(JobError::Storage("test failure".into()))
            } else {
                Ok(JobOutcome::failure())
            }
        }
    }

    #[tokio::test]
    async fn non_idempotent_failures_are_never_automatically_retried() {
        for returns_error in [false, true] {
            let handler =
                Arc::new(NonIdempotentFailure { calls: AtomicU32::new(0), returns_error });
            let (worker, queue) =
                worker_with(HandlerRegistry::new().register(handler.clone())).await;
            queue.enqueue(record("j-once", "test.non_idempotent", "Queued")).await.unwrap();
            assert_eq!(
                worker.run_once().await.unwrap(),
                WorkerOutcome::DeadLettered { job_id: "j-once".into() }
            );
            assert_eq!(worker.run_once().await.unwrap(), WorkerOutcome::Idle);
            let job = queue.get("j-once").await.unwrap().unwrap();
            assert_eq!(job.state, "DeadLettered");
            assert_eq!(job.attempts, 1);
            assert_eq!(handler.calls.load(AtomicOrdering::SeqCst), 1);
        }
    }

    #[tokio::test]
    async fn backoff_is_deterministic_distributed_and_bounded() {
        let (worker, _) = worker_with(HandlerRegistry::new()).await;
        let worker = worker.with_config(WorkerConfig::default());
        let base = worker.config.backoff_base.as_secs_f64();
        let mut buckets = [0; 10];
        for n in 0..10_000 {
            let id = format!("job-{n}");
            let delay = worker.backoff(1, &id);
            assert_eq!(delay, worker.backoff(1, &id));
            let fraction = delay.as_secs_f64() / base;
            assert!((0.8..=1.2).contains(&fraction));
            let bucket = (((fraction - 0.8) / 0.4 * 10.0) as usize).min(9);
            buckets[bucket] += 1;
            for attempt in [0, 2, 8, 16, u32::MAX] {
                assert!(worker.backoff(attempt, &id) <= worker.config.backoff_max);
            }
        }
        assert!(buckets.iter().all(|count| (700..=1300).contains(count)), "{buckets:?}");
    }

    #[tokio::test]
    async fn unknown_schema_rejects_payload() {
        assert!(!validate_payload("not json", &serde_json::json!({})));
        assert!(validate_payload(r#"{"type":"object"}"#, &serde_json::json!({"a":1})));
        assert!(!validate_payload(r#"{"type":"object"}"#, &serde_json::json!(1)));
    }
}
