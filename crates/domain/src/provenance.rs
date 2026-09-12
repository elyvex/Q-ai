//! Phase 0 — provenance and derivation tracking.
//!
//! # domain::provenance
//!
//! Tracks the origin and evolution of canonical data.

use crate::ids::SourceVersionId;
use crate::primitives::SemVer;
use serde::{Deserialize, Serialize};

/// A URN-based reference to a subject within the system.
/// Format: `urn:qai:<domain>:<type>:<id>`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectRef(pub String);

/// Records every version that a derived artifact depends on (PRD §76, §82).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivationVersions {
    pub source_version_id: SourceVersionId,
    pub parser_version: SemVer,
    pub normalizer_version: SemVer,
    pub chunker_version: Option<SemVer>,
    pub embedding_model_version: Option<String>,
    pub graph_builder_version: Option<SemVer>,
    /// Placeholder in Phase 0; Phase 1/2 populate it. Exists now so the type is
    /// not reworked later (D0.18, ADR-0702 §2).
    pub dependency_snapshot_id: Option<String>, // Placeholder until SnapshotId is defined
    pub schema_version: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subject_ref_format() {
        let urn = SubjectRef("urn:qai:quran:ayah:1:1".to_string());
        assert!(urn.0.starts_with("urn:qai:"));
    }
}
