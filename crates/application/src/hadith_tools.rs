//! Hadith typed tool backend: implements `HadithBackend` for the hadith reader.
//!
//! # application::hadith_tools
//!
//! Each method returns `ToolResult<serde_json::Value>` with research_checksum,
//! canonical_references, and analysis_sources. Per D-04 (locked): every grading
//! in the tool result is shown with scholar attribution. Per D-05 (locked):
//! computational matches carry confidence and "uncertain" label.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use hadith_core::HadithId;
use tool_registry::{
    BackendMeta, HADITH_SEARCH_VERSION, HadithBackend, HadithRef, HadithSearchToolParams,
    HadithView,
};
use tools::{AnalysisSource, ToolError, ToolResult, reproducibility, research_checksum};

use crate::hadith_reader::{HadithReader, HadithReaderError, HadithReaderService, encode_hash};
use crate::hadith_search::{HadithSearchParams, HadithSearchService, SearchError};

/// Map a hadith reader error to a tool error.
fn reader_error(error: HadithReaderError) -> ToolError {
    use storage::error::Diagnostic;
    ToolError::Backend { code: error.code().to_string(), detail: error.to_string() }
}

/// Map a search error to a tool error.
fn search_error(error: SearchError) -> ToolError {
    use storage::error::Diagnostic;
    ToolError::Backend { code: error.code().to_string(), detail: error.to_string() }
}

/// The hadith tool backend: `HadithReaderService` + `HadithSearchService`.
pub struct HadithToolBackend {
    reader: Arc<HadithReaderService>,
    search: Arc<HadithSearchService>,
}

impl HadithToolBackend {
    /// Wrap a hadith reader and search service.
    pub fn new(reader: Arc<HadithReaderService>, search: Arc<HadithSearchService>) -> Self {
        Self { reader, search }
    }

    /// The underlying reader service.
    pub fn reader(&self) -> &HadithReaderService {
        &self.reader
    }
}

#[async_trait]
impl HadithBackend for HadithToolBackend {
    async fn backend_get_hadith(
        &self,
        reference: &HadithRef,
    ) -> Result<(HadithView, BackendMeta), ToolError> {
        // Look up the collection by slug
        let collections = self.reader.list_collections().await.map_err(reader_error)?;
        let collection = collections
            .into_iter()
            .find(|c| c.slug == reference.collection_slug)
            .ok_or_else(|| ToolError::Backend {
                code: "QAI-HAD-0002".to_string(),
                detail: format!("collection not found: {}", reference.collection_slug),
            })?;

        let record = self
            .reader
            .get_record_by_source_number(&collection.id, &reference.source_number)
            .await
            .map_err(reader_error)?;

        let mut gradings =
            self.reader.get_gradings_for_hadith(&record.id).await.map_err(reader_error)?;
        // Side-by-side display: sort by scholar name, never merge (D-03/D-04).
        gradings.sort_by(|a, b| a.scholar_name.cmp(&b.scholar_name));

        let view = HadithView {
            id: record.id.to_string(),
            collection_slug: collection.slug.clone(),
            source_number: record.source_number.clone(),
            matn_text: record.matn_text.clone(),
            isnad_text: record.isnad_text.clone(),
            narrator_names: record.narrator_names.clone(),
            gradings: gradings
                .iter()
                .map(|g| tool_registry::GradingRecord {
                    id: g.id.to_string(),
                    scholar_name: g.scholar_name.clone(),
                    grading_value: g.grading_value.clone(),
                    source_citation: g.source_citation.clone(),
                    decision: g.decision.as_str().to_string(),
                    reviewer: g.reviewer.clone(),
                    provenance_layer: g.provenance_layer.as_str().to_string(),
                    confidence: g.confidence,
                })
                .collect(),
        };

        let meta = BackendMeta {
            edition_slug: collection.slug.clone(),
            edition_version: collection.version.to_string(),
            edition_id: collection.id.to_string(),
            corpus_generation: 0,
            text_hash: encode_hash(&collection.text_hash),
            script: String::new(),
            riwayah: None,
            numbering_scheme: String::new(),
            projection_id: String::new(),
            builder_version: String::new(),
        };

        Ok((view, meta))
    }

    async fn backend_search_hadith(
        &self,
        params: &HadithSearchToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let started = Instant::now();
        if params.query.trim().is_empty() {
            return Err(ToolError::InvalidInput {
                tool: "hadith.search",
                detail: "query text must not be empty".to_string(),
            });
        }

        let search_params = HadithSearchParams {
            query: params.query.clone(),
            collection_slug: params.collection_slug.clone(),
            limit: params.limit,
        };
        let results = self.search.search(&search_params).await.map_err(search_error)?;

        let canonical_references: Vec<String> =
            results.iter().map(|r| format!("{}:{}", r.collection_slug, r.source_number)).collect();
        let analysis_sources: Vec<AnalysisSource> = canonical_references
            .iter()
            .map(|r| AnalysisSource { kind: "hadith".to_string(), reference: r.clone() })
            .collect();

        let payload = serde_json::to_value(&results).unwrap_or(serde_json::Value::Null);
        let query = serde_json::to_value(params).unwrap_or(serde_json::Value::Null);

        let result = ToolResult {
            tool_name: "hadith.search".to_string(),
            tool_version: HADITH_SEARCH_VERSION,
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
                "hadith.search",
                HADITH_SEARCH_VERSION,
                &query,
                None,
                None,
                BTreeMap::new(),
                0,
            ),
            research_checksum: research_checksum(
                "hadith.search",
                HADITH_SEARCH_VERSION,
                &query,
                &serde_json::json!({}),
                &payload,
            )
            .expect("research checksum input is JSON-serializable"),
        };
        Ok(result)
    }

    async fn backend_get_gradings(
        &self,
        hadith_id: &str,
    ) -> Result<Vec<tool_registry::GradingRecord>, ToolError> {
        let id = hadith_id.parse::<HadithId>().map_err(|_| ToolError::InvalidInput {
            tool: "hadith.gradings",
            detail: format!("invalid hadith id: {hadith_id}"),
        })?;
        let mut gradings = self.reader.get_gradings_for_hadith(&id).await.map_err(reader_error)?;
        gradings.sort_by(|a, b| a.scholar_name.cmp(&b.scholar_name));
        Ok(gradings
            .iter()
            .map(|g| tool_registry::GradingRecord {
                id: g.id.to_string(),
                scholar_name: g.scholar_name.clone(),
                grading_value: g.grading_value.clone(),
                source_citation: g.source_citation.clone(),
                decision: g.decision.as_str().to_string(),
                reviewer: g.reviewer.clone(),
                provenance_layer: g.provenance_layer.as_str().to_string(),
                confidence: g.confidence,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hadith_search::HadithSearchService;
    use domain::{ContentHash, HashAlgorithm, Timestamp};
    use hadith_core::{
        GradingId, HadithCollection, HadithCollectionId, HadithRecord, ProvenanceLayer,
        RecordStatus, ReviewDecision, Tradition,
    };
    use std::sync::Arc;
    use storage_sqlite::SqliteDatabase;

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
            version: domain::SemVer::new(1, 0, 0),
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

    fn sample_grading(hadith_id: &HadithId) -> hadith_core::GradingRecord {
        hadith_core::GradingRecord {
            id: GradingId::new(),
            hadith_id: *hadith_id,
            scholar_name: "Al-Albani".to_string(),
            grading_value: "sahih".to_string(),
            source_citation: "Silsilat al-Ahadith al-Sahihah, hadith 1234".to_string(),
            evidence_json: serde_json::json!({}),
            reviewer: None,
            decision: ReviewDecision::Pending,
            decided_at: None,
            supersedes_id: None,
            provenance_layer: ProvenanceLayer::C,
            confidence: None,
            created_at: Timestamp::from_ymd_hms(2026, 1, 15, 0, 0, 0).unwrap(),
        }
    }

    #[tokio::test]
    async fn hadith_get_returns_view_with_gradings() {
        let db = test_db().await;
        let reader = Arc::new(HadithReaderService::new(db.clone()));
        let search = Arc::new(HadithSearchService::new(db.clone(), reader.clone()));
        let backend = HadithToolBackend::new(reader.clone(), search);

        let collection = sample_collection();
        reader.insert_collection(&collection).await.unwrap();
        let record = sample_record(&collection.id);
        reader.insert_record(&record).await.unwrap();
        let grading = sample_grading(&record.id);
        reader.insert_grading(&grading).await.unwrap();

        let reference = HadithRef {
            collection_slug: "bukhari-arabic".to_string(),
            source_number: "bukhari:1234".to_string(),
        };
        let (view, _meta) = backend.backend_get_hadith(&reference).await.unwrap();
        assert_eq!(view.source_number, "bukhari:1234");
        assert_eq!(view.matn_text, "إنما الأعمال بالنيات");
        assert!(view.isnad_text.is_some());
        assert_eq!(view.gradings.len(), 1);
        assert_eq!(view.gradings[0].scholar_name, "Al-Albani");
    }

    #[tokio::test]
    async fn hadith_search_returns_tool_result_with_checksum() {
        let db = test_db().await;
        let reader = Arc::new(HadithReaderService::new(db.clone()));
        let search = Arc::new(HadithSearchService::new(db.clone(), reader.clone()));
        let backend = HadithToolBackend::new(reader.clone(), search.clone());

        let collection = sample_collection();
        reader.insert_collection(&collection).await.unwrap();
        let record = sample_record(&collection.id);
        reader.insert_record(&record).await.unwrap();
        search.derive_search_forms(&record, &collection, 1).await.unwrap();

        let params = HadithSearchToolParams {
            query: "إنما الأعمال بالنيات".to_string(),
            collection_slug: None,
            limit: None,
        };
        let result = backend.backend_search_hadith(&params).await.unwrap();
        assert_eq!(result.tool_name, "hadith.search");
        assert!(!result.canonical_references.is_empty());
        assert!(!result.analysis_sources.is_empty());
    }

    #[tokio::test]
    async fn hadith_gradings_returns_all_with_attribution() {
        let db = test_db().await;
        let reader = Arc::new(HadithReaderService::new(db.clone()));
        let search = Arc::new(HadithSearchService::new(db.clone(), reader.clone()));
        let backend = HadithToolBackend::new(reader.clone(), search);

        let collection = sample_collection();
        reader.insert_collection(&collection).await.unwrap();
        let record = sample_record(&collection.id);
        reader.insert_record(&record).await.unwrap();
        let grading = sample_grading(&record.id);
        reader.insert_grading(&grading).await.unwrap();

        let gradings = backend.backend_get_gradings(&record.id.to_string()).await.unwrap();
        assert_eq!(gradings.len(), 1);
        assert_eq!(gradings[0].scholar_name, "Al-Albani");
        assert_eq!(gradings[0].source_citation, "Silsilat al-Ahadith al-Sahihah, hadith 1234");
    }

    #[tokio::test]
    async fn hadith_search_empty_query_is_invalid_input() {
        let db = test_db().await;
        let reader = Arc::new(HadithReaderService::new(db.clone()));
        let search = Arc::new(HadithSearchService::new(db.clone(), reader.clone()));
        let backend = HadithToolBackend::new(reader.clone(), search);

        let params =
            HadithSearchToolParams { query: "   ".to_string(), collection_slug: None, limit: None };
        let err = backend.backend_search_hadith(&params).await.unwrap_err();
        assert!(matches!(err, ToolError::InvalidInput { tool: "hadith.search", .. }));
    }
}
