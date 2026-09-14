//! `tests/observability/telemetry_privacy.rs` — telemetry is off by default and
//! never exports denylisted content fields (AC-P0-18).

use observability::telemetry::{FORBIDDEN_FIELDS, is_forbidden_field, sanitize_telemetry_value};

#[test]
fn telemetry_is_disabled_by_default() {
    let cfg = config::Config::default();
    assert!(!cfg.telemetry.enabled, "telemetry must be off by default (PRD §39)");
}

#[test]
fn enabling_telemetry_never_exports_denylisted_fields() {
    // Even with telemetry switched on, the denylist must strip content.
    let mut cfg = config::Config::default();
    cfg.telemetry.enabled = true;
    assert!(cfg.telemetry.enabled);

    let mut payload = serde_json::json!({
        "component": "qai.retrieval",
        "operation": "search",
        "duration_ms": 42,
        "query_text": "user's private research question",
        "prompt_text": "system prompt with context",
        "document_content": "canonical ayah text",
        "research_question": "why?",
        "model_response": "generated answer"
    });

    sanitize_telemetry_value(&mut payload);

    for field in FORBIDDEN_FIELDS {
        assert!(payload.get(field).is_none(), "denylisted field {field} must never be exported");
        assert!(is_forbidden_field(field));
    }
    // Non-content fields survive.
    assert_eq!(payload["component"], "qai.retrieval");
    assert_eq!(payload["duration_ms"], 42);
}
