//! Phase 3 (D-07, G-04) — morphology dataset license-evidence validation.
//!
//! This module is **pure**: it parses and validates license evidence and never
//! performs I/O. `activate_morphology` consults it as gate 6 so a dataset can
//! never reach `state = 'active'` without permissive evidence plus the
//! mandatory capture fields from `licenses/README.md` (`source_url`,
//! `capture_date`, `capturer`) and `redistribution_allowed: true`.
//!
//! Nothing here invents a status, SPDX id, or attribution string: the evidence
//! comes verbatim from the operator's capture, and a missing field is a typed
//! refusal (never a default-true).

use serde::{Deserialize, Serialize};

/// Dataset license statuses recognized as open/permissive evidence.
///
/// Mirrors the permissive half of the domain `LicenseStatus` vocabulary
/// (`PublicDomain` ≤ `OpenLicense` ≤ `PermissionGranted` ≤ `UserOwned`).
/// `MetadataOnly`, `Unknown`, `Restricted`, and the morphology-import
/// placeholders below are **not** bundling evidence.
pub const PERMISSIVE_LICENSE_STATUSES: [&str; 4] =
    ["PublicDomain", "OpenLicense", "PermissionGranted", "UserOwned"];

/// Statuses that can never authorize activation (D-07 fail-closed set).
pub const REJECTED_LICENSE_STATUSES: [&str; 4] =
    ["Unspecified", "metadata_only", "pending_license_review", "Unknown"];

/// TRUE when `status` is an explicitly permissive, recognized license status.
#[must_use]
pub fn is_permissive_status(status: &str) -> bool {
    PERMISSIVE_LICENSE_STATUSES.contains(&status)
}

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

/// Read a mandatory non-empty string field, recording the name when absent.
fn required_str(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    missing: &mut Vec<String>,
) -> String {
    match object.get(key).and_then(serde_json::Value::as_str).map(str::trim) {
        Some(value) if !value.is_empty() => value.to_string(),
        _ => {
            missing.push(key.to_string());
            String::new()
        }
    }
}

impl LicenseEvidence {
    /// Parse `license_status` + `license_json` into typed evidence.
    ///
    /// The capture fields (`source_url`, `capture_date`, `capturer`) and
    /// `redistribution_allowed: true` are mandatory: any absent/empty field is
    /// a typed [`LicenseEvidenceError::MissingFields`] naming every missing
    /// field, never a default-true. A `false` redistribution flag is a typed
    /// [`LicenseEvidenceError::RedistributionForbidden`]. Whether the *status*
    /// is permissive is decided by [`Self::require_activation_allowed`].
    pub fn from_status_and_json(
        status: &str,
        license_json: &str,
    ) -> Result<Self, LicenseEvidenceError> {
        let trimmed = license_json.trim();
        let value: serde_json::Value = if trimmed.is_empty() {
            serde_json::Value::Object(serde_json::Map::new())
        } else {
            serde_json::from_str(trimmed)
                .map_err(|err| LicenseEvidenceError::Malformed { detail: err.to_string() })?
        };
        let object = value.as_object().ok_or_else(|| LicenseEvidenceError::Malformed {
            detail: "expected a JSON object of capture fields".to_string(),
        })?;
        let mut missing: Vec<String> = Vec::new();
        let source_url = required_str(object, "source_url", &mut missing);
        let capture_date = required_str(object, "capture_date", &mut missing);
        let capturer = required_str(object, "capturer", &mut missing);
        let redistribution =
            object.get("redistribution_allowed").and_then(serde_json::Value::as_bool);
        if redistribution.is_none() {
            missing.push("redistribution_allowed".to_string());
        }
        if !missing.is_empty() {
            return Err(LicenseEvidenceError::MissingFields { fields: missing.join(", ") });
        }
        if redistribution != Some(true) {
            return Err(LicenseEvidenceError::RedistributionForbidden);
        }
        Ok(Self {
            status: status.to_string(),
            source_url,
            capture_date,
            capturer,
            redistribution_allowed: true,
        })
    }

    /// The status the operator's evidence declares.
    #[must_use]
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Whether this evidence authorizes activation.
    #[must_use]
    pub fn is_activation_allowed(&self) -> bool {
        is_permissive_status(&self.status) && self.redistribution_allowed
    }

    /// Reject activation, naming why, when the evidence is not permissive.
    pub fn require_activation_allowed(&self) -> Result<(), LicenseEvidenceError> {
        if !is_permissive_status(&self.status) {
            return Err(LicenseEvidenceError::NonPermissiveStatus { status: self.status.clone() });
        }
        if !self.redistribution_allowed {
            return Err(LicenseEvidenceError::RedistributionForbidden);
        }
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
                for required in ["source_url", "capture_date", "capturer", "redistribution_allowed"]
                {
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
