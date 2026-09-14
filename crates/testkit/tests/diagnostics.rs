//! `tests/diagnostics.rs` — every error type implements `Diagnostic` with a
//! unique, well-formed code and stable human/JSON renderings (AC-P0-17).

use std::collections::HashSet;

use audit::AuditError;
use jobs::JobError;
use provenance::ProvenanceError;
use sources::SourceError;
use storage::error::{Diagnostic, StorageError};

/// Exercise the trait surface and return the rendered human form.
fn assert_diagnostic<D: Diagnostic>(err: &D) -> String {
    let code = err.code().to_string();
    assert!(code.starts_with("QAI-"), "code must be QAI-*: {code}");
    assert!(!err.summary().is_empty(), "summary must be non-empty");

    let human = err.render_human();
    assert!(human.contains(&code), "human rendering must include the code");

    let json = err.render_json();
    let value: serde_json::Value = serde_json::from_str(&json).expect("render_json is valid JSON");
    assert_eq!(value["code"], code);
    assert!(value["summary"].as_str().is_some());

    // Renderings are deterministic.
    assert_eq!(human, err.render_human());
    assert_eq!(json, err.render_json());
    human
}

#[test]
fn every_error_type_implements_diagnostic() {
    assert_diagnostic(&StorageError::Conflict);
    assert_diagnostic(&JobError::Cancelled { id: "job-1".into() });
    assert_diagnostic(&AuditError::AppendOnlyViolation);
    assert_diagnostic(&ProvenanceError::MissingApprovalToken);
    assert_diagnostic(&SourceError::SignatureVerificationFailed("x".into()));
}

#[test]
fn diagnostic_codes_are_unique_across_crates() {
    let codes: Vec<String> = vec![
        Diagnostic::code(&StorageError::Conflict).to_string(),
        Diagnostic::code(&StorageError::StorageUnavailable).to_string(),
        Diagnostic::code(&JobError::Cancelled { id: "x".into() }).to_string(),
        Diagnostic::code(&JobError::MaxAttemptsExceeded { id: "x".into(), max: 5 }).to_string(),
        Diagnostic::code(&AuditError::AppendOnlyViolation).to_string(),
        Diagnostic::code(&AuditError::HashMismatch { sequence: 1 }).to_string(),
        Diagnostic::code(&ProvenanceError::MissingApprovalToken).to_string(),
        Diagnostic::code(&ProvenanceError::ImmutableCanonical).to_string(),
        Diagnostic::code(&SourceError::SignatureVerificationFailed("x".into())).to_string(),
        Diagnostic::code(&SourceError::ApprovalPreconditionNotMet("x".into())).to_string(),
    ];
    let unique: HashSet<&String> = codes.iter().collect();
    assert_eq!(codes.len(), unique.len(), "diagnostic codes must be unique");
}

#[test]
fn removable_errors_are_marked_retryable() {
    assert!(Diagnostic::is_retryable(&JobError::Storage("io".into())));
    assert!(Diagnostic::is_retryable(&SourceError::Storage(
        StorageError::StorageUnavailable
    )));
    assert!(!Diagnostic::is_retryable(&ProvenanceError::MissingApprovalToken));
}
