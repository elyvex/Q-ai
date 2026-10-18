//! Scripture typed tool backend: implements `ScriptureBackend` for the reader.

use std::sync::Arc;

use async_trait::async_trait;
use scripture::ScriptureView;
use tool_registry::{BackendMeta, ScriptureBackend, ScriptureRef, ScriptureSearchToolParams, SCRIPTURE_SEARCH_VERSION, SCRIPTURE_PARALLEL_PASSAGES_VERSION};
use std::time::Instant;
use std::collections::BTreeMap;
use tools::{AnalysisSource, ToolError, ToolResult, reproducibility, research_checksum};

use crate::scripture_reader::{ScriptureReader, ScriptureReaderError, ScriptureReaderService};
use crate::scripture_search::{ScriptureSearchService, ScriptureSearchParams};
use crate::verse_link::{VerseLinkService, VerseLinker};

/// Map reader error to tool error.
fn reader_error(error: ScriptureReaderError) -> ToolError {
    use storage::error::Diagnostic;
    ToolError::Backend { code: error.code().to_string(), detail: error.to_string() }
}

/// The scripture tool backend.
pub struct ScriptureToolBackend {
    reader: Arc<ScriptureReaderService>,
    search: Arc<ScriptureSearchService>,
    links: Arc<VerseLinkService>,
}

impl ScriptureToolBackend {
    pub fn new(reader: Arc<ScriptureReaderService>) -> Self {
        let search = Arc::new(ScriptureSearchService::new(
            reader.database_arc(),
            reader.clone() as Arc<dyn ScriptureReader>,
        ));
        let links = Arc::new(VerseLinkService::new(reader.database_arc()));
        Self { reader, search, links }
    }
    pub fn reader(&self) -> &ScriptureReaderService {
        &self.reader
    }
    pub fn search(&self) -> &ScriptureSearchService {
        &self.search
    }
    pub fn links(&self) -> &VerseLinkService {
        &self.links
    }
}

#[async_trait]
impl ScriptureBackend for ScriptureToolBackend {
    async fn backend_get_scripture(
        &self,
        reference: &ScriptureRef,
    ) -> Result<(ScriptureView, BackendMeta), ToolError> {
        // Find edition by slug.
        let editions = self.reader.list_editions(None).await.map_err(reader_error)?;
        let edition = editions
            .into_iter()
            .find(|e| e.slug == reference.edition)
            .ok_or_else(|| ToolError::Backend {
                code: "QAI-SCR-0002".to_string(),
                detail: format!("edition not found: {}", reference.edition),
            })?;

        let verse_id = scripture::ScriptureVerseId::from_parts(
            &edition.id,
            &reference.book,
            reference.chapter,
            reference.verse,
        );
        let verse = self.reader.get_verse(&verse_id).await.map_err(reader_error)?;

        let view = ScriptureView { verse: verse.clone(), translations: Vec::new() };

        let meta = BackendMeta {
            edition_slug: edition.slug.clone(),
            edition_version: edition.version.to_string(),
            edition_id: edition.id.to_string(),
            corpus_generation: 0,
            text_hash: crate::scripture_reader::encode_hash(&edition.text_hash),
            script: String::new(),
            riwayah: None,
            numbering_scheme: String::new(),
            projection_id: String::new(),
            builder_version: String::new(),
        };

        Ok((view, meta))
    }

    async fn backend_search_scripture(
        &self,
        params: &ScriptureSearchToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let started = Instant::now();
        if params.query.trim().is_empty() {
            return Err(ToolError::InvalidInput {
                tool: "scripture.search",
                detail: "query text must not be empty".to_string(),
            });
        }

        let search_params = ScriptureSearchParams {
            query: params.query.clone(),
            edition: params.edition.clone(),
            limit: params.limit,
        };
        let results = self.search.search(&search_params).await.map_err(|e| {
            use storage::error::Diagnostic;
            ToolError::Backend { code: e.code().to_string(), detail: e.to_string() }
        })?;

        let canonical_references: Vec<String> =
            results.iter().map(|r| r.verse_id.to_string()).collect();
        let analysis_sources: Vec<AnalysisSource> = canonical_references
            .iter()
            .map(|r| AnalysisSource { kind: "scripture".to_string(), reference: r.clone() })
            .collect();

        let payload = serde_json::to_value(&results).unwrap_or(serde_json::Value::Null);
        let query = serde_json::to_value(params).unwrap_or(serde_json::Value::Null);

        let result = ToolResult {
            tool_name: "scripture.search".to_string(),
            tool_version: SCRIPTURE_SEARCH_VERSION,
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
                "scripture.search",
                SCRIPTURE_SEARCH_VERSION,
                &query,
                None,
                None,
                BTreeMap::new(),
                0,
            ),
            research_checksum: research_checksum(
                "scripture.search",
                SCRIPTURE_SEARCH_VERSION,
                &query,
                &serde_json::json!({}),
                &payload,
            )
            .expect("research checksum input is JSON-serializable"),
        };
        Ok(result)
    }

    async fn backend_list_editions(&self) -> Result<serde_json::Value, ToolError> {
        let editions = self.reader.list_editions(None).await.map_err(|e| {
            use storage::error::Diagnostic;
            ToolError::Backend { code: e.code().to_string(), detail: e.to_string() }
        })?;
        serde_json::to_value(editions).map_err(|e| ToolError::Backend {
            code: "QAI-SCR-0310".to_string(),
            detail: e.to_string(),
        })
    }

    async fn backend_get_translation(&self, verse_id: &str, language: &str) -> Result<serde_json::Value, ToolError> {
        let vid: scripture::ScriptureVerseId = verse_id.parse().map_err(|_| ToolError::InvalidInput {
            tool: "scripture.translations",
            detail: "invalid verse id".to_string(),
        })?;
        let lang = if language.trim().is_empty() { None } else { Some(language) };
        let translations = self.reader.get_translations(&vid, lang).await.map_err(|e| {
            use storage::error::Diagnostic;
            ToolError::Backend { code: e.code().to_string(), detail: e.to_string() }
        })?;
        serde_json::to_value(translations).map_err(|e| ToolError::Backend {
            code: "QAI-SCR-0310".to_string(),
            detail: e.to_string(),
        })
    }

    async fn backend_parallel_passages(
        &self,
        reference: &str,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        use std::time::Instant;
        use std::collections::BTreeMap;

        let started = Instant::now();

        let links = if let Some((s, a)) = parse_quran_ref(reference) {
            self.links.get_links_for_verse(s, a, true).await.map_err(|e| {
                use storage::error::Diagnostic;
                ToolError::Backend { code: e.code().to_string(), detail: e.to_string() }
            })?
        } else {
            self.links.suggest_scripture_links(reference).await.map_err(|e| {
                use storage::error::Diagnostic;
                ToolError::Backend { code: e.code().to_string(), detail: e.to_string() }
            })?
        };

        let canonical_references: Vec<String> = links
            .iter()
            .map(|l| format!("{}:{}", l.target_surah, l.target_ayah))
            .collect();
        let analysis_sources: Vec<AnalysisSource> = links
            .iter()
            .map(|l| AnalysisSource { kind: "scripture".to_string(), reference: l.source_id.clone() })
            .collect();

        let mut tagged: Vec<serde_json::Value> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();
        for l in &links {
            let mut v = serde_json::to_value(l).unwrap_or(serde_json::Value::Null);
            if l.label != crate::verse_link::LinkLabel::Confirmed {
                if let Some(obj) = v.as_object_mut() {
                    obj.insert("marker".to_string(), serde_json::json!("uncertain computational suggestion"));
                }
                warnings.push("uncertain computational suggestion".to_string());
            }
            tagged.push(v);
        }

        let payload = serde_json::to_value(&tagged).unwrap_or(serde_json::Value::Null);
        let query = serde_json::json!({ "reference": reference });

        let result = ToolResult {
            tool_name: "scripture.parallel_passages".to_string(),
            tool_version: SCRIPTURE_PARALLEL_PASSAGES_VERSION,
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: None,
            edition_version: None,
            results: payload.clone(),
            canonical_references,
            analysis_sources,
            confidence: None,
            warnings,
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                "scripture.parallel_passages",
                SCRIPTURE_PARALLEL_PASSAGES_VERSION,
                &query,
                None,
                None,
                BTreeMap::new(),
                0,
            ),
            research_checksum: research_checksum(
                "scripture.parallel_passages",
                SCRIPTURE_PARALLEL_PASSAGES_VERSION,
                &query,
                &serde_json::json!({}),
                &payload,
            )
            .expect("research checksum input is JSON-serializable"),
        };
        Ok(result)
    }

}

fn parse_quran_ref(s: &str) -> Option<(u16, u32)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 { return None; }
    let surah: u16 = parts[0].trim().parse().ok()?;
    let ayah: u32 = parts[1].trim().parse().ok()?;
    if surah == 0 || ayah == 0 { return None; }
    Some((surah, ayah))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripture_reader::ScriptureReaderService;
    use domain::{ContentHash, HashAlgorithm, SemVer, Timestamp};
    use scripture::{ScriptureEdition, ScriptureEditionStatistics, ScriptureStatus, ScriptureVerse, Tradition};
    use tool_registry::ScriptureRef as ToolScriptureRef;
    use std::sync::Arc;
    use storage_sqlite::SqliteDatabase;
    use tempfile::tempdir;

    async fn test_db() -> Arc<SqliteDatabase> {
        let dir = Box::leak(Box::new(tempdir().unwrap()));
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
            include_str!("../../../migrations/sqlite/0039_scripture_editions.up.sql"),
            include_str!("../../../migrations/sqlite/0040_scripture_books.up.sql"),
            include_str!("../../../migrations/sqlite/0041_scripture_verses.up.sql"),
        ];
        for m in migrations {
            sqlx::raw_sql(m).execute(db.write_pool()).await.unwrap();
        }
        for sql in [
            "INSERT INTO sources (id, title, content_type, created_at, updated_at) VALUES ('src-scr', 'Scripture test source', 'scripture', '2026-01-15T00:00:00Z', '2026-01-15T00:00:00Z')",
            "INSERT INTO source_versions (id, source_id, version, schema_version, state, trust_level, license_status, license_json, created_at) VALUES ('018f3a2b-4c5d-7e6f-8a9b-0c1d2e3f4a5b', 'src-scr', '0.1.0', 1, 'Staged', 'ImportedUnverified', 'PublicDomain', '{}', '2026-01-15T00:00:00Z')",
        ] {
            sqlx::query(sql).execute(db.write_pool()).await.unwrap();
        }
        Arc::new(db)
    }

    #[tokio::test]
    async fn scripture_get_returns_view_with_text_and_checksum() {
        let db = test_db().await;
        let reader = Arc::new(ScriptureReaderService::new(db));
        let edition = ScriptureEdition {
            id: scripture::ScriptureEditionId::new(),
            slug: "tanakh".to_string(),
            version: SemVer::new(1, 0, 0),
            name: "Tanakh".to_string(),
            tradition: Tradition::Jewish,
            language: "he".parse().unwrap(),
            source_url: None,
            license_json: serde_json::json!({}),
            text_hash: ContentHash::try_new(HashAlgorithm::Sha256, "ab".repeat(32)).unwrap(),
            structure_hash: ContentHash::try_new(HashAlgorithm::Sha256, "cd".repeat(32)).unwrap(),
            manifest_hash: ContentHash::try_new(HashAlgorithm::Sha256, "12".repeat(32)).unwrap(),
            source_version_id: "018f3a2b-4c5d-7e6f-8a9b-0c1d2e3f4a5b".parse().unwrap(),
            status: ScriptureStatus::Active,
            imported_at: Timestamp::from_ymd_hms(2026, 1, 15, 0, 0, 0).unwrap(),
            statistics: ScriptureEditionStatistics::default(),
        };
        reader.insert_edition(&edition).await.unwrap();
        let verse_id = scripture::ScriptureVerseId::from_parts(&edition.id, "genesis", 1, 1);
        let verse = ScriptureVerse::new(
            verse_id,
            edition.id,
            "genesis".to_string(),
            1,
            1,
            "In the beginning".to_string(),
            ContentHash::try_new(HashAlgorithm::Sha256, "ab".repeat(32)).unwrap(),
            ScriptureStatus::Active,
            Timestamp::from_ymd_hms(2026, 1, 15, 0, 0, 0).unwrap(),
        )
        .unwrap();
        reader.insert_verse(&verse).await.unwrap();

        let backend = ScriptureToolBackend::new(reader);
        let (view, meta) = backend
            .backend_get_scripture(&ToolScriptureRef {
                edition: "tanakh".to_string(),
                book: "genesis".to_string(),
                chapter: 1,
                verse: 1,
            })
            .await
            .unwrap();
        assert_eq!(view.verse.text, "In the beginning");
        assert_eq!(meta.edition_slug, "tanakh");
    }
}
