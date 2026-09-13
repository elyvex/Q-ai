//! Repository traits for sources, provenance, audit, jobs, settings.
//!
//! Each trait defines the minimal async interface required by
//! the [`UnitOfWork`] in `lib.rs`. Concrete implementations live
//! in `storage-sqlite` or future backend crates.
//!
//! # Phase 0 scope
//!
//! These are trait definitions only. Method bodies are stubs
//! returning `Err(StorageError::StorageUnavailable)` until the
//! SQLite implementation lands in `storage-sqlite`.

use async_trait::async_trait;

use crate::error::StorageError;

// ─── Source Repository ───────────────────────────────────────────────

/// Repository for source catalog records.
///
/// Manages `sources`, `source_versions`, `source_files`,
/// `source_genealogy`, and `approvals` tables.
#[async_trait]
pub trait SourceRepository: Send {
    /// Retrieve a source by its ID.
    async fn get(&self, _id: &str) -> Result<Option<SourceRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Insert a new source version.
    async fn insert_version(&mut self, _version: SourceVersionRow) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Transition a source version's state.
    async fn transition_state(
        &mut self,
        _source_version_id: &str,
        _from: &str,
        _to: &str,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List all versions for a source.
    async fn list_versions(&self, _source_id: &str) -> Result<Vec<SourceVersionRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Record a state transition event.
    async fn record_transition(
        &mut self,
        _transition: StateTransitionRow,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }
}

/// A row in the `sources` table.
#[derive(Debug, Clone)]
pub struct SourceRow {
    pub id: String,
    pub title: String,
    pub content_type: String,
    pub language: Option<String>,
    pub created_at: String,
}

/// A row in the `source_versions` table.
#[derive(Debug, Clone)]
pub struct SourceVersionRow {
    pub id: String,
    pub source_id: String,
    pub version: String,
    pub state: String,
    pub trust_level: String,
    pub license_status: String,
    pub content_hash: Option<String>,
    pub manifest_blob_id: Option<String>,
}

/// A row in the `source_state_transitions` table.
#[derive(Debug, Clone)]
pub struct StateTransitionRow {
    pub id: String,
    pub source_version_id: String,
    pub from_state: Option<String>,
    pub to_state: String,
    pub actor_id: Option<String>,
    pub reason: Option<String>,
    pub occurred_at: String,
}

// ─── Provenance Repository ───────────────────────────────────────────

/// Repository for provenance records.
///
/// Manages `provenance_records`, `review_queue`, and
/// canonical write guard operations.
#[async_trait]
pub trait ProvenanceRepository: Send {
    /// Insert a new provenance record.
    async fn insert(&mut self, _record: ProvenanceRecord) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Retrieve a provenance record by ID.
    async fn get(&self, _id: &str) -> Result<Option<ProvenanceRecord>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List records for a subject.
    async fn list_by_subject(
        &self,
        _subject_urn: &str,
    ) -> Result<Vec<ProvenanceRecord>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Record a review decision.
    async fn record_review(
        &mut self,
        _review: ReviewRecord,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }
}

/// A provenance record as stored in the database.
#[derive(Debug, Clone)]
pub struct ProvenanceRecord {
    pub id: String,
    pub layer: String,
    pub subject_urn: String,
    pub attribution_kind: String,
    pub attribution_json: String,
    pub source_version_id: Option<String>,
    pub trust_level: String,
    pub verification_status: String,
    pub confidence: Option<f32>,
    pub versions_json: String,
    pub created_by: String,
}

/// A review queue entry.
#[derive(Debug, Clone)]
pub struct ReviewRecord {
    pub id: String,
    pub provenance_id: String,
    pub queue: String,
    pub evidence_json: String,
    pub state: String,
    pub decided_by: Option<String>,
    pub decided_at: Option<String>,
    pub decision_note: Option<String>,
    pub created_at: String,
}

// ─── Audit Repository ────────────────────────────────────────────────

/// Repository for audit events.
///
/// Provides append-only writes and hash-chain verification.
#[async_trait]
pub trait AuditRepository: Send {
    /// Append a new audit event to the chain.
    async fn append(&mut self, _event: AuditEvent) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Retrieve events for a subject.
    async fn list_by_subject(
        &self,
        _subject_urn: &str,
    ) -> Result<Vec<AuditEvent>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Verify the integrity of the audit chain.
    async fn verify_chain(&self) -> Result<ChainVerificationResult, StorageError> {
        Err(StorageError::StorageUnavailable)
    }
}

/// An audit event as stored in the database.
#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub id: String,
    pub sequence: u64,
    pub occurred_at: String,
    pub actor_kind: String,
    pub actor_id: Option<String>,
    pub action: String,
    pub subject_urn: String,
    pub outcome: String,
    pub reason: Option<String>,
    pub before_json: Option<String>,
    pub after_json: Option<String>,
    pub request_id: Option<String>,
    pub prev_chain_hash: String,
    pub chain_hash: String,
}

/// Result of a chain verification.
#[derive(Debug, Clone)]
pub struct ChainVerificationResult {
    pub valid: bool,
    pub expected_next_sequence: u64,
    pub expected_next_hash: String,
    pub gaps: Vec<u64>,
}

// ─── Job Repository ──────────────────────────────────────────────────

/// Repository for background jobs.
///
/// Manages the `jobs`, `job_events`, `corpus_generations`,
/// `outbox_events`, and `tombstones` tables.
#[async_trait]
pub trait JobRepository: Send {
    /// Enqueue a new job.
    async fn enqueue(&mut self, _job: JobRecord) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Claim a job for processing (lease acquisition).
    async fn claim(&mut self, _job_id: &str, _owner: &str) -> Result<Option<JobRecord>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Finish a job with a final state.
    async fn finish(
        &mut self,
        _job_id: &str,
        _state: &str,
        _result: Option<String>,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Cancel a running job.
    async fn cancel(&mut self, _job_id: &str) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Update job progress and checkpoint.
    async fn checkpoint(
        &mut self,
        _job_id: &str,
        _progress: Option<String>,
        _checkpoint: Option<String>,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Reap expired leases, returning job IDs to reclaim.
    async fn reap_expired_leases(&mut self) -> Result<Vec<String>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }
}

/// A job record as stored in the database.
#[derive(Debug, Clone)]
pub struct JobRecord {
    pub id: String,
    pub kind: String,
    pub payload_json: String,
    pub idempotency_key: Option<String>,
    pub state: String,
    pub priority: i32,
    pub attempts: u32,
    pub max_attempts: u32,
    pub available_at: String,
    pub lease_owner: Option<String>,
    pub lease_expires_at: Option<String>,
    pub checkpoint_json: Option<String>,
    pub cancel_requested: bool,
    pub created_by: String,
}

// ─── Settings Repository ─────────────────────────────────────────────

/// Repository for application settings.
///
/// Manages the `settings` table and config provenance.
#[async_trait]
pub trait SettingsRepository: Send {
    /// Get a setting value by key.
    async fn get(&self, _key: &str) -> Result<Option<SettingRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Set or update a setting.
    async fn set(
        &mut self,
        _key: &str,
        _value_json: &str,
        _origin: &str,
        _updated_by: Option<&str>,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List all settings.
    async fn list(&self) -> Result<Vec<SettingRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }
}

/// A setting row as stored in the database.
#[derive(Debug, Clone)]
pub struct SettingRow {
    pub key: String,
    pub value_json: String,
    pub origin: String,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_row_roundtrip() {
        let row = SourceRow {
            id: "test".into(),
            title: "Test".into(),
            content_type: "quran_edition".into(),
            language: Some("ar".into()),
            created_at: "2026-01-01T00:00:00Z".into(),
        };
        assert_eq!(row.id, "test");
    }
}
