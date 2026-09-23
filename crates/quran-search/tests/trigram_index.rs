//! Phase 2 — trigram posting index acceptance (P2-T36, real SQLite, tempdirs).
//!
//! Build → recall → verify over synthetic skeletons: every skeleton's own
//! trigram set resolves back to itself (no posting-level false negatives),
//! absent trigrams recall nothing, short queries bypass postings, and a
//! missing file reports graceful fallback (`None`, never an error).

use std::collections::BTreeSet;

use quran_search::trigram;
use quran_search::trigram::{SkelAddr, build_postings, recall, trigram_path, trigrams_of};

fn addr(surah: i64, start: i64, end: i64) -> SkelAddr {
    SkelAddr { surah, ayah_start: start, ayah_end: end }
}

fn rows() -> Vec<(SkelAddr, String)> {
    vec![
        (addr(1, 1, 1), "بسماللهالرحمنالرحيم".to_string()),
        (addr(1, 2, 2), "الحمدلللهربالعلمين".to_string()),
        (addr(1, 1, 3), "بسماللهالرحمنالرحيمالحمدللهربالعلمين".to_string()),
        (addr(2, 1, 1), "المذلكالكتب".to_string()),
    ]
}

#[tokio::test]
async fn build_recall_verify_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = trigram_path(dir.path(), 3);
    let input = rows();
    let (skels, postings) = build_postings(&path, &input).await.unwrap();
    assert_eq!(skels, 4);
    assert!(postings > 0);
    assert!(path.exists());

    // Every skeleton recalls itself through its own trigrams.
    let checked = trigram::verify_postings(&path, &input).await.unwrap();
    assert_eq!(checked, 4);

    // A shared trigram recalls exactly its owners.
    let shared = trigrams_of("بسمالله");
    let hit = recall(&path, &shared).await.unwrap().unwrap();
    assert!(hit.contains(&addr(1, 1, 1)));
    assert!(hit.contains(&addr(1, 1, 3)));
    assert!(!hit.contains(&addr(2, 1, 1)));
}

#[tokio::test]
async fn absent_trigram_recalls_empty_set() {
    let dir = tempfile::tempdir().unwrap();
    let path = trigram_path(dir.path(), 1);
    build_postings(&path, &rows()).await.unwrap();
    let hit = recall(&path, &["zzz".to_string()]).await.unwrap().unwrap();
    assert!(hit.is_empty());
}

#[tokio::test]
async fn short_queries_and_missing_files_fall_back() {
    let dir = tempfile::tempdir().unwrap();
    // Empty trigram list (sub-3-char queries): graceful bypass.
    let path = trigram_path(dir.path(), 1);
    build_postings(&path, &rows()).await.unwrap();
    assert!(recall(&path, &[]).await.unwrap().is_none());
    // Pre-T36 generation without the file: graceful bypass, not an error.
    let missing = trigram_path(dir.path(), 99);
    assert!(!missing.exists());
    let hit = recall(&missing, &["بسم".to_string()]).await.unwrap();
    assert!(hit.is_none());
}

#[tokio::test]
async fn rebuild_overwrites_and_grouping_splits_surahs() {
    let dir = tempfile::tempdir().unwrap();
    let path = trigram_path(dir.path(), 1);
    build_postings(&path, &rows()).await.unwrap();
    let fewer = vec![(addr(9, 9, 9), "ن والقلم".to_string())];
    let (skels, _) = build_postings(&path, &fewer).await.unwrap();
    assert_eq!(skels, 1);
    // Old owners no longer resolve.
    let hit = recall(&path, &trigrams_of("بسمالله")).await.unwrap().unwrap();
    assert!(hit.is_empty());

    let grouped = trigram::group_by_surah(&rows());
    assert_eq!(grouped.len(), 2);
    assert_eq!(grouped[&1].len(), 3);
    assert_eq!(grouped[&2].len(), 1);
    let _ = BTreeSet::<SkelAddr>::new();
}
