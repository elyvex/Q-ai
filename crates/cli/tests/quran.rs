//! Phase 1 — CLI snapshot acceptance: AC-P1-16 (P1-T50).
//!
//! `QAI_DATA_DIR` points the real `qai` binary at a temp database; cases run
//! in filename order (`db migrate` first). Human outputs avoid volatile ids;
//! `--json` cases run after fixed inputs so structures are deterministic.

#[test]
fn quran_snapshots() {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("QAI_DATA_DIR", dir.path());
    // Keep the directory alive for the whole trycmd run.
    let _guard = dir;
    trycmd::TestCases::new().case("tests/quran/*.toml");
}
