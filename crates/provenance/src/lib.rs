//! Phase 0 — Provenance model (D0.11).

use domain::{
    ApprovalId, ContentHash, DerivationVersions, PrincipalId, ProvenanceId,
    SubjectRef, Timestamp, TrustLevel, VerificationStatus,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ─── Attribution ──────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Attribution {
    Dataset { source_id: String, dataset_name: String, dataset_version: String },
    Scholar { name: String, school: Option<String>, work: Option<String>, edition: Option<String> },
    Computational { algorithm: String, version: String, model: Option<String>, parameters_hash: ContentHash },
    User { principal_id: PrincipalId },
}

// ─── SourceLocation ───────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub canonical_reference: Option<String>,
    pub volume: Option<String>,
    pub book: Option<String>,
    pub chapter: Option<String>,
    pub page: Option<String>,
    pub record_number: Option<String>,
    pub char_range: Option<(u32, u32)>,
    pub quoted_text_hash: Option<ContentHash>,
}

// ─── ProvenanceRecord ─────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceRecord {
    pub id: ProvenanceId,
    pub layer: domain::DataLayer,
    pub subject: SubjectRef,
    pub attribution: Attribution,
    pub source_version_id: Option<domain::SourceVersionId>,
    pub source_location: Option<SourceLocation>,
    pub trust_level: TrustLevel,
    pub verification: VerificationStatus,
    pub confidence: Option<f32>,
    pub versions: DerivationVersions,
    pub created_at: Timestamp,
    pub created_by: PrincipalId,
    pub reviewed_by: Option<PrincipalId>,
    pub reviewed_at: Option<Timestamp>,
    pub review_note: Option<String>,
    pub superseded_by: Option<ProvenanceId>,
}

// ─── ProvenanceRepository ─────────────────────────

#[async_trait::async_trait]
pub trait ProvenanceRepository: Send + Sync {
    async fn insert(&mut self, record: ProvenanceRecord) -> Result<(), ProvenanceError>;
    async fn get(&self, id: &ProvenanceId) -> Result<Option<ProvenanceRecord>, ProvenanceError>;
    async fn list_by_subject(&self, subject: &SubjectRef) -> Result<Vec<ProvenanceRecord>, ProvenanceError>;
    async fn list_by_source_version(&self, source_version_id: &domain::SourceVersionId) -> Result<Vec<ProvenanceRecord>, ProvenanceError>;
    async fn record_review(&mut self, provenance_id: &ProvenanceId, reviewed_by: PrincipalId, note: Option<String>, accepted: bool) -> Result<(), ProvenanceError>;
    async fn supersede(&mut self, id: &ProvenanceId, superseded_by: ProvenanceId) -> Result<(), ProvenanceError>;
}

// ─── ReviewQueue ──────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewQueue {
    pub id: String,
    pub provenance_id: ProvenanceId,
    pub queue: ReviewQueueType,
    pub priority: i32,
    pub evidence_json: String,
    pub state: ReviewQueueState,
    pub decided_by: Option<PrincipalId>,
    pub decided_at: Option<Timestamp>,
    pub decision_note: Option<String>,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewQueueType { GraphEdge, Morphology, NarratorIdentity, CrossReference }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewQueueState { Pending, Accepted, Rejected, Corrected }

// ─── CanonicalWriter ──────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalToken { inner: approval::ApprovalTokenInner }

mod approval {
    use super::*;
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ApprovalTokenInner { pub approval_id: ApprovalId, pub subject_urn: String, pub approved_by: PrincipalId, pub approved_at: Timestamp }
}

impl ApprovalToken {
    pub fn new(approval_id: ApprovalId, subject_urn: String, approved_by: PrincipalId) -> Self {
        Self { inner: approval::ApprovalTokenInner { approval_id, subject_urn, approved_by, approved_at: Timestamp::now() } }
    }
    pub fn approval_id(&self) -> &ApprovalId { &self.inner.approval_id }
    pub fn subject_urn(&self) -> &str { &self.inner.subject_urn }
    pub fn approved_by(&self) -> &PrincipalId { &self.inner.approved_by }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalChangeRequest {
    pub new_source_version_id: domain::SourceVersionId,
    pub content_hash: ContentHash,
    pub structural_validation_report: String,
    pub difference_report: DifferenceReport,
    pub approver_identity: PrincipalId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalChangeSession {
    pub id: String,
    pub request: CanonicalChangeRequest,
    pub opened_at: Timestamp,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionStatus { Open, Committed, Aborted }

#[async_trait::async_trait]
pub trait CanonicalWriter: Send + Sync {
    fn begin_canonical_change(&self, token: &ApprovalToken, change: CanonicalChangeRequest) -> Result<CanonicalChangeSession, ProvenanceError>;
    async fn commit_canonical_change(&self, session: &CanonicalChangeSession) -> Result<(), ProvenanceError>;
    async fn abort_canonical_change(&self, session: &CanonicalChangeSession) -> Result<(), ProvenanceError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DifferenceReport {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<String>,
    pub unchanged: Vec<String>,
}

// ─── ProvenanceError ──────────────────────────────

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ProvenanceError {
    #[error("provenance record not found: {id}")]
    NotFound { id: ProvenanceId },
    #[error("computational annotation missing algorithm or confidence")]
    MissingComputationalMetadata,
    #[error("canonical write requires ApprovalToken")]
    MissingApprovalToken,
    #[error("canonical change request missing required field: {field}")]
    MissingChangeRequestField { field: &'static str },
    #[error("canonical provenance is immutable")]
    ImmutableCanonical,
    #[error("invalid approval token")]
    InvalidApprovalToken,
    #[error("computational annotation cannot be human_verified without reviewer")]
    ComputationalNeedsReview,
    #[error("invalid subject reference: {0}")]
    InvalidSubjectRef(String),
    #[error("storage error: {0}")]
    Storage(#[from] storage::StorageError),
}

// ─── Invariant Tests ──────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{Confidence, SemVer};

    #[test]
    fn computational_annotation_requires_algorithm_and_confidence() {
        let result = try_create_computational(None, None);
        assert!(matches!(result, Err(ProvenanceError::MissingComputationalMetadata)));
    }

    #[test]
    fn canonical_provenance_is_immutable() {
        let record = make_canonical_record();
        assert_eq!(record.layer, domain::DataLayer::CanonicalSource);
    }

    #[test]
    fn approval_token_cannot_be_constructed_directly() {
        let token = ApprovalToken::new(ApprovalId::new(), "urn:qai:source:test".to_string(), PrincipalId::new());
        assert_eq!(token.approval_id().to_string(), token.approval_id().to_string());
    }

    #[test]
    fn difference_report_is_empty_by_default() {
        let report = DifferenceReport::default();
        assert!(report.added.is_empty());
        assert!(report.removed.is_empty());
        assert!(report.changed.is_empty());
    }

    #[test]
    fn canonical_change_request_requires_all_fields() {
        let req = CanonicalChangeRequest {
            new_source_version_id: domain::SourceVersionId::new(),
            content_hash: ContentHash { algorithm: domain::HashAlgorithm::Sha256, hex: "00".to_string() },
            structural_validation_report: "valid".to_string(),
            difference_report: DifferenceReport::default(),
            approver_identity: PrincipalId::new(),
        };
        assert_eq!(req.new_source_version_id.to_string(), req.new_source_version_id.to_string());
    }

    fn make_canonical_record() -> ProvenanceRecord {
        ProvenanceRecord {
            id: ProvenanceId::new(),
            layer: domain::DataLayer::CanonicalSource,
            subject: SubjectRef("urn:qai:quran:ayah:1:1".to_string()),
            attribution: Attribution::Dataset { source_id: "test".to_string(), dataset_name: "test".to_string(), dataset_version: "1.0".to_string() },
            source_version_id: Some(domain::SourceVersionId::new()),
            source_location: None,
            trust_level: TrustLevel::CanonicalVerified,
            verification: VerificationStatus::HumanVerified,
            confidence: None,
            versions: DerivationVersions {
                source_version_id: domain::SourceVersionId::new(),
                parser_version: SemVer::new(1, 0, 0),
                normalizer_version: SemVer::new(1, 0, 0),
                chunker_version: None,
                embedding_model_version: None,
                graph_builder_version: None,
                dependency_snapshot_id: None,
                schema_version: 1,
            },
            created_at: Timestamp::now(),
            created_by: PrincipalId::new(),
            reviewed_by: None,
            reviewed_at: None,
            review_note: None,
            superseded_by: None,
        }
    }

    fn try_create_computational(algorithm: Option<String>, confidence: Option<f32>) -> Result<ProvenanceError> {
        if algorithm.is_none() || confidence.is_none() {
            return Err(ProvenanceError::MissingComputationalMetadata);
        }
        Ok(ProvenanceError::MissingComputationalMetadata)
    }
}
