//! Phase 1 — importer acceptance: AC-P1-02/03/10/11 (P1-T25–T29).
//!
//! Against real SQLite in a tempdir: full import to `Staged`, the 13-prefix
//! crash matrix, cancellation cleanup, validation-failure abort, a worker-level
//! job run, and the approval-gated activation/rollback services.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use application::job_queue::SqliteJobQueue;
use application::quran::{
    QURAN_IMPORT_KIND, QuranImportHandler, activate_edition, import_idempotency_key,
    rollback_edition,
};
use domain::{PrincipalId, Timestamp};
use jobs::queue::JobQueue;
use jobs::registry::HandlerRegistry;
use jobs::worker::Worker;
use quran_corpus::import::{
    ImportCheckpoint, ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import,
};
use quran_corpus::{sha256_hex, validate_edition};
use storage::Database as _;
use storage::repository::JobRecord;
use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

const BASE_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const MISSING_AYAH: &str =
    include_str!("../../../fixtures/quran/adversarial/missing_ayah/manifest.json");
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
    // Seed FK parents through raw SQL on a scratch pool (mirrors storage-sqlite tests).
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
            "INSERT INTO provenance_records
                (id, layer, subject_urn, attribution_kind, attribution_json,
                 source_version_id, trust_level, verification_status, versions_json,
                 created_at, created_by)
             VALUES ('prov-1', 'canonical_source', 'test', 'dataset', '{{}}', 'sv-1',
                     'CanonicalVerified', 'unverified', '{{}}', '{CREATED_AT}', '{PRINCIPAL}')"
        ),
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
             VALUES ('appr-2', 'quran-edition:test-edition-min@0.2.0', 'CanonicalChange',
                     '{PRINCIPAL}', '{PRINCIPAL}', 'approved', '{{}}', '{CREATED_AT}', '{CREATED_AT}')"
        ),
    ] {
        sqlx::query(&sql).execute(&seed).await.unwrap();
    }
    seed.close().await;
    (dir, db)
}

fn input(run_id: &str, manifest: &str, declared: Option<String>) -> ImportInput {
    ImportInput {
        run_id: run_id.into(),
        job_id: None,
        source_version_id: "sv-1".into(),
        adapter: "json".into(),
        manifest_text: manifest.into(),
        declared_manifest_hash: declared,
        invoked_by: PRINCIPAL.into(),
        license_status: "PublicDomain".into(),
        license_json: "{}".into(),
        created_at: CREATED_AT.into(),
    }
}

async fn staged_count(db: &SqliteDatabase, run_id: &str) -> i64 {
    let mut uow = db.write().await.unwrap();
    let count = uow.quran().count_stg_ayahs(run_id).await.unwrap();
    uow.rollback().await.unwrap();
    count
}

#[tokio::test]
async fn import_runs_end_to_end_to_staged() {
    let (_dir, db) = migrated_db().await;
    let manifest_hash = sha256_hex(BASE_MANIFEST.as_bytes());
    let progress = ImportProgress::new();
    let outcome = run_import(
        &db,
        &input("run-1", BASE_MANIFEST, Some(manifest_hash)),
        &ImportOptions::default(),
        &AtomicBool::new(false),
        progress.clone(),
    )
    .await
    .expect("base import completes");
    let ImportOutcome::Completed(success) = outcome else { panic!("expected completion"); };
    assert!(success.stopped_at.is_none());
    assert_eq!(success.edition_id, "ed-run-1");
    assert_eq!(progress.checkpoints(), ImportCheckpoint::ALL);
    assert_eq!(staged_count(&db, "run-1").await, 14);

    let mut uow = db.write().await.unwrap();
    let report = uow.quran().get_validation_report("run-1").await.unwrap().unwrap();
    assert_eq!(report.fatal_count, 0);
    assert_eq!(report.error_count, 0);
    assert!(uow.quran().get_active().await.unwrap().is_none(), "no canonical writes");
    assert_eq!(uow.quran().count_ayahs("ed-run-1").await.unwrap(), 0);
    uow.rollback().await.unwrap();
}

#[tokio::test]
async fn crash_matrix_all_thirteen_checkpoints_leave_active_untouched() {
    let (_dir, db) = migrated_db().await;
    for (index, checkpoint) in ImportCheckpoint::ALL.iter().enumerate() {
        let run_id = format!("run-{index}");
        let progress = ImportProgress::new();
        let outcome = run_import(
            &db,
            &input(&run_id, BASE_MANIFEST, None),
            &ImportOptions { stop_after: Some(*checkpoint) },
            &AtomicBool::new(false),
            progress.clone(),
        )
        .await
        .expect("prefix run halts cleanly");
        let ImportOutcome::Completed(success) = outcome else { panic!("expected completion"); };
        if index < ImportCheckpoint::ALL.len() - 1 {
            assert_eq!(success.stopped_at, Some(*checkpoint));
        } else {
            // The terminal checkpoint runs to completion by definition.
            assert_eq!(success.stopped_at, None);
            assert_eq!(staged_count(&db, &run_id).await, 14);
        }
        assert_eq!(progress.checkpoints(), &ImportCheckpoint::ALL[..=index]);
        let mut uow = db.write().await.unwrap();
        assert!(
            uow.quran().get_active().await.unwrap().is_none(),
            "active unchanged after kill at {checkpoint:?}"
        );
        uow.rollback().await.unwrap();
    }
    // Retry after any kill completes the import.
    let progress = ImportProgress::new();
    let outcome = run_import(
        &db,
        &input("run-full", BASE_MANIFEST, None),
        &ImportOptions::default(),
        &AtomicBool::new(false),
        progress.clone(),
    )
    .await
    .expect("retry completes");
    assert!(matches!(outcome, ImportOutcome::Completed(_)));
    assert_eq!(staged_count(&db, "run-full").await, 14);
}

#[tokio::test]
async fn cancel_cleans_staging_and_marks_cancelled() {
    let (_dir, db) = migrated_db().await;
    // Reach staging, then cancel a retry of the same run.
    let progress = ImportProgress::new();
    run_import(
        &db,
        &input("run-1", BASE_MANIFEST, None),
        &ImportOptions { stop_after: Some(ImportCheckpoint::Staged) },
        &AtomicBool::new(false),
        progress,
    )
    .await
    .unwrap();
    assert_eq!(staged_count(&db, "run-1").await, 14);

    let err = run_import(
        &db,
        &input("run-1", BASE_MANIFEST, None),
        &ImportOptions::default(),
        &AtomicBool::new(true),
        ImportProgress::new(),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, quran_corpus::CorpusError::ImportCancelled { .. }));
    assert_eq!(staged_count(&db, "run-1").await, 0);
    let mut uow = db.write().await.unwrap();
    let run = uow.quran().get_import_run("run-1").await.unwrap().unwrap();
    assert_eq!(run.state, "Cancelled");
    uow.rollback().await.unwrap();
}

#[tokio::test]
async fn validation_failure_aborts_before_staging_with_report() {
    let (_dir, db) = migrated_db().await;
    let err = run_import(
        &db,
        &input("run-bad", MISSING_AYAH, None),
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .unwrap_err();
    let quran_corpus::CorpusError::ValidationFailed { fatal_count, report_id, .. } = err else {
        panic!("expected ValidationFailed, got {err}");
    };
    assert!(fatal_count > 0);
    assert_eq!(staged_count(&db, "run-bad").await, 0);
    let mut uow = db.write().await.unwrap();
    let report = uow.quran().get_validation_report(&report_id).await.unwrap().unwrap();
    assert!(report.fatal_count > 0);
    assert!(uow.quran().get_active().await.unwrap().is_none());
    uow.rollback().await.unwrap();
}

#[tokio::test]
async fn hash_mismatch_aborts_at_the_hash_checkpoint() {
    let (_dir, db) = migrated_db().await;
    let err = run_import(
        &db,
        &input("run-hash", BASE_MANIFEST, Some("0".repeat(64))),
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, quran_corpus::CorpusError::ImportFailed { .. }));
    assert_eq!(staged_count(&db, "run-hash").await, 0);
}

#[tokio::test]
async fn worker_runs_the_import_job_to_staged_with_audit() {
    let (_dir, db) = migrated_db().await;
    let db = Arc::new(db);
    let registry =
        Arc::new(HandlerRegistry::new().register(Arc::new(QuranImportHandler::new(db.clone()))));
    assert!(registry.get(application::quran::QURAN_IMPORT_KIND).is_some());
    let queue = Arc::new(SqliteJobQueue::new(db.clone()));
    let payload = serde_json::to_value(input("run-worker", BASE_MANIFEST, None)).unwrap();
    queue
        .enqueue(JobRecord {
            id: "job-1".into(),
            kind: QURAN_IMPORT_KIND.into(),
            payload_json: payload.to_string(),
            idempotency_key: Some(import_idempotency_key("sv-1")),
            state: "Queued".into(),
            priority: 0,
            attempts: 0,
            max_attempts: 5,
            available_at: CREATED_AT.into(),
            lease_owner: None,
            lease_expires_at: None,
            checkpoint_json: None,
            cancel_requested: false,
            created_by: PRINCIPAL.into(),
        })
        .await
        .unwrap();
    let worker = Worker::new(queue.clone(), registry, "test-owner");
    assert_eq!(worker.run_until_idle().await.unwrap(), 1);
    let job = queue.get("job-1").await.unwrap().unwrap();
    assert_eq!(job.state, "Succeeded");
    assert_eq!(staged_count(&db, "run-worker").await, 14);

    let mut uow = db.write().await.unwrap();
    let events = uow.audit().list_by_subject(V1_URN).await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].action, "source_staged");
    uow.rollback().await.unwrap();
}

#[tokio::test]
async fn activation_service_requires_a_granted_approval() {
    let (_dir, db) = migrated_db().await;
    run_import(
        &db,
        &input("run-1", BASE_MANIFEST, None),
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .unwrap();

    let err =
        activate_edition(&db, "test-edition-min", "0.1.0", &principal(), "missing", &timestamp())
            .await
            .unwrap_err();
    assert!(matches!(err, application::quran::ActivationError::ApprovalMissing { .. }));

    // Record a denied approval and a mismatched one.
    let mut uow = db.write().await.unwrap();
    for (id, decision, subject) in
        [("appr-denied", "denied", V1_URN), ("appr-other", "approved", "quran-edition:other@9.9.9")]
    {
        uow.sources()
            .insert_approval(storage::repository::ApprovalRow {
                id: id.into(),
                subject_urn: subject.into(),
                kind: "CanonicalChange".into(),
                requested_by: Some(PRINCIPAL.into()),
                decided_by: Some(PRINCIPAL.into()),
                decision: Some(decision.into()),
                request_payload: "{}".into(),
                decision_note: None,
                requested_at: CREATED_AT.into(),
                decided_at: Some(CREATED_AT.into()),
            })
            .await
            .unwrap();
    }
    uow.commit().await.unwrap();
    let err = activate_edition(
        &db,
        "test-edition-min",
        "0.1.0",
        &principal(),
        "appr-denied",
        &timestamp(),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, application::quran::ActivationError::ApprovalNotGranted { .. }));
    let err = activate_edition(
        &db,
        "test-edition-min",
        "0.1.0",
        &principal(),
        "appr-other",
        &timestamp(),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, application::quran::ActivationError::ApprovalSubjectMismatch { .. }));

    let generation =
        activate_edition(&db, "test-edition-min", "0.1.0", &principal(), "appr-1", &timestamp())
            .await
            .unwrap();
    assert_eq!(generation, 1);
    let mut uow = db.write().await.unwrap();
    let active = uow.quran().get_active().await.unwrap().unwrap();
    assert_eq!(active.corpus_generation, 1);
    let events = uow.audit().list_by_subject(V1_URN).await.unwrap();
    assert!(events.iter().any(|event| event.action == "source_activated"));
    uow.rollback().await.unwrap();
}

#[tokio::test]
async fn rollback_service_restores_the_prior_version() {
    let (_dir, db) = migrated_db().await;
    run_import(
        &db,
        &input("run-1", BASE_MANIFEST, None),
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .unwrap();
    let v2_manifest = BASE_MANIFEST.replace("\"version\": \"0.1.0\"", "\"version\": \"0.2.0\"");
    // The validator only needs the fixture's own consistency; versions differ.
    let v2_source: quran_corpus::EditionSource = serde_json::from_str(&v2_manifest).unwrap();
    assert_eq!(validate_edition(&v2_source).fatal_count, 0);
    run_import(
        &db,
        &input("run-2", &v2_manifest, None),
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .unwrap();

    assert_eq!(
        activate_edition(&db, "test-edition-min", "0.1.0", &principal(), "appr-1", &timestamp())
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        activate_edition(&db, "test-edition-min", "0.2.0", &principal(), "appr-2", &timestamp())
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        rollback_edition(&db, "test-edition-min", "0.1.0", &principal(), "appr-1", &timestamp())
            .await
            .unwrap(),
        3
    );
    let mut uow = db.write().await.unwrap();
    let active = uow.quran().get_active().await.unwrap().unwrap();
    assert_eq!(active.corpus_generation, 3);
    let edition = uow.quran().get_edition(&active.edition_id).await.unwrap().unwrap();
    assert_eq!(edition.version, "0.1.0");
    uow.rollback().await.unwrap();
}
