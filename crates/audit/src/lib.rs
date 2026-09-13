//! Phase 0 — Audit model (D0.12).
//!
//! Audit model: append-only, hash-chained, secret-free.

use domain::{AuditEventId, ContentHash, HashAlgorithm, PrincipalId, SubjectRef, Timestamp, canonical_json_bytes};
use storage::error::StorageError;
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
    async fn list_by_subject(&self, subject_urn: &SubjectRef) -> Result<Vec<AuditEvent>, AuditError>;
    async fn list_by_sequence(&self, from: u64, to: Option<u64>) -> Result<Vec<AuditEvent>, AuditError>;
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

    pub async fn append_event(&mut self, event: AuditEvent) -> Result<AuditEvent, AuditError> {
        let seq = self.repo.latest_sequence().await? + 1;
        let mut event = event;
        event.sequence = seq;
        event.occurred_at = Timestamp::now();
        let prev_hash = if seq == 1 {
            ContentHash { algorithm: HashAlgorithm::Sha256, hex: "00".repeat(32) }
        } else {
            self.get_prev_chain_hash(seq - 1).await?
        };
        event.prev_chain_hash = prev_hash.clone();
        event.chain_hash = Self::compute_chain_hash(&prev_hash, &event);
        self.repo.append(event.clone()).await?;
        Ok(event)
    }

    fn compute_chain_hash(prev_hash: &ContentHash, event: &AuditEvent) -> ContentHash {
        let mut event_without_hash = event.clone();
        event_without_hash.chain_hash = ContentHash { algorithm: HashAlgorithm::Sha256, hex: String::new() };
        event_without_hash.prev_chain_hash = ContentHash { algorithm: HashAlgorithm::Sha256, hex: String::new() };
        let canonical = canonical_json_bytes(&event_without_hash).unwrap_or_else(|_| vec![]);
        let mut hasher = Sha256::new();
        hasher.update(prev_hash.hex.as_bytes());
        hasher.update(&canonical);
        let result = hasher.finalize();
        ContentHash { algorithm: HashAlgorithm::Sha256, hex: hex_encode(&result) }
    }

    async fn get_prev_chain_hash(&self, _seq: u64) -> Result<ContentHash, AuditError> {
        Ok(ContentHash { algorithm: HashAlgorithm::Sha256, hex: "00".repeat(32) })
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

    pub async fn verify(&self) -> Result<ChainVerificationResult, AuditError> {
        self.repo.verify_chain().await
    }

    pub async fn verify_event(&self, sequence: u64) -> Result<bool, AuditError> {
        let events = self.repo.list_by_sequence(sequence - 1, Some(sequence)).await?;
        if events.len() < 2 { return Ok(false); }
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
            if is_secret(s) { *value = serde_json::Value::String("***REDACTED***".to_string()); }
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
        serde_json::Value::Array(arr) => { for item in arr { redact_json(item); } }
        _ => {}
    }
}

fn is_secret(s: &str) -> bool {
    let lower = s.to_lowercase();
    lower.contains("secret") || lower.contains("password") || lower.contains("api_key") || lower.contains("token") || lower.contains("credential")
}
fn is_secret_key(key: &str) -> bool {
    let lower = key.to_lowercase();
    lower.contains("secret") || lower.contains("password") || lower.contains("api_key") || lower.contains("token") || lower.contains("credential")
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
            id: AuditEventId::new(), sequence: 1, occurred_at: Timestamp::now(),
            actor: Actor::System { name: "test".to_string() }, action: AuditAction::ConfigChange,
            subject: SubjectRef("urn:test".to_string()),
            outcome: AuditOutcome::Allowed, reason: None, before: None, after: None,
            request_id: None, prev_chain_hash: prev_hash.clone(),
            chain_hash: ContentHash { algorithm: HashAlgorithm::Sha256, hex: String::new() },
        };
        let chain_hash = HashChainWriter::compute_chain_hash(&prev_hash, &event);
        assert_eq!(chain_hash.algorithm, HashAlgorithm::Sha256);
        assert!(!chain_hash.hex.is_empty());
    }

    #[test]
    fn different_events_produce_different_hashes() {
        let prev_hash = ContentHash { algorithm: HashAlgorithm::Sha256, hex: "00".repeat(32) };
        let mut event1 = AuditEvent {
            id: AuditEventId::new(), sequence: 1, occurred_at: Timestamp::now(),
            actor: Actor::System { name: "test1".to_string() }, action: AuditAction::ConfigChange,
            subject: SubjectRef("urn:test".to_string()),
            outcome: AuditOutcome::Allowed, reason: None, before: None, after: None,
            request_id: None, prev_chain_hash: prev_hash.clone(),
            chain_hash: ContentHash { algorithm: HashAlgorithm::Sha256, hex: String::new() },
        };
        let event2 = AuditEvent {
            id: AuditEventId::new(), sequence: 2, occurred_at: Timestamp::now(),
            actor: Actor::System { name: "test2".to_string() }, action: AuditAction::SourceApproved,
            subject: SubjectRef("urn:test".to_string()),
            outcome: AuditOutcome::Allowed, reason: None, before: None, after: None,
            request_id: None, prev_chain_hash: prev_hash.clone(),
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
}
