//! Scripture edition model (Phase 9).
//!
//! Edition-aware: version identity, immutable hashes, lifecycle status.
//! Tradition/religion metadata is data only.

use domain::{ContentHash, Language, SemVer, SourceVersionId, Timestamp};
use serde::{Deserialize, Serialize};

use crate::status::ScriptureStatus;

/// Identifies a scripture edition.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct ScriptureEditionId(uuid::Uuid);

impl ScriptureEditionId {
    /// Generate a fresh random (v4) ID.
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
    /// The inner `Uuid`.
    pub const fn inner(&self) -> uuid::Uuid {
        self.0
    }
}
impl Default for ScriptureEditionId {
    fn default() -> Self {
        Self::new()
    }
}
impl std::fmt::Display for ScriptureEditionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::str::FromStr for ScriptureEditionId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = uuid::Uuid::parse_str(s).map_err(|e| e.to_string())?;
        Ok(Self(id))
    }
}
impl From<ScriptureEditionId> for uuid::Uuid {
    fn from(id: ScriptureEditionId) -> Self {
        id.0
    }
}

/// Counts computed at import and stored with the edition.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptureEditionStatistics {
    /// Number of books.
    pub book_count: u16,
    /// Number of chapters.
    pub chapter_count: u32,
    /// Total verses.
    pub verse_count: u32,
}

/// The tradition/religion a scripture edition belongs to.
///
/// Metadata only — never changes record structure or code path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tradition {
    /// Jewish scripture (Torah/Tanakh).
    Jewish,
    /// Christian scripture (New Testament etc.).
    Christian,
    /// Other tradition.
    Other,
}

impl Tradition {
    /// The string form stored in the database.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Jewish => "jewish",
            Self::Christian => "christian",
            Self::Other => "other",
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
            "jewish" => Ok(Self::Jewish),
            "christian" => Ok(Self::Christian),
            "other" => Ok(Self::Other),
            other => Err(format!("unknown tradition `{other}`")),
        }
    }
}

/// A versioned edition of scripture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptureEdition {
    /// Stable edition identity.
    pub id: ScriptureEditionId,
    /// URL/CLI-safe slug.
    pub slug: String,
    /// Edition version.
    pub version: SemVer,
    /// Human-readable name.
    pub name: String,
    /// Tradition metadata.
    pub tradition: Tradition,
    /// Content language.
    pub language: Language,
    /// Source URL, if any.
    pub source_url: Option<String>,
    /// License record as JSON.
    pub license_json: serde_json::Value,
    /// Hash over canonical text.
    pub text_hash: ContentHash,
    /// Hash over structure.
    pub structure_hash: ContentHash,
    /// Hash over source manifest.
    pub manifest_hash: ContentHash,
    /// The source version this edition was imported from.
    pub source_version_id: SourceVersionId,
    /// Lifecycle status.
    pub status: ScriptureStatus,
    /// Import timestamp.
    pub imported_at: Timestamp,
    /// Computed counts.
    pub statistics: ScriptureEditionStatistics,
}

impl ScriptureEdition {
    /// Whether this edition may be served as canonical text.
    pub fn is_readable(&self) -> bool {
        matches!(self.status, ScriptureStatus::Active | ScriptureStatus::Deprecated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::HashAlgorithm;

    fn content_hash(byte: &str) -> ContentHash {
        ContentHash::try_new(HashAlgorithm::Sha256, byte.repeat(32)).unwrap()
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
            text_hash: content_hash("ab"),
            structure_hash: content_hash("cd"),
            manifest_hash: content_hash("12"),
            source_version_id: SourceVersionId::new(),
            status: ScriptureStatus::Staged,
            imported_at: Timestamp::from_ymd_hms(2026, 1, 15, 0, 0, 0).unwrap(),
            statistics: ScriptureEditionStatistics::default(),
        }
    }

    #[test]
    fn edition_roundtrips_serde() {
        let edition = sample_edition();
        let json = serde_json::to_string(&edition).unwrap();
        assert!(json.contains("\"tradition\":\"jewish\""));
        let back: ScriptureEdition = serde_json::from_str(&json).unwrap();
        assert_eq!(back, edition);
    }

    #[test]
    fn readability_depends_on_status() {
        let mut edition = sample_edition();
        assert!(!edition.is_readable());
        edition.status = ScriptureStatus::Active;
        assert!(edition.is_readable());
    }
}
