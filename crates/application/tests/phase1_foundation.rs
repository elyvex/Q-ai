//! Phase 1 — audited durable mutations (FND-04, D-09/D-10/D-12).
//!
//! Against real SQLite in a tempdir with the checked-in migrations: every
//! authoritative change commits domain state, provenance where required,
//! outbox where required, and one hash-chained audit event in a single
//! `UnitOfWork`. A failure at any required boundary — domain write,
//! provenance insert, outbox allocation/enqueue, audit append, or commit —
//! rolls the whole mutation back with zero partial rows. Tampering the
//! persisted chain reports the offending sequence with the `qai audit verify`
//! remedy. Read-only checks and derived-cache rebuilds stage no audit events
//! unless they alter authoritative state.

use std::sync::atomic::AtomicBool;

use application::audit_bridge::{
    AuditedMutation, audited_provenance_write, audited_source_activation, diagnose_invalid_audit,
    verify_persisted_audit,
};
use application::quran::{activate_edition, record_approval};
use application::quran_forms::{RebuildParams, rebuild_forms};
use audit::{Actor, AuditAction, HashChainWriter};
use domain::{ContentHash, HashAlgorithm, PrincipalId, SubjectRef, Timestamp};
use quran_corpus::import::{ImportInput, ImportOptions, ImportProgress, run_import};
use storage::Database as _;
use storage::repository::ProvenanceRecord;
use storage_sqlite::SqliteDatabase;

const BASE_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const PRINCIPAL: &str = "00000000-0000-0000-0000-000000000001";
const CREATED_AT: &str = "2026-09-26T00:00:00Z";
const SCOPE: &str = "quran:test";
const SOURCE_ID: &str = "src-fnd4";
const VERSION_ID: &str = "ver-fnd4";
const SUBJECT: &str = "urn:qai:source:src-fnd4";
const V1_URN: &str = "quran-edition:test-edition-min@0.1.0";

fn principal() -> PrincipalId {
    PRINCIPAL.parse().unwrap()
}

fn timestamp() -> Timestamp {
    Timestamp::from_ymd_hms(2026, 9, 26, 0, 0, 0).unwrap()
}

fn mutation(action: AuditAction, subject: &str) -> AuditedMutation {
    AuditedMutation::new(
        Actor::Principal { principal_id: principal() },
        action,
        SubjectRef(subject.to_string()),
    )
}

fn provenance_record(id: &str, subject: &str) -> ProvenanceRecord {
    ProvenanceRecord {
        id: id.into(),
        layer: "canonical_source".into(),
        subject_urn: subject.into(),
        attribution_kind: "dataset".into(),
        attribution_json: r#"{"dataset_name":"fnd4"}"#.into(),
        source_version_id: Some(VERSION_ID.into()),
        trust_level: "ImportedUnverified".into(),
        verification_status: "unverified".into(),
        confidence: None,
        versions_json: "{}".into(),
        created_by: PRINCIPAL.into(),
    }
}

async fn fixture() -> (tempfile::TempDir, SqliteDatabase, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("qai.db");
    let path_str = path.to_str().unwrap().to_string();
    let repo_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    storage_sqlite::migrate::apply_migrations(&path_str, &repo_root).await.unwrap();
    let db = SqliteDatabase::new(&path_str, 4, true).await.unwrap();
    let seed = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new().filename(&path_str).foreign_keys(true),
        )
        .await
        .unwrap();
    // Seed approval preconditions so `Indexing`/`Active` pass the DB CHECK
    // (mirrors `storage-sqlite/tests/common::seed_source_version`).
    let hash = format!("sha256:{}", "aa".repeat(32));
    for sql in [
        format!(
            "INSERT INTO principals (id, kind, display_name, created_at)
             VALUES ('{PRINCIPAL}', 'local_user', 'Test', '{CREATED_AT}')"
        ),
        format!(
            "INSERT INTO sources (id, title, content_type, created_at, updated_at)
             VALUES ('{SOURCE_ID}', 'FND-04 source', 'quran_edition', '{CREATED_AT}', '{CREATED_AT}')"
        ),
        format!(
            "INSERT INTO source_versions
                (id, source_id, version, schema_version, state, trust_level, license_status,
                 license_json, content_hash, validation_report, approved_by, approved_at,
                 activated_at, source_urls, created_at)
             VALUES ('{VERSION_ID}', '{SOURCE_ID}', '0.1.0', 1, 'Indexing', 'PublisherVerified',
                     'OpenLicense', '{{}}', '{hash}', '{{\"valid\":true}}', '{PRINCIPAL}',
                     '{CREATED_AT}', '{CREATED_AT}', '[]', '{CREATED_AT}')"
        ),
    ] {
        sqlx::query(&sql).execute(&seed).await.unwrap();
    }
    seed.close().await;
    (dir, db, path_str)
}

/// Open a scratch read/write pool on the fixture database (reads and test
/// fault-injection triggers only; all authoritative writes go through the
/// production repositories).
async fn scratch_pool(path: &str) -> sqlx::SqlitePool {
    sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(path).foreign_keys(true))
        .await
        .unwrap()
}

async fn source_state(path: &str, version_id: &str) -> Option<String> {
    let pool = scratch_pool(path).await;
    let row = sqlx::query_scalar::<_, String>("SELECT state FROM source_versions WHERE id = ?")
        .bind(version_id)
        .fetch_optional(&pool)
        .await
        .unwrap();
    pool.close().await;
    row
}

async fn count(path: &str, sql: &str) -> i64 {
    let pool = scratch_pool(path).await;
    let n = sqlx::query_scalar::<_, i64>(sql).fetch_one(&pool).await.unwrap();
    pool.close().await;
    n
}

async fn audit_count(path: &str) -> i64 {
    count(path, "SELECT COUNT(*) FROM audit_events").await
}

async fn outbox_pending(path: &str) -> i64 {
    count(path, "SELECT COUNT(*) FROM outbox_events WHERE state = 'Pending'").await
}

async fn provenance_count(path: &str) -> i64 {
    count(path, "SELECT COUNT(*) FROM provenance_records").await
}

async fn generation_numbers(path: &str) -> Vec<i64> {
    let pool = scratch_pool(path).await;
    let rows = sqlx::query_scalar::<_, i64>(
        "SELECT number FROM corpus_generations WHERE scope = ? ORDER BY number ASC",
    )
    .bind(SCOPE)
    .fetch_all(&pool)
    .await
    .unwrap();
    pool.close().await;
    rows
}

/// Install a failing `BEFORE INSERT` trigger: the next write to `table`
/// aborts, so the production code path returns its typed error and the
/// caller rolls the whole `UnitOfWork` back.
async fn fail_inserts_into(path: &str, table: &str) {
    let pool = scratch_pool(path).await;
    sqlx::query(&format!(
        "CREATE TRIGGER fnd4_fail_{table} BEFORE INSERT ON {table} \
         BEGIN SELECT RAISE(ABORT, 'fnd4 injected failure'); END"
    ))
    .execute(&pool)
    .await
    .unwrap();
    pool.close().await;
}

/// The new composition boundary commits domain state, provenance, outbox, and
/// one chained audit event together: a provenance write and a source
/// activation share one `UnitOfWork`, and commit makes all four record kinds
/// visible with a contiguous hash-linked audit sequence (D-09, D-10, D-12).
#[tokio::test]
async fn audited_mutation_commits_domain_provenance_outbox_and_audit_together() {
    let (_dir, db, path) = fixture().await;
    let mut uow = db.write().await.unwrap();
    let (prov_generation, prov_event) = audited_provenance_write(
        &mut *uow,
        provenance_record("prov-fnd4-1", SUBJECT),
        SCOPE,
        &mutation(AuditAction::SourceImported, SUBJECT),
    )
    .await
    .unwrap();
    let (act_generation, act_event) = audited_source_activation(
        &mut *uow,
        VERSION_ID,
        SCOPE,
        SUBJECT,
        &mutation(AuditAction::SourceActivated, SUBJECT),
    )
    .await
    .unwrap();
    uow.commit().await.unwrap();

    // The wrappers return their generation ids and sequenced audit events.
    assert!(!prov_generation.is_empty());
    assert!(!act_generation.is_empty());
    assert_ne!(prov_generation, act_generation);
    assert_eq!((prov_event.sequence, act_event.sequence), (1, 2));

    // Domain, provenance, and outbox rows are all visible after one commit.
    assert_eq!(source_state(&path, VERSION_ID).await.as_deref(), Some("Active"));
    assert_eq!(provenance_count(&path).await, 1);
    assert_eq!(outbox_pending(&path).await, 2);
    assert_eq!(generation_numbers(&path).await, vec![1, 2]);

    // The audit leg is contiguous and hash-linked (D-12).
    let mut uow = db.write().await.unwrap();
    let events = uow.audit().list_by_sequence(1, None).await.unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].action, "source_imported");
    assert_eq!(events[1].action, "source_activated");
    assert_eq!(events[1].prev_chain_hash, events[0].chain_hash);
    uow.rollback().await.unwrap();
    let report = verify_persisted_audit(&path).await.unwrap();
    assert!(report.valid, "chained events must verify: {report:?}");
    assert_eq!(report.checked_events, 2);
}

/// A failure at the audit boundary aborts the composition: the staged domain,
/// provenance, and outbox writes are discarded with the rollback and the
/// prior state is unchanged — no partial trust record (D-10).
#[tokio::test]
async fn audited_mutation_rolls_back_when_audit_append_fails() {
    let (_dir, db, path) = fixture().await;
    fail_inserts_into(&path, "audit_events").await;

    let mut uow = db.write().await.unwrap();
    let err = audited_source_activation(
        &mut *uow,
        VERSION_ID,
        SCOPE,
        SUBJECT,
        &mutation(AuditAction::SourceActivated, SUBJECT),
    )
    .await;
    assert!(err.is_err(), "injected audit failure must abort the composition");
    uow.rollback().await.unwrap();

    assert_eq!(
        source_state(&path, VERSION_ID).await.as_deref(),
        Some("Indexing"),
        "domain state must be unchanged"
    );
    assert_eq!(provenance_count(&path).await, 0, "no provenance row may survive");
    assert_eq!(outbox_pending(&path).await, 0, "no outbox row may survive");
    assert_eq!(audit_count(&path).await, 0, "no audit row may survive");
    assert!(generation_numbers(&path).await.is_empty(), "no generation may survive");
}

/// `record_approval` stages the approval row and its `ApprovalGranted` audit
/// event in the same transaction (D-09, D-10): both are visible together, and
/// the granted approval gates a later activation unchanged.
#[tokio::test]
async fn audited_mutation_records_approval_with_its_grant_event() {
    let (_dir, db, _path) = fixture().await;
    record_approval(&db, "appr-fnd4", SUBJECT, PRINCIPAL, PRINCIPAL, "{}", CREATED_AT)
        .await
        .unwrap();

    let mut uow = db.write().await.unwrap();
    let row = uow.sources().get_approval("appr-fnd4").await.unwrap().expect("approval row visible");
    assert_eq!(row.decision.as_deref(), Some("approved"));
    let events = uow.audit().list_by_subject(SUBJECT).await.unwrap();
    assert_eq!(events.len(), 1, "exactly the grant event for this subject");
    assert_eq!(events[0].action, "approval_granted");
    uow.rollback().await.unwrap();
}

/// `record_approval` fails closed on an unparseable actor (WR-10): a
/// non-principal `decided_by` is a typed error raised before any staging, so
/// no approval row and no grant event can survive a spoofed attribution.
#[tokio::test]
async fn record_approval_rejects_unparseable_actor_with_no_staged_rows() {
    let (_dir, db, _path) = fixture().await;
    let err = record_approval(&db, "appr-fnd10", SUBJECT, PRINCIPAL, "not-a-principal", "{}", CREATED_AT)
        .await
        .expect_err("a non-principal actor must fail closed");
    assert!(
        err.to_string().contains("not a principal id"),
        "typed actor error expected, got: {err}"
    );

    let mut uow = db.write().await.unwrap();
    assert!(
        uow.sources().get_approval("appr-fnd10").await.unwrap().is_none(),
        "no approval row may survive a rejected actor"
    );
    let events = uow.audit().list_by_subject(SUBJECT).await.unwrap();
    assert!(events.is_empty(), "no grant event may survive a rejected actor");
    uow.rollback().await.unwrap();
}

/// Failure matrix: a failure at EVERY required write boundary — domain write,
/// provenance insert, outbox allocation, outbox enqueue, audit append, and
/// commit — leaves zero partial rows and the previous state unchanged (D-10,
/// T-02-ATOMICITY).
#[tokio::test]
async fn audited_mutation_failure_matrix_rolls_back_every_boundary() {
    // 1. Domain write fails (transition from the wrong state): the
    // deactivation requires `Active` but the seeded version is `Indexing`,
    // so the workflow returns `NotFound` and nothing stages.
    {
        let (_dir, db, path) = fixture().await;
        let mut uow = db.write().await.unwrap();
        let bad = storage::workflows::record_source_deactivation(
            &mut *uow, VERSION_ID, SCOPE, SUBJECT, "stale",
        )
        .await;
        assert!(bad.is_err(), "deactivation from Indexing must fail");
        uow.rollback().await.unwrap();
        assert_eq!(source_state(&path, VERSION_ID).await.as_deref(), Some("Indexing"));
        assert_eq!(outbox_pending(&path).await, 0);
        assert_eq!(audit_count(&path).await, 0);
    }

    // 2. Provenance insert fails (duplicate id violates the PK): the staged
    // activation in the same UoW is discarded too.
    {
        let (_dir, db, path) = fixture().await;
        let mut uow = db.write().await.unwrap();
        uow.provenance().insert(provenance_record("prov-dup", SUBJECT)).await.unwrap();
        let dup = audited_provenance_write(
            &mut *uow,
            provenance_record("prov-dup", SUBJECT),
            SCOPE,
            &mutation(AuditAction::SourceImported, SUBJECT),
        )
        .await;
        assert!(dup.is_err(), "duplicate provenance id must abort the composition");
        uow.rollback().await.unwrap();
        assert_eq!(provenance_count(&path).await, 0, "staged provenance must not survive");
        assert_eq!(outbox_pending(&path).await, 0);
        assert_eq!(audit_count(&path).await, 0);
    }

    // 3. Outbox allocation fails (injected trigger on generations): the domain
    // write staged before it is discarded.
    {
        let (_dir, db, path) = fixture().await;
        fail_inserts_into(&path, "corpus_generations").await;
        let mut uow = db.write().await.unwrap();
        let err = audited_source_activation(
            &mut *uow,
            VERSION_ID,
            SCOPE,
            SUBJECT,
            &mutation(AuditAction::SourceActivated, SUBJECT),
        )
        .await;
        assert!(err.is_err(), "generation failure must abort the composition");
        uow.rollback().await.unwrap();
        assert_eq!(source_state(&path, VERSION_ID).await.as_deref(), Some("Indexing"));
        assert_eq!(outbox_pending(&path).await, 0);
        assert_eq!(audit_count(&path).await, 0);
    }

    // 4. Outbox enqueue fails (duplicate idempotency key is a Conflict): the
    // allocated generation and domain write are discarded.
    {
        let (_dir, db, path) = fixture().await;
        let mut uow = db.write().await.unwrap();
        audited_source_activation(
            &mut *uow,
            VERSION_ID,
            SCOPE,
            SUBJECT,
            &mutation(AuditAction::SourceActivated, SUBJECT),
        )
        .await
        .unwrap();
        let replay = audited_source_activation(
            &mut *uow,
            VERSION_ID,
            SCOPE,
            SUBJECT,
            &mutation(AuditAction::SourceActivated, SUBJECT),
        )
        .await;
        assert!(replay.is_err(), "replayed idempotency key must abort the composition");
        uow.rollback().await.unwrap();
        assert_eq!(source_state(&path, VERSION_ID).await.as_deref(), Some("Indexing"));
        assert_eq!(outbox_pending(&path).await, 0);
        assert_eq!(audit_count(&path).await, 0);
        assert!(generation_numbers(&path).await.is_empty());
    }

    // 5. Audit append fails (injected trigger): domain, provenance, and
    // outbox writes staged before it are discarded.
    {
        let (_dir, db, path) = fixture().await;
        fail_inserts_into(&path, "audit_events").await;
        let mut uow = db.write().await.unwrap();
        let err = audited_provenance_write(
            &mut *uow,
            provenance_record("prov-fnd4-5", SUBJECT),
            SCOPE,
            &mutation(AuditAction::SourceImported, SUBJECT),
        )
        .await;
        assert!(err.is_err(), "audit failure must abort the composition");
        uow.rollback().await.unwrap();
        assert_eq!(provenance_count(&path).await, 0);
        assert_eq!(outbox_pending(&path).await, 0);
        assert_eq!(audit_count(&path).await, 0);
    }

    // 6. Commit never happens (dropped/rolled-back UoW): staging alone
    // persists nothing, even with every write succeeding.
    {
        let (_dir, db, path) = fixture().await;
        let mut uow = db.write().await.unwrap();
        audited_provenance_write(
            &mut *uow,
            provenance_record("prov-fnd4-6", SUBJECT),
            SCOPE,
            &mutation(AuditAction::SourceImported, SUBJECT),
        )
        .await
        .unwrap();
        audited_source_activation(
            &mut *uow,
            VERSION_ID,
            SCOPE,
            SUBJECT,
            &mutation(AuditAction::SourceActivated, SUBJECT),
        )
        .await
        .unwrap();
        uow.rollback().await.unwrap();
        assert_eq!(source_state(&path, VERSION_ID).await.as_deref(), Some("Indexing"));
        assert_eq!(provenance_count(&path).await, 0);
        assert_eq!(outbox_pending(&path).await, 0);
        assert_eq!(audit_count(&path).await, 0);
        assert!(generation_numbers(&path).await.is_empty());
    }
}

/// A tampered persisted chain reports the exact offending sequence and names
/// `qai audit verify` as the recovery command (D-12, T-02-RECOVERY): the
/// broken predecessor link cannot be masked by row counts.
#[tokio::test]
async fn tampered_chain_reports_offending_sequence_with_verify_remedy() {
    let (_dir, db, path) = fixture().await;
    let mut uow = db.write().await.unwrap();
    audited_source_activation(
        &mut *uow,
        VERSION_ID,
        SCOPE,
        SUBJECT,
        &mutation(AuditAction::SourceActivated, SUBJECT),
    )
    .await
    .unwrap();
    // Second event through the same composition so the clean chain is 1→2.
    let second = AuditedMutation::new(
        Actor::Principal { principal_id: principal() },
        AuditAction::SourceImported,
        SubjectRef("urn:qai:source:src-fnd4-b".to_string()),
    );
    second.stage(&mut *uow).await.unwrap();
    uow.commit().await.unwrap();
    let clean = verify_persisted_audit(&path).await.unwrap();
    assert!(clean.valid, "staged chain must verify before tampering");
    assert_eq!(clean.checked_events, 2);

    // Simulate a tampered ledger: append a row whose predecessor link is
    // wrong (UPDATE/DELETE is impossible under the append-only triggers, so a
    // bad link is how a tampered ledger presents).
    let mut uow = db.write().await.unwrap();
    uow.audit()
        .append(storage::repository::AuditEvent {
            // UUID-shaped id and `algo:hex` hashes so the row reads back
            // through the production bridge; the predecessor link is still
            // wrong, which is exactly what the verifier must catch.
            id: "00000000-0000-4000-8000-000000000099".into(),
            sequence: 3,
            occurred_at: CREATED_AT.into(),
            actor_kind: "system".into(),
            actor_id: None,
            action: "source_activated".into(),
            subject_urn: SUBJECT.into(),
            outcome: "allowed".into(),
            reason: None,
            before_json: None,
            after_json: None,
            request_id: None,
            prev_chain_hash: format!("sha256:{}", "00".repeat(32)),
            chain_hash: format!("sha256:{}", "ff".repeat(32)),
        })
        .await
        .unwrap();
    uow.commit().await.unwrap();

    let broken = verify_persisted_audit(&path).await.unwrap();
    assert!(!broken.valid, "a broken predecessor link must fail verification");
    assert_eq!(broken.tampered_sequences, vec![3], "the offending sequence is reported");
    let diagnostic =
        diagnose_invalid_audit(&broken).expect("an invalid report maps to a diagnostic");
    assert_eq!(diagnostic.code, "QAI-AUD-0005");
    assert!(!diagnostic.remedy.is_empty(), "recovery guidance must be actionable");
    assert_eq!(diagnostic.next_command, "qai audit verify");
}

/// A deleted span reports the MISSING sequences and does not taint the
/// survivors (WR-05, D-12): chain [1,2,5] — the post-hole survivor of a
/// 3-4 deletion — reports `gaps == [3, 4]` with an empty tamper list and
/// maps to `QAI-AUD-0004`, so the operator restores rows 3-4 instead of
/// chasing an intact row 5 or a tamper that never happened.
#[tokio::test]
async fn deleted_span_reports_missing_sequences_without_tamper_cascade() {
    let (_dir, db, path) = fixture().await;
    let mut uow = db.write().await.unwrap();
    audited_source_activation(
        &mut *uow,
        VERSION_ID,
        SCOPE,
        SUBJECT,
        &mutation(AuditAction::SourceActivated, SUBJECT),
    )
    .await
    .unwrap();
    let second = AuditedMutation::new(
        Actor::Principal { principal_id: principal() },
        AuditAction::SourceImported,
        SubjectRef("urn:qai:source:src-fnd4-b".to_string()),
    );
    second.stage(&mut *uow).await.unwrap();
    uow.commit().await.unwrap();
    let clean = verify_persisted_audit(&path).await.unwrap();
    assert!(clean.valid, "staged chain must verify before simulating deletion");

    // The survivor's predecessor link: event 2's persisted chain hash.
    let mut uow = db.write().await.unwrap();
    let prior = uow.audit().list_by_sequence(2, Some(2)).await.unwrap();
    assert_eq!(prior.len(), 1, "event 2 must exist");
    let prev_hex = prior[0]
        .chain_hash
        .strip_prefix("sha256:")
        .expect("chain hashes render as sha256:<hex>")
        .to_string();
    let prev_hash = ContentHash { algorithm: HashAlgorithm::Sha256, hex: prev_hex };
    uow.rollback().await.unwrap();

    // Hand-built sequence-5 survivor, correctly linked to event 2: the only
    // anomaly is the missing 3-4 span, so no tamper flag may fire.
    let mut survivor = audit::AuditEvent {
        id: "00000000-0000-4000-8000-000000000098".parse().unwrap(),
        sequence: 5,
        occurred_at: timestamp(),
        actor: Actor::System { name: "system".to_string() },
        action: AuditAction::SourceActivated,
        subject: SubjectRef(SUBJECT.to_string()),
        outcome: audit::AuditOutcome::Allowed,
        reason: None,
        before: None,
        after: None,
        request_id: None,
        prev_chain_hash: prev_hash.clone(),
        chain_hash: ContentHash { algorithm: HashAlgorithm::Sha256, hex: String::new() },
    };
    survivor.chain_hash = HashChainWriter::compute_chain_hash(&prev_hash, &survivor);
    let mut uow = db.write().await.unwrap();
    uow.audit()
        .append(storage::repository::AuditEvent {
            id: survivor.id.to_string(),
            sequence: survivor.sequence,
            occurred_at: survivor.occurred_at.to_string(),
            actor_kind: "system".into(),
            actor_id: None,
            action: "source_activated".into(),
            subject_urn: SUBJECT.into(),
            outcome: "allowed".into(),
            reason: None,
            before_json: None,
            after_json: None,
            request_id: None,
            prev_chain_hash: format!("sha256:{}", prev_hash.hex),
            chain_hash: format!("sha256:{}", survivor.chain_hash.hex),
        })
        .await
        .unwrap();
    uow.commit().await.unwrap();

    let gapped = verify_persisted_audit(&path).await.unwrap();
    assert!(!gapped.valid, "a missing span must fail verification");
    assert_eq!(gapped.gaps, vec![3, 4], "the missing sequences are named");
    assert!(gapped.tampered_sequences.is_empty(), "a pure deletion is not tampering");
    let diagnostic =
        diagnose_invalid_audit(&gapped).expect("an invalid report maps to a diagnostic");
    assert_eq!(diagnostic.code, "QAI-AUD-0004");
    assert_eq!(diagnostic.next_command, "qai audit verify");
}

/// Read-only diagnostics and derived-cache rebuilds are not audited mutations
/// (D-09): reads that roll back stage nothing, and a full derived forms
/// rebuild — which writes derived rows plus its own Layer-D provenance —
/// stages zero audit events while leaving authoritative state untouched.
#[tokio::test]
async fn read_only_paths_and_derived_rebuilds_stage_no_audit_events() {
    let (_dir, db, _path) = fixture().await;

    // Read-only unit of work: reads across every repository, then rollback.
    let before = {
        let mut uow = db.write().await.unwrap();
        let seq = uow.audit().latest_sequence().await.unwrap();
        let _ = uow.sources().get(SOURCE_ID).await.unwrap();
        let _ = uow.sources().get_approval("missing").await.unwrap();
        let _ = uow.provenance().list_by_subject(SUBJECT).await.unwrap();
        let _ = uow.audit().list_by_subject(SUBJECT).await.unwrap();
        let _ = uow.audit().list_by_sequence(0, None).await.unwrap();
        let _ = uow.quran().get_active().await.unwrap();
        uow.rollback().await.unwrap();
        seq
    };
    let mut uow = db.write().await.unwrap();
    assert_eq!(uow.audit().latest_sequence().await.unwrap(), before);
    uow.rollback().await.unwrap();

    // Derived-cache rebuild on a real imported + activated edition.
    let edition_id = activate_min_edition(&db).await;
    let staged = audit_events_for(&db, V1_URN).await;
    assert!(!staged.is_empty(), "setup stages audit events for the edition");
    let report = rebuild_forms(
        &db,
        &RebuildParams {
            edition_slug: "test-edition-min".to_string(),
            edition_version: "0.1.0".to_string(),
            invoked_by: PRINCIPAL.to_string(),
            run_tag: "fnd4-no-audit".to_string(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("derived rebuild completes");
    assert!(report.token_forms > 0, "rebuild wrote derived rows");
    let after = audit_events_for(&db, V1_URN).await;
    assert_eq!(
        after.len(),
        staged.len(),
        "a derived-cache rebuild stages no audit events for the edition"
    );
    let mut uow = db.write().await.unwrap();
    let total = uow.audit().list_by_sequence(0, None).await.unwrap().len();
    uow.rollback().await.unwrap();
    assert_eq!(total, staged_total(&db).await, "no audit event anywhere from the rebuild");

    // Canonical rows are byte-identical: the rebuild touched derived state only.
    let mut uow = db.write().await.unwrap();
    let edition = uow
        .quran()
        .get_edition_by_slug_version("test-edition-min", "0.1.0")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(edition.id, edition_id);
    uow.rollback().await.unwrap();
}

async fn audit_events_for(
    db: &SqliteDatabase,
    subject: &str,
) -> Vec<storage::repository::AuditEvent> {
    let mut uow = db.write().await.unwrap();
    let events = uow.audit().list_by_subject(subject).await.unwrap();
    uow.rollback().await.unwrap();
    events
}

async fn staged_total(db: &SqliteDatabase) -> usize {
    let mut uow = db.write().await.unwrap();
    let events = uow.audit().list_by_sequence(0, None).await.unwrap();
    uow.rollback().await.unwrap();
    events.len()
}

/// Import the min fixture, record its approval through the audited
/// composition, and activate it: the shared setup for the derived-rebuild
/// proof. Returns the canonical edition id.
async fn activate_min_edition(db: &SqliteDatabase) -> String {
    let input = ImportInput {
        run_id: "run-fnd4".into(),
        job_id: None,
        source_version_id: "sv-fnd4".into(),
        adapter: "json".into(),
        manifest_text: BASE_MANIFEST.into(),
        declared_manifest_hash: None,
        invoked_by: PRINCIPAL.into(),
        license_status: "PublicDomain".into(),
        license_json: "{}".into(),
        created_at: CREATED_AT.into(),
        reference_manifest_text: None,
    };
    // Catalog rows the importer FKs into (mirrors the quran_import fixtures).
    seed_import_catalog(db).await;
    run_import(
        db,
        &input,
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .expect("min fixture imports");
    record_approval(db, "appr-fnd4-min", V1_URN, PRINCIPAL, PRINCIPAL, "{}", CREATED_AT)
        .await
        .expect("approval records with its grant event");
    activate_edition(db, "test-edition-min", "0.1.0", &principal(), "appr-fnd4-min", &timestamp())
        .await
        .expect("fixture activation completes");
    let db_read = db;
    let mut uow = db_read.write().await.unwrap();
    let edition = uow
        .quran()
        .get_edition_by_slug_version("test-edition-min", "0.1.0")
        .await
        .unwrap()
        .unwrap();
    let id = edition.id.clone();
    uow.rollback().await.unwrap();
    id
}

async fn seed_import_catalog(db: &SqliteDatabase) {
    let mut uow = db.write().await.unwrap();
    uow.sources()
        .insert_source(storage::repository::SourceRow {
            id: "src-fnd4-min".into(),
            title: "FND-04 min".into(),
            content_type: "quran_edition".into(),
            language: Some("ar".into()),
            created_at: CREATED_AT.into(),
        })
        .await
        .unwrap();
    uow.sources()
        .insert_version(storage::repository::SourceVersionRow {
            id: "sv-fnd4".into(),
            source_id: "src-fnd4-min".into(),
            version: "0.1.0".into(),
            state: "Staged".into(),
            trust_level: "ImportedUnverified".into(),
            license_status: "PublicDomain".into(),
            content_hash: None,
            manifest_blob_id: None,
        })
        .await
        .unwrap();
    uow.provenance().insert(provenance_record("prov-fnd4-min", "test")).await.unwrap();
    uow.commit().await.unwrap();
}
