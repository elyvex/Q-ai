//! Phase 1 foundation acceptance — real `qai` binary, fresh workspace.
//!
//! Plan 01-01 (TASK-001) tracer and expansions. Every case invokes the real
//! `CARGO_BIN_EXE_qai` process against an isolated temp workspace and asserts
//! the operator-visible contract:
//!
//! - the seven stable foundation groups are present in `--help` (D-08);
//! - effective configuration carries file/env values *and* their origins, with
//!   secrets redacted before emission (D-04, D-07);
//! - `qai doctor` is read-only on a fresh workspace and names `qai db migrate`
//!   for a missing database (D-05, D-06);
//! - `qai db migrate` is the only explicit creation path, and the resulting
//!   empty audit chain verifies (D-05, D-12).

use std::path::Path;
use std::process::Output;

const SENTINEL: &str = "SENTINEL_9f3c__DO_NOT_LEAK";

/// Run the real `qai` binary with an explicit argument list.
fn run(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_qai"));
    cmd.args(args);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    cmd.output().expect("run qai")
}

/// Run `qai` with `--data-dir <dir>` prepended for the common workspace case.
fn qai(dir: &Path, args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut full: Vec<&str> = vec!["--data-dir", dir.to_str().unwrap()];
    full.extend_from_slice(args);
    run(&full, envs)
}

fn stdout_json(out: &Output) -> serde_json::Value {
    assert!(
        out.status.success(),
        "expected success (code {:?}): {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("stdout is JSON")
}

fn check<'a>(doc: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    doc["checks"]
        .as_array()
        .expect("checks array")
        .iter()
        .find(|c| c["id"] == id)
        .unwrap_or_else(|| panic!("check `{id}` not found in {doc}"))
}

#[test]
fn help_lists_stable_foundation_groups() {
    let dir = tempfile::tempdir().unwrap();
    let out = qai(dir.path(), &["--help"], &[]);
    assert!(out.status.success(), "qai --help must succeed");
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    for group in ["config", "db", "doctor", "job", "audit", "secret", "source"] {
        assert!(text.contains(group), "`qai --help` must expose group `{group}`:\n{text}");
    }
}

#[test]
fn effective_config_reports_file_and_env_origin_without_leaking_secrets() {
    let dir = tempfile::tempdir().unwrap();
    let cfg_path = dir.path().join("config.toml");
    std::fs::write(
        &cfg_path,
        format!("[server]\nport = 4242\n[logging]\nfile = \"postgresql://admin:{SENTINEL}@localhost:5432/qai\"\n"),
    )
    .unwrap();
    let cfg = cfg_path.to_str().unwrap();
    let data = dir.path().join("data");
    let data = data.to_str().unwrap();

    // File layer wins over defaults and is attributed to the file.
    let out = run(&["--config", cfg, "--data-dir", data, "config", "show", "--json"], &[]);
    let doc = stdout_json(&out);
    assert_eq!(doc["config"]["server"]["port"], 4242);
    let origin = doc["origins"]["server.port"].as_str().unwrap_or_default();
    assert!(origin.contains("file"), "file origin expected, got `{origin}`");

    // Environment layer wins over the file and is attributed to the env var.
    let out = run(
        &["--config", cfg, "--data-dir", data, "config", "show", "--json"],
        &[("QAI__SERVER__PORT", "5151")],
    );
    let doc = stdout_json(&out);
    assert_eq!(doc["config"]["server"]["port"], 5151);
    let origin = doc["origins"]["server.port"].as_str().unwrap_or_default();
    assert!(origin.contains("env"), "env origin expected, got `{origin}`");

    // The secret sentinel never reaches stdout or stderr.
    let combined =
        format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    assert!(!combined.contains(SENTINEL), "config show leaked the sentinel:\n{combined}");
}

#[test]
fn doctor_on_fresh_workspace_is_read_only_and_names_migrate() {
    let dir = tempfile::tempdir().unwrap();
    let fresh = dir.path().join("fresh");
    let fresh = fresh.to_str().unwrap();

    let out = qai(Path::new(&fresh), &["doctor", "--json"], &[]);
    assert_eq!(out.status.code(), Some(3), "missing DB is a validation failure");

    assert!(!Path::new(fresh).exists(), "doctor must not create the data directory");
    assert!(!Path::new(fresh).join("qai.db").exists(), "doctor must not create a database");

    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("doctor JSON");
    let reachable = check(&doc, "database.reachable");
    assert_eq!(reachable["status"], "fail");
    let next = reachable["next_command"].as_str().unwrap_or_default();
    assert!(next.contains("qai db migrate"), "missing-DB remedy must name migrate, got `{next}`");
}

#[test]
fn explicit_migration_creates_schema_then_empty_audit_chain_verifies() {
    let dir = tempfile::tempdir().unwrap();

    let out = qai(dir.path(), &["db", "migrate"], &[]);
    assert!(
        out.status.success(),
        "db migrate must succeed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(dir.path().join("qai.db").exists(), "db migrate must create the database");

    let out = qai(dir.path(), &["audit", "verify", "--json"], &[]);
    assert!(out.status.success(), "empty chain must verify");
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("audit verify JSON");
    assert_eq!(doc["valid"], true);
    assert_eq!(doc["checked_events"], 0);
}

#[test]
fn doctor_after_migration_consumes_the_persisted_audit_verifier() {
    let dir = tempfile::tempdir().unwrap();
    let migrated = qai(dir.path(), &["db", "migrate"], &[]);
    assert!(migrated.status.success(), "db migrate must succeed");

    let out = qai(dir.path(), &["doctor", "--json"], &[]);
    assert!(out.status.success(), "healthy migrated workspace must pass doctor");
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("doctor JSON");
    let audit = check(&doc, "audit.chain_valid");
    assert_eq!(audit["status"], "pass");
}

// ─── Plan 01-01-02: effective configuration inspection (D-07, FND-01) ───

#[test]
fn config_get_returns_the_requested_value_with_its_origin() {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().to_str().unwrap();

    let out = run(&["--data-dir", data, "--json", "config", "get", "server.port"], &[]);
    assert!(out.status.success());
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("get JSON");
    assert_eq!(doc["key"], "server.port");
    assert_eq!(doc["value"], 8737);
    assert!(doc["origin"].is_string(), "keyed lookup must report its origin: {doc}");

    // Human output carries the value, not the full debug dump.
    let out = run(&["--data-dir", data, "config", "get", "server.port"], &[]);
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(text.contains("server.port = 8737"), "unexpected get output:\n{text}");
    assert!(!text.contains("SqliteConfig"), "get must not dump the whole config:\n{text}");

    // Unknown keys use the typed not-found contract in both modes.
    let out = run(&["--data-dir", data, "config", "get", "server.nope"], &[]);
    assert_eq!(out.status.code(), Some(5), "unknown key is not-found");
    let err = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(err.contains("QAI-CFG-0006"), "missing stable code:\n{err}");
    assert!(err.contains("qai config show --explain"), "missing next command:\n{err}");

    let out = run(&["--data-dir", data, "--json", "config", "get", "server.nope"], &[]);
    assert_eq!(out.status.code(), Some(5));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("get JSON");
    assert_eq!(doc["code"], "QAI-CFG-0006");
    assert!(!doc["remedy"].as_str().unwrap_or_default().is_empty());
    assert_eq!(doc["next_command"], "qai config show --explain");
}

#[test]
fn config_get_redacts_secret_shaped_values() {
    let dir = tempfile::tempdir().unwrap();
    let cfg_path = dir.path().join("config.toml");
    std::fs::write(
        &cfg_path,
        format!("[logging]\nfile = \"postgresql://admin:{SENTINEL}@localhost:5432/qai\"\n"),
    )
    .unwrap();
    let cfg = cfg_path.to_str().unwrap();
    let data = dir.path().join("data").to_str().unwrap().to_string();

    for extra in [vec!["--json"], vec![]] {
        let mut args = vec!["--config", cfg, "--data-dir", data.as_str()];
        args.extend_from_slice(&extra);
        args.extend_from_slice(&["config", "get", "logging.file"]);
        let out = run(&args, &[]);
        assert!(out.status.success());
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(!combined.contains(SENTINEL), "config get leaked the sentinel:\n{combined}");
    }
}

#[test]
fn config_show_explain_renders_values_with_origins() {
    let dir = tempfile::tempdir().unwrap();
    let cfg_path = dir.path().join("config.toml");
    std::fs::write(&cfg_path, "[server]\nport = 4242\n").unwrap();
    let cfg = cfg_path.to_str().unwrap();
    let data = dir.path().join("data").to_str().unwrap().to_string();

    let out = run(
        &["--config", cfg, "--data-dir", data.as_str(), "config", "show", "--explain"],
        &[("QAI__LOGGING__LEVEL", "debug")],
    );
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(text.contains("server.port = 4242 (file"), "file value+origin expected:\n{text}");
    assert!(text.contains("logging.level = \"debug\" (env"), "env value+origin expected:\n{text}");
}

#[test]
fn data_dir_overrides_derive_storage_keys_with_cli_origin() {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("custom").to_str().unwrap().to_string();

    let out = run(&["--data-dir", data.as_str(), "config", "show", "--json"], &[]);
    let doc = stdout_json(&out);
    let sqlite = doc["config"]["storage"]["sqlite"]["path"].as_str().unwrap_or_default();
    assert_eq!(sqlite, format!("{data}/qai.db"));
    let objects = doc["config"]["storage"]["objects"]["root"].as_str().unwrap_or_default();
    assert_eq!(objects, format!("{data}/objects"));
    for key in ["app.data_dir", "storage.sqlite.path", "storage.objects.root"] {
        let origin = doc["origins"][key].as_str().unwrap_or_default();
        assert!(origin.contains("cli"), "CLI origin expected for `{key}`, got `{origin}`");
    }
}

fn validate_shapes(out: &std::process::Output, expect_code: i32) -> (String, serde_json::Value) {
    assert_eq!(out.status.code(), Some(expect_code));
    let human =
        String::from_utf8_lossy(&out.stderr).into_owned() + &String::from_utf8_lossy(&out.stdout);
    (human, serde_json::Value::Null)
}

#[test]
fn config_validate_malformed_file_reports_same_fields_in_both_modes() {
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("bad.toml");
    std::fs::write(&bad, "[server\nport = \n").unwrap();
    let bad = bad.to_str().unwrap();
    let data = dir.path().join("data").to_str().unwrap().to_string();

    let out = run(&["--data-dir", data.as_str(), "config", "validate", "--file", bad], &[]);
    let (human, _) = validate_shapes(&out, 3);
    assert!(human.contains("QAI-CFG-0002"), "stable parse code expected:\n{human}");
    assert!(human.contains("remedy:"), "non-empty remedy expected:\n{human}");
    assert!(
        human.contains(&format!("qai config show --file {bad} --json")),
        "malformed-file next command expected:\n{human}"
    );

    let out =
        run(&["--data-dir", data.as_str(), "--json", "config", "validate", "--file", bad], &[]);
    assert_eq!(out.status.code(), Some(3));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("validate JSON");
    assert_eq!(doc["code"], "QAI-CFG-0002");
    assert!(!doc["remedy"].as_str().unwrap_or_default().is_empty());
    assert_eq!(doc["next_command"], format!("qai config show --file {bad} --json"));
}

#[test]
fn config_validate_rejected_value_reports_same_fields_in_both_modes() {
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("unsafe.toml");
    std::fs::write(&bad, "[server]\nbind = \"0.0.0.0\"\ntls = \"disabled\"\n").unwrap();
    let bad = bad.to_str().unwrap();
    let data = dir.path().join("data").to_str().unwrap().to_string();

    let out = run(&["--data-dir", data.as_str(), "config", "validate", "--file", bad], &[]);
    let (human, _) = validate_shapes(&out, 3);
    assert!(human.contains("QAI-CFG-0003"), "stable validation code expected:\n{human}");
    assert!(human.contains("remedy:"), "non-empty remedy expected:\n{human}");
    assert!(human.contains("next:"), "runnable next command expected:\n{human}");

    let out =
        run(&["--data-dir", data.as_str(), "--json", "config", "validate", "--file", bad], &[]);
    assert_eq!(out.status.code(), Some(3));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("validate JSON");
    assert_eq!(doc["code"], "QAI-CFG-0003");
    assert!(!doc["remedy"].as_str().unwrap_or_default().is_empty());
    assert!(!doc["next_command"].as_str().unwrap_or_default().is_empty());
}

// ─── Plan 01-01-03: persisted audit verification (D-12) ───
//
// Both tests seed a real migrated SQLite database, corrupt the hash chain on
// disk (tampered row vs sequence gap), then invoke the real `qai audit verify`
// binary in human and `--json` modes. Human and JSON must carry the same
// stable code, non-empty remedy, exact next command, and affected-sequence
// fields; every invalid chain exits `exit_code::VALIDATION` (3); the JSON
// form parses as one document.

/// Append `count` hash-chained audit events to a migrated database through
/// the same writer the application uses, so the chain verifies cleanly.
/// Lives in `application` (not here) to respect the arch-check boundary:
/// the CLI test target may not gain `audit`/`domain`/`storage-sqlite` edges.
fn seed_audit_chain(db_path: &str, count: u64) {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async {
        application::audit_bridge::seed_synthetic_chain(db_path, count).await.unwrap();
    });
}

/// Run one write statement against the database after dropping the
/// append-only triggers (test-only bypass of the SQL guardrail).
fn corrupt_audit_chain(db_path: &str, statement: &str) {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(db_path))
            .await
            .unwrap();
        for trigger in ["trg_audit_no_update", "trg_audit_no_delete"] {
            sqlx::query(&format!("DROP TRIGGER IF EXISTS {trigger}")).execute(&pool).await.unwrap();
        }
        sqlx::query(statement).execute(&pool).await.unwrap();
        pool.close().await;
    });
}

/// Migrate a temp workspace and return its data-dir string.
fn migrated_workspace() -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().to_str().unwrap().to_string();
    let out = run(&["--data-dir", data.as_str(), "db", "migrate"], &[]);
    assert!(out.status.success(), "db migrate must succeed");
    (dir, data)
}

/// Assert the human and JSON `audit verify` outputs carry the same stable
/// failure contract for one corrupted chain.
fn assert_verify_contract(data: &str, expect_code: &str, expect_sequences: &[u64]) {
    // Human mode: validation exit plus the typed diagnostic on stderr.
    let out = run(&["--data-dir", data, "audit", "verify"], &[]);
    assert_eq!(out.status.code(), Some(3), "invalid chain is a validation failure");
    let human =
        format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    assert!(human.contains(expect_code), "stable code expected:\n{human}");
    assert!(human.contains("remedy:"), "non-empty remedy expected:\n{human}");
    assert!(human.contains("next: qai audit verify"), "exact next command expected:\n{human}");
    for sequence in expect_sequences {
        assert!(
            human.contains(&sequence.to_string()),
            "affected sequence {sequence} expected:\n{human}"
        );
    }

    // JSON mode: one parseable document with the same fields.
    let out = run(&["--data-dir", data, "--json", "audit", "verify"], &[]);
    assert_eq!(out.status.code(), Some(3), "invalid chain is a validation failure");
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let doc: serde_json::Value =
        serde_json::from_str(&text).expect("audit verify JSON parses as one document");
    assert_eq!(doc["valid"], false);
    assert_eq!(doc["code"], expect_code);
    let remedy = doc["remedy"].as_str().unwrap_or_default();
    assert!(!remedy.is_empty(), "non-empty remedy expected: {doc}");
    assert_eq!(doc["next_command"], "qai audit verify");
    let affected = format!("{}{}", doc["gaps"], doc["tampered_sequences"]);
    for sequence in expect_sequences {
        assert!(
            affected.contains(&sequence.to_string()),
            "affected sequence {sequence} expected: {doc}"
        );
    }

    // Both modes agree on the stable contract.
    let human_remedy = human
        .lines()
        .find_map(|line| line.split_once("remedy:").map(|(_, rest)| rest.trim().to_string()))
        .unwrap_or_default();
    assert!(
        !human_remedy.is_empty() && remedy.contains(&human_remedy[..human_remedy.len().min(24)]),
        "human and JSON remedies must agree:\n{human}\n{doc}"
    );
}

#[test]
fn audit_verify_tampered_human_and_json() {
    let (_dir, data) = migrated_workspace();
    let db_path = format!("{data}/qai.db");
    seed_audit_chain(&db_path, 2);
    corrupt_audit_chain(&db_path, "UPDATE audit_events SET reason = 'tampered' WHERE sequence = 2");

    assert_verify_contract(&data, "QAI-AUD-0005", &[2]);

    // Doctor consumes the same persisted report: a tampered chain fails
    // `audit.chain_valid` and points at the authoritative verifier.
    let out = run(&["--data-dir", data.as_str(), "doctor", "--json"], &[]);
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("doctor JSON");
    let audit = check(&doc, "audit.chain_valid");
    assert_eq!(audit["status"], "fail");
    assert!(
        audit["next_command"].as_str().unwrap_or_default().contains("qai audit verify"),
        "doctor must point at the verifier: {audit}"
    );
}

#[test]
fn audit_verify_gap_human_and_json() {
    let (_dir, data) = migrated_workspace();
    let db_path = format!("{data}/qai.db");
    seed_audit_chain(&db_path, 3);
    corrupt_audit_chain(&db_path, "DELETE FROM audit_events WHERE sequence = 2");

    assert_verify_contract(&data, "QAI-AUD-0004", &[3]);
}

#[test]
fn ordinary_read_on_missing_database_names_migrate_without_creating_state() {
    // D-05 / T-01-MIG: ordinary application reads go through the
    // existing-database guard — no implicit creation, a typed
    // migration-required diagnostic (exit 3) naming `qai db migrate`.
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("nogdb").to_str().unwrap().to_string();

    let out = run(&["--data-dir", data.as_str(), "quran", "get", "2:255"], &[]);
    assert_eq!(out.status.code(), Some(3), "missing DB is a validation failure");
    let combined =
        format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    assert!(
        combined.contains("qai db migrate"),
        "ordinary read must name the migrate remedy:\n{combined}"
    );
    assert!(
        !Path::new(&format!("{data}/qai.db")).exists(),
        "ordinary read must not create a database"
    );
}

#[test]
fn cli_data_dir_beats_env_storage_path_for_the_same_key() {
    // Same-key adjacency at the real-binary boundary (D-07): the environment
    // sets `storage.sqlite.path`, but `--data-dir` (CLI) wins deterministically
    // and keeps its CLI origin. `unsafe_code = forbid` bars in-process
    // `set_var` in test targets, so adjacency above file/env is proven here
    // where the child process carries its own environment.
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("cli-data").to_str().unwrap().to_string();

    let out = run(
        &["--data-dir", data.as_str(), "config", "show", "--json"],
        &[("QAI__STORAGE__SQLITE__PATH", "/env-override/qai.db")],
    );
    let doc = stdout_json(&out);
    assert_eq!(
        doc["config"]["storage"]["sqlite"]["path"],
        serde_json::Value::String(format!("{data}/qai.db"))
    );
    let origin = doc["origins"]["storage.sqlite.path"].as_str().unwrap_or_default();
    assert!(origin.contains("cli"), "CLI origin expected, got `{origin}`");
}
