//! Phase 2 — trigram posting index for concatenated search (P2-T36, D2.4).
//!
//! The skeleton scan in `search_concatenated` probes every stored skeleton
//! with Rust `contains`. This module replaces the scan with a posting
//! index: at build time every skeleton contributes its character-trigram
//! set; at query time only skeletons containing **all** query trigrams are
//! read back for exact verification (recall, not precision — verification
//! still decides).
//!
//! Storage is generation-scoped: `<root>/gen-<N>/trigram.db` (SQLite table
//! `postings`), built during staging beside `index.db` and removed by the
//! same retention GC (P2-T35). Generations built before T36 have no file;
//! readers fall back to the Rust scan in that case (never an error).
//!
//! Trigrams are character (not byte) triples over the L6 skeleton. Queries
//! shorter than 3 chars carry no trigrams and probe the whole skeleton set
//! directly, exactly like the pre-T36 scan did.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Extract the sorted, deduplicated character-trigram set of a skeleton.
///
/// Skeletons shorter than 3 chars yield no trigrams (direct probe).
#[must_use]
pub fn trigrams_of(skeleton: &str) -> Vec<String> {
    let chars: Vec<char> = skeleton.chars().collect();
    if chars.len() < 3 {
        return Vec::new();
    }
    let mut set = BTreeSet::new();
    for window in chars.windows(3) {
        set.insert(window.iter().collect::<String>());
    }
    set.into_iter().collect()
}

/// One skeleton row address: ayah-level (`start == end`) or window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SkelAddr {
    /// 1-based surah number.
    pub surah: i64,
    /// 1-based window/ayah start.
    pub ayah_start: i64,
    /// 1-based window/ayah end.
    pub ayah_end: i64,
}

/// File name of the posting database inside a generation directory.
pub const TRIGRAM_FILE: &str = "trigram.db";

/// Path of the posting database for `generation` under `root`.
#[must_use]
pub fn trigram_path(root: &Path, generation: u64) -> PathBuf {
    root.join(format!("gen-{generation}")).join(TRIGRAM_FILE)
}

/// Build the posting database at `path` from `(address, skeleton)` rows.
///
/// Overwrites any previous file at `path` (staging always starts fresh).
/// Returns `(skeleton_count, posting_count)` for the build report.
///
/// # Errors
///
/// Returns an error string when the database cannot be created or filled.
pub async fn build_postings(
    path: &Path,
    rows: &[(SkelAddr, String)],
) -> Result<(u64, u64), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new().filename(path).create_if_missing(true),
        )
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query(
        "CREATE TABLE postings (
           trigram    TEXT NOT NULL,
           surah      INTEGER NOT NULL,
           ayah_start INTEGER NOT NULL,
           ayah_end   INTEGER NOT NULL
         )",
    )
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;
    sqlx::query("CREATE INDEX ix_postings_trigram ON postings(trigram)")
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    let mut posting_count: u64 = 0;
    // One transaction: a partial posting file is never left behind.
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    for (addr, skeleton) in rows {
        for trigram in trigrams_of(skeleton) {
            sqlx::query(
                "INSERT INTO postings (trigram, surah, ayah_start, ayah_end)
                 VALUES (?, ?, ?, ?)",
            )
            .bind(&trigram)
            .bind(addr.surah)
            .bind(addr.ayah_start)
            .bind(addr.ayah_end)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            posting_count += 1;
        }
    }
    tx.commit().await.map_err(|e| e.to_string())?;
    pool.close().await;
    Ok((rows.len() as u64, posting_count))
}

/// Recall skeleton addresses containing **all** `trigrams`.
///
/// Returns `None` when `path` does not exist (pre-T36 generation: caller
/// falls back to the Rust scan). An empty trigram list also returns `None`
/// (short queries probe directly).
///
/// # Errors
///
/// Returns an error string when the database cannot be read.
pub async fn recall(
    path: &Path,
    trigrams: &[String],
) -> Result<Option<BTreeSet<SkelAddr>>, String> {
    if trigrams.is_empty() {
        return Ok(None);
    }
    if !path.exists() {
        return Ok(None);
    }
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(path).read_only(true))
        .await
        .map_err(|e| e.to_string())?;
    // Intersect per-trigram posting sets in Rust over ordered row reads
    // (one indexed query per trigram; set sizes stay small in practice).
    let mut acc: Option<BTreeSet<SkelAddr>> = None;
    for trigram in trigrams {
        let rows: Vec<(i64, i64, i64)> =
            sqlx::query_as("SELECT surah, ayah_start, ayah_end FROM postings WHERE trigram = ?")
                .bind(trigram)
                .fetch_all(&pool)
                .await
                .map_err(|e| e.to_string())?;
        let set: BTreeSet<SkelAddr> = rows
            .into_iter()
            .map(|(surah, ayah_start, ayah_end)| SkelAddr { surah, ayah_start, ayah_end })
            .collect();
        acc = Some(match acc {
            None => set,
            Some(prev) => prev.intersection(&set).copied().collect(),
        });
        if acc.as_ref().is_some_and(BTreeSet::is_empty) {
            break;
        }
    }
    pool.close().await;
    Ok(acc)
}

/// Verify a built posting file: every skeleton's own trigram set resolves
/// back to an address set containing that skeleton (no false negatives at
/// the posting level; false positives are verification's job).
///
/// Returns the number of skeletons checked.
///
/// # Errors
///
/// Returns an error string on any false negative or I/O failure.
pub async fn verify_postings(path: &Path, rows: &[(SkelAddr, String)]) -> Result<u64, String> {
    let mut checked: u64 = 0;
    for (addr, skeleton) in rows {
        let trigrams = trigrams_of(skeleton);
        if trigrams.is_empty() {
            continue;
        }
        let hit = recall(path, &trigrams).await?.unwrap_or_default();
        if !hit.contains(addr) {
            return Err(format!(
                "trigram postings miss skeleton surah={} {}..{}",
                addr.surah, addr.ayah_start, addr.ayah_end
            ));
        }
        checked += 1;
    }
    Ok(checked)
}

/// Group posting rows per surah for staged writes (build-job helper).
#[must_use]
pub fn group_by_surah(rows: &[(SkelAddr, String)]) -> BTreeMap<i64, Vec<(SkelAddr, String)>> {
    let mut out: BTreeMap<i64, Vec<(SkelAddr, String)>> = BTreeMap::new();
    for (addr, skeleton) in rows {
        out.entry(addr.surah).or_default().push((*addr, skeleton.clone()));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trigrams_are_char_triples_sorted_deduped() {
        assert!(trigrams_of("").is_empty());
        assert!(trigrams_of("ab").is_empty());
        assert_eq!(trigrams_of("abc"), vec!["abc".to_string()]);
        assert_eq!(trigrams_of("aaaa"), vec!["aaa".to_string()]);
        assert_eq!(trigrams_of("بسمالله").len(), 5);
        // Multibyte chars count once.
        assert_eq!(trigrams_of("بسم").len(), 1);
    }

    #[test]
    fn skel_addr_orders_for_dedup() {
        let mut set = BTreeSet::new();
        set.insert(SkelAddr { surah: 1, ayah_start: 2, ayah_end: 2 });
        set.insert(SkelAddr { surah: 1, ayah_start: 2, ayah_end: 2 });
        assert_eq!(set.len(), 1);
    }
}
