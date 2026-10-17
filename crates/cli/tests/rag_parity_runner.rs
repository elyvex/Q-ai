//! Phase 8 RAG parity runner.

fn run_cases(pattern: &str) {
    let dir = tempfile::tempdir().unwrap();
    trycmd::TestCases::new()
        .default_bin_name("qai")
        .env("QAI_DATA_DIR", dir.path().to_str().unwrap())
        .case(pattern);
    drop(dir);
}

#[test]
fn rag_parity_snapshots() {
    run_cases("tests/rag_parity.trycmd");
}
