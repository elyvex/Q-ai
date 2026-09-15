//! Phase 1 — API v1 contract acceptance: AC-P1-14/18 (P1-T39/T40, T54).
//!
//! Fakes back the whole surface (no database): envelope shape, meta keys,
//! ETag/304 behavior, content-language, error bodies, and the debug reader.

use std::sync::Arc;

use axum::http::{HeaderMap, StatusCode};
use server::api::{AppState, QuranApiBackend, router};
use tool_registry::{BackendMeta, QuranBackend, ToolRegistry};
use tools::ToolError;

fn test_view() -> quran_core::AyahView {
    serde_json::from_value::<quran_core::AyahView>(serde_json::json!({
        "canonical": {
            "reference": "quran:test@0.1.0:1:1",
            "surah_number": 1,
            "surah_name_arabic": "ت",
            "surah_name_translit": null,
            "ayah_range": [1, 1],
            "arabic_text": "ب",
            "text_hash": {"algorithm": "Sha256", "hex": "ab".repeat(32)},
            "edition": {"slug": "test", "version": "0.1.0",
                        "script": "uthmani", "riwayah": null},
            "translation": null,
            "deep_link": "/read/test@0.1.0/1:1",
            "page": null,
            "juz": null
        },
        "translations": [],
        "word_glosses": null,
        "tokens": null
    }))
    .unwrap()
}

fn test_meta() -> BackendMeta {
    BackendMeta {
        edition_slug: "test".into(),
        edition_version: "0.1.0".into(),
        edition_id: "ed-1".into(),
        corpus_generation: 7,
        text_hash: "sha256:ab".into(),
        script: "uthmani".into(),
        riwayah: None,
        numbering_scheme: "hafs".into(),
    }
}

struct FakeBackend;

#[async_trait::async_trait]
impl QuranBackend for FakeBackend {
    async fn backend_get_ayah(
        &self,
        reference: &quran_core::QuranRef,
        _options: &quran_core::AyahOptions,
    ) -> Result<(quran_core::AyahView, BackendMeta), ToolError> {
        match reference {
            quran_core::QuranRef::Ayah { surah, .. } if surah.get() == 99 => {
                Err(ToolError::Backend {
                    code: "QAI-QUR-0307".into(),
                    detail: "ayah not found: 99:1".into(),
                })
            }
            _ => Ok((test_view(), test_meta())),
        }
    }

    async fn backend_get_context(
        &self,
        _reference: &quran_core::QuranRef,
        _spec: &quran_core::ContextSpec,
    ) -> Result<(quran_core::ContextView, BackendMeta), ToolError> {
        let view = test_view();
        Ok((
            quran_core::ContextView {
                canonical_reference: "quran:test@0.1.0:1:1".into(),
                focal: view,
                before: Vec::new(),
                after: Vec::new(),
                surah: None,
                global_range: (1, 1),
            },
            test_meta(),
        ))
    }
}

struct FakeApi;

#[async_trait::async_trait]
impl QuranApiBackend for FakeApi {
    async fn api_editions(&self) -> Result<Vec<serde_json::Value>, ToolError> {
        Ok(vec![serde_json::json!({"slug": "test", "version": "0.1.0"})])
    }

    async fn api_edition(&self, slug: &str) -> Result<serde_json::Value, ToolError> {
        if slug == "test" {
            Ok(serde_json::json!({"slug": "test", "version": "0.1.0"}))
        } else {
            Err(ToolError::Backend {
                code: "QAI-QUR-0306".into(),
                detail: "edition not found".into(),
            })
        }
    }

    async fn api_surahs(&self, _edition: &str) -> Result<Vec<quran_core::Surah>, ToolError> {
        Ok(Vec::new())
    }

    async fn api_surah(
        &self,
        _edition: &str,
        _surah: u16,
        _options: &quran_core::AyahOptions,
    ) -> Result<(quran_core::Surah, Vec<quran_core::AyahView>), ToolError> {
        let surah = serde_json::from_value::<quran_core::Surah>(serde_json::json!({
            "edition_id": "11111111-2222-4333-8444-555555555555",
            "number": 1,
            "name_arabic": "ت",
            "name_transliteration": null,
            "name_translations": {},
            "ayah_count": 1,
            "revelation_place": null,
            "revelation_order": null,
            "basmala": "absent",
            "ruku_count": null,
            "metadata_provenance": "11111111-2222-4333-8444-555555555555"
        }))
        .unwrap();
        Ok((surah, vec![test_view()]))
    }

    async fn api_division(
        &self,
        _edition: &str,
        kind: &str,
        _number: u32,
        _options: &quran_core::AyahOptions,
    ) -> Result<Vec<quran_core::AyahView>, ToolError> {
        if kind == "juz" {
            Ok(vec![test_view()])
        } else {
            Err(ToolError::Backend {
                code: "QAI-QUR-0309".into(),
                detail: "division not found".into(),
            })
        }
    }

    async fn api_tokens(&self, _reference: &str) -> Result<Vec<quran_core::Token>, ToolError> {
        Ok(Vec::new())
    }

    async fn api_citation(&self, id: &str) -> Result<citations::ResolvedCitation, ToolError> {
        if id == "cit-1" {
            Ok(citations::ResolvedCitation {
                citation_id: id.into(),
                verdict: citations::QuotationVerdict::ExactMatch,
                text_hash: Some("sha256:ab".into()),
                deep_link: Some("/read/test@0.1.0/1:1".into()),
            })
        } else {
            Err(ToolError::Backend {
                code: "QAI-QUR-0322".into(),
                detail: "citation not found".into(),
            })
        }
    }
}

fn test_state() -> AppState {
    AppState { tools: Arc::new(ToolRegistry::new(Arc::new(FakeBackend))), api: Arc::new(FakeApi) }
}

async fn serve_once() -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let handle = tokio::spawn(async move {
        axum::serve(listener, router(test_state())).await.unwrap();
    });
    (addr, handle)
}

async fn get(
    addr: &str,
    path: &str,
    extra_headers: &[(&str, &str)],
) -> (StatusCode, HeaderMap, Vec<u8>) {
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut request = format!("GET {path} HTTP/1.1\r\nhost: x\r\nconnection: close\r\n");
    for (name, value) in extra_headers {
        request.push_str(&format!("{name}: {value}\r\n"));
    }
    request.push_str("\r\n");
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).await.unwrap();
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    let mut lines = head.lines();
    let status_line = lines.next().unwrap_or("");
    let status: u16 = status_line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
    let mut headers = HeaderMap::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':')
            && let (Ok(name), Ok(value)) = (
                name.trim().parse::<axum::http::HeaderName>(),
                value.trim().parse::<axum::http::HeaderValue>(),
            )
        {
            headers.insert(name, value);
        }
    }
    (StatusCode::from_u16(status).unwrap(), headers, body.as_bytes().to_vec())
}

fn body_json(body: &[u8]) -> serde_json::Value {
    serde_json::from_slice(body).expect("response is JSON")
}

const OPENAPI_SPEC: &str = include_str!("../../../docs/08-api/quran-v1-openapi.json");

#[tokio::test]
async fn openapi_spec_covers_every_route() {
    let spec: serde_json::Value = serde_json::from_str(OPENAPI_SPEC).unwrap();
    let paths = spec["paths"].as_object().expect("paths object");
    // Every served route family appears in the spec (T40 contract depth).
    for path in [
        "/healthz",
        "/readyz",
        "/api/v1/quran/editions",
        "/api/v1/quran/editions/{slug}",
        "/api/v1/quran/surahs",
        "/api/v1/quran/surahs/{number}",
        "/api/v1/quran/ayahs/{reference}",
        "/api/v1/quran/context/{reference}",
        "/api/v1/quran/divisions/{kind}/{number}",
        "/api/v1/quran/tokens/{reference}",
        "/api/v1/quran/resolve",
        "/api/v1/quran/citations/{id}",
        "/api/v1/quran/normalization/preview",
        "/api/v1/quran/normalization/profiles",
        "/debug/read/{edition}/{surah}",
    ] {
        assert!(paths.contains_key(path), "spec missing {path}");
    }
}

#[tokio::test]
async fn openapi_spec_schemas_resolve_and_cover_json_responses() {
    let spec: serde_json::Value = serde_json::from_str(OPENAPI_SPEC).unwrap();
    let schemas = spec["components"]["schemas"].as_object().expect("components.schemas");
    for name in ["EditionMeta", "Meta", "Envelope", "Diagnostic"] {
        assert!(schemas.contains_key(name), "missing schema {name}");
    }
    // The contract keys asserted behaviorally in `assert_envelope` are required
    // by the machine-readable schemas too (T40 spec depth).
    let meta_required = schemas["Meta"]["required"].as_array().unwrap();
    for key in [
        "edition",
        "corpus_generation",
        "canonical_reference",
        "deep_link",
        "execution_time_ms",
        "reproducibility",
        "warnings",
    ] {
        assert!(meta_required.iter().any(|v| v == key), "Meta missing required {key}");
    }
    let edition_required = schemas["EditionMeta"]["required"].as_array().unwrap();
    for key in ["slug", "version", "text_hash", "script", "riwayah", "numbering_scheme"] {
        assert!(edition_required.iter().any(|v| v == key), "EditionMeta missing required {key}");
    }

    // Every local `$ref` resolves inside the document.
    fn resolve<'a>(spec: &'a serde_json::Value, pointer: &str) -> &'a serde_json::Value {
        assert!(pointer.starts_with("#/"), "only local refs supported: {pointer}");
        let mut node = spec;
        for part in pointer[2..].split('/') {
            node = &node[part];
        }
        assert!(!node.is_null(), "unresolvable $ref {pointer}");
        node
    }
    fn collect_refs(value: &serde_json::Value, out: &mut Vec<String>) {
        match value {
            serde_json::Value::Object(map) => {
                if let Some(reference) = map.get("$ref").and_then(|r| r.as_str()) {
                    out.push(reference.to_string());
                }
                for nested in map.values() {
                    collect_refs(nested, out);
                }
            }
            serde_json::Value::Array(items) => {
                for nested in items {
                    collect_refs(nested, out);
                }
            }
            _ => {}
        }
    }
    let mut refs = Vec::new();
    collect_refs(&spec, &mut refs);
    assert!(!refs.is_empty(), "spec has no $refs");
    for reference in &refs {
        resolve(&spec, reference);
    }

    // Every application/json response carries a schema.
    for (path, ops) in spec["paths"].as_object().unwrap() {
        for (method, op) in ops.as_object().unwrap() {
            if method == "parameters" {
                continue;
            }
            for (code, response) in op["responses"].as_object().unwrap() {
                let Some(content) = response.get("content") else {
                    continue;
                };
                let Some(json) = content.get("application/json") else {
                    continue;
                };
                assert!(json.get("schema").is_some(), "{method} {path} {code} has no schema");
            }
        }
    }
}

fn assert_envelope(value: &serde_json::Value) {
    assert_eq!(value["api_version"], "v1");
    assert!(value.get("data").is_some());
    let meta = &value["meta"];
    for key in [
        "edition",
        "corpus_generation",
        "canonical_reference",
        "deep_link",
        "execution_time_ms",
        "reproducibility",
        "warnings",
    ] {
        assert!(meta.get(key).is_some(), "missing meta key {key}");
    }
    for key in ["slug", "version", "text_hash", "script", "riwayah", "numbering_scheme"] {
        assert!(meta["edition"].get(key).is_some(), "missing edition key {key}");
    }
}

#[tokio::test]
async fn health_endpoints_keep_phase0_shapes() {
    let (addr, handle) = serve_once().await;
    let (status, _, body) = get(&addr, "/healthz", &[]).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, b"ok");
    let (status, _, body) = get(&addr, "/readyz", &[]).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, b"ready");
    handle.abort();
}

#[tokio::test]
async fn ayah_envelope_carries_meta_etag_and_language() {
    let (addr, handle) = serve_once().await;
    let (status, headers, body) = get(&addr, "/api/v1/quran/ayahs/1:1", &[]).await;
    assert_eq!(status, StatusCode::OK);
    let value = body_json(&body);
    assert_envelope(&value);
    assert_eq!(value["meta"]["corpus_generation"], 7);
    assert_eq!(value["meta"]["canonical_reference"], "quran:test@0.1.0:1:1");
    assert!(value["meta"]["reproducibility"].get("checksum").is_some());
    let etag = headers.get("etag").expect("ETag").to_str().unwrap().to_string();
    assert!(etag.contains("sha256:ab"), "ETag derives from text hash: {etag}");
    assert_eq!(headers.get("content-language").unwrap(), "ar");
    assert!(headers.get("content-type").unwrap().to_str().unwrap().contains("application/json"));

    // Conditional request short-circuits.
    let (status, _, body) =
        get(&addr, "/api/v1/quran/ayahs/1:1", &[("if-none-match", &etag)]).await;
    assert_eq!(status, StatusCode::NOT_MODIFIED);
    assert!(body.is_empty());
    handle.abort();
}

#[tokio::test]
async fn errors_use_the_diagnostic_body() {
    let (addr, handle) = serve_once().await;
    let (status, _, body) = get(&addr, "/api/v1/quran/ayahs/99:1", &[]).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let value = body_json(&body);
    assert_eq!(value["error"]["code"], "QAI-QUR-0307");
    assert!(value["error"]["summary"].as_str().is_some());
    assert!(value["error"].get("next_command").is_some());

    let (status, _, body) = get(&addr, "/api/v1/quran/ayahs/:::", &[]).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let value = body_json(&body);
    assert_eq!(value["error"]["code"], "QAI-QUR-0311");
    handle.abort();
}

#[tokio::test]
async fn listings_divisions_tokens_resolve_citations() {
    let (addr, handle) = serve_once().await;
    let (status, _, body) = get(&addr, "/api/v1/quran/editions?status=active", &[]).await;
    assert_eq!(status, StatusCode::OK);
    assert_envelope(&body_json(&body));

    let (status, _, body) = get(&addr, "/api/v1/quran/divisions/juz/1", &[]).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body_json(&body)["data"].as_array().unwrap().len(), 1);

    let (status, _, _) = get(&addr, "/api/v1/quran/divisions/nope/1", &[]).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _, body) = get(&addr, "/api/v1/quran/resolve?ref=1:1", &[]).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body_json(&body)["data"]["canonical_reference"], "quran:test@0.1.0:1:1");

    let (status, _, body) = get(&addr, "/api/v1/quran/citations/cit-1", &[]).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body_json(&body)["data"]["verdict"], "ExactMatch");

    let (status, _, _) = get(&addr, "/api/v1/quran/citations/missing", &[]).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    handle.abort();
}

#[tokio::test]
async fn debug_reader_is_labelled_rtl_without_persistence() {
    let (addr, handle) = serve_once().await;
    let (status, headers, body) = get(&addr, "/debug/read/test/1", &[]).await;
    assert_eq!(status, StatusCode::OK);
    let text = String::from_utf8_lossy(&body).into_owned();
    assert!(text.contains("debug view"), "labelled as debug");
    assert!(text.contains("dir=\"rtl\""), "correct RTL");
    assert!(text.contains("quran:test@0.1.0:1:1"));
    assert!(text.contains("font-family"), "declares an Arabic font stack");
    assert!(text.contains("class=\"ayah\""), "one marked block per ayah");
    assert!(text.contains("class=\"marker\""), "ayah markers present");
    assert_eq!(headers.get("content-type").unwrap(), "text/html; charset=utf-8");
    handle.abort();
}

async fn post_json(addr: &str, path: &str, body: &serde_json::Value) -> (StatusCode, Vec<u8>) {
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let payload = serde_json::to_string(body).unwrap();
    let request = format!(
        "POST {path} HTTP/1.1\r\nhost: x\r\nconnection: close\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{payload}",
        payload.len()
    );
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).await.unwrap();
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    let status_line = head.lines().next().unwrap_or("");
    let status: u16 = status_line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
    (StatusCode::from_u16(status).unwrap(), body.as_bytes().to_vec())
}

#[tokio::test]
async fn normalization_preview_matches_cli_pipeline() {
    let (addr, handle) = serve_once().await;

    // Default profile (latest L3): derived text plus the full trace.
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/normalization/preview",
        &serde_json::json!({"text": "بِسْمِ"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let value = body_json(&body);
    assert_envelope(&value);
    assert_eq!(value["data"]["output"], "بسم");
    assert_eq!(value["data"]["profile"], "L3.diacritics@1.0.0");
    assert_eq!(value["data"]["trace"]["profile"], "L3.diacritics@1.0.0");
    assert_eq!(value["data"]["trace"]["contains_heuristic_rules"], false);

    // Byte-identical trace to the application pipeline the CLI uses (AC-P2-39).
    let expected = application::quran_normalize::preview(
        &application::quran_normalize::builtin_registry(),
        "بِسْمِ",
        application::quran_normalize::ProfileId::L3,
        None,
    )
    .unwrap();
    let expected_trace = serde_json::to_value(&expected.trace).unwrap();
    assert_eq!(value["data"]["trace"], expected_trace);

    // Adhoc rule lists and the heuristic label.
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/normalization/preview",
        &serde_json::json!({"text": "والكتب", "profile": "L7.affix@1.0.0"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let value = body_json(&body);
    assert_eq!(value["data"]["output"], "الكتب");
    assert_eq!(value["data"]["trace"]["contains_heuristic_rules"], true);

    // Profile and rules together are a 400 with a namespaced code.
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/normalization/preview",
        &serde_json::json!({"text": "x", "profile": "L3.diacritics", "rules": "N01"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body_json(&body)["error"]["code"], "QAI-NORM-0005");

    // Unknown profiles are a 400 naming the profile.
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/normalization/preview",
        &serde_json::json!({"text": "x", "profile": "L9.nope"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body_json(&body)["error"]["code"], "QAI-NORM-0002");
    handle.abort();
}

#[tokio::test]
async fn normalization_profiles_lists_ladder() {
    let (addr, handle) = serve_once().await;
    let (status, _, body) = get(&addr, "/api/v1/quran/normalization/profiles", &[]).await;
    assert_eq!(status, StatusCode::OK);
    let value = body_json(&body);
    assert_envelope(&value);
    let profiles = value["data"]["profiles"].as_array().unwrap();
    assert_eq!(profiles.len(), 9);
    assert_eq!(profiles[3]["id"], "L3.diacritics");
    handle.abort();
}
