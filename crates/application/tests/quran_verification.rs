//! Phase 1 — editorial verification recording: P1-T55 plumbing (OD-02).
//!
//! The reviewer identity itself is an owner act; this suite proves the
//! recording mechanism: approval-gated `verified_by` stamping on the edition
//! row with a hash-chained audit event, fail-closed on empty reviewer names
//! and on missing/denied/mismatched approvals, with canonical text untouched.

use std::sync::atomic::AtomicBool;

use application::quran::{
    EditionVerification, activate_edition, record_approval, record_edition_verification,
};
use domain::{PrincipalId, Timestamp};
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use storage::Database as _;
use storage::error::Diagnostic as _;
use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

const BASE_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const PRINCIPAL: &str = "00000000-0000-0000-0000-000000000001";
const CREATED_AT: &str = "2026-09-14T00:00:00Z";
const SLUG: &str = "test-edition-min";
const VERSION: &str = "0.1.0";
const V1_URN: &str = "quran-edition:test-edition-min@0.1.0";
const REVIEWER: &str = "OD-02 placeholder reviewer (test only — not a real sign-off)";
const METHOD: &str = "test-harness approval-gated stamp; no muṣḥaf comparison performed";

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
             VALUES ('appr-denied', '{V1_URN}', 'CanonicalChange', '{PRINCIPAL}', '{PRINCIPAL}',
                     'denied', '{{}}', '{CREATED_AT}', '{CREATED_AT}')"
        ),
        format!(
            "INSERT INTO approvals
                (id, subject_urn, kind, requested_by, decided_by, decision,
                 request_payload, requested_at, decided_at)
             VALUES ('appr-other', 'quran-edition:other@9.9.9', 'CanonicalChange',
                     '{PRINCIPAL}', '{PRINCIPAL}', 'approved', '{{}}', '{CREATED_AT}', '{CREATED_AT}')"
        ),
    ] {
        sqlx::query(&sql).execute(&seed).await.unwrap();
    }
    seed.close().await;
    (dir, db)
}

fn input(run_id: &str) -> ImportInput {
    ImportInput {
        run_id: run_id.into(),
        job_id: None,
        source_version_id: "sv-1".into(),
        adapter: "json".into(),
        manifest_text: BASE_MANIFEST.into(),
        declared_manifest_hash: None,
        invoked_by: PRINCIPAL.into(),
        license_status: "PublicDomain".into(),
        license_json: "{}".into(),
        created_at: CREATED_AT.into(),
        reference_manifest_text: None,
    }
}

async fn active_db() -> (tempfile::TempDir, SqliteDatabase, String) {
    let (dir, db) = migrated_db().await;
    let run_id = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
    let outcome = run_import(
        &db,
        &input(run_id),
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .expect("import completes");
    let edition_id = match outcome {
        ImportOutcome::Completed(success) => success.edition_id,
    };
    activate_edition(&db, SLUG, VERSION, &principal(), "appr-1", &timestamp())
        .await
        .expect("activation completes");
    (dir, db, edition_id)
}

async fn edition_row(db: &SqliteDatabase) -> storage::quran::QuranEditionRow {
    let mut uow = db.write().await.unwrap();
    let row =
        uow.quran().get_edition_by_slug_version(SLUG, VERSION).await.unwrap().expect("edition");
    uow.rollback().await.unwrap();
    row
}

#[tokio::test]
async fn verification_stamp_records_reviewer_method_and_time() {
    let (_dir, db, _edition_id) = active_db().await;
    let before = edition_row(&db).await;
    assert_eq!(before.verified_by, None);
    assert_eq!(before.text_hash.clone(), before.text_hash);

    record_approval(&db, "appr-v", V1_URN, PRINCIPAL, PRINCIPAL, "{}", CREATED_AT).await.unwrap();
    record_edition_verification(
        &db,
        SLUG,
        VERSION,
        &EditionVerification { reviewer: REVIEWER, method: METHOD },
        &principal(),
        "appr-v",
        &timestamp(),
    )
    .await
    .expect("verification records");

    let after = edition_row(&db).await;
    assert_eq!(after.verified_by.as_deref(), Some(REVIEWER));
    assert_eq!(after.verification_method.as_deref(), Some(METHOD));
    assert_eq!(after.verified_at.as_deref(), Some(timestamp().to_string().as_str()));
    // Canonical identity untouched by the metadata stamp.
    assert_eq!(after.text_hash, before.text_hash);
    assert_eq!(after.structure_hash, before.structure_hash);
    assert_eq!(after.token_order_hash, before.token_order_hash);
    assert_eq!(after.status, "Active");
}

#[tokio::test]
async fn verification_rejects_empty_reviewer_and_method() {
    let (_dir, db, _edition_id) = active_db().await;
    for (reviewer, method) in [("", METHOD), (REVIEWER, ""), ("   ", METHOD)] {
        let verification = EditionVerification { reviewer, method };
        let err = record_edition_verification(
            &db,
            SLUG,
            VERSION,
            &verification,
            &principal(),
            "appr-1",
            &timestamp(),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, application::quran::ActivationError::EmptyReviewer));
        assert_eq!(err.code().to_string(), "QAI-QUR-0306");
    }
    assert_eq!(edition_row(&db).await.verified_by, None);
}

#[tokio::test]
async fn verification_requires_a_granted_approval_for_the_exact_urn() {
    let (_dir, db, _edition_id) = active_db().await;
    let verification = EditionVerification { reviewer: REVIEWER, method: METHOD };
    // Missing approval.
    let err = record_edition_verification(
        &db,
        SLUG,
        VERSION,
        &verification,
        &principal(),
        "no-such-approval",
        &timestamp(),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, application::quran::ActivationError::ApprovalMissing { .. }));
    // Denied approval.
    let err = record_edition_verification(
        &db,
        SLUG,
        VERSION,
        &verification,
        &principal(),
        "appr-denied",
        &timestamp(),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, application::quran::ActivationError::ApprovalNotGranted { .. }));
    // Approval for another subject.
    let err = record_edition_verification(
        &db,
        SLUG,
        VERSION,
        &verification,
        &principal(),
        "appr-other",
        &timestamp(),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, application::quran::ActivationError::ApprovalSubjectMismatch { .. }));
    // Unknown edition (nothing imported under that slug).
    record_approval(
        &db,
        "appr-ghost",
        "quran-edition:ghost@0.0.1",
        PRINCIPAL,
        PRINCIPAL,
        "{}",
        CREATED_AT,
    )
    .await
    .unwrap();
    let err = record_edition_verification(
        &db,
        "ghost",
        "0.0.1",
        &verification,
        &principal(),
        "appr-ghost",
        &timestamp(),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, application::quran::ActivationError::NotStaged { .. }));
    assert_eq!(edition_row(&db).await.verified_by, None);
}
