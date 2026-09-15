//! N12 `strip_punctuation` — remove Arabic and Latin punctuation.
//!
//! Removed sets (v1, pending ADR-0204 ratification):
//!
//! - ASCII punctuation U+0021–U+002F, U+003A–U+0040, U+005B–U+0060,
//!   U+007B–U+007E (letters and digits explicitly excluded).
//! - Arabic punctuation: U+060C (،)، U+061B (؛)، U+061F (؟)، U+066A (٪),
//!   U+066B (٫)، U+066C (٬)، U+066D (٭)، U+06D4 (۔).
//!
//! Off in exact profiles (L0/L1). Digits are never stripped here (N13 folds
//! them). Pure deletion. Idempotent.

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind, SemVer};
use crate::rules::{CharOut, transform};

/// True for code points removed by N12.
#[must_use]
pub fn is_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '\u{0021}'..='\u{002F}'
            | '\u{003A}'..='\u{0040}'
            | '\u{005B}'..='\u{0060}'
            | '\u{007B}'..='\u{007E}'
            | '\u{060C}'
            | '\u{061B}'
            | '\u{061F}'
            | '\u{066A}'
            | '\u{066B}'
            | '\u{066C}'
            | '\u{066D}'
            | '\u{06D4}'
    )
}

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StripPunctuation;

impl NormalizationRule for StripPunctuation {
    fn id(&self) -> RuleId {
        RuleId::N12
    }

    fn version(&self) -> SemVer {
        SemVer::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Remove Arabic and Latin punctuation"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Deterministic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        transform(
            input,
            self.id(),
            |ch| {
                if is_punctuation(ch) { CharOut::Drop } else { CharOut::Keep }
            },
        )
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
            &StripPunctuation,
            &[
                ("", ""),
                ("الحمد لله!", "الحمد لله"),
                ("،؛؟", ""),
                ("وقف۔لازم", "وقفلازم"),
                // Digits and letters survive:
                ("آية ٢٥٥.", "آية ٢٥٥"),
                ("a_b-c", "abc"),
            ],
        );
    }
}
