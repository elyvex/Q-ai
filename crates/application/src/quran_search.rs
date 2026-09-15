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
    Diagnostic as SearchDiagnostic, FieldId, Filter, Fts5Index, FtsQuery, FullTextIndex,
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
    /// Wrap hit spans in `<b>` display markers (T49).
    pub highlight: bool,
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
        let pointer =
            uow.quran().get_index_pointer(SEARCH_INDEX_ID).await.map_err(SearchError::storage)?;
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
        let rows = uow.quran().list_normalization_profiles().await.map_err(SearchError::storage)?;
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
    let index = Fts5Index::open(data_dir, pointer.generation as u64, manifest.clone(), family)
        .await
        .map_err(SearchError::Index)?;

    // Edition context (indexed edition by default; requests must match it).
    let mut uow = db.write().await.map_err(SearchError::storage)?;
    let edition =
        uow.quran().get_edition(&manifest.edition_id).await.map_err(SearchError::storage)?;
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
    segmentation: Vec<quran_search::Segmentation>,
    highlight: bool,
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
    let matched_tokens: Vec<u16> =
        matched.iter().map(|index| tokens[*index].position as u16).collect();
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
    let highlighted = if highlight {
        quran_search::apply_markers(
            &ayah_row.text,
            &[(span.char_range.start, span.char_range.end)],
            "<b>",
            "</b>",
        )
    } else {
        None
    };
    SearchHit::new(SearchHitParts {
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
        segmentation,
        highlighted,
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

/// One verified ayah match: canonical span, token indexes, and optional
/// concatenated-match segmentation.
pub struct AyahMatch {
    pub span: quran_normalization::CanonicalSpan,
    pub matched: Vec<usize>,
    pub segmentation: Vec<quran_search::Segmentation>,
}

/// Shared driver: match per candidate ayah, assemble hits in canonical
/// order, and report the exact total the caller counted separately.
/// Shared assembly context for one tool call.
struct RunContext<'a> {
    db: &'a SqliteDatabase,
    serving: &'a Serving,
    trace: NormalizationTrace,
    params: &'a SearchParams,
    field: &'a str,
    stats_docs: u64,
}

async fn run_search(
    ctx: &RunContext<'_>,
    candidates: Vec<Candidate>,
    total_matches: u64,
    match_one: impl Fn(&str, &[storage::quran::TokenRow]) -> Option<AyahMatch>,
) -> Result<SearchOutput, SearchError> {
    let (db, serving, params) = (ctx.db, ctx.serving, ctx.params);
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
        let tokens = ayah_tokens(db, &serving.edition_id, candidate.surah, candidate.ayah).await?;
        let mut uow = db.write().await.map_err(SearchError::storage)?;
        let ayah_row = uow
            .quran()
            .get_ayah(&serving.edition_id, i64::from(candidate.surah), i64::from(candidate.ayah))
            .await
            .map_err(SearchError::storage)?;
        uow.rollback().await.map_err(SearchError::storage)?;
        let Some(ayah_row) = ayah_row else { continue };
        let Some(matched) = match_one(&ayah_row.text, &tokens) else { continue };
        // Scores exist only on the relevance path (whole-token + explain);
        // scan modes always serve canonical order without scores.
        let (score, score_explain) = match (params.explain, candidate.score) {
            (true, Some(score)) => (
                Some(score),
                Some(ScoreExplain::for_hit(
                    quran_search::FtsBackend::Fts5,
                    ctx.field.to_string(),
                    score,
                    ctx.stats_docs,
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
                matched.span,
                &matched.matched,
                &tokens,
                matched.segmentation,
                params.highlight,
                score,
                score_explain,
                ctx.trace.clone(),
            )
            .await?,
        );
    }
    let limit = u64::from(params.limit.max(1));
    let truncated = u64::try_from(hits.len()).unwrap_or(u64::MAX) >= limit && total_matches > limit;
    Ok(SearchOutput {
        hits,
        total_matches,
        truncated,
        rule_set: ctx.trace.profile.clone(),
        generation: serving.generation,
        warnings: serving.stale.clone().into_iter().collect(),
    })
}

/// Normalize one token surface for whole-token comparison.
fn normalize_token(pipeline: &quran_normalization::NormalizationPipeline, surface: &str) -> String {
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
/// Metadata filter check over one canonical ayah row (scan paths).
///
/// NULL division fields never match a range (SQL-like semantics); an empty
/// filter list matches everything.
fn passes_filters(
    ayah: &storage::quran::AyahRow,
    revelation: Option<&str>,
    filters: &[Filter],
) -> bool {
    filters.iter().all(|filter| match filter {
        Filter::Surah(ids) => ids.contains(&(ayah.surah as u16)),
        Filter::JuzRange(lo, hi) => {
            ayah.juz.is_some_and(|juz| juz >= i64::from(*lo) && juz <= i64::from(*hi))
        }
        Filter::Page(pages) => ayah.page.is_some_and(|page| pages.contains(&(page as u32))),
        Filter::RevelationPlace(place) => revelation == Some(place.as_str()),
        Filter::GlobalRange(lo, hi) => {
            let global = ayah.global_ayah_index as u64;
            global >= *lo && global <= *hi
        }
    })
}

/// Revelation place per surah for filter checks.
async fn surah_revelation(
    db: &SqliteDatabase,
    edition_id: &str,
) -> Result<HashMap<i64, String>, SearchError> {
    let mut uow = db.write().await.map_err(SearchError::storage)?;
    let surahs = uow.quran().list_surahs(edition_id).await.map_err(SearchError::storage)?;
    uow.rollback().await.map_err(SearchError::storage)?;
    Ok(surahs
        .into_iter()
        .filter_map(|row| row.revelation_place.map(|place| (row.number, place)))
        .collect())
}

async fn scan_candidates(
    db: &SqliteDatabase,
    serving: &Serving,
    pipeline: &quran_normalization::NormalizationPipeline,
    query: &str,
    mode: MatchMode,
    filters: &[Filter],
    revelation: &HashMap<i64, String>,
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
        if !passes_filters(ayah, revelation.get(&ayah.surah).map(String::as_str), filters) {
            continue;
        }
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

/// Convert a token row's grapheme-cluster range to char offsets.
///
/// Phase-1 token offsets are grapheme-cluster indices (ADR-0104) while the
/// normalization layer (and every [`CanonicalSpan`]) counts Unicode scalars.
/// This function is the single unit boundary between the two: spans stay in
/// char space, tokens convert here. Returns `None` for inconsistent rows
/// (fail-closed per token, never a guessed range).
fn token_char_range(text: &str, token: &storage::quran::TokenRow) -> Option<std::ops::Range<u32>> {
    use unicode_segmentation::UnicodeSegmentation;
    // Byte offset of each cluster start, plus the string end sentinel.
    let mut starts = vec![0u32];
    let mut chars = 0u32;
    for grapheme in text.graphemes(true) {
        chars += grapheme.chars().count() as u32;
        starts.push(chars);
    }
    let clusters = starts.len() as u32 - 1;
    let (from, to) = (token.char_start as u32, token.char_end as u32);
    if from > to || to > clusters {
        return None;
    }
    Some(starts[from as usize]..starts[to as usize])
}

/// Tokens overlapping a canonical span, with converted char ranges.
fn overlapping_tokens(
    text: &str,
    tokens: &[storage::quran::TokenRow],
    span: &quran_normalization::CanonicalSpan,
) -> Vec<usize> {
    tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| {
            token_char_range(text, token).is_some_and(|range| {
                range.start < span.char_range.end && span.char_range.start < range.end
            })
        })
        .map(|(index, _)| index)
        .collect()
}

/// Span + tokens for one ayah under a pipeline (scan modes).
fn scan_match(
    pipeline: &quran_normalization::NormalizationPipeline,
    ayah_text: &str,
    tokens: &[storage::quran::TokenRow],
    query: &str,
    mode: MatchMode,
) -> Option<AyahMatch> {
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
    // Tokens overlapping the canonical span (cluster→char converted).
    let matched = overlapping_tokens(ayah_text, tokens, &span);
    if matched.is_empty() {
        return None;
    }
    Some(AyahMatch { span, matched, segmentation: Vec::new() })
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
            whole_token_candidates(
                &serving,
                &field_id,
                &params.text,
                &params.filters,
                params.explain,
            )
            .await?
        }
        MatchMode::Substring | MatchMode::AyahPrefix => {
            let revelation = surah_revelation(db, &serving.edition_id).await?;
            let found = scan_candidates(
                db,
                &serving,
                &pipeline,
                &normalized_query,
                params.mode,
                &params.filters,
                &revelation,
            )
            .await?;
            let total = found.len() as u64;
            (found, total)
        }
    };

    let ctx = RunContext {
        db,
        serving: &serving,
        trace,
        params,
        field: &field_id,
        stats_docs: stats_docs(&serving).await?,
    };
    let mut output = run_search(&ctx, candidates, total_matches, |ayah_text, tokens| {
        // Re-derive per ayah so spans come from the shared pipeline.
        match params.mode {
            MatchMode::WholeToken => verify_whole_token(&pipeline, ayah_text, tokens, &params.text),
            _ => {
                let normalized = normalize_token(&pipeline, &params.text);
                scan_match(&pipeline, ayah_text, tokens, &normalized, params.mode)
            }
        }
    })
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
    Adhoc(Vec<RuleId>),
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
        Indexed {
            field: FieldId,
            pipeline: quran_normalization::NormalizationPipeline,
            trace: NormalizationTrace,
        },
        Scanned {
            pipeline: quran_normalization::NormalizationPipeline,
            trace: NormalizationTrace,
        },
    }
    let source = match profile {
        NormalizedProfile::Registry(id, version) => {
            let stored = registry.latest(id).map_err(SearchError::Normalization)?;
            let version = version.unwrap_or(stored.version);
            let pipeline =
                quran_normalization::NormalizationPipeline::for_profile(&registry, id, version)
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
                    whole_token_candidates(
                        &serving,
                        &field,
                        &params.text,
                        &params.filters,
                        params.explain,
                    )
                    .await?
                }
                MatchMode::Substring | MatchMode::AyahPrefix => {
                    let normalized = normalize_token(&pipeline, &params.text);
                    let revelation = surah_revelation(db, &serving.edition_id).await?;
                    let found = scan_candidates(
                        db,
                        &serving,
                        &pipeline,
                        &normalized,
                        params.mode,
                        &params.filters,
                        &revelation,
                    )
                    .await?;
                    let total = found.len() as u64;
                    (found, total)
                }
            };
            let ctx = RunContext {
                db,
                serving: &serving,
                trace,
                params,
                field: &field,
                stats_docs: stats_docs(&serving).await?,
            };
            run_search(&ctx, candidates, total_matches, |ayah_text, tokens| match params.mode {
                MatchMode::WholeToken => {
                    verify_whole_token(&pipeline, ayah_text, tokens, &params.text)
                }
                _ => {
                    let normalized = normalize_token(&pipeline, &params.text);
                    scan_match(&pipeline, ayah_text, tokens, &normalized, params.mode)
                }
            })
            .await
        }
        Source::Scanned { pipeline, trace } => {
            let normalized = normalize_token(&pipeline, &params.text);
            let revelation = surah_revelation(db, &serving.edition_id).await?;
            let found = scan_candidates(
                db,
                &serving,
                &pipeline,
                &normalized,
                params.mode,
                &params.filters,
                &revelation,
            )
            .await?;
            let total = found.len() as u64;
            // Scanned modes always serve canonical order: no backend rank.
            let field = "text_bare".to_string();
            let ctx =
                RunContext { db, serving: &serving, trace, params, field: &field, stats_docs: 0 };
            run_search(&ctx, found, total, |ayah_text, tokens| {
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
    ayah_text: &str,
    tokens: &[storage::quran::TokenRow],
    text: &str,
) -> Option<AyahMatch> {
    let matched = whole_token_matches(pipeline, tokens, text);
    let first = *matched.first()?;
    let token = tokens.get(first)?;
    let range = token_char_range(ayah_text, token)?;
    Some(AyahMatch {
        span: quran_normalization::CanonicalSpan { char_range: range, exact: true },
        matched,
        segmentation: Vec::new(),
    })
}

/// Empty-input trace for a pipeline (traces never depend on input text).
fn empty_trace_for(pipeline: &quran_normalization::NormalizationPipeline) -> NormalizationTrace {
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
    let rows = uow.quran().list_normalization_profiles().await.map_err(SearchError::storage)?;
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

/// Phrase match mode (plan §5.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhraseMode {
    /// Ordered tokens, no gaps.
    OrderedExact,
    /// Ordered tokens with up to `slop` intervening tokens between neighbors.
    OrderedNear,
    /// All terms within a `terms + slop` token window, any order.
    UnorderedNear,
}

/// Derived-text tokens with char ranges (for slop/window verification).
fn derived_tokens(text: &str) -> Vec<(std::ops::Range<u32>, &str)> {
    let mut out = Vec::new();
    let mut byte = 0;
    let mut char_pos = 0u32;
    for token in text.split_whitespace() {
        let relative = text[byte..].find(token).expect("split token is present");
        let start_byte = byte + relative;
        char_pos += text[byte..start_byte].chars().count() as u32;
        let len = token.chars().count() as u32;
        out.push((char_pos..char_pos + len, token));
        char_pos += len;
        byte = start_byte + token.len();
    }
    out
}

/// Locate a term sequence under phrase semantics; returns the derived char
/// range covering the match.
fn find_term_sequence(
    tokens: &[(std::ops::Range<u32>, &str)],
    terms: &[String],
    mode: PhraseMode,
    slop: u32,
) -> Option<(u32, u32)> {
    if terms.is_empty() {
        return None;
    }
    match mode {
        PhraseMode::OrderedExact => find_ordered(tokens, terms, 0),
        PhraseMode::OrderedNear => find_ordered(tokens, terms, slop),
        PhraseMode::UnorderedNear => find_unordered(tokens, terms, slop),
    }
}

/// Ordered match with per-gap budget (`slop` intervening tokens max).
fn find_ordered(
    tokens: &[(std::ops::Range<u32>, &str)],
    terms: &[String],
    slop: u32,
) -> Option<(u32, u32)> {
    for (start, _) in tokens.iter().enumerate().filter(|(_, (_, text))| *text == terms[0]) {
        let mut position = start;
        let mut ok = true;
        for term in &terms[1..] {
            let mut next = None;
            let mut cursor = position + 1;
            while cursor < tokens.len() && cursor - position - 1 <= slop as usize {
                if tokens[cursor].1 == term {
                    next = Some(cursor);
                    break;
                }
                cursor += 1;
            }
            match next {
                Some(found) => position = found,
                None => {
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            return Some((tokens[start].0.start, tokens[position].0.end));
        }
    }
    None
}

/// Unordered match: every term (with multiplicity) inside a
/// `terms + slop` token window. Greedy distinct assignment; ayahs are short
/// enough that the window scan stays exact for realistic inputs.
fn find_unordered(
    tokens: &[(std::ops::Range<u32>, &str)],
    terms: &[String],
    slop: u32,
) -> Option<(u32, u32)> {
    let width = terms.len() + slop as usize;
    if tokens.len() < terms.len() {
        return None;
    }
    for start in 0..=tokens.len() - terms.len().min(tokens.len()) {
        let end = (start + width).min(tokens.len());
        let window = &tokens[start..end];
        let mut used = vec![false; window.len()];
        let mut first = usize::MAX;
        let mut last = 0usize;
        let mut ok = true;
        for term in terms {
            match window
                .iter()
                .enumerate()
                .find(|(index, (_, text))| !used[*index] && *text == term)
            {
                Some((index, _)) => {
                    used[index] = true;
                    first = first.min(index);
                    last = last.max(index);
                }
                None => {
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            return Some((window[first].0.start, window[last].0.end));
        }
    }
    None
}

/// `quran.search_phrase` (P2-T43): ordered/near/unordered phrase search.
///
/// Recall comes from the FTS positional index (phrase or NEAR prefilter);
/// precision comes from Rust verification of exact slop/order semantics over
/// normalized ayah text, so reported spans always satisfy the mode.
pub async fn search_phrase(
    db: &SqliteDatabase,
    index_root: &std::path::Path,
    params: &SearchParams,
    profile: NormalizedProfile,
    mode: PhraseMode,
    slop: u32,
) -> Result<SearchOutput, SearchError> {
    let serving = open_serving(db, index_root, params.edition.as_deref()).await?;
    let registry = db_registry(db).await?;
    let (pipeline, trace, field) = match profile {
        NormalizedProfile::Registry(id, version) => {
            let stored = registry.latest(id).map_err(SearchError::Normalization)?;
            let version = version.unwrap_or(stored.version);
            let pipeline =
                quran_normalization::NormalizationPipeline::for_profile(&registry, id, version)
                    .map_err(SearchError::Normalization)?;
            let trace = empty_trace_for(&pipeline);
            let field = indexed_field_for(id).ok_or_else(|| {
                SearchError::Index(IndexError::QueryRejected {
                    detail: format!("profile {id} has no ayah-level phrase field"),
                })
            })?;
            (pipeline, trace, field)
        }
        NormalizedProfile::Adhoc(ids) => {
            let pipeline = quran_normalization::NormalizationPipeline::adhoc(&ids)
                .map_err(SearchError::Normalization)?;
            let trace = empty_trace_for(&pipeline);
            // Adhoc phrases verify against the most permissive indexed
            // field, then re-verify with the adhoc rules (same discipline
            // as adhoc whole-token search).
            (pipeline, trace, "text_folded".to_string())
        }
    };

    // Query terms: normalize the whole query, then split. An empty term list
    // matches nothing (never everything).
    let normalized_query = normalize_token(&pipeline, &params.text);
    let terms: Vec<String> = normalized_query.split_whitespace().map(str::to_string).collect();
    if terms.is_empty() {
        return Ok(SearchOutput {
            hits: Vec::new(),
            total_matches: 0,
            truncated: false,
            rule_set: trace.profile.clone(),
            generation: serving.generation,
            warnings: serving.stale.clone().into_iter().collect(),
        });
    }

    // Recall prefilter from the positional index. Bounds stay loose on
    // purpose: verification enforces exact semantics, so the prefilter must
    // over-approximate (NEAR distance covers gaps plus term count).
    let (recall_slop, ordered) = match mode {
        PhraseMode::OrderedExact => (0, true),
        PhraseMode::OrderedNear => (slop + 1, false),
        PhraseMode::UnorderedNear => (slop + terms.len() as u32, false),
    };
    let prefilter = quran_search::FtsQuery::Phrase {
        field: field.clone(),
        terms: terms.clone(),
        slop: recall_slop,
        ordered,
    };
    let order = if params.explain { ResultOrder::Relevance } else { ResultOrder::CanonicalOrder };
    let opts = quran_search::SearchOpts {
        limit: 1000,
        offset: 0,
        filters: params.filters.clone(),
        order,
        highlight: false,
        timeout_ms: 10_000,
        explain: params.explain,
    };
    // Recall prefilter from the positional index (precision total comes
    // from verification below, never from this count).
    let mut candidates = Vec::new();
    let mut offset = 0u32;
    loop {
        let mut page_opts = opts.clone();
        page_opts.offset = offset;
        let page =
            serving.index.search(&prefilter, &page_opts).await.map_err(SearchError::Index)?;
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

    // Precision: verify exact mode semantics per ayah; the total counts
    // verified matches only.
    let mut verified = Vec::new();
    for candidate in &candidates {
        let text = uow_text(db, &serving.edition_id, candidate.surah, candidate.ayah).await?;
        // Verify against normalized ayah text (cheap string pass first).
        let derived = pipeline.apply(&text);
        let dtokens = derived_tokens(derived.0.text());
        if find_term_sequence(&dtokens, &terms, mode, slop).is_some() {
            verified.push(Candidate {
                surah: candidate.surah,
                ayah: candidate.ayah,
                score: candidate.score,
            });
        }
    }
    let total_matches = verified.len() as u64;
    let ctx = RunContext {
        db,
        serving: &serving,
        trace,
        params,
        field: &field,
        stats_docs: stats_docs(&serving).await?,
    };
    run_search(&ctx, verified, total_matches, |ayah_text, tokens| {
        let derived = pipeline.apply(ayah_text);
        let dtokens = derived_tokens(derived.0.text());
        let (start, end) = find_term_sequence(&dtokens, &terms, mode, slop)?;
        let span = derived.0.spans().to_canonical(start..end);
        let matched = overlapping_tokens(ayah_text, tokens, &span);
        if matched.is_empty() {
            return None;
        }
        Some(AyahMatch { span, matched, segmentation: Vec::new() })
    })
    .await
}

/// Read one ayah's canonical text (single source of truth for verify passes).
async fn uow_text(
    db: &SqliteDatabase,
    edition_id: &str,
    surah: u16,
    ayah: u32,
) -> Result<String, SearchError> {
    let mut uow = db.write().await.map_err(SearchError::storage)?;
    let row = uow
        .quran()
        .get_ayah(edition_id, i64::from(surah), i64::from(ayah))
        .await
        .map_err(SearchError::storage)?;
    uow.rollback().await.map_err(SearchError::storage)?;
    row.map(|row| row.text).ok_or_else(|| {
        SearchError::Index(IndexError::BuildFailed {
            stage: "search".to_string(),
            detail: format!("ayah {surah}:{ayah} vanished mid-search"),
        })
    })
}

/// Character trigrams of a skeleton query (candidate recall, T36 will index
/// these; until then the same trigrams drive `LIKE` probes over the stored
/// skeleton strings).
fn query_trigrams(skeleton: &str) -> Vec<String> {
    let chars: Vec<char> = skeleton.chars().collect();
    if chars.len() < 3 {
        return Vec::new();
    }
    chars.windows(3).map(|window| window.iter().collect()).collect()
}

/// Segment a verified concatenated match into per-token query parts.
///
/// Public so tools and tests share the exact segmentation the service
/// returns (no second implementation to drift).
pub fn segment_concatenated(
    query_skeleton: &str,
    ayah_text: &str,
    derived_match_start: u32,
    span: &quran_normalization::CanonicalSpan,
    map: &quran_normalization::SpanMap,
    ayah_derived_len: u32,
    tokens: &[storage::quran::TokenRow],
) -> Vec<quran_search::Segmentation> {
    // Char→byte table for slicing the query skeleton: `byte_of[i]` is the
    // byte offset of char `i`, with the string length as the sentinel.
    let mut byte_of: Vec<u32> =
        query_skeleton.char_indices().map(|(byte, _)| byte as u32).collect();
    byte_of.push(query_skeleton.len() as u32);
    let slice = |from: u32, to: u32| {
        query_skeleton
            .get(byte_of[from as usize] as usize..byte_of[to as usize] as usize)
            .unwrap_or("")
            .to_string()
    };
    // Invert the derived→canonical map one char at a time (exact for single
    // chars: every derived char has exactly one canonical image). Only chars
    // inside the match window participate, so each part tiles the query.
    let derived_len = ayah_derived_len;
    let match_end = derived_match_start + query_skeleton.chars().count() as u32;
    let mut parts = Vec::new();
    // Walk the canonical span token by token (token rows are ordered).
    for token in tokens {
        let Some(token_range) = token_char_range(ayah_text, token) else { continue };
        let (token_start, token_end) = (token_range.start, token_range.end);
        if token_end <= span.char_range.start || token_start >= span.char_range.end {
            continue;
        }
        // Derived chars of the match window whose image falls inside this token.
        let mut first: Option<u32> = None;
        let mut last: u32 = 0;
        let mut derived = derived_match_start;
        while derived < derived_len.min(match_end) {
            let image = map.to_canonical(derived..derived + 1).char_range.start;
            if image >= token_start.max(span.char_range.start)
                && image < token_end.min(span.char_range.end)
            {
                if first.is_none() {
                    first = Some(derived);
                }
                last = derived + 1;
            }
            derived += 1;
        }
        if let Some(from) = first {
            // Report query-relative offsets: the match starts at
            // `derived_match_start` in ayah-derived space.
            parts.push(quran_search::Segmentation {
                query_part: slice(from - derived_match_start, last - derived_match_start),
                canonical_token: token.position as u16,
                canonical_surface: token.surface.clone(),
            });
        }
    }
    parts
}

/// `quran.search_concatenated` (P2-T44): spaceless queries against the L6
/// skeleton store, verified and segmented.
///
/// Recall comes from trigram probes over stored skeletons; precision comes
/// from exact substring verification plus re-normalization of the sliced
/// canonical text (the slice must reproduce the matched derived substring).
/// Cross-ayah windows (`allow_cross_ayah`) arrive in P2-T45 and are rejected
/// until then — never silently answered ayah-locally.
pub async fn search_concatenated(
    db: &SqliteDatabase,
    index_root: &std::path::Path,
    params: &SearchParams,
    allow_cross_ayah: bool,
    max_ayah_span: u32,
) -> Result<SearchOutput, SearchError> {
    if allow_cross_ayah {
        return Err(SearchError::Index(IndexError::QueryRejected {
            detail: format!(
                "cross-ayah concatenated search (max span {max_ayah_span}) lands in P2-T45; retry with allow_cross_ayah=false"
            ),
        }));
    }
    let serving = open_serving(db, index_root, params.edition.as_deref()).await?;
    let registry = db_registry(db).await?;
    let profile = quran_normalization::ProfileId::L6;
    let pipeline = quran_normalization::NormalizationPipeline::for_profile(
        &registry,
        profile,
        latest_version(&registry, profile)?,
    )
    .map_err(SearchError::Normalization)?;
    let trace = empty_trace_for(&pipeline);

    // Normalize the query through L6 (spaces vanish here by design).
    let query_derived = pipeline.apply(&params.text);
    let query_skeleton = query_derived.0.text().to_string();
    if query_skeleton.is_empty() {
        return Ok(SearchOutput {
            hits: Vec::new(),
            total_matches: 0,
            truncated: false,
            rule_set: trace.profile.clone(),
            generation: serving.generation,
            warnings: serving.stale.clone().into_iter().collect(),
        });
    }

    // Candidate generation: every trigram must occur (short queries probe
    // the whole skeleton directly). Ayah-level rows only until P2-T45.
    let mut uow = db.write().await.map_err(SearchError::storage)?;
    let mut ayah_numbers: Vec<(u16, u32)> = Vec::new();
    // All ayahs with skeletons: probe in Rust (one ordered read; the T36
    // trigram posting index will replace this scan).
    let surahs =
        uow.quran().list_surahs(&serving.edition_id).await.map_err(SearchError::storage)?;
    uow.rollback().await.map_err(SearchError::storage)?;
    for surah in &surahs {
        let mut uow = db.write().await.map_err(SearchError::storage)?;
        let skeletons = uow
            .quran()
            .list_skeletons(&serving.edition_id, surah.number)
            .await
            .map_err(SearchError::storage)?;
        uow.rollback().await.map_err(SearchError::storage)?;
        let trigrams = query_trigrams(&query_skeleton);
        for row in &skeletons {
            if row.ayah_start != row.ayah_end {
                continue; // windows belong to P2-T45
            }
            let recalled = if trigrams.is_empty() {
                row.skeleton.contains(&query_skeleton)
            } else {
                trigrams.iter().all(|trigram| row.skeleton.contains(trigram))
            };
            if recalled {
                ayah_numbers.push((surah.number as u16, row.ayah_start as u32));
            }
        }
    }
    // Verify the recall set (recall is not precision): ayahs whose
    // skeleton contains the trigrams but not the query drop out here.
    // Metadata filters apply before verification (same semantics as FTS).
    ayah_numbers.sort();
    ayah_numbers.dedup();
    let revelation = surah_revelation(db, &serving.edition_id).await?;
    let mut verified: Vec<Candidate> = Vec::new();
    for (surah, ayah) in &ayah_numbers {
        let mut uow = db.write().await.map_err(SearchError::storage)?;
        let ayah_row = uow
            .quran()
            .get_ayah(&serving.edition_id, i64::from(*surah), i64::from(*ayah))
            .await
            .map_err(SearchError::storage)?;
        uow.rollback().await.map_err(SearchError::storage)?;
        let Some(ayah_row) = ayah_row else { continue };
        if !passes_filters(&ayah_row, revelation.get(&(ayah_row.surah)).map(String::as_str), &params.filters) {
            continue;
        }
        let tokens = ayah_tokens(db, &serving.edition_id, *surah, *ayah).await?;
        if verify_concatenated(&pipeline, &ayah_row.text, &tokens, &query_skeleton).is_some() {
            verified.push(Candidate { surah: *surah, ayah: *ayah, score: None });
        }
    }
    let total_matches = verified.len() as u64;
    let field = "skeleton".to_string();
    let stats = stats_docs(&serving).await?;
    let ctx = RunContext { db, serving: &serving, trace, params, field: &field, stats_docs: stats };
    let query_owned = query_skeleton.clone();
    run_search(&ctx, verified, total_matches, |ayah_text, tokens| {
        verify_concatenated(&pipeline, ayah_text, tokens, &query_owned)
    })
    .await
}

/// Verify one ayah against a skeleton query: exact substring in derived
/// space, canonical span via the pipeline map, re-normalization check, and
/// token segmentation.
///
/// Public so the M3 tools and harnesses reuse the single verify path.
pub fn verify_concatenated(
    pipeline: &quran_normalization::NormalizationPipeline,
    ayah_text: &str,
    tokens: &[storage::quran::TokenRow],
    query_skeleton: &str,
) -> Option<AyahMatch> {
    let derived = pipeline.apply(ayah_text).0;
    let text = derived.text();
    let byte = text.find(query_skeleton)?;
    let start = text[..byte].chars().count() as u32;
    let end = start + query_skeleton.chars().count() as u32;
    let span = derived.spans().to_canonical(start..end);
    // Re-normalization check (plan §3.4 property 5): the sliced canonical
    // text must reproduce the matched derived substring.
    let slice: String = ayah_text
        .chars()
        .enumerate()
        .filter(|(index, _)| {
            *index as u32 >= span.char_range.start && (*index as u32) < span.char_range.end
        })
        .map(|(_, ch)| ch)
        .collect();
    let reapplied = pipeline.apply(&slice).0;
    if reapplied.text() != query_skeleton {
        return None;
    }
    let matched = overlapping_tokens(ayah_text, tokens, &span);
    if matched.is_empty() {
        return None;
    }
    let derived_len = derived.text().chars().count() as u32;
    let segmentation = segment_concatenated(
        query_skeleton,
        ayah_text,
        start,
        &span,
        derived.spans(),
        derived_len,
        tokens,
    );
    Some(AyahMatch { span, matched, segmentation })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens_of(text: &str) -> Vec<(std::ops::Range<u32>, String)> {
        derived_tokens(text).into_iter().map(|(range, token)| (range, token.to_string())).collect()
    }

    #[test]
    fn ordered_modes_respect_gaps() {
        let tokens = tokens_of("a b c d");
        let terms = ["b".to_string(), "d".to_string()];
        assert!(
            find_term_sequence(
                &tokens
                    .iter()
                    .map(|(range, token)| (range.clone(), token.as_str()))
                    .collect::<Vec<_>>(),
                &terms,
                PhraseMode::OrderedExact,
                0
            )
            .is_none()
        );
        let (start, end) = find_term_sequence(
            &tokens
                .iter()
                .map(|(range, token)| (range.clone(), token.as_str()))
                .collect::<Vec<_>>(),
            &terms,
            PhraseMode::OrderedNear,
            1,
        )
        .expect("one gap allowed");
        assert_eq!((start, end), (2, 7));
        assert!(
            find_term_sequence(
                &tokens
                    .iter()
                    .map(|(range, token)| (range.clone(), token.as_str()))
                    .collect::<Vec<_>>(),
                &terms,
                PhraseMode::OrderedNear,
                0,
            )
            .is_none()
        );
    }

    #[test]
    fn unordered_mode_ignores_order_within_window() {
        let tokens = tokens_of("x c a b y");
        let terms = ["a".to_string(), "b".to_string(), "c".to_string()];
        // Window of 3 + slop 1 covers positions 1..=4.
        let (start, end) = find_term_sequence(
            &tokens
                .iter()
                .map(|(range, token)| (range.clone(), token.as_str()))
                .collect::<Vec<_>>(),
            &terms,
            PhraseMode::UnorderedNear,
            1,
        )
        .expect("all terms within window");
        assert_eq!((start, end), (2, 7));
        // Without slop the window of exactly 3 still covers them.
        assert!(
            find_term_sequence(
                &tokens
                    .iter()
                    .map(|(range, token)| (range.clone(), token.as_str()))
                    .collect::<Vec<_>>(),
                &terms,
                PhraseMode::UnorderedNear,
                0,
            )
            .is_some()
        );
    }

    #[test]
    fn derived_token_offsets_track_characters() {
        let tokens = derived_tokens("بسم الله");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].0, 0..3);
        assert_eq!(tokens[1].0, 4..8);
    }
}
