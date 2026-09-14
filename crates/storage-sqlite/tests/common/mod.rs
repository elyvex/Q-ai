//! Shared fixtures for `storage-sqlite` integration tests.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use storage_sqlite::{SqliteDatabase, migrate};
use tempfile::TempDir;

/// A migrated temporary database plus its on-disk path.
pub struct Fixture {
    pub _dir: TempDir,
    pub path: String,
    pub db: SqliteDatabase,
}

fn migrations_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite")
}

/// RFC3339 UTC timestamp.
pub fn now() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

pub async fn rw_pool(path: &str) -> sqlx::SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(path).foreign_keys(true))
        .await
        .expect("connect rw")
}

/// Create a migrated fixture database with a seeded principal.
pub async fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("qai.db");
    migrate::apply_migrations(path.to_str().unwrap(), &migrations_dir()).await.unwrap();
    let db = SqliteDatabase::new(path.to_str().unwrap(), 4, true).await.unwrap();
    let fx = Fixture { _dir: dir, path: path.to_str().unwrap().to_string(), db };
    let pool = rw_pool(&fx.path).await;
    sqlx::query(
        "INSERT OR IGNORE INTO principals (id, kind, display_name, created_at)
         VALUES ('principal', 'local_user', 'Test', ?)",
    )
    .bind(now())
    .execute(&pool)
    .await
    .unwrap();
    pool.close().await;
    fx
}

/// Seed a source and a source version in `state` (with approval preconditions
/// satisfied so `Active`/`Indexing`/`Approved` pass the DB CHECK).
pub async fn seed_source_version(fx: &Fixture, source_id: &str, version_id: &str, state: &str) {
    let pool = rw_pool(&fx.path).await;
    let ts = now();
    let hash = format!("sha256:{}", "aa".repeat(32));
    sqlx::query(
        "INSERT INTO sources (id, title, content_type, created_at, updated_at)
         VALUES (?, ?, 'quran_edition', ?, ?)",
    )
    .bind(source_id)
    .bind("Test source")
    .bind(&ts)
    .bind(&ts)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO source_versions
            (id, source_id, version, schema_version, state, trust_level, license_status,
             license_json, content_hash, validation_report, approved_by, approved_at,
             activated_at, source_urls, created_at)
         VALUES (?, ?, '1.0.0', 1, ?, 'PublisherVerified', 'OpenLicense', '{}', ?, ?, 'principal',
                 ?, ?, '[]', ?)",
    )
    .bind(version_id)
    .bind(source_id)
    .bind(state)
    .bind(&hash)
    .bind(r#"{"valid":true}"#)
    .bind(&ts)
    .bind(&ts)
    .bind(&ts)
    .execute(&pool)
    .await
    .unwrap();
    pool.close().await;
}

/// Current state of a source version.
pub async fn source_state(path: &str, version_id: &str) -> Option<String> {
    let pool = rw_pool(path).await;
    let row = sqlx::query_scalar::<_, String>("SELECT state FROM source_versions WHERE id = ?")
        .bind(version_id)
        .fetch_optional(&pool)
        .await
        .unwrap();
    pool.close().await;
    row
}

/// All generation numbers for a scope, ascending.
pub async fn generation_numbers(path: &str, scope: &str) -> Vec<i64> {
    let pool = rw_pool(path).await;
    let rows = sqlx::query_scalar::<_, i64>(
        "SELECT number FROM corpus_generations WHERE scope = ? ORDER BY number ASC",
    )
    .bind(scope)
    .fetch_all(&pool)
    .await
    .unwrap();
    pool.close().await;
    rows
}

/// Count outbox events in a state.
pub async fn outbox_count(path: &str, state: &str) -> i64 {
    let pool = rw_pool(path).await;
    let n = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM outbox_events WHERE state = ?")
        .bind(state)
        .fetch_one(&pool)
        .await
        .unwrap();
    pool.close().await;
    n
}

/// Count all tombstones for a subject.
pub async fn tombstone_count(path: &str, subject_urn: &str) -> i64 {
    let pool = rw_pool(path).await;
    let n = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tombstones WHERE subject_urn = ?")
        .bind(subject_urn)
        .fetch_one(&pool)
        .await
        .unwrap();
    pool.close().await;
    n
}

/// The state of a job.
pub async fn job_state(path: &str, id: &str) -> Option<String> {
    let pool = rw_pool(path).await;
    let row = sqlx::query_scalar::<_, String>("SELECT state FROM jobs WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .unwrap();
    pool.close().await;
    row
}

/// The checkpoint JSON of a job.
pub async fn job_checkpoint(path: &str, id: &str) -> Option<String> {
    let pool = rw_pool(path).await;
    let row =
        sqlx::query_scalar::<_, Option<String>>("SELECT checkpoint_json FROM jobs WHERE id = ?")
            .bind(id)
            .fetch_optional(&pool)
            .await
            .unwrap()
            .flatten();
    pool.close().await;
    row
}

/// The attempt counter of a job.
pub async fn job_attempts(path: &str, id: &str) -> i64 {
    let pool = rw_pool(path).await;
    let n = sqlx::query_scalar::<_, i64>("SELECT attempts FROM jobs WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    pool.close().await;
    n
}

/// Whether cancellation has been requested for a job.
pub async fn job_cancel_requested(path: &str, id: &str) -> bool {
    let pool = rw_pool(path).await;
    let n = sqlx::query_scalar::<_, i64>("SELECT cancel_requested FROM jobs WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    pool.close().await;
    n != 0
}

/// Force a job's lease into the past, simulating a crashed worker.
pub async fn backdate_lease(path: &str, id: &str) {
    let pool = rw_pool(path).await;
    sqlx::query("UPDATE jobs SET lease_expires_at = '2000-01-01T00:00:00Z' WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    pool.close().await;
}
