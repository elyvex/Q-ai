//! Phase 2 (plan 02-03) — operator reference-corpus path (QV-015, ADR-0114).
//!
//! Exercises the D-09 operator configuration path: an independent reference
//! manifest travels on the import payload (`ImportInput.reference_manifest_text`)
//! and is compared byte-exact, fail-closed. A matching reference yields a
//! persisted QV-015 `outcome: "pass"` with one ADR-0114 `DifferenceClass` per
//! difference; a mutated reference fails the run with the stage untouched and
//! no active edition.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use application::quran::activate_edition;
use application::quran_doctor::{CheckLevel, run_quran_checks};
use domain::{PrincipalId, Timestamp};
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use quran_corpus::sha256_hex;
use quran_corpus::validation::{Finding, Severity};
use storage::Database as _;
use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

const RICH_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-rich/manifest.json");
const RICH_REFERENCE: &str =
    include_str!("../../../fixtures/quran/test-edition-rich/reference.json");
const SLUG: &str = "test-edition-rich";
const VERSION: &str = "0.1.0";
const PRINCIPAL: &str = "00000000-0000-0000-0000-000000000001";
const CREATED_AT: &str = "2026-09-14T00:00:00Z";
const SOURCE_VERSION_ID: &str = "12345678-1234-1234-1234-123456789abc";
const URN: &str = "quran-edition:test-edition-rich@0.1.0";
const LICENSE_JSON: &str = "{\"status\":\"Unknown\",\"spdx_id\":null,\"name\":null,\
                              \"url\":null,\"attribution_required\":false,\
                              \"redistribution_allowed\":false,\"export_allowed\":false,\
                              \"notes\":null}";

fn principal() -> PrincipalId {
    PRINCIPAL.parse().unwrap()
}

fn timestamp() -> Timestamp {
    Timestamp::from_ymd_hms(2026, 9, 14, 0, 0, 0).unwrap()
}

/// The rich fixture imported with an operator-supplied reference on the payload.
fn input(run_id: &str, reference_manifest_text: Option<String>) -> ImportInput {
    ImportInput {
        run_id: run_id.into(),
        job_id: None,
        source_version_id: SOURCE_VERSION_ID.into(),
        adapter: "json".into(),
        manifest_text: RICH_MANIFEST.into(),
        declared_manifest_hash: Some(sha256_hex(RICH_MANIFEST.as_bytes())),
        invoked_by: PRINCIPAL.into(),
        license_status: "Unknown".into(),
        license_json: LICENSE_JSON.into(),
        created_at: CREATED_AT.into(),
        reference_manifest_text,
    }
}

/// Migrated, seeded temp database (FK parents + the approval for the rich URN).
async fn migrated_db() -> (tempfile::TempDir, Arc<SqliteDatabase>) {
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
             VALUES ('{SOURCE_VERSION_ID}', 'src-1', '0.1.0', 1, 'Staged',
                     'ImportedUnverified', 'Unknown', '{{}}', '{CREATED_AT}')"
        ),
        format!(
            "INSERT INTO approvals
                (id, subject_urn, kind, requested_by, decided_by, decision,
                 request_payload, requested_at, decided_at)
             VALUES ('appr-1', '{URN}', 'CanonicalChange', '{PRINCIPAL}', '{PRINCIPAL}',
                     'approved', '{{}}', '{CREATED_AT}', '{CREATED_AT}')"
        ),
    ] {
        sqlx::query(&sql).execute(&seed).await.unwrap();
    }
    seed.close().await;
    (dir, db)
}

fn qv15_findings(findings_json: &str) -> Vec<Finding> {
    let findings: Vec<Finding> = serde_json::from_str(findings_json).unwrap();
    findings.into_iter().filter(|finding| finding.rule_id == "QV-015").collect()
}

#[tokio::test]
async fn operator_reference_path_is_evaluated_and_mutation_fails_closed() {
    let (_dir, db) = migrated_db().await;

    // 1) A deliberately mutated reference fails closed: the stage is left
    //    untouched, no edition is active, and the durable report records both
    //    the byte-only Fatal and one typed per-difference classification.
    let mut mutated: quran_corpus::EditionSource = serde_json::from_str(RICH_REFERENCE).unwrap();
    mutated.ayahs[0].text.push('ب');
    let mutated_text = serde_json::to_string(&mutated).unwrap();
    let error = run_import(
        &*db,
        &input("ref-mutated", Some(mutated_text)),
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .expect_err("a mutated reference must fail closed");
    assert!(
        matches!(error, quran_corpus::CorpusError::ImportFailed { .. }),
        "expected an import failure, got {error}"
    );
    {
        let mut uow = db.write().await.unwrap();
        assert_eq!(
            uow.quran().count_staging_orphans().await.unwrap(),
            0,
            "a failed reference comparison must leave the stage untouched"
        );
        assert!(uow.quran().get_active().await.unwrap().is_none(), "no edition activated");
        let report = uow.quran().get_validation_report("ref-mutated").await.unwrap().unwrap();
        let qv15 = qv15_findings(&report.findings_json);
        assert!(
            qv15.iter().any(|finding| finding.severity == Severity::Fatal),
            "byte difference must stay a Fatal QV-015"
        );
        let comparison = qv15
            .iter()
            .find(|finding| {
                finding.severity == Severity::Info
                    && finding.message.contains("\"comparison_kind\"")
            })
            .expect("typed comparison finding");
        let evidence: serde_json::Value = serde_json::from_str(&comparison.message).unwrap();
        assert_eq!(evidence["outcome"], "fail", "classification never softens the byte verdict");
        assert_eq!(evidence["classification_vocabulary"], "ADR-0114-v1", "no new vocabulary");
        let differences = evidence["differences"].as_array().expect("per-difference metadata");
        assert_eq!(differences.len(), 1, "exactly one difference classified");
        assert_eq!(differences[0]["surah"], 1);
        assert_eq!(differences[0]["ayah"], 1);
        assert_eq!(differences[0]["class"], "unknown_difference");
        uow.rollback().await.unwrap();
    }

    // 2) The matching companion reference is evaluated end-to-end: the
    //    persisted QV-015 finding carries `outcome: "pass"`, not the skip.
    let outcome = run_import(
        &*db,
        &input("ref-matching", Some(RICH_REFERENCE.to_string())),
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .expect("a matching reference import completes");
    assert!(matches!(outcome, ImportOutcome::Completed(_)));
    {
        let mut uow = db.write().await.unwrap();
        let report = uow.quran().get_validation_report("ref-matching").await.unwrap().unwrap();
        let qv15 = qv15_findings(&report.findings_json);
        assert_eq!(qv15.len(), 1, "exactly one QV-015 comparison finding");
        assert!(!qv15[0].message.contains("skipped"), "the skip must not apply");
        let evidence: serde_json::Value = serde_json::from_str(&qv15[0].message).unwrap();
        assert_eq!(evidence["outcome"], "pass");
        assert_eq!(evidence["reference_corpus_id"], "synthetic-rich-reference");
        assert_eq!(evidence["classification_vocabulary"], "ADR-0114-v1");
        assert_eq!(
            evidence["differences"].as_array().unwrap().len(),
            0,
            "a byte-identical reference has no differences"
        );
        uow.rollback().await.unwrap();
    }

    // 3) After activation the doctor/verify reference family is `pass` — the
    //    recorded skip has been replaced by a real comparison.
    activate_edition(&*db, SLUG, VERSION, &principal(), "appr-1", &timestamp())
        .await
        .expect("activation completes");
    let checks = run_quran_checks(&*db, false).await.unwrap();
    let reference = checks.iter().find(|check| check.id == "quran.reference_corpus").unwrap();
    assert_eq!(reference.status, CheckLevel::Pass, "{}", reference.summary);
}
