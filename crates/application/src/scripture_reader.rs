//! Scripture reader service: insert/retrieve editions, books, and verses.

use std::sync::Arc;

use async_trait::async_trait;
use domain::{ContentHash, HashAlgorithm, SemVer, SourceVersionId, Timestamp};
use scripture::{
    ScriptureBook, ScriptureBookId, ScriptureEdition, ScriptureEditionId,
    ScriptureEditionStatistics, ScriptureStatus, ScriptureTranslation, ScriptureVerse, ScriptureVerseId, Tradition,
};
use sqlx::Row;
use storage::error::StorageError;
use storage_sqlite::SqliteDatabase;

/// Scripture reader errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ScriptureReaderError {
    #[error("storage failed: {0}")]
    Storage(#[from] StorageError),
    #[error("edition not found: {0}")]
    EditionNotFound(String),
    #[error("book not found: {0}")]
    BookNotFound(String),
    #[error("verse not found: {0}")]
    VerseNotFound(String),
    #[error("invalid status: {0}")]
    InvalidStatus(String),
    #[error("invalid hash: {0}")]
    InvalidHash(String),
    #[error("invalid tradition: {0}")]
    InvalidTradition(String),
}

impl storage::error::Diagnostic for ScriptureReaderError {
    fn code(&self) -> storage::error::DiagnosticCode {
        match self {
            Self::Storage(_) => storage::error::DiagnosticCode::new("QAI-SCR", 1),
            Self::EditionNotFound(_) => storage::error::DiagnosticCode::new("QAI-SCR", 2),
            Self::BookNotFound(_) => storage::error::DiagnosticCode::new("QAI-SCR", 3),
            Self::VerseNotFound(_) => storage::error::DiagnosticCode::new("QAI-SCR", 4),
            Self::InvalidStatus(_) => storage::error::DiagnosticCode::new("QAI-SCR", 5),
            Self::InvalidHash(_) => storage::error::DiagnosticCode::new("QAI-SCR", 6),
            Self::InvalidTradition(_) => storage::error::DiagnosticCode::new("QAI-SCR", 7),
        }
    }
    fn summary(&self) -> String {
        self.to_string()
    }
    fn remedy(&self) -> Option<String> {
        Some("Check the database and retry.".to_string())
    }
    fn next_command(&self) -> Option<String> {
        Some("qai scripture --help".to_string())
    }
}

fn parse_edition_id(raw: &str) -> Result<ScriptureEditionId, ScriptureReaderError> {
    raw.parse::<ScriptureEditionId>().map_err(|_| ScriptureReaderError::InvalidHash(raw.to_string()))
}

fn parse_book_id(raw: &str) -> Result<ScriptureBookId, ScriptureReaderError> {
    raw.parse::<ScriptureBookId>().map_err(|_| ScriptureReaderError::InvalidHash(raw.to_string()))
}

fn parse_status(raw: &str) -> Result<ScriptureStatus, ScriptureReaderError> {
    raw.parse::<ScriptureStatus>().map_err(|_| ScriptureReaderError::InvalidStatus(raw.to_string()))
}

fn parse_tradition(raw: &str) -> Result<Tradition, ScriptureReaderError> {
    raw.parse::<Tradition>().map_err(|_| ScriptureReaderError::InvalidTradition(raw.to_string()))
}

fn parse_hash(raw: &str) -> Result<ContentHash, ScriptureReaderError> {
    let (algorithm, hex) = raw.split_once(':').ok_or_else(|| ScriptureReaderError::InvalidHash(raw.to_string()))?;
    let algorithm = match algorithm {
        "sha256" => HashAlgorithm::Sha256,
        "blake3" => HashAlgorithm::Blake3,
        _ => return Err(ScriptureReaderError::InvalidHash(raw.to_string())),
    };
    ContentHash::try_new(algorithm, hex.to_string()).map_err(|_| ScriptureReaderError::InvalidHash(raw.to_string()))
}

pub(crate) fn encode_hash(hash: &ContentHash) -> String {
    let algorithm = match hash.algorithm {
        HashAlgorithm::Sha256 => "sha256",
        HashAlgorithm::Blake3 => "blake3",
    };
    format!("{algorithm}:{}", hash.hex)
}

fn parse_timestamp(raw: &str) -> Result<Timestamp, ScriptureReaderError> {
    raw.parse::<Timestamp>().map_err(|_| ScriptureReaderError::InvalidHash(raw.to_string()))
}

fn parse_semver(raw: &str) -> Result<SemVer, ScriptureReaderError> {
    raw.parse::<SemVer>().map_err(|_| ScriptureReaderError::InvalidHash(raw.to_string()))
}

fn parse_language(raw: &str) -> Result<domain::Language, ScriptureReaderError> {
    raw.parse::<domain::Language>().map_err(|_| ScriptureReaderError::InvalidHash(raw.to_string()))
}

fn edition_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<ScriptureEdition, ScriptureReaderError> {
    Ok(ScriptureEdition {
        id: parse_edition_id(&row.get::<String, _>(0))?,
        slug: row.get(1),
        version: parse_semver(&row.get::<String, _>(2))?,
        name: row.get(3),
        tradition: parse_tradition(&row.get::<String, _>(4))?,
        language: parse_language(&row.get::<String, _>(5))?,
        source_url: row.get(6),
        license_json: serde_json::from_str(&row.get::<String, _>(7)).unwrap_or(serde_json::json!({})),
        text_hash: parse_hash(&row.get::<String, _>(8))?,
        structure_hash: parse_hash(&row.get::<String, _>(9))?,
        manifest_hash: parse_hash(&row.get::<String, _>(10))?,
        source_version_id: row.get::<String, _>(11).parse::<SourceVersionId>().map_err(|_| ScriptureReaderError::InvalidHash(row.get::<String, _>(11)))?,
        status: parse_status(&row.get::<String, _>(12))?,
        imported_at: parse_timestamp(&row.get::<String, _>(13))?,
        statistics: ScriptureEditionStatistics::default(),
    })
}

fn book_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<ScriptureBook, ScriptureReaderError> {
    Ok(ScriptureBook {
        id: parse_book_id(&row.get::<String, _>(0))?,
        edition_id: parse_edition_id(&row.get::<String, _>(1))?,
        slug: row.get(2),
        name: row.get(3),
        book_order: row.get(4),
        chapter_count: row.get(5),
    })
}

fn verse_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<ScriptureVerse, ScriptureReaderError> {
    Ok(ScriptureVerse {
        id: row.get::<String, _>(0).parse::<ScriptureVerseId>().map_err(|_| ScriptureReaderError::InvalidHash(row.get::<String, _>(0)))?,
        edition_id: parse_edition_id(&row.get::<String, _>(1))?,
        book: row.get(2),
        chapter: row.get::<i64, _>(3) as u32,
        verse: row.get::<i64, _>(4) as u32,
        text: row.get(5),
        text_hash: parse_hash(&row.get::<String, _>(6))?,
        status: parse_status(&row.get::<String, _>(7))?,
        created_at: parse_timestamp(&row.get::<String, _>(8))?,
    })
}


fn translation_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<ScriptureTranslation, ScriptureReaderError> {
    Ok(ScriptureTranslation {
        id: row.get(0),
        verse_id: row.get::<String, _>(1).parse::<ScriptureVerseId>().map_err(|_| ScriptureReaderError::InvalidHash(row.get::<String, _>(1)))?,
        language: parse_language(&row.get::<String, _>(2))?,
        translator: row.get(3),
        text: row.get(4),
        text_hash: parse_hash(&row.get::<String, _>(5))?,
        status: parse_status(&row.get::<String, _>(6))?,
        created_at: parse_timestamp(&row.get::<String, _>(7))?,
    })
}

#[async_trait]
pub trait ScriptureReader: Send + Sync {
    async fn insert_edition(&self, edition: &ScriptureEdition) -> Result<(), ScriptureReaderError>;
    async fn get_edition(&self, id: &ScriptureEditionId) -> Result<ScriptureEdition, ScriptureReaderError>;
    async fn get_edition_by_slug(&self, slug: &str, version: &str) -> Result<ScriptureEdition, ScriptureReaderError>;
    async fn insert_book(&self, book: &ScriptureBook) -> Result<(), ScriptureReaderError>;
    async fn get_books(&self, edition_id: &ScriptureEditionId) -> Result<Vec<ScriptureBook>, ScriptureReaderError>;
    async fn insert_verse(&self, verse: &ScriptureVerse) -> Result<(), ScriptureReaderError>;
    async fn get_verse(&self, id: &ScriptureVerseId) -> Result<ScriptureVerse, ScriptureReaderError>;
    async fn insert_translation(&self, translation: &ScriptureTranslation) -> Result<(), ScriptureReaderError>;
    async fn get_translations(&self, verse_id: &ScriptureVerseId, language: Option<&str>) -> Result<Vec<ScriptureTranslation>, ScriptureReaderError>;
    async fn list_editions(&self, status: Option<ScriptureStatus>) -> Result<Vec<ScriptureEdition>, ScriptureReaderError>;
}

pub struct ScriptureReaderService {
    db: Arc<SqliteDatabase>,
}

impl ScriptureReaderService {
    pub fn new(db: Arc<SqliteDatabase>) -> Self {
        Self { db }
    }
    pub fn database(&self) -> &SqliteDatabase {
        &self.db
    }
    pub fn database_arc(&self) -> Arc<SqliteDatabase> {
        self.db.clone()
    }
}

#[async_trait]
impl ScriptureReader for ScriptureReaderService {
    async fn insert_edition(&self, edition: &ScriptureEdition) -> Result<(), ScriptureReaderError> {
        sqlx::query(
            r#"INSERT INTO scripture_editions
               (id, slug, version, name, tradition, language, source_url, license_json,
                text_hash, structure_hash, manifest_hash, source_version_id, status, imported_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(edition.id.to_string())
        .bind(&edition.slug)
        .bind(edition.version.to_string())
        .bind(&edition.name)
        .bind(edition.tradition.as_str())
        .bind(edition.language.to_string())
        .bind(&edition.source_url)
        .bind(serde_json::to_string(&edition.license_json).unwrap_or_else(|_| "{}".to_string()))
        .bind(encode_hash(&edition.text_hash))
        .bind(encode_hash(&edition.structure_hash))
        .bind(encode_hash(&edition.manifest_hash))
        .bind(edition.source_version_id.to_string())
        .bind(edition.status.as_str())
        .bind(edition.imported_at.to_string())
        .execute(self.db.write_pool())
        .await
        .map_err(|e| ScriptureReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        Ok(())
    }

    async fn get_edition(&self, id: &ScriptureEditionId) -> Result<ScriptureEdition, ScriptureReaderError> {
        let row = sqlx::query(
            "SELECT id, slug, version, name, tradition, language, source_url, license_json, text_hash, structure_hash, manifest_hash, source_version_id, status, imported_at FROM scripture_editions WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(self.db.read_pool())
        .await
        .map_err(|e| ScriptureReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?
        .ok_or_else(|| ScriptureReaderError::EditionNotFound(id.to_string()))?;
        edition_from_row(&row)
    }

    async fn get_edition_by_slug(&self, slug: &str, version: &str) -> Result<ScriptureEdition, ScriptureReaderError> {
        let row = sqlx::query(
            "SELECT id, slug, version, name, tradition, language, source_url, license_json, text_hash, structure_hash, manifest_hash, source_version_id, status, imported_at FROM scripture_editions WHERE slug = ? AND version = ?",
        )
        .bind(slug)
        .bind(version)
        .fetch_optional(self.db.read_pool())
        .await
        .map_err(|e| ScriptureReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?
        .ok_or_else(|| ScriptureReaderError::EditionNotFound(slug.to_string()))?;
        edition_from_row(&row)
    }

    async fn insert_book(&self, book: &ScriptureBook) -> Result<(), ScriptureReaderError> {
        sqlx::query(
            r#"INSERT INTO scripture_books
               (id, edition_id, slug, name, book_order, chapter_count)
               VALUES (?, ?, ?, ?, ?, ?)"#,
        )
        .bind(book.id.to_string())
        .bind(book.edition_id.to_string())
        .bind(&book.slug)
        .bind(&book.name)
        .bind(book.book_order)
        .bind(book.chapter_count)
        .execute(self.db.write_pool())
        .await
        .map_err(|e| ScriptureReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        Ok(())
    }

    async fn get_books(&self, edition_id: &ScriptureEditionId) -> Result<Vec<ScriptureBook>, ScriptureReaderError> {
        let rows = sqlx::query(
            "SELECT id, edition_id, slug, name, book_order, chapter_count FROM scripture_books WHERE edition_id = ? ORDER BY book_order ASC",
        )
        .bind(edition_id.to_string())
        .fetch_all(self.db.read_pool())
        .await
        .map_err(|e| ScriptureReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        rows.iter().map(book_from_row).collect()
    }

    async fn insert_verse(&self, verse: &ScriptureVerse) -> Result<(), ScriptureReaderError> {
        sqlx::query(
            r#"INSERT INTO scripture_verses
               (id, edition_id, book, chapter, verse, text, text_hash, status, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(verse.id.to_string())
        .bind(verse.edition_id.to_string())
        .bind(&verse.book)
        .bind(verse.chapter as i64)
        .bind(verse.verse as i64)
        .bind(&verse.text)
        .bind(encode_hash(&verse.text_hash))
        .bind(verse.status.as_str())
        .bind(verse.created_at.to_string())
        .execute(self.db.write_pool())
        .await
        .map_err(|e| ScriptureReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        Ok(())
    }

    async fn get_verse(&self, id: &ScriptureVerseId) -> Result<ScriptureVerse, ScriptureReaderError> {
        let row = sqlx::query(
            "SELECT id, edition_id, book, chapter, verse, text, text_hash, status, created_at FROM scripture_verses WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(self.db.read_pool())
        .await
        .map_err(|e| ScriptureReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?
        .ok_or_else(|| ScriptureReaderError::VerseNotFound(id.to_string()))?;
        verse_from_row(&row)
    }

    async fn insert_translation(&self, translation: &ScriptureTranslation) -> Result<(), ScriptureReaderError> {
        sqlx::query(
            r#"INSERT INTO scripture_translations
               (id, verse_id, language, translator, text, text_hash, status, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(&translation.id)
        .bind(translation.verse_id.to_string())
        .bind(translation.language.to_string())
        .bind(&translation.translator)
        .bind(&translation.text)
        .bind(encode_hash(&translation.text_hash))
        .bind(translation.status.as_str())
        .bind(translation.created_at.to_string())
        .execute(self.db.write_pool())
        .await
        .map_err(|e| ScriptureReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        Ok(())
    }

    async fn get_translations(&self, verse_id: &ScriptureVerseId, language: Option<&str>) -> Result<Vec<ScriptureTranslation>, ScriptureReaderError> {
        let rows = if let Some(lang) = language {
            sqlx::query(
                "SELECT id, verse_id, language, translator, text, text_hash, status, created_at FROM scripture_translations WHERE verse_id = ? AND language = ?",
            )
            .bind(verse_id.to_string())
            .bind(lang)
            .fetch_all(self.db.read_pool())
            .await
            .map_err(|e| ScriptureReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?
        } else {
            sqlx::query(
                "SELECT id, verse_id, language, translator, text, text_hash, status, created_at FROM scripture_translations WHERE verse_id = ?",
            )
            .bind(verse_id.to_string())
            .fetch_all(self.db.read_pool())
            .await
            .map_err(|e| ScriptureReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?
        };
        let mut out = Vec::with_capacity(rows.len());
        for row in &rows {
            out.push(translation_from_row(row)?);
        }
        Ok(out)
    }

    async fn list_editions(&self, status: Option<ScriptureStatus>) -> Result<Vec<ScriptureEdition>, ScriptureReaderError> {
        let rows = if let Some(s) = status {
            sqlx::query(
                "SELECT id, slug, version, name, tradition, language, source_url, license_json, text_hash, structure_hash, manifest_hash, source_version_id, status, imported_at FROM scripture_editions WHERE status = ?",
            )
            .bind(s.as_str())
            .fetch_all(self.db.read_pool())
            .await
            .map_err(|e| ScriptureReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?
        } else {
            sqlx::query(
                "SELECT id, slug, version, name, tradition, language, source_url, license_json, text_hash, structure_hash, manifest_hash, source_version_id, status, imported_at FROM scripture_editions",
            )
            .fetch_all(self.db.read_pool())
            .await
            .map_err(|e| ScriptureReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?
        };
        rows.iter().map(edition_from_row).collect()
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use scripture::{ScriptureBook, ScriptureEdition, ScriptureEditionStatistics, ScriptureStatus, ScriptureVerse, ScriptureVerseId, Tradition};
    use tempfile::tempdir;
    use domain::{ContentHash, HashAlgorithm, SemVer, Timestamp};

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

    fn sample_edition() -> ScriptureEdition {
        ScriptureEdition {
            id: ScriptureEditionId::new(),
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
        }
    }

    #[tokio::test]
    async fn edition_book_verse_roundtrip() {
        let db = test_db().await;
        let reader = ScriptureReaderService::new(db);
        let edition = sample_edition();
        reader.insert_edition(&edition).await.unwrap();
        let fetched = reader.get_edition(&edition.id).await.unwrap();
        assert_eq!(fetched.slug, "tanakh");

        let book = ScriptureBook {
            id: ScriptureBookId::new(),
            edition_id: edition.id,
            slug: "genesis".to_string(),
            name: "Genesis".to_string(),
            book_order: 1,
            chapter_count: 50,
        };
        reader.insert_book(&book).await.unwrap();
        let books = reader.get_books(&edition.id).await.unwrap();
        assert_eq!(books.len(), 1);

        let verse_id = ScriptureVerseId::from_parts(&edition.id, "genesis", 1, 1);
        let verse = ScriptureVerse::new(
            verse_id.clone(),
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
        let got = reader.get_verse(&verse_id).await.unwrap();
        assert_eq!(got.text, "In the beginning");

        let listed = reader.list_editions(Some(ScriptureStatus::Active)).await.unwrap();
        assert_eq!(listed.len(), 1);
    }
}
