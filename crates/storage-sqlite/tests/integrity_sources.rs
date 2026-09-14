//! `integrity_sources` — source lifecycle DB invariants (AC-P0-08).
//!
//! Exactly one `Active` version per source is enforced by the partial unique
//! index `ux_source_active`.

mod common;

#[tokio::test]
async fn at_most_one_active_version_per_source() {
    let fx = common::fixture().await;
    common::seed_source_version(&fx, "src-a", "ver-a1", "Active").await;

    let pool = common::rw_pool(&fx.path).await;
    let ts = common::now();
    let hash = format!("sha256:{}", "aa".repeat(32));
    let err = sqlx::query(
        "INSERT INTO source_versions
            (id, source_id, version, schema_version, state, trust_level, license_status,
             license_json, content_hash, validation_report, approved_by, activated_at,
             source_urls, created_at)
         VALUES ('ver-a2', 'src-a', '2.0.0', 1, 'Active', 'PublisherVerified', 'OpenLicense',
                 '{}', ?, '{}', 'principal', ?, '[]', ?)",
    )
    .bind(&hash)
    .bind(&ts)
    .bind(&ts)
    .execute(&pool)
    .await
    .unwrap_err()
    .to_string();

    assert!(
        err.to_lowercase().contains("unique"),
        "a second Active version must violate the partial unique index, got: {err}"
    );
}
