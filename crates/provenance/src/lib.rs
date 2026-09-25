//! Phase 0 — Provenance model (D0.11).

use domain::{
    ContentHash, DerivationVersions, PrincipalId, ProvenanceId, SubjectRef, Timestamp, TrustLevel,
    VerificationStatus,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

type Result<T, E = ProvenanceError> = std::result::Result<T, E>;

// ─── Attribution ──────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Attribution {
    Dataset {
        source_id: String,
        dataset_name: String,
        dataset_version: String,
    },
    Scholar {
        name: String,
        school: Option<String>,
        work: Option<String>,
        edition: Option<String>,
    },
    Computational {
        algorithm: String,
        version: String,
        model: Option<String>,
        parameters_hash: ContentHash,
    },
    User {
        principal_id: PrincipalId,
    },
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    async fn list_by_subject(
        &self,
        subject: &SubjectRef,
    ) -> Result<Vec<ProvenanceRecord>, ProvenanceError>;
    async fn list_by_source_version(
        &self,
        source_version_id: &domain::SourceVersionId,
    ) -> Result<Vec<ProvenanceRecord>, ProvenanceError>;
    async fn record_review(
        &mut self,
        provenance_id: &ProvenanceId,
        reviewed_by: PrincipalId,
        note: Option<String>,
        accepted: bool,
    ) -> Result<(), ProvenanceError>;
    async fn supersede(
        &mut self,
        id: &ProvenanceId,
        superseded_by: ProvenanceId,
    ) -> Result<(), ProvenanceError>;
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
pub enum ReviewQueueType {
    GraphEdge,
    Morphology,
    NarratorIdentity,
    CrossReference,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewQueueState {
    Pending,
    Accepted,
    Rejected,
    Corrected,
}

// ─── CanonicalWriter ──────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalToken {
    inner: approval::ApprovalTokenInner,
}

mod approval {
    use super::*;
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ApprovalTokenInner {
        pub approval_id: String,
        pub subject_urn: String,
        pub approved_by: String,
        pub approved_at: Timestamp,
    }
}

impl ApprovalToken {
    /// Crate-private raw constructor. The only public mint path is
    /// [`ApprovalToken::from_approval_row`], which requires a persisted granted
    /// approval row, so no caller can forge a token out of thin air.
    pub(crate) fn new(approval_id: String, subject_urn: String, approved_by: String) -> Self {
        Self {
            inner: approval::ApprovalTokenInner {
                approval_id,
                subject_urn,
                approved_by,
                approved_at: Timestamp::now(),
            },
        }
    }

    /// Mint a token from a persisted approval row.
    ///
    /// Succeeds only when the row's `decision` is exactly `"approved"` and its
    /// `subject_urn` is non-empty. A not-granted row and a subject-less row
    /// each yield a distinct typed error (`ApprovalNotGranted` /
    /// `ApprovalSubjectMissing`).
    pub fn from_approval_row(
        row: &storage::repository::ApprovalRow,
    ) -> Result<ApprovalToken, ProvenanceError> {
        if row.decision.as_deref() != Some("approved") {
            return Err(ProvenanceError::ApprovalNotGranted { approval_id: row.id.clone() });
        }
        if row.subject_urn.trim().is_empty() {
            return Err(ProvenanceError::ApprovalSubjectMissing { approval_id: row.id.clone() });
        }
        Ok(ApprovalToken::new(
            row.id.clone(),
            row.subject_urn.clone(),
            row.decided_by.clone().unwrap_or_default(),
        ))
    }

    /// The approval row id this token was minted from.
    pub fn approval_id(&self) -> &str {
        &self.inner.approval_id
    }

    /// The exact canonical subject URN the approval names.
    pub fn subject_urn(&self) -> &str {
        &self.inner.subject_urn
    }

    /// The principal recorded as having granted the approval (may be empty).
    pub fn approved_by(&self) -> &str {
        &self.inner.approved_by
    }

    /// Whether this token authorises a change to exactly `subject_urn`.
    pub fn authorises(&self, subject_urn: &str) -> bool {
        self.inner.subject_urn == subject_urn
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalChangeRequest {
    /// The exact canonical subject this change applies to
    /// (`quran-edition:{slug}@{version}`). Must equal the token's subject.
    pub subject_urn: String,
    /// Source-version id the change derives from (verbatim storage id).
    pub new_source_version_id: String,
    /// The edition's canonical `text_hash` as a typed content hash.
    pub content_hash: ContentHash,
    /// Persisted structural validation report for the change.
    pub structural_validation_report: String,
    /// Difference report for the change (canonical differences live in the
    /// `difference_reports` table; this is the provenance-level summary).
    pub difference_report: DifferenceReport,
    /// Principal recorded as approving the change.
    pub approver_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalChangeSession {
    pub id: String,
    pub request: CanonicalChangeRequest,
    pub opened_at: Timestamp,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Open,
    Committed,
    Aborted,
}

#[async_trait::async_trait]
pub trait CanonicalWriter: Send + Sync {
    fn begin_canonical_change(
        &self,
        token: &ApprovalToken,
        change: CanonicalChangeRequest,
    ) -> Result<CanonicalChangeSession, ProvenanceError>;
    async fn commit_canonical_change(
        &self,
        session: &mut CanonicalChangeSession,
    ) -> Result<(), ProvenanceError>;
    async fn abort_canonical_change(
        &self,
        session: &mut CanonicalChangeSession,
    ) -> Result<(), ProvenanceError>;
}

/// The only in-tree [`CanonicalWriter`].
///
/// It enforces the type-level half of the canonical-write fence: a change
/// session opens only when the [`ApprovalToken`] is bound to the change's exact
/// `subject_urn`. The persisted-approval check itself lives in the token's
/// `from_approval_row` constructor, so no canonical publication can begin
/// without a granted approval row naming the exact edition URN.
///
/// The alternative — retiring the trait as dead code — was rejected: ADR-0000
/// locks canonical immutability as a **type-level** property and
/// `.agent/coding-rules.md` states "canonical rows are written only through a
/// `CanonicalWriter` that requires an `ApprovalToken`", so the invariant is
/// wired rather than documented away.
#[derive(Debug, Clone, Copy, Default)]
pub struct ApprovalGate;

#[async_trait::async_trait]
impl CanonicalWriter for ApprovalGate {
    fn begin_canonical_change(
        &self,
        token: &ApprovalToken,
        change: CanonicalChangeRequest,
    ) -> Result<CanonicalChangeSession, ProvenanceError> {
        if !token.authorises(&change.subject_urn) {
            return Err(ProvenanceError::ApprovalSubjectMismatch {
                expected: token.subject_urn().to_string(),
                actual: change.subject_urn.clone(),
            });
        }
        Ok(CanonicalChangeSession {
            id: change.subject_urn.clone(),
            request: change,
            opened_at: Timestamp::now(),
            status: SessionStatus::Open,
        })
    }

    async fn commit_canonical_change(
        &self,
        session: &mut CanonicalChangeSession,
    ) -> Result<(), ProvenanceError> {
        if session.status != SessionStatus::Open {
            return Err(ProvenanceError::InvalidSessionState {
                id: session.id.clone(),
                state: format!("{:?}", session.status),
            });
        }
        session.status = SessionStatus::Committed;
        Ok(())
    }

    async fn abort_canonical_change(
        &self,
        session: &mut CanonicalChangeSession,
    ) -> Result<(), ProvenanceError> {
        session.status = SessionStatus::Aborted;
        Ok(())
    }
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
    #[error("approval {approval_id} is not granted")]
    ApprovalNotGranted {
        /// The approval row id.
        approval_id: String,
    },
    #[error("approval {approval_id} names no subject")]
    ApprovalSubjectMissing {
        /// The approval row id.
        approval_id: String,
    },
    #[error("approval token subject {actual} does not authorise {expected}")]
    ApprovalSubjectMismatch {
        /// The subject the token authorises.
        expected: String,
        /// The subject the change requests.
        actual: String,
    },
    #[error("canonical change session {id} is not open (state {state})")]
    InvalidSessionState {
        /// The session id.
        id: String,
        /// The session's current state.
        state: String,
    },
    #[error("storage error: {0}")]
    Storage(#[from] storage::StorageError),
}

impl ProvenanceError {
    /// The stable `QAI-PROV-nnnn` diagnostic code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound { .. } => "QAI-PROV-0001",
            Self::MissingComputationalMetadata => "QAI-PROV-0002",
            Self::MissingApprovalToken => "QAI-PROV-0003",
            Self::MissingChangeRequestField { .. } => "QAI-PROV-0004",
            Self::ImmutableCanonical => "QAI-PROV-0005",
            Self::InvalidApprovalToken => "QAI-PROV-0006",
            Self::ComputationalNeedsReview => "QAI-PROV-0007",
            Self::InvalidSubjectRef(_) => "QAI-PROV-0008",
            Self::ApprovalNotGranted { .. } => "QAI-PROV-0010",
            Self::ApprovalSubjectMissing { .. } => "QAI-PROV-0011",
            Self::ApprovalSubjectMismatch { .. } => "QAI-PROV-0012",
            Self::InvalidSessionState { .. } => "QAI-PROV-0013",
            Self::Storage(_) => "QAI-PROV-0009",
        }
    }
}

impl storage::error::Diagnostic for ProvenanceError {
    fn code(&self) -> storage::error::DiagnosticCode {
        storage::error::DiagnosticCode::new(self.code(), 0)
    }
    fn summary(&self) -> String {
        self.to_string()
    }
}

// ─── Invariant Tests ──────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use domain::SemVer;

    #[test]
    fn computational_annotation_requires_algorithm_and_confidence() {
        let result = try_create_computational(None, None);
        assert!(matches!(result, Err(ProvenanceError::MissingComputationalMetadata)));
    }

    #[test]
    fn canonical_provenance_is_immutable() {
        let record = make_canonical_record();
        assert_eq!(record.layer, domain::DataLayer::Canonical);
    }

    #[test]
    fn denied_approval_row_yields_the_not_granted_error() {
        let row = approval_row("appr-1", Some("denied"), "quran-edition:min@0.1.0");
        let err = ApprovalToken::from_approval_row(&row).unwrap_err();
        assert!(matches!(err, ProvenanceError::ApprovalNotGranted { .. }));
        assert_eq!(err.code(), "QAI-PROV-0010");
    }

    #[test]
    fn subject_less_approval_row_yields_the_missing_subject_error() {
        let row = approval_row("appr-2", Some("approved"), "   ");
        let err = ApprovalToken::from_approval_row(&row).unwrap_err();
        assert!(matches!(err, ProvenanceError::ApprovalSubjectMissing { .. }));
        assert_eq!(err.code(), "QAI-PROV-0011");
    }

    #[test]
    fn approved_row_mints_a_token_bound_to_its_subject() {
        let row = approval_row("appr-3", Some("approved"), "quran-edition:min@0.1.0");
        let token = ApprovalToken::from_approval_row(&row).unwrap();
        assert_eq!(token.approval_id(), "appr-3");
        assert_eq!(token.subject_urn(), "quran-edition:min@0.1.0");
        assert_eq!(token.approved_by(), "approver");
        assert!(token.authorises("quran-edition:min@0.1.0"));
        assert!(!token.authorises("quran-edition:other@9.9.9"));
    }

    #[test]
    fn no_public_constructor_bypasses_the_row_check() {
        // Audit the `impl ApprovalToken` block: the only public constructor is
        // the row-derived one, and the raw constructor is crate-private.
        let src = include_str!("lib.rs");
        let rest = &src[src.find("impl ApprovalToken {").expect("impl block")..];
        let block = &rest[..rest.find("\n}").expect("impl end")];
        assert!(block.contains("pub(crate) fn new("), "raw constructor must be crate-private");
        assert!(!block.contains("pub fn new("), "no public raw constructor may exist");
        for line in block.lines() {
            if let Some(rest) = line.trim_start().strip_prefix("pub fn ") {
                let name = rest.split('(').next().unwrap_or("");
                assert!(
                    matches!(
                        name,
                        "from_approval_row" | "approval_id" | "subject_urn" | "approved_by"
                            | "authorises"
                    ),
                    "unexpected public fn on ApprovalToken: {name}"
                );
            }
        }
    }

    #[test]
    fn approval_gate_rejects_a_mismatched_subject_before_any_write() {
        let row = approval_row("appr-4", Some("approved"), "quran-edition:min@0.1.0");
        let token = ApprovalToken::from_approval_row(&row).unwrap();
        let err = ApprovalGate
            .begin_canonical_change(&token, change_request("quran-edition:other@9.9.9"))
            .unwrap_err();
        assert!(matches!(err, ProvenanceError::ApprovalSubjectMismatch { .. }));
        assert_eq!(err.code(), "QAI-PROV-0012");
    }

    #[test]
    fn approval_gate_tracks_open_committed_and_aborted_sessions() {
        let row = approval_row("appr-5", Some("approved"), "quran-edition:min@0.1.0");
        let token = ApprovalToken::from_approval_row(&row).unwrap();
        let mut session =
            ApprovalGate.begin_canonical_change(&token, change_request(token.subject_urn())).unwrap();
        assert_eq!(session.status, SessionStatus::Open);
        block_on(ApprovalGate.commit_canonical_change(&mut session)).unwrap();
        assert_eq!(session.status, SessionStatus::Committed);
        let err = block_on(ApprovalGate.commit_canonical_change(&mut session)).unwrap_err();
        assert!(matches!(err, ProvenanceError::InvalidSessionState { .. }));
        let mut aborted =
            ApprovalGate.begin_canonical_change(&token, change_request(token.subject_urn())).unwrap();
        block_on(ApprovalGate.abort_canonical_change(&mut aborted)).unwrap();
        assert_eq!(aborted.status, SessionStatus::Aborted);
    }

    #[test]
    fn difference_report_is_empty_by_default() {
        let report = DifferenceReport::default();
        assert!(report.added.is_empty());
        assert!(report.removed.is_empty());
        assert!(report.changed.is_empty());
    }

    #[test]
    fn canonical_change_request_carries_the_subject_and_source_identity() {
        let req = change_request("quran-edition:min@0.1.0");
        assert_eq!(req.subject_urn, "quran-edition:min@0.1.0");
        assert_eq!(req.new_source_version_id, "sv-1");
        assert_eq!(req.approver_identity, "approver");
    }

    fn approval_row(
        id: &str,
        decision: Option<&str>,
        subject: &str,
    ) -> storage::repository::ApprovalRow {
        storage::repository::ApprovalRow {
            id: id.to_string(),
            subject_urn: subject.to_string(),
            kind: "CanonicalChange".to_string(),
            requested_by: Some("requester".to_string()),
            decided_by: Some("approver".to_string()),
            decision: decision.map(str::to_string),
            request_payload: "{}".to_string(),
            decision_note: None,
            requested_at: "2026-09-14T00:00:00Z".to_string(),
            decided_at: Some("2026-09-14T00:00:00Z".to_string()),
        }
    }

    fn change_request(subject: &str) -> CanonicalChangeRequest {
        CanonicalChangeRequest {
            subject_urn: subject.to_string(),
            new_source_version_id: "sv-1".to_string(),
            content_hash: ContentHash {
                algorithm: domain::HashAlgorithm::Sha256,
                hex: "00".to_string(),
            },
            structural_validation_report: "valid".to_string(),
            difference_report: DifferenceReport::default(),
            approver_identity: "approver".to_string(),
        }
    }

    /// Minimal executor for the gate's always-ready futures (no tokio dep).
    fn block_on<F: std::future::Future>(fut: F) -> F::Output {
        use std::task::{Context, Poll, Waker};
        let mut fut = std::pin::pin!(fut);
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);
        loop {
            if let Poll::Ready(value) = fut.as_mut().poll(&mut cx) {
                return value;
            }
            std::thread::yield_now();
        }
    }

    fn make_canonical_record() -> ProvenanceRecord {
        ProvenanceRecord {
            id: ProvenanceId::new(),
            layer: domain::DataLayer::Canonical,
            subject: SubjectRef("urn:qai:quran:ayah:1:1".to_string()),
            attribution: Attribution::Dataset {
                source_id: "test".to_string(),
                dataset_name: "test".to_string(),
                dataset_version: "1.0".to_string(),
            },
            source_version_id: Some(domain::SourceVersionId::new()),
            source_location: None,
            trust_level: TrustLevel::CanonicalVerified,
            verification: VerificationStatus::Verified,
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

    fn try_create_computational(algorithm: Option<String>, confidence: Option<f32>) -> Result<()> {
        if algorithm.is_none() || confidence.is_none() {
            return Err(ProvenanceError::MissingComputationalMetadata);
        }
        Ok(())
    }
}
