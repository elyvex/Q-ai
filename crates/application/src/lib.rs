//! Application crate — Phase 0 orchestration layer (plan §3.1).
//!
//! This crate wires domain + storage + config + provenance + audit + jobs + sources.
//! Phase 0 keeps this minimal: a `run()` skeleton for future CLI/serve integration
//! that validates the config, initializes observability, and health-checks storage.

pub use config::Config;

use observability::{Format, init};
use storage::{
    error::StorageError, DbBackend, Database as StorageDatabase,
    ReadTx as StorageReadTx, UnitOfWork as StorageUnitOfWork,
};
use tracing::{error, info};

/// Result of the application run.
#[derive(Debug, Clone)]
pub struct RunResult {
    pub health: storage::DbHealth,
}

/// Entry point: validates config, initializes observability, health-checks storage.
pub async fn run(cfg: Config) -> Result<RunResult, StorageError> {
    init(Format::Text);
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
        let result = run(cfg).await;
        let rr = result.expect("run should succeed on a fresh temp db");
        assert!(rr.health.healthy);
        assert_eq!(rr.health.backend, DbBackend::SQLite);
    }
}
