//! Phase 1 — edition fixtures acceptance (P1-T05, P1-T15–T17).
//!
//! The synthetic `test-edition-min` edition parses through both adapters with
//! identical ayah content, and every adversarial manifest is *format*-valid (it
//! must fail in the validator with its specific rule id, M5 — not at parse).

use quran_corpus::{CsvAdapter, EditionAdapter, EditionSource, JsonAdapter};
use std::collections::HashSet;

const MIN_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const MIN_CSV: &str = include_str!("../../../fixtures/quran/test-edition-min/ayahs.csv");

fn adversarial(name: &str) -> EditionSource {
    let path = format!("../../fixtures/quran/adversarial/{name}/manifest.json");
    // Integration tests run with the crate directory as CWD.
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("missing adversarial fixture {name}"));
    JsonAdapter.parse(&text).unwrap_or_else(|err| panic!("{name} must be format-valid: {err}"))
}

#[test]
fn test_edition_min_parses_with_expected_shape() {
    let source: EditionSource = JsonAdapter.parse(MIN_MANIFEST).unwrap();
    assert_eq!(source.edition.slug, "test-edition-min");
    assert_eq!(source.surahs.len(), 5);
    assert_eq!(source.ayahs.len(), 14);
    assert_eq!(source.expected.surah_count, 5);
    assert_eq!(source.expected.ayah_count, 14);
    assert_eq!(source.expected.reference_corpus_id, None);

    let mut seen = HashSet::new();
    for ayah in &source.ayahs {
        assert!(!ayah.text.trim().is_empty(), "empty ayah text");
        assert!(seen.insert((ayah.surah, ayah.ayah)), "duplicate ayah id in base fixture");
    }
    // Surah ayah ids are dense per surah in the base fixture.
    for surah in &source.surahs {
        let count = source.ayahs.iter().filter(|a| a.surah == surah.number).count();
        assert_eq!(count, usize::from(surah.ayah_count), "surah {}", surah.number);
    }
}

#[test]
fn csv_adapter_reproduces_the_same_ayahs() {
    let json_source: EditionSource = JsonAdapter.parse(MIN_MANIFEST).unwrap();
    let adapter = CsvAdapter::new(
        json_source.edition.clone(),
        json_source.surahs.clone(),
        json_source.expected.clone(),
        json_source.tokenization.clone(),
    );
    let csv_source = adapter.parse(MIN_CSV).unwrap();
    assert_eq!(csv_source.ayahs.len(), json_source.ayahs.len());
    for (csv_ayah, json_ayah) in csv_source.ayahs.iter().zip(json_source.ayahs.iter()) {
        assert_eq!((csv_ayah.surah, csv_ayah.ayah), (json_ayah.surah, json_ayah.ayah));
        assert_eq!(csv_ayah.text, json_ayah.text);
        assert_eq!(csv_ayah.juz, json_ayah.juz);
        assert_eq!(csv_ayah.page, json_ayah.page);
        assert_eq!(csv_ayah.sajdah, json_ayah.sajdah);
    }
}

#[test]
fn all_sixteen_adversarial_manifests_are_format_valid() {
    for name in [
        "missing_ayah",
        "duplicate_ayah_id",
        "extra_surah",
        "wrong_ayah_count",
        "nfd_text",
        "bom_prefix",
        "bidi_override",
        "zero_width",
        "latin_homoglyph",
        "bad_offsets",
        "shuffled_tokens",
        "truncated_ayah",
        "page_regression",
        "missing_juz",
        "translation_as_canonical",
    ] {
        adversarial(name);
    }
    // hash_mismatch is a source package (data + declared hash), not an edition manifest.
    let data = std::fs::read_to_string("../../fixtures/quran/adversarial/hash_mismatch/data.json")
        .expect("hash_mismatch data.json");
    JsonAdapter.parse(&data).expect("hash_mismatch data is format-valid");
    let declared = std::fs::read_to_string(
        "../../fixtures/quran/adversarial/hash_mismatch/declared-sha256.txt",
    )
    .expect("hash_mismatch declared hash");
    assert_eq!(declared.trim().len(), 64);
}
