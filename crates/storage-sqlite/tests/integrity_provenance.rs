//! `integrity_provenance` — DB-level provenance invariants (AC-P0-06/07).
//!
//! Canonical provenance is insert-only (triggers), and the `CHECK` constraints
//! forbid an ill-formed computational annotation or an unreviewed
//! `human_verified` row.

mod common;

const CANONICAL_INSERT: &str = "INSERT INTO provenance_records
    (id, layer, subject_urn, attribution_kind, attribution_json, source_version_id,
     trust_level, verification_status, versions_json, created_at, created_by)
    VALUES ('p-canon', 'canonical_source', 'urn:qai:quran:1:1', 'dataset', '{}', 'ver-1',
     'CanonicalVerified', 'unverified', '{}', '2026-01-01T00:00:00Z', 'principal')";

async fn insert(pool: &sqlx::SqlitePool, sql: &str) -> Result<(), String> {
    sqlx::query(sql).execute(pool).await.map(|_| ()).map_err(|e| e.to_string())
}

#[tokio::test]
async fn canonical_provenance_is_immutable() {
    let fx = common::fixture().await;
    common::seed_source_version(&fx, "src-1", "ver-1", "Active").await;
    let pool = common::rw_pool(&fx.path).await;

    insert(&pool, CANONICAL_INSERT).await.unwrap();

    // Update is blocked by trg_prov_canonical_no_update.
    let update_err = insert(
        &pool,
        "UPDATE provenance_records SET subject_urn = 'urn:evil' WHERE id = 'p-canon'",
    )
    .await
    .expect_err("canonical update must be rejected");
    assert!(update_err.contains("QAI-PROV-0001"), "expected QAI-PROV-0001, got {update_err}");

    // Delete is blocked by trg_prov_canonical_no_delete.
    let delete_err = insert(&pool, "DELETE FROM provenance_records WHERE id = 'p-canon'")
        .await
        .expect_err("canonical delete must be rejected");
    assert!(delete_err.contains("QAI-PROV-0002"), "expected QAI-PROV-0002, got {delete_err}");

    // The row survives both attempts.
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM provenance_records WHERE id = 'p-canon'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn computational_annotation_requires_confidence() {
    let fx = common::fixture().await;
    common::seed_source_version(&fx, "src-2", "ver-2", "Active").await;
    let pool = common::rw_pool(&fx.path).await;

    let sql = "INSERT INTO provenance_records
        (id, layer, subject_urn, attribution_kind, attribution_json, source_version_id,
         trust_level, verification_status, confidence, versions_json, created_at, created_by)
        VALUES ('p-comp', 'computational_annotation', 'urn:qai:quran:1:2', 'computational', '{}',
         'ver-2', 'MachineGenerated', 'unverified', NULL, '{}', '2026-01-01T00:00:00Z', 'principal')";
    let err = insert(&pool, sql).await.unwrap_err();
    assert!(err.contains("CHECK"), "expected a CHECK violation for missing confidence, got {err}");

    // With confidence it is accepted.
    let ok = "INSERT INTO provenance_records
        (id, layer, subject_urn, attribution_kind, attribution_json, source_version_id,
         trust_level, verification_status, confidence, versions_json, created_at, created_by)
        VALUES ('p-comp2', 'computational_annotation', 'urn:qai:quran:1:3', 'computational', '{}',
         'ver-2', 'MachineGenerated', 'unverified', 0.8, '{}', '2026-01-01T00:00:00Z', 'principal')";
    insert(&pool, ok).await.unwrap();
}

#[tokio::test]
async fn human_verified_requires_a_reviewer() {
    let fx = common::fixture().await;
    common::seed_source_version(&fx, "src-3", "ver-3", "Active").await;
    let pool = common::rw_pool(&fx.path).await;

    let sql = "INSERT INTO provenance_records
        (id, layer, subject_urn, attribution_kind, attribution_json, source_version_id,
         trust_level, verification_status, confidence, versions_json, created_at, created_by, reviewed_by)
        VALUES ('p-rev', 'scholarly_annotation', 'urn:qai:quran:1:4', 'scholar', '{}',
         'ver-3', 'ScholarReviewed', 'human_verified', NULL, '{}', '2026-01-01T00:00:00Z', 'principal', NULL)";
    let err = insert(&pool, sql).await.unwrap_err();
    assert!(err.contains("CHECK"), "expected a CHECK violation for missing reviewer, got {err}");
}
