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

pub mod ids;
pub mod primitives;
pub mod types;

pub use ids::{
    ApprovalId, AuditEventId, DocumentId, EditionId, JobId, PrincipalId, ProvenanceId, RunId,
    SourceId, SourceVersionId, WorkspaceId,
};
pub use primitives::{Confidence, Language, SemVer, Timestamp};
pub use types::{DataLayer, SideEffectClass, TrustLevel, VerificationStatus};
