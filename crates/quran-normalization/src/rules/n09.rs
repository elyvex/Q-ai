//! N09 `normalize_ta_marbuta` — ة U+0629 → ه U+0647.
//!
//! Optional per PRD §8.1 (only active from profile L5 upward). 1:1
//! substitution. Idempotent.

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind, SemVer};
use crate::rules::{CharOut, transform};

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NormalizeTaMarbuta;

impl NormalizationRule for NormalizeTaMarbuta {
    fn id(&self) -> RuleId {
        RuleId::N09
    }

    fn version(&self) -> SemVer {
        SemVer::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Fold U+0629 TEH MARBUTA to U+0647 HEH"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Deterministic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        transform(input, self.id(), |ch| {
            if ch == '\u{0629}' { CharOut::One('\u{0647}') } else { CharOut::Keep }
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
        check(&NormalizeTaMarbuta, &[("", ""), ("رحمة", "رحمه"), ("الصلاة", "الصلاه"), ("ه", "ه")]);
    }
}
