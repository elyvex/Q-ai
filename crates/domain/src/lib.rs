//! Phase 0 placeholder — domain.
//!
//! # domain
//!
//! The Q-ai core domain model: pure, dependency-light types. **No I/O, no async
//! runtime, no external provider dependency** (PRD §33). These types are the
//! vocabulary every other crate (storage, provenance, audit, jobs, sources, config)
//! operates on.
//!
//! ## Contents
//!
//! - [`ids`](module) — Uuid-backed newtype IDs for every aggregate/entity in the system
//! - [`primitives`](module) — `SemVer`, `Timestamp`, `Language`, `Confidence`
//!
//! ## Constraint (enforced by `cargo xtask arch-check`)
//!
//! `domain` may depend only on `(serde, thiserror, time, uuid)` (+ dev/test crates).
//! It must never pull in `sqlx`, `tokio`, or any concrete provider.

pub mod diagnostic;
pub mod hashing;
pub mod ids;
pub mod licensing;
pub mod primitives;
pub mod provenance;
pub mod security;
pub mod types;

pub use diagnostic::{
    Diagnostic, DiagnosticCategory, DiagnosticCode, DiagnosticId, DiagnosticSeverity, codes,
};
pub use hashing::{ContentHash, HashAlgorithm, HashingError, canonical_json_bytes};
pub use ids::{
    ApprovalId, AuditEventId, DocumentId, EditionId, JobId, PrincipalId, ProvenanceId, RunId,
    SourceId, SourceVersionId, WorkspaceId,
};
pub use licensing::{LicenseRecord, LicenseStatus};
pub use primitives::{Confidence, Language, SemVer, Timestamp};
pub use provenance::{DerivationVersions, SubjectRef};
pub use types::{DataLayer, SideEffectClass, TrustLevel, VerificationStatus};
