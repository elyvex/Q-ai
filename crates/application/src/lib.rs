//! Application crate — Phase 0 orchestration layer (plan §3.1).
//!
//! Composition root that wires domain + storage + storage-sqlite + config +
//! provenance + audit + jobs + sources + observability. The CLI depends on this
//! crate (never on `storage-sqlite` directly) so the concrete backend choice
//! stays in one place.

pub mod audit_bridge;
pub mod db;
pub mod job_queue;
pub mod quran;
pub mod quran_cli;
pub mod quran_doctor;
pub mod quran_forms;
pub mod quran_index;
pub mod quran_normalize;
pub mod quran_reader;
pub mod quran_search;
pub mod quran_search_api;
pub mod quran_search_cache;
pub mod quran_tools;

pub use config::Config;
pub use domain::redaction;

use observability::{Format, InitOptions, init_with_options};
use storage::Database as _;
use storage::error::StorageError;
use tracing::{error, info};

/// Result of the application `run` bootstrap.
#[derive(Debug, Clone)]
pub struct RunResult {
    pub health: storage::DbHealth,
}

/// Entry point: validates config, initializes observability, health-checks storage.
pub async fn run(cfg: Config) -> Result<RunResult, StorageError> {
    init_with_options(InitOptions {
        format: Format::Text,
        redact_secrets: cfg.logging.redact_secrets,
    });
    info!("qai application starting (Phase 0)");

    let db = storage_sqlite::SqliteDatabase::new(
        &cfg.storage.sqlite.path,
        cfg.storage.sqlite.max_connections,
        cfg.storage.sqlite.read_only_pool,
    )
    .await?;

    let health = db.health().await?;
    if !health.healthy {
        error!("database health check failed: {}", health.message);
        return Err(StorageError::StorageUnavailable);
    }

    info!(
        "application ready; schema_version={}, backend={:?}",
        health.schema_version, health.backend
    );

    Ok(RunResult { health })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn run_health_checks_the_sqlite_backend() {
        let dir = tempdir().unwrap();
        let mut cfg = Config::default();
        cfg.app.data_dir = dir.path().display().to_string();
        cfg.storage.sqlite.path = dir.path().join("qai.db").display().to_string();
        let rr = run(cfg).await.expect("run should succeed");
        assert!(rr.health.healthy);
        assert_eq!(rr.health.backend, storage::DbBackend::SQLite);
    }
}
