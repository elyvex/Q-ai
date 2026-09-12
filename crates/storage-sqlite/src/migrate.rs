//! Migration discovery and utilities for the SQLite storage backend.
//!
//! This module provides `discover_migrations` and `sha256_file` used by
//! [`SqliteDatabase::apply_migrations`] in `lib.rs`. The `xtask migrate-check`
//! command (in `xtask/src/migrate.rs`) shares the same discovery logic.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};

/// Discovery result: version -> (up_file, down_opt).
pub type Discovered = BTreeMap<u32, (PathBuf, Option<PathBuf>)>;

/// Parse `migrations/sqlite` into a map of version -> (up, down).
pub fn discover_migrations(dir: &Path) -> Result<Discovered, String> {
    let mut map: Discovered = BTreeMap::new();
    if !dir.exists() {
        return Ok(map);
    }
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()).map(String::from) else {
            continue;
        };
        let is_down = name.ends_with(".down.sql");
        let is_up = !is_down && name.ends_with(".up.sql");
        if !(is_up || is_down) {
            continue;
        }
        let stem = name
            .strip_suffix(".down.sql")
            .or_else(|| name.strip_suffix(".up.sql"))
            .unwrap_or(&name);
        let Some((ver_str, _)) = stem.split_once('_') else {
            return Err(format!("QAI-DB-0001: migration filename lacks a version prefix: {name}"));
        };
        let version: u32 = ver_str
            .parse()
            .map_err(|_| format!("QAI-DB-0001: non-numeric migration version in {name}"))?;
        let slot = map.entry(version).or_insert((PathBuf::new(), None));
        if is_down {
            slot.1 = Some(path);
        } else {
            slot.0 = path;
        }
    }
    Ok(map)
}

/// sha256 hex of the file's bytes.
pub fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let digest = Sha256::digest(&bytes);
    Ok(hex_lower(&digest))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("qai_migrate_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn empty_dir_is_ok() {
        let dir = scratch_dir("empty");
        let found = discover_migrations(&dir).unwrap();
        assert!(found.is_empty());
    }

    #[test]
    fn discovers_migrations() {
        let dir = scratch_dir("discovered");
        std::fs::write(dir.join("0001_core.up.sql"), "-- a").unwrap();
        let found = discover_migrations(&dir).unwrap();
        assert_eq!(found.len(), 1);
        assert!(found.contains_key(&1));
    }

    #[test]
    fn sha256_returns_hex() {
        let dir = scratch_dir("hash");
        let path = dir.join("test.sql");
        std::fs::write(&path, "hello").unwrap();
        let hash = sha256_file(&path).unwrap();
        assert_eq!(hash.len(), 64);
    }
}
