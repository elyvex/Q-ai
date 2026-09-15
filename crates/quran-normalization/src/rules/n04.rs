//! N04 `strip_quranic_marks` — remove Uthmani annotation marks.
//!
//! Mapping table (removed code points):
//!
//! | Range | Contents |
//! |---|---|
//! | U+06D6–U+06ED | small high signs, sajdah sign, rub-el-hizb, waqf marks |
//! | U+06DD | end-of-ayah mark |
//! | U+0615, U+0617–U+061A | small high annotation signs |
//! | U+06E5, U+06E6 | small waw / small yeh above (waqf-related) |
//!
//! Note: U+06D6–U+06DB (waqf/pause letters) are a separately addressable
//! subset, N14, so callers can "keep marks but drop waqf". Pure deletion.
//! Idempotent.

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind};
use crate::rules::{CharOut, transform};

/// True for code points removed by N04.
///
/// The U+06D6–U+06ED span already covers U+06DD (end of ayah) and U+06E5/U+06E6
/// (small waw/yeh above); they are named in the rule docs, not repeated here.
#[must_use]
pub fn is_quranic_mark(ch: char) -> bool {
    matches!(
        ch,
        '\u{06D6}'..='\u{06ED}' | '\u{0615}' | '\u{0617}'..='\u{061A}'
    )
}

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StripQuranicMarks;

impl NormalizationRule for StripQuranicMarks {
    fn id(&self) -> RuleId {
        RuleId::N04
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Remove Uthmani annotation marks (U+06D6-U+06ED, U+06DD, U+0615, U+0617-U+061A, U+06E5-U+06E6)"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Deterministic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        transform(
            input,
            self.id(),
            |ch| {
                if is_quranic_mark(ch) { CharOut::Drop } else { CharOut::Keep }
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
            &StripQuranicMarks,
            &[
                ("", ""),
                ("الحمد", "الحمد"),
                // end-of-ayah + sajdah + small high meem-isolated-form:
                ("نص\u{06DD}\u{06DC}\u{06D8}", "نص"),
                ("كلم\u{0615}ة", "كلمة"),
            ],
        );
    }
}
