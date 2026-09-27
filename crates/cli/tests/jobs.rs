//! 01-03-03 — `qai job` controls over the real binary (D-15/D-16).
//!
//! - `job show <id> --json` exposes the full redacted inspection envelope
//!   (attempts, max attempts, state, cancellation request, checkpoint/error,
//!   disposition) as one parseable document without leaking secrets.
//! - `job retry` moves only an eligible job and maps missing/ineligible ids
//!   to the centralized NOT_FOUND/CONFLICT exits without mutating the row.
//! - `job cancel` persists only the request: the lease survives, the state
//!   stays Running, and no false cancelled state is claimed.
//! - A worker-recorded disposition surfaces through `job show`.

use std::process::Output;

const SENTINEL: &str = "SENTINEL_9f3c__DO_NOT_LEAK";

fn qai(dir: &std::path::Path, args: &[&str]) -> Output {
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_qai"));
    cmd.args(["--data-dir", dir.to_str().unwrap()]);
    cmd.args(args);
    cmd.output().expect("run qai")
}

fn migrate(dir: &std::path::Path) {
    let mut cfg = config::Config::default();
    cfg.storage.sqlite.path = dir.join("qai.db").display().to_string();
    let migrations =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    runtime.block_on(application::db::migrate_database(&cfg, &migrations)).unwrap();
}

fn stdout_json(out: &Output) -> serde_json::Value {
    assert!(
        out.status.success(),
        "expected success (code {:?}): {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("stdout is one parseable JSON document")
}

fn seed(path: &std::path::Path) {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    runtime.block_on(async {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(path))
            .await
            .unwrap();
        // An exhausted job eligible for explicit retry, carrying secrets in
        // its payload and checkpoint columns.
        let payload = format!(r#"{{"n":1,"password":"{SENTINEL}"}}"#);
        let checkpoint = format!(r#"{{"version":1,"name":"stage.done","password":"{SENTINEL}"}}"#);
        sqlx::query(
            "INSERT INTO jobs (id, kind, payload_json, state, priority, attempts, \
             max_attempts, available_at, checkpoint_json, created_at) \
             VALUES ('job-failed', 'system.noop_test', ?, 'Failed', 0, 5, 5, \
             '2026-01-01T00:00:00Z', ?, '2026-01-01T00:00:00Z')",
        )
        .bind(&payload)
        .bind(&checkpoint)
        .execute(&pool)
        .await
        .unwrap();
        // A running job holding a live lease.
        sqlx::query(
            "INSERT INTO jobs (id, kind, payload_json, state, priority, attempts, \
             max_attempts, available_at, lease_owner, lease_expires_at, created_at) \
             VALUES ('job-running', 'system.noop_test', '{}', 'Running', 0, 1, 5, \
             '2026-01-01T00:00:00Z', 'worker-1', '2999-01-01T00:00:00Z', \
             '2026-01-01T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .unwrap();
        // A cancelled job carrying a worker-recorded disposition plus a secret
        // in its error column.
        let error = format!(r#"{{"disposition":"missed_boundary","api_key":"{SENTINEL}"}}"#);
        sqlx::query(
            "INSERT INTO jobs (id, kind, payload_json, state, priority, attempts, \
             max_attempts, available_at, cancel_requested, error_json, created_at) \
             VALUES ('job-cancelled', 'system.noop_test', '{}', 'Cancelled', 0, 1, 5, \
             '2026-01-01T00:00:00Z', 1, ?, '2026-01-01T00:00:00Z')",
        )
        .bind(&error)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)").execute(&pool).await.unwrap();
        pool.close().await;
    });
}

#[test]
fn job_show_exposes_the_redacted_inspection_envelope() {
    let dir = tempfile::tempdir().unwrap();
    migrate(dir.path());
    seed(&dir.path().join("qai.db"));

    let out = qai(dir.path(), &["job", "show", "job-failed", "--json"]);
    let doc = stdout_json(&out);
    assert_eq!(doc["id"], "job-failed");
    assert_eq!(doc["kind"], "system.noop_test");
    assert_eq!(doc["state"], "Failed");
    assert_eq!(doc["attempts"], 5);
    assert_eq!(doc["max_attempts"], 5);
    assert_eq!(doc["cancel_requested"], 0);
    assert!(doc["checkpoint_json"].is_string(), "{doc}");
    assert!(doc.get("disposition").is_some(), "disposition field present: {doc}");

    // Human mode renders one line and exits 0.
    let out = qai(dir.path(), &["job", "show", "job-failed"]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("job-failed"));

    // A worker-recorded disposition surfaces verbatim.
    let out = qai(dir.path(), &["job", "show", "job-cancelled", "--json"]);
    assert_eq!(stdout_json(&out)["disposition"], "missed_boundary");

    // Missing ids exit NOT_FOUND (5), never a silent success.
    let out = qai(dir.path(), &["job", "show", "missing"]);
    assert_eq!(out.status.code(), Some(5));
    let out = qai(dir.path(), &["job", "show", "missing", "--json"]);
    assert_eq!(out.status.code(), Some(5));
}

#[test]
fn job_retry_moves_only_eligible_jobs() {
    let dir = tempfile::tempdir().unwrap();
    migrate(dir.path());
    seed(&dir.path().join("qai.db"));

    // Eligible: exit 0 with the new queued state.
    let out = qai(dir.path(), &["job", "retry", "job-failed", "--json"]);
    let doc = stdout_json(&out);
    assert_eq!(doc["state"], "Queued");
    assert_eq!(doc["attempts"], 0);

    // Running jobs are ineligible: CONFLICT (6) with the row unchanged.
    let out = qai(dir.path(), &["job", "retry", "job-running", "--json"]);
    assert_eq!(out.status.code(), Some(6), "{}", String::from_utf8_lossy(&out.stderr));
    let out = qai(dir.path(), &["job", "show", "job-running", "--json"]);
    assert_eq!(stdout_json(&out)["state"], "Running");

    // Missing ids are NOT_FOUND (5).
    let out = qai(dir.path(), &["job", "retry", "missing", "--json"]);
    assert_eq!(out.status.code(), Some(5));
}

#[test]
fn job_cancel_requests_without_clearing_the_lease() {
    let dir = tempfile::tempdir().unwrap();
    migrate(dir.path());
    seed(&dir.path().join("qai.db"));

    let out = qai(dir.path(), &["job", "cancel", "job-running", "--json"]);
    let doc = stdout_json(&out);
    // The request is durable…
    assert_eq!(doc["cancel_requested"], 1);
    // …but the live lease survives and no false cancelled state is claimed.
    assert_eq!(doc["lease_owner"], "worker-1");
    assert_eq!(doc["state"], "Running");

    // Cancelling a terminal job is a conflict, not a silent success.
    let out = qai(dir.path(), &["job", "cancel", "job-cancelled", "--json"]);
    assert_eq!(out.status.code(), Some(6));
    let out = qai(dir.path(), &["job", "cancel", "missing", "--json"]);
    assert_eq!(out.status.code(), Some(5));
}

#[test]
fn job_output_never_leaks_secrets() {
    let dir = tempfile::tempdir().unwrap();
    migrate(dir.path());
    seed(&dir.path().join("qai.db"));

    let mut combined = Vec::new();
    for args in [
        &["job", "show", "job-failed"][..],
        &["job", "show", "job-failed", "--json"][..],
        &["job", "show", "job-cancelled", "--json"][..],
        &["job", "retry", "job-failed", "--json"][..],
        &["job", "show", "job-failed", "--json"][..],
    ] {
        let out = qai(dir.path(), args);
        assert!(out.status.success(), "args {args:?}: {}", String::from_utf8_lossy(&out.stderr));
        combined.extend_from_slice(&out.stdout);
        combined.extend_from_slice(&out.stderr);
    }
    let text = String::from_utf8_lossy(&combined).into_owned();
    assert!(!text.contains(SENTINEL), "no job path leaks the sentinel");
    assert!(text.contains("***REDACTED***"), "redaction marker present");
}
