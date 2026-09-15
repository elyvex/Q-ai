//! Phase 2 — pipeline, profiles, and traces: cross-cutting properties.
//!
//! The query path and the index path share one pipeline instance; these tests
//! pin the ladder shape, trace discipline, and determinism every consumer
//! relies on.

use quran_normalization::{
    NormalizationPipeline, NormalizationTrace, ProfileId, ProfileRegistry, RuleId, SemVer,
};

fn v1() -> SemVer {
    SemVer::new(1, 0, 0)
}

#[test]
fn l0_is_identity_with_empty_trace() {
    let pipeline =
        NormalizationPipeline::for_profile(&ProfileRegistry::new(), ProfileId::L0, v1()).unwrap();
    let (text, trace) = pipeline.apply("بِسْمِ ٱللَّهِ");
    assert_eq!(text.text(), "بِسْمِ ٱللَّهِ");
    assert!(trace.rules_applied.is_empty());
    assert!(!trace.contains_heuristic_rules);
    assert_eq!(trace.profile, "L0.exact@1.0.0");
    // Identity spans round-trip exactly.
    let span = text.spans().to_canonical(0..text.len_chars());
    assert_eq!(span.char_range, 0..text.len_chars());
    assert!(span.exact);
}

#[test]
fn pipeline_is_deterministic() {
    let pipeline =
        NormalizationPipeline::for_profile(&ProfileRegistry::new(), ProfileId::L5, v1()).unwrap();
    let first = pipeline.apply("ٱلرَّحْمَٰنِ ٱلرَّحِيمِ");
    let second = pipeline.apply("ٱلرَّحْمَٰنِ ٱلرَّحِيمِ");
    assert_eq!(first.0.text(), second.0.text());
    assert_eq!(first.1, second.1);
    assert_eq!(first.0.text(), "الرحمن الرحيم");
}

#[test]
fn every_indexed_profile_has_a_distinct_label() {
    let registry = ProfileRegistry::new();
    let mut labels = std::collections::BTreeSet::new();
    for id in ProfileId::all() {
        let profile = registry.latest(id).unwrap();
        assert!(labels.insert(profile.trace_label()), "duplicate label for {id}");
    }
    assert_eq!(labels.len(), 9);
}

#[test]
fn heuristic_profiles_flag_and_deterministic_ones_do_not() {
    let registry = ProfileRegistry::new();
    for id in ProfileId::all() {
        let pipeline = NormalizationPipeline::for_profile(&registry, id, v1()).unwrap();
        let (_, trace) = pipeline.apply("والكتاب");
        let profile = registry.latest(id).unwrap();
        assert_eq!(
            trace.contains_heuristic_rules, profile.heuristic,
            "trace flag disagrees with profile {id}"
        );
    }
}

#[test]
fn trace_is_required_for_results() {
    // The constructor guard every result type reuses: no blank profile.
    assert!(NormalizationTrace::new("", vec![]).is_err());
    assert!(NormalizationTrace::new("L3.diacritics@1.0.0", vec![]).is_ok());
}

#[test]
fn l7_affix_end_to_end_on_basmala_word() {
    let pipeline =
        NormalizationPipeline::for_profile(&ProfileRegistry::new(), ProfileId::L7, v1()).unwrap();
    // "and the books": conjunction stripped, article kept (single pass),
    // no pronoun to strip.
    let (text, trace) = pipeline.apply("والكتب");
    assert_eq!(text.text(), "الكتب");
    assert!(trace.contains_heuristic_rules);
    assert!(trace.rule_ids().contains(&RuleId::N19));
}
