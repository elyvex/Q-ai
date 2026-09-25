//! Phase 2 — committed corpus-integrity artifact (D-10/D-11, QC-03).
//!
//! Guards the shape and honest classification of the evidence-of-record at
//! `docs/06-progress/corpus-integrity-report.json`:
//!
//! - exactly the six integrity families are present (counts, addressing,
//!   unicode, checksums, roundtrip, reference_comparison);
//! - the reference family status is `pass`/`fail`/`skipped` and a `skipped`
//!   family is never recorded as `pass` (D-10; ADR-0114 §4);
//! - a `pass` is backed by a real persisted QV-015 evidence object;
//! - the pinned edition identity matches `test-edition-rich`'s manifest;
//! - no timestamp, temporary path, host name, or random id is committed
//!   (T-02-31 reproducible evidence).
//!
//! This harness is deliberately independent of `tests/quran.rs`: it only reads
//! the committed artifact, so it fails loudly if a future regeneration
//! overstates a family or carries a volatile value.

use std::collections::BTreeSet;

const ARTIFACT: &str = include_str!("../../../docs/06-progress/corpus-integrity-report.json");
const RICH_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-rich/manifest.json");

/// The six integrity families from `.planning/ROADMAP.md` §Phase 2 criterion 3.
const FAMILIES: [&str; 6] =
    ["counts", "addressing", "unicode", "checksums", "roundtrip", "reference_comparison"];

fn artifact() -> serde_json::Value {
    serde_json::from_str(ARTIFACT).expect("corpus-integrity artifact must parse as JSON")
}

/// True when `text` contains a 36-char hyphenated UUID-shaped token.
fn contains_uuid(text: &str) -> bool {
    text.as_bytes().windows(36).any(|window| {
        window.iter().enumerate().all(|(index, byte)| match index {
            8 | 13 | 18 | 23 => *byte == b'-',
            _ => byte.is_ascii_hexdigit(),
        })
    })
}

#[test]
fn artifact_exposes_exactly_the_six_families() {
    let doc = artifact();
    let families = doc.get("families").expect("artifact must have a `families` object");
    let keys: BTreeSet<&str> = families
        .as_object()
        .expect("`families` must be an object")
        .keys()
        .map(String::as_str)
        .collect();
    let expected: BTreeSet<&str> = FAMILIES.iter().copied().collect();
    assert_eq!(keys, expected, "artifact must expose exactly the six integrity families");
}

#[test]
fn reference_family_status_is_honest() {
    let doc = artifact();
    let reference = &doc["families"]["reference_comparison"];
    let status = reference["status"].as_str().expect("reference family must carry a string status");
    assert!(
        matches!(status, "pass" | "fail" | "skipped"),
        "reference family status must be pass/fail/skipped, got {status:?}"
    );

    let outcome = reference
        .get("evidence")
        .and_then(|evidence| evidence.get("outcome"))
        .and_then(|value| value.as_str());
    if status == "skipped" {
        // A recorded skip is never a pass — not even in the nested evidence.
        assert_ne!(
            outcome,
            Some("pass"),
            "a skipped reference family must never carry outcome=pass"
        );
    }
    if status == "pass" {
        // A pass must be backed by a real comparison, not an empty label.
        assert_eq!(
            outcome,
            Some("pass"),
            "a passing reference family must carry evidence.outcome=pass"
        );
        assert_eq!(
            reference["evidence"]["method"].as_str(),
            Some("exact-ayah-bytes-v1"),
            "a passing reference family must name the byte-exact comparison method"
        );
    }
}

#[test]
fn pinned_edition_identity_matches_the_rich_fixture() {
    let doc = artifact();
    let manifest: serde_json::Value =
        serde_json::from_str(RICH_MANIFEST).expect("rich fixture manifest must parse");
    assert_eq!(
        doc["edition"]["slug"], manifest["edition"]["slug"],
        "artifact edition slug must match the rich fixture"
    );
    assert_eq!(
        doc["edition"]["version"], manifest["edition"]["version"],
        "artifact edition version must match the rich fixture"
    );
}

#[test]
fn artifact_has_no_volatile_identity() {
    // T-02-31: the committed evidence of record must be reproducible — no
    // timestamp, temporary path, host name, or random id may leak in.
    for needle in ["/tmp/", "/var/folders", "localhost", "127.0.0.1"] {
        assert!(!ARTIFACT.contains(needle), "artifact must not contain temp/host value {needle:?}");
    }
    for key in ["timestamp", "created_at", "generated_at", "hostname", "run_id"] {
        assert!(
            !ARTIFACT.contains(&format!("\"{key}\"")),
            "artifact must not contain volatile key {key:?}"
        );
    }
    assert!(!contains_uuid(ARTIFACT), "artifact must not contain a random UUID");
}
