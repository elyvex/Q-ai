//! N06 `normalize_hamza_forms` — fold hamza carriers to bare forms.
//!
//! Mapping table (default sub-options; configurable flags arrive with the
//! pipeline in M1b, defaults frozen here):
//!
//! | Input | Output |
//! |---|
//! | أ U+0623, إ U+0625, آ U+0622 | ا U+0627 |
//! | ؤ U+0624 | و U+0648 |
//! | ئ U+0626 | ي U+064A |
//! | ء U+0621 | ∅ (deleted; the "→ ا" sub-option is recorded for ADR-0204) |
//!
//! All 1:1 substitutions except standalone hamza (deletion). Idempotent.

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind};
use crate::rules::{CharOut, transform};

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NormalizeHamzaForms;

impl NormalizationRule for NormalizeHamzaForms {
    fn id(&self) -> RuleId {
        RuleId::N06
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Fold hamza carriers to bare alef/waw/yeh; standalone hamza deleted"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Deterministic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        transform(input, self.id(), |ch| match ch {
            '\u{0622}' | '\u{0623}' | '\u{0625}' => CharOut::One('\u{0627}'),
            '\u{0624}' => CharOut::One('\u{0648}'),
            '\u{0626}' => CharOut::One('\u{064A}'),
            '\u{0621}' => CharOut::Drop,
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
            &NormalizeHamzaForms,
            &[
                ("", ""),
                ("بِسْمِ", "بِسْمِ"),
                ("أإآ", "ااا"),
                ("ؤئ", "وي"),
                ("ء", ""),
                ("مؤمن", "مومن"),
                // Diacritics are N03's job: N06 keeps the fatha after folding.
                ("ٱلْأَرْضِ", "ٱلْاَرْضِ"),
            ],
        );
    }
}
