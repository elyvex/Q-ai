//! Dev-only test fixtures for Q-ai (plan D0.16, task P0-T57).
//!
//! This crate must never be a runtime dependency; it exists so integration
//! tests across crates share one fixture vocabulary instead of re-inventing
//! builders.
//!
//! Fixtures provided:
//! - [`temp_dir`] / [`temp_db_path`] — filesystem scratch state
//! - [`sample_config`] — config layering
//! - [`sample_source_version`] / [`sample_job_record`] — catalog/job rows
//! - [`MockAuditRepo`] — in-memory audit repository

use async_trait::async_trait;
use domain::{
    Confidence, ContentHash, DataLayer, DerivationVersions, HashAlgorithm,
    LicenseStatus, PrincipalId, SemVer, SourceId, SourceVersionId,
    SubjectRef, Timestamp, TrustLevel, VerificationStatus,
};
use sources::{FileRole, SourceFile, SourceState, SourceVersion};
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a temporary directory, removed on drop.
pub fn temp_dir() -> TempDir {
    tempfile::tempdir().expect("tempdir")
}

/// Build a writable SQLite path inside `dir` and return it as a string.
pub fn temp_db_path(dir: &TempDir) -> String {
    dir.path().join("qai.db").to_string_lossy().into_owned()
}

/// A valid default [`config::Config`] with `data_dir` pointed at `dir`.
pub fn sample_config(dir: &TempDir) -> config::Config {
    let mut cfg = config::Config::default();
    cfg.app.data_dir = dir.path().display().to_string();
    cfg.storage.sqlite.path = dir.path().join("qai.db").display().to_string();
    cfg
}

/// A signable TOML fixture suitable for `Config::load(Some(path), ...)`.
pub fn sample_toml_config(dir: &TempDir) -> (String, PathBuf) {
    let path = dir.path().join("config.toml");
    let body = format!(
        r#"
[app]
data_dir = "{}"

[storage.sqlite]
path = "{}/qai.db"
read_only_pool = true
"#,
        dir.path().display(),
        dir.path().display(),
    );
    (body, path)
}

/// A canonical `SourceVersion` fixture passing state machine preconditions.
pub fn sample_source_version() -> SourceVersion {
    let content_hash = ContentHash {
        algorithm: HashAlgorithm::Sha256,
        hex: "00".repeat(32),
    };
    SourceVersion {
        id: SourceVersionId::new(),
        source_id: SourceId::new(),
        version: "1.0.0".into(),
        schema_version: 1,
        state: SourceState::Active,
        trust_level: TrustLevel::PublisherVerified,
        license_status: LicenseStatus::OpenLicense,
        license_json: r#"{"attribution_required":true}"#.into(),
        manifest_blob_id: None,
        manifest_hash: Some(content_hash.clone()),
        content_hash: Some(content_hash),
        source_urls: vec!["https://example.org/hafs.json".into()],
        publication_date: None,
        imported_at: Some(Timestamp::now()),
        validated_at: Some(Timestamp::now()),
        approved_at: Some(Timestamp::now()),
        approved_by: Some(PrincipalId::new()),
        activated_at: Some(Timestamp::now()),
        deprecated_at: None,
        quarantine_reason: None,
        validation_report: Some(r#"{"valid":true}"#.into()),
        notes: None,
        created_at: Timestamp::now(),
    }
}

/// A `SourceFile` fixture attached to the given version.
pub fn sample_source_file(version: &SourceVersion) -> SourceFile {
    SourceFile {
        id: format!("file-{}", version.id),
        source_version_id: version.id,
        role: FileRole::Primary,
        relative_path: "quran/hafs.json".into(),
        format: "json".into(),
        media_type: Some("application/json".into()),
        bytes: Some(4321),
        declared_hash: ContentHash { algorithm: HashAlgorithm::Sha256, hex: "aa".repeat(32) },
        observed_hash: None,
        encoding: Some("utf-8".into()),
        unicode_form: Some("nfc".into()),
        blob_id: None,
        verified_at: None,
    }
}

/// A canonical `JobRecord` fixture ready to be stored.
pub fn sample_job_record() -> storage::repository::JobRecord {
    use domain::ids::JobId;
    let now = Timestamp::now().to_string();
    storage::repository::JobRecord {
        id: JobId::new().to_string(),
        kind: "system.noop_test".into(),
        payload_json: "{}".into(),
        idempotency_key: Some("noop-1".into()),
        state: "Queued".into(),
        priority: 0,
        attempts: 0,
        max_attempts: 5,
        available_at: now.clone(),
        lease_owner: None,
        lease_expires_at: None,
        checkpoint_json: None,
        cancel_requested: false,
        created_by: PrincipalId::new().to_string(),
    }
}

/// A canonical `ProvenanceRecord` fixture with full metadata.
pub fn sample_provenance_record() -> provenance::ProvenanceRecord {
    provenance::ProvenanceRecord {
        id: domain::ProvenanceId::new(),
        layer: DataLayer::Scholarly,
        subject: SubjectRef("urn:qai:test:ayah:1:1".into()),
        attribution: provenance::Attribution::Scholar {
            name: "Test Scholar".into(),
            school: Some("test".into()),
            work: None,
            edition: None,
        },
        source_version_id: Some(SourceVersionId::new()),
        source_location: None,
        trust_level: TrustLevel::ScholarReviewed,
        verification: VerificationStatus::Verified,
        confidence: Some(0.8),
        versions: DerivationVersions {
            source_version_id: SourceVersionId::new(),
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

// ─── MockAuditRepo ───────────────────────────────────────────

/// In-memory audit repository for test suites.
///
/// Implements [`audit::AuditRepository`] backed by a `Vec<AuditEvent>`.
#[derive(Debug, Default)]
pub struct MockAuditRepo {
    events: Vec<audit::AuditEvent>,
}

impl MockAuditRepo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn all(&self) -> &[audit::AuditEvent] {
        &self.events
    }

    /// Replace the `chain_hash` of the event at `sequence` with a broken value.
    pub fn tamper(&mut self, sequence: u64) {
        if let Some(ev) = self.events.iter_mut().find(|e| e.sequence == sequence) {
            ev.chain_hash = ContentHash {
                algorithm: HashAlgorithm::Sha256,
                hex: "ff".repeat(32),
            };
        }
    }
}

#[async_trait]
impl audit::AuditRepository for MockAuditRepo {
    async fn append(&mut self, event: audit::AuditEvent) -> Result<(), audit::AuditError> {
        if let Some(pos) = self.events.iter().position(|e| e.sequence == event.sequence) {
            self.events[pos] = event;
            return Ok(());
        }
        self.events.push(event);
        Ok(())
    }

    async fn list_by_subject(
        &self,
        subject_urn: &SubjectRef,
    ) -> Result<Vec<audit::AuditEvent>, audit::AuditError> {
        Ok(self
            .events
            .iter()
            .filter(|e| e.subject.0 == subject_urn.0)
            .cloned()
            .collect())
    }

    async fn list_by_sequence(
        &self,
        from: u64,
        to: Option<u64>,
    ) -> Result<Vec<audit::AuditEvent>, audit::AuditError> {
        Ok(self
            .events
            .iter()
            .filter(|e| e.sequence >= from && to.map(|t| e.sequence <= t).unwrap_or(true))
            .cloned()
            .collect())
    }

    async fn verify_chain(&self) -> Result<audit::ChainVerificationResult, audit::AuditError> {
        Ok(audit::ChainVerificationResult {
            valid: true,
            expected_next_sequence: self.events.len() as u64 + 1,
            expected_next_hash: ContentHash {
                algorithm: HashAlgorithm::Sha256,
                hex: String::new(),
            },
            gaps: Vec::new(),
            tampered_events: Vec::new(),
        })
    }

    async fn latest_sequence(&self) -> Result<u64, audit::AuditError> {
        Ok(self.events.iter().map(|e| e.sequence).max().unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use audit::AuditRepository as _;

    #[tokio::test]
    async fn mock_audit_repo_appends_and_tampers() {
        let mut repo = MockAuditRepo::new();
        let ev = audit::AuditEvent {
            id: domain::AuditEventId::new(),
            sequence: 1,
            occurred_at: Timestamp::now(),
            actor: audit::Actor::System { name: "testkit".into() },
            action: audit::AuditAction::ConfigChange,
            subject: SubjectRef("urn:qai:config".into()),
            outcome: audit::AuditOutcome::Allowed,
            reason: None,
            before: None,
            after: Some(serde_json::json!({"level": "info"})),
            request_id: None,
            prev_chain_hash: ContentHash {
                algorithm: HashAlgorithm::Sha256,
                hex: "00".repeat(32),
            },
            chain_hash: ContentHash {
                algorithm: HashAlgorithm::Sha256,
                hex: "aa".repeat(32),
            },
        };
        repo.append(ev.clone()).await.unwrap();
        repo.tamper(ev.sequence);
        assert_eq!(repo.all().len(), 1);
        assert_eq!(repo.latest_sequence().await.unwrap(), 1);
        assert_eq!(repo.all()[0].chain_hash.hex, "ff".repeat(32));
    }
}
