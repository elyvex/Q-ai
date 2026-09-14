//! `backup_restore` — `VACUUM INTO` backup round-trips a populated database,
//! preserving the audit chain (AC-P0-21).

mod common;

use std::path::PathBuf;
use storage::Database as _;
use storage::repository::AuditEvent;
use storage_sqlite::{SqliteDatabase, migrate};

fn migrations_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite")
}

fn event(id: &str, sequence: u64, chain: &str, prev: &str) -> AuditEvent {
    AuditEvent {
        id: id.into(),
        sequence,
        occurred_at: common::now(),
        actor_kind: "system".into(),
        actor_id: None,
        action: "config_change".into(),
        subject_urn: "urn:qai:config".into(),
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
async fn backup_round_trips_and_preserves_the_audit_chain() {
    let fx = common::fixture().await;
    let zero = "00".repeat(32);
    let c1 = "11".repeat(32);
    let c2 = "22".repeat(32);

    // Populate two linked audit events.
    let mut uow = fx.db.write().await.unwrap();
    uow.audit().append(event("e1", 1, &c1, &zero)).await.unwrap();
    uow.audit().append(event("e2", 2, &c2, &c1)).await.unwrap();
    uow.commit().await.unwrap();

    // Consistent snapshot.
    let backup = fx.path.clone() + ".backup.db";
    migrate::backup(&fx.path, &backup).await.unwrap();

    // The snapshot verifies against the migration checksums.
    let report = migrate::verify_checksums(&backup, &migrations_dir()).await.unwrap();
    assert!(report.valid, "backup checksums must verify: {report:?}");

    // Open the snapshot as a fresh database and verify the chain end-to-end.
    let restored = SqliteDatabase::new(&backup, 4, true).await.unwrap();
    let mut uow = restored.write().await.unwrap();
    let chain = uow.audit().verify_chain().await.unwrap();
    assert!(chain.valid, "restored chain must verify: {chain:?}");
    assert_eq!(chain.expected_next_sequence, 3, "both audit events must survive the backup");
    uow.rollback().await.unwrap();
}
