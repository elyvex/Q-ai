//! N10 `normalize_persian_codepoints` — fold Persian-keyboard variants.
//!
//! Mapping table:
//!
//! | Input | Output |
//! |---|
//! | ک U+06A9 KEHEH | ك U+0643 KAF |
//! | ی U+06CC FARSI YEH | ي U+064A YEH |
//! | ۀ U+06C0 HEH WITH YEH ABOVE | ه U+0647 HEH |
//! | ہ U+06C1 HEH GOAL | ه U+0647 HEH |
//!
//! Preserved (identity) but noted: گ U+06AF, ژ U+0698, چ U+0686, پ U+067E —
//! genuine Persian phonemes with no Arabic equivalent; folding them would
//! destroy information, so they pass through (a future `contains_heuristic`
//! flag may note their presence; pending ADR-0204). 1:1. Idempotent.

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind, SemVer};
use crate::rules::{CharOut, transform};

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NormalizePersianCodepoints;

impl NormalizationRule for NormalizePersianCodepoints {
    fn id(&self) -> RuleId {
        RuleId::N10
    }

    fn version(&self) -> SemVer {
        SemVer::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Fold Persian keheh/farsi-yeh/heh variants to Arabic code points"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Deterministic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        transform(input, self.id(), |ch| match ch {
            '\u{06A9}' => CharOut::One('\u{0643}'),
            '\u{06CC}' => CharOut::One('\u{064A}'),
            '\u{06C0}' | '\u{06C1}' => CharOut::One('\u{0647}'),
            _ => CharOut::Keep,
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
            &NormalizePersianCodepoints,
            &[
                ("", ""),
                // Persian-keyboard typing ("one", "blessing"):
                ("یک", "يك"),
                ("برکت", "بركت"),
                // Heh variants fold to Arabic heh:
                ("ۀہ", "هه"),
                // Genuine Persian phonemes survive:
                ("گژچپ", "گژچپ"),
                // Unrelated code points (e.g. noon ghunna) pass through:
                ("الرحمں", "الرحمں"),
            ],
        );
    }
}
