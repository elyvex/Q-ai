//! Phase 2 — `quran-search` error model.
//!
//! Every error carries a stable `QAI-IDX-nnnn` code and answers *what / why /
//! where / how to fix / next command*, mirroring the Phase-0/Phase-1
//! `Diagnostic` contracts. The trait is redefined here (not imported from
//! `storage`) so index/search types never require a database crate.

use std::fmt;

/// Machine-readable error code in the `QAI-IDX` namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiagnosticCode {
    pub namespace: &'static str,
    pub code: u32,
}

impl DiagnosticCode {
    /// Construct a code from a namespace string and numeric component.
    pub const fn new(namespace: &'static str, code: u32) -> Self {
        Self { namespace, code }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{:04}", self.namespace, self.code)
    }
}

/// The Phase-2 index/search diagnostic contract. Only [`Diagnostic::code`]
/// and [`Diagnostic::summary`] are required; the rest have neutral defaults.
pub trait Diagnostic {
    /// The error code in its namespace.
    fn code(&self) -> DiagnosticCode;

    /// A short human-readable description of what happened.
    fn summary(&self) -> String;

    /// The chain of causes leading to this error.
    fn cause_chain(&self) -> Vec<String> {
        Vec::new()
    }

    /// Where the error occurred.
    fn location(&self) -> Option<String> {
        None
    }

    /// How to fix the error.
    fn remedy(&self) -> Option<String> {
        None
    }

    /// The next command to run.
    fn next_command(&self) -> Option<String> {
        None
    }

    /// Whether the error is safe to retry.
    fn is_retryable(&self) -> bool {
        false
    }

    /// Whether the message has already been secret-scrubbed.
    fn redacted(&self) -> bool {
        true
    }

    /// Render the diagnostic as a stable human-readable string.
    fn render_human(&self) -> String {
        let mut out = format!("[{}] {}", self.code(), self.summary());
        if let Some(loc) = self.location() {
            out.push_str(&format!(" at {loc}"));
        }
        if let Some(remedy) = self.remedy() {
            out.push_str(&format!("\nRemedy: {remedy}"));
        }
        if let Some(cmd) = self.next_command() {
            out.push_str(&format!("\nNext: {cmd}"));
        }
        out
    }
}

/// Stable `QAI-IDX-*` codes. New codes are appended, never renumbered.
pub mod codes {
    use super::DiagnosticCode;

    /// Requested backend is not available in this build.
    pub const BACKEND_UNAVAILABLE: DiagnosticCode = DiagnosticCode::new("QAI-IDX", 1);
    /// Query rejected (guardrail: pattern, size, or anchor rule).
    pub const QUERY_REJECTED: DiagnosticCode = DiagnosticCode::new("QAI-IDX", 2);
    /// Index build, commit, or verification failed.
    pub const BUILD_FAILED: DiagnosticCode = DiagnosticCode::new("QAI-IDX", 3);
    /// Index manifest disagrees with the requested build inputs.
    pub const MANIFEST_MISMATCH: DiagnosticCode = DiagnosticCode::new("QAI-IDX", 4);
    /// Canonical text changed under a build (MV-018). Always fatal: no
    /// derived artifact may ship, and the build must stop, not warn.
    pub const CANONICAL_CHANGED: DiagnosticCode = DiagnosticCode::new("QAI-IDX", 5);
    /// A search hit failed assembly validation (trace/span/quotation fault).
    pub const INVALID_HIT: DiagnosticCode = DiagnosticCode::new("QAI-IDX", 6);
    /// Index is stale relative to corpus/profile/dataset inputs (warning,
    /// never an error: drift is reported, never auto-repaired).
    pub const STALE_INDEX: DiagnosticCode = DiagnosticCode::new("QAI-IDX", 101);
}

/// Index/search errors.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IndexError {
    /// The requested backend has no adapter in this build.
    #[error("index backend '{backend}' is unavailable: {reason}")]
    BackendUnavailable {
        /// Backend name.
        backend: String,
        /// Why it cannot serve.
        reason: String,
    },
    /// A query was rejected by a safety guard.
    #[error("query rejected: {detail}")]
    QueryRejected {
        /// Which guard fired and why.
        detail: String,
    },
    /// A build/commit/verify step failed.
    #[error("index build failed at stage '{stage}': {detail}")]
    BuildFailed {
        /// Build stage (`stage`, `verify`, `flip`, …).
        stage: String,
        /// What went wrong.
        detail: String,
    },
    /// A manifest disagrees with the expected inputs.
    #[error("index manifest mismatch: {detail}")]
    ManifestMismatch {
        /// Expected vs actual.
        detail: String,
    },
    /// Canonical text changed under a build (MV-018).
    ///
    /// Recomputed canonical hashes disagree with the stored edition hashes.
    /// The build stops immediately; nothing derived ships.
    #[error("canonical text changed under build for {edition_urn}")]
    CanonicalChanged {
        /// Edition under build.
        edition_urn: String,
        /// Stored hash the build expected.
        expected_hash: String,
        /// Recomputed hash actually found.
        actual_hash: String,
    },
    /// A search hit failed assembly validation.
    ///
    /// Empty token lists, out-of-range spans, bad numbers or hashes, or a
    /// quotation the Phase-1 constructor rejects. Hits are fail-closed: no
    /// unverifiable hit is ever returned (AC-P2-12 mechanism).
    #[error("invalid search hit: {detail}")]
    InvalidHit {
        /// What failed validation.
        detail: String,
    },
}

impl Diagnostic for IndexError {
    fn code(&self) -> DiagnosticCode {
        match self {
            Self::BackendUnavailable { .. } => codes::BACKEND_UNAVAILABLE,
            Self::QueryRejected { .. } => codes::QUERY_REJECTED,
            Self::BuildFailed { .. } => codes::BUILD_FAILED,
            Self::ManifestMismatch { .. } => codes::MANIFEST_MISMATCH,
            Self::CanonicalChanged { .. } => codes::CANONICAL_CHANGED,
            Self::InvalidHit { .. } => codes::INVALID_HIT,
        }
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn location(&self) -> Option<String> {
        match self {
            Self::BackendUnavailable { backend, .. } => Some(format!("backend '{backend}'")),
            Self::BuildFailed { stage, .. } => Some(format!("build stage '{stage}'")),
            Self::CanonicalChanged { edition_urn, .. } => {
                Some(format!("canonical text of {edition_urn}"))
            }
            Self::InvalidHit { .. } => None,
            Self::QueryRejected { .. } | Self::ManifestMismatch { .. } => None,
        }
    }

    fn remedy(&self) -> Option<String> {
        Some(match self {
            Self::BackendUnavailable { .. } => {
                "Use the FTS5 backend, or enable the requested adapter.".to_string()
            }
            Self::QueryRejected { .. } => {
                "Rewrite the query within the documented limits.".to_string()
            }
            Self::BuildFailed { .. } => {
                "Inspect the build report, then rerun `qai quran index rebuild`.".to_string()
            }
            Self::ManifestMismatch { .. } => {
                "Rebuild the index from current inputs, then retry.".to_string()
            }
            Self::CanonicalChanged { .. } => {
                "Treat as an integrity incident: do not rebuild over it, investigate the canonical store first.".to_string()
            }
            Self::InvalidHit { .. } => {
                "Fix the assembling tool: hits must carry a trace, a span inside the text, and valid references.".to_string()
            }
        })
    }

    fn next_command(&self) -> Option<String> {
        Some(match self {
            Self::QueryRejected { .. } => "qai quran normalize --list-profiles".to_string(),
            Self::BuildFailed { .. } => "qai quran index rebuild --all".to_string(),
            Self::BackendUnavailable { .. }
            | Self::CanonicalChanged { .. }
            | Self::InvalidHit { .. }
            | Self::ManifestMismatch { .. } => "qai doctor --indexes".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_use_idx_namespace() {
        let errs = [
            IndexError::BackendUnavailable { backend: "x".into(), reason: "y".into() },
            IndexError::QueryRejected { detail: "x".into() },
            IndexError::BuildFailed { stage: "x".into(), detail: "y".into() },
            IndexError::ManifestMismatch { detail: "x".into() },
            IndexError::CanonicalChanged {
                edition_urn: "e".into(),
                expected_hash: "a".into(),
                actual_hash: "b".into(),
            },
        ];
        let rendered: Vec<String> = errs.iter().map(|e| e.code().to_string()).collect();
        assert!(rendered.iter().all(|c| c.starts_with("QAI-IDX-")));
        assert_eq!(rendered[0], "QAI-IDX-0001");
        assert_eq!(codes::STALE_INDEX.to_string(), "QAI-IDX-0101");
        assert_eq!(codes::CANONICAL_CHANGED.to_string(), "QAI-IDX-0005");
        assert_eq!(codes::INVALID_HIT.to_string(), "QAI-IDX-0006");
    }

    #[test]
    fn human_render_carries_code_and_remedy() {
        let err = IndexError::QueryRejected { detail: "leading .*".into() };
        let human = err.render_human();
        assert!(human.contains("QAI-IDX-0002"), "{human}");
        assert!(human.contains("Remedy:"), "{human}");
    }
}
