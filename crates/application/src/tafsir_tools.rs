//! Tafsir typed tool backend: implements `TafsirBackend` for the tafsir reader.
//!
//! # application::tafsir_tools
//!
//! Each method returns `ToolResult<serde_json::Value>` with research_checksum,
//! canonical_references, and analysis_sources. Per T-06-07 (locked): every
//! tool result carries research_checksum + ReproducibilityData, and canonical
//! references are fully qualified.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use tool_registry::{
    BackendMeta, TAFSIR_SEARCH_VERSION, TafsirBackend, TafsirRef, TafsirSearchToolParams,
    TafsirView,
};
use tools::{AnalysisSource, ToolError, ToolResult, reproducibility, research_checksum};

use crate::tafsir_reader::{TafsirReader, TafsirReaderError, TafsirReaderService, encode_hash};
use crate::tafsir_search::{SearchError, TafsirSearchParams, TafsirSearchService};

/// Map a tafsir reader error to a tool error.
fn reader_error(error: TafsirReaderError) -> ToolError {
    use storage::error::Diagnostic;
    ToolError::Backend { code: error.code().to_string(), detail: error.to_string() }
}

/// Map a search error to a tool error.
fn search_error(error: SearchError) -> ToolError {
    use storage::error::Diagnostic;
    ToolError::Backend { code: error.code().to_string(), detail: error.to_string() }
}

/// The tafsir tool backend: `TafsirReaderService` + `TafsirSearchService`.
pub struct TafsirToolBackend {
    reader: Arc<TafsirReaderService>,
    search: Arc<TafsirSearchService>,
}

impl TafsirToolBackend {
    /// Wrap a tafsir reader and search service.
    pub fn new(reader: Arc<TafsirReaderService>, search: Arc<TafsirSearchService>) -> Self {
        Self { reader, search }
    }

    /// The underlying reader service.
    pub fn reader(&self) -> &TafsirReaderService {
        &self.reader
    }
}

#[async_trait]
impl TafsirBackend for TafsirToolBackend {
    async fn backend_get_tafsir(
        &self,
        reference: &TafsirRef,
    ) -> Result<(TafsirView, BackendMeta), ToolError> {
        // Look up the work by slug
        let work =
            self.reader.get_work_by_slug(&reference.work_slug).await.map_err(reader_error)?;

        // Resolve the record by dimensions: every provided dimension field
        // must match. Slug-only references resolve to the first record in
        // work order (deterministic: volume, page, created_at).
        let records = self.reader.list_records_for_work(&work.id).await.map_err(reader_error)?;
        let record = records
            .into_iter()
            .find(|r| {
                reference.volume.is_none_or(|v| r.volume == Some(v))
                    && reference.page.is_none_or(|p| r.page == Some(p))
                    && reference.chapter.as_deref().is_none_or(|c| r.chapter.as_deref() == Some(c))
            })
            .ok_or_else(|| ToolError::Backend {
                code: "QAI-TAF-0003".to_string(),
                detail: format!("tafsir record not found: {reference}"),
            })?;

        let canonical = crate::tafsir_reader::canonical_reference(
            &work.slug,
            record.volume,
            record.page,
            record.chapter.as_deref(),
        );
        let view = TafsirView {
            id: record.id.to_string(),
            work_slug: work.slug.clone(),
            work_name: work.name.clone(),
            author: work.author.clone(),
            volume: record.volume,
            page: record.page,
            chapter: record.chapter.clone(),
            text: record.text.clone(),
            canonical_reference: canonical,
        };

        let meta = BackendMeta {
            edition_slug: work.slug.clone(),
            edition_version: work.version.to_string(),
            edition_id: work.id.to_string(),
            corpus_generation: 0,
            text_hash: encode_hash(&record.text_hash),
            script: String::new(),
            riwayah: None,
            numbering_scheme: String::new(),
            projection_id: String::new(),
            builder_version: String::new(),
        };

        Ok((view, meta))
    }

    async fn backend_search_tafsir(
        &self,
        params: &TafsirSearchToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let started = Instant::now();
        if params.query.trim().is_empty() {
            return Err(ToolError::InvalidInput {
                tool: "tafsir.search",
                detail: "query text must not be empty".to_string(),
            });
        }

        let search_params = TafsirSearchParams {
            query: params.query.clone(),
            work_slug: params.work_slug.clone(),
            limit: params.limit,
        };
        let results = self.search.search(&search_params).await.map_err(search_error)?;

        let canonical_references: Vec<String> =
            results.iter().map(|r| r.canonical_reference.clone()).collect();
        let analysis_sources: Vec<AnalysisSource> = canonical_references
            .iter()
            .map(|r| AnalysisSource { kind: "tafsir".to_string(), reference: r.clone() })
            .collect();

        let payload = serde_json::to_value(&results).unwrap_or(serde_json::Value::Null);
        let query = serde_json::to_value(params).unwrap_or(serde_json::Value::Null);

        let result = ToolResult {
            tool_name: "tafsir.search".to_string(),
            tool_version: TAFSIR_SEARCH_VERSION,
            query: query.clone(),
            normalization_rules: vec!["N01".to_string(), "N03".to_string()],
            edition_id: None,
            edition_version: None,
            results: payload.clone(),
            canonical_references,
            analysis_sources,
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                "tafsir.search",
                TAFSIR_SEARCH_VERSION,
                &query,
                None,
                None,
                BTreeMap::new(),
                0,
            ),
            research_checksum: research_checksum(
                "tafsir.search",
                TAFSIR_SEARCH_VERSION,
                &query,
                &serde_json::json!({}),
                &payload,
            )
            .expect("research checksum input is JSON-serializable"),
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tafsir_search::TafsirSearchService;
    use domain::{ContentHash, HashAlgorithm, Timestamp};
    use std::sync::Arc;
    use storage_sqlite::SqliteDatabase;
    use tafsir::{TafsirId, TafsirRecord, TafsirWork, TafsirWorkId, WorkStatus};

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
            version: domain::SemVer::new(1, 0, 0),
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
    async fn tafsir_get_returns_view_with_work_and_dimensions() {
        let db = test_db().await;
        let reader = Arc::new(TafsirReaderService::new(db.clone()));
        let search = Arc::new(TafsirSearchService::new(db.clone(), reader.clone()));
        let backend = TafsirToolBackend::new(reader.clone(), search);

        let work = sample_work();
        reader.insert_work(&work).await.unwrap();
        let record = sample_record(&work.id);
        reader.insert_record(&record).await.unwrap();

        let reference = TafsirRef::parse("tafsir-tabari:2").unwrap();
        let (view, _meta) = backend.backend_get_tafsir(&reference).await.unwrap();
        assert_eq!(view.work_slug, "tafsir-tabari");
        assert_eq!(view.author, "Muhammad ibn Jarir al-Tabari");
        assert_eq!(view.text, "القول في تأويل قوله تعالى");
        assert_eq!(view.volume, Some(3));
        assert_eq!(view.page, Some(12));
        assert_eq!(view.chapter.as_deref(), Some("2"));
        assert_eq!(view.canonical_reference, "tafsir-tabari:2");
    }

    #[tokio::test]
    async fn tafsir_search_returns_tool_result_with_checksum() {
        let db = test_db().await;
        let reader = Arc::new(TafsirReaderService::new(db.clone()));
        let search = Arc::new(TafsirSearchService::new(db.clone(), reader.clone()));
        let backend = TafsirToolBackend::new(reader.clone(), search.clone());

        let work = sample_work();
        reader.insert_work(&work).await.unwrap();
        let record = sample_record(&work.id);
        reader.insert_record(&record).await.unwrap();
        search.derive_search_forms(&record, &work, 1).await.unwrap();

        let params = TafsirSearchToolParams {
            query: "القول في تأويل".to_string(),
            work_slug: None,
            limit: None,
        };
        let result = backend.backend_search_tafsir(&params).await.unwrap();
        assert_eq!(result.tool_name, "tafsir.search");
        assert!(!result.canonical_references.is_empty());
        assert_eq!(result.canonical_references[0], "tafsir-tabari:2");
        assert!(!result.analysis_sources.is_empty());
        assert_eq!(result.analysis_sources[0].kind, "tafsir");
    }

    #[tokio::test]
    async fn tafsir_search_checksum_is_deterministic() {
        let db = test_db().await;
        let reader = Arc::new(TafsirReaderService::new(db.clone()));
        let search = Arc::new(TafsirSearchService::new(db.clone(), reader.clone()));
        let backend = TafsirToolBackend::new(reader.clone(), search.clone());

        let work = sample_work();
        reader.insert_work(&work).await.unwrap();
        let record = sample_record(&work.id);
        reader.insert_record(&record).await.unwrap();
        search.derive_search_forms(&record, &work, 1).await.unwrap();

        let params = TafsirSearchToolParams {
            query: "القول في تأويل".to_string(),
            work_slug: None,
            limit: None,
        };
        let first = backend.backend_search_tafsir(&params).await.unwrap();
        let second = backend.backend_search_tafsir(&params).await.unwrap();
        assert_eq!(first.research_checksum, second.research_checksum);
    }

    #[tokio::test]
    async fn tafsir_search_empty_query_is_invalid_input() {
        let db = test_db().await;
        let reader = Arc::new(TafsirReaderService::new(db.clone()));
        let search = Arc::new(TafsirSearchService::new(db.clone(), reader.clone()));
        let backend = TafsirToolBackend::new(reader.clone(), search);

        let params =
            TafsirSearchToolParams { query: "   ".to_string(), work_slug: None, limit: None };
        let err = backend.backend_search_tafsir(&params).await.unwrap_err();
        assert!(matches!(err, ToolError::InvalidInput { tool: "tafsir.search", .. }));
    }
}
