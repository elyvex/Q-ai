//! Phase 1 — reference grammar acceptance: AC-P1-12, AC-P1-13.
//!
//! The frozen golden set (`fixtures/quran/golden/references.jsonl`, ≥300 cases)
//! plus `parse(serialize(ref)) == ref` and never-panics property tests.
//!
//! Note on placement: `acceptance.md` §3.2 names these `tests/quran/*.rs`. The
//! workspace root has no package, so root-level `tests/` cannot be a test
//! target; the suites live as crate-level integration tests with the fixture
//! files shared at `fixtures/quran/`.

use domain::SemVer;
use proptest::prelude::*;
use quran_core::error::{Diagnostic, codes};
use quran_core::{
    AyahNumber, DivisionKind, EditionSelector, QuranRef, SurahNumber, TokenPosition,
    canonical_form, parse, serialize,
};

const GOLDEN: &str = include_str!("../../../fixtures/quran/golden/references.jsonl");

fn golden_rows() -> Vec<(String, serde_json::Value)> {
    GOLDEN
        .lines()
        .filter(|line| !line.trim().is_empty())
        .enumerate()
        .map(|(index, line)| {
            let value: serde_json::Value = serde_json::from_str(line)
                .unwrap_or_else(|_| panic!("line {} is not JSON", index + 1));
            (format!("line {}", index + 1), value)
        })
        .collect()
}

#[test]
fn golden_file_has_at_least_300_cases() {
    let rows = golden_rows();
    assert!(rows.len() >= 300, "expected ≥300 golden cases, found {}", rows.len());
}

#[test]
fn golden_valid_cases_parse_and_serialize() {
    let mut checked = 0_usize;
    for (where_, row) in golden_rows() {
        let Some(expected) = row.get("canonical").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let input = row["input"].as_str().unwrap_or("<missing input>");
        let parsed = parse(input).unwrap_or_else(|err| panic!("{where_} `{input}` failed: {err}"));
        assert_eq!(serialize(&parsed), expected, "{where_} `{input}`");
        checked += 1;
    }
    assert!(checked >= 150, "expected a substantial valid battery, found {checked}");
}

#[test]
fn golden_invalid_cases_return_the_expected_code() {
    let mut checked = 0_usize;
    for (where_, row) in golden_rows() {
        let Some(expected) = row.get("error").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let input = row["input"].as_str().unwrap_or("<missing input>");
        match parse(input) {
            Ok(parsed) => panic!("{where_} `{input}` parsed unexpectedly to {parsed:?}"),
            Err(err) => assert_eq!(err.code().to_string(), expected, "{where_} `{input}`"),
        }
        checked += 1;
    }
    assert!(checked >= 100, "expected a substantial invalid battery, found {checked}");
}

fn arb_selector() -> impl Strategy<Value = EditionSelector> {
    prop_oneof![
        Just(EditionSelector::Active),
        "[a-z][a-z0-9\\-]{0,11}".prop_map(EditionSelector::Slug),
        ("[a-z][a-z0-9\\-]{0,11}", 0u64..6, 0u64..6, 0u64..6).prop_map(
            |(slug, major, minor, patch)| EditionSelector::Pinned {
                slug,
                version: SemVer::new(major, minor, patch),
            }
        ),
    ]
}

fn arb_surah() -> impl Strategy<Value = SurahNumber> {
    (1u16..=114).prop_map(|n| SurahNumber::new(n).unwrap())
}

fn arb_ayah() -> impl Strategy<Value = AyahNumber> {
    (1u32..500).prop_map(|n| AyahNumber::new(n).unwrap())
}

fn arb_position() -> impl Strategy<Value = TokenPosition> {
    (1u16..100).prop_map(|n| TokenPosition::new(n).unwrap())
}

fn arb_kind() -> impl Strategy<Value = DivisionKind> {
    prop_oneof![
        Just(DivisionKind::Juz),
        Just(DivisionKind::Hizb),
        Just(DivisionKind::Rub),
        Just(DivisionKind::Manzil),
        Just(DivisionKind::Page),
        Just(DivisionKind::Ruku),
        Just(DivisionKind::Sajdah),
    ]
}

fn arb_ref() -> impl Strategy<Value = QuranRef> {
    let endpoints =
        (arb_surah(), arb_ayah(), arb_surah(), arb_ayah()).prop_map(|(s1, a1, s2, a2)| {
            let (start, end) = if (s1.get(), a1.get()) <= (s2.get(), a2.get()) {
                ((s1, a1), (s2, a2))
            } else {
                ((s2, a2), (s1, a1))
            };
            (start, end)
        });
    prop_oneof![
        (arb_selector(), arb_surah())
            .prop_map(|(edition, surah)| QuranRef::Surah { edition, surah }),
        (arb_selector(), arb_surah(), arb_ayah())
            .prop_map(|(edition, surah, ayah)| QuranRef::Ayah { edition, surah, ayah }),
        (arb_selector(), endpoints).prop_map(|(edition, (start, end))| QuranRef::AyahRange {
            edition,
            start,
            end
        }),
        (arb_selector(), arb_surah(), arb_ayah(), arb_position()).prop_map(
            |(edition, surah, ayah, position)| QuranRef::Token { edition, surah, ayah, position }
        ),
        (arb_selector(), arb_kind(), 1u32..700)
            .prop_map(|(edition, kind, number)| QuranRef::Division { edition, kind, number }),
    ]
}

proptest! {
    #[test]
    fn roundtrip_parse_serialize(reference in arb_ref()) {
        let text = serialize(&reference);
        let back = parse(&text)
            .unwrap_or_else(|err| panic!("serialize produced unparseable `{text}`: {err}"));
        prop_assert_eq!(&back, &reference);
        if let Some(canonical) = canonical_form(&reference) {
            let again = parse(&canonical).expect("canonical form must parse");
            prop_assert_eq!(&again, &reference);
        }
    }

    #[test]
    fn parser_never_panics_and_errors_are_coded(input in "\\PC*") {
        if let Err(err) = parse(&input) {
            let rendered = err.code().to_string();
            prop_assert!(rendered.starts_with("QAI-QUR-01"), "unexpected code {rendered}");
        }
    }
}

#[test]
fn resolve_returns_canonical_serialization() {
    let resolved = quran_core::resolve("quran:2:255").unwrap();
    assert_eq!(resolved.canonical, "2:255");
    assert_eq!(resolved.reference, parse("2:255").unwrap());
    let err = quran_core::resolve("nope:0:0").unwrap_err();
    assert_eq!(err.code(), codes::INVALID_SURAH);
}
