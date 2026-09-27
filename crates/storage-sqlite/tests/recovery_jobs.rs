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

// ─── 01-03-01: durable checkpoint / retry / cancellation adapter proof ───

/// A named (versioned) checkpoint value as `JobContext::checkpoint_named`
/// serializes it: `{"version":1,"name":"…","payload":…}`.
fn named_checkpoint(name: &str) -> String {
    format!(r#"{{"version":1,"name":"{name}","payload":{{"n":3}}}}"#)
}

#[tokio::test]
async fn named_checkpoint_is_committed_before_handler_completion() {
    let fx = common::fixture().await;

    // Enqueue, claim, and commit a named checkpoint while "running".
    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().enqueue(job("job-ncp", "idem-ncp")).await.unwrap();
    uow.jobs().claim("job-ncp", "worker-1").await.unwrap();
    let stored = named_checkpoint("stage.done");
    assert!(
        uow.jobs()
            .checkpoint_owned("job-ncp", "worker-1", None, Some(stored.clone()))
            .await
            .unwrap(),
        "the lease holder's checkpoint must apply"
    );
    uow.commit().await.unwrap();

    // Readable from the committed row before any finish/recovery.
    assert_eq!(common::job_checkpoint(&fx.path, "job-ncp").await.as_deref(), Some(stored.as_str()));
}

#[tokio::test]
async fn expired_lease_recovery_preserves_the_named_checkpoint() {
    let fx = common::fixture().await;

    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().enqueue(job("job-ncpr", "idem-ncpr")).await.unwrap();
    uow.jobs().claim("job-ncpr", "worker-1").await.unwrap();
    let stored = named_checkpoint("import.validated");
    uow.jobs().checkpoint_owned("job-ncpr", "worker-1", None, Some(stored.clone())).await.unwrap();
    uow.commit().await.unwrap();
    common::backdate_lease(&fx.path, "job-ncpr").await;

    let mut uow = fx.db.write().await.unwrap();
    let reaped = uow.jobs().reap_expired_leases().await.unwrap();
    assert_eq!(reaped, vec!["job-ncpr".to_string()]);
    uow.commit().await.unwrap();

    // Recovery keeps the boundary: resume carries it forward, attempts advance.
    assert_eq!(
        common::job_checkpoint(&fx.path, "job-ncpr").await.as_deref(),
        Some(stored.as_str())
    );
    let mut uow = fx.db.write().await.unwrap();
    let resumed = uow.jobs().claim("job-ncpr", "worker-2").await.unwrap().unwrap();
    assert_eq!(resumed.checkpoint_json.as_deref(), Some(stored.as_str()));
    uow.commit().await.unwrap();
    assert_eq!(common::job_attempts(&fx.path, "job-ncpr").await, 2);
}

#[tokio::test]
async fn cancel_request_preserves_owner_and_expiry() {
    let fx = common::fixture().await;

    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().enqueue(job("job-cx", "idem-cx")).await.unwrap();
    uow.jobs().claim("job-cx", "worker-1").await.unwrap();
    uow.commit().await.unwrap();
    let owner_before = common::job_owner(&fx.path, "job-cx").await;
    let expiry_before = common::job_lease_expiry(&fx.path, "job-cx").await;
    assert_eq!(owner_before.as_deref(), Some("worker-1"));
    assert!(expiry_before.is_some());

    // Request-only: flag set, lease untouched.
    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().request_cancel("job-cx").await.unwrap();
    uow.commit().await.unwrap();

    assert!(common::job_cancel_requested(&fx.path, "job-cx").await);
    assert_eq!(common::job_owner(&fx.path, "job-cx").await, owner_before);
    assert_eq!(common::job_lease_expiry(&fx.path, "job-cx").await, expiry_before);
    assert_eq!(common::job_state(&fx.path, "job-cx").await.as_deref(), Some("Running"));

    // Terminal jobs reject the request; missing jobs are not found.
    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().enqueue(job("job-done", "idem-done")).await.unwrap();
    uow.jobs().claim("job-done", "worker-1").await.unwrap();
    uow.jobs().finish("job-done", "Succeeded", None).await.unwrap();
    let terminal = uow.jobs().request_cancel("job-done").await;
    assert!(
        matches!(terminal, Err(StorageError::ConstraintViolation { .. })),
        "terminal cancel must be rejected, got {terminal:?}"
    );
    let missing = uow.jobs().request_cancel("job-missing").await;
    assert!(matches!(missing, Err(StorageError::NotFound { .. })), "got {missing:?}");
    uow.rollback().await.unwrap();
}

#[tokio::test]
async fn explicit_retry_transitions_only_eligible_states() {
    let fx = common::fixture().await;

    // Eligible: Failed, DeadLettered, Interrupted — each with attempts spent,
    // a cancel flag, and a committed boundary.
    for (id, state) in [("job-f", "Failed"), ("job-d", "DeadLettered"), ("job-i", "Interrupted")] {
        let mut uow = fx.db.write().await.unwrap();
        let mut j = job(id, &format!("idem-{id}"));
        j.state = state.into();
        j.attempts = 5;
        j.cancel_requested = true;
        j.checkpoint_json = Some(named_checkpoint("stage.done"));
        uow.jobs().enqueue(j).await.unwrap();
        uow.jobs().retry(id).await.unwrap();
        uow.commit().await.unwrap();

        assert_eq!(common::job_state(&fx.path, id).await.as_deref(), Some("Queued"), "{id}");
        assert_eq!(common::job_attempts(&fx.path, id).await, 0, "{id}: window reset");
        assert!(common::job_owner(&fx.path, id).await.is_none(), "{id}: no lease");
        assert!(!common::job_cancel_requested(&fx.path, id).await, "{id}: flag cleared");
        assert!(
            common::job_checkpoint(&fx.path, id).await.is_some(),
            "{id}: boundary preserved for resume"
        );
        // Reclaimable immediately with a fresh attempt count.
        let mut uow = fx.db.write().await.unwrap();
        let resumed = uow.jobs().claim(id, "worker-9").await.unwrap().unwrap();
        assert_eq!(resumed.attempts, 1);
        uow.commit().await.unwrap();
    }

    // Ineligible: Queued, Running, Succeeded — rejected without mutation.
    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().enqueue(job("job-q", "idem-q")).await.unwrap();
    uow.jobs().enqueue(job("job-r", "idem-r")).await.unwrap();
    uow.jobs().claim("job-r", "worker-1").await.unwrap();
    uow.jobs().enqueue(job("job-s", "idem-s")).await.unwrap();
    uow.jobs().claim("job-s", "worker-1").await.unwrap();
    uow.jobs().finish("job-s", "Succeeded", None).await.unwrap();
    for id in ["job-q", "job-r", "job-s"] {
        let before = uow.jobs().get(id).await.unwrap().unwrap();
        let err = uow.jobs().retry(id).await.unwrap_err();
        assert!(matches!(err, StorageError::ConstraintViolation { .. }), "{id}: got {err:?}");
        let after = uow.jobs().get(id).await.unwrap().unwrap();
        assert_eq!(
            (after.state, after.attempts),
            (before.state, before.attempts),
            "{id} unchanged"
        );
    }
    let missing = uow.jobs().retry("job-missing").await;
    assert!(matches!(missing, Err(StorageError::NotFound { .. })), "got {missing:?}");
    uow.commit().await.unwrap();
}

#[tokio::test]
async fn non_owner_heartbeat_checkpoint_and_finish_affect_zero_rows() {
    let fx = common::fixture().await;

    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().enqueue(job("job-own", "idem-own")).await.unwrap();
    uow.jobs().claim("job-own", "worker-1").await.unwrap();
    // A stranger's heartbeat/checkpoint/finish touches nothing.
    assert!(!uow.jobs().heartbeat("job-own", "worker-2", 60).await.unwrap());
    assert!(
        !uow.jobs()
            .checkpoint_owned("job-own", "worker-2", None, Some("\"x\"".into()))
            .await
            .unwrap()
    );
    assert!(!uow.jobs().finish_owned("job-own", "worker-2", "Succeeded", None).await.unwrap());
    uow.commit().await.unwrap();

    assert_eq!(common::job_state(&fx.path, "job-own").await.as_deref(), Some("Running"));
    assert_eq!(common::job_owner(&fx.path, "job-own").await.as_deref(), Some("worker-1"));
    assert!(common::job_checkpoint(&fx.path, "job-own").await.is_none());

    // The holder's writes still apply.
    let mut uow = fx.db.write().await.unwrap();
    assert!(uow.jobs().heartbeat("job-own", "worker-1", 60).await.unwrap());
    assert!(
        uow.jobs()
            .checkpoint_owned("job-own", "worker-1", None, Some("\"cp\"".into()))
            .await
            .unwrap()
    );
    assert!(uow.jobs().finish_owned("job-own", "worker-1", "Succeeded", None).await.unwrap());
    uow.commit().await.unwrap();
    assert_eq!(common::job_state(&fx.path, "job-own").await.as_deref(), Some("Succeeded"));
}

// ─── 01-03-02: worker crash / retry / cancellation durable semantics ───

#[tokio::test]
async fn crash_with_pending_cancel_keeps_request_and_boundary() {
    let fx = common::fixture().await;

    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().enqueue(job("job-cc", "idem-cc")).await.unwrap();
    uow.jobs().claim("job-cc", "worker-1").await.unwrap();
    let stored = named_checkpoint("stage.one");
    uow.jobs().checkpoint_owned("job-cc", "worker-1", None, Some(stored.clone())).await.unwrap();
    // A cancellation request lands, then the worker crashes before observing it.
    uow.jobs().request_cancel("job-cc").await.unwrap();
    uow.commit().await.unwrap();
    common::backdate_lease(&fx.path, "job-cc").await;

    let mut uow = fx.db.write().await.unwrap();
    assert_eq!(uow.jobs().reap_expired_leases().await.unwrap(), vec!["job-cc".to_string()]);
    uow.commit().await.unwrap();

    // Recovery preserves both the request and the boundary for the next worker.
    assert_eq!(common::job_state(&fx.path, "job-cc").await.as_deref(), Some("Interrupted"));
    assert!(common::job_cancel_requested(&fx.path, "job-cc").await);
    assert_eq!(common::job_checkpoint(&fx.path, "job-cc").await.as_deref(), Some(stored.as_str()));
}

#[tokio::test]
async fn per_kind_exhaustion_leaves_an_inspectable_state_with_explicit_retry() {
    let fx = common::fixture().await;

    // A kind stamped with max_attempts = 2 exhausts after two claims.
    let mut uow = fx.db.write().await.unwrap();
    let mut j = job("job-ex", "idem-ex");
    j.max_attempts = 2;
    uow.jobs().enqueue(j).await.unwrap();
    uow.jobs().claim("job-ex", "worker-1").await.unwrap();
    uow.jobs().reschedule("job-ex", 0, Some("retry".into())).await.unwrap();
    // Backdate availability: reschedule(0) lands ~now; force due for determinism.
    uow.commit().await.unwrap();
    let pool = common::rw_pool(&fx.path).await;
    sqlx::query("UPDATE jobs SET available_at = '2000-01-01T00:00:00Z' WHERE id = 'job-ex'")
        .execute(&pool)
        .await
        .unwrap();
    pool.close().await;

    let mut uow = fx.db.write().await.unwrap();
    let second = uow.jobs().claim("job-ex", "worker-1").await.unwrap().unwrap();
    assert_eq!(second.attempts, 2);
    uow.jobs()
        .finish_owned("job-ex", "worker-1", "DeadLettered", Some("max attempts exceeded".into()))
        .await
        .unwrap();
    uow.commit().await.unwrap();

    // Inspectable: terminal state, spent attempts, recorded reason.
    assert_eq!(common::job_state(&fx.path, "job-ex").await.as_deref(), Some("DeadLettered"));
    assert_eq!(common::job_attempts(&fx.path, "job-ex").await, 2);
    assert_eq!(
        common::job_error_json(&fx.path, "job-ex").await.as_deref(),
        Some("max attempts exceeded")
    );

    // Explicit operator retry reopens a fresh window from the same boundary.
    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().retry("job-ex").await.unwrap();
    uow.commit().await.unwrap();
    assert_eq!(common::job_state(&fx.path, "job-ex").await.as_deref(), Some("Queued"));
    assert_eq!(common::job_attempts(&fx.path, "job-ex").await, 0);
}

#[tokio::test]
async fn cancellation_boundaries_record_truthful_terminal_states() {
    let fx = common::fixture().await;

    // Before start: the request persists on the queued job for the worker's
    // pre-start observation.
    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().enqueue(job("job-cb", "idem-cb")).await.unwrap();
    uow.jobs().request_cancel("job-cb").await.unwrap();
    uow.commit().await.unwrap();
    assert!(common::job_cancel_requested(&fx.path, "job-cb").await);
    assert_eq!(common::job_state(&fx.path, "job-cb").await.as_deref(), Some("Queued"));

    // During a boundary: the worker finalizes Cancelled with the locked
    // disposition instead of a bare state.
    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().enqueue(job("job-cd", "idem-cd")).await.unwrap();
    uow.jobs().claim("job-cd", "worker-1").await.unwrap();
    uow.jobs().request_cancel("job-cd").await.unwrap();
    let stored = named_checkpoint("stage.one");
    uow.jobs().checkpoint_owned("job-cd", "worker-1", None, Some(stored)).await.unwrap();
    let disposition = r#"{"disposition":"cancelled_at_checkpoint"}"#;
    assert!(
        uow.jobs()
            .finish_owned("job-cd", "worker-1", "Cancelled", Some(disposition.into()))
            .await
            .unwrap()
    );
    uow.commit().await.unwrap();
    assert_eq!(common::job_state(&fx.path, "job-cd").await.as_deref(), Some("Cancelled"));
    assert_eq!(common::job_error_json(&fx.path, "job-cd").await.as_deref(), Some(disposition));

    // A missed boundary never claims a cancellation it did not observe: the
    // disposition names the miss explicitly.
    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().enqueue(job("job-cm", "idem-cm")).await.unwrap();
    uow.jobs().claim("job-cm", "worker-1").await.unwrap();
    uow.jobs().request_cancel("job-cm").await.unwrap();
    let missed = r#"{"disposition":"missed_boundary"}"#;
    uow.jobs().finish_owned("job-cm", "worker-1", "Cancelled", Some(missed.into())).await.unwrap();
    uow.commit().await.unwrap();
    assert_eq!(common::job_error_json(&fx.path, "job-cm").await.as_deref(), Some(missed));
}

#[tokio::test]
async fn stale_owner_cannot_finalize_after_lease_loss() {
    let fx = common::fixture().await;

    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().enqueue(job("job-st", "idem-st")).await.unwrap();
    uow.jobs().claim("job-st", "worker-1").await.unwrap();
    uow.commit().await.unwrap();

    // worker-1's lease expires; worker-2 reaps and reclaims the job.
    common::backdate_lease(&fx.path, "job-st").await;
    let mut uow = fx.db.write().await.unwrap();
    uow.jobs().reap_expired_leases().await.unwrap();
    uow.jobs().claim("job-st", "worker-2").await.unwrap();
    // The stale owner's terminal and checkpoint writes affect zero rows.
    assert!(!uow.jobs().finish_owned("job-st", "worker-1", "Succeeded", None).await.unwrap());
    assert!(
        !uow.jobs()
            .checkpoint_owned("job-st", "worker-1", None, Some("\"stale\"".into()))
            .await
            .unwrap()
    );
    uow.commit().await.unwrap();

    assert_eq!(common::job_state(&fx.path, "job-st").await.as_deref(), Some("Running"));
    assert_eq!(common::job_owner(&fx.path, "job-st").await.as_deref(), Some("worker-2"));
    assert!(common::job_checkpoint(&fx.path, "job-st").await.is_none());
}
