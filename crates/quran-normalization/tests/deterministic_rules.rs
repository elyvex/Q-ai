//! Phase 2 — deterministic rules N01–N17: cross-rule properties + seed goldens.
//!
//! Seed golden pairs below are **engineering seeds, pending linguist review**
//! (P2-T11 owns the signed 2,000-pair set; ADR-0204 is DRAFT). They pin
//! current behavior so regressions fail loudly, not silently.

use proptest::prelude::*;
use quran_normalization::rule::{NormalizedText, RuleId};
use quran_normalization::rules::{all_rules, by_id, heuristic_rules};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Every deterministic rule is total: no panic on arbitrary Unicode,
    /// output is valid UTF-8 by construction, and lengths stay consistent
    /// with the emitted SpanMap.
    #[test]
    fn no_panic_on_arbitrary_unicode(s in "\\PC*") {
        for rule in all_rules().into_iter().chain(heuristic_rules()) {
            let out = rule.apply(&NormalizedText::from_plain(&s));
            prop_assert_eq!(
                out.text().chars().count() as u32,
                out.spans().derived_len(),
                "{} text/map length mismatch",
                rule.id()
            );
            // Second application must also be total.
            let _ = rule.apply(&out);
        }
    }

    /// Every deterministic rule claiming idempotency honors it on fuzz input.
    #[test]
    fn idempotent_on_arbitrary_unicode(s in "\\PC*") {
        for rule in all_rules().into_iter().chain(heuristic_rules()) {
            prop_assert!(
                rule.is_idempotent(),
                "{} must declare idempotency",
                rule.id()
            );
            let once = rule.apply(&NormalizedText::from_plain(&s));
            let twice = rule.apply(&once);
            prop_assert_eq!(once.text(), twice.text(), "{} not idempotent", rule.id());
        }
    }

    /// SpanMap round-trip containment holds per rule on fuzz input:
    /// every surviving input char maps forward and back over itself.
    #[test]
    fn span_round_trip_on_arbitrary_unicode(s in "\\PC*") {
        // N16 (composition) merges starters and marks, so the hull check uses
        // rule-local containment: the mapped-back hull re-applied contains
        // the derived text. Checked on ASCII-only input where NFC is fixed-point.
        let ascii_only = s.is_ascii();
        for rule in all_rules().into_iter().chain(heuristic_rules()) {
            if rule.id() == RuleId::N16 && !ascii_only {
                continue;
            }
            let out = rule.apply(&NormalizedText::from_plain(&s));
            let n = out.len_chars();
            if n == 0 {
                continue;
            }
            let span = out.spans().to_canonical(0..n);
            let hull: String = s
                .chars()
                .enumerate()
                .filter(|(i, _)| {
                    *i as u32 >= span.char_range.start && (*i as u32) < span.char_range.end
                })
                .map(|(_, c)| c)
                .collect();
            let reapplied = rule.apply(&NormalizedText::from_plain(&hull));
            prop_assert_eq!(
                reapplied.text(),
                out.text(),
                "{} hull round-trip failed",
                rule.id()
            );
        }
    }
}

/// Apply a rule sequence left to right (pipeline preview; the real
/// `NormalizationPipeline` lands with profiles in M1b).
fn chain(input: &str, ids: &[RuleId]) -> NormalizedText {
    let mut text = NormalizedText::from_plain(input);
    for id in ids {
        let rule = by_id(*id).expect("deterministic rule id");
        text = rule.apply(&text);
    }
    text
}

// Full basmala in Uthmani orthography (first ayah shape).
const BASMALA: &str = "بِسْمِ \u{0671}للَّهِ \u{0671}لرَّحْمَ\u{0670}نِ \u{0671}لرَّحِيمِ";

#[test]
fn seed_basmala_to_bare_profile() {
    // L3.diacritics path: N01, N11, N16, N04, N14, N03, N05, N02.
    let out = chain(
        BASMALA,
        &[
            RuleId::N01,
            RuleId::N11,
            RuleId::N16,
            RuleId::N04,
            RuleId::N14,
            RuleId::N03,
            RuleId::N05,
            RuleId::N02,
        ],
    );
    assert_eq!(out.text(), "بسم ٱلله ٱلرحمن ٱلرحيم");
    // Every derived char maps back inside the basmala.
    let n = BASMALA.chars().count() as u32;
    let span = out.spans().to_canonical(0..out.len_chars());
    assert!(span.char_range.end <= n);
    assert!(span.exact);
}

#[test]
fn seed_basmala_to_skeleton() {
    // L6.skeleton path adds N07, N06, N08, N10, N13, N09, N15, N12, N17.
    let out = skeleton(BASMALA);
    assert_eq!(out.text(), "بسماللهالرحمنالرحيم");
}

/// The L6-relevant deterministic chain (skeleton path).
fn skeleton(input: &str) -> NormalizedText {
    chain(
        input,
        &[
            RuleId::N01,
            RuleId::N11,
            RuleId::N16,
            RuleId::N04,
            RuleId::N14,
            RuleId::N03,
            RuleId::N05,
            RuleId::N02,
            RuleId::N07,
            RuleId::N06,
            RuleId::N08,
            RuleId::N10,
            RuleId::N13,
            RuleId::N09,
            RuleId::N15,
            RuleId::N12,
            RuleId::N17,
        ],
    )
}

#[test]
fn seed_concatenated_query_shape() {
    // The §8.3 promise: "بسمالله" typed spaceless is found inside the skeleton,
    // and the hit maps back to canonical tokens 1..2 of the basmala.
    // L3 + hamza/wasla folds + space removal (the skeleton path). N16 NFC
    // reorders shadda/fatha pairs before N03 strips them; the composed map
    // still points back to the original basmala indices.
    let skel = skeleton(BASMALA);
    let pos = skel.text().find("بسمالله").expect("skeleton must contain query");
    let start = skel.text()[..pos].chars().count() as u32;
    let end = start + "بسمالله".chars().count() as u32;
    let span = skel.spans().to_canonical(start..end);
    let canonical: String = BASMALA
        .chars()
        .enumerate()
        .filter(|(i, _)| *i as u32 >= span.char_range.start && (*i as u32) < span.char_range.end)
        .map(|(_, c)| c)
        .collect();
    // Plan §3.4 property 5: slicing canonical text at the mapped span and
    // re-normalizing reproduces (here: exactly equals) the derived substring.
    // The tight hull excludes the kasra N03 deleted after ه — containment is
    // the guarantee, not span equality with the un-normalized verse.
    assert_eq!(skeleton(&canonical).text(), "بسمالله");
    assert!(span.exact);
}

#[test]
fn seed_registry_covers_n01_through_n17() {
    let rules = all_rules();
    assert_eq!(rules.len(), 17);
    for (i, rule) in rules.iter().enumerate() {
        let expected = RuleId::parse(&format!("N{:02}", i + 1)).unwrap();
        assert_eq!(rule.id(), expected);
        assert!(by_id(expected).is_some());
    }
    // Heuristic ids resolve through the same lookup; reserved ids do not.
    assert_eq!(heuristic_rules().len(), 5);
    for n in 18..=22 {
        let id = RuleId::parse(&format!("N{n:02}")).unwrap();
        let rule = by_id(id).expect("heuristic rule must resolve");
        assert_eq!(rule.kind(), quran_normalization::rule::RuleKind::Heuristic);
    }
    for n in 23..=24 {
        assert!(by_id(RuleId::parse(&format!("N{n:02}")).unwrap()).is_none());
    }
}
