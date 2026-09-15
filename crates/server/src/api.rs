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
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
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
    if let (Some(sent), Some(current)) = (headers.get("if-none-match"), etag_for(&meta)) {
        if sent == current {
            return StatusCode::NOT_MODIFIED.into_response();
        }
    }
    let etag = etag_for(&meta);
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
    let mut meta = empty_meta();
    meta.execution_time_ms = started.elapsed().as_secs_f64() * 1000.0;
    json_response(
        StatusCode::OK,
        &Envelope { api_version: API_VERSION, data: resolved, meta },
        None,
        false,
    )
}

async fn debug_reader_handler(
    State(state): State<AppState>,
    Path((edition, surah)): Path<(String, u16)>,
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
    let mut body = String::from(
        "<!doctype html><html lang=\"ar\" dir=\"rtl\"><head><meta charset=\"utf-8\">\
         <title>qai debug reader (not the product UI)</title></head>\
         <body><p><strong>debug view</strong> — engineering preview, no persistence</p>",
    );
    for view in &views {
        body.push_str(&format!(
            "<p>{} <span>{}</span></p>",
            view.canonical.reference(),
            view.canonical.arabic_text()
        ));
    }
    body.push_str("</body></html>");
    (StatusCode::OK, [("content-type", "text/html; charset=utf-8")], body).into_response()
}

/// Build the router (health + API v1 + debug reader).
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(|| async { "ready" }))
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
    if !crate::is_loopback(addr) {
        eprintln!(
            "warning: server binding to non-loopback address `{addr}`; \
             authentication enforcement is deferred until Phase 11"
        );
    }
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|source| crate::ServerError::Bind { addr: addr.to_string(), source })?;
    axum::serve(listener, router(state)).await.map_err(crate::ServerError::from)
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
