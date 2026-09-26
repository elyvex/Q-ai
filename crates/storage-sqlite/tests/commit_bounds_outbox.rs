//! `commit_bounds_outbox` — a projection-relevant change and its outbox row
//! commit atomically (AC-P0-23).
//!
//! Proves the guarantee is "impossible by construction, not by retry": the
//! outbox insert shares the SQLite transaction with the authoritative change,
//! so a rollback/fault between them discards both.

mod common;

use storage::Database as _;
use storage::repository::AuditEvent;
use storage::workflows::record_source_activation;

fn audit_row(id: &str, sequence: u64, subject_urn: &str, prev: &str, chain: &str) -> AuditEvent {
    AuditEvent {
        id: id.into(),
        sequence,
        occurred_at: common::now(),
        actor_kind: "system".into(),
        actor_id: None,
        action: "source_activated".into(),
        subject_urn: subject_urn.into(),
        outcome: "allowed".into(),
        reason: None,
        before_json: None,
        after_json: None,
        request_id: None,
        prev_chain_hash: prev.into(),
        chain_hash: chain.into(),
    }
}

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
    assert_eq!(common::source_state(&fx.path, "ver-1").await.as_deref(), Some("Active"));
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
        uow.sources().transition_state("ver-3", "Indexing", "Active").await.unwrap();
        // Deliberately omit the outbox enqueue; drop without commit.
    }

    assert_eq!(common::source_state(&fx.path, "ver-3").await.as_deref(), Some("Indexing"));
    assert_eq!(common::outbox_count(&fx.path, "Pending").await, 0);
}

/// Same-UoW audit case (D-10): the activation workflow and its audit row
/// share one transaction, so one commit makes the domain change, the outbox
/// row, the generation, and the audit row visible together.
#[tokio::test]
async fn audited_activation_commits_domain_outbox_and_audit_together() {
    let fx = common::fixture().await;
    common::seed_source_version(&fx, "src-audit", "ver-audit", "Indexing").await;
    let subject = "urn:qai:source:src-audit";

    let mut uow = fx.db.write().await.unwrap();
    record_source_activation(&mut *uow, "ver-audit", "quran:test", subject).await.unwrap();
    uow.audit()
        .append(audit_row("audit-1", 1, subject, &"00".repeat(32), &"aa".repeat(32)))
        .await
        .unwrap();
    uow.commit().await.unwrap();

    assert_eq!(common::source_state(&fx.path, "ver-audit").await.as_deref(), Some("Active"));
    assert_eq!(common::outbox_count(&fx.path, "Pending").await, 1);
    assert_eq!(common::generation_numbers(&fx.path, "quran:test").await, vec![1]);
    let mut uow = fx.db.write().await.unwrap();
    let rows = uow.audit().list_by_subject(subject).await.unwrap();
    assert_eq!(rows.len(), 1, "the audit row commits with the change");
    assert_eq!(rows[0].sequence, 1);
    uow.rollback().await.unwrap();
}

/// Same-UoW audit rollback (D-10): staging the activation, its outbox row,
/// and its audit row, then rolling back, discards all three — no partial
/// trust record survives.
#[tokio::test]
async fn audited_activation_rolled_back_discards_outbox_and_audit() {
    let fx = common::fixture().await;
    common::seed_source_version(&fx, "src-audit-rb", "ver-audit-rb", "Indexing").await;
    let subject = "urn:qai:source:src-audit-rb";

    let mut uow = fx.db.write().await.unwrap();
    record_source_activation(&mut *uow, "ver-audit-rb", "quran:test", subject).await.unwrap();
    uow.audit()
        .append(audit_row("audit-rb-1", 1, subject, &"00".repeat(32), &"aa".repeat(32)))
        .await
        .unwrap();
    uow.rollback().await.unwrap();

    assert_eq!(
        common::source_state(&fx.path, "ver-audit-rb").await.as_deref(),
        Some("Indexing"),
        "domain change must not persist without commit"
    );
    assert_eq!(common::outbox_count(&fx.path, "Pending").await, 0);
    assert!(common::generation_numbers(&fx.path, "quran:test").await.is_empty());
    let mut uow = fx.db.write().await.unwrap();
    let rows = uow.audit().list_by_sequence(0, None).await.unwrap();
    assert!(rows.is_empty(), "no audit row may persist without the committed change");
    uow.rollback().await.unwrap();
}
