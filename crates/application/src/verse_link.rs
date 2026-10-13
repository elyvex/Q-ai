//! Verse links: computational hadith/tafsir → Quran matches with an owner gate.
//!
//! # application::verse_link
//!
//! Per D-07 (locked): every computational match carries a confidence score
//! and the `"uncertain"` label. No computational match is presented as
//! certainty without owner confirmation. Confidence bands — high (> 0.9),
//! medium (0.7–0.9), low (< 0.7) — are display hints only: every band stays
//! `"uncertain"` until an owner confirms or rejects the link.
//!
//! The owner gate mirrors the `graph_assertions` review pattern (Phase 4):
//! `label = 'confirmed'` requires a non-empty reviewer plus `decided_at`,
//! enforced both here and by a database CHECK constraint (migration 0030).

use domain::Timestamp;
use serde::{Deserialize, Serialize};

/// Identifies a verse link record.
///
/// Manual uuid-backed newtype: `domain::typed_id!` is unusable outside the
/// `domain` crate (its `FromStr` error path names `crate::primitives`, which
/// resolves to the caller), so this follows the hadith-core/tafsir precedent.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct VerseLinkId(uuid::Uuid);

impl VerseLinkId {
    /// Generate a fresh random (v4) ID.
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// The inner `Uuid`, for (de)serialization and storage interop.
    pub const fn inner(&self) -> uuid::Uuid {
        self.0
    }
}

impl Default for VerseLinkId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for VerseLinkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for VerseLinkId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = uuid::Uuid::parse_str(s).map_err(|e| e.to_string())?;
        Ok(Self(id))
    }
}

impl From<VerseLinkId> for uuid::Uuid {
    fn from(id: VerseLinkId) -> Self {
        id.0
    }
}

/// Which record kind a verse link is anchored to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkSourceType {
    /// A hadith record (`hadith_records.id`).
    Hadith,
    /// A tafsir record (`tafsir_records.id`).
    Tafsir,
}

impl LinkSourceType {
    /// The string form stored in the database.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hadith => "hadith",
            Self::Tafsir => "tafsir",
        }
    }
}

impl std::fmt::Display for LinkSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for LinkSourceType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hadith" => Ok(Self::Hadith),
            "tafsir" => Ok(Self::Tafsir),
            other => Err(format!("unknown link source type `{other}`")),
        }
    }
}

/// Review status of a verse link.
///
/// Computational matches always start [`LinkLabel::Uncertain`]; only an
/// owner action (reviewer + timestamp) moves a link to confirmed/rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkLabel {
    /// Computational suggestion, not yet owner-reviewed.
    #[default]
    Uncertain,
    /// Owner-confirmed as authoritative.
    Confirmed,
    /// Owner-rejected.
    Rejected,
}

impl LinkLabel {
    /// The string form stored in the database.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Uncertain => "uncertain",
            Self::Confirmed => "confirmed",
            Self::Rejected => "rejected",
        }
    }
}

impl std::fmt::Display for LinkLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for LinkLabel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "uncertain" => Ok(Self::Uncertain),
            "confirmed" => Ok(Self::Confirmed),
            "rejected" => Ok(Self::Rejected),
            other => Err(format!("unknown link label `{other}`")),
        }
    }
}

/// Display band for a confidence score (D-07 thresholds).
///
/// Bands are hints only — every band stays [`LinkLabel::Uncertain`] until
/// an owner confirms the link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceBand {
    /// Confidence above 0.9.
    High,
    /// Confidence in 0.7–0.9.
    Medium,
    /// Confidence below 0.7.
    Low,
}

impl ConfidenceBand {
    /// Classify a confidence score into its display band.
    pub fn of(confidence: f64) -> Self {
        if confidence > 0.9 {
            Self::High
        } else if confidence >= 0.7 {
            Self::Medium
        } else {
            Self::Low
        }
    }
}

/// A verse link record: one computational or owner-reviewed connection
/// between a hadith/tafsir record and a Quran verse.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerseLink {
    /// Stable link identity.
    pub id: VerseLinkId,
    /// Which record kind this link is anchored to.
    pub source_type: LinkSourceType,
    /// The source record id (`hadith_records.id` or `tafsir_records.id`).
    pub source_id: String,
    /// Target surah number.
    pub target_surah: u16,
    /// Target ayah number.
    pub target_ayah: u32,
    /// Match confidence in 0.0–1.0.
    pub confidence: f64,
    /// Review status — always [`LinkLabel::Uncertain`] for new matches.
    pub label: LinkLabel,
    /// Matching algorithm name (reproducibility).
    pub algorithm: String,
    /// Matching algorithm version (reproducibility).
    pub algorithm_version: String,
    /// Owner who confirmed/rejected, if any.
    pub reviewer: Option<String>,
    /// When the owner decision was made, if any.
    pub decided_at: Option<Timestamp>,
    /// Creation timestamp.
    pub created_at: Timestamp,
}

impl VerseLink {
    /// A fresh computational suggestion: always `Uncertain`, never certain.
    pub fn suggested(
        source_type: LinkSourceType,
        source_id: String,
        target_surah: u16,
        target_ayah: u32,
        confidence: f64,
        algorithm: String,
        algorithm_version: String,
        created_at: Timestamp,
    ) -> Self {
        Self {
            id: VerseLinkId::new(),
            source_type,
            source_id,
            target_surah,
            target_ayah,
            confidence: confidence.clamp(0.0, 1.0),
            label: LinkLabel::Uncertain,
            algorithm,
            algorithm_version,
            reviewer: None,
            decided_at: None,
            created_at,
        }
    }

    /// Display band for this link's confidence.
    pub fn band(&self) -> ConfidenceBand {
        ConfidenceBand::of(self.confidence)
    }

    /// Whether this link is authoritative (owner-confirmed).
    pub fn is_authoritative(&self) -> bool {
        matches!(self.label, LinkLabel::Confirmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_link() -> VerseLink {
        VerseLink::suggested(
            LinkSourceType::Hadith,
            "hadith-id-1".to_string(),
            2,
            255,
            0.92,
            "phrase-overlap-l5".to_string(),
            "1.0.0".to_string(),
            Timestamp::from_ymd_hms(2026, 1, 15, 0, 0, 0).unwrap(),
        )
    }

    #[test]
    fn verse_link_roundtrips_serde() {
        let link = sample_link();
        let json = serde_json::to_string(&link).unwrap();
        assert!(json.contains("\"label\":\"uncertain\""));
        assert!(json.contains("\"target_surah\":2"));
        let back: VerseLink = serde_json::from_str(&json).unwrap();
        assert_eq!(back, link);
    }

    #[test]
    fn suggested_links_start_uncertain_and_never_authoritative() {
        for confidence in [0.0, 0.5, 0.7, 0.9, 0.95, 1.0] {
            let mut link = sample_link();
            link.confidence = confidence;
            // The constructor always labels Uncertain; even a 1.0 score is
            // not certainty without owner confirmation.
            assert_eq!(link.label, LinkLabel::Uncertain);
            assert!(!link.is_authoritative());
            assert!(link.reviewer.is_none());
            assert!(link.decided_at.is_none());
        }
        assert_eq!(LinkLabel::default(), LinkLabel::Uncertain);
    }

    #[test]
    fn confidence_bands_follow_d07_thresholds() {
        assert_eq!(ConfidenceBand::of(0.95), ConfidenceBand::High);
        assert_eq!(ConfidenceBand::of(0.9001), ConfidenceBand::High);
        assert_eq!(ConfidenceBand::of(0.9), ConfidenceBand::Medium);
        assert_eq!(ConfidenceBand::of(0.7), ConfidenceBand::Medium);
        assert_eq!(ConfidenceBand::of(0.6999), ConfidenceBand::Low);
        assert_eq!(ConfidenceBand::of(0.0), ConfidenceBand::Low);
        // Bands never change the label: high confidence is still uncertain.
        let link = sample_link();
        assert_eq!(link.band(), ConfidenceBand::High);
        assert_eq!(link.label, LinkLabel::Uncertain);
    }

    #[test]
    fn source_type_and_label_roundtrip_strings() {
        assert_eq!("hadith".parse::<LinkSourceType>().unwrap(), LinkSourceType::Hadith);
        assert_eq!("tafsir".parse::<LinkSourceType>().unwrap(), LinkSourceType::Tafsir);
        assert!("quran".parse::<LinkSourceType>().is_err());
        assert_eq!("uncertain".parse::<LinkLabel>().unwrap(), LinkLabel::Uncertain);
        assert_eq!("confirmed".parse::<LinkLabel>().unwrap(), LinkLabel::Confirmed);
        assert_eq!("rejected".parse::<LinkLabel>().unwrap(), LinkLabel::Rejected);
        assert!("pending".parse::<LinkLabel>().is_err());
    }

    #[test]
    fn confidence_is_clamped_to_unit_range() {
        let mut link = sample_link();
        link.confidence = 1.5;
        let clamped = VerseLink::suggested(
            link.source_type,
            link.source_id.clone(),
            link.target_surah,
            link.target_ayah,
            1.5,
            link.algorithm.clone(),
            link.algorithm_version.clone(),
            link.created_at,
        );
        assert_eq!(clamped.confidence, 1.0);
        let clamped_low = VerseLink::suggested(
            link.source_type,
            link.source_id,
            link.target_surah,
            link.target_ayah,
            -0.5,
            link.algorithm,
            link.algorithm_version,
            link.created_at,
        );
        assert_eq!(clamped_low.confidence, 0.0);
    }
}
