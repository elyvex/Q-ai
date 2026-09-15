//! Phase 2 — deterministic normalization rules N01–N17 (`plan.md` §3.2).
//!
//! Each rule is a pure, total, order-significant character transformation
//! emitting its own [`SpanMap`](crate::span::SpanMap). Mapping tables are
//! documented on each rule struct; linguistic review of the catalog is
//! ADR-0204 (DRAFT until the linguist signs off, P2-X02).
//!
//! Rules N18–N24 (heuristic / reserved) are not implemented here.

pub mod n01;
pub mod n02;
pub mod n03;
pub mod n04;
pub mod n05;
pub mod n06;
pub mod n07;
pub mod n08;
pub mod n09;
pub mod n10;
pub mod n11;
pub mod n12;
pub mod n13;
pub mod n14;
pub mod n15;
pub mod n16;
pub mod n17;

pub use n01::WhitespaceCollapse;
pub use n02::StripTatweel;
pub use n03::StripHarakat;
pub use n04::StripQuranicMarks;
pub use n05::StripSuperscriptAlef;
pub use n06::NormalizeHamzaForms;
pub use n07::NormalizeWasla;
pub use n08::NormalizeAlifMaqsura;
pub use n09::NormalizeTaMarbuta;
pub use n10::NormalizePersianCodepoints;
pub use n11::StripZeroWidthAndBidi;
pub use n12::StripPunctuation;
pub use n13::FoldDigits;
pub use n14::StripPauseMarks;
pub use n15::ExpandPresentationForms;
pub use n16::NfcCompose;
pub use n17::RemoveSpaces;

use crate::rule::{NormalizationRule, NormalizedText, RuleId};
use crate::span::SpanMap;

/// Every deterministic rule in catalog order (N01–N17).
#[must_use]
pub fn all_rules() -> Vec<Box<dyn NormalizationRule>> {
    vec![
        Box::new(WhitespaceCollapse),
        Box::new(StripTatweel),
        Box::new(StripHarakat),
        Box::new(StripQuranicMarks),
        Box::new(StripSuperscriptAlef),
        Box::new(NormalizeHamzaForms),
        Box::new(NormalizeWasla),
        Box::new(NormalizeAlifMaqsura),
        Box::new(NormalizeTaMarbuta),
        Box::new(NormalizePersianCodepoints),
        Box::new(StripZeroWidthAndBidi),
        Box::new(StripPunctuation),
        Box::new(FoldDigits),
        Box::new(StripPauseMarks),
        Box::new(ExpandPresentationForms),
        Box::new(NfcCompose),
        Box::new(RemoveSpaces),
    ]
}

/// Look up a deterministic rule implementation by id.
///
/// Returns `None` for heuristic (N18–N22) and reserved (N23–N24) ids, which
/// have no implementation in this module.
#[must_use]
pub fn by_id(id: RuleId) -> Option<Box<dyn NormalizationRule>> {
    all_rules().into_iter().find(|r| r.id() == id)
}

/// Per-character rewrite outcome for [`transform`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CharOut {
    /// Drop the character (deletion rules N02–N05, N11, N12, N14, N17).
    Drop,
    /// Keep the character unchanged.
    Keep,
    /// Replace with exactly one character (1:1 folds N06–N10, N13).
    One(char),
    /// Replace with exactly two characters (N15 expansions).
    Two(char, char),
}

/// Apply a per-character rewrite, building the derived text and its [`SpanMap`].
///
/// Every emitted char records the input index that produced it, so the map is
/// total by construction. The result map is composed with the input's map, so
/// [`NormalizedText`] always points back to the pipeline input. N01 (which
/// merges runs) and N16 (which merges sequences) implement their own loops.
pub(crate) fn transform(
    input: &NormalizedText,
    rule: RuleId,
    map: impl Fn(char) -> CharOut,
) -> NormalizedText {
    let mut text = String::new();
    let mut forward: Vec<u32> = Vec::new();
    for (i, ch) in input.text().chars().enumerate() {
        let i = i as u32;
        match map(ch) {
            CharOut::Drop => {}
            CharOut::Keep => {
                text.push(ch);
                forward.push(i);
            }
            CharOut::One(out) => {
                text.push(out);
                forward.push(i);
            }
            CharOut::Two(a, b) => {
                text.push(a);
                text.push(b);
                forward.push(i);
                forward.push(i);
            }
        }
    }
    let canonical_len = input.text().chars().count() as u32;
    let rule_map = SpanMap::build(canonical_len, forward, rule)
        .expect("transform builds a total map by construction");
    let spans = input.spans().clone().compose(&rule_map);
    NormalizedText::from_parts(text, spans)
}

#[cfg(test)]
pub(crate) mod test_support {
    //! Shared assertions for rule unit tests.
    //!
    //! `check` verifies the mapping table, idempotency, and the M1-level
    //! SpanMap round-trip (property 5 of `plan.md` §3.4 at single-rule scope:
    //! slicing the input at the mapped-back hull and re-applying the rule
    //! reproduces the derived text).

    use crate::rule::{NormalizationRule, NormalizedText};

    /// Assert `rule` maps each `(input, expected)` pair, is idempotent, and
    /// keeps offsets reversible on every case.
    pub fn check(rule: &dyn NormalizationRule, cases: &[(&str, &str)]) {
        assert!(rule.is_idempotent(), "{} must declare idempotency", rule.id());
        for (input, expected) in cases {
            let first = rule.apply(&NormalizedText::from_plain(input));
            assert_eq!(first.text(), *expected, "{}({input:?})", rule.id());
            let second = rule.apply(&first);
            assert_eq!(second.text(), *expected, "{} is not idempotent on {input:?}", rule.id());
            // Hull round-trip: map the whole derived range back, re-apply the
            // rule to the sliced input, and require the derived text back.
            // Holds for deletion/1:1/expansion rules (all but N16, which has
            // dedicated tests because composition can merge across slices).
            let span = first.spans().to_canonical(0..first.len_chars());
            let hull: String = input
                .chars()
                .enumerate()
                .filter(|(i, _)| {
                    *i as u32 >= span.char_range.start && (*i as u32) < span.char_range.end
                })
                .map(|(_, c)| c)
                .collect();
            let reapplied = rule.apply(&NormalizedText::from_plain(&hull));
            assert_eq!(
                reapplied.text(),
                *expected,
                "{} hull round-trip failed on {input:?}",
                rule.id()
            );
        }
    }
}
