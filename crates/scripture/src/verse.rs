//! Scripture verse model: canonical immutable text.

use domain::{ContentHash, Timestamp};
use serde::{Deserialize, Serialize};

use crate::edition::ScriptureEditionId;
use crate::status::ScriptureStatus;

/// Identifies a scripture verse as a stable string:
/// `scripture:{edition}:{book}:{chapter}:{verse}`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScriptureVerseId(String);

impl ScriptureVerseId {
    /// Build from parts; stores the edition id (Uuid) to avoid rename drift.
    pub fn from_parts(
        edition_id: &ScriptureEditionId,
        book: &str,
        chapter: u32,
        verse: u32,
    ) -> Self {
        Self(format!("scripture:{}:{}:{}:{}", edition_id, book, chapter, verse))
    }

    /// The inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The inner string (owned).
    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for ScriptureVerseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for ScriptureVerseId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl serde::Serialize for ScriptureVerseId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for ScriptureVerseId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(s))
    }
}

/// Error returned when a verse's chapter or verse is invalid.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ScriptureVerseError {
    #[error("chapter must be greater than zero")]
    InvalidChapter,
    #[error("verse must be greater than zero")]
    InvalidVerse,
}

pub type ScriptureError = ScriptureVerseError;

/// A single scripture verse with canonical immutable text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptureVerse {
    /// Stable verse identity.
    pub id: ScriptureVerseId,
    /// The edition this verse belongs to.
    pub edition_id: ScriptureEditionId,
    /// Book slug.
    pub book: String,
    /// Chapter number (1-based).
    pub chapter: u32,
    /// Verse number (1-based).
    pub verse: u32,
    /// Canonical text (immutable).
    pub text: String,
    /// Hash of canonical text.
    pub text_hash: ContentHash,
    /// Lifecycle status.
    pub status: ScriptureStatus,
    /// Creation timestamp.
    pub created_at: Timestamp,
}

impl ScriptureVerse {
    /// Construct a verse, validating chapter and verse > 0.
    pub fn new(
        id: ScriptureVerseId,
        edition_id: ScriptureEditionId,
        book: String,
        chapter: u32,
        verse: u32,
        text: String,
        text_hash: ContentHash,
        status: ScriptureStatus,
        created_at: Timestamp,
    ) -> Result<Self, ScriptureVerseError> {
        if chapter == 0 {
            return Err(ScriptureVerseError::InvalidChapter);
        }
        if verse == 0 {
            return Err(ScriptureVerseError::InvalidVerse);
        }
        Ok(Self { id, edition_id, book, chapter, verse, text, text_hash, status, created_at })
    }

    /// Validate chapter and verse > 0.
    pub fn validate(&self) -> Result<(), ScriptureVerseError> {
        if self.chapter == 0 {
            return Err(ScriptureVerseError::InvalidChapter);
        }
        if self.verse == 0 {
            return Err(ScriptureVerseError::InvalidVerse);
        }
        Ok(())
    }

    /// Whether this verse may be served as canonical text.
    pub fn is_readable(&self) -> bool {
        matches!(self.status, ScriptureStatus::Active | ScriptureStatus::Deprecated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edition::ScriptureEditionId;
    use domain::{HashAlgorithm, Timestamp};

    fn content_hash(byte: &str) -> ContentHash {
        ContentHash::try_new(HashAlgorithm::Sha256, byte.repeat(32)).unwrap()
    }

    #[test]
    fn verse_id_from_parts() {
        let edition = ScriptureEditionId::new();
        let id = ScriptureVerseId::from_parts(&edition, "genesis", 1, 1);
        assert_eq!(id.to_string(), format!("scripture:{}:genesis:1:1", edition));
    }

    #[test]
    fn new_validates_chapter_verse() {
        let edition = ScriptureEditionId::new();
        let id = ScriptureVerseId::from_parts(&edition, "genesis", 0, 1);
        let res = ScriptureVerse::new(
            id,
            edition,
            "genesis".to_string(),
            0,
            1,
            "text".to_string(),
            content_hash("ab"),
            ScriptureStatus::Active,
            Timestamp::from_ymd_hms(2026, 1, 15, 0, 0, 0).unwrap(),
        );
        assert!(res.is_err());
    }

    #[test]
    fn verse_roundtrips_serde() {
        let edition = ScriptureEditionId::new();
        let id = ScriptureVerseId::from_parts(&edition, "genesis", 1, 1);
        let verse = ScriptureVerse::new(
            id.clone(),
            edition,
            "genesis".to_string(),
            1,
            1,
            "In the beginning".to_string(),
            content_hash("ab"),
            ScriptureStatus::Active,
            Timestamp::from_ymd_hms(2026, 1, 15, 0, 0, 0).unwrap(),
        )
        .unwrap();
        let json = serde_json::to_string(&verse).unwrap();
        let back: ScriptureVerse = serde_json::from_str(&json).unwrap();
        assert_eq!(back, verse);
        assert_eq!(back.id, id);
    }
}
