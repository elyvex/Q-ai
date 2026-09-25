//! Phase 2 — `quran.forms.rebuild` acceptance (M2, P2-T26/T28).
//!
//! Against real SQLite in a tempdir with a real imported + activated edition:
//! full rebuild counts, skeleton windows, idempotent re-runs, MV-018
//! canonical stability, pre/post-check wiring, cancellation safety, the
//! inactive-edition refusal, and the job-handler path.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use application::quran::activate_edition;
use application::quran_forms::{
    FormsError, FormsRebuildHandler, RebuildParams, rebuild_forms, verify_canonical_unchanged,
};
use domain::{PrincipalId, Timestamp};
use jobs::{JobContext, JobHandler};
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use quran_corpus::sha256_hex;
use quran_normalization::{ProfileId, ProfileRegistry, SemVer};
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

fn input(run_id: &str) -> ImportInput {
    input_manifest(run_id, BASE_MANIFEST)
}

fn input_manifest(run_id: &str, manifest: &str) -> ImportInput {
    ImportInput {
        run_id: run_id.into(),
        job_id: None,
        source_version_id: "sv-1".into(),
        adapter: "json".into(),
        manifest_text: manifest.into(),
        declared_manifest_hash: Some(sha256_hex(manifest.as_bytes())),
        invoked_by: PRINCIPAL.into(),
        license_status: "PublicDomain".into(),
        license_json: "{}".into(),
        created_at: CREATED_AT.into(),
        reference_manifest_text: None,
    }
}

/// Import + activate the fixture edition, returning its canonical id.
async fn active_edition(db: &SqliteDatabase) -> String {
    let outcome = run_import(
        db,
        &input("run-forms-1"),
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .expect("fixture import completes");
    let ImportOutcome::Completed(success) = outcome;
    activate_edition(db, "test-edition-min", "0.1.0", &principal(), "appr-1", &timestamp())
        .await
        .expect("fixture activation completes");
    success.edition_id
}

fn params() -> RebuildParams {
    RebuildParams {
        edition_slug: "test-edition-min".to_string(),
        edition_version: "0.1.0".to_string(),
        invoked_by: PRINCIPAL.to_string(),
        run_tag: "test-run".to_string(),
    }
}

async fn counts(db: &SqliteDatabase, edition_id: &str) -> (i64, i64) {
    let mut uow = db.write().await.unwrap();
    let tokens = uow.quran().count_tokens(edition_id).await.unwrap();
    let forms = uow.quran().count_token_forms(edition_id).await.unwrap();
    uow.rollback().await.unwrap();
    (tokens, forms)
}

/// Full rebuild: every token and ayah gets forms, windows cover the surahs,
/// and provenance lands at Layer D with computational attribution.
#[tokio::test]
async fn rebuild_covers_every_token_and_ayah() {
    let (_dir, db) = migrated_db().await;
    let edition_id = active_edition(&db).await;
    let report = rebuild_forms(&db, &params(), &AtomicBool::new(false), |_| {})
        .await
        .expect("rebuild completes");

    assert_eq!(report.ayahs, 14);
    let (tokens, forms) = counts(&db, &edition_id).await;
    assert!(tokens > 0, "fixture has tokens");
    assert_eq!(forms, tokens, "one token-form row per canonical token");
    assert_eq!(report.token_forms as i64, tokens);
    assert_eq!(report.ayah_forms, 14);
    assert!(report.skeletons >= 14, "ayah skeletons plus windows");
    assert!(report.mv018.unchanged);
    assert_eq!(report.generation, 1);
    assert_eq!(report.rule_set_version, "1.0.0");
    assert!(report.provenance_id.starts_with("prov-forms-"));

    // Layer-D provenance with computational attribution + confidence.
    let mut uow = db.write().await.unwrap();
    let records =
        uow.provenance().list_by_subject(&format!("{V1_URN}/derived-forms")).await.unwrap();
    uow.rollback().await.unwrap();
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.id, report.provenance_id);
    assert_eq!(record.layer, "computational_annotation");
    assert_eq!(record.attribution_kind, "computational");
    assert!(record.attribution_json.contains("normalizer"), "{}", record.attribution_json);
    assert_eq!(record.confidence, Some(1.0));
    assert_eq!(record.verification_status, "unverified");

    // Spot-check: stored bare form equals the L3 pipeline on canonical text.
    let mut uow = db.write().await.unwrap();
    let ayah = uow.quran().get_ayah(&edition_id, 1, 1).await.unwrap().unwrap();
    let stored = uow.quran().get_ayah_form(&edition_id, 1, 1).await.unwrap().unwrap();
    uow.rollback().await.unwrap();
    let registry = ProfileRegistry::new();
    let pipeline = quran_normalization::NormalizationPipeline::for_profile(
        &registry,
        ProfileId::L3,
        SemVer::new(1, 0, 0),
    )
    .unwrap();
    assert_eq!(stored.bare, pipeline.apply(&ayah.text).0.text());
    assert_eq!(stored.rule_set_version, "1.0.0");
    assert_eq!(stored.corpus_generation, 1);
}

/// Rebuilds are idempotent: same counts, canonical hashes untouched.
#[tokio::test]
async fn rebuild_is_idempotent_and_leaves_canonical_hashes() {
    let (_dir, db) = migrated_db().await;
    let edition_id = active_edition(&db).await;
    let first = rebuild_forms(&db, &params(), &AtomicBool::new(false), |_| {}).await.unwrap();
    let hash_before = {
        let mut uow = db.write().await.unwrap();
        let edition = uow.quran().get_edition(&edition_id).await.unwrap().unwrap();
        uow.rollback().await.unwrap();
        edition.text_hash
    };
    let second = rebuild_forms(&db, &params(), &AtomicBool::new(false), |_| {}).await.unwrap();
    assert_eq!(first.token_forms, second.token_forms);
    assert_eq!(first.skeletons, second.skeletons);
    assert_eq!(first.provenance_id, second.provenance_id, "content-addressed identity converges");
    // Exactly one provenance record: the retry reused it, not duplicated it.
    let mut uow = db.write().await.unwrap();
    let records =
        uow.provenance().list_by_subject(&format!("{V1_URN}/derived-forms")).await.unwrap();
    uow.rollback().await.unwrap();
    assert_eq!(records.len(), 1);
    let hash_after = {
        let mut uow = db.write().await.unwrap();
        let edition = uow.quran().get_edition(&edition_id).await.unwrap().unwrap();
        uow.rollback().await.unwrap();
        edition.text_hash
    };
    assert_eq!(hash_before, hash_after, "MV-018: canonical text byte-identical");
}

/// MV-018 passes on a healthy edition and reports the compared hashes.
#[tokio::test]
async fn mv018_passes_with_compared_hashes() {
    let (_dir, db) = migrated_db().await;
    let edition_id = active_edition(&db).await;
    let check = verify_canonical_unchanged(&db, &edition_id).await.unwrap();
    assert!(check.unchanged);
    assert_eq!(check.edition_urn, V1_URN);
    assert_eq!(check.expected_hash, check.actual_hash);
    assert!(check.expected_hash.starts_with("sha256:"));
}

/// Cancellation before commit leaves no derived rows behind.
#[tokio::test]
async fn cancelled_rebuild_commits_nothing() {
    let (_dir, db) = migrated_db().await;
    let edition_id = active_edition(&db).await;
    let err = rebuild_forms(&db, &params(), &AtomicBool::new(true), |_| {}).await.unwrap_err();
    assert!(matches!(err, FormsError::Cancelled));
    let (_, forms) = counts(&db, &edition_id).await;
    assert_eq!(forms, 0);
}

/// Unknown editions fail typed; superseded canonical editions are refused
/// (rebuilds serve the active generation only).
#[tokio::test]
async fn unknown_and_inactive_editions_refuse() {
    let (_dir, db) = migrated_db().await;
    let missing = RebuildParams {
        edition_slug: "nope".to_string(),
        edition_version: "9.9.9".to_string(),
        invoked_by: PRINCIPAL.to_string(),
        run_tag: "t".to_string(),
    };
    let err = rebuild_forms(&db, &missing, &AtomicBool::new(false), |_| {}).await.unwrap_err();
    assert!(matches!(err, FormsError::EditionNotFound { .. }), "{err:?}");

    // A second edition version, imported and activated: 0.1.0 stays canonical
    // but is no longer active, so rebuilding it must refuse.
    active_edition(&db).await;
    let manifest_020 = BASE_MANIFEST.replace("0.1.0", "0.2.0");
    let outcome = run_import(
        &db,
        &input_manifest("run-forms-2", &manifest_020),
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .expect("0.2.0 import completes");
    assert!(matches!(outcome, ImportOutcome::Completed(_)));
    let generation =
        activate_edition(&db, "test-edition-min", "0.2.0", &principal(), "appr-2", &timestamp())
            .await
            .expect("0.2.0 activation completes");
    assert_eq!(generation, 2);
    let err = rebuild_forms(&db, &params(), &AtomicBool::new(false), |_| {}).await.unwrap_err();
    assert!(matches!(err, FormsError::NotActive { .. }), "{err:?}");

    // …while the newly active edition rebuilds at generation 2.
    let active = RebuildParams {
        edition_slug: "test-edition-min".to_string(),
        edition_version: "0.2.0".to_string(),
        invoked_by: PRINCIPAL.to_string(),
        run_tag: "t2".to_string(),
    };
    let report = rebuild_forms(&db, &active, &AtomicBool::new(false), |_| {})
        .await
        .expect("rebuilds at gen 2");
    assert_eq!(report.generation, 2);
    assert_eq!(report.ayahs, 14);
}

/// The job handler runs the same core through the worker contract.
#[tokio::test]
async fn handler_runs_through_job_contract() {
    use jobs::JobError;
    let (_dir, db) = migrated_db().await;
    active_edition(&db).await;
    let handler = FormsRebuildHandler::new(Arc::new(db));
    assert_eq!(handler.kind().to_string(), application::quran_forms::QURAN_FORMS_REBUILD_KIND);
    assert!(handler.is_idempotent());
    let record = JobRecord {
        id: "job-forms-1".to_string(),
        kind: application::quran_forms::QURAN_FORMS_REBUILD_KIND.to_string(),
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
    let outcome = handler
        .run(
            JobContext::new(record),
            serde_json::json!({
                "edition_slug": "test-edition-min",
                "edition_version": "0.1.0",
                "invoked_by": PRINCIPAL,
            }),
        )
        .await;
    match outcome {
        Ok(outcome) => assert!(outcome.success),
        Err(JobError::Storage(detail)) => panic!("handler failed: {detail}"),
        Err(other) => panic!("unexpected job error: {other:?}"),
    }
}
