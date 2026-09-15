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
pub mod telemetry;

#[cfg(feature = "otlp")]
pub mod otlp;

use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::field::RecordFields;
use tracing_subscriber::fmt::FormatFields;

/// Output format for the tracing subscriber.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Human-readable text output.
    Text,
    /// JSON-structured output.
    Json,
}

/// Options for subscriber construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitOptions {
    pub format: Format,
    pub redact_secrets: bool,
}

impl Default for InitOptions {
    fn default() -> Self {
        Self { format: Format::Text, redact_secrets: true }
    }
}

/// A `FormatFields` wrapper that scrubs secret-named field values when
/// `redact` is true, applying Rule A (key-level redaction) from the
/// shared `domain::redaction` helper. Non-secret fields pass through
/// unchanged.
struct RedactingFormatFields {
    inner: tracing_subscriber::fmt::format::DefaultFields,
    redact: bool,
}

impl<'writer> FormatFields<'writer> for RedactingFormatFields {
    fn format_fields<R: RecordFields>(
        &self,
        mut writer: Writer<'writer>,
        fields: R,
    ) -> std::fmt::Result {
        if !self.redact {
            return self.inner.format_fields(writer, fields);
        }
        let mut scratch = String::new();
        {
            let mut scratch_writer = Writer::new(&mut scratch as &mut dyn std::fmt::Write);
            self.inner.format_fields(&mut scratch_writer, fields)?;
        }
        write!(writer, "{}", redact_log_fields(&scratch))
    }
}

/// Scrub secret key=value pairs in a formatted log line (Rule A).
///
/// Matches `key=<value>` patterns where `key` is a recognized secret key
/// and replaces the value portion with the redaction marker.
fn redact_log_fields(formatted: &str) -> String {
    let mut result = formatted.to_string();
    let keys = ["api_key", "apikey", "api-key", "password", "secret", "token", "credential"];
    for key in &keys {
        let pattern = format!("{key}=");
        while let Some(pos) = result.to_lowercase().find(&pattern.to_lowercase()) {
            let val_start = pos + key.len() + 1; // skip key=
            let mut val_end = val_start;
            while val_end < result.len() {
                match result.as_bytes()[val_end] {
                    b' ' | b',' | b';' | b'\n' | b'\r' | b'}' | b']' | b'"' | b'\'' => break,
                    _ => val_end += 1,
                }
            }
            if val_end > val_start {
                result.replace_range(val_start..val_end, "***REDACTED***");
            } else {
                break; // prevent infinite loop
            }
        }
    }
    result
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

/// Initialize the tracing subscriber with options.
///
/// Configures an `EnvFilter` (respecting `RUST_LOG`) and installs
/// either a text or JSON formatter. Returns a `Shutdown` guard that
/// flushes the subscriber on drop. When `redact_secrets` is true
/// (default), field names matching the secret key pattern are redacted
/// in the output.
pub fn init_with_options(opts: InitOptions) -> Shutdown {
    let filter = EnvFilter::from_default_env();

    match opts.format {
        Format::Text => {
            let subscriber = tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_writer(std::io::stderr)
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .fmt_fields(RedactingFormatFields {
                    inner: tracing_subscriber::fmt::format::DefaultFields,
                    redact: opts.redact_secrets,
                })
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
                .fmt_fields(RedactingFormatFields {
                    inner: tracing_subscriber::fmt::format::DefaultFields,
                    redact: opts.redact_secrets,
                })
                .json()
                .finish();
            let _ = tracing::subscriber::set_global_default(subscriber);
        }
    }

    Shutdown
}

/// Initialize the tracing subscriber with default options (redaction ON).
///
/// Delegates to `init_with_options(InitOptions { format, redact_secrets: true })`.
/// This preserves the deny-by-default behavior for all existing callers.
pub fn init(format: Format) -> Shutdown {
    init_with_options(InitOptions { format, redact_secrets: true })
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
