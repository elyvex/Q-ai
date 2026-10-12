//! Hadith search service reusing quran-normalization.
//!
//! # application::hadith_search
//!
//! Follows the `quran_forms` pattern: derive search forms at ingest time into
//! `hadith_search_forms`, never write normalized text into
//! `hadith_records.matn_text`. The search service normalizes the query using
//! the same `NormalizationPipeline` and queries the FTS5 index.
//!
//! Per D-02 (locked): canonical text is immutable. The `matn_text` column is
//! never written by search index rebuilds.

use std::sync::Arc;

use hadith_core::{HadithCollection, HadithId, HadithRecord};
use quran_normalization::{NormalizationPipeline, ProfileId, ProfileRegistry, SemVer};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use storage::error::StorageError;
use storage_sqlite::SqliteDatabase;

use crate::hadith_reader::{HadithReader, HadithReaderError};

/// Search errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SearchError {
    /// Storage failure.
    #[error("storage failed: {0}")]
    Storage(#[from] StorageError),
    /// Reader failure.
    #[error("reader failed: {0}")]
    Reader(#[from] HadithReaderError),
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
            Self::Storage(_) => storage::error::DiagnosticCode::new("QAI-HAD", 10),
            Self::Reader(_) => storage::error::DiagnosticCode::new("QAI-HAD", 11),
            Self::Normalization(_) => storage::error::DiagnosticCode::new("QAI-HAD", 12),
            Self::NoForms(_) => storage::error::DiagnosticCode::new("QAI-HAD", 13),
        }
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn remedy(&self) -> Option<String> {
        Some("Check the database and retry.".to_string())
    }

    fn next_command(&self) -> Option<String> {
        Some("qai hadith --help".to_string())
    }
}

/// A single search result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HadithSearchResult {
    /// The hadith record id.
    pub hadith_id: HadithId,
    /// The collection slug.
    pub collection_slug: String,
    /// The source number (preserved verbatim).
    pub source_number: String,
    /// A text snippet of the matn.
    pub snippet: String,
    /// The canonical reference.
    pub canonical_reference: String,
}

/// Search parameters.
#[derive(Debug, Clone)]
pub struct HadithSearchParams {
    /// Query text.
    pub query: String,
    /// Optional collection filter.
    pub collection_slug: Option<String>,
    /// Result limit.
    pub limit: Option<u32>,
}

/// Hadith search service: exact + normalized search reusing quran-normalization.
pub struct HadithSearchService {
    db: Arc<SqliteDatabase>,
    // Retained for the surface wiring in plan 06-04 (CLI/HTTP/typed-tool
    // construction); search itself queries the index directly.
    #[allow(dead_code)]
    reader: Arc<dyn HadithReader>,
}

impl HadithSearchService {
    /// Wrap a database handle and reader.
    pub fn new(db: Arc<SqliteDatabase>, reader: Arc<dyn HadithReader>) -> Self {
        Self { db, reader }
    }

    /// Search for hadith records matching the query.
    ///
    /// Normalizes the query using the L3 profile (diacritics stripped) and
    /// queries the FTS5 index. Returns results with collection name,
    /// source_number, and a text snippet.
    pub async fn search(
        &self,
        params: &HadithSearchParams,
    ) -> Result<Vec<HadithSearchResult>, SearchError> {
        // Normalize the query using the L5 profile. L5 is L3 (diacritics)
        // plus hamza/alif (L4) plus Arabic/Persian code points (L5), so a
        // query that drops hamza forms and diacritics still matches the
        // indexed form — which is also normalized with L5 at ingest.
        let registry = ProfileRegistry::new();
        let pipeline =
            NormalizationPipeline::for_profile(&registry, ProfileId::L5, SemVer::new(1, 0, 0))
                .map_err(|e| SearchError::Normalization(e.to_string()))?;
        let (normalized, _trace) = pipeline.apply(&params.query);
        let normalized_query = normalized.text().to_string();

        // Query the FTS5 index
        let limit = params.limit.unwrap_or(20).clamp(1, 1000) as i64;
        let mut sql = String::from(
            r#"SELECT h.id, h.source_number, h.matn_text, c.slug
               FROM hadith_fts f
               JOIN hadith_records h ON h.id = f.hadith_id
               JOIN hadith_collections c ON c.id = h.collection_id
               WHERE hadith_fts MATCH ?
               LIMIT ?"#,
        );
        if params.collection_slug.is_some() {
            sql.push_str(" AND c.slug = ?");
        }

        let mut query = sqlx::query(&sql).bind(&normalized_query).bind(limit);
        if let Some(slug) = &params.collection_slug {
            query = query.bind(slug);
        }

        let rows = query.fetch_all(self.db.read_pool()).await.map_err(|e| {
            SearchError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
        })?;

        let mut results = Vec::with_capacity(rows.len());
        for row in &rows {
            let hadith_id_str: String = row.get(0);
            let source_number: String = row.get(1);
            let matn_text: String = row.get(2);
            let collection_slug: String = row.get(3);
            let hadith_id = hadith_id_str
                .parse::<HadithId>()
                .map_err(|_| SearchError::Normalization("invalid hadith id".to_string()))?;
            let snippet = if matn_text.len() > 100 {
                format!("{}...", &matn_text[..100])
            } else {
                matn_text.clone()
            };
            results.push(HadithSearchResult {
                hadith_id,
                collection_slug: collection_slug.clone(),
                source_number: source_number.clone(),
                snippet,
                canonical_reference: format!("{collection_slug}:{source_number}"),
            });
        }
        Ok(results)
    }

    /// Derive search forms for a hadith record and insert them into
    /// `hadith_search_forms`. This is the ingest-time operation that
    /// populates the FTS5 index.
    ///
    /// Per D-02 (locked): canonical text is never mutated. Search forms are
    /// derived into a separate table.
    pub async fn derive_search_forms(
        &self,
        record: &HadithRecord,
        collection: &HadithCollection,
        corpus_generation: i64,
    ) -> Result<(), SearchError> {
        let registry = ProfileRegistry::new();
        let pipeline_l2 =
            NormalizationPipeline::for_profile(&registry, ProfileId::L2, SemVer::new(1, 0, 0))
                .map_err(|e| SearchError::Normalization(e.to_string()))?;
        let pipeline_l5 =
            NormalizationPipeline::for_profile(&registry, ProfileId::L5, SemVer::new(1, 0, 0))
                .map_err(|e| SearchError::Normalization(e.to_string()))?;

        let (exact, _) = pipeline_l2.apply(&record.matn_text);
        let (normalized, _) = pipeline_l5.apply(&record.matn_text);

        // Insert search forms
        sqlx::query(
            r#"INSERT INTO hadith_search_forms
               (hadith_id, form_type, form_text, profile_id, profile_version, corpus_generation)
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
            r#"INSERT INTO hadith_search_forms
               (hadith_id, form_type, form_text, profile_id, profile_version, corpus_generation)
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
        // The index holds the L3-normalized matn, not the canonical text: the
        // query path normalizes with L3 too, so both sides agree. SQLite's
        // `remove_diacritics` only folds Latin marks, so relying on the
        // tokenizer to strip Arabic diacritics would silently fail. D-02
        // holds — `hadith_records.matn_text` is never written here.
        let narrator_names = record.narrator_names.join(" ");
        sqlx::query(
            r#"INSERT INTO hadith_fts (hadith_id, matn_text, narrator_names, collection_name)
               VALUES (?, ?, ?, ?)"#,
        )
        .bind(record.id.to_string())
        .bind(normalized.text())
        .bind(&narrator_names)
        .bind(&collection.slug)
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
    use crate::hadith_reader::HadithReaderService;
    use domain::{ContentHash, HashAlgorithm, Timestamp};
    use hadith_core::{HadithCollectionId, RecordStatus, Tradition};

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
        ];
        for migration in migrations {
            sqlx::raw_sql(migration).execute(db.write_pool()).await.unwrap();
        }
        for sql in [
            "INSERT INTO sources (id, title, content_type, created_at, updated_at)
             VALUES ('src-hadith', 'Hadith test source', 'hadith_collection',
                     '2026-01-15T00:00:00Z', '2026-01-15T00:00:00Z')",
            "INSERT INTO source_versions
                 (id, source_id, version, schema_version, state, trust_level,
                  license_status, license_json, created_at)
             VALUES ('018f3a2b-4c5d-7e6f-8a9b-0c1d2e3f4a5b', 'src-hadith', '0.1.0', 1, 'Staged', 'ImportedUnverified',
                     'PublicDomain', '{}', '2026-01-15T00:00:00Z')",
        ] {
            sqlx::query(sql).execute(db.write_pool()).await.unwrap();
        }
        Arc::new(db)
    }

    fn sample_collection() -> HadithCollection {
        HadithCollection {
            id: HadithCollectionId::new(),
            slug: "bukhari-arabic".to_string(),
            name: "Sahih al-Bukhari (Arabic)".to_string(),
            tradition: Tradition::Sunni,
            source_url: None,
            license_json: r#"{"status":"PublicDomain"}"#.to_string(),
            text_hash: ContentHash::try_new(HashAlgorithm::Sha256, "ab".repeat(32)).unwrap(),
            structure_hash: ContentHash::try_new(HashAlgorithm::Sha256, "cd".repeat(32)).unwrap(),
            version: SemVer::new(1, 0, 0),
            source_version_id: "018f3a2b-4c5d-7e6f-8a9b-0c1d2e3f4a5b".parse().unwrap(),
            imported_at: Timestamp::from_ymd_hms(2026, 1, 15, 0, 0, 0).unwrap(),
            status: RecordStatus::Active,
        }
    }

    fn sample_record(collection_id: &HadithCollectionId) -> HadithRecord {
        HadithRecord {
            id: HadithId::new(),
            collection_id: *collection_id,
            source_number: "bukhari:1234".to_string(),
            matn_text: "إنما الأعمال بالنيات".to_string(),
            matn_hash: ContentHash::try_new(HashAlgorithm::Sha256, "ab".repeat(32)).unwrap(),
            isnad_text: Some("حدثنا يحيى بن بكير".to_string()),
            narrator_names: vec!["يحيى بن بكير".to_string()],
            source_version_id: "018f3a2b-4c5d-7e6f-8a9b-0c1d2e3f4a5b".parse().unwrap(),
            status: RecordStatus::Active,
            metadata_json: serde_json::json!({}),
            created_at: Timestamp::from_ymd_hms(2026, 1, 15, 0, 0, 0).unwrap(),
        }
    }

    #[tokio::test]
    async fn search_returns_results() {
        let db = test_db().await;
        let reader = Arc::new(HadithReaderService::new(db.clone()));
        let search = HadithSearchService::new(db, reader.clone());

        let collection = sample_collection();
        reader.insert_collection(&collection).await.unwrap();
        let record = sample_record(&collection.id);
        reader.insert_record(&record).await.unwrap();
        search.derive_search_forms(&record, &collection, 1).await.unwrap();

        let results = search
            .search(&HadithSearchParams {
                query: "إنما الأعمال بالنيات".to_string(),
                collection_slug: None,
                limit: None,
            })
            .await
            .unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].source_number, "bukhari:1234");
        assert_eq!(results[0].collection_slug, "bukhari-arabic");
    }

    #[tokio::test]
    async fn normalized_search_matches() {
        let db = test_db().await;
        let reader = Arc::new(HadithReaderService::new(db.clone()));
        let search = HadithSearchService::new(db, reader.clone());

        let collection = sample_collection();
        reader.insert_collection(&collection).await.unwrap();
        let record = sample_record(&collection.id);
        reader.insert_record(&record).await.unwrap();
        search.derive_search_forms(&record, &collection, 1).await.unwrap();

        // Search without diacritics (normalized form)
        let results = search
            .search(&HadithSearchParams {
                query: "انما الاعمال بالنيات".to_string(),
                collection_slug: None,
                limit: None,
            })
            .await
            .unwrap();
        assert!(!results.is_empty());
    }
}
