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
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
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
    assert!(out.status.success(), "db migrate must succeed: {}", String::from_utf8_lossy(&out.stderr));
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
