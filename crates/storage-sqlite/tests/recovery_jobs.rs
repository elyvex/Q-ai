//! `recovery_jobs` — reproducibility and crash recovery guarantees (AC-P0-11).
//!
//! - Duplicate enqueue with the same idempotency key yields one execution.
//! - An expired lease marks the run `Interrupted` and it resumes from its
//!   checkpoint without losing recorded progress.
//! - Cancellation is durably recorded.

mod common;

use storage::Database as _;
use storage::error::StorageError;
use storage::repository::JobRecord;

fn job(id: &str, key: &str) -> JobRecord {
    JobRecord {
        id: id.into(),
        kind: "system.noop_test".into(),
        payload_json: "{}".into(),
        idempotency_key: Some(key.into()),
        state: "Queued".into(),
        priority: 0,
        attempts: 0,
        max_attempts: 5,
        available_at: common::now(),
        lease_owner: None,
        lease_expires_at: None,
        checkpoint_json: None,
        cancel_requested: false,
        created_by: "principal".into(),
    }
}

#[tokio::test]
async fn duplicate_enqueue_and_double_claim_yield_one_execution() {
    let fx = common::fixture().await;

    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().enqueue(job("job-1", "idem-1")).await.unwrap();
    let dup = uow.jobs().enqueue(job("job-1b", "idem-1")).await;
    assert!(matches!(dup, Err(StorageError::Conflict)), "duplicate idempotency key must conflict");
    uow.commit().await.unwrap();

    // First claim succeeds, second is refused (already leased).
    let mut uow = fx.db.write().await.unwrap();
    let first = uow.jobs().claim("job-1", "worker-1").await.unwrap();
    assert!(first.is_some());
    let second = uow.jobs().claim("job-1", "worker-2").await.unwrap();
    assert!(second.is_none(), "a leased job must not be claimed twice");
    uow.commit().await.unwrap();
}

#[tokio::test]
async fn expired_lease_marks_interrupted_and_resumes_from_checkpoint() {
    let fx = common::fixture().await;

    // Enqueue and claim (Running, attempts = 1).
    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().enqueue(job("job-2", "idem-2")).await.unwrap();
    uow.jobs().claim("job-2", "worker-1").await.unwrap();
    // Record progress before the "crash".
    uow.jobs().checkpoint("job-2", None, Some("\"stage-3\"".into())).await.unwrap();
    uow.commit().await.unwrap();

    assert_eq!(common::job_checkpoint(&fx.path, "job-2").await.as_deref(), Some("\"stage-3\""));

    // The worker "crashes": its lease expires.
    common::backdate_lease(&fx.path, "job-2").await;

    // Startup scan reaps the expired lease → Interrupted.
    let mut uow = fx.db.write().await.unwrap();
    let reaped = uow.jobs().reap_expired_leases().await.unwrap();
    assert_eq!(reaped, vec!["job-2".to_string()]);
    uow.commit().await.unwrap();
    assert_eq!(common::job_state(&fx.path, "job-2").await.as_deref(), Some("Interrupted"));

    // Resume: the checkpoint is preserved (no completed stage is repeated).
    assert_eq!(common::job_checkpoint(&fx.path, "job-2").await.as_deref(), Some("\"stage-3\""));
    let mut uow = fx.db.write().await.unwrap();
    let resumed = uow.jobs().claim("job-2", "worker-2").await.unwrap();
    assert!(resumed.is_some(), "an Interrupted job must be reclaimable");
    assert_eq!(
        resumed.unwrap().checkpoint_json.as_deref(),
        Some("\"stage-3\""),
        "resume must carry the checkpoint forward"
    );
    uow.commit().await.unwrap();

    assert_eq!(common::job_state(&fx.path, "job-2").await.as_deref(), Some("Running"));
    assert_eq!(common::job_attempts(&fx.path, "job-2").await, 2);
}

#[tokio::test]
async fn rescheduled_job_is_not_claimable_until_due() {
    let fx = common::fixture().await;

    // Two due jobs; delay the second by an hour.
    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().enqueue(job("job-due", "idem-due")).await.unwrap();
    uow.jobs().enqueue(job("job-later", "idem-later")).await.unwrap();
    uow.jobs().reschedule("job-later", 3600, None).await.unwrap();
    uow.commit().await.unwrap();

    // Only the due job is claimable; the rescheduled one waits.
    let mut uow = fx.db.write().await.unwrap();
    let first = uow.jobs().claim_next("worker-1", 300).await.unwrap().unwrap();
    assert_eq!(first.id, "job-due");
    let second = uow.jobs().claim_next("worker-1", 300).await.unwrap();
    assert!(second.is_none(), "a job delayed by reschedule must not be claimable yet");
    uow.commit().await.unwrap();

    // Once its `available_at` passes (simulated by backdating), it claims normally.
    let pool = common::rw_pool(&fx.path).await;
    sqlx::query("UPDATE jobs SET available_at = '2000-01-01T00:00:00Z' WHERE id = 'job-later'")
        .execute(&pool)
        .await
        .unwrap();
    pool.close().await;

    let mut uow = fx.db.write().await.unwrap();
    let resumed = uow.jobs().claim_next("worker-2", 300).await.unwrap().unwrap();
    assert_eq!(resumed.id, "job-later");
    uow.commit().await.unwrap();
}

#[tokio::test]
async fn cancel_is_durably_recorded() {
    let fx = common::fixture().await;
    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().enqueue(job("job-3", "idem-3")).await.unwrap();
    uow.jobs().claim("job-3", "worker-1").await.unwrap();
    uow.jobs().cancel("job-3").await.unwrap();
    uow.commit().await.unwrap();

    assert!(common::job_cancel_requested(&fx.path, "job-3").await);
}
