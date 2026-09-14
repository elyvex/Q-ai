//! Phase 0 — Audit model (D0.12).
//!
//! Audit model: append-only, hash-chained, secret-free.

use domain::{
    AuditEventId, ContentHash, HashAlgorithm, PrincipalId, SubjectRef, Timestamp,
    canonical_json_bytes,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

// ─── Actor ────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Actor {
    Principal { principal_id: PrincipalId },
    System { name: String },
    Job { job_id: String },
}

// ─── AuditAction ──────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    ConfigChange,
    SecretRefChange,
    MigrationApplied,
    SourceDiscovered,
    SourceImported,
    SourceValidated,
    SourceStaged,
    SourceApproved,
    SourceActivated,
    SourceRolledBack,
    CanonicalChangeSessionOpened,
    CanonicalChangeSessionCommitted,
    CanonicalChangeSessionAborted,
    ApprovalGranted,
    ApprovalDenied,
    PrincipalChanged,
    RoleChanged,
    JobDeadLettered,
    DoctorRepairExecuted,
}

// ─── AuditOutcome ──────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    Allowed,
    Denied,
    Failed,
}

// ─── AuditEvent ────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: AuditEventId,
    pub sequence: u64,
    pub occurred_at: Timestamp,
    pub actor: Actor,
    pub action: AuditAction,
    pub subject: SubjectRef,
    pub outcome: AuditOutcome,
    pub reason: Option<String>,
    pub before: Option<serde_json::Value>,
    pub after: Option<serde_json::Value>,
    pub request_id: Option<String>,
    pub prev_chain_hash: ContentHash,
    pub chain_hash: ContentHash,
}

// ─── AuditRepository ──────────────────────────────

#[async_trait::async_trait]
pub trait AuditRepository: Send + Sync {
    async fn append(&mut self, event: AuditEvent) -> Result<(), AuditError>;
    async fn list_by_subject(
        &self,
        subject_urn: &SubjectRef,
    ) -> Result<Vec<AuditEvent>, AuditError>;
    async fn list_by_sequence(
        &self,
        from: u64,
        to: Option<u64>,
    ) -> Result<Vec<AuditEvent>, AuditError>;
    async fn verify_chain(&self) -> Result<ChainVerificationResult, AuditError>;
    async fn latest_sequence(&self) -> Result<u64, AuditError>;
}

// ─── ChainVerificationResult ──────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainVerificationResult {
    pub valid: bool,
    pub expected_next_sequence: u64,
    pub expected_next_hash: ContentHash,
    pub gaps: Vec<u64>,
    pub tampered_events: Vec<TamperEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TamperEvidence {
    pub sequence: u64,
    pub expected_hash: ContentHash,
    pub actual_hash: ContentHash,
}

// ─── HashChainWriter ──────────────────────────────

pub struct HashChainWriter {
    repo: Box<dyn AuditRepository>,
}

impl HashChainWriter {
    pub fn new(repo: Box<dyn AuditRepository>) -> Self {
        Self { repo }
    }

    fn genesis_hash() -> ContentHash {
        ContentHash { algorithm: HashAlgorithm::Sha256, hex: "00".repeat(32) }
    }

    pub async fn append_event(&mut self, event: AuditEvent) -> Result<AuditEvent, AuditError> {
        let seq = self.repo.latest_sequence().await? + 1;
        let mut event = event;
        event.sequence = seq;
        event.occurred_at = Timestamp::now();
        let prev_hash = if seq == 1 {
            Self::genesis_hash()
        } else {
            let prev = self.repo.list_by_sequence(seq - 1, Some(seq - 1)).await?;
            prev.into_iter()
                .find(|e| e.sequence == seq - 1)
                .map(|e| e.chain_hash)
                .unwrap_or_else(Self::genesis_hash)
        };
        event.prev_chain_hash = prev_hash.clone();
        event.chain_hash = Self::compute_chain_hash(&prev_hash, &event);
        self.repo.append(event.clone()).await?;
        Ok(event)
    }

    pub(crate) fn compute_chain_hash(prev_hash: &ContentHash, event: &AuditEvent) -> ContentHash {
        let mut event_without_hash = event.clone();
        event_without_hash.chain_hash =
            ContentHash { algorithm: HashAlgorithm::Sha256, hex: String::new() };
        event_without_hash.prev_chain_hash =
            ContentHash { algorithm: HashAlgorithm::Sha256, hex: String::new() };
        let canonical = canonical_json_bytes(&event_without_hash).unwrap_or_else(|_| vec![]);
        let mut hasher = Sha256::new();
        hasher.update(prev_hash.hex.as_bytes());
        hasher.update(&canonical);
        let result = hasher.finalize();
        ContentHash { algorithm: HashAlgorithm::Sha256, hex: hex_encode(&result) }
    }
}

// ─── AuditVerifier ────────────────────────────────

pub struct AuditVerifier {
    repo: Box<dyn AuditRepository>,
}

impl AuditVerifier {
    pub fn new(repo: Box<dyn AuditRepository>) -> Self {
        Self { repo }
    }

    /// Verify the whole chain end-to-end by **recomputing every hash**.
    ///
    /// Detects a tampered row (changed content, chain hash, or sequence) and
    /// reports the offending sequence. This is stronger than a structural
    /// linkage check because it re-derives each `chain_hash` from the event's
    /// canonical serialization.
    pub async fn verify(&self) -> Result<ChainVerificationResult, AuditError> {
        let events = self.repo.list_by_sequence(0, None).await?;

        let mut valid = true;
        let mut gaps = Vec::new();
        let mut tampered = Vec::new();
        let mut prev_hash = HashChainWriter::genesis_hash();
        let mut expected_seq: u64 = 1;

        for event in &events {
            if event.sequence != expected_seq {
                gaps.push(event.sequence);
                valid = false;
            }
            let expected = HashChainWriter::compute_chain_hash(&prev_hash, event);
            if expected != event.chain_hash {
                valid = false;
                tampered.push(TamperEvidence {
                    sequence: event.sequence,
                    expected_hash: expected,
                    actual_hash: event.chain_hash.clone(),
                });
            }
            prev_hash = event.chain_hash.clone();
            expected_seq = event.sequence + 1;
        }

        Ok(ChainVerificationResult {
            valid,
            expected_next_sequence: expected_seq,
            expected_next_hash: prev_hash,
            gaps,
            tampered_events: tampered,
        })
    }

    pub async fn verify_event(&self, sequence: u64) -> Result<bool, AuditError> {
        let events = self.repo.list_by_sequence(sequence - 1, Some(sequence)).await?;
        if events.len() < 2 {
            return Ok(false);
        }
        let prev = &events[0];
        let current = &events[1];
        let expected = HashChainWriter::compute_chain_hash(&prev.chain_hash, current);
        Ok(current.chain_hash == expected)
    }
}

// ─── Redaction ────────────────────────────────────

pub fn redact_audit_value(value: &mut serde_json::Value) {
    redact_json(value);
}

fn redact_json(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(s) => {
            if is_secret(s) {
                *value = serde_json::Value::String("***REDACTED***".to_string());
            }
        }
        serde_json::Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                if is_secret_key(k) {
                    *v = serde_json::Value::String("***REDACTED***".to_string());
                } else {
                    redact_json(v);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                redact_json(item);
            }
        }
        _ => {}
    }
}

fn is_secret(s: &str) -> bool {
    let lower = s.to_lowercase();
    lower.contains("secret")
        || lower.contains("password")
        || lower.contains("api_key")
        || lower.contains("token")
        || lower.contains("credential")
}
fn is_secret_key(key: &str) -> bool {
    let lower = key.to_lowercase();
    lower.contains("secret")
        || lower.contains("password")
        || lower.contains("api_key")
        || lower.contains("token")
        || lower.contains("credential")
}

// ─── AuditError ──────────────────────────────────

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AuditError {
    #[error("audit event not found: {id}")]
    NotFound { id: AuditEventId },
    #[error("audit log is append-only")]
    AppendOnlyViolation,
    #[error("audit chain verification failed at sequence {sequence}")]
    ChainVerificationFailed { sequence: u64 },
    #[error("audit chain gap detected at sequence {sequence}")]
    SequenceGap { sequence: u64 },
    #[error("chain hash mismatch at sequence {sequence}")]
    HashMismatch { sequence: u64 },
    #[error("secret detected in audit record at sequence {sequence}")]
    SecretLeakDetected { sequence: u64 },
    #[error("storage error: {0}")]
    Storage(#[from] storage::StorageError),
}

impl AuditError {
    /// The stable `QAI-AUD-nnnn` diagnostic code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound { .. } => "QAI-AUD-0001",
            Self::AppendOnlyViolation => "QAI-AUD-0002",
            Self::ChainVerificationFailed { .. } => "QAI-AUD-0003",
            Self::SequenceGap { .. } => "QAI-AUD-0004",
            Self::HashMismatch { .. } => "QAI-AUD-0005",
            Self::SecretLeakDetected { .. } => "QAI-AUD-0006",
            Self::Storage(_) => "QAI-AUD-0007",
        }
    }
}

impl storage::error::Diagnostic for AuditError {
    fn code(&self) -> storage::error::DiagnosticCode {
        storage::error::DiagnosticCode::new(self.code(), 0)
    }
    fn summary(&self) -> String {
        self.to_string()
    }
    fn next_command(&self) -> Option<String> {
        match self {
            Self::ChainVerificationFailed { .. }
            | Self::SequenceGap { .. }
            | Self::HashMismatch { .. } => Some("qai audit verify".to_string()),
            _ => None,
        }
    }
}

// ─── Helper ───────────────────────────────────────

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_chain_computation() {
        let prev_hash = ContentHash { algorithm: HashAlgorithm::Sha256, hex: "00".repeat(32) };
        let event = AuditEvent {
            id: AuditEventId::new(),
            sequence: 1,
            occurred_at: Timestamp::now(),
            actor: Actor::System { name: "test".to_string() },
            action: AuditAction::ConfigChange,
            subject: SubjectRef("urn:test".to_string()),
            outcome: AuditOutcome::Allowed,
            reason: None,
            before: None,
            after: None,
            request_id: None,
            prev_chain_hash: prev_hash.clone(),
            chain_hash: ContentHash { algorithm: HashAlgorithm::Sha256, hex: String::new() },
        };
        let chain_hash = HashChainWriter::compute_chain_hash(&prev_hash, &event);
        assert_eq!(chain_hash.algorithm, HashAlgorithm::Sha256);
        assert!(!chain_hash.hex.is_empty());
    }

    #[test]
    fn different_events_produce_different_hashes() {
        let prev_hash = ContentHash { algorithm: HashAlgorithm::Sha256, hex: "00".repeat(32) };
        let event1 = AuditEvent {
            id: AuditEventId::new(),
            sequence: 1,
            occurred_at: Timestamp::now(),
            actor: Actor::System { name: "test1".to_string() },
            action: AuditAction::ConfigChange,
            subject: SubjectRef("urn:test".to_string()),
            outcome: AuditOutcome::Allowed,
            reason: None,
            before: None,
            after: None,
            request_id: None,
            prev_chain_hash: prev_hash.clone(),
            chain_hash: ContentHash { algorithm: HashAlgorithm::Sha256, hex: String::new() },
        };
        let event2 = AuditEvent {
            id: AuditEventId::new(),
            sequence: 2,
            occurred_at: Timestamp::now(),
            actor: Actor::System { name: "test2".to_string() },
            action: AuditAction::SourceApproved,
            subject: SubjectRef("urn:test".to_string()),
            outcome: AuditOutcome::Allowed,
            reason: None,
            before: None,
            after: None,
            request_id: None,
            prev_chain_hash: prev_hash.clone(),
            chain_hash: ContentHash { algorithm: HashAlgorithm::Sha256, hex: String::new() },
        };
        let hash1 = HashChainWriter::compute_chain_hash(&prev_hash, &event1);
        let hash2 = HashChainWriter::compute_chain_hash(&prev_hash, &event2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn redaction_strips_secrets() {
        let mut json = serde_json::json!({ "password": "mysecret", "api_key": "sk-12345", "name": "public_value" });
        redact_audit_value(&mut json);
        assert_eq!(json["password"], "***REDACTED***");
        assert_eq!(json["api_key"], "***REDACTED***");
        assert_eq!(json["name"], "public_value");
    }

    #[test]
    fn actor_variants() {
        let principal = Actor::Principal { principal_id: PrincipalId::new() };
        let system = Actor::System { name: "systemd".to_string() };
        let job = Actor::Job { job_id: "job-123".to_string() };
        assert!(matches!(principal, Actor::Principal { .. }));
        assert!(matches!(system, Actor::System { .. }));
        assert!(matches!(job, Actor::Job { .. }));
    }

    // ─── In-memory repo for end-to-end chain tests ───

    #[derive(Clone, Default)]
    struct SharedRepo(std::sync::Arc<std::sync::Mutex<Vec<AuditEvent>>>);

    #[async_trait::async_trait]
    impl AuditRepository for SharedRepo {
        async fn append(&mut self, event: AuditEvent) -> Result<(), AuditError> {
            self.0.lock().unwrap().push(event);
            Ok(())
        }
        async fn list_by_subject(
            &self,
            subject: &SubjectRef,
        ) -> Result<Vec<AuditEvent>, AuditError> {
            Ok(self
                .0
                .lock()
                .unwrap()
                .iter()
                .filter(|e| e.subject.0 == subject.0)
                .cloned()
                .collect())
        }
        async fn list_by_sequence(
            &self,
            from: u64,
            to: Option<u64>,
        ) -> Result<Vec<AuditEvent>, AuditError> {
            Ok(self
                .0
                .lock()
                .unwrap()
                .iter()
                .filter(|e| e.sequence >= from && to.map(|t| e.sequence <= t).unwrap_or(true))
                .cloned()
                .collect())
        }
        async fn verify_chain(&self) -> Result<ChainVerificationResult, AuditError> {
            Ok(ChainVerificationResult {
                valid: true,
                expected_next_sequence: self.0.lock().unwrap().len() as u64 + 1,
                expected_next_hash: ContentHash {
                    algorithm: HashAlgorithm::Sha256,
                    hex: String::new(),
                },
                gaps: Vec::new(),
                tampered_events: Vec::new(),
            })
        }
        async fn latest_sequence(&self) -> Result<u64, AuditError> {
            Ok(self.0.lock().unwrap().iter().map(|e| e.sequence).max().unwrap_or(0))
        }
    }

    fn sample_event(subject: &str) -> AuditEvent {
        AuditEvent {
            id: AuditEventId::new(),
            sequence: 0,
            occurred_at: Timestamp::now(),
            actor: Actor::System { name: "test".into() },
            action: AuditAction::ConfigChange,
            subject: SubjectRef(subject.into()),
            outcome: AuditOutcome::Allowed,
            reason: None,
            before: None,
            after: None,
            request_id: None,
            prev_chain_hash: ContentHash { algorithm: HashAlgorithm::Sha256, hex: String::new() },
            chain_hash: ContentHash { algorithm: HashAlgorithm::Sha256, hex: String::new() },
        }
    }

    #[tokio::test]
    async fn writer_links_each_event_to_the_previous() {
        let shared = SharedRepo::default();
        let mut writer = HashChainWriter::new(Box::new(shared));
        let e1 = writer.append_event(sample_event("urn:a")).await.unwrap();
        let e2 = writer.append_event(sample_event("urn:b")).await.unwrap();
        assert_eq!(e1.prev_chain_hash.hex, "00".repeat(32));
        assert_eq!(e2.prev_chain_hash, e1.chain_hash);
    }

    #[tokio::test]
    async fn verifier_accepts_a_valid_chain_then_detects_a_tampered_row() {
        let shared = SharedRepo::default();
        let mut writer = HashChainWriter::new(Box::new(shared.clone()));
        writer.append_event(sample_event("urn:a")).await.unwrap();
        writer.append_event(sample_event("urn:b")).await.unwrap();

        let verifier = AuditVerifier::new(Box::new(shared.clone()));
        let clean = verifier.verify().await.unwrap();
        assert!(clean.valid, "freshly written chain must verify: {clean:?}");

        // Manually tamper with the second row's content (not its hash).
        {
            let mut events = shared.0.lock().unwrap();
            events[1].reason = Some("tampered".into());
        }

        let tampered = verifier.verify().await.unwrap();
        assert!(!tampered.valid, "tampered chain must fail verification");
        assert_eq!(tampered.tampered_events.len(), 1);
        assert_eq!(tampered.tampered_events[0].sequence, 2);
    }

    #[tokio::test]
    async fn verifier_detects_a_sequence_gap() {
        let shared = SharedRepo::default();
        let mut writer = HashChainWriter::new(Box::new(shared.clone()));
        writer.append_event(sample_event("urn:a")).await.unwrap();
        writer.append_event(sample_event("urn:b")).await.unwrap();
        writer.append_event(sample_event("urn:c")).await.unwrap();

        // Drop the middle row to create a gap.
        {
            let mut events = shared.0.lock().unwrap();
            events.remove(1);
        }

        let verifier = AuditVerifier::new(Box::new(shared));
        let result = verifier.verify().await.unwrap();
        assert!(!result.valid);
        assert!(!result.gaps.is_empty());
    }
}
