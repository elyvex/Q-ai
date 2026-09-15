//! N14 `strip_pause_marks` — remove waqf (pause) letters.
//!
//! Removed: U+06D6–U+06DB (ۖ ۗ ۘ ۙ ۚ ۛ). This is the subset of N04 that callers
//! address separately to express "keep marks but drop waqf". Pure deletion.
//! Idempotent.

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind, SemVer};
use crate::rules::{CharOut, transform};

/// True for code points removed by N14 (a strict subset of N04).
#[must_use]
pub fn is_pause_mark(ch: char) -> bool {
    matches!(ch, '\u{06D6}'..='\u{06DB}')
}

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StripPauseMarks;

impl NormalizationRule for StripPauseMarks {
    fn id(&self) -> RuleId {
        RuleId::N14
    }

    fn version(&self) -> SemVer {
        SemVer::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Remove waqf pause letters (U+06D6-U+06DB)"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Deterministic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        transform(
            input,
            self.id(),
            |ch| {
                if is_pause_mark(ch) { CharOut::Drop } else { CharOut::Keep }
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
            &StripPauseMarks,
            &[
                ("", ""),
                // N14 drops the waqf letters but keeps other N04 marks:
                ("نص\u{06D6}\u{06DC}", "نص\u{06DC}"),
                ("ۖۗۘۙۚۛ", ""),
            ],
        );
    }

    #[test]
    fn subset_of_n04() {
        // Every N14 victim is also an N04 victim (plan §3.2: "subset of N04").
        for cp in '\u{06D6}'..='\u{06DB}' {
            assert!(crate::rules::n04::is_quranic_mark(cp), "{cp:?}");
        }
    }
}
