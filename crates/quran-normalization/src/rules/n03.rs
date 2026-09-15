//! N03 `strip_harakat` — remove diacritics. The core "ignore diacritics" rule.
//!
//! Mapping table (removed code points):
//!
//! | Range | Names |
//! |---|---|
//! | U+064B–U+0652 | fathatan, dammatan, kasratan, fatha, damma, kasra, shadda, sukun |
//! | U+0656–U+0659 | subscript alef, inverted damma, mark noon ghunna, zel |
//! | U+065A–U+065F | wah, small v above, small v below, damma reflected, fatha reflected |
//!
//! Deliberately **kept**: U+0653 (maddah above), U+0654 (hamza above),
//! U+0655 (hamza below) — they change the letter's reading, not just its
//! vowelling (linguist review pending, ADR-0204). Pure deletion. Idempotent.

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind, SemVer};
use crate::rules::{CharOut, transform};

/// True for code points removed by N03.
#[must_use]
pub fn is_haraka(ch: char) -> bool {
    matches!(ch, '\u{064B}'..='\u{0652}' | '\u{0656}'..='\u{0659}' | '\u{065A}'..='\u{065F}')
}

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StripHarakat;

impl NormalizationRule for StripHarakat {
    fn id(&self) -> RuleId {
        RuleId::N03
    }

    fn version(&self) -> SemVer {
        SemVer::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Remove harakat and related diacritics (U+064B-U+0652, U+0656-U+0659, U+065A-U+065F)"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Deterministic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        transform(input, self.id(), |ch| if is_haraka(ch) { CharOut::Drop } else { CharOut::Keep })
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
            &StripHarakat,
            &[
                ("", ""),
                ("الرحمن", "الرحمن"),
                // fatha + shadda + superscript alef kept (N05's job) + fatha:
                ("ٱلرَّحْمَٰنِ", "ٱلرحمٰن"),
                ("بِسْمِ", "بسم"),
                // maddah / hamza-above / hamza-below survive N03:
                ("آ\u{0654}\u{0655}", "آ\u{0654}\u{0655}"),
            ],
        );
    }

    #[test]
    fn basmala_first_word_span() {
        // "بِسْمِ" (6 chars) -> "بسم"; derived 1 ('س') maps to canonical 2.
        let out = StripHarakat.apply(&NormalizedText::from_plain("بِسْمِ"));
        assert_eq!(out.text(), "بسم");
        let span = out.spans().to_canonical(1..2);
        assert_eq!(span.char_range, 2..3);
        assert!(span.exact);
    }
}
