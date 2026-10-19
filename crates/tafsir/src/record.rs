//! Tafsir record with work/author/dimension indexing.
//!
//! Follows the source-aware record pattern: `text` is canonical-only
//! (T-06-05/T-06-06) — never mutated by search index rebuilds. Derived search
//! forms live in `tafsir_search_forms` (migration 0029). Each record carries
//! its work, dimensions (volume, page, chapter), and linked verses come from
//! the `verse_links` table (plan 06-03).

use domain::{ContentHash, SourceVersionId, Timestamp};
use serde::{Deserialize, Serialize};

use crate::work::{TafsirWorkId, WorkStatus};

/// Identifies a tafsir record.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct TafsirId(uuid::Uuid);

impl TafsirId {
    /// Generate a fresh random (v4) ID.
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// The inner `Uuid`, for (de)serialization and storage interop.
    pub const fn inner(&self) -> uuid::Uuid {
        self.0
    }
}

impl Default for TafsirId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TafsirId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for TafsirId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = uuid::Uuid::parse_str(s).map_err(|e| e.to_string())?;
        Ok(Self(id))
    }
}

impl From<TafsirId> for uuid::Uuid {
    fn from(id: TafsirId) -> Self {
        id.0
    }
}

/// A single tafsir record: canonical text located by work + dimensions.
///
/// `text` is the canonical tafsir text — immutable, never mutated by search.
/// `volume`/`page`/`chapter` locate the passage inside the work; all are
/// optional because works are organized differently.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TafsirRecord {
    /// Stable record identity.
    pub id: TafsirId,
    /// The work this record belongs to.
    pub work_id: TafsirWorkId,
    /// Volume number, when the work is multi-volume.
    pub volume: Option<u32>,
    /// Page number, when the work is paginated.
    pub page: Option<u32>,
    /// Chapter (e.g. surah name or number), when the work is chaptered.
    pub chapter: Option<String>,
    /// Canonical tafsir text (immutable).
    pub text: String,
    /// Hash of the canonical text.
    pub text_hash: ContentHash,
    /// The source version this record was imported from.
    pub source_version_id: SourceVersionId,
    /// Lifecycle status.
    pub status: WorkStatus,
    /// Creation timestamp.
    pub created_at: Timestamp,
}

impl TafsirRecord {
    /// Whether this record may be served as canonical text.
    pub fn is_readable(&self) -> bool {
        matches!(self.status, WorkStatus::Active | WorkStatus::Deprecated)
    }

    /// Whether this record belongs to the given surah's chapter organization.
    ///
    /// Structural navigation only (the work's own chaptering): computational
    /// verse links with confidence + owner gate arrive in plan 06-03.
    pub fn is_for_surah(&self, surah: u16) -> bool {
        self.chapter.as_deref() == Some(surah.to_string().as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::HashAlgorithm;

    fn content_hash(byte: &str) -> ContentHash {
        ContentHash::try_new(HashAlgorithm::Sha256, byte.repeat(32)).unwrap()
    }

    fn sample_record() -> TafsirRecord {
        TafsirRecord {
            id: TafsirId::new(),
            work_id: TafsirWorkId::new(),
            volume: Some(3),
            page: Some(12),
            chapter: Some("2".to_string()),
            text: "القول في تأويل قوله تعالى".to_string(),
            text_hash: content_hash("ab"),
            source_version_id: SourceVersionId::new(),
            status: WorkStatus::Active,
            created_at: Timestamp::from_ymd_hms(2026, 1, 15, 0, 0, 0).unwrap(),
        }
    }

    #[test]
    fn record_roundtrips_serde() {
        let record = sample_record();
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"chapter\":\"2\""));
        assert!(json.contains("\"status\":\"Active\""));
        let back: TafsirRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back, record);
    }

    #[test]
    fn record_without_dimensions_roundtrips() {
        let mut record = sample_record();
        record.volume = None;
        record.page = None;
        record.chapter = None;
        let json = serde_json::to_string(&record).unwrap();
        let back: TafsirRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back, record);
    }

    #[test]
    fn readability_depends_on_status() {
        let mut record = sample_record();
        assert!(record.is_readable());
        record.status = WorkStatus::Staged;
        assert!(!record.is_readable());
        record.status = WorkStatus::Quarantined;
        assert!(!record.is_readable());
    }

    #[test]
    fn surah_match_is_structural() {
        let record = sample_record();
        assert!(record.is_for_surah(2));
        assert!(!record.is_for_surah(3));
    }
}
