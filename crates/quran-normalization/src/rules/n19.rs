//! N19 `strip_conjunction_prefix` — heuristic leading و/ف strip.
//!
//! Strips one leading `و` or `ف` when the remainder holds **≥ 2 chars
//! starting with a letter**, applied to fixpoint (idempotent).
//!
//! Heuristic loss: a word-initial و/ف that is radical (part of the stem)
//! is damaged. `RuleKind::Heuristic`.

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind, SemVer};
use crate::rules::transform_mask;

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StripConjunctionPrefix;

impl NormalizationRule for StripConjunctionPrefix {
    fn id(&self) -> RuleId {
        RuleId::N19
    }

    fn version(&self) -> SemVer {
        SemVer::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Heuristically strip a leading conjunction (و/ف) with ≥2-letter remainder"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Heuristic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        let chars: Vec<char> = input.text().chars().collect();
        let mut drop = 0;
        while chars.len() - drop > 2
            && matches!(chars[drop], 'و' | 'ف')
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
            &StripConjunctionPrefix,
            &[
                ("", ""),
                ("والبيت", "البيت"),
                ("فكتب", "كتب"),
                // Guard: single letters and short remainders survive.
                ("و", "و"),
                ("وب", "وب"),
                ("في", "في"),
                ("فلسطين", "لسطين"),
            ],
        );
    }

    #[test]
    fn kind_is_heuristic() {
        assert_eq!(StripConjunctionPrefix.kind(), RuleKind::Heuristic);
        assert!(RuleId::N19.is_heuristic());
    }
}
