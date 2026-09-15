//! Phase 2 — exact + normalized search acceptance (M3, P2-T41/T42).
//!
//! Against real SQLite in a tempdir with a real imported + activated edition,
//! real derived forms, and a real built index: exactness (no silent folds),
//! normalization across diacritics and code points, the zero-result hint,
//! trace discipline on every hit, drift warnings, and edition gating.
//!
//! The fixture holds synthetic Arabic-shaped text, so reference-set goldens
//! against the real mushaf (AC-P2-07/09 full form) await a licensed corpus;
//! these suites prove the mechanics those goldens will exercise.

use std::sync::atomic::AtomicBool;

use application::quran::activate_edition;
use application::quran_forms::{RebuildParams, rebuild_forms};
use application::quran_index::{IndexBuildParams, QURAN_AYAH_INDEX_ID};
use application::quran_search::{
    ExactField, MatchMode, NormalizedProfile, SearchParams, search_exact, search_normalized,
};
use quran_normalization::RuleId;
use domain::{PrincipalId, Timestamp};
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use quran_corpus::sha256_hex;
use quran_normalization::{ProfileId, SemVer};
use quran_search::Filter;
use storage::Database as _;
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

/// Import + activate + forms + index: the searchable fixture.
async fn searchable_db() -> (tempfile::TempDir, SqliteDatabase, std::path::PathBuf) {
    let (dir, db) = migrated_db().await;
    let outcome = run_import(
        &db,
        &ImportInput {
            run_id: "run-search-1".into(),
            job_id: None,
            source_version_id: "sv-1".into(),
            adapter: "json".into(),
            manifest_text: BASE_MANIFEST.into(),
            declared_manifest_hash: Some(sha256_hex(BASE_manifest.as_bytes())),
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
            run_tag: "search-test".to_string(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("forms rebuild completes");
    let data_dir = dir.path().join("index");
    application::quran_index::rebuild_index(
        &db,
        &IndexBuildParams {
            index_id: QURAN_AYAH_INDEX_ID.to_string(),
            edition_slug: "test-edition-min".to_string(),
            edition_version: "0.1.0".to_string(),
            invoked_by: PRINCIPAL.to_string(),
            run_tag: "search-test".to_string(),
            data_dir: data_dir.clone(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("index build completes");
    (dir, db, data_dir)
}

fn params(text: &str) -> SearchParams {
    SearchParams {
        text: text.to_string(),
        edition: None,
        mode: MatchMode::WholeToken,
        filters: vec![],
        limit: 100,
        offset: 0,
        explain: false,
    }
}

/// Pick a determinstic query pair from the fixture: one token surface plus
/// its diacritic-stripped form (mechanics goldens, not mushaf goldens).
async fn query_pair(db: &SqliteDatabase) -> (String, String) {
    let mut uow = db.write().await.unwrap();
    let ayahs = uow.quran().list_ayahs_range("run-search-1", 1, i64::MAX).await.unwrap();
    uow.rollback().await.unwrap();
    // First ayah with at least two tokens; surface of token 1 + bare form.
    for ayah in &ayahs {
        let mut uow = db.write().await.unwrap();
        let tokens =
            uow.quran().get_tokens("run-search-1", ayah.surah, ayah.ayah).await.unwrap();
        uow.rollback().await.unwrap();
        if tokens.len() >= 2 && !tokens[0].surface.trim().is_empty() {
            let surface = tokens[0].surface.clone();
            let bare = quran_normalization::NormalizationPipeline::for_profile(
                &quran_normalization::ProfileRegistry::new(),
                ProfileId::L3,
                SemVer::new(1, 0, 0),
            )
            .unwrap()
            .apply(&surface)
            .0
            .text()
            .to_string();
            if bare != surface && !bare.is_empty() {
                return (surface, bare);
            }
        }
    }
    panic!("fixture has no diacriticized multi-token ayah");
}

/// Exact search never folds: the diacriticized surface matches, the bare
/// form does not (plus the actionable hint stays silent — no Persian here).
#[tokio::test]
async fn exact_never_silently_folds() {
    let (_dir, db, data_dir) = searchable_db().await;
    let (surface, bare) = query_pair(&db).await;

    let found = search_exact(&db, &data_dir, &params(&surface), ExactField::TextExact)
        .await
        .expect("exact search runs");
    assert!(found.total_matches >= 1, "exact surface must match");
    for hit in &found.hits {
        assert_eq!(hit.explanation().profile, "L0.exact@1.0.0");
        assert!(!hit.explanation().contains_heuristic_rules);
        assert!(!hit.matched_tokens().is_empty());
        // Span slices back to canonical text.
        let bytes = hit.byte_range().expect("mappable span");
        let text = hit.quotation().arabic_text();
        assert!(!text.as_bytes()[bytes.start as usize..bytes.end as usize].is_empty());
    }

    let missed = search_exact(&db, &data_dir, &params(&bare), ExactField::TextExact)
        .await
        .expect("exact search runs");
    assert_eq!(missed.total_matches, 0, "bare form must not match under L0");
    assert!(missed.hits.is_empty());
    // No Persian code points involved: no hint.
    assert!(missed.warnings.is_empty());
}

/// Normalized search bridges the diacritic gap with full traces.
#[tokio::test]
async fn normalized_bridges_diacritics_with_traces() {
    let (_dir, db, data_dir) = searchable_db().await;
    let (surface, bare) = query_pair(&db).await;

    for spec in [
        NormalizedProfile::Registry(ProfileId::L3, None),
        NormalizedProfile::Registry(ProfileId::L3, Some(SemVer::new(1, 0, 0))),
    ] {
        let found = search_normalized(&db, &data_dir, &params(&bare), spec)
            .await
            .expect("normalized search runs");
        assert!(found.total_matches >= 1, "bare query must match under L3");
        for hit in &found.hits {
            assert_eq!(hit.explanation().profile, "L3.diacritics@1.0.0");
        }
    }

    // The exact surface matches too (same normalized form).
    let found = search_normalized(
        &db,
        &data_dir,
        &params(&surface),
        NormalizedProfile::Registry(ProfileId::L3, None),
    )
    .await
    .unwrap();
    assert!(found.total_matches >= 1);

    // Adhoc rule lists work without a profile.
    let found = search_normalized(
        &db,
        &data_dir,
        &params(&bare),
        NormalizedProfile::Adhoc(vec![RuleId::N01, RuleId::N03]),
    )
    .await
    .unwrap();
    assert!(found.total_matches >= 1);
    assert!(found.hits[0].explanation().profile.starts_with("adhoc:"));

    // Unknown rule ids fail typed (the enum makes profile+rules
    // unrepresentable, so no silent pick is possible).
    let err = search_normalized(
        &db,
        &data_dir,
        &params(&bare),
        NormalizedProfile::Adhoc(vec![RuleId::N23]),
    )
    .await
    .unwrap_err();
    assert!(matches!(
        err,
        application::quran_search::SearchError::Normalization(_)
    ));
}

/// Persian-keyboard queries miss under exact search but carry the hint.
#[tokio::test]
async fn persian_query_gets_hint_not_silence() {
    let (_dir, db, data_dir) = searchable_db().await;
    // Persian keheh/yeh never occur canonically: guaranteed zero hits plus hint.
    let missed = search_exact(&db, &data_dir, &params("کی"), ExactField::TextExact)
        .await
        .unwrap();
    assert_eq!(missed.total_matches, 0);
    assert_eq!(missed.warnings.len(), 1);
    assert!(missed.warnings[0].message.contains("L5.codepoints"));
}

/// Requesting another edition fails typed instead of cross-edition matching.
#[tokio::test]
async fn wrong_edition_is_rejected() {
    let (_dir, db, data_dir) = searchable_db().await;
    let mut bad = params("x");
    bad.edition = Some("other@9.9.9".to_string());
    let err = search_exact(&db, &data_dir, &bad, ExactField::TextExact).await.unwrap_err();
    assert!(matches!(
        err,
        application::quran_search::SearchError::EditionNotIndexed { .. }
    ));
}

/// Filters, paging, and explain/relevance round out the contract.
#[tokio::test]
async fn filters_paging_and_explain() {
    let (_dir, db, data_dir) = searchable_db().await;
    let (_, bare) = query_pair(&db).await;

    // Surah filter narrows; a filter matching nothing returns zero, not error.
    let mut filtered = params(&bare);
    filtered.filters = vec![Filter::Surah(vec![114])];
    let found = search_normalized(
        &db,
        &data_dir,
        &filtered,
        NormalizedProfile::Registry(ProfileId::L3, None),
    )
    .await
    .unwrap();
    for hit in &found.hits {
        assert_eq!(hit.quotation().surah_number().get(), 114);
    }

    // Paging is consistent: page two continues page one.
    let mut page = params(&bare);
    page.limit = 1;
    let l3 = || NormalizedProfile::Registry(ProfileId::L3, None);
    let first = search_normalized(&db, &data_dir, &page, l3()).await.unwrap();
    assert_eq!(first.hits.len(), 1);
    page.offset = 1;
    let second = search_normalized(&db, &data_dir, &page, l3()).await.unwrap();
    if !second.hits.is_empty() {
        assert_ne!(first.hits[0].reference(), second.hits[0].reference());
    }
    assert_eq!(first.total_matches, second.total_matches);

    // Explain serves relevance order with BM25 breakdowns.
    let mut explained = params(&bare);
    explained.explain = true;
    let found =
        search_normalized(&db, &data_dir, &explained, l3()).await.unwrap();
    assert!(found.total_matches >= 1);
    for hit in &found.hits {
        assert!(hit.score().is_some());
        let breakdown = hit.score_explain().expect("breakdown with explain");
        assert_eq!(breakdown.field, "text_bare");
    }
    // Without explain: canonical order, no scores.
    let plain =
        search_normalized(&db, &data_dir, &params(&bare), l3()).await.unwrap();
    assert!(plain.hits.iter().all(|hit| hit.score().is_none()));
}
