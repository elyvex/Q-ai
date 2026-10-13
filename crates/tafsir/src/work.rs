//! Tafsir work/author/dimension types.
//!
//! Follows the `QuranEdition` pattern: versioned, identified, with immutable
//! hashes and lifecycle status. Per D-06 (locked): tafsir is indexed by
//! work/author/dimensions following the existing catalog pattern.
//!
//! [`TafsirId`] and [`TafsirWorkId`] are uuid-backed newtypes with the same
//! trait set as the `hadith-core` IDs (`Display`/`FromStr` with `String`
//! errors, string-transparent serde): the `domain::typed_id!` macro cannot be
//! used here because its `FromStr` error path names `crate::primitives`, which
//! resolves to the *calling* crate under `macro_rules!` hygiene.

use domain::{SemVer, SourceVersionId, Timestamp};
use serde::{Deserialize, Serialize};

/// Identifies a tafsir work.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct TafsirWorkId(uuid::Uuid);

impl TafsirWorkId {
    /// Generate a fresh random (v4) ID.
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// The inner `Uuid`, for (de)serialization and storage interop.
    pub const fn inner(&self) -> uuid::Uuid {
        self.0
    }
}

impl Default for TafsirWorkId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TafsirWorkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for TafsirWorkId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = uuid::Uuid::parse_str(s).map_err(|e| e.to_string())?;
        Ok(Self(id))
    }
}

impl From<TafsirWorkId> for uuid::Uuid {
    fn from(id: TafsirWorkId) -> Self {
        id.0
    }
}

/// Lifecycle status for tafsir works and records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum WorkStatus {
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

impl WorkStatus {
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

impl std::fmt::Display for WorkStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for WorkStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Staged" => Ok(Self::Staged),
            "Approved" => Ok(Self::Approved),
            "Active" => Ok(Self::Active),
            "Deprecated" => Ok(Self::Deprecated),
            "Quarantined" => Ok(Self::Quarantined),
            other => Err(format!("unknown work status `{other}`")),
        }
    }
}

/// The dimensions locating a tafsir record inside its work.
///
/// All fields are optional: works are organized differently (some by volume +
/// page, some by chapter, some by all three). `chapter` is free text — e.g. a
/// surah name or number — interpreted per the work's `dimensions_schema_json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TafsirDimensions {
    /// Volume number, when the work is multi-volume.
    #[serde(default)]
    pub volume: Option<u32>,
    /// Page number, when the work is paginated.
    #[serde(default)]
    pub page: Option<u32>,
    /// Chapter (e.g. surah name or number), when the work is chaptered.
    #[serde(default)]
    pub chapter: Option<String>,
}

/// A versioned tafsir work.
///
/// The identity fields (`slug`, `version`, `author`) are immutable once
/// written. `author` is non-empty and attributed (T-06-06).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TafsirWork {
    /// Stable work identity.
    pub id: TafsirWorkId,
    /// URL/CLI-safe slug, e.g. `tafsir-tabari`.
    pub slug: String,
    /// Human-readable name.
    pub name: String,
    /// Attributed author (non-empty).
    pub author: String,
    /// How to interpret volume/page/chapter for this work (JSON).
    pub dimensions_schema_json: String,
    /// Work version.
    pub version: SemVer,
    /// The source version this work was imported from.
    pub source_version_id: SourceVersionId,
    /// Import timestamp.
    pub created_at: Timestamp,
    /// Lifecycle status.
    pub status: WorkStatus,
}

impl TafsirWork {
    /// Whether this work may be served as canonical text.
    pub fn is_readable(&self) -> bool {
        matches!(self.status, WorkStatus::Active | WorkStatus::Deprecated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_work() -> TafsirWork {
        TafsirWork {
            id: TafsirWorkId::new(),
            slug: "tafsir-tabari".to_string(),
            name: "Jami' al-Bayan (Tabari)".to_string(),
            author: "Muhammad ibn Jarir al-Tabari".to_string(),
            dimensions_schema_json: r#"{"chapter":"surah-number"}"#.to_string(),
            version: SemVer::new(1, 0, 0),
            source_version_id: SourceVersionId::new(),
            created_at: Timestamp::from_ymd_hms(2026, 1, 15, 0, 0, 0).unwrap(),
            status: WorkStatus::Active,
        }
    }

    #[test]
    fn work_roundtrips_serde() {
        let work = sample_work();
        let json = serde_json::to_string(&work).unwrap();
        assert!(json.contains("\"author\":\"Muhammad ibn Jarir al-Tabari\""));
        assert!(json.contains("\"status\":\"Active\""));
        let back: TafsirWork = serde_json::from_str(&json).unwrap();
        assert_eq!(back, work);
    }

    #[test]
    fn dimensions_roundtrip_serde() {
        let dims =
            TafsirDimensions { volume: Some(3), page: Some(12), chapter: Some("2".to_string()) };
        let json = serde_json::to_string(&dims).unwrap();
        let back: TafsirDimensions = serde_json::from_str(&json).unwrap();
        assert_eq!(back, dims);
        let empty = TafsirDimensions { volume: None, page: None, chapter: None };
        let back: TafsirDimensions =
            serde_json::from_str(&serde_json::to_string(&empty).unwrap()).unwrap();
        assert_eq!(back, empty);
    }

    #[test]
    fn readability_depends_on_status() {
        let mut work = sample_work();
        assert!(work.is_readable());
        work.status = WorkStatus::Staged;
        assert!(!work.is_readable());
        work.status = WorkStatus::Quarantined;
        assert!(!work.is_readable());
    }

    #[test]
    fn status_roundtrips() {
        for status in [
            WorkStatus::Staged,
            WorkStatus::Approved,
            WorkStatus::Active,
            WorkStatus::Deprecated,
            WorkStatus::Quarantined,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let back: WorkStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(back, status);
        }
    }
}
