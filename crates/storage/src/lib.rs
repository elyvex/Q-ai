//! Storage abstraction layer — traits and error types for the persistence backend.
//!
//! # storage
//!
//! This crate defines the core storage traits (`Database`, `ReadTx`, `UnitOfWork`)
//! and repository traits used by all Q-ai subsystems. It also provides the
//! `StorageError` enum and `DbHealth`/`DbBackend` types.
//!
//! The storage crate depends only on `domain` (plus `serde`, `thiserror`,
//! `async-trait`). It contains **traits only** — the SQLite implementation
//! lives in `storage-sqlite`.
//!
//! ## Dependency rule (enforced by `cargo xtask arch-check`)
//!
//! `storage` -> `domain` (traits + errors only)

pub mod error;
pub mod repository;

use std::fmt;

// Re-export commonly used types
pub use error::StorageError;

/// The storage backend type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbBackend {
    /// SQLite backend (default for Phase 0).
    SQLite,
    /// PostgreSQL backend (Phase 12+).
    Postgres,
}

impl fmt::Display for DbBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SQLite => write!(f, "sqlite"),
            Self::Postgres => write!(f, "postgres"),
        }
    }
}

/// Health status of the database.
#[derive(Debug, Clone)]
pub struct DbHealth {
    /// Whether the database is reachable and responsive.
    pub healthy: bool,
    /// The backend type.
    pub backend: DbBackend,
    /// Current schema version.
    pub schema_version: u32,
    /// Human-readable status message.
    pub message: String,
}

impl DbHealth {
    /// Create a healthy status.
    pub fn ok(backend: DbBackend, schema_version: u32) -> Self {
        Self {
            healthy: true,
            backend,
            schema_version,
            message: "ok".to_string(),
        }
    }

    /// Create an unhealthy status.
    pub fn fail(backend: DbBackend, schema_version: u32, message: String) -> Self {
        Self {
            healthy: false,
            backend,
            schema_version,
            message,
        }
    }
}

/// The primary database interface.
///
/// Implementations provide read-only transactions via [`Database::read`]
/// and write transactions via [`Database::write`]. All methods are async
/// to allow run-time selection of connection pools.
#[async_trait::async_trait]
pub trait Database: Send + Sync {
    /// Obtain a read-only transaction handle.
    async fn read(&self) -> Result<Box<dyn ReadTx>, error::StorageError>;

    /// Obtain a writable unit of work. Serialized transaction semantics.
    async fn write(&self) -> Result<Box<dyn UnitOfWork>, error::StorageError>;

    /// Check database health.
    async fn health(&self) -> Result<DbHealth, error::StorageError>;

    /// Return the current schema version.
    fn schema_version(&self) -> u32;

    /// Return the backend type.
    fn backend(&self) -> DbBackend;
}

/// A read-only transaction handle.
///
/// Provides methods for querying data without mutation.
#[async_trait::async_trait]
pub trait ReadTx: Send {
    /// Return the current schema version visible through this transaction.
    fn schema_version(&self) -> u32;

    /// Execute a read-only query returning a scalar value.
    async fn query_one(&self, sql: &str) -> Result<Option<String>, error::StorageError>;
}

/// A unit of work representing a write transaction.
///
/// Provides access to all repository traits. Commit or rollback must be
/// called to finalize the transaction. The `Box<dyn UnitOfWork>` self
/// requirement allows async drop semantics.
#[async_trait::async_trait]
pub trait UnitOfWork: Send {
    /// Return the source repository for this transaction.
    fn sources(&mut self) -> &mut dyn repository::SourceRepository;

    /// Return the provenance repository for this transaction.
    fn provenance(&mut self) -> &mut dyn repository::ProvenanceRepository;

    /// Return the audit repository for this transaction.
    fn audit(&mut self) -> &mut dyn repository::AuditRepository;

    /// Return the job repository for this transaction.
    fn jobs(&mut self) -> &mut dyn repository::JobRepository;

    /// Return the settings repository for this transaction.
    fn settings(&mut self) -> &mut dyn repository::SettingsRepository;

    /// Commit the transaction, persisting all changes.
    async fn commit(self: Box<Self>) -> Result<(), error::StorageError>;

    /// Roll back the transaction, discarding all changes.
    async fn rollback(self: Box<Self>) -> Result<(), error::StorageError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_backend_display() {
        assert_eq!(DbBackend::SQLite.to_string(), "sqlite");
        assert_eq!(DbBackend::Postgres.to_string(), "postgres");
    }

    #[test]
    fn health_ok() {
        let h = DbHealth::ok(DbBackend::SQLite, 1);
        assert!(h.healthy);
        assert_eq!(h.schema_version, 1);
    }
}
