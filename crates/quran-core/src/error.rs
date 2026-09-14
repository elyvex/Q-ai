//! Phase 1 — `quran-core` error model.
//!
//! # quran_core::error
//!
//! Every error carries a stable `QAI-QUR-nnnn` code and answers *what / why /
//! where / how to fix / next command*, exactly like the Phase-0 `Diagnostic`
//! contract.
//!
//! The `Diagnostic` trait is defined here rather than reusing
//! `storage::error::Diagnostic` because invariant I2 forbids `quran-core` from
//! depending on `storage`: the canonical read path must not require a database
//! crate. Its shape mirrors the Phase-0 trait method for method.

use std::fmt;

/// Machine-readable error code in the `QAI-QUR` namespace.
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

/// The Phase-1 diagnostic contract. Only [`Diagnostic::code`] and
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
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
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

/// Quran-core error codes. See `docs/architecture/error-codes.md`.
///
/// `0001`–`0005` are the canonical-table trigger codes (raised from SQL).
/// `0006`–`0099` are Rust construction errors. `0100`–`0199` are the reference
/// grammar (ADR-0102).
pub mod codes {
    use super::DiagnosticCode;

    // Canonical-table triggers (D1.5 / migration 0011).
    /// `quran_ayahs` text is immutable.
    pub const CANONICAL_AYAH_IMMUTABLE: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 1);
    /// Edition identity and hashes are immutable.
    pub const EDITION_IDENTITY_IMMUTABLE: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 2);
    /// Canonical ayah rows cannot be deleted.
    pub const CANONICAL_AYAH_NO_DELETE: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 3);
    /// Canonical tokens are immutable.
    pub const CANONICAL_TOKEN_IMMUTABLE: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 4);
    /// Canonical tokens cannot be deleted.
    pub const CANONICAL_TOKEN_NO_DELETE: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 5);

    // Construction errors.
    /// A surah number outside `1..=114`.
    pub const INVALID_SURAH_NUMBER: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 6);
    /// An ayah number of zero.
    pub const INVALID_AYAH_NUMBER: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 7);
    /// A token position of zero.
    pub const INVALID_TOKEN_POSITION: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 8);
    /// A required textual field was empty or whitespace-only.
    pub const EMPTY_FIELD: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 9);
    /// An edition slug with invalid characters or an empty value.
    pub const INVALID_SLUG: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 10);
    /// A translation was supplied without a non-empty translator (principle 5).
    pub const MISSING_TRANSLATOR: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 11);
    /// A quotation was constructed without an edition identity (invariant I6).
    pub const QUOTATION_MISSING_EDITION: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 12);
    /// A quotation was constructed without a content hash (invariant I6).
    pub const QUOTATION_MISSING_HASH: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 13);

    // Reference grammar errors (ADR-0102).
    /// The reference string was empty.
    pub const REFERENCE_EMPTY: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 100);
    /// The edition segment (`slug` or `slug@version`) was malformed.
    pub const INVALID_EDITION: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 101);
    /// The `@version` part of an edition segment was not `MAJOR.MINOR.PATCH`.
    pub const INVALID_EDITION_VERSION: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 102);
    /// The surah number in a reference was out of range.
    pub const INVALID_SURAH: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 103);
    /// The ayah number in a reference was zero or malformed.
    pub const INVALID_AYAH: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 104);
    /// The token position in a reference was zero or malformed.
    pub const INVALID_REFERENCE_POSITION: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 105);
    /// A range was malformed.
    pub const INVALID_RANGE: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 106);
    /// The division keyword was unknown.
    pub const INVALID_DIVISION: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 107);
    /// The division number was zero or malformed.
    pub const INVALID_DIVISION_NUMBER: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 108);
    /// Trailing or misplaced input remained after a complete reference.
    pub const UNEXPECTED_INPUT: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 109);
    /// The reference exceeded the maximum parseable length.
    pub const REFERENCE_TOO_LONG: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 110);
    /// A range ended before it started.
    pub const RANGE_ORDER: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 111);
    /// A locator was required but missing.
    pub const MISSING_LOCATOR: DiagnosticCode = DiagnosticCode::new("QAI-QUR", 112);
}
/// Errors originating in the Quran domain.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum QuranError {
    /// A surah number outside `1..=114`.
    #[error("surah number out of range: {number} (valid 1..=114)")]
    InvalidSurahNumber { number: u16 },

    /// An ayah number of zero.
    #[error("ayah number must be >= 1, got 0")]
    InvalidAyahNumber,

    /// A token position of zero.
    #[error("token position must be >= 1, got 0")]
    InvalidTokenPosition,

    /// A required textual field was empty or whitespace-only.
    #[error("required field `{field}` is empty")]
    EmptyField { field: &'static str },

    /// An edition slug with invalid characters or an empty value.
    #[error("invalid edition slug: `{slug}`")]
    InvalidSlug { slug: String },

    /// A translation was supplied without a non-empty translator.
    #[error("a translation requires a non-empty translator (principle 5)")]
    MissingTranslator,

    /// A quotation was constructed without an edition identity or a hash.
    #[error("a quotation requires an edition id, version, and content hash")]
    IncompleteQuotation,

    /// A reference string that failed to parse.
    #[error("invalid reference `{input}`: {detail}")]
    InvalidReference {
        /// The specific grammar failure.
        code: DiagnosticCode,
        /// The offending input.
        input: String,
        /// What was wrong with it.
        detail: String,
    },
}

impl Diagnostic for QuranError {
    fn code(&self) -> DiagnosticCode {
        match self {
            Self::InvalidSurahNumber { .. } => codes::INVALID_SURAH_NUMBER,
            Self::InvalidAyahNumber => codes::INVALID_AYAH_NUMBER,
            Self::InvalidTokenPosition => codes::INVALID_TOKEN_POSITION,
            Self::EmptyField { .. } => codes::EMPTY_FIELD,
            Self::InvalidSlug { .. } => codes::INVALID_SLUG,
            Self::MissingTranslator => codes::MISSING_TRANSLATOR,
            Self::IncompleteQuotation => codes::QUOTATION_MISSING_EDITION,
            Self::InvalidReference { code, .. } => *code,
        }
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn cause_chain(&self) -> Vec<String> {
        vec![match self {
            Self::InvalidSurahNumber { number } => {
                format!("The value {number} is not a surah number (1..=114).")
            }
            Self::InvalidAyahNumber => "Ayah numbering is 1-based; 0 is not a valid ayah.".into(),
            Self::InvalidTokenPosition => {
                "Token positions are 1-based; 0 is not a valid position.".into()
            }
            Self::EmptyField { field } => {
                format!("The canonical field `{field}` must contain non-whitespace text.")
            }
            Self::InvalidSlug { slug } => {
                format!("The slug `{slug}` must match [a-z0-9-] and be non-empty.")
            }
            Self::MissingTranslator => {
                "A translated text must never be presented as the original; it needs a named translator.".into()
            }
            Self::IncompleteQuotation => {
                "Canonical quotations must carry the edition identity and text hash (invariant I6)."
                    .into()
            }
            Self::InvalidReference { detail, .. } => {
                format!("{detail} Expected `[quran:] [edition:] locator` (ADR-0102).")
            }
        }]
    }

    fn remedy(&self) -> Option<String> {
        Some(
            match self {
                Self::InvalidSurahNumber { .. } => "Use a surah number in 1..=114.",
                Self::InvalidAyahNumber | Self::InvalidTokenPosition => {
                    "Use a 1-based number greater than zero."
                }
                Self::EmptyField { .. } => "Populate the field from the approved source.",
                Self::InvalidSlug { .. } => {
                    "Use a lowercase slug such as `hafs-uthmani` (letters, digits, hyphen)."
                }
                Self::MissingTranslator => "Set the translator to the named human translator.",
                Self::IncompleteQuotation => {
                    "Construct the quotation through the corpus repository, which supplies them."
                }
                Self::InvalidReference { .. } => {
                    "Rewrite it in the documented form (e.g. `quran:2:255` or `quran:juz:30`)."
                }
            }
            .to_string(),
        )
    }

    fn next_command(&self) -> Option<String> {
        match self {
            Self::InvalidReference { input, .. } => Some(format!("qai quran resolve \"{input}\"")),
            _ => None,
        }
    }

    fn is_retryable(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn all_errors() -> Vec<QuranError> {
        vec![
            QuranError::InvalidSurahNumber { number: 0 },
            QuranError::InvalidAyahNumber,
            QuranError::InvalidTokenPosition,
            QuranError::EmptyField { field: "text" },
            QuranError::InvalidSlug { slug: String::new() },
            QuranError::MissingTranslator,
            QuranError::IncompleteQuotation,
            QuranError::InvalidReference {
                code: codes::UNEXPECTED_INPUT,
                input: "2:".into(),
                detail: "trailing separator".into(),
            },
        ]
    }

    #[test]
    fn codes_are_unique_and_render_canonically() {
        let rendered: Vec<String> = all_errors().iter().map(|e| e.code().to_string()).collect();
        let unique: HashSet<_> = rendered.iter().collect();
        assert_eq!(rendered.len(), unique.len(), "QAI-QUR codes must be unique");
        assert!(rendered.iter().all(|c| c.starts_with("QAI-QUR-")));
    }

    #[test]
    fn every_error_has_summary_and_remedy() {
        for err in all_errors() {
            assert!(!err.summary().is_empty());
            assert!(err.remedy().is_some());
            assert!(!err.is_retryable());
        }
    }

    #[test]
    fn render_json_escapes_and_is_well_formed() {
        let err = QuranError::EmptyField { field: "text" };
        let json = err.render_json();
        assert!(json.contains("\"code\":\"QAI-QUR-0009\""));
        assert!(json.contains("\"summary\""));
        assert!(json.contains("\"retryable\":false"));
        // The hand-rolled JSON parses (validated via the test-only serde_json dev-dep).
        let _: serde_json::Value = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn trigger_codes_render_as_documented() {
        assert_eq!(codes::CANONICAL_AYAH_IMMUTABLE.to_string(), "QAI-QUR-0001");
        assert_eq!(codes::CANONICAL_AYAH_NO_DELETE.to_string(), "QAI-QUR-0003");
        assert_eq!(codes::CANONICAL_TOKEN_IMMUTABLE.to_string(), "QAI-QUR-0004");
    }
}
