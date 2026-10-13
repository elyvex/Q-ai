//! Retrieval errors.

/// Result alias for retrieval operations.
pub type Result<T> = std::result::Result<T, RetrievalError>;

/// Failures a retrieval backend can report.
#[derive(Debug, thiserror::Error)]
pub enum RetrievalError {
    /// The configured backend name is not one this build supports.
    #[error("unknown retrieval backend: {0}")]
    UnknownBackend(String),

    /// The backend was requested but this build was compiled without it.
    #[error("retrieval backend `{0}` is not compiled in (missing cargo feature)")]
    BackendUnavailable(&'static str),

    /// The query text was empty or whitespace only.
    #[error("query text must not be empty")]
    EmptyQuery,

    /// The backend's own failure.
    #[error("retrieval backend failed: {0}")]
    Backend(String),

    /// Filesystem or I/O failure (opening the store, writing a document).
    #[error("io failed: {0}")]
    Io(#[from] std::io::Error),
}

impl storage::error::Diagnostic for RetrievalError {
    fn code(&self) -> storage::error::DiagnosticCode {
        match self {
            Self::UnknownBackend(_) => storage::error::DiagnosticCode::new("QAI-RET", 1),
            Self::BackendUnavailable(_) => storage::error::DiagnosticCode::new("QAI-RET", 2),
            Self::EmptyQuery => storage::error::DiagnosticCode::new("QAI-RET", 3),
            Self::Backend(_) => storage::error::DiagnosticCode::new("QAI-RET", 4),
            Self::Io(_) => storage::error::DiagnosticCode::new("QAI-RET", 5),
        }
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn remedy(&self) -> Option<String> {
        Some(match self {
            Self::UnknownBackend(_) => "Set a supported backend, e.g. `memvid`.".to_string(),
            Self::BackendUnavailable(name) => {
                format!("Rebuild with the `{name}` cargo feature enabled.")
            }
            Self::EmptyQuery => "Provide non-empty query text.".to_string(),
            Self::Backend(_) => "Check the store file and retry.".to_string(),
            Self::Io(_) => "Check the data directory and permissions.".to_string(),
        })
    }

    fn next_command(&self) -> Option<String> {
        Some("qai --help".to_string())
    }
}
