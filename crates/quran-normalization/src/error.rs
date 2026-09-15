//! Phase 2 — `quran-normalization` error model.
//!
//! Every error carries a stable `QAI-NORM-nnnn` code and answers *what / why /
//! where / how to fix / next command*, mirroring the Phase-1 `QAI-QUR-*`
//! contract (`quran-core::error`). The trait is redefined here (not imported
//! from `storage`) because `quran-normalization` must never depend on `storage`
//! (arch rule AC-P2-36).
//!
//! No new `QAI-QUR-*` codes are created here; those remain Phase-1-owned.

use std::fmt;

/// Machine-readable error code in the `QAI-NORM` namespace.
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

/// The Phase-2 normalization diagnostic contract. Only [`Diagnostic::code`]
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

    /// Render the diagnostic as a stable JSON object string.
    fn render_json(&self) -> String {
        let mut out = String::from("{\"code\":");
        push_json_string(&mut out, &self.code().to_string());
        out.push_str(",\"summary\":");
        push_json_string(&mut out, &self.summary());
        out.push_str(",\"why\":[");
        for (i, cause) in self.cause_chain().iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            push_json_string(&mut out, cause);
        }
        out.push(']');
        out.push_str(",\"location\":");
        push_json_opt(&mut out, self.location().as_deref());
        out.push_str(",\"remedy\":");
        push_json_opt(&mut out, self.remedy().as_deref());
        out.push_str(",\"next_command\":");
        push_json_opt(&mut out, self.next_command().as_deref());
        out.push_str(",\"retryable\":");
        out.push_str(if self.is_retryable() { "true" } else { "false" });
        out.push('}');
        out
    }
}

/// Append `"..."` with the mandatory JSON escapes (no allocation-heavy deps).
fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

fn push_json_opt(out: &mut String, value: Option<&str>) {
    match value {
        Some(v) => push_json_string(out, v),
        None => out.push_str("null"),
    }
}

/// Stable `QAI-NORM-*` codes. New codes are appended, never renumbered.
pub mod codes {
    use super::DiagnosticCode;

    /// Unknown normalization rule id.
    pub const UNKNOWN_RULE: DiagnosticCode = DiagnosticCode::new("QAI-NORM", 1);
    /// Unknown normalization profile id.
    pub const UNKNOWN_PROFILE: DiagnosticCode = DiagnosticCode::new("QAI-NORM", 2);
    /// Attempt to mutate an immutable (registered) profile.
    pub const PROFILE_IMMUTABLE: DiagnosticCode = DiagnosticCode::new("QAI-NORM", 3);
    /// Span offset out of range for the mapped text.
    pub const SPAN_OUT_OF_RANGE: DiagnosticCode = DiagnosticCode::new("QAI-NORM", 4);
    /// Inconsistent mapping supplied to a `SpanMap` constructor.
    pub const INVALID_MAPPING: DiagnosticCode = DiagnosticCode::new("QAI-NORM", 5);
    /// A trace or profile was constructed without the mandatory profile label.
    pub const EMPTY_PROFILE: DiagnosticCode = DiagnosticCode::new("QAI-NORM", 6);
}

/// Normalization errors. Construction sites must supply enough context for
/// `remedy` / `next_command` to be actionable.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NormalizationError {
    /// A rule id with no registered implementation was requested.
    #[error("unknown normalization rule '{rule}'")]
    UnknownRule {
        /// The unrecognized rule id string.
        rule: String,
    },
    /// A profile id with no registered definition was requested.
    #[error("unknown normalization profile '{profile}'")]
    UnknownProfile {
        /// The unrecognized profile id string.
        profile: String,
    },
    /// A caller attempted to change a registered (append-only) profile.
    #[error("normalization profile '{profile}' is immutable; create a new version instead")]
    ProfileImmutable {
        /// The profile id that was targeted.
        profile: String,
    },
    /// A span range fell outside the mapped text.
    #[error("span {start}..{end} out of range for text of length {len}")]
    SpanOutOfRange {
        /// Range start (chars).
        start: u32,
        /// Range end (chars, exclusive).
        end: u32,
        /// Length of the mapped text (chars).
        len: u32,
    },
    /// A `SpanMap` constructor received an inconsistent mapping.
    #[error("invalid offset mapping: {detail}")]
    InvalidMapping {
        /// What was inconsistent.
        detail: String,
    },
    /// A trace or profile was constructed without its mandatory profile label.
    ///
    /// The label is what ties a derived result to the exact rule set that
    /// produced it; an unlabeled result is unverifiable (I9).
    #[error("normalization result requires a profile label")]
    EmptyProfile,
}

impl Diagnostic for NormalizationError {
    fn code(&self) -> DiagnosticCode {
        match self {
            Self::UnknownRule { .. } => codes::UNKNOWN_RULE,
            Self::UnknownProfile { .. } => codes::UNKNOWN_PROFILE,
            Self::ProfileImmutable { .. } => codes::PROFILE_IMMUTABLE,
            Self::SpanOutOfRange { .. } => codes::SPAN_OUT_OF_RANGE,
            Self::InvalidMapping { .. } => codes::INVALID_MAPPING,
            Self::EmptyProfile => codes::EMPTY_PROFILE,
        }
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn location(&self) -> Option<String> {
        match self {
            Self::UnknownRule { rule } => Some(format!("rule '{rule}'")),
            Self::UnknownProfile { profile } | Self::ProfileImmutable { profile } => {
                Some(format!("profile '{profile}'"))
            }
            Self::SpanOutOfRange { .. } | Self::InvalidMapping { .. } => None,
            Self::EmptyProfile => Some("trace/profile label".to_string()),
        }
    }

    fn remedy(&self) -> Option<String> {
        Some(match self {
            Self::UnknownRule { .. } => {
                "Use a rule id from the N01..N24 catalog (ADR-0204).".to_string()
            }
            Self::UnknownProfile { .. } => {
                "Use a registered profile id (L0..L8) or an adhoc:<hash> set.".to_string()
            }
            Self::ProfileImmutable { .. } => {
                "Register a new profile version and rebuild derived data.".to_string()
            }
            Self::SpanOutOfRange { .. } => {
                "Clamp the range to the derived text length before mapping.".to_string()
            }
            Self::InvalidMapping { .. } => {
                "Rebuild the map from the rule application output.".to_string()
            }
            Self::EmptyProfile => "Always build results through a profiled pipeline.".to_string(),
        })
    }

    fn next_command(&self) -> Option<String> {
        Some(match self {
            Self::UnknownRule { .. } | Self::UnknownProfile { .. } => {
                "qai quran normalize --list-profiles".to_string()
            }
            Self::ProfileImmutable { .. } => "qai doctor --indexes".to_string(),
            Self::SpanOutOfRange { .. } | Self::InvalidMapping { .. } => {
                "qai doctor --indexes".to_string()
            }
            Self::EmptyProfile => "qai quran normalize --list-profiles".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_use_norm_namespace() {
        let errs = [
            NormalizationError::UnknownRule { rule: "N99".into() },
            NormalizationError::UnknownProfile { profile: "L9".into() },
            NormalizationError::ProfileImmutable { profile: "L3".into() },
            NormalizationError::SpanOutOfRange { start: 0, end: 9, len: 3 },
            NormalizationError::InvalidMapping { detail: "x".into() },
            NormalizationError::EmptyProfile,
        ];
        let rendered: Vec<String> = errs.iter().map(|e| e.code().to_string()).collect();
        assert!(rendered.iter().all(|c| c.starts_with("QAI-NORM-")));
        assert_eq!(rendered[0], "QAI-NORM-0001");
        assert_eq!(codes::PROFILE_IMMUTABLE.to_string(), "QAI-NORM-0003");
        assert_eq!(codes::EMPTY_PROFILE.to_string(), "QAI-NORM-0006");
    }

    #[test]
    fn human_and_json_render_carry_code_and_remedy() {
        let err = NormalizationError::UnknownRule { rule: "N99".into() };
        let human = err.render_human();
        assert!(human.contains("QAI-NORM-0001"), "{human}");
        assert!(human.contains("Remedy:"), "{human}");
        let json = err.render_json();
        assert!(json.contains("\"code\":\"QAI-NORM-0001\""), "{json}");
        assert!(json.contains("\"remedy\":"), "{json}");
        assert!(json.contains("\"next_command\":"), "{json}");
    }
}
