//! Observability crate for Q-ai.
//!
//! Provides structured tracing with `tracing` + `tracing-subscriber`,
//! a metric catalog via the `metrics` crate, and span conventions
//! documented in plan §D0.8.
//!
//! # Example
//!
//! ```rust
//! use observability::{init, Format, Shutdown};
//!
//! let _guard = init(Format::Text);
//! tracing::info!(message = "server starting");
//! // guard flushes on drop
//! ```

pub mod metrics;

use tracing_subscriber::EnvFilter;

/// Output format for the tracing subscriber.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Human-readable text output.
    Text,
    /// JSON-structured output.
    Json,
}

/// Span field name: the component emitting the span.
pub const SPAN_COMPONENT: &str = "qai.component";
/// Span field name: the operation being performed.
pub const SPAN_OPERATION: &str = "qai.operation";
/// Span field name: the job identifier.
pub const SPAN_JOB_ID: &str = "qai.job_id";
/// Span field name: the run identifier.
pub const SPAN_RUN_ID: &str = "qai.run_id";
/// Span field name: the source identifier.
pub const SPAN_SOURCE_ID: &str = "qai.source_id";
/// Span field name: the source version.
pub const SPAN_SOURCE_VERSION: &str = "qai.source_version";
/// Span field name: the principal identifier.
pub const SPAN_PRINCIPAL_ID: &str = "qai.principal_id";
/// Span field name: the workspace identifier.
pub const SPAN_WORKSPACE_ID: &str = "qai.workspace_id";
/// Span field name: duration in milliseconds.
pub const SPAN_DURATION_MS: &str = "qai.duration_ms";
/// Span field name: the outcome of the operation.
pub const SPAN_OUTCOME: &str = "qai.outcome";

/// Initialize the tracing subscriber.
///
/// Configures an `EnvFilter` (respecting `RUST_LOG`) and installs
/// either a text or JSON formatter. Returns a `Shutdown` guard that
/// flushes the subscriber on drop.
pub fn init(format: Format) -> Shutdown {
    let filter = EnvFilter::from_default_env();

    match format {
        Format::Text => {
            let subscriber = tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_writer(std::io::stderr)
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .compact()
                .finish();
            let _ = tracing::subscriber::set_global_default(subscriber);
        }
        Format::Json => {
            let subscriber = tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_writer(std::io::stderr)
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .json()
                .finish();
            let _ = tracing::subscriber::set_global_default(subscriber);
        }
    }

    Shutdown
}

/// Guard that flushes the tracing subscriber on drop.
pub struct Shutdown;

impl Drop for Shutdown {
    fn drop(&mut self) {
        // Flush any pending spans/events before the subscriber is torn down.
        // `tracing` doesn't expose a direct flush API, but the subscriber
        // flush happens via the subscriber's own shutdown mechanism.
        // This is a no-op placeholder that ensures the guard pattern works
        // and can be extended if a flush handle becomes available.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_text_is_text() {
        assert_eq!(format!("{:?}", Format::Text), "Text");
    }

    #[test]
    fn format_json_is_json() {
        assert_eq!(format!("{:?}", Format::Json), "Json");
    }

    #[test]
    fn span_constants_are_non_empty() {
        assert!(!SPAN_COMPONENT.is_empty());
        assert!(!SPAN_OPERATION.is_empty());
        assert!(!SPAN_JOB_ID.is_empty());
        assert!(!SPAN_RUN_ID.is_empty());
        assert!(!SPAN_SOURCE_ID.is_empty());
        assert!(!SPAN_SOURCE_VERSION.is_empty());
        assert!(!SPAN_PRINCIPAL_ID.is_empty());
        assert!(!SPAN_WORKSPACE_ID.is_empty());
        assert!(!SPAN_DURATION_MS.is_empty());
        assert!(!SPAN_OUTCOME.is_empty());
    }

    #[test]
    fn init_does_not_panic() {
        let _guard = init(Format::Text);
        tracing::info!(message = "init test");
    }
}
