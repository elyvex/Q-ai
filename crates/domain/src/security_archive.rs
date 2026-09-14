//! Archive safety guard (D0.15 / T43).
//!
//! Blocks zip-slip path traversal, archive bombs (expansion ratio), entry-count
//! exhaustion, symlink/device escapes, and nested-archive depth attacks. The
//! functions are pure: the caller extracts entry metadata from the archive
//! format (zip, tar, …) and passes it here. Network/IO stays outside `domain`.
//!
//! **Fail-closed:** any entry that cannot be positively validated is rejected.

use std::path::{Path, PathBuf};

use crate::security::{Limits, SecurityError};

/// Metadata for one archive entry, format-independent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    /// The entry's declared name/path inside the archive.
    pub name: String,
    /// Compressed size in bytes (the bytes physically stored).
    pub compressed_size: u64,
    /// Uncompressed size in bytes (the bytes produced on extraction).
    pub uncompressed_size: u64,
    /// Whether the entry is a symbolic link.
    pub is_symlink: bool,
    /// Whether the entry is a directory.
    pub is_directory: bool,
}

impl ArchiveEntry {
    /// A regular file entry.
    pub fn file(name: impl Into<String>, compressed_size: u64, uncompressed_size: u64) -> Self {
        Self {
            name: name.into(),
            compressed_size,
            uncompressed_size,
            is_symlink: false,
            is_directory: false,
        }
    }

    /// A symlink entry.
    pub fn symlink(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            compressed_size: 0,
            uncompressed_size: 0,
            is_symlink: true,
            is_directory: false,
        }
    }
}

/// Validate a single archive entry before extraction.
///
/// Checks, in order:
/// 1. entry-count cap (`entries_seen`, 1-based count including this entry)
/// 2. rejected link/device entries
/// 3. path-traversal (zip-slip) via `..` / absolute paths / Windows drive prefixes
/// 4. expansion-ratio bomb
/// 5. nested-archive depth cap (caller supplies current depth)
///
/// Returns the safe extraction path on success.
pub fn check_archive_entry(
    root: &Path,
    entry: &ArchiveEntry,
    entries_seen: usize,
    depth: u32,
    limits: &Limits,
) -> Result<PathBuf, SecurityError> {
    // 1. Entry count.
    limits.check_entry_count(entries_seen)?;

    // 2. Symlinks and device entries are never extracted.
    if entry.is_symlink {
        return Err(SecurityError::SymlinkEscape);
    }

    // Empty names are nonsensical.
    if entry.name.is_empty() {
        return Err(SecurityError::PathTraversal);
    }

    // 3. Zip-slip / path traversal.
    let entry_path = Path::new(&entry.name);
    if entry_path.is_absolute() {
        return Err(SecurityError::PathTraversal);
    }
    // Windows drive prefix (e.g. `C:\...`) or backslash separators.
    if entry.name.contains('\\') || entry.name.as_bytes().get(1).is_some_and(|b| *b == b':') {
        return Err(SecurityError::PathTraversal);
    }
    let mut depth_budget: i32 = 0;
    for component in entry_path.components() {
        match component {
            std::path::Component::ParentDir => {
                depth_budget -= 1;
                if depth_budget < 0 {
                    return Err(SecurityError::PathTraversal);
                }
            }
            std::path::Component::Normal(_) => depth_budget += 1,
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                return Err(SecurityError::PathTraversal);
            }
            std::path::Component::CurDir => {}
        }
    }

    // 4. Expansion-ratio bomb.
    limits.check_expansion(entry.compressed_size, entry.uncompressed_size)?;

    // 5. Nested-archive depth cap.
    if depth > limits.max_archive_depth {
        return Err(SecurityError::ArchiveTooDeep);
    }

    let candidate = root.join(entry_path);
    // Defense in depth: confirm the lexical join stays under root.
    if let Ok(canonical) = std::fs::canonicalize(root) {
        let root_str = canonical.to_string_lossy();
        if !candidate.to_string_lossy().starts_with(root_str.as_ref()) {
            return Err(SecurityError::PathTraversal);
        }
    }
    Ok(candidate)
}

/// Validate aggregate extraction limits after processing entries.
pub fn check_archive_extraction(
    entries_seen: usize,
    bytes_compressed: u64,
    bytes_decompressed: u64,
    limits: &Limits,
) -> Result<(), SecurityError> {
    limits.check_entry_count(entries_seen)?;
    limits.check_expansion(bytes_compressed, bytes_decompressed)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::Limits;

    fn limits() -> Limits {
        Limits {
            max_download_bytes: 256 * 1024 * 1024,
            max_archive_entries: 20_000,
            max_archive_expansion_ratio: 100.0,
            max_archive_depth: 2,
        }
    }

    #[test]
    fn zip_slip_is_rejected() {
        let root = Path::new("/tmp/qai-extract");
        for name in [
            "../../etc/passwd",
            "a/../../../b",
            "..\\windows\\system32",
            "/absolute/path",
            "C:\\windows",
        ] {
            let entry = ArchiveEntry::file(name, 10, 10);
            let r = check_archive_entry(root, &entry, 1, 0, &limits());
            assert!(
                matches!(r, Err(SecurityError::PathTraversal)),
                "expected rejection for {name}, got {r:?}"
            );
        }
    }

    #[test]
    fn symlink_entry_is_rejected() {
        let root = Path::new("/tmp/qai-extract");
        let entry = ArchiveEntry::symlink("link");
        let r = check_archive_entry(root, &entry, 1, 0, &limits());
        assert!(matches!(r, Err(SecurityError::SymlinkEscape)));
    }

    #[test]
    fn normal_entry_is_accepted() {
        let root = Path::new("/tmp/qai-extract");
        let entry = ArchiveEntry::file("data/edition.json", 10, 10);
        let r = check_archive_entry(root, &entry, 1, 0, &limits());
        assert!(r.is_ok(), "expected ok, got {r:?}");
    }

    #[test]
    fn entry_count_limit_is_enforced() {
        let root = Path::new("/tmp/qai-extract");
        let entry = ArchiveEntry::file("x", 1, 1);
        let r = check_archive_entry(root, &entry, 20_001, 0, &limits());
        assert!(matches!(r, Err(SecurityError::EntryCount)));
    }

    #[test]
    fn expansion_ratio_limit_is_enforced() {
        let root = Path::new("/tmp/qai-extract");
        // 1 compressed -> 200 uncompressed exceeds ratio of 100.
        let entry = ArchiveEntry::file("bomb", 1, 200);
        let r = check_archive_entry(root, &entry, 1, 0, &limits());
        assert!(matches!(r, Err(SecurityError::ExpansionRatio)));
    }

    #[test]
    fn nested_depth_limit_is_enforced() {
        let root = Path::new("/tmp/qai-extract");
        let entry = ArchiveEntry::file("inner.zip", 1, 1);
        let r = check_archive_entry(root, &entry, 1, 3, &limits());
        assert!(matches!(r, Err(SecurityError::ArchiveTooDeep)));
    }
}
