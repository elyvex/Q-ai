//! Search service surface for the API and CLI (P2-T51/T52).
//!
//! # application::quran_search_api
//!
//! Thin dispatch over [`crate::quran_search`] services. Routing lives here so
//! `server` and `cli` gain no new workspace edges (`arch-check`): both crates
//! already depend on `application`. No canonical access happens here beyond
//! what the underlying read-only search services do.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use storage_sqlite::SqliteDatabase;

use crate::quran_search::{
    ExactField, MatchMode, NormalizedProfile, PhraseMode, SearchError, SearchOutput, SearchParams,
};
use quran_search::{Filter, IndexError};

/// Arguments for `quran.search_exact`.
pub struct ExactArgs {
    /// Shared tool parameters.
    pub params: SearchParams,
    /// Exact-search field (`L0`/`L1`).
    pub field: ExactField,
}

/// Arguments for `quran.search_normalized`.
pub struct NormalizedArgs {
    /// Shared tool parameters.
    pub params: SearchParams,
    /// Registry profile or adhoc rule list (never both, by type).
    pub profile: NormalizedProfile,
}

/// Arguments for `quran.search_phrase`.
pub struct PhraseArgs {
    /// Shared tool parameters.
    pub params: SearchParams,
    /// Registry profile or adhoc rule list.
    pub profile: NormalizedProfile,
    /// Phrase mode.
    pub mode: PhraseMode,
    /// Max intervening tokens for `*_near` modes.
    pub slop: u32,
}

/// Arguments for `quran.search_concatenated`.
pub struct ConcatenatedArgs {
    /// Shared tool parameters.
    pub params: SearchParams,
    /// Whether 3-ayah window matches may serve.
    pub allow_cross_ayah: bool,
    /// Max ayahs per window match (`>= 1`).
    pub max_ayah_span: u32,
}

/// Arguments for `quran.search_regex`.
pub struct RegexArgs {
    /// Shared tool parameters.
    pub params: SearchParams,
    /// Indexed normalized field (`text_bare`, …).
    pub field: String,
    /// DFA-safe pattern.
    pub pattern: String,
    /// Rate-limit principal (caller identity, never logged with usage).
    pub principal: String,
    /// Wall-clock budget in ms (clamped to `1..=10_000`).
    pub timeout_ms: u64,
}

/// Read-only search backend. Implemented by [`SearchApiService`]; faked in
/// `server` contract tests.
#[async_trait]
pub trait SearchBackend: Send + Sync {
    /// `quran.search_exact` (P2-T41).
    async fn search_exact(&self, args: ExactArgs) -> Result<SearchOutput, SearchError>;
    /// `quran.search_normalized` (P2-T42).
    async fn search_normalized(&self, args: NormalizedArgs) -> Result<SearchOutput, SearchError>;
    /// `quran.search_phrase` (P2-T43).
    async fn search_phrase(&self, args: PhraseArgs) -> Result<SearchOutput, SearchError>;
    /// `quran.search_concatenated` (P2-T44/T45).
    async fn search_concatenated(
        &self,
        args: ConcatenatedArgs,
    ) -> Result<SearchOutput, SearchError>;
    /// `quran.search_regex` (P2-T46, rate-limited per principal).
    async fn search_regex(&self, args: RegexArgs) -> Result<SearchOutput, SearchError>;
}

/// A `QAI-IDX-0002` query rejection (bad request value, never a panic).
pub fn reject(detail: impl Into<String>) -> SearchError {
    SearchError::Index(quran_search::IndexError::QueryRejected { detail: detail.into() })
}

/// Shared filter type without forcing callers onto the `quran-search` crate.
pub type SearchFilter = quran_search::Filter;

/// Parse structured filters shared by every search surface (CLI strings in,
///
/// typed filters out). Bad values are `QAI-IDX-0002` rejections, never panics.
pub fn build_filters(
    surah: Option<Vec<u16>>,
    juz: Option<&str>,
    page: Option<Vec<u32>>,
    revelation_place: Option<&str>,
    global_range: Option<&str>,
) -> Result<Vec<Filter>, SearchError> {
    let reject = |detail: String| SearchError::Index(IndexError::QueryRejected { detail });
    let mut filters = Vec::new();
    if let Some(surah) = surah {
        if surah.is_empty() {
            return Err(reject("surah filter is empty".to_string()));
        }
        filters.push(Filter::Surah(surah));
    }
    if let Some(juz) = juz {
        let (lo, hi) = parse_lo_hi(juz, "juz")?;
        let (lo, hi) = (u16::try_from(lo).ok(), u16::try_from(hi).ok());
        match (lo, hi) {
            (Some(lo), Some(hi)) => filters.push(Filter::JuzRange(lo, hi)),
            _ => return Err(reject(format!("bad juz `{juz}`; use `2` or `1-5`"))),
        }
    }
    if let Some(page) = page {
        if page.is_empty() {
            return Err(reject("page filter is empty".to_string()));
        }
        filters.push(Filter::Page(page));
    }
    if let Some(place) = revelation_place {
        if !matches!(place, "makki" | "madani") {
            return Err(reject(format!("bad revelation place `{place}`; use `makki` or `madani`")));
        }
        filters.push(Filter::RevelationPlace(place.to_string()));
    }
    if let Some(range) = global_range {
        let (lo, hi) = parse_lo_hi(range, "global range")?;
        filters.push(Filter::GlobalRange(lo, hi));
    }
    Ok(filters)
}

fn parse_lo_hi(raw: &str, what: &str) -> Result<(u64, u64), SearchError> {
    let reject = || {
        SearchError::Index(IndexError::QueryRejected {
            detail: format!("bad {what} `{raw}`; use `LO-HI`"),
        })
    };
    match raw.split_once('-') {
        Some((lo, hi)) => match (lo.trim().parse::<u64>(), hi.trim().parse::<u64>()) {
            (Ok(lo), Ok(hi)) if lo <= hi => Ok((lo, hi)),
            _ => Err(reject()),
        },
        None => match raw.trim().parse::<u64>() {
            Ok(value) => Ok((value, value)),
            Err(_) => Err(reject()),
        },
    }
}

/// Resolve the normalized-search profile selector: registry profile (default
/// `L3.diacritics`) or an explicit adhoc rule list, never both.
pub fn parse_normalized_profile(
    profile: Option<&str>,
    rules: Option<&str>,
) -> Result<NormalizedProfile, SearchError> {
    use NormalizedProfile;
    let reject = |detail: String| SearchError::Index(IndexError::QueryRejected { detail });
    match (profile, rules) {
        (Some(_), Some(_)) => Err(reject("use either profile or rules, never both".to_string())),
        (Some(spec), None) => crate::quran_normalize::parse_profile_spec(spec)
            .map(|(id, version)| NormalizedProfile::Registry(id, version))
            .map_err(|err| reject(err.to_string())),
        (None, Some(list)) => crate::quran_normalize::parse_rule_list(list)
            .map(NormalizedProfile::Adhoc)
            .map_err(|err| reject(err.to_string())),
        (None, None) => Ok(NormalizedProfile::Registry(quran_normalization::ProfileId::L3, None)),
    }
}

/// Parse a token match mode name (defaults to `whole_token`).
pub fn parse_match_mode(raw: Option<&str>) -> Result<MatchMode, SearchError> {
    use MatchMode;
    match raw.unwrap_or("whole_token") {
        "whole_token" => Ok(MatchMode::WholeToken),
        "substring" => Ok(MatchMode::Substring),
        "ayah_prefix" => Ok(MatchMode::AyahPrefix),
        other => Err(SearchError::Index(IndexError::QueryRejected {
            detail: format!(
                "unknown match mode `{other}`; use `whole_token`, `substring`, or `ayah_prefix`"
            ),
        })),
    }
}

/// Parse an exact-search field name (defaults to `text_exact`).
pub fn parse_exact_field(raw: Option<&str>) -> Result<ExactField, SearchError> {
    use ExactField;
    match raw.unwrap_or("text_exact") {
        "text_exact" => Ok(ExactField::TextExact),
        "text_ws" => Ok(ExactField::TextWs),
        other => Err(SearchError::Index(IndexError::QueryRejected {
            detail: format!("unknown exact field `{other}`; use `text_exact` or `text_ws`"),
        })),
    }
}

/// Parse a phrase mode name (defaults to `ordered_exact`).
pub fn parse_phrase_mode(raw: Option<&str>) -> Result<PhraseMode, SearchError> {
    use PhraseMode;
    match raw.unwrap_or("ordered_exact") {
        "ordered_exact" => Ok(PhraseMode::OrderedExact),
        "ordered_near" => Ok(PhraseMode::OrderedNear),
        "unordered_near" => Ok(PhraseMode::UnorderedNear),
        other => Err(SearchError::Index(IndexError::QueryRejected {
            detail: format!(
                "unknown phrase mode `{other}`; use `ordered_exact`, `ordered_near`, or `unordered_near`"
            ),
        })),
    }
}

/// Build shared tool parameters with ceiling enforcement.
#[allow(clippy::too_many_arguments)]
pub fn search_params(
    text: String,
    edition: Option<String>,
    mode: MatchMode,
    filters: Vec<Filter>,
    limit: Option<u32>,
    offset: Option<u32>,
    explain: Option<bool>,
    highlight: Option<bool>,
) -> SearchParams {
    SearchParams {
        text,
        edition,
        mode,
        filters,
        limit: limit.unwrap_or(20).clamp(1, 1000),
        offset: offset.unwrap_or(0),
        explain: explain.unwrap_or(false),
        highlight: highlight.unwrap_or(false),
    }
}
/// Live backend over a database file plus its sibling `index/` directory.
pub struct SearchApiService {
    db: Arc<SqliteDatabase>,
    index_root: PathBuf,
    limiter: crate::quran_search::RateLimiter,
}

impl SearchApiService {
    /// Open the database at `db_path`; the index root is derived with
    /// [`crate::quran_index::index_root_for_db`].
    ///
    /// # Errors
    ///
    /// Returns a message when the database cannot be opened.
    pub async fn open(db_path: &str) -> Result<Self, String> {
        let db = SqliteDatabase::new(db_path, 4, true).await.map_err(|err| err.to_string())?;
        Ok(Self {
            db: Arc::new(db),
            index_root: crate::quran_index::index_root_for_db(db_path),
            limiter: crate::quran_search::RateLimiter::default_regex(),
        })
    }
}

#[async_trait]
impl SearchBackend for SearchApiService {
    async fn search_exact(&self, args: ExactArgs) -> Result<SearchOutput, SearchError> {
        crate::quran_search::search_exact(&self.db, &self.index_root, &args.params, args.field)
            .await
    }

    async fn search_normalized(&self, args: NormalizedArgs) -> Result<SearchOutput, SearchError> {
        crate::quran_search::search_normalized(
            &self.db,
            &self.index_root,
            &args.params,
            args.profile,
        )
        .await
    }

    async fn search_phrase(&self, args: PhraseArgs) -> Result<SearchOutput, SearchError> {
        crate::quran_search::search_phrase(
            &self.db,
            &self.index_root,
            &args.params,
            args.profile,
            args.mode,
            args.slop,
        )
        .await
    }

    async fn search_concatenated(
        &self,
        args: ConcatenatedArgs,
    ) -> Result<SearchOutput, SearchError> {
        crate::quran_search::search_concatenated(
            &self.db,
            &self.index_root,
            &args.params,
            args.allow_cross_ayah,
            args.max_ayah_span,
        )
        .await
    }

    async fn search_regex(&self, args: RegexArgs) -> Result<SearchOutput, SearchError> {
        crate::quran_search::search_regex(
            &self.db,
            &self.index_root,
            &args.params,
            &args.field,
            &args.pattern,
            &args.principal,
            &self.limiter,
            args.timeout_ms,
        )
        .await
    }
}
