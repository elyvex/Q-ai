//! Tafsir reader service: insert/retrieve tafsir records and works.
//!
//! # application::tafsir_reader
//!
//! Follows the `HadithReader` pattern: async trait + service wrapping
//! `SqliteDatabase`. All queries use parameterized SQL via `sqlx` — never
//! string interpolation. Canonical tafsir text is immutable (T-06-05/T-06-06).
//! Verse navigation here is structural (the work's own chapter organization);
//! computational verse links with confidence + owner gate arrive in plan 06-03.

use std::sync::Arc;

use async_trait::async_trait;
use domain::{ContentHash, HashAlgorithm, SemVer, SourceVersionId, Timestamp};
use sqlx::Row;
use storage::error::StorageError;
use storage_sqlite::SqliteDatabase;
use tafsir::{TafsirId, TafsirRecord, TafsirWork, TafsirWorkId, WorkStatus};

/// Tafsir reader errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum TafsirReaderError {
    /// Storage failure.
    #[error("storage failed: {0}")]
    Storage(#[from] StorageError),
    /// Work not found.
    #[error("work not found: {0}")]
    WorkNotFound(String),
    /// Record not found.
    #[error("record not found: {0}")]
    RecordNotFound(String),
    /// Invalid status value.
    #[error("invalid status: {0}")]
    InvalidStatus(String),
    /// Invalid hash.
    #[error("invalid hash: {0}")]
    InvalidHash(String),
}

impl storage::error::Diagnostic for TafsirReaderError {
    fn code(&self) -> storage::error::DiagnosticCode {
        match self {
            Self::Storage(_) => storage::error::DiagnosticCode::new("QAI-TAF", 1),
            Self::WorkNotFound(_) => storage::error::DiagnosticCode::new("QAI-TAF", 2),
            Self::RecordNotFound(_) => storage::error::DiagnosticCode::new("QAI-TAF", 3),
            Self::InvalidStatus(_) => storage::error::DiagnosticCode::new("QAI-TAF", 4),
            Self::InvalidHash(_) => storage::error::DiagnosticCode::new("QAI-TAF", 5),
        }
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn remedy(&self) -> Option<String> {
        Some(match self {
            Self::Storage(_) => "Check the database and retry.".to_string(),
            Self::WorkNotFound(_) => "Import the tafsir work first.".to_string(),
            Self::RecordNotFound(_) => "Check the work and dimensions.".to_string(),
            Self::InvalidStatus(_) | Self::InvalidHash(_) => "Fix the input value.".to_string(),
        })
    }

    fn next_command(&self) -> Option<String> {
        Some("qai tafsir --help".to_string())
    }
}

fn parse_work_id(raw: &str) -> Result<TafsirWorkId, TafsirReaderError> {
    raw.parse::<TafsirWorkId>().map_err(|_| TafsirReaderError::InvalidHash(raw.to_string()))
}

fn parse_tafsir_id(raw: &str) -> Result<TafsirId, TafsirReaderError> {
    raw.parse::<TafsirId>().map_err(|_| TafsirReaderError::InvalidHash(raw.to_string()))
}

fn parse_source_version_id(raw: &str) -> Result<SourceVersionId, TafsirReaderError> {
    raw.parse::<SourceVersionId>().map_err(|_| TafsirReaderError::InvalidHash(raw.to_string()))
}

fn parse_status(raw: &str) -> Result<WorkStatus, TafsirReaderError> {
    raw.parse::<WorkStatus>().map_err(|_| TafsirReaderError::InvalidStatus(raw.to_string()))
}

fn parse_hash(raw: &str) -> Result<ContentHash, TafsirReaderError> {
    let (algorithm, hex) =
        raw.split_once(':').ok_or_else(|| TafsirReaderError::InvalidHash(raw.to_string()))?;
    let algorithm = match algorithm {
        "sha256" => HashAlgorithm::Sha256,
        "blake3" => HashAlgorithm::Blake3,
        _ => return Err(TafsirReaderError::InvalidHash(raw.to_string())),
    };
    ContentHash::try_new(algorithm, hex.to_string())
        .map_err(|_| TafsirReaderError::InvalidHash(raw.to_string()))
}

/// Encode a hash for storage as `"{algorithm}:{hex}"` — the exact inverse of
/// [`parse_hash`]. Both sides live here so they cannot drift apart.
pub(crate) fn encode_hash(hash: &ContentHash) -> String {
    let algorithm = match hash.algorithm {
        HashAlgorithm::Sha256 => "sha256",
        HashAlgorithm::Blake3 => "blake3",
    };
    format!("{algorithm}:{}", hash.hex)
}

fn parse_timestamp(raw: &str) -> Result<Timestamp, TafsirReaderError> {
    raw.parse::<Timestamp>().map_err(|_| TafsirReaderError::InvalidHash(raw.to_string()))
}

fn parse_semver(raw: &str) -> Result<SemVer, TafsirReaderError> {
    raw.parse::<SemVer>().map_err(|_| TafsirReaderError::InvalidHash(raw.to_string()))
}

fn work_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<TafsirWork, TafsirReaderError> {
    Ok(TafsirWork {
        id: parse_work_id(&row.get::<String, _>(0))?,
        slug: row.get(1),
        name: row.get(2),
        author: row.get(3),
        dimensions_schema_json: row.get(4),
        version: parse_semver(&row.get::<String, _>(5))?,
        source_version_id: parse_source_version_id(&row.get::<String, _>(6))?,
        status: parse_status(&row.get::<String, _>(7))?,
        created_at: parse_timestamp(&row.get::<String, _>(8))?,
    })
}

fn record_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<TafsirRecord, TafsirReaderError> {
    let volume: Option<i64> = row.get(2);
    let page: Option<i64> = row.get(3);
    Ok(TafsirRecord {
        id: parse_tafsir_id(&row.get::<String, _>(0))?,
        work_id: parse_work_id(&row.get::<String, _>(1))?,
        volume: volume.map(|v| v as u32),
        page: page.map(|p| p as u32),
        chapter: row.get(4),
        text: row.get(5),
        text_hash: parse_hash(&row.get::<String, _>(6))?,
        source_version_id: parse_source_version_id(&row.get::<String, _>(7))?,
        status: parse_status(&row.get::<String, _>(8))?,
        created_at: parse_timestamp(&row.get::<String, _>(9))?,
    })
}

const WORK_COLUMNS: &str = "id, slug, name, author, dimensions_schema_json, version, \
                             source_version_id, status, created_at";
const RECORD_COLUMNS: &str = "id, work_id, volume, page, chapter, text, text_hash, \
                               source_version_id, status, created_at";

/// The canonical reference for a tafsir record: work slug plus its location
/// inside the work. Chapter-first (the work's own verse organization), else
/// volume/page. Deterministic for identical inputs.
pub fn canonical_reference(
    work_slug: &str,
    volume: Option<u32>,
    page: Option<u32>,
    chapter: Option<&str>,
) -> String {
    let mut reference = work_slug.to_string();
    if let Some(chapter) = chapter {
        reference.push(':');
        reference.push_str(chapter);
    } else {
        if let Some(volume) = volume {
            reference.push_str(&format!(":{volume}"));
        }
        if let Some(page) = page {
            reference.push_str(&format!(":{page}"));
        }
    }
    reference
}

/// Tafsir reader trait: insert/retrieve tafsir works and records.
#[async_trait]
pub trait TafsirReader: Send + Sync {
    /// Insert a tafsir work.
    async fn insert_work(&self, work: &TafsirWork) -> Result<(), TafsirReaderError>;
    /// Get a tafsir work by id.
    async fn get_work(&self, id: &TafsirWorkId) -> Result<TafsirWork, TafsirReaderError>;
    /// Get a tafsir work by slug.
    async fn get_work_by_slug(&self, slug: &str) -> Result<TafsirWork, TafsirReaderError>;
    /// Insert a tafsir record.
    async fn insert_record(&self, record: &TafsirRecord) -> Result<(), TafsirReaderError>;
    /// Get a tafsir record by id.
    async fn get_record(&self, id: &TafsirId) -> Result<TafsirRecord, TafsirReaderError>;
    /// List all records for a work, ordered by (volume, page, created_at).
    async fn list_records_for_work(
        &self,
        work_id: &TafsirWorkId,
    ) -> Result<Vec<TafsirRecord>, TafsirReaderError>;
    /// Get tafsir records for a verse via the works' own chapter organization
    /// (structural navigation; computational links arrive in plan 06-03).
    async fn get_records_for_verse(
        &self,
        surah: u16,
        ayah: u32,
    ) -> Result<Vec<TafsirRecord>, TafsirReaderError>;
}

/// The tafsir reader service.
pub struct TafsirReaderService {
    db: Arc<SqliteDatabase>,
}

impl TafsirReaderService {
    /// Wrap a database handle.
    pub fn new(db: Arc<SqliteDatabase>) -> Self {
        Self { db }
    }

    /// The underlying database handle.
    pub fn database(&self) -> &SqliteDatabase {
        &self.db
    }
}

#[async_trait]
impl TafsirReader for TafsirReaderService {
    async fn insert_work(&self, work: &TafsirWork) -> Result<(), TafsirReaderError> {
        sqlx::query(
            r#"INSERT INTO tafsir_works
               (id, slug, version, name, author, dimensions_schema_json,
                source_version_id, status, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(work.id.to_string())
        .bind(&work.slug)
        .bind(work.version.to_string())
        .bind(&work.name)
        .bind(&work.author)
        .bind(&work.dimensions_schema_json)
        .bind(work.source_version_id.to_string())
        .bind(work.status.as_str())
        .bind(work.created_at.to_string())
        .execute(self.db.write_pool())
        .await
        .map_err(|e| {
            TafsirReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
        })?;
        Ok(())
    }

    async fn get_work(&self, id: &TafsirWorkId) -> Result<TafsirWork, TafsirReaderError> {
        let row = sqlx::query(&format!("SELECT {WORK_COLUMNS} FROM tafsir_works WHERE id = ?"))
            .bind(id.to_string())
            .fetch_optional(self.db.read_pool())
            .await
            .map_err(|e| {
                TafsirReaderError::Storage(StorageError::ConstraintViolation {
                    message: e.to_string(),
                })
            })?
            .ok_or_else(|| TafsirReaderError::WorkNotFound(id.to_string()))?;
        work_from_row(&row)
    }

    async fn get_work_by_slug(&self, slug: &str) -> Result<TafsirWork, TafsirReaderError> {
        let row = sqlx::query(&format!("SELECT {WORK_COLUMNS} FROM tafsir_works WHERE slug = ?"))
            .bind(slug)
            .fetch_optional(self.db.read_pool())
            .await
            .map_err(|e| {
                TafsirReaderError::Storage(StorageError::ConstraintViolation {
                    message: e.to_string(),
                })
            })?
            .ok_or_else(|| TafsirReaderError::WorkNotFound(slug.to_string()))?;
        work_from_row(&row)
    }

    async fn insert_record(&self, record: &TafsirRecord) -> Result<(), TafsirReaderError> {
        sqlx::query(
            r#"INSERT INTO tafsir_records
               (id, work_id, volume, page, chapter, text, text_hash,
                source_version_id, status, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(record.id.to_string())
        .bind(record.work_id.to_string())
        .bind(record.volume.map(|v| v as i64))
        .bind(record.page.map(|p| p as i64))
        .bind(&record.chapter)
        .bind(&record.text)
        .bind(encode_hash(&record.text_hash))
        .bind(record.source_version_id.to_string())
        .bind(record.status.as_str())
        .bind(record.created_at.to_string())
        .execute(self.db.write_pool())
        .await
        .map_err(|e| {
            TafsirReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
        })?;
        Ok(())
    }

    async fn get_record(&self, id: &TafsirId) -> Result<TafsirRecord, TafsirReaderError> {
        let row = sqlx::query(&format!("SELECT {RECORD_COLUMNS} FROM tafsir_records WHERE id = ?"))
            .bind(id.to_string())
            .fetch_optional(self.db.read_pool())
            .await
            .map_err(|e| {
                TafsirReaderError::Storage(StorageError::ConstraintViolation {
                    message: e.to_string(),
                })
            })?
            .ok_or_else(|| TafsirReaderError::RecordNotFound(id.to_string()))?;
        record_from_row(&row)
    }

    async fn list_records_for_work(
        &self,
        work_id: &TafsirWorkId,
    ) -> Result<Vec<TafsirRecord>, TafsirReaderError> {
        let rows = sqlx::query(&format!(
            "SELECT {RECORD_COLUMNS} FROM tafsir_records WHERE work_id = ? \
             ORDER BY volume ASC, page ASC, created_at ASC"
        ))
        .bind(work_id.to_string())
        .fetch_all(self.db.read_pool())
        .await
        .map_err(|e| {
            TafsirReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
        })?;
        rows.iter().map(record_from_row).collect()
    }

    async fn get_records_for_verse(
        &self,
        surah: u16,
        _ayah: u32,
    ) -> Result<Vec<TafsirRecord>, TafsirReaderError> {
        // Structural navigation only: the work's own chapter organization is
        // surah-level, so the ayah refines nothing yet. Ayah-scoped
        // computational links (confidence + owner gate) arrive in plan 06-03.
        let rows = sqlx::query(&format!(
            "SELECT {RECORD_COLUMNS} FROM tafsir_records WHERE chapter = ? ORDER BY created_at ASC"
        ))
        .bind(surah.to_string())
        .fetch_all(self.db.read_pool())
        .await
        .map_err(|e| {
            TafsirReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
        })?;
        rows.iter().map(record_from_row).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn test_db() -> Arc<SqliteDatabase> {
        // The TempDir must outlive the pool: dropping it deletes the database
        // file while open connections still hold it (SQLITE_IOERR).
        let dir = Box::leak(Box::new(tempdir().unwrap()));
        let db = SqliteDatabase::new(&dir.path().join("test.db").display().to_string(), 1, false)
            .await
            .unwrap();
        // Run migrations
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
    async fn insert_and_retrieve_work() {
        let db = test_db().await;
        let reader = TafsirReaderService::new(db);
        let work = sample_work();
        reader.insert_work(&work).await.unwrap();
        let fetched = reader.get_work(&work.id).await.unwrap();
        assert_eq!(fetched.id, work.id);
        assert_eq!(fetched.slug, work.slug);
        assert_eq!(fetched.author, "Muhammad ibn Jarir al-Tabari");
        let by_slug = reader.get_work_by_slug("tafsir-tabari").await.unwrap();
        assert_eq!(by_slug.id, work.id);
    }

    #[tokio::test]
    async fn insert_and_retrieve_record() {
        let db = test_db().await;
        let reader = TafsirReaderService::new(db);
        let work = sample_work();
        reader.insert_work(&work).await.unwrap();
        let record = sample_record(&work.id);
        reader.insert_record(&record).await.unwrap();
        let fetched = reader.get_record(&record.id).await.unwrap();
        assert_eq!(fetched.id, record.id);
        assert_eq!(fetched.volume, Some(3));
        assert_eq!(fetched.page, Some(12));
        assert_eq!(fetched.chapter.as_deref(), Some("2"));
        assert_eq!(fetched.text, "القول في تأويل قوله تعالى");
    }

    #[tokio::test]
    async fn records_for_verse_uses_chapter_organization() {
        let db = test_db().await;
        let reader = TafsirReaderService::new(db);
        let work = sample_work();
        reader.insert_work(&work).await.unwrap();
        let record = sample_record(&work.id);
        reader.insert_record(&record).await.unwrap();
        let linked = reader.get_records_for_verse(2, 255).await.unwrap();
        assert_eq!(linked.len(), 1);
        assert_eq!(linked[0].id, record.id);
        let other = reader.get_records_for_verse(3, 1).await.unwrap();
        assert!(other.is_empty());
    }

    #[tokio::test]
    async fn record_not_found_error() {
        let db = test_db().await;
        let reader = TafsirReaderService::new(db);
        let result = reader.get_record(&TafsirId::new()).await;
        assert!(matches!(result, Err(TafsirReaderError::RecordNotFound(_))));
    }
}
