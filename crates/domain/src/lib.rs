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
pub mod generation;
pub mod hashing;
pub mod ids;
pub mod licensing;
pub mod primitives;
pub mod provenance;
pub mod redaction;
pub mod security;
pub mod security_archive;
pub mod security_input;
pub mod security_net;
pub mod security_sanitize;
pub mod types;

pub use diagnostic::{
    Diagnostic, DiagnosticCategory, DiagnosticCode, DiagnosticId, DiagnosticSeverity, codes,
};
pub use generation::{
    CorpusGeneration, CorpusScope, OutboxEvent, OutboxOperation, OutboxState, PropagationState,
    Tombstone, TombstoneReason,
};
pub use hashing::{ContentHash, HashAlgorithm, HashingError, canonical_json_bytes};
pub use ids::{
    ApprovalId, AuditEventId, CorpusGenerationId, DependencySnapshotId, DocumentId, EditionId,
    JobId, OutboxEventId, PrincipalId, ProvenanceId, RunId, SourceId, SourceVersionId, TombstoneId,
    WorkspaceId,
};
pub use licensing::{LicenseRecord, LicenseStatus};
pub use primitives::{Confidence, Language, SemVer, Timestamp};
pub use provenance::{DerivationVersions, SubjectRef};
pub use types::{DataLayer, SideEffectClass, TrustLevel, VerificationStatus};
