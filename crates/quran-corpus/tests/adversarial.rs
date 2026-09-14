//! Phase 1 — adversarial-corpus rejection (P1-T30, AC-P1-04).
//!
//! Each of the 16 adversarial fixtures must be rejected with its **specific**
//! rule id — a validator that rejects everything with one code fails AC-P1-04.

use quran_corpus::adapters::EditionAdapter;
use quran_corpus::{JsonAdapter, Severity, validate_edition};
fn manifest(name: &str) -> String {
    std::fs::read_to_string(format!("../../fixtures/quran/adversarial/{name}/manifest.json"))
        .unwrap_or_else(|_| panic!("missing adversarial fixture {name}"))
}

fn expect_rule(name: &str, rule: &str) {
    let source = JsonAdapter
        .parse(&manifest(name))
        .unwrap_or_else(|err| panic!("{name} must be format-valid: {err}"));
    let report = validate_edition(&source);
    assert_eq!(report.outcome, quran_corpus::Outcome::Fail, "{name} must fail validation");
    assert!(
        report.has_rule(rule),
        "{name}: expected {rule} in {:?}",
        report.findings.iter().map(|f| (&f.rule_id, &f.location)).collect::<Vec<_>>()
    );
    assert!(
        report.findings.iter().any(|f| f.rule_id == rule
            && (f.severity == Severity::Fatal || f.severity == Severity::Error)),
        "{name}: {rule} must be Fatal or Error"
    );
}

#[test]
fn identifier_and_count_faults() {
    expect_rule("missing_ayah", "QV-005");
    expect_rule("duplicate_ayah_id", "QV-005");
    expect_rule("extra_surah", "QV-001");
    expect_rule("wrong_ayah_count", "QV-003");
}

#[test]
fn unicode_faults() {
    expect_rule("nfd_text", "QV-007");
    expect_rule("bom_prefix", "QV-008");
    expect_rule("bidi_override", "QV-008");
    expect_rule("zero_width", "QV-008");
    expect_rule("latin_homoglyph", "QV-009");
}

#[test]
fn token_faults() {
    expect_rule("bad_offsets", "QV-012");
    expect_rule("shuffled_tokens", "QV-010");
    expect_rule("truncated_ayah", "QV-011");
}

#[test]
fn hash_mismatch_fails_qv_013() {
    let data = std::fs::read("../../fixtures/quran/adversarial/hash_mismatch/data.json")
        .expect("hash_mismatch data.json");
    let declared = std::fs::read_to_string(
        "../../fixtures/quran/adversarial/hash_mismatch/declared-sha256.txt",
    )
    .expect("hash_mismatch declared hash");
    let finding = quran_corpus::check_file_hash("hash_mismatch/data.json", declared.trim(), &data)
        .expect("tampered bytes must fail QV-013");
    assert_eq!(finding.rule_id, "QV-013");
    assert_eq!(finding.severity, Severity::Fatal);
}

#[test]
fn division_and_alignment_faults() {
    expect_rule("page_regression", "QV-018");
    expect_rule("missing_juz", "QV-016");
    expect_rule("translation_as_canonical", "QV-027");
}

#[tokio::test]
async fn validator_registry_bridge_reports_through_sources_types() {
    use quran_corpus::QuranEditionValidator;
    use sources::{SourceVersion, ValidatorRegistry};

    let base = std::fs::read_to_string("../../fixtures/quran/test-edition-min/manifest.json")
        .expect("base manifest");
    let mut registry = ValidatorRegistry::new();
    registry.register(std::sync::Arc::new(QuranEditionValidator::for_document(base)));
    assert!(registry.contains(QuranEditionValidator::NAME));

    // A SourceVersion is only an identity here; the bridge carries the bytes.
    let version: SourceVersion = serde_json::from_value(serde_json::json!({
        "id": "00000000-0000-0000-0000-000000000001",
        "source_id": "00000000-0000-0000-0000-000000000002",
        "version": "0.1.0",
        "schema_version": 1,
        "state": "validating",
        "trust_level": "ImportedUnverified",
        "license_status": "Unknown",
        "license_json": "{}",
        "manifest_blob_id": null,
        "manifest_hash": null,
        "content_hash": null,
        "source_urls": [],
        "publication_date": null,
        "imported_at": null,
        "validated_at": null,
        "approved_at": null,
        "approved_by": null,
        "activated_at": null,
        "deprecated_at": null,
        "quarantine_reason": null,
        "validation_report": null,
        "notes": null,
        "created_at": "2026-01-15T00:00:00Z"
    }))
    .expect("test SourceVersion");

    let report = registry
        .get(QuranEditionValidator::NAME)
        .expect("registered")
        .validate(&version)
        .await
        .expect("validator runs");
    assert!(report.valid, "base fixture must validate: {:?}", report.errors);

    let mut registry = ValidatorRegistry::new();
    registry.register(std::sync::Arc::new(QuranEditionValidator::for_document(manifest(
        "missing_ayah",
    ))));
    let report = registry
        .get(QuranEditionValidator::NAME)
        .expect("registered")
        .validate(&version)
        .await
        .expect("validator runs");
    assert!(!report.valid);
    assert!(report.errors.iter().any(|e| e.code == "QV-005"));
}
