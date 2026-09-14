//! `tests/security/archive_guard.rs` — zip-slip, bomb, symlink-entry, depth.
//!
//! AC-P0-15: the archive guard rejects the full attack corpus.

use domain::security::{Limits, SecurityError};
use domain::security_archive::{ArchiveEntry, check_archive_entry};
use std::path::Path;

fn limits() -> Limits {
    Limits {
        max_download_bytes: 256 * 1024 * 1024,
        max_archive_entries: 20_000,
        max_archive_expansion_ratio: 100.0,
        max_archive_depth: 2,
    }
}

#[test]
fn zip_slip_entries_are_rejected() {
    let root = Path::new("/tmp/qai-extract");
    let slips = [
        "../../etc/passwd",
        "a/../../b",
        "..\\..\\windows\\system32",
        "/etc/passwd",
        "C:\\windows\\system32",
        "x/../../../../../../root/.ssh/id_rsa",
    ];
    for name in slips {
        let entry = ArchiveEntry::file(name, 10, 10);
        let r = check_archive_entry(root, &entry, 1, 0, &limits());
        assert!(
            matches!(r, Err(SecurityError::PathTraversal)),
            "expected PathTraversal for {name:?}, got {r:?}"
        );
    }
}

#[test]
fn symlink_entries_are_rejected() {
    let root = Path::new("/tmp/qai-extract");
    let entry = ArchiveEntry::symlink("link-to-etc");
    let r = check_archive_entry(root, &entry, 1, 0, &limits());
    assert!(matches!(r, Err(SecurityError::SymlinkEscape)));
}

#[test]
fn decompression_bomb_is_rejected() {
    let root = Path::new("/tmp/qai-extract");
    // 1 compressed byte → 10 MiB uncompressed, far above the 100× ratio.
    let entry = ArchiveEntry::file("bomb.bin", 1, 10 * 1024 * 1024);
    let r = check_archive_entry(root, &entry, 1, 0, &limits());
    assert!(matches!(r, Err(SecurityError::ExpansionRatio)));
}

#[test]
fn entry_count_cap_is_enforced() {
    let root = Path::new("/tmp/qai-extract");
    let entry = ArchiveEntry::file("x", 1, 1);
    let r = check_archive_entry(root, &entry, 20_001, 0, &limits());
    assert!(matches!(r, Err(SecurityError::EntryCount)));
}

#[test]
fn nested_archive_depth_cap_is_enforced() {
    let root = Path::new("/tmp/qai-extract");
    let entry = ArchiveEntry::file("inner.zip", 1, 1);
    let r = check_archive_entry(root, &entry, 1, 3, &limits());
    assert!(matches!(r, Err(SecurityError::ArchiveTooDeep)));
}

#[test]
fn benign_entry_is_accepted() {
    let root = Path::new("/tmp/qai-extract");
    let entry = ArchiveEntry::file("corpus/hafs.json", 100, 100);
    let r = check_archive_entry(root, &entry, 1, 0, &limits());
    assert!(r.is_ok(), "benign entry should be accepted: {r:?}");
}
