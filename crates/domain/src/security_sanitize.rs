/// Strip scripts/handlers/`javascript:` URIs from HTML/text (D0.15 / T45).
///
/// A conservative sanitizer for importer-produced text. Removes `<script>` blocks,
/// `on*=` event-handler attributes, and `javascript:`/`data:` URI schemes.
/// This is not a full HTML sanitizer — it is a defense-in-depth layer for
/// content that will never be rendered as HTML.
pub fn sanitize_html(input: &str) -> String {
    let mut out = input.to_string();

    // Remove <script>…</script> blocks (case-insensitive, non-greedy).
    out = replace_all_case_insensitive(&out, "<script", "</script>", "");

    // Remove on*= event-handler attributes.
    out = strip_on_attributes(&out);

    // Neutralize javascript: and data: URI schemes.
    out = neutralize_schemes(&out);

    out
}

fn replace_all_case_insensitive(
    haystack: &str,
    open: &str,
    close: &str,
    replacement: &str,
) -> String {
    let mut result = String::with_capacity(haystack.len());
    let lower = haystack.to_lowercase();
    let open_l = open.to_lowercase();
    let close_l = close.to_lowercase();
    let mut i = 0;
    let bytes = haystack.as_bytes();
    while i < bytes.len() {
        if lower[i..].starts_with(&open_l) {
            // Find the matching close.
            let after_open = i + open_l.len();
            if let Some(close_idx) = lower[after_open..].find(&close_l) {
                let _ = &bytes[after_open..after_open + close_idx]; // content dropped
                i = after_open + close_idx + close_l.len();
                result.push_str(replacement);
                continue;
            }
        }
        // Push the original byte (preserve case).
        let ch = haystack[i..].chars().next().unwrap();
        result.push(ch);
        i += ch.len_utf8();
    }
    result
}

fn strip_on_attributes(input: &str) -> String {
    // Remove attributes that start with "on" followed by an ASCII letter and
    // an `="..."'` value. Operates on a best-effort basis.
    let mut out = String::with_capacity(input.len());
    let lower = input.to_lowercase();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let remaining = &lower[i..];
        // Look for a word boundary followed by "on" + letter.
        let is_attr_start = |j: usize| -> Option<usize> {
            if !remaining[j..].starts_with("on") {
                return None;
            }
            let after = j + 2;
            if after >= remaining.len() {
                return None;
            }
            let b = remaining.as_bytes()[after];
            if b.is_ascii_alphabetic() { Some(after + 1) } else { None }
        };

        if let Some(mut j) = is_attr_start(0) {
            // Skip optional whitespace, then '=' and the quoted value.
            while j < remaining.len() && remaining.as_bytes()[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < remaining.len() && remaining.as_bytes()[j] == b'=' {
                j += 1;
                while j < remaining.len() && remaining.as_bytes()[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < remaining.len() {
                    let quote = remaining.as_bytes()[j];
                    if quote == b'"' || quote == b'\'' {
                        j += 1;
                        while j < remaining.len() && remaining.as_bytes()[j] != quote {
                            j += 1;
                        }
                        if j < remaining.len() {
                            j += 1; // closing quote
                        }
                    } else {
                        // unquoted value: read to whitespace or '>'
                        while j < remaining.len()
                            && !remaining.as_bytes()[j].is_ascii_whitespace()
                            && remaining.as_bytes()[j] != b'>'
                        {
                            j += 1;
                        }
                    }
                }
            }
            // Skip the attribute in the output.
            i += j;
            continue;
        }
        let ch = input[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn neutralize_schemes(input: &str) -> String {
    let lower = input.to_lowercase();
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let remaining = &lower[i..];
        let dangerous = if remaining.starts_with("javascript:") {
            Some("javascript:".len())
        } else if remaining.starts_with("data:") {
            Some("data:".len())
        } else {
            None
        };
        if let Some(len) = dangerous {
            out.push_str("removed:");
            i += len;
        } else {
            let ch = input[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_blocks_are_stripped() {
        let input = r#"<p>hello</p><script>alert(1)</script><p>world</p>"#;
        let out = sanitize_html(input);
        assert!(!out.to_lowercase().contains("<script"));
        assert!(!out.to_lowercase().contains("alert(1)"));
        assert!(out.contains("hello"));
        assert!(out.contains("world"));
    }

    #[test]
    fn event_handlers_are_stripped() {
        let input = r#"<img src="x" onerror="alert(1)" onload="steal()">"#;
        let out = sanitize_html(input);
        assert!(!out.to_lowercase().contains("onerror"));
        assert!(!out.to_lowercase().contains("onload"));
    }

    #[test]
    fn dangerous_schemes_are_neutralized() {
        let input = r#"<a href="javascript:alert(1)">x</a>"#;
        let out = sanitize_html(input);
        assert!(!out.to_lowercase().contains("javascript:"));
        assert!(out.to_lowercase().contains("removed:"));
    }
}
