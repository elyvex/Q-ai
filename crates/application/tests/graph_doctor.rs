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
use application::quran_graph_annotations::{
    DecideInput, ProposeInput, accept, dispute, propose, reject,
};
use application::quran_graph_build::{collect_structural_input, publish_structural_build};
use application::quran_graph_doctor::{
    GRAPH_CHECK_IDS, RebuildInput, RepairInput, TombstoneGcInput, repair_quarantine_dangling,
    repair_rebuild_projection, repair_tombstone_gc, run_quran_graph_checks,
};
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

// ─── Explicit confirmed repair (D-08/D-11, T-04-15) ───

fn repair_input(confirmed: bool) -> RepairInput {
    RepairInput { invoked_by: OPERATOR.to_string(), confirmed }
}

fn propose_at(id: &str, src: &str, edge: &str, dst: &str) -> ProposeInput {
    ProposeInput {
        id: Some(id.to_string()),
        kind: AssertionKind::Annotation,
        src: src.to_string(),
        edge: edge.to_string(),
        dst: dst.to_string(),
        // Distinct evidence per claim: identical triples under different ids
        // reuse the existing row (content-addressed conflict reuse), so each
        // repair specimen carries its own evidence.
        evidence: serde_json::json!({"source": "graph-doctor-repair-test", "id": id}),
        source_id: "graph-doctor-repair-test".to_string(),
        source_location: format!("graph_doctor.rs#{id}"),
        author: REVIEWER.to_string(),
        invoked_by: OPERATOR.to_string(),
        projection_id: PROJECTION.to_string(),
        edition_id: EDITION.to_string(),
        dataset_scope: String::new(),
    }
}

async fn repair_audit_count(path: &str, operation: &str) -> i64 {
    let pool = pool(path).await;
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_events
         WHERE action = 'doctor_repair_executed' AND subject_urn = ?",
    )
    .bind(format!("quran-graph-repair:{operation}"))
    .fetch_one(&pool)
    .await
    .expect("audit reads");
    pool.close().await;
    count
}

async fn repair_audit_after(path: &str, operation: &str) -> String {
    let pool = pool(path).await;
    let after: String = sqlx::query_scalar(
        "SELECT after_json FROM audit_events
         WHERE action = 'doctor_repair_executed' AND subject_urn = ?
         ORDER BY sequence DESC LIMIT 1",
    )
    .bind(format!("quran-graph-repair:{operation}"))
    .fetch_one(&pool)
    .await
    .expect("audit reads");
    pool.close().await;
    after
}

async fn backdate_decision(path: &str, id: &str, decided_at: &str) {
    let pool = pool(path).await;
    sqlx::query("UPDATE graph_assertions SET decided_at = ? WHERE id = ?")
        .bind(decided_at)
        .bind(id)
        .execute(&pool)
        .await
        .expect("backdate writes");
    pool.close().await;
}

async fn insert_dangling_specimen(path: &str, id: &str) {
    let row_id = active_build_row(path, PROJECTION).await;
    let pool = pool(path).await;
    sqlx::query(
        "INSERT INTO graph_edges
            (id, projection_row_id, src_stable_id, edge, dst_stable_id,
             assertion_id, budgets_json, attrs_json, created_at)
         VALUES (?, ?, 'ayah:1:1', 'NEXT', 'ayah:999:999',
                 NULL, '{}', '{}', '2026-09-29T00:00:00Z')",
    )
    .bind(id)
    .bind(&row_id)
    .execute(&pool)
    .await
    .expect("specimen inserts");
    pool.close().await;
}

#[tokio::test]
async fn repair_refuses_without_confirmation() {
    let (_dir, path) = built_db().await;
    let before = table_counts(&path);

    let denied = repair_quarantine_dangling(&path, repair_input(false)).await;
    assert!(
        denied
            .expect_err("quarantine without confirmation refuses")
            .to_string()
            .contains("without explicit confirmation"),
        "refusal names the confirmation gate"
    );
    let denied = repair_tombstone_gc(
        &path,
        TombstoneGcInput { invoked_by: OPERATOR.to_string(), confirmed: false, retention_days: 90 },
    )
    .await;
    assert!(
        denied
            .expect_err("gc without confirmation refuses")
            .to_string()
            .contains("without explicit confirmation"),
        "refusal names the confirmation gate"
    );
    let denied = repair_rebuild_projection(
        &path,
        RebuildInput {
            invoked_by: OPERATOR.to_string(),
            confirmed: false,
            projection: PROJECTION.to_string(),
            seed_file: None,
        },
    )
    .await;
    assert!(
        denied
            .expect_err("rebuild without confirmation refuses")
            .to_string()
            .contains("without explicit confirmation"),
        "refusal names the confirmation gate"
    );

    assert_eq!(before, table_counts(&path), "refused repairs mutate nothing");
    assert_eq!(repair_audit_count(&path, "quarantine-dangling").await, 0, "refusal audits nothing");
}

#[tokio::test]
async fn repair_quarantine_removes_dangling_edges_and_doctor_turns_green() {
    let (_dir, path) = built_db().await;
    insert_dangling_specimen(&path, "specimen-repair-1").await;

    let found = checks(&path, true).await;
    let (status, _) = status_of(&found, "quran.graph.dangling_edges");
    assert_eq!(status, CheckLevel::Fail, "specimen fails before repair");

    let report =
        repair_quarantine_dangling(&path, repair_input(true)).await.expect("quarantine repairs");
    assert_eq!(report.operation, "quarantine-dangling");
    assert_eq!(report.edges_removed, 1, "one dangling edge quarantined");
    assert_eq!(report.assertions_removed, 0, "quarantine never touches authority");
    assert!(
        report.summary.contains("1 before, 0 after"),
        "summary carries before-and-after counts: {}",
        report.summary
    );
    assert!(report.audit_sequence >= 1, "repair emits an audit event");

    // Audit emission: one event on the repair subject carrying the removed row.
    assert_eq!(repair_audit_count(&path, "quarantine-dangling").await, 1);
    let after = repair_audit_after(&path, "quarantine-dangling").await;
    assert!(after.contains("specimen-repair-1"), "audit records the removed edge: {after}");

    // Post-repair doctor is green on the dangling check.
    let found = checks(&path, true).await;
    let (status, summary) = status_of(&found, "quran.graph.dangling_edges");
    assert_eq!(status, CheckLevel::Pass, "adjacency clean after quarantine: {summary}");
}

#[tokio::test]
async fn repair_tombstone_gc_collects_only_old_tombstones() {
    let (_dir, path) = built_db().await;
    // One old tombstone (past retention), one young tombstone, and one live
    // row per surviving decision — distinct triples so no two proposals
    // collapse under content-addressed reuse.
    propose(&path, propose_at("gc-old-1", "ayah:1:1", "PARALLELS", "ayah:1:2"))
        .await
        .expect("propose");
    reject(&path, decide("gc-old-1")).await.expect("reject tombstones");
    backdate_decision(&path, "gc-old-1", "2020-01-01T00:00:00Z").await;

    propose(&path, propose_at("gc-young-1", "ayah:1:2", "PARALLELS", "ayah:1:3"))
        .await
        .expect("propose");
    reject(&path, decide("gc-young-1")).await.expect("reject tombstones");

    propose(&path, propose_at("gc-pending-1", "ayah:2:1", "RELATED_TO", "ayah:2:2"))
        .await
        .expect("pending stays");
    propose(&path, propose_at("gc-accepted-1", "ayah:3:1", "SUPPORTED_BY", "ayah:3:2"))
        .await
        .expect("propose");
    accept(&path, decide("gc-accepted-1")).await.expect("accept decides");
    propose(&path, propose_at("gc-disputed-1", "ayah:1:1", "RELATED_TO", "ayah:1:3"))
        .await
        .expect("propose");
    dispute(&path, decide("gc-disputed-1")).await.expect("dispute decides");

    let report = repair_tombstone_gc(
        &path,
        TombstoneGcInput { invoked_by: OPERATOR.to_string(), confirmed: true, retention_days: 30 },
    )
    .await
    .expect("gc collects");
    assert_eq!(report.operation, "tombstone-gc");
    assert_eq!(report.assertions_removed, 1, "only the backdated tombstone is due");
    assert!(
        report.summary.contains("older than 30 day(s)"),
        "summary names the retention gate: {}",
        report.summary
    );
    assert!(
        report.records["collected"][0]["id"] == serde_json::json!("gc-old-1"),
        "records name the collected row: {}",
        report.records
    );

    // Retention refusal: the young tombstone plus every live row survives.
    for (id, decision) in [
        ("gc-young-1", quran_graph::AssertionDecision::Rejected),
        ("gc-pending-1", quran_graph::AssertionDecision::Pending),
        ("gc-accepted-1", quran_graph::AssertionDecision::Accepted),
        ("gc-disputed-1", quran_graph::AssertionDecision::Disputed),
    ] {
        let kept = application::quran_graph_annotations::get_assertion(&path, id)
            .await
            .expect("survivor retained");
        assert_eq!(kept.decision, decision, "{id} survives the collection");
    }
    assert!(
        report.live_rows_refused >= 3,
        "report counts the refused live rows: {}",
        report.summary
    );
    // The collected row is gone from authority.
    assert!(
        application::quran_graph_annotations::get_assertion(&path, "gc-old-1").await.is_err(),
        "collected tombstone leaves authority"
    );

    // Audit emission on the GC subject.
    assert_eq!(repair_audit_count(&path, "tombstone-gc").await, 1);
    let after = repair_audit_after(&path, "tombstone-gc").await;
    assert!(after.contains("gc-old-1"), "audit records the collection: {after}");

    // Doctor stays honest after GC: no dangling edges left behind.
    let found = checks(&path, true).await;
    let (status, summary) = status_of(&found, "quran.graph.dangling_edges");
    assert_eq!(status, CheckLevel::Pass, "gc leaves no dangling adjacency: {summary}");
}

#[tokio::test]
async fn repair_rebuild_preserves_authority_and_doctor_stays_green() {
    let (_dir, path) = built_db().await;
    propose(&path, propose_at("rebuild-keep-1", "ayah:1:1", "PARALLELS", "ayah:1:2"))
        .await
        .expect("propose");
    accept(&path, decide("rebuild-keep-1")).await.expect("accept decides");

    let report = repair_rebuild_projection(
        &path,
        RebuildInput {
            invoked_by: OPERATOR.to_string(),
            confirmed: true,
            projection: PROJECTION.to_string(),
            seed_file: None,
        },
    )
    .await
    .expect("rebuild republishes");
    assert_eq!(report.operation, "rebuild-projection");
    assert!(
        report.summary.contains("authority preserved"),
        "summary says authority is preserved: {}",
        report.summary
    );
    assert!(report.records["build_row_id"].is_string(), "records carry the new build row");

    // Authority-preserving rebuild: the accepted assertion survives with its
    // decision, and doctor is green on the rebuilt projection.
    let kept = application::quran_graph_annotations::get_assertion(&path, "rebuild-keep-1")
        .await
        .expect("authority survives rebuild");
    assert_eq!(kept.decision, quran_graph::AssertionDecision::Accepted);
    assert_eq!(repair_audit_count(&path, "rebuild-projection").await, 1);

    let found = checks(&path, false).await;
    let (status, summary) = status_of(&found, "quran.graph.structural.current");
    assert_eq!(status, CheckLevel::Pass, "rebuilt projection is current: {summary}");
    let (status, _) = status_of(&found, "quran.graph.dangling_edges");
    assert_eq!(status, CheckLevel::Pass, "rebuilt adjacency has no dangling edges");
}
