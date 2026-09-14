//! Quran application services: the import job handler plus approval-gated
//! activation and rollback (P1-T25, P1-T28).
//!
//! # application::quran
//!
//! The importer ([`QuranImportHandler`], kind `quran.import`) drives the
//! 13-checkpoint pipeline and leaves the edition `Staged`. It cannot activate.
//! Activation and rollback go through [`activate_edition`] /
//! [`rollback_edition`], which require a granted human approval covering the
//! exact edition URN and record hash-chained audit events. Every Quran
//! mutation therefore has a human decision and an audit trail behind it
//! (AC-P1-03, I5/I7).

use std::sync::Arc;

use async_trait::async_trait;
use audit::{Actor, AuditAction, AuditEvent, AuditOutcome};
use domain::{AuditEventId, PrincipalId, SubjectRef, Timestamp};
use jobs::{JobContext, JobError, JobHandler, JobKind, JobOutcome};
use quran_corpus::error::QuranDiagnostic as _;
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use storage::Database as _;
use storage::error::StorageError;
use storage_sqlite::SqliteDatabase;

use crate::audit_bridge::append_audit_event;

/// Job kind for canonical imports.
pub const QURAN_IMPORT_KIND: &str = "quran.import";
/// Adapter version pinned into idempotency keys.
pub const ADAPTER_VERSION: &str = "1.0.0";
/// Parser version pinned into idempotency keys.
pub const PARSER_VERSION: &str = "1.0.0";

/// Idempotency key for an import: same source + adapter + parser ⇒ one job.
pub fn import_idempotency_key(source_version_id: &str) -> String {
    format!("{source_version_id}:{ADAPTER_VERSION}:{PARSER_VERSION}")
}

fn edition_urn(slug: &str, version: &str) -> String {
    format!("quran-edition:{slug}@{version}")
}

/// The `quran.import` job handler.
pub struct QuranImportHandler {
    db: Arc<SqliteDatabase>,
}

impl QuranImportHandler {
    /// Wrap a database handle.
    pub fn new(db: Arc<SqliteDatabase>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl JobHandler for QuranImportHandler {
    fn kind(&self) -> JobKind {
        QURAN_IMPORT_KIND.into()
    }

    fn payload_schema(&self) -> &'static str {
        r#"{"type":"object","required":["run_id","source_version_id","adapter","manifest_text","invoked_by","license_status","license_json","created_at"]}"#
    }

    fn is_idempotent(&self) -> bool {
        true
    }

    async fn run(
        &self,
        ctx: JobContext,
        payload: serde_json::Value,
    ) -> Result<JobOutcome, JobError> {
        let mut input: ImportInput = serde_json::from_value(payload)
            .map_err(|err| JobError::Storage(format!("bad import payload: {err}")))?;
        if input.job_id.is_none() {
            input.job_id = Some(ctx.job.id.clone());
        }
        let cancel = ctx.cancel_flag();
        let progress = ImportProgress::new();
        let outcome =
            run_import(&*self.db, &input, &ImportOptions::default(), &cancel, progress.clone())
                .await;
        if let Some(last) = progress.checkpoints().last() {
            ctx.checkpoint(last.as_str());
        }
        match outcome {
            Ok(ImportOutcome::Completed(success)) => {
                let urn = edition_urn(&success.edition_slug, &success.edition_version);
                let mut uow =
                    self.db.write().await.map_err(|err| JobError::Storage(err.to_string()))?;
                append_audit_event(
                    &mut *uow,
                    AuditEvent {
                        id: AuditEventId::new(),
                        sequence: 0,
                        occurred_at: Timestamp::from_ymd_hms(2026, 1, 1, 0, 0, 0)
                            .unwrap_or_else(|_| Timestamp::now()),
                        actor: Actor::Job { job_id: ctx.job.id.clone() },
                        action: AuditAction::SourceStaged,
                        subject: SubjectRef(urn),
                        outcome: AuditOutcome::Allowed,
                        reason: None,
                        before: None,
                        after: Some(serde_json::json!({
                            "edition_id": success.edition_id,
                            "validation_report": success.validation_report_id,
                        })),
                        request_id: Some(ctx.job.id.clone()),
                        prev_chain_hash: audit_chain_genesis(),
                        chain_hash: audit_chain_genesis(),
                    },
                )
                .await
                .map_err(|err| JobError::Storage(err.to_string()))?;
                uow.commit().await.map_err(|err| JobError::Storage(err.to_string()))?;
                Ok(JobOutcome {
                    success: true,
                    result: Some(format!(
                        "staged {} validation {}",
                        success.edition_id, success.validation_report_id
                    )),
                })
            }
            Err(quran_corpus::CorpusError::ImportCancelled { .. }) => {
                Err(JobError::Cancelled { id: ctx.job.id.clone() })
            }
            Err(err) => Err(JobError::Storage(format!("{}: {}", err.code(), err.summary()))),
        }
    }
}

fn audit_chain_genesis() -> domain::ContentHash {
    domain::ContentHash { algorithm: domain::HashAlgorithm::Sha256, hex: "00".repeat(32) }
}

/// Errors from the approval-gated Quran services.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ActivationError {
    /// No approval row with that id.
    #[error("approval not found: {id}")]
    ApprovalMissing {
        /// The requested approval id.
        id: String,
    },
    /// The approval was not granted.
    #[error("approval {id} is not granted")]
    ApprovalNotGranted {
        /// The approval id.
        id: String,
    },
    /// The approval covers a different subject.
    #[error("approval {id} covers {actual}, not {expected}")]
    ApprovalSubjectMismatch {
        /// The approval id.
        id: String,
        /// The subject it covers.
        actual: String,
        /// The subject required.
        expected: String,
    },
    /// No staged edition for that slug and version.
    #[error("no staged edition {slug}@{version}")]
    NotStaged {
        /// Edition slug.
        slug: String,
        /// Edition version.
        version: String,
    },
    /// Storage failure.
    #[error("storage failed: {0}")]
    Storage(String),
    /// Audit failure.
    #[error("audit failed: {0}")]
    Audit(String),
}

impl ActivationError {
    fn storage(error: StorageError) -> Self {
        Self::Storage(error.to_string())
    }
}

impl storage::error::Diagnostic for ActivationError {
    fn code(&self) -> storage::error::DiagnosticCode {
        let number = match self {
            Self::ApprovalMissing { .. } => 300,
            Self::ApprovalNotGranted { .. } => 301,
            Self::ApprovalSubjectMismatch { .. } => 302,
            Self::NotStaged { .. } => 303,
            Self::Storage(_) => 304,
            Self::Audit(_) => 305,
        };
        storage::error::DiagnosticCode::new("QAI-QUR", number)
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn remedy(&self) -> Option<String> {
        Some(
            match self {
                Self::ApprovalMissing { .. } => "Record a human approval first.",
                Self::ApprovalNotGranted { .. } => "Only a granted approval can activate.",
                Self::ApprovalSubjectMismatch { .. } => {
                    "Approve the exact edition URN being activated."
                }
                Self::NotStaged { .. } => "Import the edition to Staged first.",
                Self::Storage(_) => "Check the database and retry.",
                Self::Audit(_) => "Check the audit chain and retry.",
            }
            .to_string(),
        )
    }

    fn next_command(&self) -> Option<String> {
        Some("qai quran activate --help".to_string())
    }
}

async fn check_approval(
    uow: &mut dyn storage::UnitOfWork,
    approval_id: &str,
    expected_subject: &str,
) -> Result<(), ActivationError> {
    let approval = uow
        .sources()
        .get_approval(approval_id)
        .await
        .map_err(ActivationError::storage)?
        .ok_or_else(|| ActivationError::ApprovalMissing { id: approval_id.to_string() })?;
    if approval.decision.as_deref() != Some("approved") {
        return Err(ActivationError::ApprovalNotGranted { id: approval_id.to_string() });
    }
    if approval.subject_urn != expected_subject {
        return Err(ActivationError::ApprovalSubjectMismatch {
            id: approval_id.to_string(),
            actual: approval.subject_urn,
            expected: expected_subject.to_string(),
        });
    }
    Ok(())
}

async fn audit_activation(
    uow: &mut dyn storage::UnitOfWork,
    action: AuditAction,
    subject: &str,
    invoked_by: &PrincipalId,
    edition_id: &str,
    generation: i64,
) -> Result<(), ActivationError> {
    append_audit_event(
        uow,
        AuditEvent {
            id: AuditEventId::new(),
            sequence: 0,
            occurred_at: Timestamp::now(),
            actor: Actor::Principal { principal_id: *invoked_by },
            action,
            subject: SubjectRef(subject.to_string()),
            outcome: AuditOutcome::Allowed,
            reason: None,
            before: None,
            after: Some(serde_json::json!({
                "edition_id": edition_id,
                "corpus_generation": generation,
            })),
            request_id: None,
            prev_chain_hash: audit_chain_genesis(),
            chain_hash: audit_chain_genesis(),
        },
    )
    .await
    .map_err(|err| ActivationError::Audit(err.to_string()))?;
    Ok(())
}

/// Activate a staged edition under a granted human approval.
///
/// Verifies the approval (granted, covering this edition), moves staging to
/// canonical, flips the pointer, bumps the generation, and audits — in one
/// transaction. Returns the new `corpus_generation`.
pub async fn activate_edition(
    db: &dyn storage::Database,
    slug: &str,
    version: &str,
    invoked_by: &PrincipalId,
    approval_id: &str,
    at: &Timestamp,
) -> Result<i64, ActivationError> {
    let expected_subject = edition_urn(slug, version);
    let mut uow = db.write().await.map_err(ActivationError::storage)?;
    check_approval(&mut *uow, approval_id, &expected_subject).await?;
    let staged = uow
        .quran()
        .find_staged_edition(slug, version)
        .await
        .map_err(ActivationError::storage)?
        .ok_or_else(|| ActivationError::NotStaged {
            slug: slug.to_string(),
            version: version.to_string(),
        })?;
    let generation = uow
        .quran()
        .activate_edition(
            &staged.run_id,
            &staged.edition_id,
            &invoked_by.to_string(),
            approval_id,
            &at.to_string(),
        )
        .await
        .map_err(ActivationError::storage)?;
    audit_activation(
        &mut *uow,
        AuditAction::SourceActivated,
        &expected_subject,
        invoked_by,
        &staged.edition_id,
        generation,
    )
    .await?;
    uow.commit().await.map_err(ActivationError::storage)?;
    Ok(generation)
}

/// Roll back to a prior edition version under a granted human approval.
pub async fn rollback_edition(
    db: &dyn storage::Database,
    slug: &str,
    version: &str,
    invoked_by: &PrincipalId,
    approval_id: &str,
    at: &Timestamp,
) -> Result<i64, ActivationError> {
    let expected_subject = edition_urn(slug, version);
    let mut uow = db.write().await.map_err(ActivationError::storage)?;
    check_approval(&mut *uow, approval_id, &expected_subject).await?;
    let generation = uow
        .quran()
        .rollback_edition(slug, version, &invoked_by.to_string(), approval_id, &at.to_string())
        .await
        .map_err(ActivationError::storage)?;
    let edition = uow
        .quran()
        .get_edition_by_slug_version(slug, version)
        .await
        .map_err(ActivationError::storage)?;
    audit_activation(
        &mut *uow,
        AuditAction::SourceRolledBack,
        &expected_subject,
        invoked_by,
        &edition.map(|row| row.id).unwrap_or_default(),
        generation,
    )
    .await?;
    uow.commit().await.map_err(ActivationError::storage)?;
    Ok(generation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idempotency_key_pins_source_adapter_parser() {
        assert_eq!(import_idempotency_key("sv-1"), "sv-1:1.0.0:1.0.0");
        assert_eq!(QURAN_IMPORT_KIND, "quran.import");
    }

    #[test]
    fn activation_errors_carry_coded_diagnostics() {
        use storage::error::Diagnostic;
        let err = ActivationError::NotStaged { slug: "s".into(), version: "v".into() };
        assert_eq!(err.code().to_string(), "QAI-QUR-0303");
        assert!(err.remedy().is_some());
    }
}
