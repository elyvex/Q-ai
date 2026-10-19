//! Reranking providers for the multi-RAG platform.

/// A ranked candidate flowing through merge/rerank.
#[derive(Debug, Clone)]
pub struct RankedCandidate {
    pub id: String,
    pub score: f32,
    pub source: String,
}

/// Reranker trait.
pub trait Reranker: Send + Sync {
    fn name(&self) -> &'static str;
    fn rerank(&self, query: &str, hits: &[RankedCandidate]) -> Vec<RankedCandidate>;
}

/// Score-based reranker: sorts by score desc, stable tie-break by id.
pub struct ScoreReranker;

impl Reranker for ScoreReranker {
    fn name(&self) -> &'static str {
        "score"
    }
    fn rerank(&self, _query: &str, hits: &[RankedCandidate]) -> Vec<RankedCandidate> {
        let mut out: Vec<RankedCandidate> = hits.to_vec();
        out.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_desc() {
        let r = ScoreReranker;
        let hits = vec![
            RankedCandidate { id: "a".into(), score: 0.1, source: "s".into() },
            RankedCandidate { id: "b".into(), score: 0.9, source: "s".into() },
        ];
        let out = r.rerank("q", &hits);
        assert_eq!(out[0].id, "b");
    }
}
