//! N01 `whitespace_collapse` — runs of whitespace → single U+0020; trim ends.
//!
//! Always first in every profile. Idempotent. Whitespace is Unicode
//! `char::is_whitespace` (covers tab, newline, NBSP, etc.), so queries pasted
//! from any source collapse identically.

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind, SemVer};
use crate::span::SpanMap;

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhitespaceCollapse;

impl NormalizationRule for WhitespaceCollapse {
    fn id(&self) -> RuleId {
        RuleId::N01
    }

    fn version(&self) -> SemVer {
        SemVer::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Collapse whitespace runs to a single U+0020 and trim ends"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Deterministic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        let mut text = String::new();
        let mut forward: Vec<u32> = Vec::new();
        let mut in_run = false;
        // Trailing run is trimmed: remember where it started.
        let mut run_start_out: Option<usize> = None;
        let mut run_start_fwd: usize = 0;
        for (i, ch) in input.text().chars().enumerate() {
            if ch.is_whitespace() {
                if !in_run {
                    in_run = true;
                    run_start_out = Some(text.len());
                    run_start_fwd = forward.len();
                    text.push(' ');
                    forward.push(i as u32);
                }
            } else {
                in_run = false;
                run_start_out = None;
                text.push(ch);
                forward.push(i as u32);
            }
        }
        if in_run && let Some(out_len) = run_start_out.take() {
            text.truncate(out_len);
            forward.truncate(run_start_fwd);
        }
        // Leading run: the first emitted char is the collapsed space; drop it.
        if input.text().chars().next().is_some_and(|c| c.is_whitespace()) && !forward.is_empty() {
            text.remove(0);
            forward.remove(0);
        }
        let canonical_len = input.text().chars().count() as u32;
        let rule_map = SpanMap::build(canonical_len, forward, self.id())
            .expect("N01 builds a total map by construction");
        let spans = input.spans().clone().compose(&rule_map);
        NormalizedText::from_parts(text, spans)
    }

    fn is_idempotent(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::test_support::check;

    #[test]
    fn mapping_table() {
        check(
            &WhitespaceCollapse,
            &[
                ("", ""),
                ("a", "a"),
                ("  a  b  ", "a b"),
                ("a\t\nb", "a b"),
                ("   ", ""),
                ("بِسْمِ  ٱللَّهِ", "بِسْمِ ٱللَّهِ"),
                ("a\u{00A0}b", "a b"),
            ],
        );
    }

    #[test]
    fn span_points_at_first_space_of_run() {
        let out = WhitespaceCollapse.apply(&NormalizedText::from_plain("a  b"));
        assert_eq!(out.text(), "a b");
        // Derived 1 (the collapsed space) maps to canonical 1 (run start).
        let span = out.spans().to_canonical(1..2);
        assert_eq!(span.char_range, 1..2);
    }
}
