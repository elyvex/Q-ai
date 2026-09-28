//! In-process worker pool (D0.9 / T37–T39).
//!
//! A [`Worker`] claims jobs from a [`JobQueue`], runs the registered handler
//! with cancellation + heartbeat + checkpointing, and applies the retry /
//! backoff / dead-letter policy.

use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde_json::Value;
use tracing::{info, warn};

use crate::queue::JobQueue;
use crate::registry::HandlerRegistry;
use crate::{JobContext, JobError, JobState};
use storage::repository::JobRecord;

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
    /// The job was cancelled, with the locked cooperative disposition.
    Cancelled { job_id: String, disposition: CancellationDisposition },
    /// The job kind had no registered handler.
    UnknownKind { job_id: String, kind: String },
}

/// How a cooperative cancellation request resolved (D-16).
///
/// Exactly one of these is persisted into the job's result/error JSON and
/// reported through [`WorkerOutcome::Cancelled`]; a cancellation is never
/// reported without naming which boundary state applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancellationDisposition {
    /// The request was observed at a named checkpoint boundary (including
    /// the pre-start boundary) and the job stopped there.
    CancelledAtCheckpoint,
    /// The handler completed successfully before observing the request.
    CompletedBeforeObservation,
    /// The handler could not reach a safe boundary, so the job was
    /// finalized without falsely claiming it stopped at one.
    MissedBoundary,
}

impl std::fmt::Display for CancellationDisposition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CancelledAtCheckpoint => write!(f, "cancelled_at_checkpoint"),
            Self::CompletedBeforeObservation => write!(f, "completed_before_observation"),
            Self::MissedBoundary => write!(f, "missed_boundary"),
        }
    }
}

impl FromStr for CancellationDisposition {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "cancelled_at_checkpoint" => Ok(Self::CancelledAtCheckpoint),
            "completed_before_observation" => Ok(Self::CompletedBeforeObservation),
            "missed_boundary" => Ok(Self::MissedBoundary),
            other => Err(format!("unknown cancellation disposition `{other}`")),
        }
    }
}

/// Serialize a disposition into the existing job result/error JSON column.
pub fn disposition_json(disposition: CancellationDisposition) -> String {
    serde_json::json!({"disposition": disposition.to_string()}).to_string()
}

/// Parse a disposition back out of a stored result/error JSON value.
/// Returns `None` when the payload carries no locked disposition.
pub fn parse_disposition(raw: &str) -> Option<CancellationDisposition> {
    serde_json::from_str::<Value>(raw).ok()?.get("disposition")?.as_str()?.parse().ok()
}

/// The resolved retry/backoff/lease rule for one claimed job.
struct EffectivePolicy {
    max_attempts: u32,
    backoff_base: Duration,
    backoff_max: Duration,
    backoff_jitter: f64,
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
        self.process_claimed(job, None).await
    }

    /// Long-lived host loop (D-13): recover interrupted leases, claim one job
    /// at a time, and stop claiming once shutdown is signalled. When shutdown
    /// arrives during a handler, persist an owner-safe cancellation request
    /// for that job and await its durable terminal transition instead of
    /// aborting it (D-14, D-16). Returns the number of jobs processed.
    ///
    /// The 01-03 named-checkpoint, retry, and [`CancellationDisposition`]
    /// behavior is preserved: the per-job path below is shared with
    /// [`run_once`](Self::run_once).
    pub async fn run_until_shutdown(
        &self,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
    ) -> Result<u32, JobError> {
        self.recover_interrupted().await?;
        let mut processed = 0;
        loop {
            if *shutdown.borrow() {
                break;
            }
            let Some(job) = self.queue.claim_next(&self.owner, self.config.lease).await? else {
                // Bounded idle wait: wake for shutdown or the next poll.
                tokio::select! {
                    _ = shutdown.changed() => break,
                    _ = tokio::time::sleep(self.config.poll_interval) => continue,
                }
            };
            let outcome = self.process_claimed(job, Some(&mut shutdown)).await?;
            debug_assert!(!matches!(outcome, WorkerOutcome::Idle));
            processed += 1;
        }
        Ok(processed)
    }

    /// Run one claimed job through validation, the handler/watchdog, and the
    /// owner-checked terminal transition. With a shutdown receiver, a signal
    /// during the handler persists an owner-safe cancellation request and
    /// awaits the durable outcome instead of aborting (D-14, D-16).
    async fn process_claimed(
        &self,
        job: JobRecord,
        shutdown: Option<&mut tokio::sync::watch::Receiver<bool>>,
    ) -> Result<WorkerOutcome, JobError> {
        let Some(handler) = self.registry.get(&job.kind) else {
            warn!(job_id = %job.id, kind = %job.kind, "no handler registered; dead-lettering");
            let _ = self
                .queue
                .finish_owned(
                    &job.id,
                    &self.owner,
                    JobState::DeadLettered,
                    Some("unknown job kind".into()),
                )
                .await?;
            return Ok(WorkerOutcome::UnknownKind { job_id: job.id, kind: job.kind });
        };

        // Payload schema validation (fail closed).
        let payload: Value = serde_json::from_str(&job.payload_json).unwrap_or(Value::Null);
        if !validate_payload(handler.payload_schema(), &payload) {
            let _ = self
                .queue
                .finish_owned(
                    &job.id,
                    &self.owner,
                    JobState::DeadLettered,
                    Some("invalid payload".into()),
                )
                .await?;
            return Ok(WorkerOutcome::DeadLettered { job_id: job.id });
        }

        // A request already pending observes the pre-start boundary: no stage
        // ran, so the job stops at the start checkpoint.
        if self.queue.cancel_requested(&job.id).await? {
            self.finish_cancelled(&job.id, CancellationDisposition::CancelledAtCheckpoint).await?;
            return Ok(WorkerOutcome::Cancelled {
                job_id: job.id,
                disposition: CancellationDisposition::CancelledAtCheckpoint,
            });
        }

        // Watchdog: poll cancellation + renew the lease + persist each named
        // checkpoint immediately while the handler runs.
        let cancel = Arc::new(AtomicBool::new(false));
        let ctx = JobContext::with_cancel(job.clone(), cancel.clone());
        let checkpoint_sink = ctx.checkpoint_sink();
        let watchdog = tokio::spawn(watchdog(
            self.queue.clone(),
            job.id.clone(),
            self.owner.clone(),
            cancel.clone(),
            checkpoint_sink.clone(),
            self.config.lease,
            self.config.poll_interval,
        ));

        let handler_future = handler.run(ctx, payload);
        tokio::pin!(handler_future);
        let result = match shutdown {
            Some(rx) => {
                tokio::select! {
                    r = &mut handler_future => r,
                    _ = rx.changed() => {
                        // Owner-safe cooperative request (D-14/D-16): flag
                        // only, never clears the lease; the watchdog raises
                        // the handler flag within one poll, then the terminal
                        // path below awaits the durable outcome.
                        let _ = self.queue.request_cancel(&job.id).await;
                        handler_future.await
                    }
                }
            }
            None => handler_future.await,
        };
        watchdog.abort();

        // Flush any boundary the watchdog did not persist yet.
        let pending = checkpoint_sink.lock().ok().and_then(|mut slot| slot.take());
        if let Some(cp) = pending {
            let _ = self.queue.checkpoint_owned(&job.id, &self.owner, None, Some(cp)).await?;
        }

        let cancel_seen =
            cancel.load(Ordering::SeqCst) || self.queue.cancel_requested(&job.id).await?;
        match result {
            Ok(outcome) if outcome.success => {
                if cancel_seen {
                    // The request arrived but the handler finished first: say
                    // so instead of claiming a cancellation (D-16).
                    self.finish_succeeded(
                        &job.id,
                        Some(disposition_json(CancellationDisposition::CompletedBeforeObservation)),
                    )
                    .await?;
                } else {
                    self.finish_succeeded(&job.id, outcome.result).await?;
                }
                Ok(WorkerOutcome::Succeeded { job_id: job.id })
            }
            Ok(_) | Err(_) => {
                // Cooperative cancellation takes precedence over failure, but
                // the disposition must name the true boundary state.
                if cancel_seen {
                    let at_boundary = self
                        .queue
                        .get(&job.id)
                        .await?
                        .and_then(|row| row.checkpoint_json)
                        .is_some();
                    let disposition = if at_boundary {
                        CancellationDisposition::CancelledAtCheckpoint
                    } else {
                        CancellationDisposition::MissedBoundary
                    };
                    self.finish_cancelled(&job.id, disposition).await?;
                    return Ok(WorkerOutcome::Cancelled { job_id: job.id, disposition });
                }
                if !handler.is_idempotent() {
                    // Prohibition: a non-idempotent job is never retried
                    // automatically, even when attempts remain.
                    let _ = self
                        .queue
                        .finish_owned(
                            &job.id,
                            &self.owner,
                            JobState::DeadLettered,
                            Some("non-idempotent job cannot be retried automatically".into()),
                        )
                        .await?;
                    return Ok(WorkerOutcome::DeadLettered { job_id: job.id });
                }
                let policy = self.effective_policy(&job);
                self.handle_failure(&job, &policy).await
            }
        }
    }

    /// Process jobs until the queue is empty. Returns the number processed.
    ///
    /// Long-lived hosts only (D-13). One-shot CLI paths must not call this;
    /// host ownership lands in 01-04.
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
        policy: &EffectivePolicy,
    ) -> Result<WorkerOutcome, JobError> {
        let attempts = job.attempts.max(1);
        if attempts >= policy.max_attempts {
            let _ = self
                .queue
                .finish_owned(
                    &job.id,
                    &self.owner,
                    JobState::DeadLettered,
                    Some("max attempts exceeded".into()),
                )
                .await?;
            Ok(WorkerOutcome::DeadLettered { job_id: job.id.clone() })
        } else {
            let delay = Self::backoff_with(
                policy.backoff_base,
                policy.backoff_max,
                policy.backoff_jitter,
                attempts,
                &job.id,
            );
            self.queue.reschedule(&job.id, delay, Some("retry".into())).await?;
            Ok(WorkerOutcome::Retried {
                job_id: job.id.clone(),
                next_delay_ms: delay.as_millis() as u64,
            })
        }
    }

    /// Resolve the retry/backoff rule for a claimed job (D-15): an explicit
    /// per-kind policy wins; otherwise the enqueue-time snapshot
    /// (`JobRecord.max_attempts`) applies, bounded above by the host config.
    fn effective_policy(&self, job: &JobRecord) -> EffectivePolicy {
        if self.registry.has_policy(&job.kind) {
            let policy = self.registry.policy_for(&job.kind);
            EffectivePolicy {
                max_attempts: policy.max_attempts.max(1),
                backoff_base: Duration::from_millis(policy.backoff_base_ms),
                backoff_max: Duration::from_millis(policy.backoff_max_ms),
                backoff_jitter: policy.jitter.clamp(0.0, 1.0),
            }
        } else {
            EffectivePolicy {
                max_attempts: job.max_attempts.max(1).min(self.config.max_attempts.max(1)),
                backoff_base: self.config.backoff_base,
                backoff_max: self.config.backoff_max,
                backoff_jitter: self.config.backoff_jitter,
            }
        }
    }

    /// Owner-checked success finish; warns (without misreporting the
    /// outcome) when the lease was lost mid-run — the job will be reaped as
    /// `Interrupted` and resume from its committed checkpoint.
    async fn finish_succeeded(&self, job_id: &str, result: Option<String>) -> Result<(), JobError> {
        let applied =
            self.queue.finish_owned(job_id, &self.owner, JobState::Succeeded, result).await?;
        if !applied {
            warn!(job_id, owner = %self.owner, "lease lost before success finish");
        }
        Ok(())
    }

    /// Owner-checked cancellation finish carrying the locked disposition.
    async fn finish_cancelled(
        &self,
        job_id: &str,
        disposition: CancellationDisposition,
    ) -> Result<(), JobError> {
        let applied = self
            .queue
            .finish_owned(
                job_id,
                &self.owner,
                JobState::Cancelled,
                Some(disposition_json(disposition)),
            )
            .await?;
        if !applied {
            warn!(job_id, owner = %self.owner, "lease lost before cancel finish");
        }
        Ok(())
    }

    /// Exponential backoff with deterministic jitter (no `rand` dependency),
    /// parameterized by the resolved policy.
    fn backoff_with(
        base: Duration,
        max: Duration,
        jitter: f64,
        attempts: u32,
        job_id: &str,
    ) -> Duration {
        let shift = attempts.saturating_sub(1).min(16);
        let exp = base.saturating_mul(1u32 << shift).min(max);
        // Deterministic 0..1 fraction from the job id.
        let mut hash: u64 = 1469598103934665603;
        for b in job_id.bytes() {
            hash ^= b as u64;
            hash = hash.wrapping_mul(1099511628211);
        }
        let frac = (hash % 1000) as f64 / 1000.0;
        let factor = 1.0 + jitter * (frac * 2.0 - 1.0);
        Duration::from_secs_f64((exp.as_secs_f64() * factor).max(0.0)).min(max)
    }
}

async fn watchdog(
    queue: Arc<dyn JobQueue>,
    job_id: String,
    owner: String,
    cancel: Arc<AtomicBool>,
    checkpoints: Arc<Mutex<Option<String>>>,
    lease: Duration,
    poll: Duration,
) {
    let mut last_persisted: Option<String> = None;
    loop {
        tokio::time::sleep(poll).await;
        if queue.cancel_requested(&job_id).await.unwrap_or(false) {
            cancel.store(true, Ordering::SeqCst);
        }
        // Immediate durable checkpoint (D-14): each newly recorded named
        // boundary is persisted through the owner-checked operation while the
        // handler is still running — never only after failure.
        let pending = checkpoints.lock().ok().and_then(|mut slot| slot.take());
        if let Some(cp) = pending
            && last_persisted.as_deref() != Some(cp.as_str())
        {
            if queue
                .checkpoint_owned(&job_id, &owner, None, Some(cp.clone()))
                .await
                .unwrap_or(false)
            {
                last_persisted = Some(cp);
            } else if let Ok(mut slot) = checkpoints.lock() {
                // Lease lost mid-run: hand the value back so the terminal
                // path can attempt one final persist.
                *slot = Some(cp);
            }
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
        let defaults = WorkerConfig::default();
        let base = defaults.backoff_base.as_secs_f64();
        let mut buckets = [0; 10];
        for n in 0..10_000 {
            let id = format!("job-{n}");
            let delay = Worker::backoff_with(
                defaults.backoff_base,
                defaults.backoff_max,
                defaults.backoff_jitter,
                1,
                &id,
            );
            assert_eq!(
                delay,
                Worker::backoff_with(
                    defaults.backoff_base,
                    defaults.backoff_max,
                    defaults.backoff_jitter,
                    1,
                    &id
                )
            );
            let fraction = delay.as_secs_f64() / base;
            assert!((0.8..=1.2).contains(&fraction));
            let bucket = (((fraction - 0.8) / 0.4 * 10.0) as usize).min(9);
            buckets[bucket] += 1;
            for attempt in [0, 2, 8, 16, u32::MAX] {
                assert!(
                    Worker::backoff_with(
                        defaults.backoff_base,
                        defaults.backoff_max,
                        defaults.backoff_jitter,
                        attempt,
                        &id
                    ) <= defaults.backoff_max
                );
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

    // ─── 01-03-02: cooperative cancellation, per-kind retry, immediacy ───

    use crate::registry::RetryPolicy;

    /// Records one named boundary, then waits for cancellation and fails.
    struct CheckpointThenWait;
    #[async_trait::async_trait]
    impl JobHandler for CheckpointThenWait {
        fn kind(&self) -> JobKind {
            "test.checkpoint_wait".into()
        }
        fn payload_schema(&self) -> &'static str {
            r#"{"type":"object"}"#
        }
        fn is_idempotent(&self) -> bool {
            true
        }
        async fn run(&self, ctx: JobContext, _p: Value) -> Result<JobOutcome, JobError> {
            ctx.checkpoint_named("stage.one", serde_json::json!({"n": 1}));
            for _ in 0..1000 {
                if ctx.is_cancelled() {
                    return Ok(JobOutcome::failure());
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Err(JobError::Storage("cancel never arrived".into()))
        }
    }

    /// Waits for cancellation and fails without ever reaching a boundary.
    struct WaitCancelNoCheckpoint;
    #[async_trait::async_trait]
    impl JobHandler for WaitCancelNoCheckpoint {
        fn kind(&self) -> JobKind {
            "test.wait_cancel".into()
        }
        fn payload_schema(&self) -> &'static str {
            r#"{"type":"object"}"#
        }
        fn is_idempotent(&self) -> bool {
            true
        }
        async fn run(&self, ctx: JobContext, _p: Value) -> Result<JobOutcome, JobError> {
            for _ in 0..1000 {
                if ctx.is_cancelled() {
                    return Ok(JobOutcome::failure());
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Err(JobError::Storage("cancel never arrived".into()))
        }
    }

    /// Succeeds after a fixed delay, ignoring mid-run cancellation.
    struct SlowSucceed;
    #[async_trait::async_trait]
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
            tokio::time::sleep(Duration::from_millis(150)).await;
            Ok(JobOutcome::success(None))
        }
    }

    /// Records a boundary and then fails slowly, so the test can observe
    /// the row while the handler is still running.
    struct CheckpointThenSlowFail;
    #[async_trait::async_trait]
    impl JobHandler for CheckpointThenSlowFail {
        fn kind(&self) -> JobKind {
            "test.slow_fail".into()
        }
        fn payload_schema(&self) -> &'static str {
            r#"{"type":"object"}"#
        }
        fn is_idempotent(&self) -> bool {
            true
        }
        async fn run(&self, ctx: JobContext, _p: Value) -> Result<JobOutcome, JobError> {
            ctx.checkpoint_named("stage.one", serde_json::json!({"n": 1}));
            tokio::time::sleep(Duration::from_secs(30)).await;
            Ok(JobOutcome::failure())
        }
    }

    #[test]
    fn disposition_json_round_trips_all_outcomes() {
        for d in [
            CancellationDisposition::CancelledAtCheckpoint,
            CancellationDisposition::CompletedBeforeObservation,
            CancellationDisposition::MissedBoundary,
        ] {
            let raw = disposition_json(d);
            assert_eq!(parse_disposition(&raw), Some(d), "{raw}");
            assert_eq!(d.to_string().parse::<CancellationDisposition>(), Ok(d));
        }
        assert_eq!(parse_disposition("{}"), None);
        assert!("bogus".parse::<CancellationDisposition>().is_err());
    }

    #[tokio::test]
    async fn pre_start_cancel_reports_cancelled_at_checkpoint() {
        let (worker, queue) = worker_with(HandlerRegistry::new().register(Arc::new(Succeed))).await;
        // A valid payload so the run would succeed absent cancellation.
        let mut job = record("j-pre", kinds::SYSTEM_NOOP_TEST, "Queued");
        job.payload_json = r#"{"n":1}"#.into();
        queue.enqueue(job).await.unwrap();
        queue.request_cancel("j-pre").await.unwrap();

        assert_eq!(
            worker.run_once().await.unwrap(),
            WorkerOutcome::Cancelled {
                job_id: "j-pre".into(),
                disposition: CancellationDisposition::CancelledAtCheckpoint,
            }
        );
        assert_eq!(queue.get("j-pre").await.unwrap().unwrap().state, "Cancelled");
    }

    #[tokio::test]
    async fn cancel_at_boundary_reports_cancelled_at_checkpoint() {
        let (worker, queue) =
            worker_with(HandlerRegistry::new().register(Arc::new(CheckpointThenWait))).await;
        queue.enqueue(record("j-cb", "test.checkpoint_wait", "Queued")).await.unwrap();

        let handle = tokio::spawn(async move { worker.run_once().await });
        tokio::time::sleep(Duration::from_millis(100)).await;
        queue.request_cancel("j-cb").await.unwrap();
        let outcome = handle.await.unwrap().unwrap();
        assert_eq!(
            outcome,
            WorkerOutcome::Cancelled {
                job_id: "j-cb".into(),
                disposition: CancellationDisposition::CancelledAtCheckpoint,
            }
        );
        assert_eq!(queue.get("j-cb").await.unwrap().unwrap().state, "Cancelled");
    }

    #[tokio::test]
    async fn cancel_without_boundary_reports_missed_boundary() {
        let (worker, queue) =
            worker_with(HandlerRegistry::new().register(Arc::new(WaitCancelNoCheckpoint))).await;
        queue.enqueue(record("j-mb", "test.wait_cancel", "Queued")).await.unwrap();

        let handle = tokio::spawn(async move { worker.run_once().await });
        tokio::time::sleep(Duration::from_millis(100)).await;
        queue.request_cancel("j-mb").await.unwrap();
        let outcome = handle.await.unwrap().unwrap();
        assert_eq!(
            outcome,
            WorkerOutcome::Cancelled {
                job_id: "j-mb".into(),
                disposition: CancellationDisposition::MissedBoundary,
            },
            "no boundary reached: must not claim a checkpoint cancellation"
        );
        assert_eq!(queue.get("j-mb").await.unwrap().unwrap().state, "Cancelled");
    }

    #[tokio::test]
    async fn success_before_observation_stays_succeeded() {
        let (worker, queue) =
            worker_with(HandlerRegistry::new().register(Arc::new(SlowSucceed))).await;
        queue.enqueue(record("j-cbo", "test.slow_succeed", "Queued")).await.unwrap();

        let handle = tokio::spawn(async move { worker.run_once().await });
        // The handler runs 150 ms; the request lands mid-run but the handler
        // finishes without observing it.
        tokio::time::sleep(Duration::from_millis(30)).await;
        queue.request_cancel("j-cbo").await.unwrap();
        assert_eq!(
            handle.await.unwrap().unwrap(),
            WorkerOutcome::Succeeded { job_id: "j-cbo".into() }
        );
        assert_eq!(queue.get("j-cbo").await.unwrap().unwrap().state, "Succeeded");
    }

    #[tokio::test]
    async fn per_kind_policy_caps_attempts_and_backoff() {
        let policy =
            RetryPolicy { max_attempts: 2, backoff_base_ms: 10, backoff_max_ms: 20, jitter: 0.0 };
        struct Flaky;
        #[async_trait::async_trait]
        impl JobHandler for Flaky {
            fn kind(&self) -> JobKind {
                "test.flaky_kind".into()
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
        let (worker, queue) =
            worker_with(HandlerRegistry::new().register_with_policy(Arc::new(Flaky), policy)).await;
        queue.enqueue(record("j-pk", "test.flaky_kind", "Queued")).await.unwrap();

        // Attempt 1 of 2: bounded per-kind backoff (10 ms base, zero jitter).
        match worker.run_once().await.unwrap() {
            WorkerOutcome::Retried { next_delay_ms, .. } => assert_eq!(next_delay_ms, 10),
            other => panic!("expected Retried, got {other:?}"),
        }
        queue.reschedule("j-pk", Duration::ZERO, None).await.unwrap();
        // Attempt 2 of 2: exhausted into the inspectable state.
        assert_eq!(
            worker.run_once().await.unwrap(),
            WorkerOutcome::DeadLettered { job_id: "j-pk".into() }
        );
        let row = queue.get("j-pk").await.unwrap().unwrap();
        assert_eq!(row.state, "DeadLettered");
        assert_eq!(row.attempts, 2);
    }

    #[tokio::test]
    async fn checkpoint_reaches_storage_while_the_handler_runs() {
        let (worker, queue) =
            worker_with(HandlerRegistry::new().register(Arc::new(CheckpointThenSlowFail))).await;
        queue.enqueue(record("j-imm", "test.slow_fail", "Queued")).await.unwrap();

        let handle = tokio::spawn(async move { worker.run_once().await });
        // The handler sleeps 30 s; the watchdog (5 ms poll) must persist the
        // boundary long before the handler returns.
        let mut seen = false;
        for _ in 0..200 {
            tokio::time::sleep(Duration::from_millis(10)).await;
            if queue.get("j-imm").await.unwrap().unwrap().checkpoint_json.is_some() {
                seen = true;
                break;
            }
        }
        assert!(seen, "named checkpoint must be queryable while the handler runs");
        handle.abort();
    }
}
