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
pub trait SourceRepository: Send + Sync {
    /// Retrieve a source by its ID.
    async fn get(&self, _id: &str) -> Result<Option<SourceRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Insert a new source catalog entry.
    async fn insert_source(&mut self, _source: SourceRow) -> Result<(), StorageError> {
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

    /// Record a human approval decision.
    async fn insert_approval(&mut self, _approval: ApprovalRow) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Fetch an approval by id.
    async fn get_approval(&self, _id: &str) -> Result<Option<ApprovalRow>, StorageError> {
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

/// A row in the `approvals` table.
#[derive(Debug, Clone)]
pub struct ApprovalRow {
    pub id: String,
    pub subject_urn: String,
    pub kind: String,
    pub requested_by: Option<String>,
    pub decided_by: Option<String>,
    pub decision: Option<String>,
    pub request_payload: String,
    pub decision_note: Option<String>,
    pub requested_at: String,
    pub decided_at: Option<String>,
}

// ─── Provenance Repository ───────────────────────────────────────────

/// Repository for provenance records.
///
/// Manages `provenance_records`, `review_queue`, and
/// canonical write guard operations.
#[async_trait]
pub trait ProvenanceRepository: Send + Sync {
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
    async fn record_review(&mut self, _review: ReviewRecord) -> Result<(), StorageError> {
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
pub trait AuditRepository: Send + Sync {
    /// Append a new audit event to the chain.
    async fn append(&mut self, _event: AuditEvent) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Retrieve events for a subject.
    async fn list_by_subject(&self, _subject_urn: &str) -> Result<Vec<AuditEvent>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List events in a sequence range, ordered ascending.
    async fn list_by_sequence(
        &self,
        _from: u64,
        _to: Option<u64>,
    ) -> Result<Vec<AuditEvent>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// The highest allocated sequence, or 0 when the log is empty.
    async fn latest_sequence(&self) -> Result<u64, StorageError> {
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
pub trait JobRepository: Send + Sync {
    /// Enqueue a new job.
    async fn enqueue(&mut self, _job: JobRecord) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Claim a job for processing (lease acquisition).
    async fn claim(
        &mut self,
        _job_id: &str,
        _owner: &str,
    ) -> Result<Option<JobRecord>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Claim the next available job (Queued/Interrupted/Checkpointed, due now),
    /// leasing it for `lease_seconds`.
    async fn claim_next(
        &mut self,
        _owner: &str,
        _lease_seconds: u64,
    ) -> Result<Option<JobRecord>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Re-queue a job after a delay (retry/backoff).
    async fn reschedule(
        &mut self,
        _job_id: &str,
        _delay_seconds: u64,
        _reason: Option<String>,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Fetch a job by id.
    async fn get(&self, _job_id: &str) -> Result<Option<JobRecord>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Renew a job's lease if `owner` still holds it.
    async fn heartbeat(
        &mut self,
        _job_id: &str,
        _owner: &str,
        _lease_seconds: u64,
    ) -> Result<bool, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Count jobs in a given state.
    async fn count_by_state(&self, _state: &str) -> Result<i64, StorageError> {
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
pub trait SettingsRepository: Send + Sync {
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

// ─── Outbox / generations / tombstones (D0.18, ADR-0702) ─────────────

/// Repository for the cross-store consistency primitives.
///
/// Manages `corpus_generations`, `outbox_events`, and `tombstones`. Every
/// mutation here participates in the caller's [`UnitOfWork`], so a
/// projection-relevant change and its outbox row commit atomically.
#[async_trait]
pub trait OutboxRepository: Send + Sync {
    /// Allocate the next generation number for `scope` (monotonic, never regresses).
    async fn allocate_generation(
        &mut self,
        _scope: &str,
        _reason: &str,
    ) -> Result<GenerationRow, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// The highest allocated generation for `scope`, if any.
    async fn current_generation(
        &self,
        _scope: &str,
    ) -> Result<Option<GenerationRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Enqueue an outbox event. Duplicate `(operation, idempotency_key)` yields
    /// [`StorageError::Conflict`] (idempotent no-op at the call site).
    async fn enqueue(&mut self, _event: NewOutboxEvent) -> Result<String, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Claim up to `limit` pending events for `owner` (lease acquisition).
    async fn claim_pending(
        &mut self,
        _owner: &str,
        _limit: u32,
    ) -> Result<Vec<OutboxEventRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Mark a claimed event dispatched.
    async fn mark_dispatched(&mut self, _id: &str) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Mark a claimed event failed (dead-letter candidate).
    async fn mark_failed(&mut self, _id: &str, _reason: &str) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List events in a given state (oldest first).
    async fn list_by_state(&self, _state: &str) -> Result<Vec<OutboxEventRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Insert a tombstone (write-before-visibility, ADR-0702 §9).
    async fn insert_tombstone(&mut self, _tombstone: TombstoneRow) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List tombstones that have not yet propagated.
    async fn list_pending_tombstones(&self) -> Result<Vec<TombstoneRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }
}

/// A row in `corpus_generations`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationRow {
    pub id: String,
    pub scope: String,
    pub number: u64,
    pub reason: String,
    pub created_at: String,
}

/// A new outbox event to persist.
#[derive(Debug, Clone)]
pub struct NewOutboxEvent {
    pub scope: String,
    pub target_generation: String,
    pub operation: String,
    pub subject_urn: String,
    pub idempotency_key: String,
    pub payload_json: String,
}

/// A row in `outbox_events`.
#[derive(Debug, Clone)]
pub struct OutboxEventRow {
    pub id: String,
    pub scope: String,
    pub target_generation: String,
    pub operation: String,
    pub subject_urn: String,
    pub idempotency_key: String,
    pub payload_json: String,
    pub state: String,
    pub lease_owner: Option<String>,
    pub lease_expires_at: Option<String>,
    pub attempts: u32,
    pub created_at: String,
    pub dispatched_at: Option<String>,
}

/// A row in `tombstones`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TombstoneRow {
    pub id: String,
    pub subject_urn: String,
    pub reason: String,
    pub effective_at: String,
    pub created_by: Option<String>,
    pub propagation_state: String,
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
