//! Adapter integration tests: both adapters parse their samples; feeding
//! one shape to the other adapter fails with a typed error.

use quran_morphology::{DatasetRef, MorphologyError, parse_array_shape, parse_flat_csv};

fn dataset() -> DatasetRef {
    DatasetRef::new("synthetic-morph-test", "0.1.0")
}

#[test]
fn json_adapter_parses_array_sample() {
    let json = std::fs::read_to_string("../../fixtures/quran/lexicon/sample-a.json")
        .expect("read sample-a");
    let doc = parse_array_shape(&json, &dataset(), "test-edition-min").expect("parse");
    assert_eq!(doc.analyses.len(), 4);
    assert!(doc.synthetic_test_only);
    // Multi-analysis side-by-side: token (1,1,1) carries two analyses.
    let first_token: Vec<_> =
        doc.analyses.iter().filter(|a| (a.surah, a.ayah, a.token_position) == (1, 1, 1)).collect();
    assert_eq!(first_token.len(), 2);
    assert_ne!(first_token[0].lemma, first_token[1].lemma);
    assert_eq!(doc.analyses[2].segments.len(), 3);
}

#[test]
fn csv_adapter_parses_flat_sample() {
    let csv = std::fs::read_to_string("../../fixtures/quran/lexicon/sample-b.csv")
        .expect("read sample-b");
    let doc = parse_flat_csv(&csv, &dataset(), "test-edition-min").expect("parse");
    assert_eq!(doc.analyses.len(), 3);
    assert!(doc.synthetic_test_only);
    assert_eq!(doc.analyses[0].surface, "يَعْلَمُ");
    assert_eq!(doc.analyses[0].root, "علم");
    assert_eq!(doc.analyses[2].segments.len(), 3);
}

#[test]
fn csv_adapter_on_json_shape_fails_typed() {
    let json = std::fs::read_to_string("../../fixtures/quran/lexicon/sample-a.json")
        .expect("read sample-a");
    let err = parse_flat_csv(&json, &dataset(), "test-edition-min").expect_err("must fail");
    assert!(matches!(err, MorphologyError::ValidationFailed { .. }), "unexpected: {err:?}");
}

#[test]
fn json_adapter_on_csv_shape_fails_typed() {
    let csv = std::fs::read_to_string("../../fixtures/quran/lexicon/sample-b.csv")
        .expect("read sample-b");
    let err = parse_array_shape(&csv, &dataset(), "test-edition-min").expect_err("must fail");
    assert!(matches!(err, MorphologyError::ValidationFailed { .. }), "unexpected: {err:?}");
}
