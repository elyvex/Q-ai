//! `commit_bounds_outbox` — a projection-relevant change and its outbox row
//! commit atomically (AC-P0-23).
//!
//! Proves the guarantee is "impossible by construction, not by retry": the
//! outbox insert shares the SQLite transaction with the authoritative change,
//! so a rollback/fault between them discards both.

mod common;

use storage::Database as _;
use storage::UnitOfWork;
use storage::workflows::record_source_activation;

#[tokio::test]
async fn committed_change_and_outbox_are_durable_together() {
    let fx = common::fixture().await;
    common::seed_source_version(&fx, "src-1", "ver-1", "Indexing").await;

    let mut uow = fx.db.write().await.unwrap();
    record_source_activation(&mut *uow, "ver-1", "quran:test", "urn:qai:source:src-1")
        .await
        .unwrap();
    uow.commit().await.unwrap();

    // Both the state change and its outbox row are visible.
    assert_eq!(
        common::source_state(&fx.path, "ver-1").await.as_deref(),
        Some("Active")
    );
    assert_eq!(common::outbox_count(&fx.path, "Pending").await, 1);
    assert_eq!(common::generation_numbers(&fx.path, "quran:test").await, vec![1]);
}

#[tokio::test]
async fn a_change_rolled_back_never_leaves_an_outbox_row() {
    let fx = common::fixture().await;
    common::seed_source_version(&fx, "src-2", "ver-2", "Indexing").await;

    // Simulate a crash between the authoritative change and the outbox write:
    // perform both, then drop the unit of work without committing.
    {
        let mut uow = fx.db.write().await.unwrap();
        record_source_activation(&mut *uow, "ver-2", "quran:test", "urn:qai:source:src-2")
            .await
            .unwrap();
        // no commit — dropped here; the transaction rolls back
    }

    assert_eq!(
        common::source_state(&fx.path, "ver-2").await.as_deref(),
        Some("Indexing"),
        "state change must not persist without commit"
    );
    assert_eq!(
        common::outbox_count(&fx.path, "Pending").await,
        0,
        "no outbox row may persist without the committed change"
    );
    assert!(common::generation_numbers(&fx.path, "quran:test").await.is_empty());
}

#[tokio::test]
async fn witness_change_without_outbox_rolls_back() {
    // Manual writes to the same unit of work show that the change and the
    // outbox row live in one transaction: transitioning without enqueueing,
    // then rolling back, discards the transition too.
    let fx = common::fixture().await;
    common::seed_source_version(&fx, "src-3", "ver-3", "Indexing").await;

    {
        let mut uow = fx.db.write().await.unwrap();
        uow.sources()
            .transition_state("ver-3", "Indexing", "Active")
            .await
            .unwrap();
        // Deliberately omit the outbox enqueue; drop without commit.
    }

    assert_eq!(
        common::source_state(&fx.path, "ver-3").await.as_deref(),
        Some("Indexing")
    );
    assert_eq!(common::outbox_count(&fx.path, "Pending").await, 0);
}
