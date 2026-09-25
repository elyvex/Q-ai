//! Canonical importer: the §34 pipeline as 13 checkpoints (D1.3, P1-T25).
//!
//! # quran_corpus::import
//!
//! The importer moves a validated edition into staging and leaves it `Staged`
//! for human approval. It holds no `ApprovalToken` and literally cannot
//! activate (I5/I7): activation is a separate, human-gated transaction.
//!
//! Crash and retry model: every step is deterministic in `(run_id, manifest)`.
//! A run always starts by cleaning its own staging rows, so a retry after a
//! crash is a clean restart — restart *is* resume here. Nothing before the
//! activation transaction (which this module never calls) touches canonical
//! tables, so a kill at any checkpoint leaves the active edition unchanged.
//!
//! Checkpoints exist for progress, cancellation, dry-run (`stop_after`), and
//! the crash matrix — not as transaction boundaries. Compute checkpoints
//! re-run on retry; only staging writes persist.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};

use domain::ContentHash;
use quran_core::enums::{
    BasmalaPolicy, NumberingScheme, RevelationPlace, SajdahKind, Script, UnicodeForm,
};
use quran_core::{EditionStatistics, grapheme_count};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::adapters::{EditionAdapter, JsonAdapter};
use crate::differ::{DIFFER_NAME, DIFFER_VERSION, diff_ayahs};
use crate::error::CorpusError;
use crate::format::EditionSource;
use crate::hashing::{AyahLayout, TokenOrder, structure_hash, tagged, text_hash, token_order_hash};
use crate::tokenize::{TokenizedAyah, reconstruct, tokenize};
use crate::unicode::{find_forbidden, normalization_form};
use crate::validation::{Finding, Outcome, Severity, ValidationReport, validate_edition};
use storage::Database;
use storage::error::StorageError;
use storage::quran::{
    AyahRow, DifferenceReportRow, DivisionRow, ImportRunRow, QuranEditionRow, SeparatorRow,
    SurahRow, TokenRow, ValidationReportRow,
};
use storage::repository::ProvenanceRecord;

/// One of the 13 §34 checkpoints, in pipeline order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportCheckpoint {
    /// Source version claimed; import run claimed (idempotent restart).
    Claimed,
    /// Manifest bytes hashed and compared against the declared hash.
    ManifestHashed,
    /// Adapter selected for the dataset shape.
    AdapterSelected,
    /// Manifest parsed into the intermediate format.
    Parsed,
    /// Unicode pre-audit scan complete (findings reported at `Validated`).
    UnicodeAudited,
    /// Full validation complete with zero `Fatal` findings.
    Validated,
    /// Every ayah tokenized with separators and offsets.
    Tokenized,
    /// `text_hash` / `structure_hash` / `token_order_hash` computed.
    HashesComputed,
    /// All rows written to staging.
    Staged,
    /// Staging read back: reconstruction and hashes verified.
    RoundtripVerified,
    /// Reference-corpus comparison recorded (skipped when unconfigured).
    ReferenceCompared,
    /// Difference report against the active edition persisted.
    Diffed,
    /// Run marked `Staged`; terminal state for the import job.
    ApprovalRequested,
}

impl ImportCheckpoint {
    /// All checkpoints in pipeline order.
    pub const ALL: [Self; 13] = [
        Self::Claimed,
        Self::ManifestHashed,
        Self::AdapterSelected,
        Self::Parsed,
        Self::UnicodeAudited,
        Self::Validated,
        Self::Tokenized,
        Self::HashesComputed,
        Self::Staged,
        Self::RoundtripVerified,
        Self::ReferenceCompared,
        Self::Diffed,
        Self::ApprovalRequested,
    ];

    /// The §34 short name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Claimed => "claimed",
            Self::ManifestHashed => "hashed",
            Self::AdapterSelected => "adapter",
            Self::Parsed => "parsed",
            Self::UnicodeAudited => "unicode",
            Self::Validated => "validated",
            Self::Tokenized => "tokenized",
            Self::HashesComputed => "hashed2",
            Self::Staged => "staged",
            Self::RoundtripVerified => "roundtrip",
            Self::ReferenceCompared => "compared",
            Self::Diffed => "diffed",
            Self::ApprovalRequested => "approval-requested",
        }
    }
}

/// Input to one import run. The job payload deserializes into this.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInput {
    /// Unique run id (fresh UUID per attempt series; retries reuse it).
    pub run_id: String,
    /// Importing job id, if run as a job.
    pub job_id: Option<String>,
    /// Source version the bytes came from.
    pub source_version_id: String,
    /// Adapter name (`json` in the job path v1).
    pub adapter: String,
    /// Raw manifest bytes.
    pub manifest_text: String,
    /// CLI-computed SHA-256 of the manifest, hex (QV-013).
    pub declared_manifest_hash: Option<String>,
    /// Invoking principal, recorded on provenance rows.
    pub invoked_by: String,
    /// License status snapshot from the source catalog.
    pub license_status: String,
    /// License record JSON snapshot from the source catalog.
    pub license_json: String,
    /// RFC 3339 timestamp used for every row this run writes.
    pub created_at: String,
}

/// Import options.
#[derive(Debug, Clone, Default)]
pub struct ImportOptions {
    /// Stop after recording this checkpoint (dry-run / chaos support).
    /// `None` runs the full pipeline.
    pub stop_after: Option<ImportCheckpoint>,
    /// Independent reference corpus snapshot for the QV-015 comparison
    /// (ADR-0114). `None` records the explicit skip; `Some` forces the
    /// exact byte-for-byte comparison and the run fails closed on any
    /// mismatch, missing ayah, or incompatible edition policy.
    ///
    /// Tier model (OD-03, ADR-0114): supplying `Some` here is **Tier-1**
    /// verification — a pinned-artifact exact diff plus the
    /// `reference_text_hash` digest check in [`compared()`]. **Tier-2** (an
    /// independent second corpus from a separate editorial chain, signer ≠
    /// OD-02 reviewer per A1) is pending owner selection (A2 shortlist); until
    /// it lands, a green run closes QV-015 as `passed_tier1_only` with an
    /// explicit exit-gate flag — never a silent pass.
    pub reference: Option<EditionSource>,
}

/// Tier-1 reference comparison: exact, order-independent, fail-closed
/// (ADR-0114 §4, OD-03).
///
/// Compares every ayah of `source` against `reference` byte-for-byte. Any text
/// difference, missing or duplicate ayah, empty corpus, or incompatible
/// edition policy (script, qiraah, riwayah, numbering, basmala) is `Fatal`.
/// With `None`, emits the single `Info` skip finding — the `passed_tier1_only`
/// outcome — and never passes silently. Cross-reading differences are edition
/// comparisons, not corruption signals: incompatible policies fail closed here
/// so they cannot be mistaken for integrity results.
pub fn compare_reference(
    source: &EditionSource,
    reference: Option<&EditionSource>,
) -> Vec<Finding> {
    let Some(reference) = reference else {
        return vec![Finding::new(
            "QV-015",
            Severity::Info,
            "edition",
            "reference-corpus comparison skipped: no reference corpus configured",
        )];
    };
    let mut findings = Vec::new();
    if source.edition.script != reference.edition.script
        || source.edition.riwayah != reference.edition.riwayah
        || source.edition.qiraah != reference.edition.qiraah
        || source.edition.verse_numbering_scheme != reference.edition.verse_numbering_scheme
        || source.edition.basmala_policy != reference.edition.basmala_policy
    {
        findings.push(Finding::new(
            "QV-015",
            Severity::Fatal,
            "edition",
            "reference edition has incompatible script, reading, numbering, or basmala policy",
        ));
    }
    let reference_surahs: BTreeMap<_, _> =
        reference.surahs.iter().map(|surah| (surah.number, surah)).collect();
    for surah in &source.surahs {
        if let Some(expected) = reference_surahs.get(&surah.number)
            && surah.basmala != expected.basmala
        {
            findings.push(Finding::new(
                "QV-015",
                Severity::Fatal,
                format!("surah {}", surah.number),
                "reference surah has incompatible basmala policy",
            ));
        }
    }
    let mut maps = Vec::new();
    for (label, edition) in [("import", source), ("reference", reference)] {
        let mut map = BTreeMap::new();
        if edition.ayahs.is_empty() {
            findings.push(Finding::new(
                "QV-015",
                Severity::Fatal,
                label,
                "cannot compare an empty corpus",
            ));
        }
        for ayah in &edition.ayahs {
            let key = (ayah.surah, ayah.ayah);
            if map.insert(key, ayah.text.as_str()).is_some() {
                findings.push(Finding::new(
                    "QV-015",
                    Severity::Fatal,
                    format!("{label} surah {} ayah {}", key.0, key.1),
                    "duplicate ayah identifier in comparison input",
                ));
            }
        }
        maps.push(map);
    }
    let keys: BTreeSet<_> = maps[0].keys().chain(maps[1].keys()).copied().collect();
    for key in keys {
        let message = match (maps[0].get(&key), maps[1].get(&key)) {
            (Some(actual), Some(expected)) if actual != expected => "text differs byte-for-byte",
            (None, Some(_)) => "ayah missing from imported corpus",
            (Some(_), None) => "ayah missing from reference corpus",
            _ => continue,
        };
        findings.push(Finding::new(
            "QV-015",
            Severity::Fatal,
            format!("surah {} ayah {}", key.0, key.1),
            message,
        ));
    }
    findings
}

/// A completed import.
#[derive(Debug, Clone)]
pub struct ImportSuccess {
    /// Canonical edition row id (== run id: a bare UUID, so it maps back to
    /// the typed domain IDs).
    pub edition_id: String,
    /// Edition slug.
    pub edition_slug: String,
    /// Edition version.
    pub edition_version: String,
    /// Persisted validation report id (== run id).
    pub validation_report_id: String,
    /// Persisted difference report id.
    pub difference_report_id: String,
    /// Corpus text hash.
    pub text_hash: ContentHash,
    /// Set when `stop_after` halted the run early.
    pub stopped_at: Option<ImportCheckpoint>,
}

/// The outcome of [`run_import`].
#[derive(Debug, Clone)]
pub enum ImportOutcome {
    /// The pipeline ran to the requested end.
    Completed(ImportSuccess),
}

fn storage_err(error: StorageError) -> CorpusError {
    CorpusError::StorageFailed { detail: error.to_string() }
}

fn script_str(script: &Script) -> String {
    match script {
        Script::Uthmani => "uthmani".to_string(),
        Script::ImlaeiSimple => "imlaei_simple".to_string(),
        Script::Other(name) => format!("other:{name}"),
    }
}

fn numbering_str(scheme: &NumberingScheme) -> String {
    match scheme {
        NumberingScheme::Hafs => "hafs".to_string(),
        NumberingScheme::Kufi => "kufi".to_string(),
        NumberingScheme::Custom(name) => name.clone(),
    }
}

fn basmala_str(policy: &BasmalaPolicy) -> &'static str {
    match policy {
        BasmalaPolicy::CountedAsFirstAyah => "counted_as_first_ayah",
        BasmalaPolicy::UnnumberedHeader => "unnumbered_header",
        BasmalaPolicy::Absent => "absent",
        BasmalaPolicy::PerSurah => "per_surah",
    }
}

fn unicode_str(form: &UnicodeForm) -> &'static str {
    match form {
        UnicodeForm::Nfc => "nfc",
        UnicodeForm::Nfd => "nfd",
        UnicodeForm::Nfkc => "nfkc",
        UnicodeForm::Nfkd => "nfkd",
    }
}

fn revelation_str(place: &RevelationPlace) -> &'static str {
    match place {
        RevelationPlace::Makki => "makki",
        RevelationPlace::Madani => "madani",
    }
}

fn sajdah_str(kind: &SajdahKind) -> &'static str {
    match kind {
        SajdahKind::Recommended => "recommended",
        SajdahKind::Obligatory => "obligatory",
    }
}

/// Owned progress sink: every recorded checkpoint is appended here.
///
/// `Send + Sync + 'static` by construction, so drivers and handlers can hold
/// it across awaits without auto-trait friction. Tests assert exact sequences.
#[derive(Debug, Clone, Default)]
pub struct ImportProgress {
    checkpoints: std::sync::Arc<std::sync::Mutex<Vec<ImportCheckpoint>>>,
}

impl ImportProgress {
    /// An empty sink.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a checkpoint.
    pub fn record(&self, checkpoint: ImportCheckpoint) {
        if let Ok(mut checkpoints) = self.checkpoints.lock() {
            checkpoints.push(checkpoint);
        }
    }

    /// All recorded checkpoints in order.
    pub fn checkpoints(&self) -> Vec<ImportCheckpoint> {
        self.checkpoints.lock().map(|guard| guard.clone()).unwrap_or_default()
    }
}

struct Driver<'a> {
    db: &'a dyn Database,
    input: &'a ImportInput,
    options: &'a ImportOptions,
    cancel: &'a AtomicBool,
    progress: ImportProgress,
    last: Option<ImportCheckpoint>,
    manifest_hash: String,
    doc: Option<EditionSource>,
    tokenized: Vec<TokenizedAyah>,
    text_hash: Option<ContentHash>,
    structure_hash: Option<ContentHash>,
    token_order_hash: Option<ContentHash>,
    edition_id: String,
    provenance_id: String,
    findings: Vec<Finding>,
    report: Option<ValidationReport>,
}

impl<'a> Driver<'a> {
    fn new(
        db: &'a dyn Database,
        input: &'a ImportInput,
        options: &'a ImportOptions,
        cancel: &'a AtomicBool,
        progress: ImportProgress,
    ) -> Self {
        let run_id = input.run_id.clone();
        Self {
            db,
            input,
            options,
            cancel,
            progress,
            last: None,
            manifest_hash: String::new(),
            doc: None,
            tokenized: Vec::new(),
            text_hash: None,
            structure_hash: None,
            token_order_hash: None,
            edition_id: run_id.clone(),
            provenance_id: run_id.clone(),
            findings: Vec::new(),
            report: None,
        }
    }

    fn run_id(&self) -> &str {
        &self.input.run_id
    }

    /// Record a checkpoint; returns true when `stop_after` halts the run.
    fn reached(&mut self, checkpoint: ImportCheckpoint) -> bool {
        self.progress.record(checkpoint);
        self.last = Some(checkpoint);
        self.options.stop_after == Some(checkpoint)
    }

    async fn check_cancel(&mut self) -> Result<(), CorpusError> {
        if self.cancel.load(Ordering::SeqCst) {
            return Err(self.cancel_run().await);
        }
        Ok(())
    }

    /// Best-effort cancellation cleanup: staging removed, run marked Cancelled.
    async fn cancel_run(&mut self) -> CorpusError {
        let run_id = self.run_id().to_string();
        if let Ok(mut uow) = self.db.write().await {
            let _ = uow.quran().clear_staging(&run_id).await;
            let _ = uow.quran().set_import_run_state(&run_id, "Cancelled").await;
            let _ = uow.commit().await;
        }
        CorpusError::ImportCancelled {
            run_id,
            checkpoint: self.last.map(ImportCheckpoint::as_str).unwrap_or("start").to_string(),
        }
    }

    async fn fail_run(&mut self, state: &str) {
        let run_id = self.run_id().to_string();
        if let Ok(mut uow) = self.db.write().await {
            let _ = uow.quran().set_import_run_state(&run_id, state).await;
            let _ = uow.commit().await;
        }
    }

    async fn claimed(&mut self) -> Result<(), CorpusError> {
        let run_id = self.run_id().to_string();
        let mut uow = self.db.write().await.map_err(storage_err)?;
        match uow.quran().get_import_run(&run_id).await.map_err(storage_err)? {
            Some(_) => {
                uow.quran().clear_staging(&run_id).await.map_err(storage_err)?;
                uow.quran().set_import_run_state(&run_id, "Running").await.map_err(storage_err)?;
            }
            None => {
                uow.quran()
                    .insert_import_run(ImportRunRow {
                        run_id: run_id.clone(),
                        job_id: self.input.job_id.clone(),
                        edition_slug: String::new(),
                        edition_version: String::new(),
                        adapter: self.input.adapter.clone(),
                        state: "Running".to_string(),
                        created_at: self.input.created_at.clone(),
                    })
                    .await
                    .map_err(storage_err)?;
            }
        }
        uow.commit().await.map_err(storage_err)?;
        Ok(())
    }

    async fn manifest_hashed(&mut self) -> Result<(), CorpusError> {
        let mut hasher = Sha256::new();
        hasher.update(self.input.manifest_text.as_bytes());
        let observed = format!("{:x}", hasher.finalize());
        if let Some(declared) = &self.input.declared_manifest_hash
            && observed != declared.to_lowercase()
        {
            self.fail_run("Failed").await;
            return Err(CorpusError::ImportFailed {
                step: "hashed",
                detail: "declared manifest hash does not match the observed bytes (QV-013)"
                    .to_string(),
            });
        }
        self.manifest_hash = format!("sha256:{observed}");
        Ok(())
    }

    async fn adapter_selected(&self) -> Result<(), CorpusError> {
        if self.input.adapter != JsonAdapter::NAME {
            return Err(CorpusError::UnknownAdapter { name: self.input.adapter.clone() });
        }
        Ok(())
    }

    async fn parsed(&mut self) -> Result<(), CorpusError> {
        let doc = JsonAdapter.parse(&self.input.manifest_text)?;
        self.doc = Some(doc);
        Ok(())
    }

    async fn unicode_audited(&mut self) -> Result<(), CorpusError> {
        // Progress marker for the unicode phase: the scan runs here so cancellation
        // and observability see it; its findings are reported by validate_edition.
        let doc = self.doc.as_ref().expect("parsed before unicode audit");
        let declared = doc.edition.unicode_normalization;
        for ayah in &doc.ayahs {
            let _ = normalization_form(&ayah.text) == Some(declared);
            let _ = find_forbidden(&ayah.text);
        }
        Ok(())
    }

    async fn validated(&mut self) -> Result<(), CorpusError> {
        let doc = self.doc.as_ref().expect("parsed before validation").clone();
        let report = validate_edition(&doc);
        if report.has_fatal() {
            let mut uow = self.db.write().await.map_err(storage_err)?;
            uow.quran()
                .insert_validation_report(validation_row(
                    &self.input.run_id,
                    &doc,
                    &report,
                    &self.input.created_at,
                ))
                .await
                .map_err(storage_err)?;
            uow.quran().set_import_run_state(self.run_id(), "Failed").await.map_err(storage_err)?;
            uow.commit().await.map_err(storage_err)?;
            return Err(CorpusError::ValidationFailed {
                run_id: self.run_id().to_string(),
                report_id: self.input.run_id.clone(),
                fatal_count: report.fatal_count,
            });
        }
        self.findings = report.findings.clone();
        self.report = Some(report);
        Ok(())
    }

    async fn tokenized(&mut self) -> Result<(), CorpusError> {
        let doc = self.doc.as_ref().expect("parsed before tokenizing");
        self.tokenized = doc.ayahs.iter().map(|ayah| tokenize(&ayah.text)).collect();
        Ok(())
    }

    async fn hashes_computed(&mut self) -> Result<(), CorpusError> {
        let doc = self.doc.as_ref().expect("parsed before hashing").clone();
        let slug = doc.edition.slug.clone();
        let version = doc.edition.version.to_string();
        let texts: Vec<&str> = doc.ayahs.iter().map(|ayah| ayah.text.as_str()).collect();
        let text_hash = text_hash(&slug, &version, &texts);
        let surahs: Vec<(u16, u16)> = doc.surahs.iter().map(|s| (s.number, s.ayah_count)).collect();
        let layouts: Vec<AyahLayout> = doc
            .ayahs
            .iter()
            .map(|ayah| AyahLayout {
                surah: ayah.surah,
                ayah: ayah.ayah,
                juz: ayah.juz,
                hizb: ayah.hizb,
                rub: ayah.rub,
                manzil: ayah.manzil,
                ruku: ayah.ruku,
                page: ayah.page,
                sajdah: ayah.sajdah,
            })
            .collect();
        let structure_hash = structure_hash(&slug, &version, &surahs, &layouts);
        let mut orders = Vec::new();
        for (ayah, computed) in doc.ayahs.iter().zip(self.tokenized.iter()) {
            for token in &computed.tokens {
                orders.push(TokenOrder {
                    surah: ayah.surah,
                    ayah: ayah.ayah,
                    position: token.position,
                    surface: token.surface.as_str(),
                });
            }
        }
        let token_order_hash = token_order_hash(&slug, &version, &orders);
        self.text_hash = Some(text_hash);
        self.structure_hash = Some(structure_hash);
        self.token_order_hash = Some(token_order_hash);
        Ok(())
    }

    async fn staged(&mut self) -> Result<(), CorpusError> {
        let doc = self.doc.as_ref().expect("parsed before staging").clone();
        let run_id = self.run_id().to_string();
        let edition_id = self.edition_id.clone();
        let provenance_id = self.provenance_id.clone();
        let created_at = self.input.created_at.clone();
        let text_hash = self.text_hash.as_ref().expect("hashes before staging").clone();
        let structure_hash = self.structure_hash.as_ref().expect("hashes before staging").clone();
        let token_order_hash =
            self.token_order_hash.as_ref().expect("hashes before staging").clone();

        // Statistics derive from validated content + computed tokens (no DB needed).
        let mut surfaces = BTreeSet::new();
        let mut juz_set = BTreeSet::new();
        let mut pages = BTreeSet::new();
        let mut sajdah_count = 0_u16;
        let mut token_total = 0_u64;
        for (ayah, computed) in doc.ayahs.iter().zip(self.tokenized.iter()) {
            token_total += computed.tokens.len() as u64;
            surfaces.extend(computed.tokens.iter().map(|token| token.surface.as_str()));
            if let Some(juz) = ayah.juz {
                juz_set.insert(juz);
            }
            if let Some(page) = ayah.page {
                pages.insert(page);
            }
            if ayah.sajdah.is_some() {
                sajdah_count += 1;
            }
        }
        let statistics_json = serde_json::to_string(&EditionStatistics {
            surah_count: doc.surahs.len() as u16,
            ayah_count: doc.ayahs.len() as u32,
            token_count: token_total,
            distinct_surface_forms: surfaces.len() as u64,
            juz_count: juz_set.len() as u16,
            page_count: Some(pages.len() as u32),
            sajdah_count,
        })
        .map_err(|err| CorpusError::ImportFailed {
            step: "staged",
            detail: format!("cannot serialize statistics: {err}"),
        })?;

        let edition = &doc.edition;
        let edition_urn = format!("quran-edition:{}@{}", edition.slug, edition.version);

        let reference_snapshot_hash = self
            .options
            .reference
            .as_ref()
            .map(crate::validation::intermediate_hash)
            .transpose()?
            .map(|hash| tagged(&hash));
        let mut uow = self.db.write().await.map_err(storage_err)?;
        let record = ProvenanceRecord {
            id: provenance_id.clone(),
            layer: "canonical_source".to_string(),
            subject_urn: edition_urn.clone(),
            attribution_kind: "dataset".to_string(),
            attribution_json: format!(
                "{{\"dataset_name\":{:?},\"dataset_version\":{:?},\"source_version_id\":{:?}}}",
                edition.slug,
                edition.version.to_string(),
                self.input.source_version_id
            ),
            source_version_id: Some(self.input.source_version_id.clone()),
            trust_level: "ImportedUnverified".to_string(),
            verification_status: "unverified".to_string(),
            confidence: None,
            versions_json: serde_json::json!({
                "source_version_id": self.input.source_version_id,
                "parser_version": "1.0.0",
                "schema_version": 1,
                "manifest_hash": self.manifest_hash,
                "reference_snapshot_hash": reference_snapshot_hash,
            })
            .to_string(),
            created_by: self.input.invoked_by.clone(),
        };
        if let Some(existing) = uow.provenance().get(&provenance_id).await.map_err(storage_err)? {
            if existing.versions_json != record.versions_json
                || existing.created_by != record.created_by
            {
                uow.quran().set_import_run_state(&run_id, "Failed").await.map_err(storage_err)?;
                uow.commit().await.map_err(storage_err)?;
                return Err(CorpusError::ImportFailed {
                    step: "staged",
                    detail: "retry input or reference changed; use a new run id".into(),
                });
            }
        } else {
            uow.provenance().insert(record).await.map_err(storage_err)?;
        }

        uow.quran().clear_staging(&run_id).await.map_err(storage_err)?;
        uow.quran()
            .insert_stg_edition(
                &run_id,
                QuranEditionRow {
                    id: edition_id.clone(),
                    slug: edition.slug.clone(),
                    version: edition.version.to_string(),
                    name: edition.name.clone(),
                    script: script_str(&edition.script),
                    riwayah: edition.riwayah.clone(),
                    qiraah: edition.qiraah.clone(),
                    publisher: edition.publisher.clone(),
                    source_url: None,
                    upstream_edition_slug: edition.upstream_edition_slug.clone(),
                    qai_edition_id: edition.qai_edition_id.clone(),
                    is_primary: edition.is_primary,
                    language: edition.language.to_string(),
                    verse_numbering_scheme: numbering_str(&edition.verse_numbering_scheme),
                    basmala_policy: basmala_str(&edition.basmala_policy).to_string(),
                    unicode_normalization: unicode_str(&edition.unicode_normalization).to_string(),
                    license_json: self.input.license_json.clone(),
                    text_hash: tagged(&text_hash),
                    structure_hash: tagged(&structure_hash),
                    token_order_hash: tagged(&token_order_hash),
                    manifest_hash: self.manifest_hash.clone(),
                    source_version_id: self.input.source_version_id.clone(),
                    statistics_json,
                    status: "Staged".to_string(),
                    imported_at: created_at.clone(),
                    verified_at: None,
                    verified_by: None,
                    verification_method: None,
                    activated_at: None,
                    deprecated_at: None,
                },
            )
            .await
            .map_err(storage_err)?;

        for surah in &doc.surahs {
            uow.quran()
                .insert_stg_surah(
                    &run_id,
                    SurahRow {
                        edition_id: edition_id.clone(),
                        number: i64::from(surah.number),
                        name_arabic: surah.name_arabic.clone(),
                        name_transliteration: surah.name_transliteration.clone(),
                        name_translations_json: "{}".to_string(),
                        ayah_count: i64::from(surah.ayah_count),
                        revelation_place: surah
                            .revelation_place
                            .map(|place| revelation_str(&place))
                            .map(str::to_string),
                        revelation_order: surah.revelation_order.map(i64::from),
                        basmala: basmala_str(&surah.basmala).to_string(),
                        ruku_count: surah.ruku_count.map(i64::from),
                        metadata_provenance_id: Some(provenance_id.clone()),
                    },
                )
                .await
                .map_err(storage_err)?;
        }

        let mut global_ayah = 0_u32;
        let mut global_token = 0_u64;
        for (ayah, computed) in doc.ayahs.iter().zip(self.tokenized.iter()) {
            global_ayah += 1;
            let mut ayah_hasher = Sha256::new();
            ayah_hasher.update(ayah.text.as_bytes());
            let ayah_hash = format!("sha256:{:x}", ayah_hasher.finalize());
            uow.quran()
                .insert_stg_ayah(
                    &run_id,
                    AyahRow {
                        edition_id: edition_id.clone(),
                        surah: i64::from(ayah.surah),
                        ayah: i64::from(ayah.ayah),
                        text: ayah.text.clone(),
                        text_hash: ayah_hash,
                        char_count: i64::from(grapheme_count(&ayah.text)),
                        token_count: computed.tokens.len() as i64,
                        global_ayah_index: i64::from(global_ayah),
                        juz: ayah.juz.map(i64::from),
                        hizb: ayah.hizb.map(i64::from),
                        rub: ayah.rub.map(i64::from),
                        manzil: ayah.manzil.map(i64::from),
                        ruku: ayah.ruku.map(i64::from),
                        page: ayah.page.map(i64::from),
                        sajdah: ayah.sajdah.map(|kind| sajdah_str(&kind)).map(str::to_string),
                        provenance_id: provenance_id.clone(),
                    },
                )
                .await
                .map_err(storage_err)?;
            for token in &computed.tokens {
                let position =
                    u16::try_from(token.position).map_err(|_| CorpusError::ImportFailed {
                        step: "staged",
                        detail: format!("token position {} exceeds u16", token.position),
                    })?;
                global_token += 1;
                let mut surface_hasher = Sha256::new();
                surface_hasher.update(token.surface.as_bytes());
                uow.quran()
                    .insert_stg_token(
                        &run_id,
                        TokenRow {
                            edition_id: edition_id.clone(),
                            surah: i64::from(ayah.surah),
                            ayah: i64::from(ayah.ayah),
                            position: i64::from(position),
                            surface: token.surface.clone(),
                            surface_hash: format!("sha256:{:x}", surface_hasher.finalize()),
                            char_start: i64::from(token.char_start),
                            char_end: i64::from(token.char_end),
                            byte_start: i64::from(token.byte_start),
                            byte_end: i64::from(token.byte_end),
                            is_pause_mark: token.is_pause_mark,
                            global_token_index: global_token as i64,
                        },
                    )
                    .await
                    .map_err(storage_err)?;
            }
            for (index, separator) in computed.separators.iter().enumerate() {
                uow.quran()
                    .insert_stg_separator(
                        &run_id,
                        SeparatorRow {
                            edition_id: edition_id.clone(),
                            surah: i64::from(ayah.surah),
                            ayah: i64::from(ayah.ayah),
                            after_position: index as i64,
                            separator: separator.clone(),
                        },
                    )
                    .await
                    .map_err(storage_err)?;
            }
        }
        for division in build_divisions(&doc, &edition_id, &provenance_id) {
            uow.quran().insert_stg_division(&run_id, division).await.map_err(storage_err)?;
        }
        uow.commit().await.map_err(storage_err)?;
        Ok(())
    }

    async fn roundtrip_verified(&mut self) -> Result<(), CorpusError> {
        let run_id = self.run_id().to_string();
        let edition_id = self.edition_id.clone();
        let expected_text = self.text_hash.as_ref().expect("hashes before roundtrip").clone();
        let expected_order =
            self.token_order_hash.as_ref().expect("hashes before roundtrip").clone();
        let doc = self.doc.as_ref().expect("parsed before roundtrip");

        let mut uow = self.db.write().await.map_err(storage_err)?;
        let staged = uow.quran().list_stg_ayahs(&run_id).await.map_err(storage_err)?;
        if staged.len() != doc.ayahs.len() {
            return Err(CorpusError::ImportFailed {
                step: "roundtrip",
                detail: format!("staged {} ayahs, expected {}", staged.len(), doc.ayahs.len()),
            });
        }
        let mut texts = Vec::with_capacity(staged.len());
        let mut order_parts: Vec<(u16, u32, u32, String)> = Vec::new();
        for row in &staged {
            let tokens = uow
                .quran()
                .list_stg_tokens(&run_id, &edition_id, row.surah, row.ayah)
                .await
                .map_err(storage_err)?;
            let separators = uow
                .quran()
                .list_stg_separators(&run_id, &edition_id, row.surah, row.ayah)
                .await
                .map_err(storage_err)?;
            let computed: Vec<crate::tokenize::ComputedToken> = tokens
                .iter()
                .map(|token| crate::tokenize::ComputedToken {
                    position: token.position as u32,
                    surface: token.surface.clone(),
                    char_start: token.char_start as u32,
                    char_end: token.char_end as u32,
                    byte_start: token.byte_start as u32,
                    byte_end: token.byte_end as u32,
                    is_pause_mark: token.is_pause_mark,
                })
                .collect();
            let separators: Vec<String> =
                separators.iter().map(|row| row.separator.clone()).collect();
            if reconstruct(&computed, &separators) != row.text {
                return Err(CorpusError::ImportFailed {
                    step: "roundtrip",
                    detail: format!(
                        "staged tokens do not reconstruct surah {} ayah {}",
                        row.surah, row.ayah
                    ),
                });
            }
            texts.push(row.text.as_str());
            for token in &tokens {
                order_parts.push((
                    row.surah as u16,
                    row.ayah as u32,
                    token.position as u32,
                    token.surface.clone(),
                ));
            }
        }
        uow.commit().await.map_err(storage_err)?;
        let orders: Vec<TokenOrder<'_>> = order_parts
            .iter()
            .map(|(surah, ayah, position, surface)| TokenOrder {
                surah: *surah,
                ayah: *ayah,
                position: *position,
                surface: surface.as_str(),
            })
            .collect();

        // QV-014 (half) + QV-024: hashes reproduce from stored rows.
        if text_hash(&doc.edition.slug, &doc.edition.version.to_string(), &texts) != expected_text {
            return Err(CorpusError::ImportFailed {
                step: "roundtrip",
                detail: "text_hash recomputed from staged rows differs (QV-014)".to_string(),
            });
        }
        if token_order_hash(&doc.edition.slug, &doc.edition.version.to_string(), &orders)
            != expected_order
        {
            return Err(CorpusError::ImportFailed {
                step: "roundtrip",
                detail: "token_order_hash recomputed from staged rows differs (QV-024)".to_string(),
            });
        }
        Ok(())
    }

    async fn compared(&mut self) -> Result<(), CorpusError> {
        let doc = self.doc.as_ref().expect("parsed before comparison");
        let mut findings = Vec::new();
        if let Some(reference) = &self.options.reference {
            let snapshot_hash = tagged(&crate::validation::intermediate_hash(reference)?);
            let mut reference = reference.clone();
            reference.ayahs.sort_by_key(|ayah| (ayah.surah, ayah.ayah));
            reference.surahs.sort_by_key(|surah| surah.number);
            if let Err(error) = reference.validate() {
                findings.push(Finding::new(
                    "QV-015",
                    Severity::Fatal,
                    "reference",
                    format!("invalid reference format: {error}"),
                ));
            }
            findings.extend(
                validate_edition(&reference)
                    .findings
                    .into_iter()
                    .filter(|finding| matches!(finding.severity, Severity::Fatal | Severity::Error))
                    .map(|finding| {
                        Finding::new(
                            "QV-015",
                            Severity::Fatal,
                            format!("reference {}", finding.location),
                            format!("invalid reference: {}: {}", finding.rule_id, finding.message),
                        )
                    }),
            );
            let reference_id = &reference.edition.slug;
            let version = reference.edition.version.to_string();
            let texts: Vec<_> = reference.ayahs.iter().map(|ayah| ayah.text.as_str()).collect();
            let hash = tagged(&text_hash(reference_id, &version, &texts));
            if let Some(expected) = &doc.expected.reference_corpus_id
                && expected != reference_id
            {
                findings.push(Finding::new(
                    "QV-015",
                    Severity::Fatal,
                    "reference",
                    format!("reference_corpus_id mismatch: expected {expected:?}, observed {reference_id:?}"),
                ));
            }
            if let Some(expected) = &doc.expected.reference_text_hash
                && expected != &hash
            {
                findings.push(Finding::new(
                    "QV-015",
                    Severity::Fatal,
                    "reference",
                    format!(
                        "reference_text_hash mismatch: expected {expected:?}, observed {hash:?}"
                    ),
                ));
            }
            findings.extend(compare_reference(doc, Some(&reference)));
            findings.push(Finding::new(
                "QV-015",
                Severity::Info,
                format!("quran-edition:{reference_id}@{version}"),
                serde_json::json!({
                    "method": "exact-ayah-bytes-v1",
                    "comparison_kind": "reference",
                    "classification_vocabulary": crate::differ::CLASSIFICATION_VOCABULARY,
                    "normalization_applied": [],
                    "reference_corpus_id": reference_id,
                    "reference_version": version,
                    "reference_text_hash": hash,
                    "reference_snapshot_hash": snapshot_hash,
                    "outcome": if findings.iter().any(|f| f.severity == Severity::Fatal) {
                        "fail"
                    } else {
                        "pass"
                    },
                })
                .to_string(),
            ));
        } else if doc.expected.reference_corpus_id.is_some()
            || doc.expected.reference_text_hash.is_some()
        {
            findings.push(Finding::new(
                "QV-015",
                Severity::Fatal,
                "reference",
                "requested reference corpus is unavailable; comparison cannot be skipped",
            ));
        } else {
            findings.extend(compare_reference(doc, None));
        }
        let failed = findings.iter().any(|finding| finding.severity == Severity::Fatal);
        self.findings.extend(findings);
        if failed {
            let row = self.findings_row()?;
            let mut uow = self.db.write().await.map_err(storage_err)?;
            let existing =
                uow.quran().get_validation_report(self.run_id()).await.map_err(storage_err)?;
            if let Some(existing) = existing {
                if existing.findings_json != row.findings_json {
                    uow.quran()
                        .set_import_run_state(self.run_id(), "Failed")
                        .await
                        .map_err(storage_err)?;
                    uow.commit().await.map_err(storage_err)?;
                    return Err(CorpusError::ImportFailed {
                        step: "reference_comparison",
                        detail: "QV-015: retry evidence changed; use a new run id".into(),
                    });
                }
            } else {
                uow.quran().insert_validation_report(row).await.map_err(storage_err)?;
            }
            uow.quran().set_import_run_state(self.run_id(), "Failed").await.map_err(storage_err)?;
            uow.commit().await.map_err(storage_err)?;
            return Err(CorpusError::ImportFailed {
                step: "reference_comparison",
                detail: "QV-015: reference comparison failed; see validation report".into(),
            });
        }
        Ok(())
    }

    fn findings_row(&self) -> Result<ValidationReportRow, CorpusError> {
        let doc = self.doc.as_ref().expect("parsed before findings");
        let fatal_count =
            self.findings.iter().filter(|f| f.severity == Severity::Fatal).count() as i64;
        let error_count =
            self.findings.iter().filter(|f| f.severity == Severity::Error).count() as i64;
        let warning_count =
            self.findings.iter().filter(|f| f.severity == Severity::Warning).count() as i64;
        Ok(ValidationReportRow {
            id: self.run_id().to_string(),
            subject_urn: format!("quran-edition:{}@{}", doc.edition.slug, doc.edition.version),
            validator: crate::validation::VALIDATOR_NAME.to_string(),
            validator_version: crate::validation::VALIDATOR_VERSION.to_string(),
            outcome: if fatal_count > 0 || error_count > 0 {
                "fail"
            } else if warning_count > 0 {
                "pass_with_warnings"
            } else {
                "pass"
            }
            .to_string(),
            fatal_count,
            error_count,
            warning_count,
            findings_json: serde_json::to_string(&self.findings).map_err(|error| {
                CorpusError::ImportFailed {
                    step: "reference_comparison",
                    detail: format!("cannot serialize findings: {error}"),
                }
            })?,
            created_at: self.input.created_at.clone(),
        })
    }

    async fn diffed(&mut self) -> Result<(), CorpusError> {
        let run_id = self.run_id().to_string();
        let doc = self.doc.as_ref().expect("parsed before diff").clone();
        let edition_urn = format!("quran-edition:{}@{}", doc.edition.slug, doc.edition.version);
        let report = self.findings_row()?;

        let mut uow = self.db.write().await.map_err(storage_err)?;
        if let Some(existing) =
            uow.quran().get_validation_report(&run_id).await.map_err(storage_err)?
        {
            if existing.findings_json == report.findings_json && existing.outcome == report.outcome
            {
                uow.commit().await.map_err(storage_err)?;
                return Ok(());
            }
            uow.quran().set_import_run_state(&run_id, "Failed").await.map_err(storage_err)?;
            uow.commit().await.map_err(storage_err)?;
            return Err(CorpusError::ImportFailed {
                step: "diffed",
                detail: "retry validation evidence changed; use a new run id".into(),
            });
        }
        let active = uow.quran().get_active().await.map_err(storage_err)?;
        let (old_list, from_version, metadata_changed) = match active {
            Some(pointer) => {
                let old_edition =
                    uow.quran().get_edition(&pointer.edition_id).await.map_err(storage_err)?;
                let old_edition = old_edition.ok_or_else(|| CorpusError::ImportFailed {
                    step: "diffed",
                    detail: "active pointer dangles".to_string(),
                })?;
                let rows = uow
                    .quran()
                    .list_ayahs_range(&pointer.edition_id, 1, i64::MAX)
                    .await
                    .map_err(storage_err)?;
                let list: Vec<(u16, u32, String)> = rows
                    .iter()
                    .map(|row| (row.surah as u16, row.ayah as u32, row.text.clone()))
                    .collect();
                let changed = old_edition.name != doc.edition.name
                    || old_edition.script != script_str(&doc.edition.script)
                    || old_edition.verse_numbering_scheme
                        != numbering_str(&doc.edition.verse_numbering_scheme);
                (list, old_edition.version, changed)
            }
            None => (Vec::new(), "none".to_string(), false),
        };
        let staged = uow.quran().list_stg_ayahs(&run_id).await.map_err(storage_err)?;
        let new_list: Vec<(u16, u32, String)> = staged
            .iter()
            .map(|row| (row.surah as u16, row.ayah as u32, row.text.clone()))
            .collect();
        let diff = diff_ayahs(&old_list, &new_list, metadata_changed);

        uow.quran()
            .insert_difference_report(DifferenceReportRow {
                id: format!("diff-{run_id}"),
                subject_urn: edition_urn.clone(),
                from_version,
                to_version: doc.edition.version.to_string(),
                differ: DIFFER_NAME.to_string(),
                differ_version: DIFFER_VERSION.to_string(),
                summary_json: serde_json::json!({
                    "ayahs_added": diff.added,
                    "ayahs_removed": diff.removed,
                    "ayahs_changed": diff.changed,
                    "ayahs_unchanged": diff.unchanged,
                    "metadata_changed": diff.metadata_changed,
                })
                .to_string(),
                details_json: serde_json::to_string(&diff.changes).map_err(|err| {
                    CorpusError::ImportFailed {
                        step: "diffed",
                        detail: format!("cannot serialize diff: {err}"),
                    }
                })?,
                created_at: self.input.created_at.clone(),
            })
            .await
            .map_err(storage_err)?;

        uow.quran().insert_validation_report(report).await.map_err(storage_err)?;
        uow.commit().await.map_err(storage_err)?;
        Ok(())
    }

    async fn approval_requested(&mut self) -> Result<ImportSuccess, CorpusError> {
        let run_id = self.run_id().to_string();
        let doc = self.doc.as_ref().expect("parsed before approval request");
        let mut uow = self.db.write().await.map_err(storage_err)?;
        uow.quran().set_import_run_state(&run_id, "Staged").await.map_err(storage_err)?;
        uow.commit().await.map_err(storage_err)?;
        Ok(ImportSuccess {
            edition_id: self.edition_id.clone(),
            edition_slug: doc.edition.slug.clone(),
            edition_version: doc.edition.version.to_string(),
            validation_report_id: run_id,
            difference_report_id: format!("diff-{}", self.input.run_id),
            text_hash: self.text_hash.clone().expect("hashes before approval request"),
            stopped_at: None,
        })
    }
}

/// Build division rows by run-length encoding per-ayah metadata.
///
/// Range kinds (`juz`…`page`) become one row per contiguous run; `sajdah`
/// markers become one row per occurrence (number = occurrence index). Ruku
/// numbers must already be globally unique (see DEV-03): per-surah ruku would
/// collide on `(edition, kind, number)`.
fn build_divisions(doc: &EditionSource, edition_id: &str, provenance_id: &str) -> Vec<DivisionRow> {
    let mut divisions = Vec::new();
    let mut global = 0_i64;
    let mut globals: Vec<i64> = Vec::with_capacity(doc.ayahs.len());
    for _ in &doc.ayahs {
        global += 1;
        globals.push(global);
    }
    let mut run: Option<(String, i64, usize)>;
    let flush =
        |divisions: &mut Vec<DivisionRow>, kind: &str, number: i64, start: usize, end: usize| {
            let first = &doc.ayahs[start];
            let last = &doc.ayahs[end];
            divisions.push(DivisionRow {
                edition_id: edition_id.to_string(),
                kind: kind.to_string(),
                number,
                start_surah: i64::from(first.surah),
                start_ayah: i64::from(first.ayah),
                end_surah: i64::from(last.surah),
                end_ayah: i64::from(last.ayah),
                start_global: globals[start],
                end_global: globals[end],
                label: None,
                provenance_id: provenance_id.to_string(),
            });
        };
    for (kind, values) in [
        ("juz", doc.ayahs.iter().map(|a| a.juz.map(i64::from)).collect::<Vec<_>>()),
        ("hizb", doc.ayahs.iter().map(|a| a.hizb.map(i64::from)).collect::<Vec<_>>()),
        ("rub", doc.ayahs.iter().map(|a| a.rub.map(i64::from)).collect::<Vec<_>>()),
        ("manzil", doc.ayahs.iter().map(|a| a.manzil.map(i64::from)).collect::<Vec<_>>()),
        ("ruku", doc.ayahs.iter().map(|a| a.ruku.map(i64::from)).collect::<Vec<_>>()),
        ("page", doc.ayahs.iter().map(|a| a.page.map(i64::from)).collect::<Vec<_>>()),
    ] {
        run = None;
        for (index, value) in values.iter().enumerate() {
            match (*value, run.take()) {
                (Some(number), Some((open_kind, open_number, start)))
                    if open_kind == kind && open_number == number =>
                {
                    run = Some((open_kind, open_number, start));
                }
                (Some(number), previous) => {
                    if let Some((open_kind, open_number, start)) = previous {
                        flush(&mut divisions, &open_kind, open_number, start, index - 1);
                    }
                    run = Some((kind.to_string(), number, index));
                }
                (None, previous) => {
                    if let Some((open_kind, open_number, start)) = previous {
                        flush(&mut divisions, &open_kind, open_number, start, index - 1);
                    }
                }
            }
        }
        if let Some((open_kind, open_number, start)) = run.take() {
            flush(&mut divisions, &open_kind, open_number, start, doc.ayahs.len() - 1);
        }
    }
    let mut occurrence = 0_i64;
    for (index, ayah) in doc.ayahs.iter().enumerate() {
        if ayah.sajdah.is_some() {
            occurrence += 1;
            divisions.push(DivisionRow {
                edition_id: edition_id.to_string(),
                kind: "sajdah".to_string(),
                number: occurrence,
                start_surah: i64::from(ayah.surah),
                start_ayah: i64::from(ayah.ayah),
                end_surah: i64::from(ayah.surah),
                end_ayah: i64::from(ayah.ayah),
                start_global: globals[index],
                end_global: globals[index],
                label: None,
                provenance_id: provenance_id.to_string(),
            });
        }
    }
    divisions
}

fn validation_row(
    run_id: &str,
    doc: &EditionSource,
    report: &ValidationReport,
    created_at: &str,
) -> ValidationReportRow {
    ValidationReportRow {
        id: run_id.to_string(),
        subject_urn: format!("quran-edition:{}@{}", doc.edition.slug, doc.edition.version),
        validator: crate::validation::VALIDATOR_NAME.to_string(),
        validator_version: crate::validation::VALIDATOR_VERSION.to_string(),
        outcome: match report.outcome {
            Outcome::Pass => "pass".to_string(),
            Outcome::PassWithWarnings => "pass_with_warnings".to_string(),
            Outcome::Fail => "fail".to_string(),
        },
        fatal_count: report.fatal_count as i64,
        error_count: report.error_count as i64,
        warning_count: report.warning_count as i64,
        findings_json: serde_json::to_string(&report.findings).unwrap_or_else(|_| "[]".to_string()),
        created_at: created_at.to_string(),
    }
}

/// Run the 13-checkpoint import pipeline.
///
/// Crash/restart model: the run always starts by cleaning its own staging
/// rows, so a retry after a kill is a clean restart. Canonical tables are
/// never touched here — only the separate, human-gated activation writes
/// them. Cancellation cleans staging and marks the run `Cancelled`.
pub async fn run_import(
    db: &dyn Database,
    input: &ImportInput,
    options: &ImportOptions,
    cancel: &AtomicBool,
    progress: ImportProgress,
) -> Result<ImportOutcome, CorpusError> {
    use ImportCheckpoint as C;
    let mut driver = Driver::new(db, input, options, cancel, progress);

    driver.check_cancel().await?;
    driver.claimed().await?;
    if driver.reached(C::Claimed) {
        return Ok(ImportOutcome::Completed(driver.stopped()));
    }
    driver.check_cancel().await?;
    driver.manifest_hashed().await?;
    if driver.reached(C::ManifestHashed) {
        return Ok(ImportOutcome::Completed(driver.stopped()));
    }
    driver.check_cancel().await?;
    driver.adapter_selected().await?;
    if driver.reached(C::AdapterSelected) {
        return Ok(ImportOutcome::Completed(driver.stopped()));
    }
    driver.check_cancel().await?;
    driver.parsed().await?;
    if driver.reached(C::Parsed) {
        return Ok(ImportOutcome::Completed(driver.stopped()));
    }
    driver.check_cancel().await?;
    driver.unicode_audited().await?;
    if driver.reached(C::UnicodeAudited) {
        return Ok(ImportOutcome::Completed(driver.stopped()));
    }
    driver.check_cancel().await?;
    driver.validated().await?;
    if driver.reached(C::Validated) {
        return Ok(ImportOutcome::Completed(driver.stopped()));
    }
    driver.check_cancel().await?;
    driver.tokenized().await?;
    if driver.reached(C::Tokenized) {
        return Ok(ImportOutcome::Completed(driver.stopped()));
    }
    driver.check_cancel().await?;
    driver.hashes_computed().await?;
    if driver.reached(C::HashesComputed) {
        return Ok(ImportOutcome::Completed(driver.stopped()));
    }
    driver.check_cancel().await?;
    driver.staged().await?;
    if driver.reached(C::Staged) {
        return Ok(ImportOutcome::Completed(driver.stopped()));
    }
    driver.check_cancel().await?;
    driver.roundtrip_verified().await?;
    if driver.reached(C::RoundtripVerified) {
        return Ok(ImportOutcome::Completed(driver.stopped()));
    }
    driver.check_cancel().await?;
    driver.compared().await?;
    if driver.reached(C::ReferenceCompared) {
        return Ok(ImportOutcome::Completed(driver.stopped()));
    }
    driver.check_cancel().await?;
    driver.diffed().await?;
    if driver.reached(C::Diffed) {
        return Ok(ImportOutcome::Completed(driver.stopped()));
    }
    driver.check_cancel().await?;
    let mut success = driver.approval_requested().await?;
    success.stopped_at = None;
    driver.reached(C::ApprovalRequested);
    Ok(ImportOutcome::Completed(success))
}

impl Driver<'_> {
    fn stopped(&self) -> ImportSuccess {
        let (slug, version) = match &self.doc {
            Some(doc) => (doc.edition.slug.clone(), doc.edition.version.to_string()),
            None => (String::new(), String::new()),
        };
        ImportSuccess {
            edition_id: self.edition_id.clone(),
            edition_slug: slug,
            edition_version: version,
            validation_report_id: self.input.run_id.clone(),
            difference_report_id: format!("diff-{}", self.input.run_id),
            text_hash: self.text_hash.clone().unwrap_or(ContentHash {
                algorithm: domain::HashAlgorithm::Sha256,
                hex: String::new(),
            }),
            stopped_at: self.last,
        }
    }
}
