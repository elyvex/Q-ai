//! Phase 2 — highlight rendering: canonical char ranges → display markers.
//!
//! Pure string function over validated spans (T49). Ranges are char offsets
//! into canonical text; markers wrap each range in order. Invalid input
//! (out-of-range, unsorted, or overlapping ranges) yields `None` — display
//! never guesses. v1 markers are `<b>`/`</b>`, matching the FTS
//! `highlight()` convention from the backend probe.

/// Wrap each char range in markers, in order.
///
/// `ranges` must be sorted, non-overlapping, and inside the text; an empty
/// list returns the text unchanged.
///
/// Returns `None` on any violation (fail-closed display).
#[must_use]
pub fn apply_markers(
    text: &str,
    ranges: &[(u32, u32)],
    open: &str,
    close: &str,
) -> Option<String> {
    let total_chars = text.chars().count() as u32;
    let mut cursor = 0u32;
    for (start, end) in ranges {
        if *start >= *end || *end > total_chars || *start < cursor {
            return None;
        }
        cursor = *end;
    }
    // Char→byte table once (multibyte-safe slicing).
    let mut byte_of = Vec::with_capacity(total_chars as usize + 1);
    for (byte, _) in text.char_indices() {
        byte_of.push(byte as u32);
    }
    byte_of.push(text.len() as u32);
    let mut out = String::new();
    let mut cursor_byte = 0u32;
    for (start, end) in ranges {
        let (from, to) = (byte_of[*start as usize], byte_of[*end as usize]);
        out.push_str(&text[cursor_byte as usize..from as usize]);
        out.push_str(open);
        out.push_str(&text[from as usize..to as usize]);
        out.push_str(close);
        cursor_byte = to;
    }
    out.push_str(&text[cursor_byte as usize..]);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_multibyte_ranges() {
        let text = "بِسْمِ ٱللَّهِ";
        assert_eq!(
            apply_markers(text, &[(0, 6)], "<b>", "</b>"),
            Some("<b>بِسْمِ</b> ٱللَّهِ".to_string())
        );
        // Empty ranges return the text unchanged.
        assert_eq!(apply_markers(text, &[], "<b>", "</b>"), Some(text.to_string()));
        // Multiple ranges in order.
        assert_eq!(
            apply_markers(text, &[(0, 1), (13, 14)], "<b>", "</b>"),
            Some("<b>ب</b>ِسْمِ ٱللَّه<b>ِ</b>".to_string())
        );
    }

    #[test]
    fn rejects_invalid_ranges() {
        let text = "abc";
        assert_eq!(apply_markers(text, &[(0, 9)], "<b>", "</b>"), None);
        assert_eq!(apply_markers(text, &[(2, 2)], "<b>", "</b>"), None);
        assert_eq!(apply_markers(text, &[(0, 2), (1, 3)], "<b>", "</b>"), None);
        assert_eq!(apply_markers(text, &[(2, 1)], "<b>", "</b>"), None);
    }
}
