//! A7 — normalization test vectors executed against the rules (OD-12).
//!
//! Vector data lives in `quran-core` (`tests/vectors/normalization/`, per A7);
//! this suite loads each covered rule's file and asserts the rule under test
//! maps every `input_codepoints` row to `expected_output_codepoints` exactly,
//! and that the output is a fixpoint (idempotency where claimed).
//!
//! Coverage rule: stabilizing a rule means adding its vector file **and** a
//! row in [`VECTOR_COVERED`]. CI fails otherwise — a rule without vectors
//! cannot ship.

use quran_normalization::rule::{NormalizationRule, NormalizedText};
use quran_normalization::rules::StripTatweel;

/// (rule short id, file stem version, rule under test).
/// Extend this list — never shrink it — as rules stabilize with vectors.
const VECTOR_COVERED: &[(&str, &str)] = &[("N02", "1.0.0")];

const VECTORS_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../quran-core/tests/vectors/normalization");

fn decode(codes: &[serde_json::Value]) -> String {
    codes
        .iter()
        .map(|code| {
            let text = code.as_str().expect("vector codepoint is a string");
            let hex = text.strip_prefix("U+").expect("vector codepoint has U+ prefix");
            let value = u32::from_str_radix(hex, 16).expect("vector codepoint is hex");
            char::from_u32(value).expect("vector codepoint is a valid char")
        })
        .collect()
}

fn load_vectors(rule: &str, version: &str) -> Vec<(String, String, String)> {
    let path = format!("{VECTORS_PATH}/{rule}-v{version}.json");
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("missing vector file: {path}"));
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(&text).expect("vector file is a JSON array");
    rows.into_iter()
        .map(|row| {
            let input = decode(
                row.get("input_codepoints")
                    .and_then(|v| v.as_array())
                    .expect("row has input_codepoints"),
            );
            let expected = decode(
                row.get("expected_output_codepoints")
                    .and_then(|v| v.as_array())
                    .expect("row has expected_output_codepoints"),
            );
            let note = row.get("note").and_then(|v| v.as_str()).expect("row has note").to_string();
            (input, expected, note)
        })
        .collect()
}

#[test]
fn every_covered_rule_has_its_vector_file() {
    assert!(!VECTOR_COVERED.is_empty(), "vector coverage list must not be empty");
    for (rule, version) in VECTOR_COVERED {
        let rows = load_vectors(rule, version);
        assert!(rows.len() >= 20, "{rule}: {} vectors, need ≥ 20", rows.len());
    }
}

#[test]
fn n02_vectors_match_strip_tatweel_exactly() {
    let rule = StripTatweel;
    assert_eq!(rule.id().as_str(), "N02");
    let rows = load_vectors("N02", "1.0.0");
    assert!(rows.len() >= 20);
    for (input, expected, note) in &rows {
        let got = rule.apply(&NormalizedText::from_plain(input));
        assert_eq!(got.text(), expected, "N02 vector failed ({note})");
        if rule.is_idempotent() {
            let again = rule.apply(&got);
            assert_eq!(again.text(), expected, "N02 not a fixpoint ({note})");
        }
    }
}
