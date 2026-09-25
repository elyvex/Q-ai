//! Quran read API v1 + health endpoints (D1.7, P1-T39).
//!
//! # server::api
//!
//! axum handlers over a [`QuranApiBackend`] trait (implemented by the
//! application layer) plus [`ToolRegistry`] for ayah/context endpoints, so
//! contract tests run against fakes with no database. Response envelope
//! (stable, reused by later phases):
//!
//! ```json
//! {
//!   "api_version": "v1",
//!   "data": {},
//!   "meta": {
//!     "edition": {"slug": "", "version": "", "text_hash": "",
//!                 "script": "", "riwayah": null, "numbering_scheme": ""},
//!     "corpus_generation": 0,
//!     "canonical_reference": "",
//!     "deep_link": "",
//!     "execution_time_ms": 0.0,
//!     "reproducibility": {},
//!     "warnings": []
//!   }
//! }
//! ```
//!
//! Cross-cutting: `ETag` derives from `(text_hash, corpus_generation)`,
//! `Cache-Control` is present on canonical resources, Arabic payloads carry
//! `Content-Language`, and errors use the Phase-0 `Diagnostic` body shape.

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use axum::Router;
use axum::extract::{Json, Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use quran_core::{AyahOptions, AyahView, Surah};
use serde::{Deserialize, Serialize};
use tool_registry::{BackendMeta, ToolRegistry};
use tools::{ToolError, ToolResult};

/// API version string.
pub const API_VERSION: &str = "v1";

/// Edition descriptor in the envelope meta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditionMeta {
    /// Edition slug.
    pub slug: String,
    /// Edition version.
    pub version: String,
    /// Edition text hash.
    pub text_hash: String,
    /// Script.
    pub script: String,
    /// Transmission, when declared.
    pub riwayah: Option<String>,
    /// Numbering scheme.
    pub numbering_scheme: String,
}

/// Envelope metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    /// Edition descriptor (empty slug when no edition is in scope).
    pub edition: EditionMeta,
    /// Corpus generation read from.
    pub corpus_generation: u64,
    /// Fully-qualified canonical reference (empty for listings).
    pub canonical_reference: String,
    /// Reader deep link (empty for listings).
    pub deep_link: String,
    /// Wall-clock execution time.
    pub execution_time_ms: f64,
    /// Reproducibility record (tool calls) or checksum-only.
    pub reproducibility: serde_json::Value,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
}

/// The stable response envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T> {
    /// API version.
    pub api_version: &'static str,
    /// Payload.
    pub data: T,
    /// Metadata.
    pub meta: Meta,
}

/// Phase-0 `Diagnostic` error body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    /// Error envelope.
    pub error: ErrorDetail,
}

/// Error details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    /// Namespaced code.
    pub code: String,
    /// Human summary.
    pub summary: String,
    /// Location, when known.
    pub location: Option<String>,
    /// Cause chain.
    pub why: Vec<String>,
    /// Remedy, when known.
    pub remedy: Option<String>,
    /// Next command.
    pub next_command: Option<String>,
}

/// Reader surface the HTTP layer needs. Implemented by `application`.
#[async_trait]
pub trait QuranApiBackend: Send + Sync {
    /// List editions as JSON values.
    async fn api_editions(&self) -> Result<Vec<serde_json::Value>, ToolError>;
    /// One edition by slug (active version).
    async fn api_edition(&self, slug: &str) -> Result<serde_json::Value, ToolError>;
    /// Surahs of an edition selector string.
    async fn api_surahs(&self, edition: &str) -> Result<Vec<Surah>, ToolError>;
    /// One surah with its ayah views.
    async fn api_surah(
        &self,
        edition: &str,
        surah: u16,
        options: &AyahOptions,
    ) -> Result<(Surah, Vec<AyahView>), ToolError>;
    /// Ayahs of one division.
    async fn api_division(
        &self,
        edition: &str,
        kind: &str,
        number: u32,
        options: &AyahOptions,
    ) -> Result<Vec<AyahView>, ToolError>;
    /// Tokens for a fully-qualified ayah reference string.
    async fn api_tokens(&self, reference: &str) -> Result<Vec<quran_core::Token>, ToolError>;
    /// Resolve a stored citation by id.
    async fn api_citation(&self, id: &str) -> Result<citations::ResolvedCitation, ToolError>;
}

/// Shared handler state.
#[derive(Clone)]
pub struct AppState {
    /// Tool registry for ayah/context endpoints.
    pub tools: Arc<ToolRegistry>,
    /// Reader backend for listings and structure endpoints.
    pub api: Arc<dyn QuranApiBackend>,
    /// Read-only search backend (P2-T51).
    pub search: Arc<dyn application::quran_search_api::SearchBackend>,
}

fn empty_meta() -> Meta {
    Meta {
        edition: EditionMeta {
            slug: String::new(),
            version: String::new(),
            text_hash: String::new(),
            script: String::new(),
            riwayah: None,
            numbering_scheme: String::new(),
        },
        corpus_generation: 0,
        canonical_reference: String::new(),
        deep_link: String::new(),
        execution_time_ms: 0.0,
        reproducibility: serde_json::Value::Null,
        warnings: Vec::new(),
    }
}

fn meta_from_tool<T>(
    result: &ToolResult<T>,
    meta: &BackendMeta,
    started: Instant,
    deep_link: String,
) -> Meta {
    Meta {
        edition: EditionMeta {
            slug: meta.edition_slug.clone(),
            version: meta.edition_version.clone(),
            text_hash: meta.text_hash.clone(),
            script: meta.script.clone(),
            riwayah: meta.riwayah.clone(),
            numbering_scheme: meta.numbering_scheme.clone(),
        },
        corpus_generation: meta.corpus_generation,
        canonical_reference: result.canonical_references.first().cloned().unwrap_or_default(),
        deep_link,
        execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
        reproducibility: serde_json::to_value(&result.reproducibility)
            .unwrap_or(serde_json::Value::Null),
        warnings: result.warnings.clone(),
    }
}

fn etag_for(meta: &Meta) -> Option<HeaderValue> {
    if meta.edition.text_hash.is_empty() {
        return None;
    }
    HeaderValue::from_str(&format!("\"{}:{}\"", meta.edition.text_hash, meta.corpus_generation))
        .ok()
}

fn tool_status(error: &ToolError) -> StatusCode {
    match error {
        ToolError::InvalidInput { .. } => StatusCode::BAD_REQUEST,
        ToolError::Backend { code, .. } => match code.as_str() {
            "QAI-QUR-0306" | "QAI-QUR-0307" | "QAI-QUR-0308" | "QAI-QUR-0309" | "QAI-QUR-0322" => {
                StatusCode::NOT_FOUND
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        },
    }
}

fn error_body(error: &ToolError) -> ErrorBody {
    let (code, summary) = match error {
        ToolError::InvalidInput { tool, detail } => {
            (error.code().to_string(), format!("invalid input for {tool}: {detail}"))
        }
        ToolError::Backend { code, detail } => (code.clone(), detail.clone()),
    };
    ErrorBody {
        error: ErrorDetail {
            code,
            summary,
            location: None,
            why: Vec::new(),
            remedy: None,
            next_command: Some("qai quran resolve --help".to_string()),
        },
    }
}

fn json_response<T: Serialize>(
    status: StatusCode,
    body: &T,
    etag: Option<HeaderValue>,
    content_language: bool,
) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert("content-type", HeaderValue::from_static("application/json; charset=utf-8"));
    headers.insert("cache-control", HeaderValue::from_static("public, max-age=3600"));
    if content_language {
        headers.insert("content-language", HeaderValue::from_static("ar"));
    }
    if let Some(etag) = etag {
        headers.insert("etag", etag);
    }
    let bytes = serde_json::to_vec(body).unwrap_or_default();
    (status, headers, bytes).into_response()
}

fn ok_envelope<T: Serialize>(data: T, meta: Meta, headers: &HeaderMap) -> Response {
    // Conditional requests: matching ETag short-circuits the body.
    let etag = etag_for(&meta);
    if etag.as_ref().is_some_and(|current| headers.get("if-none-match") == Some(current)) {
        return StatusCode::NOT_MODIFIED.into_response();
    }
    json_response(StatusCode::OK, &Envelope { api_version: API_VERSION, data, meta }, etag, true)
}

fn tool_error_response(error: ToolError) -> Response {
    json_response(tool_status(&error), &error_body(&error), None, false)
}

#[derive(Debug, Deserialize, Default)]
struct EditionQuery {
    status: Option<String>,
    language: Option<String>,
}

async fn editions_handler(
    State(state): State<AppState>,
    Query(query): Query<EditionQuery>,
) -> Response {
    let started = Instant::now();
    let editions = match state.api.api_editions().await {
        Ok(editions) => editions,
        Err(error) => return tool_error_response(error),
    };
    let data: Vec<_> = editions
        .into_iter()
        .filter(|edition| {
            query.status.as_deref().is_none_or(|status| {
                edition
                    .get("status")
                    .and_then(|value| value.as_str())
                    .is_some_and(|value| value.eq_ignore_ascii_case(status))
            }) && query.language.as_deref().is_none_or(|language| {
                edition
                    .get("language")
                    .and_then(|value| value.as_str())
                    .is_some_and(|value| value == language)
            })
        })
        .collect();
    let mut meta = empty_meta();
    meta.execution_time_ms = started.elapsed().as_secs_f64() * 1000.0;
    json_response(StatusCode::OK, &Envelope { api_version: API_VERSION, data, meta }, None, false)
}

async fn edition_handler(State(state): State<AppState>, Path(slug): Path<String>) -> Response {
    let started = Instant::now();
    let edition = match state.api.api_edition(&slug).await {
        Ok(edition) => edition,
        Err(error) => return tool_error_response(error),
    };
    let mut meta = empty_meta();
    meta.execution_time_ms = started.elapsed().as_secs_f64() * 1000.0;
    json_response(
        StatusCode::OK,
        &Envelope { api_version: API_VERSION, data: edition, meta },
        None,
        false,
    )
}

#[derive(Debug, Deserialize, Default)]
struct SurahsQuery {
    edition: Option<String>,
}

async fn surahs_handler(
    State(state): State<AppState>,
    Query(query): Query<SurahsQuery>,
) -> Response {
    let started = Instant::now();
    let surahs = match state.api.api_surahs(query.edition.as_deref().unwrap_or("")).await {
        Ok(surahs) => surahs,
        Err(error) => return tool_error_response(error),
    };
    let data: Vec<serde_json::Value> = surahs
        .iter()
        .map(|surah| {
            serde_json::json!({
                "number": surah.number.get(),
                "name_arabic": surah.name_arabic,
                "name_transliteration": surah.name_transliteration,
                "ayah_count": surah.ayah_count,
            })
        })
        .collect();
    let mut meta = empty_meta();
    meta.execution_time_ms = started.elapsed().as_secs_f64() * 1000.0;
    json_response(StatusCode::OK, &Envelope { api_version: API_VERSION, data, meta }, None, false)
}

async fn surah_handler(
    State(state): State<AppState>,
    Path(number): Path<u16>,
    Query(query): Query<SurahsQuery>,
) -> Response {
    let started = Instant::now();
    let options = AyahOptions::default();
    let (surah, ayahs) =
        match state.api.api_surah(query.edition.as_deref().unwrap_or(""), number, &options).await {
            Ok(value) => value,
            Err(error) => return tool_error_response(error),
        };
    let data = serde_json::json!({
        "number": surah.number.get(),
        "name_arabic": surah.name_arabic,
        "ayahs": ayahs.iter().map(|view| view.canonical.arabic_text()).collect::<Vec<_>>(),
    });
    let mut meta = empty_meta();
    meta.execution_time_ms = started.elapsed().as_secs_f64() * 1000.0;
    if let Some(first) = ayahs.first() {
        meta.canonical_reference = first.canonical.reference().to_string();
        meta.deep_link = first.canonical.deep_link().to_string();
    }
    let etag = etag_for(&meta);
    json_response(StatusCode::OK, &Envelope { api_version: API_VERSION, data, meta }, etag, true)
}

#[derive(Debug, Deserialize, Default)]
struct AyahsQuery {
    edition: Option<String>,
    translations: Option<String>,
    glosses: Option<bool>,
    tokens: Option<bool>,
}

fn ayah_options(query: &AyahsQuery) -> AyahOptions {
    AyahOptions {
        translations: query
            .translations
            .as_deref()
            .unwrap_or("")
            .split(',')
            .filter(|part| !part.trim().is_empty())
            .map(|part| part.trim().to_string())
            .collect(),
        glosses: query.glosses.unwrap_or(false),
        tokens: query.tokens.unwrap_or(false),
    }
}

fn qualified(reference: &str, edition: Option<&str>) -> String {
    match edition {
        Some(edition) if !edition.is_empty() && !reference.starts_with("quran:") => {
            format!("quran:{edition}:{reference}")
        }
        _ => reference.to_string(),
    }
}

async fn ayahs_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(reference): Path<String>,
    Query(query): Query<AyahsQuery>,
) -> Response {
    let started = Instant::now();
    let params = tool_registry::GetAyahParams {
        reference: qualified(&reference, query.edition.as_deref()),
        translations: ayah_options(&query).translations,
        glosses: query.glosses.unwrap_or(false),
        tokens: query.tokens.unwrap_or(false),
    };
    let (result, meta) = match state.tools.get_ayah(params).await {
        Ok(value) => value,
        Err(error) => return tool_error_response(error),
    };
    let deep_link = result
        .results
        .first()
        .map(|view| view.canonical.deep_link().to_string())
        .unwrap_or_default();
    let envelope_meta = meta_from_tool(&result, &meta, started, deep_link);
    ok_envelope(result.results, envelope_meta, &headers)
}

#[derive(Debug, Deserialize, Default)]
struct ContextQuery {
    edition: Option<String>,
    before: Option<u16>,
    after: Option<u16>,
    boundary: Option<String>,
    max_ayahs: Option<u16>,
}

async fn context_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(reference): Path<String>,
    Query(query): Query<ContextQuery>,
) -> Response {
    let started = Instant::now();
    let boundary = match query.boundary.as_deref().unwrap_or("surah") {
        "surah" => tool_registry::ContextBoundaryArg::Surah,
        "juz" => tool_registry::ContextBoundaryArg::Juz,
        "ruku" => tool_registry::ContextBoundaryArg::Ruku,
        "page" => tool_registry::ContextBoundaryArg::Page,
        "none" => tool_registry::ContextBoundaryArg::None,
        other => {
            return tool_error_response(ToolError::InvalidInput {
                tool: "quran.get_context",
                detail: format!("unknown boundary `{other}`"),
            });
        }
    };
    let params = tool_registry::GetContextParams {
        reference: qualified(&reference, query.edition.as_deref()),
        before: query.before.unwrap_or(3),
        after: query.after.unwrap_or(3),
        boundary,
        max_ayahs: query.max_ayahs.unwrap_or(11),
    };
    let (result, meta) = match state.tools.get_context(params).await {
        Ok(value) => value,
        Err(error) => return tool_error_response(error),
    };
    let envelope_meta = meta_from_tool(&result, &meta, started, String::new());
    ok_envelope(result.results, envelope_meta, &headers)
}

async fn divisions_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((kind, number)): Path<(String, u32)>,
    Query(query): Query<SurahsQuery>,
) -> Response {
    let started = Instant::now();
    let views = match state
        .api
        .api_division(
            query.edition.as_deref().unwrap_or(""),
            &kind,
            number,
            &AyahOptions::default(),
        )
        .await
    {
        Ok(views) => views,
        Err(error) => return tool_error_response(error),
    };
    let data: Vec<serde_json::Value> = views
        .iter()
        .map(|view| {
            serde_json::json!({
                "canonical_reference": view.canonical.reference(),
                "arabic_text": view.canonical.arabic_text(),
            })
        })
        .collect();
    let mut meta = empty_meta();
    meta.execution_time_ms = started.elapsed().as_secs_f64() * 1000.0;
    if let Some(first) = views.first() {
        meta.canonical_reference = first.canonical.reference().to_string();
    }
    let _ = headers;
    json_response(StatusCode::OK, &Envelope { api_version: API_VERSION, data, meta }, None, true)
}

async fn tokens_handler(
    State(state): State<AppState>,
    Path(reference): Path<String>,
    Query(query): Query<SurahsQuery>,
) -> Response {
    let started = Instant::now();
    let tokens = match state.api.api_tokens(&qualified(&reference, query.edition.as_deref())).await
    {
        Ok(tokens) => tokens,
        Err(error) => return tool_error_response(error),
    };
    let data: Vec<serde_json::Value> = tokens
        .iter()
        .map(|token| {
            serde_json::json!({
                "position": token.position.get(),
                "surface": token.surface,
                "char_start": token.char_start,
                "char_end": token.char_end,
                "byte_start": token.byte_start,
                "byte_end": token.byte_end,
                "is_pause_mark": token.is_pause_mark,
            })
        })
        .collect();
    let mut meta = empty_meta();
    meta.execution_time_ms = started.elapsed().as_secs_f64() * 1000.0;
    json_response(StatusCode::OK, &Envelope { api_version: API_VERSION, data, meta }, None, true)
}

#[derive(Debug, Deserialize)]
struct ResolveQuery {
    #[serde(rename = "ref")]
    reference: String,
}

async fn resolve_handler(
    State(state): State<AppState>,
    Query(query): Query<ResolveQuery>,
) -> Response {
    let started = Instant::now();
    let params = tool_registry::GetAyahParams {
        reference: query.reference.clone(),
        translations: Vec::new(),
        glosses: false,
        tokens: false,
    };
    let (result, _meta) = match state.tools.get_ayah(params).await {
        Ok(value) => value,
        Err(error) => return tool_error_response(error),
    };
    let data = serde_json::json!({
        "canonical_reference": result.canonical_references.first(),
        "edition_version": result.edition_version,
    });
    let mut meta = empty_meta();
    meta.execution_time_ms = started.elapsed().as_secs_f64() * 1000.0;
    json_response(StatusCode::OK, &Envelope { api_version: API_VERSION, data, meta }, None, false)
}

async fn citation_handler(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    let started = Instant::now();
    let resolved = match state.api.api_citation(&id).await {
        Ok(resolved) => resolved,
        Err(error) => return tool_error_response(error),
    };
    // A stored verdict that is a hard failure must never be served as a 200
    // success (T-02-18). This is the same shared mapping the CLI uses
    // (`citations::require_exact`), so a mismatch cannot fail on one surface
    // while being softened into a success on another.
    if let Err(error) = citations::require_exact(&resolved.verdict) {
        return tool_error_response(ToolError::Backend {
            code: error.code().to_string(),
            detail: error.to_string(),
        });
    }
    let mut meta = empty_meta();
    meta.execution_time_ms = started.elapsed().as_secs_f64() * 1000.0;
    json_response(
        StatusCode::OK,
        &Envelope { api_version: API_VERSION, data: resolved, meta },
        None,
        false,
    )
}

/// Preview normalization without touching the database (M1c, P2-T24).
///
/// Runs the same [`application::quran_normalize`] pipeline the CLI uses, so
/// the trace here is byte-identical to `qai quran normalize --json` (AC-P2-39).
#[derive(Debug, Deserialize)]
struct PreviewRequest {
    /// Text to normalize.
    text: String,
    /// Profile (`L3.diacritics`, optionally `@version`-pinned).
    /// Defaults to the latest `L3.diacritics`.
    profile: Option<String>,
    /// Explicit rule list (`N01,N03`); never with `profile`.
    rules: Option<String>,
}

fn norm_error_response(error: &application::quran_normalize::NormalizationError) -> Response {
    use application::quran_normalize::{NormalizationDiagnostic as _, NormalizationError as E};
    let status = match error {
        E::UnknownRule { .. } | E::UnknownProfile { .. } | E::InvalidMapping { .. } => {
            StatusCode::BAD_REQUEST
        }
        E::ProfileImmutable { .. } | E::SpanOutOfRange { .. } | E::EmptyProfile => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    };
    json_response(
        status,
        &ErrorBody {
            error: ErrorDetail {
                code: error.code().to_string(),
                summary: error.summary(),
                location: error.location(),
                why: error.cause_chain(),
                remedy: error.remedy(),
                next_command: error.next_command(),
            },
        },
        None,
        false,
    )
}

async fn normalization_preview_handler(Json(body): Json<PreviewRequest>) -> Response {
    use application::quran_normalize;

    let started = Instant::now();
    if body.profile.is_some() && body.rules.is_some() {
        return norm_error_response(&quran_normalize::NormalizationError::InvalidMapping {
            detail: "use either profile or rules, never both".to_string(),
        });
    }
    let registry = quran_normalize::builtin_registry();
    let preview = if let Some(rules) = body.rules.as_deref() {
        let rule_ids = match quran_normalize::parse_rule_list(rules) {
            Ok(ids) => ids,
            Err(error) => return norm_error_response(&error),
        };
        match quran_normalize::preview_adhoc(&body.text, &rule_ids) {
            Ok(preview) => preview,
            Err(error) => return norm_error_response(&error),
        }
    } else {
        let profile = body.profile.as_deref().unwrap_or("L3.diacritics");
        let (id, version) = match quran_normalize::parse_profile_spec(profile) {
            Ok(spec) => spec,
            Err(error) => return norm_error_response(&error),
        };
        match quran_normalize::preview(&registry, &body.text, id, version) {
            Ok(preview) => preview,
            Err(error) => return norm_error_response(&error),
        }
    };
    let data = serde_json::json!({
        "input": preview.input,
        "profile": preview.profile,
        "output": preview.output,
        "trace": preview.trace,
        "steps": preview.steps,
    });
    let mut meta = empty_meta();
    meta.execution_time_ms = started.elapsed().as_secs_f64() * 1000.0;
    json_response(StatusCode::OK, &Envelope { api_version: API_VERSION, data, meta }, None, false)
}

/// Seeded profile catalog over the built-in ladder (M1c, P2-T24).
async fn normalization_profiles_handler() -> Response {
    use application::quran_normalize;
    let started = Instant::now();
    let data = serde_json::json!({
        "profiles": quran_normalize::builtin_registry_profiles(),
    });
    let mut meta = empty_meta();
    meta.execution_time_ms = started.elapsed().as_secs_f64() * 1000.0;
    json_response(StatusCode::OK, &Envelope { api_version: API_VERSION, data, meta }, None, false)
}

/// Search endpoints over the five lexical tools (M3, P2-T51).
///
/// Every endpoint serves the same read-only services the CLI uses
/// (`application::quran_search_api::SearchBackend`), wrapped in the stable
/// envelope. `Accept: text/event-stream` switches the representation to SSE:
/// one `hit` event per match plus a terminal `totals` event carrying the
/// exact total and the reproducibility block (§10 streaming).
/// Structured metadata filters shared by every search endpoint.
#[derive(Debug, Deserialize, Default)]
struct SearchFiltersBody {
    /// Restrict to surahs.
    surah: Option<Vec<u16>>,
    /// Juz filter (`2` or `1-5`).
    juz: Option<String>,
    /// Restrict to pages.
    page: Option<Vec<u32>>,
    /// Revelation-place filter (`makki`|`madani`).
    revelation_place: Option<String>,
    /// Global ayah-index filter (`10-99`).
    global_range: Option<String>,
}

/// Paging/explain envelope shared by every search endpoint.
#[derive(Debug, Deserialize, Default)]
struct SearchPagingBody {
    /// Edition `slug@version` (defaults to the indexed edition).
    edition: Option<String>,
    /// Token match mode (`whole_token`|`substring`|`ayah_prefix`).
    match_mode: Option<String>,
    /// Result cap (ceiling 1000).
    limit: Option<u32>,
    /// Result offset.
    offset: Option<u32>,
    /// Relevance order with per-hit BM25 breakdowns.
    explain: Option<bool>,
    /// Wrap hit spans in `<b>` display markers.
    highlight: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ExactSearchBody {
    /// Query text.
    text: String,
    /// Exact-search field (`text_exact`|`text_ws`).
    field: Option<String>,
    /// Metadata filters.
    filters: Option<SearchFiltersBody>,
    /// Paging/explain options.
    #[serde(flatten)]
    paging: SearchPagingBody,
}

#[derive(Debug, Deserialize)]
struct NormalizedSearchBody {
    /// Query text.
    text: String,
    /// Registry profile (`L3.diacritics`, optionally `@version`-pinned).
    profile: Option<String>,
    /// Explicit rule list (`N01,N03`); never with `profile`.
    rules: Option<String>,
    /// Metadata filters.
    filters: Option<SearchFiltersBody>,
    /// Paging/explain options.
    #[serde(flatten)]
    paging: SearchPagingBody,
}

#[derive(Debug, Deserialize)]
struct PhraseSearchBody {
    /// Query text.
    text: String,
    /// Registry profile or adhoc rules (see normalized search).
    profile: Option<String>,
    /// Explicit rule list; never with `profile`.
    rules: Option<String>,
    /// Phrase mode (`ordered_exact`|`ordered_near`|`unordered_near`).
    phrase_mode: Option<String>,
    /// Max intervening tokens for `*_near` modes.
    slop: Option<u32>,
    /// Metadata filters.
    filters: Option<SearchFiltersBody>,
    /// Paging/explain options.
    #[serde(flatten)]
    paging: SearchPagingBody,
}

#[derive(Debug, Deserialize)]
struct ConcatenatedSearchBody {
    /// Spaceless query text.
    text: String,
    /// Allow 3-ayah window matches.
    allow_cross_ayah: Option<bool>,
    /// Max ayahs per window match.
    max_ayah_span: Option<u32>,
    /// Metadata filters.
    filters: Option<SearchFiltersBody>,
    /// Paging/explain options.
    #[serde(flatten)]
    paging: SearchPagingBody,
}

#[derive(Debug, Deserialize)]
struct RegexSearchBody {
    /// DFA-safe pattern.
    pattern: String,
    /// Indexed normalized field (default `text_bare`).
    field: Option<String>,
    /// Wall-clock budget in ms (default 3000, ceiling 10000).
    timeout_ms: Option<u64>,
    /// Metadata filters.
    filters: Option<SearchFiltersBody>,
    /// Paging/explain options.
    #[serde(flatten)]
    paging: SearchPagingBody,
}

fn search_error_status(error: &application::quran_search::SearchError) -> StatusCode {
    use application::quran_search::SearchError as E;
    use storage::error::Diagnostic as _;
    // 404 first: missing index/edition is a state problem, not a bad query.
    if matches!(error, E::EditionNotIndexed { .. } | E::NoServingIndex { .. }) {
        return StatusCode::NOT_FOUND;
    }
    // Guard rejections travel as codes so `server` never names the
    // `quran-search` error enum (no new workspace edge, `arch-check`).
    match error.code().to_string().as_str() {
        "QAI-IDX-0002" | "QAI-NORM-0001" | "QAI-NORM-0002" => StatusCode::BAD_REQUEST,
        "QAI-IDX-0007" => StatusCode::TOO_MANY_REQUESTS,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn search_error_response(error: application::quran_search::SearchError) -> Response {
    use storage::error::Diagnostic as _;
    json_response(
        search_error_status(&error),
        &ErrorBody {
            error: ErrorDetail {
                code: error.code().to_string(),
                summary: error.summary(),
                location: error.location(),
                why: error.cause_chain(),
                remedy: error.remedy(),
                next_command: error.next_command(),
            },
        },
        None,
        false,
    )
}

/// Split `quran:slug@version:surah:ayah` into its edition parts.
fn edition_of(reference: &str) -> (String, String) {
    let body = reference.strip_prefix("quran:").unwrap_or(reference);
    let mut parts = body.split(':');
    match (parts.next(), parts.next()) {
        (Some(edition), Some(_)) => match edition.split_once('@') {
            Some((slug, version)) => (slug.to_string(), version.to_string()),
            None => (edition.to_string(), String::new()),
        },
        _ => (String::new(), String::new()),
    }
}

fn meta_from_search(
    tool: &str,
    output: &application::quran_search::SearchOutput,
    started: Instant,
) -> Meta {
    let hits = serde_json::to_value(&output.hits).unwrap_or_default();
    let first = hits.as_array().and_then(|hits| hits.first());
    let canonical_reference =
        first.and_then(|hit| hit.get("reference")).and_then(|v| v.as_str()).unwrap_or_default();
    let (slug, version) = edition_of(canonical_reference);
    Meta {
        edition: EditionMeta {
            slug,
            version,
            text_hash: String::new(),
            script: String::new(),
            riwayah: None,
            numbering_scheme: String::new(),
        },
        corpus_generation: output.generation as u64,
        canonical_reference: canonical_reference.to_string(),
        deep_link: first
            .and_then(|hit| hit.get("deep_link"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
        reproducibility: serde_json::json!({
            "tool": tool,
            "rule_set": output.rule_set,
            "generation": output.generation,
        }),
        warnings: output
            .warnings
            .iter()
            .map(|warning| format!("{}: {}", warning.code, warning.message))
            .collect(),
    }
}

fn wants_sse(headers: &HeaderMap) -> bool {
    headers.get("accept").and_then(|value| value.to_str().ok()).is_some_and(|value| {
        value.split(',').any(|part| part.trim().starts_with("text/event-stream"))
    })
}

/// SSE representation: one `hit` event per match, then a terminal `totals`
/// event with the exact total and the reproducibility block.
///
/// v1 buffers the tool output before emitting (the services verify before
/// they emit, so hits are complete when streamed); true incremental emission
/// is a follow-up once the services expose a streaming cursor.
fn sse_response(
    tool: &str,
    output: &application::quran_search::SearchOutput,
    meta: &Meta,
) -> Response {
    let mut body = String::new();
    for hit in &output.hits {
        let data = serde_json::to_string(hit).unwrap_or_default();
        body.push_str(&format!("event: hit\ndata: {data}\n\n"));
    }
    let totals = serde_json::json!({
        "total_matches": output.total_matches,
        "truncated": output.truncated,
        "rule_set": output.rule_set,
        "generation": output.generation,
        "reproducibility": meta.reproducibility,
        "warnings": meta.warnings,
    });
    body.push_str(&format!("event: totals\ndata: {totals}\n\n"));
    let _ = tool;
    (
        StatusCode::OK,
        [
            ("content-type", "text/event-stream"),
            ("cache-control", "no-cache"),
            ("x-content-type-options", "nosniff"),
        ],
        body,
    )
        .into_response()
}

fn search_response(
    headers: &HeaderMap,
    tool: &str,
    output: application::quran_search::SearchOutput,
    started: Instant,
) -> Response {
    let meta = meta_from_search(tool, &output, started);
    if wants_sse(headers) {
        return sse_response(tool, &output, &meta);
    }
    json_response(
        StatusCode::OK,
        &Envelope { api_version: API_VERSION, data: output, meta },
        None,
        true,
    )
}

fn search_service_filters(
    filters: Option<SearchFiltersBody>,
) -> Result<Vec<application::quran_search_api::SearchFilter>, application::quran_search::SearchError>
{
    use application::quran_search_api::build_filters;
    match filters {
        Some(SearchFiltersBody { surah, juz, page, revelation_place, global_range }) => {
            build_filters(
                surah,
                juz.as_deref(),
                page,
                revelation_place.as_deref(),
                global_range.as_deref(),
            )
        }
        None => build_filters(None, None, None, None, None),
    }
}

fn search_service_params(
    text: String,
    paging: SearchPagingBody,
    filters: Vec<application::quran_search_api::SearchFilter>,
) -> Result<application::quran_search::SearchParams, application::quran_search::SearchError> {
    use application::quran_search_api::{parse_match_mode, search_params};
    Ok(search_params(
        text,
        paging.edition,
        parse_match_mode(paging.match_mode.as_deref())?,
        filters,
        paging.limit,
        paging.offset,
        paging.explain,
        paging.highlight,
    ))
}

async fn search_exact_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ExactSearchBody>,
) -> Response {
    use application::quran_search_api::{ExactArgs, parse_exact_field};
    let started = Instant::now();
    if body.text.trim().is_empty() {
        return search_error_response(application::quran_search_api::reject(
            "provide query text".to_string(),
        ));
    }
    let params = match search_service_filters(body.filters)
        .and_then(|filters| search_service_params(body.text, body.paging, filters))
    {
        Ok(params) => params,
        Err(error) => return search_error_response(error),
    };
    let field = match parse_exact_field(body.field.as_deref()) {
        Ok(field) => field,
        Err(error) => return search_error_response(error),
    };
    match state.search.search_exact(ExactArgs { params, field }).await {
        Ok(output) => search_response(&headers, "quran.search_exact", output, started),
        Err(error) => search_error_response(error),
    }
}

async fn search_normalized_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<NormalizedSearchBody>,
) -> Response {
    use application::quran_search_api::{NormalizedArgs, parse_normalized_profile};
    let started = Instant::now();
    if body.text.trim().is_empty() {
        return search_error_response(application::quran_search_api::reject(
            "provide query text".to_string(),
        ));
    }
    let params = match search_service_filters(body.filters)
        .and_then(|filters| search_service_params(body.text, body.paging, filters))
    {
        Ok(params) => params,
        Err(error) => return search_error_response(error),
    };
    let profile = match parse_normalized_profile(body.profile.as_deref(), body.rules.as_deref()) {
        Ok(profile) => profile,
        Err(error) => return search_error_response(error),
    };
    match state.search.search_normalized(NormalizedArgs { params, profile }).await {
        Ok(output) => search_response(&headers, "quran.search_normalized", output, started),
        Err(error) => search_error_response(error),
    }
}

async fn search_phrase_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PhraseSearchBody>,
) -> Response {
    use application::quran_search_api::{PhraseArgs, parse_normalized_profile, parse_phrase_mode};
    let started = Instant::now();
    if body.text.trim().is_empty() {
        return search_error_response(application::quran_search_api::reject(
            "provide query text".to_string(),
        ));
    }
    let params = match search_service_filters(body.filters)
        .and_then(|filters| search_service_params(body.text, body.paging, filters))
    {
        Ok(params) => params,
        Err(error) => return search_error_response(error),
    };
    let profile = match parse_normalized_profile(body.profile.as_deref(), body.rules.as_deref()) {
        Ok(profile) => profile,
        Err(error) => return search_error_response(error),
    };
    let mode = match parse_phrase_mode(body.phrase_mode.as_deref()) {
        Ok(mode) => mode,
        Err(error) => return search_error_response(error),
    };
    match state
        .search
        .search_phrase(PhraseArgs { params, profile, mode, slop: body.slop.unwrap_or(0) })
        .await
    {
        Ok(output) => search_response(&headers, "quran.search_phrase", output, started),
        Err(error) => search_error_response(error),
    }
}

async fn search_concatenated_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ConcatenatedSearchBody>,
) -> Response {
    use application::quran_search_api::ConcatenatedArgs;
    let started = Instant::now();
    if body.text.trim().is_empty() {
        return search_error_response(application::quran_search_api::reject(
            "provide query text".to_string(),
        ));
    }
    let params = match search_service_filters(body.filters)
        .and_then(|filters| search_service_params(body.text, body.paging, filters))
    {
        Ok(params) => params,
        Err(error) => return search_error_response(error),
    };
    match state
        .search
        .search_concatenated(ConcatenatedArgs {
            params,
            allow_cross_ayah: body.allow_cross_ayah.unwrap_or(false),
            max_ayah_span: body.max_ayah_span.unwrap_or(3).max(1),
        })
        .await
    {
        Ok(output) => search_response(&headers, "quran.search_concatenated", output, started),
        Err(error) => search_error_response(error),
    }
}

async fn search_regex_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RegexSearchBody>,
) -> Response {
    use application::quran_search_api::RegexArgs;
    let started = Instant::now();
    if body.pattern.trim().is_empty() {
        return search_error_response(application::quran_search_api::reject(
            "provide a regex pattern".to_string(),
        ));
    }
    let principal = headers
        .get("x-principal")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("api-anonymous")
        .to_string();
    let params = match search_service_filters(body.filters).and_then(|filters| {
        search_service_params(format!("regex:{}", body.pattern), body.paging, filters)
    }) {
        Ok(params) => params,
        Err(error) => return search_error_response(error),
    };
    match state
        .search
        .search_regex(RegexArgs {
            params,
            field: body.field.unwrap_or_else(|| "text_bare".to_string()),
            pattern: body.pattern,
            principal,
            timeout_ms: body.timeout_ms.unwrap_or(3000).clamp(1, 10_000),
        })
        .await
    {
        Ok(output) => search_response(&headers, "quran.search_regex", output, started),
        Err(error) => search_error_response(error),
    }
}

async fn debug_reader_handler(
    State(state): State<AppState>,
    Path((edition, surah)): Path<(String, u16)>,
    font: Option<axum::Extension<DebugFont>>,
) -> Response {
    if quran_core::SurahNumber::new(surah).is_err() {
        return (
            StatusCode::BAD_REQUEST,
            [("content-type", "text/html; charset=utf-8")],
            "<!doctype html><p>bad surah number</p>",
        )
            .into_response();
    }
    let views = match state.api.api_surah(&edition, surah, &AyahOptions::default()).await {
        Ok((_, views)) => views,
        Err(error) => return tool_error_response(error),
    };
    // Debug typography (P1-T54, OD-04): explicit RTL, an Arabic-capable
    // system font stack, and one marked block per ayah. OD-04 decided
    // system-installed fonts only — nothing is bundled or downloaded, so no
    // font/licensing decision is needed for the Phase-1 debug reader. Amiri
    // (Quran variant, SIL OFL 1.1) is the Phase-2 bundling candidate only.
    // `router_with_debug_font` serves a caller-supplied woff2 in memory for
    // tests; wiring it implies no licensing decision. Text is HTML-escaped so
    // a mangled dataset cannot break the page it exposes.
    let font_css = match font {
        Some(_) => concat!(
            "@font-face{font-family:\"Qai Debug Reader\";",
            "src:url(\"/debug/assets/reader.woff2\") format(\"woff2\");",
            "font-display:swap}",
            "body{font-family:\"Qai Debug Reader\",\"KFGQPC Uthmanic Script HAFS\",",
            "\"Amiri Quran\",\"Scheherazade New\",\"Noto Naskh Arabic\",",
            "\"Traditional Arabic\",serif;",
        ),
        None => concat!(
            "body{font-family:\"KFGQPC Uthmanic Script HAFS\",\"Amiri Quran\",",
            "\"Scheherazade New\",\"Noto Naskh Arabic\",\"Traditional Arabic\",serif;",
        ),
    };
    let mut body = String::from(
        "<!doctype html><html lang=\"ar\" dir=\"rtl\"><head><meta charset=\"utf-8\">\
         <title>qai debug reader (not the product UI)</title>\
         <style>",
    );
    body.push_str(font_css);
    body.push_str(
        "line-height:2}\
         .ayah{margin:0.6em 0}.marker{display:inline-block;min-width:3em;\
         font-family:sans-serif;font-size:0.8em;opacity:0.75}\
         .ref{font-family:sans-serif;font-size:0.75em;opacity:0.6}</style></head>\
         <body><p><strong>debug view</strong> — engineering preview, no persistence</p>",
    );
    for view in &views {
        let (start, _) = view.canonical.ayah_range();
        body.push_str(&format!(
            "<article class=\"ayah\"><span class=\"marker\">{}:{}</span> \
             <span class=\"text\">{}</span><br>\
             <span class=\"ref\">{}</span></article>",
            view.canonical.surah_number().get(),
            start.get(),
            escape_html(view.canonical.arabic_text()),
            escape_html(view.canonical.reference())
        ));
    }
    body.push_str("</body></html>");
    (StatusCode::OK, [("content-type", "text/html; charset=utf-8")], body).into_response()
}

/// Minimal HTML escaping for debug rendering: the reader exists to expose
/// mangled datasets, so its own markup must survive hostile text.
fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

#[derive(Clone)]
pub struct DebugFont(Arc<[u8]>);

/// Build the debug-reader router with a locally served font asset. The font
/// bytes stay in memory; nothing is bundled into releases or downloaded at
/// runtime, so wiring this in tests implies no font/licensing decision
/// (OD-04: Phase-1 ships the system stack only).
pub fn router_with_debug_font(state: AppState, woff2: Vec<u8>) -> Router {
    router(state)
        .route("/debug/assets/reader.woff2", get(debug_font_handler))
        .layer(axum::Extension(DebugFont(woff2.into())))
}

async fn debug_font_handler(axum::Extension(font): axum::Extension<DebugFont>) -> Response {
    (
        [
            ("content-type", "font/woff2"),
            ("cache-control", "no-store"),
            ("x-content-type-options", "nosniff"),
        ],
        font.0.to_vec(),
    )
        .into_response()
}

/// Build the router (health + API v1 + debug reader).
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(|| async { "ready" }))
        .route(
            "/api/v1/meta",
            get(|| async {
                json_response(
                    StatusCode::OK,
                    &Envelope {
                        api_version: API_VERSION,
                        data: serde_json::json!({
                            "name": "Q-ai",
                            "version": env!("CARGO_PKG_VERSION"),
                        }),
                        meta: empty_meta(),
                    },
                    None,
                    false,
                )
            }),
        )
        .route("/api/v1/quran/editions", get(editions_handler))
        .route("/api/v1/quran/editions/{slug}", get(edition_handler))
        .route("/api/v1/quran/surahs", get(surahs_handler))
        .route("/api/v1/quran/surahs/{number}", get(surah_handler))
        .route("/api/v1/quran/ayahs/{reference}", get(ayahs_handler))
        .route("/api/v1/quran/context/{reference}", get(context_handler))
        .route("/api/v1/quran/divisions/{kind}/{number}", get(divisions_handler))
        .route("/api/v1/quran/tokens/{reference}", get(tokens_handler))
        .route("/api/v1/quran/resolve", get(resolve_handler))
        .route("/api/v1/quran/citations/{id}", get(citation_handler))
        .route("/api/v1/quran/normalization/preview", post(normalization_preview_handler))
        .route("/api/v1/quran/normalization/profiles", get(normalization_profiles_handler))
        .route("/api/v1/quran/search/exact", post(search_exact_handler))
        .route("/api/v1/quran/search/normalized", post(search_normalized_handler))
        .route("/api/v1/quran/search/phrase", post(search_phrase_handler))
        .route("/api/v1/quran/search/concatenated", post(search_concatenated_handler))
        .route("/api/v1/quran/search/regex", post(search_regex_handler))
        .route("/debug/read/{edition}/{surah}", get(debug_reader_handler))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::limit::RequestBodyLimitLayer::new(1024 * 1024))
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            std::time::Duration::from_secs(30),
        ))
        .layer(tower::limit::ConcurrencyLimitLayer::new(128))
        .with_state(state)
}

/// Serve until the process is killed.
pub async fn serve(addr: &str, state: AppState) -> Result<(), crate::ServerError> {
    let socket = crate::loopback_addr(addr)?;
    let listener = tokio::net::TcpListener::bind(socket)
        .await
        .map_err(|source| crate::ServerError::Bind { addr: addr.to_string(), source })?;
    axum::serve(listener, router(state)).await.map_err(crate::ServerError::from)
}

/// Application-backed implementation of [`QuranApiBackend`].
pub struct ReaderBackend {
    reader: std::sync::Arc<application::quran_reader::QuranReaderService>,
}

impl ReaderBackend {
    /// Wrap a reader.
    pub fn new(reader: std::sync::Arc<application::quran_reader::QuranReaderService>) -> Self {
        Self { reader }
    }
}

fn edition_selector(edition: &str) -> Result<quran_core::EditionSelector, ToolError> {
    if edition.is_empty() {
        return Ok(quran_core::EditionSelector::Active);
    }
    match edition.split_once('@') {
        Some((slug, version)) => match version.parse() {
            Ok(version) => {
                Ok(quran_core::EditionSelector::Pinned { slug: slug.to_string(), version })
            }
            Err(_) => Err(ToolError::InvalidInput {
                tool: "quran.api",
                detail: format!("bad edition `{edition}`"),
            }),
        },
        None => Ok(quran_core::EditionSelector::Slug(edition.to_string())),
    }
}

fn edition_json(edition: &quran_core::QuranEdition) -> serde_json::Value {
    serde_json::json!({
        "slug": edition.slug,
        "version": edition.version.to_string(),
        "name": edition.name,
        "script": edition.script,
        "riwayah": edition.riwayah,
        "language": edition.language.to_string(),
        "verse_numbering_scheme": edition.verse_numbering_scheme,
        "status": edition.status,
        "text_hash": edition.text_hash.hex,
        "statistics": edition.statistics,
    })
}

#[async_trait::async_trait]
impl QuranApiBackend for ReaderBackend {
    async fn api_editions(&self) -> Result<Vec<serde_json::Value>, ToolError> {
        use application::quran_reader::{EditionFilter, QuranReader};
        let editions = self
            .reader
            .list_editions(EditionFilter::default())
            .await
            .map_err(application::quran_tools::to_tool_error)?;
        Ok(editions.iter().map(edition_json).collect())
    }

    async fn api_edition(&self, slug: &str) -> Result<serde_json::Value, ToolError> {
        use application::quran_reader::QuranReader;
        let selector = quran_core::EditionSelector::Slug(slug.to_string());
        let edition = self.reader.get_edition(&selector).await.map_err(|err| match err {
            application::quran_reader::ReaderError::EditionNotFound(_) => {
                ToolError::Backend { code: "QAI-QUR-0306".into(), detail: err.to_string() }
            }
            other => application::quran_tools::to_tool_error(other),
        })?;
        if edition.slug != slug {
            return Err(ToolError::Backend {
                code: "QAI-QUR-0306".into(),
                detail: format!("edition `{slug}` is not active"),
            });
        }
        Ok(edition_json(&edition))
    }

    async fn api_surahs(&self, edition: &str) -> Result<Vec<quran_core::Surah>, ToolError> {
        use application::quran_reader::QuranReader;
        let selector = edition_selector(edition)?;
        self.reader.list_surahs(&selector).await.map_err(application::quran_tools::to_tool_error)
    }

    async fn api_surah(
        &self,
        edition: &str,
        surah: u16,
        options: &quran_core::AyahOptions,
    ) -> Result<(quran_core::Surah, Vec<quran_core::AyahView>), ToolError> {
        use application::quran_reader::QuranReader;
        let selector = edition_selector(edition)?;
        let number = quran_core::SurahNumber::new(surah).map_err(|_| ToolError::InvalidInput {
            tool: "quran.api",
            detail: format!("bad surah number {surah}"),
        })?;
        let reference = quran_core::QuranRef::Surah { edition: selector, surah: number };
        let views = self
            .reader
            .get_ayahs(&reference, options)
            .await
            .map_err(application::quran_tools::to_tool_error)?;
        let surahs = self
            .reader
            .list_surahs(reference.edition())
            .await
            .map_err(application::quran_tools::to_tool_error)?;
        let meta = surahs.into_iter().find(|row| row.number == number).ok_or_else(|| {
            ToolError::Backend {
                code: "QAI-QUR-0307".into(),
                detail: format!("surah {surah} not found"),
            }
        })?;
        Ok((meta, views))
    }

    async fn api_division(
        &self,
        edition: &str,
        kind: &str,
        number: u32,
        options: &quran_core::AyahOptions,
    ) -> Result<Vec<quran_core::AyahView>, ToolError> {
        use application::quran_reader::QuranReader;
        let kind = match kind {
            "juz" => quran_core::DivisionKind::Juz,
            "hizb" => quran_core::DivisionKind::Hizb,
            "rub" => quran_core::DivisionKind::Rub,
            "manzil" => quran_core::DivisionKind::Manzil,
            "page" => quran_core::DivisionKind::Page,
            "ruku" => quran_core::DivisionKind::Ruku,
            "sajdah" => quran_core::DivisionKind::Sajdah,
            _ => {
                return Err(ToolError::InvalidInput {
                    tool: "quran.api",
                    detail: format!("unknown division `{kind}`"),
                });
            }
        };
        let selector = edition_selector(edition)?;
        let reference = quran_core::QuranRef::Division { edition: selector, kind, number };
        self.reader
            .get_ayahs(&reference, options)
            .await
            .map_err(application::quran_tools::to_tool_error)
    }

    async fn api_tokens(&self, reference: &str) -> Result<Vec<quran_core::Token>, ToolError> {
        use application::quran_reader::QuranReader;
        let parsed = quran_core::parse(reference).map_err(|err| ToolError::InvalidInput {
            tool: "quran.api",
            detail: err.to_string(),
        })?;
        self.reader.get_tokens(&parsed).await.map_err(application::quran_tools::to_tool_error)
    }

    async fn api_citation(&self, id: &str) -> Result<citations::ResolvedCitation, ToolError> {
        use storage::Database as _;
        let mut uow = self.reader.database().write().await.map_err(|err| ToolError::Backend {
            code: "QAI-QUR-0310".into(),
            detail: err.to_string(),
        })?;
        let row = uow
            .quran()
            .get_citation(id)
            .await
            .map_err(|err| ToolError::Backend {
                code: "QAI-QUR-0310".into(),
                detail: err.to_string(),
            })?
            .ok_or_else(|| ToolError::Backend {
                code: "QAI-QUR-0322".into(),
                detail: format!("citation `{id}` not found"),
            })?;
        uow.rollback().await.map_err(|err| ToolError::Backend {
            code: "QAI-QUR-0310".into(),
            detail: err.to_string(),
        })?;
        let location: serde_json::Value =
            serde_json::from_str(&row.location_json).map_err(|_| ToolError::Backend {
                code: "QAI-QUR-0310".into(),
                detail: "stored citation location is corrupt".to_string(),
            })?;
        let stored = citations::StoredCitation {
            id: row.id.clone(),
            canonical_reference: row.canonical_reference.clone(),
            quoted_text_hash: row.quoted_text_hash.unwrap_or_default(),
            edition_slug: row
                .edition_ref
                .as_deref()
                .and_then(|reference| reference.split_once('@').map(|(slug, _)| slug.to_string()))
                .unwrap_or_default(),
            edition_version: row
                .edition_ref
                .as_deref()
                .and_then(|reference| {
                    reference.split_once('@').map(|(_, version)| version.to_string())
                })
                .unwrap_or_default(),
            surah: location.get("surah").and_then(|value| value.as_u64()).unwrap_or(0) as u16,
            ayah: location.get("ayah").and_then(|value| value.as_u64()).unwrap_or(0) as u32,
        };
        let resolver = application::quran_tools::ReaderCitationSource::resolver(
            std::sync::Arc::clone(&self.reader),
        );
        resolver.resolve_stored(&stored).await.map_err(|err| ToolError::Backend {
            code: err.code().to_string(),
            detail: err.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_meta_serializes_with_required_keys() {
        let meta = empty_meta();
        let value = serde_json::to_value(&meta).unwrap();
        for key in [
            "edition",
            "corpus_generation",
            "canonical_reference",
            "deep_link",
            "execution_time_ms",
            "reproducibility",
            "warnings",
        ] {
            assert!(value.get(key).is_some(), "missing meta key {key}");
        }
    }

    #[test]
    fn debug_escape_html_neutralizes_markup() {
        assert_eq!(escape_html("بِسْمِ"), "بِسْمِ");
        assert_eq!(escape_html("<b>&\"x\"</b>"), "&lt;b&gt;&amp;&quot;x&quot;&lt;/b&gt;");
    }

    #[test]
    fn tool_status_mapping() {
        assert_eq!(
            tool_status(&ToolError::InvalidInput { tool: "t", detail: "d".into() }),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            tool_status(&ToolError::Backend { code: "QAI-QUR-0307".into(), detail: "d".into() }),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            tool_status(&ToolError::Backend { code: "QAI-QUR-0310".into(), detail: "d".into() }),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
