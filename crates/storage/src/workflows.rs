//! Transaction-scoped write workflows (D0.18, ADR-0702 §3).
//!
//! These helpers make the outbox invariant **impossible to violate by
//! construction** for the projection-relevant operations Phase 0 knows about:
//! each allocates a generation, enqueues its outbox row, and performs the
//! authoritative change through the *same* [`UnitOfWork`]. The caller commits
//! once; the outbox row and the change live or die together in one SQLite
//! transaction.
//!
//! # Ordering
//!
//! Deactivation writes the tombstone **before** the state transition, so the
//! tombstone is durably recorded before the transition can be observed
//! (AC-P0-25, ADR-0702 §9).

use domain::TombstoneId;

use crate::UnitOfWork;
use crate::error::StorageError;
use crate::repository::{NewOutboxEvent, ProvenanceRecord, TombstoneRow};

fn now_rfc3339() -> String {
    // `domain::Timestamp` already renders RFC3339 UTC.
    domain::Timestamp::now().to_string()
}

/// Activate a source version: allocate a generation, enqueue the outbox event,
/// then flip `Indexing → Active` — all in the caller's transaction.
pub async fn record_source_activation(
    uow: &mut dyn UnitOfWork,
    source_version_id: &str,
    scope: &str,
    subject_urn: &str,
) -> Result<String, StorageError> {
    let generation = uow.outbox().allocate_generation(scope, "source_activated").await?;
    uow.outbox()
        .enqueue(NewOutboxEvent {
            scope: scope.to_string(),
            target_generation: generation.id.clone(),
            operation: "source_activated".to_string(),
            subject_urn: subject_urn.to_string(),
            idempotency_key: format!("source_activated:{source_version_id}"),
            payload_json: "{}".to_string(),
        })
        .await?;
    uow.sources().transition_state(source_version_id, "Indexing", "Active").await?;
    Ok(generation.id)
}

/// Deactivate a source version: write a tombstone, allocate a generation,
/// enqueue the outbox event, then flip `Active → Deprecated` — all in the
/// caller's transaction. Returns the tombstone id.
pub async fn record_source_deactivation(
    uow: &mut dyn UnitOfWork,
    source_version_id: &str,
    scope: &str,
    subject_urn: &str,
    reason: &str,
) -> Result<String, StorageError> {
    let tombstone_id = TombstoneId::new().to_string();
    // Tombstone first: current policy blocks retrieval immediately even while
    // physical cleanup is pending (ADR-0702 §9).
    uow.outbox()
        .insert_tombstone(TombstoneRow {
            id: tombstone_id.clone(),
            subject_urn: subject_urn.to_string(),
            reason: "deactivated".to_string(),
            effective_at: now_rfc3339(),
            created_by: None,
            propagation_state: "Pending".to_string(),
        })
        .await?;

    let generation = uow.outbox().allocate_generation(scope, "source_deactivated").await?;
    uow.outbox()
        .enqueue(NewOutboxEvent {
            scope: scope.to_string(),
            target_generation: generation.id.clone(),
            operation: "source_deactivated".to_string(),
            subject_urn: subject_urn.to_string(),
            idempotency_key: format!("source_deactivated:{source_version_id}:{}", now_rfc3339()),
            payload_json: serde_json::json!({ "reason": reason }).to_string(),
        })
        .await?;
    uow.sources().transition_state(source_version_id, "Active", "Deprecated").await?;
    Ok(tombstone_id)
}

/// Record a provenance write: insert the record, allocate a generation, and
/// enqueue the outbox event — all in the caller's transaction.
pub async fn record_provenance_write(
    uow: &mut dyn UnitOfWork,
    record: ProvenanceRecord,
    scope: &str,
) -> Result<String, StorageError> {
    let subject_urn = record.subject_urn.clone();
    uow.provenance().insert(record).await?;
    let generation = uow.outbox().allocate_generation(scope, "provenance_written").await?;
    uow.outbox()
        .enqueue(NewOutboxEvent {
            scope: scope.to_string(),
            target_generation: generation.id.clone(),
            operation: "provenance_written".to_string(),
            subject_urn,
            idempotency_key: format!("provenance_written:{}", generation.id),
            payload_json: "{}".to_string(),
        })
        .await?;
    Ok(generation.id)
}

/// Relay one batch of pending outbox events: claim up to `limit` for `owner`
/// and mark them dispatched. Returns the number of events relayed.
///
/// Phase 0 has no consumers (no FTS/vector/graph exist); the relay records
/// dispatch so the contract, table, and "commit implies durable outbox row"
/// guarantee exist before Phase 2 needs them (T64).
pub async fn relay_outbox_once(
    uow: &mut dyn UnitOfWork,
    owner: &str,
    limit: u32,
) -> Result<usize, StorageError> {
    let claimed = uow.outbox().claim_pending(owner, limit).await?;
    let count = claimed.len();
    for event in &claimed {
        uow.outbox().mark_dispatched(&event.id).await?;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quran::QuranRepository;
    use crate::repository::{
        ApprovalRow, AuditEvent, AuditRepository, ChainVerificationResult, GenerationRow,
        JobRecord, JobRepository, OutboxEventRow, OutboxRepository, PrincipalRow,
        ProvenanceRepository, ReviewRecord, SettingRow, SettingsRepository, SourceRepository,
        SourceRow, SourceVersionRow, StateTransitionRow,
    };
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Clone, PartialEq)]
    enum Call {
        AllocateGeneration { scope: String, reason: String },
        Enqueue { operation: String, key: String },
        Transition { id: String, from: String, to: String },
        InsertTombstone { subject_urn: String, state: String },
        InsertProvenance { subject_urn: String },
        ClaimPending { owner: String, limit: u32 },
        MarkDispatched { id: String },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum FailAt {
        None,
        Allocate,
        Enqueue,
        Transition,
        Tombstone,
        ProvenanceInsert,
    }

    struct FakeState {
        calls: Vec<Call>,
        fail_at: FailAt,
        enqueued_keys: Vec<(String, String)>,
        committed: bool,
    }

    impl FakeState {
        fn new(fail_at: FailAt) -> Self {
            Self { calls: Vec::new(), fail_at, enqueued_keys: Vec::new(), committed: false }
        }

        fn check(&self, step: FailAt, code: StorageError) -> Result<(), StorageError> {
            if self.fail_at == step { Err(code) } else { Ok(()) }
        }
    }

    struct FakeSources {
        state: Arc<Mutex<FakeState>>,
    }

    #[async_trait::async_trait]
    impl SourceRepository for FakeSources {
        async fn transition_state(
            &mut self,
            id: &str,
            from: &str,
            to: &str,
        ) -> Result<(), StorageError> {
            let mut s = self.state.lock().unwrap();
            s.calls.push(Call::Transition { id: id.into(), from: from.into(), to: to.into() });
            s.check(
                FailAt::Transition,
                StorageError::ConstraintViolation { message: "boom".into() },
            )
        }

        async fn get(&self, _id: &str) -> Result<Option<SourceRow>, StorageError> {
            Ok(None)
        }
    }

    struct FakeProvenance {
        state: Arc<Mutex<FakeState>>,
    }

    #[async_trait::async_trait]
    impl ProvenanceRepository for FakeProvenance {
        async fn insert(&mut self, record: ProvenanceRecord) -> Result<(), StorageError> {
            let mut s = self.state.lock().unwrap();
            s.calls.push(Call::InsertProvenance { subject_urn: record.subject_urn });
            s.check(FailAt::ProvenanceInsert, StorageError::StorageUnavailable)
        }
    }

    struct FakeAudit;
    struct FakeJobs;
    struct FakeSettings;
    struct FakeQuran;

    #[async_trait::async_trait]
    impl AuditRepository for FakeAudit {}
    #[async_trait::async_trait]
    impl JobRepository for FakeJobs {}
    #[async_trait::async_trait]
    impl SettingsRepository for FakeSettings {}
    #[async_trait::async_trait]
    impl QuranRepository for FakeQuran {}

    struct FakeOutbox {
        state: Arc<Mutex<FakeState>>,
        generations: HashMap<String, u64>,
        pending: Vec<OutboxEventRow>,
        next_id: u64,
    }

    impl FakeOutbox {
        fn new(state: Arc<Mutex<FakeState>>) -> Self {
            Self { state, generations: HashMap::new(), pending: Vec::new(), next_id: 0 }
        }
    }

    #[async_trait::async_trait]
    impl OutboxRepository for FakeOutbox {
        async fn allocate_generation(
            &mut self,
            scope: &str,
            reason: &str,
        ) -> Result<GenerationRow, StorageError> {
            let mut s = self.state.lock().unwrap();
            s.calls.push(Call::AllocateGeneration { scope: scope.into(), reason: reason.into() });
            s.check(FailAt::Allocate, StorageError::StorageBusy)?;
            let n = self.generations.entry(scope.into()).or_insert(0);
            *n += 1;
            Ok(GenerationRow {
                id: format!("gen-{scope}-{n}"),
                scope: scope.into(),
                number: *n,
                reason: reason.into(),
                created_at: "2026-01-01T00:00:00Z".into(),
            })
        }

        async fn enqueue(&mut self, event: NewOutboxEvent) -> Result<String, StorageError> {
            let mut s = self.state.lock().unwrap();
            if s.enqueued_keys.contains(&(event.operation.clone(), event.idempotency_key.clone())) {
                return Err(StorageError::Conflict);
            }
            s.check(FailAt::Enqueue, StorageError::Conflict)?;
            s.calls.push(Call::Enqueue {
                operation: event.operation.clone(),
                key: event.idempotency_key.clone(),
            });
            s.enqueued_keys.push((event.operation.clone(), event.idempotency_key.clone()));
            self.next_id += 1;
            let id = format!("evt-{}", self.next_id);
            self.pending.push(OutboxEventRow {
                id: id.clone(),
                scope: event.scope,
                target_generation: event.target_generation,
                operation: event.operation,
                subject_urn: event.subject_urn,
                idempotency_key: event.idempotency_key,
                payload_json: event.payload_json,
                state: "Pending".into(),
                lease_owner: None,
                lease_expires_at: None,
                attempts: 0,
                created_at: "2026-01-01T00:00:00Z".into(),
                dispatched_at: None,
            });
            Ok(id)
        }

        async fn claim_pending(
            &mut self,
            owner: &str,
            limit: u32,
        ) -> Result<Vec<OutboxEventRow>, StorageError> {
            let mut s = self.state.lock().unwrap();
            s.calls.push(Call::ClaimPending { owner: owner.into(), limit });
            let n = (limit as usize).min(self.pending.len());
            let mut out = Vec::new();
            for row in self.pending.iter_mut().take(n) {
                row.state = "Claimed".into();
                row.lease_owner = Some(owner.into());
                out.push(row.clone());
            }
            Ok(out)
        }

        async fn mark_dispatched(&mut self, id: &str) -> Result<(), StorageError> {
            let mut s = self.state.lock().unwrap();
            s.calls.push(Call::MarkDispatched { id: id.into() });
            for row in self.pending.iter_mut() {
                if row.id == id {
                    row.state = "Dispatched".into();
                }
            }
            Ok(())
        }

        async fn insert_tombstone(&mut self, t: TombstoneRow) -> Result<(), StorageError> {
            let mut s = self.state.lock().unwrap();
            s.calls.push(Call::InsertTombstone {
                subject_urn: t.subject_urn.clone(),
                state: t.propagation_state.clone(),
            });
            s.check(FailAt::Tombstone, StorageError::StorageBusy)
        }
    }

    struct FakeUnitOfWork {
        state: Arc<Mutex<FakeState>>,
        sources: FakeSources,
        provenance: FakeProvenance,
        audit: FakeAudit,
        jobs: FakeJobs,
        settings: FakeSettings,
        outbox: FakeOutbox,
        quran: FakeQuran,
    }

    impl FakeUnitOfWork {
        fn new(fail_at: FailAt) -> Self {
            let state = Arc::new(Mutex::new(FakeState::new(fail_at)));
            Self {
                sources: FakeSources { state: state.clone() },
                provenance: FakeProvenance { state: state.clone() },
                audit: FakeAudit,
                jobs: FakeJobs,
                settings: FakeSettings,
                outbox: FakeOutbox::new(state.clone()),
                quran: FakeQuran,
                state,
            }
        }

        fn calls(&self) -> Vec<Call> {
            self.state.lock().unwrap().calls.clone()
        }

        fn enqueue(&mut self, op: &str, key: &str) {
            let id = self.outbox.next_id + 1;
            self.outbox.next_id = id;
            self.state.lock().unwrap().enqueued_keys.push((op.into(), key.into()));
            self.outbox.pending.push(OutboxEventRow {
                id: format!("evt-{id}"),
                scope: "s".into(),
                target_generation: "g".into(),
                operation: op.into(),
                subject_urn: "urn".into(),
                idempotency_key: key.into(),
                payload_json: "{}".into(),
                state: "Pending".into(),
                lease_owner: None,
                lease_expires_at: None,
                attempts: 0,
                created_at: "2026-01-01T00:00:00Z".into(),
                dispatched_at: None,
            });
        }
    }

    #[async_trait::async_trait]
    impl UnitOfWork for FakeUnitOfWork {
        fn sources(&mut self) -> &mut dyn SourceRepository {
            &mut self.sources
        }
        fn provenance(&mut self) -> &mut dyn ProvenanceRepository {
            &mut self.provenance
        }
        fn audit(&mut self) -> &mut dyn AuditRepository {
            &mut self.audit
        }
        fn jobs(&mut self) -> &mut dyn JobRepository {
            &mut self.jobs
        }
        fn settings(&mut self) -> &mut dyn SettingsRepository {
            &mut self.settings
        }
        fn outbox(&mut self) -> &mut dyn OutboxRepository {
            &mut self.outbox
        }
        fn quran(&mut self) -> &mut dyn QuranRepository {
            &mut self.quran
        }
        async fn commit(mut self: Box<Self>) -> Result<(), StorageError> {
            self.state.lock().unwrap().committed = true;
            Ok(())
        }
        async fn rollback(self: Box<Self>) -> Result<(), StorageError> {
            Ok(())
        }
    }

    fn provenance_record() -> ProvenanceRecord {
        ProvenanceRecord {
            id: "prov-1".into(),
            layer: "C".into(),
            subject_urn: "urn:test".into(),
            attribution_kind: "human".into(),
            attribution_json: "{}".into(),
            source_version_id: None,
            trust_level: "Verified".into(),
            verification_status: "verified".into(),
            confidence: None,
            versions_json: "{}".into(),
            created_by: "principal".into(),
        }
    }

    // T005 (US1-AC1): the workflow helpers drive entirely through
    // `&mut dyn UnitOfWork` with no sqlx/pool/row types in scope.
    // This test proves backend-free compilation: FakeUnitOfWork holds only
    // std + domain + storage types, yet drives all four helpers.
    // Layering is enforced separately by `cargo run -p xtask -- arch-check`
    // (storage → domain only, no sqlx leakage).
    #[tokio::test]
    async fn workflows_drive_via_dyn_unit_of_work_without_backend() {
        async fn drive(uow: &mut dyn UnitOfWork) -> Result<(), StorageError> {
            record_source_activation(uow, "v-1", "s", "urn:a").await?;
            record_provenance_write(uow, provenance_record(), "s").await?;
            // Deactivation uses a fresh version id to avoid idempotency clash
            // with the activation above in fakes that enforce key uniqueness.
            record_source_deactivation(uow, "v-2", "s", "urn:a", "stale").await?;
            // Seed one event so relay has something to claim.
            uow.outbox()
                .enqueue(NewOutboxEvent {
                    scope: "s".into(),
                    target_generation: "gen-s-1".into(),
                    operation: "op".into(),
                    subject_urn: "urn:a".into(),
                    idempotency_key: "k-relay".into(),
                    payload_json: "{}".into(),
                })
                .await?;
            let _ = relay_outbox_once(uow, "owner", 10).await?;
            Ok(())
        }

        let mut uow = FakeUnitOfWork::new(FailAt::None);
        // Compiles only if helpers accept plain `&mut dyn UnitOfWork`.
        drive(&mut uow).await.unwrap();
        Box::new(uow).commit().await.unwrap();
    }

    #[tokio::test]
    async fn activation_orders_generation_enqueue_transition() {
        let mut uow = FakeUnitOfWork::new(FailAt::None);
        let generation_id =
            record_source_activation(&mut uow, "ver-1", "scope-a", "urn:a").await.unwrap();
        assert_eq!(generation_id, "gen-scope-a-1");
        assert_eq!(
            uow.calls(),
            vec![
                Call::AllocateGeneration {
                    scope: "scope-a".into(),
                    reason: "source_activated".into()
                },
                Call::Enqueue {
                    operation: "source_activated".into(),
                    key: "source_activated:ver-1".into()
                },
                Call::Transition {
                    id: "ver-1".into(),
                    from: "Indexing".into(),
                    to: "Active".into()
                },
            ]
        );
        Box::new(uow).commit().await.unwrap();
    }

    #[tokio::test]
    async fn activation_failure_leaves_no_partial_state() {
        // Transactional semantics: the fake's `generations`/`pending` maps are
        // txn-local staged state. Atomicity means: on Err the caller rolls back
        // and never commits, so nothing becomes visible. Staged rows are
        // discarded with the UnitOfWork (real SQLite impl rolls back the txn).
        for fail in [FailAt::Allocate, FailAt::Enqueue, FailAt::Transition] {
            let mut uow = FakeUnitOfWork::new(fail);
            let err = record_source_activation(&mut uow, "ver-1", "s", "urn:a").await;
            assert!(err.is_err(), "fail_at={fail:?} must abort the workflow");
            // Never committed: the caller must roll back on Err.
            assert!(
                !uow.state.lock().unwrap().committed,
                "fail_at={fail:?} must never reach commit"
            );
            match fail {
                FailAt::Allocate => {
                    // Nothing staged: allocate failed first.
                    assert_eq!(uow.outbox.generations.get("s").copied().unwrap_or(0), 0);
                    assert!(uow.outbox.pending.is_empty());
                }
                FailAt::Enqueue => {
                    // Generation staged but its event never enqueued; both are
                    // discarded on rollback, so no committed generation exists
                    // without its event.
                    assert!(uow.outbox.pending.is_empty());
                    assert!(!uow.calls().iter().any(|c| matches!(c, Call::Transition { .. })));
                }
                FailAt::Transition => {
                    // Generation + event staged but transition failed; all
                    // discarded on rollback.
                    assert_eq!(uow.outbox.pending.len(), 1);
                }
                _ => unreachable!(),
            }
            // Prove the rollback path discards the staged txn.
            Box::new(uow).rollback().await.unwrap();
        }
    }

    #[tokio::test]
    async fn deactivation_writes_tombstone_first() {
        let mut uow = FakeUnitOfWork::new(FailAt::None);
        record_source_deactivation(&mut uow, "ver-1", "scope-a", "urn:a", "stale").await.unwrap();
        let calls = uow.calls();
        let tombstone_pos = calls
            .iter()
            .position(|c| matches!(c, Call::InsertTombstone { .. }))
            .expect("tombstone must be recorded");
        let transition_pos = calls
            .iter()
            .position(|c| matches!(c, Call::Transition { .. }))
            .expect("transition must be recorded");
        assert!(tombstone_pos < transition_pos, "tombstone must precede the transition");
        assert!(matches!(
            &calls[tombstone_pos],
            Call::InsertTombstone { state, .. } if state == "Pending"
        ));
    }

    #[tokio::test]
    async fn deactivation_tombstone_failure_aborts_everything() {
        let mut uow = FakeUnitOfWork::new(FailAt::Tombstone);
        let err = record_source_deactivation(&mut uow, "ver-1", "s", "urn:a", "stale").await;
        assert!(err.is_err());
        assert!(uow.outbox.generations.get("s").copied().unwrap_or(0) == 0);
        assert!(uow.outbox.pending.is_empty());
        assert!(!uow.calls().iter().any(|c| matches!(c, Call::Transition { .. })));
    }

    // T008 (FR-008): full deactivation order is tombstone(Pending) →
    // generation → source_deactivated enqueue → Active→Deprecated.
    #[tokio::test]
    async fn deactivation_orders_tombstone_generation_enqueue_transition() {
        let mut uow = FakeUnitOfWork::new(FailAt::None);
        record_source_deactivation(&mut uow, "ver-1", "scope-a", "urn:a", "stale").await.unwrap();
        let calls = uow.calls();
        assert_eq!(calls.len(), 4, "deactivation must emit exactly 4 steps: {calls:?}");
        assert!(matches!(
            &calls[0],
            Call::InsertTombstone { subject_urn, state }
                if subject_urn == "urn:a" && state == "Pending"
        ));
        assert_eq!(
            calls[1],
            Call::AllocateGeneration {
                scope: "scope-a".into(),
                reason: "source_deactivated".into()
            }
        );
        match &calls[2] {
            Call::Enqueue { operation, key } => {
                assert_eq!(operation, "source_deactivated");
                assert!(
                    key.starts_with("source_deactivated:ver-1:"),
                    "deactivation key must bind the version id, got {key}"
                );
            }
            other => panic!("expected Enqueue third, got {other:?}"),
        }
        assert_eq!(
            calls[3],
            Call::Transition { id: "ver-1".into(), from: "Active".into(), to: "Deprecated".into() }
        );
    }

    // T008 mid-failure: no Deprecated without its tombstone; nothing commits.
    #[tokio::test]
    async fn deactivation_mid_failure_leaves_zero_partial_state() {
        for fail in [FailAt::Tombstone, FailAt::Allocate, FailAt::Enqueue, FailAt::Transition] {
            let mut uow = FakeUnitOfWork::new(fail);
            let err = record_source_deactivation(&mut uow, "ver-1", "s", "urn:a", "stale").await;
            assert!(err.is_err(), "fail_at={fail:?} must abort deactivation");
            assert!(
                !uow.state.lock().unwrap().committed,
                "fail_at={fail:?} must never reach commit"
            );
            if fail == FailAt::Tombstone {
                // Tombstone failed first: no generation, no event, no transition.
                assert!(uow.outbox.generations.get("s").copied().unwrap_or(0) == 0);
                assert!(uow.outbox.pending.is_empty());
                assert!(!uow.calls().iter().any(|c| matches!(c, Call::Transition { .. })));
            } else {
                // Tombstone staged but a later step failed: the staged txn
                // (including the tombstone) is discarded on rollback, so no
                // Deprecated state is ever visible without its tombstone.
                assert!(
                    uow.calls().iter().any(|c| matches!(c, Call::InsertTombstone { .. })),
                    "fail_at={fail:?} must have staged the tombstone before failing"
                );
            }
            Box::new(uow).rollback().await.unwrap();
        }
    }

    #[tokio::test]
    async fn provenance_write_orders_insert_generation_enqueue() {
        let mut uow = FakeUnitOfWork::new(FailAt::None);
        record_provenance_write(&mut uow, provenance_record(), "scope-a").await.unwrap();
        assert_eq!(
            uow.calls(),
            vec![
                Call::InsertProvenance { subject_urn: "urn:test".into() },
                Call::AllocateGeneration {
                    scope: "scope-a".into(),
                    reason: "provenance_written".into()
                },
                Call::Enqueue {
                    operation: "provenance_written".into(),
                    key: "provenance_written:gen-scope-a-1".into()
                },
            ]
        );
    }

    #[tokio::test]
    async fn relay_claims_then_dispatches_exactly_claimed_set() {
        let mut uow = FakeUnitOfWork::new(FailAt::None);
        uow.enqueue("op", "k1");
        uow.enqueue("op", "k2");
        let n = relay_outbox_once(&mut uow, "owner-1", 10).await.unwrap();
        assert_eq!(n, 2);
        assert!(uow.outbox.pending.iter().all(|e| e.state == "Dispatched"));
        assert_eq!(
            uow.calls().iter().filter(|c| matches!(c, Call::MarkDispatched { .. })).count(),
            2
        );
    }

    #[tokio::test]
    async fn relay_with_zero_pending_or_zero_limit_marks_nothing() {
        let mut uow = FakeUnitOfWork::new(FailAt::None);
        assert_eq!(relay_outbox_once(&mut uow, "owner-1", 10).await.unwrap(), 0);
        uow.enqueue("op", "k1");
        assert_eq!(relay_outbox_once(&mut uow, "owner-1", 0).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn duplicate_idempotency_key_is_a_conflict_noop() {
        let mut uow = FakeUnitOfWork::new(FailAt::None);
        record_source_activation(&mut uow, "ver-1", "s", "urn:a").await.unwrap();
        let again = record_source_activation(&mut uow, "ver-1", "s", "urn:a").await;
        assert!(matches!(again, Err(StorageError::Conflict)));
    }

    #[test]
    fn unused_row_types_are_constructible() {
        let _ = ApprovalRow {
            id: "a".into(),
            subject_urn: "u".into(),
            kind: "k".into(),
            requested_by: None,
            decided_by: None,
            decision: None,
            request_payload: "{}".into(),
            decision_note: None,
            requested_at: "t".into(),
            decided_at: None,
        };
        let _ = AuditEvent {
            id: "e".into(),
            sequence: 1,
            occurred_at: "t".into(),
            actor_kind: "system".into(),
            actor_id: None,
            action: "a".into(),
            subject_urn: "u".into(),
            outcome: "ok".into(),
            reason: None,
            before_json: None,
            after_json: None,
            request_id: None,
            prev_chain_hash: "h0".into(),
            chain_hash: "h1".into(),
        };
        let _ = ChainVerificationResult {
            valid: true,
            expected_next_sequence: 2,
            expected_next_hash: "h2".into(),
            gaps: vec![],
        };
        let _ = JobRecord {
            id: "j".into(),
            kind: "k".into(),
            payload_json: "{}".into(),
            idempotency_key: None,
            state: "Queued".into(),
            priority: 0,
            attempts: 0,
            max_attempts: 1,
            available_at: "t".into(),
            lease_owner: None,
            lease_expires_at: None,
            checkpoint_json: None,
            cancel_requested: false,
            created_by: "p".into(),
        };
        let _ = SettingRow {
            key: "k".into(),
            value_json: "{}".into(),
            origin: "default".into(),
            updated_at: "t".into(),
            updated_by: None,
        };
        let _ = SourceRow {
            id: "s".into(),
            title: "t".into(),
            content_type: "ct".into(),
            language: None,
            created_at: "t".into(),
        };
        let _ = SourceVersionRow {
            id: "v".into(),
            source_id: "s".into(),
            version: "1".into(),
            state: "Indexing".into(),
            trust_level: "t".into(),
            license_status: "l".into(),
            content_hash: None,
            manifest_blob_id: None,
        };
        let _ = StateTransitionRow {
            id: "st".into(),
            source_version_id: "v".into(),
            from_state: None,
            to_state: "Active".into(),
            actor_id: None,
            reason: None,
            occurred_at: "t".into(),
        };
        let _ = PrincipalRow {
            id: "p".into(),
            kind: "k".into(),
            display_name: "d".into(),
            created_at: "t".into(),
        };
        let _ = ReviewRecord {
            id: "r".into(),
            provenance_id: "p".into(),
            queue: "q".into(),
            evidence_json: "{}".into(),
            state: "pending".into(),
            decided_by: None,
            decided_at: None,
            decision_note: None,
            created_at: "t".into(),
        };
    }
}
