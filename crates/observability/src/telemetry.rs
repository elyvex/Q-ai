//! Telemetry privacy gate (D0.8 / P0-T42, AC-P0-18).
//!
//! Telemetry is **off by default** (`telemetry.enabled = false`). When enabled,
//! a compile-time denylist ensures content-bearing fields are never exported:
//! query text, prompt text, document content, research questions, and model
//! responses (PRD §39).

/// Fields that must never leave the process via telemetry (PRD §39).
pub const FORBIDDEN_FIELDS: [&str; 5] =
    ["query_text", "prompt_text", "document_content", "research_question", "model_response"];

/// Whether a field name is denylisted for telemetry export.
///
/// Matches the canonical names as well as common variants (`prompt`,
/// `research_questions`, `document_text`, …).
pub fn is_forbidden_field(name: &str) -> bool {
    let lower = name.to_lowercase();
    FORBIDDEN_FIELDS.iter().any(|f| lower == *f)
        || lower.contains("query")
        || lower.contains("prompt")
        || lower.contains("document_content")
        || lower.contains("research_question")
        || lower.contains("model_response")
}

/// Whether an OTLP span attribute key must be scrubbed before export.
///
/// Union of the telemetry content denylist ([`is_forbidden_field`]) and the
/// secret-name rule (`domain::redaction::is_secret_key`). Applied at the
/// exporter boundary by the OTLP scrubbing processor (see `otlp.rs`) so span
/// fields — which bypass the stderr `RedactingWriter` — cannot carry secret
/// or content-bearing values to the backend.
pub fn is_scrubbed_span_field(name: &str) -> bool {
    is_forbidden_field(name) || domain::redaction::is_secret_key(name)
}

/// Recursively strip denylisted keys from a telemetry payload.
///
/// Returns the number of keys removed.
pub fn sanitize_telemetry_value(value: &mut serde_json::Value) -> usize {
    let mut removed = 0;
    match value {
        serde_json::Value::Object(map) => {
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in keys {
                if is_forbidden_field(&key) {
                    map.remove(&key);
                    removed += 1;
                } else if let Some(inner) = map.get_mut(&key) {
                    removed += sanitize_telemetry_value(inner);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                removed += sanitize_telemetry_value(item);
            }
        }
        _ => {}
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forbidden_names_are_recognized() {
        for name in FORBIDDEN_FIELDS {
            assert!(is_forbidden_field(name), "{name} must be forbidden");
        }
        assert!(is_forbidden_field("query"));
        assert!(is_forbidden_field("Prompt"));
        assert!(!is_forbidden_field("job_id"));
        assert!(!is_forbidden_field("duration_ms"));
    }

    #[test]
    fn forbidden_fields_are_stripped_from_a_payload() {
        let mut payload = serde_json::json!({
            "job_id": "job-1",
            "duration_ms": 12,
            "query_text": "secret research question",
            "prompt_text": "summarize the tabari passage",
            "model_response": "…",
            "document_content": "…",
            "research_question": "…"
        });
        let removed = sanitize_telemetry_value(&mut payload);
        assert_eq!(removed, 5);
        assert_eq!(payload["job_id"], "job-1");
        assert_eq!(payload["duration_ms"], 12);
        for field in FORBIDDEN_FIELDS {
            assert!(payload.get(field).is_none(), "{field} must be removed");
        }
    }

    #[test]
    fn scrubbed_span_fields_cover_denylist_and_secret_names() {
        // Content denylist (AC-P0-18).
        for name in [
            "query_text",
            "prompt_text",
            "document_content",
            "research_question",
            "model_response",
            "query",
            "Prompt",
        ] {
            assert!(is_scrubbed_span_field(name), "{name} must be scrubbed");
        }
        // Secret-name rule (Rule A): values under these keys must not reach OTLP.
        for name in ["api_key", "password", "secret", "token", "credential", "api-key"] {
            assert!(is_scrubbed_span_field(name), "{name} must be scrubbed");
        }
        // Operational fields pass through.
        for name in ["job_id", "duration_ms", "service.name", "http.status_code"] {
            assert!(!is_scrubbed_span_field(name), "{name} must survive scrubbing");
        }
    }

    #[test]
    fn nested_forbidden_fields_are_stripped() {
        let mut payload = serde_json::json!({
            "span": {
                "job_id": "job-2",
                "context": { "prompt_text": "…", "ok": true }
            }
        });
        let removed = sanitize_telemetry_value(&mut payload);
        assert_eq!(removed, 1);
        assert!(payload["span"]["context"].get("prompt_text").is_none());
        assert_eq!(payload["span"]["context"]["ok"], true);
    }
}
