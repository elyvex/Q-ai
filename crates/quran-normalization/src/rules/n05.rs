//! N05 `strip_superscript_alef` — remove U+0670 (ٰ).
//!
//! Separated from N03 because it changes the reading, not just the vowelling:
//! profiles can drop diacritics (L3) while callers still see this fold listed
//! explicitly in the trace. Pure deletion. Idempotent.

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind};
use crate::rules::{CharOut, transform};

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StripSuperscriptAlef;

impl NormalizationRule for StripSuperscriptAlef {
    fn id(&self) -> RuleId {
        RuleId::N05
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Remove U+0670 ARABIC LETTER SUPERSCRIPT ALEF"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Deterministic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        transform(
            input,
            self.id(),
            |ch| {
                if ch == '\u{0670}' { CharOut::Drop } else { CharOut::Keep }
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
            &StripSuperscriptAlef,
            &[
                ("", ""),
                ("الرحمن", "الرحمن"),
                // الرَّحْمَٰنِ -> superscript alef dropped, rest kept:
                ("ٱلرَّحْمَٰنِ", "ٱلرَّحْمَنِ"),
            ],
        );
    }
}
