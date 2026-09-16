//! Phase 0 — Diagnostic trait and error-code registry.
//!
//! # domain::diagnostic
//!
//! The `Diagnostic` trait provides a standardized way to render errors
//! for both human and machine consumption. Every domain error must implement
//! this trait.

use std::fmt::{self, Write as FmtWrite};

use crate::redaction;

/// Unique identifier for a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiagnosticId(u64);

impl DiagnosticId {
    /// Create a diagnostic ID with an explicit numeric value.
    ///
    /// Intended for test/sample use; production code should use `DiagnosticId::new()`
    /// if random generation is preferred.
    pub fn explicit(value: u64) -> Self {
        Self(value)
    }
}

/// The severity of a diagnostic event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
    Fatal,
}

/// The category of a diagnostic, used for routing and filtering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticCategory {
    Validation,
    Security,
    Integrity,
    Storage,
    Network,
    Configuration,
    Provenance,
    Audit,
    Jobs,
    Sources,
    Observability,
}

/// A structured diagnostic event carrying all metadata needed for
/// human-readable and JSON rendering.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub id: DiagnosticId,
    pub timestamp: String,
    pub severity: DiagnosticSeverity,
    pub code: DiagnosticCode,
    pub category: DiagnosticCategory,
    pub message: String,
    pub location: Option<String>,
    pub affected_resource: Option<String>,
    pub remedy: Option<String>,
    pub next_command: Option<String>,
}

/// A machine-readable error code. Must be unique across the entire domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiagnosticCode {
    pub namespace: &'static str,
    pub code: u32,
}

impl Diagnostic {
    /// Render the diagnostic as a human-readable string.
    pub fn render_human(&self) -> String {
        let message = redaction::redact_text(&self.message);
        let location = self.location.as_ref().map(|l| redaction::redact_text(l));
        let affected_resource = self.affected_resource.as_ref().map(|r| redaction::redact_text(r));
        let remedy = self.remedy.as_ref().map(|r| redaction::redact_text(r));
        let next_command = self.next_command.as_ref().map(|c| redaction::redact_text(c));

        let mut out = String::new();
        let _ = write!(
            out,
            "[{}] {:?} [{}] {}",
            self.timestamp, self.severity, self.code, message
        );
        if let Some(ref loc) = location {
            let _ = write!(out, " at {}", loc);
        }
        if let Some(ref res) = affected_resource {
            let _ = write!(out, " (resource: {})", res);
        }
        if let Some(ref remedy) = remedy {
            let _ = write!(out, "\nRemedy: {}", remedy);
        }
        if let Some(ref cmd) = next_command {
            let _ = write!(out, "\nNext: {}", cmd);
        }
        out
    }

    /// Render the diagnostic as a compact JSON object.
    pub fn render_json(&self) -> String {
        let message = redaction::redact_text(&self.message);
        let location = self.location.as_ref().map(|l| redaction::redact_text(l));
        let affected_resource = self.affected_resource.as_ref().map(|r| redaction::redact_text(r));
        let remedy = self.remedy.as_ref().map(|r| redaction::redact_text(r));
        let next_command = self.next_command.as_ref().map(|c| redaction::redact_text(c));

        let mut out = String::new();
        let _ = write!(
            out,
            "{{\"id\":{},\"timestamp\":\"{}\",\"severity\":\"{:?}\",\"code\":\"{}\",\"category\":\"{:?}\",\"message\":\"{}\"",
            self.id.0, self.timestamp, self.severity, self.code, self.category, message
        );
        if let Some(ref loc) = location {
            let _ = write!(out, ",\"location\":\"{}\"", loc);
        }
        if let Some(ref res) = affected_resource {
            let _ = write!(out, ",\"affected_resource\":\"{}\"", res);
        }
        if let Some(ref remedy) = remedy {
            let _ = write!(out, ",\"remedy\":\"{}\"", remedy);
        }
        if let Some(ref cmd) = next_command {
            let _ = write!(out, ",\"next_command\":\"{}\"", cmd);
        }
        out.push('}');
        out
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render_human())
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Canonical form is `QAI-<NS>-<nnnn>` (D0.3 / docs/architecture/error-codes.md).
        write!(f, "{}-{:04}", self.namespace, self.code)
    }
}

// Domain-specific error codes using the QAI-DOM namespace.
pub mod codes {
    use super::DiagnosticCode;

    pub const INVALID_TRUST_TRANSITION: DiagnosticCode =
        DiagnosticCode { namespace: "QAI-DOM", code: 1001 };
    pub const DATA_LAYER_MISMATCH: DiagnosticCode =
        DiagnosticCode { namespace: "QAI-DOM", code: 1002 };
    pub const MISSING_DIAGNOSTIC_ID: DiagnosticCode =
        DiagnosticCode { namespace: "QAI-DOM", code: 1003 };
    pub const INVALID_DIAGNOSTIC_LEVEL: DiagnosticCode =
        DiagnosticCode { namespace: "QAI-DOM", code: 1004 };
    pub const INVALID_DIAGNOSTIC_CODE_FORMAT: DiagnosticCode =
        DiagnosticCode { namespace: "QAI-DOM", code: 1005 };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Diagnostic {
        Diagnostic {
            id: DiagnosticId(7),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            severity: DiagnosticSeverity::Error,
            code: codes::DATA_LAYER_MISMATCH,
            category: DiagnosticCategory::Integrity,
            message: "boom".to_string(),
            location: Some("config.toml:3".to_string()),
            affected_resource: Some("storage.sqlite.path".to_string()),
            remedy: Some("choose a writable path".to_string()),
            next_command: Some("qai doctor".to_string()),
        }
    }

    #[test]
    fn render_human_includes_all_populated_fields() {
        let rendered = sample().render_human();
        assert!(rendered.contains("boom"));
        assert!(rendered.contains("QAI-DOM-1002"));
        assert!(rendered.contains("config.toml:3"));
        assert!(rendered.contains("storage.sqlite.path"));
        assert!(rendered.contains("Remedy: choose a writable path"));
        assert!(rendered.contains("Next: qai doctor"));
        assert!(rendered.contains("Error"));
    }

    #[test]
    fn render_json_is_valid_and_carries_the_code() {
        let json = sample().render_json();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["message"], "boom");
        assert_eq!(value["code"], "QAI-DOM-1002");
        assert_eq!(value["location"], "config.toml:3");
        assert_eq!(value["remedy"], "choose a writable path");
        assert_eq!(value["next_command"], "qai doctor");
    }

    #[test]
    fn render_without_optional_fields_omits_them() {
        let mut diagnostic = sample();
        diagnostic.location = None;
        diagnostic.affected_resource = None;
        diagnostic.remedy = None;
        diagnostic.next_command = None;

        let human = diagnostic.render_human();
        assert!(!human.contains("Remedy"));
        assert!(!human.contains("Next"));
        let json: serde_json::Value = serde_json::from_str(&diagnostic.render_json()).unwrap();
        assert!(json.get("location").is_none());
        assert!(json.get("remedy").is_none());
    }

    #[test]
    fn display_matches_render_human() {
        let diagnostic = sample();
        assert_eq!(format!("{diagnostic}"), diagnostic.render_human());
    }

    #[test]
    fn severity_and_category_are_ordered_and_comparable() {
        assert!(DiagnosticSeverity::Info < DiagnosticSeverity::Warning);
        assert!(DiagnosticSeverity::Warning < DiagnosticSeverity::Error);
        assert!(DiagnosticSeverity::Error < DiagnosticSeverity::Fatal);
        assert_eq!(DiagnosticCategory::Security, DiagnosticCategory::Security);
        assert_ne!(DiagnosticCategory::Audit, DiagnosticCategory::Network);
    }

    #[test]
    fn codes_render_in_canonical_form() {
        assert_eq!(codes::INVALID_TRUST_TRANSITION.to_string(), "QAI-DOM-1001");
        assert_eq!(codes::INVALID_DIAGNOSTIC_CODE_FORMAT.to_string(), "QAI-DOM-1005");
        let unique: std::collections::HashSet<String> = [
            codes::INVALID_TRUST_TRANSITION,
            codes::DATA_LAYER_MISMATCH,
            codes::MISSING_DIAGNOSTIC_ID,
            codes::INVALID_DIAGNOSTIC_LEVEL,
            codes::INVALID_DIAGNOSTIC_CODE_FORMAT,
        ]
        .iter()
        .map(|c| c.to_string())
        .collect();
        assert_eq!(unique.len(), 5);
    }
}
