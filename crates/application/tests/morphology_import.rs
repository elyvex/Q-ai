//! Phase 2 — morphology import/activate acceptance (P2-T66/T67/T71/T72/T75).
//!
//! Against real SQLite with a real active edition: aligned synthetic imports
//! reach `Staged` with zero fatals; adversarial documents fail with specific
//! MV ids; activation is approval-gated and atomic; cancellation at every
//! checkpoint preserves prior state; read tools are typed-unavailable until
//! activation and attributed after.

use std::sync::atomic::AtomicBool;

use application::quran::activate_edition;
use application::quran_forms::{RebuildParams, rebuild_forms};
use application::quran_index::{IndexBuildParams, QURAN_AYAH_INDEX_ID, rebuild_index};
use application::quran_morphology::{
    IMPORT_CHECKPOINTS, MorphologyActivateParams, MorphologyDiffKind, MorphologyImportParams,
    activate_morphology, affix_search, browse_lemmas, browse_roots, build_same_root_relations,
    dataset_urn, diff_datasets, lemma_search, morphology_compare, morphology_for_token,
    pattern_search, root_search, run_morphology_import, word_family,
};
use domain::{PrincipalId, Timestamp};
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use quran_corpus::sha256_hex;
use quran_search::{Fts5Index, FtsQuery, FullTextIndex, TokenizerFamily};
use storage::Database as _;
use storage::repository::ApprovalRow;
use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

const BASE_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const PRINCIPAL: &str = "00000000-0000-0000-0000-000000000001";
const CREATED_AT: &str = "2026-09-14T00:00:00Z";
const V1_URN: &str = "quran-edition:test-edition-min@0.1.0";
const SLUG: &str = "test-morph";
const VERSION: &str = "0.1.0";

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
             VALUES ('appr-morph', '{}', 'CanonicalChange', '{PRINCIPAL}', '{PRINCIPAL}',
                     'approved', '{{}}', '{CREATED_AT}', '{CREATED_AT}')",
            dataset_urn(SLUG, VERSION)
        ),
    ] {
        sqlx::query(&sql).execute(&seed).await.unwrap();
    }
    seed.close().await;
    (dir, db)
}

async fn ready_db() -> (tempfile::TempDir, SqliteDatabase) {
    let (dir, db) = migrated_db().await;
    let outcome = run_import(
        &db,
        &ImportInput {
            run_id: "run-morph-1".into(),
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
            run_tag: "morph-test".to_string(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("forms rebuild completes");
    (dir, db)
}

/// Build an aligned array-shape document from real fixture tokens (all
/// surfaces match the inventory → direct-key alignment).
async fn aligned_document(db: &SqliteDatabase) -> String {
    let mut uow = db.write().await.unwrap();
    let active = uow.quran().get_active().await.unwrap().unwrap();
    let ayahs = uow.quran().list_ayahs_range(&active.edition_id, 1, i64::MAX).await.unwrap();
    let mut rows = Vec::new();
    for ayah in &ayahs {
        let tokens =
            uow.quran().get_tokens(&active.edition_id, ayah.surah, ayah.ayah).await.unwrap();
        for token in &tokens {
            // Two competing analyses of every token (multi-analysis reads).
            for analysis_no in [0u32, 1u32] {
                rows.push(serde_json::json!({
                    "sura_no": ayah.surah,
                    "aya_no": ayah.ayah,
                    "tok_idx": token.position,
                    "analysis_no": analysis_no,
                    "surface_form": token.surface,
                    "lemma_str": format!("lem-{}", token.surface),
                    "root_str": "tst-root",
                    "stem_str": token.surface,
                    "tag_native": if analysis_no == 0 { "N" } else { "V" },
                    "tag_unified": if analysis_no == 0 { "noun" } else { "verb" },
                    "layer": "B",
                    "state": "imported",
                    "pattern": "synthetic-pattern",
                    "synthetic_test_only": true,
                }));
            }
        }
    }
    uow.rollback().await.unwrap();
    serde_json::to_string(&rows).unwrap()
}

fn import_params(document_text: String, batch: &str) -> MorphologyImportParams {
    MorphologyImportParams {
        dataset_slug: SLUG.to_string(),
        dataset_version: VERSION.to_string(),
        adapter: "json".to_string(),
        document_text,
        edition_slug: "test-edition-min".to_string(),
        edition_version: "0.1.0".to_string(),
        invoked_by: PRINCIPAL.to_string(),
        batch_id: Some(batch.to_string()),
        attribution: "synthetic test import (not scholarly data)".to_string(),
        license_status: "PublicDomain".to_string(),
        license_json: "{}".to_string(),
    }
}

/// P2-T66: aligned import reaches `Staged` with zero fatals + MV-018 clean.
#[tokio::test]
async fn import_aligned_document_stages() {
    let (_dir, db) = ready_db().await;
    let doc = aligned_document(&db).await;
    let report =
        run_morphology_import(&db, &import_params(doc, "batch-1"), &AtomicBool::new(false), |_| {})
            .await
            .expect("import completes");
    assert_eq!(report.state, "staged");
    assert!(report.matched > 0);
    assert!(report.unmatched_by_surah.is_empty());
    assert_eq!(report.findings.get("fatal").copied().unwrap_or(0), 0);
    assert!(report.mv018_unchanged);

    let mut uow = db.write().await.unwrap();
    let batch = uow.quran().get_staging_batch("batch-1").await.unwrap().unwrap();
    uow.rollback().await.unwrap();
    assert_eq!(batch.state, "staged");
    assert_eq!(batch.checkpoint, "finalize");
}

/// P2-T71 (application scope): adversarial documents are flagged with
/// specific MV ids. MV-002 is severity Error: the batch still stages (zero
/// fatals) but activation refuses with BlockingFindings.
#[tokio::test]
async fn adversarial_documents_reject_with_mv_ids() {
    let (_dir, db) = ready_db().await;
    // MV-002: empty surface.
    let doc = serde_json::json!([{
        "sura_no": 1, "aya_no": 1, "tok_idx": 1, "analysis_no": 0,
        "surface_form": "", "lemma_str": "x", "root_str": "y",
        "tag_native": "N", "tag_unified": "noun",
        "layer": "B", "state": "imported", "synthetic_test_only": true,
    }])
    .to_string();
    let report = run_morphology_import(
        &db,
        &import_params(doc, "batch-mv002"),
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("import runs to a verdict");
    assert_eq!(report.state, "staged", "Error (not Fatal) still stages");
    let mut uow = db.write().await.unwrap();
    let findings = uow.quran().list_findings("batch-mv002").await.unwrap();
    uow.rollback().await.unwrap();
    assert!(findings.iter().any(|f| f.rule_id == "MV-002" && f.severity == "error"));

    // Activation refuses on error findings (has_blocking).
    let err = activate_morphology(
        &db,
        &MorphologyActivateParams {
            batch_id: "batch-mv002".to_string(),
            approval_id: "appr-morph".to_string(),
            invoked_by: PRINCIPAL.to_string(),
        },
        &principal(),
    )
    .await
    .unwrap_err();
    assert!(matches!(
        err,
        application::quran_morphology::MorphologyJobError::BlockingFindings(_, _)
    ));

    // MV-003 (Fatal): surah out of range fails the batch outright.
    let doc = serde_json::json!([{
        "sura_no": 999, "aya_no": 1, "tok_idx": 1, "analysis_no": 0,
        "surface_form": "x", "lemma_str": "x", "root_str": "y",
        "tag_native": "N", "tag_unified": "noun",
        "layer": "B", "state": "imported", "synthetic_test_only": true,
    }])
    .to_string();
    let report = run_morphology_import(
        &db,
        &import_params(doc, "batch-mv003"),
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("import runs to a verdict");
    assert_eq!(report.state, "failed");
    let mut uow = db.write().await.unwrap();
    let findings = uow.quran().list_findings("batch-mv003").await.unwrap();
    uow.rollback().await.unwrap();
    assert!(findings.iter().any(|f| f.rule_id == "MV-003" && f.severity == "fatal"));

    // Unknown adapter fails typed before any staging.
    let mut params = import_params("[]".to_string(), "batch-nope");
    params.adapter = "xml".to_string();
    let err =
        run_morphology_import(&db, &params, &AtomicBool::new(false), |_| {}).await.unwrap_err();
    assert!(matches!(err, application::quran_morphology::MorphologyJobError::Morphology(_)));
}

/// P2-T67: activation is approval-gated, alignment-gated, and atomic.
#[tokio::test]
async fn activation_gates_and_promotes() {
    use application::quran_morphology::MorphologyJobError;
    let (_dir, db) = ready_db().await;
    let doc = aligned_document(&db).await;
    run_morphology_import(&db, &import_params(doc, "batch-act"), &AtomicBool::new(false), |_| {})
        .await
        .unwrap();

    // Missing approval → refused.
    let err = activate_morphology(
        &db,
        &MorphologyActivateParams {
            batch_id: "batch-act".to_string(),
            approval_id: "nope".to_string(),
            invoked_by: PRINCIPAL.to_string(),
        },
        &principal(),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, MorphologyJobError::Approval(_)));

    // Wrong subject (edition approval, not dataset) → refused.
    let err = activate_morphology(
        &db,
        &MorphologyActivateParams {
            batch_id: "batch-act".to_string(),
            approval_id: "appr-1".to_string(),
            invoked_by: PRINCIPAL.to_string(),
        },
        &principal(),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, MorphologyJobError::Approval(_)));

    // Correct approval → active with promoted rows.
    let report = activate_morphology(
        &db,
        &MorphologyActivateParams {
            batch_id: "batch-act".to_string(),
            approval_id: "appr-morph".to_string(),
            invoked_by: PRINCIPAL.to_string(),
        },
        &principal(),
    )
    .await
    .expect("activation completes");
    assert_eq!(report.dataset, format!("{SLUG}@{VERSION}"));
    assert!(report.previous_dataset.is_none());
    let (roots, lemmas, analyses, morphemes) = report.promoted;
    assert!(roots > 0 && lemmas > 0 && analyses > 0);

    // Read tools now serve attributed analyses (two per token).
    let (dataset, list) =
        morphology_for_token(&db, "test-edition-min", "0.1.0", 1, 1, 1).await.unwrap();
    assert_eq!(dataset, format!("{SLUG}@{VERSION}"));
    assert_eq!(list.len(), 2, "competing analyses coexist");
    assert!(list.iter().all(|a| a.provenance_layer == "B"));

    // Root search groups occurrences with attribution.
    let (_, occ) = root_search(&db, "tst-root").await.unwrap();
    assert!(!occ.is_empty());
    assert!(occ.iter().all(|o| o.dataset == format!("{SLUG}@{VERSION}")));
    let _ = (morphemes, lemmas);
}

/// P2-T75/T77: compare verdicts without resolution; tools typed-unavailable
/// before activation.
#[tokio::test]
async fn compare_and_unavailable_tools() {
    use application::quran_morphology::MorphologyToolError;
    let (_dir, db) = ready_db().await;
    // Before any dataset: every dataset-backed tool names the capability.
    for err in [
        morphology_for_token(&db, "test-edition-min", "0.1.0", 1, 1, 1).await.unwrap_err(),
        root_search(&db, "tst-root").await.unwrap_err(),
        lemma_search(&db, "lem-x").await.unwrap_err(),
    ] {
        assert!(matches!(err, MorphologyToolError::UnavailableDataset { .. }));
        use storage::error::Diagnostic as _;
        assert_eq!(err.code().to_string(), "QAI-MORPH-0004");
    }

    // After activation: compare returns verdicts with no resolution field.
    let doc = aligned_document(&db).await;
    run_morphology_import(&db, &import_params(doc, "batch-cmp"), &AtomicBool::new(false), |_| {})
        .await
        .unwrap();
    activate_morphology(
        &db,
        &MorphologyActivateParams {
            batch_id: "batch-cmp".to_string(),
            approval_id: "appr-morph".to_string(),
            invoked_by: PRINCIPAL.to_string(),
        },
        &principal(),
    )
    .await
    .unwrap();
    let verdicts = morphology_compare(&db, "test-edition-min", "0.1.0", 1, 1, 1).await.unwrap();
    assert!(!verdicts.is_empty(), "two analyses must produce verdicts");
    let json = serde_json::to_string(&verdicts).unwrap();
    assert!(
        !json.contains("resolution") && !json.contains("winner") && !json.contains("synthesis")
    );

    // L7 affix backend answers with its mandatory label (no dataset needed
    // for the heuristic path — here a dataset IS active, so empty scan).
    let hits = affix_search(&db, "ويت", "L7.affix").await.unwrap();
    assert!(hits.iter().all(|h| h.backend == "Heuristic (pattern-based)"));
}

/// P2-T72/T66: cancel at every checkpoint preserves state; retry succeeds.
#[tokio::test]
async fn crash_matrix_cancel_at_each_checkpoint() {
    assert_eq!(IMPORT_CHECKPOINTS.len(), 12);
    for checkpoint in IMPORT_CHECKPOINTS {
        let (_dir, db) = ready_db().await;
        let doc = aligned_document(&db).await;
        let cancel = AtomicBool::new(false);
        let seen = std::sync::Mutex::new(false);
        let err = run_morphology_import(&db, &import_params(doc, "batch-kill"), &cancel, |stage| {
            if stage == checkpoint {
                *seen.lock().unwrap() = true;
                cancel.store(true, std::sync::atomic::Ordering::SeqCst);
            }
        })
        .await
        .unwrap_err();
        assert!(*seen.lock().unwrap(), "checkpoint {checkpoint} must be reached");
        assert!(
            matches!(err, application::quran_morphology::MorphologyJobError::Cancelled),
            "checkpoint {checkpoint}: {err:?}"
        );
        // No dataset activated by a killed run.
        let mut uow = db.write().await.unwrap();
        let active = uow.quran().active_dataset().await.unwrap();
        uow.rollback().await.unwrap();
        assert!(active.is_none(), "checkpoint {checkpoint}: no dataset may activate");

        // Retry without cancellation stages cleanly.
        let doc = aligned_document(&db).await;
        let report = run_morphology_import(
            &db,
            &import_params(doc, "batch-retry"),
            &AtomicBool::new(false),
            |_| {},
        )
        .await
        .expect("retry succeeds");
        assert_eq!(report.state, "staged");
    }
}

/// P2-T83…T88: family relations build with mandatory explanations and serve.
#[tokio::test]
async fn family_relations_explained() {
    let (_dir, db) = ready_db().await;
    let doc = aligned_document(&db).await;
    run_morphology_import(&db, &import_params(doc, "batch-fam"), &AtomicBool::new(false), |_| {})
        .await
        .unwrap();
    activate_morphology(
        &db,
        &MorphologyActivateParams {
            batch_id: "batch-fam".to_string(),
            approval_id: "appr-morph".to_string(),
            invoked_by: PRINCIPAL.to_string(),
        },
        &principal(),
    )
    .await
    .unwrap();
    let built =
        build_same_root_relations(&db, &format!("{SLUG}@{VERSION}")).await.expect("builders run");
    assert!(built > 0);
    let (_, members) = word_family(&db, "token", "token:1:1:1").await.unwrap();
    // Member 1:1:1 pairs with its window sibling under the shared test root.
    assert!(members.iter().all(|m| !m.explanation.is_empty()));
    assert!(members.iter().all(|m| m.relation == "same_root"));
}

/// P2-T69: compare two registered dataset versions without merging analyses.
#[tokio::test]
async fn morphology_dataset_version_diff_preserves_analysis_identity() {
    let (_dir, db) = ready_db().await;
    let first_doc = aligned_document(&db).await;
    run_morphology_import(
        &db,
        &import_params(first_doc.clone(), "batch-diff-v1"),
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .unwrap();
    activate_morphology(
        &db,
        &MorphologyActivateParams {
            batch_id: "batch-diff-v1".to_string(),
            approval_id: "appr-morph".to_string(),
            invoked_by: PRINCIPAL.to_string(),
        },
        &principal(),
    )
    .await
    .unwrap();

    let second_slug = "test-morph-v2";
    let second_version = "0.2.0";
    let second_urn = dataset_urn(second_slug, second_version);
    let mut uow = db.write().await.unwrap();
    uow.sources()
        .insert_approval(ApprovalRow {
            id: "appr-morph-v2".to_string(),
            subject_urn: second_urn,
            kind: "CanonicalChange".to_string(),
            requested_by: Some(PRINCIPAL.to_string()),
            decided_by: Some(PRINCIPAL.to_string()),
            decision: Some("approved".to_string()),
            request_payload: "{}".to_string(),
            decision_note: None,
            requested_at: CREATED_AT.to_string(),
            decided_at: Some(CREATED_AT.to_string()),
        })
        .await
        .unwrap();
    uow.commit().await.unwrap();

    let mut second_rows: Vec<serde_json::Value> = serde_json::from_str(&first_doc).unwrap();
    second_rows[0]["lemma_str"] = serde_json::json!("changed-lemma");
    let second_doc = serde_json::to_string(&second_rows).unwrap();
    let mut second_params = import_params(second_doc, "batch-diff-v2");
    second_params.dataset_slug = second_slug.to_string();
    second_params.dataset_version = second_version.to_string();
    run_morphology_import(&db, &second_params, &AtomicBool::new(false), |_| {}).await.unwrap();
    activate_morphology(
        &db,
        &MorphologyActivateParams {
            batch_id: "batch-diff-v2".to_string(),
            approval_id: "appr-morph-v2".to_string(),
            invoked_by: PRINCIPAL.to_string(),
        },
        &principal(),
    )
    .await
    .unwrap();

    let report = diff_datasets(&db, SLUG, VERSION, second_slug, second_version).await.unwrap();
    assert_eq!(report.from_dataset, format!("{SLUG}@{VERSION}"));
    assert_eq!(report.to_dataset, format!("{second_slug}@{second_version}"));
    assert_eq!(report.added, 0);
    assert_eq!(report.removed, 0);
    assert_eq!(report.changed, 1);
    assert!(report.unchanged > 0);
    let change = report.changes.iter().find(|c| c.kind == MorphologyDiffKind::Changed).unwrap();
    assert!(change.old.is_some() && change.new.is_some());
    assert!(change.verdicts.iter().any(|v| v.field == "lemma"));
    let json = serde_json::to_string(&report).unwrap();
    assert!(!json.contains("winner"));
    assert!(!json.contains("resolution"));

    let missing = diff_datasets(&db, SLUG, VERSION, "missing", "9.9.9").await.unwrap_err();
    assert!(missing.to_string().contains("unknown dataset"));
}

/// P2-T73: an active morphology dataset is projected into all five FTS
/// lexicon columns, while the index remains edition-relative and deterministic.
#[tokio::test]
async fn morphology_lexicon_fields_reach_serving_index() {
    let (dir, db) = ready_db().await;
    let doc = aligned_document(&db).await;
    run_morphology_import(
        &db,
        &import_params(doc, "batch-fts-lexicon"),
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .unwrap();
    activate_morphology(
        &db,
        &MorphologyActivateParams {
            batch_id: "batch-fts-lexicon".to_string(),
            approval_id: "appr-morph".to_string(),
            invoked_by: PRINCIPAL.to_string(),
        },
        &principal(),
    )
    .await
    .unwrap();

    let (stem_value, lemma_value) = {
        let mut uow = db.write().await.unwrap();
        let active = uow.quran().get_active().await.unwrap().unwrap();
        let ayahs = uow.quran().list_ayahs_range(&active.edition_id, 1, 1).await.unwrap();
        let token = uow
            .quran()
            .get_tokens(&active.edition_id, ayahs[0].surah, ayahs[0].ayah)
            .await
            .unwrap()[0]
            .surface
            .clone();
        uow.rollback().await.unwrap();
        (token.clone(), format!("lem-{token}"))
    };

    let data_dir = dir.path().join("index");
    let report = rebuild_index(
        &db,
        &IndexBuildParams {
            index_id: QURAN_AYAH_INDEX_ID.to_string(),
            edition_slug: "test-edition-min".to_string(),
            edition_version: "0.1.0".to_string(),
            invoked_by: PRINCIPAL.to_string(),
            run_tag: "fts-lexicon".to_string(),
            data_dir: data_dir.clone(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .unwrap();
    assert_eq!(report.doc_count, 14);

    let mut uow = db.write().await.unwrap();
    let pointer = uow.quran().get_index_pointer(QURAN_AYAH_INDEX_ID).await.unwrap().unwrap();
    uow.rollback().await.unwrap();
    let manifest: quran_search::IndexManifest =
        serde_json::from_str(&pointer.manifest_json).unwrap();
    assert!(manifest.morphology_dataset_versions.contains_key(SLUG));

    let family = TokenizerFamily::new(
        &application::quran_normalize::builtin_registry(),
        manifest.tokenizer_version,
    )
    .unwrap();
    let index =
        Fts5Index::open(&data_dir, pointer.generation as u64, manifest, family).await.unwrap();
    for (field, value) in [
        ("roots", "tst-root".to_string()),
        ("lemmas", lemma_value),
        ("stems", stem_value),
        ("pos_tags", "noun".to_string()),
        ("patterns", "synthetic-pattern".to_string()),
    ] {
        let found = index
            .search(
                &FtsQuery::Term { field: field.to_string(), term: value.clone() },
                &quran_search::SearchOpts::default(),
            )
            .await
            .unwrap();
        assert!(found.total_matches > 0, "field {field} value {value}");
    }
}

/// P2-T80: exact pattern search distinguishes a supported dataset, a zero-match
/// query, and an active dataset that supplies no pattern metadata.
#[tokio::test]
async fn pattern_search_exact_and_typed_unavailable() {
    let (_dir, db) = ready_db().await;
    let patterned_doc = aligned_document(&db).await;
    run_morphology_import(
        &db,
        &import_params(patterned_doc.clone(), "batch-pattern"),
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .unwrap();
    activate_morphology(
        &db,
        &MorphologyActivateParams {
            batch_id: "batch-pattern".to_string(),
            approval_id: "appr-morph".to_string(),
            invoked_by: PRINCIPAL.to_string(),
        },
        &principal(),
    )
    .await
    .unwrap();

    let matched = pattern_search(&db, "synthetic-pattern").await.unwrap();
    assert_eq!(matched.dataset, format!("{SLUG}@{VERSION}"));
    assert!(!matched.occurrences.is_empty());
    assert!(matched.occurrences.iter().all(|hit| {
        hit.matched_label == "synthetic-pattern"
            && hit.matched_field == "pattern"
            && !hit.surface.is_empty()
    }));
    let unmatched = pattern_search(&db, "not-present").await.unwrap();
    assert!(unmatched.occurrences.is_empty());

    let mut rows: Vec<serde_json::Value> = serde_json::from_str(&patterned_doc).unwrap();
    for row in &mut rows {
        row.as_object_mut().unwrap().remove("pattern");
    }
    let no_pattern_doc = serde_json::to_string(&rows).unwrap();
    let second_slug = "test-morph-no-pattern";
    let second_version = "0.1.0";
    let mut uow = db.write().await.unwrap();
    uow.sources()
        .insert_approval(ApprovalRow {
            id: "appr-morph-no-pattern".to_string(),
            subject_urn: dataset_urn(second_slug, second_version),
            kind: "CanonicalChange".to_string(),
            requested_by: Some(PRINCIPAL.to_string()),
            decided_by: Some(PRINCIPAL.to_string()),
            decision: Some("approved".to_string()),
            request_payload: "{}".to_string(),
            decision_note: None,
            requested_at: CREATED_AT.to_string(),
            decided_at: Some(CREATED_AT.to_string()),
        })
        .await
        .unwrap();
    uow.commit().await.unwrap();
    let mut params = import_params(no_pattern_doc, "batch-no-pattern");
    params.dataset_slug = second_slug.to_string();
    params.dataset_version = second_version.to_string();
    run_morphology_import(&db, &params, &AtomicBool::new(false), |_| {}).await.unwrap();
    activate_morphology(
        &db,
        &MorphologyActivateParams {
            batch_id: "batch-no-pattern".to_string(),
            approval_id: "appr-morph-no-pattern".to_string(),
            invoked_by: PRINCIPAL.to_string(),
        },
        &principal(),
    )
    .await
    .unwrap();

    let error = pattern_search(&db, "synthetic-pattern").await.unwrap_err();
    assert!(matches!(
        error,
        application::quran_morphology::MorphologyToolError::UnavailablePatternField { .. }
    ));
    assert!(error.to_string().contains("pattern, verb_form, morphological_pattern"));
}

/// P2-T82: active-dataset root/lemma browse is deterministic and prefixable.
#[tokio::test]
async fn browse_roots_and_lemmas_are_attributed_and_bounded() {
    let (_dir, db) = ready_db().await;
    let doc = aligned_document(&db).await;
    run_morphology_import(
        &db,
        &import_params(doc, "batch-browse"),
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .unwrap();
    activate_morphology(
        &db,
        &MorphologyActivateParams {
            batch_id: "batch-browse".to_string(),
            approval_id: "appr-morph".to_string(),
            invoked_by: PRINCIPAL.to_string(),
        },
        &principal(),
    )
    .await
    .unwrap();

    let (dataset, roots) = browse_roots(&db, Some("tst"), 10).await.unwrap();
    assert_eq!(dataset, format!("{SLUG}@{VERSION}"));
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0].root, "tst-root");
    assert_eq!(roots[0].dataset, dataset);
    let (dataset, lemmas) = browse_lemmas(&db, Some("lem-"), 2).await.unwrap();
    assert_eq!(dataset, format!("{SLUG}@{VERSION}"));
    assert_eq!(lemmas.len(), 2);
    assert!(lemmas.iter().all(|lemma| lemma.dataset == dataset));
}
