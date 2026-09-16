//! Phase 2 — search result cache acceptance (M3, P2-T50).
//!
//! Against real SQLite in a tempdir: put/get round-trips preserve the full
//! output shape, generation mismatches and garbage payloads are misses (and
//! are deleted), LRU eviction honors recency under a byte cap, and wholesale
//! invalidation drops every stale generation.

use application::quran_search::{MatchMode, SearchParams};
use application::quran_search_cache::{
    SEARCH_CACHE_CAP_BYTES, cache_invalidate, cache_key, cache_lookup, cache_store,
};
use storage::Database as _;
use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

// Reuse the searchable fixture via the tools suite would duplicate ~60 lines;
// the cache contract needs only a migrated database plus one real output.
async fn migrated_db() -> (tempfile::TempDir, SqliteDatabase) {
    let dir = tempdir().unwrap();
    let path = dir.path().join("qai.db");
    let path_str = path.to_str().unwrap().to_string();
    let repo_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    storage_sqlite::migrate::apply_migrations(&path_str, &repo_root).await.unwrap();
    let db = SqliteDatabase::new(&path_str, 4, true).await.unwrap();
    (dir, db)
}

fn params(text: &str) -> SearchParams {
    SearchParams {
        text: text.to_string(),
        edition: None,
        mode: MatchMode::WholeToken,
        filters: vec![],
        limit: 100,
        offset: 0,
        explain: false,
        highlight: false,
    }
}

fn l3_params() -> (SearchParams, String) {
    (params("الرحمن"), "L3.diacritics@1.0.0".to_string())
}

/// Put/get round-trips preserve the output byte-identically (compared
/// through `serde_json::Value`, so field order cannot hide drift).
#[tokio::test]
async fn roundtrip_preserves_output() {
    let (_dir, db) = migrated_db().await;
    let output =
        serde_json::from_value::<application::quran_search::SearchOutput>(serde_json::json!({
            "hits": [],
            "total_matches": 0,
            "truncated": false,
            "rule_set": "L3.diacritics@1.0.0",
            "generation": 1,
            "warnings": [],
            "regex_report": null,
        }))
        .unwrap();
    let (search_params, profile) = l3_params();
    let key = cache_key("quran.search_normalized", &search_params, &profile, "", 1);
    cache_store(&db, &key, 1, &output, SEARCH_CACHE_CAP_BYTES).await.unwrap();
    let back = cache_lookup(&db, &key, 1).await.unwrap().expect("cache hit");
    assert_eq!(
        serde_json::to_value(&back).unwrap(),
        serde_json::to_value(&output).unwrap(),
        "cached output is byte-identical through serde"
    );
}

/// Generation mismatches are misses, and the stale row is deleted.
#[tokio::test]
async fn generation_mismatch_is_a_miss() {
    let (_dir, db) = migrated_db().await;
    let (search_params, profile) = l3_params();
    let key = cache_key("quran.search_normalized", &search_params, &profile, "", 1);
    let output = serde_json::from_value(serde_json::json!({
        "hits": [], "total_matches": 0, "truncated": false,
        "rule_set": "L3.diacritics@1.0.0", "generation": 1,
        "warnings": [], "regex_report": null,
    }))
    .unwrap();
    cache_store(&db, &key, 1, &output, SEARCH_CACHE_CAP_BYTES).await.unwrap();

    // Same key namespace, new generation: miss.
    assert!(cache_lookup(&db, &key, 2).await.unwrap().is_none());
    // The stale row was deleted, not left behind.
    let mut uow = db.write().await.unwrap();
    assert!(uow.quran().cache_get(&key).await.unwrap().is_none());
    uow.rollback().await.unwrap();
}

/// Garbage payloads are misses, never errors — and are cleaned up.
#[tokio::test]
async fn garbage_payload_is_a_miss() {
    let (_dir, db) = migrated_db().await;
    let key = "v1:1:garbage-test".to_string();
    let mut uow = db.write().await.unwrap();
    uow.quran()
        .cache_put(storage::quran::SearchCacheRow {
            key: key.clone(),
            generation: 1,
            payload_json: "this is not json{{{".to_string(),
            bytes: 20,
            created_at: "2026-09-15T00:00:00Z".to_string(),
            last_hit_at: "2026-09-15T00:00:00Z".to_string(),
        })
        .await
        .unwrap();
    uow.commit().await.unwrap();
    assert!(cache_lookup(&db, &key, 1).await.unwrap().is_none());
    let mut uow = db.write().await.unwrap();
    assert!(uow.quran().cache_get(&key).await.unwrap().is_none(), "garbage deleted");
    uow.rollback().await.unwrap();
}

/// LRU eviction keeps the most-recently-hit entries under the byte cap.
#[tokio::test]
async fn lru_evicts_oldest_first() {
    let (_dir, db) = migrated_db().await;
    let mut uow = db.write().await.unwrap();
    for (key, at) in [
        ("k-old", "2026-09-15T00:00:00Z"),
        ("k-mid", "2026-09-15T00:00:01Z"),
        ("k-new", "2026-09-15T00:00:02Z"),
    ] {
        uow.quran()
            .cache_put(storage::quran::SearchCacheRow {
                key: key.to_string(),
                generation: 1,
                payload_json: "{}".to_string(),
                bytes: 10,
                created_at: at.to_string(),
                last_hit_at: at.to_string(),
            })
            .await
            .unwrap();
    }
    uow.commit().await.unwrap();

    // 30 bytes total; cap 25 keeps the two newest (10 + 10 + 10 running
    // newest-first: 10, 20, 30 — the oldest exceeds the cap).
    let mut uow = db.write().await.unwrap();
    let evicted = uow.quran().cache_enforce_cap(25).await.unwrap();
    assert_eq!(evicted, 1);
    assert!(uow.quran().cache_get("k-old").await.unwrap().is_none());
    assert!(uow.quran().cache_get("k-mid").await.unwrap().is_some());
    assert!(uow.quran().cache_get("k-new").await.unwrap().is_some());
    // Commit (not rollback): the eviction must persist for the next stage.
    uow.commit().await.unwrap();

    // Touching k-mid makes k-new the eviction candidate on a tighter cap.
    let mut uow = db.write().await.unwrap();
    uow.quran().cache_touch("k-mid", "2026-09-15T00:00:03Z").await.unwrap();
    uow.commit().await.unwrap();
    let mut uow = db.write().await.unwrap();
    let evicted = uow.quran().cache_enforce_cap(15).await.unwrap();
    assert_eq!(evicted, 1);
    assert!(uow.quran().cache_get("k-new").await.unwrap().is_none());
    assert!(uow.quran().cache_get("k-mid").await.unwrap().is_some());
    uow.rollback().await.unwrap();
}

/// Wholesale invalidation drops every stale generation and keeps current.
#[tokio::test]
async fn invalidate_drops_only_stale_generations() {
    let (_dir, db) = migrated_db().await;
    let mut uow = db.write().await.unwrap();
    for (key, generation) in [("k-g1", 1), ("k-g2a", 2), ("k-g2b", 2)] {
        uow.quran()
            .cache_put(storage::quran::SearchCacheRow {
                key: key.to_string(),
                generation,
                payload_json: "{}".to_string(),
                bytes: 2,
                created_at: "2026-09-15T00:00:00Z".to_string(),
                last_hit_at: "2026-09-15T00:00:00Z".to_string(),
            })
            .await
            .unwrap();
    }
    uow.commit().await.unwrap();

    let deleted = cache_invalidate(&db, 2).await.unwrap();
    assert_eq!(deleted, 1);
    let mut uow = db.write().await.unwrap();
    assert!(uow.quran().cache_get("k-g1").await.unwrap().is_none());
    assert!(uow.quran().cache_get("k-g2a").await.unwrap().is_some());
    assert!(uow.quran().cache_get("k-g2b").await.unwrap().is_some());
    uow.rollback().await.unwrap();

    // Nothing stale: zero deletions.
    assert_eq!(cache_invalidate(&db, 2).await.unwrap(), 0);
}

/// Cache stats report counts and bytes for `doctor`.
#[tokio::test]
async fn stats_report_counts_and_bytes() {
    let (_dir, db) = migrated_db().await;
    let mut uow = db.write().await.unwrap();
    let (count, bytes) = uow.quran().cache_stats().await.unwrap();
    uow.rollback().await.unwrap();
    assert_eq!((count, bytes), (0, 0));
    let output = serde_json::from_value(serde_json::json!({
        "hits": [], "total_matches": 0, "truncated": false,
        "rule_set": "L3.diacritics@1.0.0", "generation": 1,
        "warnings": [], "regex_report": null,
    }))
    .unwrap();
    let (search_params, profile) = l3_params();
    let key = cache_key("quran.search_normalized", &search_params, &profile, "", 1);
    cache_store(&db, &key, 1, &output, SEARCH_CACHE_CAP_BYTES).await.unwrap();
    let mut uow = db.write().await.unwrap();
    let (count, bytes) = uow.quran().cache_stats().await.unwrap();
    uow.rollback().await.unwrap();
    assert_eq!(count, 1);
    assert!(bytes > 0);
}
