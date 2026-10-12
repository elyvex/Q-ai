//! Hadith collection metadata.
//!
//! Follows the `QuranEdition` pattern: versioned, identified, with immutable
//! hashes and lifecycle status. The `tradition` field is metadata only — no
//! separate code path per CONTEXT.md locked decision.

use domain::{ContentHash, SemVer, SourceVersionId, Timestamp};
use serde::{Deserialize, Serialize};

/// Identifies a hadith collection.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct HadithCollectionId(uuid::Uuid);

impl HadithCollectionId {
    /// Generate a fresh random (v4) ID.
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// The inner `Uuid`, for (de)serialization and storage interop.
    pub const fn inner(&self) -> uuid::Uuid {
        self.0
    }
}

impl Default for HadithCollectionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for HadithCollectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for HadithCollectionId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = uuid::Uuid::parse_str(s).map_err(|e| e.to_string())?;
        Ok(Self(id))
    }
}

impl From<HadithCollectionId> for uuid::Uuid {
    fn from(id: HadithCollectionId) -> Self {
        id.0
    }
}

/// The Islamic tradition a collection belongs to.
///
/// This is metadata only — it never changes the record structure or code path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tradition {
    /// Sunni tradition.
    Sunni,
    /// Shia tradition.
    Shia,
}

impl Tradition {
    /// The string form stored in the database.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sunni => "sunni",
            Self::Shia => "shia",
        }
    }
}

impl std::fmt::Display for Tradition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Tradition {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sunni" => Ok(Self::Sunni),
            "shia" => Ok(Self::Shia),
            other => Err(format!("unknown tradition `{other}`")),
        }
    }
}

/// A versioned hadith collection.
///
/// The identity/hash fields (`text_hash`, `structure_hash`, `slug`, `version`)
/// are immutable once written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HadithCollection {
    /// Stable collection identity.
    pub id: HadithCollectionId,
    /// URL/CLI-safe slug, e.g. `bukhari-arabic`.
    pub slug: String,
    /// Human-readable name.
    pub name: String,
    /// Islamic tradition (metadata only).
    pub tradition: Tradition,
    /// Source URL, if any.
    pub source_url: Option<String>,
    /// License record as JSON.
    pub license_json: String,
    /// Hash over the canonical matn text stream.
    pub text_hash: ContentHash,
    /// Hash over the structural skeleton.
    pub structure_hash: ContentHash,
    /// Collection version.
    pub version: SemVer,
    /// The source version this collection was imported from.
    pub source_version_id: SourceVersionId,
    /// Import timestamp.
    pub imported_at: Timestamp,
    /// Lifecycle status.
    pub status: crate::RecordStatus,
}

impl HadithCollection {
    /// Whether this collection may be served as canonical text.
    pub fn is_readable(&self) -> bool {
        matches!(self.status, crate::RecordStatus::Active | crate::RecordStatus::Deprecated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::HashAlgorithm;

    fn content_hash(byte: &str) -> ContentHash {
        ContentHash::try_new(HashAlgorithm::Sha256, byte.repeat(32)).unwrap()
    }

    fn sample_collection() -> HadithCollection {
        HadithCollection {
            id: HadithCollectionId::new(),
            slug: "bukhari-arabic".to_string(),
            name: "Sahih al-Bukhari (Arabic)".to_string(),
            tradition: Tradition::Sunni,
            source_url: None,
            license_json: r#"{"status":"PublicDomain"}"#.to_string(),
            text_hash: content_hash("ab"),
            structure_hash: content_hash("cd"),
            version: SemVer::new(1, 0, 0),
            source_version_id: SourceVersionId::new(),
            imported_at: Timestamp::from_ymd_hms(2026, 1, 15, 0, 0, 0).unwrap(),
            status: crate::RecordStatus::Staged,
        }
    }

    #[test]
    fn collection_roundtrips_serde() {
        let collection = sample_collection();
        let json = serde_json::to_string(&collection).unwrap();
        assert!(json.contains("\"tradition\":\"sunni\""));
        assert!(json.contains("\"status\":\"Staged\""));
        let back: HadithCollection = serde_json::from_str(&json).unwrap();
        assert_eq!(back, collection);
    }

    #[test]
    fn readability_depends_on_status() {
        let mut collection = sample_collection();
        assert!(!collection.is_readable());
        collection.status = crate::RecordStatus::Active;
        assert!(collection.is_readable());
        collection.status = crate::RecordStatus::Deprecated;
        assert!(collection.is_readable());
        collection.status = crate::RecordStatus::Quarantined;
        assert!(!collection.is_readable());
    }

    #[test]
    fn tradition_roundtrips() {
        let sunni = Tradition::Sunni;
        let shia = Tradition::Shia;
        assert_eq!(sunni.as_str(), "sunni");
        assert_eq!(shia.as_str(), "shia");
        assert_eq!("sunni".parse::<Tradition>().unwrap(), sunni);
        assert_eq!("shia".parse::<Tradition>().unwrap(), shia);
        assert!("invalid".parse::<Tradition>().is_err());
    }
}
