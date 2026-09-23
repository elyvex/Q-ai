//! Phase 2 — search latency gates (P2-T55, AC-P2-43 harness shape).
//!
//! Measures single-shot wall times for every search tool on the fixture
//! index and asserts generous CI bounds. These bounds lock the benchmark
//! harness shape; the plan §17.1 p50/p99 targets gate full-corpus runs on
//! reference hardware (recorded per-op below for the runbook), not the
//! 14-ayah fixture where every op completes in milliseconds.

use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use application::quran::activate_edition;
use application::quran_forms::{RebuildParams, rebuild_forms};
use application::quran_index::{IndexBuildParams, QURAN_AYAH_INDEX_ID, rebuild_index};
use application::quran_search::{
    ExactField, MatchMode, NormalizedProfile, PhraseMode, RateLimiter, SearchParams,
    search_concatenated, search_exact, search_normalized, search_phrase, search_regex,
};
use domain::{PrincipalId, Timestamp};
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use quran_corpus::sha256_hex;
use quran_normalization::{ProfileId, SemVer};
use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

const BASE_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const PRINCIPAL: &str = "00000000-0000-0000-0000-000000000001";
const CREATED_AT: &str = "2026-09-14T00:00:00Z";
const V1_URN: &str = "quran-edition:test-edition-min@0.1.0";

/// Generous CI bound per op on the fixture (fixture ops take milliseconds;
/// the bound guards against orders-of-magnitude regressions, not p99).
const FIXTURE_BOUND: Duration = Duration::from_millis(2000);

fn principal() -> PrincipalId {
    PRINCIPAL.parse().unwrap()
}

fn timestamp() -> Timestamp {
    Timestamp::from_ymd_hms(2026, 9, 14, 0, 0, 0).unwrap()
}

async fn searchable_db() -> (tempfile::TempDir, SqliteDatabase, std::path::PathBuf) {
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
            run_id: "run-latency-1".into(),
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
            run_tag: "latency-test".to_string(),
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
            edition_slug: "test-edition-min".to_string(),
            edition_version: "0.1.0".to_string(),
            invoked_by: PRINCIPAL.to_string(),
            run_tag: "latency-idx".to_string(),
            data_dir: data_dir.clone(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("index build completes");
    (dir, db, data_dir)
}

fn base_params(text: &str) -> SearchParams {
    SearchParams {
        text: text.to_string(),
        edition: None,
        mode: MatchMode::Substring,
        filters: vec![],
        limit: 100,
        offset: 0,
        explain: false,
        highlight: false,
    }
}

/// Every search tool completes within the fixture bound; actuals print for
/// the runbook. Plan §17.1 targets (4-core laptop, warm index, full corpus)
/// are recorded in the assertion messages as the full-corpus gate.
#[tokio::test]
async fn search_latency_within_fixture_bounds() {
    let (_dir, db, data_dir) = searchable_db().await;
    let v = SemVer::new(1, 0, 0);
    let limiter = RateLimiter::new(10_000);

    // (op name, plan §17.1 p50 target on full corpus, measured).
    let mut actuals: Vec<(&str, &str, Duration)> = Vec::new();

    let started = Instant::now();
    search_exact(&db, &data_dir, &base_params("ويت"), ExactField::TextExact).await.unwrap();
    actuals.push(("search_exact", "p50 < 5ms", started.elapsed()));

    let started = Instant::now();
    search_normalized(
        &db,
        &data_dir,
        &base_params("ويت"),
        NormalizedProfile::Registry(ProfileId::L3, Some(v)),
    )
    .await
    .unwrap();
    actuals.push(("search_normalized/L3", "p50 < 8ms", started.elapsed()));

    let started = Instant::now();
    search_normalized(
        &db,
        &data_dir,
        &base_params("ويت"),
        NormalizedProfile::Adhoc(vec![
            quran_normalization::RuleId::N01,
            quran_normalization::RuleId::N03,
        ]),
    )
    .await
    .unwrap();
    actuals.push(("search_normalized/adhoc", "p50 < 60ms", started.elapsed()));

    let started = Instant::now();
    search_phrase(
        &db,
        &data_dir,
        &base_params("ويت تسا"),
        NormalizedProfile::Registry(ProfileId::L0, Some(v)),
        PhraseMode::OrderedExact,
        0,
    )
    .await
    .unwrap();
    actuals.push(("search_phrase", "p50 < 12ms", started.elapsed()));

    let started = Instant::now();
    search_concatenated(&db, &data_dir, &base_params("ويتسا"), false, 1).await.unwrap();
    actuals.push(("search_concatenated", "p50 < 35ms", started.elapsed()));

    let started = Instant::now();
    search_regex(
        &db,
        &data_dir,
        &base_params("^ويت"),
        "text_exact",
        "^ويت",
        PRINCIPAL,
        &limiter,
        3000,
    )
    .await
    .unwrap();
    actuals.push(("search_regex", "p50 < 80ms", started.elapsed()));

    let started = Instant::now();
    let registry = application::quran_normalize::builtin_registry();
    let pipe = quran_normalization::NormalizationPipeline::for_profile(
        &registry,
        ProfileId::L3,
        SemVer::new(1, 0, 0),
    )
    .unwrap();
    let _ = pipe.apply("بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيمِ");
    actuals.push(("normalize --explain path", "p50 < 1ms", started.elapsed()));

    for (op, target, elapsed) in &actuals {
        eprintln!("latency: {op} took {elapsed:?} (plan §17.1 {target} on full corpus)");
        assert!(
            *elapsed < FIXTURE_BOUND,
            "{op} exceeded the fixture bound {FIXTURE_BOUND:?}: {elapsed:?}"
        );
    }
    assert_eq!(actuals.len(), 7);
}
