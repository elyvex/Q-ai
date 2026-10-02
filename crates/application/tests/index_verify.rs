//! Phase 3.5 — index verification job acceptance (P2-T107, D-3.5-09).
//!
//! Against real SQLite in tempdirs: the read-only index verification job
//! detects drift, reports sample metadata, and is byte-identical read-only.

use std::sync::atomic::AtomicBool;

use application::quran::activate_edition;
use application::quran_forms::{RebuildParams, rebuild_forms};
use application::quran_index::{
    IndexBuildParams, IndexVerifyParams, QURAN_AYAH_INDEX_ID, QURAN_INDEX_VERIFY_KIND,
    rebuild_index, verify_index,
};
use application::quran_morphology::dataset_urn;
use domain::{PrincipalId, Timestamp};
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use quran_corpus::sha256_hex;
use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

const BASE_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const PRINCIPAL: &str = "00000000-0000-0000-0000-000000000001";
const CREATED_AT: &str = "2026-09-14T00:00:00Z";
const V1_URN: &str = "quran-edition:test-edition-min@0.1.0";
const V2_URN: &str = "quran-edition:test-edition-min@0.2.0";
const MORPH_SLUG: &str = "test-morph";
const MORPH_VERSION: &str = "0.1.0";

fn principal() -> PrincipalId {
    PRINCIPAL.parse().unwrap()
}

fn timestamp() -> Timestamp {
    Timestamp::from_ymd_hms(2026, 9, 14, 0, 0, 0).unwrap()
}

async fn migrated_db() -> (tempfile::TempDir, SqliteDatabase, String) {
    let dir = tempdir().unwrap();
    let path = dir.path().join("qai.db");
    let path_str = path.to_str().unwrap().to_string();
    let repo_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    storage_sqlite::migrate::apply_migrations(&path_str, &repo_root).await.unwrap();
    let db = SqliteDatabase::new(&path_str, 4, true).await.unwrap();
    let seed = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new().filename(&path_str).foreign_keys(true),
        )
        .await
        .unwrap();
    for sql in [
        format!(
            "INSERT INTO principals (id, kind, display_name, created_at)
             VALUES ('{PRINCIPAL}', 'local_user', 'Test', '{CREATED_AT}')"
        ),
        "INSERT INTO sources (id, title, content_type, created_at, updated_at)
         VALUES ('src-1', 'Test source', 'quran_edition', '2026-09-14T00:00:00Z', '2026-09-14T00:00:00Z')"
            .to_string(),
        "INSERT INTO source_versions
            (id, source_id, version, schema_version, state, trust_level,
             license_status, license_json, created_at)
         VALUES ('sv-1', 'src-1', '0.1.0', 1, 'Staged', 'ImportedUnverified',
                 'PublicDomain', '{}', '2026-09-14T00:00:00Z')"
            .to_string(),
        format!(
            "INSERT INTO approvals
                (id, subject_urn, kind, requested_by, decided_by, decision,
                 request_payload, requested_at, decided_at)
             VALUES ('appr-1', '{V1_URN}', 'CanonicalChange', '{PRINCIPAL}', '{PRINCIPAL}',
                     'approved', '{{}}', '{CREATED_AT}', '{CREATED_AT}')"
        ),
        format!(
            "INSERT INTO approvals
                (id, subject_urn, kind, requested_by, decided_by, decision,
                 request_payload, requested_at, decided_at)
             VALUES ('appr-v2', '{V2_URN}', 'CanonicalChange', '{PRINCIPAL}', '{PRINCIPAL}',
                     'approved', '{{}}', '{CREATED_AT}', '{CREATED_AT}')"
        ),
        format!(
            "INSERT INTO approvals
                (id, subject_urn, kind, requested_by, decided_by, decision,
                 request_payload, requested_at, decided_at)
             VALUES ('appr-morph', '{}', 'CanonicalChange', '{PRINCIPAL}', '{PRINCIPAL}',
                     'approved', '{{}}', '{CREATED_AT}', '{CREATED_AT}')",
            dataset_urn(MORPH_SLUG, MORPH_VERSION)
        ),
    ] {
        sqlx::query(&sql).execute(&seed).await.unwrap();
    }
    seed.close().await;
    (dir, db, path_str)
}

async fn ready_db() -> (tempfile::TempDir, SqliteDatabase, String) {
    let (dir, db, path_str) = migrated_db().await;
    let outcome = run_import(
        &db,
        &ImportInput {
            run_id: "run-index-verify-1".into(),
            job_id: None,
            source_version_id: "sv-1".into(),
            adapter: "json".into(),
            manifest_text: BASE_MANIFEST.into(),
            declared_manifest_hash: Some(sha256_hex(BASE_MANIFEST.as_bytes())),
            invoked_by: PRINCIPAL.into(),
            license_status: "PublicDomain".into(),
            license_json: "{}".into(),
            created_at: CREATED_AT.into(),
            reference_manifest_text: None,
        },
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .expect("fixture import completes");
    assert!(matches!(outcome, ImportOutcome::Completed(_)));
    activate_edition(&db, "test-edition-min", "0.1.0", &principal(), "appr-1", &timestamp())
        .await
        .expect("fixture activation completes");
    rebuild_forms(
        &db,
        &RebuildParams {
            edition_slug: "test-edition-min".to_string(),
            edition_version: "0.1.0".to_string(),
            invoked_by: PRINCIPAL.to_string(),
            run_tag: "index-verify-test".to_string(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("forms rebuild completes");
    (dir, db, path_str)
}

async fn build_index(db: &SqliteDatabase, data_dir: &std::path::Path) {
    rebuild_index(
        db,
        &IndexBuildParams {
            index_id: QURAN_AYAH_INDEX_ID.to_string(),
            edition_slug: "test-edition-min".to_string(),
            edition_version: "0.1.0".to_string(),
            invoked_by: PRINCIPAL.to_string(),
            run_tag: "index-verify-build".to_string(),
            data_dir: data_dir.to_path_buf(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("index build completes");
}

/// P2-T107: green on a freshly built index (no drift).
#[tokio::test]
async fn index_verify_green_on_fresh_index() {
    let (_dir, db, _path) = ready_db().await;
    let data_dir = _dir.path().join("index");
    build_index(&db, &data_dir).await;
    let report = verify_index(
        &db,
        &IndexVerifyParams {
            index_id: QURAN_AYAH_INDEX_ID.to_string(),
            deep: false,
            data_dir: data_dir.clone(),
        },
    )
    .await
    .expect("verify succeeds");
    assert!(!report.skipped, "must not skip with an active edition");
    assert!(!report.drift_detected, "fresh index must not drift");
    assert!(report.serving_generation.is_some());
    assert!(report.sample_size > 0, "sample size must be reported");
    assert_eq!(report.sample_rate, 0.01, "default sample rate is 1%");
}

/// P2-T107: drift detected after a generation bump with the correct code.
#[tokio::test]
async fn index_verify_detects_drift() {
    let (dir, db, _path) = ready_db().await;
    let data_dir = dir.path().join("index");
    build_index(&db, &data_dir).await;

    // Inject drift: import + activate v2 (new corpus generation).
    let mut v2: serde_json::Value = serde_json::from_str(BASE_MANIFEST).unwrap();
    v2["edition"]["version"] = serde_json::json!("0.2.0");
    let v2_text = serde_json::to_string(&v2).unwrap();
    run_import(
        &db,
        &ImportInput {
            run_id: "run-index-verify-2".into(),
            job_id: None,
            source_version_id: "sv-1".into(),
            adapter: "json".into(),
            manifest_text: v2_text,
            declared_manifest_hash: None,
            invoked_by: PRINCIPAL.into(),
            license_status: "PublicDomain".into(),
            license_json: "{}".into(),
            created_at: CREATED_AT.into(),
            reference_manifest_text: None,
        },
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .expect("v2 import completes");
    activate_edition(&db, "test-edition-min", "0.2.0", &principal(), "appr-v2", &timestamp())
        .await
        .expect("v2 activation completes");

    let report = verify_index(
        &db,
        &IndexVerifyParams {
            index_id: QURAN_AYAH_INDEX_ID.to_string(),
            deep: false,
            data_dir: data_dir.clone(),
        },
    )
    .await
    .expect("verify succeeds");
    assert!(!report.skipped);
    assert!(report.drift_detected, "drift must be detected after generation bump");
    assert!(
        report.drift_details.iter().any(|d| d.contains("corpus generation")),
        "drift details must name the generation mismatch: {:?}",
        report.drift_details
    );
}

/// P2-T107: read-only proven by byte-identical state before and after.
#[tokio::test]
async fn index_verify_is_read_only() {
    let (_dir, db, path_str) = ready_db().await;
    let data_dir = _dir.path().join("index");
    build_index(&db, &data_dir).await;
    let before = std::fs::read(&path_str).unwrap();
    verify_index(
        &db,
        &IndexVerifyParams {
            index_id: QURAN_AYAH_INDEX_ID.to_string(),
            deep: false,
            data_dir: data_dir.clone(),
        },
    )
    .await
    .expect("verify succeeds");
    let after = std::fs::read(&path_str).unwrap();
    assert_eq!(before, after, "verify must not mutate the database file");
}

/// P2-T107: sampled-report metadata present.
#[tokio::test]
async fn index_verify_reports_sample_metadata() {
    let (_dir, db, _path) = ready_db().await;
    let data_dir = _dir.path().join("index");
    build_index(&db, &data_dir).await;
    let report = verify_index(
        &db,
        &IndexVerifyParams {
            index_id: QURAN_AYAH_INDEX_ID.to_string(),
            deep: false,
            data_dir: data_dir.clone(),
        },
    )
    .await
    .expect("verify succeeds");
    assert!(report.sample_size > 0, "sample size must be reported");
    assert_eq!(report.sample_rate, 0.01, "sample rate must be 1%");
    assert!(
        report.sample_size <= report.serving_generation.unwrap_or(0) as usize + 1,
        "sample size must be bounded"
    );
}

/// P2-T107: empty-database Skipped with remedy.
#[tokio::test]
async fn index_verify_skipped_on_empty_database() {
    let (_dir, db, _path) = migrated_db().await;
    let data_dir = _dir.path().join("index");
    let report = verify_index(
        &db,
        &IndexVerifyParams {
            index_id: QURAN_AYAH_INDEX_ID.to_string(),
            deep: false,
            data_dir: data_dir.clone(),
        },
    )
    .await
    .expect("verify succeeds");
    assert!(report.skipped, "must skip with no active edition");
    assert!(!report.drift_detected);
    assert!(report.remedy.is_some(), "skipped must carry a remedy");
    assert!(
        report.remedy.as_deref().unwrap().contains("import"),
        "remedy must name the activation command"
    );
}

/// P2-T107: the job kind is registered and the handler is wired.
#[tokio::test]
async fn index_verify_handler_is_registered() {
    use jobs::JobHandler;
    let handler =
        application::quran_index::IndexVerifyHandler::new(std::sync::Arc::new(ready_db().await.1));
    assert_eq!(handler.kind().as_str(), QURAN_INDEX_VERIFY_KIND);
    assert!(handler.is_idempotent());
    let schema = handler.payload_schema();
    assert!(schema.contains("data_dir"), "payload schema must require data_dir");
}
