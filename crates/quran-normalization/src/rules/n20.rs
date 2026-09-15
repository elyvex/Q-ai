//! N20 `strip_preposition_prefix` — heuristic leading ب/ل/ك strip.
//!
//! Strips one leading `ب`, `ل`, or `ك` when the remainder holds **≥ 2 chars
//! starting with a letter**, applied to fixpoint (idempotent).
//!
//! Heuristic loss: radical initial ب/ل/ك is damaged (`بلد` → `لد` only when
//! the guard passes — it does, and that is exactly why the trace flags it).
//! `RuleKind::Heuristic`.

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind, SemVer};
use crate::rules::transform_mask;

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StripPrepositionPrefix;

impl NormalizationRule for StripPrepositionPrefix {
    fn id(&self) -> RuleId {
        RuleId::N20
    }

    fn version(&self) -> SemVer {
        SemVer::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Heuristically strip a leading preposition (ب/ل/ك) with ≥2-letter remainder"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Heuristic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        let chars: Vec<char> = input.text().chars().collect();
        let mut drop = 0;
        while chars.len() - drop > 2
            && matches!(chars[drop], 'ب' | 'ل' | 'ك')
            && chars[drop + 1].is_alphabetic()
        {
            drop += 1;
        }
        let keep: Vec<bool> = (0..chars.len()).map(|i| i >= drop).collect();
        transform_mask(input, self.id(), &keep)
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
            &StripPrepositionPrefix,
            &[
                ("", ""),
                ("بالبيت", "البيت"),
                ("لله", "له"),
                // Stacked prefixes strip to fixpoint:
                ("ككتاب", "تاب"),
                // Guard cases:
                ("ب", "ب"),
                ("بل", "بل"),
                ("به", "به"),
            ],
        );
    }

    #[test]
    fn kind_is_heuristic() {
        assert_eq!(StripPrepositionPrefix.kind(), RuleKind::Heuristic);
        assert!(RuleId::N20.is_heuristic());
    }
}
