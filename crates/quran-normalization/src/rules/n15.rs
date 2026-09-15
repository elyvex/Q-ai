//! N15 `expand_presentation_forms` — compatibility forms → base sequences.
//!
//! Mapping table (limited set, v1 — the Forms-B lam-alef ligature family
//! explicitly named by the plan; wider coverage is follow-up work recorded
//! for ADR-0204):
//!
//! | Input | Output |
//! |---|
//! | ﻵ U+FEF5, ﻶ U+FEF6 | ل U+0644 + آ U+0622 |
//! | ﻷ U+FEF7, ﻸ U+FEF8 | ل + أ U+0623 |
//! | ﻹ U+FEF9, ﻺ U+FEFA | ل + إ U+0625 |
//! | ﻻ U+FEFB, ﻼ U+FEFC | ل + ا U+0627 |
//!
//! Expansion is 1→2 chars; both derived chars map to the single source index
//! (many-to-one, supported by [`SpanMap`](crate::span::SpanMap)). Never
//! applied to canonical text. Idempotent (output holds no ligatures).

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind, SemVer};
use crate::rules::{CharOut, transform};

/// Expand one presentation form, if it is in the v1 table.
#[must_use]
pub fn expand(ch: char) -> Option<(char, char)> {
    match ch {
        '\u{FEF5}' | '\u{FEF6}' => Some(('\u{0644}', '\u{0622}')),
        '\u{FEF7}' | '\u{FEF8}' => Some(('\u{0644}', '\u{0623}')),
        '\u{FEF9}' | '\u{FEFA}' => Some(('\u{0644}', '\u{0625}')),
        '\u{FEFB}' | '\u{FEFC}' => Some(('\u{0644}', '\u{0627}')),
        _ => None,
    }
}

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpandPresentationForms;

impl NormalizationRule for ExpandPresentationForms {
    fn id(&self) -> RuleId {
        RuleId::N15
    }

    fn version(&self) -> SemVer {
        SemVer::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Expand lam-alef presentation ligatures to base sequences"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Deterministic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        transform(input, self.id(), |ch| match expand(ch) {
            Some((a, b)) => CharOut::Two(a, b),
            None => CharOut::Keep,
        })
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
            &ExpandPresentationForms,
            &[
                ("", ""),
                ("ب\u{FEFB}م", "بلام"),
                ("\u{FEF5}\u{FEF7}\u{FEF9}\u{FEFB}", "لآلألإلا"),
                // Already-base text is untouched:
                ("لا", "لا"),
            ],
        );
    }

    #[test]
    fn expansion_maps_both_chars_to_source() {
        let out = ExpandPresentationForms.apply(&NormalizedText::from_plain("a\u{FEFB}b"));
        assert_eq!(out.text(), "a\u{0644}\u{0627}b");
        // Derived 1,2 (the expansion) both map to canonical 1.
        let span = out.spans().to_canonical(1..3);
        assert_eq!(span.char_range, 1..2);
    }
}
