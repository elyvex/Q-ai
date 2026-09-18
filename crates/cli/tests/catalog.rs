//! P0-T50 / FU-10 — `source`, `job`, `audit list`, `secret list`, and
//! `completions` dispatch to real read-only implementations.
//!
//! Regression guard: these commands used to be `phase_stub`s that printed
//! "will become available" and exited 0 without touching the database.
//! They now list/show persisted rows (or refuse mutations explicitly), and
//! every read path is byte-identical read-only with no secret leakage.

use std::process::Output;

const SENTINEL: &str = "SENTINEL_9f3c__DO_NOT_LEAK";

fn qai(dir: &std::path::Path, args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_qai"));
    cmd.args(["--data-dir", dir.to_str().unwrap()]);
    cmd.args(args);
    for (key, value) in envs {
        cmd.env(key, value);
    }
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
    assert!(out.status.success(), "expected success: {}", String::from_utf8_lossy(&out.stderr));
    serde_json::from_slice(&out.stdout).expect("stdout is JSON")
}

#[test]
fn empty_catalog_lists_and_guarded_mutations() {
    let dir = tempfile::tempdir().unwrap();
    migrate(dir.path());

    for command in ["source", "job"] {
        let out = qai(dir.path(), &[command, "list", "--json"], &[]);
        let doc = stdout_json(&out);
        assert_eq!(doc["items"], serde_json::json!([]));
        assert_eq!(doc["truncated"], false);
    }
    let out = qai(dir.path(), &["audit", "list", "--json"], &[]);
    assert_eq!(stdout_json(&out)["items"], serde_json::json!([]));

    // Unknown ids are NOT_FOUND (5), never a silent empty success.
    for args in [&["source", "show", "missing"][..], &["job", "show", "missing"][..]] {
        let out = qai(dir.path(), args, &[]);
        assert_eq!(out.status.code(), Some(5), "args {args:?}");
    }

    // Mutations are refused explicitly (USAGE), not stubbed as success.
    for args in [
        &["source", "import", "manifest.json"][..],
        &["job", "cancel", "job-1"][..],
        &["secret", "set", "secret://env/a/b"][..],
        &["secret", "delete", "secret://env/a/b"][..],
    ] {
        let out = qai(dir.path(), args, &[]);
        assert_eq!(out.status.code(), Some(2), "args {args:?}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("refusing"), "args {args:?}");
    }

    // Completions render static scripts; unknown shells are USAGE.
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        let out = qai(dir.path(), &["completions", shell], &[]);
        assert!(out.status.success(), "shell {shell}");
        assert!(String::from_utf8_lossy(&out.stdout).contains("qai"));
    }
    let out = qai(dir.path(), &["completions", "bogus-shell"], &[]);
    assert_eq!(out.status.code(), Some(2));

    // Secret list exposes references only — never values.
    let out = qai(
        dir.path(),
        &["secret", "list", "--json"],
        &[("QAI_SECRET_CATALOG_PROBE", "s3cr3t-value")],
    );
    let doc = stdout_json(&out);
    let refs = doc["refs"].as_array().unwrap();
    assert!(refs.iter().any(|r| r == "secret://env/catalog/probe"), "{refs:?}");
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(!text.contains("s3cr3t-value"));
}

#[test]
fn seeded_catalog_shows_without_mutation_or_leak() {
    let dir = tempfile::tempdir().unwrap();
    migrate(dir.path());
    let path = dir.path().join("qai.db");

    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    runtime.block_on(async {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&path))
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO sources (id, title, content_type, language, created_at, updated_at) \
             VALUES ('src-1', 'Seed Source', 'quran_edition', 'ar', \
             '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO source_versions (id, source_id, version, schema_version, state, \
             trust_level, license_status, license_json, content_hash, created_at) \
             VALUES ('ver-1', 'src-1', '1.0.0', 1, 'Staged', 'ImportedUnverified', \
             'OpenLicense', '{}', 'sha256:aa', '2026-01-01T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .unwrap();
        let payload = format!(r#"{{"n":1,"password":"{SENTINEL}"}}"#);
        sqlx::query(
            "INSERT INTO jobs (id, kind, payload_json, state, priority, attempts, \
             max_attempts, available_at, created_at) \
             VALUES ('job-1', 'system.noop_test', ?, 'Queued', 0, 0, 5, \
             '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        )
        .bind(&payload)
        .execute(&pool)
        .await
        .unwrap();
        let subject = format!("urn:qai:test:password={SENTINEL}");
        sqlx::query(
            "INSERT INTO audit_events (id, sequence, occurred_at, actor_kind, actor_id, \
             action, subject_urn, outcome, prev_chain_hash, chain_hash) \
             VALUES ('00000000-0000-4000-8000-000000000001', 1, '2026-01-01T00:00:00Z', \
             'system', 'test', 'config_change', ?, 'allowed', 'sha256:00', 'sha256:ab')",
        )
        .bind(&subject)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)").execute(&pool).await.unwrap();
        pool.close().await;
    });
    let before = std::fs::read(&path).unwrap();

    let mut combined = Vec::new();
    for args in [
        &["source", "list"][..],
        &["source", "list", "--json"][..],
        &["source", "show", "src-1"][..],
        &["source", "show", "src-1", "--json"][..],
        &["job", "list"][..],
        &["job", "list", "--json"][..],
        &["job", "show", "job-1"][..],
        &["job", "show", "job-1", "--json"][..],
        &["audit", "list"][..],
        &["audit", "list", "--json"][..],
        &["secret", "list"][..],
    ] {
        let out = qai(dir.path(), args, &[]);
        assert!(out.status.success(), "args {args:?}: {}", String::from_utf8_lossy(&out.stderr));
        combined.extend_from_slice(&out.stdout);
        combined.extend_from_slice(&out.stderr);
    }
    let text = String::from_utf8_lossy(&combined).into_owned();
    assert!(!text.contains(SENTINEL), "no read path leaks the sentinel");
    assert!(text.contains("***REDACTED***"), "redaction marker present");
    assert!(text.contains("src-1") && text.contains("job-1"));

    let out = qai(dir.path(), &["job", "show", "job-1", "--json"], &[]);
    let job: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(job["id"], "job-1");
    assert_eq!(job["kind"], "system.noop_test");

    let out = qai(dir.path(), &["source", "show", "src-1", "--json"], &[]);
    let source: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(source["versions"].as_array().unwrap().len(), 1);

    assert_eq!(before, std::fs::read(&path).unwrap(), "read paths never mutate");
}
