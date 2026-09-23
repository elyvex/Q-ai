//! Family + policy + compare + tagset integration tests.

use quran_morphology::{
    AnalysisPolicy, DatasetRef, FamilyMember, FamilyRelation, PolicyAnalysis, ReviewPromotion,
    SUGGESTION_LABEL, TokenAnalysis, UnifiedTag, Verdict, apply_policy, compare, explain_relation,
    map_tag, suggest_computational,
};

fn token(surface: &str, lemma: &str, root: &str, native: &str, unified: &str) -> TokenAnalysis {
    TokenAnalysis {
        surah: 2,
        ayah: 1,
        token_position: 1,
        analysis_index: 0,
        surface: surface.to_string(),
        lemma: lemma.to_string(),
        root: root.to_string(),
        stem: root.to_string(),
        pos_unified: unified.to_string(),
        pos_native: native.to_string(),
        features_json: serde_json::json!({}),
        segments: Vec::new(),
        provenance_layer: "B".to_string(),
        algorithm: String::new(),
        algorithm_version: String::new(),
        confidence: None,
        reviewer: String::new(),
        status: "imported".to_string(),
    }
}

#[test]
fn explanation_is_mandatory_for_family_members() {
    assert!(FamilyMember::new("m1", "token", "").is_err());
    let member = FamilyMember::new("m1", "token", "shares root كتب").expect("valid");
    assert_eq!(member.explanation, "shares root كتب");
}

#[test]
fn suggestions_are_off_without_opt_in() {
    assert!(suggest_computational("s", "a", "b", 0.9, false).is_err());
    assert!(suggest_computational("s", "a", "b", 0.2, true).is_err());
    let suggestion = suggest_computational("s", "a", "b", 0.9, true).expect("opted in");
    assert_eq!(suggestion.label, SUGGESTION_LABEL);
    // Promotion still needs reviewer + timestamp + evidence.
    assert!(ReviewPromotion::new("s", "", "t", serde_json::json!({})).is_err());
    assert!(
        ReviewPromotion::new(
            "s",
            "linguist-1",
            "2026-01-01T00:00:00Z",
            serde_json::json!({"note": "x"})
        )
        .is_ok()
    );
}

#[test]
fn compare_has_no_resolution_field() {
    let verdict =
        quran_morphology::FieldVerdict { field: "root".to_string(), verdict: Verdict::Conflicting };
    let value = serde_json::to_value(&verdict).expect("serialize");
    let object = value.as_object().expect("object");
    let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["field", "verdict"], "no resolution/synthesis field allowed");
}

#[test]
fn compare_reports_identity_and_conflict() {
    let a = token("كَتَبَ", "كَتَبَ", "كتب", "V_perf", "Verb");
    let b = token("كِتَاب", "كِتَاب", "كتب", "N", "Noun");
    let verdicts = compare(&a, &b);
    let root = verdicts.iter().find(|v| v.field == "root").expect("root");
    assert_eq!(root.verdict, Verdict::Identical);
    let lemma = verdicts.iter().find(|v| v.field == "lemma").expect("lemma");
    assert_eq!(lemma.verdict, Verdict::Conflicting);
    let text = explain_relation(&a, &b, FamilyRelation::SameRoot);
    assert!(text.contains("same_root") && text.contains("differing"), "{text}");
}

#[test]
fn suppression_is_always_counted() {
    let policy = AnalysisPolicy::new(vec![DatasetRef::new("kept", "1")], false);
    let rows = vec![
        PolicyAnalysis::new("1", DatasetRef::new("kept", "1"), serde_json::json!({})),
        PolicyAnalysis::new("2", DatasetRef::new("dropped", "1"), serde_json::json!({})),
    ];
    let view = apply_policy(rows, &policy);
    assert_eq!(view.returned.len(), 1);
    assert_eq!(view.suppressed_count, 1);
    assert_eq!(view.analyses_suppressed.len(), view.suppressed_count);
}

#[test]
fn unmapped_tags_preserve_native_without_guessing() {
    let mapping =
        quran_morphology::TagMapping::new("test-tags-0.1", [("V_perf", UnifiedTag::Verb)]);
    let (tag, native) = map_tag("V_perf", &mapping);
    assert_eq!((tag, native.as_str()), (UnifiedTag::Verb, "V_perf"));
    let (tag, native) = map_tag("X_unknown", &mapping);
    assert_eq!((tag, native.as_str()), (UnifiedTag::Unmapped, "X_unknown"));
}
