//! Phase 1 — `quran-corpus` error model.
//!
//! `QAI-QUR-02xx` covers corpus infrastructure: intermediate-format parsing,
//! adapters, and (later) import/activation. Content findings use QV rule ids in
//! the `ValidationReport`, not error codes.

use quran_core::error::{Diagnostic, DiagnosticCode};

/// Re-exported so downstream crates implement one diagnostic contract without
/// depending on `quran-core` directly (invariant I2 keeps `quran-core` narrow).
pub use quran_core::error::{Diagnostic as QuranDiagnostic, DiagnosticCode as QuranDiagnosticCode};

/// Corpus infrastructure error codes.
pub mod codes {
    use super::DiagnosticCode;

    /// The `format` tag or `format_version` is not `qai.quran.edition` v1.
    pub const UNSUPPORTED_FORMAT: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 200);
    /// The edition source is structurally or semantically invalid.
    pub const INVALID_FORMAT: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 201);
    /// No adapter is registered under the requested name.
    pub const UNKNOWN_ADAPTER: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 202);
    /// A registered adapter failed to parse its input.
    pub const ADAPTER_FAILED: DiagnosticCode =
        DiagnosticCode::new("QAI-QUR", 203);
    /// Content validation found `Fatal` findings; the edition cannot stage.
    pub const VALIDATION_FAILED: DiagnosticCode =
        DiagnosticCode::new("QAI-QUR", 204);
    /// The import was cancelled; staging was cleaned up.
    pub const IMPORT_CANCELLED: DiagnosticCode =
        DiagnosticCode::new("QAI-QUR", 205);
    /// An import step failed after validation (round-trip, diff, state).
    pub const IMPORT_FAILED: DiagnosticCode =
        DiagnosticCode::new("QAI-QUR", 206);
    /// The storage backend failed during import.
    pub const STORAGE_FAILED: DiagnosticCode =
        DiagnosticCode::new("QAI-QUR", 207);
}
/// Errors originating in the corpus pipeline.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CorpusError {
    /// The `format` tag or `format_version` is unsupported.
    #[error("unsupported edition format `{found}` (expected `qai.quran.edition` v1)")]
    UnsupportedFormat {
        /// The declared format tag/version.
        found: String,
    },

    /// The edition source is structurally or semantically invalid.
    #[error("invalid edition source: {detail}")]
    InvalidFormat {
        /// What was wrong.
        detail: String,
    },

    /// No adapter is registered under the requested name.
    #[error("unknown adapter `{name}`")]
    UnknownAdapter {
        /// The requested adapter name.
        name: String,
    },

    /// A registered adapter failed to parse its input.
    #[error("adapter `{adapter}` failed: {detail}")]
    AdapterFailed {
        /// The adapter that failed.
        adapter: &'static str,
        /// What was wrong.
        detail: String,
    },

    /// Content validation found `Fatal` findings.
    #[error("validation failed with {fatal_count} fatal finding(s) (report {report_id})")]
    ValidationFailed {
        /// The import run id.
        run_id: String,
        /// The persisted validation report id.
        report_id: String,
        /// Number of fatal findings.
        fatal_count: u32,
    },

    /// The import was cancelled.
    #[error("import {run_id} cancelled at {checkpoint}")]
    ImportCancelled {
        /// The import run id.
        run_id: String,
        /// The checkpoint reached before cancellation.
        checkpoint: String,
    },

    /// An import step failed after validation.
    #[error("import step {step} failed: {detail}")]
    ImportFailed {
        /// The step that failed.
        step: &'static str,
        /// What was wrong.
        detail: String,
    },

    /// The storage backend failed during import.
    #[error("storage failed during import: {detail}")]
    StorageFailed {
        /// What was wrong.
        detail: String,
    },
}

impl Diagnostic for CorpusError {
    fn code(&self) -> DiagnosticCode {
        match self {
            Self::UnsupportedFormat { .. } => codes::UNSUPPORTED_FORMAT,
            Self::InvalidFormat { .. } => codes::INVALID_FORMAT,
            Self::UnknownAdapter { .. } => codes::UNKNOWN_ADAPTER,
            Self::AdapterFailed { .. } => codes::ADAPTER_FAILED,
            Self::ValidationFailed { .. } => codes::VALIDATION_FAILED,
            Self::ImportCancelled { .. } => codes::IMPORT_CANCELLED,
            Self::ImportFailed { .. } => codes::IMPORT_FAILED,
            Self::StorageFailed { .. } => codes::STORAGE_FAILED,
        }
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn remedy(&self) -> Option<String> {
        Some(
            match self {
                Self::UnsupportedFormat { .. } => {
                    "Convert the dataset to `qai.quran.edition` v1 (see the adapter guide)."
                }
                Self::InvalidFormat { .. } => {
                    "Fix the reported field and re-run the adapter; validate with `qai quran validate`."
                }
                Self::UnknownAdapter { .. } => "Use a registered adapter (`json`, `csv`).",
                Self::AdapterFailed { .. } => "Fix the reported input problem and retry the import.",
                Self::ValidationFailed { .. } => {
                    "Fix the fatal findings (see the validation report) and re-import."
                }
                Self::ImportCancelled { .. } => "Re-run the import; staging was cleaned up.",
                Self::ImportFailed { .. } => "Fix the reported problem and retry the import.",
                Self::StorageFailed { .. } => "Check the database and retry the import.",
            }
            .to_string(),
        )
    }

    fn next_command(&self) -> Option<String> {
        Some("qai quran validate".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn corpus_codes_are_unique_and_namespaced() {
        let rendered = [
            CorpusError::UnsupportedFormat { found: "x".into() }.code().to_string(),
            CorpusError::InvalidFormat { detail: "x".into() }.code().to_string(),
            CorpusError::UnknownAdapter { name: "x".into() }.code().to_string(),
            CorpusError::AdapterFailed { adapter: "json", detail: "x".into() }.code().to_string(),
            CorpusError::ValidationFailed {
                run_id: "r".into(),
                report_id: "v".into(),
                fatal_count: 1,
            }
            .code()
            .to_string(),
            CorpusError::ImportCancelled { run_id: "r".into(), checkpoint: "c".into() }
                .code()
                .to_string(),
            CorpusError::ImportFailed { step: "s", detail: "x".into() }.code().to_string(),
            CorpusError::StorageFailed { detail: "x".into() }.code().to_string(),
        ];
        let unique: HashSet<_> = rendered.iter().collect();
        assert_eq!(rendered.len(), unique.len());
        assert!(rendered.iter().all(|c| c.starts_with("QAI-QUR-02")));
    }
}
