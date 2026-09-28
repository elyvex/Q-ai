//! Phase 2 — search latency gates (P2-T55, AC-P2-43 harness shape).
//!
//! Measures single-shot wall times for every search tool on the fixture
//! index and asserts the machine-readable fixture bound. The bound lives in
//! `fixtures/quran/performance/budgets.json` (G-06) together with the ADR-0207
//! concatenated `p99 <= 150 ms` target and the plan §17.1 p50/p99 table. The
//! full-corpus p50/p99 rows gate reference-hardware runs on a licensed corpus
//! and are **OD-11-dependent**: no licensed corpus is active, so they are
//! recorded here and never executed against the 14-ayah fixture.

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

/// The codified budget artifact (G-06): ADR-0207 concatenated `p99 <= 150 ms`,
/// the plan §17.1 p50/p99 table, and the fixture bound. `include_str!` keeps
/// the artifact from silently disappearing.
const BUDGETS_JSON: &str = include_str!("../../../fixtures/quran/performance/budgets.json");

/// Generous CI bound per op on the fixture (fixture ops take milliseconds;
/// the bound guards against orders-of-magnitude regressions, not p99). It is
/// asserted equal to the artifact's `fixture_bound_ms` — never loosened.
const FIXTURE_BOUND: Duration = Duration::from_millis(2000);

/// Parse the committed budget artifact.
fn budgets() -> serde_json::Value {
    serde_json::from_str(BUDGETS_JSON).expect("budgets.json parses")
}

/// Read one named budget row's `value_ms`.
fn budget_ms(doc: &serde_json::Value, name: &str) -> u64 {
    doc["budgets"][name]["value_ms"]
        .as_u64()
        .unwrap_or_else(|| panic!("missing budget row {name} in budgets.json"))
}

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
    // The fixture bound comes from the codified artifact (G-06) and must equal
    // the pre-existing 2000 ms — the artifact may never silently loosen it.
    let doc = budgets();
    let fixture_bound = Duration::from_millis(budget_ms(&doc, "fixture_bound_ms"));
    assert_eq!(fixture_bound, FIXTURE_BOUND, "the fixture bound must not be loosened");

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
            *elapsed < fixture_bound,
            "{op} exceeded the fixture bound {fixture_bound:?}: {elapsed:?}"
        );
    }
    assert_eq!(actuals.len(), 7);

    // The plan §17.1 p50/p99 rows (search.exact/normalized/concatenated/…)
    // gate full-corpus runs on reference hardware and are OD-11-dependent: no
    // licensed corpus is active yet, so they are recorded, never executed
    // against the fixture.
    assert_eq!(
        doc["budgets"]["search.concatenated.p99_ms"]["applies_to"], "full-corpus",
        "full-corpus rows are dataset-gated (OD-11), not fixture-executed"
    );
    assert!(
        doc["notes"].as_array().is_some_and(|notes| {
            notes.iter().any(|note| note.as_str().is_some_and(|text| text.contains("OD-11")))
        }),
        "the artifact must record the OD-11 dependency"
    );
}

/// G-06: the budget artifact exists, parses, codifies ADR-0207's concatenated
/// `p99 <= 150 ms` plus the plan §17.1 p50/p99 table, and every row carries
/// `applies_to` + `rationale` so no threshold is undocumented.
#[test]
fn budget_artifact_codifies_adr_0207_and_fixture_bound() {
    let doc = budgets();
    assert_eq!(doc["format"], "qai.quran.performance-budgets");
    assert_eq!(doc["format_version"], 1);
    assert!(doc["regression_tolerance_percent"].as_u64().is_some());

    // ADR-0207's concatenated target, and the plan §17.1 neighbours.
    assert_eq!(budget_ms(&doc, "search.concatenated.p99_ms"), 150);
    assert_eq!(budget_ms(&doc, "search.concatenated.p50_ms"), 35);
    assert_eq!(budget_ms(&doc, "search.normalized.p50_ms"), 8);
    assert_eq!(budget_ms(&doc, "search.normalized.p99_ms"), 40);
    assert_eq!(budget_ms(&doc, "search.exact.p50_ms"), 5);
    // The fixture bound is carried over unchanged.
    assert_eq!(budget_ms(&doc, "fixture_bound_ms"), 2000);

    // Every named row is documented and typed.
    let budgets = doc["budgets"].as_object().expect("budgets is an object");
    assert!(budgets.len() >= 20, "the §17.1 table is codified in full");
    for (name, row) in budgets {
        assert!(row["value_ms"].as_u64().is_some(), "{name} must carry a numeric value_ms");
        let applies_to =
            row["applies_to"].as_str().unwrap_or_else(|| panic!("{name} needs applies_to"));
        assert!(
            matches!(applies_to, "full-corpus" | "fixture"),
            "{name} has an unknown applies_to `{applies_to}`"
        );
        assert!(
            row["rationale"].as_str().is_some_and(|text| !text.is_empty()),
            "{name} needs a non-empty rationale"
        );
    }
    // Exactly one fixture-scoped row: the bound the harness gates against.
    let fixture_rows: Vec<&String> = budgets
        .iter()
        .filter(|(_, row)| row["applies_to"] == "fixture")
        .map(|(name, _)| name)
        .collect();
    assert_eq!(fixture_rows, vec![&"fixture_bound_ms".to_string()]);
}
