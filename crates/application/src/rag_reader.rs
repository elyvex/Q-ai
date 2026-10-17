//! RAG reader service: project CRUD, document import, source status, tombstones.

use std::sync::Arc;

use async_trait::async_trait;
use embeddings::EmbeddingProvider;
use rag::{RagConfig, RagDocument, RagProject, RagSource};
use sqlx::Row;
use storage::error::StorageError;
use storage_sqlite::SqliteDatabase;

#[derive(Debug, Clone, thiserror::Error)]
pub enum RagReaderError {
    #[error("storage failed: {0}")]
    Storage(#[from] StorageError),
    #[error("project not found: {0}")]
    ProjectNotFound(String),
    #[error("project already exists: {0}")]
    DuplicateProject(String),
    #[error("invalid config: {0}")]
    ConfigInvalid(String),
    #[error("invalid status: {0}")]
    InvalidStatus(String),
}

impl storage::error::Diagnostic for RagReaderError {
    fn code(&self) -> storage::error::DiagnosticCode {
        match self {
            Self::Storage(_) => storage::error::DiagnosticCode::new("QAI-RAG", 1),
            Self::ProjectNotFound(_) => storage::error::DiagnosticCode::new("QAI-RAG", 2),
            Self::ConfigInvalid(_) => storage::error::DiagnosticCode::new("QAI-RAG", 3),
            Self::DuplicateProject(_) => storage::error::DiagnosticCode::new("QAI-RAG", 5),
            Self::InvalidStatus(_) => storage::error::DiagnosticCode::new("QAI-RAG", 4),
        }
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

#[async_trait]
pub trait RagReader: Send + Sync {
    async fn create_project(&self, name: &str, config_toml: &str) -> Result<RagProject, RagReaderError>;
    async fn get_project(&self, name: &str) -> Result<RagProject, RagReaderError>;
    async fn list_projects(&self) -> Result<Vec<RagProject>, RagReaderError>;
    async fn import_documents(&self, project: &str, source_path: &str, glob: &str, files: Vec<(String, String, serde_json::Value)>) -> Result<(usize, usize), RagReaderError>;
    async fn list_sources(&self, project: &str) -> Result<Vec<RagSource>, RagReaderError>;
    async fn insert_source(&self, source: &RagSource) -> Result<(), RagReaderError>;
    async fn set_source_status(&self, project: &str, source: &str, status: &str) -> Result<(), RagReaderError>;
    async fn source_audit_seq(&self, project: &str, source: &str) -> Result<Option<i64>, RagReaderError>;
    async fn insert_document(&self, doc: &RagDocument, embedding: &[f32]) -> Result<(), RagReaderError>;
    async fn list_documents(&self, project: &str) -> Result<Vec<RagDocument>, RagReaderError>;
    async fn search_fts(&self, project: &str, query: &str) -> Result<Vec<String>, RagReaderError>;
    async fn get_embedding(&self, doc_id: &str) -> Result<(Vec<f32>, usize), RagReaderError>;
    async fn delete_embeddings_for_source(&self, project: &str, source: &str) -> Result<(), RagReaderError>;
}

pub struct SqliteRagReader {
    db: Arc<SqliteDatabase>,
}

impl SqliteRagReader {
    pub fn new(db: Arc<SqliteDatabase>) -> Self {
        Self { db }
    }
    pub fn database(&self) -> &SqliteDatabase {
        &self.db
    }
    async fn project_id(&self, name: &str) -> Result<String, RagReaderError> {
        let row = sqlx::query("SELECT id FROM rag_projects WHERE name = ?")
            .bind(name)
            .fetch_optional(self.db.read_pool())
            .await
            .map_err(|e| RagReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        row.map(|r| r.get::<String, _>("id"))
            .ok_or_else(|| RagReaderError::ProjectNotFound(name.to_string()))
    }
    fn now() -> String {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string())
    }
}

#[async_trait]
impl RagReader for SqliteRagReader {
    async fn create_project(&self, name: &str, config_toml: &str) -> Result<RagProject, RagReaderError> {
        let cfg = RagConfig::from_toml_str(config_toml)
            .map_err(|e| RagReaderError::ConfigInvalid(e.to_string()))?;
        // duplicate check
        let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM rag_projects WHERE name = ?")
            .bind(name)
            .fetch_one(self.db.read_pool())
            .await
            .map_err(|e| RagReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        if exists > 0 {
            return Err(RagReaderError::DuplicateProject(name.to_string()));
        }
        let id = uuid::Uuid::new_v4().to_string();
        let now = Self::now();
        sqlx::query(
            r#"INSERT INTO rag_projects (id, name, config_toml, embedding_provider, embedding_dim, reranker, router, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(&id)
        .bind(name)
        .bind(config_toml)
        .bind(&cfg.embedding.provider)
        .bind(cfg.embedding.dimension as i64)
        .bind(&cfg.reranker.provider)
        .bind(&cfg.router.name)
        .bind(&now)
        .bind(&now)
        .execute(self.db.write_pool())
        .await
        .map_err(|e| RagReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        Ok(RagProject { id, name: name.to_string(), config: cfg, created_at: now })
    }

    async fn get_project(&self, name: &str) -> Result<RagProject, RagReaderError> {
        let row = sqlx::query("SELECT id, name, config_toml, created_at FROM rag_projects WHERE name = ?")
            .bind(name)
            .fetch_optional(self.db.read_pool())
            .await
            .map_err(|e| RagReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?
            .ok_or_else(|| RagReaderError::ProjectNotFound(name.to_string()))?;
        let id: String = row.get("id");
        let name: String = row.get("name");
        let config_toml: String = row.get("config_toml");
        let created_at: String = row.get("created_at");
        let cfg = RagConfig::from_toml_str(&config_toml)
            .map_err(|e| RagReaderError::ConfigInvalid(e.to_string()))?;
        Ok(RagProject { id, name, config: cfg, created_at })
    }

    async fn list_projects(&self) -> Result<Vec<RagProject>, RagReaderError> {
        let rows = sqlx::query("SELECT id, name, config_toml, created_at FROM rag_projects")
            .fetch_all(self.db.read_pool())
            .await
            .map_err(|e| RagReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        let mut out = Vec::new();
        for row in rows {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let config_toml: String = row.get("config_toml");
            let created_at: String = row.get("created_at");
            if let Ok(cfg) = RagConfig::from_toml_str(&config_toml) {
                out.push(RagProject { id, name, config: cfg, created_at });
            }
        }
        Ok(out)
    }

    async fn import_documents(&self, project: &str, source_path: &str, glob: &str, files: Vec<(String, String, serde_json::Value)>) -> Result<(usize, usize), RagReaderError> {
        let proj = self.get_project(project).await?;
        let source = RagSource {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: proj.id.clone(),
            path: source_path.to_string(),
            glob: glob.to_string(),
            status: rag::SourceStatus::Active,
        };
        self.insert_source(&source).await?;
        let embedder = embeddings::LocalHashEmbedding::new(proj.config.embedding.dimension);
        let mut imported = 0usize;
        let mut skipped = 0usize;
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (path, text, metadata) in files {
            if text.trim().is_empty() {
                skipped += 1;
                continue;
            }
            if !seen.insert(path.clone()) {
                skipped += 1;
                continue;
            }
            // unchanged by path: skip if already exists
            let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM rag_documents WHERE project_id = ? AND path = ?")
                .bind(&proj.id)
                .bind(&path)
                .fetch_one(self.db.read_pool())
                .await
                .map_err(|e| RagReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
            if exists > 0 {
                skipped += 1;
                continue;
            }
            let doc = RagDocument {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: proj.id.clone(),
                source_id: source.id.clone(),
                path,
                title: None,
                text: text.clone(),
                metadata,
            };
            let emb = embedder.embed(&text);
            self.insert_document(&doc, &emb).await?;
            imported += 1;
        }
        Ok((imported, skipped))
    }

    async fn list_sources(&self, project: &str) -> Result<Vec<RagSource>, RagReaderError> {
        let pid = self.project_id(project).await?;
        let rows = sqlx::query("SELECT id, project_id, path, glob, status FROM rag_sources WHERE project_id = ?")
            .bind(&pid)
            .fetch_all(self.db.read_pool())
            .await
            .map_err(|e| RagReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        let mut out = Vec::new();
        for row in rows {
            let status_str: String = row.get("status");
            let status = rag::SourceStatus::from_str(&status_str).unwrap_or(rag::SourceStatus::Active);
            out.push(RagSource {
                id: row.get("id"),
                project_id: row.get("project_id"),
                path: row.get("path"),
                glob: row.get("glob"),
                status,
            });
        }
        Ok(out)
    }

    async fn insert_source(&self, source: &RagSource) -> Result<(), RagReaderError> {
        sqlx::query(
            r#"INSERT INTO rag_sources (id, project_id, path, glob, status, tombstoned_at, audit_seq, created_at)
               VALUES (?, ?, ?, ?, ?, NULL, NULL, ?)"#,
        )
        .bind(&source.id)
        .bind(&source.project_id)
        .bind(&source.path)
        .bind(&source.glob)
        .bind(source.status.as_str())
        .bind(Self::now())
        .execute(self.db.write_pool())
        .await
        .map_err(|e| RagReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        Ok(())
    }

    async fn set_source_status(&self, project: &str, source: &str, status: &str) -> Result<(), RagReaderError> {
        if rag::SourceStatus::from_str(status).is_none() {
            return Err(RagReaderError::InvalidStatus(status.to_string()));
        }
        let pid = self.project_id(project).await?;
        let tombstoned_at = if status == "tombstoned" { Some(Self::now()) } else { None };
        let audit_seq = if status == "tombstoned" {
            let max_seq: Option<i64> = sqlx::query_scalar("SELECT MAX(audit_seq) FROM rag_sources WHERE project_id = ?")
                .bind(&pid)
                .fetch_one(self.db.read_pool())
                .await
                .map_err(|e| RagReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
            Some(max_seq.unwrap_or(0) + 1)
        } else {
            None
        };
        sqlx::query("UPDATE rag_sources SET status = ?, tombstoned_at = ?, audit_seq = COALESCE(?, audit_seq) WHERE id = ? AND project_id = ?")
            .bind(status)
            .bind(tombstoned_at)
            .bind(audit_seq)
            .bind(source)
            .bind(&pid)
            .execute(self.db.write_pool())
            .await
            .map_err(|e| RagReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        if status == "deactivated" || status == "tombstoned" {
            self.delete_embeddings_for_source(project, source).await?;
            let pid = self.project_id(project).await?;
            sqlx::query("DELETE FROM rag_documents_fts WHERE doc_id IN (SELECT id FROM rag_documents WHERE project_id = ? AND source_id = ?)")
                .bind(&pid)
                .bind(source)
                .execute(self.db.write_pool())
                .await
                .map_err(|e| RagReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        }
        Ok(())
    }

    async fn source_audit_seq(&self, project: &str, source: &str) -> Result<Option<i64>, RagReaderError> {
        let pid = self.project_id(project).await?;
        let seq: Option<i64> = sqlx::query_scalar("SELECT audit_seq FROM rag_sources WHERE id = ? AND project_id = ?")
            .bind(source)
            .bind(&pid)
            .fetch_optional(self.db.read_pool())
            .await
            .map_err(|e| RagReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?
            .flatten();
        Ok(seq)
    }

    async fn insert_document(&self, doc: &RagDocument, embedding: &[f32]) -> Result<(), RagReaderError> {
        sqlx::query(
            r#"INSERT INTO rag_documents (id, project_id, source_id, path, title, text, metadata_json, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(&doc.id)
        .bind(&doc.project_id)
        .bind(&doc.source_id)
        .bind(&doc.path)
        .bind(&doc.title)
        .bind(&doc.text)
        .bind(doc.metadata.to_string())
        .bind(Self::now())
        .execute(self.db.write_pool())
        .await
        .map_err(|e| RagReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;

        sqlx::query("INSERT INTO rag_documents_fts (doc_id, text) VALUES (?, ?)")
            .bind(&doc.id)
            .bind(&doc.text)
            .execute(self.db.write_pool())
            .await
            .map_err(|e| RagReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;

        sqlx::query(
            r#"INSERT INTO rag_embeddings (doc_id, project_id, embedding, dim, created_at)
               VALUES (?, ?, ?, ?, ?)"#,
        )
        .bind(&doc.id)
        .bind(&doc.project_id)
        .bind(embeddings::to_blob(embedding))
        .bind(embedding.len() as i64)
        .bind(Self::now())
        .execute(self.db.write_pool())
        .await
        .map_err(|e| RagReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        Ok(())
    }

    async fn list_documents(&self, project: &str) -> Result<Vec<RagDocument>, RagReaderError> {
        let pid = self.project_id(project).await?;
        // filter active sources
        let rows = sqlx::query(
            r#"SELECT d.id, d.project_id, d.source_id, d.path, d.title, d.text, d.metadata_json
               FROM rag_documents d
               JOIN rag_sources s ON s.id = d.source_id AND s.project_id = d.project_id
               WHERE d.project_id = ? AND s.status = 'active'"#,
        )
        .bind(&pid)
        .fetch_all(self.db.read_pool())
        .await
        .map_err(|e| RagReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        let mut out = Vec::new();
        for row in rows {
            let metadata_str: String = row.get("metadata_json");
            let metadata = serde_json::from_str(&metadata_str).unwrap_or(serde_json::json!({}));
            out.push(RagDocument {
                id: row.get("id"),
                project_id: row.get("project_id"),
                source_id: row.get("source_id"),
                path: row.get("path"),
                title: row.get("title"),
                text: row.get("text"),
                metadata,
            });
        }
        Ok(out)
    }

    async fn search_fts(&self, project: &str, query: &str) -> Result<Vec<String>, RagReaderError> {
        let pid = self.project_id(project).await?;
        // active doc ids only
        let active_docs: Vec<String> = sqlx::query_scalar(
            r#"SELECT d.id FROM rag_documents d
               JOIN rag_sources s ON s.id = d.source_id AND s.project_id = d.project_id
               WHERE d.project_id = ? AND s.status = 'active'"#,
        )
        .bind(&pid)
        .fetch_all(self.db.read_pool())
        .await
        .map_err(|e| RagReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        let set: std::collections::HashSet<String> = active_docs.into_iter().collect();

        // Escape FTS5 query by quoting as a phrase (mitigate injection).
        let escaped = format!("\"{}\"", query.replace('"', "\"\""));

        let rows = sqlx::query("SELECT doc_id FROM rag_documents_fts WHERE text MATCH ? ORDER BY bm25(rag_documents_fts)")
            .bind(&escaped)
            .fetch_all(self.db.read_pool())
            .await;
        let mut out = Vec::new();
        if let Ok(rows) = rows {
            for row in rows {
                let id: String = row.get("doc_id");
                if set.contains(&id) {
                    out.push(id);
                }
            }
        } else {
            // fallback: try raw query
            let rows = sqlx::query("SELECT doc_id FROM rag_documents_fts WHERE text MATCH ?")
                .bind(query)
                .fetch_all(self.db.read_pool())
                .await;
            if let Ok(rows) = rows {
                for row in rows {
                    let id: String = row.get("doc_id");
                    if set.contains(&id) {
                        out.push(id);
                    }
                }
            }
        }
        Ok(out)
    }

    async fn get_embedding(&self, doc_id: &str) -> Result<(Vec<f32>, usize), RagReaderError> {
        let row = sqlx::query("SELECT embedding, dim FROM rag_embeddings WHERE doc_id = ?")
            .bind(doc_id)
            .fetch_optional(self.db.read_pool())
            .await
            .map_err(|e| RagReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?
            .ok_or_else(|| RagReaderError::ProjectNotFound(doc_id.to_string()))?;
        let bytes: Vec<u8> = row.get("embedding");
        let dim: i64 = row.get("dim");
        let v = embeddings::from_blob(&bytes, dim as usize);
        Ok((v, dim as usize))
    }

    async fn delete_embeddings_for_source(&self, project: &str, source: &str) -> Result<(), RagReaderError> {
        let pid = self.project_id(project).await?;
        sqlx::query("DELETE FROM rag_embeddings WHERE doc_id IN (SELECT id FROM rag_documents WHERE project_id = ? AND source_id = ?)")
            .bind(&pid)
            .bind(source)
            .execute(self.db.write_pool())
            .await
            .map_err(|e| RagReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        Ok(())
    }
}
