//! Tafsir search service reusing quran-normalization.
//!
//! # application::tafsir_search
//!
//! Follows the `quran_forms` pattern: derive search forms at ingest time into
//! `tafsir_search_forms`, never write normalized text into
//! `tafsir_records.text`. The search service normalizes the query using the
//! same `NormalizationPipeline` and queries the FTS5 index.
//!
//! Per T-06-05/T-06-06 (locked): canonical text is immutable. The `text`
//! column is never written by search index rebuilds.

use std::sync::Arc;

use quran_normalization::{NormalizationPipeline, ProfileId, ProfileRegistry, SemVer};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use storage::error::StorageError;
use storage_sqlite::SqliteDatabase;
use tafsir::{TafsirDimensions, TafsirId, TafsirRecord, TafsirWork};

use crate::tafsir_reader::{TafsirReader, TafsirReaderError, canonical_reference};

/// Search errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SearchError {
    /// Storage failure.
    #[error("storage failed: {0}")]
    Storage(#[from] StorageError),
    /// Reader failure.
    #[error("reader failed: {0}")]
    Reader(#[from] TafsirReaderError),
    /// Normalization failure.
    #[error("normalization failed: {0}")]
    Normalization(String),
    /// No search forms found for the given profile.
    #[error("no search forms for profile {0}")]
    NoForms(String),
}

impl storage::error::Diagnostic for SearchError {
    fn code(&self) -> storage::error::DiagnosticCode {
        match self {
            Self::Storage(_) => storage::error::DiagnosticCode::new("QAI-TAF", 10),
            Self::Reader(_) => storage::error::DiagnosticCode::new("QAI-TAF", 11),
            Self::Normalization(_) => storage::error::DiagnosticCode::new("QAI-TAF", 12),
            Self::NoForms(_) => storage::error::DiagnosticCode::new("QAI-TAF", 13),
        }
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn remedy(&self) -> Option<String> {
        Some("Check the database and retry.".to_string())
    }

    fn next_command(&self) -> Option<String> {
        Some("qai tafsir --help".to_string())
    }
}

/// A single search result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TafsirSearchResult {
    /// The tafsir record id.
    pub tafsir_id: TafsirId,
    /// The work slug.
    pub work_slug: String,
    /// The work name.
    pub work_name: String,
    /// The work author.
    pub author: String,
    /// The record dimensions (volume, page, chapter).
    pub dimensions: TafsirDimensions,
    /// A text snippet of the tafsir text.
    pub snippet: String,
    /// The canonical reference.
    pub canonical_reference: String,
}

/// Search parameters.
#[derive(Debug, Clone)]
pub struct TafsirSearchParams {
    /// Query text.
    pub query: String,
    /// Optional work filter.
    pub work_slug: Option<String>,
    /// Result limit.
    pub limit: Option<u32>,
}

/// Tafsir search service: exact + normalized search reusing quran-normalization.
pub struct TafsirSearchService {
    db: Arc<SqliteDatabase>,
    // Retained for the surface wiring in plan 06-04 (CLI/HTTP/typed-tool
    // construction); search itself queries the index directly.
    #[allow(dead_code)]
    reader: Arc<dyn TafsirReader>,
}

impl TafsirSearchService {
    /// Wrap a database handle and reader.
    pub fn new(db: Arc<SqliteDatabase>, reader: Arc<dyn TafsirReader>) -> Self {
        Self { db, reader }
    }

    /// Search for tafsir records matching the query.
    ///
    /// Normalizes the query using the L5 profile (diacritics + hamza/alif +
    /// Arabic code points) and queries the FTS5 index. Returns results with
    /// work name, author, dimensions, and a text snippet.
    pub async fn search(
        &self,
        params: &TafsirSearchParams,
    ) -> Result<Vec<TafsirSearchResult>, SearchError> {
        // Normalize the query using the L5 profile so a query that drops
        // hamza forms and diacritics still matches the indexed form — which
        // is also normalized with L5 at ingest.
        let registry = ProfileRegistry::new();
        let pipeline =
            NormalizationPipeline::for_profile(&registry, ProfileId::L5, SemVer::new(1, 0, 0))
                .map_err(|e| SearchError::Normalization(e.to_string()))?;
        let (normalized, _trace) = pipeline.apply(&params.query);
        let normalized_query = normalized.text().to_string();

        // Query the FTS5 index
        let limit = params.limit.unwrap_or(20).clamp(1, 1000) as i64;
        let mut sql = String::from(
            r#"SELECT t.id, t.volume, t.page, t.chapter, t.text, w.slug, w.name, w.author
               FROM tafsir_fts f
               JOIN tafsir_records t ON t.id = f.tafsir_id
               JOIN tafsir_works w ON w.id = t.work_id
               WHERE tafsir_fts MATCH ?"#,
        );
        if params.work_slug.is_some() {
            sql.push_str(" AND w.slug = ?");
        }
        sql.push_str(" LIMIT ?");

        let mut query = sqlx::query(&sql).bind(&normalized_query);
        if let Some(slug) = &params.work_slug {
            query = query.bind(slug);
        }
        let query = query.bind(limit);

        let rows = query.fetch_all(self.db.read_pool()).await.map_err(|e| {
            SearchError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
        })?;

        let mut results = Vec::with_capacity(rows.len());
        for row in &rows {
            let tafsir_id_str: String = row.get(0);
            let volume: Option<i64> = row.get(1);
            let page: Option<i64> = row.get(2);
            let chapter: Option<String> = row.get(3);
            let text: String = row.get(4);
            let work_slug: String = row.get(5);
            let work_name: String = row.get(6);
            let author: String = row.get(7);
            let tafsir_id = tafsir_id_str
                .parse::<TafsirId>()
                .map_err(|_| SearchError::Normalization("invalid tafsir id".to_string()))?;
            let dimensions = TafsirDimensions {
                volume: volume.map(|v| v as u32),
                page: page.map(|p| p as u32),
                chapter: chapter.clone(),
            };
            let snippet =
                if text.len() > 100 { format!("{}...", &text[..100]) } else { text.clone() };
            let reference = canonical_reference(
                &work_slug,
                dimensions.volume,
                dimensions.page,
                dimensions.chapter.as_deref(),
            );
            results.push(TafsirSearchResult {
                tafsir_id,
                work_slug,
                work_name,
                author,
                dimensions,
                snippet,
                canonical_reference: reference,
            });
        }
        Ok(results)
    }

    /// Derive search forms for a tafsir record and insert them into
    /// `tafsir_search_forms`. This is the ingest-time operation that
    /// populates the FTS5 index.
    ///
    /// Per T-06-05/T-06-06 (locked): canonical text is never mutated. Search
    /// forms are derived into a separate table.
    pub async fn derive_search_forms(
        &self,
        record: &TafsirRecord,
        work: &TafsirWork,
        corpus_generation: i64,
    ) -> Result<(), SearchError> {
        let registry = ProfileRegistry::new();
        let pipeline_l2 =
            NormalizationPipeline::for_profile(&registry, ProfileId::L2, SemVer::new(1, 0, 0))
                .map_err(|e| SearchError::Normalization(e.to_string()))?;
        let pipeline_l5 =
            NormalizationPipeline::for_profile(&registry, ProfileId::L5, SemVer::new(1, 0, 0))
                .map_err(|e| SearchError::Normalization(e.to_string()))?;

        let (exact, _) = pipeline_l2.apply(&record.text);
        let (normalized, _) = pipeline_l5.apply(&record.text);

        // Insert search forms
        sqlx::query(
            r#"INSERT INTO tafsir_search_forms
               (tafsir_id, form_type, form_text, profile_id, profile_version, corpus_generation)
               VALUES (?, 'exact', ?, 'L2', '1.0.0', ?)"#,
        )
        .bind(record.id.to_string())
        .bind(exact.text().to_string())
        .bind(corpus_generation)
        .execute(self.db.write_pool())
        .await
        .map_err(|e| {
            SearchError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
        })?;

        sqlx::query(
            r#"INSERT INTO tafsir_search_forms
               (tafsir_id, form_type, form_text, profile_id, profile_version, corpus_generation)
               VALUES (?, 'normalized', ?, 'L5', '1.0.0', ?)"#,
        )
        .bind(record.id.to_string())
        .bind(normalized.text().to_string())
        .bind(corpus_generation)
        .execute(self.db.write_pool())
        .await
        .map_err(|e| {
            SearchError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
        })?;

        // Insert into FTS5.
        // The index holds the L5-normalized text, not the canonical text: the
        // query path normalizes with L5 too, so both sides agree. SQLite's
        // `remove_diacritics` only folds Latin marks, so relying on the
        // tokenizer to strip Arabic diacritics would silently fail. T-06-05
        // holds — `tafsir_records.text` is never written here.
        sqlx::query(
            r#"INSERT INTO tafsir_fts (tafsir_id, content_text, author, verse_reference)
               VALUES (?, ?, ?, ?)"#,
        )
        .bind(record.id.to_string())
        .bind(normalized.text())
        .bind(&work.author)
        .bind(record.chapter.as_deref().unwrap_or(""))
        .execute(self.db.write_pool())
        .await
        .map_err(|e| {
            SearchError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tafsir_reader::TafsirReaderService;
    use domain::{ContentHash, HashAlgorithm, Timestamp};
    use tafsir::{TafsirWorkId, WorkStatus};

    async fn test_db() -> Arc<SqliteDatabase> {
        // The TempDir must outlive the pool: dropping it deletes the database
        // file while open connections still hold it (SQLITE_IOERR).
        let dir = Box::leak(Box::new(tempfile::tempdir().unwrap()));
        let db = SqliteDatabase::new(&dir.path().join("test.db").display().to_string(), 1, false)
            .await
            .unwrap();
        let migrations = [
            include_str!("../../../migrations/sqlite/0001_core.up.sql"),
            include_str!("../../../migrations/sqlite/0002_sources.up.sql"),
            include_str!("../../../migrations/sqlite/0003_provenance.up.sql"),
            include_str!("../../../migrations/sqlite/0004_jobs.up.sql"),
            include_str!("../../../migrations/sqlite/0005_audit.up.sql"),
            include_str!("../../../migrations/sqlite/0006_outbox_generations_tombstones.up.sql"),
            include_str!("../../../migrations/sqlite/0007_quran_editions.up.sql"),
            include_str!("../../../migrations/sqlite/0008_quran_structure.up.sql"),
            include_str!("../../../migrations/sqlite/0009_quran_divisions.up.sql"),
            include_str!("../../../migrations/sqlite/0010_quran_translations.up.sql"),
            include_str!("../../../migrations/sqlite/0011_quran_staging.up.sql"),
            include_str!("../../../migrations/sqlite/0012_quran_validation.up.sql"),
            include_str!("../../../migrations/sqlite/0013_quran_normalization.up.sql"),
            include_str!("../../../migrations/sqlite/0014_quran_forms.up.sql"),
            include_str!("../../../migrations/sqlite/0015_quran_indexes.up.sql"),
            include_str!("../../../migrations/sqlite/0016_quran_search_cache.up.sql"),
            include_str!("../../../migrations/sqlite/0017_quran_lexicon.up.sql"),
            include_str!("../../../migrations/sqlite/0018_morphology_staging.up.sql"),
            include_str!("../../../migrations/sqlite/0019_quran_graph.up.sql"),
            include_str!("../../../migrations/sqlite/0020_quran_edition_identity.up.sql"),
            include_str!("../../../migrations/sqlite/0021_canonical_write_fence.up.sql"),
            include_str!("../../../migrations/sqlite/0022_quran_graph_fix.up.sql"),
            include_str!("../../../migrations/sqlite/0023_hadith_collections.up.sql"),
            include_str!("../../../migrations/sqlite/0024_hadith_records.up.sql"),
            include_str!("../../../migrations/sqlite/0025_hadith_gradings.up.sql"),
            include_str!("../../../migrations/sqlite/0026_hadith_search_forms.up.sql"),
            include_str!("../../../migrations/sqlite/0027_tafsir_works.up.sql"),
            include_str!("../../../migrations/sqlite/0028_tafsir_records.up.sql"),
            include_str!("../../../migrations/sqlite/0029_tafsir_search_forms.up.sql"),
        ];
        for migration in migrations {
            sqlx::raw_sql(migration).execute(db.write_pool()).await.unwrap();
        }
        for sql in [
            "INSERT INTO sources (id, title, content_type, created_at, updated_at)
             VALUES ('src-tafsir', 'Tafsir test source', 'tafsir',
                     '2026-01-15T00:00:00Z', '2026-01-15T00:00:00Z')",
            "INSERT INTO source_versions
                 (id, source_id, version, schema_version, state, trust_level,
                  license_status, license_json, created_at)
             VALUES ('018f3a2b-4c5d-7e6f-8a9b-0c1d2e3f4a5b', 'src-tafsir', '0.1.0', 1, 'Staged', 'ImportedUnverified',
                     'PublicDomain', '{}', '2026-01-15T00:00:00Z')",
        ] {
            sqlx::query(sql).execute(db.write_pool()).await.unwrap();
        }
        Arc::new(db)
    }

    fn sample_work() -> TafsirWork {
        TafsirWork {
            id: TafsirWorkId::new(),
            slug: "tafsir-tabari".to_string(),
            name: "Jami' al-Bayan (Tabari)".to_string(),
            author: "Muhammad ibn Jarir al-Tabari".to_string(),
            dimensions_schema_json: r#"{"chapter":"surah-number"}"#.to_string(),
            version: SemVer::new(1, 0, 0),
            source_version_id: "018f3a2b-4c5d-7e6f-8a9b-0c1d2e3f4a5b".parse().unwrap(),
            created_at: Timestamp::from_ymd_hms(2026, 1, 15, 0, 0, 0).unwrap(),
            status: WorkStatus::Active,
        }
    }

    fn sample_record(work_id: &TafsirWorkId) -> TafsirRecord {
        TafsirRecord {
            id: TafsirId::new(),
            work_id: *work_id,
            volume: Some(3),
            page: Some(12),
            chapter: Some("2".to_string()),
            text: "القول في تأويل قوله تعالى".to_string(),
            text_hash: ContentHash::try_new(HashAlgorithm::Sha256, "ab".repeat(32)).unwrap(),
            source_version_id: "018f3a2b-4c5d-7e6f-8a9b-0c1d2e3f4a5b".parse().unwrap(),
            status: WorkStatus::Active,
            created_at: Timestamp::from_ymd_hms(2026, 1, 15, 0, 0, 0).unwrap(),
        }
    }

    #[tokio::test]
    async fn search_returns_results() {
        let db = test_db().await;
        let reader = Arc::new(TafsirReaderService::new(db.clone()));
        let search = TafsirSearchService::new(db, reader.clone());

        let work = sample_work();
        reader.insert_work(&work).await.unwrap();
        let record = sample_record(&work.id);
        reader.insert_record(&record).await.unwrap();
        search.derive_search_forms(&record, &work, 1).await.unwrap();

        let results = search
            .search(&TafsirSearchParams {
                query: "القول في تأويل".to_string(),
                work_slug: None,
                limit: None,
            })
            .await
            .unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].work_slug, "tafsir-tabari");
        assert_eq!(results[0].author, "Muhammad ibn Jarir al-Tabari");
        assert_eq!(results[0].dimensions.chapter.as_deref(), Some("2"));

        // Canonical text is never modified by search operations.
        let stored = reader.get_record(&record.id).await.unwrap();
        assert_eq!(stored.text, "القول في تأويل قوله تعالى");
    }

    #[tokio::test]
    async fn normalized_search_matches() {
        let db = test_db().await;
        let reader = Arc::new(TafsirReaderService::new(db.clone()));
        let search = TafsirSearchService::new(db, reader.clone());

        let work = sample_work();
        reader.insert_work(&work).await.unwrap();
        let record = sample_record(&work.id);
        reader.insert_record(&record).await.unwrap();
        search.derive_search_forms(&record, &work, 1).await.unwrap();

        // Search without diacritics/hamza forms (normalized form)
        let results = search
            .search(&TafsirSearchParams {
                query: "القول في تاويل قوله تعالي".to_string(),
                work_slug: None,
                limit: None,
            })
            .await
            .unwrap();
        assert!(!results.is_empty());
    }
}
