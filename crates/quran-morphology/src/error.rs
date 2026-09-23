//! Phase 2 — `quran-morphology` error model.
//!
//! Every error carries a stable `QAI-MORPH-nnnn` code and answers *what /
//! why / how to fix / next command*, mirroring the `quran-search`
//! `Diagnostic` contract. The trait is redefined here (not imported) so
//! morphology types never require another Phase-2 crate.
//!
//! Numeric assignments (append-only, never renumbered):
//!
//! | Code | Variant | Meaning |
//! |---|---|---|
//! | `QAI-MORPH-0001` | `UnknownDataset` | Dataset slug/version not recognised |
//! | `QAI-MORPH-0002` | `ValidationFailed` | Adapter or MV-rule validation failure |
//! | `QAI-MORPH-0003` | `AlignmentFailed` | Alignment gate failure (unmatched tokens) |
//! | `QAI-MORPH-0004` | `UnavailableDataset` | Known dataset present but not loadable |
//! | `QAI-MORPH-0005` | `ReviewViolation` | Review-queue / suggestion policy breach |

use std::fmt;

/// Machine-readable error code in the `QAI-MORPH` namespace.
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

/// The morphology diagnostic contract, mirroring `quran-search`.
/// Only [`Diagnostic::code`] and [`Diagnostic::summary`] are required;
/// the rest have neutral defaults.
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

/// Stable `QAI-MORPH-*` codes. New codes are appended, never renumbered.
pub mod codes {
    use super::DiagnosticCode;

    /// Dataset slug/version not recognised.
    pub const UNKNOWN_DATASET: DiagnosticCode = DiagnosticCode::new("QAI-MORPH", 1);
    /// Adapter or MV-rule validation failure.
    pub const VALIDATION_FAILED: DiagnosticCode = DiagnosticCode::new("QAI-MORPH", 2);
    /// Alignment gate failure (unmatched tokens remain).
    pub const ALIGNMENT_FAILED: DiagnosticCode = DiagnosticCode::new("QAI-MORPH", 3);
    /// Known dataset present but not loadable.
    pub const UNAVAILABLE_DATASET: DiagnosticCode = DiagnosticCode::new("QAI-MORPH", 4);
    /// Review-queue / suggestion policy breach.
    pub const REVIEW_VIOLATION: DiagnosticCode = DiagnosticCode::new("QAI-MORPH", 5);
}

/// Morphology errors.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MorphologyError {
    /// The referenced dataset slug/version is not a known dataset.
    #[error("unknown dataset '{slug}:{version}': {detail}")]
    UnknownDataset {
        /// Dataset slug that was referenced.
        slug: String,
        /// Dataset version that was referenced.
        version: String,
        /// Why the reference is unknown.
        detail: String,
    },
    /// An adapter shape or MV-rule validation failure.
    #[error("morphology validation failed: {detail}")]
    ValidationFailed {
        /// What failed validation.
        detail: String,
    },
    /// An alignment gate failure (e.g. unmatched tokens remain).
    #[error("morphology alignment failed: {detail}")]
    AlignmentFailed {
        /// What failed to align.
        detail: String,
    },
    /// A known dataset that cannot currently be served.
    #[error("dataset '{slug}:{version}' unavailable: {detail}")]
    UnavailableDataset {
        /// Dataset slug.
        slug: String,
        /// Dataset version.
        version: String,
        /// Why it cannot be served.
        detail: String,
    },
    /// A review-queue / suggestion policy breach (opt-in, confidence
    /// floor, missing reviewer, missing explanation, …).
    #[error("review policy violation: {detail}")]
    ReviewViolation {
        /// Which policy was breached.
        detail: String,
    },
}

impl Diagnostic for MorphologyError {
    fn code(&self) -> DiagnosticCode {
        match self {
            Self::UnknownDataset { .. } => codes::UNKNOWN_DATASET,
            Self::ValidationFailed { .. } => codes::VALIDATION_FAILED,
            Self::AlignmentFailed { .. } => codes::ALIGNMENT_FAILED,
            Self::UnavailableDataset { .. } => codes::UNAVAILABLE_DATASET,
            Self::ReviewViolation { .. } => codes::REVIEW_VIOLATION,
        }
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn location(&self) -> Option<String> {
        match self {
            Self::UnknownDataset { slug, version, .. } => {
                Some(format!("dataset '{slug}:{version}'"))
            }
            Self::UnavailableDataset { slug, version, .. } => {
                Some(format!("dataset '{slug}:{version}'"))
            }
            Self::ValidationFailed { .. }
            | Self::AlignmentFailed { .. }
            | Self::ReviewViolation { .. } => None,
        }
    }

    fn remedy(&self) -> Option<String> {
        Some(match self {
            Self::UnknownDataset { .. } => {
                "Register the dataset slug/version before importing, or fix the reference."
                    .to_string()
            }
            Self::ValidationFailed { .. } => {
                "Inspect the MV-rule findings, fix the source rows, then re-run validation."
                    .to_string()
            }
            Self::AlignmentFailed { .. } => {
                "Inspect the per-surah unmatched report, then re-run alignment.".to_string()
            }
            Self::UnavailableDataset { .. } => {
                "Retry once the dataset snapshot is restored; do not substitute another dataset."
                    .to_string()
            }
            Self::ReviewViolation { .. } => {
                "Route the item through the review queue with reviewer, timestamp, and evidence."
                    .to_string()
            }
        })
    }

    fn next_command(&self) -> Option<String> {
        Some(match self {
            Self::UnknownDataset { .. } | Self::UnavailableDataset { .. } => {
                "qai quran datasets --list".to_string()
            }
            Self::ValidationFailed { .. } => "qai quran morphology validate --help".to_string(),
            Self::AlignmentFailed { .. } => "qai quran morphology align --report".to_string(),
            Self::ReviewViolation { .. } => "qai quran morphology review --pending".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_use_morph_namespace_and_documented_numbers() {
        assert_eq!(codes::UNKNOWN_DATASET.to_string(), "QAI-MORPH-0001");
        assert_eq!(codes::VALIDATION_FAILED.to_string(), "QAI-MORPH-0002");
        assert_eq!(codes::ALIGNMENT_FAILED.to_string(), "QAI-MORPH-0003");
        assert_eq!(codes::UNAVAILABLE_DATASET.to_string(), "QAI-MORPH-0004");
        assert_eq!(codes::REVIEW_VIOLATION.to_string(), "QAI-MORPH-0005");
    }

    #[test]
    fn every_variant_maps_to_its_code() {
        let errs = [
            MorphologyError::UnknownDataset {
                slug: "s".into(),
                version: "v".into(),
                detail: "d".into(),
            },
            MorphologyError::ValidationFailed { detail: "d".into() },
            MorphologyError::AlignmentFailed { detail: "d".into() },
            MorphologyError::UnavailableDataset {
                slug: "s".into(),
                version: "v".into(),
                detail: "d".into(),
            },
            MorphologyError::ReviewViolation { detail: "d".into() },
        ];
        let rendered: Vec<String> = errs.iter().map(|e| e.code().to_string()).collect();
        assert!(rendered.iter().all(|c| c.starts_with("QAI-MORPH-")));
        assert_eq!(rendered.len(), 5);
    }

    #[test]
    fn human_render_carries_code_remedy_and_next() {
        let err = MorphologyError::ValidationFailed { detail: "x".into() };
        let human = err.render_human();
        assert!(human.contains("QAI-MORPH-0002"), "{human}");
        assert!(human.contains("Remedy:"), "{human}");
        assert!(human.contains("Next:"), "{human}");
    }
}
