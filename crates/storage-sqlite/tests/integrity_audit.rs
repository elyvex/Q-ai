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

/// Tampered-sequence case (D-12, T-02-RECOVERY): a ledger whose third event
/// links to the wrong predecessor fails verification, and the offending
/// sequence is identifiable from storage reads — the first event whose
/// predecessor link breaks the chain. Sequence-level attribution and the
/// operator remedy (`tampered_sequences`, `QAI-AUD-0005`, `qai audit verify`)
/// live one layer up in the persisted verifier
/// (`application::audit_bridge::{verify_persisted_audit, diagnose_invalid_audit}`,
/// proven in `application --test phase1_foundation
/// tampered_chain_reports_offending_sequence_with_verify_remedy`), which reads
/// this same persisted state without changing the database.
#[tokio::test]
async fn tampered_sequence_is_identifiable_from_storage_reads() {
    let fx = common::fixture().await;
    let zero = "00".repeat(32);

    let mut uow = fx.db.write().await.unwrap();
    let c1 = "aa".repeat(32);
    let c2 = "bb".repeat(32);
    uow.audit().append(event("t1", 1, &c1, &zero)).await.unwrap();
    uow.audit().append(event("t2", 2, &c2, &c1)).await.unwrap();
    uow.commit().await.unwrap();

    // Tampered ledger: sequence 3 links to the wrong predecessor. Appending a
    // bad link is how a tampered ledger presents — UPDATE/DELETE is impossible
    // under the append-only triggers.
    let mut uow = fx.db.write().await.unwrap();
    uow.audit().append(event("t3", 3, &"cc".repeat(32), &"99".repeat(32))).await.unwrap();
    uow.commit().await.unwrap();

    let mut uow = fx.db.write().await.unwrap();
    let report = uow.audit().verify_chain().await.unwrap();
    assert!(!report.valid, "a broken predecessor link must fail verification");
    let events = uow.audit().list_by_sequence(1, None).await.unwrap();
    assert_eq!(events.len(), 3, "row counts alone must not mask the tamper");
    let mut offending = None;
    let mut prev = zero.clone();
    for row in &events {
        if row.prev_chain_hash != prev {
            offending = Some(row.sequence);
            break;
        }
        prev = row.chain_hash.clone();
    }
    assert_eq!(offending, Some(3), "the offending sequence is identifiable");
    uow.rollback().await.unwrap();

    // The persisted rows are unchanged by the reads above: the chain still
    // fails verification on a fresh handle.
    let mut uow = fx.db.write().await.unwrap();
    assert!(!uow.audit().verify_chain().await.unwrap().valid);
    uow.rollback().await.unwrap();
}
