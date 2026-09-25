//! Phase 1 — Quran corpus doctor acceptance: AC-P1-07 / AC-P1-17 + §5.4 bullet 6.
//!
//! Recomputes `text_hash`, `structure_hash`, and `token_order_hash` from the
//! stored canonical rows and asserts they equal the import-time values, then
//! runs the full 19-check `--deep` scan on the synthetic edition and asserts
//! nothing fails (license/reference-corpus checks may warn/skip by design).

#[path = "common/mod.rs"]
mod common;

use application::quran_cli::cmd_quran_verify;
use application::quran_doctor::{CheckLevel, run_quran_checks};
use application::quran_reader::QuranReader;
use common::{BASE_MANIFEST, SLUG, VERSION, active_reader};
use quran_core::AyahOptions;
use quran_corpus::format::EditionSource;
use storage::Database as _;

async fn active_generation(db: &storage_sqlite::SqliteDatabase) -> i64 {
    let mut uow = db.write().await.unwrap();
    let active = uow.quran().get_active().await.unwrap().expect("active edition");
    uow.rollback().await.unwrap();
    active.corpus_generation
}

#[tokio::test]
async fn fixture_soak_ten_thousand_lookups_preserves_corpus_integrity() {
    let (_dir, db, reader, _path) = active_reader().await;
    let source: EditionSource = serde_json::from_str(BASE_MANIFEST).unwrap();
    let references: Vec<_> = source
        .ayahs
        .iter()
        .map(|ayah| {
            quran_core::parse(&format!("quran:{SLUG}@{VERSION}:{}:{}", ayah.surah, ayah.ayah))
                .unwrap()
        })
        .collect();
    assert!(!references.is_empty());
    let mut visits = vec![0_usize; references.len()];
    let mut random = 0x51a1_0058_u64;
    for iteration in 0..10_000 {
        random ^= random << 13;
        random ^= random >> 7;
        random ^= random << 17;
        let index = (random % references.len() as u64) as usize;
        visits[index] += 1;
        let expected = &source.ayahs[index];
        let options =
            AyahOptions { translations: Vec::new(), glosses: false, tokens: iteration % 2 == 0 };
        let view = reader.get_ayah(&references[index], &options).await.unwrap();
        assert_eq!(view.canonical.arabic_text().as_bytes(), expected.text.as_bytes());
        assert_eq!(
            view.canonical.reference(),
            format!("quran:{SLUG}@{VERSION}:{}:{}", expected.surah, expected.ayah)
        );
        assert_eq!(view.canonical.edition().slug, SLUG);
        assert_eq!(view.canonical.edition().version.to_string(), VERSION);
        if options.tokens {
            for token in view.tokens.expect("tokens requested") {
                assert_eq!(
                    expected.text.get(token.byte_start as usize..token.byte_end as usize),
                    Some(token.surface.as_str())
                );
            }
        } else {
            assert!(view.tokens.is_none());
        }
    }
    assert!(visits.iter().all(|count| *count > 0));
    let checks = run_quran_checks(&*db, true).await.expect("post-soak deep scan");
    assert_eq!(checks.len(), 19);
    for check in &checks {
        assert_ne!(check.status, CheckLevel::Fail, "{}: {}", check.id, check.summary);
    }
    for id in [
        "quran.edition_checksum",
        "quran.structure_hash",
        "quran.token_order_hash",
        "quran.token_roundtrip",
    ] {
        let check = checks.iter().find(|check| check.id == id).unwrap();
        assert_eq!(check.status, CheckLevel::Pass, "{}: {}", id, check.summary);
    }
}

#[tokio::test]
async fn recomputed_hashes_match_import_time() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let checks = run_quran_checks(&*db, true).await.expect("doctor runs");

    for id in ["quran.edition_checksum", "quran.structure_hash", "quran.token_order_hash"] {
        let check = checks.iter().find(|c| c.id == id).unwrap_or_else(|| panic!("missing {id}"));
        assert_eq!(
            check.status,
            CheckLevel::Pass,
            "{id} did not pass: {} (recomputed hashes must equal import-time values)",
            check.summary
        );
    }
}

#[tokio::test]
async fn deep_scan_of_the_fixture_has_no_failures() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let checks = run_quran_checks(&*db, true).await.expect("doctor runs");

    // Exactly the 19 corpus checks, in the documented set (AC-P1-17).
    assert_eq!(checks.len(), 19, "expected 19 doctor checks, got {}", checks.len());

    let failures: Vec<&str> =
        checks.iter().filter(|c| c.status == CheckLevel::Fail).map(|c| c.id).collect();
    assert!(failures.is_empty(), "fixture edition must not fail any check: {failures:?}");

    // The token round-trip is the point of `--deep`: it must pass on the fixture.
    let roundtrip =
        checks.iter().find(|c| c.id == "quran.token_roundtrip").expect("roundtrip check");
    assert_eq!(roundtrip.status, CheckLevel::Pass, "{}", roundtrip.summary);

    // Non-passing checks here are the two owner-gated ones, never silent passes.
    for check in checks.iter().filter(|c| c.status != CheckLevel::Pass) {
        assert!(
            matches!(check.status, CheckLevel::Warn | CheckLevel::Skipped),
            "unexpected non-pass severity for {}",
            check.id
        );
        assert!(check.remedy.is_some(), "{} must carry a remedy", check.id);
    }
}

/// Plan 02-03 Task 1 — `qai quran verify` classifies the six integrity families
/// from persisted state, reports the reference family as `skipped` (never
/// `pass`) for a reference-less edition, and performs no write (T-02-07).
#[tokio::test]
async fn verify_reports_the_six_families_and_does_not_write() {
    let (_dir, db, _reader, path) = active_reader().await;
    let before = active_generation(&db).await;
    let out = cmd_quran_verify(&path, "active", false).await;
    assert_eq!(out.exit, 0, "{}", out.human);

    let object = out.json.as_object().expect("verify --json must be an object");
    let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        ["addressing", "checksums", "counts", "reference_comparison", "roundtrip", "unicode"],
        "exactly the six integrity families"
    );
    assert_eq!(
        out.json["reference_comparison"]["status"], "skipped",
        "a reference-less edition must report the recorded skip, never pass"
    );
    assert_eq!(
        out.json["reference_comparison"]["gate"], "exit::OK",
        "a skipped family exits OK but is never counted as pass"
    );

    let after = active_generation(&db).await;
    assert_eq!(before, after, "verify must leave corpus_generation unchanged");
}
