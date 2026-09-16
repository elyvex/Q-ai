//! Phase 2 — generation-keyed search result cache (M3, P2-T50).
//!
//! Correctness contract (I14 / cache safety):
//!
//! - the cache key binds the corpus generation, so a generation bump can
//!   never address stale entries (namespace isolation, not best effort);
//! - reads additionally verify the stored generation and drop mismatches
//!   (defense in depth against out-of-band writes);
//! - unparseable payloads are misses, never errors (derived data; the index
//!   is always available to recompute);
//! - eviction is LRU over `last_hit_at`, capped at
//!   [`SEARCH_CACHE_CAP_BYTES`] (128 MiB) by default;
//! - wholesale invalidation on a generation bump deletes every row from
//!   other generations ([`cache_invalidate`]).
//!
//! The cached value is a serialized [`SearchOutput`](super::quran_search::SearchOutput).
//! Cache rows never touch canonical tables.

use storage::Database as _;
use storage_sqlite::SqliteDatabase;

use super::quran_search::{SearchError, SearchOutput, SearchParams};

/// Default cache cap: 128 MiB (plan §13).
pub const SEARCH_CACHE_CAP_BYTES: i64 = 128 * 1024 * 1024;

/// Deterministic cache key binding tool, params, profile, extras, and
/// generation: `v1:{generation}:{sha256(canonical_json)}`.
///
/// `serde_json::Map` is `BTreeMap`-backed, so the canonical JSON (and the
/// key) is stable for equal inputs. Equal queries always share a key;
/// different generations never do.
#[must_use]
pub fn cache_key(
    tool: &str,
    params: &SearchParams,
    profile: &str,
    extra: &str,
    generation: i64,
) -> String {
    let canonical = serde_json::json!({
        "tool": tool,
        "params": params,
        "profile": profile,
        "extra": extra,
    });
    let bytes = serde_json::to_vec(&canonical).expect("SearchParams serializes by construction");
    format!("v1:{generation}:{}", quran_corpus::sha256_hex(&bytes))
}

/// Look up a cached response.
///
/// Returns `None` on misses, generation mismatches (row deleted), and
/// unparseable payloads (row deleted). Hits refresh `last_hit_at`.
pub async fn cache_lookup(
    db: &SqliteDatabase,
    key: &str,
    generation: i64,
) -> Result<Option<SearchOutput>, SearchError> {
    fn storage(error: storage::error::StorageError) -> SearchError {
        SearchError::Storage(error.to_string())
    }
    let mut uow = db.write().await.map_err(storage)?;
    let row = uow.quran().cache_get(key).await.map_err(storage)?;
    let Some(row) = row else {
        uow.rollback().await.map_err(storage)?;
        return Ok(None);
    };
    if row.generation != generation {
        uow.quran().cache_delete(key).await.map_err(storage)?;
        uow.commit().await.map_err(storage)?;
        return Ok(None);
    }
    match serde_json::from_str::<SearchOutput>(&row.payload_json) {
        Ok(output) => {
            let at = domain::Timestamp::now().to_string();
            uow.quran().cache_touch(key, &at).await.map_err(storage)?;
            uow.commit().await.map_err(storage)?;
            Ok(Some(output))
        }
        Err(_) => {
            uow.quran().cache_delete(key).await.map_err(storage)?;
            uow.commit().await.map_err(storage)?;
            Ok(None)
        }
    }
}

/// Store a response and enforce the byte cap.
///
/// # Errors
///
/// Returns [`SearchError::Storage`] on database failures. Serialization
/// cannot fail (`SearchOutput` serializes by construction).
pub async fn cache_store(
    db: &SqliteDatabase,
    key: &str,
    generation: i64,
    output: &SearchOutput,
    cap_bytes: i64,
) -> Result<(), SearchError> {
    fn storage(error: storage::error::StorageError) -> SearchError {
        SearchError::Storage(error.to_string())
    }
    let payload_json =
        serde_json::to_string(output).expect("SearchOutput serializes by construction");
    let at = domain::Timestamp::now().to_string();
    let mut uow = db.write().await.map_err(storage)?;
    uow.quran()
        .cache_put(storage::quran::SearchCacheRow {
            key: key.to_string(),
            generation,
            payload_json: payload_json.clone(),
            bytes: payload_json.len() as i64,
            created_at: at.clone(),
            last_hit_at: at,
        })
        .await
        .map_err(storage)?;
    uow.quran().cache_enforce_cap(cap_bytes).await.map_err(storage)?;
    uow.commit().await.map_err(storage)?;
    Ok(())
}

/// Wholesale invalidation: delete every entry from other generations.
/// Returns the deleted count.
pub async fn cache_invalidate(
    db: &SqliteDatabase,
    current_generation: i64,
) -> Result<u64, SearchError> {
    fn storage(error: storage::error::StorageError) -> SearchError {
        SearchError::Storage(error.to_string())
    }
    let mut uow = db.write().await.map_err(storage)?;
    let deleted = uow.quran().cache_delete_stale(current_generation).await.map_err(storage)?;
    uow.commit().await.map_err(storage)?;
    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quran_search::SearchParams;

    fn test_params(text: &str) -> SearchParams {
        SearchParams {
            text: text.to_string(),
            edition: None,
            mode: crate::quran_search::MatchMode::WholeToken,
            filters: vec![],
            limit: 100,
            offset: 0,
            explain: false,
            highlight: false,
        }
    }

    #[test]
    fn keys_bind_generation_and_vary_with_inputs() {
        let params = test_params("الرحمن");
        let a = cache_key("quran.search_normalized", &params, "L3.diacritics@1.0.0", "", 1);
        let b = cache_key("quran.search_normalized", &params, "L3.diacritics@1.0.0", "", 1);
        assert_eq!(a, b, "equal inputs share a key");
        assert!(a.starts_with("v1:1:"), "{a}");
        let bumped = cache_key("quran.search_normalized", &params, "L3.diacritics@1.0.0", "", 2);
        assert_ne!(a, bumped, "generation bump changes the namespace");
        let other_tool = cache_key("quran.search_exact", &params, "L0.exact@1.0.0", "", 1);
        assert_ne!(a, other_tool, "tool binds into the key");
        let mut reordered = test_params("الرحمن");
        reordered.limit = 50;
        assert_ne!(
            a,
            cache_key("quran.search_normalized", &reordered, "L3.diacritics@1.0.0", "", 1),
            "paging binds into the key"
        );
    }
}
