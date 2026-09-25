//! Phase 1 — edition fixtures acceptance (P1-T05, P1-T15–T17).
//!
//! The synthetic `test-edition-min` edition parses through both adapters with
//! identical ayah content, and every adversarial manifest is *format*-valid (it
//! must fail in the validator with its specific rule id, M5 — not at parse).

use quran_corpus::validation::Severity;
use quran_corpus::{CsvAdapter, EditionAdapter, EditionSource, JsonAdapter, validate_edition};
use std::collections::{BTreeMap, HashMap, HashSet};

const MIN_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const MIN_CSV: &str = include_str!("../../../fixtures/quran/test-edition-min/ayahs.csv");
const RICH_MANIFEST: &str =
    include_str!("../../../fixtures/quran/test-edition-rich/manifest.json");
const RICH_CSV: &str = include_str!("../../../fixtures/quran/test-edition-rich/ayahs.csv");
const RICH_REFERENCE: &str =
    include_str!("../../../fixtures/quran/test-edition-rich/reference.json");
const IDENTITY_MANIFEST: &str =
    include_str!("../../../fixtures/quran/test-edition-identity/manifest.json");

/// Load the golden rows for one edition, keyed by `(surah, ayah)`; a duplicate
/// key within the edition is a fixture bug (each ayah gets exactly one row).
///
/// Golden keys are edition-scoped because more than one synthetic edition is
/// numbered `1..=N` (both must satisfy QV-002 before the importer accepts
/// them), so a bare `(surah, ayah)` key would collide across editions.
fn load_golden(edition: &str) -> BTreeMap<(u16, u32), String> {
    let path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/quran/golden/ayah_texts.jsonl");
    let golden = std::fs::read_to_string(path).expect("golden ayah_texts.jsonl");
    let mut expected = BTreeMap::new();
    for line in golden.lines().filter(|line| !line.trim().is_empty()) {
        let value: serde_json::Value = serde_json::from_str(line).expect("golden row is JSON");
        if value["edition"].as_str().expect("golden edition") != edition {
            continue;
        }
        let surah = value["surah"].as_u64().expect("golden surah") as u16;
        let ayah = value["ayah"].as_u64().expect("golden ayah") as u32;
        let text = value["text"].as_str().expect("golden text").to_string();
        assert!(expected.insert((surah, ayah), text).is_none(), "duplicate golden row");
    }
    expected
}

/// Every ayah of `source` must have a golden row whose text matches exactly,
/// and the golden set must cover exactly those ayahs (no extras, no gaps).
fn assert_golden_covers(source: &EditionSource, golden: &BTreeMap<(u16, u32), String>) {
    assert_eq!(
        golden.len(),
        source.ayahs.len(),
        "golden set must cover exactly every ayah of {}",
        source.edition.slug
    );
    for row in &source.ayahs {
        let got = golden
            .get(&(row.surah, row.ayah))
            .unwrap_or_else(|| panic!("golden set missing {}:{}", row.surah, row.ayah));
        assert_eq!(*got, row.text, "golden text for {}:{}", row.surah, row.ayah);
    }
}

fn is_pause_mark(c: char) -> bool {
    ('\u{06D6}'..='\u{06ED}').contains(&c)
}

fn adversarial(name: &str) -> EditionSource {
    let path = format!("../../fixtures/quran/adversarial/{name}/manifest.json");
    // Integration tests run with the crate directory as CWD.
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("missing adversarial fixture {name}"));
    JsonAdapter.parse(&text).unwrap_or_else(|err| panic!("{name} must be format-valid: {err}"))
}

#[test]
fn reference_comparison_is_exact_and_independent_of_row_order() {
    let source = JsonAdapter.parse(MIN_MANIFEST).unwrap();
    let mut reference = source.clone();
    reference.ayahs.reverse();
    assert!(quran_corpus::import::compare_reference(&source, Some(&reference)).is_empty());
    reference.ayahs[0].text.push(' ');
    let findings = quran_corpus::import::compare_reference(&source, Some(&reference));
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule_id, "QV-015");
    assert_eq!(findings[0].severity, quran_corpus::validation::Severity::Fatal);
    assert_eq!(findings[0].message, "text differs byte-for-byte");
}

#[test]
fn reference_comparison_rejects_missing_duplicate_and_empty_rows() {
    let source = JsonAdapter.parse(MIN_MANIFEST).unwrap();
    let mut reference = source.clone();
    reference.ayahs.pop();
    let findings = quran_corpus::import::compare_reference(&source, Some(&reference));
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].message, "ayah missing from reference corpus");
    let findings = quran_corpus::import::compare_reference(&reference, Some(&source));
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].message, "ayah missing from imported corpus");
    reference = source.clone();
    reference.ayahs.push(reference.ayahs[0].clone());
    assert!(
        quran_corpus::import::compare_reference(&source, Some(&reference))
            .iter()
            .any(|f| f.message.contains("duplicate"))
    );
    assert!(
        quran_corpus::import::compare_reference(&reference, Some(&source))
            .iter()
            .any(|f| f.message.contains("duplicate"))
    );
    reference.ayahs.clear();
    assert!(
        quran_corpus::import::compare_reference(&reference, Some(&reference))
            .iter()
            .all(|f| f.severity == quran_corpus::validation::Severity::Fatal)
    );
    assert!(!quran_corpus::import::compare_reference(&reference, Some(&reference)).is_empty());
}

#[test]
fn reference_comparison_skips_unconfigured_and_rejects_incompatible_readings() {
    let source = JsonAdapter.parse(MIN_MANIFEST).unwrap();
    let findings = quran_corpus::import::compare_reference(&source, None);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule_id, "QV-015");
    assert_eq!(findings[0].severity, quran_corpus::validation::Severity::Info);
    assert!(findings[0].message.contains("skipped"));
    let mut reference = source.clone();
    reference.edition.riwayah = Some("different test reading".into());
    let findings = quran_corpus::import::compare_reference(&source, Some(&reference));
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].severity, quran_corpus::validation::Severity::Fatal);
    assert!(findings[0].message.contains("incompatible"));
}

#[test]
fn golden_ayah_texts_match_the_imported_fixture() {
    let source = JsonAdapter.parse(MIN_MANIFEST).unwrap();
    assert_golden_covers(&source, &load_golden("test-edition-min"));
}

#[test]
fn golden_ayah_texts_cover_the_rich_fixture() {
    let source = JsonAdapter.parse(RICH_MANIFEST).unwrap();
    assert_golden_covers(&source, &load_golden("test-edition-rich"));
}

#[test]
fn test_edition_min_parses_with_expected_shape() {
    let source: EditionSource = JsonAdapter.parse(MIN_MANIFEST).unwrap();
    assert_eq!(source.edition.slug, "test-edition-min");
    // OD-01 B-track: the pipeline-exercise fixture is flagged synthetic and
    // must stay that way — it is never canonical.
    assert!(source.edition.synthetic, "test-edition-min must stay synthetic");
    assert_eq!(source.edition.upstream_edition_slug, None);
    assert_eq!(source.edition.verified_by, None);
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
fn test_edition_rich_parses_with_expected_shape_and_coverage() {
    let source = JsonAdapter.parse(RICH_MANIFEST).unwrap();
    assert_eq!(source.edition.slug, "test-edition-rich");
    // D-05/ADR-0101: synthetic fixtures are never canonical and carry no
    // invented upstream identity, primary flag, or license.
    assert!(source.edition.synthetic, "test-edition-rich must stay synthetic");
    assert_eq!(source.edition.upstream_edition_slug, None);
    assert_eq!(source.edition.qai_edition_id, None);
    assert!(!source.edition.is_primary);
    assert_eq!(source.edition.publisher, None);
    assert_eq!(source.edition.license, None);

    assert_eq!(source.surahs.len(), 6);
    assert_eq!(source.ayahs.len(), 24);
    // QV-002 (Fatal) requires surah numbers to be exactly 1..=N, so the fixture
    // must be numbered from 1 for the importer to accept it (plan 02-03).
    let mut numbers: Vec<u16> = source.surahs.iter().map(|s| s.number).collect();
    numbers.sort_unstable();
    assert_eq!(numbers, (1..=6_u16).collect::<Vec<_>>(), "rich surahs must be 1..=N");
    assert_eq!(source.expected.surah_count, 6);
    assert_eq!(source.expected.ayah_count, 24);
    assert_eq!(source.expected.reference_corpus_id.as_deref(), Some("synthetic-rich-reference"));
    assert_eq!(
        source.expected.reference_text_hash.as_deref(),
        Some("sha256:6c9e6916a7a04850805f4205bd0993042d338af1a82ddd37972745f3e409bb3a")
    );

    let mut seen = HashSet::new();
    for ayah in &source.ayahs {
        assert!(!ayah.text.trim().is_empty(), "empty ayah text");
        assert!(seen.insert((ayah.surah, ayah.ayah)), "duplicate ayah id in rich fixture");
    }
    for surah in &source.surahs {
        let numbered: Vec<u32> =
            source.ayahs.iter().filter(|a| a.surah == surah.number).map(|a| a.ayah).collect();
        assert_eq!(numbered.len(), usize::from(surah.ayah_count), "surah {}", surah.number);
        assert_eq!(
            numbered,
            (1..=u32::from(surah.ayah_count)).collect::<Vec<_>>(),
            "dense per-surah numbering for surah {}",
            surah.number
        );
    }

    assert!(
        source.ayahs.iter().any(|a| a.text.split_whitespace().any(|t| t == "الٓمٓ")),
        "rich fixture must include a muqatta'at standalone token"
    );
    assert!(
        source
            .ayahs
            .iter()
            .any(|a| a.text.contains("بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ")
                && !a.text.trim_start().starts_with("بِسْمِ")),
        "rich fixture must include a basmala inside an ayah body"
    );
    assert!(
        source.ayahs.iter().any(|a| a.text.contains('\u{0670}')),
        "rich fixture must include a superscript alef U+0670"
    );
    assert!(
        source.ayahs.iter().any(|a| a.text.chars().any(is_pause_mark)),
        "rich fixture must include a U+06D6..U+06ED pause mark"
    );
    let mark_separated = source.ayahs.iter().any(|a| {
        let chars: Vec<char> = a.text.chars().collect();
        (1..chars.len().saturating_sub(1)).any(|i| {
            is_pause_mark(chars[i])
                && !chars[i - 1].is_whitespace()
                && !chars[i + 1].is_whitespace()
        })
    });
    assert!(mark_separated, "rich fixture must include a mark-separated token");
    assert!(
        source.ayahs.iter().any(|a| a.text.split_whitespace().count() >= 8),
        "rich fixture must include a long multi-token ayah"
    );
    let pages: Vec<u32> = source.ayahs.iter().filter_map(|a| a.page).collect();
    assert!(pages.windows(2).any(|w| w[0] != w[1]), "rich fixture must span a page boundary");
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for ayah in &source.ayahs {
        *counts.entry(ayah.text.as_str()).or_insert(0) += 1;
    }
    assert!(counts.values().any(|&n| n > 1), "rich fixture must include a repeated refrain");
}

#[test]
fn csv_adapter_reproduces_the_rich_ayahs() {
    let json_source = JsonAdapter.parse(RICH_MANIFEST).unwrap();
    let adapter = CsvAdapter::new(
        json_source.edition.clone(),
        json_source.surahs.clone(),
        json_source.expected.clone(),
        json_source.tokenization.clone(),
    );
    let csv_source = adapter.parse(RICH_CSV).unwrap();
    assert_eq!(csv_source.ayahs.len(), json_source.ayahs.len());
    for (csv_ayah, json_ayah) in csv_source.ayahs.iter().zip(json_source.ayahs.iter()) {
        assert_eq!((csv_ayah.surah, csv_ayah.ayah), (json_ayah.surah, json_ayah.ayah));
        assert_eq!(csv_ayah.text, json_ayah.text);
        assert_eq!(csv_ayah.juz, json_ayah.juz);
        assert_eq!(csv_ayah.hizb, json_ayah.hizb);
        assert_eq!(csv_ayah.rub, json_ayah.rub);
        assert_eq!(csv_ayah.manzil, json_ayah.manzil);
        assert_eq!(csv_ayah.ruku, json_ayah.ruku);
        assert_eq!(csv_ayah.page, json_ayah.page);
        assert_eq!(csv_ayah.sajdah, json_ayah.sajdah);
    }
}

#[test]
fn rich_reference_is_a_valid_synthetic_edition_with_both_pins() {
    let rich = JsonAdapter.parse(RICH_MANIFEST).unwrap();
    let reference = JsonAdapter.parse(RICH_REFERENCE).unwrap();
    assert_eq!(reference.edition.slug, "synthetic-rich-reference");
    assert!(reference.edition.synthetic, "reference fixture must stay synthetic");
    assert_eq!(reference.edition.upstream_edition_slug, None);
    assert_eq!(reference.edition.publisher, None);
    assert_eq!(reference.edition.license, None);
    assert_eq!(reference.expected.reference_corpus_id.as_deref(), Some("synthetic-rich-reference"));
    assert!(
        reference.expected.reference_text_hash.as_deref().is_some_and(|h| h.starts_with("sha256:")),
        "reference fixture must carry a sha256 pin"
    );
    // The reference is byte-for-byte the rich ayah stream (Tier-1 exact diff).
    assert_eq!(reference.ayahs, rich.ayahs);
    assert_eq!(reference.surahs, rich.surahs);
}

#[test]
fn identity_fixture_declares_identity_primary_and_declared_license() {
    let source = JsonAdapter.parse(IDENTITY_MANIFEST).unwrap();
    assert_eq!(source.edition.slug, "test-edition-identity");
    assert!(source.edition.synthetic, "identity fixture must stay synthetic");
    assert_eq!(
        source.edition.upstream_edition_slug.as_deref(),
        Some("synthetic-identity-upstream")
    );
    assert_eq!(source.edition.qai_edition_id.as_deref(), Some("qai:synthetic-identity"));
    assert!(source.edition.is_primary, "identity fixture must flag the primary edition");
    let license = source.edition.license.expect("identity fixture must declare a license");
    assert_ne!(license.status.as_str(), "unknown", "declared license status must be concrete");
    assert!(license.expression.is_some(), "declared license must carry an expression");
    assert_eq!(source.expected.reference_corpus_id, None);
}

/// Plan 02-03 imports these fixtures through `run_import`, which aborts on any
/// Fatal finding (and promotes Error findings from the reference). Prove here
/// that each fixture validates cleanly so the operator path stays viable.
#[test]
fn rich_reference_and_identity_fixtures_validate_without_blocking_findings() {
    for (name, text) in [
        ("test-edition-rich", RICH_MANIFEST),
        ("synthetic-rich-reference", RICH_REFERENCE),
        ("test-edition-identity", IDENTITY_MANIFEST),
    ] {
        let source = JsonAdapter.parse(text).unwrap();
        let blocking: Vec<String> = validate_edition(&source)
            .findings
            .iter()
            .filter(|finding| matches!(finding.severity, Severity::Fatal | Severity::Error))
            .map(|finding| {
                format!("{} {}: {}", finding.rule_id, finding.location, finding.message)
            })
            .collect();
        assert!(blocking.is_empty(), "{name} must import cleanly, found: {blocking:?}");
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
