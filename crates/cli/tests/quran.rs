//! Phase 1 CLI acceptance — end-to-end reading flow (AC-P1-16, P1-T50).
//!
//! Points the real `qai` binary at a fresh temp database via `QAI_DATA_DIR`,
//! then drives `trycmd` cases (see `tests/quran/`). Cases in a file run in
//! order against the one database; human output elides volatile values.

#[test]
fn quran_snapshots() {
    let dir = tempfile::tempdir().unwrap();
    trycmd::TestCases::new()
        .default_bin_name("qai")
        .env("QAI_DATA_DIR", dir.path().to_str().unwrap())
        .case("tests/quran/*.toml")
        .case("tests/quran/*.trycmd");
    // Keep the temp database alive until assertions complete.
    drop(dir);
}
