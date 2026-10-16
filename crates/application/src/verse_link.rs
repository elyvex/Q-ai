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

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use domain::Timestamp;
use quran_normalization::{NormalizationPipeline, ProfileId, ProfileRegistry, SemVer};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use storage::error::StorageError;
use storage_sqlite::SqliteDatabase;

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

/// Matching algorithm recorded on every computational link.
pub const LINK_ALGORITHM: &str = "phrase-overlap-l5";
/// Version of the matching algorithm (reproducibility).
pub const LINK_ALGORITHM_VERSION: &str = "1.0.0";

/// A candidate Quran verse scored against a hadith/tafsir source text.
#[derive(Debug, Clone, PartialEq)]
pub struct VerseCandidate {
    /// Target surah number.
    pub surah: u16,
    /// Target ayah number.
    pub ayah: u32,
    /// Canonical verse text (scored, never stored here).
    pub text: String,
}

/// Verse link errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum LinkError {
    /// Storage failure.
    #[error("storage failed: {0}")]
    Storage(#[from] StorageError),
    /// Source record not found.
    #[error("source record not found: {0}")]
    SourceNotFound(String),
    /// Link not found.
    #[error("link not found: {0}")]
    LinkNotFound(String),
    /// Link already decided (confirmed or rejected).
    #[error("link already decided: {0}")]
    AlreadyDecided(String),
    /// Reviewer name missing or blank.
    #[error("reviewer name must not be empty")]
    InvalidReviewer,
    /// Normalization failure.
    #[error("normalization failed: {0}")]
    Normalization(String),
    /// Invalid label value.
    #[error("invalid label: {0}")]
    InvalidLabel(String),
    /// Invalid source type value.
    #[error("invalid source type: {0}")]
    InvalidSourceType(String),
    /// Invalid id.
    #[error("invalid id: {0}")]
    InvalidId(String),
    /// Invalid timestamp.
    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(String),
}

impl storage::error::Diagnostic for LinkError {
    fn code(&self) -> storage::error::DiagnosticCode {
        match self {
            Self::Storage(_) => storage::error::DiagnosticCode::new("QAI-VLK", 1),
            Self::SourceNotFound(_) => storage::error::DiagnosticCode::new("QAI-VLK", 2),
            Self::LinkNotFound(_) => storage::error::DiagnosticCode::new("QAI-VLK", 3),
            Self::AlreadyDecided(_) => storage::error::DiagnosticCode::new("QAI-VLK", 4),
            Self::InvalidReviewer => storage::error::DiagnosticCode::new("QAI-VLK", 5),
            Self::Normalization(_) => storage::error::DiagnosticCode::new("QAI-VLK", 6),
            Self::InvalidLabel(_)
            | Self::InvalidSourceType(_)
            | Self::InvalidId(_)
            | Self::InvalidTimestamp(_) => storage::error::DiagnosticCode::new("QAI-VLK", 7),
        }
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn remedy(&self) -> Option<String> {
        Some(match self {
            Self::InvalidReviewer => "Provide a non-empty reviewer name.".to_string(),
            Self::AlreadyDecided(_) => "The link already has an owner decision.".to_string(),
            Self::SourceNotFound(_) | Self::LinkNotFound(_) => {
                "Check the source or link id.".to_string()
            }
            _ => "Check the database and retry.".to_string(),
        })
    }

    fn next_command(&self) -> Option<String> {
        Some("qai hadith --help".to_string())
    }
}

/// Normalize text with the L5 profile (diacritics + hamza/alif + Arabic code
/// points), the same profile both sides of the hadith/tafsir FTS indexes use.
fn normalize_l5(text: &str) -> Result<String, LinkError> {
    let registry = ProfileRegistry::new();
    let pipeline =
        NormalizationPipeline::for_profile(&registry, ProfileId::L5, SemVer::new(1, 0, 0))
            .map_err(|e| LinkError::Normalization(e.to_string()))?;
    let (normalized, _trace) = pipeline.apply(text);
    Ok(normalized.text().to_string())
}

/// Score a source text against a candidate verse: Dice coefficient over
/// L5-normalized whitespace-token multisets.
///
/// Deterministic and explainable (RESEARCH recommendation): shared tokens
/// over mean length. Empty source text scores 0.0 — no evidence, no link.
pub fn score_overlap(source_text: &str, verse_text: &str) -> Result<f64, LinkError> {
    let source = normalize_l5(source_text)?;
    let verse = normalize_l5(verse_text)?;
    let mut counts: HashMap<&str, usize> = HashMap::new();
    let mut source_len = 0usize;
    for token in source.split_whitespace() {
        *counts.entry(token).or_insert(0) += 1;
        source_len += 1;
    }
    if source_len == 0 {
        return Ok(0.0);
    }
    let mut verse_len = 0usize;
    let mut shared = 0usize;
    for token in verse.split_whitespace() {
        verse_len += 1;
        if let Some(count) = counts.get_mut(token) {
            if *count > 0 {
                *count -= 1;
                shared += 1;
            }
        }
    }
    if verse_len == 0 {
        return Ok(0.0);
    }
    Ok(2.0 * shared as f64 / (source_len + verse_len) as f64)
}

fn parse_link_id(raw: &str) -> Result<VerseLinkId, LinkError> {
    raw.parse::<VerseLinkId>().map_err(|_| LinkError::InvalidId(raw.to_string()))
}

fn parse_timestamp(raw: &str) -> Result<Timestamp, LinkError> {
    raw.parse::<Timestamp>().map_err(|_| LinkError::InvalidTimestamp(raw.to_string()))
}

fn row_to_link(row: &sqlx::sqlite::SqliteRow) -> Result<VerseLink, LinkError> {
    // Column order matches the SELECT lists below: id, source_type,
    // source_id, target_surah, target_ayah, confidence, label, algorithm,
    // algorithm_version, reviewer, decided_at, created_at.
    let confidence: f64 = row.get(5);
    let label: String = row.get(6);
    let reviewer: Option<String> = row.get(9);
    let decided_at: Option<String> = row.get(10);
    Ok(VerseLink {
        id: parse_link_id(&row.get::<String, _>(0))?,
        source_type: row
            .get::<String, _>(1)
            .parse::<LinkSourceType>()
            .map_err(|_| LinkError::InvalidSourceType(row.get::<String, _>(1)))?,
        source_id: row.get(2),
        target_surah: row.get::<i64, _>(3) as u16,
        target_ayah: row.get::<i64, _>(4) as u32,
        confidence,
        label: label.parse::<LinkLabel>().map_err(|_| LinkError::InvalidLabel(label.clone()))?,
        algorithm: row.get(7),
        algorithm_version: row.get(8),
        reviewer,
        decided_at: decided_at.as_deref().map(parse_timestamp).transpose()?,
        created_at: parse_timestamp(&row.get::<String, _>(11))?,
    })
}

/// Verse link trait: computational suggestions plus the owner gate.
#[async_trait]
pub trait VerseLinker: Send + Sync {
    /// Score candidates against the source text and persist the matches as
    /// `Uncertain` links. Candidates with zero overlap are skipped (no
    /// evidence, no link).
    async fn suggest_links(
        &self,
        source_type: LinkSourceType,
        source_id: &str,
        candidates: &[VerseCandidate],
    ) -> Result<Vec<VerseLink>, LinkError>;
    /// Confirm a link as authoritative (owner action).
    async fn confirm_link(&self, link_id: &VerseLinkId, reviewer: &str) -> Result<(), LinkError>;
    /// Reject a link (owner action).
    async fn reject_link(&self, link_id: &VerseLinkId, reviewer: &str) -> Result<(), LinkError>;
    /// Links for a verse. `include_uncertain = false` (default) returns only
    /// owner-confirmed links — the authoritative display path.
    async fn get_links_for_verse(
        &self,
        surah: u16,
        ayah: u32,
        include_uncertain: bool,
    ) -> Result<Vec<VerseLink>, LinkError>;
}

/// The verse link service.
pub struct VerseLinkService {
    db: Arc<SqliteDatabase>,
}

impl VerseLinkService {
    /// Wrap a database handle.
    pub fn new(db: Arc<SqliteDatabase>) -> Self {
        Self { db }
    }

    /// Load the matchable source text: hadith `matn_text` (the isnad is a
    /// chain, not matchable content) or tafsir `text`.
    async fn source_text(
        &self,
        source_type: LinkSourceType,
        source_id: &str,
    ) -> Result<String, LinkError> {
        let text: Option<String> = match source_type {
            LinkSourceType::Hadith => {
                sqlx::query_scalar("SELECT matn_text FROM hadith_records WHERE id = ?")
                    .bind(source_id)
                    .fetch_optional(self.db.read_pool())
                    .await
                    .map_err(|e| {
                        LinkError::Storage(StorageError::ConstraintViolation {
                            message: e.to_string(),
                        })
                    })?
            }
            LinkSourceType::Tafsir => {
                sqlx::query_scalar("SELECT text FROM tafsir_records WHERE id = ?")
                    .bind(source_id)
                    .fetch_optional(self.db.read_pool())
                    .await
                    .map_err(|e| {
                        LinkError::Storage(StorageError::ConstraintViolation {
                            message: e.to_string(),
                        })
                    })?
            }
        };
        text.ok_or_else(|| LinkError::SourceNotFound(format!("{source_type}:{source_id}")))
    }

    /// Insert a link record.
    async fn insert_link(&self, link: &VerseLink) -> Result<(), LinkError> {
        sqlx::query(
            r#"INSERT INTO verse_links
               (id, source_type, source_id, target_surah, target_ayah, confidence,
                label, algorithm, algorithm_version, reviewer, decided_at, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(link.id.to_string())
        .bind(link.source_type.as_str())
        .bind(&link.source_id)
        .bind(link.target_surah as i64)
        .bind(link.target_ayah as i64)
        .bind(link.confidence)
        .bind(link.label.as_str())
        .bind(&link.algorithm)
        .bind(&link.algorithm_version)
        .bind(&link.reviewer)
        .bind(link.decided_at.map(|t| t.to_string()))
        .bind(link.created_at.to_string())
        .execute(self.db.write_pool())
        .await
        .map_err(|e| {
            LinkError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
        })?;
        Ok(())
    }

    /// Suggest links against every ayah of a Quran edition: loads the
    /// edition's verses from `quran_ayahs` and delegates to
    /// [`VerseLinker::suggest_links`].
    pub async fn suggest_links_for_edition(
        &self,
        source_type: LinkSourceType,
        source_id: &str,
        edition_id: &str,
    ) -> Result<Vec<VerseLink>, LinkError> {
        let rows = sqlx::query("SELECT surah, ayah, text FROM quran_ayahs WHERE edition_id = ?")
            .bind(edition_id)
            .fetch_all(self.db.read_pool())
            .await
            .map_err(|e| {
                LinkError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
            })?;
        let candidates: Vec<VerseCandidate> = rows
            .iter()
            .map(|row| VerseCandidate {
                surah: row.get::<i64, _>(0) as u16,
                ayah: row.get::<i64, _>(1) as u32,
                text: row.get(2),
            })
            .collect();
        self.suggest_links(source_type, source_id, &candidates).await
    }

    /// Decide a link (shared confirm/reject path): requires a non-empty
    /// reviewer, stamps `decided_at`, and refuses already-decided links.
    async fn decide(
        &self,
        link_id: &VerseLinkId,
        reviewer: &str,
        label: LinkLabel,
    ) -> Result<(), LinkError> {
        if reviewer.trim().is_empty() {
            return Err(LinkError::InvalidReviewer);
        }
        let decided_at = Timestamp::now().to_string();
        let result = sqlx::query(
            "UPDATE verse_links SET label = ?, reviewer = ?, decided_at = ?
             WHERE id = ? AND label = 'uncertain'",
        )
        .bind(label.as_str())
        .bind(reviewer.trim())
        .bind(&decided_at)
        .bind(link_id.to_string())
        .execute(self.db.write_pool())
        .await
        .map_err(|e| {
            LinkError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
        })?;
        if result.rows_affected() == 0 {
            // Either missing or already decided — distinguish for a typed error.
            let exists: Option<String> =
                sqlx::query_scalar("SELECT id FROM verse_links WHERE id = ?")
                    .bind(link_id.to_string())
                    .fetch_optional(self.db.read_pool())
                    .await
                    .map_err(|e| {
                        LinkError::Storage(StorageError::ConstraintViolation {
                            message: e.to_string(),
                        })
                    })?;
            if exists.is_some() {
                return Err(LinkError::AlreadyDecided(link_id.to_string()));
            }
            return Err(LinkError::LinkNotFound(link_id.to_string()));
        }
        Ok(())
    }
}

#[async_trait]
impl VerseLinker for VerseLinkService {
    async fn suggest_links(
        &self,
        source_type: LinkSourceType,
        source_id: &str,
        candidates: &[VerseCandidate],
    ) -> Result<Vec<VerseLink>, LinkError> {
        let text = self.source_text(source_type, source_id).await?;
        let now = Timestamp::now();
        let mut links = Vec::new();
        for candidate in candidates {
            let confidence = score_overlap(&text, &candidate.text)?;
            if confidence <= 0.0 {
                continue;
            }
            let link = VerseLink::suggested(
                source_type,
                source_id.to_string(),
                candidate.surah,
                candidate.ayah,
                confidence,
                LINK_ALGORITHM.to_string(),
                LINK_ALGORITHM_VERSION.to_string(),
                now,
            );
            self.insert_link(&link).await?;
            links.push(link);
        }
        Ok(links)
    }

    async fn confirm_link(&self, link_id: &VerseLinkId, reviewer: &str) -> Result<(), LinkError> {
        self.decide(link_id, reviewer, LinkLabel::Confirmed).await
    }

    async fn reject_link(&self, link_id: &VerseLinkId, reviewer: &str) -> Result<(), LinkError> {
        self.decide(link_id, reviewer, LinkLabel::Rejected).await
    }

    async fn get_links_for_verse(
        &self,
        surah: u16,
        ayah: u32,
        include_uncertain: bool,
    ) -> Result<Vec<VerseLink>, LinkError> {
        let sql = if include_uncertain {
            "SELECT id, source_type, source_id, target_surah, target_ayah, confidence,
                    label, algorithm, algorithm_version, reviewer, decided_at, created_at
             FROM verse_links WHERE target_surah = ? AND target_ayah = ?
             ORDER BY confidence DESC"
        } else {
            "SELECT id, source_type, source_id, target_surah, target_ayah, confidence,
                    label, algorithm, algorithm_version, reviewer, decided_at, created_at
             FROM verse_links WHERE target_surah = ? AND target_ayah = ?
               AND label = 'confirmed'
             ORDER BY confidence DESC"
        };
        let rows = sqlx::query(sql)
            .bind(surah as i64)
            .bind(ayah as i64)
            .fetch_all(self.db.read_pool())
            .await
            .map_err(|e| {
                LinkError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
            })?;
        rows.iter().map(row_to_link).collect()
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

#[cfg(test)]
mod service_tests {
    use super::*;
    use domain::{ContentHash, HashAlgorithm};
    use hadith_core::{HadithCollection, HadithCollectionId, HadithId, HadithRecord};
    use std::sync::Arc;
    use storage_sqlite::SqliteDatabase;
    use tafsir::{TafsirId, TafsirRecord, TafsirWork, TafsirWorkId};

    const SOURCE_VERSION_ID: &str = "018f3a2b-4c5d-7e6f-8a9b-0c1d2e3f4a5b";

    async fn test_db() -> Arc<SqliteDatabase> {
        // The TempDir must outlive the pool: dropping it deletes the database
        // file while open connections still hold it (SQLITE_IOERR).
        let dir = Box::leak(Box::new(tempfile::tempdir().unwrap()));
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
            include_str!("../../../migrations/sqlite/0023_hadith_collections.up.sql"),
            include_str!("../../../migrations/sqlite/0024_hadith_records.up.sql"),
            include_str!("../../../migrations/sqlite/0025_hadith_gradings.up.sql"),
            include_str!("../../../migrations/sqlite/0027_tafsir_works.up.sql"),
            include_str!("../../../migrations/sqlite/0028_tafsir_records.up.sql"),
            include_str!("../../../migrations/sqlite/0030_verse_links.up.sql"),
        ];
        for migration in migrations {
            sqlx::raw_sql(migration).execute(db.write_pool()).await.unwrap();
        }
        for sql in [
            "INSERT INTO sources (id, title, content_type, created_at, updated_at)
             VALUES ('src-test', 'Test source', 'hadith_collection',
                     '2026-01-15T00:00:00Z', '2026-01-15T00:00:00Z')",
            "INSERT INTO source_versions
                  (id, source_id, version, schema_version, state, trust_level,
                   license_status, license_json, created_at)
             VALUES ('018f3a2b-4c5d-7e6f-8a9b-0c1d2e3f4a5b', 'src-test', '0.1.0', 1, 'Staged', 'ImportedUnverified',
                     'PublicDomain', '{}', '2026-01-15T00:00:00Z')",
        ] {
            sqlx::query(sql).execute(db.write_pool()).await.unwrap();
        }
        Arc::new(db)
    }

    fn hash() -> ContentHash {
        ContentHash::try_new(HashAlgorithm::Sha256, "ab".repeat(32)).unwrap()
    }

    fn timestamp() -> Timestamp {
        Timestamp::from_ymd_hms(2026, 1, 15, 0, 0, 0).unwrap()
    }

    async fn seed_hadith(db: &Arc<SqliteDatabase>, matn: &str) -> HadithId {
        use crate::hadith_reader::{HadithReader, HadithReaderService};
        let reader = HadithReaderService::new(db.clone());
        let collection = HadithCollection {
            id: HadithCollectionId::new(),
            slug: "bukhari-arabic".to_string(),
            name: "Sahih al-Bukhari (Arabic)".to_string(),
            tradition: hadith_core::Tradition::Sunni,
            source_url: None,
            license_json: r#"{"status":"PublicDomain"}"#.to_string(),
            text_hash: hash(),
            structure_hash: hash(),
            version: domain::SemVer::new(1, 0, 0),
            source_version_id: SOURCE_VERSION_ID.parse().unwrap(),
            imported_at: timestamp(),
            status: hadith_core::RecordStatus::Active,
        };
        reader.insert_collection(&collection).await.unwrap();
        let record = HadithRecord {
            id: HadithId::new(),
            collection_id: collection.id,
            source_number: "bukhari:1234".to_string(),
            matn_text: matn.to_string(),
            matn_hash: hash(),
            isnad_text: None,
            narrator_names: vec![],
            source_version_id: SOURCE_VERSION_ID.parse().unwrap(),
            status: hadith_core::RecordStatus::Active,
            metadata_json: serde_json::json!({}),
            created_at: timestamp(),
        };
        reader.insert_record(&record).await.unwrap();
        record.id
    }

    async fn seed_tafsir(db: &Arc<SqliteDatabase>, text: &str) -> TafsirId {
        use crate::tafsir_reader::{TafsirReader, TafsirReaderService};
        let reader = TafsirReaderService::new(db.clone());
        let work = TafsirWork {
            id: TafsirWorkId::new(),
            slug: "tafsir-tabari".to_string(),
            name: "Jami' al-Bayan (Tabari)".to_string(),
            author: "Muhammad ibn Jarir al-Tabari".to_string(),
            dimensions_schema_json: r#"{"chapter":"surah-number"}"#.to_string(),
            version: domain::SemVer::new(1, 0, 0),
            source_version_id: SOURCE_VERSION_ID.parse().unwrap(),
            created_at: timestamp(),
            status: tafsir::WorkStatus::Active,
        };
        reader.insert_work(&work).await.unwrap();
        let record = TafsirRecord {
            id: TafsirId::new(),
            work_id: work.id,
            volume: Some(3),
            page: Some(12),
            chapter: Some("2".to_string()),
            text: text.to_string(),
            text_hash: hash(),
            source_version_id: SOURCE_VERSION_ID.parse().unwrap(),
            status: tafsir::WorkStatus::Active,
            created_at: timestamp(),
        };
        reader.insert_record(&record).await.unwrap();
        record.id
    }

    #[test]
    fn overlap_scoring_is_deterministic_and_bounded() {
        let a = score_overlap("إنما الأعمال بالنيات", "إنما الأعمال بالنيات").unwrap();
        assert_eq!(a, 1.0);
        let b = score_overlap("إنما الأعمال بالنيات", "قال رسول الله").unwrap();
        assert_eq!(b, 0.0);
        let c = score_overlap("إنما الأعمال بالنيات", "إنما الأعمال").unwrap();
        assert!(c > 0.0 && c < 1.0, "partial overlap scores strictly inside (0, 1)");
        // Empty source is no evidence, never a link.
        assert_eq!(score_overlap("", "إنما الأعمال").unwrap(), 0.0);
        assert_eq!(score_overlap("   ", "إنما الأعمال").unwrap(), 0.0);
    }

    #[tokio::test]
    async fn suggest_links_creates_uncertain_links_with_scores() {
        let db = test_db().await;
        let service = VerseLinkService::new(db.clone());
        let hadith_id = seed_hadith(&db, "إنما الأعمال بالنيات").await;

        let candidates = vec![
            VerseCandidate {
                surah: 2, ayah: 255, text: "إنما الأعمال بالنيات".to_string()
            },
            VerseCandidate { surah: 1, ayah: 1, text: "قال رسول الله".to_string() },
        ];
        let links = service
            .suggest_links(LinkSourceType::Hadith, &hadith_id.to_string(), &candidates)
            .await
            .unwrap();
        // Only the overlapping candidate becomes a link; zero overlap is no evidence.
        assert_eq!(links.len(), 1);
        let link = &links[0];
        assert_eq!(link.target_surah, 2);
        assert_eq!(link.target_ayah, 255);
        assert_eq!(link.label, LinkLabel::Uncertain);
        assert!(!link.is_authoritative());
        assert!(link.confidence > 0.0 && link.confidence <= 1.0);
        assert_eq!(link.algorithm, LINK_ALGORITHM);
        assert_eq!(link.algorithm_version, LINK_ALGORITHM_VERSION);
        assert!(link.reviewer.is_none() && link.decided_at.is_none());
    }

    #[tokio::test]
    async fn suggest_links_works_for_tafsir_sources() {
        let db = test_db().await;
        let service = VerseLinkService::new(db.clone());
        let tafsir_id = seed_tafsir(&db, "القول في تأويل قوله تعالى").await;

        let candidates =
            vec![VerseCandidate {
                surah: 2, ayah: 255, text: "القول في تأويل".to_string()
            }];
        let links = service
            .suggest_links(LinkSourceType::Tafsir, &tafsir_id.to_string(), &candidates)
            .await
            .unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].source_type, LinkSourceType::Tafsir);
        assert_eq!(links[0].label, LinkLabel::Uncertain);
    }

    #[tokio::test]
    async fn suggest_links_for_missing_source_is_not_found() {
        let db = test_db().await;
        let service = VerseLinkService::new(db);
        let err = service
            .suggest_links(
                LinkSourceType::Hadith,
                "00000000-0000-4000-8000-000000000000",
                &[VerseCandidate { surah: 2, ayah: 255, text: "نص".to_string() }],
            )
            .await
            .unwrap_err();
        assert!(matches!(err, LinkError::SourceNotFound(_)));
    }

    #[tokio::test]
    async fn owner_gate_confirms_and_rejects_with_audit_trail() {
        let db = test_db().await;
        let service = VerseLinkService::new(db.clone());
        let hadith_id = seed_hadith(&db, "إنما الأعمال بالنيات").await;
        let candidates =
            vec![VerseCandidate {
                surah: 2, ayah: 255, text: "إنما الأعمال بالنيات".to_string()
            }];
        let links = service
            .suggest_links(LinkSourceType::Hadith, &hadith_id.to_string(), &candidates)
            .await
            .unwrap();
        let link_id = links[0].id;

        // Blank reviewer is rejected at the gate, not the database.
        let err = service.confirm_link(&link_id, "   ").await.unwrap_err();
        assert!(matches!(err, LinkError::InvalidReviewer));

        service.confirm_link(&link_id, "owner").await.unwrap();
        let confirmed = service.get_links_for_verse(2, 255, false).await.unwrap();
        assert_eq!(confirmed.len(), 1);
        assert_eq!(confirmed[0].label, LinkLabel::Confirmed);
        assert!(confirmed[0].is_authoritative());
        assert_eq!(confirmed[0].reviewer.as_deref(), Some("owner"));
        assert!(confirmed[0].decided_at.is_some());

        // A decided link cannot be decided again.
        let err = service.reject_link(&link_id, "owner").await.unwrap_err();
        assert!(matches!(err, LinkError::AlreadyDecided(_)));

        // Missing links are typed, not silent.
        let err = service.confirm_link(&VerseLinkId::new(), "owner").await.unwrap_err();
        assert!(matches!(err, LinkError::LinkNotFound(_)));
    }

    #[tokio::test]
    async fn read_path_hides_uncertain_links_by_default() {
        let db = test_db().await;
        let service = VerseLinkService::new(db.clone());
        let hadith_id = seed_hadith(&db, "إنما الأعمال بالنيات").await;
        let candidates =
            vec![VerseCandidate {
                surah: 2, ayah: 255, text: "إنما الأعمال بالنيات".to_string()
            }];
        service
            .suggest_links(LinkSourceType::Hadith, &hadith_id.to_string(), &candidates)
            .await
            .unwrap();

        // Authoritative display: uncertain links are hidden.
        assert!(service.get_links_for_verse(2, 255, false).await.unwrap().is_empty());
        // Review display: all links visible with their labels.
        let all = service.get_links_for_verse(2, 255, true).await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].label, LinkLabel::Uncertain);

        // After rejection the link stays hidden from the authoritative path.
        service.reject_link(&all[0].id, "owner").await.unwrap();
        assert!(service.get_links_for_verse(2, 255, false).await.unwrap().is_empty());
        let all = service.get_links_for_verse(2, 255, true).await.unwrap();
        assert_eq!(all[0].label, LinkLabel::Rejected);
    }

    #[tokio::test]
    async fn edition_path_with_no_verses_returns_empty() {
        let db = test_db().await;
        let service = VerseLinkService::new(db.clone());
        let hadith_id = seed_hadith(&db, "إنما الأعمال بالنيات").await;
        // No quran_ayahs rows exist: valid SQL, empty result, no error.
        let links = service
            .suggest_links_for_edition(
                LinkSourceType::Hadith,
                &hadith_id.to_string(),
                "no-such-edition",
            )
            .await
            .unwrap();
        assert!(links.is_empty());
    }

    #[tokio::test]
    async fn database_check_rejects_confirmed_without_reviewer() {
        let db = test_db().await;
        // The owner-gate CHECK fires at the database level, below the service.
        let err = sqlx::query(
            r#"INSERT INTO verse_links
               (id, source_type, source_id, target_surah, target_ayah, confidence,
                label, algorithm, algorithm_version, reviewer, decided_at, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(VerseLinkId::new().to_string())
        .bind("hadith")
        .bind("hadith-id-1")
        .bind(2i64)
        .bind(255i64)
        .bind(0.95)
        .bind("confirmed")
        .bind(LINK_ALGORITHM)
        .bind(LINK_ALGORITHM_VERSION)
        .bind(Option::<String>::None)
        .bind(Option::<String>::None)
        .bind(Timestamp::now().to_string())
        .execute(db.write_pool())
        .await
        .unwrap_err();
        assert!(err.to_string().contains("CHECK"), "gate CHECK must fire, got: {err}");
    }
}
