//! Hadith reader service: insert/retrieve hadith records, collections, and
//! per-scholar grading records.
//!
//! # application::hadith_reader
//!
//! Follows the `QuranReader` pattern: async trait + service wrapping
//! `SqliteDatabase`. All queries use parameterized SQL via `sqlx` — never
//! string interpolation. Source numbering is preserved verbatim (D-01).
//! Canonical matn text is immutable (D-02). Grading records are per-scholar,
//! never consensus-merged (D-03).

use std::sync::Arc;

use async_trait::async_trait;
use domain::{ContentHash, HashAlgorithm, SemVer, SourceVersionId, Timestamp};
use hadith_core::{
    GradingId, GradingRecord, HadithCollection, HadithCollectionId, HadithId, HadithRecord,
    ProvenanceLayer, RecordStatus, ReviewDecision, Tradition,
};
use sqlx::Row;
use storage::error::StorageError;
use storage_sqlite::SqliteDatabase;

/// Hadith reader errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum HadithReaderError {
    /// Storage failure.
    #[error("storage failed: {0}")]
    Storage(#[from] StorageError),
    /// Collection not found.
    #[error("collection not found: {0}")]
    CollectionNotFound(String),
    /// Record not found.
    #[error("record not found: {0}")]
    RecordNotFound(String),
    /// Grading not found.
    #[error("grading not found: {0}")]
    GradingNotFound(String),
    /// Invalid status value.
    #[error("invalid status: {0}")]
    InvalidStatus(String),
    /// Invalid tradition value.
    #[error("invalid tradition: {0}")]
    InvalidTradition(String),
    /// Invalid decision value.
    #[error("invalid decision: {0}")]
    InvalidDecision(String),
    /// Invalid provenance layer.
    #[error("invalid provenance layer: {0}")]
    InvalidProvenanceLayer(String),
    /// Invalid hash.
    #[error("invalid hash: {0}")]
    InvalidHash(String),
}

impl storage::error::Diagnostic for HadithReaderError {
    fn code(&self) -> storage::error::DiagnosticCode {
        match self {
            Self::Storage(_) => storage::error::DiagnosticCode::new("QAI-HAD", 1),
            Self::CollectionNotFound(_) => storage::error::DiagnosticCode::new("QAI-HAD", 2),
            Self::RecordNotFound(_) => storage::error::DiagnosticCode::new("QAI-HAD", 3),
            Self::GradingNotFound(_) => storage::error::DiagnosticCode::new("QAI-HAD", 4),
            Self::InvalidStatus(_) => storage::error::DiagnosticCode::new("QAI-HAD", 5),
            Self::InvalidTradition(_) => storage::error::DiagnosticCode::new("QAI-HAD", 6),
            Self::InvalidDecision(_) => storage::error::DiagnosticCode::new("QAI-HAD", 7),
            Self::InvalidProvenanceLayer(_) => storage::error::DiagnosticCode::new("QAI-HAD", 8),
            Self::InvalidHash(_) => storage::error::DiagnosticCode::new("QAI-HAD", 9),
        }
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn remedy(&self) -> Option<String> {
        Some(match self {
            Self::Storage(_) => "Check the database and retry.".to_string(),
            Self::CollectionNotFound(_) => "Import the collection first.".to_string(),
            Self::RecordNotFound(_) => "Check the collection and source number.".to_string(),
            Self::GradingNotFound(_) => "Check the hadith id.".to_string(),
            Self::InvalidStatus(_)
            | Self::InvalidTradition(_)
            | Self::InvalidDecision(_)
            | Self::InvalidProvenanceLayer(_)
            | Self::InvalidHash(_) => "Fix the input value.".to_string(),
        })
    }

    fn next_command(&self) -> Option<String> {
        Some("qai hadith --help".to_string())
    }
}

fn parse_collection_id(raw: &str) -> Result<HadithCollectionId, HadithReaderError> {
    raw.parse::<HadithCollectionId>().map_err(|_| HadithReaderError::InvalidHash(raw.to_string()))
}

fn parse_hadith_id(raw: &str) -> Result<HadithId, HadithReaderError> {
    raw.parse::<HadithId>().map_err(|_| HadithReaderError::InvalidHash(raw.to_string()))
}

fn parse_grading_id(raw: &str) -> Result<GradingId, HadithReaderError> {
    raw.parse::<GradingId>().map_err(|_| HadithReaderError::InvalidHash(raw.to_string()))
}

fn parse_source_version_id(raw: &str) -> Result<SourceVersionId, HadithReaderError> {
    raw.parse::<SourceVersionId>().map_err(|_| HadithReaderError::InvalidHash(raw.to_string()))
}

fn parse_record_status(raw: &str) -> Result<RecordStatus, HadithReaderError> {
    raw.parse::<RecordStatus>().map_err(|_| HadithReaderError::InvalidStatus(raw.to_string()))
}

fn parse_tradition(raw: &str) -> Result<Tradition, HadithReaderError> {
    raw.parse::<Tradition>().map_err(|_| HadithReaderError::InvalidTradition(raw.to_string()))
}

fn parse_decision(raw: &str) -> Result<ReviewDecision, HadithReaderError> {
    raw.parse::<ReviewDecision>().map_err(|_| HadithReaderError::InvalidDecision(raw.to_string()))
}

fn parse_provenance_layer(raw: &str) -> Result<ProvenanceLayer, HadithReaderError> {
    raw.parse::<ProvenanceLayer>()
        .map_err(|_| HadithReaderError::InvalidProvenanceLayer(raw.to_string()))
}

fn parse_hash(raw: &str) -> Result<ContentHash, HadithReaderError> {
    let (algorithm, hex) =
        raw.split_once(':').ok_or_else(|| HadithReaderError::InvalidHash(raw.to_string()))?;
    let algorithm = match algorithm {
        "sha256" => HashAlgorithm::Sha256,
        "blake3" => HashAlgorithm::Blake3,
        _ => return Err(HadithReaderError::InvalidHash(raw.to_string())),
    };
    ContentHash::try_new(algorithm, hex.to_string())
        .map_err(|_| HadithReaderError::InvalidHash(raw.to_string()))
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

fn parse_timestamp(raw: &str) -> Result<Timestamp, HadithReaderError> {
    raw.parse::<Timestamp>().map_err(|_| HadithReaderError::InvalidHash(raw.to_string()))
}

fn parse_semver(raw: &str) -> Result<SemVer, HadithReaderError> {
    raw.parse::<SemVer>().map_err(|_| HadithReaderError::InvalidHash(raw.to_string()))
}

/// Hadith reader trait: insert/retrieve hadith records, collections, and gradings.
#[async_trait]
pub trait HadithReader: Send + Sync {
    /// Insert a hadith collection.
    async fn insert_collection(
        &self,
        collection: &HadithCollection,
    ) -> Result<(), HadithReaderError>;
    /// Get a hadith collection by id.
    async fn get_collection(
        &self,
        id: &HadithCollectionId,
    ) -> Result<HadithCollection, HadithReaderError>;
    /// Get a hadith collection by slug.
    async fn get_collection_by_slug(
        &self,
        slug: &str,
    ) -> Result<HadithCollection, HadithReaderError>;
    /// List all hadith collections.
    async fn list_collections(&self) -> Result<Vec<HadithCollection>, HadithReaderError>;
    /// Insert a hadith record.
    async fn insert_record(&self, record: &HadithRecord) -> Result<(), HadithReaderError>;
    /// Get a hadith record by collection and source number.
    async fn get_record_by_source_number(
        &self,
        collection_id: &HadithCollectionId,
        source_number: &str,
    ) -> Result<HadithRecord, HadithReaderError>;
    /// Insert a grading record.
    async fn insert_grading(&self, grading: &GradingRecord) -> Result<(), HadithReaderError>;
    /// Get all grading records for a hadith.
    async fn get_gradings_for_hadith(
        &self,
        hadith_id: &HadithId,
    ) -> Result<Vec<GradingRecord>, HadithReaderError>;
}

/// The hadith reader service.
pub struct HadithReaderService {
    db: Arc<SqliteDatabase>,
}

impl HadithReaderService {
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
impl HadithReader for HadithReaderService {
    async fn insert_collection(
        &self,
        collection: &HadithCollection,
    ) -> Result<(), HadithReaderError> {
        sqlx::query(
            r#"INSERT INTO hadith_collections
               (id, slug, version, name, tradition, source_url, license_json,
                text_hash, structure_hash, source_version_id, status, imported_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(collection.id.to_string())
        .bind(&collection.slug)
        .bind(collection.version.to_string())
        .bind(&collection.name)
        .bind(collection.tradition.as_str())
        .bind(&collection.source_url)
        .bind(&collection.license_json)
        .bind(encode_hash(&collection.text_hash))
        .bind(encode_hash(&collection.structure_hash))
        .bind(collection.source_version_id.to_string())
        .bind(collection.status.as_str())
        .bind(collection.imported_at.to_string())
        .execute(self.db.write_pool())
        .await
        .map_err(|e| {
            HadithReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
        })?;
        Ok(())
    }

    async fn get_collection(
        &self,
        id: &HadithCollectionId,
    ) -> Result<HadithCollection, HadithReaderError> {
        let row = sqlx::query(
            r#"SELECT id, slug, version, name, tradition, source_url, license_json,
                      text_hash, structure_hash, source_version_id, status, imported_at
               FROM hadith_collections WHERE id = ?"#,
        )
        .bind(id.to_string())
        .fetch_optional(self.db.read_pool())
        .await
        .map_err(|e| {
            HadithReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
        })?
        .ok_or_else(|| HadithReaderError::CollectionNotFound(id.to_string()))?;
        Ok(HadithCollection {
            id: parse_collection_id(&row.get::<String, _>(0))?,
            slug: row.get(1),
            version: parse_semver(&row.get::<String, _>(2))?,
            name: row.get(3),
            tradition: parse_tradition(&row.get::<String, _>(4))?,
            source_url: row.get(5),
            license_json: row.get(6),
            text_hash: parse_hash(&row.get::<String, _>(7))?,
            structure_hash: parse_hash(&row.get::<String, _>(8))?,
            source_version_id: parse_source_version_id(&row.get::<String, _>(9))?,
            status: parse_record_status(&row.get::<String, _>(10))?,
            imported_at: parse_timestamp(&row.get::<String, _>(11))?,
        })
    }

    async fn get_collection_by_slug(
        &self,
        slug: &str,
    ) -> Result<HadithCollection, HadithReaderError> {
        let row = sqlx::query(
            r#"SELECT id, slug, version, name, tradition, source_url, license_json,
                      text_hash, structure_hash, source_version_id, status, imported_at
               FROM hadith_collections WHERE slug = ?"#,
        )
        .bind(slug)
        .fetch_optional(self.db.read_pool())
        .await
        .map_err(|e| {
            HadithReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
        })?
        .ok_or_else(|| HadithReaderError::CollectionNotFound(slug.to_string()))?;
        Ok(HadithCollection {
            id: parse_collection_id(&row.get::<String, _>(0))?,
            slug: row.get(1),
            version: parse_semver(&row.get::<String, _>(2))?,
            name: row.get(3),
            tradition: parse_tradition(&row.get::<String, _>(4))?,
            source_url: row.get(5),
            license_json: row.get(6),
            text_hash: parse_hash(&row.get::<String, _>(7))?,
            structure_hash: parse_hash(&row.get::<String, _>(8))?,
            source_version_id: parse_source_version_id(&row.get::<String, _>(9))?,
            status: parse_record_status(&row.get::<String, _>(10))?,
            imported_at: parse_timestamp(&row.get::<String, _>(11))?,
        })
    }

    async fn list_collections(&self) -> Result<Vec<HadithCollection>, HadithReaderError> {
        let rows = sqlx::query(
            r#"SELECT id, slug, version, name, tradition, source_url, license_json,
                      text_hash, structure_hash, source_version_id, status, imported_at
               FROM hadith_collections ORDER BY name ASC"#,
        )
        .fetch_all(self.db.read_pool())
        .await
        .map_err(|e| {
            HadithReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
        })?;
        let mut collections = Vec::with_capacity(rows.len());
        for row in &rows {
            collections.push(HadithCollection {
                id: parse_collection_id(&row.get::<String, _>(0))?,
                slug: row.get(1),
                version: parse_semver(&row.get::<String, _>(2))?,
                name: row.get(3),
                tradition: parse_tradition(&row.get::<String, _>(4))?,
                source_url: row.get(5),
                license_json: row.get(6),
                text_hash: parse_hash(&row.get::<String, _>(7))?,
                structure_hash: parse_hash(&row.get::<String, _>(8))?,
                source_version_id: parse_source_version_id(&row.get::<String, _>(9))?,
                status: parse_record_status(&row.get::<String, _>(10))?,
                imported_at: parse_timestamp(&row.get::<String, _>(11))?,
            });
        }
        Ok(collections)
    }

    async fn insert_record(&self, record: &HadithRecord) -> Result<(), HadithReaderError> {
        sqlx::query(
            r#"INSERT INTO hadith_records
               (id, collection_id, source_number, matn_text, matn_hash, isnad_text,
                narrator_names_json, source_version_id, status, metadata_json, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(record.id.to_string())
        .bind(record.collection_id.to_string())
        .bind(&record.source_number)
        .bind(&record.matn_text)
        .bind(encode_hash(&record.matn_hash))
        .bind(&record.isnad_text)
        .bind(serde_json::to_string(&record.narrator_names).unwrap_or_else(|_| "[]".to_string()))
        .bind(record.source_version_id.to_string())
        .bind(record.status.as_str())
        .bind(record.metadata_json.to_string())
        .bind(record.created_at.to_string())
        .execute(self.db.write_pool())
        .await
        .map_err(|e| {
            HadithReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
        })?;
        Ok(())
    }

    async fn get_record_by_source_number(
        &self,
        collection_id: &HadithCollectionId,
        source_number: &str,
    ) -> Result<HadithRecord, HadithReaderError> {
        let row = sqlx::query(
            r#"SELECT id, collection_id, source_number, matn_text, matn_hash, isnad_text,
                      narrator_names_json, source_version_id, status, metadata_json, created_at
               FROM hadith_records WHERE collection_id = ? AND source_number = ?"#,
        )
        .bind(collection_id.to_string())
        .bind(source_number)
        .fetch_optional(self.db.read_pool())
        .await
        .map_err(|e| {
            HadithReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
        })?
        .ok_or_else(|| {
            HadithReaderError::RecordNotFound(format!("{collection_id}:{source_number}"))
        })?;
        let narrator_names: Vec<String> =
            serde_json::from_str(&row.get::<String, _>(6)).unwrap_or_default();
        Ok(HadithRecord {
            id: parse_hadith_id(&row.get::<String, _>(0))?,
            collection_id: parse_collection_id(&row.get::<String, _>(1))?,
            source_number: row.get(2),
            matn_text: row.get(3),
            matn_hash: parse_hash(&row.get::<String, _>(4))?,
            isnad_text: row.get(5),
            narrator_names,
            source_version_id: parse_source_version_id(&row.get::<String, _>(7))?,
            status: parse_record_status(&row.get::<String, _>(8))?,
            metadata_json: serde_json::from_str(&row.get::<String, _>(9))
                .unwrap_or(serde_json::Value::Null),
            created_at: parse_timestamp(&row.get::<String, _>(10))?,
        })
    }

    async fn insert_grading(&self, grading: &GradingRecord) -> Result<(), HadithReaderError> {
        sqlx::query(
            r#"INSERT INTO hadith_gradings
               (id, hadith_id, scholar_name, grading_value, source_citation, evidence_json,
                reviewer, decision, decided_at, supersedes_id, provenance_layer, confidence, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(grading.id.to_string())
        .bind(grading.hadith_id.to_string())
        .bind(&grading.scholar_name)
        .bind(&grading.grading_value)
        .bind(&grading.source_citation)
        .bind(grading.evidence_json.to_string())
        .bind(&grading.reviewer)
        .bind(grading.decision.as_str())
        .bind(grading.decided_at.map(|t| t.to_string()))
        .bind(grading.supersedes_id.map(|id| id.to_string()))
        .bind(grading.provenance_layer.as_str())
        .bind(grading.confidence)
        .bind(grading.created_at.to_string())
        .execute(self.db.write_pool())
        .await
        .map_err(|e| HadithReaderError::Storage(StorageError::ConstraintViolation {
            message: e.to_string(),
        }))?;
        Ok(())
    }

    async fn get_gradings_for_hadith(
        &self,
        hadith_id: &HadithId,
    ) -> Result<Vec<GradingRecord>, HadithReaderError> {
        let rows = sqlx::query(
            r#"SELECT id, hadith_id, scholar_name, grading_value, source_citation, evidence_json,
                      reviewer, decision, decided_at, supersedes_id, provenance_layer, confidence, created_at
               FROM hadith_gradings WHERE hadith_id = ?
               ORDER BY created_at ASC"#,
        )
        .bind(hadith_id.to_string())
        .fetch_all(self.db.read_pool())
        .await
        .map_err(|e| HadithReaderError::Storage(StorageError::ConstraintViolation {
            message: e.to_string(),
        }))?;
        let mut gradings = Vec::with_capacity(rows.len());
        for row in &rows {
            let decided_at: Option<String> = row.get(8);
            let supersedes_id: Option<String> = row.get(9);
            let confidence: Option<f64> = row.get(11);
            gradings.push(GradingRecord {
                id: parse_grading_id(&row.get::<String, _>(0))?,
                hadith_id: parse_hadith_id(&row.get::<String, _>(1))?,
                scholar_name: row.get(2),
                grading_value: row.get(3),
                source_citation: row.get(4),
                evidence_json: serde_json::from_str(&row.get::<String, _>(5))
                    .unwrap_or(serde_json::Value::Null),
                reviewer: row.get(6),
                decision: parse_decision(&row.get::<String, _>(7))?,
                decided_at: decided_at.as_deref().map(parse_timestamp).transpose()?,
                supersedes_id: supersedes_id.as_deref().map(parse_grading_id).transpose()?,
                provenance_layer: parse_provenance_layer(&row.get::<String, _>(10))?,
                confidence,
                created_at: parse_timestamp(&row.get::<String, _>(12))?,
            });
        }
        Ok(gradings)
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

    fn sample_grading(hadith_id: &HadithId) -> GradingRecord {
        GradingRecord {
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
    async fn insert_and_retrieve_collection() {
        let db = test_db().await;
        let reader = HadithReaderService::new(db);
        let collection = sample_collection();
        reader.insert_collection(&collection).await.unwrap();
        let fetched = reader.get_collection(&collection.id).await.unwrap();
        assert_eq!(fetched.id, collection.id);
        assert_eq!(fetched.slug, collection.slug);
        assert_eq!(fetched.tradition, Tradition::Sunni);
    }

    #[tokio::test]
    async fn insert_and_retrieve_record() {
        let db = test_db().await;
        let reader = HadithReaderService::new(db);
        let collection = sample_collection();
        reader.insert_collection(&collection).await.unwrap();
        let record = sample_record(&collection.id);
        reader.insert_record(&record).await.unwrap();
        let fetched =
            reader.get_record_by_source_number(&collection.id, "bukhari:1234").await.unwrap();
        assert_eq!(fetched.id, record.id);
        assert_eq!(fetched.source_number, "bukhari:1234");
        assert_eq!(fetched.matn_text, "إنما الأعمال بالنيات");
        assert!(fetched.isnad_text.is_some());
    }

    #[tokio::test]
    async fn insert_and_retrieve_grading() {
        let db = test_db().await;
        let reader = HadithReaderService::new(db);
        let collection = sample_collection();
        reader.insert_collection(&collection).await.unwrap();
        let record = sample_record(&collection.id);
        reader.insert_record(&record).await.unwrap();
        let grading = sample_grading(&record.id);
        reader.insert_grading(&grading).await.unwrap();
        let gradings = reader.get_gradings_for_hadith(&record.id).await.unwrap();
        assert_eq!(gradings.len(), 1);
        assert_eq!(gradings[0].scholar_name, "Al-Albani");
        assert_eq!(gradings[0].grading_value, "sahih");
    }

    #[tokio::test]
    async fn record_not_found_error() {
        let db = test_db().await;
        let reader = HadithReaderService::new(db);
        let collection = sample_collection();
        reader.insert_collection(&collection).await.unwrap();
        let result = reader.get_record_by_source_number(&collection.id, "nonexistent").await;
        assert!(matches!(result, Err(HadithReaderError::RecordNotFound(_))));
    }
}
