//! Phase 3 (D-07, G-04) — morphology dataset license-evidence validation.
//!
//! This module is **pure**: it parses and validates license evidence and never
//! performs I/O. `activate_morphology` consults it as gate 6 so a dataset can
//! never reach `state = 'active'` without permissive evidence plus the
//! mandatory capture fields from `licenses/README.md` (`source_url`,
//! `capture_date`, `capturer`) and `redistribution_allowed: true`.
//!
//! RED stub: the API surface exists so the behaviour tests can be written
//! first; the bodies are intentionally inert and the GREEN commit replaces
//! them with the real validation.

use serde::{Deserialize, Serialize};

/// A dataset license status recognized as open/permissive evidence.
///
/// Mirrors the permissive half of the domain `LicenseStatus` vocabulary.
pub const PERMISSIVE_LICENSE_STATUSES: [&str; 4] =
    ["PublicDomain", "OpenLicense", "PermissionGranted", "UserOwned"];

/// Statuses that can never authorize activation.
pub const REJECTED_LICENSE_STATUSES: [&str; 4] =
    ["Unspecified", "metadata_only", "pending_license_review", "Unknown"];

/// Errors raised when license evidence is absent, malformed, or non-permissive.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LicenseEvidenceError {
    /// The status is not in the permissive allowlist.
    #[error("license status `{status}` is not permissive evidence")]
    NonPermissiveStatus {
        /// The offending status.
        status: String,
    },
    /// The license JSON did not parse or was not an object.
    #[error("license evidence JSON is not an object: {detail}")]
    Malformed {
        /// Parse detail.
        detail: String,
    },
    /// A mandatory capture field is missing or empty.
    #[error("license evidence is missing mandatory field(s): {fields}")]
    MissingFields {
        /// Comma-separated missing field names.
        fields: String,
    },
    /// The capture explicitly forbids redistribution.
    #[error("license evidence forbids redistribution (redistribution_allowed != true)")]
    RedistributionForbidden,
}

/// Parsed license evidence for one dataset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenseEvidence {
    status: String,
    source_url: String,
    capture_date: String,
    capturer: String,
    redistribution_allowed: bool,
}

impl LicenseEvidence {
    /// Parse `license_status` + `license_json` into typed evidence.
    pub fn from_status_and_json(
        _status: &str,
        _license_json: &str,
    ) -> Result<Self, LicenseEvidenceError> {
        Ok(Self {
            status: String::new(),
            source_url: String::new(),
            capture_date: String::new(),
            capturer: String::new(),
            redistribution_allowed: false,
        })
    }

    /// The status the operator's evidence declares.
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Whether this evidence authorizes activation.
    pub fn is_activation_allowed(&self) -> bool {
        true
    }

    /// Reject activation, naming why, when the evidence is not permissive.
    pub fn require_activation_allowed(&self) -> Result<(), LicenseEvidenceError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capture_json() -> String {
        serde_json::json!({
            "artifact": "synthetic-test-lexicon",
            "source_url": "https://example.invalid/qai-synthetic-test-lexicon",
            "capture_date": "2026-09-28",
            "capturer": "qai-test-fixtures",
            "spdx_id": "CC0-1.0",
            "redistribution_allowed": true,
            "modification_allowed": true,
            "attribution_required": false,
        })
        .to_string()
    }

    #[test]
    fn license_evidence_accepts_captured_evidence() {
        let evidence = LicenseEvidence::from_status_and_json("PublicDomain", &capture_json())
            .expect("well-formed capture parses");
        assert!(evidence.is_activation_allowed(), "captured evidence must activate");
        assert_eq!(evidence.require_activation_allowed(), Ok(()));
        assert_eq!(evidence.status(), "PublicDomain");
    }

    #[test]
    fn license_evidence_rejects_unspecified() {
        let evidence = LicenseEvidence::from_status_and_json("Unspecified", &capture_json())
            .expect("structure parses; the status is the refusal");
        assert!(!evidence.is_activation_allowed(), "Unspecified must never activate");
        assert!(matches!(
            evidence.require_activation_allowed(),
            Err(LicenseEvidenceError::NonPermissiveStatus { .. })
        ));
    }

    #[test]
    fn license_evidence_rejects_missing_capture_fields() {
        let err = LicenseEvidence::from_status_and_json("PublicDomain", "{}")
            .expect_err("an empty capture is not evidence");
        match err {
            LicenseEvidenceError::MissingFields { fields } => {
                for required in ["source_url", "capture_date", "capturer", "redistribution_allowed"] {
                    assert!(fields.contains(required), "{required} named in: {fields}");
                }
            }
            other => panic!("expected MissingFields, got {other:?}"),
        }
    }

    #[test]
    fn license_evidence_rejects_redistribution_false() {
        let json = serde_json::json!({
            "source_url": "https://example.invalid/restricted",
            "capture_date": "2026-09-28",
            "capturer": "qai-test-fixtures",
            "redistribution_allowed": false,
        })
        .to_string();
        let err = LicenseEvidence::from_status_and_json("PublicDomain", &json)
            .expect_err("redistribution forbidden is not evidence");
        assert_eq!(err, LicenseEvidenceError::RedistributionForbidden);
    }
}
