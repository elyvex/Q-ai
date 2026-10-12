//! Per-scholar grading records.
//!
//! Follows the graph_assertions pattern (Phase 4): independent records with
//! attribution, evidence, and review decision. No consensus field exists (D-03).
//! Each grading is attributed to its scholar with a source citation.

use domain::Timestamp;
use serde::{Deserialize, Serialize};

use crate::record::HadithId;

/// Identifies a grading record.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct GradingId(uuid::Uuid);

impl GradingId {
    /// Generate a fresh random (v4) ID.
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// The inner `Uuid`, for (de)serialization and storage interop.
    pub const fn inner(&self) -> uuid::Uuid {
        self.0
    }
}

impl Default for GradingId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for GradingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for GradingId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = uuid::Uuid::parse_str(s).map_err(|e| e.to_string())?;
        Ok(Self(id))
    }
}

impl From<GradingId> for uuid::Uuid {
    fn from(id: GradingId) -> Self {
        id.0
    }
}

/// Review decision for a grading record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDecision {
    /// Pending review.
    Pending,
    /// Accepted by a reviewer.
    Accepted,
    /// Rejected by a reviewer.
    Rejected,
    /// Superseded by a newer grading.
    Superseded,
}

impl ReviewDecision {
    /// The string form stored in the database.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
        }
    }
}

impl std::fmt::Display for ReviewDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ReviewDecision {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            "superseded" => Ok(Self::Superseded),
            other => Err(format!("unknown review decision `{other}`")),
        }
    }
}

/// Provenance layer for a grading record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ProvenanceLayer {
    /// Layer B: publisher-provided.
    B,
    /// Layer C: scholarly assertion.
    C,
    /// Layer D: computational suggestion.
    D,
}

impl ProvenanceLayer {
    /// The string form stored in the database.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
        }
    }
}

impl std::fmt::Display for ProvenanceLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ProvenanceLayer {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "B" => Ok(Self::B),
            "C" => Ok(Self::C),
            "D" => Ok(Self::D),
            other => Err(format!("unknown provenance layer `{other}`")),
        }
    }
}

/// A per-scholar grading record.
///
/// Each grading is an independent record: scholar name, grading value, source
/// citation. Multiple scholarly gradings for the same hadith are stored as
/// independent records — never merged into a consensus (D-03).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GradingRecord {
    /// Stable grading identity.
    pub id: GradingId,
    /// The hadith this grading belongs to.
    pub hadith_id: HadithId,
    /// Scholar name (non-empty, attributed).
    pub scholar_name: String,
    /// Grading value (free text: "sahih", "hasan", "da'if", etc.).
    pub grading_value: String,
    /// Where this grading was published.
    pub source_citation: String,
    /// Supporting evidence as JSON.
    pub evidence_json: serde_json::Value,
    /// Reviewer who made the decision, if any.
    pub reviewer: Option<String>,
    /// Review decision.
    pub decision: ReviewDecision,
    /// When the decision was made, if any.
    pub decided_at: Option<Timestamp>,
    /// The grading this record supersedes, if any.
    pub supersedes_id: Option<GradingId>,
    /// Provenance layer.
    pub provenance_layer: ProvenanceLayer,
    /// Confidence score, for computational suggestions.
    pub confidence: Option<f64>,
    /// Creation timestamp.
    pub created_at: Timestamp,
}

impl GradingRecord {
    /// Whether this grading has been reviewed (accepted, rejected, or superseded).
    pub fn is_reviewed(&self) -> bool {
        !matches!(self.decision, ReviewDecision::Pending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_grading() -> GradingRecord {
        GradingRecord {
            id: GradingId::new(),
            hadith_id: HadithId::new(),
            scholar_name: "Al-Albani".to_string(),
            grading_value: "sahih".to_string(),
            source_citation: "Silsilat al-Ahadith al-Sahihah, hadith 1234".to_string(),
            evidence_json: serde_json::json!({ "chain_analysis": "..." }),
            reviewer: None,
            decision: ReviewDecision::Pending,
            decided_at: None,
            supersedes_id: None,
            provenance_layer: ProvenanceLayer::C,
            confidence: None,
            created_at: Timestamp::from_ymd_hms(2026, 1, 15, 0, 0, 0).unwrap(),
        }
    }

    #[test]
    fn grading_roundtrips_serde() {
        let grading = sample_grading();
        let json = serde_json::to_string(&grading).unwrap();
        assert!(json.contains("\"scholar_name\":\"Al-Albani\""));
        assert!(json.contains("\"grading_value\":\"sahih\""));
        assert!(json.contains("\"decision\":\"pending\""));
        let back: GradingRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back, grading);
    }

    #[test]
    fn reviewed_gradings_are_not_pending() {
        let mut grading = sample_grading();
        assert!(!grading.is_reviewed());
        grading.decision = ReviewDecision::Accepted;
        grading.reviewer = Some("owner".to_string());
        grading.decided_at = Some(Timestamp::from_ymd_hms(2026, 1, 16, 0, 0, 0).unwrap());
        assert!(grading.is_reviewed());
    }

    #[test]
    fn decision_roundtrips() {
        for decision in [
            ReviewDecision::Pending,
            ReviewDecision::Accepted,
            ReviewDecision::Rejected,
            ReviewDecision::Superseded,
        ] {
            let json = serde_json::to_string(&decision).unwrap();
            let back: ReviewDecision = serde_json::from_str(&json).unwrap();
            assert_eq!(back, decision);
        }
    }

    #[test]
    fn provenance_layer_roundtrips() {
        for layer in [ProvenanceLayer::B, ProvenanceLayer::C, ProvenanceLayer::D] {
            let json = serde_json::to_string(&layer).unwrap();
            let back: ProvenanceLayer = serde_json::from_str(&json).unwrap();
            assert_eq!(back, layer);
        }
    }
}
