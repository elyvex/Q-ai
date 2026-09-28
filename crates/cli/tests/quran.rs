//! CLI acceptance snapshots — end-to-end flows (AC-P1-16, P1-T50, P2-T23).
//!
//! Points the real `qai` binary at a fresh temp database via `QAI_DATA_DIR`,
//! then drives `trycmd` cases (see `tests/quran/`). Cases in a file run in
//! order against the one database; human output elides volatile values.
//!
//! Each file gets its own database: trycmd files execute in parallel, so two
//! files sharing one `QAI_DATA_DIR` race (notably two `db migrate` runs on
//! the same SQLite file).

fn run_cases(pattern: &str) {
    let dir = tempfile::tempdir().unwrap();
    trycmd::TestCases::new()
        .default_bin_name("qai")
        .env("QAI_DATA_DIR", dir.path().to_str().unwrap())
        .case("tests/quran/*.toml")
        .case(pattern);
    // Keep the temp database alive until assertions complete.
    drop(dir);
}

/// Phase 1 reading flow (AC-P1-16, P1-T50).
#[test]
fn quran_snapshots() {
    run_cases("tests/quran/read_flow.trycmd");
}

/// Phase 2 normalization introspection (P2-T23).
#[test]
fn quran_normalize_snapshots() {
    run_cases("tests/quran/normalize.trycmd");
}

/// Phase 2 search surfaces (P2-T51/T52).
#[test]
fn quran_search_snapshots() {
    run_cases("tests/quran/search.trycmd");
}

/// Phase 2 counting + Phase 4 graph CLI surfaces (P2-T104, TASK-424 slice).
#[test]
fn quran_counting_graph_snapshots() {
    run_cases("tests/quran/counting_graph.trycmd");
}

/// Phase 2 six-family integrity surface (D-10/D-11).
#[test]
fn quran_verify_snapshots() {
    run_cases("tests/quran/verify.trycmd");
}

/// Phase 2 operator reference-corpus path (QV-015, ADR-0114).
#[test]
fn quran_reference_snapshots() {
    run_cases("tests/quran/reference.trycmd");
}

/// Phase 2 edition identity/primary/license operator surface (D-07).
#[test]
fn edition_identity_snapshots() {
    run_cases("tests/quran/edition_identity.trycmd");
}

/// Phase 2 quotation hard-failure surface (D-15, QC-07).
#[test]
fn quran_verify_quotation_snapshots() {
    run_cases("tests/quran/verify_quotation.trycmd");
}

/// Upstream catalog ingestion (metadata-only, no database).
#[test]
fn quran_catalog_snapshots() {
    run_cases("tests/quran/catalog.trycmd");
}

// ─── 01-04-01: real `qai serve` host proof (D-13/D-14/D-16) ───────────
//
// Spawns the built `qai serve` binary on an isolated loopback port against a
// migrated temp database, proves readiness via `/readyz`, seeds a real
// `quran.import` job through SQL, observes the host driving it to
// `Succeeded`, then delivers SIGINT and asserts a joined shutdown: exit 0,
// port released, and later jobs untouched (no detached worker).

const HOST_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const HOST_PRINCIPAL: &str = "00000000-0000-0000-0000-000000000000";
const HOST_CREATED_AT: &str = "2026-09-27T00:00:00Z";

fn migrate_db(dir: &std::path::Path) {
    let mut cfg = config::Config::default();
    cfg.storage.sqlite.path = dir.join("qai.db").display().to_string();
    let migrations =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    runtime.block_on(application::db::migrate_database(&cfg, &migrations)).unwrap();
}

fn qai_out(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_qai"));
    cmd.args(["--data-dir", dir.to_str().unwrap()]);
    cmd.args(args);
    cmd.output().expect("run qai")
}

/// Seed one job row plus the FK parents a `quran.import` handler needs.
/// `payload` is the raw `payload_json`; pass `"{}"` for inert kinds.
fn seed_job(db_path: &std::path::Path, id: &str, kind: &str, payload: &str) {
    let db = db_path.to_str().unwrap().to_string();
    let (id, kind, payload) = (id.to_string(), kind.to_string(), payload.to_string());
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    runtime.block_on(async {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&db))
            .await
            .unwrap();
        for setup in [
            format!(
                "INSERT OR IGNORE INTO principals (id, kind, display_name, created_at) \
                 VALUES ('{HOST_PRINCIPAL}', 'local_user', 'Serve host test', '{HOST_CREATED_AT}')"
            ),
            "INSERT OR IGNORE INTO sources (id, title, content_type, created_at, updated_at) \
             VALUES ('src-test-edition-min', 'test-edition-min', 'quran_edition', \
             '2026-09-27T00:00:00Z', '2026-09-27T00:00:00Z')"
                .to_string(),
            "INSERT OR IGNORE INTO source_versions \
                (id, source_id, version, schema_version, state, trust_level, \
                 license_status, license_json, created_at) \
             VALUES ('sv-host-1', 'src-test-edition-min', '0.1.0', 1, 'Staged', \
                     'ImportedUnverified', 'Unknown', '{}', '2026-09-27T00:00:00Z')"
                .to_string(),
        ] {
            sqlx::query(&setup).execute(&pool).await.unwrap();
        }
        let key = application::quran::import_idempotency_key("sv-host-1");
        sqlx::query(
            "INSERT INTO jobs (id, kind, payload_json, idempotency_key, state, priority, \
             attempts, max_attempts, available_at, created_at, created_by) \
             VALUES (?, ?, ?, ?, 'Queued', 0, 0, 5, '2026-09-27T00:00:00Z', \
             '2026-09-27T00:00:00Z', '00000000-0000-0000-0000-000000000000')",
        )
        .bind(&id)
        .bind(&kind)
        .bind(&payload)
        .bind(if kind == "quran.import" { key } else { format!("{id}:manual") })
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)").execute(&pool).await.unwrap();
        pool.close().await;
    });
}

fn import_payload() -> String {
    serde_json::json!({
        "run_id": "run-host-1",
        "job_id": "job-host-import",
        "source_version_id": "sv-host-1",
        "adapter": "json",
        "manifest_text": HOST_MANIFEST,
        "declared_manifest_hash": null,
        "invoked_by": HOST_PRINCIPAL,
        "license_status": "Unknown",
        "license_json": "{}",
        "created_at": HOST_CREATED_AT,
        "reference_manifest_text": null,
    })
    .to_string()
}

fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

fn readyz_ok(port: u16) -> bool {
    use std::io::{Read, Write};
    let addr: std::net::SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let Ok(mut stream) =
        std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(200))
    else {
        return false;
    };
    stream.set_read_timeout(Some(std::time::Duration::from_secs(2))).ok();
    if write!(stream, "GET /readyz HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .is_err()
    {
        return false;
    }
    let mut buf = Vec::new();
    if stream.read_to_end(&mut buf).is_err() {
        return false;
    }
    let text = String::from_utf8_lossy(&buf);
    text.contains("200") && text.contains("ready")
}

/// A live `qai serve` child with its tempdir and port. `Drop` SIGKILLs and
/// reaps survivors so no test leaves a server behind; the test itself asserts
/// the graceful SIGINT exit before the guard drops.
struct ServeGuard {
    dir: tempfile::TempDir,
    port: u16,
    child: Option<std::process::Child>,
}

impl ServeGuard {
    fn start() -> Self {
        let dir = tempfile::tempdir().unwrap();
        migrate_db(dir.path());
        seed_job(&dir.path().join("qai.db"), "job-host-import", "quran.import", &import_payload());
        let port = free_port();
        let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_qai"));
        cmd.args(["--data-dir", dir.path().to_str().unwrap()]);
        cmd.args(["serve", "--bind", &format!("127.0.0.1:{port}")]);
        cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::piped());
        let child = cmd.spawn().expect("spawn qai serve");
        let guard = Self { dir, port, child: Some(child) };
        let start = std::time::Instant::now();
        let budget = std::time::Duration::from_secs(20);
        while !readyz_ok(guard.port) {
            assert!(
                start.elapsed() < budget,
                "qai serve never became ready on {port}",
                port = guard.port
            );
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        guard
    }

    fn job_state(&self, id: &str) -> String {
        let out = qai_out(self.dir.path(), &["job", "show", id, "--json"]);
        assert!(out.status.success(), "job show {id}: {}", String::from_utf8_lossy(&out.stderr));
        serde_json::from_slice::<serde_json::Value>(&out.stdout).unwrap()["state"]
            .as_str()
            .unwrap()
            .to_string()
    }

    fn wait_job_terminal(&self, id: &str, budget: std::time::Duration) -> String {
        let start = std::time::Instant::now();
        loop {
            let state = self.job_state(id);
            if ["Succeeded", "Failed", "DeadLettered", "Cancelled"].contains(&state.as_str()) {
                return state;
            }
            assert!(
                start.elapsed() < budget,
                "job {id} not terminal within {budget:?} (state {state})"
            );
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    /// Deliver SIGINT and wait for the joined exit. Returns the exit status.
    fn shutdown_graceful(&mut self, budget: std::time::Duration) -> std::process::ExitStatus {
        let child = self.child.as_mut().expect("serve child running");
        let pid = child.id();
        let status = std::process::Command::new("kill")
            .arg("-INT")
            .arg(pid.to_string())
            .status()
            .expect("kill -INT the serve child");
        assert!(status.success(), "kill -INT {pid}");
        let start = std::time::Instant::now();
        loop {
            if let Some(status) = child.try_wait().expect("try_wait the serve child") {
                return status;
            }
            assert!(start.elapsed() < budget, "serve did not exit within {budget:?} after SIGINT");
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    fn stderr(&mut self) -> String {
        let mut text = String::new();
        if let Some(mut child) = self.child.take() {
            if let Some(mut pipe) = child.stderr.take() {
                use std::io::Read;
                let _ = pipe.read_to_string(&mut text);
            }
            let _ = child.try_wait();
            self.child = Some(child);
        }
        text
    }
}

impl Drop for ServeGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[test]
fn serve_hosts_the_worker_and_shuts_down_joined() {
    let mut guard = ServeGuard::start();

    // The host owns the seeded import: exactly one claimed-and-terminal job,
    // with no one-shot drain involved.
    assert_eq!(
        guard.wait_job_terminal("job-host-import", std::time::Duration::from_secs(90)),
        "Succeeded"
    );
    let out = qai_out(guard.dir.path(), &["job", "list", "--json"]);
    assert!(out.status.success());
    let items = serde_json::from_slice::<serde_json::Value>(&out.stdout).unwrap()["items"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert_eq!(items.len(), 1, "exactly one job: {items:?}");
    assert_eq!(items[0]["state"], "Succeeded");

    // Staging landed through the host: the edition validates from storage.
    let out = qai_out(guard.dir.path(), &["quran", "validate", "test-edition-min@0.1.0"]);
    assert!(
        out.status.success(),
        "staged edition validates: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Signal-driven joined shutdown: SIGINT exits 0 after the worker joins.
    let status = guard.shutdown_graceful(std::time::Duration::from_secs(15));
    assert!(status.success(), "serve exits 0 on SIGINT (stderr: {})", guard.stderr());

    // The port is released and no detached worker survives: a later job stays
    // queued well past the host poll interval.
    assert!(
        std::net::TcpStream::connect(format!("127.0.0.1:{}", guard.port)).is_err(),
        "serve port released after shutdown"
    );
    seed_job(&guard.dir.path().join("qai.db"), "job-host-late", "system.noop_test", "{}");
    std::thread::sleep(std::time::Duration::from_secs(2));
    assert_eq!(guard.job_state("job-host-late"), "Queued");
}

#[test]
fn serve_refuses_without_a_migrated_database() {
    let dir = tempfile::tempdir().unwrap();
    let out = qai_out(dir.path(), &["serve", "--bind", "127.0.0.1:18737"]);
    assert_eq!(out.status.code(), Some(3), "exit VALIDATION");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("qai db migrate"), "names the remedy: {err}");
    assert!(!dir.path().join("qai.db").exists(), "serve creates no state");
}

#[test]
fn serve_still_refuses_non_loopback_binds() {
    let dir = tempfile::tempdir().unwrap();
    let out = qai_out(dir.path(), &["serve", "--bind", "0.0.0.0:18738"]);
    assert_eq!(out.status.code(), Some(4), "exit POLICY");
}

// ─── 01-04-02: enqueue-only import (D-13) ─────────────────────────────
//
// Without any host running, `qai quran import` validates, persists a queued
// `quran.import` job, and returns its id in `Queued` state. The test fails if
// the one-shot process constructs or drains a worker: then the job would be
// terminal and the edition already staged.

#[test]
fn enqueue_only_import_is_queued_without_host() {
    let dir = tempfile::tempdir().unwrap();
    migrate_db(dir.path());
    // No `qai serve` child exists anywhere in this test.
    let manifest = "../../fixtures/quran/test-edition-min/manifest.json";

    let out = qai_out(dir.path(), &["quran", "import", manifest, "--json"]);
    assert!(out.status.success(), "import enqueues: {}", String::from_utf8_lossy(&out.stderr));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("one JSON document");
    assert_eq!(doc["kind"], "quran.import", "{doc}");
    assert_eq!(doc["state"], "Queued", "{doc}");
    let job_id = doc["job_id"].as_str().expect("durable job id");
    assert!(!job_id.is_empty());
    assert!(doc["max_attempts"].as_u64().unwrap_or(0) >= 1, "retry metadata: {doc}");
    assert!(
        doc["inspect"].as_str().unwrap_or_default().contains(job_id),
        "inspect command names the job: {doc}"
    );

    // The job is inspectable through the existing read path and still queued:
    // no worker ran inside the one-shot process.
    let out = qai_out(dir.path(), &["job", "show", job_id, "--json"]);
    assert!(out.status.success());
    let shown: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(shown["state"], "Queued", "{shown}");

    // Nothing is staged until a host processes the job.
    let out = qai_out(dir.path(), &["quran", "validate", "test-edition-min@0.1.0"]);
    assert_eq!(out.status.code(), Some(5), "unstaged edition is not found");

    // Human wording reports queued state and never claims terminal staging.
    // A second import uses the v2 manifest: re-importing the same
    // `slug@version` is a catalog conflict (UNIQUE(source_id, version),
    // pre-existing on the sync path — the setup seam is untouched here).
    let out = qai_out(
        dir.path(),
        &["quran", "import", "../../fixtures/quran/test-edition-min-v2.json"],
    );
    assert!(out.status.success(), "second import enqueues: {}", String::from_utf8_lossy(&out.stderr));
    let human = String::from_utf8_lossy(&out.stdout);
    let human = String::from_utf8_lossy(&out.stdout);
    assert!(human.contains("queued"), "queued wording: {human}");
    assert!(human.contains("Queued"), "queued state: {human}");
    // The human import mints its own job; assert the shape, not the id.
    assert!(human.contains("job"), "names the queued job: {human}");
    assert!(human.contains("qai job show"), "points at inspection: {human}");
    assert!(!human.contains("to Staged"), "no terminal claim: {human}");
}
