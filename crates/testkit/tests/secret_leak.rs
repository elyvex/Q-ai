//! `tests/security/secret_leak.rs` — a sentinel secret must appear in zero bytes.
//!
//! AC-P0-05 / permanent CI gate: proves `Secret<T>` redaction across `Debug`,
//! `Display`, JSON serialization, and the audit redaction layer.

use config::Secret;

const SENTINEL: &str = "SENTINEL_9f3c__DO_NOT_LEAK";

#[test]
fn sentinel_never_appears_in_secret_representations() {
    let secret = Secret::new(SENTINEL.to_string());

    let debug = format!("{secret:?}");
    let display = format!("{secret}");
    let json = serde_json::to_string(&secret).unwrap();

    for (label, rendered) in [("Debug", debug), ("Display", display), ("Serialize", json)] {
        assert!(!rendered.contains(SENTINEL), "{label} leaked the sentinel: {rendered}");
        assert!(rendered.contains("***"), "{label} did not render a redaction marker: {rendered}");
    }
}

#[test]
fn sentinel_does_not_leak_from_a_container() {
    // Wrapping a secret in a tuple/struct and formatting it must still redact.
    let secret = Secret::new(SENTINEL.to_string());
    let container = ("prefix", secret, 42_i32);
    let rendered = format!("{container:?}");
    assert!(!rendered.contains(SENTINEL), "container Debug leaked: {rendered}");
}

#[test]
fn audit_redaction_strips_sentinel_under_secret_keys() {
    let mut payload = serde_json::json!({
        "password": SENTINEL,
        "api_key": SENTINEL,
        "nested": { "secret": SENTINEL, "public": "ok" }
    });
    audit::redact_audit_value(&mut payload);
    let rendered = serde_json::to_string(&payload).unwrap();
    assert!(!rendered.contains(SENTINEL), "audit serialization leaked the sentinel: {rendered}");
    assert_eq!(payload["nested"]["public"], "ok");
}
