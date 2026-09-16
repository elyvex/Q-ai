//! Secret-redaction helpers — single source of truth for Rules A/B/C.
//!
//! This module is the leaf-level shared matcher for secret material across
//! log emission, error renderers, and CLI inspection outputs. It must remain
//! dependency-free (`unsafe_code = "forbid"` inherited from the workspace).
//!
//! Rules (see data-model.md):
//! - **Rule A**: secret-named keys/fields → whole value replaced with marker.
//! - **Rule B**: `key[:=]value` in free text → value portion replaced.
//! - **Rule C**: URL `scheme://…@` → userinfo replaced.

use std::borrow::Cow;

/// Replacement marker for redacted values (matches audit contract).
pub const REDACTED_MARKER: &str = "***REDACTED***";

/// Rule A: does this key/field name denote secret material?
///
/// Case-insensitive, separator-normalized check against:
/// `secret | password | api_key | token | credential`. Separators (`-`) are
/// normalized to `_` and the compact `apikey` form is also recognized, so
/// `api_key`, `api-key`, `apiKey`, and `APIKEY` all match.
///
/// When `true`, the corresponding value MUST be replaced with
/// [`REDACTED_MARKER`] (Rule A) and no recursion into that value is needed.
pub fn is_secret_key(name: &str) -> bool {
    let lower = name.to_lowercase().replace('-', "_");
    lower.contains("secret")
        || lower.contains("password")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("token")
        || lower.contains("credential")
}

/// Rules A+C: redact a JSON value in place; returns number of redactions.
///
/// - Objects: secret-named keys → [`REDACTED_MARKER`]; other keys recurse.
/// - Strings: Rule C (URL userinfo) scrub.
/// - Arrays: recurse into each element.
/// - Other scalars: unchanged.
///
/// **Postcondition**: structure (keys, nesting, non-secret leaf bytes) is
/// identical before and after.
pub fn redact_json_value(value: &mut serde_json::Value) -> usize {
    let mut count = 0;
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                if is_secret_key(k) {
                    *v = serde_json::Value::String(REDACTED_MARKER.to_string());
                    count += 1;
                } else {
                    count += redact_json_value(v);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                count += redact_json_value(item);
            }
        }
        serde_json::Value::String(s) => {
            let scrubbed = redact_text(s);
            match scrubbed {
                Cow::Borrowed(_) => {}
                Cow::Owned(new) => {
                    *s = new;
                    count += 1;
                }
            }
        }
        _ => {}
    }
    count
}

/// Rules B+C: redact credential patterns inside free text.
///
/// - **Rule B**: `secret|password|api_key|token|credential` followed by `:` or `=`
///   and a non-space value → value portion replaced.
/// - **Rule C**: URL userinfo `scheme://…@…` → credentials replaced.
///
/// On clean input, returns a borrowed slice (zero-alloc hot path).
pub fn redact_text(input: &str) -> Cow<'_, str> {
    let mut out = String::new();
    let mut wrote = false;

    for line in input.split('\n') {
        if wrote {
            out.push('\n');
        }
        let scrubbed = redact_line(line);
        match &scrubbed {
            Cow::Borrowed(_) if !wrote => {}
            Cow::Borrowed(b) => {
                out.push_str(b);
                wrote = true;
            }
            Cow::Owned(s) => {
                out.push_str(s);
                wrote = true;
            }
        }
    }

    if wrote { Cow::Owned(out) } else { Cow::Borrowed(input) }
}

fn redact_line(input: &str) -> Cow<'_, str> {
    // Rule C: URL userinfo — scheme://…@…
    // Find "://" followed by any characters then "@", replace userinfo.
    if let Some(scheme_end) = input.find("://") {
        let after_scheme = scheme_end + 3;
        if let Some(at_pos) = input[after_scheme..].find('@') {
            let abs_at = after_scheme + at_pos;
            let mut result = String::new();
            result.push_str(&input[..after_scheme]);
            result.push_str(REDACTED_MARKER);
            result.push_str(&input[abs_at..]);
            return Cow::Owned(result);
        }
    }

    // Rule B: key[:=]value for secret keys
    let lower = input.to_lowercase();
    let keys = ["api_key", "api-key", "apikey", "password", "secret", "token", "credential"];

    for key in &keys {
        if let Some(pos) = lower.find(key) {
            // Check what follows: must be `=` or `:` then non-space
            let after_key = pos + key.len();
            if after_key < input.len() {
                let next_ch = input.as_bytes()[after_key];
                if next_ch == b'=' || next_ch == b':' {
                    // Find value start (skip separator and any spaces)
                    let mut val_start = after_key + 1;
                    while val_start < input.len() && input.as_bytes()[val_start] == b' ' {
                        val_start += 1;
                    }
                    if val_start < input.len()
                        && input.as_bytes()[val_start] != b' '
                        && input.as_bytes()[val_start] != b'\n'
                    {
                        // Find value end (next space, comma, semicolon, or end)
                        let mut val_end = val_start;
                        while val_end < input.len() {
                            match input.as_bytes()[val_end] {
                                b' ' | b',' | b';' | b'\n' | b'\r' | b'"' | b'\'' | b')' | b'}'
                                | b']' => break,
                                _ => val_end += 1,
                            }
                        }
                        if val_end > val_start {
                            let mut result = String::new();
                            result.push_str(&input[..val_start]);
                            result.push_str(REDACTED_MARKER);
                            result.push_str(&input[val_end..]);
                            return Cow::Owned(result);
                        }
                    }
                }
            }
        }
    }

    Cow::Borrowed(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Rule A tests ──────────────────────────────────────────────

    #[test]
    fn is_secret_key_matches_expected_names() {
        for key in &[
            "api_key",
            "API_KEY",
            "password",
            "secret",
            "token",
            "credential",
            "api-key",
            "apiKey",
            "user_secret",
            "access_token",
            "auth_credential",
        ] {
            assert!(is_secret_key(key), "expected `{key}` to be recognized");
        }
    }

    #[test]
    fn is_secret_key_rejects_benign_names() {
        for key in &["name", "job_id", "duration_ms", "status", "version", "path"] {
            assert!(!is_secret_key(key), "`{key}` must NOT be flagged");
        }
    }

    // ── Rule B tests ──────────────────────────────────────────────

    #[test]
    fn rule_b_scrubs_key_value_pair() {
        let input = "connect failed: api_key=sk-abc123def";
        let result = redact_text(input);
        assert!(!result.contains("sk-abc123def"));
        assert!(result.contains("***REDACTED***"));
    }

    #[test]
    fn rule_b_scrubs_password_colon() {
        let input = "login failed password: hunter2";
        let result = redact_text(input);
        assert!(!result.contains("hunter2"));
        assert!(result.contains("***REDACTED***"));
    }

    #[test]
    fn rule_b_preserves_benign_token_usage() {
        let input = "approval token issued for session";
        let result = redact_text(input);
        // "token" is followed by " issued" (space then word, not key=value pattern)
        // The key matching finds "token" at position where next is space — not `=` or `:`
        assert_eq!(*result, *input, "benign prose must pass through unchanged");
    }

    // ── Rule C tests ──────────────────────────────────────────────

    #[test]
    fn rule_c_scrubs_url_userinfo() {
        let input = "connect to https://user:pass@db.example.com/data";
        let result = redact_text(input);
        assert!(!result.contains("user:pass"));
        assert!(result.contains("***REDACTED***"));
        assert!(result.contains("db.example.com"));
    }

    #[test]
    fn rule_c_scrubs_url_at_only() {
        let input = "endpoint: postgresql://admin:s3cret@localhost:5432/qai";
        let result = redact_text(input);
        assert!(!result.contains("admin:s3cret"));
        assert!(result.contains("***REDACTED***"));
        assert!(result.contains("localhost:5432"));
    }

    // ── JSON redaction tests ──────────────────────────────────────

    #[test]
    fn redact_json_scrubs_secret_keys() {
        let mut val = serde_json::json!({
            "api_key": "sk-live-12345",
            "password": "hunter2",
            "name": "public",
            "nested": { "token": "tok-abc" }
        });
        let count = redact_json_value(&mut val);
        assert_eq!(count, 3);
        assert_eq!(val["api_key"], serde_json::json!(REDACTED_MARKER));
        assert_eq!(val["password"], serde_json::json!(REDACTED_MARKER));
        assert_eq!(val["name"], "public");
        assert_eq!(val["nested"]["token"], serde_json::json!(REDACTED_MARKER));
    }

    #[test]
    fn redact_json_scrubs_url_in_string_values() {
        let mut val = serde_json::json!({
            "db_url": "postgresql://admin:pass@localhost/db"
        });
        redact_json_value(&mut val);
        let s = val["db_url"].as_str().unwrap();
        assert!(!s.contains("admin:pass"));
        assert!(s.contains("***REDACTED***"));
    }

    #[test]
    fn redact_json_preserves_structure() {
        let original = serde_json::json!({
            "version": "1.0.0",
            "checks": [{"id": "x", "status": "pass"}],
            "api_key": "sk-1"
        });
        let mut val = original.clone();
        redact_json_value(&mut val);
        assert_eq!(val["version"], "1.0.0");
        assert_eq!(val["checks"][0]["id"], "x");
        assert_eq!(val["checks"][0]["status"], "pass");
        assert_eq!(val["api_key"], serde_json::json!(REDACTED_MARKER));
    }

    // ── Borrow-on-clean tests ─────────────────────────────────────

    #[test]
    fn redact_text_returns_borrowed_on_clean_input() {
        let input = "hello world, nothing sensitive here";
        let result = redact_text(input);
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn redact_text_returns_owned_on_dirty_input() {
        let input = "api_key=sk-12345 something else";
        let result = redact_text(input);
        assert!(matches!(result, Cow::Owned(_)));
    }
}
