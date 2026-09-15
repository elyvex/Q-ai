//! N11 `strip_zero_width_and_bidi` — remove invisible format controls.
//!
//! Removed: U+200B–U+200F (zero-width space/preserved-joiners/LRM/RLM),
//! U+202A–U+202E (embeddings, overrides, isolates initiators), U+2066–U+2069
//! (isolates), U+FEFF (zero-width no-break space / BOM).
//!
//! Security as well as correctness: bidi overrides can visually reorder
//! quoted text. Pure deletion. Idempotent.

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind, SemVer};
use crate::rules::{CharOut, transform};

/// True for code points removed by N11.
#[must_use]
pub fn is_invisible_format(ch: char) -> bool {
    matches!(
        ch,
        '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2066}'..='\u{2069}'
            | '\u{FEFF}'
    )
}

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StripZeroWidthAndBidi;

impl NormalizationRule for StripZeroWidthAndBidi {
    fn id(&self) -> RuleId {
        RuleId::N11
    }

    fn version(&self) -> SemVer {
        SemVer::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Remove zero-width and bidi control characters"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Deterministic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        transform(input, self.id(), |ch| {
            if is_invisible_format(ch) { CharOut::Drop } else { CharOut::Keep }
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
            &StripZeroWidthAndBidi,
            &[
                ("", ""),
                ("ab", "ab"),
                ("a\u{200B}b\u{200C}c\u{200D}d", "abcd"),
                ("\u{FEFF}نص", "نص"),
                ("a\u{202B}b\u{202C}c", "abc"),
            ],
        );
    }
}
