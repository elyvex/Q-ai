//! Handler registry for the worker pool (D0.9 / T37).
//!
//! Maps a job kind to its [`JobHandler`]. Unknown kinds are dead-lettered by
//! the worker rather than silently dropped.

use std::collections::HashMap;
use std::sync::Arc;

use crate::JobHandler;

/// Bounded retry policy for one job kind (D-15).
///
/// `max_attempts` counts claim attempts (each `claim_next` increments
/// `JobRecord.attempts`); the worker dead-letters once attempts reach the
/// limit. Backoff between attempts is exponential from `backoff_base_ms`,
/// capped at `backoff_max_ms`, with `jitter` (0.0–1.0) applied
/// deterministically per job id. The default mirrors
/// [`crate::worker::WorkerConfig`] so unregistered kinds behave exactly as
/// before.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RetryPolicy {
    /// Maximum claim attempts before the job is dead-lettered.
    pub max_attempts: u32,
    /// Base of the exponential backoff, in milliseconds.
    pub backoff_base_ms: u64,
    /// Cap on the backoff delay, in milliseconds.
    pub backoff_max_ms: u64,
    /// Deterministic jitter fraction (0.0–1.0).
    pub jitter: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self { max_attempts: 5, backoff_base_ms: 500, backoff_max_ms: 60_000, jitter: 0.2 }
    }
}

/// A registry of job handlers keyed by kind.
#[derive(Default)]
pub struct HandlerRegistry {
    handlers: HashMap<String, Arc<dyn JobHandler>>,
    policies: HashMap<String, RetryPolicy>,
}

impl HandlerRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a handler under its declared kind, returning `self` for chaining.
    ///
    /// The kind keeps the default [`RetryPolicy`]; use
    /// [`register_with_policy`](Self::register_with_policy) or
    /// [`set_policy`](Self::set_policy) for a per-kind limit (D-15).
    pub fn register(mut self, handler: Arc<dyn JobHandler>) -> Self {
        self.handlers.insert(handler.kind(), handler);
        self
    }

    /// Register a handler with an explicit per-kind [`RetryPolicy`].
    pub fn register_with_policy(
        mut self,
        handler: Arc<dyn JobHandler>,
        policy: RetryPolicy,
    ) -> Self {
        self.policies.insert(handler.kind(), policy);
        self.handlers.insert(handler.kind(), handler);
        self
    }

    /// Set (or replace) the retry policy for `kind`, registered or not.
    pub fn set_policy(&mut self, kind: &str, policy: RetryPolicy) {
        self.policies.insert(kind.to_string(), policy);
    }

    /// The retry policy for `kind`: the explicit per-kind policy when one
    /// was registered, otherwise the default policy.
    pub fn policy_for(&self, kind: &str) -> RetryPolicy {
        self.policies.get(kind).copied().unwrap_or_default()
    }

    /// Whether `kind` carries an explicit per-kind policy (as opposed to the
    /// default). The worker uses this to decide between the kind policy and
    /// the enqueue-time snapshot bounded by the host config.
    pub fn has_policy(&self, kind: &str) -> bool {
        self.policies.contains_key(kind)
    }

    /// Snapshot the kind's retry policy into the job's `max_attempts` field
    /// at enqueue time (D-15). Reuses the existing column; no schema change.
    pub fn stamp_job_policy(&self, job: &mut storage::repository::JobRecord) {
        job.max_attempts = self.policy_for(&job.kind).max_attempts;
    }

    /// Look up the handler for a kind.
    pub fn get(&self, kind: &str) -> Option<Arc<dyn JobHandler>> {
        self.handlers.get(kind).cloned()
    }

    /// The registered kinds.
    pub fn kinds(&self) -> Vec<String> {
        let mut kinds: Vec<String> = self.handlers.keys().cloned().collect();
        kinds.sort();
        kinds
    }

    /// Number of registered handlers.
    pub fn len(&self) -> usize {
        self.handlers.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{JobContext, JobError, JobKind, JobOutcome};

    struct Noop;
    #[async_trait::async_trait]
    impl JobHandler for Noop {
        fn kind(&self) -> JobKind {
            "test.noop".into()
        }
        fn payload_schema(&self) -> &'static str {
            r#"{"type":"object"}"#
        }
        fn is_idempotent(&self) -> bool {
            true
        }
        async fn run(
            &self,
            _ctx: JobContext,
            _payload: serde_json::Value,
        ) -> Result<JobOutcome, JobError> {
            Ok(JobOutcome::success(None))
        }
    }

    #[test]
    fn register_and_lookup() {
        let registry = HandlerRegistry::new().register(Arc::new(Noop));
        assert_eq!(registry.len(), 1);
        assert!(registry.get("test.noop").is_some());
        assert!(registry.get("missing").is_none());
    }

    #[test]
    fn default_policy_matches_worker_defaults() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 5);
        assert_eq!(policy.backoff_base_ms, 500);
        assert_eq!(policy.backoff_max_ms, 60_000);
        assert!((policy.jitter - 0.2).abs() < f64::EPSILON);
        let registry = HandlerRegistry::new();
        assert!(!registry.has_policy("test.noop"));
        assert_eq!(registry.policy_for("test.noop"), RetryPolicy::default());
    }

    #[test]
    fn per_kind_policy_is_isolated_and_stamped() {
        let strict = RetryPolicy {
            max_attempts: 2,
            backoff_base_ms: 100,
            backoff_max_ms: 1_000,
            jitter: 0.0,
        };
        let registry = HandlerRegistry::new().register_with_policy(Arc::new(Noop), strict);
        assert!(registry.has_policy("test.noop"));
        assert_eq!(registry.policy_for("test.noop"), strict);
        // Unrelated kinds keep the default; stamping one kind never leaks.
        assert!(!registry.has_policy("other.kind"));
        assert_eq!(registry.policy_for("other.kind"), RetryPolicy::default());

        let mut job = crate::test_support::record("j1", "test.noop", "Queued");
        registry.stamp_job_policy(&mut job);
        assert_eq!(job.max_attempts, 2);
        let mut other = crate::test_support::record("j2", "other.kind", "Queued");
        registry.stamp_job_policy(&mut other);
        assert_eq!(other.max_attempts, 5);
    }
}
