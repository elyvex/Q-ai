//! Bridge from storage audit rows to the audit crate's hash-chained writer.
//!
//! # application::audit_bridge
//!
//! The audit crate owns the hash-chain recipe; storage owns the tables. This
//! bridge maps between the two so application services can append verifiable
//! audit events inside their own units of work. It is the first real writer
//! path — Phase 0 defined the chain but never wired it into the app.
//!
//! Conventions (documented once, here):
//! - hashes render as `sha256:<hex>` / `blake3:<hex>` (D0.6 storage form);
//! - actions and outcomes use their `snake_case` JSON spellings;
//! - a stored `agent` actor (which the audit enum cannot represent) reads back
//!   as `System` with the stored id as its name.

use audit::{
    Actor, AuditAction, AuditError, AuditEvent, AuditOutcome, HashChainWriter,
};
use audit::AuditRepository as _;
use domain::{AuditEventId, ContentHash, HashAlgorithm, PrincipalId, SubjectRef, Timestamp};
use storage::repository::AuditEvent as AuditRow;

fn hash_str(hash: &ContentHash) -> String {
    let algorithm = match hash.algorithm {
        HashAlgorithm::Sha256 => "sha256",
        HashAlgorithm::Blake3 => "blake3",
    };
    format!("{algorithm}:{}", hash.hex)
}

fn parse_hash(raw: &str) -> Result<ContentHash, AuditError> {
    let (algorithm, hex) =
        raw.split_once(':').ok_or_else(|| storage_error(format!("bad hash `{raw}`")))?;
    let algorithm = match algorithm {
        "sha256" => HashAlgorithm::Sha256,
        "blake3" => HashAlgorithm::Blake3,
        _ => return Err(storage_error(format!("unknown hash algorithm in `{raw}`"))),
    };
    ContentHash::try_new(algorithm, hex.to_string())
        .map_err(|err| storage_error(format!("bad hash `{raw}`: {err}")))
}

fn storage_error(detail: String) -> AuditError {
    AuditError::Storage(storage::StorageError::ConstraintViolation { message: detail })
}

fn action_str(action: &AuditAction) -> String {
    serde_json::to_value(action)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".to_string())
}

fn parse_action(raw: &str) -> Result<AuditAction, AuditError> {
    serde_json::from_value::<AuditAction>(serde_json::Value::String(raw.to_string()))
        .map_err(|_| storage_error(format!("unknown audit action `{raw}`")))
}

fn outcome_str(outcome: &AuditOutcome) -> String {
    serde_json::to_value(outcome)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| "failed".to_string())
}

fn parse_outcome(raw: &str) -> Result<AuditOutcome, AuditError> {
    serde_json::from_value::<AuditOutcome>(serde_json::Value::String(raw.to_string()))
        .map_err(|_| storage_error(format!("unknown audit outcome `{raw}`")))
}

fn actor_parts(actor: &Actor) -> (String, Option<String>) {
    match actor {
        Actor::Principal { principal_id } => ("principal".to_string(), Some(principal_id.to_string())),
        Actor::System { name } => ("system".to_string(), Some(name.clone())),
        Actor::Job { job_id } => ("job".to_string(), Some(job_id.clone())),
    }
}

fn parse_actor(kind: &str, id: Option<&str>) -> Result<Actor, AuditError> {
    match kind {
        "principal" => {
            let id = id.ok_or_else(|| storage_error("principal actor needs an id".to_string()))?;
            Ok(Actor::Principal {
                principal_id: id
                    .parse::<PrincipalId>()
                    .map_err(|_| storage_error(format!("bad principal id `{id}`")))?,
            })
        }
        // `agent` has no audit-enum spelling; it reads back as a named system.
        "system" | "agent" => {
            Ok(Actor::System { name: id.unwrap_or("system").to_string() })
        }
        "job" => Ok(Actor::Job { job_id: id.unwrap_or_default().to_string() }),
        other => Err(storage_error(format!("unknown actor kind `{other}`"))),
    }
}

fn parse_timestamp(raw: &str) -> Result<Timestamp, AuditError> {
    raw.parse::<Timestamp>()
        .map_err(|_| storage_error(format!("bad timestamp `{raw}`")))
}

fn parse_json(raw: &Option<String>) -> Result<Option<serde_json::Value>, AuditError> {
    raw.as_deref()
        .map(|text| {
            serde_json::from_str(text)
                .map_err(|_| storage_error("audit payload is not JSON".to_string()))
        })
        .transpose()
}

/// Adapts a storage audit repository to the audit crate's trait.
pub struct StorageAuditBridge<'a> {
    inner: &'a mut dyn storage::repository::AuditRepository,
}

impl<'a> StorageAuditBridge<'a> {
    /// Wrap a storage audit repository.
    pub fn new(inner: &'a mut dyn storage::repository::AuditRepository) -> Self {
        Self { inner }
    }

    fn to_row(event: &AuditEvent) -> Result<AuditRow, AuditError> {
        let (actor_kind, actor_id) = actor_parts(&event.actor);
        Ok(AuditRow {
            id: event.id.to_string(),
            sequence: event.sequence,
            occurred_at: event.occurred_at.to_string(),
            actor_kind,
            actor_id,
            action: action_str(&event.action),
            subject_urn: event.subject.0.clone(),
            outcome: outcome_str(&event.outcome),
            reason: event.reason.clone(),
            before_json: event.before.as_ref().map(ToString::to_string),
            after_json: event.after.as_ref().map(ToString::to_string),
            request_id: event.request_id.clone(),
            prev_chain_hash: hash_str(&event.prev_chain_hash),
            chain_hash: hash_str(&event.chain_hash),
        })
    }

    fn from_row(row: &AuditRow) -> Result<AuditEvent, AuditError> {
        Ok(AuditEvent {
            id: row
                .id
                .parse::<AuditEventId>()
                .map_err(|_| storage_error(format!("bad audit id `{}`", row.id)))?,
            sequence: row.sequence,
            occurred_at: parse_timestamp(&row.occurred_at)?,
            actor: parse_actor(&row.actor_kind, row.actor_id.as_deref())?,
            action: parse_action(&row.action)?,
            subject: SubjectRef(row.subject_urn.clone()),
            outcome: parse_outcome(&row.outcome)?,
            reason: row.reason.clone(),
            before: parse_json(&row.before_json)?,
            after: parse_json(&row.after_json)?,
            request_id: row.request_id.clone(),
            prev_chain_hash: parse_hash(&row.prev_chain_hash)?,
            chain_hash: parse_hash(&row.chain_hash)?,
        })
    }
}

#[async_trait::async_trait]
impl audit::AuditRepository for StorageAuditBridge<'_> {
    async fn append(&mut self, event: AuditEvent) -> Result<(), AuditError> {
        let row = Self::to_row(&event)?;
        self.inner.append(row).await?;
        Ok(())
    }

    async fn list_by_subject(
        &self,
        subject: &SubjectRef,
    ) -> Result<Vec<AuditEvent>, AuditError> {
        let rows = self.inner.list_by_subject(&subject.0).await?;
        rows.iter().map(Self::from_row).collect()
    }

    async fn list_by_sequence(
        &self,
        from: u64,
        to: Option<u64>,
    ) -> Result<Vec<AuditEvent>, AuditError> {
        let rows = self.inner.list_by_sequence(from, to).await?;
        rows.iter().map(Self::from_row).collect()
    }

    async fn verify_chain(&self) -> Result<audit::ChainVerificationResult, AuditError> {
        let storage_result = self.inner.verify_chain().await?;
        Ok(audit::ChainVerificationResult {
            valid: storage_result.valid,
            expected_next_sequence: storage_result.expected_next_sequence,
            expected_next_hash: parse_hash(&storage_result.expected_next_hash)?,
            gaps: storage_result.gaps,
            tampered_events: Vec::new(),
        })
    }

    async fn latest_sequence(&self) -> Result<u64, AuditError> {
        self.inner.latest_sequence().await.map_err(AuditError::from)
    }
}

/// Append one hash-chained audit event inside a unit of work.
///
/// Mirrors [`HashChainWriter::append_event`] exactly (sequence, timestamp, and
/// linkage) so events written here verify under `AuditVerifier`. Same-tx
/// callers keep the mutation and its audit record atomic.
pub async fn append_audit_event(
    uow: &mut dyn storage::UnitOfWork,
    mut event: AuditEvent,
) -> Result<AuditEvent, AuditError> {
    let (sequence, prev_hash) = {
        let bridge = StorageAuditBridge::new(uow.audit());
        let sequence = bridge.latest_sequence().await? + 1;
        let prev_hash = if sequence == 1 {
            genesis_hash()
        } else {
            bridge
                .list_by_sequence(sequence - 1, Some(sequence - 1))
                .await?
                .into_iter()
                .find(|candidate| candidate.sequence == sequence - 1)
                .map(|candidate| candidate.chain_hash)
                .unwrap_or_else(genesis_hash)
        };
        (sequence, prev_hash)
    };
    event.sequence = sequence;
    event.occurred_at = Timestamp::now();
    event.prev_chain_hash = prev_hash.clone();
    event.chain_hash = HashChainWriter::compute_chain_hash(&prev_hash, &event);
    let row = StorageAuditBridge::to_row(&event)?;
    uow.audit().append(row).await?;
    Ok(event)
}

fn genesis_hash() -> ContentHash {
    ContentHash { algorithm: HashAlgorithm::Sha256, hex: "00".repeat(32) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(id: &str) -> AuditEvent {
        AuditEvent {
            id: id.parse().unwrap(),
            sequence: 0,
            occurred_at: Timestamp::from_ymd_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            actor: Actor::Job { job_id: "job-1".to_string() },
            action: AuditAction::SourceStaged,
            subject: SubjectRef("quran-edition:test@0.1.0".to_string()),
            outcome: AuditOutcome::Allowed,
            reason: None,
            before: None,
            after: None,
            request_id: None,
            prev_chain_hash: ContentHash {
                algorithm: HashAlgorithm::Sha256,
                hex: "00".repeat(32),
            },
            chain_hash: ContentHash {
                algorithm: HashAlgorithm::Sha256,
                hex: "ff".repeat(32),
            },
        }
    }

    #[test]
    fn rows_roundtrip_through_the_bridge() {
        let row = StorageAuditBridge::to_row(&event("00000000-0000-0000-0000-000000000001"))
            .unwrap();
        assert_eq!(row.actor_kind, "job");
        assert_eq!(row.action, "source_staged");
        assert_eq!(row.outcome, "allowed");
        assert!(row.chain_hash.starts_with("sha256:"));
        let back = StorageAuditBridge::from_row(&row).unwrap();
        assert_eq!(back.action, AuditAction::SourceStaged);
        assert_eq!(back.actor, Actor::Job { job_id: "job-1".to_string() });
    }

    #[test]
    fn agent_actors_read_back_as_systems() {
        let actor = parse_actor("agent", Some("worker-2")).unwrap();
        assert_eq!(actor, Actor::System { name: "worker-2".to_string() });
        assert!(parse_actor("principal", Some("not-a-uuid")).is_err());
        assert!(parse_hash("sha256:gg").is_err());
    }
}
