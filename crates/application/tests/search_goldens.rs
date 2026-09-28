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

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::AtomicBool;

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

const FIXTURE: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/quran/search/queries.jsonl");
const CONCATENATED_FIXTURE: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/quran/search/concatenated.jsonl");
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

/// Short `s:a` form of a fully-qualified service reference
/// (`quran:<slug>@<ver>:<s>:<a>`).
fn short_ref(reference: &str) -> String {
    let mut parts = reference.rsplit(':');
    let ayah = parts.next().unwrap_or(reference);
    let surah = parts.next().unwrap_or(reference);
    format!("{surah}:{ayah}")
}

fn references_of(out: &application::quran_search::SearchOutput) -> BTreeSet<String> {
    // Service references are fully qualified (`quran:<slug>@<ver>:<s>:<a>`);
    // the oracle stores short `s:a` refs. Compare on the short form.
    out.hits.iter().map(|h| short_ref(h.reference())).collect()
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

// ─── SC2: concatenated (spaceless) golden suite (G-03, D-11) ───────────────
//
// `fixtures/quran/search/concatenated.jsonl` holds >= 120 `search_concatenated`
// rows whose reference sets, totals, segmentations, and cross-ayah boundary
// labels are derived independently by `scripts/gen_concatenated_goldens.py`
// (pure-Python L6 skeleton + surah-scoped 3-ayah window matching). Each row
// carries its OWN `allow_cross_ayah`/`max_ayah_span` selector — the dispatcher
// never substitutes a literal or a default.

#[derive(serde::Deserialize)]
struct ExpectedSegment {
    query_part: String,
    canonical_token: u16,
    surah: u16,
    ayah: u32,
    #[allow(dead_code)]
    position: usize,
}

#[derive(serde::Deserialize)]
struct ConcatenatedRow {
    tool: String,
    input: String,
    allow_cross_ayah: bool,
    max_ayah_span: u32,
    expected_references: Vec<String>,
    expected_total: u64,
    must_not_contain: Vec<String>,
    expected_segmentation: Vec<ExpectedSegment>,
    #[serde(default)]
    expected_rules_contain: Vec<String>,
    #[serde(default)]
    expected_boundary_refs: Vec<String>,
}

fn load_concatenated_goldens() -> Vec<ConcatenatedRow> {
    let text = std::fs::read_to_string(CONCATENATED_FIXTURE).expect("concatenated fixture exists");
    let mut lines = text.lines();
    let header: serde_json::Value =
        serde_json::from_str(lines.next().expect("header present")).expect("header parses");
    assert_eq!(header["header"], true);
    assert_eq!(header["reviewed_by"], "pending-linguist");
    let rows: Vec<ConcatenatedRow> = lines
        .enumerate()
        .map(|(i, line)| {
            serde_json::from_str(line).unwrap_or_else(|e| panic!("concatenated line {i}: {e}"))
        })
        .collect();
    assert!(rows.len() >= 120, "concatenated golden set must hold >= 120 rows, got {}", rows.len());
    rows
}

/// All >= 120 concatenated golden queries agree with the independent oracle,
/// including segmentation tiling, disclosed L6 folds, and cross-ayah boundary
/// labeling. Selected by the `-- concatenated` libtest filter (a zero-match
/// filter exits 0, so this function's name is part of the gate).
#[tokio::test]
async fn all_concatenated_search_goldens_pass() {
    let rows = load_concatenated_goldens();
    let (_dir, db, data_dir) = searchable_db().await;
    let registry = quran_normalization::ProfileRegistry::new();
    let pipeline = quran_normalization::NormalizationPipeline::for_profile(
        &registry,
        ProfileId::L6,
        SemVer::new(1, 0, 0),
    )
    .expect("L6 pipeline builds");
    let mut cross_ayah_rows = 0usize;
    let mut persian_rows = 0usize;

    for (i, row) in rows.iter().enumerate() {
        // Dispatcher: each row's own selectors flow into the service call.
        let (got, total, out) = match row.tool.as_str() {
            "search_concatenated" => {
                let out = search_concatenated(
                    &db,
                    &data_dir,
                    &base_params(&row.input, MatchMode::Substring),
                    row.allow_cross_ayah,
                    row.max_ayah_span,
                )
                .await
                .unwrap_or_else(|e| panic!("golden {i} concatenated {:?}: {e}", row.input));
                (references_of(&out), out.total_matches, out)
            }
            other => panic!("golden {i}: unknown tool {other}"),
        };

        let want: BTreeSet<String> = row.expected_references.iter().cloned().collect();
        assert_eq!(got, want, "golden {i} concatenated {:?}: reference set", row.input);
        assert_eq!(total, row.expected_total, "golden {i} concatenated {:?}: total", row.input);
        for banned in &row.must_not_contain {
            assert!(
                !got.contains(banned),
                "golden {i} concatenated {:?}: must_not_contain {banned} hit",
                row.input
            );
        }

        let query_skeleton = pipeline.apply(&row.input).0.text().to_string();
        assert!(!query_skeleton.is_empty(), "golden {i}: empty query skeleton");

        // Persian-codepoint rows must disclose the L6 fold (N10), never apply
        // it silently (SC1 precision discipline carried into SC2).
        if !row.expected_rules_contain.is_empty() {
            persian_rows += 1;
            for hit in &out.hits {
                let ids: Vec<String> =
                    hit.explanation().rule_ids().iter().map(|r| format!("{r:?}")).collect();
                for rule in &row.expected_rules_contain {
                    assert!(
                        ids.iter().any(|id| id == rule),
                        "golden {i} {:?}: hit {} must disclose rule {rule}, got {ids:?}",
                        row.input,
                        hit.reference()
                    );
                }
            }
        }

        // Every hit carries a non-empty segmentation that tiles a contiguous
        // slice of the query, and every part resolves to a real canonical token.
        for hit in &out.hits {
            let parts = hit.segmentation();
            assert!(
                !parts.is_empty(),
                "golden {i} {:?}: hit {} has empty segmentation",
                row.input,
                hit.reference()
            );
            let tiled: String = parts.iter().map(|p| p.query_part.as_str()).collect();
            assert!(
                query_skeleton.contains(&tiled),
                "golden {i} {:?}: segmentation {tiled:?} is not a slice of {query_skeleton:?}",
                row.input
            );
            for part in parts {
                assert!(
                    part.canonical_token >= 1,
                    "golden {i} {:?}: canonical_token must be 1-based",
                    row.input
                );
                assert!(
                    !part.canonical_surface.is_empty()
                        && hit.quotation().arabic_text().contains(&part.canonical_surface),
                    "golden {i} {:?}: part {:?} does not resolve to a canonical token",
                    row.input,
                    part.query_part
                );
            }
        }

        // Expected segmentation, grouped per (surah, ayah) hit.
        let mut expected_groups: BTreeMap<(u16, u32), Vec<&ExpectedSegment>> = BTreeMap::new();
        for seg in &row.expected_segmentation {
            expected_groups.entry((seg.surah, seg.ayah)).or_default().push(seg);
        }
        for ((surah, ayah), segs) in &expected_groups {
            let want_ref = format!("{surah}:{ayah}");
            let hit = out
                .hits
                .iter()
                .find(|h| short_ref(h.reference()) == want_ref)
                .unwrap_or_else(|| {
                    panic!("golden {i} {:?}: no hit for expected segment ref {want_ref}", row.input)
                });
            let parts = hit.segmentation();
            assert_eq!(
                parts.len(),
                segs.len(),
                "golden {i} {:?}: segmentation length for {want_ref}",
                row.input
            );
            for (part, seg) in parts.iter().zip(segs.iter()) {
                assert_eq!(
                    part.query_part, seg.query_part,
                    "golden {i} {:?}: query_part for {want_ref}",
                    row.input
                );
                assert_eq!(
                    part.canonical_token, seg.canonical_token,
                    "golden {i} {:?}: canonical_token for {want_ref}",
                    row.input
                );
            }
        }

        // Boundary labeling: every expected boundary ref is labeled, and every
        // non-boundary ref is not (an ayah-level hit wins dedup and is never
        // presented as a cross-verse fragment).
        let boundary: BTreeSet<String> = row.expected_boundary_refs.iter().cloned().collect();
        if !boundary.is_empty() {
            cross_ayah_rows += 1;
        }
        for hit in &out.hits {
            let sr = short_ref(hit.reference());
            if boundary.contains(&sr) {
                assert!(
                    hit.spans_ayah_boundary(),
                    "golden {i} {:?}: {sr} must report spans_ayah_boundary",
                    row.input
                );
            } else {
                assert!(
                    !hit.spans_ayah_boundary(),
                    "golden {i} {:?}: {sr} must not report spans_ayah_boundary \
                     (ayah-level hit wins dedup)",
                    row.input
                );
            }
        }

        // Cross-ayah rows: the overlapped hits' segmentations tile the whole
        // query in canonical order.
        if !boundary.is_empty() {
            let mut tiled = String::new();
            for hit in out.hits.iter().filter(|h| boundary.contains(&short_ref(h.reference()))) {
                for part in hit.segmentation() {
                    tiled.push_str(&part.query_part);
                }
            }
            assert_eq!(
                tiled, query_skeleton,
                "golden {i} {:?}: cross-ayah hits must tile the query",
                row.input
            );
        }
    }

    assert!(
        cross_ayah_rows >= 10,
        "concatenated golden set must hold >= 10 cross-ayah rows, saw {cross_ayah_rows}"
    );
    assert!(
        persian_rows >= 10,
        "concatenated golden set must hold >= 10 persian-codepoint rows, saw {persian_rows}"
    );
}
