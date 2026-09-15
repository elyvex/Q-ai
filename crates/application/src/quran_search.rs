//! Phase 2 — exact + normalized search services (M3, P2-T41/T42).
//!
//! Both tools assemble [`SearchHit`](quran_search::SearchHit) through the
//! single validating constructor, so every hit carries its trace (I9), its
//! exact canonical span (I10), and a verified quotation — regardless of
//! which match mode found it.
//!
//! Engine split (documented, not incidental):
//!
//! - `WholeToken` runs on the FTS index (token-precise `Term` queries);
//! - `Substring` and `AyahPrefix` scan normalized ayah text in Rust (exact
//!   substring semantics FTS cannot give), with spans resolved through the
//!   shared pipeline's [`SpanMap`](quran_normalization::SpanMap).
//!
//! `explain: false` serves canonical order without scores (reference-set
//! semantics, AC-P2-07); `explain: true` serves backend relevance order with
//! per-hit BM25 breakdowns (plan §5.2). Scan modes have no backend rank and
//! always serve canonical order; their traces are complete either way.

use std::collections::HashMap;

use quran_normalization::{NormalizationTrace, RuleId, SemVer};
use quran_search::{
    Diagnostic as SearchDiagnostic, FieldId, Filter, FtsQuery, FullTextIndex, Fts5Index,
    IndexError, IndexManifest, ResultOrder, ScoreExplain, SearchHit, SearchHitParts,
    TokenizerFamily, Warning,
};
use storage::Database as _;
use storage::error::StorageError;
use storage_sqlite::SqliteDatabase;

use crate::quran_normalize;

/// Ayah-level index backing both tools.
pub const SEARCH_INDEX_ID: &str = crate::quran_index::QURAN_AYAH_INDEX_ID;

/// Token match mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchMode {
    /// Whole-token equality after normalization.
    WholeToken,
    /// Substring of normalized ayah text.
    Substring,
    /// Normalized ayah-text prefix.
    AyahPrefix,
}

/// Exact-search field (`L0` identity or `L1` whitespace profile).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExactField {
    /// Canonical surface, no linguistic expansion.
    TextExact,
    /// After whitespace normalization only.
    TextWs,
}

impl ExactField {
    fn field_id(self) -> FieldId {
        match self {
            Self::TextExact => "text_exact".to_string(),
            Self::TextWs => "text_ws".to_string(),
        }
    }
}

/// Shared tool parameters.
#[derive(Debug, Clone)]
pub struct SearchParams {
    /// Query text.
    pub text: String,
    /// Edition `slug@version` (defaults to the indexed edition).
    pub edition: Option<String>,
    /// Match mode.
    pub mode: MatchMode,
    /// Metadata filters.
    pub filters: Vec<Filter>,
    /// Result cap.
    pub limit: u32,
    /// Result offset.
    pub offset: u32,
    /// `false` = canonical order without scores; `true` = relevance order
    /// with per-hit BM25 breakdowns (indexed modes only).
    pub explain: bool,
}

/// Tool output: hits plus exact totals and provenance.
#[derive(Debug, Clone)]
pub struct SearchOutput {
    /// Hits in the requested order (one per matching ayah).
    pub hits: Vec<SearchHit>,
    /// Exact matching-ayah count (separate count path, never estimated).
    pub total_matches: u64,
    /// Whether `limit` truncated the hit list.
    pub truncated: bool,
    /// Trace label of the profile that served the query.
    pub rule_set: String,
    /// Corpus generation indexed.
    pub generation: i64,
    /// Output-level advisories (zero-result hints, drift notes).
    pub warnings: Vec<Warning>,
}

/// Search-service failures.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SearchError {
    /// Storage failure.
    #[error("search storage failed: {0}")]
    Storage(String),
    /// Normalization failure.
    #[error(transparent)]
    Normalization(#[from] quran_normalization::error::NormalizationError),
    /// Index-engine failure.
    #[error(transparent)]
    Index(#[from] IndexError),
    /// Requested edition is not the indexed one.
    #[error("edition {requested} is not indexed (serving {indexed})")]
    EditionNotIndexed {
        /// Requested edition.
        requested: String,
        /// Indexed edition.
        indexed: String,
    },
    /// No serving generation exists (index never built).
    #[error("no serving generation for index {index_id}; build it first")]
    NoServingIndex {
        /// Index id.
        index_id: String,
    },
}

impl SearchError {
    fn storage(error: StorageError) -> Self {
        Self::Storage(error.to_string())
    }
}

impl storage::error::Diagnostic for SearchError {
    fn code(&self) -> storage::error::DiagnosticCode {
        match self {
            Self::Storage(_) => storage::error::DiagnosticCode::new("QAI-IDX", 3),
            Self::Normalization(inner) => {
                use quran_normalization::error::NormalizationError as NE;
                match inner {
                    NE::UnknownRule { .. } => storage::error::DiagnosticCode::new("QAI-NORM", 1),
                    NE::UnknownProfile { .. } => storage::error::DiagnosticCode::new("QAI-NORM", 2),
                    _ => storage::error::DiagnosticCode::new("QAI-NORM", 5),
                }
            }
            Self::Index(inner) => {
                let rendered = inner.code().to_string();
                let number =
                    rendered.split('-').next_back().and_then(|n| n.parse().ok()).unwrap_or(3);
                storage::error::DiagnosticCode::new("QAI-IDX", number)
            }
            Self::EditionNotIndexed { .. } => storage::error::DiagnosticCode::new("QAI-IDX", 2),
            Self::NoServingIndex { .. } => storage::error::DiagnosticCode::new("QAI-IDX", 3),
        }
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn remedy(&self) -> Option<String> {
        Some(match self {
            Self::Storage(_) => "Check the database and retry.".to_string(),
            Self::Normalization(_) => "Check the profile seed against the code.".to_string(),
            Self::Index(inner) => inner.remedy().unwrap_or_else(|| "See above.".to_string()),
            Self::EditionNotIndexed { .. } => {
                "Search the indexed edition, or build an index for the requested one.".to_string()
            }
            Self::NoServingIndex { .. } => "Run `qai quran index rebuild` first.".to_string(),
        })
    }

    fn next_command(&self) -> Option<String> {
        Some("qai doctor --indexes".to_string())
    }
}

/// Open serving context: adapter on the pointed generation, family pinned to
/// the manifest tokenizer version, and canonical rows for assembly.
struct Serving {
    index: Fts5Index,
    manifest: IndexManifest,
    edition_id: String,
    edition_slug: String,
    edition_version: String,
    generation: i64,
    stale: Option<Warning>,
    surahs: HashMap<i64, storage::quran::SurahRow>,
    script: String,
    riwayah: Option<String>,
}

async fn open_serving(
    db: &SqliteDatabase,
    data_dir: &std::path::Path,
    requested_edition: Option<&str>,
) -> Result<Serving, SearchError> {
    // Pointer → manifest → adapter.
    let (pointer, manifest) = {
        let mut uow = db.write().await.map_err(SearchError::storage)?;
        let pointer = uow
            .quran()
            .get_index_pointer(SEARCH_INDEX_ID)
            .await
            .map_err(SearchError::storage)?;
        uow.rollback().await.map_err(SearchError::storage)?;
        let Some(pointer) = pointer else {
            return Err(SearchError::NoServingIndex { index_id: SEARCH_INDEX_ID.to_string() });
        };
        let manifest: IndexManifest =
            serde_json::from_str(&pointer.manifest_json).map_err(|err| {
                SearchError::Index(IndexError::BuildFailed {
                    stage: "open".to_string(),
                    detail: format!("stored manifest is corrupt: {err}"),
                })
            })?;
        (pointer, manifest)
    };
    let profile_rows = {
        let mut uow = db.write().await.map_err(SearchError::storage)?;
        let rows =
            uow.quran().list_normalization_profiles().await.map_err(SearchError::storage)?;
        uow.rollback().await.map_err(SearchError::storage)?;
        rows
    };
    let registry = quran_normalize::registry_from_rows(&profile_rows).map_err(|err| {
        SearchError::Index(IndexError::BuildFailed {
            stage: "open".to_string(),
            detail: err.to_string(),
        })
    })?;
    let family = TokenizerFamily::new(&registry, manifest.tokenizer_version).map_err(|err| {
        SearchError::Index(IndexError::BuildFailed {
            stage: "open".to_string(),
            detail: err.to_string(),
        })
    })?;
    let ladder = manifest.tokenizer_version;
    let index =
        Fts5Index::open(data_dir, pointer.generation as u64, manifest.clone(), family)
            .await
            .map_err(SearchError::Index)?;

    // Edition context (indexed edition by default; requests must match it).
    let mut uow = db.write().await.map_err(SearchError::storage)?;
    let edition = uow.quran().get_edition(&manifest.edition_id).await.map_err(SearchError::storage)?;
    let Some(edition) = edition else {
        return Err(SearchError::Index(IndexError::BuildFailed {
            stage: "open".to_string(),
            detail: format!("indexed edition {} is gone", manifest.edition_id),
        }));
    };
    if let Some(requested) = requested_edition {
        let (slug, version) = requested.split_once('@').unwrap_or((requested, ""));
        if slug != edition.slug || (!version.is_empty() && version != edition.version) {
            return Err(SearchError::EditionNotIndexed {
                requested: requested.to_string(),
                indexed: format!("{}@{}", edition.slug, edition.version),
            });
        }
    }
    let surahs = uow.quran().list_surahs(&edition.id).await.map_err(SearchError::storage)?;
    let active = uow.quran().get_active().await.map_err(SearchError::storage)?;
    uow.rollback().await.map_err(SearchError::storage)?;

    // Drift is a warning, never an auto-repair: serving a superseded
    // generation still works but every hit says so.
    let stale = match active {
        Some(active) if active.edition_id != edition.id => Some(Warning::stale_index(format!(
            "serving {}@{} but {} is active; rebuild the index",
            edition.slug, edition.version, active.edition_id
        ))),
        _ => None,
    };
    let surahs = surahs.into_iter().map(|row| (row.number, row)).collect();
    Ok(Serving {
        index,
        manifest: manifest.clone(),
        edition_id: edition.id.clone(),
        edition_slug: edition.slug.clone(),
        edition_version: edition.version.clone(),
        generation: manifest.corpus_generation as i64,
        stale,
        surahs,
        script: edition.script.clone(),
        riwayah: edition.riwayah.clone(),
    })
}

/// Parse `doc_id` (`<index>:<edition>:<surah>:<ayah>`) back to location.
///
/// Cross-checked against the serving edition id: a hit pointing outside the
/// served corpus is a backend bug, surfaced as [`IndexError::BuildFailed`],
/// never silently served.
fn parse_doc_id(doc_id: &str, index_id: &str, edition_id: &str) -> Result<(u16, u32), SearchError> {
    let mut parts = doc_id.rsplitn(4, ':');
    let ayah = parts.next().unwrap_or_default();
    let surah = parts.next().unwrap_or_default();
    let edition = parts.next().unwrap_or_default();
    let index = parts.next().unwrap_or_default();
    if index != index_id || edition != edition_id {
        return Err(SearchError::Index(IndexError::BuildFailed {
            stage: "search".to_string(),
            detail: format!("hit outside the serving corpus: {doc_id}"),
        }));
    }
    let (surah, ayah) = (surah.parse::<u16>(), ayah.parse::<u32>());
    match (surah, ayah) {
        (Ok(surah), Ok(ayah)) => Ok((surah, ayah)),
        _ => Err(SearchError::Index(IndexError::BuildFailed {
            stage: "search".to_string(),
            detail: format!("unparseable hit location: {doc_id}"),
        })),
    }
}

/// Tokens of one ayah whose normalized surface equals the normalized term.
fn whole_token_matches(
    pipeline: &quran_normalization::NormalizationPipeline,
    tokens: &[storage::quran::TokenRow],
    term: &str,
) -> Vec<usize> {
    let wanted = pipeline.apply(term).0.text().to_string();
    tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| pipeline.apply(&token.surface).0.text() == wanted)
        .map(|(index, _)| index)
        .collect()
}

/// Assemble one hit from canonical rows (the single assembly path).
#[allow(clippy::too_many_arguments)]
async fn assemble_hit(
    db: &SqliteDatabase,
    serving: &Serving,
    surah: u16,
    ayah: u32,
    span: quran_normalization::CanonicalSpan,
    matched: &[usize],
    tokens: &[storage::quran::TokenRow],
    score: Option<f32>,
    score_explain: Option<ScoreExplain>,
    trace: NormalizationTrace,
) -> Result<SearchHit, SearchError> {
    let mut uow = db.write().await.map_err(SearchError::storage)?;
    let ayah_row = uow
        .quran()
        .get_ayah(&serving.edition_id, i64::from(surah), i64::from(ayah))
        .await
        .map_err(SearchError::storage)?;
    uow.rollback().await.map_err(SearchError::storage)?;
    let Some(ayah_row) = ayah_row else {
        return Err(SearchError::Index(IndexError::BuildFailed {
            stage: "search".to_string(),
            detail: format!("hit ayah {surah}:{ayah} is gone"),
        }));
    };
    let surah_row = serving.surahs.get(&i64::from(surah)).ok_or_else(|| {
        SearchError::Index(IndexError::BuildFailed {
            stage: "search".to_string(),
            detail: format!("hit surah {surah} is gone"),
        })
    })?;
    let matched_tokens: Vec<u16> = matched.iter().map(|index| tokens[*index].position as u16).collect();
    let edition_version: SemVer = serving.edition_version.parse().map_err(|_| {
        SearchError::Index(IndexError::BuildFailed {
            stage: "search".to_string(),
            detail: format!("stored edition version {} is not semver", serving.edition_version),
        })
    })?;
    let mut warnings = Vec::new();
    if let Some(stale) = serving.stale.clone() {
        warnings.push(stale);
    }
    SearchHit::new(quran_search::SearchHitParts {
        edition: quran_core::EditionRef {
            slug: serving.edition_slug.clone(),
            version: edition_version,
            script: parse_script(&serving.script),
            riwayah: serving.riwayah.clone(),
        },
        surah_number: surah,
        surah_name_arabic: surah_row.name_arabic.clone(),
        surah_name_translit: surah_row.name_transliteration.clone(),
        ayah,
        arabic_text: ayah_row.text.clone(),
        text_hash: ayah_row.text_hash.clone(),
        page: ayah_row.page.map(|page| page as u32),
        juz: ayah_row.juz.map(|juz| juz as u16),
        span,
        matched_tokens,
        score,
        score_explain,
        explanation: trace,
        warnings,
    })
    .map_err(SearchError::Index)
}

fn parse_script(raw: &str) -> quran_core::Script {
    match raw {
        "uthmani" => quran_core::Script::Uthmani,
        "imlaei_simple" => quran_core::Script::ImlaeiSimple,
        other => match other.strip_prefix("other:") {
            Some(name) => quran_core::Script::Other(name.to_string()),
            None => quran_core::Script::Other(other.to_string()),
        },
    }
}

/// Read ordered token rows for one ayah.
async fn ayah_tokens(
    db: &SqliteDatabase,
    edition_id: &str,
    surah: u16,
    ayah: u32,
) -> Result<Vec<storage::quran::TokenRow>, SearchError> {
    let mut uow = db.write().await.map_err(SearchError::storage)?;
    let mut tokens = uow
        .quran()
        .get_tokens(edition_id, i64::from(surah), i64::from(ayah))
        .await
        .map_err(SearchError::storage)?;
    uow.rollback().await.map_err(SearchError::storage)?;
    tokens.sort_by_key(|token| token.position);
    Ok(tokens)
}

/// One candidate ayah with its backend score, if any.
struct Candidate {
    surah: u16,
    ayah: u32,
    score: Option<f32>,
}

/// Shared driver: match per candidate ayah, assemble hits in canonical
/// order, and report the exact total the caller counted separately.
async fn run_search(
    db: &SqliteDatabase,
    serving: &Serving,
    trace: NormalizationTrace,
    params: &SearchParams,
    field: &str,
    stats_docs: u64,
    candidates: Vec<Candidate>,
    total_matches: u64,
    match_one: impl Fn(&str, &[storage::quran::TokenRow]) -> Option<(quran_normalization::CanonicalSpan, Vec<usize>)>,
) -> Result<SearchOutput, SearchError> {
    let mut ordered = candidates;
    ordered.sort_by_key(|candidate| (candidate.surah, candidate.ayah));
    ordered.dedup_by_key(|candidate| (candidate.surah, candidate.ayah));
    let mut hits = Vec::new();
    let start = params.offset as usize;
    let end = start.saturating_add(params.limit.max(1) as usize);
    for (position, candidate) in ordered.iter().enumerate() {
        if position < start || position >= end {
            continue;
        }
        let tokens =
            ayah_tokens(db, &serving.edition_id, candidate.surah, candidate.ayah).await?;
        let mut uow = db.write().await.map_err(SearchError::storage)?;
        let ayah_row = uow
            .quran()
            .get_ayah(&serving.edition_id, i64::from(candidate.surah), i64::from(candidate.ayah))
            .await
            .map_err(SearchError::storage)?;
        uow.rollback().await.map_err(SearchError::storage)?;
        let Some(ayah_row) = ayah_row else { continue };
        let Some((span, matched)) = match_one(&ayah_row.text, &tokens) else { continue };
        // Scores exist only on the relevance path (whole-token + explain);
        // scan modes always serve canonical order without scores.
        let (score, score_explain) = match (params.explain, candidate.score) {
            (true, Some(score)) => (
                Some(score),
                Some(ScoreExplain::for_hit(
                    quran_search::FtsBackend::Fts5,
                    field.to_string(),
                    score,
                    stats_docs,
                    ResultOrder::Relevance,
                )),
            ),
            _ => (None, None),
        };
        hits.push(
            assemble_hit(
                db,
                serving,
                candidate.surah,
                candidate.ayah,
                span,
                &matched,
                &tokens,
                score,
                score_explain,
                trace.clone(),
            )
            .await?,
        );
    }
    let limit = u64::from(params.limit.max(1));
    let truncated =
        u64::try_from(hits.len()).unwrap_or(u64::MAX) >= limit && total_matches > limit;
    Ok(SearchOutput {
        hits,
        total_matches,
        truncated,
        rule_set: trace.profile.clone(),
        generation: serving.generation,
        warnings: serving.stale.clone().into_iter().collect(),
    })
}

/// Normalize one token surface for whole-token comparison.
fn normalize_token(
    pipeline: &quran_normalization::NormalizationPipeline,
    surface: &str,
) -> String {
    pipeline.apply(surface).0.text().to_string()
}

/// Whole-token candidates through the shared family (index path).
async fn whole_token_candidates(
    serving: &Serving,
    field: &str,
    term: &str,
    filters: &[Filter],
    explain: bool,
) -> Result<(Vec<Candidate>, u64), SearchError> {
    use quran_search::FtsQuery;
    let order = if explain { ResultOrder::Relevance } else { ResultOrder::CanonicalOrder };
    let opts = quran_search::SearchOpts {
        limit: 1000,
        offset: 0,
        filters: filters.to_vec(),
        order,
        highlight: false,
        timeout_ms: 10_000,
        explain,
    };
    let query = FtsQuery::Term { field: field.to_string(), term: term.to_string() };
    let total_matches = serving.index.count(&query).await.map_err(SearchError::Index)?;
    // Page through the full match set (1,000-doc pages cover any corpus here;
    // the tool-level limit applies at assembly).
    let mut candidates = Vec::new();
    let mut offset = 0u32;
    loop {
        let mut page_opts = opts.clone();
        page_opts.offset = offset;
        let page = serving.index.search(&query, &page_opts).await.map_err(SearchError::Index)?;
        if page.hits.is_empty() {
            break;
        }
        for hit in &page.hits {
            let (surah, ayah) =
                parse_doc_id(&hit.doc_id, &serving.manifest.index_id, &serving.edition_id)?;
            candidates.push(Candidate { surah, ayah, score: hit.score });
        }
        if page.hits.len() < page_opts.limit as usize {
            break;
        }
        offset += page_opts.limit;
    }
    Ok((candidates, total_matches))
}

/// Scan every ayah through a pipeline (substring/prefix/non-indexed modes).
async fn scan_candidates(
    db: &SqliteDatabase,
    serving: &Serving,
    pipeline: &quran_normalization::NormalizationPipeline,
    query: &str,
    mode: MatchMode,
) -> Result<Vec<Candidate>, SearchError> {
    let mut uow = db.write().await.map_err(SearchError::storage)?;
    let ayahs = uow
        .quran()
        .list_ayahs_range(&serving.edition_id, 1, i64::MAX)
        .await
        .map_err(SearchError::storage)?;
    uow.rollback().await.map_err(SearchError::storage)?;
    let mut ordered = ayahs;
    ordered.sort_by_key(|ayah| (ayah.surah, ayah.ayah));
    let mut candidates = Vec::new();
    for ayah in &ordered {
        let derived = pipeline.apply(&ayah.text).0;
        let matched = match mode {
            MatchMode::WholeToken => {
                // Whole-token over scanned text: tokenize the derived form.
                derived.text().split_whitespace().any(|token| token == query)
            }
            MatchMode::Substring => derived.text().contains(query),
            MatchMode::AyahPrefix => derived.text().starts_with(query),
        };
        if matched {
            candidates.push(Candidate {
                surah: ayah.surah as u16,
                ayah: ayah.ayah as u32,
                score: None,
            });
        }
    }
    Ok(candidates)
}

/// Span + tokens for one ayah under a pipeline (scan modes).
fn scan_match(
    pipeline: &quran_normalization::NormalizationPipeline,
    ayah_text: &str,
    tokens: &[storage::quran::TokenRow],
    query: &str,
    mode: MatchMode,
) -> Option<(quran_normalization::CanonicalSpan, Vec<usize>)> {
    let derived = pipeline.apply(ayah_text).0;
    let text = derived.text();
    let (start, end) = match mode {
        MatchMode::WholeToken => {
            // First token equal to the query.
            let mut offset = 0u32;
            let mut found = None;
            for token in text.split_whitespace() {
                let len = token.chars().count() as u32;
                if token == query {
                    found = Some((offset, offset + len));
                    break;
                }
                offset += len + 1;
            }
            found?
        }
        MatchMode::Substring => {
            let byte = text.find(query)?;
            let start = text[..byte].chars().count() as u32;
            start.checked_add(query.chars().count() as u32).map(|end| (start, end))?
        }
        MatchMode::AyahPrefix => {
            if !text.starts_with(query) {
                return None;
            }
            (0, query.chars().count() as u32)
        }
    };
    let span = derived.spans().to_canonical(start..end);
    // Tokens overlapping the canonical span.
    let matched: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| {
            let (token_start, token_end) = (token.char_start as u32, token.char_end as u32);
            token_start < span.char_range.end && span.char_range.start < token_end
        })
        .map(|(index, _)| index)
        .collect();
    if matched.is_empty() {
        return None;
    }
    Some((span, matched))
}

/// `quran.search_exact` (P2-T41): no linguistic expansion beyond L0/L1.
///
/// Foreign code points never fold silently: a zero-result query that the
/// Persian fold would change carries an actionable warning instead.
pub async fn search_exact(
    db: &SqliteDatabase,
    index_root: &std::path::Path,
    params: &SearchParams,
    field: ExactField,
) -> Result<SearchOutput, SearchError> {
    let serving = open_serving(db, index_root, params.edition.as_deref()).await?;
    let field_id = field.field_id();
    let registry = db_registry(db).await?;
    let profile = match field {
        ExactField::TextExact => quran_normalization::ProfileId::L0,
        ExactField::TextWs => quran_normalization::ProfileId::L1,
    };
    let pipeline = quran_normalization::NormalizationPipeline::for_profile(
        &registry,
        profile,
        latest_version(&registry, profile)?,
    )
    .map_err(SearchError::Normalization)?;
    let trace = empty_trace_for(&pipeline);
    let normalized_query = normalize_token(&pipeline, &params.text);

    let (candidates, total_matches) = match params.mode {
        MatchMode::WholeToken => {
            whole_token_candidates(&serving, &field_id, &params.text, &params.filters, params.explain).await?
        }
        MatchMode::Substring | MatchMode::AyahPrefix => {
            let found = scan_candidates(db, &serving, &pipeline, &normalized_query, params.mode).await?;
            let total = found.len() as u64;
            (found, total)
        }
    };

    let mut output = run_search(
        db,
        &serving,
        trace,
        params,
        &field_id,
        stats_docs(&serving).await?,
        candidates,
        total_matches,
        |ayah_text, tokens| {
            // Re-derive per ayah so spans come from the shared pipeline.
            match params.mode {
                MatchMode::WholeToken => verify_whole_token(&pipeline, tokens, &params.text),
                _ => {
                    let normalized = normalize_token(&pipeline, &params.text);
                    scan_match(&pipeline, ayah_text, tokens, &normalized, params.mode)
                }
            }
        },
    )
    .await?;

    // Zero-result hint (T41): never a silent fold under exact search.
    if output.total_matches == 0 && persian_fold_differs(&params.text) {
        output.warnings.push(Warning::new(
            "QAI-NORM-0000-hint",
            "no exact match; the query contains Persian code points — retry quran.search_normalized with L5.codepoints",
        ));
    }
    Ok(output)
}

/// Profile selector for normalized search: registry id or adhoc rule list.
///
/// Typed so `profile + rules` can never be constructed (the plan's "never
/// both" rule holds by construction, not by runtime check).
pub enum NormalizedProfile {
    /// Registry profile (latest or pinned).
    Registry(quran_normalization::ProfileId, Option<SemVer>),
    /// Explicit rule list (adhoc pipeline over a superset scan).
    Adhoc(Vec<quran_normalization::RuleId>),
}

/// `quran.search_normalized` (P2-T42): profile or explicit rules, never both.
///
/// Indexed profiles (L0–L5) query their FTS field; heuristic, experimental,
/// and adhoc rule sets scan with per-candidate verification instead.
pub async fn search_normalized(
    db: &SqliteDatabase,
    index_root: &std::path::Path,
    params: &SearchParams,
    profile: NormalizedProfile,
) -> Result<SearchOutput, SearchError> {
    let serving = open_serving(db, index_root, params.edition.as_deref()).await?;
    let registry = db_registry(db).await?;

    enum Source {
        Indexed { field: FieldId, pipeline: quran_normalization::NormalizationPipeline, trace: NormalizationTrace },
        Scanned { pipeline: quran_normalization::NormalizationPipeline, trace: NormalizationTrace },
    }
    let source = match profile {
        NormalizedProfile::Registry(id, version) => {
            let stored = registry.latest(id).map_err(SearchError::Normalization)?;
            let version = version.unwrap_or(stored.version);
            let pipeline = quran_normalization::NormalizationPipeline::for_profile(&registry, id, version)
                .map_err(SearchError::Normalization)?;
            let trace = empty_trace_for(&pipeline);
            match indexed_field_for(id) {
                Some(field) => Source::Indexed { field, pipeline, trace },
                None => Source::Scanned { pipeline, trace },
            }
        }
        NormalizedProfile::Adhoc(ids) => {
            let pipeline = quran_normalization::NormalizationPipeline::adhoc(&ids)
                .map_err(SearchError::Normalization)?;
            let trace = empty_trace_for(&pipeline);
            Source::Scanned { pipeline, trace }
        }
    };

    match source {
        Source::Indexed { field, pipeline, trace } => {
            let (candidates, total_matches) = match params.mode {
                MatchMode::WholeToken => {
                    whole_token_candidates(&serving, &field, &params.text, &params.filters, params.explain).await?
                }
                MatchMode::Substring | MatchMode::AyahPrefix => {
                    let normalized = normalize_token(&pipeline, &params.text);
                    let found = scan_candidates(db, &serving, &pipeline, &normalized, params.mode).await?;
                    let total = found.len() as u64;
                    (found, total)
                }
            };
            run_search(
                db,
                &serving,
                trace,
                params,
                &field,
                stats_docs(&serving).await?,
                candidates,
                total_matches,
                |ayah_text, tokens| match params.mode {
                    MatchMode::WholeToken => verify_whole_token(&pipeline, tokens, &params.text),
                    _ => {
                        let normalized = normalize_token(&pipeline, &params.text);
                        scan_match(&pipeline, ayah_text, tokens, &normalized, params.mode)
                    }
                },
            )
            .await
        }
        Source::Scanned { pipeline, trace } => {
            let normalized = normalize_token(&pipeline, &params.text);
            let found = scan_candidates(db, &serving, &pipeline, &normalized, params.mode).await?;
            let total = found.len() as u64;
            // Scanned modes always serve canonical order: no backend rank.
            let field = "text_bare".to_string();
            run_search(db, &serving, trace, params, &field, 0, found, total, |ayah_text, tokens| {
                scan_match(&pipeline, ayah_text, tokens, &normalized, params.mode)
            })
            .await
        }
    }
}

/// Indexed FTS field for a profile, if the profile has ayah-level index
/// columns (L0–L5). Heuristic, experimental, and skeleton profiles scan.
fn indexed_field_for(id: quran_normalization::ProfileId) -> Option<FieldId> {
    use quran_normalization::ProfileId as P;
    match id {
        P::L0 => Some("text_exact".to_string()),
        P::L1 => Some("text_ws".to_string()),
        P::L2 => Some("text_marks".to_string()),
        P::L3 => Some("text_bare".to_string()),
        P::L4 => Some("text_hamza".to_string()),
        P::L5 => Some("text_folded".to_string()),
        P::L6 | P::L7 | P::L8 => None,
    }
}

/// Verify a whole-token candidate against token rows (index/verify split).
fn verify_whole_token(
    pipeline: &quran_normalization::NormalizationPipeline,
    tokens: &[storage::quran::TokenRow],
    text: &str,
) -> Option<(quran_normalization::CanonicalSpan, Vec<usize>)> {
    let matched = whole_token_matches(pipeline, tokens, text);
    let first = *matched.first()?;
    let token = tokens.get(first)?;
    Some((
        quran_normalization::CanonicalSpan {
            char_range: token.char_start as u32..token.char_end as u32,
            exact: true,
        },
        matched,
    ))
}

/// Empty-input trace for a pipeline (traces never depend on input text).
fn empty_trace_for(
    pipeline: &quran_normalization::NormalizationPipeline,
) -> NormalizationTrace {
    pipeline.apply("").1
}

/// Latest identical-profile version helper for exact search.
fn latest_version(
    registry: &quran_normalization::ProfileRegistry,
    id: quran_normalization::ProfileId,
) -> Result<SemVer, SearchError> {
    registry.latest(id).map(|profile| profile.version).map_err(SearchError::Normalization)
}

/// Index doc count for score explanations.
async fn stats_docs(serving: &Serving) -> Result<u64, SearchError> {
    serving.index.stats().await.map(|stats| stats.doc_count).map_err(SearchError::Index)
}

/// Profile definitions from the seeded catalog (proves the seed per call).
async fn db_registry(
    db: &SqliteDatabase,
) -> Result<quran_normalization::ProfileRegistry, SearchError> {
    let mut uow = db.write().await.map_err(SearchError::storage)?;
    let rows =
        uow.quran().list_normalization_profiles().await.map_err(SearchError::storage)?;
    uow.rollback().await.map_err(SearchError::storage)?;
    crate::quran_normalize::registry_from_rows(&rows).map_err(|err| {
        SearchError::Index(IndexError::BuildFailed {
            stage: "open".to_string(),
            detail: err.to_string(),
        })
    })
}

/// True when the Persian fold would change the query (hint trigger).
fn persian_fold_differs(text: &str) -> bool {
    use quran_normalization::{NormalizationRule, NormalizedText};
    let rule = quran_normalization::rules::NormalizePersianCodepoints;
    rule.apply(&NormalizedText::from_plain(text)).text() != text
}
