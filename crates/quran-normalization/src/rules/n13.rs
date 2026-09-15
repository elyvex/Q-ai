//! N13 `fold_digits` — Eastern digits → ASCII.
//!
//! Mapping table: ٠–٩ U+0660–U+0669 and ۰–۹ U+06F0–U+06F9 → `0`–`9`.
//! For reference/number queries. 1:1 substitution. Idempotent.

use crate::rule::{NormalizationRule, NormalizedText, RuleId, RuleKind, SemVer};
use crate::rules::{CharOut, transform};

/// See the [module](crate::rules) documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoldDigits;

impl NormalizationRule for FoldDigits {
    fn id(&self) -> RuleId {
        RuleId::N13
    }

    fn version(&self) -> SemVer {
        SemVer::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Fold Arabic-Indic and Extended Arabic-Indic digits to ASCII"
    }

    fn kind(&self) -> RuleKind {
        RuleKind::Deterministic
    }

    fn apply(&self, input: &NormalizedText) -> NormalizedText {
        transform(input, self.id(), |ch| match ch {
            '\u{0660}'..='\u{0669}' => {
                CharOut::One((u32::from(ch) - 0x0660 + u32::from('0')) as u8 as char)
            }
            '\u{06F0}'..='\u{06F9}' => {
                CharOut::One((u32::from(ch) - 0x06F0 + u32::from('0')) as u8 as char)
            }
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
            &FoldDigits,
            &[
                ("", ""),
                ("٢٥٥", "255"),
                ("۱۲۳", "123"),
                ("2:255", "2:255"),
                ("آية ٢٥٥ و ٣", "آية 255 و 3"),
            ],
        );
    }
}
