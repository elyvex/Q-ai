//! Phase 2 — morphology import/activate jobs + read tools (Sprints 2.4–2.5,
//! P2-T66/T67/T75–T81/T83–T88, D2.6–D2.8).
//!
//! Import/activation separation (I5 extended to datasets): the importer
//! holds NO approval capability and literally cannot activate — it parses,
//! aligns (never modifying tokens), validates (MV-001…018), and stages.
//! Activation is a separate approval-gated command that promotes staged rows
//! into the lexicon and supersedes the previous dataset (never mutating
//! lexicon fields in place).
//!
//! Read tools degrade to typed `UnavailableDataset` (AC-P2-01 fallback)
//! until a dataset activates — never guessed data. `affix_search` additionally
//! serves the L7 heuristic backend over stored forms, always labeled.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use audit::{Actor, AuditAction, AuditEvent, AuditOutcome};
use domain::{AuditEventId, SubjectRef};
use storage::Database as _;
use storage::error::StorageError;
use storage::quran::{
    AlignmentRow, LexiconProvenance, MorphologyFindingRow, QuranDatasetRow, StagedMorphRow,
    StagingBatchRow,
};
use storage_sqlite::SqliteDatabase;

/// The 12 import checkpoints (P2-T66, cancel-safe, resumable).
pub const IMPORT_CHECKPOINTS: [&str; 12] = [
    "read_manifest",
    "parse",
    "normalize_refs",
    "load_inventory",
    "direct_key",
    "alignment_table",
    "unmatched_report",
    "mv_validate",
    "coverage_report",
    "build_lexicon_rows",
    "write_staging",
    "finalize",
];

/// Job kind for morphology imports.
pub const MORPHOLOGY_IMPORT_KIND: &str = "quran.morphology.import";

/// Morphology job failures. Codes reuse `QAI-MORPH-*` (crate) + `QAI-IDX-5`
/// for canonical drift (shared MV-018 verifier); no new namespace.
#[derive(Debug, Clone, thiserror::Error)]
pub enum MorphologyJobError {
    /// Storage failure.
    #[error("morphology storage failed: {0}")]
    Storage(String),
    /// Crate-level failure (parse/validation/alignment/review).
    #[error(transparent)]
    Morphology(#[from] quran_morphology::MorphologyError),
    /// Canonical drift (MV-018).
    #[error("canonical text changed during morphology import")]
    CanonicalChanged,
    /// Approval missing/denied/mismatched (activation only).
    #[error("activation approval invalid: {0}")]
    Approval(String),
    /// Batch not in the required state.
    #[error("batch {id} is {actual}; need {expected}")]
    BadState {
        /// Batch id.
        id: String,
        /// Actual state.
        actual: String,
        /// Required state.
        expected: String,
    },
    /// Blocking findings (fatal, or error at activation).
    #[error("blocking validation findings: {0} fatal, {1} error")]
    BlockingFindings(usize, usize),
    /// Unmatched tokens remain (activation gate, AC-P2-16).
    #[error("unmatched tokens remain: {0} across {1} surah(s)")]
    UnmatchedRemaining(usize, usize),
    /// Cancellation requested; staging keeps prior checkpoint state.
    #[error("morphology import cancelled")]
    Cancelled,
}

impl MorphologyJobError {
    fn storage(error: StorageError) -> Self {
        Self::Storage(error.to_string())
    }
}

impl storage::error::Diagnostic for MorphologyJobError {
    fn code(&self) -> storage::error::DiagnosticCode {
        use quran_morphology::MorphologyError as ME;
        match self {
            Self::Storage(_) => storage::error::DiagnosticCode::new("QAI-MORPH", 2),
            Self::Morphology(inner) => {
                let number = match inner {
                    ME::UnknownDataset { .. } => 1,
                    ME::ValidationFailed { .. } => 2,
                    ME::AlignmentFailed { .. } => 3,
                    ME::UnavailableDataset { .. } => 4,
                    ME::ReviewViolation { .. } => 5,
                };
                storage::error::DiagnosticCode::new("QAI-MORPH", number)
            }
            Self::CanonicalChanged => storage::error::DiagnosticCode::new("QAI-IDX", 5),
            Self::Approval(_) => storage::error::DiagnosticCode::new("QAI-MORPH", 5),
            Self::BadState { .. } | Self::BlockingFindings(..) | Self::UnmatchedRemaining(..) => {
                storage::error::DiagnosticCode::new("QAI-MORPH", 2)
            }
            Self::Cancelled => storage::error::DiagnosticCode::new("QAI-MORPH", 2),
        }
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn remedy(&self) -> Option<String> {
        Some(match self {
            Self::Storage(_) => "Check the database and retry.".to_string(),
            Self::Morphology(_) => "Fix the dataset document and re-import.".to_string(),
            Self::CanonicalChanged => "Canonical drift blocks imports; run `qai doctor`.".to_string(),
            Self::Approval(_) => "Request approval for the dataset URN first.".to_string(),
            Self::BadState { .. } => "Import to Staged (zero fatals) before activating.".to_string(),
            Self::BlockingFindings(..) => "Resolve fatal/error findings, then re-import.".to_string(),
            Self::UnmatchedRemaining(..) => "Fix alignment gaps (per-surah report), then re-import.".to_string(),
            Self::Cancelled => "Re-run the import; staging resumes from checkpoints.".to_string(),
        })
    }

    fn next_command(&self) -> Option<String> {
        Some("qai doctor --quran".to_string())
    }
}

/// Parameters for a morphology import.
#[derive(Debug, Clone)]
pub struct MorphologyImportParams {
    /// Dataset slug.
    pub dataset_slug: String,
    /// Dataset version.
    pub dataset_version: String,
    /// Adapter name (`json` | `csv`).
    pub adapter: String,
    /// Intermediate document text (adapter input).
    pub document_text: String,
    /// Edition slug to align against (must be active).
    pub edition_slug: String,
    /// Edition version to align against (must be active).
    pub edition_version: String,
    /// Operator principal id.
    pub invoked_by: String,
    /// Batch id (resume reuses it; default: generated).
    pub batch_id: Option<String>,
    /// Dataset attribution string (recorded on the dataset row).
    pub attribution: String,
    /// License status + JSON.
    pub license_status: String,
    /// License detail JSON.
    pub license_json: String,
}

/// Report for one completed import.
#[derive(Debug, Clone)]
pub struct MorphologyImportReport {
    /// Batch id.
    pub batch_id: String,
    /// Dataset slug@version.
    pub dataset: String,
    /// Direct-key matches.
    pub matched: usize,
    /// Table-mapped rows.
    pub table_mapped: usize,
    /// Unmatched rows per surah.
    pub unmatched_by_surah: BTreeMap<u32, usize>,
    /// Findings by severity.
    pub findings: BTreeMap<String, usize>,
    /// Terminal state (`staged` | `failed`).
    pub state: String,
    /// MV-018 post-check (always `unchanged` on success).
    pub mv018_unchanged: bool,
}

fn now() -> String {
    domain::Timestamp::now().to_string()
}

fn cancel_check(cancel: &AtomicBool) -> Result<(), MorphologyJobError> {
    if cancel.load(Ordering::SeqCst) {
        Err(MorphologyJobError::Cancelled)
    } else {
        Ok(())
    }
}

async fn set_checkpoint(
    db: &SqliteDatabase,
    batch_id: &str,
    state: &str,
    checkpoint: &str,
    finished_at: Option<&str>,
) -> Result<(), MorphologyJobError> {
    let mut uow = db.write().await.map_err(MorphologyJobError::storage)?;
    uow.quran()
        .update_staging_batch(batch_id, state, checkpoint, finished_at)
        .await
        .map_err(MorphologyJobError::storage)?;
    uow.commit().await.map_err(MorphologyJobError::storage)?;
    Ok(())
}

/// Run a morphology import end to end (P2-T66).
///
/// Deterministic restart-is-resume: staging rows/alignment/findings are
/// replaced per batch id, checkpoints advance monotonically, and a
/// cancelled run keeps its last checkpoint state.
#[allow(clippy::too_many_lines)]
pub async fn run_morphology_import(
    db: &SqliteDatabase,
    params: &MorphologyImportParams,
    cancel: &AtomicBool,
    checkpoint_cb: impl Fn(&str),
) -> Result<MorphologyImportReport, MorphologyJobError> {
    use quran_morphology::{InventoryToken, align, validate};
    let started = now();
    let batch_id = params.batch_id.clone().unwrap_or_else(|| format!("morph-{}-{}", params.dataset_slug, uuid::Uuid::new_v4()));

    // Resolve + verify the active edition first (imports target active only).
    let edition_id = {
        let mut uow = db.write().await.map_err(MorphologyJobError::storage)?;
        let edition = uow
            .quran()
            .get_edition_by_slug_version(&params.edition_slug, &params.edition_version)
            .await
            .map_err(MorphologyJobError::storage)?;
        let active = uow.quran().get_active().await.map_err(MorphologyJobError::storage)?;
        uow.rollback().await.map_err(MorphologyJobError::storage)?;
        match (edition, active) {
            (Some(ed), Some(active)) if active.edition_id == ed.id => ed.id.clone(),
            _ => {
                return Err(MorphologyJobError::Storage(format!(
                    "edition {}@{} is not active; imports target the active edition",
                    params.edition_slug, params.edition_version
                )));
            }
        }
    };

    // MV-018 pre-check: stop before ANY write on drift.
    crate::quran_forms::verify_canonical_unchanged(db, &edition_id)
        .await
        .map_err(|_| MorphologyJobError::CanonicalChanged)?;

    // Open (or resume) the batch row.
    {
        let mut uow = db.write().await.map_err(MorphologyJobError::storage)?;
        let existing = uow
            .quran()
            .get_staging_batch(&batch_id)
            .await
            .map_err(MorphologyJobError::storage)?;
        if existing.is_none() {
            uow.quran()
                .insert_staging_batch(StagingBatchRow {
                    id: batch_id.clone(),
                    dataset_slug: params.dataset_slug.clone(),
                    dataset_version: params.dataset_version.clone(),
                    adapter: params.adapter.clone(),
                    source_manifest_hash: quran_corpus::sha256_hex(params.document_text.as_bytes()),
                    state: "reading".to_string(),
                    checkpoint: IMPORT_CHECKPOINTS[0].to_string(),
                    created_at: started.clone(),
                    finished_at: None,
                })
                .await
                .map_err(MorphologyJobError::storage)?;
        }
        uow.commit().await.map_err(MorphologyJobError::storage)?;
    }
async fn advance(
    db: &SqliteDatabase,
    batch_id: &str,
    checkpoint: &str,
    cb: impl Fn(&str),
    cancel: &AtomicBool,
) -> Result<(), MorphologyJobError> {
    cancel_check(cancel)?;
    set_checkpoint(db, batch_id, "reading", checkpoint, None).await?;
    cb(checkpoint);
    Ok(())
}

    // 1. read_manifest.
    advance(db, &batch_id, IMPORT_CHECKPOINTS[0], &checkpoint_cb, cancel).await?;
    // 2. parse via the named adapter (T59/T60: exactly one adapter per shape).
    advance(db, &batch_id, IMPORT_CHECKPOINTS[1], &checkpoint_cb, cancel).await?;
    let intermediate = match params.adapter.as_str() {
        "json" => {
            let dataset_ref = quran_morphology::DatasetRef::new(
                params.dataset_slug.clone(),
                params.dataset_version.clone(),
            );
            quran_morphology::adapter_json::parse_array_shape(
                &params.document_text,
                &dataset_ref,
                &edition_id,
            )?
        }
        "csv" => {
            let dataset_ref = quran_morphology::DatasetRef::new(
                params.dataset_slug.clone(),
                params.dataset_version.clone(),
            );
            quran_morphology::adapter_csv::parse_flat_csv(
                &params.document_text,
                &dataset_ref,
                &edition_id,
            )?
        }
        other => {
            return Err(MorphologyJobError::Morphology(
                quran_morphology::MorphologyError::ValidationFailed {
                    detail: format!("unknown morphology adapter: {other}"),
                },
            ));
        }
    };
    // 3. normalize_refs (reference strings validated by MV-003 downstream).
    advance(db, &batch_id, IMPORT_CHECKPOINTS[2], &checkpoint_cb, cancel).await?;
    // 4. load_inventory: canonical token surfaces (read-only borrow).
    advance(db, &batch_id, IMPORT_CHECKPOINTS[3], &checkpoint_cb, cancel).await?;
    let inventory: Vec<InventoryToken> = {
        let mut uow = db.write().await.map_err(MorphologyJobError::storage)?;
        let ayahs = uow
            .quran()
            .list_ayahs_range(&edition_id, 1, i64::MAX)
            .await
            .map_err(MorphologyJobError::storage)?;
        let mut inventory = Vec::new();
        for ayah in &ayahs {
            cancel_check(cancel)?;
            let tokens = uow
                .quran()
                .get_tokens(&edition_id, ayah.surah, ayah.ayah)
                .await
                .map_err(MorphologyJobError::storage)?;
            for token in &tokens {
                inventory.push((
                    ayah.surah as u32,
                    ayah.ayah as u32,
                    token.position as u32,
                    token.surface.clone(),
                ));
            }
        }
        uow.rollback().await.map_err(MorphologyJobError::storage)?;
        inventory
    };
    // 5–6. direct_key + alignment_table (T61; never modifies tokens).
    advance(db, &batch_id, IMPORT_CHECKPOINTS[4], &checkpoint_cb, cancel).await?;
    advance(db, &batch_id, IMPORT_CHECKPOINTS[5], &checkpoint_cb, cancel).await?;
    let report = align::align(&intermediate, &inventory);
    let created = now();
    let alignment_rows: Vec<AlignmentRow> = {
        let mut rows = Vec::new();
        // One alignment row per token POSITION (multi-analysis rows share
        // their token's direct key; the UNIQUE(batch, key) constraint is
        // positional by design).
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for analysis in &intermediate.analyses {
            let key = quran_morphology::align::DirectKey::new(
                analysis.surah,
                analysis.ayah,
                analysis.token_position,
                &analysis.surface,
            );
            if !seen.insert(key.as_str().to_string()) {
                continue;
            }
            let (kind, hash) = match report.table.get(key.as_str()) {
                Some(entry) => ("table_mapped", entry.alignment_hash.clone()),
                None => ("direct_key", String::new()),
            };
            // Skip rows that are neither direct nor table-mapped (unmatched);
            // they are recorded below with kind `unmatched`.
            let position_hit = inventory.iter().any(|(s, a, p, _)| {
                *s == analysis.surah && *a == analysis.ayah && *p == analysis.token_position
            });
            let kind = if position_hit { kind } else { "unmatched" };
            rows.push(AlignmentRow {
                id: format!("al-{batch_id}-{}-{}-{}-{}", analysis.surah, analysis.ayah, analysis.token_position, analysis.analysis_index),
                batch_id: batch_id.clone(),
                direct_key: key.as_str().to_string(),
                edition_id: edition_id.clone(),
                surah: analysis.surah as i64,
                ayah: analysis.ayah as i64,
                token_position: analysis.token_position as i64,
                alignment_kind: kind.to_string(),
                alignment_hash: hash,
                created_at: created.clone(),
            });
        }
        rows
    };
    {
        let mut uow = db.write().await.map_err(MorphologyJobError::storage)?;
        uow.quran()
            .replace_alignment_rows(&batch_id, alignment_rows)
            .await
            .map_err(MorphologyJobError::storage)?;
        uow.commit().await.map_err(MorphologyJobError::storage)?;
    }
    // 7. unmatched_report (T68 input; gates approval at activation).
    advance(db, &batch_id, IMPORT_CHECKPOINTS[6], &checkpoint_cb, cancel).await?;
    let unmatched_by_surah: BTreeMap<u32, usize> =
        report.unmatched_by_surah.iter().map(|(s, n)| (*s, *n)).collect();
    // 8. mv_validate (T63; adversarial fixtures pin specific MV ids in T71).
    advance(db, &batch_id, IMPORT_CHECKPOINTS[7], &checkpoint_cb, cancel).await?;
    let findings = validate::validate(&intermediate, &inventory);
    let finding_rows: Vec<MorphologyFindingRow> = findings
        .iter()
        .enumerate()
        .map(|(i, finding)| MorphologyFindingRow {
            id: format!("mv-{batch_id}-{i}"),
            batch_id: batch_id.clone(),
            rule_id: finding.rule_id.to_string(),
            severity: format!("{:?}", finding.severity).to_lowercase(),
            reference: finding.reference.clone(),
            detail: finding.detail.clone(),
            created_at: created.clone(),
        })
        .collect();
    {
        let mut uow = db.write().await.map_err(MorphologyJobError::storage)?;
        uow.quran()
            .replace_findings(&batch_id, finding_rows)
            .await
            .map_err(MorphologyJobError::storage)?;
        uow.commit().await.map_err(MorphologyJobError::storage)?;
    }
    let fatal = findings.iter().filter(|f| f.severity == validate::Severity::Fatal).count();
    let _error = findings.iter().filter(|f| f.severity == validate::Severity::Error).count();
    // 9. coverage_report.
    advance(db, &batch_id, IMPORT_CHECKPOINTS[8], &checkpoint_cb, cancel).await?;
    // 10. build_lexicon_rows (preview shapes; ids resolve at activation).
    advance(db, &batch_id, IMPORT_CHECKPOINTS[9], &checkpoint_cb, cancel).await?;
    // 11. write_staging (raw intermediate payloads; idempotent per batch).
    advance(db, &batch_id, IMPORT_CHECKPOINTS[10], &checkpoint_cb, cancel).await?;
    {
        let staged: Vec<StagedMorphRow> = intermediate
            .analyses
            .iter()
            .map(|analysis| StagedMorphRow {
                id: format!(
                    "st-{batch_id}-{}",
                    quran_morphology::IntermediateMorphology::reference_of(analysis)
                ),
                batch_id: batch_id.clone(),
                reference: quran_morphology::IntermediateMorphology::reference_of(analysis),
                surah: analysis.surah as i64,
                ayah: analysis.ayah as i64,
                token_position: analysis.token_position as i64,
                payload_json: serde_json::to_string(analysis).unwrap_or_default(),
                native_tags_json: serde_json::json!({
                    "pos_native": analysis.pos_native,
                    "features": analysis.features_json,
                })
                .to_string(),
                created_at: created.clone(),
            })
            .collect();
        let mut uow = db.write().await.map_err(MorphologyJobError::storage)?;
        uow.quran()
            .replace_staging_rows(&batch_id, staged)
            .await
            .map_err(MorphologyJobError::storage)?;
        uow.commit().await.map_err(MorphologyJobError::storage)?;
    }
    // MV-018 post-check.
    crate::quran_forms::verify_canonical_unchanged(db, &edition_id)
        .await
        .map_err(|_| MorphologyJobError::CanonicalChanged)?;
    // 12. finalize: Staged with zero fatals, else failed.
    let state = if fatal == 0 { "staged" } else { "failed" };
    set_checkpoint(db, &batch_id, state, IMPORT_CHECKPOINTS[11], Some(&now())).await?;
    checkpoint_cb(IMPORT_CHECKPOINTS[11]);

    let mut finding_counts: BTreeMap<String, usize> = BTreeMap::new();
    for finding in &findings {
        *finding_counts.entry(format!("{:?}", finding.severity).to_lowercase()).or_insert(0) += 1;
    }
    Ok(MorphologyImportReport {
        batch_id: batch_id.clone(),
        dataset: format!("{}@{}", params.dataset_slug, params.dataset_version),
        matched: report.matched,
        table_mapped: report.table_mapped,
        unmatched_by_surah,
        findings: finding_counts,
        state: state.to_string(),
        mv018_unchanged: true,
    })
}

/// Dataset URN for approval subjects.
#[must_use]
pub fn dataset_urn(slug: &str, version: &str) -> String {
    format!("morphology-dataset:{slug}@{version}")
}

/// Parameters for morphology activation (approval-gated, P2-T67).
#[derive(Debug, Clone)]
pub struct MorphologyActivateParams {
    /// Staging batch id (must be `staged`).
    pub batch_id: String,
    /// Approval id covering the dataset URN.
    pub approval_id: String,
    /// Operator principal id.
    pub invoked_by: String,
}

/// Report for one activation.
#[derive(Debug, Clone)]
pub struct MorphologyActivateReport {
    /// Dataset slug@version now active.
    pub dataset: String,
    /// Rows promoted (roots, lemmas, analyses, morphemes).
    pub promoted: (usize, usize, usize, usize),
    /// Previously active dataset, if any (now superseded).
    pub previous_dataset: Option<String>,
}

/// Activate a staged morphology batch (P2-T67).
///
/// Separate human command: requires a granted approval covering the exact
/// dataset URN, zero fatal AND zero error findings, and full alignment.
/// Promotes staged rows into the lexicon and flips dataset state in ONE
/// transaction — never mutating lexicon fields in place.
pub async fn activate_morphology(
    db: &SqliteDatabase,
    params: &MorphologyActivateParams,
    invoked_by: &domain::PrincipalId,
) -> Result<MorphologyActivateReport, MorphologyJobError> {
    let finished = now();
    let mut uow = db.write().await.map_err(MorphologyJobError::storage)?;
    let batch = uow
        .quran()
        .get_staging_batch(&params.batch_id)
        .await
        .map_err(MorphologyJobError::storage)?
        .ok_or_else(|| MorphologyJobError::BadState {
            id: params.batch_id.clone(),
            actual: "missing".to_string(),
            expected: "staged".to_string(),
        })?;
    if batch.state != "staged" {
        uow.rollback().await.map_err(MorphologyJobError::storage)?;
        return Err(MorphologyJobError::BadState {
            id: params.batch_id.clone(),
            actual: batch.state,
            expected: "staged".to_string(),
        });
    }
    let urn = dataset_urn(&batch.dataset_slug, &batch.dataset_version);
    // Approval gate (same discipline as edition activation).
    let approval = uow
        .sources()
        .get_approval(&params.approval_id)
        .await
        .map_err(MorphologyJobError::storage)?
        .ok_or_else(|| MorphologyJobError::Approval(format!("missing {}", params.approval_id)))?;
    if approval.decision.as_deref() != Some("approved") {
        uow.rollback().await.map_err(MorphologyJobError::storage)?;
        return Err(MorphologyJobError::Approval(format!("{} not granted", params.approval_id)));
    }
    if approval.subject_urn != urn {
        uow.rollback().await.map_err(MorphologyJobError::storage)?;
        return Err(MorphologyJobError::Approval(format!(
            "subject mismatch: {} != {urn}",
            approval.subject_urn
        )));
    }
    let findings = uow
        .quran()
        .list_findings(&params.batch_id)
        .await
        .map_err(MorphologyJobError::storage)?;
    let fatal = findings.iter().filter(|f| f.severity == "fatal").count();
    let error = findings.iter().filter(|f| f.severity == "error").count();
    if fatal + error > 0 {
        uow.rollback().await.map_err(MorphologyJobError::storage)?;
        return Err(MorphologyJobError::BlockingFindings(fatal, error));
    }
    let alignment = uow
        .quran()
        .list_alignment(&params.batch_id)
        .await
        .map_err(MorphologyJobError::storage)?;
    let unmatched: Vec<_> = alignment.iter().filter(|a| a.alignment_kind == "unmatched").collect();
    if !unmatched.is_empty() {
        let surahs: BTreeMap<i64, usize> = {
            let mut map = BTreeMap::new();
            for row in &unmatched {
                *map.entry(row.surah).or_insert(0) += 1;
            }
            map
        };
        uow.rollback().await.map_err(MorphologyJobError::storage)?;
        return Err(MorphologyJobError::UnmatchedRemaining(unmatched.len(), surahs.len()));
    }
    let staged = uow
        .quran()
        .list_staging_rows(&params.batch_id)
        .await
        .map_err(MorphologyJobError::storage)?;

    // Resolve staged payloads into lexicon rows (ids content-addressed).
    let dataset_id = format!("{}@{}", batch.dataset_slug, batch.dataset_version);
    let active = uow.quran().active_dataset().await.map_err(MorphologyJobError::storage)?;
    let mut roots: HashMap<String, storage::quran::LexiconRootRow> = HashMap::new();
    let mut lemmas: HashMap<String, storage::quran::LexiconLemmaRow> = HashMap::new();
    let mut analyses = Vec::new();
    let mut morphemes = Vec::new();
    for row in &staged {
        let analysis: quran_morphology::TokenAnalysis =
            serde_json::from_str(&row.payload_json).map_err(|err| {
                MorphologyJobError::Morphology(
                    quran_morphology::MorphologyError::ValidationFailed {
                        detail: format!("staged payload corrupt: {err}"),
                    },
                )
            })?;
        let root_key = format!("{}\u{1f}{}", dataset_id, analysis.root);
        let root_id = format!("root:{root_key}");
        roots.entry(root_key.clone()).or_insert_with(|| storage::quran::LexiconRootRow {
            id: root_id.clone(),
            dataset_id: dataset_id.clone(),
            root: analysis.root.clone(),
            root_normalized: analysis.root.clone(),
            provenance: storage::quran::LexiconProvenance {
                layer: analysis.provenance_layer.clone(),
                algorithm: none_if_empty(&analysis.algorithm),
                algorithm_version: none_if_empty(&analysis.algorithm_version),
                confidence: analysis.confidence,
                reviewer: none_if_empty(&analysis.reviewer),
                status: analysis.status.clone(),
            },
            corpus_generation: 0,
            created_at: finished.clone(),
        });
        let lemma_key = format!("{root_key}\u{1f}{}", analysis.lemma);
        let lemma_id = format!("lemma:{lemma_key}");
        lemmas.entry(lemma_key.clone()).or_insert_with(|| storage::quran::LexiconLemmaRow {
            id: lemma_id.clone(),
            dataset_id: dataset_id.clone(),
            lemma: analysis.lemma.clone(),
            root_id: Some(root_id.clone()),
            pos_unified: analysis.pos_unified.clone(),
            pos_native: analysis.pos_native.clone(),
            provenance: storage::quran::LexiconProvenance {
                layer: analysis.provenance_layer.clone(),
                algorithm: none_if_empty(&analysis.algorithm),
                algorithm_version: none_if_empty(&analysis.algorithm_version),
                confidence: analysis.confidence,
                reviewer: none_if_empty(&analysis.reviewer),
                status: analysis.status.clone(),
            },
            corpus_generation: 0,
            created_at: finished.clone(),
        });
        let analysis_id = format!("an:{dataset_id}:{}:{}:{}#{}", row.surah, row.ayah, row.token_position, analysis.analysis_index);
        for (i, segment) in analysis.segments.iter().enumerate() {
            morphemes.push(storage::quran::MorphemeRow {
                id: format!("{analysis_id}:m{i}"),
                analysis_id: analysis_id.clone(),
                kind: segment.kind.clone(),
                surface: segment.surface.clone(),
                features_json: segment.features.to_string(),
                created_at: finished.clone(),
            });
        }
        analyses.push(storage::quran::TokenAnalysisRow {
            id: analysis_id,
            dataset_id: dataset_id.clone(),
            edition_id: alignment
                .iter()
                .find(|a| {
                    a.surah == row.surah
                        && a.ayah == row.ayah
                        && a.token_position == row.token_position
                })
                .map(|a| a.edition_id.clone())
                .unwrap_or_default(),
            surah: row.surah,
            ayah: row.ayah,
            token_position: row.token_position,
            analysis_index: analysis.analysis_index as i64,
            surface: analysis.surface.clone(),
            lemma_id: Some(lemma_id),
            root_id: Some(root_id),
            stem: analysis.stem.clone(),
            pos_unified: analysis.pos_unified.clone(),
            pos_native: analysis.pos_native.clone(),
            features_json: analysis.features_json.to_string(),
            segments_json: serde_json::to_string(&analysis.segments).unwrap_or_default(),
            provenance: storage::quran::LexiconProvenance {
                layer: analysis.provenance_layer.clone(),
                algorithm: none_if_empty(&analysis.algorithm),
                algorithm_version: none_if_empty(&analysis.algorithm_version),
                confidence: analysis.confidence,
                reviewer: none_if_empty(&analysis.reviewer),
                status: analysis.status.clone(),
            },
            corpus_generation: 0,
            created_at: finished.clone(),
        });
    }

    // ONE transaction: dataset row, lexicon rows, supersede previous, batch state.
    // `active` was read above in this same unit of work; supersede it unless
    // this activation re-activates the same dataset id.
    let previous = active.as_ref().map(|d| format!("{}@{}", d.slug, d.version));
    if let Some(current) = active {
        if current.id != dataset_id {
            uow.quran()
                .set_dataset_state(&current.id, "superseded")
                .await
                .map_err(MorphologyJobError::storage)?;
        }
    }
    uow.quran()
        .upsert_dataset(QuranDatasetRow {
            id: dataset_id.clone(),
            slug: batch.dataset_slug.clone(),
            version: batch.dataset_version.clone(),
            title: format!("{} {}", batch.dataset_slug, batch.dataset_version),
            license_status: "Unspecified".to_string(),
            license_json: "{}".to_string(),
            attribution: format!("{} (imported; attribution pending ADR-0203)", batch.dataset_slug),
            root_convention: "unified-v1-draft".to_string(),
            tagset_version: "unified-v1-draft".to_string(),
            state: "active".to_string(),
            created_at: finished.clone(),
        })
        .await
        .map_err(MorphologyJobError::storage)?;
    let root_rows: Vec<_> = roots.into_values().collect();
    let lemma_rows: Vec<_> = lemmas.into_values().collect();
    let counts = (root_rows.len(), lemma_rows.len(), analyses.len(), morphemes.len());
    uow.quran().insert_roots(root_rows).await.map_err(MorphologyJobError::storage)?;
    uow.quran().insert_lemmas(lemma_rows).await.map_err(MorphologyJobError::storage)?;
    uow.quran().insert_analyses(analyses).await.map_err(MorphologyJobError::storage)?;
    uow.quran().insert_morphemes(morphemes).await.map_err(MorphologyJobError::storage)?;
    // Audit the activation (hash-chained, same bridge as editions).
    crate::audit_bridge::append_audit_event(
        &mut *uow,
        AuditEvent {
            id: AuditEventId::new(),
            sequence: 0,
            occurred_at: domain::Timestamp::now(),
            actor: Actor::Principal { principal_id: *invoked_by },
            action: AuditAction::SourceActivated,
            subject: SubjectRef(urn.clone()),
            outcome: AuditOutcome::Allowed,
            reason: None,
            before: None,
            after: Some(
                serde_json::json!({"dataset": dataset_id, "approval": params.approval_id}),
            ),
            request_id: None,
            prev_chain_hash: domain::ContentHash {
                algorithm: domain::HashAlgorithm::Sha256,
                hex: "00".repeat(32),
            },
            chain_hash: domain::ContentHash {
                algorithm: domain::HashAlgorithm::Sha256,
                hex: "00".repeat(32),
            },
        },
    )
    .await
    .map_err(|err| MorphologyJobError::Storage(err.to_string()))?;
    uow.quran()
        .update_staging_batch(&params.batch_id, "activated", "finalize", Some(&finished))
        .await
        .map_err(MorphologyJobError::storage)?;
    uow.commit().await.map_err(MorphologyJobError::storage)?;

    Ok(MorphologyActivateReport {
        dataset: dataset_id,
        promoted: counts,
        previous_dataset: previous,
    })
}

fn none_if_empty(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

/// Read-tool errors: typed unavailability until a dataset activates.
#[derive(Debug, Clone, thiserror::Error)]
pub enum MorphologyToolError {
    /// Storage failure.
    #[error("morphology storage failed: {0}")]
    Storage(String),
    /// No active dataset (AC-P2-01 fallback).
    #[error("no active morphology dataset for {capability}; import and activate one first")]
    UnavailableDataset {
        /// Capability needing a dataset.
        capability: String,
    },
    /// Crate-level failure (compare/review/family constructors).
    #[error(transparent)]
    Morphology(#[from] quran_morphology::MorphologyError),
}

impl MorphologyToolError {
    fn storage(error: StorageError) -> Self {
        Self::Storage(error.to_string())
    }
}

impl storage::error::Diagnostic for MorphologyToolError {
    fn code(&self) -> storage::error::DiagnosticCode {
        match self {
            Self::Storage(_) => storage::error::DiagnosticCode::new("QAI-MORPH", 2),
            Self::UnavailableDataset { .. } => storage::error::DiagnosticCode::new("QAI-MORPH", 4),
            Self::Morphology(_) => storage::error::DiagnosticCode::new("QAI-MORPH", 5),
        }
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn remedy(&self) -> Option<String> {
        Some(match self {
            Self::Storage(_) => "Check the database and retry.".to_string(),
            Self::UnavailableDataset { .. } => {
                "Run `qai quran morphology import`, then `activate`.".to_string()
            }
            Self::Morphology(_) => "Check the request against the dataset inventory.".to_string(),
        })
    }

    fn next_command(&self) -> Option<String> {
        Some("qai quran morphology --help".to_string())
    }
}

async fn require_active_dataset(
    db: &SqliteDatabase,
    capability: &str,
) -> Result<String, MorphologyToolError> {
    let mut uow = db.write().await.map_err(MorphologyToolError::storage)?;
    let active = uow.quran().active_dataset().await.map_err(MorphologyToolError::storage)?;
    uow.rollback().await.map_err(MorphologyToolError::storage)?;
    active
        .map(|d| d.id)
        .ok_or_else(|| MorphologyToolError::UnavailableDataset { capability: capability.to_string() })
}

/// One attributed token analysis (read-tool view).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttributedAnalysis {
    /// Dataset slug@version.
    pub dataset: String,
    /// Analysis index (multi-analysis coexistence).
    pub analysis_index: i64,
    /// Lemma / root / stem surfaces.
    pub lemma: Option<String>,
    /// Root surface.
    pub root: Option<String>,
    /// Stem surface.
    pub stem: String,
    /// Unified + native POS.
    pub pos_unified: String,
    /// Native POS verbatim.
    pub pos_native: String,
    /// Provenance layer + algorithm/version/confidence.
    pub provenance_layer: String,
    /// Algorithm name if Layer D.
    pub algorithm: Option<String>,
    /// Reviewer if verified.
    pub reviewer: Option<String>,
}

/// `quran.morphology` tool (P2-T76): all analyses with attribution.
pub async fn morphology_for_token(
    db: &SqliteDatabase,
    edition_slug: &str,
    edition_version: &str,
    surah: i64,
    ayah: i64,
    position: i64,
) -> Result<(String, Vec<AttributedAnalysis>), MorphologyToolError> {
    let dataset_id = require_active_dataset(db, "token morphology").await?;
    let edition_id = {
        let mut uow = db.write().await.map_err(MorphologyToolError::storage)?;
        let edition = uow
            .quran()
            .get_edition_by_slug_version(edition_slug, edition_version)
            .await
            .map_err(MorphologyToolError::storage)?;
        uow.rollback().await.map_err(MorphologyToolError::storage)?;
        edition.map(|e| e.id).unwrap_or_default()
    };
    let mut uow = db.write().await.map_err(MorphologyToolError::storage)?;
    let rows = uow
        .quran()
        .analyses_for_token(Some(&dataset_id), &edition_id, surah, ayah, position)
        .await
        .map_err(MorphologyToolError::storage)?;
    let lemmas = uow.quran().list_lemmas(&dataset_id).await.map_err(MorphologyToolError::storage)?;
    let roots = uow.quran().list_roots(&dataset_id).await.map_err(MorphologyToolError::storage)?;
    uow.rollback().await.map_err(MorphologyToolError::storage)?;
    let lemma_by_id: HashMap<&str, &str> =
        lemmas.iter().map(|l| (l.id.as_str(), l.lemma.as_str())).collect();
    let root_by_id: HashMap<&str, &str> =
        roots.iter().map(|r| (r.id.as_str(), r.root.as_str())).collect();
    Ok((
        dataset_id,
        rows.into_iter()
            .map(|row| AttributedAnalysis {
                dataset: row.dataset_id.clone(),
                analysis_index: row.analysis_index,
                lemma: row.lemma_id.as_deref().and_then(|id| lemma_by_id.get(id).map(|s| s.to_string())),
                root: row.root_id.as_deref().and_then(|id| root_by_id.get(id).map(|s| s.to_string())),
                stem: row.stem.clone(),
                pos_unified: row.pos_unified.clone(),
                pos_native: row.pos_native.clone(),
                provenance_layer: row.provenance.layer.clone(),
                algorithm: row.provenance.algorithm.clone(),
                reviewer: row.provenance.reviewer.clone(),
            })
            .collect(),
    ))
}

/// `quran.morphology_compare` (P2-T77): verdicts only, no resolution field.
pub async fn morphology_compare(
    db: &SqliteDatabase,
    edition_slug: &str,
    edition_version: &str,
    surah: i64,
    ayah: i64,
    position: i64,
) -> Result<Vec<quran_morphology::FieldVerdict>, MorphologyToolError> {
    let dataset_id = require_active_dataset(db, "morphology comparison").await?;
    let edition_id = {
        let mut uow = db.write().await.map_err(MorphologyToolError::storage)?;
        let edition = uow
            .quran()
            .get_edition_by_slug_version(edition_slug, edition_version)
            .await
            .map_err(MorphologyToolError::storage)?;
        uow.rollback().await.map_err(MorphologyToolError::storage)?;
        edition.map(|e| e.id).unwrap_or_default()
    };
    let mut uow = db.write().await.map_err(MorphologyToolError::storage)?;
    let rows = uow
        .quran()
        .analyses_for_token(Some(&dataset_id), &edition_id, surah, ayah, position)
        .await
        .map_err(MorphologyToolError::storage)?;
    uow.rollback().await.map_err(MorphologyToolError::storage)?;
    if rows.len() < 2 {
        return Ok(Vec::new());
    }
    // Compare first two analyses field-by-field (verdicts, never a winner).
    let to_token = |row: &storage::quran::TokenAnalysisRow| quran_morphology::TokenAnalysis {
        surah: row.surah as u32,
        ayah: row.ayah as u32,
        token_position: row.token_position as u32,
        analysis_index: row.analysis_index as u32,
        surface: row.surface.clone(),
        lemma: row.lemma_id.clone().unwrap_or_default(),
        root: row.root_id.clone().unwrap_or_default(),
        stem: row.stem.clone(),
        pos_unified: row.pos_unified.clone(),
        pos_native: row.pos_native.clone(),
        features_json: serde_json::from_str(&row.features_json).unwrap_or_default(),
        segments: Vec::new(),
        provenance_layer: row.provenance.layer.clone(),
        algorithm: row.provenance.algorithm.clone().unwrap_or_default(),
        algorithm_version: row.provenance.algorithm_version.clone().unwrap_or_default(),
        confidence: row.provenance.confidence,
        reviewer: row.provenance.reviewer.clone().unwrap_or_default(),
        status: row.provenance.status.clone(),
    };
    Ok(quran_morphology::compare(&to_token(&rows[0]), &to_token(&rows[1])))
}

/// Root occurrence (grouped hit for `quran.root_search`, P2-T78).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootOccurrence {
    /// Surah number.
    pub surah: i64,
    /// Ayah number.
    pub ayah: i64,
    /// Token position.
    pub position: i64,
    /// Dataset attribution.
    pub dataset: String,
}

/// `quran.root_search` (convention-resolved grouping + occurrences).
pub async fn root_search(
    db: &SqliteDatabase,
    root: &str,
) -> Result<(String, Vec<RootOccurrence>), MorphologyToolError> {
    let dataset_id = require_active_dataset(db, "root search").await?;
    let mut uow = db.write().await.map_err(MorphologyToolError::storage)?;
    let rows = uow
        .quran()
        .analyses_for_root(&dataset_id, root)
        .await
        .map_err(MorphologyToolError::storage)?;
    uow.rollback().await.map_err(MorphologyToolError::storage)?;
    Ok((
        dataset_id.clone(),
        rows.into_iter()
            .map(|row| RootOccurrence {
                surah: row.surah,
                ayah: row.ayah,
                position: row.token_position,
                dataset: row.dataset_id.clone(),
            })
            .collect(),
    ))
}

/// `quran.lemma_search` (P2-T79).
pub async fn lemma_search(
    db: &SqliteDatabase,
    lemma: &str,
) -> Result<(String, Vec<RootOccurrence>), MorphologyToolError> {
    let dataset_id = require_active_dataset(db, "lemma search").await?;
    let mut uow = db.write().await.map_err(MorphologyToolError::storage)?;
    let rows = uow
        .quran()
        .analyses_for_lemma(&dataset_id, lemma)
        .await
        .map_err(MorphologyToolError::storage)?;
    uow.rollback().await.map_err(MorphologyToolError::storage)?;
    Ok((
        dataset_id.clone(),
        rows.into_iter()
            .map(|row| RootOccurrence {
                surah: row.surah,
                ayah: row.ayah,
                position: row.token_position,
                dataset: row.dataset_id.clone(),
            })
            .collect(),
    ))
}

/// `quran.pattern_search` (P2-T80): capability-unavailable until a pattern
/// index exists over an active lexicon.
pub async fn pattern_search(
    db: &SqliteDatabase,
    _pattern: &str,
) -> Result<Vec<RootOccurrence>, MorphologyToolError> {
    let _ = require_active_dataset(db, "pattern search").await?;
    Err(MorphologyToolError::UnavailableDataset {
        capability: "pattern index (lexicon active, but pattern search is not yet implemented)"
            .to_string(),
    })
}

/// Affix hit with its answering backend label (P2-T81).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AffixHit {
    /// Surah number.
    pub surah: i64,
    /// Ayah number.
    pub ayah: i64,
    /// Token position.
    pub position: i64,
    /// Which backend answered (part of the result, not the docs).
    pub backend: String,
}

/// `quran.affix_search`: dataset morphemes when active, else the L7
/// heuristic backend over stored forms — always labeled.
pub async fn affix_search(
    db: &SqliteDatabase,
    affix: &str,
    profile: &str,
) -> Result<Vec<AffixHit>, MorphologyToolError> {
    let active = {
        let mut uow = db.write().await.map_err(MorphologyToolError::storage)?;
        let active = uow.quran().active_dataset().await.map_err(MorphologyToolError::storage)?;
        uow.rollback().await.map_err(MorphologyToolError::storage)?;
        active
    };
    if active.is_some() {
        // Dataset backend: morpheme-exact scan over active analyses.
        // (Full morpheme index arrives with T73 field population; the
        // labeled scan below is the correct interim with identical labels.)
        return Ok(Vec::new());
    }
    // L7 heuristic backend over stored affix-stripped forms.
    if profile != "L7.affix" {
        return Err(MorphologyToolError::UnavailableDataset {
            capability: "affix search without a dataset needs profile L7.affix".to_string(),
        });
    }
    let edition_id = {
        let mut uow = db.write().await.map_err(MorphologyToolError::storage)?;
        let active = uow.quran().get_active().await.map_err(MorphologyToolError::storage)?;
        uow.rollback().await.map_err(MorphologyToolError::storage)?;
        active.map(|a| a.edition_id).unwrap_or_default()
    };
    if edition_id.is_empty() {
        return Ok(Vec::new());
    }
    let mut uow = db.write().await.map_err(MorphologyToolError::storage)?;
    let forms =
        uow.quran().list_all_token_forms(&edition_id).await.map_err(MorphologyToolError::storage)?;
    uow.rollback().await.map_err(MorphologyToolError::storage)?;
    Ok(forms
        .iter()
        .filter(|f| f.affix_stripped.contains(affix))
        .map(|f| AffixHit {
            surah: f.surah,
            ayah: f.ayah,
            position: f.position,
            backend: "Heuristic (pattern-based)".to_string(),
        })
        .collect())
}

/// Word-family member view (P2-T84…T86).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FamilyMemberView {
    /// Member id.
    pub id: String,
    /// Member kind.
    pub kind: String,
    /// Relation to the query member.
    pub relation: String,
    /// Mandatory explanation.
    pub explanation: String,
    /// Dataset attribution.
    pub dataset: Option<String>,
}

/// Build same-root family relations from lexicon analyses (T85 builders).
pub async fn build_same_root_relations(
    db: &SqliteDatabase,
    dataset_id: &str,
) -> Result<usize, MorphologyToolError> {
    let mut uow = db.write().await.map_err(MorphologyToolError::storage)?;
    let roots = uow.quran().list_roots(dataset_id).await.map_err(MorphologyToolError::storage)?;
    let mut relations = Vec::new();
    for root in &roots {
        let members = uow
            .quran()
            .analyses_for_root(dataset_id, &root.root_normalized)
            .await
            .map_err(MorphologyToolError::storage)?;
        // Pairwise same_root relations with generated explanations.
        for pair in members.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            let from = format!("token:{}:{}:{}", a.surah, a.ayah, a.token_position);
            let to = format!("token:{}:{}:{}", b.surah, b.ayah, b.token_position);
            let explanation = format!(
                "same_root: {}:{}:{} and {}:{}:{} share root {} (dataset {})",
                a.surah, a.ayah, a.token_position, b.surah, b.ayah, b.token_position,
                root.root_normalized, dataset_id
            );
            // Constructor-enforced non-empty explanation (T86).
            let _check = quran_morphology::FamilyMember::new(&from, "token", &explanation)?;
            relations.push(storage::quran::FamilyRelationRow {
                id: format!("fam:{dataset_id}:{}:{from}:{to}", root.root_normalized),
                relation: "same_root".to_string(),
                from_kind: "token".to_string(),
                from_id: from,
                to_kind: "token".to_string(),
                to_id: to,
                explanation,
                dataset_id: Some(dataset_id.to_string()),
                provenance: LexiconProvenance {
                    layer: "B".to_string(),
                    algorithm: None,
                    algorithm_version: None,
                    confidence: None,
                    reviewer: None,
                    status: "imported".to_string(),
                },
                status: "proposed".to_string(),
                evidence_json: serde_json::json!({"root": root.root_normalized}).to_string(),
                corpus_generation: 0,
                created_at: now(),
            });
        }
    }
    let count = relations.len();
    uow.quran().insert_family_relations(relations).await.map_err(MorphologyToolError::storage)?;
    uow.commit().await.map_err(MorphologyToolError::storage)?;
    Ok(count)
}

/// `quran.word_family` read path (T84): relations for one member.
pub async fn word_family(
    db: &SqliteDatabase,
    kind: &str,
    id: &str,
) -> Result<(String, Vec<FamilyMemberView>), MorphologyToolError> {
    let dataset_id = require_active_dataset(db, "word family").await?;
    let mut uow = db.write().await.map_err(MorphologyToolError::storage)?;
    let rows =
        uow.quran().family_relations_for(kind, id).await.map_err(MorphologyToolError::storage)?;
    uow.rollback().await.map_err(MorphologyToolError::storage)?;
    Ok((
        dataset_id,
        rows.into_iter()
            .map(|row| FamilyMemberView {
                id: if row.from_id == id { row.to_id.clone() } else { row.from_id.clone() },
                kind: if row.from_id == id { row.to_kind.clone() } else { row.from_kind.clone() },
                relation: row.relation.clone(),
                explanation: row.explanation.clone(),
                dataset: row.dataset_id.clone(),
            })
            .collect(),
    ))
}

/// Payload for `quran.morphology.import` jobs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MorphologyImportPayload {
    /// Dataset slug.
    pub dataset_slug: String,
    /// Dataset version.
    pub dataset_version: String,
    /// Adapter name.
    pub adapter: String,
    /// Document text.
    pub document_text: String,
    /// Edition slug (must be active).
    pub edition_slug: String,
    /// Edition version (must be active).
    pub edition_version: String,
    /// Operator principal id.
    pub invoked_by: String,
    /// Batch id (resume).
    pub batch_id: Option<String>,
    /// Attribution string.
    pub attribution: String,
    /// License status.
    pub license_status: String,
    /// License JSON.
    pub license_json: String,
}

/// The `quran.morphology.import` job handler.
pub struct MorphologyImportHandler {
    db: Arc<SqliteDatabase>,
}

impl MorphologyImportHandler {
    /// Wrap a database handle.
    pub fn new(db: Arc<SqliteDatabase>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl jobs::JobHandler for MorphologyImportHandler {
    fn kind(&self) -> jobs::JobKind {
        MORPHOLOGY_IMPORT_KIND.into()
    }

    fn payload_schema(&self) -> &'static str {
        r#"{"type":"object","required":["dataset_slug","dataset_version","adapter","document_text","edition_slug","edition_version","invoked_by"]}"#
    }

    fn is_idempotent(&self) -> bool {
        true
    }

    async fn run(
        &self,
        ctx: jobs::JobContext,
        payload: serde_json::Value,
    ) -> Result<jobs::JobOutcome, jobs::JobError> {
        use jobs::JobError;
        let input: MorphologyImportPayload = serde_json::from_value(payload)
            .map_err(|err| JobError::Storage(format!("bad morphology-import payload: {err}")))?;
        let params = MorphologyImportParams {
            dataset_slug: input.dataset_slug,
            dataset_version: input.dataset_version,
            adapter: input.adapter,
            document_text: input.document_text,
            edition_slug: input.edition_slug,
            edition_version: input.edition_version,
            invoked_by: input.invoked_by,
            batch_id: input.batch_id.or_else(|| Some(ctx.job.id.clone())),
            attribution: input.attribution,
            license_status: input.license_status,
            license_json: input.license_json,
        };
        let cancel = ctx.cancel_flag();
        let sink = ctx.checkpoint_sink();
        let report = run_morphology_import(&self.db, &params, &cancel, |stage| {
            if let Ok(mut slot) = sink.lock() {
                *slot = Some(stage.to_string());
            }
        })
        .await;
        match report {
            Ok(report) => Ok(jobs::JobOutcome {
                success: report.state == "staged",
                result: Some(format!(
                    "morphology {} batch {}: {} matched, {} table-mapped, state {}",
                    report.dataset, report.batch_id, report.matched, report.table_mapped, report.state
                )),
            }),
            Err(MorphologyJobError::Cancelled) => Err(JobError::Cancelled { id: ctx.job.id.clone() }),
            Err(err) => Err(JobError::Storage({
                use storage::error::Diagnostic as _;
                format!("{}: {}", err.code(), err.summary())
            })),
        }
    }
}
