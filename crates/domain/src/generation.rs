//! Cross-store consistency primitives (D0.18, ADR-0702 §§2–4, §9).
//!
//! Phase 0 builds the **relational primitives only** — a monotonic corpus
//! generation, a transactional outbox, and tombstones. Multi-store publish /
//! reconcile / doctor-repair remains Phase 7.
//!
//! These types are pure (serde + domain primitives); persistence lives behind
//! the `storage::repository::OutboxRepository` trait.

use crate::ids::{CorpusGenerationId, OutboxEventId, PrincipalId, TombstoneId};
use crate::primitives::Timestamp;
use serde::{Deserialize, Serialize};

/// A scope over which generation numbers are monotonic (ADR-0702 §2).
///
/// Examples: `"quran:hafs-uthmani"`, `"hadith:al-kafi"`, `"global"`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CorpusScope(pub String);

impl CorpusScope {
    /// The scope as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The global scope.
    pub fn global() -> Self {
        Self("global".to_string())
    }
}

impl From<&str> for CorpusScope {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// A monotonic generation within a [`CorpusScope`] (PRD §76, ADR-0702 §2).
///
/// The `number` never regresses within a scope; allocation happens inside the
/// same write transaction as the authoritative change it stamps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorpusGeneration {
    pub id: CorpusGenerationId,
    pub scope: CorpusScope,
    pub number: u64,
    pub reason: String,
    pub created_at: Timestamp,
}

/// The closed set of outbox operations (ADR-0702 §3).
///
/// Extended per phase; Phase 0 covers source and provenance projection-relevant
/// changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutboxOperation {
    SourceActivated,
    SourceDeactivated,
    SourceRolledBack,
    ProvenanceWritten,
    CanonicalChangeCommitted,
}

impl OutboxOperation {
    /// The stable string form stored in the database.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceActivated => "source_activated",
            Self::SourceDeactivated => "source_deactivated",
            Self::SourceRolledBack => "source_rolled_back",
            Self::ProvenanceWritten => "provenance_written",
            Self::CanonicalChangeCommitted => "canonical_change_committed",
        }
    }
}

/// Lifecycle of an outbox event (ADR-0702 §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutboxState {
    Pending,
    Claimed,
    Dispatched,
    Failed,
}

impl OutboxState {
    /// The stable string form stored in the database.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Claimed => "Claimed",
            Self::Dispatched => "Dispatched",
            Self::Failed => "Failed",
        }
    }
}

/// A durable outbox event, written in the same transaction as the change it
/// announces (ADR-0001 §6, ADR-0702 §3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutboxEvent {
    pub id: OutboxEventId,
    pub scope: CorpusScope,
    pub target_generation: CorpusGenerationId,
    pub operation: OutboxOperation,
    pub subject_urn: String,
    pub idempotency_key: String,
    pub payload: serde_json::Value,
    pub state: OutboxState,
    pub lease_owner: Option<String>,
    pub lease_expires_at: Option<Timestamp>,
    pub attempts: u32,
    pub created_at: Timestamp,
    pub dispatched_at: Option<Timestamp>,
}

/// Why a tombstone was written (ADR-0702 §9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TombstoneReason {
    Deactivated,
    LicenseRevoked,
    UserDeleted,
    Superseded,
}

impl TombstoneReason {
    /// The stable string form stored in the database.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Deactivated => "deactivated",
            Self::LicenseRevoked => "license_revoked",
            Self::UserDeleted => "user_deleted",
            Self::Superseded => "superseded",
        }
    }
}

/// How far a tombstone has propagated to derived stores (ADR-0702 §9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropagationState {
    Pending,
    Propagated,
    PartiallyFailed,
}

impl PropagationState {
    /// The stable string form stored in the database.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Propagated => "Propagated",
            Self::PartiallyFailed => "PartiallyFailed",
        }
    }
}

/// A tombstone marks authoritative state as deleted/deactivated (ADR-0702 §9).
///
/// Current policy blocks retrieval immediately even while physical cleanup is
/// pending; the table exists in Phase 0 so nothing later bolts it on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tombstone {
    pub id: TombstoneId,
    pub subject_urn: String,
    pub reason: TombstoneReason,
    pub effective_at: Timestamp,
    pub created_by: Option<PrincipalId>,
    pub propagation_state: PropagationState,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Timestamp;

    #[test]
    fn scope_and_operation_strings_are_stable() {
        assert_eq!(CorpusScope::from("quran:hafs").as_str(), "quran:hafs");
        assert_eq!(OutboxOperation::SourceActivated.as_str(), "source_activated");
        assert_eq!(OutboxState::Dispatched.as_str(), "Dispatched");
        assert_eq!(TombstoneReason::LicenseRevoked.as_str(), "license_revoked");
        assert_eq!(PropagationState::Pending.as_str(), "Pending");
    }

    #[test]
    fn generation_roundtrips_through_serde() {
        let g = CorpusGeneration {
            id: CorpusGenerationId::new(),
            scope: CorpusScope::global(),
            number: 7,
            reason: "test".into(),
            created_at: Timestamp::now(),
        };
        let json = serde_json::to_string(&g).unwrap();
        let back: CorpusGeneration = serde_json::from_str(&json).unwrap();
        assert_eq!(back, g);
    }

    #[test]
    fn every_enum_variant_has_a_stable_string() {
        for (op, s) in [
            (OutboxOperation::SourceActivated, "source_activated"),
            (OutboxOperation::SourceDeactivated, "source_deactivated"),
            (OutboxOperation::SourceRolledBack, "source_rolled_back"),
            (OutboxOperation::ProvenanceWritten, "provenance_written"),
            (OutboxOperation::CanonicalChangeCommitted, "canonical_change_committed"),
        ] {
            assert_eq!(op.as_str(), s);
        }
        for (st, s) in [
            (OutboxState::Pending, "Pending"),
            (OutboxState::Claimed, "Claimed"),
            (OutboxState::Dispatched, "Dispatched"),
            (OutboxState::Failed, "Failed"),
        ] {
            assert_eq!(st.as_str(), s);
        }
        for (r, s) in [
            (TombstoneReason::Deactivated, "deactivated"),
            (TombstoneReason::LicenseRevoked, "license_revoked"),
            (TombstoneReason::UserDeleted, "user_deleted"),
            (TombstoneReason::Superseded, "superseded"),
        ] {
            assert_eq!(r.as_str(), s);
        }
        assert_eq!(CorpusScope::global().as_str(), "global");
    }

    #[test]
    fn outbox_event_and_tombstone_round_trip() {
        let event = OutboxEvent {
            id: OutboxEventId::new(),
            scope: CorpusScope::from("quran:test"),
            target_generation: CorpusGenerationId::new(),
            operation: OutboxOperation::SourceActivated,
            subject_urn: "urn:qai:source:test".into(),
            idempotency_key: "source_activated:1".into(),
            payload: serde_json::json!({"k": "v"}),
            state: OutboxState::Pending,
            lease_owner: None,
            lease_expires_at: None,
            attempts: 0,
            created_at: Timestamp::now(),
            dispatched_at: None,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(serde_json::from_str::<OutboxEvent>(&json).unwrap(), event);

        let tombstone = Tombstone {
            id: TombstoneId::new(),
            subject_urn: "urn:qai:source:test".into(),
            reason: TombstoneReason::Deactivated,
            effective_at: Timestamp::now(),
            created_by: None,
            propagation_state: PropagationState::Pending,
        };
        let json = serde_json::to_string(&tombstone).unwrap();
        assert_eq!(serde_json::from_str::<Tombstone>(&json).unwrap(), tombstone);
    }
}
