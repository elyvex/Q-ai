//! CLI acceptance snapshots — end-to-end flows (AC-P1-16, P1-T50, P2-T23).
//!
//! Points the real `qai` binary at a fresh temp database via `QAI_DATA_DIR`,
//! then drives `trycmd` cases (see `tests/quran/`). Cases in a file run in
//! order against the one database; human output elides volatile values.
//!
//! Each file gets its own database: trycmd files execute in parallel, so two
//! files sharing one `QAI_DATA_DIR` race (notably two `db migrate` runs on
//! the same SQLite file).

fn run_cases(pattern: &str) {
    let dir = tempfile::tempdir().unwrap();
    trycmd::TestCases::new()
        .default_bin_name("qai")
        .env("QAI_DATA_DIR", dir.path().to_str().unwrap())
        .case("tests/quran/*.toml")
        .case(pattern);
    // Keep the temp database alive until assertions complete.
    drop(dir);
}

/// Phase 1 reading flow (AC-P1-16, P1-T50).
#[test]
fn quran_snapshots() {
    run_cases("tests/quran/read_flow.trycmd");
}

/// Phase 2 normalization introspection (P2-T23).
#[test]
fn quran_normalize_snapshots() {
    run_cases("tests/quran/normalize.trycmd");
}
