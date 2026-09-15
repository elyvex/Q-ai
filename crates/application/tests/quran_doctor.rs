//! Phase 1 — Quran corpus doctor acceptance: AC-P1-07 / AC-P1-17 + §5.4 bullet 6.
//!
//! Recomputes `text_hash`, `structure_hash`, and `token_order_hash` from the
//! stored canonical rows and asserts they equal the import-time values, then
//! runs the full 19-check `--deep` scan on the synthetic edition and asserts
//! nothing fails (license/reference-corpus checks may warn/skip by design).

#[path = "common/mod.rs"]
mod common;

use application::quran_doctor::{CheckLevel, run_quran_checks};
use common::active_reader;

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
