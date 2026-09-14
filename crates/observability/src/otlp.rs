//! Opt-in OTLP trace export (D0.8 / T42).
//!
//! Telemetry is **off by default** (PRD §39). When `telemetry.enabled = true`
//! and the `otlp` feature is compiled in, this module installs an
//! OpenTelemetry tracer that exports spans to an OTLP endpoint. The telemetry
//! denylist in [`crate::telemetry`] must be applied to any exported fields.

use opentelemetry::KeyValue;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Flushes and shuts the tracer provider down on drop.
pub struct OtlpGuard {
    provider: TracerProvider,
}

impl Drop for OtlpGuard {
    fn drop(&mut self) {
        let _ = self.provider.shutdown();
    }
}

/// Build a tracer provider exporting OTLP/HTTP to `endpoint`.
pub fn build_provider(endpoint: &str) -> Result<TracerProvider, String> {
    let exporter = SpanExporter::builder()
        .with_http()
        .with_endpoint(endpoint)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(TracerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_resource(Resource::new(vec![KeyValue::new("service.name", "qai")]))
        .build())
}

/// Install an OTLP tracer as the global subscriber and return a shutdown guard.
///
/// Call only when telemetry is explicitly enabled.
pub fn init_otlp(endpoint: &str) -> Result<OtlpGuard, String> {
    let provider = build_provider(endpoint)?;
    let tracer = provider.tracer("qai");
    let subscriber = tracing_subscriber::registry()
        .with(tracing_opentelemetry::layer().with_tracer(tracer));
    let _ = tracing::subscriber::set_global_default(subscriber);
    Ok(OtlpGuard { provider })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn builds_a_provider_for_a_valid_endpoint() {
        // Building does not connect; the exporter is lazy.
        let provider = build_provider("http://127.0.0.1:4318/v1/traces");
        assert!(provider.is_ok());
        if let Ok(provider) = provider {
            let _ = provider.shutdown();
        }
    }

    #[test]
    fn rejects_a_malformed_endpoint() {
        assert!(build_provider("not a url").is_err());
    }
}
