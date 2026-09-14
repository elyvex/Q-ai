//! `tests/error_codes.rs` — error codes are unique and well-formed (AC-P0-17).
//!
//! Every registered code must match `QAI-<NS>-<nnnn>` and no two codes may
//! collide across namespaces.

use std::collections::HashSet;

fn is_well_formed(code: &str) -> bool {
    let Some(rest) = code.strip_prefix("QAI-") else {
        return false;
    };
    let mut parts = rest.splitn(2, '-');
    let (Some(ns), Some(num)) = (parts.next(), parts.next()) else {
        return false;
    };
    (2..=4).contains(&ns.len())
        && ns.chars().all(|c| c.is_ascii_uppercase())
        && num.len() == 4
        && num.chars().all(|c| c.is_ascii_digit())
}

#[test]
fn storage_error_codes_are_unique_and_well_formed() {
    use storage::error::StorageError;
    let codes = [
        StorageError::Conflict.code(),
        StorageError::NotFound { urn: "x".into() }.code(),
        StorageError::ImmutableSourceVersion.code(),
        StorageError::ConstraintViolation { message: "x".into() }.code(),
        StorageError::StorageBusy.code(),
        StorageError::MigrationRequired { at_schema: 1, required: 2 }.code(),
        StorageError::MigrationChecksumMismatch { version: 1 }.code(),
        StorageError::IdempotencyKeyReplay.code(),
        StorageError::StorageUnavailable.code(),
    ];
    assert_unique_and_formed(&codes);
}

#[test]
fn domain_error_codes_are_unique_and_well_formed() {
    use domain::codes;
    let codes = [
        codes::INVALID_TRUST_TRANSITION.to_string(),
        codes::DATA_LAYER_MISMATCH.to_string(),
        codes::MISSING_DIAGNOSTIC_ID.to_string(),
        codes::INVALID_DIAGNOSTIC_LEVEL.to_string(),
        codes::INVALID_DIAGNOSTIC_CODE_FORMAT.to_string(),
    ];
    assert_unique_and_formed(&codes);
}

#[test]
fn no_code_collides_across_namespaces() {
    use domain::codes;
    use storage::error::StorageError;

    let storage_codes = [
        StorageError::Conflict.code(),
        StorageError::NotFound { urn: "x".into() }.code(),
        StorageError::ImmutableSourceVersion.code(),
        StorageError::ConstraintViolation { message: "x".into() }.code(),
        StorageError::StorageBusy.code(),
        StorageError::MigrationRequired { at_schema: 1, required: 2 }.code(),
        StorageError::MigrationChecksumMismatch { version: 1 }.code(),
        StorageError::IdempotencyKeyReplay.code(),
        StorageError::StorageUnavailable.code(),
    ];
    let domain_codes = [
        codes::INVALID_TRUST_TRANSITION.to_string(),
        codes::DATA_LAYER_MISMATCH.to_string(),
        codes::MISSING_DIAGNOSTIC_ID.to_string(),
        codes::INVALID_DIAGNOSTIC_LEVEL.to_string(),
        codes::INVALID_DIAGNOSTIC_CODE_FORMAT.to_string(),
    ];

    let all: Vec<String> =
        storage_codes.iter().map(|s| s.to_string()).chain(domain_codes).collect();
    assert_unique_and_formed(&all);
}

fn assert_unique_and_formed<S: AsRef<str>>(codes: &[S]) {
    let mut seen = HashSet::new();
    for code in codes {
        let code = code.as_ref();
        assert!(is_well_formed(code), "malformed error code: {code}");
        assert!(seen.insert(code.to_string()), "duplicate error code: {code}");
    }
}
