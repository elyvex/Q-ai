//! Phase 3 (03-04) — SC3 word-family golden suite (G-05 / P2-T92 mechanics).
//!
//! Runs `fixtures/quran/lexicon/families/curated.jsonl` (>= 120 curated
//! families) against the real family builders on the synthetic `test-morph`
//! lexicon: every curated row must resolve through `word_family` to its typed
//! relation with a non-empty explanation, and the round must cover every
//! relation kind the builders emit.
//!
//! Mechanics on synthetic data — the set is not scholarly ground truth, every
//! row is `synthetic_test_only`, and the header must read
//! `reviewed_by: pending-linguist` until OD-12 closes.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::AtomicBool;

use application::quran::activate_edition;
use application::quran_forms::{RebuildParams, rebuild_forms};
use application::quran_morphology::{
    MorphologyActivateParams, MorphologyImportParams, activate_morphology, build_affix_relations,
    build_derived_relations, build_inflectional_relations, build_same_form_relations,
    build_same_lemma_relations, build_same_root_relations, build_same_stem_relations, dataset_urn,
    run_morphology_import, word_family,
};
use domain::{PrincipalId, Timestamp};
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use quran_corpus::sha256_hex;
use storage::Database as _;
use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

const GOLDENS: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/quran/lexicon/families/curated.jsonl");
const BASE_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const PRINCIPAL: &str = "00000000-0000-0000-0000-000000000001";
const CREATED_AT: &str = "2026-09-14T00:00:00Z";
const V1_URN: &str = "quran-edition:test-edition-min@0.1.0";
const SLUG: &str = "test-morph";
const VERSION: &str = "0.1.0";

const TYPED_KINDS: [&str; 7] =
    ["same_form", "same_lemma", "same_stem", "same_root", "derived", "inflectional", "affix"];

#[derive(serde::Deserialize)]
struct FamilyGoldenRow {
    family_id: String,
    kind: String,
    from: String,
    to: String,
    expected_relation: String,
    synthetic_test_only: bool,
    reviewed_by: String,
    reviewed_at: Option<String>,
}

fn principal() -> PrincipalId {
    PRINCIPAL.parse().unwrap()
}

fn timestamp() -> Timestamp {
    Timestamp::from_ymd_hms(2026, 9, 14, 0, 0, 0).unwrap()
}

/// The curated family oracle: header + rows, with the synthetic/pending
/// labeling pinned (Pitfall 4 / T-03-15).
fn load_goldens() -> Vec<FamilyGoldenRow> {
    let text = std::fs::read_to_string(GOLDENS).expect("family golden fixture exists");
    let mut lines = text.lines();
    let header: serde_json::Value =
        serde_json::from_str(lines.next().expect("header present")).expect("header parses");
    assert_eq!(header["header"], true);
    assert_eq!(header["corpus"], "fixtures/quran/test-edition-min");
    assert_eq!(header["reviewed_by"], "pending-linguist");
    assert!(header["reviewed_at"].is_null(), "golden set stays unreviewed (OD-12)");
    assert_eq!(header["synthetic_test_only"], true);
    let rows: Vec<FamilyGoldenRow> = lines
        .enumerate()
        .map(|(i, line)| {
            serde_json::from_str(line).unwrap_or_else(|e| panic!("family golden line {i}: {e}"))
        })
        .collect();
    assert!(rows.len() >= 120, "family golden set must hold >= 120 rows, got {}", rows.len());
    for row in &rows {
        assert!(row.synthetic_test_only, "{} must stay synthetic_test_only", row.family_id);
        assert_eq!(
            row.reviewed_by, "pending-linguist",
            "{} must stay pending-linguist",
            row.family_id
        );
        assert!(row.reviewed_at.is_none(), "{} must stay unreviewed (OD-12)", row.family_id);
        assert_eq!(row.kind, row.expected_relation, "{}: kind/relation disagree", row.family_id);
        assert!(TYPED_KINDS.contains(&row.expected_relation.as_str()), "{}", row.family_id);
    }
    rows
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

/// Migrated db + imported/activated synthetic edition + rebuilt forms.
async fn ready_db() -> (tempfile::TempDir, SqliteDatabase) {
    let (dir, db) = migrated_db().await;
    let outcome = run_import(
        &db,
        &ImportInput {
            run_id: "run-family-goldens".into(),
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
            run_tag: "family-goldens".to_string(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("forms rebuild completes");
    (dir, db)
}

/// Synchronous synthetic dictionary keys for the token at canonical index `k`.
///
/// Kept in lockstep with `crates/application/tests/morphology_import.rs`:
/// the curated fixture is computed from this same scheme, so a drift here
/// shows up as a missing curated relation, not a silently weaker assertion.
/// `lemma = k % 9` / `root = k % 3` keep a lemma mapped to exactly one root
/// (`quran_lemmas` is UNIQUE(dataset_id, lemma)).
fn family_keys(index: usize) -> (String, String, String, String, String) {
    (
        format!("root-{}", index % 3),
        format!("lem-{}", index % 9),
        format!("stem-{}", index % 5),
        format!("pre-{}", index % 4),
        format!("suf-{}", index % 3),
    )
}

/// Build an aligned array-shape document whose derived lexicon keys exercise
/// every family relation kind (G-05). Real fixture surfaces keep direct-key
/// alignment; every row stays `synthetic_test_only: true`.
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
            let (root, lemma, stem, prefix, suffix) = family_keys(index);
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

fn import_params(document_text: String) -> MorphologyImportParams {
    MorphologyImportParams {
        dataset_slug: SLUG.to_string(),
        dataset_version: VERSION.to_string(),
        adapter: "json".to_string(),
        document_text,
        edition_slug: "test-edition-min".to_string(),
        edition_version: "0.1.0".to_string(),
        invoked_by: PRINCIPAL.to_string(),
        batch_id: Some("batch-family-goldens".to_string()),
        attribution: "synthetic test import (not scholarly data)".to_string(),
        license_status: "PublicDomain".to_string(),
        // Gate 6 (D-07): mandatory capture fields + redistribution_allowed.
        license_json: r#"{"source_url":"https://example.invalid/qai-synthetic-test-lexicon","capture_date":"2026-09-28","capturer":"qai-test-fixtures","spdx_id":"CC0-1.0","redistribution_allowed":true}"#.to_string(),
    }
}

/// Import + activate the synthetic lexicon and build every relation kind,
/// returning the total row count.
async fn build_all_relations(db: &SqliteDatabase) -> usize {
    let doc = family_document(db).await;
    run_morphology_import(db, &import_params(doc), &AtomicBool::new(false), |_| {})
        .await
        .expect("family import completes");
    activate_morphology(
        db,
        &MorphologyActivateParams {
            batch_id: "batch-family-goldens".to_string(),
            approval_id: "appr-morph".to_string(),
            invoked_by: PRINCIPAL.to_string(),
        },
        &principal(),
    )
    .await
    .expect("family activation completes");

    let dataset = format!("{SLUG}@{VERSION}");
    let mut total = 0usize;
    for built in [
        build_same_form_relations(db, &dataset).await.expect("same_form"),
        build_same_lemma_relations(db, &dataset).await.expect("same_lemma"),
        build_same_stem_relations(db, &dataset).await.expect("same_stem"),
        build_same_root_relations(db, &dataset).await.expect("same_root"),
        build_derived_relations(db, &dataset).await.expect("derived"),
        build_inflectional_relations(db, &dataset).await.expect("inflectional"),
        build_affix_relations(db, &dataset).await.expect("affix"),
    ] {
        assert!(built > 0, "every relation kind must build rows");
        total += built;
    }
    total
}

/// Every curated family resolves to its typed relation with a non-empty
/// explanation, and the built set covers every relation kind the builders
/// emit. Selected by `-- family_goldens` (a zero-match filter exits 0).
#[tokio::test]
async fn all_curated_family_goldens_pass() {
    let rows = load_goldens();
    let (_dir, db) = ready_db().await;
    let built_total = build_all_relations(&db).await;
    assert!(
        built_total >= rows.len(),
        "built {built_total} rows must cover {} families",
        rows.len()
    );

    let mut cache: BTreeMap<String, Vec<application::quran_morphology::FamilyMemberView>> =
        BTreeMap::new();
    let mut kinds_seen: BTreeSet<String> = BTreeSet::new();
    for row in &rows {
        if !cache.contains_key(&row.from) {
            let (dataset, members) =
                word_family(&db, "token", &row.from).await.unwrap_or_else(|e| {
                    panic!("{}: word_family({}) failed: {e}", row.family_id, row.from)
                });
            assert_eq!(dataset, format!("{SLUG}@{VERSION}"));
            cache.insert(row.from.clone(), members);
        }
        let members = cache.get(&row.from).expect("cached members");
        let partner = members
            .iter()
            .find(|m| m.id == row.to && m.relation == row.expected_relation)
            .unwrap_or_else(|| {
                let seen: Vec<(String, String)> =
                    members.iter().map(|m| (m.id.clone(), m.relation.clone())).collect();
                panic!(
                    "{}: {} must resolve to {} under {} — saw {seen:?}",
                    row.family_id, row.from, row.to, row.expected_relation
                )
            });
        assert!(
            !partner.explanation.trim().is_empty(),
            "{}: explanation is mandatory",
            row.family_id
        );
        assert_eq!(partner.kind, "token");
        kinds_seen.insert(row.expected_relation.clone());
    }

    let expected: BTreeSet<String> = TYPED_KINDS.iter().map(|k| (*k).to_string()).collect();
    assert_eq!(kinds_seen, expected, "curated set must cover every relation kind");
}

// ── Root/lemma golden set (P2-T91, D-3.5-08) ────────────────────────────────

const ROOT_LEMMA_GOLDENS: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/quran/lexicon/root-lemma-goldens.jsonl");

#[derive(serde::Deserialize)]
struct RootLemmaGoldenRow {
    case_id: String,
    tool: String,
    input: String,
    expected_attribution: String,
    expected_typed_unavailable: bool,
    synthetic_test_only: bool,
    reviewed_by: String,
    reviewed_at: Option<String>,
}

fn load_root_lemma_goldens() -> Vec<RootLemmaGoldenRow> {
    let text =
        std::fs::read_to_string(ROOT_LEMMA_GOLDENS).expect("root/lemma golden fixture exists");
    let mut lines = text.lines();
    let header: serde_json::Value =
        serde_json::from_str(lines.next().expect("header present")).expect("header parses");
    assert_eq!(header["header"], true);
    assert_eq!(header["reviewed_by"], "pending-linguist");
    assert!(header["reviewed_at"].is_null(), "root/lemma goldens stay unreviewed (OD-12)");
    assert_eq!(header["synthetic_test_only"], true);
    let rows: Vec<RootLemmaGoldenRow> = lines
        .enumerate()
        .map(|(i, line)| {
            serde_json::from_str(line).unwrap_or_else(|e| panic!("root/lemma golden line {i}: {e}"))
        })
        .collect();
    assert_eq!(rows.len(), 500, "root/lemma golden set must hold exactly 500 cases");
    for row in &rows {
        assert!(row.synthetic_test_only, "{} must stay synthetic_test_only", row.case_id);
        assert_eq!(
            row.reviewed_by, "pending-linguist",
            "{} must stay pending-linguist",
            row.case_id
        );
        assert!(row.reviewed_at.is_none(), "{} must stay unreviewed (OD-12)", row.case_id);
        // Every row resolves against the active synthetic dataset, so none may
        // expect typed-unavailable here; the QAI-MORPH-0004 path is covered by
        // the tool-registry conformance suite (unavailable_dataset case) instead.
        assert!(
            !row.expected_typed_unavailable,
            "{} must resolve, not expect unavailable",
            row.case_id
        );
    }
    rows
}

/// Every root/lemma golden row resolves through the real service. Selected by
/// `-- root_lemma_goldens` (a zero-match filter exits 0).
#[tokio::test]
async fn all_root_lemma_goldens_pass() {
    let rows = load_root_lemma_goldens();
    let (_dir, db) = ready_db().await;
    let built_total = build_all_relations(&db).await;
    assert!(built_total > 0, "family relations must be built");

    let dataset = format!("{SLUG}@{VERSION}");
    for row in &rows {
        assert_eq!(
            row.expected_attribution, dataset,
            "{}: row pins its synthetic dataset attribution",
            row.case_id
        );
        match row.tool.as_str() {
            "root_search" => {
                let (ds, occurrences) = application::quran_morphology::root_search(&db, &row.input)
                    .await
                    .unwrap_or_else(|e| {
                        panic!("{}: root_search({}) failed: {e}", row.case_id, row.input)
                    });
                assert_eq!(ds, dataset, "{}: dataset attribution", row.case_id);
                assert!(!occurrences.is_empty(), "{}: root must have occurrences", row.case_id);
            }
            "lemma_search" => {
                let (ds, occurrences) =
                    application::quran_morphology::lemma_search(&db, &row.input)
                        .await
                        .unwrap_or_else(|e| {
                            panic!("{}: lemma_search({}) failed: {e}", row.case_id, row.input)
                        });
                assert_eq!(ds, dataset, "{}: dataset attribution", row.case_id);
                assert!(!occurrences.is_empty(), "{}: lemma must have occurrences", row.case_id);
            }
            "word_family" => {
                let (ds, members) =
                    word_family(&db, "token", &row.input).await.unwrap_or_else(|e| {
                        panic!("{}: word_family({}) failed: {e}", row.case_id, row.input)
                    });
                assert_eq!(ds, dataset, "{}: dataset attribution", row.case_id);
                assert!(!members.is_empty(), "{}: token must have family members", row.case_id);
            }
            "morphology_for_token" => {
                let parts: Vec<&str> = row.input.split(':').collect();
                assert_eq!(parts.len(), 4, "{}: morphology input must be token:S:A:P", row.case_id);
                assert_eq!(
                    parts[0], "token",
                    "{}: morphology input must name a token",
                    row.case_id
                );
                let surah: i64 = parts[1].parse().unwrap();
                let ayah: i64 = parts[2].parse().unwrap();
                let position: i64 = parts[3].parse().unwrap();
                let (ds, analyses) = application::quran_morphology::morphology_for_token(
                    &db,
                    "test-edition-min",
                    "0.1.0",
                    surah,
                    ayah,
                    position,
                )
                .await
                .unwrap_or_else(|e| {
                    panic!("{}: morphology_for_token({}) failed: {e}", row.case_id, row.input)
                });
                assert_eq!(ds, dataset, "{}: dataset attribution", row.case_id);
                assert!(!analyses.is_empty(), "{}: token must have analyses", row.case_id);
            }
            other => panic!("{}: unknown tool {other}", row.case_id),
        }
    }
}
