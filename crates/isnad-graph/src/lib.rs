//! Isnad & narrator research graph (Phase 7).
//!
//! Composition over duplication: this crate re-exports the shared graph
//! vocabulary from `quran-graph` (NodeKind, AssertionKind, Assertion,
//! AssertionDecision, ProvenanceLayer, GraphEdge, GraphNode, QueryBudgets,
//! AuthzScope, traversal) and adds narrator/isnad-specific domain types.
//! No new external crates — only existing workspace dependencies.

pub use quran_graph::{
    Assertion, AssertionDecision, AssertionKind, AuthzScope, GraphEdge, GraphNode, NodeKind,
    ProvenanceLayer, QueryBudgets,
};
pub use quran_graph::traverse;
pub use quran_graph::traverse::up_to_k_paths;

use serde::{Deserialize, Serialize};
use std::fmt;

/// Stable narrator identifier in `narrator:{slug}` form.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NarratorId(String);

impl NarratorId {
    pub fn new(slug: &str) -> Self {
        Self(format!("narrator:{}", slug))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn slug(&self) -> &str {
        self.0.strip_prefix("narrator:").unwrap_or(&self.0)
    }
}

impl fmt::Display for NarratorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for NarratorId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("narrator:") || s.len() <= "narrator:".len() {
            return Err(format!("invalid narrator id `{s}`"));
        }
        Ok(Self(s.to_string()))
    }
}

/// A narrator record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NarratorRecord {
    pub id: NarratorId,
    pub slug: String,
    pub primary_name: String,
    #[serde(default)]
    pub name_variants: Vec<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub created_at: String,
}

/// A per-scholar rijal source record. No consensus field (D-03 pattern).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RijalSource {
    pub id: String,
    pub narrator_id: NarratorId,
    pub scholar_name: String,
    pub source_citation: String,
    pub grading_value: Option<String>,
    #[serde(default)]
    pub evidence: serde_json::Value,
    pub reviewer: Option<String>,
    pub decision: String,
    pub decided_at: Option<String>,
    pub supersedes_id: Option<String>,
    pub provenance_layer: String,
    pub confidence: Option<f64>,
    pub created_at: String,
}

/// Label for an isnad transmission link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IsnadLinkLabel {
    Uncertain,
    Confirmed,
    Rejected,
}

impl IsnadLinkLabel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Uncertain => "uncertain",
            Self::Confirmed => "confirmed",
            Self::Rejected => "rejected",
        }
    }
}

/// Display band for a confidence score (D-07 thresholds, inclusive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceBand {
    High,
    Medium,
    Low,
}

impl ConfidenceBand {
    /// high = confidence > 0.9; medium = 0.7 <= confidence <= 0.9; low = < 0.7.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn narrator_id_stable_form() {
        let id = NarratorId::new("abu-hurayra");
        assert_eq!(id.to_string(), "narrator:abu-hurayra");
        assert_eq!(id.slug(), "abu-hurayra");
        let parsed: NarratorId = "narrator:abu-hurayra".parse().unwrap();
        assert_eq!(parsed, id);
    }

    #[test]
    fn narrator_roundtrip_serde() {
        let rec = NarratorRecord {
            id: NarratorId::new("x"),
            slug: "x".into(),
            primary_name: "Name".into(),
            name_variants: vec!["N".into()],
            metadata: serde_json::json!({}),
            created_at: "2026-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&rec).unwrap();
        let back: NarratorRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back, rec);
    }

    #[test]
    fn rijal_source_per_scholar_no_consensus() {
        let rs = RijalSource {
            id: "r1".into(),
            narrator_id: NarratorId::new("x"),
            scholar_name: "Ibn Hajar".into(),
            source_citation: "Tahdhib".into(),
            grading_value: Some("thiqa".into()),
            evidence: serde_json::json!({}),
            reviewer: None,
            decision: "pending".into(),
            decided_at: None,
            supersedes_id: None,
            provenance_layer: "B".into(),
            confidence: None,
            created_at: "2026-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&rs).unwrap();
        assert!(!json.contains("consensus"));
    }

    #[test]
    fn confidence_bands_thresholds() {
        assert_eq!(ConfidenceBand::of(0.95), ConfidenceBand::High);
        assert_eq!(ConfidenceBand::of(0.9), ConfidenceBand::Medium);
        assert_eq!(ConfidenceBand::of(0.7), ConfidenceBand::Medium);
        assert_eq!(ConfidenceBand::of(0.69), ConfidenceBand::Low);
    }
}
