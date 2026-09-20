//! `qai quran catalog` acceptance: metadata-only ingestion of an upstream
//! `editions.json` file (no database, no text import).

use application::quran_cli::{CommandOutput, cmd_catalog};

const MINI_CATALOG: &str = r#"{
    "ara_quranuthmanihaf": {
        "name": "ara-quranuthmanihaf",
        "author": "Quran Uthmani Hafs",
        "language": "Arabic",
        "direction": "rtl",
        "source": "https://qurancomplex.gov.sa/",
        "comments": "Version 13",
        "link": "https://cdn.example/ara-quranuthmanihaf.json",
        "linkmin": "https://cdn.example/ara-quranuthmanihaf.min.json"
    },
    "eng_abdelhaleem": {
        "name": "eng-abdelhaleem",
        "author": "Abdel Haleem",
        "language": "English",
        "direction": "ltr",
        "source": "",
        "comments": "",
        "link": "https://cdn.example/eng-abdelhaleem.json",
        "linkmin": "https://cdn.example/eng-abdelhaleem.min.json"
    }
}"#;

fn write_catalog(dir: &tempfile::TempDir, name: &str, text: &str) -> String {
    let path = dir.path().join(name);
    std::fs::write(&path, text).unwrap();
    path.to_str().unwrap().to_string()
}

#[tokio::test]
async fn catalog_reports_metadata_and_stays_unpinned_without_revision() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_catalog(&dir, "editions.json", MINI_CATALOG);
    let CommandOutput { exit, human, json } = cmd_catalog(&path, None, None, None).await;
    assert_eq!(exit, 0);
    assert!(human.contains("entries: 2 (quran_text: 1, translation: 1,"));
    assert!(human.contains("named transmissions: 1"));
    assert!(human.contains("ara-quranuthmanihaf: hafs (inferred-from-upstream-name:uthmanihaf)"));
    assert!(human.contains("revision: unpinned"));
    assert!(human.contains("licenses: all unknown"));
    assert_eq!(json["summary"]["entries"], 2);
    assert_eq!(json["summary"]["pinned"], false);
    assert_eq!(json["entries"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn catalog_pins_revision_and_writes_report() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_catalog(&dir, "editions.json", MINI_CATALOG);
    let report = dir.path().join("report.json");
    let report_str = report.to_str().unwrap().to_string();
    let CommandOutput { exit, human, json } =
        cmd_catalog(&path, Some("abc123"), Some(&report_str), None).await;
    assert_eq!(exit, 0);
    assert!(human.contains("revision: abc123"));
    assert_eq!(json["summary"]["pinned"], true);
    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(written["entries"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn catalog_rejects_unreadable_paths_and_malformed_catalogs() {
    // Exit 2 (usage) for an unreadable path.
    let missing = cmd_catalog("/nonexistent/editions.json", None, None, None).await;
    assert_eq!(missing.exit, 2);
    assert!(missing.human.starts_with("error: cannot read manifest"));

    // Exit 3 (validation) for a malformed catalog.
    let dir = tempfile::tempdir().unwrap();
    let path = write_catalog(&dir, "bad.json", "{not json");
    let bad = cmd_catalog(&path, None, None, None).await;
    assert_eq!(bad.exit, 3);

    // Exit 2 (usage) when the report cannot be written.
    let good = write_catalog(&dir, "editions.json", MINI_CATALOG);
    let unwritable = cmd_catalog(&good, None, Some("/nonexistent-dir/report.json"), None).await;
    assert_eq!(unwritable.exit, 2);
}

#[tokio::test]
async fn catalog_describes_mirrored_text_files_per_edition() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_catalog(&dir, "editions.json", MINI_CATALOG);
    let database = dir.path().join("database");
    std::fs::create_dir_all(database.join("chapterverse")).unwrap();
    std::fs::create_dir_all(database.join("linebyline")).unwrap();
    let chapterverse = "1|1|Bismillah\n1|2|Praise\n{\n    \"name\": \"ara-quranuthmanihaf\"\n}";
    std::fs::write(database.join("chapterverse/ara-quranuthmanihaf.txt"), chapterverse).unwrap();
    let linebyline = "Bismillah\nPraise\n{\n    \"name\": \"ara-quranuthmanihaf\"\n}";
    std::fs::write(database.join("linebyline/ara-quranuthmanihaf.txt"), linebyline).unwrap();
    // linebyline without a trailer fails per-file but must not fail the run.
    std::fs::write(database.join("linebyline/eng-abdelhaleem.txt"), "Hello\n").unwrap();

    let CommandOutput { exit, human, json } =
        cmd_catalog(&path, None, None, Some(database.to_str().unwrap())).await;
    assert_eq!(exit, 0);
    assert!(human.contains("texts ("));
    assert!(human.contains("1/2 chapterverse, 1/2 linebyline"));
    assert_eq!(json["summary"]["with_chapterverse"], 1);
    assert_eq!(json["summary"]["with_linebyline"], 1);
    let texts = json["texts"].as_array().unwrap();
    assert_eq!(texts[0]["chapterverse"]["verses"], 2);
    assert!(texts[0]["chapterverse"]["sha256"].is_string());
    assert_eq!(texts[0]["linebyline"]["lines"], 2);
    assert_eq!(texts[1]["chapterverse"], serde_json::Value::Null);
    assert!(texts[1]["linebyline"]["error"].is_string());
}
