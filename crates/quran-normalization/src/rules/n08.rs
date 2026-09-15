//! N08 `normalize_alif_maqsura` — ى U+0649 → ي U+064A.
//!
//! Fold direction fixed by ADR-0204 (toward yeh). 1:1 substitution. Idempotent.

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind, SemVer};
use crate::rules::{CharOut, transform};

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NormalizeAlifMaqsura;

impl NormalizationRule for NormalizeAlifMaqsura {
    fn id(&self) -> RuleId {
        RuleId::N08
    }

    fn version(&self) -> SemVer {
        SemVer::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Fold U+0649 ALEF MAKSURA to U+064A YEH"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Deterministic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        transform(input, self.id(), |ch| {
            if ch == '\u{0649}' { CharOut::One('\u{064A}') } else { CharOut::Keep }
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
        check(&NormalizeAlifMaqsura, &[("", ""), ("على", "علي"), ("موسى", "موسي"), ("ي", "ي")]);
    }
}
