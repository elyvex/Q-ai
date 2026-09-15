//! Phase 2 — `quran.forms.rebuild` job and the MV-018 canonical-unchanged
//! verifier (M2, P2-T26/T28).
//!
//! The job derives search forms for every token and ayah plus skeleton rows,
//! writing **only** to derived Layer-D tables. MV-018 (canonical text
//! byte-identical, `QAI-IDX-0005`) runs **before any write and again before
//! commit**: a hash drift fails the build instead of shipping derived data
//! over a moved canonical baseline.
//!
//! MV-018 is defined here because the forms job needs it first; the
//! morphology importer (M4, MV-001…018) reuses
//! [`verify_canonical_unchanged`] and [`quran_search::IndexError::CanonicalChanged`]
//! rather than inventing a second verifier.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use quran_normalization::Diagnostic as NormDiagnostic;
use quran_search::Diagnostic as IndexDiagnostic;
use quran_search::IndexError;
use storage::Database as _;
use storage::error::Diagnostic as _;
use storage::error::StorageError;
use storage_sqlite::SqliteDatabase;

use crate::quran_normalize;

/// Job kind for derived-forms rebuilds.
pub const QURAN_FORMS_REBUILD_KIND: &str = "quran.forms.rebuild";
/// Rule-set id stamped on every derived row.
pub const RULE_SET_ID: &str = "quran-normalization";

/// Parameters for a forms rebuild.
#[derive(Debug, Clone)]
pub struct RebuildParams {
    /// Edition slug (must be the active edition).
    pub edition_slug: String,
    /// Edition version (must be the active edition).
    pub edition_version: String,
    /// Operator principal id (provenance `created_by`).
    pub invoked_by: String,
    /// Unique tag for this run (job id or CLI-generated).
    pub run_tag: String,
}

/// MV-018 outcome: recomputed canonical hash vs the stored edition hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalCheck {
    /// Edition URN checked.
    pub edition_urn: String,
    /// Stored `text_hash` the build expected.
    pub expected_hash: String,
    /// Recomputed hash actually found.
    pub actual_hash: String,
    /// Whether they agree.
    pub unchanged: bool,
}

/// Report for one completed rebuild.
#[derive(Debug, Clone)]
pub struct RebuildReport {
    /// Canonical edition id rebuilt.
    pub edition_id: String,
    /// Edition URN.
    pub edition_urn: String,
    /// Corpus generation stamped on every row.
    pub generation: i64,
    /// Profile ladder version stamped on every row.
    pub rule_set_version: String,
    /// Ayahs read.
    pub ayahs: usize,
    /// Tokens read.
    pub tokens: usize,
    /// Token-form rows written.
    pub token_forms: usize,
    /// Ayah-form rows written.
    pub ayah_forms: usize,
    /// Skeleton rows written (ayahs + windows).
    pub skeletons: usize,
    /// Layer-D provenance record id shared by all rows.
    pub provenance_id: String,
    /// MV-018 post-build check (always `unchanged` on success).
    pub mv018: CanonicalCheck,
}

/// Forms-rebuild failures. Codes reuse the `QAI-IDX` taxonomy owned by
/// `quran-search` (build infrastructure) and `QAI-NORM` for pipeline faults;
/// no new `QAI-QUR` codes are minted here (Phase-1-owned).
#[derive(Debug, Clone, thiserror::Error)]
pub enum FormsError {
    /// Storage failure.
    #[error("forms rebuild storage failed: {0}")]
    Storage(String),
    /// Normalization pipeline failure.
    #[error(transparent)]
    Normalization(#[from] quran_normalization::error::NormalizationError),
    /// Index-build failure, including MV-018 canonical drift.
    #[error(transparent)]
    Index(#[from] IndexError),
    /// Edition is not the active one (rebuilds serve the active generation).
    #[error("edition {slug}@{version} is not active; rebuilds target the active edition")]
    NotActive {
        /// Edition slug.
        slug: String,
        /// Edition version.
        version: String,
    },
    /// No such edition.
    #[error("no edition {slug}@{version}")]
    EditionNotFound {
        /// Edition slug.
        slug: String,
        /// Edition version.
        version: String,
    },
    /// Cancellation was requested mid-build; nothing was committed.
    #[error("forms rebuild cancelled")]
    Cancelled,
}

impl FormsError {
    fn storage(error: StorageError) -> Self {
        Self::Storage(error.to_string())
    }
}

impl storage::error::Diagnostic for FormsError {
    fn code(&self) -> storage::error::DiagnosticCode {
        match self {
            Self::Storage(_) => storage::error::DiagnosticCode::new("QAI-IDX", 3),
            Self::Normalization(inner) => {
                let rendered = inner.code().to_string();
                let number =
                    rendered.split('-').next_back().and_then(|n| n.parse().ok()).unwrap_or(3);
                storage::error::DiagnosticCode::new("QAI-NORM", number)
            }
            Self::Index(inner) => {
                let rendered = inner.code().to_string();
                let number =
                    rendered.split('-').next_back().and_then(|n| n.parse().ok()).unwrap_or(3);
                storage::error::DiagnosticCode::new("QAI-IDX", number)
            }
            Self::EditionNotFound { .. } | Self::NotActive { .. } => {
                storage::error::DiagnosticCode::new("QAI-IDX", 3)
            }
            Self::Cancelled => storage::error::DiagnosticCode::new("QAI-IDX", 3),
        }
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn remedy(&self) -> Option<String> {
        Some(match self {
            Self::Storage(_) => "Check the database and retry the rebuild.".to_string(),
            Self::Normalization(_) => {
                "Check the profile seed against the code, then retry.".to_string()
            }
            Self::Index(inner) => inner.remedy().unwrap_or_else(|| "See above.".to_string()),
            Self::EditionNotFound { .. } => "Import and activate the edition first.".to_string(),
            Self::NotActive { .. } => {
                "Rebuild the active edition, or activate this one first.".to_string()
            }
            Self::Cancelled => "Rerun the rebuild; it is idempotent.".to_string(),
        })
    }

    fn next_command(&self) -> Option<String> {
        Some("qai doctor --indexes".to_string())
    }
}

/// Recompute the canonical text hash for an edition (MV-018 probe).
///
/// Reads every ayah text in `(surah, ayah)` order and replays the exact
/// Phase-1 `text_hash` recipe. Pure read path: safe to call at any time.
pub async fn recompute_text_hash(
    db: &SqliteDatabase,
    edition_id: &str,
) -> Result<String, FormsError> {
    let mut uow = db.write().await.map_err(FormsError::storage)?;
    let (hash, _) = text_hash_in(&mut uow, edition_id).await?;
    uow.rollback().await.map_err(FormsError::storage)?;
    Ok(hash)
}

/// Read `(stored_hash, recomputed_hash, edition_urn)` inside a live unit of
/// work (no commit/rollback here; the caller owns the transaction).
async fn text_hash_in(
    uow: &mut Box<dyn storage::UnitOfWork>,
    edition_id: &str,
) -> Result<(String, CanonicalCheck), FormsError> {
    let edition = uow.quran().get_edition(edition_id).await.map_err(FormsError::storage)?;
    let Some(edition) = edition else {
        return Err(FormsError::EditionNotFound {
            slug: edition_id.to_string(),
            version: String::new(),
        });
    };
    let mut ayahs =
        uow.quran().list_ayahs_range(edition_id, 1, i64::MAX).await.map_err(FormsError::storage)?;
    ayahs.sort_by_key(|a| (a.surah, a.ayah));
    let texts: Vec<&str> = ayahs.iter().map(|a| a.text.as_str()).collect();
    let hash = quran_corpus::hashing::text_hash(&edition.slug, &edition.version, &texts);
    let actual_hash = quran_corpus::hashing::tagged(&hash);
    let check = CanonicalCheck {
        edition_urn: format!("quran-edition:{}@{}", edition.slug, edition.version),
        expected_hash: edition.text_hash.clone(),
        actual_hash: actual_hash.clone(),
        unchanged: edition.text_hash == actual_hash,
    };
    Ok((actual_hash, check))
}

/// MV-018: prove canonical text is byte-identical to the stored edition hash.
///
/// # Errors
///
/// Returns [`IndexError::CanonicalChanged`] (`QAI-IDX-0005`) on any drift —
/// the caller must stop the build, never warn-and-continue.
pub async fn verify_canonical_unchanged(
    db: &SqliteDatabase,
    edition_id: &str,
) -> Result<CanonicalCheck, FormsError> {
    let mut uow = db.write().await.map_err(FormsError::storage)?;
    let check = verify_canonical_unchanged_in(&mut uow, edition_id).await?;
    uow.rollback().await.map_err(FormsError::storage)?;
    Ok(check)
}

/// MV-018 inside a live transaction (for the post-build check before commit).
///
/// # Errors
///
/// Same contract as [`verify_canonical_unchanged`]; performs no commit or
/// rollback itself.
pub async fn verify_canonical_unchanged_in(
    uow: &mut Box<dyn storage::UnitOfWork>,
    edition_id: &str,
) -> Result<CanonicalCheck, FormsError> {
    let (_, check) = text_hash_in(uow, edition_id).await?;
    if !check.unchanged {
        return Err(FormsError::Index(IndexError::CanonicalChanged {
            edition_urn: check.edition_urn.clone(),
            expected_hash: check.expected_hash.clone(),
            actual_hash: check.actual_hash.clone(),
        }));
    }
    Ok(check)
}
/// Ladder version stamped on rows: the max over contributing profiles.
///
/// Profiles version independently, but one rebuild mixes their outputs in a
/// single row set — the max pins the newest input so drift reports fire when
/// any contributor moves.
fn ladder_version(registry: &quran_normalize::ProfileRegistry) -> quran_normalization::SemVer {
    use quran_normalization::ProfileId;
    let mut version = quran_normalization::SemVer::new(0, 0, 0);
    for id in
        [ProfileId::L2, ProfileId::L3, ProfileId::L4, ProfileId::L5, ProfileId::L6, ProfileId::L7]
    {
        if let Ok(profile) = registry.latest(id) {
            version = version.max(profile.version);
        }
    }
    version
}

/// Max implementation version over the rules a registry actually uses.
fn impl_version(registry: &quran_normalize::ProfileRegistry) -> quran_normalization::SemVer {
    use quran_normalization::ProfileId;
    let mut version = quran_normalization::SemVer::new(0, 0, 0);
    for id in ProfileId::all() {
        if let Ok(profile) = registry.latest(id) {
            for rule in &profile.rules {
                if let Some(implementation) = quran_normalize::rule_impl(*rule) {
                    version = version.max(implementation.version());
                }
            }
        }
    }
    version
}

/// Canonical descriptor hashed into the provenance `parameters_hash`.
fn parameters_descriptor(registry: &quran_normalize::ProfileRegistry) -> String {
    use quran_normalization::ProfileId;
    let mut parts = Vec::new();
    for id in ProfileId::all() {
        if let Ok(profile) = registry.latest(id) {
            let rules = profile.rules.iter().map(|r| r.as_str()).collect::<Vec<_>>().join(",");
            parts.push(format!("{}@{}:[{}]", id, profile.version, rules));
        }
    }
    parts.join(";")
}

/// Rebuild all derived forms for the active edition (P2-T26).
///
/// Stages, in order: resolve + active check → MV-018 pre-check → load →
/// per-surah build (cancel-checked) → single write transaction (principal,
/// wipe, provenance, inserts, MV-018 post-check) → commit. Nothing commits
/// on failure or cancellation.
pub async fn rebuild_forms(
    db: &SqliteDatabase,
    params: &RebuildParams,
    cancel: &AtomicBool,
    checkpoint: impl Fn(&str),
) -> Result<RebuildReport, FormsError> {
    use quran_normalization::ProfileId;

    // 1. Resolve the edition and require it to be active.
    let (edition_id, generation, edition_urn) = {
        let mut uow = db.write().await.map_err(FormsError::storage)?;
        let edition = uow
            .quran()
            .get_edition_by_slug_version(&params.edition_slug, &params.edition_version)
            .await
            .map_err(FormsError::storage)?;
        let Some(edition) = edition else {
            return Err(FormsError::EditionNotFound {
                slug: params.edition_slug.clone(),
                version: params.edition_version.clone(),
            });
        };
        let active = uow.quran().get_active().await.map_err(FormsError::storage)?;
        uow.rollback().await.map_err(FormsError::storage)?;
        match active {
            Some(active) if active.edition_id == edition.id => (
                edition.id.clone(),
                active.corpus_generation,
                format!("quran-edition:{}@{}", edition.slug, edition.version),
            ),
            _ => {
                return Err(FormsError::NotActive {
                    slug: params.edition_slug.clone(),
                    version: params.edition_version.clone(),
                });
            }
        }
    };
    checkpoint("resolved");

    // 2. MV-018 pre-check: stop before ANY write on drift.
    verify_canonical_unchanged(db, &edition_id).await?;
    checkpoint("mv018-pre");

    // 3. Load canonical rows (read-only) grouped by surah.
    let surahs: Vec<(i64, Vec<storage::quran::AyahRow>)> = {
        let mut uow = db.write().await.map_err(FormsError::storage)?;
        let ayahs = uow
            .quran()
            .list_ayahs_range(&edition_id, 1, i64::MAX)
            .await
            .map_err(FormsError::storage)?;
        uow.rollback().await.map_err(FormsError::storage)?;
        let mut ordered = ayahs;
        ordered.sort_by_key(|a| (a.surah, a.ayah));
        let mut groups: Vec<(i64, Vec<storage::quran::AyahRow>)> = Vec::new();
        for ayah in ordered {
            match groups.last_mut() {
                Some((surah, rows)) if *surah == ayah.surah => rows.push(ayah),
                _ => groups.push((ayah.surah, vec![ayah])),
            }
        }
        groups
    };
    checkpoint("loaded");

    // 4. Pipelines from the seeded definitions (proves the seed per build).
    let profile_rows = {
        let mut uow = db.write().await.map_err(FormsError::storage)?;
        let rows = uow.quran().list_normalization_profiles().await.map_err(FormsError::storage)?;
        uow.rollback().await.map_err(FormsError::storage)?;
        rows
    };
    let registry =
        quran_normalize::registry_from_rows(&profile_rows).map_err(FormsError::Normalization)?;
    let ladder = ladder_version(&registry);
    let rule_set_version = ladder.to_string();
    let pipe = |id: ProfileId| {
        let profile = registry.latest(id).map_err(FormsError::Normalization)?;
        quran_normalization::NormalizationPipeline::for_profile(
            &registry,
            profile.id,
            profile.version,
        )
        .map_err(FormsError::Normalization)
    };
    let (p_l2, p_l3, p_l4, p_l5, p_l7) = (
        pipe(ProfileId::L2)?,
        pipe(ProfileId::L3)?,
        pipe(ProfileId::L4)?,
        pipe(ProfileId::L5)?,
        pipe(ProfileId::L7)?,
    );
    let impl_v = impl_version(&registry).to_string();

    // 5. Per-surah build (cancel-checked between surahs).
    let mut token_rows: Vec<storage::quran::TokenFormRow> = Vec::new();
    let mut ayah_rows: Vec<storage::quran::AyahFormRow> = Vec::new();
    let mut skeleton_rows: Vec<storage::quran::SkeletonRow> = Vec::new();
    for (surah, ayahs) in &surahs {
        if cancel.load(Ordering::SeqCst) {
            return Err(FormsError::Cancelled);
        }
        // Skeletons from joined raw texts (boundary-correct normalization).
        let pairs: Vec<(u32, String)> =
            ayahs.iter().map(|a| (a.ayah as u32, a.text.clone())).collect();
        for skeleton in quran_search::skeletons_for_surah(&pairs) {
            skeleton_rows.push(storage::quran::SkeletonRow {
                edition_id: edition_id.clone(),
                surah: *surah,
                ayah_start: skeleton.ayah_start as i64,
                ayah_end: skeleton.ayah_end as i64,
                skeleton: skeleton.skeleton,
                rule_set_id: RULE_SET_ID.to_string(),
                rule_set_version: rule_set_version.clone(),
                corpus_generation: generation,
                provenance_id: String::new(),
            });
        }
        for ayah in ayahs {
            let simple = p_l2.apply(&ayah.text).0;
            let bare = p_l3.apply(&ayah.text).0;
            let hamza_folded = p_l4.apply(&ayah.text).0;
            let folded = p_l5.apply(&ayah.text).0;
            ayah_rows.push(storage::quran::AyahFormRow {
                edition_id: edition_id.clone(),
                surah: ayah.surah,
                ayah: ayah.ayah,
                simple: simple.text().to_string(),
                bare: bare.text().to_string(),
                hamza_folded: hamza_folded.text().to_string(),
                folded: folded.text().to_string(),
                transliteration: None,
                rule_set_id: RULE_SET_ID.to_string(),
                rule_set_version: rule_set_version.clone(),
                corpus_generation: generation,
                provenance_id: String::new(),
            });
            let tokens = {
                let mut uow = db.write().await.map_err(FormsError::storage)?;
                let tokens = uow
                    .quran()
                    .get_tokens(&edition_id, ayah.surah, ayah.ayah)
                    .await
                    .map_err(FormsError::storage)?;
                uow.rollback().await.map_err(FormsError::storage)?;
                tokens
            };
            let mut ordered = tokens;
            ordered.sort_by_key(|t| t.position);
            for token in &ordered {
                let simple = p_l2.apply(&token.surface).0;
                let bare = p_l3.apply(&token.surface).0;
                let hamza_folded = p_l4.apply(&token.surface).0;
                let folded = p_l5.apply(&token.surface).0;
                let affix = p_l7.apply(&token.surface).0;
                token_rows.push(storage::quran::TokenFormRow {
                    edition_id: edition_id.clone(),
                    surah: ayah.surah,
                    ayah: ayah.ayah,
                    position: token.position,
                    simple: simple.text().to_string(),
                    bare: bare.text().to_string(),
                    hamza_folded: hamza_folded.text().to_string(),
                    folded: folded.text().to_string(),
                    affix_stripped: affix.text().to_string(),
                    transliteration: None,
                    phonetic: None,
                    rule_set_id: RULE_SET_ID.to_string(),
                    rule_set_version: rule_set_version.clone(),
                    corpus_generation: generation,
                    provenance_id: String::new(),
                });
            }
        }
    }
    checkpoint("forms");
    // 6. Provenance identity: content-addressed over (edition, generation,
    //    derivation descriptor), so retries and duplicate builds converge on
    //    one record instead of conflicting. The run tag stays a payload
    //    field for job correlation; identity comes from content.
    let parameters_descriptor = parameters_descriptor(&registry);
    let parameters_hash =
        format!("sha256:{}", quran_corpus::sha256_hex(parameters_descriptor.as_bytes()));
    let slug = params.edition_slug.clone();
    let version = params.edition_version.clone();
    let id_seed = format!("{edition_urn}|{generation}|{parameters_hash}");
    let short = &quran_corpus::sha256_hex(id_seed.as_bytes())[..12];
    let provenance_id = format!("prov-forms-{slug}-{version}-g{generation}-{short}");
    for row in &mut token_rows {
        row.provenance_id = provenance_id.clone();
    }
    for row in &mut ayah_rows {
        row.provenance_id = provenance_id.clone();
    }
    for row in &mut skeleton_rows {
        row.provenance_id = provenance_id.clone();
    }
    let record = storage::repository::ProvenanceRecord {
        id: provenance_id.clone(),
        layer: "computational_annotation".to_string(),
        subject_urn: format!("{edition_urn}/derived-forms"),
        attribution_kind: "computational".to_string(),
        attribution_json: serde_json::json!({
            "algorithm": "normalizer",
            "version": impl_v,
            "parameters_hash": parameters_hash,
        })
        .to_string(),
        source_version_id: None,
        trust_level: "ComputedUnverified".to_string(),
        verification_status: "unverified".to_string(),
        confidence: Some(1.0),
        versions_json: serde_json::json!({
            "rule_set_id": RULE_SET_ID,
            "rule_set_version": rule_set_version,
            "corpus_generation": generation,
        })
        .to_string(),
        created_by: params.invoked_by.clone(),
    };

    // 7. Single write transaction: principal, wipe, provenance, inserts,
    //    MV-018 post-check, commit.
    let at = domain::Timestamp::now().to_string();
    crate::quran::ensure_principal(db, &params.invoked_by, &params.invoked_by, &at)
        .await
        .map_err(FormsError::storage)?;
    let token_total = token_rows.len();
    let ayah_total = ayah_rows.len();
    let skeleton_total = skeleton_rows.len();
    let mut uow = db.write().await.map_err(FormsError::storage)?;
    uow.quran().delete_forms_for_edition(&edition_id).await.map_err(FormsError::storage)?;
    // Content-addressed id: a retry of the same derivation reuses the
    // existing record instead of conflicting.
    match uow.provenance().insert(record).await {
        Ok(()) => {}
        Err(StorageError::Conflict) => {}
        Err(error) => return Err(FormsError::storage(error)),
    }
    uow.quran().insert_token_forms(token_rows).await.map_err(FormsError::storage)?;
    uow.quran().insert_ayah_forms(ayah_rows).await.map_err(FormsError::storage)?;
    uow.quran().insert_skeletons(skeleton_rows).await.map_err(FormsError::storage)?;
    checkpoint("staged");
    // MV-018 post-check INSIDE the transaction: the job wrote derived rows
    // only, so any drift here aborts the commit.
    let post = verify_canonical_unchanged_in(&mut uow, &edition_id).await?;
    uow.commit().await.map_err(FormsError::storage)?;
    checkpoint("committed");

    Ok(RebuildReport {
        edition_id,
        edition_urn,
        generation,
        ayahs: ayah_total,
        tokens: token_total,
        token_forms: token_total,
        ayah_forms: ayah_total,
        skeletons: skeleton_total,
        provenance_id,
        rule_set_version,
        mv018: post,
    })
}

/// Payload for `quran.forms.rebuild` jobs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FormsRebuildPayload {
    /// Edition slug (must be active).
    pub edition_slug: String,
    /// Edition version (must be active).
    pub edition_version: String,
    /// Operator principal id.
    pub invoked_by: String,
    /// Unique run tag (defaults to the job id).
    pub run_tag: Option<String>,
}

/// The `quran.forms.rebuild` job handler.
pub struct FormsRebuildHandler {
    db: Arc<SqliteDatabase>,
}

impl FormsRebuildHandler {
    /// Wrap a database handle.
    pub fn new(db: Arc<SqliteDatabase>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl jobs::JobHandler for FormsRebuildHandler {
    fn kind(&self) -> jobs::JobKind {
        QURAN_FORMS_REBUILD_KIND.into()
    }

    fn payload_schema(&self) -> &'static str {
        r#"{"type":"object","required":["edition_slug","edition_version","invoked_by"]}"#
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
        let input: FormsRebuildPayload = serde_json::from_value(payload)
            .map_err(|err| JobError::Storage(format!("bad forms-rebuild payload: {err}")))?;
        let params = RebuildParams {
            edition_slug: input.edition_slug,
            edition_version: input.edition_version,
            invoked_by: input.invoked_by,
            run_tag: input.run_tag.unwrap_or_else(|| ctx.job.id.clone()),
        };
        let cancel = ctx.cancel_flag();
        let sink = ctx.checkpoint_sink();
        let report = rebuild_forms(&self.db, &params, &cancel, |stage| {
            if let Ok(mut slot) = sink.lock() {
                *slot = Some(stage.to_string());
            }
        })
        .await;
        match report {
            Ok(report) => Ok(jobs::JobOutcome {
                success: true,
                result: Some(format!(
                    "forms rebuilt for {}: {} ayahs, {} tokens, {} skeletons (generation {})",
                    report.edition_urn,
                    report.ayahs,
                    report.tokens,
                    report.skeletons,
                    report.generation
                )),
            }),
            Err(FormsError::Cancelled) => Err(JobError::Cancelled { id: ctx.job.id.clone() }),
            Err(err) => Err(JobError::Storage(format!("{}: {}", err.code(), err.summary()))),
        }
    }
}
