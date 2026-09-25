//! Phase 2 — counting & discovery acceptance (P2-T95…T103, T109).
//!
//! Against real SQLite with real derived forms: exact SQL aggregation,
//! determinism (same rules ⇒ same number), verbatim disclaimers, hapax
//! rule-relativity, co-occurrence/collocation structure, near-duplicate
//! verification, and typed unavailability for lexicon-gated targets.

use std::sync::atomic::AtomicBool;

use application::quran::activate_edition;
use application::quran_counting::{
    COLLOCATION_MIN_COUNT, INTERVAL_DISCLAIMER, MISSING_FORM_DISCLAIMER, NO_INTERPRETATION_NOTE,
    collocation, cooccurrence, distribution, first_last_occurrence, frequency, hapax_search,
    interval_analysis, lemma_frequency, missing_expected_form, near_duplicate_passages,
    numeric_report, root_frequency, unusual_usage,
};
use application::quran_forms::{RebuildParams, rebuild_forms};
use domain::{PrincipalId, Timestamp};
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use quran_corpus::sha256_hex;
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

async fn ready_db() -> (tempfile::TempDir, SqliteDatabase) {
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
    let outcome = run_import(
        &db,
        &ImportInput {
            run_id: "run-count-1".into(),
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
            run_tag: "count-test".to_string(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("forms rebuild completes");
    (dir, db)
}

/// Pick a frequent bare form straight from the stored forms (no guessing).
async fn sample_form(db: &SqliteDatabase) -> String {
    use storage::Database as _;
    use storage::quran::FormColumn;
    let mut uow = db.write().await.unwrap();
    let active = uow.quran().get_active().await.unwrap().unwrap();
    let distinct =
        uow.quran().list_distinct_forms(&active.edition_id, FormColumn::Bare).await.unwrap();
    uow.rollback().await.unwrap();
    distinct.into_iter().max_by_key(|(_, c)| *c).map(|(f, _)| f).expect("forms exist")
}

/// T95: exact aggregation with rules + checksum; T109: determinism.
#[tokio::test]
async fn frequency_exact_and_deterministic() {
    let (_dir, db) = ready_db().await;
    let target = sample_form(&db).await;
    let first = frequency(&db, &target, "L3.diacritics").await.unwrap();
    assert!(first.count > 0);
    assert_eq!(first.rules.profile, "L3.diacritics");
    assert!(!first.by_surah.is_empty());
    let total: u64 = first.by_surah.values().sum();
    assert_eq!(total, first.count, "breakdown must sum to the total");
    assert!(first.checksum.starts_with("sha256:"));

    // Determinism: same rules ⇒ same number, same checksum.
    let second = frequency(&db, &target, "L3.diacritics").await.unwrap();
    assert_eq!(second.count, first.count);
    assert_eq!(second.checksum, first.checksum);
    assert_eq!(second.rules, first.rules);

    // Profile change visibly changes the rules block (rule-relativity).
    let other = frequency(&db, &target, "L5.codepoints").await.unwrap();
    assert_ne!(other.rules.profile, first.rules.profile);
}

/// T96/T100: distribution provenance + numeric report without interpretation.
#[tokio::test]
async fn distribution_and_numeric_report() {
    let (_dir, db) = ready_db().await;
    let target = sample_form(&db).await;
    let dist = distribution(&db, &target, "L3.diacritics").await.unwrap();
    assert!(!dist.partition_provenance.is_empty());
    assert!(!dist.warnings.is_empty(), "single-source warning required");
    assert_eq!(dist.frequency.count, frequency(&db, &target, "L3.diacritics").await.unwrap().count);

    let report = numeric_report(&db, &target, "L3.diacritics").await.unwrap();
    assert_eq!(report.note, NO_INTERPRETATION_NOTE);
    let json: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&report).unwrap()).unwrap();
    // Report shape is fixed: frequency payload + note. No free-text field
    // exists for interpretation to hide in.
    assert_eq!(json.as_object().unwrap().len(), 2);
    assert!(json.get("frequency").is_some() && json.get("note").is_some());
}

/// T99/T103: occurrences carry verbatim disclaimers; missing form proves zero.
#[tokio::test]
async fn occurrences_and_missing_form_disclaimers() {
    let (_dir, db) = ready_db().await;
    let target = sample_form(&db).await;
    let span = first_last_occurrence(&db, &target, "L3.diacritics").await.unwrap();
    assert!(span.first.is_some() && span.last.is_some());
    assert!(span.first.unwrap() <= span.last.unwrap());
    assert_eq!(span.disclaimer, INTERVAL_DISCLAIMER);
    let interval = interval_analysis(&db, &target, "L3.diacritics").await.unwrap();
    assert_eq!(interval.disclaimer, INTERVAL_DISCLAIMER);

    let missing = missing_expected_form(&db, "zzqqxxnotaword", "L3.diacritics").await.unwrap();
    assert_eq!(missing.count, 0);
    assert_eq!(missing.disclaimer, MISSING_FORM_DISCLAIMER);

    // A present target is NOT missing (typed refusal, not a zero).
    assert!(missing_expected_form(&db, &target, "L3.diacritics").await.is_err());
}

/// T101: hapax states its profile; every entry counts exactly once.
#[tokio::test]
async fn hapax_is_profile_relative() {
    let (_dir, db) = ready_db().await;
    let report = hapax_search(&db, "L3.diacritics", 50).await.unwrap();
    assert_eq!(report.profile, "L3.diacritics");
    assert!(report.hapax.iter().all(|(_, c)| *c == 1));
    // Sorted + truncated deterministically.
    let mut sorted = report.hapax.clone();
    sorted.sort();
    assert_eq!(sorted, report.hapax);
    // A different profile yields its own (possibly different) set.
    let other = hapax_search(&db, "L5.codepoints", 50).await.unwrap();
    assert_eq!(other.profile, "L5.codepoints");
}

/// T97/T98: co-occurrence structure + collocation floors and ordering.
#[tokio::test]
async fn cooccurrence_and_collocation() {
    let (_dir, db) = ready_db().await;
    let target = sample_form(&db).await;
    let (rules, hits) = cooccurrence(&db, &target, "L3.diacritics", 3, 10).await.unwrap();
    assert_eq!(rules.profile, "L3.diacritics");
    assert!(hits.windows(2).all(|w| w[0].count >= w[1].count), "ranked desc");
    assert!(hits.iter().all(|h| h.form != target), "target never co-occurs with itself");

    let (crules, col) = collocation(&db, &target, "L3.diacritics", 3, 10).await.unwrap();
    assert_eq!(crules.profile, "L3.diacritics");
    for hit in &col {
        assert!(hit.observed >= COLLOCATION_MIN_COUNT, "min-count floor");
        assert!(hit.expected > 0.0 && hit.llr >= 0.0);
        assert!(hit.pmi.is_finite() && hit.t_score.is_finite());
    }
    assert!(col.windows(2).all(|w| w[0].llr >= w[1].llr), "LLR-ranked");
}

/// T102: near-duplicates verify exactly (Jaccard ≥ threshold, ordered).
#[tokio::test]
async fn near_duplicates_verified() {
    let (_dir, db) = ready_db().await;
    let (rules, hits) = near_duplicate_passages(&db, 0.9, 20).await.unwrap();
    assert_eq!(rules.profile, "L6.skeleton");
    for hit in &hits {
        assert!(hit.jaccard >= 0.9);
        assert!(hit.first < hit.second, "ordered pairs, no self-matches");
    }
    assert!(hits.windows(2).all(|w| w[0].jaccard >= w[1].jaccard));
}

/// Lexicon-gated targets fail typed (AC-P2-01 fallback), never guessed.
#[tokio::test]
async fn morphology_gated_targets_unavailable() {
    use storage::error::Diagnostic as _;
    let (_dir, db) = ready_db().await;
    for err in [
        root_frequency(&db, "كتب").await.unwrap_err(),
        lemma_frequency(&db, "كتاب").await.unwrap_err(),
        unusual_usage(&db, "ويت", "L3.diacritics").await.unwrap_err(),
    ] {
        assert_eq!(err.code().to_string(), "QAI-CNT-0005");
        assert!(err.remedy().unwrap().contains("morphology"));
    }
}
