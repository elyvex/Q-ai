//! `phase1_jobs_audit` — job lifecycle audit atomicity at the storage boundary
//! (01-03-03, D-10/D-11).
//!
//! The application adapter (`SqliteJobQueue`) stages each lifecycle audit
//! event in the same unit of work as its job mutation. These cases prove the
//! durable shape that composition relies on, against real SQLite:
//!
//! - a job transition and its lifecycle audit row commit atomically;
//! - an audit-write failure rolls the job transition back with it;
//! - lifecycle rows for one job order by sequence and carry only the
//!   allowlisted metadata (id, kind, state, attempts) — never payload,
//!   checkpoint, or error content;
//! - checkpoint/heartbeat writes stage no audit rows (D-11 enumerates
//!   enqueue, lease, completion, failure, and cancellation).
//!
//! The adapter-level proof that `SqliteJobQueue` actually emits the
//! `job_enqueued / job_leased / job_completed / job_failed / job_retried /
//! job_cancellation_requested / job_cancelled` actions lives in the
//! application job-queue suite, which owns the audit composition.

mod common;

use storage::Database as _;
use storage::error::StorageError;
use storage::repository::{AuditEvent, JobRecord};

const SENTINEL: &str = "SENTINEL_9f3c__DO_NOT_LEAK";

fn job(id: &str, key: &str) -> JobRecord {
    JobRecord {
        id: id.into(),
        kind: "system.noop_test".into(),
        payload_json: format!(r#"{{"n":1,"password":"{SENTINEL}"}}"#),
        idempotency_key: Some(key.into()),
        state: "Queued".into(),
        priority: 0,
        attempts: 0,
        max_attempts: 5,
        available_at: common::now(),
        lease_owner: None,
        lease_expires_at: None,
        checkpoint_json: Some(format!(r#"{{"password":"{SENTINEL}"}}"#)),
        cancel_requested: false,
        created_by: "principal".into(),
    }
}

/// A lifecycle audit row for `job_id` carrying only allowlisted metadata.
fn lifecycle_event(
    sequence: u64,
    action: &str,
    job_id: &str,
    state: &str,
    attempts: i64,
) -> AuditEvent {
    AuditEvent {
        id: format!("00000000-0000-4000-8000-{sequence:012}"),
        sequence,
        occurred_at: common::now(),
        actor_kind: "job".into(),
        actor_id: Some(job_id.into()),
        action: action.into(),
        subject_urn: format!("urn:qai:job:{job_id}"),
        outcome: "allowed".into(),
        reason: None,
        before_json: None,
        after_json: Some(format!(
            r#"{{"job_id":"{job_id}","kind":"system.noop_test","state":"{state}","attempts":{attempts},"max_attempts":5}}"#
        )),
        request_id: Some(job_id.into()),
        prev_chain_hash: format!("sha256:{}", "00".repeat(32)),
        chain_hash: format!("sha256:{}", "ab".repeat(32)),
    }
}

async fn audit_for(path: &str, subject: &str) -> Vec<AuditEvent> {
    let pool = common::rw_pool(path).await;
    let rows = sqlx::query(
        "SELECT id, sequence, occurred_at, actor_kind, actor_id, action, subject_urn, outcome, \
         reason, before_json, after_json, request_id, prev_chain_hash, chain_hash \
         FROM audit_events WHERE subject_urn = ? ORDER BY sequence ASC",
    )
    .bind(subject)
    .fetch_all(&pool)
    .await
    .unwrap();
    pool.close().await;
    rows.into_iter()
        .map(|r| {
            use sqlx::Row as _;
            AuditEvent {
                id: r.get("id"),
                sequence: r.get::<i64, _>("sequence") as u64,
                occurred_at: r.get("occurred_at"),
                actor_kind: r.get("actor_kind"),
                actor_id: r.get("actor_id"),
                action: r.get("action"),
                subject_urn: r.get("subject_urn"),
                outcome: r.get("outcome"),
                reason: r.get("reason"),
                before_json: r.get("before_json"),
                after_json: r.get("after_json"),
                request_id: r.get("request_id"),
                prev_chain_hash: r.get("prev_chain_hash"),
                chain_hash: r.get("chain_hash"),
            }
        })
        .collect()
}

#[tokio::test]
async fn job_transition_and_audit_row_commit_together() {
    let fx = common::fixture().await;

    // One unit of work: the enqueue plus its lifecycle event.
    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().enqueue(job("job-a", "idem-a")).await.unwrap();
    uow.audit().append(lifecycle_event(1, "job_enqueued", "job-a", "Queued", 0)).await.unwrap();
    uow.commit().await.unwrap();

    assert_eq!(common::job_state(&fx.path, "job-a").await.as_deref(), Some("Queued"));
    let events = audit_for(&fx.path, "urn:qai:job:job-a").await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].action, "job_enqueued");
    assert_eq!(events[0].actor_kind, "job");
    let after = events[0].after_json.clone().unwrap_or_default();
    assert!(after.contains("\"job-a\"") && after.contains("\"Queued\""), "{after}");
    assert!(!after.contains(SENTINEL), "audit payloads never carry job secrets");
}

#[tokio::test]
async fn audit_failure_rolls_back_the_job_transition() {
    let fx = common::fixture().await;

    // The audit append fails (duplicate id): the enqueue must vanish with it.
    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().enqueue(job("job-b", "idem-b")).await.unwrap();
    uow.audit().append(lifecycle_event(1, "job_enqueued", "job-b", "Queued", 0)).await.unwrap();
    let dup = uow.audit().append(lifecycle_event(1, "job_enqueued", "job-b", "Queued", 0)).await;
    assert!(
        matches!(dup, Err(StorageError::Conflict)),
        "duplicate audit id must conflict, got {dup:?}"
    );
    uow.rollback().await.unwrap();

    assert!(common::job_state(&fx.path, "job-b").await.is_none());
    assert!(audit_for(&fx.path, "urn:qai:job:job-b").await.is_empty());
}

#[tokio::test]
async fn lifecycle_events_order_by_sequence_with_allowlisted_payloads() {
    let fx = common::fixture().await;

    // Enqueue → lease → completion, each with its lifecycle event.
    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().enqueue(job("job-c", "idem-c")).await.unwrap();
    uow.audit().append(lifecycle_event(1, "job_enqueued", "job-c", "Queued", 0)).await.unwrap();
    uow.commit().await.unwrap();

    let mut uow = fx.db.write().await.unwrap();
    let claimed = uow.jobs().claim("job-c", "worker-1").await.unwrap().unwrap();
    uow.audit()
        .append(lifecycle_event(2, "job_leased", "job-c", "Running", claimed.attempts as i64))
        .await
        .unwrap();
    uow.commit().await.unwrap();

    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().finish("job-c", "Succeeded", None).await.unwrap();
    uow.audit().append(lifecycle_event(3, "job_completed", "job-c", "Succeeded", 1)).await.unwrap();
    uow.commit().await.unwrap();

    let events = audit_for(&fx.path, "urn:qai:job:job-c").await;
    let actions: Vec<&str> = events.iter().map(|e| e.action.as_str()).collect();
    assert_eq!(actions, vec!["job_enqueued", "job_leased", "job_completed"]);
    let sequences: Vec<u64> = events.iter().map(|e| e.sequence).collect();
    assert_eq!(sequences, vec![1, 2, 3]);
    let rendered: String =
        events.iter().filter_map(|e| e.after_json.clone()).collect::<Vec<_>>().join("\n");
    assert!(!rendered.contains(SENTINEL), "no lifecycle payload leaks job secrets");
}

#[tokio::test]
async fn cancellation_and_retry_lifecycle_rows() {
    let fx = common::fixture().await;

    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().enqueue(job("job-d", "idem-d")).await.unwrap();
    uow.audit().append(lifecycle_event(1, "job_enqueued", "job-d", "Queued", 0)).await.unwrap();
    uow.jobs().claim("job-d", "worker-1").await.unwrap();
    uow.audit().append(lifecycle_event(2, "job_leased", "job-d", "Running", 1)).await.unwrap();
    uow.jobs().request_cancel("job-d").await.unwrap();
    uow.audit()
        .append(lifecycle_event(3, "job_cancellation_requested", "job-d", "Running", 1))
        .await
        .unwrap();
    uow.jobs()
        .finish_owned(
            "job-d",
            "worker-1",
            "Cancelled",
            Some(r#"{"disposition":"cancelled_at_checkpoint"}"#.into()),
        )
        .await
        .unwrap();
    uow.audit().append(lifecycle_event(4, "job_cancelled", "job-d", "Cancelled", 1)).await.unwrap();
    uow.commit().await.unwrap();

    let events = audit_for(&fx.path, "urn:qai:job:job-d").await;
    let actions: Vec<&str> = events.iter().map(|e| e.action.as_str()).collect();
    assert_eq!(
        actions,
        vec!["job_enqueued", "job_leased", "job_cancellation_requested", "job_cancelled"]
    );
}

#[tokio::test]
async fn checkpoint_and_heartbeat_stage_no_audit_rows() {
    let fx = common::fixture().await;

    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().enqueue(job("job-e", "idem-e")).await.unwrap();
    uow.jobs().claim("job-e", "worker-1").await.unwrap();
    uow.jobs()
        .checkpoint_owned("job-e", "worker-1", None, Some("\"stage-1\"".into()))
        .await
        .unwrap();
    assert!(uow.jobs().heartbeat("job-e", "worker-1", 60).await.unwrap());
    uow.commit().await.unwrap();

    // Progress writes are durable but are not lifecycle transitions: the
    // chain carries no rows for them (D-11).
    let pool = common::rw_pool(&fx.path).await;
    let n: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM audit_events").fetch_one(&pool).await.unwrap();
    pool.close().await;
    assert_eq!(n, 0);
}
