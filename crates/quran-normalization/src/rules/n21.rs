//! N21 `strip_pronoun_suffix` — heuristic trailing-pronoun strip.
//!
//! Strips one trailing pronoun from the table below when the stem keeps
//! **≥ 2 chars**, longest match first, applied to fixpoint (idempotent).
//!
//! | Suffix | Example |
//! |---|
//! | هما | كتابهما → كتاب |
//! | هم، هن، ها | كتابهم → كتاب |
//! | كم، كن | كتابكم → كتاب |
//! | ني، نا | كتابني → كتاب |
//! | ه، ك، ي | كتابه → كتاب |
//!
//! Heuristic loss: stem-final ه/ك/ي that is radical is damaged (`التي` →
//! `الت`). The guard keeps two-letter words whole (`في`, `هي`, `به`).
//! `RuleKind::Heuristic`.

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind, SemVer};
use crate::rules::transform_mask;

/// Suffixes stripped by N21, longest first (match order matters).
const SUFFIXES: [&str; 11] = ["هما", "هم", "هن", "ها", "كم", "كن", "ني", "نا", "ه", "ك", "ي"];

/// Strip one suffix per the guard, returning its char length.
fn strip_once(chars: &[char]) -> Option<usize> {
    for suffix in SUFFIXES {
        let len = suffix.chars().count();
        if chars.len() > len
            && chars.len() - len >= 2
            && chars[chars.len() - len..].iter().collect::<String>() == suffix
        {
            return Some(len);
        }
    }
    None
}

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StripPronounSuffix;

impl NormalizationRule for StripPronounSuffix {
    fn id(&self) -> RuleId {
        RuleId::N21
    }

    fn version(&self) -> SemVer {
        SemVer::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Heuristically strip a trailing pronoun with ≥2-char stem"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Heuristic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        let chars: Vec<char> = input.text().chars().collect();
        let mut keep_len = chars.len();
        while let Some(len) = strip_once(&chars[..keep_len]) {
            keep_len -= len;
        }
        let keep: Vec<bool> = (0..chars.len()).map(|i| i < keep_len).collect();
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
            &StripPronounSuffix,
            &[
                ("", ""),
                ("كتابه", "كتاب"),
                ("كتابهم", "كتاب"),
                ("كتابهما", "كتاب"),
                ("كتابكم", "كتاب"),
                ("كتابني", "كتاب"),
                ("كتابي", "كتاب"),
                ("كتابك", "كتاب"),
                // Guard: stems shorter than 2 chars survive.
                ("في", "في"),
                ("هي", "هي"),
                ("به", "به"),
                ("ما", "ما"),
                // Fixpoint across stacked suffixes:
                ("كتابهه", "كتاب"),
                // Documented heuristic damage:
                ("التي", "الت"),
            ],
        );
    }

    #[test]
    fn kind_is_heuristic() {
        assert_eq!(StripPronounSuffix.kind(), RuleKind::Heuristic);
        assert!(RuleId::N21.is_heuristic());
    }
}
