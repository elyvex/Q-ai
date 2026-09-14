//! Phase 1 — `quran-corpus` error model.
//!
//! `QAI-QUR-02xx` covers corpus infrastructure: intermediate-format parsing,
//! adapters, and (later) import/activation. Content findings use QV rule ids in
//! the `ValidationReport`, not error codes.

use quran_core::error::{Diagnostic, DiagnosticCode};

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
    pub const ADAPTER_FAILED: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 203);
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
}

impl Diagnostic for CorpusError {
    fn code(&self) -> DiagnosticCode {
        match self {
            Self::UnsupportedFormat { .. } => codes::UNSUPPORTED_FORMAT,
            Self::InvalidFormat { .. } => codes::INVALID_FORMAT,
            Self::UnknownAdapter { .. } => codes::UNKNOWN_ADAPTER,
            Self::AdapterFailed { .. } => codes::ADAPTER_FAILED,
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
        ];
        let unique: HashSet<_> = rendered.iter().collect();
        assert_eq!(rendered.len(), unique.len());
        assert!(rendered.iter().all(|c| c.starts_with("QAI-QUR-02")));
    }
}
