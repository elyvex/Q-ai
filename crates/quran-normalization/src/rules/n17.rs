//! N17 `remove_spaces` — delete all U+0020.
//!
//! Enables §8.3 concatenated (space-insensitive skeleton) search. Only ASCII
//! space is removed here; other whitespace was already collapsed by N01.
//! Pure deletion. Idempotent.

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind, SemVer};
use crate::rules::{CharOut, transform};

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoveSpaces;

impl NormalizationRule for RemoveSpaces {
    fn id(&self) -> RuleId {
        RuleId::N17
    }

    fn version(&self) -> SemVer {
        SemVer::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Delete all U+0020 spaces"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Deterministic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        transform(input, self.id(), |ch| if ch == ' ' { CharOut::Drop } else { CharOut::Keep })
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
            &RemoveSpaces,
            &[
                ("", ""),
                ("بسمالله", "بسمالله"),
                ("بسم الله", "بسمالله"),
                // Only ASCII space is removed (N01 owns the rest):
                ("a\tb", "a\tb"),
            ],
        );
    }

    #[test]
    fn skeleton_span_segments() {
        // "بسم الله" -> "بسمالله": derived 3 maps to canonical 4.
        let out = RemoveSpaces.apply(&NormalizedText::from_plain("بسم الله"));
        assert_eq!(out.text(), "بسمالله");
        let span = out.spans().to_canonical(3..4);
        assert_eq!(span.char_range, 4..5);
    }
}
