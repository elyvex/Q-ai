//! Scripture search service reusing quran-normalization.
//!
//! # application::scripture_search
//!
//! Follows the `quran_forms` pattern: derive search forms at ingest time into
//! `scripture_search_forms`, never write normalized text into
//! `scripture_verses.text`. The search service normalizes the query using the
//! same `NormalizationPipeline` and queries the FTS5 index.

use std::sync::Arc;

use quran_normalization::{NormalizationPipeline, ProfileId, ProfileRegistry, SemVer};
use scripture::{ScriptureVerse, ScriptureVerseId};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use storage::error::StorageError;
use storage_sqlite::SqliteDatabase;

use crate::scripture_reader::{ScriptureReader, ScriptureReaderError};

/// Search errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SearchError {
    #[error("storage failed: {0}")]
    Storage(#[from] StorageError),
    #[error("reader failed: {0}")]
    Reader(#[from] ScriptureReaderError),
    #[error("normalization failed: {0}")]
    Normalization(String),
    #[error("no search forms for profile {0}")]
    NoForms(String),
}

impl storage::error::Diagnostic for SearchError {
    fn code(&self) -> storage::error::DiagnosticCode {
        match self {
            Self::Storage(_) => storage::error::DiagnosticCode::new("QAI-SCR", 10),
            Self::Reader(_) => storage::error::DiagnosticCode::new("QAI-SCR", 11),
            Self::Normalization(_) => storage::error::DiagnosticCode::new("QAI-SCR", 12),
            Self::NoForms(_) => storage::error::DiagnosticCode::new("QAI-SCR", 13),
        }
    }
    fn summary(&self) -> String { self.to_string() }
    fn remedy(&self) -> Option<String> { Some("Check the database and retry.".to_string()) }
    fn next_command(&self) -> Option<String> { Some("qai scripture --help".to_string()) }
}

/// A single search result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptureSearchResult {
    pub verse_id: ScriptureVerseId,
    pub edition: String,
    pub book: String,
    pub chapter: u32,
    pub verse: u32,
    pub snippet: String,
    pub score: f64,
}

/// Search parameters.
#[derive(Debug, Clone)]
pub struct ScriptureSearchParams {
    pub query: String,
    pub edition: Option<String>,
    pub limit: Option<u32>,
}

/// Scripture search service.
pub struct ScriptureSearchService {
    db: Arc<SqliteDatabase>,
    #[allow(dead_code)]
    reader: Arc<dyn ScriptureReader>,
}

impl ScriptureSearchService {
    pub fn new(db: Arc<SqliteDatabase>, reader: Arc<dyn ScriptureReader>) -> Self {
        Self { db, reader }
    }

    pub async fn search(
        &self,
        params: &ScriptureSearchParams,
    ) -> Result<Vec<ScriptureSearchResult>, SearchError> {
        let registry = ProfileRegistry::new();
        let pipeline =
            NormalizationPipeline::for_profile(&registry, ProfileId::L5, SemVer::new(1, 0, 0))
                .map_err(|e| SearchError::Normalization(e.to_string()))?;
        let (normalized, _trace) = pipeline.apply(&params.query);
        let normalized_query = normalized.text().to_string();

        // Escape FTS5 query by phrase-quoting.
        let escaped = format!("\"{}\"", normalized_query.replace('"', "\"\""));

        let limit = params.limit.unwrap_or(20).clamp(1, 1000) as i64;
        let mut sql = String::from(
            r#"SELECT v.id, v.book, v.chapter, v.verse, v.text, e.slug, bm25(scripture_fts) AS score
               FROM scripture_fts f
               JOIN scripture_verses v ON v.id = f.verse_id
               JOIN scripture_editions e ON e.id = v.edition_id
               WHERE scripture_fts MATCH ?"#,
        );
        if params.edition.is_some() {
            sql.push_str(" AND e.slug = ?");
        }
        sql.push_str(" ORDER BY score LIMIT ?");

        let mut query = sqlx::query(&sql).bind(&escaped);
        if let Some(edition) = &params.edition {
            query = query.bind(edition);
        }
        let query = query.bind(limit);

        let rows = query.fetch_all(self.db.read_pool()).await.map_err(|e| {
            SearchError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
        })?;

        let mut results = Vec::with_capacity(rows.len());
        for row in &rows {
            let verse_id_str: String = row.get(0);
            let book: String = row.get(1);
            let chapter: i64 = row.get(2);
            let verse: i64 = row.get(3);
            let text: String = row.get(4);
            let edition: String = row.get(5);
            let score: f64 = row.get(6);
            let verse_id = verse_id_str
                .parse::<ScriptureVerseId>()
                .map_err(|_| SearchError::Normalization("invalid verse id".to_string()))?;
            let snippet = if text.len() > 100 { format!("{}...", &text[..100]) } else { text.clone() };
            results.push(ScriptureSearchResult {
                verse_id,
                edition,
                book,
                chapter: chapter as u32,
                verse: verse as u32,
                snippet,
                score,
            });
        }
        Ok(results)
    }

    /// Derive search forms for a verse and insert them.
    pub async fn derive_search_forms(
        &self,
        verse: &ScriptureVerse,
        corpus_generation: i64,
    ) -> Result<(), SearchError> {
        let registry = ProfileRegistry::new();
        let pipeline_l2 =
            NormalizationPipeline::for_profile(&registry, ProfileId::L2, SemVer::new(1, 0, 0))
                .map_err(|e| SearchError::Normalization(e.to_string()))?;
        let pipeline_l5 =
            NormalizationPipeline::for_profile(&registry, ProfileId::L5, SemVer::new(1, 0, 0))
                .map_err(|e| SearchError::Normalization(e.to_string()))?;

        let (exact, _) = pipeline_l2.apply(&verse.text);
        let (normalized, _) = pipeline_l5.apply(&verse.text);

        sqlx::query(
            r#"INSERT INTO scripture_search_forms
               (verse_id, form_type, form_text, profile_id, profile_version, corpus_generation)
               VALUES (?, 'exact', ?, 'L2', '1.0.0', ?)"#,
        )
        .bind(verse.id.to_string())
        .bind(exact.text().to_string())
        .bind(corpus_generation)
        .execute(self.db.write_pool())
        .await
        .map_err(|e| SearchError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;

        sqlx::query(
            r#"INSERT INTO scripture_search_forms
               (verse_id, form_type, form_text, profile_id, profile_version, corpus_generation)
               VALUES (?, 'normalized', ?, 'L5', '1.0.0', ?)"#,
        )
        .bind(verse.id.to_string())
        .bind(normalized.text().to_string())
        .bind(corpus_generation)
        .execute(self.db.write_pool())
        .await
        .map_err(|e| SearchError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;

        // FTS5 index holds L5-normalized text.
        sqlx::query(
            r#"INSERT INTO scripture_fts (verse_id, form_text) VALUES (?, ?)"#,
        )
        .bind(verse.id.to_string())
        .bind(normalized.text())
        .execute(self.db.write_pool())
        .await
        .map_err(|e| SearchError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;

        Ok(())
    }

    /// Alias for derive_search_forms with default corpus_generation 1.
    pub async fn index_verse(
        &self,
        verse: &ScriptureVerse,
    ) -> Result<(), SearchError> {
        self.derive_search_forms(verse, 1).await
    }

    /// Alias accepting corpus_generation.
    pub async fn index_verse_with_generation(
        &self,
        verse: &ScriptureVerse,
        corpus_generation: i64,
    ) -> Result<(), SearchError> {
        self.derive_search_forms(verse, corpus_generation).await
    }
}

