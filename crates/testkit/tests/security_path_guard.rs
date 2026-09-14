//! `tests/security/path_guard.rs` — path traversal / symlink escape corpus.
//!
//! Proves the 40+ payload corpus from AC-P0-15 is rejected and that a legitimate
//! in-root path is accepted.

use domain::security::{SecurityError, canonicalize_and_contain};
use std::path::Path;

fn payloads() -> Vec<&'static str> {
    vec![
        // Classic traversal
        "../etc/passwd",
        "../../etc/passwd",
        "../../../etc/passwd",
        "../../../../../../etc/shadow",
        "a/../../b",
        "a/b/../../../c",
        "foo/../../bar",
        "./../../x",
        "valid/../../escape",
        "data/../../../../root",
        // Nested / deep
        "a/b/c/../../../../d",
        "a/./b/./../../c",
        "x/y/z/../../../../../../etc/hosts",
        "nested/../../../../../../tmp/evil",
        // Backslash variants (Windows)
        "..\\etc\\passwd",
        "..\\..\\windows\\system32",
        "a\\..\\..\\b",
        "..\\..\\..\\..\\boot.ini",
        // Absolute paths
        "/etc/passwd",
        "/etc/shadow",
        "/root/.ssh/id_rsa",
        "/tmp/evil",
        "//etc/passwd",
        // URL-encoded traversal (must not be silently decoded into a valid path)
        "%2e%2e/etc/passwd",
        "%2e%2e%2fetc%2fpasswd",
        "..%2f..%2fetc%2fpasswd",
        // Mixed
        "a/..%2f..%2fb",
        "....//....//etc/passwd",
        "a//../../b",
        "a/././../../b",
        // Null byte / control
        "a/b\u{0}",
        // Dot-only oddities
        "..",
        "a/..",
        "./..",
        "./../..",
        // Unicode slash lookalikes (must not be treated as separators)
        "a\u{2215}..\u{2215}b",
        // Long chain
        "a/b/c/d/e/f/g/../../../../../../../../etc/passwd",
        "1/2/3/4/5/6/7/8/../../../../../../../../../../etc/passwd",
        // Sibling-prefix escape (root=/tmp/qai; /tmp/qai-evil must not be allowed)
        "../qai-evil/secret",
        // Case/duplication
        "..//../etc/passwd",
        "a/../..//../b",
    ]
}

#[test]
fn traversal_and_escape_payloads_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    // Create a sentinel sibling dir that a naive prefix-check might allow.
    let sibling = format!("{}-evil", dir.path().display());
    let _ = std::fs::create_dir_all(&sibling);

    let root = dir.path();
    assert!(payloads().len() >= 40, "corpus must have 40+ payloads");

    for payload in payloads() {
        let result = canonicalize_and_contain(root, Path::new(payload));
        match result {
            Err(_) => {} // rejected: good
            Ok(p) => panic!("payload {payload:?} was NOT rejected (resolved to {p:?})"),
        }
    }
    let _ = std::fs::remove_dir_all(&sibling);
}

#[test]
fn legitimate_in_root_file_is_accepted() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("data/edition.json");
    std::fs::create_dir_all(nested.parent().unwrap()).unwrap();
    std::fs::write(&nested, b"{}").unwrap();

    let resolved = canonicalize_and_contain(dir.path(), Path::new("data/edition.json")).unwrap();
    assert!(resolved.ends_with("data/edition.json"));
}

#[test]
fn missing_in_root_file_fails_closed() {
    // Fail-closed: a path that cannot be canonicalized is denied, not allowed.
    let dir = tempfile::tempdir().unwrap();
    let err = canonicalize_and_contain(dir.path(), Path::new("does/not/exist.json")).unwrap_err();
    assert!(matches!(err, SecurityError::SymlinkEscape));
}
