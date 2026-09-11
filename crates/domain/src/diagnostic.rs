//! Phase 0 — Diagnostic trait and error-code registry.
//!
//! # domain::diagnostic
//!
//! The `Diagnostic` trait provides a standardized way to render errors
//! for both human and machine consumption. Every domain error must implement
//! this trait.

use std::fmt::{self, Write as FmtWrite};

/// Unique identifier for a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiagnosticId(u64);

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
        let mut out = String::new();
        let _ = write!(
            out,
            "[{}] {:?} [{}] {}: {}",
            self.timestamp, self.severity, self.code.namespace, self.code.code, self.message
        );
        if let Some(ref loc) = self.location {
            let _ = write!(out, " at {}", loc);
        }
        if let Some(ref res) = self.affected_resource {
            let _ = write!(out, " (resource: {})", res);
        }
        if let Some(ref remedy) = self.remedy {
            let _ = write!(out, "\nRemedy: {}", remedy);
        }
        if let Some(ref cmd) = self.next_command {
            let _ = write!(out, "\nNext: {}", cmd);
        }
        out
    }

    /// Render the diagnostic as a compact JSON object.
    pub fn render_json(&self) -> String {
        let mut out = String::new();
        let _ = write!(
            out,
            "{{\"id\":{},\"timestamp\":\"{}\",\"severity\":\"{:?}\",\"code\":\"{}:{}\",\"category\":\"{:?}\",\"message\":\"{}\"",
            self.id.0,
            self.timestamp,
            self.severity,
            self.code.namespace,
            self.code.code,
            self.category,
            self.message
        );
        if let Some(ref loc) = self.location {
            let _ = write!(out, ",\"location\":\"{}\"", loc);
        }
        if let Some(ref res) = self.affected_resource {
            let _ = write!(out, ",\"affected_resource\":\"{}\"", res);
        }
        if let Some(ref remedy) = self.remedy {
            let _ = write!(out, ",\"remedy\":\"{}\"", remedy);
        }
        if let Some(ref cmd) = self.next_command {
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
        write!(f, "{}:{}", self.namespace, self.code)
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
