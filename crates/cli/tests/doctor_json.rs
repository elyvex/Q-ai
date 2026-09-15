//! AC-P1-17 / P1-T52 — `qai doctor --quran --json` must emit **one** JSON
//! document that validates against `docs/schemas/doctor.v1.schema.json`.
//!
//! Regression guard: the Phase-0 doctor and the Quran corpus doctor each used to
//! print their own `{"checks": [...]}` document back to back, so `--quran --json`
//! produced two concatenated objects and no JSON parser (or schema validator)
//! could consume it. Merging them into a single `checks` array is asserted here.

#[test]
fn doctor_quran_json_is_a_single_merged_document() {
    let dir = tempfile::tempdir().unwrap();
    let mut cfg = config::Config::default();
    cfg.app.data_dir = dir.path().display().to_string();
    cfg.storage.sqlite.path = dir.path().join("qai.db").display().to_string();
    cfg.storage.objects.root = dir.path().join("objects").display().to_string();

    let migrations =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    runtime.block_on(application::db::migrate_database(&cfg, &migrations)).unwrap();

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_qai"))
        .env("QAI_DATA_DIR", dir.path())
        .args(["doctor", "--quran", "--json"])
        .output()
        .expect("run qai doctor");

    // An empty corpus legitimately fails checks (non-zero exit); stdout must
    // still be exactly one parseable JSON document.
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout)
        .expect("`doctor --json` must be a single JSON document, not two concatenated ones");
    let ids: Vec<&str> = doc["checks"]
        .as_array()
        .expect("top-level `checks` array")
        .iter()
        .filter_map(|check| check["id"].as_str())
        .collect();
    assert!(ids.contains(&"configuration.valid"), "Phase-0 checks missing: {ids:?}");
    assert!(ids.contains(&"quran.edition_active"), "Quran checks not merged: {ids:?}");
    // The schema forbids extra top-level keys.
    let object = doc.as_object().unwrap();
    assert_eq!(object.len(), 1, "only `checks` is allowed at the top level: {object:?}");
}
