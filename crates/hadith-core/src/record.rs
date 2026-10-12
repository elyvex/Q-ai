//! Hadith record with matn/isnad separation.
//!
//! Follows the source-aware record pattern: `source_number` is NOT NULL and
//! preserved verbatim from the source (D-01). `matn_text` is canonical-only
//! (D-02) — never mutated by search index rebuilds. `isnad_text` is NULL when
//! the source doesn't separate matn from isnad.

use domain::{ContentHash, SourceVersionId, Timestamp};
use serde::{Deserialize, Serialize};

use crate::collection::HadithCollectionId;

/// Identifies a hadith record.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct HadithId(uuid::Uuid);

impl HadithId {
    /// Generate a fresh random (v4) ID.
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// The inner `Uuid`, for (de)serialization and storage interop.
    pub const fn inner(&self) -> uuid::Uuid {
        self.0
    }
}

impl Default for HadithId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for HadithId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for HadithId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = uuid::Uuid::parse_str(s).map_err(|e| e.to_string())?;
        Ok(Self(id))
    }
}

impl From<HadithId> for uuid::Uuid {
    fn from(id: HadithId) -> Self {
        id.0
    }
}

/// Lifecycle status for hadith records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum RecordStatus {
    /// Staged but not yet approved.
    Staged,
    /// Approved for serving.
    Approved,
    /// Active (canonical).
    Active,
    /// Deprecated but still readable.
    Deprecated,
    /// Quarantined (not readable).
    Quarantined,
}

impl RecordStatus {
    /// The string form stored in the database.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Staged => "Staged",
            Self::Approved => "Approved",
            Self::Active => "Active",
            Self::Deprecated => "Deprecated",
            Self::Quarantined => "Quarantined",
        }
    }
}

impl std::fmt::Display for RecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for RecordStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Staged" => Ok(Self::Staged),
            "Approved" => Ok(Self::Approved),
            "Active" => Ok(Self::Active),
            "Deprecated" => Ok(Self::Deprecated),
            "Quarantined" => Ok(Self::Quarantined),
            other => Err(format!("unknown record status `{other}`")),
        }
    }
}

/// A single hadith record with matn/isnad separation.
///
/// `source_number` is preserved verbatim from the source (e.g., "bukhari:1234").
/// `matn_text` is the canonical matn text — immutable, never mutated by search.
/// `isnad_text` is `None` when the source doesn't separate matn from isnad.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HadithRecord {
    /// Stable record identity.
    pub id: HadithId,
    /// The collection this record belongs to.
    pub collection_id: HadithCollectionId,
    /// Source numbering, preserved verbatim (e.g., "bukhari:1234").
    pub source_number: String,
    /// Canonical matn text (immutable).
    pub matn_text: String,
    /// Hash of the canonical matn text.
    pub matn_hash: ContentHash,
    /// Isnad text, when the source separates it from the matn.
    pub isnad_text: Option<String>,
    /// Narrator names extracted for search.
    pub narrator_names: Vec<String>,
    /// The source version this record was imported from.
    pub source_version_id: SourceVersionId,
    /// Lifecycle status.
    pub status: RecordStatus,
    /// Collection-specific metadata.
    pub metadata_json: serde_json::Value,
    /// Creation timestamp.
    pub created_at: Timestamp,
}

impl HadithRecord {
    /// Whether this record may be served as canonical text.
    pub fn is_readable(&self) -> bool {
        matches!(self.status, RecordStatus::Active | RecordStatus::Deprecated)
    }

    /// The canonical reference string: `collection_slug:source_number`.
    pub fn canonical_reference(&self, collection_slug: &str) -> String {
        format!("{collection_slug}:{}", self.source_number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::HashAlgorithm;

    fn content_hash(byte: &str) -> ContentHash {
        ContentHash::try_new(HashAlgorithm::Sha256, byte.repeat(32)).unwrap()
    }

    fn sample_record() -> HadithRecord {
        HadithRecord {
            id: HadithId::new(),
            collection_id: HadithCollectionId::new(),
            source_number: "bukhari:1234".to_string(),
            matn_text: "إنما الأعمال بالنيات".to_string(),
            matn_hash: content_hash("ab"),
            isnad_text: Some("حدثنا يحيى بن بكير".to_string()),
            narrator_names: vec!["يحيى بن بكير".to_string(), "إبراهيم بن المنذر".to_string()],
            source_version_id: SourceVersionId::new(),
            status: RecordStatus::Active,
            metadata_json: serde_json::json!({}),
            created_at: Timestamp::from_ymd_hms(2026, 1, 15, 0, 0, 0).unwrap(),
        }
    }

    #[test]
    fn record_roundtrips_serde() {
        let record = sample_record();
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"source_number\":\"bukhari:1234\""));
        assert!(json.contains("\"status\":\"Active\""));
        let back: HadithRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back, record);
    }

    #[test]
    fn record_without_isnad_roundtrips() {
        let mut record = sample_record();
        record.isnad_text = None;
        let json = serde_json::to_string(&record).unwrap();
        let back: HadithRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back, record);
        assert!(back.isnad_text.is_none());
    }

    #[test]
    fn readability_depends_on_status() {
        let mut record = sample_record();
        assert!(record.is_readable());
        record.status = RecordStatus::Staged;
        assert!(!record.is_readable());
        record.status = RecordStatus::Quarantined;
        assert!(!record.is_readable());
    }

    #[test]
    fn canonical_reference_includes_source_number() {
        let record = sample_record();
        assert_eq!(record.canonical_reference("bukhari-arabic"), "bukhari-arabic:bukhari:1234");
    }

    #[test]
    fn status_roundtrips() {
        for status in [
            RecordStatus::Staged,
            RecordStatus::Approved,
            RecordStatus::Active,
            RecordStatus::Deprecated,
            RecordStatus::Quarantined,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let back: RecordStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(back, status);
        }
    }
}
