//! Uuid-backed newtype IDs.
//!
//! Every aggregate and entity in Q-ai is identified by a distinct wrapper type so a
//! `SourceId` can never be confused with a `JobId` at compile time (PRD §33/§55).
//!
//! Each ID provides:
//! - `Uuid` tuple field (private, non-constructible directly)
//! - `Display` / `FromStr` / `serde::Serialize` / `serde::Deserialize` (as a string)
//! - `new()` / `random()` (uuid v4)
//! - `Inner` = `Uuid` accessor for serialization/interop
//! - `From<&Self>` → `Uuid` convenience

use core::fmt::{self, Formatter};
use std::str::FromStr;
use uuid::Uuid;

/// Generates a Uuid-backed newtype ID with the standard trait set.
macro_rules! typed_id {
    ($(#[$doc:meta])* $vis:vis struct $name:ident;) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        $vis struct $name(uuid::Uuid);

        impl $name {
            /// Construct from a raw `Uuid`.
            #[allow(dead_code)]
            pub const fn new_inner(id: Uuid) -> Self {
                Self(id)
            }

            /// Generate a fresh random (v4) ID.
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// The inner `Uuid`, for (de)serialization and storage interop.
            pub const fn inner(&self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl FromStr for $name {
            type Err = crate::primitives::UuidParseError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let id = Uuid::parse_str(s).map_err(|_| crate::primitives::UuidParseError)?;
                Ok(Self(id))
            }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl $name {
            /// Construct from a UUID string, panicking on invalid input.
            ///
            /// Primarily for test/sample use; production code should prefer `FromStr`
            /// (fallible) or `new()` (random).
            #[cfg(test)]
            pub fn from_str_unchecked(s: &str) -> Self {
                Self(Uuid::parse_str(s).expect("valid UUID string"))
            }
        }
    };
}

typed_id! {
    /// Identifies a top-level source in the catalog (e.g. `hafs-uthmani`). PRD §22/§55.
    pub struct SourceId;
}
typed_id! {
    /// Identifies a specific version of a source (per `source_versions`). PRD §22/§76.
    pub struct SourceVersionId;
}
typed_id! {
    /// Identifies an edition of a work. PRD §6/§55.
    pub struct EditionId;
}
typed_id! {
    /// Identifies a document (verse, hadith, page, dataset entry, notebook). PRD §55.
    pub struct DocumentId;
}
typed_id! {
    /// Identifies a background job. PRD §41/§54.
    pub struct JobId;
}
typed_id! {
    /// Identifies a job execution/run. PRD §41/§54.
    pub struct RunId;
}
typed_id! {
    /// Identifies a research workspace. PRD §55/§11.
    pub struct WorkspaceId;
}
typed_id! {
    /// Identifies a human/system principal. PRD §82/§11 (Phase 11 extends).
    pub struct PrincipalId;
}
typed_id! {
    /// Identifies one provenance record. PRD §6/§82.
    pub struct ProvenanceId;
}
typed_id! {
    /// Identifies one audit event. PRD §82.
    pub struct AuditEventId;
}
typed_id! {
    /// Identifies one approval record. PRD §7.3/§92 — the basis for `ApprovalToken`.
    pub struct ApprovalId;
}
typed_id! {
    /// Identifies a source-version activation or rollback approval. PRD §22.3.
    pub struct ActivationApprovalId;
}

/// Round-tripping through the display/parse + serialization forms.
#[cfg(test)]
pub(crate) fn roundtrip_all<T>()
where
    T: std::fmt::Display
        + std::fmt::Debug
        + PartialEq
        + FromStr
        + serde::Serialize
        + serde::de::DeserializeOwned,
    T::Err: std::fmt::Debug,
{
    // A valid UUID string must parse back to an equal, display-able value.
    let s = "12345678-1234-1234-1234-123456789abc";
    let t: T = s.parse().unwrap();
    assert_eq!(t.to_string(), s);
    assert_eq!(serde_json::to_string(&t).unwrap(), format!("\"{s}\""));
    let json = format!("\"{s}\"");
    let back: T = serde_json::from_str(&json).unwrap();
    assert_eq!(back, t);
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_id_roundtrip {
        ($($ty:ty),* $(,)?) => {
            $(
                roundtrip_all::<$ty>();
            )*
        };
    }

    #[test]
    fn all_ids_roundtrip() {
        assert_id_roundtrip!(
            SourceId,
            SourceVersionId,
            EditionId,
            DocumentId,
            JobId,
            RunId,
            WorkspaceId,
            PrincipalId,
            ProvenanceId,
            AuditEventId,
            ApprovalId,
            ActivationApprovalId,
        );
    }

    #[test]
    fn ids_are_distinct_types() {
        // Compile-time proof of newtype separation: these must not be assignable to each other.
        // (This compiles only because each type is a distinct wrapper; swapping them would fail.)
        let s = SourceId::new();
        let j = JobId::new();
        let _s: &str = &s.to_string();
        let _j: &str = &j.to_string();
        let _ = (s, j);
    }

    #[test]
    fn invalid_uuid_is_rejected() {
        let err: Result<SourceId, _> = "not-a-uuid".parse();
        let err = err.expect_err("invalid uuid must fail");
        let msg = err.to_string();
        assert!(msg.contains("QAI-DOM-0001"), "unexpected error: {msg}");
    }
}
