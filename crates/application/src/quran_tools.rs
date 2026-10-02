//! Tool and citation backends over the deterministic reader (D1.8, D1.9).
//!
//! # application::quran_tools
//!
//! Implements `tool_registry::QuranBackend` and `citations::CitationSource`
//! for [`QuranReaderService`](crate::quran_reader::QuranReaderService), plus
//! citation persistence through the `citations` table. Tools stay read-only;
//! every result carries edition identity, canonical references, and a
//! deterministic reproducibility checksum.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use quran_core::{
    AyahNumber, AyahOptions, AyahView, ContextSpec, ContextView, EditionSelector, QuranEdition,
    QuranRef, SurahNumber,
};
use storage::Database as _;
use tool_registry::{
    BackendMeta, FAMILY_TOOL_VERSION, FamilyToolParams, GetAyahParams, GetContextParams,
    LEMMA_TOOL_VERSION, LemmaToolParams, MORPHOLOGY_TOOL_VERSION, MorphologyToolParams,
    QuranBackend, ROOT_TOOL_VERSION, RootToolParams, SEARCH_TOOL_VERSION, SearchToolParams,
    ToolRegistry,
};
use tools::{AnalysisSource, ToolError, ToolResult, reproducibility};

use crate::quran_reader::{QuranReader, QuranReaderService, ReaderError};

/// Map a reader error to a tool error (codes delegated for HTTP mapping).
pub fn to_tool_error(error: ReaderError) -> ToolError {
    use storage::error::Diagnostic;
    ToolError::Backend { code: error.code().to_string(), detail: error.to_string() }
}

fn tool_error(error: ReaderError) -> ToolError {
    to_tool_error(error)
}

/// Map a search failure to a typed tool backend error (codes delegated).
fn search_tool_error(error: crate::quran_search::SearchError) -> ToolError {
    use storage::error::Diagnostic;
    ToolError::Backend { code: error.code().to_string(), detail: error.to_string() }
}

/// Map a morphology/lexicon failure to a typed tool backend error. An
/// unavailable dataset is `QAI-MORPH-0004`, never an empty result.
fn morphology_tool_error(error: crate::quran_morphology::MorphologyToolError) -> ToolError {
    use storage::error::Diagnostic;
    ToolError::Backend { code: error.code().to_string(), detail: error.to_string() }
}

/// Build read metadata for one edition row.
async fn meta_from_edition(
    reader: &QuranReaderService,
    edition: &QuranEdition,
) -> Result<BackendMeta, ToolError> {
    let mut uow = reader.database().write().await.map_err(|err| ToolError::Backend {
        code: "QAI-QUR-0310".into(),
        detail: err.to_string(),
    })?;
    let generation = uow.quran().get_active().await.map_err(|err| ToolError::Backend {
        code: "QAI-QUR-0310".into(),
        detail: err.to_string(),
    })?;
    uow.rollback().await.map_err(|err| ToolError::Backend {
        code: "QAI-QUR-0310".into(),
        detail: err.to_string(),
    })?;
    Ok(BackendMeta {
        edition_version: edition.version.to_string(),
        edition_slug: edition.slug.clone(),
        edition_id: edition.id.to_string(),
        corpus_generation: generation.map(|row| row.corpus_generation as u64).unwrap_or(0),
        text_hash: edition.text_hash.hex.clone(),
        script: match &edition.script {
            quran_core::Script::Uthmani => "uthmani".to_string(),
            quran_core::Script::ImlaeiSimple => "imlaei_simple".to_string(),
            quran_core::Script::Other(name) => format!("other:{name}"),
        },
        riwayah: edition.riwayah.clone(),
        numbering_scheme: match &edition.verse_numbering_scheme {
            quran_core::NumberingScheme::Hafs => "hafs".to_string(),
            quran_core::NumberingScheme::Kufi => "kufi".to_string(),
            quran_core::NumberingScheme::Custom(name) => name.clone(),
        },
        // Reader tools serve canonical text, not a graph projection.
        projection_id: String::new(),
        builder_version: String::new(),
    })
}

async fn backend_meta(
    reader: &QuranReaderService,
    reference: &QuranRef,
) -> Result<BackendMeta, ToolError> {
    let edition = reader.get_edition(reference.edition()).await.map_err(tool_error)?;
    meta_from_edition(reader, &edition).await
}

/// Read metadata for the active edition (the lexicon tools' read context).
async fn active_meta(reader: &QuranReaderService) -> Result<BackendMeta, ToolError> {
    let edition = reader.get_edition(&EditionSelector::Active).await.map_err(tool_error)?;
    meta_from_edition(reader, &edition).await
}

/// Map a `quran.search` edition request to a selector (WR-02).
///
/// The tool accepts an optional `slug` or `slug@version`; a bare slug resolves
/// at its active version, an explicit version pins it.
fn search_edition_selector(spec: &str) -> Result<EditionSelector, ToolError> {
    match spec.split_once('@') {
        Some((slug, version)) if !slug.is_empty() && !version.is_empty() => {
            let version = version.parse().map_err(|_| ToolError::InvalidInput {
                tool: "quran.search",
                detail: format!("edition `{spec}` is not a valid slug@version"),
            })?;
            Ok(EditionSelector::Pinned { slug: slug.to_string(), version })
        }
        _ => Ok(EditionSelector::Slug(spec.to_string())),
    }
}

/// Read metadata for the edition a search request selects (WR-02).
///
/// `quran.search` may be asked for a non-active edition; the envelope must
/// name the same edition the hits come from, never the active pointer.
async fn search_meta(
    reader: &QuranReaderService,
    edition: Option<&str>,
) -> Result<BackendMeta, ToolError> {
    match edition {
        None => active_meta(reader).await,
        Some(spec) => {
            let selector = search_edition_selector(spec)?;
            let edition = reader.get_edition(&selector).await.map_err(tool_error)?;
            meta_from_edition(reader, &edition).await
        }
    }
}

/// A pinned `quran:<slug>@<version>:<surah>:<ayah>` reference.
fn pinned_ref(meta: &BackendMeta, surah: i64, ayah: i64) -> String {
    format!("quran:{}@{}:{surah}:{ayah}", meta.edition_slug, meta.edition_version)
}

/// One `dataset` analysis source for a lexicon result.
fn dataset_source(dataset: &str) -> Vec<AnalysisSource> {
    vec![AnalysisSource { kind: "dataset".to_string(), reference: dataset.to_string() }]
}

/// The default index root used by the reader-only constructors (matches
/// [`crate::quran_index::index_root_for_db`]'s fallback). The direct-read tools
/// never read it.
fn default_index_root() -> std::path::PathBuf {
    std::path::PathBuf::from("index")
}

/// The tool backend: [`QuranReaderService`] behind the registry trait.
pub struct ReaderToolBackend {
    reader: Arc<QuranReaderService>,
    /// Serving-index root; only `quran.search` reads it.
    index_root: std::path::PathBuf,
}

impl ReaderToolBackend {
    /// Wrap a reader (direct-read tools; the index root is only used by
    /// `quran.search`).
    pub fn new(reader: Arc<QuranReaderService>) -> Self {
        Self { reader, index_root: default_index_root() }
    }

    /// Wrap a reader with an explicit serving-index root (D-13 search tool).
    pub fn with_index_root(
        reader: Arc<QuranReaderService>,
        index_root: std::path::PathBuf,
    ) -> Self {
        Self { reader, index_root }
    }

    /// A registry wired to the reader (direct-read tools).
    pub fn registry(reader: Arc<QuranReaderService>) -> ToolRegistry {
        ToolRegistry::new(Arc::new(Self::new(reader)))
    }

    /// A registry wired to the reader and its serving-index root: the full
    /// D-13 tool surface (search + lexicon tools included).
    pub fn registry_with_index_root(
        reader: Arc<QuranReaderService>,
        index_root: std::path::PathBuf,
    ) -> ToolRegistry {
        ToolRegistry::new(Arc::new(Self::with_index_root(reader, index_root)))
    }
}

#[async_trait]
impl QuranBackend for ReaderToolBackend {
    async fn backend_get_ayah(
        &self,
        reference: &QuranRef,
        options: &AyahOptions,
    ) -> Result<(AyahView, BackendMeta), ToolError> {
        let view = self.reader.get_ayah(reference, options).await.map_err(tool_error)?;
        let meta = backend_meta(&self.reader, reference).await?;
        Ok((view, meta))
    }

    async fn backend_get_context(
        &self,
        reference: &QuranRef,
        spec: &ContextSpec,
    ) -> Result<(ContextView, BackendMeta), ToolError> {
        let view = self.reader.get_context(reference, spec).await.map_err(tool_error)?;
        let meta = backend_meta(&self.reader, reference).await?;
        Ok((view, meta))
    }

    /// `quran.search` (D-13): normalized L3 search over the serving index. The
    /// envelope carries the hit trace's rule ids (I9) and canonical attribution;
    /// an empty query never reaches here (the registry guard rejects it).
    async fn backend_search(
        &self,
        params: &SearchToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let started = Instant::now();
        // WR-02: attribute the envelope to the edition the search will serve.
        let meta = search_meta(&self.reader, params.edition.as_deref()).await?;
        let search_params = crate::quran_search::SearchParams {
            text: params.text.clone(),
            edition: params.edition.clone(),
            mode: crate::quran_search::MatchMode::WholeToken,
            filters: Vec::new(),
            limit: params.limit.unwrap_or(20).clamp(1, 1000),
            offset: 0,
            explain: false,
            highlight: false,
        };
        let output = crate::quran_search::search_normalized(
            self.reader.database(),
            &self.index_root,
            &search_params,
            crate::quran_search::NormalizedProfile::Registry(
                quran_normalization::ProfileId::L3,
                None,
            ),
        )
        .await
        .map_err(search_tool_error)?;
        let normalization_rules: Vec<String> = output
            .hits
            .first()
            .map(|hit| {
                hit.explanation().rule_ids().iter().map(|rule| rule.as_str().to_string()).collect()
            })
            .unwrap_or_default();
        let canonical_references: Vec<String> =
            output.hits.iter().map(|hit| hit.reference().to_string()).collect();
        let analysis_sources = canonical_references
            .iter()
            .map(|reference| AnalysisSource {
                kind: "canonical".to_string(),
                reference: reference.clone(),
            })
            .collect();
        let query = serde_json::to_value(params).unwrap_or(serde_json::Value::Null);
        Ok(ToolResult {
            tool_name: "quran.search".to_string(),
            tool_version: SEARCH_TOOL_VERSION,
            query: query.clone(),
            normalization_rules,
            edition_id: Some(meta.edition_id.clone()),
            edition_version: Some(meta.edition_version.clone()),
            canonical_references,
            analysis_sources,
            results: serde_json::to_value(&output).unwrap_or(serde_json::Value::Null),
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                "quran.search",
                SEARCH_TOOL_VERSION,
                &query,
                Some(&meta.edition_slug),
                Some(&meta.edition_version),
                BTreeMap::new(),
                meta.corpus_generation,
            ),
        })
    }

    /// `quran.root` (D-13): root-grouped occurrences from the active dataset,
    /// attributed per dataset (no attribution means no result).
    async fn backend_root(
        &self,
        params: &RootToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let started = Instant::now();
        let meta = active_meta(&self.reader).await?;
        let (dataset_id, occurrences) =
            crate::quran_morphology::root_search(self.reader.database(), &params.root)
                .await
                .map_err(morphology_tool_error)?;
        let canonical_references: Vec<String> =
            occurrences.iter().map(|occ| pinned_ref(&meta, occ.surah, occ.ayah)).collect();
        let query = serde_json::to_value(params).unwrap_or(serde_json::Value::Null);
        Ok(ToolResult {
            tool_name: "quran.root".to_string(),
            tool_version: ROOT_TOOL_VERSION,
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: Some(meta.edition_id.clone()),
            edition_version: Some(meta.edition_version.clone()),
            canonical_references,
            analysis_sources: dataset_source(&dataset_id),
            results: serde_json::to_value(&occurrences).unwrap_or(serde_json::Value::Null),
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                "quran.root",
                ROOT_TOOL_VERSION,
                &query,
                Some(&meta.edition_slug),
                Some(&meta.edition_version),
                BTreeMap::new(),
                meta.corpus_generation,
            ),
        })
    }

    /// `quran.lemma` (D-13): lemma-grouped occurrences from the active dataset.
    async fn backend_lemma(
        &self,
        params: &LemmaToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let started = Instant::now();
        let meta = active_meta(&self.reader).await?;
        let (dataset_id, occurrences) =
            crate::quran_morphology::lemma_search(self.reader.database(), &params.lemma)
                .await
                .map_err(morphology_tool_error)?;
        let canonical_references: Vec<String> =
            occurrences.iter().map(|occ| pinned_ref(&meta, occ.surah, occ.ayah)).collect();
        let query = serde_json::to_value(params).unwrap_or(serde_json::Value::Null);
        Ok(ToolResult {
            tool_name: "quran.lemma".to_string(),
            tool_version: LEMMA_TOOL_VERSION,
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: Some(meta.edition_id.clone()),
            edition_version: Some(meta.edition_version.clone()),
            canonical_references,
            analysis_sources: dataset_source(&dataset_id),
            results: serde_json::to_value(&occurrences).unwrap_or(serde_json::Value::Null),
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                "quran.lemma",
                LEMMA_TOOL_VERSION,
                &query,
                Some(&meta.edition_slug),
                Some(&meta.edition_version),
                BTreeMap::new(),
                meta.corpus_generation,
            ),
        })
    }

    /// `quran.morphology` (D-13): every analysis of one token, attributed.
    async fn backend_morphology(
        &self,
        params: &MorphologyToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let started = Instant::now();
        let meta = active_meta(&self.reader).await?;
        let (dataset_id, analyses) = crate::quran_morphology::morphology_for_token(
            self.reader.database(),
            &meta.edition_slug,
            &meta.edition_version,
            i64::from(params.surah),
            i64::from(params.ayah),
            i64::from(params.position),
        )
        .await
        .map_err(morphology_tool_error)?;
        let canonical_references =
            vec![pinned_ref(&meta, i64::from(params.surah), i64::from(params.ayah))];
        let query = serde_json::to_value(params).unwrap_or(serde_json::Value::Null);
        Ok(ToolResult {
            tool_name: "quran.morphology".to_string(),
            tool_version: MORPHOLOGY_TOOL_VERSION,
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: Some(meta.edition_id.clone()),
            edition_version: Some(meta.edition_version.clone()),
            canonical_references,
            analysis_sources: dataset_source(&dataset_id),
            results: serde_json::to_value(&analyses).unwrap_or(serde_json::Value::Null),
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                "quran.morphology",
                MORPHOLOGY_TOOL_VERSION,
                &query,
                Some(&meta.edition_slug),
                Some(&meta.edition_version),
                BTreeMap::new(),
                meta.corpus_generation,
            ),
        })
    }

    /// `quran.family` (D-13): explained family relations, attributed per dataset.
    async fn backend_family(
        &self,
        params: &FamilyToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let started = Instant::now();
        let meta = active_meta(&self.reader).await?;
        let (dataset_id, members) =
            crate::quran_morphology::word_family(self.reader.database(), &params.kind, &params.id)
                .await
                .map_err(morphology_tool_error)?;
        let query = serde_json::to_value(params).unwrap_or(serde_json::Value::Null);
        Ok(ToolResult {
            tool_name: "quran.family".to_string(),
            tool_version: FAMILY_TOOL_VERSION,
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: Some(meta.edition_id.clone()),
            edition_version: Some(meta.edition_version.clone()),
            canonical_references: Vec::new(),
            analysis_sources: dataset_source(&dataset_id),
            results: serde_json::to_value(&members).unwrap_or(serde_json::Value::Null),
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                "quran.family",
                FAMILY_TOOL_VERSION,
                &query,
                Some(&meta.edition_slug),
                Some(&meta.edition_version),
                BTreeMap::new(),
                meta.corpus_generation,
            ),
        })
    }
}

/// The citation source: [`QuranReaderService`] behind the resolver trait.
///
/// # Verification scope (D-15, Pitfall 4)
///
/// `verify_quotation` is meaningful only for an *externally supplied*
/// quotation. The direct-read answer paths are **structurally exempt** because
/// they serve canonical text straight from the source of truth and therefore
/// cannot mismatch by construction — wrapping them would compare canonical text
/// against itself and prove nothing:
///
/// - tool-result reads: `quran.get_ayah` / `quran.get_context`
///   ([`ReaderToolBackend`], this file);
/// - CLI direct reads: `cmd_get` / `cmd_context` / `cmd_surah` / `cmd_division`
///   (`crates/application/src/quran_cli.rs`);
/// - HTTP direct reads: the ayah/context/token handlers backed by
///   `ReaderBackend` (`crates/server/src/api.rs`).
///
/// The verifying surfaces that do apply this mapping are
/// [`verify_canonical_quotation`] (used by `qai quran verify-quotation`) and the
/// HTTP stored-verdict path (`GET /api/v1/quran/citations/{id}`, which resolves
/// a persisted citation and enforces [`citations::require_exact`]).
pub struct ReaderCitationSource {
    reader: Arc<QuranReaderService>,
}

impl ReaderCitationSource {
    /// Wrap a reader.
    pub fn new(reader: Arc<QuranReaderService>) -> Self {
        Self { reader }
    }

    /// A resolver wired to the reader.
    ///
    /// The resolver verifies an externally supplied quotation; see the
    /// [`ReaderCitationSource`] doc for the list of direct-read paths that are
    /// structurally exempt from `verify_quotation`.
    pub fn resolver(reader: Arc<QuranReaderService>) -> citations::CitationResolver {
        citations::CitationResolver::new(Arc::new(Self::new(reader)))
    }
}

#[async_trait]
impl citations::CitationSource for ReaderCitationSource {
    async fn edition_exists(
        &self,
        slug: &str,
        version: &str,
    ) -> Result<bool, citations::CitationError> {
        use quran_core::EditionSelector;
        let selector = EditionSelector::Pinned {
            slug: slug.to_string(),
            version: version.parse().map_err(|_| citations::CitationError::Backend {
                detail: format!("bad version `{version}`"),
            })?,
        };
        match self.reader.get_edition(&selector).await {
            Ok(_) => Ok(true),
            Err(ReaderError::EditionNotFound(_)) => Ok(false),
            Err(err) => Err(citations::CitationError::Backend { detail: err.to_string() }),
        }
    }

    async fn fetch_ayah_text(
        &self,
        slug: &str,
        version: &str,
        surah: u16,
        ayah: u32,
    ) -> Result<Option<String>, citations::CitationError> {
        use quran_core::{AyahNumber, EditionSelector, SurahNumber};
        let reference = QuranRef::Ayah {
            edition: EditionSelector::Pinned {
                slug: slug.to_string(),
                version: version.parse().map_err(|_| citations::CitationError::Backend {
                    detail: format!("bad version `{version}`"),
                })?,
            },
            surah: SurahNumber::new(surah).map_err(|_| {
                citations::CitationError::InvalidReference {
                    reference: format!("quran:{slug}@{version}:{surah}:{ayah}"),
                }
            })?,
            ayah: AyahNumber::new(ayah).map_err(|_| {
                citations::CitationError::InvalidReference {
                    reference: format!("quran:{slug}@{version}:{surah}:{ayah}"),
                }
            })?,
        };
        match self.reader.get_ayah(&reference, &AyahOptions::default()).await {
            Ok(view) => Ok(Some(view.canonical.arabic_text().to_string())),
            Err(ReaderError::AyahNotFound(_)) | Err(ReaderError::EditionNotFound(_)) => Ok(None),
            Err(err) => Err(citations::CitationError::Backend { detail: err.to_string() }),
        }
    }
}

/// Persist a resolved citation for later re-verification (§12.1, §21.2).
pub async fn persist_citation(
    db: &dyn storage::Database,
    resolved: &citations::ResolvedCitation,
    citation: &citations::Citation,
    ingestion_version: &str,
    at: &str,
) -> Result<(), storage::error::StorageError> {
    let mut uow = db.write().await?;
    uow.quran()
        .insert_citation(storage::quran::CitationRow {
            id: citation.id.clone(),
            kind: "quran".to_string(),
            canonical_reference: citation.canonical_reference.clone(),
            source_id: None,
            source_version_id: None,
            edition_ref: Some(format!("{}@{}", citation.edition_slug, citation.edition_version)),
            location_json: serde_json::json!({
                "surah": citation.surah.get(),
                "ayah": citation.ayah.get(),
            })
            .to_string(),
            quoted_text_hash: resolved.text_hash.clone(),
            ingestion_version: ingestion_version.to_string(),
            resolved_at: at.to_string(),
            verdict: verdict_str(&resolved.verdict),
        })
        .await?;
    uow.commit().await
}

fn verdict_str(verdict: &citations::QuotationVerdict) -> String {
    verdict.label().to_string()
}

/// Verify an externally supplied quotation against the canonical source of
/// truth, returning the verdict and the RESOLVED canonical text hash.
///
/// This is the one shared verifying implementation behind every quoted-text
/// answer path (the `qai quran verify-quotation` CLI verb and the HTTP path).
/// The citation is built with the pinned `slug@version`, so the edition is
/// never inferred from the supplied text (ADR-0111). The returned hash is the
/// one read from the canonical row by the reader-backed resolver — it is never
/// recomputed from the supplied text, so a caller can never present a hash it
/// synthesized (T-02-19). A hard-failure verdict is mapped through
/// [`citations::require_exact`], so `Mismatch`/`LocationNotFound`/
/// `EditionNotFound`/`AccessDenied` return a typed error instead of a success.
///
/// `verify_quotation` delegates to the same `resolve` path used here; `resolve`
/// is called directly because it also yields the resolved canonical hash in a
/// single fetch.
pub async fn verify_canonical_quotation(
    reader: &Arc<QuranReaderService>,
    edition_slug: &str,
    edition_version: &str,
    surah: u16,
    ayah: u32,
    text: &str,
) -> Result<(citations::QuotationVerdict, String), citations::CitationError> {
    let citation = citations::Citation {
        id: "verify-quotation".to_string(),
        kind: citations::CitationKind::Quran,
        canonical_reference: format!("quran:{edition_slug}@{edition_version}:{surah}:{ayah}"),
        quoted_text: text.to_string(),
        edition_slug: edition_slug.to_string(),
        edition_version: edition_version.to_string(),
        surah: SurahNumber::new(surah).map_err(|_| citations::CitationError::InvalidReference {
            reference: format!("quran:{edition_slug}@{edition_version}:{surah}:{ayah}"),
        })?,
        ayah: AyahNumber::new(ayah).map_err(|_| citations::CitationError::InvalidReference {
            reference: format!("quran:{edition_slug}@{edition_version}:{surah}:{ayah}"),
        })?,
    };
    let resolver = ReaderCitationSource::resolver(reader.clone());
    let resolved = resolver.resolve(&citation).await?;
    citations::require_exact(&resolved.verdict)?;
    let text_hash = resolved.text_hash.ok_or_else(|| citations::CitationError::Backend {
        detail: "resolved citation carried no canonical hash".to_string(),
    })?;
    Ok((resolved.verdict, text_hash))
}

/// Convenience: run `quran.get_ayah` against a reader.
pub async fn tool_get_ayah(
    reader: &Arc<QuranReaderService>,
    params: GetAyahParams,
) -> Result<(ToolResult<Vec<AyahView>>, tool_registry::BackendMeta), ToolError> {
    ReaderToolBackend::registry(reader.clone()).get_ayah(params).await
}

/// Convenience: run `quran.get_context` against a reader.
pub async fn tool_get_context(
    reader: &Arc<QuranReaderService>,
    params: GetContextParams,
) -> Result<(ToolResult<quran_core::ContextView>, tool_registry::BackendMeta), ToolError> {
    ReaderToolBackend::registry(reader.clone()).get_context(params).await
}

#[cfg(test)]
mod search_edition_tests {
    use super::search_edition_selector;
    use quran_core::EditionSelector;

    #[test]
    fn bare_slug_selects_the_active_version() {
        assert_eq!(
            search_edition_selector("hafs-uthmani").unwrap(),
            EditionSelector::Slug("hafs-uthmani".to_string())
        );
    }

    #[test]
    fn versioned_spec_pins_the_edition() {
        match search_edition_selector("hafs-uthmani@1.0.0").unwrap() {
            EditionSelector::Pinned { slug, version } => {
                assert_eq!(slug, "hafs-uthmani");
                assert_eq!(version.to_string(), "1.0.0");
            }
            other => panic!("expected a pinned selector, got {other:?}"),
        }
    }

    #[test]
    fn invalid_version_is_a_typed_input_error() {
        let err = search_edition_selector("hafs-uthmani@not-semver").unwrap_err();
        assert!(matches!(err, tools::ToolError::InvalidInput { tool: "quran.search", .. }));
    }
}
