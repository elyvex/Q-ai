//! Handler registry for the worker pool (D0.9 / T37).
//!
//! Maps a job kind to its [`JobHandler`]. Unknown kinds are dead-lettered by
//! the worker rather than silently dropped.

use std::collections::HashMap;
use std::sync::Arc;

use crate::JobHandler;

/// A registry of job handlers keyed by kind.
#[derive(Default)]
pub struct HandlerRegistry {
    handlers: HashMap<String, Arc<dyn JobHandler>>,
}

impl HandlerRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a handler under its declared kind, returning `self` for chaining.
    pub fn register(mut self, handler: Arc<dyn JobHandler>) -> Self {
        self.handlers.insert(handler.kind(), handler);
        self
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
}
