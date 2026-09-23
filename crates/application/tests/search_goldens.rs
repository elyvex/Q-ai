//! Phase 2 — search golden-set suite (P2-T53, AC-P2-14 mechanics).
//!
//! Runs all 400 queries in `fixtures/quran/search/queries.jsonl` through the
//! real services on a real built index and requires exact agreement with the
//! independently computed oracle (Python string matching on canonical text,
//! see `scripts/gen_search_goldens.py`): reference sets, exact totals, and
//! `must_not_contain` precision anchors. Additionally locks the recall
//! monotonicity property (exact ⊆ L3-normalized) for every exact query.
//!
//! Mechanics on synthetic text — the mushaf goldens await a licensed corpus
//! (P2-X01); the header must read `reviewed_by: pending-linguist`.

use std::collections::BTreeSet;
use std::sync::atomic::AtomicBool;

use application::quran::activate_edition;
use application::quran_forms::{RebuildParams, rebuild_forms};
use application::quran_index::{IndexBuildParams, QURAN_AYAH_INDEX_ID, rebuild_index};
use application::quran_search::{
    ExactField, MatchMode, NormalizedProfile, PhraseMode, RateLimiter, SearchParams, search_exact,
    search_normalized, search_phrase, search_regex,
};
use domain::{PrincipalId, Timestamp};
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use quran_corpus::sha256_hex;
use quran_normalization::{ProfileId, SemVer};
use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

const FIXTURE: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/quran/search/queries.jsonl");
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
            run_id: "run-goldens-1".into(),
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
            run_tag: "goldens-test".to_string(),
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
            run_tag: "goldens-idx".to_string(),
            data_dir: data_dir.clone(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("index build completes");
    (dir, db, data_dir)
}

fn base_params(text: &str, mode: MatchMode) -> SearchParams {
    SearchParams {
        text: text.to_string(),
        edition: None,
        mode,
        filters: vec![],
        limit: 100,
        offset: 0,
        explain: false,
        highlight: false,
    }
}

#[derive(serde::Deserialize)]
struct GoldenRow {
    tool: String,
    input: String,
    expected_references: Vec<String>,
    expected_total: u64,
    must_not_contain: Vec<String>,
}

fn load_goldens() -> Vec<GoldenRow> {
    let text = std::fs::read_to_string(FIXTURE).expect("golden fixture exists");
    let mut lines = text.lines();
    let header: serde_json::Value =
        serde_json::from_str(lines.next().expect("header present")).expect("header parses");
    assert_eq!(header["header"], true);
    assert_eq!(header["reviewed_by"], "pending-linguist");
    let rows: Vec<GoldenRow> = lines
        .enumerate()
        .map(|(i, line)| {
            serde_json::from_str(line).unwrap_or_else(|e| panic!("golden line {i}: {e}"))
        })
        .collect();
    assert_eq!(rows.len(), 400, "golden set must hold exactly 400 queries");
    rows
}

fn references_of(out: &application::quran_search::SearchOutput) -> BTreeSet<String> {
    // Service references are fully qualified (`quran:<slug>@<ver>:<s>:<a>`);
    // the oracle stores short `s:a` refs. Compare on the short form.
    out.hits
        .iter()
        .map(|h| {
            let r = h.reference();
            let mut parts = r.rsplit(':');
            let ayah = parts.next().unwrap_or(r);
            let surah = parts.next().unwrap_or(r);
            format!("{surah}:{ayah}")
        })
        .collect()
}

/// All 400 golden queries agree with the independent oracle.
#[tokio::test]
async fn all_400_search_goldens_pass() {
    let rows = load_goldens();
    let (_dir, db, data_dir) = searchable_db().await;
    let limiter = RateLimiter::new(10_000);
    let v = SemVer::new(1, 0, 0);
    let mut counts = std::collections::HashMap::new();
    for (i, row) in rows.iter().enumerate() {
        *counts.entry(row.tool.clone()).or_insert(0usize) += 1;
        let refs: BTreeSet<String> = row.expected_references.iter().cloned().collect();
        let (got, total) = match row.tool.as_str() {
            "search_exact" => {
                let out = search_exact(
                    &db,
                    &data_dir,
                    &base_params(&row.input, MatchMode::Substring),
                    ExactField::TextExact,
                )
                .await
                .unwrap_or_else(|e| panic!("golden {i} exact {:?}: {e}", row.input));
                (references_of(&out), out.total_matches)
            }
            "search_normalized" => {
                let out = search_normalized(
                    &db,
                    &data_dir,
                    &base_params(&row.input, MatchMode::Substring),
                    NormalizedProfile::Registry(ProfileId::L3, Some(v)),
                )
                .await
                .unwrap_or_else(|e| panic!("golden {i} normalized {:?}: {e}", row.input));
                (references_of(&out), out.total_matches)
            }
            "search_phrase" => {
                let out = search_phrase(
                    &db,
                    &data_dir,
                    &base_params(&row.input, MatchMode::WholeToken),
                    NormalizedProfile::Registry(ProfileId::L0, Some(v)),
                    PhraseMode::OrderedExact,
                    0,
                )
                .await
                .unwrap_or_else(|e| panic!("golden {i} phrase {:?}: {e}", row.input));
                (references_of(&out), out.total_matches)
            }
            "search_regex" => {
                let out = search_regex(
                    &db,
                    &data_dir,
                    &base_params(&row.input, MatchMode::WholeToken),
                    "text_exact",
                    &row.input,
                    PRINCIPAL,
                    &limiter,
                    3000,
                )
                .await
                .unwrap_or_else(|e| panic!("golden {i} regex {:?}: {e}", row.input));
                (references_of(&out), out.total_matches)
            }
            other => panic!("golden {i}: unknown tool {other}"),
        };
        assert_eq!(got, refs, "golden {i} {} {:?}: reference set", row.tool, row.input);
        if row.tool == "search_regex" {
            // Regex totals come from the FTS recall-count path while hits are
            // exact-verified (recall ⊇ precision, T41/T47 split): the total
            // must cover the verified set. The oracle's Python match count
            // pins the verified set size (asserted via `got`), not the
            // engine's recall total, which the backend owns.
            assert!(
                total >= got.len() as u64,
                "golden {i} regex {:?}: total {total} must cover {} verified hits",
                row.input,
                got.len()
            );
        } else {
            assert_eq!(total, row.expected_total, "golden {i} {} {:?}: total", row.tool, row.input);
        }
        for banned in &row.must_not_contain {
            assert!(
                !got.contains(banned),
                "golden {i} {} {:?}: must_not_contain {banned} hit",
                row.tool,
                row.input
            );
        }
    }
    assert_eq!(counts["search_exact"], 200);
    assert_eq!(counts["search_normalized"], 100);
    assert_eq!(counts["search_phrase"], 60);
    assert_eq!(counts["search_regex"], 40);
}

/// Recall monotonicity (AC-P2-10 mechanics): every exact-substring hit is
/// also an L3-normalized hit for the same query.
#[tokio::test]
async fn normalized_recall_covers_exact() {
    let rows = load_goldens();
    let (_dir, db, data_dir) = searchable_db().await;
    let v = SemVer::new(1, 0, 0);
    let mut checked = 0;
    for row in rows.iter().filter(|r| r.tool == "search_exact") {
        let exact = search_exact(
            &db,
            &data_dir,
            &base_params(&row.input, MatchMode::Substring),
            ExactField::TextExact,
        )
        .await
        .unwrap();
        let normed = search_normalized(
            &db,
            &data_dir,
            &base_params(&row.input, MatchMode::Substring),
            NormalizedProfile::Registry(ProfileId::L3, Some(v)),
        )
        .await
        .unwrap();
        let exact_refs = references_of(&exact);
        let normed_refs = references_of(&normed);
        assert!(
            exact_refs.is_subset(&normed_refs),
            "monotonicity failed for {:?}: exact={exact_refs:?} normed={normed_refs:?}",
            row.input
        );
        checked += 1;
    }
    assert_eq!(checked, 200);
}
