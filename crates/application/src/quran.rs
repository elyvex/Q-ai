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
use domain::{AuditEventId, Language, PrincipalId, SubjectRef, Timestamp};
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
    /// That version is already active.
    #[error("{slug}@{version} is already active")]
    AlreadyActive {
        /// Edition slug.
        slug: String,
        /// Edition version.
        version: String,
    },
    /// The reviewer name is empty (P1-T55 records, never invents).
    #[error("verified_by must name the reviewer")]
    EmptyReviewer,
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
            Self::AlreadyActive { .. } => 313,
            Self::EmptyReviewer => 306,
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
                Self::AlreadyActive { .. } => "That edition version is already active.",
                Self::EmptyReviewer => "Pass --reviewer with the reviewer's name (OD-02).",
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

/// Editorial verification claim recorded on an edition (P1-T55; OD-02).
///
/// Both fields are operator-supplied and recorded verbatim — never invented.
/// Empty values fail closed (`EmptyReviewer`, `QAI-QUR-0306`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditionVerification<'a> {
    /// Reviewer name, recorded in `verified_by`.
    pub reviewer: &'a str,
    /// Comparison method, recorded in `verification_method`.
    pub method: &'a str,
}

/// Record editorial verification (`verified_by`) on an edition (P1-T55; OD-02).
///
/// Approval-gated like activation/deprecation: requires a granted human
/// approval covering the exact edition URN, stamps `verified_by` /
/// `verified_at` / `verification_method` on the edition row, and records a
/// hash-chained `SourceApproved` audit event — in one transaction. Canonical
/// text is untouched (only the `verified_*` metadata columns are written).
///
/// The reviewer identity itself is an owner act this function records, never
/// invents: empty reviewer names or methods fail closed, and
/// missing/denied/mismatched approvals are rejected exactly like activation.
pub async fn record_edition_verification(
    db: &dyn storage::Database,
    slug: &str,
    version: &str,
    verification: &EditionVerification<'_>,
    invoked_by: &PrincipalId,
    approval_id: &str,
    at: &Timestamp,
) -> Result<(), ActivationError> {
    if verification.reviewer.trim().is_empty() || verification.method.trim().is_empty() {
        return Err(ActivationError::EmptyReviewer);
    }
    let expected_subject = edition_urn(slug, version);
    let mut uow = db.write().await.map_err(ActivationError::storage)?;
    check_approval(&mut *uow, approval_id, &expected_subject).await?;
    let edition = uow
        .quran()
        .get_edition_by_slug_version(slug, version)
        .await
        .map_err(ActivationError::storage)?
        .ok_or_else(|| ActivationError::NotStaged {
            slug: slug.to_string(),
            version: version.to_string(),
        })?;
    uow.quran()
        .set_edition_verification(
            &edition.id,
            verification.reviewer,
            &at.to_string(),
            verification.method,
        )
        .await
        .map_err(ActivationError::storage)?;
    append_audit_event(
        &mut *uow,
        AuditEvent {
            id: AuditEventId::new(),
            sequence: 0,
            occurred_at: Timestamp::now(),
            actor: Actor::Principal { principal_id: *invoked_by },
            action: AuditAction::SourceApproved,
            subject: SubjectRef(expected_subject),
            outcome: AuditOutcome::Allowed,
            reason: None,
            before: None,
            after: Some(serde_json::json!({
                "edition_id": edition.id,
                "verified_by": verification.reviewer,
                "verification_method": verification.method,
            })),
            request_id: None,
            prev_chain_hash: audit_chain_genesis(),
            chain_hash: audit_chain_genesis(),
        },
    )
    .await
    .map_err(|err| ActivationError::Audit(err.to_string()))?;
    uow.commit().await.map_err(ActivationError::storage)?;
    Ok(())
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
        .map_err(|err| match err {
            StorageError::Conflict => ActivationError::AlreadyActive {
                slug: slug.to_string(),
                version: version.to_string(),
            },
            other => ActivationError::storage(other),
        })?;
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

/// Open a database from configuration (CLI entry point helper).
pub async fn open_database(
    path: &str,
    max_connections: u32,
) -> Result<storage_sqlite::SqliteDatabase, StorageError> {
    storage_sqlite::SqliteDatabase::new(path, max_connections, true).await
}

/// Ensure a local principal row exists (single-user interim; Phase 11 owns identity).
pub async fn ensure_principal(
    db: &dyn storage::Database,
    id: &str,
    display_name: &str,
    at: &str,
) -> Result<(), StorageError> {
    let mut uow = db.write().await?;
    uow.sources()
        .upsert_principal(storage::repository::PrincipalRow {
            id: id.to_string(),
            kind: "local_user".to_string(),
            display_name: display_name.to_string(),
            created_at: at.to_string(),
        })
        .await?;
    uow.commit().await
}

/// Record a human approval decision and return its id.
pub async fn record_approval(
    db: &dyn storage::Database,
    id: &str,
    subject_urn: &str,
    requested_by: &str,
    decided_by: &str,
    request_payload: &str,
    at: &str,
) -> Result<(), StorageError> {
    let mut uow = db.write().await?;
    uow.sources()
        .insert_approval(storage::repository::ApprovalRow {
            id: id.to_string(),
            subject_urn: subject_urn.to_string(),
            kind: "CanonicalChange".to_string(),
            requested_by: Some(requested_by.to_string()),
            decided_by: Some(decided_by.to_string()),
            decision: Some("approved".to_string()),
            request_payload: request_payload.to_string(),
            decision_note: None,
            requested_at: at.to_string(),
            decided_at: Some(at.to_string()),
        })
        .await?;
    uow.commit().await
}

/// Validate a manifest document without touching the database (dry run).
pub fn dry_run_validate(
    manifest_text: &str,
) -> Result<quran_corpus::ValidationReport, quran_corpus::CorpusError> {
    use quran_corpus::{EditionAdapter, JsonAdapter};
    let source = JsonAdapter.parse(manifest_text)?;
    Ok(quran_corpus::validate_edition(&source))
}

/// Re-validate staged rows against stored statistics and hashes.
pub async fn validate_staged(
    db: &dyn storage::Database,
    slug: &str,
    version: &str,
) -> Result<quran_corpus::ValidationReport, ActivationError> {
    use quran_corpus::{reconstruct, tokenize};
    let mut uow = db.write().await.map_err(ActivationError::storage)?;
    let staged = uow
        .quran()
        .find_staged_edition(slug, version)
        .await
        .map_err(ActivationError::storage)?
        .ok_or_else(|| ActivationError::NotStaged {
            slug: slug.to_string(),
            version: version.to_string(),
        })?;
    let edition = uow
        .quran()
        .get_stg_edition(&staged.run_id, &staged.edition_id)
        .await
        .map_err(ActivationError::storage)?
        .ok_or_else(|| ActivationError::NotStaged {
            slug: slug.to_string(),
            version: version.to_string(),
        })?;
    let ayahs =
        uow.quran().list_stg_ayahs(&staged.run_id).await.map_err(ActivationError::storage)?;
    let mut findings = Vec::new();
    let stats: quran_core::EditionStatistics = serde_json::from_str(&edition.statistics_json)
        .map_err(|err| ActivationError::Storage(format!("stored statistics are corrupt: {err}")))?;
    if ayahs.len() as u32 != stats.ayah_count {
        findings.push(quran_corpus::Finding::new(
            "QV-004",
            quran_corpus::Severity::Fatal,
            "edition",
            format!("staged {} ayahs, statistics say {}", ayahs.len(), stats.ayah_count),
        ));
    }
    for row in &ayahs {
        let computed = tokenize(&row.text);
        let separators: Vec<String> = uow
            .quran()
            .list_stg_separators(&staged.run_id, &staged.edition_id, row.surah, row.ayah)
            .await
            .map_err(ActivationError::storage)?
            .into_iter()
            .map(|separator| separator.separator)
            .collect();
        if reconstruct(&computed.tokens, &separators) != row.text {
            findings.push(quran_corpus::Finding::new(
                "QV-011",
                quran_corpus::Severity::Fatal,
                format!("surah {} ayah {}", row.surah, row.ayah),
                "staged tokens do not reconstruct the ayah text".to_string(),
            ));
        }
    }
    // Recompute the text hash from staged rows and compare with the stored one.
    let texts: Vec<&str> = ayahs.iter().map(|row| row.text.as_str()).collect();
    let recomputed = quran_corpus::text_hash(&edition.slug, &edition.version, &texts);
    if quran_corpus::tagged(&recomputed) != edition.text_hash {
        findings.push(quran_corpus::Finding::new(
            "QV-014",
            quran_corpus::Severity::Fatal,
            "edition",
            "text_hash recomputed from staged rows differs".to_string(),
        ));
    }
    uow.rollback().await.map_err(ActivationError::storage)?;
    let fatal =
        findings.iter().filter(|finding| finding.severity == quran_corpus::Severity::Fatal).count()
            as u32;
    let error =
        findings.iter().filter(|finding| finding.severity == quran_corpus::Severity::Error).count()
            as u32;
    let warning = findings
        .iter()
        .filter(|finding| finding.severity == quran_corpus::Severity::Warning)
        .count() as u32;
    Ok(quran_corpus::ValidationReport {
        subject_urn: format!("quran-staged:{slug}@{version}"),
        validator: quran_corpus::VALIDATOR_NAME.to_string(),
        validator_version: quran_corpus::VALIDATOR_VERSION,
        outcome: if fatal > 0 || error > 0 {
            quran_corpus::Outcome::Fail
        } else if warning > 0 {
            quran_corpus::Outcome::PassWithWarnings
        } else {
            quran_corpus::Outcome::Pass
        },
        fatal_count: fatal,
        error_count: error,
        warning_count: warning,
        findings,
    })
}

/// Translation import manifest (v1 JSON shape).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TranslationManifest {
    /// Translation edition metadata.
    pub translation: TranslationMeta,
    /// Verse passages.
    pub passages: Vec<TranslationPassage>,
}

/// Translation edition metadata in the manifest.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TranslationMeta {
    /// Slug (unique with version).
    pub slug: String,
    /// Version.
    pub version: String,
    /// Display name.
    pub name: String,
    /// Named human translator (required, principle 5).
    pub translator: String,
    /// BCP-47 language.
    pub language: String,
    /// Aligned Arabic edition `slug@version`.
    pub aligned_edition: String,
    /// Numbering scheme.
    pub numbering_scheme: String,
    /// SPDX license id, when known.
    pub spdx_id: Option<String>,
}

/// One translated passage.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TranslationPassage {
    /// Surah number.
    pub surah: u16,
    /// Ayah number.
    pub ayah: u32,
    /// Translated text.
    pub text: String,
}

/// Import a translation edition: validate attribution + alignment, then store.
pub async fn import_translations(
    db: &dyn storage::Database,
    manifest_text: &str,
    source_version_id: &str,
    invoked_by: &PrincipalId,
    at: &Timestamp,
) -> Result<String, ActivationError> {
    use quran_core::{AyahNumber, SurahNumber};
    let manifest: TranslationManifest = serde_json::from_str(manifest_text).map_err(|err| {
        ActivationError::Storage(format!("translation manifest is not valid: {err}"))
    })?;
    if manifest.translation.translator.trim().is_empty() {
        return Err(ActivationError::Storage(
            "translation requires a non-empty translator (principle 5)".to_string(),
        ));
    }
    let (aligned_slug, aligned_version) =
        manifest.translation.aligned_edition.split_once('@').ok_or_else(|| {
            ActivationError::Storage("aligned_edition must be `slug@version`".to_string())
        })?;
    let mut uow = db.write().await.map_err(ActivationError::storage)?;
    // Alignment target must exist (canonical or staged).
    let canonical = uow
        .quran()
        .get_edition_by_slug_version(aligned_slug, aligned_version)
        .await
        .map_err(ActivationError::storage)?;
    // For a staged target the aligned ayah locations are cached once; for a
    // canonical target each passage is checked against the canonical ayah.
    let (aligned_id, staged_ayahs) = match canonical {
        Some(row) => (row.id, None),
        None => {
            let staged = uow
                .quran()
                .find_staged_edition(aligned_slug, aligned_version)
                .await
                .map_err(ActivationError::storage)?
                .ok_or_else(|| ActivationError::NotStaged {
                    slug: aligned_slug.to_string(),
                    version: aligned_version.to_string(),
                })?;
            let ayahs = uow
                .quran()
                .list_stg_ayahs(&staged.run_id)
                .await
                .map_err(ActivationError::storage)?
                .into_iter()
                .filter(|row| row.edition_id == staged.edition_id)
                .map(|row| (row.surah, row.ayah))
                .collect::<std::collections::HashSet<_>>();
            (staged.edition_id, Some(ayahs))
        }
    };
    // Every passage must be well-formed, non-empty, unique, and name a real ayah
    // of the aligned edition (structural alignment).
    let mut seen = std::collections::HashSet::new();
    for passage in &manifest.passages {
        SurahNumber::new(passage.surah)
            .map_err(|_| ActivationError::Storage(format!("bad surah {}", passage.surah)))?;
        AyahNumber::new(passage.ayah)
            .map_err(|_| ActivationError::Storage(format!("bad ayah {}", passage.ayah)))?;
        if passage.text.trim().is_empty() {
            return Err(ActivationError::Storage(format!(
                "empty translation for {}:{}",
                passage.surah, passage.ayah
            )));
        }
        if !seen.insert((passage.surah, passage.ayah)) {
            return Err(ActivationError::Storage(format!(
                "duplicate translation passage for {}:{}",
                passage.surah, passage.ayah
            )));
        }
        let exists = match &staged_ayahs {
            Some(ayahs) => ayahs.contains(&(i64::from(passage.surah), i64::from(passage.ayah))),
            None => uow
                .quran()
                .get_ayah(&aligned_id, i64::from(passage.surah), i64::from(passage.ayah))
                .await
                .map_err(ActivationError::storage)?
                .is_some(),
        };
        if !exists {
            return Err(ActivationError::Storage(format!(
                "aligned edition {aligned_slug}@{aligned_version} has no ayah {}:{}",
                passage.surah, passage.ayah
            )));
        }
    }
    let id = format!("tr-{}-{}", manifest.translation.slug, manifest.translation.version);
    let license_json = serde_json::json!({
        "status": "Unknown",
        "spdx_id": manifest.translation.spdx_id,
        "attribution_required": true,
        "redistribution_allowed": false,
        "export_allowed": false,
        "notes": "declared at import; verify before activation use",
    })
    .to_string();
    uow.quran()
        .insert_translation_edition(storage::quran::TranslationEditionRow {
            id: id.clone(),
            slug: manifest.translation.slug.clone(),
            version: manifest.translation.version.clone(),
            name: manifest.translation.name.clone(),
            translator: manifest.translation.translator.clone(),
            language: manifest.translation.language.clone(),
            aligned_edition_id: aligned_id,
            numbering_scheme: manifest.translation.numbering_scheme.clone(),
            license_json,
            trust_level: "ImportedUnverified".to_string(),
            source_version_id: source_version_id.to_string(),
            text_hash: String::new(),
            status: "Staged".to_string(),
            imported_at: at.to_string(),
        })
        .await
        .map_err(ActivationError::storage)?;
    // Attributed provenance for the translation dataset (principle 5: every
    // passage must point at a provenance row, not at a principal id).
    let provenance_id = format!("prov-{id}");
    let translation_urn =
        format!("quran-translation:{}@{}", manifest.translation.slug, manifest.translation.version);
    uow.provenance()
        .insert(storage::repository::ProvenanceRecord {
            id: provenance_id.clone(),
            layer: "canonical_source".to_string(),
            subject_urn: translation_urn,
            attribution_kind: "dataset".to_string(),
            attribution_json: serde_json::json!({
                "dataset_name": manifest.translation.slug,
                "dataset_version": manifest.translation.version,
                "translator": manifest.translation.translator,
                "aligned_edition": manifest.translation.aligned_edition,
                "source_version_id": source_version_id,
            })
            .to_string(),
            source_version_id: Some(source_version_id.to_string()),
            trust_level: "ImportedUnverified".to_string(),
            verification_status: "unverified".to_string(),
            confidence: None,
            versions_json: serde_json::json!({
                "source_version_id": source_version_id,
                "schema_version": 1,
            })
            .to_string(),
            created_by: invoked_by.to_string(),
        })
        .await
        .map_err(ActivationError::storage)?;
    for passage in &manifest.passages {
        uow.quran()
            .insert_translation_passage(storage::quran::TranslationPassageRow {
                translation_edition_id: id.clone(),
                surah: i64::from(passage.surah),
                ayah: i64::from(passage.ayah),
                text: passage.text.clone(),
                footnotes_json: "[]".to_string(),
                provenance_id: provenance_id.clone(),
            })
            .await
            .map_err(ActivationError::storage)?;
    }
    uow.commit().await.map_err(ActivationError::storage)?;
    Ok(id)
}

/// Word-gloss import manifest (v1 JSON shape).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GlossManifest {
    /// Gloss dataset metadata.
    pub dataset: GlossDatasetMeta,
    /// Word glosses.
    pub glosses: Vec<WordGlossEntry>,
}

/// Word-gloss dataset metadata in the manifest.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GlossDatasetMeta {
    /// Dataset id; must name an existing `sources` row (provisioned by the
    /// CLI, mirroring translation source versions).
    pub id: String,
    /// Aligned Arabic edition `slug@version`.
    pub aligned_edition: String,
}

/// One word gloss aligned to a canonical token position.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct WordGlossEntry {
    /// Surah number.
    pub surah: u16,
    /// Ayah number.
    pub ayah: u32,
    /// 1-based token position within the ayah.
    pub position: u32,
    /// BCP-47 language.
    pub language: String,
    /// Gloss text.
    pub gloss: String,
}

/// Import a word-gloss dataset: validate attribution + token alignment, then
/// store. The dataset is a separate attributed artifact, never canonical text
/// (principle 5); every gloss must name a real token of the aligned edition.
pub async fn import_glosses(
    db: &dyn storage::Database,
    manifest_text: &str,
    invoked_by: &PrincipalId,
    _at: &Timestamp,
) -> Result<usize, ActivationError> {
    use quran_core::{AyahNumber, SurahNumber};
    let manifest: GlossManifest = serde_json::from_str(manifest_text)
        .map_err(|err| ActivationError::Storage(format!("gloss manifest is not valid: {err}")))?;
    if manifest.dataset.id.trim().is_empty() {
        return Err(ActivationError::Storage("gloss dataset requires a non-empty id".to_string()));
    }
    let (aligned_slug, aligned_version) =
        manifest.dataset.aligned_edition.split_once('@').ok_or_else(|| {
            ActivationError::Storage("aligned_edition must be `slug@version`".to_string())
        })?;
    let mut uow = db.write().await.map_err(ActivationError::storage)?;
    // The dataset must be a catalogued source: glosses are attributed data.
    let dataset_known =
        uow.sources().get(&manifest.dataset.id).await.map_err(ActivationError::storage)?.is_some();
    if !dataset_known {
        return Err(ActivationError::Storage(format!(
            "unknown gloss dataset `{}`; provision its source row first",
            manifest.dataset.id
        )));
    }
    // Alignment target must exist (canonical or staged).
    let canonical = uow
        .quran()
        .get_edition_by_slug_version(aligned_slug, aligned_version)
        .await
        .map_err(ActivationError::storage)?;
    let (aligned_id, staged) = match canonical {
        Some(row) => (row.id, None),
        None => {
            let staged = uow
                .quran()
                .find_staged_edition(aligned_slug, aligned_version)
                .await
                .map_err(ActivationError::storage)?
                .ok_or_else(|| ActivationError::NotStaged {
                    slug: aligned_slug.to_string(),
                    version: aligned_version.to_string(),
                })?;
            let edition_id = staged.edition_id.clone();
            let run_id = staged.run_id.clone();
            (edition_id, Some(run_id))
        }
    };
    // Token counts per ayah, so every gloss names a real token position.
    // Unique ayahs are read once; positions are validated against the count.
    let mut ayahs: std::collections::HashSet<(u16, u32)> = std::collections::HashSet::new();
    for entry in &manifest.glosses {
        ayahs.insert((entry.surah, entry.ayah));
    }
    let mut token_counts: std::collections::HashMap<(u16, u32), usize> =
        std::collections::HashMap::new();
    for (surah, ayah) in &ayahs {
        let count = match &staged {
            Some(run_id) => {
                let rows = uow
                    .quran()
                    .list_stg_tokens(run_id, &aligned_id, i64::from(*surah), i64::from(*ayah))
                    .await
                    .map_err(ActivationError::storage)?;
                // A staged ayah with no token rows is not a real ayah.
                if rows.is_empty() {
                    let staged_ayahs = uow
                        .quran()
                        .list_stg_ayahs(run_id)
                        .await
                        .map_err(ActivationError::storage)?;
                    let exists = staged_ayahs.iter().any(|row| {
                        row.edition_id == aligned_id
                            && row.surah == i64::from(*surah)
                            && row.ayah == i64::from(*ayah)
                    });
                    if !exists {
                        return Err(ActivationError::Storage(format!(
                            "aligned edition {aligned_slug}@{aligned_version} has no ayah {surah}:{ayah}"
                        )));
                    }
                }
                rows.len()
            }
            None => {
                uow.quran()
                    .get_ayah(&aligned_id, i64::from(*surah), i64::from(*ayah))
                    .await
                    .map_err(ActivationError::storage)?
                    .ok_or_else(|| {
                        ActivationError::Storage(format!(
                            "aligned edition {aligned_slug}@{aligned_version} has no ayah {surah}:{ayah}"
                        ))
                    })?;
                uow.quran()
                    .get_tokens(&aligned_id, i64::from(*surah), i64::from(*ayah))
                    .await
                    .map_err(ActivationError::storage)?
                    .len()
            }
        };
        token_counts.insert((*surah, *ayah), count);
    }
    // Every gloss must be well-formed, non-empty, unique, and land on a real
    // token (structural alignment at token granularity).
    let mut seen = std::collections::HashSet::new();
    for entry in &manifest.glosses {
        SurahNumber::new(entry.surah)
            .map_err(|_| ActivationError::Storage(format!("bad surah {}", entry.surah)))?;
        AyahNumber::new(entry.ayah)
            .map_err(|_| ActivationError::Storage(format!("bad ayah {}", entry.ayah)))?;
        entry
            .language
            .parse::<Language>()
            .map_err(|_| ActivationError::Storage(format!("bad language `{}`", entry.language)))?;
        if entry.gloss.trim().is_empty() {
            return Err(ActivationError::Storage(format!(
                "empty gloss for {}:{}#{}",
                entry.surah, entry.ayah, entry.position
            )));
        }
        if entry.position == 0 {
            return Err(ActivationError::Storage(format!(
                "bad token position {} (positions are 1-based)",
                entry.position
            )));
        }
        let count = token_counts[&(entry.surah, entry.ayah)];
        if (entry.position as usize) > count {
            return Err(ActivationError::Storage(format!(
                "aligned edition {aligned_slug}@{aligned_version} ayah {}:{} has {count} tokens; no position {}",
                entry.surah, entry.ayah, entry.position
            )));
        }
        if !seen.insert((entry.surah, entry.ayah, entry.position, entry.language.clone())) {
            return Err(ActivationError::Storage(format!(
                "duplicate gloss for {}:{}#{} [{}]",
                entry.surah, entry.ayah, entry.position, entry.language
            )));
        }
    }
    // Attributed provenance for the gloss dataset (principle 5). Glosses are
    // scholarly annotations aligned to canonical tokens, not canonical text,
    // so they use the annotation layer rather than `canonical_source`.
    let provenance_id =
        format!("prov-gloss-{}-{aligned_slug}-{aligned_version}", manifest.dataset.id);
    let gloss_urn = format!("quran-gloss:{}", manifest.dataset.id);
    uow.provenance()
        .insert(storage::repository::ProvenanceRecord {
            id: provenance_id.clone(),
            layer: "scholarly_annotation".to_string(),
            subject_urn: gloss_urn,
            attribution_kind: "dataset".to_string(),
            attribution_json: serde_json::json!({
                "dataset_id": manifest.dataset.id,
                "aligned_edition": manifest.dataset.aligned_edition,
                "gloss_count": manifest.glosses.len(),
            })
            .to_string(),
            source_version_id: None,
            trust_level: "ImportedUnverified".to_string(),
            verification_status: "unverified".to_string(),
            confidence: None,
            versions_json: serde_json::json!({"schema_version": 1}).to_string(),
            created_by: invoked_by.to_string(),
        })
        .await
        .map_err(ActivationError::storage)?;
    for entry in &manifest.glosses {
        uow.quran()
            .insert_word_gloss(storage::quran::WordGlossRow {
                gloss_dataset_id: manifest.dataset.id.clone(),
                edition_id: aligned_id.clone(),
                surah: i64::from(entry.surah),
                ayah: i64::from(entry.ayah),
                position: i64::from(entry.position),
                language: entry.language.clone(),
                gloss: entry.gloss.clone(),
                provenance_id: provenance_id.clone(),
            })
            .await
            .map_err(ActivationError::storage)?;
    }
    uow.commit().await.map_err(ActivationError::storage)?;
    Ok(manifest.glosses.len())
}

/// Deprecate a canonical edition (human-gated maintenance; rows are kept).
pub async fn deprecate_edition(
    db: &dyn storage::Database,
    slug: &str,
    version: &str,
    invoked_by: &PrincipalId,
    approval_id: &str,
    at: &Timestamp,
) -> Result<(), ActivationError> {
    let expected_subject = edition_urn(slug, version);
    let mut uow = db.write().await.map_err(ActivationError::storage)?;
    check_approval(&mut *uow, approval_id, &expected_subject).await?;
    let edition = uow
        .quran()
        .get_edition_by_slug_version(slug, version)
        .await
        .map_err(ActivationError::storage)?
        .ok_or_else(|| ActivationError::NotStaged {
            slug: slug.to_string(),
            version: version.to_string(),
        })?;
    uow.quran()
        .set_edition_status(&edition.id, "Deprecated")
        .await
        .map_err(ActivationError::storage)?;
    audit_activation(
        &mut *uow,
        AuditAction::SourceRolledBack,
        &expected_subject,
        invoked_by,
        &edition.id,
        0,
    )
    .await?;
    let _ = at;
    uow.commit().await.map_err(ActivationError::storage)?;
    Ok(())
}

/// Enqueue a `quran.import` job and run the worker inline to completion.
pub async fn run_import_job(
    db: &std::sync::Arc<storage_sqlite::SqliteDatabase>,
    input: quran_corpus::import::ImportInput,
) -> Result<jobs::JobOutcome, jobs::JobError> {
    use jobs::queue::JobQueue;
    use jobs::registry::HandlerRegistry;
    use jobs::worker::Worker;
    let queue = std::sync::Arc::new(crate::job_queue::SqliteJobQueue::new(db.clone()));
    let registry = std::sync::Arc::new(
        HandlerRegistry::new().register(std::sync::Arc::new(QuranImportHandler::new(db.clone()))),
    );
    let job_id = input.job_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let mut payload = input.clone();
    payload.job_id = Some(job_id.clone());
    let payload_json = serde_json::to_value(&payload)
        .map_err(|err| jobs::JobError::Storage(format!("bad import payload: {err}")))?;
    queue
        .enqueue(storage::repository::JobRecord {
            id: job_id.clone(),
            kind: QURAN_IMPORT_KIND.into(),
            payload_json: payload_json.to_string(),
            idempotency_key: Some(import_idempotency_key(&input.source_version_id)),
            state: "Queued".into(),
            priority: 0,
            attempts: 0,
            max_attempts: 1,
            available_at: input.created_at.clone(),
            lease_owner: None,
            lease_expires_at: None,
            checkpoint_json: None,
            cancel_requested: false,
            created_by: input.invoked_by.clone(),
        })
        .await?;
    let worker = Worker::new(queue.clone(), registry, "qai-cli");
    worker.run_until_idle().await?;
    // Read the terminal state through the queue.
    let job =
        queue.get(&job_id).await?.ok_or_else(|| jobs::JobError::NotFound { id: job_id.clone() })?;
    match job.state.as_str() {
        "Succeeded" => Ok(jobs::JobOutcome { success: true, result: Some(job_id) }),
        "Cancelled" => Err(jobs::JobError::Cancelled { id: job_id }),
        _ => Err(jobs::JobError::Storage(format!("import job ended as {}", job.state))),
    }
}
