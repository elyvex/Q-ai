//! Hybrid FTS + vector retrieval with RRF (k=60).

/// Reciprocal rank fusion merge of two ranked id lists.
pub fn rrf_merge(fts: &[String], vec: &[String], k: usize) -> Vec<String> {
    use std::collections::BTreeMap;
    let mut scores: BTreeMap<String, f32> = BTreeMap::new();
    for (i, id) in fts.iter().enumerate() {
        *scores.entry(id.clone()).or_insert(0.0) += 1.0 / (k as f32 + (i + 1) as f32);
    }
    for (i, id) in vec.iter().enumerate() {
        *scores.entry(id.clone()).or_insert(0.0) += 1.0 / (k as f32 + (i + 1) as f32);
    }
    let mut out: Vec<(String, f32)> = scores.into_iter().collect();
    out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then_with(|| a.0.cmp(&b.0)));
    out.into_iter().map(|(id, _)| id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rrf_merges() {
        let fts = vec!["a".to_string(), "b".to_string()];
        let vec = vec!["b".to_string(), "c".to_string()];
        let out = rrf_merge(&fts, &vec, 60);
        assert!(out.contains(&"b".to_string()));
    }
}
