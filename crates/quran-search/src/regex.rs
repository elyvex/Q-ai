//! Phase 2 — DFA-only regex compilation and searching (I16).
//!
//! Exactly one engine may ever compile user patterns: the dense DFA builder
//! below with construction budgets. Backtracking engines are not a fallback
//! anywhere in this crate — a pattern the DFA cannot build is rejected, not
//! retried elsewhere. Services reuse [`compile_dfa`] + [`first_match`] so
//! tool-layer span resolution runs the identical automaton the backend
//! expanded.

use crate::error::IndexError;

/// NFA construction budget (bytes).
pub const NFA_SIZE_LIMIT: usize = 1024 * 1024;
/// DFA construction budget (bytes).
pub const DFA_SIZE_LIMIT: usize = 4 * 1024 * 1024;
/// Pattern length cap (chars, measured in bytes — stricter for multibyte).
pub const MAX_PATTERN_LEN: usize = 512;

/// Compile a pattern with the DFA-only engine and I16 budgets.
///
/// Rejects over-long patterns and unanchored leading `.*`/`.+` before
/// compiling; anything the DFA builder refuses is a rejection, never a
/// fallback to another engine.
pub fn compile_dfa(pattern: &str) -> Result<regex_automata::dfa::regex::Regex, IndexError> {
    use regex_automata::dfa::{dense, regex};
    use regex_automata::nfa::thompson;
    if pattern.len() > MAX_PATTERN_LEN {
        return Err(IndexError::QueryRejected {
            detail: format!("pattern exceeds {MAX_PATTERN_LEN} characters"),
        });
    }
    let stripped = pattern.strip_prefix('^').unwrap_or(pattern);
    if stripped.starts_with(".*") || stripped.starts_with(".+") {
        return Err(IndexError::QueryRejected {
            detail: "unanchored leading .* is rejected; anchor the pattern instead".to_string(),
        });
    }
    let mut builder = regex::Builder::new();
    builder
        .dense(dense::Config::new().minimize(true).dfa_size_limit(Some(DFA_SIZE_LIMIT)))
        .thompson(thompson::Config::new().nfa_size_limit(Some(NFA_SIZE_LIMIT)));
    builder.build(pattern).map_err(|err| IndexError::QueryRejected {
        detail: format!("invalid pattern: {err}"),
    })
}

/// First match of a compiled pattern as a char range, if any.
#[must_use]
pub fn first_match(
    dfa: &regex_automata::dfa::regex::Regex,
    text: &str,
) -> Option<std::ops::Range<u32>> {
    let found = dfa.try_search(text).ok()??;
    let (start_byte, end_byte) = (found.start(), found.end());
    let start = text[..start_byte].chars().count() as u32;
    let end = start + text[start_byte..end_byte].chars().count() as u32;
    Some(start..end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guards_reject_before_compiling() {
        assert!(compile_dfa(&"a".repeat(600)).is_err());
        assert!(compile_dfa(".*abc").is_err());
        assert!(compile_dfa(".+abc").is_err());
        assert!(compile_dfa("^.*abc").is_err());
        assert!(compile_dfa("^ا?ل?رحم").is_ok());
        assert!(compile_dfa("[invalid").is_err());
    }

    #[test]
    fn first_match_reports_char_ranges() {
        let dfa = compile_dfa("^ا?ل?رحم").unwrap();
        assert_eq!(first_match(&dfa, "الرحمن"), Some(0..4));
        assert_eq!(first_match(&dfa, "xxالرحمن"), Some(2..6));
        assert_eq!(first_match(&dfa, "xyz"), None);
    }
}
