//! Phase 0 — Source catalog, manifest schema, and state machine (D0.10).

use domain::{
    ApprovalId, ContentHash, DerivationVersions, PrincipalId, SourceId,
    SourceVersionId, Timestamp, TrustLevel, LicenseStatus, HashAlgorithm, canonical_json_bytes,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;

// ─── Source ────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub id: SourceId,
    pub title: String,
    pub alternate_titles: Vec<String>,
    pub content_type: SourceContentType,
    pub authors: Vec<String>,
    pub compiler: Option<String>,
    pub translator: Option<String>,
    pub editor: Option<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
    pub tradition: Option<String>,
    pub school: Option<String>,
    pub identifiers: SourceIdentifiers,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceContentType {
    QuranEdition, Translation, Tafsir, HadithCollection, Scripture, Book, Dataset,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SourceIdentifiers {
    pub isbn: Option<String>,
    pub oclc: Option<String>,
    pub other: BTreeMap<String, String>,
}

// ─── SourceVersion ─────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceVersion {
    pub id: SourceVersionId,
    pub source_id: SourceId,
    pub version: String,
    pub schema_version: u32,
    pub state: SourceState,
    pub trust_level: TrustLevel,
    pub license_status: LicenseStatus,
    pub license_json: String,
    pub manifest_blob_id: Option<String>,
    pub manifest_hash: Option<ContentHash>,
    pub content_hash: Option<ContentHash>,
    pub source_urls: Vec<String>,
    pub publication_date: Option<Timestamp>,
    pub imported_at: Option<Timestamp>,
    pub validated_at: Option<Timestamp>,
    pub approved_at: Option<Timestamp>,
    pub approved_by: Option<PrincipalId>,
    pub activated_at: Option<Timestamp>,
    pub deprecated_at: Option<Timestamp>,
    pub quarantine_reason: Option<String>,
    pub validation_report: Option<String>,
    pub notes: Option<String>,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceState {
    Discovered, PendingReview, Downloading, Downloaded, Validating,
    ValidationFailed, Staged, Approved, Indexing, Active, Deprecated, Quarantined, Removed,
}

// ─── SourceFile ────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFile {
    pub id: String,
    pub source_version_id: SourceVersionId,
    pub role: FileRole,
    pub relative_path: String,
    pub format: String,
    pub media_type: Option<String>,
    pub bytes: Option<u64>,
    pub declared_hash: ContentHash,
    pub observed_hash: Option<ContentHash>,
    pub encoding: Option<String>,
    pub unicode_form: Option<String>,
    pub blob_id: Option<String>,
    pub verified_at: Option<Timestamp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileRole { Primary, Metadata, Scan, Audio, Aux }

// ─── SourceGenealogy ────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceGenealogy {
    pub child_source_id: SourceId,
    pub parent_source_id: SourceId,
    pub derivation_type: DerivationType,
    pub derivation_language: Option<String>,
    pub derivation_date: Option<Timestamp>,
    pub derivation_note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DerivationType { Translation, Summary, Edition, Abridgment, Commentary, Original }

// ─── SourceStateTransition ──────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceStateTransition {
    pub id: String,
    pub source_version_id: SourceVersionId,
    pub from_state: Option<SourceState>,
    pub to_state: SourceState,
    pub actor_id: Option<PrincipalId>,
    pub reason: Option<String>,
    pub occurred_at: Timestamp,
}

// ─── ApprovalRecord ─────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRecord {
    pub id: ApprovalId,
    pub subject_urn: String,
    pub kind: ApprovalKind,
    pub requested_by: Option<PrincipalId>,
    pub decided_by: Option<PrincipalId>,
    pub decision: ApprovalDecision,
    pub request_payload: String,
    pub decision_note: Option<String>,
    pub requested_at: Timestamp,
    pub decided_at: Option<Timestamp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalKind { SourceActivation, CanonicalChange, Rollback }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalDecision { Approved, Denied }

// ─── StructureValidator ─────────────────────────

#[async_trait::async_trait]
pub trait StructureValidator: Send + Sync {
    async fn validate(&self, version: &SourceVersion) -> Result<ValidationReport, ValidationError>;
    fn name(&self) -> &str;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    pub valid: bool,
    pub validator_name: String,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub location: Option<String>,
}

// ─── DifferenceReport ───────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DifferenceReport {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<String>,
    pub unchanged: Vec<String>,
}

// ─── ManifestParser ─────────────────────────────

pub struct ManifestParser { allow_unsigned: bool }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestParseResult {
    pub manifest_version: String,
    pub catalog_version: String,
    pub generated_at: Timestamp,
    pub sources: Vec<SourceVersion>,
}

impl ManifestParser {
    pub fn new(allow_unsigned: bool) -> Self { Self { allow_unsigned } }
    pub fn parse(&self, json: &str) -> Result<ManifestParseResult, SourceError> {
        serde_json::from_str(json).map_err(SourceError::Serialization)
    }
    pub fn validate_schema(&self, manifest: &ManifestParseResult) -> Result<(), SourceError> {
        if manifest.manifest_version.is_empty() {
            return Err(SourceError::ManifestValidationError("manifest_version is required".to_string()));
        }
        Ok(())
    }
    pub fn validate_semantic(&self, manifest: &ManifestParseResult) -> Result<(), SourceError> {
        for source in &manifest.sources {
            if source.version.is_empty() {
                return Err(SourceError::ManifestValidationError(format!("source {} has empty version", source.id)));
            }
        }
        Ok(())
    }
    pub fn canonical_reserialize(&self, manifest: &ManifestParseResult) -> Result<String, SourceError> {
        let bytes = canonical_json_bytes(manifest).map_err(SourceError::Serialization)?;
        Ok(String::from_utf8(bytes).map_err(|_| SourceError::Serialization("invalid UTF-8".to_string()))?)
    }
    pub fn verify_signature(&self, _manifest: &ManifestParseResult, signature: Option<&str>) -> Result<(), SourceError> {
        if signature.is_none() && !self.allow_unsigned {
            return Err(SourceError::SignatureVerificationFailed("unsigned manifests not allowed by policy".to_string()));
        }
        Ok(())
    }
}

fn verify_ed25519(_manifest: &ManifestParseResult, _signature: &str) -> Result<(), String> {
    // ed25519-dalek is not a workspace dep; stub implementation
    Ok(())
}

// ─── GenealogyResolver ──────────────────────────

pub struct GenealogyResolver {
    graph: Arc<TokioRwLock<BTreeMap<SourceId, Vec<SourceId>>>>,
}

impl GenealogyResolver {
    pub fn new() -> Self {
        Self { graph: Arc::new(TokioRwLock::new(BTreeMap::new())) }
    }
    pub async fn add_relationship(&self, child: SourceId, parent: SourceId) -> Result<(), SourceError> {
        if child == parent {
            return Err(SourceError::GenealogyCycleDetected("self-referencing genealogy".to_string()));
        }
        self.graph.write().await.insert(child, vec![parent]);
        Ok(())
    }
    pub async fn resolve_lineage(&self, source_id: &SourceId) -> Result<GenealogyResult, SourceError> {
        let graph = self.graph.read().await;
        let mut visited = BTreeMap::new();
        let mut lineage = Vec::new();
        let mut cycle_detected = false;
        self.dfs_resolve(source_id, &graph, &mut visited, &mut lineage, &mut cycle_detected)?;
        Ok(GenealogyResult { lineage, cycle_detected, rendering: self.render_lineage(&lineage) })
    }
    fn dfs_resolve(&self, current: &SourceId, graph: &BTreeMap<SourceId, Vec<SourceId>>, visited: &mut BTreeMap<SourceId, bool>, lineage: &mut Vec<LineageNode>, cycle_detected: &mut bool) -> Result<(), SourceError> {
        if visited.get(current) == Some(&true) { *cycle_detected = true; return Err(SourceError::GenealogyCycleDetected(format!("cycle detected at {}", current))); }
        visited.insert(current.clone(), true);
        if let Some(parents) = graph.get(current) {
            for parent in parents {
                lineage.push(LineageNode { source_id: parent.clone(), relationship: DerivationType::Original });
                self.dfs_resolve(parent, graph, visited, lineage, cycle_detected)?;
            }
        }
        Ok(())
    }
    fn render_lineage(&self, nodes: &[LineageNode]) -> String {
        if nodes.is_empty() { "original (no parents)".to_string() } else { nodes.iter().map(|n| format!("{} (derived from {})", n.source_id, n.relationship)).collect::<Vec<_>>().join(", ") }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineageNode { pub source_id: SourceId, pub relationship: DerivationType }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenealogyResult { pub lineage: Vec<LineageNode>, pub cycle_detected: bool, pub rendering: String }

// ─── StateMachine ────────────────────────────────

pub struct StateMachine;

impl StateMachine {
    pub fn is_legal_transition(from: &SourceState, to: &SourceState) -> bool {
        match (from, to) {
            (SourceState::Discovered, SourceState::PendingReview) |
            (SourceState::Discovered, SourceState::Quarantined) |
            (SourceState::PendingReview, SourceState::Downloading) |
            (SourceState::PendingReview, SourceState::Quarantined) |
            (SourceState::Downloading, SourceState::Downloaded) |
            (SourceState::Downloading, SourceState::Quarantined) |
            (SourceState::Downloaded, SourceState::Validating) |
            (SourceState::Downloaded, SourceState::Quarantined) |
            (SourceState::Validating, SourceState::Staged) |
            (SourceState::Validating, SourceState::ValidationFailed) |
            (SourceState::ValidationFailed, SourceState::Quarantined) |
            (SourceState::Staged, SourceState::Approved) |
            (SourceState::Staged, SourceState::Quarantined) |
            (SourceState::Approved, SourceState::Indexing) |
            (SourceState::Indexing, SourceState::Active) |
            (SourceState::Indexing, SourceState::ValidationFailed) |
            (SourceState::Active, SourceState::Deprecated) |
            (SourceState::Deprecated, SourceState::Removed) |
            (_, SourceState::Quarantined) => true,
            _ => false,
        }
    }
    pub fn transition(version: &mut SourceVersion, new_state: SourceState, actor: Option<PrincipalId>, reason: Option<String>) -> Result<(), SourceError> {
        if !Self::is_legal_transition(&version.state, &new_state) {
            return Err(SourceError::IllegalStateTransition { from: format!("{:?}", version.state), to: format!("{:?}", new_state) });
        }
        if new_state == SourceState::Approved { Self::check_approved_preconditions(version)?; }
        version.state = new_state;
        Ok(())
    }
    fn check_approved_preconditions(version: &SourceVersion) -> Result<(), SourceError> {
        if version.license_status == LicenseStatus::Unknown {
            return Err(SourceError::ApprovalPreconditionNotMet("license status must not be Unknown".to_string()));
        }
        if version.content_hash.is_none() {
            return Err(SourceError::ApprovalPreconditionNotMet("content hash is required".to_string()));
        }
        if version.validation_report.is_none() {
            return Err(SourceError::ApprovalPreconditionNotMet("validation report is required".to_string()));
        }
        Ok(())
    }
}

// ─── SourceRepository ──────────────────────────

#[async_trait::async_trait]
pub trait SourceRepository: Send + Sync {
    async fn get_source(&self, id: &SourceId) -> Result<Option<Source>, SourceError>;
    async fn insert_source(&mut self, source: Source) -> Result<(), SourceError>;
    async fn get_version(&self, id: &SourceVersionId) -> Result<Option<SourceVersion>, SourceError>;
    async fn insert_version(&mut self, version: SourceVersion) -> Result<(), SourceError>;
    async fn list_versions(&self, source_id: &SourceId) -> Result<Vec<SourceVersion>, SourceError>;
    async fn transition_state(&mut self, version_id: &SourceVersionId, from: &SourceState, to: SourceState, actor: Option<PrincipalId>, reason: Option<String>) -> Result<(), SourceError>;
    async fn record_transition(&mut self, transition: SourceStateTransition) -> Result<(), SourceError>;
    async fn list_active_versions(&self, source_id: &SourceId) -> Result<Vec<SourceVersion>, SourceError>;
    async fn insert_genealogy(&mut self, genealogy: SourceGenealogy) -> Result<(), SourceError>;
    async fn get_approval(&self, id: &ApprovalId) -> Result<Option<ApprovalRecord>, SourceError>;
    async fn insert_approval(&mut self, approval: ApprovalRecord) -> Result<(), SourceError>;
}

// ─── SourceError ────────────────────────────────

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SourceError {
    #[error("source not found: {id}")]
    NotFound { id: String },
    #[error("version not found: {id}")]
    VersionNotFound { id: String },
    #[error("illegal state transition: {from} → {to}")]
    IllegalStateTransition { from: String, to: String },
    #[error("approval preconditions not met: {0}")]
    ApprovalPreconditionNotMet(String),
    #[error("manifest validation error: {0}")]
    ManifestValidationError(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("signature verification failed: {0}")]
    SignatureVerificationFailed(String),
    #[error("genealogy cycle detected: {0}")]
    GenealogyCycleDetected(String),
    #[error("multiple active versions for source {0}")]
    MultipleActiveVersions(String),
    #[error("source is already in state {0:?}")]
    AlreadyInState(SourceState),
    #[error("storage error: {0}")]
    Storage(#[from] storage::StorageError),
}

// ─── Tests ──────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{SourceId, SourceVersionId};

    #[test]
    fn legal_transitions() {
        assert!(StateMachine::is_legal_transition(&SourceState::Discovered, &SourceState::PendingReview));
        assert!(StateMachine::is_legal_transition(&SourceState::Staged, &SourceState::Approved));
        assert!(StateMachine::is_legal_transition(&SourceState::Approved, &SourceState::Indexing));
        assert!(StateMachine::is_legal_transition(&SourceState::Active, &SourceState::Deprecated));
    }
    #[test]
    fn illegal_transitions() {
        assert!(!StateMachine::is_legal_transition(&SourceState::Active, &SourceState::Staged));
        assert!(!StateMachine::is_legal_transition(&SourceState::Discovered, &SourceState::Active));
    }
    #[test]
    fn quarantine_from_any_state() {
        assert!(StateMachine::is_legal_transition(&SourceState::Active, &SourceState::Quarantined));
        assert!(StateMachine::is_legal_transition(&SourceState::Approved, &SourceState::Quarantined));
    }
    #[test]
    fn approved_requires_content_hash() {
        let mut version = SourceVersion {
            id: SourceVersionId::new(), source_id: SourceId::new(), version: "1.0.0".to_string(),
            schema_version: 1, state: SourceState::Staged, trust_level: TrustLevel::ImportedUnverified,
            license_status: crate::domain::LicenseStatus::OpenLicense, license_json: "{}".to_string(),
            manifest_blob_id: None, manifest_hash: None, content_hash: None, source_urls: vec![],
            publication_date: None, imported_at: None, validated_at: None, approved_at: None,
            approved_by: None, activated_at: None, deprecated_at: None, quarantine_reason: None,
            validation_report: None, notes: None, created_at: Timestamp::now(),
        };
        let result = StateMachine::transition(&mut version, SourceState::Approved, None, None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SourceError::ApprovalPreconditionNotMet(_)));
    }
    #[test]
    fn approved_requires_validation_report() {
        let mut version = SourceVersion {
            id: SourceVersionId::new(), source_id: SourceId::new(), version: "1.0.0".to_string(),
            schema_version: 1, state: SourceState::Staged, trust_level: TrustLevel::ImportedUnverified,
            license_status: crate::domain::LicenseStatus::OpenLicense, license_json: "{}".to_string(),
            manifest_blob_id: None, manifest_hash: None,
            content_hash: Some(ContentHash { algorithm: HashAlgorithm::Sha256, hex: "00".repeat(32) }),
            source_urls: vec![], publication_date: None, imported_at: None, validated_at: None,
            approved_at: None, approved_by: None, activated_at: None, deprecated_at: None,
            quarantine_reason: None, validation_report: None, notes: None, created_at: Timestamp::now(),
        };
        let result = StateMachine::transition(&mut version, SourceState::Approved, None, None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SourceError::ApprovalPreconditionNotMet(_)));
    }
    #[test]
    fn manifest_parser_roundtrip() {
        let parser = ManifestParser::new(false);
        let manifest = ManifestParseResult { manifest_version: "1.0.0".to_string(), catalog_version: "1.4.0".to_string(), generated_at: Timestamp::now(), sources: vec![] };
        let json = serde_json::to_string(&manifest).unwrap();
        let parsed = parser.parse(&json).unwrap();
        assert_eq!(parsed.manifest_version, "1.0.0");
    }
}
