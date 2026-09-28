//! Phase 3 — alpha smoke: the thinnest end-to-end search path (plan 03-01, Task 1).
//!
//! One path only, proven on the synthetic fixture through the REAL services:
//! migrate → import → activate → forms rebuild → index rebuild → normalized
//! search of a diacritic-free Arabic token → a hit that explains itself (the
//! ordered rule trace I9, a canonical span I10, and a verified quotation that
//! is byte-identical to the canonical row).
//!
//! The same binary also exercises the REAL empty-input guard at the CLI
//! boundary (`application::quran_cli::cmd_search`), not the service: the
//! service has no empty guard and legitimately returns `Ok` with zero hits for
//! an empty term, so the `empty` fallback edge is disposed where the operator
//! actually calls it.
//!
//! Mechanics on synthetic text — the mushaf goldens await a licensed corpus
//! (P2-X01); the fixture header reads `reviewed_by: pending-linguist`.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use application::quran::activate_edition;
use application::quran_cli::{SearchCliMode, SearchCliOptions, cmd_search, exit};
use application::quran_forms::{RebuildParams, rebuild_forms};
use application::quran_index::{IndexBuildParams, QURAN_AYAH_INDEX_ID, rebuild_index};
use application::quran_reader::{QuranReader, QuranReaderService};
use application::quran_search::{MatchMode, NormalizedProfile, SearchParams, search_normalized};
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
const RUN_ID: &str = "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee";

fn principal() -> PrincipalId {
    PRINCIPAL.parse().unwrap()
}

fn timestamp() -> Timestamp {
    Timestamp::from_ymd_hms(2026, 9, 14, 0, 0, 0).unwrap()
}

/// Migrate → seed → import → activate → forms rebuild → index rebuild.
///
/// The harness is the `search_goldens.rs` `searchable_db` pattern verbatim
/// (tempdir + repo migrations + `SqliteDatabase::new(dir, 4, true)` + the four
/// raw seed rows), extended to return the canonical `edition_id` so token reads
/// are scoped to the imported edition. The `edition_id` equals the import run
/// id (see `quran_corpus::import`), but it is captured rather than assumed.
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
            run_tag: "alpha-smoke".to_string(),
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
            run_tag: "alpha-smoke-idx".to_string(),
            data_dir: data_dir.clone(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("index build completes");
    (dir, db, data_dir, edition_id)
}

/// The default search normalization profile for the tracer: `L3.diacritics`.
fn l3_pipeline() -> NormalizationPipeline {
    NormalizationPipeline::for_profile(&ProfileRegistry::new(), ProfileId::L3, SemVer::new(1, 0, 0))
        .expect("L3.diacritics@1.0.0 is a seeded profile")
}

/// One deterministic diacriticized fixture token plus its `L3.diacritics` form.
///
/// Mechanics goldens (not mushaf goldens): the first ayah whose first token
/// actually changes under the L3 pipeline.
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

/// Resolve one canonical ayah through the reader service (the independent
/// canonical source of truth for the displayed-text comparison).
async fn resolve_ayah(reader: &QuranReaderService, surah: u16, ayah: u32) -> quran_core::AyahView {
    let reference = QuranRef::Ayah {
        edition: EditionSelector::Pinned { slug: SLUG.to_string(), version: SemVer::new(0, 1, 0) },
        surah: SurahNumber::new(surah).expect("valid surah"),
        ayah: AyahNumber::new(ayah).expect("valid ayah"),
    };
    reader.get_ayah(&reference, &AyahOptions::default()).await.expect("canonical ayah resolves")
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

/// The whole Phase-3 search path, one command, one path.
#[tokio::test]
async fn alpha_smoke_normalized_search_end_to_end() {
    let (_dir, db, data_dir, edition_id) = searchable_db().await;
    let (surface, bare) = diacritic_free_query(&db, &edition_id).await;
    let reader = QuranReaderService::new(db.clone());

    let out = search_normalized(
        &db,
        &data_dir,
        &params(&bare),
        NormalizedProfile::Registry(ProfileId::L3, Some(SemVer::new(1, 0, 0))),
    )
    .await
    .expect("normalized search runs");

    assert!(
        out.total_matches >= 1,
        "diacritic-free {bare:?} (from surface {surface:?}) must return at least one hit; \
         got total_matches={} hits={}",
        out.total_matches,
        out.hits.len()
    );
    assert!(!out.hits.is_empty(), "hits present alongside total_matches");
    assert_eq!(out.rule_set, "L3.diacritics@1.0.0", "served profile");

    for hit in &out.hits {
        let reference = hit.reference();
        let trace = hit.explanation();
        // I9: the ordered rule set is mandatory and non-empty on every hit.
        assert!(
            !trace.profile.is_empty(),
            "hit {reference}: trace profile must be non-empty (I9)"
        );
        assert!(
            !trace.rules_applied.is_empty(),
            "hit {reference}: ordered rule set must be non-empty (I9); trace={trace:?}"
        );
        assert!(
            !trace.contains_heuristic_rules,
            "hit {reference}: L3.diacritics is deterministic; trace={trace:?}"
        );

        let quotation = hit.quotation();
        let text = quotation.arabic_text();
        let bytes = hit.byte_range().expect("canonical span maps to a byte range");
        assert!(
            bytes.start < bytes.end,
            "hit {reference}: canonical span must be non-empty; span={:?}",
            hit.canonical_span()
        );
        assert!(
            (bytes.end as usize) <= text.len(),
            "hit {reference}: span end {} must lie inside canonical text ({} bytes); \
             span={:?} rules={:?}",
            bytes.end,
            text.len(),
            hit.canonical_span(),
            trace.rules_applied
        );

        // SC5: independently resolve the canonical row and pin byte identity.
        let surah = quotation.surah_number().get();
        let ayah = quotation.ayah_range().0.get();
        let view = resolve_ayah(&reader, surah, ayah).await;
        let canonical = view.canonical.arabic_text();
        assert_eq!(
            text.as_bytes(),
            canonical.as_bytes(),
            "hit {reference}: displayed canonical text must be byte-identical to the canonical row"
        );
        assert_eq!(
            quotation.text_hash(),
            view.canonical.text_hash(),
            "hit {reference}: the quoted hash must be the canonical row hash (never recomputed)"
        );

        // The span slices back to a non-empty substring of the displayed text,
        // byte-identical in the displayed and canonical text.
        let displayed_slice = &text.as_bytes()[bytes.start as usize..bytes.end as usize];
        let canonical_slice = &canonical.as_bytes()[bytes.start as usize..bytes.end as usize];
        assert!(!displayed_slice.is_empty(), "hit {reference}: span slice is non-empty");
        assert_eq!(
            displayed_slice, canonical_slice,
            "hit {reference}: span slice must be byte-identical in displayed and canonical text"
        );

        // A normalized (non-concatenated) hit carries no segmentation.
        assert!(
            hit.segmentation().is_empty(),
            "hit {reference}: normalized hits are not segmented"
        );
    }
}

/// Empty and whitespace-only queries are rejected at the CLI boundary by the
/// REAL guard, with a typed usage error — never a panic and never an all-match.
#[tokio::test]
async fn empty_and_whitespace_queries_are_rejected_at_the_cli_boundary() {
    // The guard at the top of `cmd_search` fires before the service opens, so
    // no database is required; the tempdir path stands in for `db_path`.
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap().to_string();

    for text in ["", " ", "   ", "\t\n"] {
        let output = cmd_search(&db_path, &cli_options(text)).await;
        assert_eq!(
            output.exit,
            exit::USAGE,
            "query {text:?} must be a typed usage error, got exit={} human={:?}",
            output.exit,
            output.human
        );
        assert!(
            output.human.contains("provide query text"),
            "query {text:?}: message must name the fix; human={:?}",
            output.human
        );
        assert!(
            output.json.get("error").is_some(),
            "query {text:?}: typed error envelope; json={}",
            output.json
        );
        assert!(
            output.json.get("hits").is_none(),
            "query {text:?}: no hits are produced; json={}",
            output.json
        );
    }
}

fn cli_options(text: &str) -> SearchCliOptions {
    SearchCliOptions {
        text: text.to_string(),
        mode: SearchCliMode::Normalized,
        edition: None,
        field: None,
        match_mode: "whole_token".to_string(),
        profile: Some("L3.diacritics".to_string()),
        rules: None,
        phrase_mode: "ordered_exact".to_string(),
        slop: 0,
        allow_cross_ayah: false,
        max_ayah_span: 1,
        surah: None,
        juz: None,
        page: None,
        revelation_place: None,
        global_range: None,
        limit: 100,
        offset: 0,
        explain: false,
        highlight: false,
        timeout_ms: 3000,
    }
}
