//! RAG typed tool backend.

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use domain::SemVer;
use embeddings::EmbeddingProvider;
use rag::{RagDocument, RagSource, SourceStatus};
use serde_json::Value;
use tool_registry::RagBackend;
use tools::{AnalysisSource, ToolError, ToolResult, reproducibility, research_checksum};

use crate::rag_reader::{RagReader, SqliteRagReader};
use storage_sqlite::SqliteDatabase;
use crate::rag_search::RagSearchService;

pub struct RagToolBackend {
    reader: Arc<SqliteRagReader>,
    search: Arc<RagSearchService>,
}

impl RagToolBackend {
    pub fn new(reader: Arc<SqliteRagReader>, search: Arc<RagSearchService>) -> Self {
        Self { reader, search }
    }
    pub fn reader(&self) -> &SqliteRagReader {
        &self.reader
    }

    pub fn from_db(db: std::sync::Arc<SqliteDatabase>) -> Self {
        let reader = std::sync::Arc::new(crate::rag_reader::SqliteRagReader::new(db.clone()));
        let search = std::sync::Arc::new(crate::rag_search::RagSearchService::with_defaults(db));
        Self::new(reader, search)
    }
}

fn reader_err(e: crate::rag_reader::RagReaderError) -> ToolError {
    use storage::error::Diagnostic;
    ToolError::Backend { code: e.code().to_string(), detail: e.to_string() }
}
fn search_err(e: crate::rag_search::RagSearchError) -> ToolError {
    use storage::error::Diagnostic;
    ToolError::Backend { code: e.code().to_string(), detail: e.to_string() }
}

#[async_trait]
impl RagBackend for RagToolBackend {
    async fn backend_create_project(&self, name: &str, config_toml: &str) -> Result<ToolResult<Value>, ToolError> {
        let started = Instant::now();
        let proj = self.reader.create_project(name, config_toml).await.map_err(reader_err)?;
        let query = serde_json::json!({ "name": name });
        let results = serde_json::json!({ "project_id": proj.id, "name": proj.name });
        let payload = results.clone();
        let result = ToolResult {
            tool_name: "rag.create_project".to_string(),
            tool_version: SemVer::new(1, 0, 0),
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: None,
            edition_version: None,
            results,
            canonical_references: Vec::new(),
            analysis_sources: vec![AnalysisSource { kind: "rag".to_string(), reference: proj.id.clone() }],
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility("rag.create_project", SemVer::new(1,0,0), &query, None, None, std::collections::BTreeMap::new(), 0),
            research_checksum: research_checksum("rag.create_project", SemVer::new(1,0,0), &query, &serde_json::json!({}), &payload).expect("checksum"),
        };
        Ok(result)
    }

    async fn backend_import(&self, project: &str, docs: &[Value]) -> Result<ToolResult<Value>, ToolError> {
        let started = Instant::now();
        let proj = self.reader.get_project(project).await.map_err(reader_err)?;
        let embedder = embeddings::LocalHashEmbedding::default();
        let mut count = 0usize;
        for d in docs {
            let source_id = d.get("source_id").and_then(|v| v.as_str()).unwrap_or("src-1").to_string();
            let path = d.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let title = d.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());
            let text = d.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let metadata = d.get("metadata").cloned().unwrap_or(serde_json::json!({}));
            let src = RagSource {
                id: source_id.clone(),
                project_id: proj.id.clone(),
                path: path.clone(),
                glob: "**/*".to_string(),
                status: SourceStatus::Active,
            };
            let _ = self.reader.insert_source(&src).await;
            let doc = RagDocument {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: proj.id.clone(),
                source_id: source_id.clone(),
                path,
                title,
                text: text.clone(),
                metadata,
            };
            let emb = embedder.embed(&text);
            self.reader.insert_document(&doc, &emb).await.map_err(reader_err)?;
            count += 1;
        }
        let query = serde_json::json!({ "project": project, "count": docs.len() });
        let results = serde_json::json!({ "imported": count });
        let payload = results.clone();
        let result = ToolResult {
            tool_name: "rag.import".to_string(),
            tool_version: SemVer::new(1, 0, 0),
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: None,
            edition_version: None,
            results,
            canonical_references: Vec::new(),
            analysis_sources: Vec::new(),
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility("rag.import", SemVer::new(1,0,0), &query, None, None, std::collections::BTreeMap::new(), 0),
            research_checksum: research_checksum("rag.import", SemVer::new(1,0,0), &query, &serde_json::json!({}), &payload).expect("checksum"),
        };
        Ok(result)
    }

    async fn backend_query(&self, project: &str, query: &str) -> Result<ToolResult<Value>, ToolError> {
        let started = Instant::now();
        let (hits, rd) = self.search.search(project, query).await.map_err(search_err)?;
        let q = serde_json::json!({ "project": project, "query": query });
        // RagHit not serde? It derives? rag::RagHit not serde derive. Let's manually build.
        let hits_arr: Vec<Value> = hits.iter().map(|h| serde_json::json!({
            "doc_id": h.doc_id,
            "score": h.score,
            "source": h.source,
            "rank": h.rank,
        })).collect();
        let rd_json = serde_json::json!({
            "router": rd.router,
            "tools": rd.tools,
            "sources": rd.sources,
            "scores": rd.scores,
            "final_ranking": rd.final_ranking,
            "checksum": rd.checksum,
            "candidates": rd.candidates,
            "merged": rd.merged,
            "reranked": rd.reranked,
            "excluded": rd.excluded,
        });
        let results = serde_json::json!({ "hits": hits_arr, "route_decision": rd_json });
        let payload = results.clone();
        let result = ToolResult {
            tool_name: "rag.query".to_string(),
            tool_version: SemVer::new(1, 0, 0),
            query: q.clone(),
            normalization_rules: Vec::new(),
            edition_id: None,
            edition_version: None,
            results,
            canonical_references: rd.final_ranking.clone(),
            analysis_sources: Vec::new(),
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility("rag.query", SemVer::new(1,0,0), &q, None, None, std::collections::BTreeMap::new(), 0),
            research_checksum: research_checksum("rag.query", SemVer::new(1,0,0), &q, &serde_json::json!({}), &payload).expect("checksum"),
        };
        Ok(result)
    }

    async fn backend_debug(&self, project: &str, query: &str) -> Result<ToolResult<Value>, ToolError> {
        self.backend_query(project, query).await
    }

    async fn backend_source_status(&self, project: &str, source: &str, status: &str) -> Result<ToolResult<Value>, ToolError> {
        let started = Instant::now();
        self.reader.set_source_status(project, source, status).await.map_err(reader_err)?;
        let audit_seq = self.reader.source_audit_seq(project, source).await.map_err(reader_err)?;
        let q = serde_json::json!({ "project": project, "source": source, "status": status });
        let results = serde_json::json!({ "ok": true, "audit_seq": audit_seq });
        let payload = results.clone();
        let result = ToolResult {
            tool_name: "rag.source_status".to_string(),
            tool_version: SemVer::new(1, 0, 0),
            query: q.clone(),
            normalization_rules: Vec::new(),
            edition_id: None,
            edition_version: None,
            results,
            canonical_references: Vec::new(),
            analysis_sources: Vec::new(),
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility("rag.source_status", SemVer::new(1,0,0), &q, None, None, std::collections::BTreeMap::new(), 0),
            research_checksum: research_checksum("rag.source_status", SemVer::new(1,0,0), &q, &serde_json::json!({}), &payload).expect("checksum"),
        };
        Ok(result)
    }
}
