//! Phase 0 — domain types.
//!
//! # domain::types
//!
//! Core domain types for Q-ai.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Represents the layer of the system or data that a component operates within.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataLayer {
    Canonical,
    Publisher,
    Scholarly,
    Computational,
    User,
    Agent,
}

/// Represents the level of trust assigned to a source or derived data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TrustLevel {
    CanonicalVerified,
    PublisherVerified,
    ScholarReviewed,
    CommunityReviewed,
    ImportedUnverified,
    MachineGenerated,
    UserProvided,
    Quarantined,
}

/// Represents the verification status of data within the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Unverified,
    Processing,
    Verified,
    Rejected,
}

/// Defines the class of side effects that an operation may trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SideEffectClass {
    None,
    Readonly,
    Mutation,
    CanonicalWrite,
    ExternalAccess,
}

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Invalid trust level transition")]
    InvalidTrustTransition,
    #[error("Data layer mismatch: expected {expected:?}, found {found:?}")]
    DataLayerMismatch { expected: DataLayer, found: DataLayer },
}
