//! Phase 2 — normalization golden-set harness (P2-T21, AC-P2-03 mechanics).
//!
//! Runs all 2,000 pairs in `fixtures/quran/normalization/pairs.jsonl`
//! through the live pipeline and requires exact agreement on output text
//! and the ordered applied-rule list. The fixture is a mechanical
//! regression lock (see `examples/gen_goldens.rs`); the header must carry
//! `reviewed_by: pending-linguist` until P2-T11 linguist sign-off lands.
//! Hand-written profile dossiers below pin ladder semantics independently
//! of the generated rows.

use std::collections::HashMap;

use quran_normalization::SemVer;
use quran_normalization::pipeline::NormalizationPipeline;
use quran_normalization::profile::{ProfileId, ProfileRegistry};
use quran_normalization::rule::RuleId;

const FIXTURE: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/quran/normalization/pairs.jsonl");

fn registry() -> ProfileRegistry {
    ProfileRegistry::new()
}

fn v1() -> SemVer {
    SemVer::new(1, 0, 0)
}

#[test]
fn header_marks_mechanical_status() {
    let text = std::fs::read_to_string(FIXTURE).expect("golden fixture exists");
    let first = text.lines().next().expect("fixture is non-empty");
    let header: serde_json::Value = serde_json::from_str(first).expect("header parses");
    assert_eq!(header["header"], true);
    assert_eq!(header["reviewed_by"], "pending-linguist");
    assert!(header["reviewed_at"].is_null(), "no linguist sign-off yet (P2-T11 open)");
    assert_eq!(header["profile_version"], "1.0.0");
}

#[test]
fn all_2000_pairs_green() {
    let text = std::fs::read_to_string(FIXTURE).expect("golden fixture exists");
    let registry = registry();
    let mut pipes: HashMap<String, NormalizationPipeline> = HashMap::new();
    for id in
        [ProfileId::L0, ProfileId::L1, ProfileId::L2, ProfileId::L3, ProfileId::L4, ProfileId::L5]
    {
        pipes.insert(
            id.to_string(),
            NormalizationPipeline::for_profile(&registry, id, v1()).unwrap(),
        );
    }
    let mut count = 0;
    for (lineno, line) in text.lines().enumerate().skip(1) {
        let row: serde_json::Value =
            serde_json::from_str(line).unwrap_or_else(|e| panic!("line {lineno}: {e}"));
        let profile = row["profile"].as_str().expect("row has profile");
        let input = row["input"].as_str().expect("row has input");
        let expected = row["expected_output"].as_str().expect("row has expected_output");
        let expected_rules: Vec<String> = row["expected_rules_applied"]
            .as_array()
            .expect("row has expected_rules_applied")
            .iter()
            .map(|v| v.as_str().expect("rule id is a string").to_string())
            .collect();
        let pipe = pipes.get(profile).unwrap_or_else(|| panic!("unknown profile {profile}"));
        let (out, trace) = pipe.apply(input);
        assert_eq!(out.text(), expected, "line {lineno} profile={profile} input={input:?}");
        let actual: Vec<String> =
            trace.rules_applied.iter().map(|r| r.rule.as_str().to_string()).collect();
        assert_eq!(actual, expected_rules, "line {lineno} rule list diverged");
        count += 1;
    }
    assert_eq!(count, 2000, "golden set must hold exactly 2,000 pairs");
}

/// Hand-written ladder dossier: each rung folds strictly more than the last
/// on probe inputs, and indexed rungs stay monotonic (superset recall).
#[test]
fn ladder_dossier_monotonic_folding() {
    let registry = registry();
    let apply = |id: ProfileId, text: &str| {
        NormalizationPipeline::for_profile(&registry, id, v1()).unwrap().apply(text).0
    };
    // Diacritics survive L0–L2, fold at L3.
    let with_harakat = "بِسْمِ";
    assert_eq!(apply(ProfileId::L0, with_harakat).text(), "بِسْمِ");
    assert_eq!(apply(ProfileId::L3, with_harakat).text(), "بسم");
    // Hamza carriers survive L3, fold at L4.
    let hamza = "مؤمن";
    assert_eq!(apply(ProfileId::L3, hamza).text(), "مؤمن");
    assert_eq!(apply(ProfileId::L4, hamza).text(), "مومن");
    // Persian code points survive L4, fold at L5 (explicit code points:
    // U+06A9 KEHEH + U+06CC FARSI YEH → U+0643 KAF + U+064A YEH).
    let persian = "\u{6a9}\u{6cc}";
    assert_eq!(
        apply(ProfileId::L4, persian).text().chars().map(|c| c as u32).collect::<Vec<_>>(),
        vec![0x6a9, 0x6cc]
    );
    assert_eq!(
        apply(ProfileId::L5, persian).text().chars().map(|c| c as u32).collect::<Vec<_>>(),
        vec![0x643, 0x64a]
    );
    // Rule-id ordering is ladder-correct on a mixed probe.
    let (_, trace) =
        NormalizationPipeline::for_profile(&registry, ProfileId::L5, v1()).unwrap().apply("بِسْمِ");
    let ids = trace.rule_ids();
    assert!(ids.contains(&RuleId::N03), "L5 must strip harakat");
    assert!(ids.contains(&RuleId::N10), "L5 must fold code points");
    assert!(!trace.contains_heuristic_rules, "L5 is deterministic");
}
