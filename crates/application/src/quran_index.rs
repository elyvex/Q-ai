//! Phase 2 — `quran.index.build` job: staged index builds with atomic
//! activation (M2, P2-T33/T34).
//!
//! Flow per build: resolve active edition → MV-018 pre-check → stage fresh
//! generation dir → stream ayah docs (stored derived forms feed the text
//! fields; the adapter re-normalizes idempotently) → commit → reopen with
//! the counted manifest → verify → MV-018 post-check → flip the pointer and
//! mark runs in ONE SQLite transaction.
//!
//! A partial index can never serve queries: activation is the pointer flip,
//! and the previous generation stays on disk for single-step rollback.
//! Retention enforcement is `qai index gc` (P2-T35), not this job.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use quran_search::{
    Diagnostic as IndexDiagnostic, FieldId, Fts5Index, FtsDoc, FullTextIndex, IndexError,
    IndexManifest, SemVer, TokenizerFamily,
};
use storage::Database as _;
use storage::error::StorageError;
use storage::quran::TokenAnalysisRow;
use storage_sqlite::SqliteDatabase;

/// Ayah-level index id built here.
pub const QURAN_AYAH_INDEX_ID: &str = "quran.ayah.v1";
/// Job kind for index builds.
pub const QURAN_INDEX_BUILD_KIND: &str = "quran.index.build";
/// Index schema version.
pub const INDEX_SCHEMA_VERSION: u32 = 1;
/// Documents per writer batch.
pub const BATCH_SIZE: usize = 500;

/// Parameters for an index build.
#[derive(Debug, Clone)]
pub struct IndexBuildParams {
    /// Index id (defaults to [`QURAN_AYAH_INDEX_ID`]).
    pub index_id: String,
    /// Edition slug (must be the active edition).
    pub edition_slug: String,
    /// Edition version (must be the active edition).
    pub edition_version: String,
    /// Operator principal id.
    pub invoked_by: String,
    /// Unique tag for this run (job id or CLI-generated).
    pub run_tag: String,
    /// Index root directory (`<root>/gen-<N>/` lives here).
    pub data_dir: PathBuf,
}

/// Report for one completed build.
#[derive(Debug, Clone)]
pub struct IndexBuildReport {
    /// Index identity.
    pub index_id: String,
    /// Build generation just activated.
    pub generation: u64,
    /// Corpus generation indexed.
    pub corpus_generation: i64,
    /// Documents committed.
    pub doc_count: u64,
    /// Serving manifest content hash.
    pub manifest_hash: String,
    /// Previously serving generation, if any.
    pub previous_generation: Option<u64>,
    /// Skeletons indexed into trigram postings (T36).
    pub trigram_skeletons: u64,
    /// Trigram posting rows written (T36).
    pub trigram_postings: u64,
    /// Posting-level self-recall checks passed during the build (T36).
    pub trigram_verified: u64,
    /// MV-018 post-build check (always `unchanged` on success).
    pub mv018: crate::quran_forms::CanonicalCheck,
}

/// Index-build failures. Codes reuse `QAI-IDX` (build infrastructure) and
/// `QAI-NORM` (pipeline faults); no new `QAI-QUR` codes are minted here.
#[derive(Debug, Clone, thiserror::Error)]
pub enum IndexBuildError {
    /// Storage failure.
    #[error("index build storage failed: {0}")]
    Storage(String),
    /// Forms-layer failure (resolution, MV-018 probe setup).
    #[error(transparent)]
    Forms(#[from] crate::quran_forms::FormsError),
    /// Index-engine failure (staging, verification, activation).
    #[error(transparent)]
    Index(#[from] IndexError),
    /// Cancellation was requested mid-build; the pointer is untouched.
    #[error("index build cancelled")]
    Cancelled,
    /// Single-step rollback found no previous generation to restore.
    #[error("index `{index_id}` has no previous generation to restore")]
    NoPreviousGeneration {
        /// Index identity.
        index_id: String,
    },
    /// Single-step rollback refused: the previous generation's directory is
    /// gone (GC evicted it); flipping the pointer would stop serving.
    #[error("index `{index_id}` generation {generation} was evicted from disk")]
    PreviousGenerationEvicted {
        /// Index identity.
        index_id: String,
        /// Missing generation.
        generation: u64,
    },
}

impl IndexBuildError {
    fn storage(error: StorageError) -> Self {
        Self::Storage(error.to_string())
    }
}

impl storage::error::Diagnostic for IndexBuildError {
    fn code(&self) -> storage::error::DiagnosticCode {
        use quran_search::IndexError as IE;
        match self {
            Self::Storage(_) => storage::error::DiagnosticCode::new("QAI-IDX", 3),
            Self::Forms(crate::quran_forms::FormsError::Normalization(inner)) => {
                use quran_normalization::error::NormalizationError as NE;
                let number = match inner {
                    NE::UnknownRule { .. } => 1,
                    NE::UnknownProfile { .. } => 2,
                    NE::ProfileImmutable { .. } => 3,
                    NE::SpanOutOfRange { .. } => 4,
                    NE::InvalidMapping { .. } => 5,
                    NE::EmptyProfile => 6,
                };
                storage::error::DiagnosticCode::new("QAI-NORM", number)
            }
            Self::Forms(_) => storage::error::DiagnosticCode::new("QAI-IDX", 3),
            Self::Index(inner) => {
                let number = match inner {
                    IE::BackendUnavailable { .. } => 1,
                    IE::QueryRejected { .. } => 2,
                    IE::BuildFailed { .. } => 3,
                    IE::ManifestMismatch { .. } => 4,
                    IE::CanonicalChanged { .. } => 5,
                    IE::InvalidHit { .. } => 6,
                    IE::RateLimited { .. } => 7,
                };
                storage::error::DiagnosticCode::new("QAI-IDX", number)
            }
            Self::Cancelled => storage::error::DiagnosticCode::new("QAI-IDX", 3),
            Self::NoPreviousGeneration { .. } => storage::error::DiagnosticCode::new("QAI-IDX", 8),
            Self::PreviousGenerationEvicted { .. } => {
                storage::error::DiagnosticCode::new("QAI-IDX", 9)
            }
        }
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn remedy(&self) -> Option<String> {
        Some(match self {
            Self::Storage(_) => "Check the database and retry the build.".to_string(),
            Self::Forms(_) => {
                "Fix the forms layer first (see its remedy), then rebuild.".to_string()
            }
            Self::Index(inner) => inner.remedy().unwrap_or_else(|| "See above.".to_string()),
            Self::Cancelled => "Rerun the build; staging is wiped and rebuilt.".to_string(),
            Self::NoPreviousGeneration { .. } => {
                "Only one generation ever served (or GC already evicted the previous one); nothing to undo."
                    .to_string()
            }
            Self::PreviousGenerationEvicted { .. } => {
                "The previous generation's directory is gone; rebuild to create a fresh generation."
                    .to_string()
            }
        })
    }

    fn next_command(&self) -> Option<String> {
        Some("qai doctor --indexes".to_string())
    }
}

/// Index root for a database file: colocated `<db-dir>/index`.
#[must_use]
pub fn index_root_for_db(db_path: &str) -> PathBuf {
    Path::new(db_path)
        .parent()
        .map_or_else(|| PathBuf::from("index"), |parent| parent.join("index"))
}

type LexiconProjection = BTreeMap<(i64, i64), BTreeMap<FieldId, BTreeSet<String>>>;

#[derive(Debug, Default)]
struct MorphologyProjection {
    dataset_versions: BTreeMap<String, SemVer>,
    fields: LexiconProjection,
}

fn add_lexicon_value(
    projection: &mut LexiconProjection,
    reference: (i64, i64),
    field: &str,
    value: &str,
) {
    let value = value.trim();
    if value.is_empty() {
        return;
    }
    projection
        .entry(reference)
        .or_default()
        .entry(field.to_string())
        .or_default()
        .insert(value.to_string());
}

fn pattern_values(row: &TokenAnalysisRow) -> Vec<String> {
    let Ok(features) = serde_json::from_str::<serde_json::Value>(&row.features_json) else {
        return Vec::new();
    };
    ["pattern", "verb_form", "morphological_pattern"]
        .iter()
        .filter_map(|key| features.get(*key).and_then(serde_json::Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

/// Load the active, edition-relative morphology projection for an index build.
///
/// A dataset is never applied to a different edition: rows are filtered by the
/// exact `edition_id` before any lexicon value reaches an FTS document. If the
/// active dataset has no rows for this edition, the index simply remains
/// text-only rather than borrowing another edition's analysis.
async fn load_morphology_projection(
    db: &SqliteDatabase,
    edition_id: &str,
) -> Result<Option<MorphologyProjection>, IndexBuildError> {
    let mut uow = db.write().await.map_err(IndexBuildError::storage)?;
    let dataset = uow.quran().active_dataset().await.map_err(IndexBuildError::storage)?;
    let Some(dataset) = dataset else {
        uow.rollback().await.map_err(IndexBuildError::storage)?;
        return Ok(None);
    };
    let analyses =
        uow.quran().list_analyses(&dataset.id).await.map_err(IndexBuildError::storage)?;
    let roots = uow.quran().list_roots(&dataset.id).await.map_err(IndexBuildError::storage)?;
    let lemmas = uow.quran().list_lemmas(&dataset.id).await.map_err(IndexBuildError::storage)?;
    uow.rollback().await.map_err(IndexBuildError::storage)?;

    let analyses: Vec<TokenAnalysisRow> =
        analyses.into_iter().filter(|row| row.edition_id == edition_id).collect();
    if analyses.is_empty() {
        return Ok(None);
    }

    let root_map: HashMap<String, String> =
        roots.into_iter().map(|row| (row.id, row.root)).collect();
    let lemma_map: HashMap<String, String> =
        lemmas.into_iter().map(|row| (row.id, row.lemma)).collect();
    let mut fields = LexiconProjection::new();
    for row in &analyses {
        let reference = (row.surah, row.ayah);
        if let Some(root) = row.root_id.as_ref().and_then(|id| root_map.get(id)) {
            add_lexicon_value(&mut fields, reference, "roots", root);
        }
        if let Some(lemma) = row.lemma_id.as_ref().and_then(|id| lemma_map.get(id)) {
            add_lexicon_value(&mut fields, reference, "lemmas", lemma);
        }
        add_lexicon_value(&mut fields, reference, "stems", &row.stem);
        add_lexicon_value(&mut fields, reference, "pos_tags", &row.pos_unified);
        add_lexicon_value(&mut fields, reference, "pos_tags", &row.pos_native);
        for pattern in pattern_values(row) {
            add_lexicon_value(&mut fields, reference, "patterns", &pattern);
        }
    }

    let mut dataset_versions = BTreeMap::new();
    if let Ok(version) = dataset.version.parse::<SemVer>() {
        dataset_versions.insert(dataset.slug, version);
    }
    Ok(Some(MorphologyProjection { dataset_versions, fields }))
}

/// Rebuild one index end to end (P2-T34).
///
/// See the module docs for the stage order. Nothing serves partial results:
/// readers keep the previous pointer until the flip transaction commits.
pub async fn rebuild_index(
    db: &SqliteDatabase,
    params: &IndexBuildParams,
    cancel: &AtomicBool,
    checkpoint: impl Fn(&str),
) -> Result<IndexBuildReport, IndexBuildError> {
    use quran_normalization::ProfileId;

    let started_at = domain::Timestamp::now().to_string();
    // 1. Resolve the active edition.
    let (edition_id, corpus_generation, _edition_slug, edition_version) = {
        let mut uow = db.write().await.map_err(IndexBuildError::storage)?;
        let edition = uow
            .quran()
            .get_edition_by_slug_version(&params.edition_slug, &params.edition_version)
            .await
            .map_err(IndexBuildError::storage)?;
        let Some(edition) = edition else {
            return Err(IndexBuildError::Index(IndexError::BuildFailed {
                stage: "resolve".to_string(),
                detail: format!("no edition {}@{}", params.edition_slug, params.edition_version),
            }));
        };
        let active = uow.quran().get_active().await.map_err(IndexBuildError::storage)?;
        uow.rollback().await.map_err(IndexBuildError::storage)?;
        match active {
            Some(active) if active.edition_id == edition.id => (
                edition.id.clone(),
                active.corpus_generation,
                edition.slug.clone(),
                edition.version.clone(),
            ),
            _ => {
                return Err(IndexBuildError::Index(IndexError::BuildFailed {
                    stage: "resolve".to_string(),
                    detail: format!(
                        "edition {}@{} is not active; builds target the active edition",
                        params.edition_slug, params.edition_version
                    ),
                }));
            }
        }
    };
    checkpoint("resolved");

    // 2. MV-018 pre-check: stop before ANY write on drift.
    crate::quran_forms::verify_canonical_unchanged(db, &edition_id)
        .await
        .map_err(IndexBuildError::Forms)?;
    checkpoint("mv018-pre");

    // 3. Seeded definitions prove the seed per build; family pins versions.
    let profile_rows = {
        let mut uow = db.write().await.map_err(IndexBuildError::storage)?;
        let rows =
            uow.quran().list_normalization_profiles().await.map_err(IndexBuildError::storage)?;
        uow.rollback().await.map_err(IndexBuildError::storage)?;
        rows
    };
    let registry = crate::quran_normalize::registry_from_rows(&profile_rows).map_err(|err| {
        IndexBuildError::Index(IndexError::BuildFailed {
            stage: "resolve".to_string(),
            detail: err.to_string(),
        })
    })?;
    let ladder = {
        let mut version = SemVer::new(0, 0, 0);
        for id in ProfileId::all() {
            if let Ok(profile) = registry.latest(id) {
                version = version.max(profile.version);
            }
        }
        version
    };
    let family = TokenizerFamily::new(&registry, ladder).map_err(IndexBuildError::Index)?;
    let mut rule_set_versions = BTreeMap::new();
    for id in [
        ProfileId::L0,
        ProfileId::L1,
        ProfileId::L2,
        ProfileId::L3,
        ProfileId::L4,
        ProfileId::L5,
        ProfileId::L7,
    ] {
        let profile = registry.latest(id).map_err(|err| {
            IndexBuildError::Index(IndexError::BuildFailed {
                stage: "resolve".to_string(),
                detail: err.to_string(),
            })
        })?;
        rule_set_versions.insert(id.to_string(), profile.version);
    }
    let edition_semver: SemVer = edition_version.parse().map_err(|_| {
        IndexBuildError::Index(IndexError::BuildFailed {
            stage: "resolve".to_string(),
            detail: format!("edition version {edition_version} is not MAJOR.MINOR.PATCH"),
        })
    })?;
    let morphology = load_morphology_projection(db, &edition_id).await?;
    let morphology_dataset_versions = morphology
        .as_ref()
        .map(|projection| projection.dataset_versions.clone())
        .unwrap_or_default();

    // 4. Stage the next generation.
    let generation = {
        let mut uow = db.write().await.map_err(IndexBuildError::storage)?;
        let max = uow
            .quran()
            .max_build_generation(&params.index_id)
            .await
            .map_err(IndexBuildError::storage)?;
        uow.rollback().await.map_err(IndexBuildError::storage)?;
        (max + 1).max(1) as u64
    };
    let manifest = IndexManifest {
        index_id: params.index_id.clone(),
        schema_version: INDEX_SCHEMA_VERSION,
        corpus_generation: corpus_generation as u64,
        edition_id: edition_id.clone(),
        edition_version: edition_semver,
        rule_set_versions,
        tokenizer_version: ladder,
        morphology_dataset_versions,
        built_at: started_at.clone(),
        doc_count: 0,
        trigram_postings: 0,
        content_hash: String::new(),
    };
    let index_root = params.data_dir.clone();
    let staged = Fts5Index::stage(&index_root, generation, manifest.clone(), family)
        .await
        .map_err(IndexBuildError::Index)?;
    let run_id = format!("index-build-{}-g{}-{}", params.index_id, generation, params.run_tag);
    {
        let mut uow = db.write().await.map_err(IndexBuildError::storage)?;
        uow.quran()
            .insert_build_run(storage::quran::IndexBuildRunRow {
                id: run_id.clone(),
                index_id: params.index_id.clone(),
                generation: generation as i64,
                corpus_generation,
                state: "staged".to_string(),
                doc_count: 0,
                manifest_hash: String::new(),
                error: None,
                started_at: started_at.clone(),
                finished_at: None,
            })
            .await
            .map_err(IndexBuildError::storage)?;
        uow.commit().await.map_err(IndexBuildError::storage)?;
    }
    checkpoint("staged");

    // 5. Stream ayah docs from stored forms (canonical text feeds `text_exact`
    //    and the L1 field; the adapter re-normalizes idempotently).
    let l1 =
        quran_normalization::NormalizationPipeline::for_profile(&registry, ProfileId::L1, ladder)
            .map_err(|err| {
            IndexBuildError::Index(IndexError::BuildFailed {
                stage: "build".to_string(),
                detail: err.to_string(),
            })
        })?;
    let surahs = {
        let mut uow = db.write().await.map_err(IndexBuildError::storage)?;
        let surahs =
            uow.quran().list_surahs(&edition_id).await.map_err(IndexBuildError::storage)?;
        uow.rollback().await.map_err(IndexBuildError::storage)?;
        surahs
    };
    let revelation: BTreeMap<i64, String> =
        surahs.iter().map(|s| (s.number, s.revelation_place.clone().unwrap_or_default())).collect();
    let ayahs = {
        let mut uow = db.write().await.map_err(IndexBuildError::storage)?;
        let ayahs = uow
            .quran()
            .list_ayahs_range(&edition_id, 1, i64::MAX)
            .await
            .map_err(IndexBuildError::storage)?;
        uow.rollback().await.map_err(IndexBuildError::storage)?;
        let mut ordered = ayahs;
        ordered.sort_by_key(|a| (a.surah, a.ayah));
        ordered
    };
    let mut batch: Vec<FtsDoc> = Vec::with_capacity(BATCH_SIZE);
    for ayah in &ayahs {
        if cancel.load(Ordering::SeqCst) {
            fail_run(db, &run_id, "cancelled by operator").await;
            return Err(IndexBuildError::Cancelled);
        }
        let forms = {
            let mut uow = db.write().await.map_err(IndexBuildError::storage)?;
            let forms = uow
                .quran()
                .get_ayah_form(&edition_id, ayah.surah, ayah.ayah)
                .await
                .map_err(IndexBuildError::storage)?;
            uow.rollback().await.map_err(IndexBuildError::storage)?;
            forms
        };
        let Some(forms) = forms else {
            fail_run(db, &run_id, &format!("missing forms for {}:{}", ayah.surah, ayah.ayah)).await;
            return Err(IndexBuildError::Index(IndexError::BuildFailed {
                stage: "build".to_string(),
                detail: format!(
                    "missing forms for {}:{}; rebuild forms first",
                    ayah.surah, ayah.ayah
                ),
            }));
        };
        let mut fields: BTreeMap<FieldId, String> = BTreeMap::new();
        fields.insert("text_exact".to_string(), ayah.text.clone());
        fields.insert("text_ws".to_string(), l1.apply(&ayah.text).0.text().to_string());
        fields.insert("text_marks".to_string(), forms.simple.clone());
        fields.insert("text_bare".to_string(), forms.bare.clone());
        fields.insert("text_hamza".to_string(), forms.hamza_folded.clone());
        fields.insert("text_folded".to_string(), forms.folded.clone());
        fields.insert("text_affix".to_string(), String::new());
        let lexicon_fields = morphology
            .as_ref()
            .and_then(|projection| projection.fields.get(&(ayah.surah, ayah.ayah)))
            .map(|values| {
                values
                    .iter()
                    .map(|(field, entries)| {
                        (field.clone(), entries.iter().cloned().collect::<Vec<_>>().join(" "))
                    })
                    .collect()
            })
            .unwrap_or_default();
        batch.push(FtsDoc {
            id: format!("{}:{}:{}:{}", params.index_id, edition_id, ayah.surah, ayah.ayah),
            edition_id: edition_id.clone(),
            surah: ayah.surah as u16,
            ayah: ayah.ayah as u16,
            global_index: ayah.global_ayah_index as u64,
            generation: corpus_generation as u64,
            metadata: BTreeMap::from([
                ("juz".to_string(), ayah.juz.unwrap_or(0).to_string()),
                ("page".to_string(), ayah.page.unwrap_or(0).to_string()),
                (
                    "revelation".to_string(),
                    revelation.get(&ayah.surah).cloned().unwrap_or_default(),
                ),
            ]),
            fields,
            lexicon_fields,
        });
        if batch.len() >= BATCH_SIZE {
            let chunk = std::mem::take(&mut batch);
            staged.add_batch(chunk).await.map_err(IndexBuildError::Index)?;
            checkpoint("build");
        }
    }
    if !batch.is_empty() {
        staged.add_batch(batch).await.map_err(IndexBuildError::Index)?;
    }
    checkpoint("built");

    // 5b. Trigram posting index (P2-T36): every stored skeleton contributes
    // its character-trigram set to `<root>/gen-<N>/trigram.db`. Derived from
    // the same stored skeletons the pre-T36 scan probed, so recall is
    // unchanged; verification still decides. Cancel-safe: staging is wiped
    // on the next build and the pointer never names this generation yet.
    let trigram_built = {
        let mut posting_rows: Vec<(quran_search::SkelAddr, String)> = Vec::new();
        for surah in &surahs {
            if cancel.load(Ordering::SeqCst) {
                fail_run(db, &run_id, "cancelled by operator").await;
                return Err(IndexBuildError::Cancelled);
            }
            let mut uow = db.write().await.map_err(IndexBuildError::storage)?;
            let skeletons = uow
                .quran()
                .list_skeletons(&edition_id, surah.number)
                .await
                .map_err(IndexBuildError::storage)?;
            uow.rollback().await.map_err(IndexBuildError::storage)?;
            posting_rows.extend(skeletons.into_iter().map(|row| {
                (
                    quran_search::SkelAddr {
                        surah: row.surah,
                        ayah_start: row.ayah_start,
                        ayah_end: row.ayah_end,
                    },
                    row.skeleton,
                )
            }));
        }
        let trigram_file = quran_search::trigram_path(&index_root, generation);
        let (skel_count, posting_count) = quran_search::trigram::build_postings(
            &trigram_file,
            &posting_rows,
        )
        .await
        .map_err(|detail| {
            IndexBuildError::Index(IndexError::BuildFailed { stage: "trigram".to_string(), detail })
        })?;
        // Posting-level verify: deterministic sample (every row under 2000,
        // every 16th above) must resolve back to itself — no false negatives.
        let sample: Vec<(quran_search::SkelAddr, String)> = if posting_rows.len() <= 2000 {
            posting_rows.clone()
        } else {
            posting_rows
                .iter()
                .enumerate()
                .filter(|(i, _)| i % 16 == 0)
                .map(|(_, row)| row.clone())
                .collect()
        };
        let checked = quran_search::trigram::verify_postings(&trigram_file, &sample)
            .await
            .map_err(|detail| {
                IndexBuildError::Index(IndexError::BuildFailed {
                    stage: "trigram-verify".to_string(),
                    detail,
                })
            })?;
        (skel_count, posting_count, checked)
    };
    checkpoint("trigram");

    // 6. Commit, reopen with the counted manifest, verify.
    let stamp = staged.commit().await.map_err(IndexBuildError::Index)?;
    // Identity-bound (not row-surrogate-bound): re-imports of the same
    // edition version reproduce the same hash, so drift reports compare
    // derivations, not import runs.
    let manifest_hash = quran_corpus::sha256_hex(
        format!(
            "{}|{}|{}@{}|{}|{}",
            params.index_id,
            corpus_generation,
            params.edition_slug,
            params.edition_version,
            stamp.doc_count,
            ladder
        )
        .as_bytes(),
    );
    let manifest_hash = format!("sha256:{manifest_hash}");
    let manifest = IndexManifest {
        doc_count: stamp.doc_count,
        trigram_postings: trigram_built.1,
        content_hash: manifest_hash.clone(),
        ..manifest
    };
    let serving =
        Fts5Index::open(&index_root, generation, manifest.clone(), family_from(&registry, ladder)?)
            .await
            .map_err(IndexBuildError::Index)?;
    set_run_state(db, &run_id, "verifying", stamp.doc_count as i64, &manifest_hash, None).await;
    let report = serving.verify().await.map_err(IndexBuildError::Index)?;
    if !report.ok {
        fail_run(db, &run_id, &report.findings.join("; ")).await;
        return Err(IndexBuildError::Index(IndexError::BuildFailed {
            stage: "verify".to_string(),
            detail: report.findings.join("; "),
        }));
    }
    checkpoint("verified");

    // A cancelled build must never flip the pointer, even when every stage
    // already succeeded (P2-T37 crash matrix).
    if cancel.load(Ordering::SeqCst) {
        fail_run(db, &run_id, "cancelled by operator").await;
        return Err(IndexBuildError::Cancelled);
    }

    // 7. MV-018 post-check, then the atomic flip (pointer + run states).
    let mv018 =
        crate::quran_forms::verify_canonical_unchanged(db, &edition_id).await.map_err(|err| {
            match err {
                crate::quran_forms::FormsError::Index(inner) => IndexBuildError::Index(inner),
                crate::quran_forms::FormsError::Normalization(inner) => {
                    IndexBuildError::Index(IndexError::BuildFailed {
                        stage: "mv018".to_string(),
                        detail: inner.to_string(),
                    })
                }
                other => IndexBuildError::Index(IndexError::BuildFailed {
                    stage: "mv018".to_string(),
                    detail: other.to_string(),
                }),
            }
        })?;
    let finished_at = domain::Timestamp::now().to_string();
    let previous_generation = {
        let mut uow = db.write().await.map_err(IndexBuildError::storage)?;
        let previous = uow
            .quran()
            .get_index_pointer(&params.index_id)
            .await
            .map_err(IndexBuildError::storage)?;
        let manifest_json = serde_json::to_string(&manifest).map_err(|err| {
            IndexBuildError::Index(IndexError::BuildFailed {
                stage: "activate".to_string(),
                detail: err.to_string(),
            })
        })?;
        uow.quran()
            .upsert_index_pointer(storage::quran::IndexPointerRow {
                index_id: params.index_id.clone(),
                generation: generation as i64,
                manifest_json,
                updated_at: finished_at.clone(),
                updated_by: params.invoked_by.clone(),
            })
            .await
            .map_err(IndexBuildError::storage)?;
        // The previously serving run steps aside; it stays on disk.
        if let Some(previous) = previous.as_ref() {
            for run in uow
                .quran()
                .list_build_runs(&params.index_id)
                .await
                .map_err(IndexBuildError::storage)?
            {
                if run.generation == previous.generation && run.state == "active" {
                    uow.quran()
                        .set_build_run_state(
                            &run.id,
                            "superseded",
                            run.doc_count,
                            &run.manifest_hash,
                            None,
                            &finished_at,
                        )
                        .await
                        .map_err(IndexBuildError::storage)?;
                }
            }
        }
        uow.quran()
            .set_build_run_state(
                &run_id,
                "active",
                stamp.doc_count as i64,
                &manifest_hash,
                None,
                &finished_at,
            )
            .await
            .map_err(IndexBuildError::storage)?;
        uow.commit().await.map_err(IndexBuildError::storage)?;
        previous.map(|p| p.generation as u64)
    };
    checkpoint("activated");

    Ok(IndexBuildReport {
        index_id: params.index_id.clone(),
        generation,
        corpus_generation,
        doc_count: stamp.doc_count,
        manifest_hash,
        previous_generation,
        trigram_skeletons: trigram_built.0,
        trigram_postings: trigram_built.1,
        trigram_verified: trigram_built.2,
        mv018: crate::quran_forms::CanonicalCheck {
            edition_urn: mv018.edition_urn,
            expected_hash: mv018.expected_hash,
            actual_hash: mv018.actual_hash,
            unchanged: mv018.unchanged,
        },
    })
}

/// Rebuild the tokenizer family (helper kept beside the build for clarity).
fn family_from(
    registry: &quran_normalization::ProfileRegistry,
    ladder: SemVer,
) -> Result<quran_search::TokenizerFamily, IndexBuildError> {
    quran_search::TokenizerFamily::new(registry, ladder).map_err(IndexBuildError::Index)
}

/// Parameters for index retention GC (P2-T35).
#[derive(Debug, Clone)]
pub struct GcParams {
    /// Index id (defaults to [`QURAN_AYAH_INDEX_ID`]).
    pub index_id: String,
    /// Generations to retain on disk, newest-first including the active one.
    /// Clamped to `>= 1`; the active generation is never removed even when
    /// `keep` would exclude it. Default `2` = single-step rollback.
    pub keep: usize,
    /// Operator principal id (recorded in the report; GC writes no catalog rows).
    pub invoked_by: String,
    /// Index root directory (`<root>/gen-<N>/` lives here).
    pub data_dir: PathBuf,
}

/// One removed generation.
#[derive(Debug, Clone)]
pub struct GcRemoved {
    /// Build generation removed from disk.
    pub generation: u64,
    /// Run-row id deleted (`None` for failed runs, whose history row is kept).
    pub run_id_deleted: Option<String>,
    /// Documents counted before removal.
    pub docs_removed: u64,
}

/// Report for one retention-GC pass.
#[derive(Debug, Clone)]
pub struct GcReport {
    /// Index identity.
    pub index_id: String,
    /// Serving generation (always protected).
    pub active_generation: Option<u64>,
    /// Generations retained on disk, newest-first.
    pub kept: Vec<u64>,
    /// Generations removed.
    pub removed: Vec<GcRemoved>,
}

/// Enforce index-generation retention (P2-T35).
///
/// Policy: the active generation is never touched. The newest `keep - 1`
/// superseded generations are retained for rollback; older superseded
/// generations lose their directory (`index.db` + `trigram.db`) and their
/// run row. Failed runs lose orphaned staging directories but keep their
/// history rows. Non-terminal runs (`staged`, `verifying`) are left alone —
/// they may belong to a live build; run GC when no build is in flight.
///
/// Crash-safe: directory removal precedes row deletion, and
/// [`Fts5Index::remove_generation`] reports `0` for missing directories, so
/// a crash between the two retries cleanly. Cancel-safe: cancellation stops
/// further removals; already-removed generations stay removed (their rows
/// are deleted in the same pass).
pub async fn gc_index(
    db: &SqliteDatabase,
    params: &GcParams,
    cancel: &AtomicBool,
) -> Result<GcReport, IndexBuildError> {
    let keep = params.keep.max(1);
    let (pointer, runs) = {
        let mut uow = db.write().await.map_err(IndexBuildError::storage)?;
        let pointer = uow
            .quran()
            .get_index_pointer(&params.index_id)
            .await
            .map_err(IndexBuildError::storage)?;
        let runs = uow
            .quran()
            .list_build_runs(&params.index_id)
            .await
            .map_err(IndexBuildError::storage)?;
        uow.rollback().await.map_err(IndexBuildError::storage)?;
        (pointer, runs)
    };
    let active_generation = pointer.as_ref().map(|p| p.generation as u64);

    // Newest-first protection window over terminal servable generations.
    let mut servable: Vec<u64> = runs
        .iter()
        .filter(|r| r.state == "active" || r.state == "superseded")
        .map(|r| r.generation as u64)
        .collect();
    servable.sort_unstable();
    servable.dedup();
    let protected: std::collections::BTreeSet<u64> =
        servable.iter().rev().take(keep).copied().chain(active_generation).collect();

    let mut kept: Vec<u64> = protected.iter().copied().collect();
    kept.sort_unstable_by(|a, b| b.cmp(a));
    let mut removed: Vec<GcRemoved> = Vec::new();
    let mut rows_to_delete: Vec<String> = Vec::new();

    for run in &runs {
        if cancel.load(Ordering::SeqCst) {
            return Err(IndexBuildError::Cancelled);
        }
        let generation = run.generation as u64;
        match run.state.as_str() {
            "active" => {}
            "superseded" if protected.contains(&generation) => {}
            "superseded" => {
                let docs = Fts5Index::remove_generation(&params.data_dir, generation)
                    .await
                    .map_err(IndexBuildError::Index)?;
                rows_to_delete.push(run.id.clone());
                removed.push(GcRemoved {
                    generation,
                    run_id_deleted: Some(run.id.clone()),
                    docs_removed: docs,
                });
            }
            "failed" => {
                // Free orphaned staging output; the row stays as history.
                let docs = Fts5Index::remove_generation(&params.data_dir, generation)
                    .await
                    .map_err(IndexBuildError::Index)?;
                if docs > 0 {
                    removed.push(GcRemoved {
                        generation,
                        run_id_deleted: None,
                        docs_removed: docs,
                    });
                }
            }
            // Non-terminal runs may belong to a live build: never touch.
            _ => {}
        }
    }

    if !rows_to_delete.is_empty() {
        let mut uow = db.write().await.map_err(IndexBuildError::storage)?;
        for id in &rows_to_delete {
            uow.quran().delete_build_run(id).await.map_err(IndexBuildError::storage)?;
        }
        uow.commit().await.map_err(IndexBuildError::storage)?;
    }

    removed.sort_by_key(|r| r.generation);
    Ok(GcReport { index_id: params.index_id.clone(), active_generation, kept, removed })
}

/// Best-effort run-state update (failures here must not mask the real error).
async fn set_run_state(
    db: &SqliteDatabase,
    run_id: &str,
    state: &str,
    doc_count: i64,
    manifest_hash: &str,
    error: Option<&str>,
) {
    let finished_at = domain::Timestamp::now().to_string();
    if let Ok(mut uow) = db.write().await {
        let _ = uow
            .quran()
            .set_build_run_state(run_id, state, doc_count, manifest_hash, error, &finished_at)
            .await;
        let _ = uow.commit().await;
    }
}

/// Best-effort failure marking.
async fn fail_run(db: &SqliteDatabase, run_id: &str, error: &str) {
    set_run_state(db, run_id, "failed", 0, "", Some(error)).await;
}

/// Payload for `quran.index.build` jobs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndexBuildPayload {
    /// Index id (defaults to [`QURAN_AYAH_INDEX_ID`]).
    pub index_id: Option<String>,
    /// Edition slug (must be active).
    pub edition_slug: String,
    /// Edition version (must be active).
    pub edition_version: String,
    /// Operator principal id.
    pub invoked_by: String,
    /// Unique run tag (defaults to the job id).
    pub run_tag: Option<String>,
    /// Index root directory.
    pub data_dir: PathBuf,
}

/// The `quran.index.build` job handler.
pub struct IndexBuildHandler {
    db: Arc<SqliteDatabase>,
}

impl IndexBuildHandler {
    /// Wrap a database handle.
    pub fn new(db: Arc<SqliteDatabase>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl jobs::JobHandler for IndexBuildHandler {
    fn kind(&self) -> jobs::JobKind {
        QURAN_INDEX_BUILD_KIND.into()
    }

    fn payload_schema(&self) -> &'static str {
        r#"{"type":"object","required":["edition_slug","edition_version","invoked_by","data_dir"]}"#
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
        use storage::error::Diagnostic as _;
        let input: IndexBuildPayload = serde_json::from_value(payload)
            .map_err(|err| JobError::Storage(format!("bad index-build payload: {err}")))?;
        let params = IndexBuildParams {
            index_id: input.index_id.unwrap_or_else(|| QURAN_AYAH_INDEX_ID.to_string()),
            edition_slug: input.edition_slug,
            edition_version: input.edition_version,
            invoked_by: input.invoked_by,
            run_tag: input.run_tag.unwrap_or_else(|| ctx.job.id.clone()),
            data_dir: input.data_dir,
        };
        let cancel = ctx.cancel_flag();
        let sink = ctx.checkpoint_sink();
        let report = rebuild_index(&self.db, &params, &cancel, |stage| {
            if let Ok(mut slot) = sink.lock() {
                *slot = Some(stage.to_string());
            }
        })
        .await;
        match report {
            Ok(report) => Ok(jobs::JobOutcome {
                success: true,
                result: Some(format!(
                    "index {} generation {} active: {} docs (corpus generation {})",
                    report.index_id, report.generation, report.doc_count, report.corpus_generation
                )),
            }),
            Err(IndexBuildError::Cancelled) => Err(JobError::Cancelled { id: ctx.job.id.clone() }),
            Err(err) => Err(JobError::Storage(format!("{}: {}", err.code(), err.summary()))),
        }
    }
}

/// Parameters for single-step index rollback (P2-T35).
#[derive(Debug, Clone)]
pub struct RollbackParams {
    /// Index id (defaults to [`QURAN_AYAH_INDEX_ID`]).
    pub index_id: String,
    /// Operator principal id (recorded on the pointer row).
    pub invoked_by: String,
    /// Index root directory (`<root>/gen-<N>/` lives here).
    pub data_dir: PathBuf,
}

/// Report for one single-step rollback.
#[derive(Debug, Clone)]
pub struct RollbackReport {
    /// Index identity.
    pub index_id: String,
    /// Previously serving generation (now superseded; still on disk).
    pub from_generation: u64,
    /// Restored serving generation.
    pub to_generation: u64,
}

/// Restore the previous serving generation (P2-T35).
///
/// Single-step undo of the last activation: the serving pointer flips back to
/// the newest `superseded` run below the current generation, run states swap
/// (`active` ↔ `superseded`) in ONE SQLite transaction, and the newer
/// generation stays on disk (a later rebuild or a second flip can move
/// forward again only through a new build — rollback never moves the pointer
/// to a *newer* generation, so it cannot ping-pong).
///
/// Fail-closed: with no previous generation ([`IndexBuildError::NoPreviousGeneration`])
/// or with the previous directory evicted ([`IndexBuildError::PreviousGenerationEvicted`])
/// the pointer is untouched. Like activation, this needs no approval (derived
/// data, not canonical text) and writes no audit event.
pub async fn rollback_index_single_step(
    db: &SqliteDatabase,
    params: &RollbackParams,
) -> Result<RollbackReport, IndexBuildError> {
    let mut uow = db.write().await.map_err(IndexBuildError::storage)?;
    let pointer =
        uow.quran().get_index_pointer(&params.index_id).await.map_err(IndexBuildError::storage)?;
    let Some(pointer) = pointer else {
        uow.rollback().await.map_err(IndexBuildError::storage)?;
        return Err(IndexBuildError::NoPreviousGeneration { index_id: params.index_id.clone() });
    };
    let current = pointer.generation as u64;
    let runs =
        uow.quran().list_build_runs(&params.index_id).await.map_err(IndexBuildError::storage)?;
    let previous = runs
        .iter()
        .filter(|run| run.state == "superseded" && (run.generation as u64) < current)
        .map(|run| run.generation as u64)
        .max();
    let Some(previous) = previous else {
        uow.rollback().await.map_err(IndexBuildError::storage)?;
        return Err(IndexBuildError::NoPreviousGeneration { index_id: params.index_id.clone() });
    };
    // The retained directory must still exist: flipping to a missing
    // generation would stop serving. GC eviction (or manual deletion) makes
    // rollback impossible — rebuild instead.
    if !params.data_dir.join(format!("gen-{previous}")).exists() {
        uow.rollback().await.map_err(IndexBuildError::storage)?;
        return Err(IndexBuildError::PreviousGenerationEvicted {
            index_id: params.index_id.clone(),
            generation: previous,
        });
    }
    let now = domain::Timestamp::now().to_string();
    uow.quran()
        .upsert_index_pointer(storage::quran::IndexPointerRow {
            index_id: params.index_id.clone(),
            generation: previous as i64,
            manifest_json: previous_run_manifest(&params.data_dir, previous)?,
            updated_at: now.clone(),
            updated_by: params.invoked_by.clone(),
        })
        .await
        .map_err(IndexBuildError::storage)?;
    for run in &runs {
        let generation = run.generation as u64;
        if generation == current && run.state == "active" {
            uow.quran()
                .set_build_run_state(
                    &run.id,
                    "superseded",
                    run.doc_count,
                    &run.manifest_hash,
                    None,
                    &now,
                )
                .await
                .map_err(IndexBuildError::storage)?;
        } else if generation == previous && run.state == "superseded" {
            uow.quran()
                .set_build_run_state(
                    &run.id,
                    "active",
                    run.doc_count,
                    &run.manifest_hash,
                    None,
                    &now,
                )
                .await
                .map_err(IndexBuildError::storage)?;
        }
    }
    uow.commit().await.map_err(IndexBuildError::storage)?;
    Ok(RollbackReport {
        index_id: params.index_id.clone(),
        from_generation: current,
        to_generation: previous,
    })
}

/// Read the complete manifest of a retained generation for the pointer row.
fn previous_run_manifest(data_dir: &Path, generation: u64) -> Result<String, IndexBuildError> {
    let path = data_dir.join(format!("gen-{generation}")).join("manifest.json");
    std::fs::read_to_string(&path).map_err(|error| {
        IndexBuildError::Index(IndexError::BuildFailed {
            stage: "rollback".to_string(),
            detail: format!("cannot read retained manifest {}: {error}", path.display()),
        })
    })
}
