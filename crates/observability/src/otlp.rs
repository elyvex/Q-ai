//! Opt-in OTLP trace export (D0.8 / T42).
//!
//! Telemetry is **off by default** (PRD §39). When `telemetry.enabled = true`
//! and the `otlp` feature is compiled in, this module installs an
//! OpenTelemetry tracer that exports spans to an OTLP endpoint. The telemetry
//! denylist in [`crate::telemetry`] must be applied to any exported fields.

use opentelemetry::Context;
use opentelemetry::KeyValue;
use opentelemetry::trace::{TraceResult, TracerProvider as _};
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::export::trace::SpanData;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::{BatchSpanProcessor, Span, SpanProcessor, TracerProvider};
use tracing_subscriber::layer::SubscriberExt;

use crate::telemetry::is_scrubbed_span_field;

/// Drop scrubbed attribute keys from a span attribute list in place.
///
/// Returns the number of attributes removed.
pub fn scrub_span_attributes(attrs: &mut Vec<KeyValue>) -> usize {
    let before = attrs.len();
    attrs.retain(|kv| !is_scrubbed_span_field(kv.key.as_str()));
    before - attrs.len()
}

/// A [`SpanProcessor`] that strips scrubbed attribute keys (content-denylist
/// + secret-name rule) from spans and their events before delegating.
///
/// The stderr redaction layer (`RedactingWriter`, Rule A on field names)
/// cannot see spans sent to the OTLP backend, so scrubbing happens here at
/// the exporter boundary instead. Attribute keys are dropped (not replaced):
/// OTLP has no redaction marker convention and a dropped key cannot leak.
#[derive(Debug)]
pub struct ScrubbingProcessor<P: SpanProcessor> {
    inner: P,
}

impl<P: SpanProcessor> ScrubbingProcessor<P> {
    /// Wrap an inner processor with exporter-boundary scrubbing.
    pub fn new(inner: P) -> Self {
        Self { inner }
    }
}

impl<P: SpanProcessor> SpanProcessor for ScrubbingProcessor<P> {
    fn on_start(&self, span: &mut Span, cx: &Context) {
        self.inner.on_start(span, cx);
    }

    fn on_end(&self, mut span: SpanData) {
        scrub_span_attributes(&mut span.attributes);
        for event in &mut span.events.events {
            scrub_span_attributes(&mut event.attributes);
        }
        self.inner.on_end(span);
    }

    fn force_flush(&self) -> TraceResult<()> {
        self.inner.force_flush()
    }

    fn shutdown(&self) -> TraceResult<()> {
        self.inner.shutdown()
    }

    fn set_resource(&mut self, resource: &Resource) {
        self.inner.set_resource(resource);
    }
}

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
///
/// Spans pass through [`ScrubbingProcessor`] so content-denylisted and
/// secret-named attributes never leave the process.
pub fn build_provider(endpoint: &str) -> Result<TracerProvider, String> {
    let exporter = SpanExporter::builder()
        .with_http()
        .with_endpoint(endpoint)
        .build()
        .map_err(|e| e.to_string())?;
    let batch = BatchSpanProcessor::builder(exporter, Tokio).build();
    Ok(TracerProvider::builder()
        .with_span_processor(ScrubbingProcessor::new(batch))
        .with_resource(Resource::new(vec![KeyValue::new("service.name", "qai")]))
        .build())
}

/// Install an OTLP tracer as the global subscriber and return a shutdown guard.
///
/// Call only when telemetry is explicitly enabled.
pub fn init_otlp(endpoint: &str) -> Result<OtlpGuard, String> {
    let provider = build_provider(endpoint)?;
    let tracer = provider.tracer("qai");
    let subscriber =
        tracing_subscriber::registry().with(tracing_opentelemetry::layer().with_tracer(tracer));
    let _ = tracing::subscriber::set_global_default(subscriber);
    Ok(OtlpGuard { provider })
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::InstrumentationScope;
    use opentelemetry::trace::{
        SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState,
    };
    use opentelemetry_sdk::trace::{SpanEvents, SpanLinks};
    use std::sync::Mutex;

    #[test]
    fn rejects_a_malformed_endpoint() {
        assert!(build_provider("not a url").is_err());
    }

    #[test]
    fn scrub_span_attributes_drops_denylisted_and_secret_keys() {
        let mut attrs = vec![
            KeyValue::new("job_id", "job-1"),
            KeyValue::new("prompt_text", "summarize the passage"),
            KeyValue::new("api_key", "SECRET"),
            KeyValue::new("duration_ms", 12),
        ];
        let removed = scrub_span_attributes(&mut attrs);
        assert_eq!(removed, 2);
        let keys: Vec<_> = attrs.iter().map(|kv| kv.key.as_str()).collect();
        assert_eq!(keys, vec!["job_id", "duration_ms"]);
    }

    /// Recording test double: keeps the spans it receives after scrubbing.
    #[derive(Debug, Default)]
    struct RecordingProcessor {
        spans: Mutex<Vec<SpanData>>,
    }

    impl SpanProcessor for RecordingProcessor {
        fn on_start(&self, _span: &mut Span, _cx: &Context) {}
        fn on_end(&self, span: SpanData) {
            self.spans.lock().unwrap().push(span);
        }
        fn force_flush(&self) -> TraceResult<()> {
            Ok(())
        }
        fn shutdown(&self) -> TraceResult<()> {
            Ok(())
        }
    }

    fn span_data(attrs: Vec<KeyValue>) -> SpanData {
        SpanData {
            span_context: SpanContext::new(
                TraceId::from(1u128),
                SpanId::from(1u64),
                TraceFlags::SAMPLED,
                false,
                TraceState::default(),
            ),
            parent_span_id: SpanId::from(0u64),
            span_kind: SpanKind::Internal,
            name: "test".into(),
            start_time: std::time::SystemTime::now(),
            end_time: std::time::SystemTime::now(),
            attributes: attrs,
            dropped_attributes_count: 0,
            events: SpanEvents::default(),
            links: SpanLinks::default(),
            status: Status::Unset,
            instrumentation_scope: InstrumentationScope::builder("test").build(),
        }
    }

    #[test]
    fn processor_strips_scrubbed_attributes_before_delegation() {
        let recorder = std::sync::Arc::new(RecordingProcessor::default());
        let scrubbing = ScrubbingProcessor::new(SharedRecorder(recorder.clone()));
        scrubbing.on_end(span_data(vec![
            KeyValue::new("job_id", "job-1"),
            KeyValue::new("query_text", "secret research question"),
            KeyValue::new("token", "SECRET"),
        ]));
        let spans = recorder.spans.lock().unwrap();
        assert_eq!(spans.len(), 1);
        let keys: Vec<_> = spans[0].attributes.iter().map(|kv| kv.key.as_str()).collect();
        assert_eq!(keys, vec!["job_id"]);
    }

    /// Shared wrapper so the test can inspect what the inner received.
    #[derive(Debug, Clone)]
    struct SharedRecorder(std::sync::Arc<RecordingProcessor>);

    impl SpanProcessor for SharedRecorder {
        fn on_start(&self, span: &mut Span, cx: &Context) {
            self.0.on_start(span, cx);
        }
        fn on_end(&self, span: SpanData) {
            self.0.on_end(span);
        }
        fn force_flush(&self) -> TraceResult<()> {
            self.0.force_flush()
        }
        fn shutdown(&self) -> TraceResult<()> {
            self.0.shutdown()
        }
    }

    #[tokio::test]
    #[ignore = "starts a batch exporter; run against a live OTLP endpoint"]
    async fn builds_a_provider_for_a_valid_endpoint() {
        let provider = build_provider("http://127.0.0.1:4318/v1/traces").unwrap();
        let _ = provider.shutdown();
    }
}
