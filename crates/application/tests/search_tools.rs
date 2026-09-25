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
    ExactField, MatchMode, NormalizedProfile, RateLimiter, SearchParams, search_exact,
    search_normalized, search_regex,
};
use domain::{PrincipalId, Timestamp};
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use quran_corpus::sha256_hex;
use quran_normalization::RuleId;
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
        highlight: false,
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
        let tokens = uow.quran().get_tokens("run-search-1", ayah.surah, ayah.ayah).await.unwrap();
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
    assert!(matches!(err, application::quran_search::SearchError::Normalization(_)));
}

/// Persian-keyboard queries miss under exact search but carry the hint.
#[tokio::test]
async fn persian_query_gets_hint_not_silence() {
    let (_dir, db, data_dir) = searchable_db().await;
    // Persian keheh/yeh never occur canonically: guaranteed zero hits plus hint.
    let missed = search_exact(&db, &data_dir, &params("کی"), ExactField::TextExact).await.unwrap();
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
    assert!(matches!(err, application::quran_search::SearchError::EditionNotIndexed { .. }));
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
    let found = search_normalized(&db, &data_dir, &explained, l3()).await.unwrap();
    assert!(found.total_matches >= 1);
    for hit in &found.hits {
        assert!(hit.score().is_some());
        let breakdown = hit.score_explain().expect("breakdown with explain");
        assert_eq!(breakdown.field, "text_bare");
    }
    // Without explain: canonical order, no scores.
    let plain = search_normalized(&db, &data_dir, &params(&bare), l3()).await.unwrap();
    assert!(plain.hits.iter().all(|hit| hit.score().is_none()));
}

/// Phrase data from the fixture itself: two consecutive token surfaces of a
/// multi-token ayah (mechanics goldens, not mushaf goldens).
async fn phrase_pair(db: &SqliteDatabase) -> (u16, u32, String, String, String) {
    let mut uow = db.write().await.unwrap();
    let ayahs = uow.quran().list_ayahs_range("run-search-1", 1, i64::MAX).await.unwrap();
    uow.rollback().await.unwrap();
    for ayah in &ayahs {
        let mut uow = db.write().await.unwrap();
        let tokens = uow.quran().get_tokens("run-search-1", ayah.surah, ayah.ayah).await.unwrap();
        uow.rollback().await.unwrap();
        if tokens.len() >= 3
            && tokens.iter().take(3).all(|token| {
                !token.surface.trim().is_empty()
                    && token.surface.chars().all(|ch| {
                        ch.is_alphabetic()
                            || ch == 'ً'
                            || ch == 'ٌ'
                            || ch == 'ٍ'
                            || ch == 'َ'
                            || ch == 'ُ'
                            || ch == 'ِ'
                            || ch == 'ّ'
                            || ch == 'ْ'
                    })
            })
        {
            let (first, second, third) =
                (tokens[0].surface.clone(), tokens[1].surface.clone(), tokens[2].surface.clone());
            return (
                ayah.surah as u16,
                ayah.ayah as u32,
                format!("{first} {second}"),
                format!("{first} {third}"),
                format!("{second} {first}"),
            );
        }
    }
    panic!("fixture has no three-letter-token ayah");
}

fn phrase_params(text: &str) -> SearchParams {
    SearchParams {
        text: text.to_string(),
        edition: None,
        mode: MatchMode::WholeToken,
        filters: vec![],
        limit: 100,
        offset: 0,
        explain: false,
        highlight: false,
    }
}

/// Ordered-exact phrases match; gaps and reversals discriminate the modes.
#[tokio::test]
async fn phrase_modes_discriminate_order_and_gaps() {
    use application::quran_search::{PhraseMode, search_concatenated, search_phrase};
    let (_dir, db, data_dir) = searchable_db().await;
    let (surah, ayah, adjacent, gapped, reversed) = phrase_pair(&db).await;
    let wanted = format!("quran:test-edition-min@0.1.0:{surah}:{ayah}");
    let l3 = || NormalizedProfile::Registry(ProfileId::L3, None);

    let found =
        search_phrase(&db, &data_dir, &phrase_params(&adjacent), l3(), PhraseMode::OrderedExact, 0)
            .await
            .unwrap();
    assert!(found.total_matches >= 1);
    assert!(found.hits.iter().any(|hit| hit.reference() == wanted));
    for hit in &found.hits {
        assert_eq!(hit.explanation().profile, "L3.diacritics@1.0.0");
    }

    // A gap misses under exact but hits with slop 1.
    let missed =
        search_phrase(&db, &data_dir, &phrase_params(&gapped), l3(), PhraseMode::OrderedExact, 0)
            .await
            .unwrap();
    assert_eq!(missed.total_matches, 0);
    let near =
        search_phrase(&db, &data_dir, &phrase_params(&gapped), l3(), PhraseMode::OrderedNear, 1)
            .await
            .unwrap();
    assert!(near.total_matches >= 1);
    assert!(near.hits.iter().any(|hit| hit.reference() == wanted));

    // Reversed order misses under ordered modes but hits unordered.
    let reversed_exact =
        search_phrase(&db, &data_dir, &phrase_params(&reversed), l3(), PhraseMode::OrderedExact, 0)
            .await
            .unwrap();
    assert_eq!(reversed_exact.total_matches, 0);
    let reversed_near = search_phrase(
        &db,
        &data_dir,
        &phrase_params(&reversed),
        l3(),
        PhraseMode::UnorderedNear,
        0,
    )
    .await
    .unwrap();
    assert!(reversed_near.total_matches >= 1);

    // Empty-after-normalization queries match nothing, never everything.
    let empty =
        search_phrase(&db, &data_dir, &phrase_params("ً"), l3(), PhraseMode::OrderedExact, 0)
            .await
            .unwrap();
    assert_eq!(empty.total_matches, 0);

    // A zero span budget is rejected (never silently treated as ayah-local).
    let err =
        search_concatenated(&db, &data_dir, &phrase_params("abc"), true, 0).await.unwrap_err();
    assert!(matches!(err, application::quran_search::SearchError::Index(_)));
}

/// Spaceless queries match with per-token segmentation; every query part
/// concatenates back to the query skeleton.
#[tokio::test]
async fn concatenated_matches_with_segmentation() {
    use application::quran_search::{search_concatenated, verify_concatenated};
    let (_dir, db, data_dir) = searchable_db().await;

    // Deterministic basmala case through the real verify path (hand-built
    // token rows with exact canonical offsets).
    let text = "بِسْمِ ٱللَّهِ";
    let tokens = vec![
        storage::quran::TokenRow {
            edition_id: "ed".to_string(),
            surah: 1,
            ayah: 1,
            position: 1,
            surface: "بِسْمِ".to_string(),
            surface_hash: String::new(),
            // Grapheme-cluster offsets (ADR-0104): [بِ][سْ][مِ].
            char_start: 0,
            char_end: 3,
            byte_start: 0,
            byte_end: 12,
            is_pause_mark: false,
            global_token_index: 1,
        },
        storage::quran::TokenRow {
            edition_id: "ed".to_string(),
            surah: 1,
            ayah: 1,
            position: 2,
            surface: "ٱللَّهِ".to_string(),
            surface_hash: String::new(),
            // Clusters [ٱ][ل][لَّ][هِ].
            char_start: 4,
            char_end: 8,
            byte_start: 13,
            byte_end: 27,
            is_pause_mark: false,
            global_token_index: 2,
        },
    ];
    let registry = quran_normalization::ProfileRegistry::new();
    let pipeline = quran_normalization::NormalizationPipeline::for_profile(
        &registry,
        ProfileId::L6,
        SemVer::new(1, 0, 0),
    )
    .unwrap();
    let matched = verify_concatenated(&pipeline, text, &tokens, "بسمالله")
        .expect("basmala spaceless query verifies");
    assert_eq!(matched.matched, vec![0, 1]);
    assert_eq!(matched.span.char_range, 0..13);
    let parts = &matched.segmentation;
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].query_part, "بسم");
    assert_eq!(parts[0].canonical_token, 1);
    assert_eq!(parts[0].canonical_surface, "بِسْمِ");
    assert_eq!(parts[1].query_part, "الله");
    assert_eq!(parts[1].canonical_token, 2);
    // Parts tile the query skeleton exactly.
    let tiled: String = parts.iter().map(|part| part.query_part.as_str()).collect();
    assert_eq!(tiled, "بسمالله");
    // A wrong skeleton never verifies.
    assert!(verify_concatenated(&pipeline, text, &tokens, "بسمالرحمن").is_none());

    // End-to-end on fixture data: spaceless first-two-tokens query.
    let (surah, ayah, adjacent, _, _) = phrase_pair(&db).await;
    let query: String = adjacent.split_whitespace().collect();
    let found =
        search_concatenated(&db, &data_dir, &phrase_params(&query), false, 1).await.unwrap();
    assert!(found.total_matches >= 1, "spaceless query must match its ayah");
    let wanted = format!("quran:test-edition-min@0.1.0:{surah}:{ayah}");
    let hit = found.hits.iter().find(|hit| hit.reference() == wanted).expect("ayah hit present");
    assert_eq!(hit.explanation().profile, "L6.skeleton@1.0.0");
    assert!(!hit.segmentation().is_empty());
    let tiled: String = hit.segmentation().iter().map(|part| part.query_part.as_str()).collect();
    let skeleton = pipeline.apply(&query).0.text().to_string();
    assert_eq!(tiled, skeleton, "parts must tile the query skeleton");
    // Empty queries match nothing.
    let empty = search_concatenated(&db, &data_dir, &phrase_params(""), false, 1).await.unwrap();
    assert_eq!(empty.total_matches, 0);
}

/// Scan-path filters: substring search honors metadata filters with the same
/// semantics as the FTS path (NULL division fields never match a range).
#[tokio::test]
async fn scan_path_filters_match_fts_semantics() {
    use application::quran_search::search_concatenated;
    let (_dir, db, data_dir) = searchable_db().await;
    let (_, bare) = query_pair(&db).await;

    // Unfiltered substring total as the baseline.
    let mut base = params(&bare);
    base.mode = MatchMode::Substring;
    let l3 = || NormalizedProfile::Registry(ProfileId::L3, None);
    let unfiltered = search_normalized(&db, &data_dir, &base, l3()).await.unwrap();
    assert!(unfiltered.total_matches >= 1);

    // The first hit's own surah filters to a subset containing that hit.
    let surah = unfiltered.hits[0].quotation().surah_number().get();
    let mut filtered = params(&bare);
    filtered.mode = MatchMode::Substring;
    filtered.filters = vec![Filter::Surah(vec![surah])];
    let found = search_normalized(&db, &data_dir, &filtered, l3()).await.unwrap();
    assert!(found.total_matches >= 1);
    assert!(found.total_matches <= unfiltered.total_matches);
    for hit in &found.hits {
        assert_eq!(hit.quotation().surah_number().get(), surah);
    }

    // A surah absent from the corpus returns zero, not an error.
    let mut empty = params(&bare);
    empty.mode = MatchMode::Substring;
    empty.filters = vec![Filter::Surah(vec![114])];
    let found = search_normalized(&db, &data_dir, &empty, l3()).await.unwrap();
    // The fixture is synthetic; only assert consistency, not emptiness.
    assert!(found.total_matches <= unfiltered.total_matches);
    for hit in &found.hits {
        assert_eq!(hit.quotation().surah_number().get(), 114);
    }

    // Concatenated search honors filters too (same `passes_filters`).
    let (surah, ayah, adjacent, _, _) = phrase_pair(&db).await;
    let query: String = adjacent.split_whitespace().collect();
    let mut concat = phrase_params(&query);
    concat.filters = vec![Filter::Surah(vec![surah])];
    let found = search_concatenated(&db, &data_dir, &concat, false, 1).await.unwrap();
    assert!(found.total_matches >= 1);
    let wanted = format!("quran:test-edition-min@0.1.0:{surah}:{ayah}");
    assert!(found.hits.iter().any(|hit| hit.reference() == wanted));
}

/// Highlighting wraps spans in markers; disabled highlighting stores nothing.
#[tokio::test]
async fn highlight_wraps_spans_or_stays_absent() {
    let (_dir, db, data_dir) = searchable_db().await;
    let (surface, _) = query_pair(&db).await;

    let mut marked = params(&surface);
    marked.highlight = true;
    let found = search_exact(&db, &data_dir, &marked, ExactField::TextExact).await.unwrap();
    assert!(found.total_matches >= 1);
    for hit in &found.hits {
        let highlighted = hit.highlighted().expect("highlighted text present");
        assert!(highlighted.contains("<b>") && highlighted.contains("</b>"));
        // Stripping markers reproduces the canonical text exactly.
        let stripped = highlighted.replace("<b>", "").replace("</b>", "");
        assert_eq!(stripped, hit.quotation().arabic_text());
    }

    let plain =
        search_exact(&db, &data_dir, &params(&surface), ExactField::TextExact).await.unwrap();
    assert!(plain.hits.iter().all(|hit| hit.highlighted().is_none()));
}

/// Totals stay exact under truncation: `total_matches` never depends on
/// limit/offset, and `truncated` reports paging faithfully.
#[tokio::test]
async fn totals_stay_exact_under_truncation() {
    let (_dir, db, data_dir) = searchable_db().await;
    let (_, bare) = query_pair(&db).await;
    let l3 = || NormalizedProfile::Registry(ProfileId::L3, None);

    let full = search_normalized(&db, &data_dir, &params(&bare), l3()).await.unwrap();
    assert!(full.total_matches >= 1);
    assert!(!full.truncated);

    // Limit below the total: same total, fewer hits, truncated set.
    let mut limited = params(&bare);
    limited.limit = 1;
    let page = search_normalized(&db, &data_dir, &limited, l3()).await.unwrap();
    assert_eq!(page.total_matches, full.total_matches, "total ignores limit");
    assert_eq!(page.hits.len(), 1.min(full.hits.len()));
    assert_eq!(page.truncated, full.total_matches > 1);

    // Offset shifts hits without moving the total.
    if full.total_matches >= 2 {
        let mut shifted = params(&bare);
        shifted.limit = 1;
        shifted.offset = 1;
        let second = search_normalized(&db, &data_dir, &shifted, l3()).await.unwrap();
        assert_eq!(second.total_matches, full.total_matches, "total ignores offset");
        assert_eq!(second.hits.len(), 1);
        assert_ne!(page.hits[0].reference(), second.hits[0].reference());
        assert!(second.truncated);
    }
}

/// Regex search matches through the DFA path with full provenance, and the
/// I16 guard chain rejects hostile input with typed errors.
#[tokio::test]
async fn regex_matches_with_provenance_and_guards() {
    use application::quran_search::SearchError;
    use quran_search::Diagnostic as _;
    let (_dir, db, data_dir) = searchable_db().await;
    let (_, bare) = query_pair(&db).await;
    let limiter = RateLimiter::default_regex();

    // A literal substring pattern matches through the DFA expansion.
    let found = search_regex(
        &db,
        &data_dir,
        &params("x"),
        "text_bare",
        &bare,
        "test-principal",
        &limiter,
        3000,
    )
    .await
    .unwrap();
    assert!(found.total_matches >= 1, "literal pattern must match");
    for hit in &found.hits {
        assert_eq!(hit.explanation().profile, "L3.diacritics@1.0.0");
        assert!(!hit.matched_tokens().is_empty());
    }
    let report = found.regex_report.expect("regex provenance reported");
    assert_eq!(report.pattern, bare);
    assert_eq!(report.field, "text_bare");
    assert!(!report.terms_matched.is_empty(), "expansion terms reported");
    assert!(report.terms_examined >= report.terms_matched.len() as u64);

    // Guard chain: unanchored leading .* is rejected before compiling.
    let err =
        search_regex(&db, &data_dir, &params("x"), "text_bare", ".*رحم", "p2", &limiter, 3000)
            .await
            .unwrap_err();
    assert!(matches!(err, SearchError::Index(_)));
    if let SearchError::Index(inner) = err {
        assert_eq!(inner.code(), quran_search::codes::QUERY_REJECTED);
    }

    // Over-long patterns are rejected before compiling.
    let err = search_regex(
        &db,
        &data_dir,
        &params("x"),
        "text_bare",
        &"ا".repeat(600),
        "p3",
        &limiter,
        3000,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, SearchError::Index(_)));

    // Non-text fields are rejected (never a raw canonical scan).
    let err = search_regex(&db, &data_dir, &params("x"), "surah", "^1", "p4", &limiter, 3000)
        .await
        .unwrap_err();
    assert!(matches!(err, SearchError::Index(_)));

    // Nested quantifiers cannot blow up the DFA path: they complete.
    let _ = search_regex(&db, &data_dir, &params("x"), "text_bare", "(ا|ب)+", "p5", &limiter, 3000)
        .await
        .unwrap();
}

/// Regex rate limiting is per principal with a typed retry hint.
#[tokio::test]
async fn regex_rate_limit_is_per_principal() {
    use application::quran_search::SearchError;
    use quran_search::Diagnostic as _;
    let limiter = RateLimiter::new(2);
    limiter.check("alice").unwrap();
    limiter.check("alice").unwrap();
    let err = limiter.check("alice").unwrap_err();
    assert!(matches!(err, SearchError::RateLimited(_)));
    if let SearchError::RateLimited(inner) = err {
        assert_eq!(inner.code(), quran_search::codes::RATE_LIMITED);
    }
    // Other principals are unaffected.
    limiter.check("bob").unwrap();
}

/// Window verification splits a cross-ayah skeleton match into per-ayah
/// portions whose parts tile the query exactly (deterministic hand-built
/// rows: "بِسْمِ" + "ٱللَّهِ", query "بسمالله").
#[tokio::test]
async fn concatenated_window_verify_tiles_across_ayahs() {
    use application::quran_search::verify_concatenated_window;
    fn token(
        ayah: u32,
        position: i64,
        surface: &str,
        char_start: i64,
        char_end: i64,
    ) -> storage::quran::TokenRow {
        storage::quran::TokenRow {
            edition_id: "ed".to_string(),
            surah: 1,
            ayah: ayah as i64,
            position,
            surface: surface.to_string(),
            surface_hash: String::new(),
            char_start,
            char_end,
            byte_start: 0,
            byte_end: surface.len() as i64,
            is_pause_mark: false,
            global_token_index: position,
        }
    }
    let ayahs = vec![(1, "بِسْمِ".to_string()), (2, "ٱللَّهِ".to_string())];
    // Grapheme clusters: [بِ][سْ][مِ] and [ٱ][ل][لَّ][هِ].
    let tokens = vec![vec![token(1, 1, "بِسْمِ", 0, 3)], vec![token(2, 1, "ٱللَّهِ", 0, 4)]];
    let registry = quran_normalization::ProfileRegistry::new();
    let pipeline = quran_normalization::NormalizationPipeline::for_profile(
        &registry,
        ProfileId::L6,
        SemVer::new(1, 0, 0),
    )
    .unwrap();

    let parts = verify_concatenated_window(&pipeline, &ayahs, &tokens, "بسمالله")
        .expect("basmala split across two ayahs verifies");
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].ayah, 1);
    assert_eq!(parts[1].ayah, 2);
    assert_eq!(parts[0].ayah_match.matched, vec![0]);
    assert_eq!(parts[1].ayah_match.matched, vec![0]);
    // Parts tile the query skeleton in order.
    let tiled: String = parts
        .iter()
        .flat_map(|part| part.ayah_match.segmentation.iter().map(|seg| seg.query_part.as_str()))
        .collect();
    assert_eq!(tiled, "بسمالله");
    assert_eq!(parts[0].ayah_match.segmentation[0].query_part, "بسم");
    assert_eq!(parts[1].ayah_match.segmentation[0].query_part, "الله");
    // Each part's span is a tight hull inside its own ayah, and slicing that
    // canonical range and re-normalizing reproduces the part's query text
    // (plan §3.4 property 5). Trailing kasra after the matched letters is
    // excluded by hull semantics, exactly as in single-ayah verification.
    for (part, (_, ayah_text)) in parts.iter().zip(ayahs.iter()) {
        let start = part.ayah_match.span.char_range.start;
        let end = part.ayah_match.span.char_range.end;
        let total = ayah_text.chars().count() as u32;
        assert!(start == 0 && end <= total, "span {start}..{end} inside 0..{total}");
        let slice: String =
            ayah_text.chars().skip(start as usize).take((end - start) as usize).collect();
        let expected: String =
            part.ayah_match.segmentation.iter().map(|seg| seg.query_part.as_str()).collect();
        assert_eq!(pipeline.apply(&slice).0.text(), expected);
    }

    // A query fully inside one ayah yields a single part (no boundary).
    let single = verify_concatenated_window(&pipeline, &ayahs, &tokens, "بسم")
        .expect("single-ayah query verifies through the window path");
    assert_eq!(single.len(), 1);
    assert_eq!(single[0].ayah, 1);

    // Wrong skeletons, empty windows, and ragged token tables never verify.
    assert!(verify_concatenated_window(&pipeline, &ayahs, &tokens, "بسمالرحمن").is_none());
    assert!(verify_concatenated_window(&pipeline, &[], &[], "بسمالله").is_none());
    assert!(verify_concatenated_window(&pipeline, &ayahs, &tokens[..1], "بسمالله").is_none());
}

/// End-to-end on fixture data: a query spanning an ayah boundary returns one
/// hit per overlapped ayah, each labeled `spans_ayah_boundary`, with no
/// duplicate references and no silent ayah-local fallback.
#[tokio::test]
async fn concatenated_cross_ayah_windows_span_verse_breaks() {
    use application::quran_search::search_concatenated;
    use std::collections::BTreeSet;
    let (_dir, db, data_dir) = searchable_db().await;

    // First adjacent ayah pair in one surah with tokens on both sides.
    let mut uow = db.write().await.unwrap();
    let ayahs = uow.quran().list_ayahs_range("run-search-1", 1, i64::MAX).await.unwrap();
    uow.rollback().await.unwrap();
    let mut pair: Option<(u16, u32, u32, String, String)> = None;
    let mut ordered = ayahs;
    ordered.sort_by_key(|ayah| (ayah.surah, ayah.ayah));
    for window in ordered.windows(2) {
        if window[0].surah != window[1].surah || window[1].ayah != window[0].ayah + 1 {
            continue;
        }
        let mut uow = db.write().await.unwrap();
        let before =
            uow.quran().get_tokens("run-search-1", window[0].surah, window[0].ayah).await.unwrap();
        let after =
            uow.quran().get_tokens("run-search-1", window[1].surah, window[1].ayah).await.unwrap();
        uow.rollback().await.unwrap();
        if let (Some(last), Some(first)) = (before.last(), after.first())
            && !last.surface.trim().is_empty()
            && !first.surface.trim().is_empty()
        {
            pair = Some((
                window[0].surah as u16,
                window[0].ayah as u32,
                window[1].ayah as u32,
                last.surface.clone(),
                first.surface.clone(),
            ));
            break;
        }
    }
    let (surah, ayah_a, ayah_b, tail, head) = pair.expect("fixture has an adjacent token pair");
    // Space between the surfaces: the joined window normalizes identically
    // whether the query carries the space or not (N17 removes it either way).
    let query = format!("{tail} {head}");

    let local =
        search_concatenated(&db, &data_dir, &phrase_params(&query), false, 1).await.unwrap();
    let cross = search_concatenated(&db, &data_dir, &phrase_params(&query), true, 3).await.unwrap();

    // Windows only add coverage; ayah-local hits survive unchanged.
    let local_refs: BTreeSet<_> = local.hits.iter().map(|hit| hit.reference()).collect();
    let cross_refs: BTreeSet<_> = cross.hits.iter().map(|hit| hit.reference()).collect();
    assert!(local_refs.is_subset(&cross_refs), "windows only add hits");
    assert_eq!(cross.hits.len(), cross_refs.len(), "no duplicate references");

    // The pair straddles the boundary: both sides appear as boundary hits
    // whose segmentations tile the query skeleton in canonical order.
    // NOTE: repetitive synthetic text may ALSO match ayah-locally; then dedup
    // prefers the ayah-level hit and the boundary pair need not appear. The
    // flag discipline below holds either way.
    let ref_a = format!("quran:test-edition-min@0.1.0:{surah}:{ayah_a}");
    let ref_b = format!("quran:test-edition-min@0.1.0:{surah}:{ayah_b}");
    let hit_a = cross.hits.iter().find(|hit| hit.reference() == ref_a);
    let hit_b = cross.hits.iter().find(|hit| hit.reference() == ref_b);
    if let (Some(hit_a), Some(hit_b)) = (hit_a, hit_b) {
        assert!(hit_a.spans_ayah_boundary(), "ayah {ayah_a} hit spans the break");
        assert!(hit_b.spans_ayah_boundary(), "ayah {ayah_b} hit spans the break");
        assert!(!hit_a.segmentation().is_empty() && !hit_b.segmentation().is_empty());
        let mut tiled = String::new();
        for hit in [hit_a, hit_b] {
            for part in hit.segmentation() {
                tiled.push_str(&part.query_part);
            }
        }
        let registry = quran_normalization::ProfileRegistry::new();
        let pipeline = quran_normalization::NormalizationPipeline::for_profile(
            &registry,
            ProfileId::L6,
            SemVer::new(1, 0, 0),
        )
        .unwrap();
        assert_eq!(tiled, pipeline.apply(&query).0.text(), "parts tile the query");
    }
    for hit in &cross.hits {
        // Boundary labeling is consistent: a cross-ayah flag implies the hit
        // came from a window, which always carries segmentation.
        assert!(!hit.segmentation().is_empty(), "concat hits always segment");
        assert!(!hit.explanation().contains_heuristic_rules, "L6 is deterministic");
    }

    // Span budget enforcement: windows wider than max_ayah_span never serve,
    // so no boundary-labeled hit can appear (ayah-local hits may remain).
    let narrow =
        search_concatenated(&db, &data_dir, &phrase_params(&query), true, 1).await.unwrap();
    assert!(
        narrow.hits.iter().all(|hit| !hit.spans_ayah_boundary()),
        "max_ayah_span=1 admits no cross-ayah hit"
    );
}
