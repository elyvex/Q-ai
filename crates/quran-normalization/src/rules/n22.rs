//! N22 `dedupe_repeated_letters` — collapse runs of ≥3 identical chars to 2.
//!
//! Experimental, off by default (only active in L8 query-time handling, which
//! itself ships experimental and disabled). Exactly-2 runs are kept as-is.
//! Comparison is per Unicode scalar; combining marks break runs (documented
//! v1 limitation, pending ADR-0204). Idempotent.

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind, SemVer};
use crate::rules::transform_mask;

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DedupeRepeatedLetters;

impl NormalizationRule for DedupeRepeatedLetters {
    fn id(&self) -> RuleId {
        RuleId::N22
    }

    fn version(&self) -> SemVer {
        SemVer::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Experimentally collapse runs of 3+ identical letters to 2"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Heuristic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        let chars: Vec<char> = input.text().chars().collect();
        let mut keep = vec![true; chars.len()];
        let mut i = 0;
        while i < chars.len() {
            let mut j = i + 1;
            while j < chars.len() && chars[j] == chars[i] {
                j += 1;
            }
            // Run is chars[i..j]; keep at most the first two.
            for slot in keep.iter_mut().take(j).skip(i + 2) {
                *slot = false;
            }
            i = j;
        }
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
            &DedupeRepeatedLetters,
            &[
                ("", ""),
                ("abc", "abc"),
                ("مرررحبا", "مررحبا"),
                ("aa", "aa"),
                ("aaa", "aa"),
                ("aaaa", "aa"),
                ("aabbbcc", "aabbcc"),
            ],
        );
    }

    #[test]
    fn kind_is_heuristic() {
        assert_eq!(DedupeRepeatedLetters.kind(), RuleKind::Heuristic);
        assert!(RuleId::N22.is_heuristic());
    }
}
