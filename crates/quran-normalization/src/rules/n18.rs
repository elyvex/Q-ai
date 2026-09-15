//! N18 `strip_definite_article` — heuristic leading-article strip.
//!
//! Strips a leading `ال` or `لل` when the remainder holds **≥ 2 chars
//! starting with a letter**. Applied to fixpoint (repeatedly while the guard
//! holds), which makes the rule idempotent: one more application changes
//! nothing.
//!
//! Heuristic loss (labeled in every trace): the article is a genuine
//! morpheme — `الله` folds toward `له`, and words whose first two letters
//! merely look like the article are damaged. True morphological search is the
//! lexicon path (§7), never this rule. `RuleKind::Heuristic`.

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind, SemVer};
use crate::rules::transform_mask;

/// Prefixes stripped by N18, in match order.
const PREFIXES: [&str; 2] = ["ال", "لل"];

/// Strip one article prefix per the guard, returning the survivor count.
/// Operates on chars so spans stay exact.
fn strip_once(chars: &[char]) -> Option<usize> {
    for prefix in PREFIXES {
        let len = prefix.chars().count();
        if chars.len() > len
            && chars[..len].iter().collect::<String>() == prefix
            && chars.len() - len >= 2
            && chars[len].is_alphabetic()
        {
            return Some(len);
        }
    }
    None
}

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StripDefiniteArticle;

impl NormalizationRule for StripDefiniteArticle {
    fn id(&self) -> RuleId {
        RuleId::N18
    }

    fn version(&self) -> SemVer {
        SemVer::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Heuristically strip a leading definite article (ال/لل) with ≥2-letter remainder"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Heuristic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        let chars: Vec<char> = input.text().chars().collect();
        let mut drop = 0;
        while let Some(len) = strip_once(&chars[drop..]) {
            drop += len;
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
            &StripDefiniteArticle,
            &[
                ("", ""),
                ("الكتاب", "كتاب"),
                ("للبيت", "بيت"),
                // Guard: remainder must hold ≥ 2 chars starting with a letter.
                ("الx", "الx"),
                ("الأ", "الأ"),
                ("ال", "ال"),
                ("له", "له"),
                // Fixpoint: repeated articles strip until the guard stops.
                ("الالكتاب", "كتاب"),
            ],
        );
    }

    #[test]
    fn kind_is_heuristic() {
        assert_eq!(StripDefiniteArticle.kind(), RuleKind::Heuristic);
        assert!(RuleId::N18.is_heuristic());
    }
}
