//! Graph error model with stable `QAI-GRAPH-*` codes.
//!
//! Every error answers *what / why / how to fix / next command*, mirroring the
//! `quran-search` `Diagnostic` contract. The trait is redefined here (not
//! imported from `domain` or `storage`) so graph port types never require a
//! database or application crate.

use std::fmt;

/// Machine-readable error code in the `QAI-GRAPH` namespace.
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

/// The graph diagnostic contract. Only [`Diagnostic::code`] and
/// [`Diagnostic::summary`] are required; the rest have neutral defaults.
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

/// Stable `QAI-GRAPH-*` codes. New codes are appended, never renumbered.
pub mod codes {
    use super::DiagnosticCode;

    /// Requested projection is unknown or not active.
    pub const UNKNOWN_PROJECTION: DiagnosticCode = DiagnosticCode::new("QAI-GRAPH", 1);
    /// A traversal budget was invalid, exhausted, or cancelled.
    pub const BUDGET_EXCEEDED: DiagnosticCode = DiagnosticCode::new("QAI-GRAPH", 2);
    /// A typed pattern was rejected (unknown edge, oversize, raw text).
    pub const PATTERN_REJECTED: DiagnosticCode = DiagnosticCode::new("QAI-GRAPH", 3);
    /// A node stable ID does not resolve in this projection.
    pub const NODE_NOT_FOUND: DiagnosticCode = DiagnosticCode::new("QAI-GRAPH", 4);
    /// A projection build, stage, or verify step failed.
    pub const BUILD_FAILED: DiagnosticCode = DiagnosticCode::new("QAI-GRAPH", 5);
    /// The caller is not entitled to the requested graph content.
    pub const AUTHZ_DENIED: DiagnosticCode = DiagnosticCode::new("QAI-GRAPH", 6);
}

/// Graph errors.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GraphError {
    /// The requested projection id is unknown or not active in this store.
    #[error("unknown graph projection '{projection}'")]
    UnknownProjection {
        /// Requested projection id.
        projection: String,
    },
    /// A budget was invalid before execution, exhausted during execution,
    /// or cut short by the cancellation flag.
    #[error("graph budget exceeded: {detail}")]
    BudgetExceeded {
        /// Which budget fired and why.
        detail: String,
    },
    /// A typed pattern failed validation. No raw query text is accepted.
    #[error("graph pattern rejected: {detail}")]
    PatternRejected {
        /// Which rule fired and why.
        detail: String,
    },
    /// A node stable ID does not resolve in this projection.
    #[error("graph node '{stable_id}' not found")]
    NodeNotFound {
        /// Requested stable ID.
        stable_id: String,
    },
    /// A build, staging, or verification step failed.
    #[error("graph build failed at stage '{stage}': {detail}")]
    BuildFailed {
        /// Build stage (`reserve`, `stage`, `verify`, `publish`, …).
        stage: String,
        /// What went wrong.
        detail: String,
    },
    /// The caller is not entitled to the requested graph content.
    #[error("graph access denied: {detail}")]
    AuthzDenied {
        /// What was refused, without leaking hidden content.
        detail: String,
    },
}

impl Diagnostic for GraphError {
    fn code(&self) -> DiagnosticCode {
        match self {
            Self::UnknownProjection { .. } => codes::UNKNOWN_PROJECTION,
            Self::BudgetExceeded { .. } => codes::BUDGET_EXCEEDED,
            Self::PatternRejected { .. } => codes::PATTERN_REJECTED,
            Self::NodeNotFound { .. } => codes::NODE_NOT_FOUND,
            Self::BuildFailed { .. } => codes::BUILD_FAILED,
            Self::AuthzDenied { .. } => codes::AUTHZ_DENIED,
        }
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn location(&self) -> Option<String> {
        match self {
            Self::UnknownProjection { projection } => Some(format!("projection '{projection}'")),
            Self::NodeNotFound { stable_id } => Some(format!("node '{stable_id}'")),
            Self::BuildFailed { stage, .. } => Some(format!("build stage '{stage}'")),
            Self::BudgetExceeded { .. }
            | Self::PatternRejected { .. }
            | Self::AuthzDenied { .. } => None,
        }
    }

    fn remedy(&self) -> Option<String> {
        Some(match self {
            Self::UnknownProjection { .. } => {
                "List active projections with build inspection, then retry with a live projection id."
                    .to_string()
            }
            Self::BudgetExceeded { .. } => {
                "Raise the relevant budget explicitly, narrow the seeds, or accept the truncated result."
                    .to_string()
            }
            Self::PatternRejected { .. } => {
                "Use only allowlisted edge names with 1-8 typed steps; raw query text is never accepted."
                    .to_string()
            }
            Self::NodeNotFound { .. } => {
                "Check the stable ID spelling and the projection manifest, then retry.".to_string()
            }
            Self::BuildFailed { .. } => {
                "Inspect the staged build report, fix the inputs, then rerun the build.".to_string()
            }
            Self::AuthzDenied { .. } => {
                "Request access to the restricted assertion, or query the visible projection only."
                    .to_string()
            }
        })
    }

    fn next_command(&self) -> Option<String> {
        Some(match self {
            Self::UnknownProjection { .. } | Self::BuildFailed { .. } => {
                "qai graph build --help".to_string()
            }
            Self::BudgetExceeded { .. } => "qai graph subgraph --help".to_string(),
            Self::PatternRejected { .. } => "qai graph pattern --help".to_string(),
            Self::NodeNotFound { .. } => "qai graph neighbors --help".to_string(),
            Self::AuthzDenied { .. } => "qai graph inspect --help".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_use_graph_namespace() {
        let errs = [
            GraphError::UnknownProjection { projection: "p".into() },
            GraphError::BudgetExceeded { detail: "d".into() },
            GraphError::PatternRejected { detail: "d".into() },
            GraphError::NodeNotFound { stable_id: "s".into() },
            GraphError::BuildFailed { stage: "s".into(), detail: "d".into() },
            GraphError::AuthzDenied { detail: "d".into() },
        ];
        let rendered: Vec<String> = errs.iter().map(|e| e.code().to_string()).collect();
        assert!(rendered.iter().all(|c| c.starts_with("QAI-GRAPH-")));
        assert_eq!(rendered[0], "QAI-GRAPH-0001");
        assert_eq!(rendered[5], "QAI-GRAPH-0006");
    }

    #[test]
    fn human_render_carries_code_and_remedy() {
        let err = GraphError::PatternRejected { detail: "unknown edge".into() };
        let human = err.render_human();
        assert!(human.contains("QAI-GRAPH-0003"), "{human}");
        assert!(human.contains("Remedy:"), "{human}");
        assert!(human.contains("Next:"), "{human}");
    }
}
