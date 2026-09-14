//! `tombstone_before_visibility` — deactivating a source writes a tombstone in
//! the same transaction as the state transition (AC-P0-25).
//!
//! Readers that can observe the `Deprecated` state can also observe the
//! tombstone, and a deactivation that fails to transition leaves neither.

mod common;

use storage::Database as _;
use storage::UnitOfWork;
use storage::workflows::record_source_deactivation;

const SUBJECT: &str = "urn:qai:source:src-t";

#[tokio::test]
async fn deactivation_writes_tombstone_with_the_transition() {
    let fx = common::fixture().await;
    common::seed_source_version(&fx, "src-t", "ver-t", "Active").await;

    // Before deactivation: active, no tombstone.
    assert_eq!(
        common::source_state(&fx.path, "ver-t").await.as_deref(),
        Some("Active")
    );
    assert_eq!(common::tombstone_count(&fx.path, SUBJECT).await, 0);

    let mut uow = fx.db.write().await.unwrap();
    let tombstone_id =
        record_source_deactivation(&mut *uow, "ver-t", "quran:test", SUBJECT, "superseded")
            .await
            .unwrap();
    assert!(!tombstone_id.is_empty());
    uow.commit().await.unwrap();

    // After: deactivated, with a tombstone and a pending outbox event.
    assert_eq!(
        common::source_state(&fx.path, "ver-t").await.as_deref(),
        Some("Deprecated")
    );
    assert_eq!(common::tombstone_count(&fx.path, SUBJECT).await, 1);
    assert_eq!(common::outbox_count(&fx.path, "Pending").await, 1);
}

#[tokio::test]
async fn failed_transition_leaves_no_tombstone() {
    // The version is `Staged`, not `Active`, so the `Active → Deprecated`
    // transition affects zero rows and the helper errors. Nothing the helper
    // wrote — including the tombstone — may persist.
    let fx = common::fixture().await;
    common::seed_source_version(&fx, "src-s", "ver-s", "Staged").await;

    let result = {
        let mut uow = fx.db.write().await.unwrap();
        let result =
            record_source_deactivation(&mut *uow, "ver-s", "quran:test", SUBJECT, "test").await;
        // Drop without commit (the helper errored).
        result
    };

    assert!(result.is_err(), "transition from Staged must fail");
    assert_eq!(common::tombstone_count(&fx.path, SUBJECT).await, 0);
    assert_eq!(common::outbox_count(&fx.path, "Pending").await, 0);
    assert_eq!(
        common::source_state(&fx.path, "ver-s").await.as_deref(),
        Some("Staged")
    );
}
