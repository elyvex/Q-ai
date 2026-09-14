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
