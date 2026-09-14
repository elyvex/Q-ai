//! `integrity_audit` — audit persistence, append-only triggers, and chain
//! verification at the database level (AC-P0-13).

mod common;

use storage::Database as _;
use storage::repository::AuditEvent;

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
async fn audit_log_is_append_only() {
    let fx = common::fixture().await;
    let mut uow = fx.db.write().await.unwrap();
    uow.audit().append(event("a1", 1, &"aa".repeat(32), &"00".repeat(32))).await.unwrap();
    uow.commit().await.unwrap();

    let pool = common::rw_pool(&fx.path).await;
    let update = sqlx::query("UPDATE audit_events SET reason = 'x' WHERE id = 'a1'")
        .execute(&pool)
        .await
        .unwrap_err()
        .to_string();
    assert!(update.contains("QAI-AUD-0001"), "expected append-only update guard, got {update}");

    let delete = sqlx::query("DELETE FROM audit_events WHERE id = 'a1'")
        .execute(&pool)
        .await
        .unwrap_err()
        .to_string();
    assert!(delete.contains("QAI-AUD-0002"), "expected append-only delete guard, got {delete}");
}

#[tokio::test]
async fn structural_chain_verification_detects_tampering() {
    let fx = common::fixture().await;
    let zero = "00".repeat(32);

    // Two linked events: seq2.prev == seq1.chain.
    let mut uow = fx.db.write().await.unwrap();
    let c1 = "11".repeat(32);
    let c2 = "22".repeat(32);
    uow.audit().append(event("e1", 1, &c1, &zero)).await.unwrap();
    uow.audit().append(event("e2", 2, &c2, &c1)).await.unwrap();
    uow.commit().await.unwrap();

    // Structural verification passes for a well-formed chain.
    let mut uow = fx.db.write().await.unwrap();
    let ok = uow.audit().verify_chain().await.unwrap();
    assert!(ok.valid, "well-formed chain must verify: {ok:?}");
    uow.rollback().await.unwrap();

    // Tampering via UPDATE/DELETE is impossible (covered above), so simulate a
    // tampered ledger by appending a row whose predecessor link is wrong.
    let mut uow = fx.db.write().await.unwrap();
    uow.audit().append(event("e3", 3, &"33".repeat(32), &"99".repeat(32))).await.unwrap();
    uow.commit().await.unwrap();

    let mut uow = fx.db.write().await.unwrap();
    let bad = uow.audit().verify_chain().await.unwrap();
    assert!(!bad.valid, "a broken predecessor link must fail verification");
    uow.rollback().await.unwrap();
}
