//! 01-04-01 — long-lived worker host over real SQLite (D-13/D-14/D-16).
//!
//! - The default host claims a queued `quran.import` job and drives it to a
//!   terminal state while the host task stays alive (host ownership, D-13).
//! - A shutdown signal stops new claims, requests cooperative cancellation
//!   for an in-flight job, awaits the terminal transition, and returns with
//!   no detached worker behind it (D-14, D-16).
//! - A pre-start cancellation request is observed at the start boundary with
//!   the locked `cancelled_at_checkpoint` disposition (D-16).

use std::sync::Arc;
use std::time::Duration;

use application::job_queue::{SqliteJobQueue, build_default_registry, run_worker_host};
use application::quran::{QURAN_IMPORT_KIND, import_idempotency_key};
use jobs::queue::JobQueue;
use quran_corpus::import::ImportInput;
use storage::Database as _;
use storage::repository::JobRecord;
use storage_sqlite::SqliteDatabase;

const BASE_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const PRINCIPAL: &str = "00000000-0000-0000-0000-000000000001";
const CREATED_AT: &str = "2026-09-27T00:00:00Z";

async fn migrated_db() -> (tempfile::TempDir, Arc<SqliteDatabase>, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("qai.db");
    let path_str = path.to_str().unwrap().to_string();
    let mut cfg = application::Config::default();
    cfg.storage.sqlite.path.clone_from(&path_str);
    let migrations =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    application::db::migrate_database(&cfg, &migrations).await.unwrap();
    let db = SqliteDatabase::new(&path_str, 4, true).await.unwrap();
    // FK parents for the import handler (mirrors quran_import.rs seeds).
    let seed = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new().filename(&path_str).foreign_keys(true),
        )
        .await
        .unwrap();
    for sql in [
        format!(
            "INSERT INTO principals (id, kind, display_name, created_at) \
             VALUES ('{PRINCIPAL}', 'local_user', 'Test', '{CREATED_AT}')"
        ),
        "INSERT INTO sources (id, title, content_type, created_at, updated_at) \
         VALUES ('src-1', 'Test source', 'quran_edition', '2026-09-27T00:00:00Z', \
         '2026-09-27T00:00:00Z')"
            .to_string(),
        "INSERT INTO sources (id, title, content_type, created_at, updated_at) \
         VALUES ('src-2', 'Test source 2', 'quran_edition', '2026-09-27T00:00:00Z', \
         '2026-09-27T00:00:00Z')"
            .to_string(),
        "INSERT INTO source_versions \
            (id, source_id, version, schema_version, state, trust_level, \
             license_status, license_json, created_at) \
         VALUES ('sv-1', 'src-1', '0.1.0', 1, 'Staged', 'ImportedUnverified', \
                 'PublicDomain', '{}', '2026-09-27T00:00:00Z')"
            .to_string(),
        // A second source at the same manifest version: distinct idempotency
        // keys per job while the manifest/source version check still passes.
        "INSERT INTO source_versions \
            (id, source_id, version, schema_version, state, trust_level, \
             license_status, license_json, created_at) \
         VALUES ('sv-2', 'src-2', '0.1.0', 1, 'Staged', 'ImportedUnverified', \
                 'PublicDomain', '{}', '2026-09-27T00:00:00Z')"
            .to_string(),
    ] {
        sqlx::query(&sql).execute(&seed).await.unwrap();
    }
    seed.close().await;
    (dir, Arc::new(db), path_str)
}

fn import_input(run_id: &str, source_version_id: &str) -> ImportInput {
    ImportInput {
        run_id: run_id.into(),
        job_id: None,
        source_version_id: source_version_id.into(),
        adapter: "json".into(),
        manifest_text: BASE_MANIFEST.into(),
        declared_manifest_hash: None,
        invoked_by: PRINCIPAL.into(),
        license_status: "PublicDomain".into(),
        license_json: "{}".into(),
        created_at: CREATED_AT.into(),
        reference_manifest_text: None,
    }
}

fn import_record(id: &str, run_id: &str, source_version_id: &str) -> JobRecord {
    let mut payload = import_input(run_id, source_version_id);
    payload.job_id = Some(id.into());
    JobRecord {
        id: id.into(),
        kind: QURAN_IMPORT_KIND.into(),
        payload_json: serde_json::to_value(&payload).unwrap().to_string(),
        idempotency_key: Some(import_idempotency_key(source_version_id)),
        state: "Queued".into(),
        priority: 0,
        attempts: 0,
        max_attempts: 5,
        available_at: CREATED_AT.into(),
        lease_owner: None,
        lease_expires_at: None,
        checkpoint_json: None,
        cancel_requested: false,
        created_by: PRINCIPAL.into(),
    }
}

async fn staged_count(db: &SqliteDatabase, run_id: &str) -> i64 {
    let mut uow = db.write().await.unwrap();
    let count = uow.quran().count_stg_ayahs(run_id).await.unwrap();
    uow.rollback().await.unwrap();
    count
}

async fn poll_terminal(queue: &SqliteJobQueue, id: &str, timeout: Duration) -> JobRecord {
    let start = std::time::Instant::now();
    loop {
        let job = queue.get(id).await.unwrap().unwrap();
        if matches!(job.state.as_str(), "Succeeded" | "Failed" | "DeadLettered" | "Cancelled") {
            return job;
        }
        assert!(
            start.elapsed() < timeout,
            "job {id} not terminal within {timeout:?} (state {})",
            job.state
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

fn is_terminal(state: &str) -> bool {
    matches!(state, "Succeeded" | "Failed" | "DeadLettered" | "Cancelled")
}

/// The default host owns a queued import to `Succeeded` while staying alive,
/// stages the edition, keeps the audit chain valid, and joins on shutdown
/// with no detached worker left behind.
#[tokio::test]
async fn host_claims_and_completes_a_queued_import() {
    let (_dir, db, db_path) = migrated_db().await;
    let registry = Arc::new(build_default_registry(&db));
    // The default host registers every existing application job handler and
    // leaves unknown kinds to the worker dead-letter policy.
    assert!(registry.get(QURAN_IMPORT_KIND).is_some());
    assert!(registry.get(application::quran_forms::QURAN_FORMS_REBUILD_KIND).is_some());
    assert!(registry.get(application::quran_index::QURAN_INDEX_BUILD_KIND).is_some());
    assert!(registry.get(application::quran_morphology::MORPHOLOGY_IMPORT_KIND).is_some());

    let queue = Arc::new(SqliteJobQueue::new(db.clone()).with_retry_registry(registry));
    queue.enqueue(import_record("job-host-1", "run-host-1", "sv-1")).await.unwrap();

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let host = tokio::spawn(run_worker_host(db.clone(), "test-host", shutdown_rx));

    let job = poll_terminal(&queue, "job-host-1", Duration::from_secs(60)).await;
    assert_eq!(job.state, "Succeeded", "{job:?}");
    assert_eq!(staged_count(&db, "run-host-1").await, 14);

    // The composed lifecycle keeps the persisted audit chain valid (D-12).
    let report = application::audit_bridge::verify_persisted_audit(&db_path).await.unwrap();
    assert!(report.valid, "{report:?}");

    // Joined shutdown: the host returns instead of detaching.
    shutdown_tx.send(true).unwrap();
    let processed = tokio::time::timeout(Duration::from_secs(15), host)
        .await
        .expect("host joins on shutdown")
        .unwrap()
        .unwrap();
    assert!(processed >= 1, "host reports the claimed job");

    // No detached worker survives the join: a later job stays queued.
    queue.enqueue(import_record("job-after", "run-after", "sv-2")).await.unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(queue.get("job-after").await.unwrap().unwrap().state, "Queued");
}

/// Shutdown during handler execution requests cooperative cancellation and
/// awaits the truthful terminal transition; a pre-start request stops at the
/// start boundary with the locked disposition.
#[tokio::test]
async fn host_shutdown_finalizes_an_in_flight_import_truthfully() {
    let (_dir, db, db_path) = migrated_db().await;
    let queue = Arc::new(
        SqliteJobQueue::new(db.clone()).with_retry_registry(Arc::new(build_default_registry(&db))),
    );

    // Deterministic pre-start boundary: requested before the host starts.
    queue.enqueue(import_record("job-pre", "run-pre", "sv-1")).await.unwrap();
    queue.request_cancel("job-pre").await.unwrap();
    // In-flight race: the request lands while the handler may be running.
    queue.enqueue(import_record("job-race", "run-race", "sv-2")).await.unwrap();

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let host = tokio::spawn(run_worker_host(db.clone(), "test-host", shutdown_rx));

    // Gate the shutdown on the in-flight window when the host is slow enough
    // to observe; when the host already finished, the assertions below still
    // hold (a completed job reports truthfully instead of falsely cancelling).
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(30) {
        let job = queue.get("job-race").await.unwrap().unwrap();
        if !matches!(job.state.as_str(), "Queued") || is_terminal(&job.state) {
            break;
        }
        // Gentle polling: each read takes the single write connection, and
        // hammering it starves the running import into a spurious failure.
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    shutdown_tx.send(true).unwrap();
    tokio::time::timeout(Duration::from_secs(60), host)
        .await
        .expect("host joins after an in-flight shutdown")
        .unwrap()
        .unwrap();

    // Pre-start request: cancelled at the start checkpoint with the locked
    // disposition surfaced through the existing inspection envelope.
    let pre = application::db::get_job(&db_path, "job-pre").await.unwrap();
    assert_eq!(pre["state"], "Cancelled", "{pre}");
    assert_eq!(pre["disposition"], "cancelled_at_checkpoint", "{pre}");

    // In-flight job: reached a durable terminal state without a false claim —
    // either it finished first (Succeeded, possibly CompletedBeforeObservation
    // when the late request landed) or it stopped at a boundary (Cancelled
    // with a locked disposition).
    let race = application::db::get_job(&db_path, "job-race").await.unwrap();
    match race["state"].as_str().unwrap() {
        "Succeeded" => {
            assert!(
                race["disposition"].is_null()
                    || race["disposition"] == "completed_before_observation",
                "{race}"
            );
        }
        "Cancelled" => {
            assert!(
                ["cancelled_at_checkpoint", "missed_boundary"]
                    .contains(&race["disposition"].as_str().unwrap_or_default()),
                "{race}"
            );
        }
        other => panic!("in-flight job must reach a truthful terminal state, got {other}: {race}"),
    }
}

/// Shutdown with an empty queue returns promptly and claims nothing.
#[tokio::test]
async fn host_shutdown_on_an_idle_queue_claims_nothing() {
    let (_dir, db, _path) = migrated_db().await;
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let host = tokio::spawn(run_worker_host(db.clone(), "test-host", shutdown_rx));
    shutdown_tx.send(true).unwrap();
    let processed = tokio::time::timeout(Duration::from_secs(15), host)
        .await
        .expect("idle host joins on shutdown")
        .unwrap()
        .unwrap();
    assert_eq!(processed, 0);
}
