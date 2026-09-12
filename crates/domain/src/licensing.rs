//! Phase 0 — licensing.
//!
//! # domain::licensing
//!
//! License records for tracking source and derivation permissions.

use serde::{Deserialize, Serialize};

/// The status of a license, ordered from most to least permissive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LicenseStatus {
    /// No known copyright restrictions.
    PublicDomain,
    /// Licensed under an open license (e.g., MIT, Apache-2.0, CC-BY).
    OpenLicense,
    /// Permission granted by the rights holder for this specific use.
    PermissionGranted,
    /// The user owns the rights (e.g., self-authored work).
    UserOwned,
    /// Only metadata is available; the content itself is not accessible.
    MetadataOnly,
    /// License status is unknown (e.g., unexamined internet source).
    Unknown,
    /// Use is restricted by license or law.
    Restricted,
}

/// A record of licensing information for a source or derived work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenseRecord {
    pub status: LicenseStatus,
    pub spdx_id: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
    pub attribution_required: bool,
    pub redistribution_allowed: bool,
    pub export_allowed: bool,
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn license_status_order() {
        assert!(LicenseStatus::PublicDomain < LicenseStatus::OpenLicense);
        assert!(LicenseStatus::OpenLicense < LicenseStatus::PermissionGranted);
        assert!(LicenseStatus::PermissionGranted < LicenseStatus::UserOwned);
        assert!(LicenseStatus::UserOwned < LicenseStatus::MetadataOnly);
        assert!(LicenseStatus::MetadataOnly < LicenseStatus::Unknown);
        assert!(LicenseStatus::Unknown < LicenseStatus::Restricted);
    }

    #[test]
    fn license_record_defaults() {
        let record = LicenseRecord {
            status: LicenseStatus::Unknown,
            spdx_id: None,
            name: None,
            url: None,
            attribution_required: false,
            redistribution_allowed: false,
            export_allowed: false,
            notes: None,
        };
        assert_eq!(record.status, LicenseStatus::Unknown);
        assert!(!record.attribution_required);
        assert!(!record.redistribution_allowed);
        assert!(!record.export_allowed);
    }
}
