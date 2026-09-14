//! `outbox_idempotency` — duplicate `(operation, idempotency_key)` yields
//! exactly one event (AC-P0-23).

mod common;

use storage::Database as _;
use storage::error::StorageError;
use storage::repository::NewOutboxEvent;

fn event(key: &str) -> NewOutboxEvent {
    NewOutboxEvent {
        scope: "quran:test".into(),
        target_generation: String::new(), // replaced per-test after allocation
        operation: "source_activated".into(),
        subject_urn: "urn:qai:source:src-1".into(),
        idempotency_key: key.into(),
        payload_json: "{}".into(),
    }
}

#[tokio::test]
async fn duplicate_enqueue_in_one_transaction_conflicts() {
    let fx = common::fixture().await;
    let mut uow = fx.db.write().await.unwrap();
    let generation = uow.outbox().allocate_generation("quran:test", "test").await.unwrap();

    let mut first = event("dup-1");
    first.target_generation = generation.id.clone();
    uow.outbox().enqueue(first.clone()).await.unwrap();

    let mut second = event("dup-1");
    second.target_generation = generation.id.clone();
    let err = uow.outbox().enqueue(second).await.unwrap_err();
    assert!(matches!(err, StorageError::Conflict), "expected Conflict, got {err:?}");

    uow.commit().await.unwrap();
    assert_eq!(common::outbox_count(&fx.path, "Pending").await, 1);
}

#[tokio::test]
async fn duplicate_enqueue_across_transactions_conflicts() {
    let fx = common::fixture().await;

    // First transaction allocates and enqueues.
    let mut uow = fx.db.write().await.unwrap();
    let generation = uow.outbox().allocate_generation("quran:test", "test").await.unwrap();
    let mut first = event("dup-2");
    first.target_generation = generation.id.clone();
    uow.outbox().enqueue(first).await.unwrap();
    uow.commit().await.unwrap();

    // Second transaction reuses the same idempotency key.
    let mut uow = fx.db.write().await.unwrap();
    let gen2 = uow.outbox().allocate_generation("quran:test", "test").await.unwrap();
    let mut second = event("dup-2");
    second.target_generation = gen2.id;
    let err = uow.outbox().enqueue(second).await.unwrap_err();
    assert!(matches!(err, StorageError::Conflict));
    uow.rollback().await.unwrap();

    assert_eq!(common::outbox_count(&fx.path, "Pending").await, 1);
}

#[tokio::test]
async fn distinct_keys_both_persist() {
    let fx = common::fixture().await;
    let mut uow = fx.db.write().await.unwrap();
    let generation = uow.outbox().allocate_generation("quran:test", "test").await.unwrap();

    let mut a = event("key-a");
    a.target_generation = generation.id.clone();
    let mut b = event("key-b");
    b.target_generation = generation.id;
    uow.outbox().enqueue(a).await.unwrap();
    uow.outbox().enqueue(b).await.unwrap();
    uow.commit().await.unwrap();

    assert_eq!(common::outbox_count(&fx.path, "Pending").await, 2);
}
