//! N07 `normalize_wasla` — ٱ U+0671 → ا U+0627. Very common in Uthmani
//! text. 1:1 substitution. Idempotent.

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind};
use crate::rules::{CharOut, transform};

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NormalizeWasla;

impl NormalizationRule for NormalizeWasla {
    fn id(&self) -> RuleId {
        RuleId::N07
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Fold U+0671 ARABIC LETTER SUPERSCRIPT ALEF WASLA to bare alef"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Deterministic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        transform(input, self.id(), |ch| {
            if ch == '\u{0671}' { CharOut::One('\u{0627}') } else { CharOut::Keep }
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
        check(&NormalizeWasla, &[("", ""), ("ٱللَّهِ", "اللَّهِ"), ("ٱلرَّحْمَٰنِ", "الرَّحْمَٰنِ")]);
    }
}
