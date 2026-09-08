//! `cargo xtask migrate-check` — enforce migrations are append-only and checksums stable.
//!
//! Migration files live in `migrations/sqlite/` as `NNNN_name.up.sql` + optional
//! `NNNN_name.down.sql`. Two properties must hold:
//!
//! 1. **Ordering**: versions are unique, contiguous from 1, and monotonic.
//! 2. **Append-only**: the published checksum manifest `checksums.json` records the sha256
//!    of each migration file; any drift between the manifest and the on-disk content for an
//!    already-recorded file is a hard failure (`QAI-DB-0003` is raised at runtime by the
//!    migration runner; this xtask is the CI gate that blocks the edit before merge).

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const MIGRATIONS_SUBDIR: &str = "migrations/sqlite";
pub const CHECKSUMS_FILE: &str = "checksums.json";

/// Discovery result: version -> (up_file, down_opt).
type Discovered = BTreeMap<u32, (PathBuf, Option<PathBuf>)>;

/// Parse `migrations/sqlite` into a map of version -> (up, down).
fn discover_migrations(dir: &Path) -> Result<Discovered> {
    let mut map: Discovered = BTreeMap::new();
    if !dir.exists() {
        return Ok(map);
    }
    for entry in std::fs::read_dir(dir).with_context(|| format!("read_dir {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()).map(String::from) else {
            continue;
        };
        // Only migration SQL files (skip checksums.json and any dotfiles / non-sql).
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
            bail!("QAI-DB-0001: migration filename lacks a version prefix: {name}");
        };
        let version: u32 = ver_str
            .parse()
            .with_context(|| format!("QAI-DB-0001: non-numeric migration version in {name}"))?;
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
fn sha256_file(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
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

/// Canonical manifest key for a migration file: `<version>_<name>.<up|down>.sql`.
fn manifest_key(path: &Path, kind: &str) -> String {
    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
    let base = file_name
        .strip_suffix(".up.sql")
        .or_else(|| file_name.strip_suffix(".down.sql"))
        .unwrap_or(&file_name);
    format!("{base}.{kind}.sql")
}

/// Validate ordering and (if a manifest exists) checksum stability.
pub fn run(dir: Option<&Path>) -> Result<()> {
    let root = std::env::current_dir()?;
    let migrations_dir = match dir {
        Some(d) => d.to_path_buf(),
        None => root.join(MIGRATIONS_SUBDIR),
    };

    let discovered = discover_migrations(&migrations_dir)?;
    if discovered.is_empty() {
        println!("migrate-check: OK — no migrations found (fresh checkout).");
        return Ok(());
    }

    // Ordering: versions must be contiguous from 1.
    for (expected, version) in (1u32..).zip(discovered.keys()) {
        if *version != expected {
            bail!(
                "QAI-DB-0002: migration versions must be contiguous from 1; expected {}, found {}",
                expected,
                version
            );
        }
    }

    // Checksum stability against the published manifest.
    let manifest_path = migrations_dir.join(CHECKSUMS_FILE);
    if manifest_path.exists() {
        let manifest: BTreeMap<String, String> = serde_json::from_str(
            &std::fs::read_to_string(&manifest_path)
                .with_context(|| format!("read {}", manifest_path.display()))?,
        )
        .context("QAI-DB-0004: checksums.json is not a map of filename->sha256")?;

        for (up, down) in discovered.values() {
            for (opt_file, kind) in [(Some(up), "up"), (down.as_ref(), "down")] {
                let Some(file) = opt_file else { continue };
                let key = manifest_key(file, kind);
                let Some(recorded) = manifest.get(&key) else {
                    continue; // not yet recorded -> newly appended, allowed
                };
                let actual = sha256_file(file)?;
                if recorded != &actual {
                    bail!(
                        "QAI-DB-0003: checksum drift for {} ({kind}); editing an applied \
                         migration is forbidden",
                        file.display()
                    );
                }
            }
        }
    }

    println!("migrate-check: OK — {} migration(s) ordered; checksums stable.", discovered.len());
    Ok(())
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
        assert!(run(Some(&dir)).is_ok());
    }

    #[test]
    fn numbers_must_be_contiguous() {
        let dir = scratch_dir("contig");
        std::fs::write(dir.join("0001_core.up.sql"), "-- a").unwrap();
        std::fs::write(dir.join("0003_broken.up.sql"), "-- b").unwrap();
        let err = run(Some(&dir)).unwrap_err();
        assert!(err.to_string().contains("QAI-DB-0002"), "{err}");
    }

    #[test]
    fn manifest_drift_is_detected() {
        let dir = scratch_dir("drift");
        std::fs::write(dir.join("0001_core.up.sql"), "-- original").unwrap();
        // Record a matching manifest, then corrupt the file to simulate an edit.
        let key = "0001_core.up.sql";
        let digest = sha256_file(&dir.join("0001_core.up.sql")).unwrap();
        let mut manifest: BTreeMap<String, String> = BTreeMap::new();
        manifest.insert(key.to_string(), digest);
        std::fs::write(dir.join(CHECKSUMS_FILE), serde_json::to_string(&manifest).unwrap())
            .unwrap();
        assert!(run(Some(&dir)).is_ok(), "clean manifest must pass");

        std::fs::write(dir.join("0001_core.up.sql"), "-- edited after record").unwrap();
        let err = run(Some(&dir)).unwrap_err();
        assert!(err.to_string().contains("QAI-DB-0003"), "{err}");
    }
}
