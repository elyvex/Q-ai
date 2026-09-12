//! `cargo xtask migrate` / `cargo xtask migrate-check` — migration management.
//!
//! Migration files live in `migrations/sqlite/` as `NNNN_name.up.sql` + optional
//! `NNNN_name.down.sql`. Two properties must hold:
//!
//! 1. **Ordering**: versions are unique, contiguous from 1, and monotonic.
//! 2. **Append-only**: the published checksum manifest `checksums.json` records the sha256
//!    of each migration file; any drift between the manifest and the on-disk content for an
//!    already-recorded file is a hard failure (`QAI-DB-0003` is raised at runtime by the
//!    migration runner; this xtask is the CI gate that blocks the edit before merge).
//!
//! `cargo xtask migrate` applies pending migrations to the SQLite database.
//! `cargo xtask migrate-check` only validates ordering and checksums.

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const MIGRATIONS_SUBDIR: &str = "migrations/sqlite";
pub const CHECKSUMS_FILE: &str = "checksums.json";

/// Discovery result: version -> (up_file, down_opt).
type Discovered = BTreeMap<u32, (PathBuf, Option<PathBuf>)>;

/// Parse `migrations/sqlite` into a map of version -> (up, down).
pub fn discover_migrations(dir: &Path) -> Result<Discovered> {
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

/// Apply all pending migrations to the SQLite database.
pub fn apply() -> Result<()> {
    let root = std::env::current_dir()?;
    let migrations_dir = root.join(MIGRATIONS_SUBDIR);
    let files = discover_migrations(&migrations_dir)?;

    if files.is_empty() {
        println!("migrate: OK — no pending migrations.");
        return Ok(());
    }

    // Check if the database exists and has schema_migrations.
    let db_path = std::env::var("QAI_DB_PATH").unwrap_or_else(|_| {
        std::env::var("XDG_DATA_HOME")
            .map(|p| format!("{p}/qai/qai.db"))
            .unwrap_or_else(|_| "~/.local/share/qai/qai.db".to_string())
    });

    // Use the storage-sqlite crate to apply migrations.
    // We import it here to avoid a compile-time dependency on the full crate
    // from xtask. Instead, we use sqlx directly.
    apply_migrations_sqlx(&migrations_dir, &db_path)
}

/// Apply migrations using sqlx directly.
fn apply_migrations_sqlx(migrations_dir: &Path, db_path: &str) -> Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let url = if db_path == "~/.local/share/qai/qai.db" {
            format!("sqlite:{}?mode=rwc", dirs::home_dir().map(|h| h.join(".local/share/qai/qai.db").to_string_lossy().to_string()).unwrap_or_else(|| "/tmp/qai.db".to_string()))
        } else {
            format!("sqlite://{db_path}")
        };

        let pool = sqlx::SqlitePool::connect(&url).await?;

        // Ensure schema_migrations table exists.
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                checksum TEXT NOT NULL,
                applied_at TEXT NOT NULL,
                applied_by TEXT NOT NULL,
                duration_ms INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await?;

        let applied: Vec<i64> = sqlx::query_scalar("SELECT version FROM schema_migrations ORDER BY version")
            .fetch_all(&pool)
            .await
            .unwrap_or_default();
        let applied_set: std::collections::HashSet<i64> = applied.iter().copied().collect();

        for (version, (up_path, _)) in files.iter() {
            if applied_set.contains(&(*version as i64)) {
                continue;
            }

            let sql = std::fs::read_to_string(up_path)?;
            let start = std::time::Instant::now();

            sqlx::query(&sql).execute(&pool).await
                .map_err(|e| anyhow::anyhow!("migration {}: {}", version, e))?;

            let checksum = sha256_file(up_path)?;
            let now = time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default();

            sqlx::query(
                "INSERT INTO schema_migrations (version, name, checksum, applied_at, applied_by, duration_ms)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(*version as i64)
            .bind(up_path.file_name().and_then(|f| f.to_str()).unwrap_or(""))
            .bind(checksum)
            .bind(now)
            .bind("qai")
            .bind(start.elapsed().as_millis() as i64)
            .execute(&pool)
            .await?;

            println!("Applied migration {version}");
        }

        sqlx::query("PRAGMA foreign_keys = ON").execute(&pool).await?;

        Ok::<_, anyhow::Error>(())
    })
}

/// sha256 hex of the file's bytes.
pub fn sha256_file(path: &Path) -> Result<String> {
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

/// Canonical manifest key for a migration file: `<version>_<name>.<up|down>.sql`.
fn manifest_key(path: &Path, kind: &str) -> String {
    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
    let base = file_name
        .strip_suffix(".up.sql")
        .or_else(|| file_name.strip_suffix(".down.sql"))
        .unwrap_or(&file_name);
    format!("{base}.{kind}.sql")
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
