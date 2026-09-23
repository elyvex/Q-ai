//! Adversarial MV-rule fixtures: each `mv-NNN.json` violates exactly one
//! MV rule (MV-018 excepted — see below) and is rejected with its specific
//! MV id, while clean samples pass with zero fatals.
//!
//! MV-018 is a deferral marker, not a rejectable rule: its fixture is a
//! clean document asserting the marker fires as `Warn` with the documented
//! "deferred to build-time MV-018" text.

use quran_morphology::{
    InventoryToken, Severity, has_blocking, has_fatal, load_intermediate, validate,
};

fn fixture_path(name: &str) -> String {
    format!("../../fixtures/quran/adversarial/morphology/{name}.json")
}

fn inventory_for(
    rule_id: &str,
    doc: &quran_morphology::IntermediateMorphology,
) -> Vec<InventoryToken> {
    // MV-004 needs an inventory SHORTER than the claimed position; every
    // other fixture validates against its own positions so only the
    // targeted rule can fire.
    if rule_id == "MV-004" {
        return vec![(2, 1, 1, "كَتَبَ".to_string()), (2, 1, 2, "الْعِلْمُ".to_string())];
    }
    doc.analyses.iter().map(|a| (a.surah, a.ayah, a.token_position, a.surface.clone())).collect()
}

#[test]
fn each_adversarial_fixture_reports_its_specific_mv_id() {
    for n in 1..=17_u32 {
        let rule_id = format!("MV-{n:03}");
        let text = std::fs::read_to_string(fixture_path(&format!("mv-{n:03}")))
            .unwrap_or_else(|e| panic!("read mv-{n:03}.json: {e}"));
        let doc =
            load_intermediate(&text).unwrap_or_else(|e| panic!("mv-{n:03}.json must parse: {e}"));
        let findings = validate(&doc, &inventory_for(&rule_id, &doc));
        assert!(
            findings.iter().any(|f| f.rule_id == rule_id),
            "{rule_id}: no finding with the targeted id; got: {findings:?}"
        );
        let strays: Vec<_> =
            findings.iter().filter(|f| f.rule_id != rule_id && f.rule_id != "MV-018").collect();
        assert!(
            strays.is_empty(),
            "{rule_id}: fixture must violate exactly one rule; strays: {strays:?}"
        );
    }
}

#[test]
fn fatal_fixtures_block_import() {
    for n in [1_u32, 3, 10, 11] {
        let text = std::fs::read_to_string(fixture_path(&format!("mv-{n:03}")))
            .unwrap_or_else(|e| panic!("read: {e}"));
        let doc = load_intermediate(&text).expect("parse");
        let findings = validate(&doc, &inventory_for(&format!("MV-{n:03}"), &doc));
        assert!(has_fatal(&findings), "mv-{n:03} must carry a fatal finding: {findings:?}");
    }
}

#[test]
fn mv018_fixture_is_clean_but_carries_the_deferral_marker() {
    let text = std::fs::read_to_string(fixture_path("mv-018")).expect("read mv-018");
    let doc = load_intermediate(&text).expect("parse mv-018");
    let inventory = inventory_for("MV-018", &doc);
    let findings = validate(&doc, &inventory);
    assert!(!has_fatal(&findings), "{findings:?}");
    assert!(!has_blocking(&findings), "{findings:?}");
    let marker = findings.iter().find(|f| f.rule_id == "MV-018").expect("MV-018 marker present");
    assert_eq!(marker.severity, Severity::Warn);
    assert!(
        marker.detail.contains("deferred to build-time MV-018"),
        "marker text: {}",
        marker.detail
    );
}

#[test]
fn clean_adapter_samples_pass_with_zero_fatals() {
    use quran_morphology::{DatasetRef, parse_array_shape, parse_flat_csv};

    let dataset = DatasetRef::new("synthetic-morph-test", "0.1.0");
    let json = std::fs::read_to_string("../../fixtures/quran/lexicon/sample-a.json")
        .expect("read sample-a");
    let csv = std::fs::read_to_string("../../fixtures/quran/lexicon/sample-b.csv")
        .expect("read sample-b");
    for doc in [
        parse_array_shape(&json, &dataset, "test-edition-min").expect("parse sample-a"),
        parse_flat_csv(&csv, &dataset, "test-edition-min").expect("parse sample-b"),
    ] {
        let inventory: Vec<InventoryToken> = doc
            .analyses
            .iter()
            .map(|a| (a.surah, a.ayah, a.token_position, a.surface.clone()))
            .collect();
        let findings = validate(&doc, &inventory);
        assert!(!has_fatal(&findings), "{findings:?}");
        assert!(!has_blocking(&findings), "{findings:?}");
    }
}
