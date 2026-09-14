//! Transaction-scoped write workflows (D0.18, ADR-0702 §3).
//!
//! These helpers make the outbox invariant **impossible to violate by
//! construction** for the projection-relevant operations Phase 0 knows about:
//! each allocates a generation, enqueues its outbox row, and performs the
//! authoritative change through the *same* [`UnitOfWork`]. The caller commits
//! once; the outbox row and the change live or die together in one SQLite
//! transaction.
//!
//! # Ordering
//!
//! Deactivation writes the tombstone **before** the state transition, so the
//! tombstone is durably recorded before the transition can be observed
//! (AC-P0-25, ADR-0702 §9).

use domain::TombstoneId;

use crate::UnitOfWork;
use crate::error::StorageError;
use crate::repository::{NewOutboxEvent, ProvenanceRecord, TombstoneRow};

fn now_rfc3339() -> String {
    // `domain::Timestamp` already renders RFC3339 UTC.
    domain::Timestamp::now().to_string()
}

/// Activate a source version: allocate a generation, enqueue the outbox event,
/// then flip `Indexing → Active` — all in the caller's transaction.
pub async fn record_source_activation(
    uow: &mut dyn UnitOfWork,
    source_version_id: &str,
    scope: &str,
    subject_urn: &str,
) -> Result<String, StorageError> {
    let generation = uow
        .outbox()
        .allocate_generation(scope, "source_activated")
        .await?;
    uow.outbox()
        .enqueue(NewOutboxEvent {
            scope: scope.to_string(),
            target_generation: generation.id.clone(),
            operation: "source_activated".to_string(),
            subject_urn: subject_urn.to_string(),
            idempotency_key: format!("source_activated:{source_version_id}"),
            payload_json: "{}".to_string(),
        })
        .await?;
    uow.sources()
        .transition_state(source_version_id, "Indexing", "Active")
        .await?;
    Ok(generation.id)
}

/// Deactivate a source version: write a tombstone, allocate a generation,
/// enqueue the outbox event, then flip `Active → Deprecated` — all in the
/// caller's transaction. Returns the tombstone id.
pub async fn record_source_deactivation(
    uow: &mut dyn UnitOfWork,
    source_version_id: &str,
    scope: &str,
    subject_urn: &str,
    reason: &str,
) -> Result<String, StorageError> {
    let tombstone_id = TombstoneId::new().to_string();
    // Tombstone first: current policy blocks retrieval immediately even while
    // physical cleanup is pending (ADR-0702 §9).
    uow.outbox()
        .insert_tombstone(TombstoneRow {
            id: tombstone_id.clone(),
            subject_urn: subject_urn.to_string(),
            reason: "deactivated".to_string(),
            effective_at: now_rfc3339(),
            created_by: None,
            propagation_state: "Pending".to_string(),
        })
        .await?;

    let generation = uow
        .outbox()
        .allocate_generation(scope, "source_deactivated")
        .await?;
    uow.outbox()
        .enqueue(NewOutboxEvent {
            scope: scope.to_string(),
            target_generation: generation.id.clone(),
            operation: "source_deactivated".to_string(),
            subject_urn: subject_urn.to_string(),
            idempotency_key: format!("source_deactivated:{source_version_id}:{}", now_rfc3339()),
            payload_json: serde_json::json!({ "reason": reason }).to_string(),
        })
        .await?;
    uow.sources()
        .transition_state(source_version_id, "Active", "Deprecated")
        .await?;
    Ok(tombstone_id)
}

/// Record a provenance write: insert the record, allocate a generation, and
/// enqueue the outbox event — all in the caller's transaction.
pub async fn record_provenance_write(
    uow: &mut dyn UnitOfWork,
    record: ProvenanceRecord,
    scope: &str,
) -> Result<String, StorageError> {
    let subject_urn = record.subject_urn.clone();
    uow.provenance().insert(record).await?;
    let generation = uow
        .outbox()
        .allocate_generation(scope, "provenance_written")
        .await?;
    uow.outbox()
        .enqueue(NewOutboxEvent {
            scope: scope.to_string(),
            target_generation: generation.id.clone(),
            operation: "provenance_written".to_string(),
            subject_urn,
            idempotency_key: format!("provenance_written:{}", generation.id),
            payload_json: "{}".to_string(),
        })
        .await?;
    Ok(generation.id)
}

/// Relay one batch of pending outbox events: claim up to `limit` for `owner`
/// and mark them dispatched. Returns the number of events relayed.
///
/// Phase 0 has no consumers (no FTS/vector/graph exist); the relay records
/// dispatch so the contract, table, and "commit implies durable outbox row"
/// guarantee exist before Phase 2 needs them (T64).
pub async fn relay_outbox_once(
    uow: &mut dyn UnitOfWork,
    owner: &str,
    limit: u32,
) -> Result<usize, StorageError> {
    let claimed = uow.outbox().claim_pending(owner, limit).await?;
    let count = claimed.len();
    for event in &claimed {
        uow.outbox().mark_dispatched(&event.id).await?;
    }
    Ok(count)
}
