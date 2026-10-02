//! Phase 3 — SC5 pin: displayed canonical text is never modified by
//! normalization in any result (plan 03-01, Task 2).
//!
//! For every search mode on the synthetic fixture, each hit's displayed
//! canonical text is independently resolved through the reader service and
//! compared byte-for-byte with the hit's quotation; the canonical span slices
//! back to a non-empty substring of that displayed text; and the canonical
//! stored text hash is unchanged across the searches (no write-back).
//!
//! Concatenated hits additionally carry a non-empty segmentation whose
//! `canonical_surface` values occur inside the canonical ayah text.
//!
//! This locks the Phase-5 consumer contract: what a UI renders is the stored
//! canonical bytes at every hit's span, for every mode.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use application::quran::activate_edition;
use application::quran_forms::{RebuildParams, rebuild_forms};
use application::quran_index::{IndexBuildParams, QURAN_AYAH_INDEX_ID, rebuild_index};
use application::quran_reader::{QuranReader, QuranReaderService};
use application::quran_search::{
    ExactField, MatchMode, NormalizedProfile, PhraseMode, RateLimiter, SearchOutput, SearchParams,
    search_concatenated, search_exact, search_normalized, search_phrase, search_regex,
};
use domain::{PrincipalId, Timestamp};
use quran_core::{AyahNumber, AyahOptions, EditionSelector, QuranRef, SurahNumber};
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use quran_corpus::sha256_hex;
use quran_normalization::{NormalizationPipeline, ProfileId, ProfileRegistry, SemVer};
use storage::Database as _;
use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

const BASE_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const PRINCIPAL: &str = "00000000-0000-0000-0000-000000000001";
const CREATED_AT: &str = "2026-09-14T00:00:00Z";
const V1_URN: &str = "quran-edition:test-edition-min@0.1.0";
const SLUG: &str = "test-edition-min";
const VERSION: &str = "0.1.0";
// The canonical edition id is the import run id and must be UUID-shaped for the
// reader's typed id mapping (mirrors `tests/common/mod.rs`).
const RUN_ID: &str = "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee";

fn principal() -> PrincipalId {
    PRINCIPAL.parse().unwrap()
}

fn timestamp() -> Timestamp {
    Timestamp::from_ymd_hms(2026, 9, 14, 0, 0, 0).unwrap()
}

/// Migrate → seed → import → activate → forms rebuild → index rebuild
/// (the `search_goldens.rs` `searchable_db` harness verbatim).
async fn searchable_db() -> (tempfile::TempDir, Arc<SqliteDatabase>, std::path::PathBuf, String) {
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
        &*db,
        &ImportInput {
            run_id: RUN_ID.into(),
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
    let ImportOutcome::Completed(success) = outcome;
    let edition_id = success.edition_id;

    activate_edition(&*db, SLUG, VERSION, &principal(), "appr-1", &timestamp())
        .await
        .expect("fixture activation completes");
    rebuild_forms(
        &db,
        &RebuildParams {
            edition_slug: SLUG.to_string(),
            edition_version: VERSION.to_string(),
            invoked_by: PRINCIPAL.to_string(),
            run_tag: "display-identity".to_string(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("forms rebuild completes");
    let data_dir = dir.path().join("index");
    rebuild_index(
        &db,
        &IndexBuildParams {
            index_id: QURAN_AYAH_INDEX_ID.to_string(),
            edition_slug: SLUG.to_string(),
            edition_version: VERSION.to_string(),
            invoked_by: PRINCIPAL.to_string(),
            run_tag: "display-identity-idx".to_string(),
            data_dir: data_dir.clone(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("index build completes");
    (dir, db, data_dir, edition_id)
}

fn l3_pipeline() -> NormalizationPipeline {
    NormalizationPipeline::for_profile(&ProfileRegistry::new(), ProfileId::L3, SemVer::new(1, 0, 0))
        .expect("L3.diacritics@1.0.0 is a seeded profile")
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

/// One deterministic diacriticized fixture token plus its `L3.diacritics` form.
async fn diacritic_free_query(db: &SqliteDatabase, edition_id: &str) -> (String, String) {
    let pipeline = l3_pipeline();
    let mut uow = db.write().await.unwrap();
    let ayahs = uow.quran().list_ayahs_range(edition_id, 1, i64::MAX).await.unwrap();
    uow.rollback().await.unwrap();
    for ayah in &ayahs {
        let mut uow = db.write().await.unwrap();
        let tokens = uow.quran().get_tokens(edition_id, ayah.surah, ayah.ayah).await.unwrap();
        uow.rollback().await.unwrap();
        if tokens.len() >= 2 && !tokens[0].surface.trim().is_empty() {
            let surface = tokens[0].surface.clone();
            let bare = pipeline.apply(&surface).0.text().to_string();
            if bare != surface && !bare.is_empty() {
                return (surface, bare);
            }
        }
    }
    panic!("fixture has no diacriticized multi-token ayah");
}

/// Two adjacent, clean fixture token surfaces (phrase + concatenated queries).
async fn two_token_pair(db: &SqliteDatabase, edition_id: &str) -> (String, String) {
    let mut uow = db.write().await.unwrap();
    let ayahs = uow.quran().list_ayahs_range(edition_id, 1, i64::MAX).await.unwrap();
    uow.rollback().await.unwrap();
    for ayah in &ayahs {
        let mut uow = db.write().await.unwrap();
        let tokens = uow.quran().get_tokens(edition_id, ayah.surah, ayah.ayah).await.unwrap();
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
            return (tokens[0].surface.clone(), tokens[1].surface.clone());
        }
    }
    panic!("fixture has no three-letter-token ayah");
}

async fn resolve_ayah(reader: &QuranReaderService, surah: u16, ayah: u32) -> quran_core::AyahView {
    let reference = QuranRef::Ayah {
        edition: EditionSelector::Pinned { slug: SLUG.to_string(), version: SemVer::new(0, 1, 0) },
        surah: SurahNumber::new(surah).expect("valid surah"),
        ayah: AyahNumber::new(ayah).expect("valid ayah"),
    };
    reader.get_ayah(&reference, &AyahOptions::default()).await.expect("canonical ayah resolves")
}

/// Hash every canonical ayah of the pinned edition, keyed by reference.
async fn canonical_hashes(reader: &QuranReaderService) -> BTreeMap<String, String> {
    let selector =
        EditionSelector::Pinned { slug: SLUG.to_string(), version: SemVer::new(0, 1, 0) };
    let surahs = reader.list_surahs(&selector).await.expect("surahs list");
    let mut hashes = BTreeMap::new();
    for surah in &surahs {
        let reference = QuranRef::Surah { edition: selector.clone(), surah: surah.number };
        let views =
            reader.get_ayahs(&reference, &AyahOptions::default()).await.expect("ayahs list");
        for view in &views {
            hashes.insert(
                view.canonical.reference().to_string(),
                view.canonical.text_hash().hex.clone(),
            );
        }
    }
    hashes
}

/// The mode must return at least one hit — a silently-empty assertion is a bug.
fn assert_mode_hits(out: &SearchOutput, label: &str) {
    assert!(
        !out.hits.is_empty(),
        "{label}: mode must return at least one hit on the fixture query \
         (guards against a silently-empty assertion); total_matches={}",
        out.total_matches
    );
    assert!(out.total_matches >= 1, "{label}: total_matches must be >= 1");
}

/// SC5 per-hit pin: the displayed canonical text is the stored canonical bytes.
///
/// `expect_segmentation` is true only for the concatenated mode, whose hits
/// must additionally tile the query with `canonical_surface`s that occur in the
/// canonical ayah text.
async fn assert_display_identity(
    reader: &QuranReaderService,
    out: &SearchOutput,
    label: &str,
    expect_segmentation: bool,
) {
    for hit in &out.hits {
        let quotation = hit.quotation();
        let surah = quotation.surah_number().get();
        let ayah = quotation.ayah_range().0.get();
        let view = resolve_ayah(reader, surah, ayah).await;
        let canonical = view.canonical.arabic_text();

        // Displayed text == canonical row, byte for byte (no normalization
        // write-back, no re-rendering from a normalized form).
        assert_eq!(
            quotation.arabic_text().as_bytes(),
            canonical.as_bytes(),
            "{label} {surah}:{ayah}: displayed canonical text must be byte-identical to the row"
        );
        assert_eq!(
            quotation.text_hash(),
            view.canonical.text_hash(),
            "{label} {surah}:{ayah}: the quoted hash must be the canonical row hash"
        );

        // The canonical span slices back to a non-empty substring, byte-identical
        // in the displayed and canonical text.
        let bytes = hit
            .byte_range()
            .unwrap_or_else(|| panic!("{label} {surah}:{ayah}: span must map to a byte range"));
        assert!(
            bytes.start < bytes.end,
            "{label} {surah}:{ayah}: span must be non-empty; span={:?}",
            hit.canonical_span()
        );
        assert!(
            (bytes.end as usize) <= canonical.len(),
            "{label} {surah}:{ayah}: span end {} must lie inside canonical text ({} bytes)",
            bytes.end,
            canonical.len()
        );
        let canonical_slice = &canonical.as_bytes()[bytes.start as usize..bytes.end as usize];
        let displayed_slice =
            &quotation.arabic_text().as_bytes()[bytes.start as usize..bytes.end as usize];
        assert!(!canonical_slice.is_empty(), "{label} {surah}:{ayah}: span slice is non-empty");
        assert_eq!(
            canonical_slice, displayed_slice,
            "{label} {surah}:{ayah}: span slice must be byte-identical in displayed and canonical text"
        );

        if expect_segmentation {
            assert!(
                !hit.segmentation().is_empty(),
                "{label} {surah}:{ayah}: concatenated hits must carry segmentation"
            );
            for segment in hit.segmentation() {
                assert!(
                    canonical.contains(&segment.canonical_surface),
                    "{label} {surah}:{ayah}: segmentation surface {:?} must occur in the canonical ayah text",
                    segment.canonical_surface
                );
            }
        } else {
            assert!(
                hit.segmentation().is_empty(),
                "{label} {surah}:{ayah}: only concatenated hits are segmented"
            );
        }
    }
}

/// SC5 — exact search.
#[tokio::test]
async fn exact_hits_display_canonical_bytes() {
    let (_dir, db, data_dir, edition_id) = searchable_db().await;
    let (surface, _bare) = diacritic_free_query(&db, &edition_id).await;
    let reader = QuranReaderService::new(db.clone());

    let out = search_exact(&db, &data_dir, &params(&surface), ExactField::TextExact)
        .await
        .expect("exact search runs");
    assert_mode_hits(&out, "exact");
    assert_display_identity(&reader, &out, "exact", false).await;
}

/// SC5 — normalized search.
#[tokio::test]
async fn normalized_hits_display_canonical_bytes() {
    let (_dir, db, data_dir, edition_id) = searchable_db().await;
    let (_surface, bare) = diacritic_free_query(&db, &edition_id).await;
    let reader = QuranReaderService::new(db.clone());

    let out = search_normalized(
        &db,
        &data_dir,
        &params(&bare),
        NormalizedProfile::Registry(ProfileId::L3, Some(SemVer::new(1, 0, 0))),
    )
    .await
    .expect("normalized search runs");
    assert_mode_hits(&out, "normalized");
    assert_display_identity(&reader, &out, "normalized", false).await;
}

/// SC5 — phrase search.
#[tokio::test]
async fn phrase_hits_display_canonical_bytes() {
    let (_dir, db, data_dir, edition_id) = searchable_db().await;
    let (first, second) = two_token_pair(&db, &edition_id).await;
    let reader = QuranReaderService::new(db.clone());

    let query = format!("{first} {second}");
    let out = search_phrase(
        &db,
        &data_dir,
        &params(&query),
        NormalizedProfile::Registry(ProfileId::L3, Some(SemVer::new(1, 0, 0))),
        PhraseMode::OrderedExact,
        0,
    )
    .await
    .expect("phrase search runs");
    assert_mode_hits(&out, "phrase");
    assert_display_identity(&reader, &out, "phrase", false).await;
}

/// SC5 — concatenated (spaceless) search, with a tiling segmentation.
#[tokio::test]
async fn concatenated_hits_display_canonical_bytes_and_segment() {
    let (_dir, db, data_dir, edition_id) = searchable_db().await;
    let (first, second) = two_token_pair(&db, &edition_id).await;
    let reader = QuranReaderService::new(db.clone());

    let query: String = format!("{first}{second}").split_whitespace().collect();
    let out = search_concatenated(&db, &data_dir, &params(&query), false, 1)
        .await
        .expect("concatenated search runs");
    assert_mode_hits(&out, "concatenated");
    assert_display_identity(&reader, &out, "concatenated", true).await;
}

/// SC5 — regex search.
#[tokio::test]
async fn regex_hits_display_canonical_bytes() {
    let (_dir, db, data_dir, edition_id) = searchable_db().await;
    let (_surface, bare) = diacritic_free_query(&db, &edition_id).await;
    let reader = QuranReaderService::new(db.clone());

    let limiter = RateLimiter::default_regex();
    let out =
        search_regex(&db, &data_dir, &params("x"), "text_bare", &bare, PRINCIPAL, &limiter, 3000)
            .await
            .expect("regex search runs");
    assert_mode_hits(&out, "regex");
    assert_display_identity(&reader, &out, "regex", false).await;
}

/// No search mode writes back to canonical text: the stored hashes are
/// identical before and after running all five modes.
#[tokio::test]
async fn canonical_hashes_are_stable_across_all_modes() {
    let (_dir, db, data_dir, edition_id) = searchable_db().await;
    let (surface, bare) = diacritic_free_query(&db, &edition_id).await;
    let (first, second) = two_token_pair(&db, &edition_id).await;
    let reader = QuranReaderService::new(db.clone());

    let before = canonical_hashes(&reader).await;
    assert!(!before.is_empty(), "the fixture edition must expose canonical ayahs");

    let _ = search_exact(&db, &data_dir, &params(&surface), ExactField::TextExact).await.unwrap();
    let _ = search_normalized(
        &db,
        &data_dir,
        &params(&bare),
        NormalizedProfile::Registry(ProfileId::L3, Some(SemVer::new(1, 0, 0))),
    )
    .await
    .unwrap();
    let _ = search_phrase(
        &db,
        &data_dir,
        &params(&format!("{first} {second}")),
        NormalizedProfile::Registry(ProfileId::L3, Some(SemVer::new(1, 0, 0))),
        PhraseMode::OrderedExact,
        0,
    )
    .await
    .unwrap();
    let _ = search_concatenated(&db, &data_dir, &params(&format!("{first}{second}")), false, 1)
        .await
        .unwrap();
    let limiter = RateLimiter::default_regex();
    let _ =
        search_regex(&db, &data_dir, &params("x"), "text_bare", &bare, PRINCIPAL, &limiter, 3000)
            .await
            .unwrap();

    let after = canonical_hashes(&reader).await;
    assert_eq!(
        before, after,
        "no search mode may modify canonical text (SC5/I8): stored hashes must be stable"
    );
}
