//! Phase 4 graph doctor acceptance (plan 04-05, D-08/D-11).
//!
//! Against real SQLite in tempdirs with a real imported + activated edition
//! plus a structural build: manifest mismatch surfaces as Warn (never Fail)
//! after a generation bump, dangling edges Fail, tombstoned ids stay retained
//! in authority yet invisible in traversal results and export bytes, a fresh
//! database reports Skipped with the build remedy, and doctor runs leave
//! database state untouched. Repair cases (quarantine, retention-gated GC,
//! authority-preserving rebuild, audit emission, confirmation gate) live in
//! this same file under `repair_*` names.

mod common;

use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;

use application::quran::activate_edition;
use application::quran_doctor::CheckLevel;
use application::quran_graph_annotations::{DecideInput, ProposeInput, propose, reject};
use application::quran_graph_build::{collect_structural_input, publish_structural_build};
use application::quran_graph_doctor::{GRAPH_CHECK_IDS, run_quran_graph_checks};
use application::quran_graph_store::SqliteGraphStore;
use quran_corpus::import::{ImportInput, ImportOptions, ImportProgress, run_import};
use quran_graph::{AssertionKind, AuthzScope, EdgeFilter, GraphStore, QueryBudgets};

const PROJECTION: &str = quran_graph::STRUCTURAL_PROJECTION_ID;
const EDITION: &str = "test-edition-min@0.1.0";
const REVIEWER: &str = "pending-scholar";
const OPERATOR: &str = common::PRINCIPAL;

fn principal() -> domain::PrincipalId {
    OPERATOR.parse().unwrap()
}

fn timestamp() -> domain::Timestamp {
    domain::Timestamp::from_ymd_hms(2026, 9, 14, 0, 0, 0).unwrap()
}

async fn built_db() -> (tempfile::TempDir, String) {
    let (dir, db, _reader, path_str) = common::active_reader().await;
    let collected = collect_structural_input(&db)
        .await
        .expect("collect from active edition")
        .expect("the common harness activates test-edition-min, so input collection succeeds");
    publish_structural_build(&path_str, &collected).await.expect("structural build publishes");
    (dir, path_str)
}

async fn open_ro(path: &str) -> storage_sqlite::SqliteDatabase {
    storage_sqlite::SqliteDatabase::open_read_only(path).await.expect("read-only open works")
}

async fn checks(path: &str, deep: bool) -> Vec<application::quran_doctor::QuranDoctorCheck> {
    let db = open_ro(path).await;
    run_quran_graph_checks(&db, deep).await.expect("doctor runs")
}

fn status_of(
    checks: &[application::quran_doctor::QuranDoctorCheck],
    id: &str,
) -> (CheckLevel, String) {
    let check = checks.iter().find(|check| check.id == id).expect("check id present");
    (check.status, check.summary.clone())
}

fn propose_input(id: &str) -> ProposeInput {
    ProposeInput {
        id: Some(id.to_string()),
        kind: AssertionKind::Annotation,
        src: "ayah:1:1".to_string(),
        edge: "PARALLELS".to_string(),
        dst: "ayah:1:2".to_string(),
        evidence: serde_json::json!({"source": "graph-doctor-test"}),
        source_id: "graph-doctor-test".to_string(),
        source_location: "graph_doctor.rs".to_string(),
        author: REVIEWER.to_string(),
        invoked_by: OPERATOR.to_string(),
        projection_id: PROJECTION.to_string(),
        edition_id: EDITION.to_string(),
        dataset_scope: String::new(),
    }
}

fn decide(id: &str) -> DecideInput {
    DecideInput {
        id: id.to_string(),
        reviewer: REVIEWER.to_string(),
        decided_at: None,
        invoked_by: OPERATOR.to_string(),
    }
}

async fn pool(path: &str) -> sqlx::sqlite::SqlitePool {
    sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(path).foreign_keys(true))
        .await
        .expect("pool opens")
}

async fn active_build_row(path: &str, family: &str) -> String {
    let pool = pool(path).await;
    let row: Option<String> = sqlx::query_scalar(
        "SELECT id FROM graph_projections
         WHERE projection_id = ? AND status = 'active'
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(family)
    .fetch_optional(&pool)
    .await
    .expect("build row read");
    pool.close().await;
    row.expect("active build row exists")
}

#[tokio::test]
async fn doctor_manifest_current_passes_on_fresh_build() {
    let (_dir, path) = built_db().await;
    let found = checks(&path, false).await;
    assert_eq!(
        found.iter().map(|check| check.id).collect::<Vec<_>>(),
        GRAPH_CHECK_IDS,
        "checks run in stable id order so runs diff cleanly"
    );
    let (status, summary) = status_of(&found, "quran.graph.structural.current");
    assert_eq!(status, CheckLevel::Pass, "fresh build is current: {summary}");
    let (status, _) = status_of(&found, "quran.graph.dangling_edges");
    assert_eq!(status, CheckLevel::Pass, "fenced build stages zero dangling edges");
    let (status, summary) = status_of(&found, "quran.graph.tombstone_invisibility");
    assert_eq!(status, CheckLevel::Pass, "no tombstones yet: {summary}");
    let (status, _) = status_of(&found, "quran.graph.dependency_snapshot");
    assert_eq!(status, CheckLevel::Pass, "builder pins a complete snapshot");
    for id in ["quran.graph.wordroot.current", "quran.graph.annotated.current"] {
        let (status, summary) = status_of(&found, id);
        assert_eq!(status, CheckLevel::Skipped, "{id} is optional: {summary}");
        let check = found.iter().find(|check| check.id == id).unwrap();
        assert!(!check.remedy.as_deref().unwrap_or_default().is_empty(), "{id} names a remedy");
    }
    for check in &found {
        assert!(!check.summary.trim().is_empty(), "{} carries a summary", check.id);
    }
}

#[tokio::test]
async fn doctor_manifest_mismatch_warns_after_generation_bump() {
    let (_dir, path) = built_db().await;
    // Inject drift the honest way: import + activate a v2 edition (new
    // edition id + corpus generation) while the projection still pins v1.
    let db = storage_sqlite::SqliteDatabase::new(&path, 4, true).await.expect("db opens");
    let pool = pool(&path).await;
    sqlx::query(
        "INSERT INTO approvals
            (id, subject_urn, kind, requested_by, decided_by, decision,
             request_payload, requested_at, decided_at)
         VALUES ('appr-v2', 'quran-edition:test-edition-min@0.2.0', 'CanonicalChange',
                 '00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001',
                 'approved', '{}', '2026-09-14T00:00:00Z', '2026-09-14T00:00:00Z')",
    )
    .execute(&pool)
    .await
    .expect("v2 approval seeds");
    pool.close().await;
    let mut v2: serde_json::Value =
        serde_json::from_str(common::BASE_MANIFEST).expect("fixture parses");
    v2["edition"]["version"] = serde_json::json!("0.2.0");
    let v2_text = serde_json::to_string(&v2).unwrap();
    run_import(
        &db,
        &ImportInput {
            run_id: "run-graph-doctor-2".into(),
            job_id: None,
            source_version_id: common::SOURCE_VERSION_ID.into(),
            adapter: "json".into(),
            manifest_text: v2_text,
            declared_manifest_hash: None,
            invoked_by: OPERATOR.into(),
            license_status: "PublicDomain".into(),
            license_json: "{}".into(),
            created_at: common::CREATED_AT.into(),
            reference_manifest_text: None,
        },
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .expect("v2 import completes");
    activate_edition(&db, "test-edition-min", "0.2.0", &principal(), "appr-v2", &timestamp())
        .await
        .expect("v2 activation completes");
    drop(db);

    let found = checks(&path, false).await;
    let (status, summary) = status_of(&found, "quran.graph.structural.current");
    assert_eq!(status, CheckLevel::Warn, "stale-but-servable drift is Warn, never Fail: {summary}");
    assert!(summary.contains("stale-but-servable"), "drift says stale-but-servable: {summary}");
    let check = found.iter().find(|c| c.id == "quran.graph.structural.current").unwrap();
    assert!(
        check.next_command.as_deref() == Some("qai quran graph build"),
        "drift remedy names the rebuild command"
    );
    // Drift is severity-honest: nothing fails just because it is stale.
    assert!(
        !found.iter().any(|c| c.status == CheckLevel::Fail),
        "staleness alone never Fails: {:?}",
        found.iter().map(|c| (c.id, c.status)).collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn doctor_dangling_edges_fail_on_corrupt_specimen() {
    let (_dir, path) = built_db().await;
    let row_id = active_build_row(&path, PROJECTION).await;
    // A corrupt specimen: one edge row whose destination was never staged.
    // Builders reject these, so the detector test inserts the specimen
    // directly — one dangling endpoint fails the edge once.
    let pool = pool(&path).await;
    sqlx::query(
        "INSERT INTO graph_edges
            (id, projection_row_id, src_stable_id, edge, dst_stable_id,
             assertion_id, budgets_json, attrs_json, created_at)
         VALUES ('specimen-dangling-1', ?, 'ayah:1:1', 'NEXT', 'ayah:999:999',
                 NULL, '{}', '{}', '2026-09-29T00:00:00Z')",
    )
    .bind(&row_id)
    .execute(&pool)
    .await
    .expect("specimen inserts");
    pool.close().await;

    let found = checks(&path, true).await;
    let (status, summary) = status_of(&found, "quran.graph.dangling_edges");
    assert_eq!(status, CheckLevel::Fail, "dangling endpoints Fail: {summary}");
    assert!(summary.contains("ayah:999:999"), "failure names the dangling endpoint: {summary}");
    let check = found.iter().find(|c| c.id == "quran.graph.dangling_edges").unwrap();
    assert!(check.remedy.as_deref().unwrap_or_default().contains("quarantine"));
}

#[tokio::test]
async fn doctor_tombstone_probe_passes_with_rejected_assertion() {
    let (_dir, path) = built_db().await;
    let report = propose(&path, propose_input("dr-tomb-1")).await.expect("propose commits");
    assert!(report.edge_staged, "the probe needs a staged edge carrying the assertion");
    reject(&path, decide("dr-tomb-1")).await.expect("reject tombstones");

    let found = checks(&path, false).await;
    let (status, summary) = status_of(&found, "quran.graph.tombstone_invisibility");
    assert_eq!(status, CheckLevel::Pass, "tombstoned ids stay invisible: {summary}");
    assert!(summary.contains("1 tombstoned"), "probe counts the tombstone: {summary}");

    // Retained in authority …
    let kept = application::quran_graph_annotations::get_assertion(&path, "dr-tomb-1")
        .await
        .expect("authority retains the tombstoned row");
    assert_eq!(kept.decision, quran_graph::AssertionDecision::Rejected);
    // … yet in neither traversal results …
    let db = open_ro(&path).await;
    let store = SqliteGraphStore::open_active(&db, PROJECTION).await.expect("store opens");
    let view = store
        .neighbors(
            "ayah:1:1",
            &EdgeFilter::any(),
            &QueryBudgets::default(),
            &AtomicBool::new(false),
            &AuthzScope::all_visible(),
        )
        .expect("neighbors run");
    assert!(
        view.edges.iter().all(|edge| edge.assertion_id.as_deref() != Some("dr-tomb-1")),
        "rejected assertion invisible in traversal"
    );
    // … nor export bytes.
    let (nodes, edges, assertions) = store.export_sets();
    let (nodes, edges, assertions) =
        application::quran_graph_annotations::visible_export_sets(&nodes, &edges, &assertions);
    let document = quran_graph::export_json(&nodes, &edges, &assertions, store.manifest());
    let bytes = serde_json::to_string(&document).unwrap_or_default();
    assert!(!bytes.contains("dr-tomb-1"), "rejected assertion absent from export bytes");
}

#[tokio::test]
async fn doctor_empty_database_skips_with_build_remedy() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("qai.db");
    let path_str = path.to_str().unwrap().to_string();
    let repo_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    storage_sqlite::migrate::apply_migrations(&path_str, &repo_root).await.unwrap();

    let found = checks(&path_str, false).await;
    assert_eq!(found.len(), GRAPH_CHECK_IDS.len(), "fresh DB still reports every check");
    for check in &found {
        assert_eq!(
            check.status,
            CheckLevel::Skipped,
            "{} is Skipped on a fresh database, never Fail and never Pass",
            check.id
        );
        assert!(
            !check.remedy.as_deref().unwrap_or_default().is_empty(),
            "{} carries a remedy",
            check.id
        );
        assert!(
            !check.next_command.as_deref().unwrap_or_default().is_empty(),
            "{} names a next command",
            check.id
        );
    }
    let structural = found.iter().find(|c| c.id == "quran.graph.structural.current").unwrap();
    assert!(
        structural.next_command.as_deref() == Some("qai quran import --help")
            || structural.remedy.as_deref().unwrap_or_default().contains("build"),
        "fresh-database remedy names the build path: {:?}",
        structural
    );
}

fn table_counts(path: &str) -> BTreeMap<String, i64> {
    let path = path.to_string();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(async {
            let pool = pool(&path).await;
            // Force pending WAL frames into the main database file before
            // snapshotting: prior write paths drop (rather than close) their
            // pools, so a background last-close auto-checkpoint could
            // otherwise move bytes between the before/after reads. With the
            // WAL empty, later closes cannot touch the main file, and doctor
            // reads create no frames — so any remaining byte drift is a real
            // mutation.
            for _ in 0..10 {
                let row: (i32, i32, i32) = sqlx::query_as("PRAGMA wal_checkpoint(TRUNCATE)")
                    .fetch_one(&pool)
                    .await
                    .expect("checkpoint runs");
                if row.1 == 0 {
                    break;
                }
                tokio::task::yield_now().await;
            }
            let mut counts = BTreeMap::new();
            for table in [
                "graph_projections",
                "graph_nodes",
                "graph_edges",
                "graph_assertions",
                "graph_build_progress",
                "audit_events",
                "quran_active_edition",
            ] {
                let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
                    .fetch_one(&pool)
                    .await
                    .expect("count reads");
                counts.insert(table.to_string(), count);
            }
            let active: Option<(String, i64)> =
                sqlx::query_as("SELECT edition_id, corpus_generation FROM quran_active_edition")
                    .fetch_optional(&pool)
                    .await
                    .expect("pointer reads");
            counts.insert(
                "pointer-check".to_string(),
                active.map(|(_, generation)| generation).unwrap_or(-1),
            );
            pool.close().await;
            counts
        })
    })
    .join()
    .unwrap()
}

#[tokio::test]
async fn doctor_run_leaves_state_untouched() {
    let (_dir, path) = built_db().await;
    propose(&path, propose_input("dr-immutable-1")).await.expect("propose commits");
    reject(&path, decide("dr-immutable-1")).await.expect("reject commits");

    let before_counts = table_counts(&path);
    let before_bytes = std::fs::read(&path).expect("db file reads");
    // Shallow and deep runs alike must not mutate.
    for deep in [false, true] {
        let found = checks(&path, deep).await;
        assert_eq!(found.len(), GRAPH_CHECK_IDS.len());
    }
    let after_counts = table_counts(&path);
    let after_bytes = std::fs::read(&path).expect("db file reads");
    assert_eq!(before_counts, after_counts, "doctor mutates no rows");
    assert_eq!(before_bytes, after_bytes, "doctor leaves database bytes identical");
}
