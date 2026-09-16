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

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use quran_search::{
    Diagnostic as IndexDiagnostic, FieldId, Fts5Index, FtsDoc, FullTextIndex, IndexError,
    IndexManifest, SemVer, TokenizerFamily,
};
use storage::Database as _;
use storage::error::StorageError;
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
        morphology_dataset_versions: BTreeMap::new(),
        built_at: started_at.clone(),
        doc_count: 0,
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
    let revelation: BTreeMap<i64, String> = surahs
        .iter()
        .map(|s| (s.number, s.revelation_place.clone().unwrap_or_default()))
        .collect();
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
