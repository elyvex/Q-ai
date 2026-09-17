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
    let ImportOutcome::Completed(success) = outcome;
    assert!(success.stopped_at.is_none());
    assert_eq!(success.edition_id, "run-1");
    assert_eq!(progress.checkpoints(), ImportCheckpoint::ALL);
    assert_eq!(staged_count(&db, "run-1").await, 14);

    let mut uow = db.write().await.unwrap();
    let report = uow.quran().get_validation_report("run-1").await.unwrap().unwrap();
    assert_eq!(report.fatal_count, 0);
    assert_eq!(report.error_count, 0);
    let findings: Vec<quran_corpus::validation::Finding> =
        serde_json::from_str(&report.findings_json).unwrap();
    assert!(findings.iter().any(|finding| finding.rule_id == "QV-015"
        && finding.severity == quran_corpus::validation::Severity::Info
        && finding.message.contains("skipped")));
    assert!(uow.quran().get_active().await.unwrap().is_none(), "no canonical writes");
    assert_eq!(uow.quran().count_ayahs("run-1").await.unwrap(), 0);
    uow.rollback().await.unwrap();
}

#[tokio::test]
async fn requested_reference_cannot_be_silently_skipped() {
    let (_dir, db) = migrated_db().await;
    let mut manifest: serde_json::Value = serde_json::from_str(BASE_MANIFEST).unwrap();
    manifest["expected"]["reference_corpus_id"] = "unavailable-test-reference".into();
    let manifest = serde_json::to_string(&manifest).unwrap();
    let progress = ImportProgress::new();
    let error = run_import(
        &db,
        &input("run-reference", &manifest, None),
        &ImportOptions::default(),
        &AtomicBool::new(false),
        progress.clone(),
    )
    .await
    .expect_err("a requested but unavailable reference must fail closed");
    let quran_corpus::CorpusError::ImportFailed { step, detail } = error else {
        panic!("expected reference comparison failure, got {error}");
    };
    assert_eq!(step, "reference_comparison");
    assert!(detail.contains("QV-015"));
    assert!(!progress.checkpoints().contains(&ImportCheckpoint::ApprovalRequested));
    let mut uow = db.write().await.unwrap();
    let run = uow.quran().get_import_run("run-reference").await.unwrap().unwrap();
    assert_eq!(run.state, "Failed");
    assert!(uow.quran().get_active().await.unwrap().is_none());
    assert_eq!(uow.quran().count_ayahs("run-reference").await.unwrap(), 0);
    uow.rollback().await.unwrap();
}

fn reference_fixture() -> quran_corpus::EditionSource {
    let mut reference: quran_corpus::EditionSource = serde_json::from_str(BASE_MANIFEST).unwrap();
    reference.edition.slug = "synthetic-reference".into();
    reference
}

fn reference_text_hash(reference: &quran_corpus::EditionSource) -> String {
    let mut ayahs: Vec<_> = reference.ayahs.iter().collect();
    ayahs.sort_by_key(|ayah| (ayah.surah, ayah.ayah));
    let texts: Vec<_> = ayahs.iter().map(|ayah| ayah.text.as_str()).collect();
    quran_corpus::hashing::tagged(&quran_corpus::hashing::text_hash(
        &reference.edition.slug,
        &reference.edition.version.to_string(),
        &texts,
    ))
}

#[tokio::test]
async fn configured_reference_records_exact_comparison_and_retries_every_checkpoint() {
    let (_dir, db) = migrated_db().await;
    let mut reference = reference_fixture();
    let expected_hash = reference_text_hash(&reference);
    reference.ayahs.reverse();
    reference.surahs.reverse();
    let original = serde_json::to_string(&reference).unwrap();
    let mut manifest: serde_json::Value = serde_json::from_str(BASE_MANIFEST).unwrap();
    manifest["expected"]["reference_corpus_id"] = reference.edition.slug.clone().into();
    manifest["expected"]["reference_text_hash"] = expected_hash.clone().into();
    let manifest = serde_json::to_string(&manifest).unwrap();
    for (index, checkpoint) in ImportCheckpoint::ALL.iter().enumerate() {
        let run_id = format!("configured-{index}");
        let input = input(&run_id, &manifest, None);
        let mut options =
            ImportOptions { stop_after: Some(*checkpoint), reference: Some(reference.clone()) };
        run_import(&db, &input, &options, &AtomicBool::new(false), ImportProgress::new())
            .await
            .unwrap();
        options.stop_after = None;
        for _ in 0..2 {
            let progress = ImportProgress::new();
            run_import(&db, &input, &options, &AtomicBool::new(false), progress.clone())
                .await
                .unwrap();
            assert_eq!(progress.checkpoints(), ImportCheckpoint::ALL);
            assert_eq!(staged_count(&db, &run_id).await, 14);
        }
        assert_eq!(serde_json::to_string(options.reference.as_ref().unwrap()).unwrap(), original);
        let mut uow = db.write().await.unwrap();
        let report = uow.quran().get_validation_report(&run_id).await.unwrap().unwrap();
        assert_eq!(report.fatal_count, 0);
        let findings: Vec<quran_corpus::validation::Finding> =
            serde_json::from_str(&report.findings_json).unwrap();
        let comparison: Vec<_> = findings.iter().filter(|f| f.rule_id == "QV-015").collect();
        assert_eq!(comparison.len(), 1);
        let evidence: serde_json::Value = serde_json::from_str(&comparison[0].message).unwrap();
        assert_eq!(evidence["reference_corpus_id"], "synthetic-reference");
        assert_eq!(evidence["reference_version"], "0.1.0");
        assert_eq!(evidence["reference_text_hash"], expected_hash);
        assert_eq!(evidence["method"], "exact-ayah-bytes-v1");
        assert_eq!(evidence["outcome"], "pass");
        let snapshot_hash = quran_corpus::hashing::tagged(
            &quran_corpus::validation::intermediate_hash(&reference).unwrap(),
        );
        assert_eq!(evidence["reference_snapshot_hash"], snapshot_hash);
        let provenance = uow.provenance().get(&run_id).await.unwrap().unwrap();
        let versions: serde_json::Value = serde_json::from_str(&provenance.versions_json).unwrap();
        assert_eq!(versions["reference_snapshot_hash"], snapshot_hash);
        assert_eq!(provenance.verification_status, "unverified");
        assert_eq!(uow.quran().get_import_run(&run_id).await.unwrap().unwrap().state, "Staged");
        assert!(uow.quran().get_active().await.unwrap().is_none());
        assert_eq!(uow.quran().count_ayahs(&run_id).await.unwrap(), 0);
        uow.rollback().await.unwrap();
    }
}

#[tokio::test]
async fn configured_invalid_references_fail_closed_with_durable_findings() {
    let (_dir, db) = migrated_db().await;
    let base = reference_fixture();
    let mut cases = Vec::new();
    let mut reference = base.clone();
    reference.ayahs[0].text.push(' ');
    cases.push(("bytes", reference, "text differs byte-for-byte"));
    let mut reference = base.clone();
    reference.ayahs.remove(0);
    cases.push(("missing", reference, "ayah missing from reference"));
    let mut reference = base.clone();
    let mut extra = reference.ayahs[0].clone();
    extra.ayah = 4;
    reference.ayahs.push(extra);
    cases.push(("extra", reference, "ayah missing from imported"));
    let mut reference = base.clone();
    reference.ayahs.push(reference.ayahs[0].clone());
    cases.push(("duplicate", reference, "duplicate ayah identifier"));
    let mut reference = base.clone();
    reference.ayahs.clear();
    cases.push(("empty", reference, "empty corpus"));
    let mut reference = base.clone();
    reference.edition.script = quran_core::enums::Script::ImlaeiSimple;
    cases.push(("script", reference, "incompatible script"));
    let mut reference = base.clone();
    reference.edition.riwayah = Some("synthetic-other".into());
    cases.push(("riwayah", reference, "incompatible script"));
    let mut reference = base.clone();
    reference.edition.qiraah = Some("synthetic-other".into());
    cases.push(("qiraah", reference, "incompatible script"));
    let mut reference = base.clone();
    reference.edition.verse_numbering_scheme =
        quran_core::enums::NumberingScheme::Custom("synthetic".into());
    cases.push(("numbering", reference, "incompatible script"));
    let mut reference = base.clone();
    reference.edition.basmala_policy = quran_core::enums::BasmalaPolicy::Absent;
    cases.push(("basmala", reference, "incompatible script"));
    let mut reference = base.clone();
    reference.surahs[0].basmala = quran_core::enums::BasmalaPolicy::Absent;
    cases.push(("surah-basmala", reference, "incompatible basmala"));
    let mut reference = base.clone();
    reference.expected.ayah_count += 1;
    cases.push(("counts", reference, "QV-004"));
    let mut reference = base.clone();
    reference.format_version += 1;
    cases.push(("format", reference, "invalid reference format"));
    let mut reference = base.clone();
    reference.edition.slug.clear();
    cases.push(("identity", reference, "invalid reference format"));
    let mut reference = base.clone();
    reference.edition.language = "en".parse().unwrap();
    cases.push(("language", reference, "QV-027"));
    let mut reference = base.clone();
    reference.ayahs[0].text.push('\u{202e}');
    cases.push(("unicode", reference, "QV-008"));
    let mut reference = base.clone();
    reference.ayahs[0].text.clear();
    cases.push(("empty-text", reference, "QV-006"));
    for (name, reference, expected) in cases {
        let run_id = format!("invalid-{name}");
        let options = ImportOptions { reference: Some(reference), ..ImportOptions::default() };
        let mut previous_report = None;
        for _ in 0..2 {
            let progress = ImportProgress::new();
            let error = run_import(
                &db,
                &input(&run_id, BASE_MANIFEST, None),
                &options,
                &AtomicBool::new(false),
                progress.clone(),
            )
            .await
            .unwrap_err();
            assert!(
                matches!(
                    error,
                    quran_corpus::CorpusError::ImportFailed { step: "reference_comparison", .. }
                ),
                "{name}: {error}"
            );
            assert!(!progress.checkpoints().contains(&ImportCheckpoint::ReferenceCompared));
            assert!(!progress.checkpoints().contains(&ImportCheckpoint::ApprovalRequested));
            let mut uow = db.write().await.unwrap();
            let report = uow.quran().get_validation_report(&run_id).await.unwrap().unwrap();
            assert_eq!(report.outcome, "fail");
            assert!(report.fatal_count > 0);
            let findings: Vec<quran_corpus::validation::Finding> =
                serde_json::from_str(&report.findings_json).unwrap();
            assert!(
                findings.iter().any(|f| f.rule_id == "QV-015"
                    && f.severity == quran_corpus::validation::Severity::Fatal
                    && f.message.contains(expected)),
                "{name}: {findings:?}"
            );
            let evidence = findings
                .iter()
                .find_map(|f| serde_json::from_str::<serde_json::Value>(&f.message).ok())
                .unwrap();
            assert_eq!(evidence["outcome"], "fail");
            assert!(evidence["reference_text_hash"].as_str().unwrap().starts_with("sha256:"));
            if let Some(previous) = previous_report.replace(report.findings_json.clone()) {
                assert_eq!(previous, report.findings_json);
            }
            assert_eq!(uow.quran().get_import_run(&run_id).await.unwrap().unwrap().state, "Failed");
            assert!(uow.quran().get_active().await.unwrap().is_none());
            assert_eq!(uow.quran().count_ayahs(&run_id).await.unwrap(), 0);
            uow.rollback().await.unwrap();
        }
    }
}

#[tokio::test]
async fn reference_manifest_pins_are_enforced_individually() {
    let (_dir, db) = migrated_db().await;
    for (index, (field, value, configured)) in [
        ("reference_corpus_id", "wrong-reference".to_string(), true),
        ("reference_corpus_id", String::new(), true),
        ("reference_text_hash", format!("sha256:{}", "0".repeat(64)), true),
        ("reference_text_hash", "malformed".to_string(), true),
        ("reference_text_hash", reference_text_hash(&reference_fixture()), false),
    ]
    .into_iter()
    .enumerate()
    {
        let mut manifest: serde_json::Value = serde_json::from_str(BASE_MANIFEST).unwrap();
        manifest["expected"][field] = value.into();
        let manifest = serde_json::to_string(&manifest).unwrap();
        let run_id = format!("pins-{index}");
        let options = ImportOptions {
            reference: configured.then(reference_fixture),
            ..ImportOptions::default()
        };
        let error = run_import(
            &db,
            &input(&run_id, &manifest, None),
            &options,
            &AtomicBool::new(false),
            ImportProgress::new(),
        )
        .await
        .unwrap_err();
        assert!(matches!(
            error,
            quran_corpus::CorpusError::ImportFailed { step: "reference_comparison", .. }
        ));
        let mut uow = db.write().await.unwrap();
        let report = uow.quran().get_validation_report(&run_id).await.unwrap().unwrap();
        assert!(report.fatal_count > 0);
        assert!(!report.findings_json.contains("comparison skipped"));
        uow.rollback().await.unwrap();
    }
}

#[tokio::test]
async fn retry_cannot_replace_a_configured_reference_or_its_evidence() {
    let (_dir, db) = migrated_db().await;
    let mut options =
        ImportOptions { reference: Some(reference_fixture()), ..ImportOptions::default() };
    let input = input("pinned-retry", BASE_MANIFEST, None);
    run_import(&db, &input, &options, &AtomicBool::new(false), ImportProgress::new())
        .await
        .unwrap();
    options.reference = None;
    let error = run_import(&db, &input, &options, &AtomicBool::new(false), ImportProgress::new())
        .await
        .unwrap_err();
    assert!(matches!(error, quran_corpus::CorpusError::ImportFailed { step: "staged", .. }));
    let mut uow = db.write().await.unwrap();
    assert_eq!(uow.quran().get_import_run(&input.run_id).await.unwrap().unwrap().state, "Failed");
    let report = uow.quran().get_validation_report(&input.run_id).await.unwrap().unwrap();
    assert!(report.findings_json.contains("synthetic-reference"));
    assert!(!report.findings_json.contains("skipped"));
    assert!(uow.quran().get_active().await.unwrap().is_none());
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
            &ImportOptions { stop_after: Some(*checkpoint), ..ImportOptions::default() },
            &AtomicBool::new(false),
            progress.clone(),
        )
        .await
        .expect("prefix run halts cleanly");
        let ImportOutcome::Completed(success) = outcome;
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
        &ImportOptions { stop_after: Some(ImportCheckpoint::Staged), ..ImportOptions::default() },
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

/// AC-P1-09 (runtime half): a refused activation must change no canonical
/// state. The error alone is not the guarantee; the active pointer stays
/// absent and no canonical ayah rows appear, so there is no write path that
/// bypasses a granted approval.
async fn assert_no_canonical(db: &SqliteDatabase) {
    let mut uow = db.write().await.unwrap();
    assert!(uow.quran().get_active().await.unwrap().is_none(), "no active pointer");
    assert_eq!(uow.quran().count_ayahs("run-1").await.unwrap(), 0, "no canonical ayahs");
    uow.rollback().await.unwrap();
}

#[tokio::test]
async fn rejected_activations_leave_canonical_state_untouched() {
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
    assert_no_canonical(&db).await;

    // Missing approval.
    let err =
        activate_edition(&db, "test-edition-min", "0.1.0", &principal(), "missing", &timestamp())
            .await
            .unwrap_err();
    assert!(matches!(err, application::quran::ActivationError::ApprovalMissing { .. }));
    assert_no_canonical(&db).await;

    // Denied and mismatched approvals.
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
    assert_no_canonical(&db).await;
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
    assert_no_canonical(&db).await;

    // Positive control: a granted approval moves canonical state exactly once.
    let generation =
        activate_edition(&db, "test-edition-min", "0.1.0", &principal(), "appr-1", &timestamp())
            .await
            .unwrap();
    assert_eq!(generation, 1);
    let mut uow = db.write().await.unwrap();
    assert!(uow.quran().get_active().await.unwrap().is_some());
    assert_eq!(uow.quran().count_ayahs("run-1").await.unwrap(), 14);
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
