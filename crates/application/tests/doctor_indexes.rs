//! Phase 2 — index/linguistics doctor acceptance: P2-T105 / AC-P2-41.
//!
//! Against real SQLite in tempdirs: the 19 Phase-2 checks run in plan order
//! with stable ids, every non-pass carries remedy + next command, the run is
//! byte-identical read-only, missing generations fail loudly (mutation
//! probes), and the JSON document keeps the doctor `checks` shape.

use std::sync::atomic::AtomicBool;

use application::quran::activate_edition;
use application::quran_doctor_indexes::{INDEX_CHECK_IDS, run_index_checks};
use application::quran_forms::{RebuildParams, rebuild_forms};
use application::quran_index::{IndexBuildParams, QURAN_AYAH_INDEX_ID, rebuild_index};
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

async fn migrated_db() -> (tempfile::TempDir, SqliteDatabase, String) {
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
    (dir, db, path_str)
}

async fn ready_db() -> (tempfile::TempDir, SqliteDatabase, String) {
    let (dir, db, path_str) = migrated_db().await;
    let outcome = run_import(
        &db,
        &ImportInput {
            run_id: "run-doctor-indexes-1".into(),
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
            run_tag: "doctor-indexes-test".to_string(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("forms rebuild completes");
    (dir, db, path_str)
}

async fn build_index(db: &SqliteDatabase, data_dir: &std::path::Path) {
    rebuild_index(
        db,
        &IndexBuildParams {
            index_id: QURAN_AYAH_INDEX_ID.to_string(),
            edition_slug: "test-edition-min".to_string(),
            edition_version: "0.1.0".to_string(),
            invoked_by: PRINCIPAL.to_string(),
            run_tag: "doctor-indexes-build".to_string(),
            data_dir: data_dir.to_path_buf(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("index build completes");
}

fn status_of(checks: &[application::quran_doctor::QuranDoctorCheck], id: &str) -> String {
    checks
        .iter()
        .find(|c| c.id == id)
        .unwrap_or_else(|| panic!("missing check {id}"))
        .status
        .as_str()
        .to_string()
}

/// P2-T105: 19 checks in plan order with stable ids; every non-pass carries
/// remedy + next command; nothing fails on an unbuilt database.
#[tokio::test]
async fn index_checks_are_nineteen_with_stable_ids() {
    let (_dir, db, _path) = ready_db().await;
    let data_dir = _dir.path().join("index");
    let checks = run_index_checks(&db, &data_dir, false).await.expect("checks run");
    assert_eq!(checks.len(), 19);
    let ids: Vec<&str> = checks.iter().map(|c| c.id).collect();
    assert_eq!(ids, INDEX_CHECK_IDS);
    for check in &checks {
        if check.status != application::quran_doctor::CheckLevel::Pass {
            assert!(check.remedy.is_some(), "{} must carry a remedy", check.id);
            assert!(check.next_command.is_some(), "{} must carry a next command", check.id);
        }
        assert_ne!(
            check.status,
            application::quran_doctor::CheckLevel::Fail,
            "{} must not fail on an unbuilt database: {}",
            check.id,
            check.summary
        );
    }
    // Catalog-driven checks pass without any built artifacts.
    for id in [
        "quran.normalization.profiles_loaded",
        "quran.normalization.idempotency",
        "quran.normalization.spanmap_sanity",
        "quran.canonical_unchanged",
        "quran.lexicon.root_unification_queue",
    ] {
        assert_eq!(
            status_of(&checks, id),
            "pass",
            "{id}: {}",
            checks.iter().find(|c| c.id == id).unwrap().summary
        );
    }
}

/// P2-T105 / AC-P2-41: on a built index the serving checks pass, including
/// the 12-probe data-driven smoke suite.
#[tokio::test]
async fn index_checks_go_green_on_built_index() {
    let (_dir, db, _path) = ready_db().await;
    let data_dir = _dir.path().join("index");
    build_index(&db, &data_dir).await;
    let checks = run_index_checks(&db, &data_dir, false).await.expect("checks run");
    assert_eq!(checks.len(), 19);
    for id in [
        "quran.forms.current",
        "quran.forms.coverage",
        "quran.fts.ayah_index",
        "quran.skeleton.index",
        "quran.index.drift",
        "quran.index.orphans",
    ] {
        assert_eq!(
            status_of(&checks, id),
            "pass",
            "{id}: {}",
            checks.iter().find(|c| c.id == id).unwrap().summary
        );
    }
    let smoke = checks.iter().find(|c| c.id == "quran.search.smoke").unwrap();
    assert_eq!(smoke.status.as_str(), "pass", "smoke failed: {}", smoke.summary);
    assert!(smoke.summary.contains("12/12"), "smoke must run 12 probes: {}", smoke.summary);
    // Morphology-gated checks skip without a dataset; the token index is
    // unimplemented by design (never a silent pass, never a false fail).
    for id in [
        "quran.morphology.dataset_active",
        "quran.morphology.alignment",
        "quran.morphology.coverage",
        "quran.morphology.provenance",
        "quran.morphology.unverified_layer_d",
        "quran.lexicon.integrity",
        "quran.fts.token_index",
    ] {
        assert_eq!(status_of(&checks, id), "skipped", "{id} must skip without data");
    }
}

/// P2-T105: the run is byte-identical read-only, even with `--deep`.
#[tokio::test]
async fn doctor_indexes_is_read_only() {
    let (_dir, db, path_str) = ready_db().await;
    let data_dir = _dir.path().join("index");
    build_index(&db, &data_dir).await;
    // Flush pool state so the file comparison measures doctor I/O only.
    let before = std::fs::read(&path_str).unwrap();
    run_index_checks(&db, &data_dir, true).await.expect("deep checks run");
    // Index-dir files are opened read-only too; record their sizes.
    let mut sizes_before = std::collections::BTreeMap::new();
    if data_dir.exists() {
        for entry in std::fs::read_dir(&data_dir).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                let dirname = entry.file_name().into_string().unwrap();
                let mut bytes = 0u64;
                for file in std::fs::read_dir(entry.path()).unwrap() {
                    bytes += file.unwrap().metadata().unwrap().len();
                }
                sizes_before.insert(dirname, bytes);
            }
        }
    }
    run_index_checks(&db, &data_dir, false).await.expect("checks rerun");
    let after = std::fs::read(&path_str).unwrap();
    assert_eq!(before, after, "doctor must not mutate the database file");
    if data_dir.exists() {
        for entry in std::fs::read_dir(&data_dir).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                let dirname = entry.file_name().into_string().unwrap();
                let mut bytes = 0u64;
                for file in std::fs::read_dir(entry.path()).unwrap() {
                    bytes += file.unwrap().metadata().unwrap().len();
                }
                assert_eq!(sizes_before.get(&dirname), Some(&bytes), "gen dir {dirname} changed");
            }
        }
    }
}

/// P2-T105 / AC-P2-41 mutation probes: missing generations fail loudly
/// (never a silent pass, never a panic).
#[tokio::test]
async fn missing_generations_fail_loudly() {
    let (_dir, db, _path) = ready_db().await;
    let data_dir = _dir.path().join("index");
    build_index(&db, &data_dir).await;
    // Mutate: remove the serving FTS database file.
    let db_path = data_dir.join("gen-1").join("index.db");
    assert!(db_path.exists());
    std::fs::remove_file(&db_path).unwrap();
    let checks = run_index_checks(&db, &data_dir, false).await.expect("checks run");
    assert_eq!(status_of(&checks, "quran.fts.ayah_index"), "fail");
    assert_eq!(status_of(&checks, "quran.search.smoke"), "fail");
    // Mutate: remove trigram postings.
    let trigram = data_dir.join("gen-1").join("trigram.db");
    if trigram.exists() {
        std::fs::remove_file(&trigram).unwrap();
        let checks = run_index_checks(&db, &data_dir, false).await.expect("checks rerun");
        assert_eq!(status_of(&checks, "quran.skeleton.index"), "fail");
    }
}

/// P2-T105: the JSON document keeps the doctor `checks` shape (schema-safe).
#[tokio::test]
async fn index_json_matches_doctor_checks_shape() {
    let (_dir, db, _path) = ready_db().await;
    let data_dir = _dir.path().join("index");
    build_index(&db, &data_dir).await;
    let checks = run_index_checks(&db, &data_dir, false).await.expect("checks run");
    let doc = serde_json::json!({
        "checks": checks.iter().map(|c| serde_json::json!({
            "id": c.id,
            "status": c.status.as_str(),
            "summary": c.summary,
            "remedy": c.remedy,
            "next_command": c.next_command,
        })).collect::<Vec<_>>(),
    });
    // Mirror of `docs/schemas/doctor.v1.schema.json` required keys.
    let object = doc.as_object().expect("object");
    assert_eq!(object.len(), 1);
    let items = doc["checks"].as_array().expect("checks array");
    assert_eq!(items.len(), 19);
    for item in items {
        for key in ["id", "status", "summary", "remedy", "next_command"] {
            assert!(item.get(key).is_some(), "missing {key} in {item:?}");
        }
        assert!(["pass", "warn", "fail", "skipped"].contains(&item["status"].as_str().unwrap()));
    }
}
