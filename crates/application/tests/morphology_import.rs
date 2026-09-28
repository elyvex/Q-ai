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
    RootUnificationCandidate, activate_morphology, affix_search, browse_lemmas, browse_roots,
    build_affix_relations, build_derived_relations, build_inflectional_relations,
    build_same_form_relations, build_same_lemma_relations, build_same_root_relations,
    build_same_stem_relations, dataset_urn, diff_datasets, enqueue_root_unification, lemma_search,
    morphology_compare, morphology_for_token, pattern_search, root_search, run_morphology_import,
    word_family,
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
            run_tag: "morph-test".to_string(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("forms rebuild completes");
    (dir, db)
}

/// A second read-only pool over the same temp database for raw assertions on
/// persisted rows that have no service read path.
async fn read_pool(dir: &tempfile::TempDir) -> sqlx::SqlitePool {
    let path = dir.path().join("qai.db");
    sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(path).foreign_keys(true))
        .await
        .unwrap()
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

/// Build an aligned array-shape document whose derived lexicon keys exercise
/// every family relation kind (G-05): synthetic root/lemma/stem groups plus a
/// declared prefix/suffix on every token. Real fixture surfaces keep
/// direct-key alignment; every row stays `synthetic_test_only: true`.
///
/// The token index `k` (canonical order across the whole fixture) drives the
/// groups, so `fixtures/quran/lexicon/families/curated.jsonl` can be computed
/// independently and must agree row-for-row. `lemma = k % 9` and
/// `root = k % 3` are chosen so a lemma maps to exactly one root (9 is a
/// multiple of 3): `quran_lemmas` is UNIQUE(dataset_id, lemma), so two roots
/// sharing a lemma text would collide at activation.
async fn family_document(db: &SqliteDatabase) -> String {
    let mut uow = db.write().await.unwrap();
    let active = uow.quran().get_active().await.unwrap().unwrap();
    let ayahs = uow.quran().list_ayahs_range(&active.edition_id, 1, i64::MAX).await.unwrap();
    let mut rows = Vec::new();
    let mut index = 0usize;
    for ayah in &ayahs {
        let tokens =
            uow.quran().get_tokens(&active.edition_id, ayah.surah, ayah.ayah).await.unwrap();
        for token in &tokens {
            let root = format!("root-{}", index % 3);
            let lemma = format!("lem-{}", index % 9);
            let stem = format!("stem-{}", index % 5);
            let prefix = format!("pre-{}", index % 4);
            let suffix = format!("suf-{}", index % 3);
            // Two competing analyses per token; both carry the same family
            // keys so a token has one root/lemma/stem/affix, never a winner.
            for analysis_no in [0u32, 1u32] {
                rows.push(serde_json::json!({
                    "sura_no": ayah.surah,
                    "aya_no": ayah.ayah,
                    "tok_idx": token.position,
                    "analysis_no": analysis_no,
                    "surface_form": token.surface,
                    "lemma_str": lemma,
                    "root_str": root,
                    "stem_str": stem,
                    "tag_native": if analysis_no == 0 { "N" } else { "V" },
                    "tag_unified": if analysis_no == 0 { "noun" } else { "verb" },
                    "layer": "B",
                    "state": "imported",
                    "segmented": true,
                    "morphs": [
                        {"part": "prefix", "text": prefix},
                        {"part": "stem", "text": stem},
                        {"part": "suffix", "text": suffix},
                    ],
                    "synthetic_test_only": true,
                }));
            }
            index += 1;
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

    // G-09: with a dataset active but no populated morpheme index, affix
    // search fails closed with a typed capability error (never a silent empty).
    let err = affix_search(&db, "ويت", "L7.affix").await.unwrap_err();
    assert!(matches!(err, MorphologyToolError::UnavailableDataset { .. }));
    use storage::error::Diagnostic as _;
    assert_eq!(err.code().to_string(), "QAI-MORPH-0004");
    assert!(err.to_string().contains("affix/morpheme index"), "{err}");
}

/// G-09: `affix_search` never returns a silent empty for an active dataset.
/// The dataset path fails closed with a typed capability error; the labeled
/// L7 heuristic still answers (and stays labeled) when no dataset is active.
#[tokio::test]
async fn affix_search_typed_unavailable() {
    use application::quran_morphology::MorphologyToolError;
    use storage::error::Diagnostic as _;
    let (dir, db) = ready_db().await;

    // No active dataset: the labeled L7 heuristic answers over stored forms.
    let pool = read_pool(&dir).await;
    let sample: String = sqlx::query_scalar("SELECT affix_stripped FROM quran_token_forms LIMIT 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    pool.close().await;
    assert!(!sample.is_empty(), "forms rebuild must populate affix_stripped");
    let probe: String = sample.chars().take(2).collect();
    let hits = affix_search(&db, &probe, "L7.affix").await.expect("L7 heuristic answers");
    assert!(!hits.is_empty(), "L7 heuristic must return results for {probe:?}");
    assert!(hits.iter().all(|h| h.backend == "Heuristic (pattern-based)"));

    // Without a dataset the L7 profile is required.
    let err = affix_search(&db, &probe, "other").await.unwrap_err();
    assert!(matches!(err, MorphologyToolError::UnavailableDataset { .. }));
    assert_eq!(err.code().to_string(), "QAI-MORPH-0004");

    // Active dataset without a populated morpheme index: typed error, code
    // QAI-MORPH-0004, message naming the missing capability.
    let doc = family_document(&db).await;
    run_morphology_import(&db, &import_params(doc, "batch-affix"), &AtomicBool::new(false), |_| {})
        .await
        .unwrap();
    activate_morphology(
        &db,
        &MorphologyActivateParams {
            batch_id: "batch-affix".to_string(),
            approval_id: "appr-morph".to_string(),
            invoked_by: PRINCIPAL.to_string(),
        },
        &principal(),
    )
    .await
    .unwrap();

    let err = affix_search(&db, &probe, "L7.affix").await.unwrap_err();
    assert!(matches!(err, MorphologyToolError::UnavailableDataset { .. }));
    assert_eq!(err.code().to_string(), "QAI-MORPH-0004");
    assert!(err.to_string().contains("affix/morpheme index"), "{err}");
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

/// P2-T83…T88 / G-05: every typed family relation kind is built, each row
/// carries a non-empty explanation and an in-domain typed relation, and no
/// analysis is merged, promoted, or marked preferred.
#[tokio::test]
async fn family_relations_explained() {
    let (dir, db) = ready_db().await;
    let doc = family_document(&db).await;
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

    let dataset = format!("{SLUG}@{VERSION}");
    let built = vec![
        ("same_form", build_same_form_relations(&db, &dataset).await.expect("same_form")),
        ("same_lemma", build_same_lemma_relations(&db, &dataset).await.expect("same_lemma")),
        ("same_stem", build_same_stem_relations(&db, &dataset).await.expect("same_stem")),
        ("same_root", build_same_root_relations(&db, &dataset).await.expect("same_root")),
        ("derived", build_derived_relations(&db, &dataset).await.expect("derived")),
        ("inflectional", build_inflectional_relations(&db, &dataset).await.expect("inflectional")),
        ("affix", build_affix_relations(&db, &dataset).await.expect("affix")),
    ];
    for (kind, count) in &built {
        assert!(*count > 0, "relation kind {kind} must build rows, saw {count}");
    }

    let typed =
        ["same_form", "same_lemma", "same_stem", "same_root", "derived", "inflectional", "affix"];
    let pool = read_pool(&dir).await;
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, String)>(
        "SELECT relation, from_id, to_id, explanation, dataset_id, status
         FROM word_family_relations ORDER BY relation, from_id, to_id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(!rows.is_empty());
    let kinds_seen: std::collections::BTreeSet<&str> = rows.iter().map(|r| r.0.as_str()).collect();
    for row in &rows {
        let (relation, from_id, to_id, explanation, dataset_id, status) = row;
        assert!(typed.contains(&relation.as_str()), "relation {relation} outside the 0017 domain");
        assert!(!explanation.trim().is_empty(), "explanation mandatory for {from_id}->{to_id}");
        assert!(
            explanation.contains(relation.as_str()),
            "explanation must state the relation it claims: {explanation}"
        );
        assert_eq!(dataset_id.as_deref(), Some(dataset.as_str()), "dataset attribution");
        assert_eq!(status, "proposed", "built relations stay proposed (never auto-verified)");
    }
    assert_eq!(kinds_seen.len(), typed.len(), "every typed kind present: {kinds_seen:?}");

    // No merge / preferred-analysis surface exists, and no analysis was flipped.
    let ddl: Vec<String> = sqlx::query_scalar(
        "SELECT sql FROM sqlite_master WHERE name IN
           ('word_family_relations','quran_token_analyses','quran_roots','quran_lemmas')",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    for statement in &ddl {
        let lower = statement.to_lowercase();
        for banned in ["is_correct", "is_primary", "selected"] {
            assert!(!lower.contains(banned), "no merge/preferred column may exist: {statement}");
        }
    }
    let flipped: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM quran_token_analyses WHERE status != 'imported'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(flipped, 0, "building relations never promotes/merges an analysis");

    // The read path resolves one member per kind to its typed, explained
    // partner (a single query per kind, driven by the built rows).
    for (kind, _) in &built {
        let (from_id, to_id) = rows
            .iter()
            .find(|r| r.0 == *kind)
            .map(|r| (r.1.clone(), r.2.clone()))
            .unwrap_or_else(|| panic!("kind {kind} has a built row"));
        let (served_dataset, members) = word_family(&db, "token", &from_id).await.unwrap();
        assert_eq!(served_dataset, dataset);
        let partner = members
            .iter()
            .find(|m| m.id == to_id && m.relation == *kind)
            .unwrap_or_else(|| panic!("{kind}: {from_id} must resolve to {to_id}"));
        assert!(!partner.explanation.trim().is_empty());
    }
    pool.close().await;
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

/// P2-T65: explicitly supplied root-unification candidates are queued once and
/// remain pending; no automatic merge or reviewer state is created.
#[tokio::test]
async fn root_unification_candidate_is_idempotent_and_reviewable() {
    let (_dir, db) = ready_db().await;
    let candidate = RootUnificationCandidate {
        left_dataset: "dataset-a".to_string(),
        left_root_id: "root-a".to_string(),
        left_root: "كَتَب".to_string(),
        left_normalized: "كتب".to_string(),
        left_convention: "bare-v1".to_string(),
        right_dataset: "dataset-b".to_string(),
        right_root_id: "root-b".to_string(),
        right_root: "كَتَبَ".to_string(),
        right_normalized: "كتب".to_string(),
        right_convention: "bare-v1".to_string(),
        confidence: 0.91,
        evidence: serde_json::json!({"source": "explicit-review-fixture"}),
        algorithm: Some("manual-review".to_string()),
        algorithm_version: Some("1".to_string()),
    };
    let first = enqueue_root_unification(&db, &candidate).await.unwrap();
    let second = enqueue_root_unification(&db, &candidate).await.unwrap();
    assert_eq!(first.id, second.id);
    assert_eq!(first.status, "pending");
    assert!(first.reviewer.is_none());

    let mut uow = db.write().await.unwrap();
    let pending = uow.quran().list_review_items("pending").await.unwrap();
    uow.rollback().await.unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].kind, "root_unification");
    assert!(pending[0].subject_json.contains("dataset-a"));
    assert!(pending[0].subject_json.contains("dataset-b"));
    assert!(pending[0].evidence_json.contains("explicit-review-fixture"));

    let mut invalid = candidate;
    invalid.confidence = 1.1;
    let error = enqueue_root_unification(&db, &invalid).await.unwrap_err();
    assert!(error.to_string().contains("[0,1]"));
}
