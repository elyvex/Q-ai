//! Shared acceptance-test harness: migrated tempdb + imported + activated
//! synthetic edition + reader (used by `quran_reader` and `quran_tools` suites).

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use application::quran_reader::QuranReaderService;
use domain::{PrincipalId, Timestamp};
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use quran_corpus::sha256_hex;
use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

pub const BASE_MANIFEST: &str =
    include_str!("../../../../fixtures/quran/test-edition-min/manifest.json");
pub const RUN_ID: &str = "11111111-2222-4333-8444-555555555555";
pub const PRINCIPAL: &str = "00000000-0000-0000-0000-000000000001";
pub const CREATED_AT: &str = "2026-09-14T00:00:00Z";
pub const SLUG: &str = "test-edition-min";
pub const VERSION: &str = "0.1.0";
pub const SOURCE_VERSION_ID: &str = "12345678-1234-1234-1234-123456789abc";
pub const LICENSE_JSON: &str = "{\"status\":\"PublicDomain\",\"spdx_id\":null,\"name\":null,\
                                  \"url\":null,\"attribution_required\":false,\
                                  \"redistribution_allowed\":true,\"export_allowed\":true,\
                                  \"notes\":null}";

pub fn principal() -> PrincipalId {
    PRINCIPAL.parse().unwrap()
}

pub fn timestamp() -> Timestamp {
    Timestamp::from_ymd_hms(2026, 9, 14, 0, 0, 0).unwrap()
}

pub fn input(run_id: &str, manifest: &str) -> ImportInput {
    ImportInput {
        run_id: run_id.into(),
        job_id: None,
        source_version_id: SOURCE_VERSION_ID.into(),
        adapter: "json".into(),
        manifest_text: manifest.into(),
        declared_manifest_hash: Some(sha256_hex(manifest.as_bytes())),
        invoked_by: PRINCIPAL.into(),
        license_status: "PublicDomain".into(),
        license_json: LICENSE_JSON.into(),
        created_at: CREATED_AT.into(),
    }
}

pub async fn active_reader() -> (tempfile::TempDir, Arc<SqliteDatabase>, QuranReaderService, String)
{
    let dir = tempdir().unwrap();
    let path = dir.path().join("qai.db");
    let path_str = path.to_str().unwrap().to_string();
    let repo_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    storage_sqlite::migrate::apply_migrations(&path_str, &repo_root).await.unwrap();
    let db = Arc::new(SqliteDatabase::new(&path_str, 4, true).await.unwrap());

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
        format!(
            "INSERT INTO source_versions
            (id, source_id, version, schema_version, state, trust_level,
             license_status, license_json, created_at)
         VALUES ('{SOURCE_VERSION_ID}', 'src-1', '0.1.0', 1, 'Staged', 'ImportedUnverified',
                 'PublicDomain', '{{}}', '2026-09-14T00:00:00Z')"
        ),
        format!(
            "INSERT INTO approvals
                (id, subject_urn, kind, requested_by, decided_by, decision,
                 request_payload, requested_at, decided_at)
             VALUES ('appr-1', 'quran-edition:{SLUG}@{VERSION}', 'CanonicalChange',
                     '{PRINCIPAL}', '{PRINCIPAL}', 'approved', '{{}}', '{CREATED_AT}', '{CREATED_AT}')"
        ),
    ] {
        sqlx::query(&sql).execute(&seed).await.unwrap();
    }
    seed.close().await;

    let outcome = run_import(
        &*db,
        &input(RUN_ID, BASE_MANIFEST),
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .expect("import completes");
    assert!(matches!(outcome, ImportOutcome::Completed(_)));
    let generation = application::quran::activate_edition(
        &*db,
        SLUG,
        VERSION,
        &principal(),
        "appr-1",
        &timestamp(),
    )
    .await
    .expect("activation completes");
    assert_eq!(generation, 1);
    let reader = QuranReaderService::new(db.clone());
    (dir, db, reader, path_str)
}
