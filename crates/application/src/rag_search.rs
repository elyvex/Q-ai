//! RAG search service: hybrid query returning hits + RouteDecision.

use std::collections::BTreeMap;
use std::sync::Arc;

use embeddings::EmbeddingProvider;
use rag::{RagHit, RagProject, RouteDecision};

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

pub struct RagSearchService {
    reader: Arc<SqliteRagReader>,
    embedder: Arc<dyn EmbeddingProvider>,
}

impl RagSearchService {
    pub fn new(reader: Arc<SqliteRagReader>, embedder: Arc<dyn EmbeddingProvider>) -> Self {
        Self { reader, embedder }
    }
    pub async fn search(&self, project: &str, query: &str) -> Result<(Vec<RagHit>, RouteDecision), RagSearchError> {
        let proj: RagProject = self.reader.get_project(project).await?;
        let docs = self.reader.list_documents(project).await?;
        let fts_ids = self.reader.search_fts(project, query).await.unwrap_or_default();
        let qemb = self.embedder.embed(query);
        let mut vec_ranked: Vec<(String, f32)> = Vec::new();
        for doc in &docs {
            if let Ok((emb, _)) = self.reader.get_embedding(&doc.id).await {
                vec_ranked.push((doc.id.clone(), embeddings::cosine(&qemb, &emb)));
            }
        }
        vec_ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then_with(|| a.0.cmp(&b.0)));
        let vec_ids: Vec<String> = vec_ranked.iter().map(|x| x.0.clone()).collect();
        let merged = rag::hybrid::rrf_merge(&fts_ids, &vec_ids, 60);
        let score_map: BTreeMap<String, f32> = vec_ranked.into_iter().collect();
        let mut hits = Vec::new();
        for (rank, id) in merged.iter().enumerate() {
            let source = docs.iter().find(|d| d.id == *id).map(|d| d.source_id.clone()).unwrap_or_default();
            hits.push(RagHit {
                doc_id: id.clone(),
                score: score_map.get(id).copied().unwrap_or(0.0),
                source,
                rank: rank + 1,
            });
        }
        let ids: Vec<String> = hits.iter().map(|h| h.doc_id.clone()).collect();
        let rd = RouteDecision {
            router: proj.config.router.name.clone(),
            tools: vec!["rag.query".to_string()],
            sources: docs.iter().map(|d| d.source_id.clone()).collect(),
            scores: hits.iter().map(|h| (h.doc_id.clone(), h.score)).collect(),
            final_ranking: ids.clone(),
            checksum: RouteDecision::compute_checksum(&proj.config.router.name, query, &ids),
        };
        Ok((hits, rd))
    }
}
