//! Hybrid FTS + vector retrieval with RRF (k=60).

use std::collections::BTreeMap;

/// Reciprocal rank fusion merge of two ranked id lists.
/// Inputs are `(id, score)` pairs; only rank position contributes to RRF.
/// Returns `(id, rrf_score)` sorted by score desc, tie-break by id asc.
pub fn rrf_merge(fts: &[(String, f32)], vec: &[(String, f32)], k: f32) -> Vec<(String, f32)> {
    let mut scores: BTreeMap<String, f32> = BTreeMap::new();
    for (i, (id, _)) in fts.iter().enumerate() {
        *scores.entry(id.clone()).or_insert(0.0) += 1.0 / (k + (i + 1) as f32);
    }
    for (i, (id, _)) in vec.iter().enumerate() {
        *scores.entry(id.clone()).or_insert(0.0) += 1.0 / (k + (i + 1) as f32);
    }
    let mut out: Vec<(String, f32)> = scores.into_iter().collect();
    out.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    out
}

/// Stable sort by score desc then id asc.
pub fn stable_sort(hits: &[(String, f32)]) -> Vec<(String, f32)> {
    let mut out: Vec<(String, f32)> = hits.to_vec();
    out.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    out
}

/// Apply metadata filters (exact match on key/value) to hits carrying metadata.
#[derive(Debug, Clone)]
pub struct MetaHit {
    pub id: String,
    pub score: f32,
    pub metadata: BTreeMap<String, String>,
}

pub fn apply_filters(hits: &[MetaHit], filters: &BTreeMap<String, String>) -> Vec<MetaHit> {
    hits.iter()
        .filter(|h| filters.iter().all(|(k, v)| h.metadata.get(k) == Some(v)))
        .cloned()
        .collect()
}

/// Convenience: filter `(id, score)` pairs using an external metadata map.
pub fn apply_filters_with_metadata(
    hits: &[(String, f32)],
    filters: &BTreeMap<String, String>,
    metadata: &BTreeMap<String, BTreeMap<String, String>>,
) -> Vec<(String, f32)> {
    hits.iter()
        .filter(|(id, _)| {
            filters
                .iter()
                .all(|(k, v)| metadata.get(id).and_then(|m| m.get(k)) == Some(v))
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rrf_merges() {
        let fts = vec![("a".to_string(), 0.0f32), ("b".to_string(), 0.0)];
        let vec = vec![("b".to_string(), 0.0), ("c".to_string(), 0.0)];
        let out = rrf_merge(&fts, &vec, 60.0);
        assert!(out.iter().any(|(id, _)| id == "b"));
        // b should rank first (appears in both)
        assert_eq!(out[0].0, "b");
    }

    #[test]
    fn rrf_determinism() {
        let fts = vec![("a".to_string(), 1.0), ("b".to_string(), 0.5)];
        let vec = vec![("b".to_string(), 0.9), ("c".to_string(), 0.8)];
        let a = rrf_merge(&fts, &vec, 60.0);
        let b = rrf_merge(&fts, &vec, 60.0);
        assert_eq!(a, b);
    }

    #[test]
    fn rrf_tie_break() {
        let fts = vec![("z".to_string(), 0.0)];
        let vec = vec![("a".to_string(), 0.0)];
        let out = rrf_merge(&fts, &vec, 60.0);
        // equal scores -> id asc
        let mut scores: BTreeMap<String, f32> = BTreeMap::new();
        // just check sorted
        for i in 1..out.len() {
            if (out[i - 1].1 - out[i].1).abs() < 1e-9 {
                assert!(out[i - 1].0 < out[i].0);
            }
        }
    }

    #[test]
    fn rrf_empty() {
        let out = rrf_merge(&[], &[], 60.0);
        assert!(out.is_empty());
    }

    #[test]
    fn stable_sort_works() {
        let hits = vec![("b".to_string(), 0.5), ("a".to_string(), 0.5), ("c".to_string(), 0.9)];
        let out = stable_sort(&hits);
        assert_eq!(out[0].0, "c");
        assert_eq!(out[1].0, "a");
        assert_eq!(out[2].0, "b");
    }
}