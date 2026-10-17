//! RAG search service: hybrid query returning hits + RouteDecision.

use std::collections::BTreeMap;
use std::sync::Arc;

use embeddings::EmbeddingProvider;
use rag::{RagHit, RagProject, RouteDecision};
use reranking::{Reranker, ScoreReranker};
use storage_sqlite::SqliteDatabase;

use crate::rag_reader::{RagReader, RagReaderError, SqliteRagReader};

#[derive(Debug, Clone, thiserror::Error)]
pub enum RagSearchError {
    #[error("reader: {0}")]
    Reader(#[from] RagReaderError),
}

impl storage::error::Diagnostic for RagSearchError {
    fn code(&self) -> storage::error::DiagnosticCode {
        storage::error::DiagnosticCode::new("QAI-RAG", 10)
    }
    fn summary(&self) -> String {
        self.to_string()
    }
    fn remedy(&self) -> Option<String> {
        Some("Check the database and retry.".to_string())
    }
    fn next_command(&self) -> Option<String> {
        Some("qai rag --help".to_string())
    }
}

/// Trait to accept flexible filter inputs.
pub trait IntoFilterMap {
    fn into_filter_map(self) -> BTreeMap<String, String>;
}
impl IntoFilterMap for BTreeMap<String, String> {
    fn into_filter_map(self) -> BTreeMap<String, String> {
        self
    }
}
impl IntoFilterMap for &BTreeMap<String, String> {
    fn into_filter_map(self) -> BTreeMap<String, String> {
        self.clone()
    }
}
impl IntoFilterMap for Option<BTreeMap<String, String>> {
    fn into_filter_map(self) -> BTreeMap<String, String> {
        self.unwrap_or_default()
    }
}
impl IntoFilterMap for Option<&BTreeMap<String, String>> {
    fn into_filter_map(self) -> BTreeMap<String, String> {
        self.cloned().unwrap_or_default()
    }
}

/// Trait to accept flexible limit inputs.
pub trait IntoLimit {
    fn into_limit(self) -> usize;
}
impl IntoLimit for usize {
    fn into_limit(self) -> usize {
        if self == 0 {
            usize::MAX
        } else {
            self
        }
    }
}
impl IntoLimit for u32 {
    fn into_limit(self) -> usize {
        if self == 0 {
            usize::MAX
        } else {
            self as usize
        }
    }
}
impl IntoLimit for Option<usize> {
    fn into_limit(self) -> usize {
        self.unwrap_or(usize::MAX)
    }
}
impl IntoLimit for Option<u32> {
    fn into_limit(self) -> usize {
        self.map(|x| if x == 0 { usize::MAX } else { x as usize })
            .unwrap_or(usize::MAX)
    }
}

pub struct RagSearchService {
    reader: Arc<SqliteRagReader>,
    embedder: Arc<dyn EmbeddingProvider>,
    reranker: Arc<dyn Reranker>,
}

impl RagSearchService {
    pub fn new(
        db: Arc<SqliteDatabase>,
        embedder: Arc<dyn EmbeddingProvider>,
        reranker: Arc<dyn Reranker>,
    ) -> Self {
        let reader = Arc::new(SqliteRagReader::new(db));
        Self {
            reader,
            embedder,
            reranker,
        }
    }

    /// Convenience constructor (reader + embedder) with default reranker.
    pub fn from_reader(
        reader: Arc<SqliteRagReader>,
        embedder: Arc<dyn EmbeddingProvider>,
    ) -> Self {
        Self {
            reader,
            embedder,
            reranker: Arc::new(ScoreReranker),
        }
    }

    /// Backward-compatible constructor (reader + embedder).
    pub fn new_reader(
        reader: Arc<SqliteRagReader>,
        embedder: Arc<dyn EmbeddingProvider>,
    ) -> Self {
        Self::from_reader(reader, embedder)
    }

    pub fn reader(&self) -> &SqliteRagReader {
        &self.reader
    }

    /// Convenience constructor using default local embedder + score reranker.
    pub fn with_defaults(db: std::sync::Arc<SqliteDatabase>) -> Self {
        let embedder: std::sync::Arc<dyn EmbeddingProvider> =
            std::sync::Arc::new(embeddings::LocalHashEmbedding::default());
        let reranker: std::sync::Arc<dyn Reranker> =
            std::sync::Arc::new(reranking::ScoreReranker);
        Self::new(db, embedder, reranker)
    }

    pub async fn search(
        &self,
        project: &str,
        query: &str,
    ) -> Result<(Vec<RagHit>, RouteDecision), RagSearchError> {
        self.query(project, query, BTreeMap::new(), usize::MAX).await
    }

    pub async fn query<F, L>(
        &self,
        project: &str,
        query_text: &str,
        filters: F,
        limit: L,
    ) -> Result<(Vec<RagHit>, RouteDecision), RagSearchError>
    where
        F: IntoFilterMap,
        L: IntoLimit,
    {
        let filters = filters.into_filter_map();
        let limit = limit.into_limit();

        let proj: RagProject = self.reader.get_project(project).await?;
        let docs = self.reader.list_documents(project).await?;

        // active source ids
        let mut source_set: BTreeMap<String, ()> = BTreeMap::new();
        for d in &docs {
            source_set.insert(d.source_id.clone(), ());
        }
        let active_sources: Vec<String> = source_set.keys().cloned().collect();

        // doc metadata map
        let mut metadata_map: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
        for d in &docs {
            let mut m = BTreeMap::new();
            if let serde_json::Value::Object(obj) = &d.metadata {
                for (k, v) in obj {
                    if let Some(s) = v.as_str() {
                        m.insert(k.clone(), s.to_string());
                    } else {
                        m.insert(k.clone(), v.to_string());
                    }
                }
            }
            metadata_map.insert(d.id.clone(), m);
        }

        // FTS
        let fts_ids = self.reader.search_fts(project, query_text).await.unwrap_or_default();
        let fts_ranked: Vec<(String, f32)> = fts_ids.into_iter().map(|id| (id, 0.0f32)).collect();

        // Vector
        let qemb = self.embedder.embed(query_text);
        let mut vec_ranked: Vec<(String, f32)> = Vec::new();
        for doc in &docs {
            if let Ok((emb, dim)) = self.reader.get_embedding(&doc.id).await {
                if dim != 0 && emb.len() != dim {
                    // ignore mismatch
                }
                let score = embeddings::cosine(&qemb, &emb);
                vec_ranked.push((doc.id.clone(), score));
            }
        }
        vec_ranked.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        // RRF merge
        let merged = rag::hybrid::rrf_merge(&fts_ranked, &vec_ranked, 60.0);

        // apply metadata filters
        let merged = rag::hybrid::apply_filters_with_metadata(&merged, &filters, &metadata_map);
        let merged_count = merged.len();

        // rerank
        let candidates: Vec<reranking::RankedCandidate> = merged
            .iter()
            .map(|(id, score)| {
                let source = docs
                    .iter()
                    .find(|d| d.id == *id)
                    .map(|d| d.source_id.clone())
                    .unwrap_or_default();
                reranking::RankedCandidate {
                    id: id.clone(),
                    score: *score,
                    source,
                }
            })
            .collect();
        let reranked = self.reranker.rerank(query_text, &candidates);

        // truncate
        let reranked_count_before_truncate = reranked.len();
        let reranked = if limit == usize::MAX {
            reranked
        } else {
            reranked.into_iter().take(limit).collect()
        };

        // build hits
        let mut hits = Vec::new();
        for (rank, c) in reranked.iter().enumerate() {
            hits.push(RagHit {
                doc_id: c.id.clone(),
                score: c.score,
                source: c.source.clone(),
                rank: rank + 1,
            });
        }

        let excluded_count = self
            .reader
            .list_sources(project)
            .await
            .map(|srcs| srcs.iter().filter(|s| s.status != rag::SourceStatus::Active).count())
            .unwrap_or(0);
        let ids: Vec<String> = hits.iter().map(|h| h.doc_id.clone()).collect();
        let rd = RouteDecision {
            router: proj.config.router.name.clone(),
            tools: vec!["rag.query".to_string()],
            sources: active_sources,
            scores: hits.iter().map(|h| (h.doc_id.clone(), h.score)).collect(),
            final_ranking: ids.clone(),
            checksum: RouteDecision::compute_checksum(&proj.config.router.name, query_text, &ids),
            candidates: docs.len(),
            merged: merged_count,
            reranked: reranked_count_before_truncate,
            excluded: excluded_count,
        };

        Ok((hits, rd))
    }
}
