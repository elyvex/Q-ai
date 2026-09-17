//! AC-P1-17 / P1-T52 — `qai doctor --quran --json` must emit **one** JSON
//! document that validates against `docs/schemas/doctor.v1.schema.json`.
//!
//! Regression guard: the Phase-0 doctor and the Quran corpus doctor each used to
//! print their own `{"checks": [...]}` document back to back, so `--quran --json`
//! produced two concatenated objects and no JSON parser (or schema validator)
//! could consume it. Merging them into a single `checks` array is asserted here.

#[test]
fn audit_verify_rejects_corrupt_chain_without_modifying_database() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("qai.db");
    let run = || {
        std::process::Command::new(env!("CARGO_BIN_EXE_qai"))
            .args(["--data-dir", dir.path().to_str().unwrap(), "audit", "verify", "--json"])
            .output()
            .unwrap()
    };
    let missing = run();
    assert!(!missing.status.success());
    assert!(!path.exists());
    let missing_json: serde_json::Value = serde_json::from_slice(&missing.stdout).unwrap();
    assert_eq!(missing_json["valid"], false);

    let mut cfg = config::Config::default();
    cfg.storage.sqlite.path = path.display().to_string();
    let migrations =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    runtime.block_on(application::db::migrate_database(&cfg, &migrations)).unwrap();
    let empty = run();
    assert!(empty.status.success());
    let report: serde_json::Value = serde_json::from_slice(&empty.stdout).unwrap();
    assert_eq!(report["valid"], true);
    assert_eq!(report["checked_events"], 0);

    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/quran/test-edition-min/manifest.json");
    let imported = std::process::Command::new(env!("CARGO_BIN_EXE_qai"))
        .args([
            "--data-dir",
            dir.path().to_str().unwrap(),
            "quran",
            "import",
            manifest.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(imported.status.success(), "{}", String::from_utf8_lossy(&imported.stderr));
    let valid = run();
    assert!(valid.status.success(), "{}", String::from_utf8_lossy(&valid.stdout));
    let valid_report: serde_json::Value = serde_json::from_slice(&valid.stdout).unwrap();
    assert_eq!(valid_report["valid"], true);
    let event_count = valid_report["checked_events"].as_u64().unwrap();
    assert!(event_count > 0);

    runtime.block_on(async {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&path))
            .await.unwrap();
        sqlx::query("DROP TRIGGER trg_audit_no_update").execute(&pool).await.unwrap();
        sqlx::query("UPDATE audit_events SET chain_hash = ?, reason = ? WHERE sequence = (SELECT MAX(sequence) FROM audit_events)")
            .bind(format!("sha256:{}", "ab".repeat(32)))
            .bind("password=CLI_AUDIT_SENTINEL")
            .execute(&pool).await.unwrap();
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)").execute(&pool).await.unwrap();
        pool.close().await;
    });
    let before = std::fs::read(&path).unwrap();
    let corrupt = run();
    assert_eq!(corrupt.status.code(), Some(3));
    let report: serde_json::Value = serde_json::from_slice(&corrupt.stdout).unwrap();
    assert_eq!(report["valid"], false);
    assert_eq!(report["checked_events"], event_count);
    assert_eq!(report["tampered_sequences"], serde_json::json!([event_count]));
    assert!(!String::from_utf8_lossy(&corrupt.stdout).contains("CLI_AUDIT_SENTINEL"));
    assert!(!String::from_utf8_lossy(&corrupt.stderr).contains("CLI_AUDIT_SENTINEL"));
    assert_eq!(before, std::fs::read(&path).unwrap());
}

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
