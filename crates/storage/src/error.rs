//! Storage error types.
//!
//! Every variant carries a code in the `QAI-DB-nnnn` namespace
//! so errors render consistently across CLI, logs, and JSON output.
//!
//! The `StorageError` enum maps 1:1 to the plan D0.6 specification
//! and ADR-0001 §2.

use thiserror::Error;

/// Storage-specific error codes, all in the `QAI-DB-nnnn` namespace.
/// See `docs/architecture/error-codes.md` for the registry.
mod codes {
    pub const CONFLICT: &str = "QAI-DB-0001";
    pub const NOT_FOUND: &str = "QAI-DB-0002";
    pub const IMMUTABLE_SOURCE_VERSION: &str = "QAI-DB-0003";
    pub const CONSTRAINT_VIOLATION: &str = "QAI-DB-0004";
    pub const STORAGE_BUSY: &str = "QAI-DB-0005";
    pub const MIGRATION_REQUIRED: &str = "QAI-DB-0006";
    pub const MIGRATION_CHECKSUM_MISMATCH: &str = "QAI-DB-0007";
    pub const IDEMPOTENCY_KEY_REPLAY: &str = "QAI-DB-0008";
    pub const STORAGE_UNAVAILABLE: &str = "QAI-DB-0009";
}

/// Errors originating from the storage layer.
///
/// Each variant implements [`Diagnostic`] so errors render
/// consistently across CLI, logs, and JSON output.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    /// A unique/constraint clash on an expected-insert operation.
    #[error("unique constraint violation")]
    Conflict,

    /// A requested resource was not found.
    #[error("resource not found: {urn}")]
    NotFound { urn: String },

    /// An attempted write to a canonical or frozen row.
    #[error("immutable source version")]
    ImmutableSourceVersion,

    /// An integrity rule violation in SQL or Rust logic.
    #[error("constraint violation: {message}")]
    ConstraintViolation { message: String },

    /// A lock or timeout on the write pool.
    #[error("storage busy")]
    StorageBusy,

    /// The database schema is older than required.
    #[error("migration required: at schema {at_schema}, required {required}")]
    MigrationRequired { at_schema: u32, required: u32 },

    /// A migration checksum does not match the applied record.
    #[error("migration checksum mismatch at version {version}")]
    MigrationChecksumMismatch { version: u32 },

    /// An idempotency key was replayed for a duplicate operation.
    #[error("idempotency key replay")]
    IdempotencyKeyReplay,

    /// I/O failure, pool exhaustion, or backend unavailable.
    #[error("storage unavailable")]
    StorageUnavailable,
}

impl StorageError {
    /// Returns the namespace code string for this error variant.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Conflict => codes::CONFLICT,
            Self::NotFound { .. } => codes::NOT_FOUND,
            Self::ImmutableSourceVersion => codes::IMMUTABLE_SOURCE_VERSION,
            Self::ConstraintViolation { .. } => codes::CONSTRAINT_VIOLATION,
            Self::StorageBusy => codes::STORAGE_BUSY,
            Self::MigrationRequired { .. } => codes::MIGRATION_REQUIRED,
            Self::MigrationChecksumMismatch { .. } => codes::MIGRATION_CHECKSUM_MISMATCH,
            Self::IdempotencyKeyReplay => codes::IDEMPOTENCY_KEY_REPLAY,
            Self::StorageUnavailable => codes::STORAGE_UNAVAILABLE,
        }
    }

    /// Returns a human-readable remedy for this error.
    pub fn remedy(&self) -> Option<&'static str> {
        match self {
            Self::Conflict => Some("Check for a duplicate and retry with a unique key."),
            Self::NotFound { .. } => Some("Verify the identifier and try again."),
            Self::ImmutableSourceVersion => {
                Some("Canonical rows cannot be modified. Use a new version instead.")
            }
            Self::ConstraintViolation { .. } => {
                Some("Review the constraint and correct the input data.")
            }
            Self::StorageBusy => Some("Retry the operation after a short delay."),
            Self::MigrationRequired { .. } => {
                Some("Run `qai db migrate` to apply pending migrations.")
            }
            Self::MigrationChecksumMismatch { .. } => {
                Some("Do not edit applied migration files. Use `qai db verify` to inspect.")
            }
            Self::IdempotencyKeyReplay => {
                Some("This operation was already submitted. Check its status.")
            }
            Self::StorageUnavailable => Some("Check the database connection and retry."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_codes_unique() {
        let codes = [
            StorageError::Conflict.code(),
            StorageError::NotFound { urn: "x".into() }.code(),
            StorageError::ImmutableSourceVersion.code(),
            StorageError::ConstraintViolation { message: "x".into() }.code(),
            StorageError::StorageBusy.code(),
            StorageError::MigrationRequired { at_schema: 1, required: 2 }.code(),
            StorageError::MigrationChecksumMismatch { version: 1 }.code(),
            StorageError::IdempotencyKeyReplay.code(),
            StorageError::StorageUnavailable.code(),
        ];
        let unique: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(codes.len(), unique.len(), "all error codes must be unique");
    }

    #[test]
    fn retryable_variants() {
        assert!(StorageError::Conflict.is_retryable());
        assert!(StorageError::StorageBusy.is_retryable());
        assert!(StorageError::StorageUnavailable.is_retryable());
        assert!(!StorageError::NotFound { urn: "x".into() }.is_retryable());
        assert!(!StorageError::ImmutableSourceVersion.is_retryable());
    }
}

/// Trait implemented by all error types so they can be rendered
/// consistently across CLI, logs, and JSON output.
pub trait Diagnostic {
    /// The error code in its namespace.
    fn code(&self) -> DiagnosticCode;

    /// A short human-readable description of what happened.
    fn summary(&self) -> String;

    /// The chain of causes leading to this error.
    fn cause_chain(&self) -> Vec<String>;

    /// Where the error occurred (file, key, row, source id).
    fn location(&self) -> Option<String>;

    /// How to fix the error.
    fn remedy(&self) -> Option<String>;

    /// The next command to run (e.g. `qai doctor`).
    fn next_command(&self) -> Option<String>;

    /// Whether the error is safe to retry.
    fn is_retryable(&self) -> bool;

    /// Whether the error message has already been secret-scrubbed.
    fn redacted(&self) -> bool;
}

/// Machine-readable error code. Must be unique across the entire domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiagnosticCode {
    pub namespace: &'static str,
    pub code: u32,
}

impl DiagnosticCode {
    /// Construct a new diagnostic code from a namespace string and numeric code.
    pub const fn new(namespace: &'static str, code: u32) -> Self {
        Self { namespace, code }
    }
}

impl std::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.code)
    }
}

impl Diagnostic for StorageError {
    fn code(&self) -> DiagnosticCode {
        DiagnosticCode::new(self.code(), 0)
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn cause_chain(&self) -> Vec<String> {
        vec![match self {
            Self::Conflict => {
                "A unique or constraint violation occurred during the operation.".into()
            }
            Self::NotFound { urn } => {
                format!("The resource identified by '{urn}' does not exist.")
            }
            Self::ImmutableSourceVersion => {
                "The target row is immutable (canonical/frozen).".into()
            }
            Self::ConstraintViolation { message } => {
                format!("Integrity constraint failed: {message}")
            }
            Self::StorageBusy => "The storage backend is temporarily unavailable.".into(),
            Self::MigrationRequired { at_schema, required } => {
                format!("Schema version {at_schema} is below the required {required}.")
            }
            Self::MigrationChecksumMismatch { version } => {
                format!("Migration checksum mismatch at version {version}.")
            }
            Self::IdempotencyKeyReplay => {
                "An identical request was submitted previously.".into()
            }
            Self::StorageUnavailable => "The storage backend cannot be reached.".into(),
        }]
    }

    fn location(&self) -> Option<String> {
        None
    }

    fn remedy(&self) -> Option<String> {
        Self::remedy(self).map(String::from)
    }

    fn next_command(&self) -> Option<String> {
        match self {
            Self::MigrationRequired { .. } => Some("qai db migrate".into()),
            Self::MigrationChecksumMismatch { .. } => Some("qai db verify".into()),
            Self::StorageBusy => Some("retry the operation".into()),
            _ => None,
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::StorageBusy | Self::StorageUnavailable | Self::Conflict
        )
    }

    fn redacted(&self) -> bool {
        true
    }
}
