//! N02 `strip_tatweel` — remove U+0640 (ـ). Pure deletion. Idempotent.

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind};
use crate::rules::{CharOut, transform};

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StripTatweel;

impl NormalizationRule for StripTatweel {
    fn id(&self) -> RuleId {
        RuleId::N02
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Remove U+0640 ARABIC TATWEEL"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Deterministic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        transform(
            input,
            self.id(),
            |ch| {
                if ch == '\u{0640}' { CharOut::Drop } else { CharOut::Keep }
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
            &StripTatweel,
            &[("", ""), ("abc", "abc"), ("ا\u{0640}ب", "اب"), ("\u{0640}\u{0640}", "")],
        );
    }
}
