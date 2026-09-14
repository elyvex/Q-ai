//! `outbox_relay` — the generic relay claims pending events and marks them
//! dispatched (T64), reusing the outbox lease semantics.

mod common;

use storage::Database as _;
use storage::UnitOfWork;
use storage::workflows::{record_source_activation, relay_outbox_once};

#[tokio::test]
async fn relay_dispatches_pending_events() {
    let fx = common::fixture().await;
    common::seed_source_version(&fx, "src-r1", "ver-r1", "Indexing").await;
    common::seed_source_version(&fx, "src-r2", "ver-r2", "Indexing").await;

    // Produce two pending outbox events.
    let mut uow = fx.db.write().await.unwrap();
    record_source_activation(&mut *uow, "ver-r1", "quran:test", "urn:qai:source:src-r1")
        .await
        .unwrap();
    uow.commit().await.unwrap();

    let mut uow = fx.db.write().await.unwrap();
    record_source_activation(&mut *uow, "ver-r2", "quran:test", "urn:qai:source:src-r2")
        .await
        .unwrap();
    uow.commit().await.unwrap();
    assert_eq!(common::outbox_count(&fx.path, "Pending").await, 2);

    // Relay both.
    let mut uow = fx.db.write().await.unwrap();
    let relayed = relay_outbox_once(&mut *uow, "worker-1", 10).await.unwrap();
    assert_eq!(relayed, 2);
    uow.commit().await.unwrap();

    assert_eq!(common::outbox_count(&fx.path, "Pending").await, 0);
    assert_eq!(common::outbox_count(&fx.path, "Dispatched").await, 2);
}

#[tokio::test]
async fn relay_is_a_noop_with_no_pending_events() {
    let fx = common::fixture().await;
    let mut uow = fx.db.write().await.unwrap();
    let relayed = relay_outbox_once(&mut *uow, "worker-1", 10).await.unwrap();
    assert_eq!(relayed, 0);
    uow.commit().await.unwrap();
}
