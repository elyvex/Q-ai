//! Phase 2 — `quran.index.build` acceptance (M2, P2-T33/T34).
//!
//! Against real SQLite in a tempdir with a real imported + activated edition
//! and real derived forms: staged builds, atomic pointer flips, generation
//! retention, searchability of the serving generation, cancellation safety,
//! and the job-handler path. Crash-mid-build (kill at each stage) belongs to
//! P2-T37 with the process harness.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use application::quran::activate_edition;
use application::quran_forms::{RebuildParams, rebuild_forms};
use application::quran_index::{
    IndexBuildParams, QURAN_AYAH_INDEX_ID, QURAN_INDEX_BUILD_KIND, IndexBuildHandler,
};
use domain::{PrincipalId, Timestamp};
use jobs::{JobContext, JobHandler};
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use quran_corpus::sha256_hex;
use quran_search::{FullTextIndex, Fts5Index, TokenizerFamily};
use storage::Database as _;
use storage::repository::JobRecord;
use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

const BASE_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const PRINCIPAL: &str = "00000000-0000-0000-0000-000000000001";
const CREATED_AT: &str = "2026-09-14T00:00:00Z";
const V1_URN: &str = "quran-edition:test-edition-min@0.1.0";

fn principal() -> PrincipalId {
    PRINCIPAL.parse().unwrap()
}

fn timestamp() -> Timestamp {
    Timestamp::from_ymd_hms(2026, 9, 14, 0, 0, 0).unwrap()
}

async fn migrated_db() -> (tempfile::TempDir, SqliteDatabase) {
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
    ] {
        sqlx::query(&sql).execute(&seed).await.unwrap();
    }
    seed.close().await;
    (dir, db)
}

/// Import, activate, and build derived forms: the preconditions for indexing.
async fn ready_db() -> (tempfile::TempDir, SqliteDatabase, String) {
    let (dir, db) = migrated_db().await;
    let outcome = run_import(
        &db,
        &ImportInput {
            run_id: "run-idx-1".into(),
            job_id: None,
            source_version_id: "sv-1".into(),
            adapter: "json".into(),
            manifest_text: BASE_MANIFEST.into(),
            declared_manifest_hash: Some(sha256_hex(BASE_MANIFEST.as_bytes())),
            invoked_by: PRINCIPAL.into(),
            license_status: "PublicDomain".into(),
            license_json: "{}".into(),
            created_at: CREATED_AT.into(),
        },
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .expect("fixture import completes");
    let ImportOutcome::Completed(success) = outcome;
    activate_edition(&db, "test-edition-min", "0.1.0", &principal(), "appr-1", &timestamp())
        .await
        .expect("fixture activation completes");
    rebuild_forms(
        &db,
        &RebuildParams {
            edition_slug: "test-edition-min".to_string(),
            edition_version: "0.1.0".to_string(),
            invoked_by: PRINCIPAL.to_string(),
            run_tag: "idx-test".to_string(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("forms rebuild completes");
    (dir, db, success.edition_id)
}

fn params(data_dir: &std::path::Path, tag: &str) -> IndexBuildParams {
    IndexBuildParams {
        index_id: QURAN_AYAH_INDEX_ID.to_string(),
        edition_slug: "test-edition-min".to_string(),
        edition_version: "0.1.0".to_string(),
        invoked_by: PRINCIPAL.to_string(),
        run_tag: tag.to_string(),
        data_dir: data_dir.to_path_buf(),
    }
}

async fn pointer(db: &SqliteDatabase) -> Option<storage::quran::IndexPointerRow> {
    let mut uow = db.write().await.unwrap();
    let pointer = uow.quran().get_index_pointer(QURAN_AYAH_INDEX_ID).await.unwrap();
    uow.rollback().await.unwrap();
    pointer
}

/// First build activates generation 1; the serving generation answers search.
#[tokio::test]
async fn first_build_activates_and_serves() {
    let (dir, db, _edition) = ready_db().await;
    let data_dir = dir.path().join("index");
    let report = application::quran_index::rebuild_index(
        &db,
        &params(&data_dir, "t1"),
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("index build completes");

    assert_eq!(report.index_id, QURAN_AYAH_INDEX_ID);
    assert_eq!(report.generation, 1);
    assert_eq!(report.corpus_generation, 1);
    assert_eq!(report.doc_count, 14);
    assert!(report.previous_generation.is_none());
    assert!(report.mv018.unchanged);
    assert!(report.manifest_hash.starts_with("sha256:"));

    let pointer = pointer(&db).await.expect("pointer flipped");
    assert_eq!(pointer.generation, 1);
    assert!(pointer.manifest_json.contains("quran.ayah.v1"));

    // The serving generation answers real search through the adapter.
    let manifest: quran_search::IndexManifest =
        serde_json::from_str(&pointer.manifest_json).unwrap();
    assert_eq!(manifest.doc_count, 14);
    let registry = application::quran_normalize::builtin_registry();
    let family =
        TokenizerFamily::new(&registry, manifest.tokenizer_version).unwrap();
    let index =
        Fts5Index::open(&data_dir, 1, manifest, family).await.expect("serving generation opens");
    let total = index.count(&quran_search::FtsQuery::All).await.unwrap();
    assert_eq!(total, 14);
    let stats = index.stats().await.unwrap();
    assert_eq!(stats.doc_count, 14);
    assert!(index.verify().await.unwrap().ok);
}

/// Second build flips to generation 2 and retains generation 1 on disk.
#[tokio::test]
async fn second_build_flips_and_retains_previous() {
    let (dir, db, _edition) = ready_db().await;
    let data_dir = dir.path().join("index");
    application::quran_index::rebuild_index(&db, &params(&data_dir, "t1"), &AtomicBool::new(false), |_| {})
        .await
        .unwrap();
    let second = application::quran_index::rebuild_index(
        &db,
        &params(&data_dir, "t2"),
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("second build completes");

    assert_eq!(second.generation, 2);
    assert_eq!(second.previous_generation, Some(1));
    let pointer = pointer(&db).await.expect("pointer flipped");
    assert_eq!(pointer.generation, 2);
    // Previous generation retained for single-step rollback.
    assert!(data_dir.join("gen-1").exists());
    assert!(data_dir.join("gen-2").exists());

    let mut uow = db.write().await.unwrap();
    let runs = uow.quran().list_build_runs(QURAN_AYAH_INDEX_ID).await.unwrap();
    uow.rollback().await.unwrap();
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].state, "superseded");
    assert_eq!(runs[1].state, "active");
}

/// Cancellation leaves the pointer untouched and commits nothing new.
#[tokio::test]
async fn cancelled_build_changes_nothing() {
    let (dir, db, _edition) = ready_db().await;
    let data_dir = dir.path().join("index");
    application::quran_index::rebuild_index(&db, &params(&data_dir, "t1"), &AtomicBool::new(false), |_| {})
        .await
        .unwrap();
    let err = application::quran_index::rebuild_index(
        &db,
        &params(&data_dir, "t2"),
        &AtomicBool::new(true),
        |_| {},
    )
    .await
    .unwrap_err();
    assert!(matches!(err, application::quran_index::IndexBuildError::Cancelled));
    let pointer = pointer(&db).await.expect("pointer still generation 1");
    assert_eq!(pointer.generation, 1);
}

/// The job handler runs the same core through the worker contract.
#[tokio::test]
async fn handler_runs_through_job_contract() {
    use jobs::JobError;
    let (dir, db, _edition) = ready_db().await;
    let data_dir = dir.path().join("index");
    let handler = IndexBuildHandler::new(Arc::new(db));
    assert_eq!(handler.kind().to_string(), application::quran_index::QURAN_INDEX_BUILD_KIND);
    assert!(handler.is_idempotent());
    let record = JobRecord {
        id: "job-index-1".to_string(),
        kind: application::quran_index::QURAN_INDEX_BUILD_KIND.to_string(),
        payload_json: serde_json::json!({
            "edition_slug": "test-edition-min",
            "edition_version": "0.1.0",
            "invoked_by": PRINCIPAL,
        })
        .to_string(),
        idempotency_key: None,
        state: "Running".to_string(),
        priority: 0,
        attempts: 0,
        max_attempts: 3,
        available_at: CREATED_AT.to_string(),
        lease_owner: None,
        lease_expires_at: None,
        checkpoint_json: None,
        cancel_requested: false,
        created_by: PRINCIPAL.to_string(),
    };
    // NOTE: data_dir travels in the job payload (handlers only hold the DB).
    let outcome = handler
        .run(
            JobContext::new(record),
            serde_json::json!({
                "edition_slug": "test-edition-min",
                "edition_version": "0.1.0",
                "invoked_by": PRINCIPAL,
                "data_dir": data_dir.to_str().unwrap(),
            }),
        )
        .await;
    match outcome {
        Ok(outcome) => assert!(outcome.success),
        Err(JobError::Storage(detail)) => panic!("handler failed: {detail}"),
        Err(other) => panic!("unexpected job error: {other:?}"),
    }
}
