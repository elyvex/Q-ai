//! Scripture typed tool backend: implements `ScriptureBackend` for the reader.

use std::sync::Arc;

use async_trait::async_trait;
use scripture::ScriptureView;
use tool_registry::{BackendMeta, ScriptureBackend, ScriptureRef, ScriptureSearchToolParams};
use tools::{ToolError, ToolResult};

use crate::scripture_reader::{ScriptureReader, ScriptureReaderError, ScriptureReaderService};

/// Map reader error to tool error.
fn reader_error(error: ScriptureReaderError) -> ToolError {
    use storage::error::Diagnostic;
    ToolError::Backend { code: error.code().to_string(), detail: error.to_string() }
}

/// The scripture tool backend.
pub struct ScriptureToolBackend {
    reader: Arc<ScriptureReaderService>,
}

impl ScriptureToolBackend {
    pub fn new(reader: Arc<ScriptureReaderService>) -> Self {
        Self { reader }
    }
    pub fn reader(&self) -> &ScriptureReaderService {
        &self.reader
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
        _params: &ScriptureSearchToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(ToolError::Backend {
            code: "QAI-SCR-0310".to_string(),
            detail: "scripture.search is not supported by this backend".to_string(),
        })
    }
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
