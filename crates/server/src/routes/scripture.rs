//! HTTP routes for comparative scripture (Phase 9).

use axum::extract::{Json, Path, Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use serde::Deserialize;
use serde_json::Value;

use crate::api::{AppState, Envelope, empty_meta, json_response, tool_error_response, API_VERSION};
use tool_registry::{GetScriptureParams, ScriptureSearchToolParams};
use application::knowledge_domain::{CrossDomainHit, KnowledgeDomain};

#[derive(Deserialize, Default)]
struct SearchBody {
    query: String,
    #[serde(default)]
    edition: Option<String>,
    #[serde(default)]
    limit: Option<u32>,
}

#[derive(Deserialize, Default)]
struct ParallelBody {
    #[serde(default)]
    reference: String,
}

#[derive(Deserialize, Default)]
struct ResearchBody {
    query: String,
    #[serde(default)]
    domains: Option<String>,
}

#[derive(Deserialize, Default)]
struct ResearchQuery {
    #[serde(default)]
    query: String,
    #[serde(default)]
    domains: Option<String>,
}

#[derive(Deserialize, Default)]
struct TranslationsQuery {
    #[serde(default)]
    language: Option<String>,
}

#[derive(Deserialize, Default)]
struct ParallelQuery {
    #[serde(default)]
    reference: String,
}

#[derive(Deserialize, Default)]
struct ScriptureSearchQuery {
    #[serde(default)]
    query: String,
    #[serde(default)]
    edition: Option<String>,
    #[serde(default)]
    limit: Option<u32>,
}

fn parse_domains(s: Option<&str>) -> std::collections::BTreeSet<KnowledgeDomain> {
    let mut out = std::collections::BTreeSet::new();
    if let Some(s) = s {
        for part in s.split(',') {
            match part.trim().to_lowercase().as_str() {
                "quran" => { out.insert(KnowledgeDomain::Quran); }
                "hadith" => { out.insert(KnowledgeDomain::Hadith); }
                "tafsir" => { out.insert(KnowledgeDomain::Tafsir); }
                "scripture" => { out.insert(KnowledgeDomain::Scripture); }
                "books" => { out.insert(KnowledgeDomain::Books); }
                _ => {}
            }
        }
    }
    if out.is_empty() {
        KnowledgeDomain::all()
    } else {
        out
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/scripture/editions", get(editions_handler))
        .route("/api/v1/scripture/search", get(search_handler_get).post(search_handler_post))
        .route("/api/v1/scripture/{edition}/{book}/{chapter}/{verse}", get(get_handler))
        .route("/api/v1/scripture/translations/{verse_id}", get(translations_handler))
        .route("/api/v1/scripture/parallel", get(parallel_handler_get).post(parallel_handler_post))
        .route("/api/v1/scripture/research", get(research_handler_get).post(research_handler_post))
        .route("/api/v1/cross-domain/research", get(research_handler_get).post(research_handler_post))
        .route("/api/v1/research/cross-domain", get(research_handler_get).post(research_handler_post))
}

async fn editions_handler(State(state): State<AppState>) -> Response {
    match state.tools.list_scripture_editions().await {
        Ok(result) => {
            let checksum = result.research_checksum.hex.clone();
            let mut meta = empty_meta();
            meta.research_checksum = format!("sha256:{}", checksum);
            let data = serde_json::to_value(&result).unwrap_or(Value::Null);
            json_response(StatusCode::OK, &Envelope { api_version: API_VERSION, data, meta }, None, false)
        }
        Err(e) => tool_error_response(e),
    }
}

async fn get_handler(
    State(state): State<AppState>,
    Path((edition, book, chapter, verse)): Path<(String, String, u32, u32)>,
) -> Response {
    let reference = format!("{}:{}:{}:{}", edition, book, chapter, verse);
    let params = GetScriptureParams { reference };
    match state.tools.get_scripture(params).await {
        Ok((result, _meta)) => {
            let checksum = result.research_checksum.hex.clone();
            let mut meta = empty_meta();
            meta.research_checksum = format!("sha256:{}", checksum);
            let data = serde_json::to_value(&result).unwrap_or(Value::Null);
            json_response(StatusCode::OK, &Envelope { api_version: API_VERSION, data, meta }, None, false)
        }
        Err(e) => tool_error_response(e),
    }
}

async fn search_handler_post(State(state): State<AppState>, Json(body): Json<SearchBody>) -> Response {
    let params = ScriptureSearchToolParams {
        query: body.query,
        edition: body.edition,
        limit: body.limit,
    };
    match state.tools.search_scripture(params).await {
        Ok(result) => {
            let checksum = result.research_checksum.hex.clone();
            let mut meta = empty_meta();
            meta.research_checksum = format!("sha256:{}", checksum);
            let data = serde_json::to_value(&result).unwrap_or(Value::Null);
            json_response(StatusCode::OK, &Envelope { api_version: API_VERSION, data, meta }, None, false)
        }
        Err(e) => tool_error_response(e),
    }
}

async fn search_handler_get(State(state): State<AppState>, Query(q): Query<ScriptureSearchQuery>) -> Response {
    let params = ScriptureSearchToolParams {
        query: q.query,
        edition: q.edition,
        limit: q.limit,
    };
    match state.tools.search_scripture(params).await {
        Ok(result) => {
            let checksum = result.research_checksum.hex.clone();
            let mut meta = empty_meta();
            meta.research_checksum = format!("sha256:{}", checksum);
            let data = serde_json::to_value(&result).unwrap_or(Value::Null);
            json_response(StatusCode::OK, &Envelope { api_version: API_VERSION, data, meta }, None, false)
        }
        Err(e) => tool_error_response(e),
    }
}

async fn translations_handler(
    State(state): State<AppState>,
    Path(verse_id): Path<String>,
    Query(q): Query<TranslationsQuery>,
) -> Response {
    match state.tools.get_scripture_translations(verse_id, q.language).await {
        Ok(result) => {
            let checksum = result.research_checksum.hex.clone();
            let mut meta = empty_meta();
            meta.research_checksum = format!("sha256:{}", checksum);
            let data = serde_json::to_value(&result).unwrap_or(Value::Null);
            json_response(StatusCode::OK, &Envelope { api_version: API_VERSION, data, meta }, None, false)
        }
        Err(e) => tool_error_response(e),
    }
}

async fn parallel_handler_post(State(state): State<AppState>, Json(body): Json<ParallelBody>) -> Response {
    parallel_impl(state, body.reference).await
}

async fn parallel_handler_get(State(state): State<AppState>, Query(q): Query<ParallelQuery>) -> Response {
    parallel_impl(state, q.reference).await
}

async fn parallel_impl(state: AppState, reference: String) -> Response {
    match state.tools.parallel_passages_scripture(reference).await {
        Ok(result) => {
            let checksum = result.research_checksum.hex.clone();
            let mut meta = empty_meta();
            meta.research_checksum = format!("sha256:{}", checksum);
            let data = serde_json::to_value(&result).unwrap_or(Value::Null);
            json_response(StatusCode::OK, &Envelope { api_version: API_VERSION, data, meta }, None, false)
        }
        Err(e) => tool_error_response(e),
    }
}

async fn research_handler_post(State(state): State<AppState>, Json(body): Json<ResearchBody>) -> Response {
    research_impl(state, body.query, body.domains).await
}

async fn research_handler_get(State(state): State<AppState>, Query(q): Query<ResearchQuery>) -> Response {
    research_impl(state, q.query, q.domains).await
}

async fn research_impl(state: AppState, query: String, domains: Option<String>) -> Response {
    let domain_set = parse_domains(domains.as_deref());
    let mut hits: Vec<CrossDomainHit> = Vec::new();

    // scripture
    if domain_set.contains(&KnowledgeDomain::Scripture) {
        let params = ScriptureSearchToolParams { query: query.clone(), edition: None, limit: Some(20) };
        if let Ok(result) = state.tools.search_scripture(params).await {
            if let Some(arr) = result.results.as_array() {
                for item in arr {
                    hits.push(CrossDomainHit {
                        domain: KnowledgeDomain::Scripture,
                        source: item.get("edition").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        reference: item.get("verse_id").map(|v| v.to_string()).unwrap_or_default(),
                        snippet: item.get("snippet").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        score: item.get("score").and_then(|v| v.as_f64()).unwrap_or(1.0),
                        confidence: 1.0,
                    });
                }
            }
        }
    }
    // hadith
    if domain_set.contains(&KnowledgeDomain::Hadith) {
        let params = tool_registry::HadithSearchToolParams { query: query.clone(), collection_slug: None, limit: Some(20) };
        if let Ok(result) = state.tools.search_hadith(params).await {
            if let Some(arr) = result.results.as_array() {
                for item in arr {
                    hits.push(CrossDomainHit {
                        domain: KnowledgeDomain::Hadith,
                        source: item.get("collection_slug").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        reference: item.get("canonical_reference").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        snippet: item.get("snippet").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        score: 1.0,
                        confidence: 1.0,
                    });
                }
            }
        }
    }
    // tafsir
    if domain_set.contains(&KnowledgeDomain::Tafsir) {
        let params = tool_registry::TafsirSearchToolParams { query: query.clone(), work_slug: None, limit: Some(20) };
        if let Ok(result) = state.tools.search_tafsir(params).await {
            if let Some(arr) = result.results.as_array() {
                for item in arr {
                    hits.push(CrossDomainHit {
                        domain: KnowledgeDomain::Tafsir,
                        source: item.get("work_slug").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        reference: item.get("canonical_reference").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        snippet: item.get("snippet").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        score: 1.0,
                        confidence: 1.0,
                    });
                }
            }
        }
    }

    // sort
    hits.sort_by(|a, b| {
        b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.domain.cmp(&b.domain))
            .then_with(|| a.source.cmp(&b.source))
    });

    let mut meta = empty_meta();
    meta.research_checksum = format!("cross-domain:{}", query);
    let data = serde_json::to_value(&hits).unwrap_or(Value::Null);
    json_response(StatusCode::OK, &Envelope { api_version: API_VERSION, data, meta }, None, false)
}
