//! `tests/config/precedence.rs` — CLI > Env > File > Defaults.
//!
//! AC-P0-04: the precedence contract holds and `config show --explain` can
//! attribute every value to its origin.

use config::{Config, ValueOrigin};
use std::collections::BTreeMap;

#[test]
fn defaults_apply_with_no_file_env_or_cli() {
    let overrides = BTreeMap::new();
    let (cfg, origins) = Config::load(None, "QAI_TEST_NONE", &overrides).unwrap();
    assert_eq!(cfg.server.bind, "127.0.0.1");
    assert_eq!(cfg.server.port, 8737);
    assert_eq!(origins.get("server.bind"), Some(&ValueOrigin::Default));
}

#[test]
fn file_overrides_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "[server]\nport = 4242\n").unwrap();

    let overrides = BTreeMap::new();
    let (cfg, origins) = Config::load(Some(&path), "QAI_TEST_FILE", &overrides).unwrap();
    assert_eq!(cfg.server.port, 4242);
    assert!(
        matches!(origins.get("server.port"), Some(ValueOrigin::File { .. })),
        "origin should be the file: {:?}",
        origins.get("server.port")
    );
}

#[test]
fn cli_overrides_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "[server]\nport = 4242\n").unwrap();

    let mut overrides = BTreeMap::new();
    overrides.insert("server.port".to_string(), "5151".to_string());
    let (cfg, origins) = Config::load(Some(&path), "QAI_TEST_CLI", &overrides).unwrap();
    assert_eq!(cfg.server.port, 5151);
    assert!(
        matches!(origins.get("server.port"), Some(ValueOrigin::Cli(_))),
        "origin should be CLI: {:?}",
        origins.get("server.port")
    );
}

#[test]
fn invalid_file_config_is_rejected_actionably() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    // Non-loopback bind with TLS disabled and auth-required is unsafe.
    std::fs::write(&path, "[server]\nbind = \"0.0.0.0\"\ntls = \"disabled\"\n").unwrap();

    let overrides = BTreeMap::new();
    let err = Config::load(Some(&path), "QAI_TEST_BAD", &overrides).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("0.0.0.0"), "error must name the offending value: {msg}");
}

#[test]
fn explain_map_contains_every_set_key() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "[logging]\nlevel = \"debug\"\n").unwrap();
    let overrides = BTreeMap::new();
    let (_cfg, origins) = Config::load(Some(&path), "QAI_TEST_EXPLAIN", &overrides).unwrap();
    let explain = origins.explain();
    assert!(explain.contains("logging.level"));
    assert!(!origins.is_empty());
}

// ─── Plan 01-01-02 matrix: defaults/file/env/CLI adjacency, empty and
// malformed input, origin totality, and nested secret redaction (D-07) ───

#[test]
fn missing_file_path_falls_back_to_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("no-such-config.toml");
    let (cfg, origins) =
        Config::load(Some(&missing), "QAI_TEST_MISSING", &BTreeMap::new()).unwrap();
    assert_eq!(cfg.server.port, 8737);
    assert_eq!(origins.get("server.port"), Some(&ValueOrigin::Default));
}

#[test]
fn malformed_toml_is_rejected_with_a_parse_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "[server\nport = \n").unwrap();

    let err = Config::load(Some(&path), "QAI_TEST_MALFORMED", &BTreeMap::new()).unwrap_err();
    assert!(
        matches!(err, config::ConfigError::Parse(_)),
        "malformed TOML must surface a Parse error: {err}"
    );
}

#[test]
fn invalid_values_are_rejected_with_a_validation_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "[server]\nbind = \"0.0.0.0\"\ntls = \"disabled\"\n").unwrap();

    let err = Config::load(Some(&path), "QAI_TEST_INVALID", &BTreeMap::new()).unwrap_err();
    assert!(
        matches!(err, config::ConfigError::Validation(_)),
        "unsafe bind must surface a Validation error: {err}"
    );
    assert!(!err.to_string().is_empty(), "validation error must carry a message");
}

fn flatten_keys(value: &serde_json::Value, prefix: &str, out: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, nested) in map {
                let dotted =
                    if prefix.is_empty() { key.clone() } else { format!("{prefix}.{key}") };
                flatten_keys(nested, &dotted, out);
            }
        }
        _ => out.push(prefix.to_string()),
    }
}

#[test]
fn every_serialized_leaf_has_an_origin() {
    // Origin totality (D-07): `mark_defaults` must cover every leaf the
    // effective document can serialize, so no leaf is ever origin-less.
    let (cfg, origins) = Config::load(None, "QAI_TEST_TOTAL", &BTreeMap::new()).unwrap();
    let value = serde_json::to_value(&cfg).unwrap();
    let mut leaves = Vec::new();
    flatten_keys(&value, "", &mut leaves);
    assert!(!leaves.is_empty(), "serialized config must have leaves");
    for leaf in &leaves {
        assert!(origins.get(leaf).is_some(), "serialized leaf `{leaf}` has no origin recorded");
    }
}

#[test]
fn nested_secret_values_are_redacted_before_emission() {
    const SENTINEL: &str = "SENTINEL_9f3c__DO_NOT_LEAK";
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(
        &path,
        format!("[logging]\nfile = \"postgresql://admin:{SENTINEL}@localhost:5432/qai\"\n"),
    )
    .unwrap();

    let (cfg, _origins) = Config::load(Some(&path), "QAI_TEST_REDACT", &BTreeMap::new()).unwrap();
    let mut envelope = serde_json::json!({
        "config": serde_json::to_value(&cfg).unwrap(),
        "origins": {},
    });
    domain::redaction::redact_json_value(&mut envelope);
    let rendered = serde_json::to_string(&envelope).unwrap();
    assert!(!rendered.contains(SENTINEL), "redacted envelope leaked: {rendered}");
    assert!(rendered.contains(domain::redaction::REDACTED_MARKER));
}
