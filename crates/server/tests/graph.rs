//! Phase 4 — graph HTTP read surface contract (D-11/D-12, T-04-11/T-04-13).
//!
//! Fakes the [`application::quran_graph_api::GraphBackend`] trait (no
//! database): every CLI read op has a route returning the identical result
//! shape under the versioned Envelope, with ETag plus conditional-GET
//! handling, typed pre-flight (422), unknown-node (404), truncated-partial
//! (200 plus reason), unavailable-dataset (404 with the morphology code),
//! and denied (403) semantics. No mutation route exists under the graph
//! prefix (D-11 fence).

use std::sync::Arc;

use axum::http::{HeaderMap, StatusCode};
use server::api::{AppState, router};
use tool_registry::{BackendMeta, QuranBackend, ToolRegistry};
use tools::ToolError;

struct FakeBackend;

#[async_trait::async_trait]
impl QuranBackend for FakeBackend {
    async fn backend_get_ayah(
        &self,
        _reference: &quran_core::QuranRef,
        _options: &quran_core::AyahOptions,
    ) -> Result<(quran_core::AyahView, BackendMeta), ToolError> {
        Err(ToolError::Backend { code: "QAI-QUR-0310".into(), detail: "unused".into() })
    }

    async fn backend_get_context(
        &self,
        _reference: &quran_core::QuranRef,
        _spec: &quran_core::ContextSpec,
    ) -> Result<(quran_core::ContextView, BackendMeta), ToolError> {
        Err(ToolError::Backend { code: "QAI-QUR-0310".into(), detail: "unused".into() })
    }
}

struct FakeApi;

#[async_trait::async_trait]
impl server::api::QuranApiBackend for FakeApi {
    async fn api_editions(&self) -> Result<Vec<serde_json::Value>, ToolError> {
        Ok(Vec::new())
    }

    async fn api_edition(&self, _slug: &str) -> Result<serde_json::Value, ToolError> {
        Err(ToolError::Backend { code: "QAI-QUR-0306".into(), detail: "unused".into() })
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
        Err(ToolError::Backend { code: "QAI-QUR-0306".into(), detail: "unused".into() })
    }

    async fn api_division(
        &self,
        _edition: &str,
        _kind: &str,
        _number: u32,
        _options: &quran_core::AyahOptions,
    ) -> Result<Vec<quran_core::AyahView>, ToolError> {
        Err(ToolError::Backend { code: "QAI-QUR-0309".into(), detail: "unused".into() })
    }

    async fn api_tokens(&self, _reference: &str) -> Result<Vec<quran_core::Token>, ToolError> {
        Ok(Vec::new())
    }

    async fn api_citation(&self, _id: &str) -> Result<citations::ResolvedCitation, ToolError> {
        Err(ToolError::Backend { code: "QAI-QUR-0322".into(), detail: "unused".into() })
    }
}

struct FakeSearch;

#[async_trait::async_trait]
impl application::quran_search_api::SearchBackend for FakeSearch {
    async fn search_exact(
        &self,
        _args: application::quran_search_api::ExactArgs,
    ) -> Result<application::quran_search::SearchOutput, application::quran_search::SearchError>
    {
        Err(application::quran_search_api::reject("unused"))
    }

    async fn search_normalized(
        &self,
        _args: application::quran_search_api::NormalizedArgs,
    ) -> Result<application::quran_search::SearchOutput, application::quran_search::SearchError>
    {
        Err(application::quran_search_api::reject("unused"))
    }

    async fn search_phrase(
        &self,
        _args: application::quran_search_api::PhraseArgs,
    ) -> Result<application::quran_search::SearchOutput, application::quran_search::SearchError>
    {
        Err(application::quran_search_api::reject("unused"))
    }

    async fn search_concatenated(
        &self,
        _args: application::quran_search_api::ConcatenatedArgs,
    ) -> Result<application::quran_search::SearchOutput, application::quran_search::SearchError>
    {
        Err(application::quran_search_api::reject("unused"))
    }

    async fn search_regex(
        &self,
        _args: application::quran_search_api::RegexArgs,
    ) -> Result<application::quran_search::SearchOutput, application::quran_search::SearchError>
    {
        Err(application::quran_search_api::reject("unused"))
    }
}

struct FakeLexicon;

#[async_trait::async_trait]
impl application::quran_lexicon_api::LexiconBackend for FakeLexicon {
    async fn word_family(
        &self,
        _args: application::quran_lexicon_api::FamilyArgs,
    ) -> Result<
        Vec<application::quran_morphology::FamilyMemberView>,
        application::quran_lexicon_api::LexiconApiError,
    > {
        Err(application::quran_lexicon_api::LexiconApiError::Unsupported {
            detail: "unused".to_string(),
        })
    }

    async fn root_frequency(
        &self,
        _args: application::quran_lexicon_api::RootFrequencyArgs,
    ) -> Result<
        application::quran_counting::FrequencyReport,
        application::quran_lexicon_api::LexiconApiError,
    > {
        Err(application::quran_lexicon_api::LexiconApiError::Unsupported {
            detail: "unused".to_string(),
        })
    }

    async fn lemma_frequency(
        &self,
        _args: application::quran_lexicon_api::LemmaFrequencyArgs,
    ) -> Result<
        application::quran_counting::FrequencyReport,
        application::quran_lexicon_api::LexiconApiError,
    > {
        Err(application::quran_lexicon_api::LexiconApiError::Unsupported {
            detail: "unused".to_string(),
        })
    }

    async fn count_frequency(
        &self,
        _target: &str,
        _profile: &str,
    ) -> Result<
        application::quran_counting::FrequencyReport,
        application::quran_lexicon_api::LexiconApiError,
    > {
        Err(application::quran_lexicon_api::LexiconApiError::Unsupported {
            detail: "unused".to_string(),
        })
    }

    async fn count_distribution(
        &self,
        _target: &str,
        _profile: &str,
    ) -> Result<
        application::quran_counting::DistributionReport,
        application::quran_lexicon_api::LexiconApiError,
    > {
        Err(application::quran_lexicon_api::LexiconApiError::Unsupported {
            detail: "unused".to_string(),
        })
    }

    async fn count_occurrences(
        &self,
        _target: &str,
        _profile: &str,
    ) -> Result<
        application::quran_counting::OccurrenceSpan,
        application::quran_lexicon_api::LexiconApiError,
    > {
        Err(application::quran_lexicon_api::LexiconApiError::Unsupported {
            detail: "unused".to_string(),
        })
    }

    async fn count_hapax(
        &self,
        _profile: &str,
        _limit: usize,
    ) -> Result<
        application::quran_counting::HapaxReport,
        application::quran_lexicon_api::LexiconApiError,
    > {
        Err(application::quran_lexicon_api::LexiconApiError::Unsupported {
            detail: "unused".to_string(),
        })
    }

    async fn count_cooccurrence(
        &self,
        _target: &str,
        _profile: &str,
        _window: usize,
        _limit: usize,
    ) -> Result<
        (application::quran_counting::CountingRules, Vec<application::quran_counting::CooccurrenceHit>),
        application::quran_lexicon_api::LexiconApiError,
    > {
        Err(application::quran_lexicon_api::LexiconApiError::Unsupported {
            detail: "unused".to_string(),
        })
    }

    async fn count_collocation(
        &self,
        _target: &str,
        _profile: &str,
        _window: usize,
        _limit: usize,
    ) -> Result<
        (application::quran_counting::CountingRules, Vec<application::quran_counting::CollocationHit>),
        application::quran_lexicon_api::LexiconApiError,
    > {
        Err(application::quran_lexicon_api::LexiconApiError::Unsupported {
            detail: "unused".to_string(),
        })
    }

    async fn count_numeric_report(
        &self,
        _target: &str,
        _profile: &str,
    ) -> Result<
        application::quran_counting::NumericReport,
        application::quran_lexicon_api::LexiconApiError,
    > {
        Err(application::quran_lexicon_api::LexiconApiError::Unsupported {
            detail: "unused".to_string(),
        })
    }

    async fn count_missing_form(
        &self,
        _target: &str,
        _profile: &str,
    ) -> Result<
        application::quran_counting::MissingFormReport,
        application::quran_lexicon_api::LexiconApiError,
    > {
        Err(application::quran_lexicon_api::LexiconApiError::Unsupported {
            detail: "unused".to_string(),
        })
    }

    async fn count_near_duplicates(
        &self,
        _threshold: f64,
        _limit: usize,
    ) -> Result<
        (application::quran_counting::CountingRules, Vec<application::quran_counting::NearDuplicateHit>),
        application::quran_lexicon_api::LexiconApiError,
    > {
        Err(application::quran_lexicon_api::LexiconApiError::Unsupported {
            detail: "unused".to_string(),
        })
    }

    async fn count_interval(
        &self,
        _target: &str,
        _profile: &str,
    ) -> Result<
        application::quran_counting::OccurrenceSpan,
        application::quran_lexicon_api::LexiconApiError,
    > {
        Err(application::quran_lexicon_api::LexiconApiError::Unsupported {
            detail: "unused".to_string(),
        })
    }

    async fn count_unusual_usage(
        &self,
        _target: &str,
        _profile: &str,
    ) -> Result<
        application::quran_counting::NumericReport,
        application::quran_lexicon_api::LexiconApiError,
    > {
        Err(application::quran_lexicon_api::LexiconApiError::Unsupported {
            detail: "unused".to_string(),
        })
    }
}

/// Graph backend stub: sentinel selectors stand in for the dataset states a
/// real deployment reaches — an unknown node (`ayah:9:9`), a truncated
/// expansion (`truncated-node`), a denied scope (`denied-node`), and a
/// missing word-root dataset (`no-dataset` for root-family).
struct FakeGraph;

fn fake_snapshot() -> application::quran_graph_api::GraphSnapshotMeta {
    application::quran_graph_api::GraphSnapshotMeta {
        edition_slug: "test".into(),
        edition_version: "0.1.0".into(),
        edition_id: "ed-1".into(),
        text_hash: "sha256:ab".into(),
        corpus_generation: 7,
    }
}

fn explained_edge_json() -> serde_json::Value {
    serde_json::json!({
        "src": "ayah:1:1",
        "edge": "NEXT",
        "dst": "ayah:1:2",
        "provenance": {"kind": "structural", "input_version": "v1"},
    })
}

fn explanation_json(start: &str, end: Option<&str>, truncated: bool) -> serde_json::Value {
    serde_json::json!({
        "start": start,
        "end": end,
        "nodes": ["ayah:1:1", "ayah:1:2"],
        "edges": [explained_edge_json()],
        "paths": [],
        "applied_filters": {
            "budgets": {
                "max_hops": 1,
                "max_nodes": 500,
                "max_paths": 10,
                "max_edges": 2000,
                "max_fanout": 128,
                "timeout_ms": 5000,
            },
            "edge_types": null,
            "direction": "both",
            "authz": "unrestricted",
        },
        "snapshot": {
            "projection_id": "quran-structural-v1",
            "builder_version": "quran-structural-builder@1.0.0",
            "corpus_generation": 7,
        },
        "truncated": truncated,
        "incomplete_reason": if truncated {
            serde_json::Value::String("node budget 500 exhausted".to_string())
        } else {
            serde_json::Value::Null
        },
        "duration_ms": 3,
    })
}

fn nodes_json() -> serde_json::Value {
    serde_json::json!([
        {"stable_id": "ayah:1:1", "kind": "ayah", "attrs": {}},
        {"stable_id": "ayah:1:2", "kind": "ayah", "attrs": {}},
    ])
}

fn edges_json() -> serde_json::Value {
    serde_json::json!([{
        "src": "ayah:1:1",
        "edge": "NEXT",
        "dst": "ayah:1:2",
        "assertion_id": null,
        "attrs": {"input_version": "v1"},
    }])
}

fn path_json() -> serde_json::Value {
    serde_json::json!({
        "node_ids": ["ayah:1:1", "ayah:1:2"],
        "edges": [explained_edge_json()],
    })
}

#[async_trait::async_trait]
impl application::quran_graph_api::GraphBackend for FakeGraph {
    async fn neighbors(
        &self,
        args: application::quran_graph_api::NeighborsArgs,
    ) -> Result<
        application::quran_graph_api::NeighborsOutput,
        application::quran_graph_api::GraphApiError,
    > {
        use application::quran_graph_api::GraphApiError;
        match args.node.as_str() {
            "ayah:9:9" => Err(GraphApiError::unknown_node("ayah:9:9")),
            "denied-node" => Err(GraphApiError::access_denied("restricted assertion scope")),
            _ => {
                let truncated = args.node == "truncated-node";
                Ok(serde_json::from_value(serde_json::json!({
                    "nodes": nodes_json(),
                    "edges": edges_json(),
                    "truncated": truncated,
                    "incomplete_reason": if truncated {
                        serde_json::Value::String("node budget 500 exhausted".to_string())
                    } else {
                        serde_json::Value::Null
                    },
                    "explanation": explanation_json("ayah:1:1", None, truncated),
                }))
                .unwrap())
            }
        }
    }

    async fn reachability(
        &self,
        args: application::quran_graph_api::PathArgs,
    ) -> Result<
        application::quran_graph_api::ReachabilityOutput,
        application::quran_graph_api::GraphApiError,
    > {
        use application::quran_graph_api::GraphApiError;
        if args.from == "ayah:9:9" {
            return Err(GraphApiError::unknown_node("ayah:9:9"));
        }
        Ok(serde_json::from_value(serde_json::json!({
            "reachable": true,
            "hops": 1,
            "explanation": explanation_json(&args.from, Some(&args.to), false),
        }))
        .unwrap())
    }

    async fn shortest_path(
        &self,
        args: application::quran_graph_api::PathArgs,
    ) -> Result<
        application::quran_graph_api::ShortestOutput,
        application::quran_graph_api::GraphApiError,
    > {
        use application::quran_graph_api::GraphApiError;
        if args.from == "ayah:9:9" {
            return Err(GraphApiError::unknown_node("ayah:9:9"));
        }
        Ok(serde_json::from_value(serde_json::json!({
            "path": path_json(),
            "explanation": explanation_json(&args.from, Some(&args.to), false),
        }))
        .unwrap())
    }

    async fn paths(
        &self,
        args: application::quran_graph_api::PathsArgs,
    ) -> Result<
        application::quran_graph_api::PathsOutput,
        application::quran_graph_api::GraphApiError,
    > {
        use application::quran_graph_api::GraphApiError;
        if args.from == "ayah:9:9" {
            return Err(GraphApiError::unknown_node("ayah:9:9"));
        }
        let mut explanation = explanation_json(&args.from, Some(&args.to), false);
        explanation["paths"] = serde_json::json!([path_json()]);
        Ok(serde_json::from_value(serde_json::json!({
            "paths": [path_json()],
            "truncated": false,
            "incomplete_reason": null,
            "explanation": explanation,
        }))
        .unwrap())
    }

    async fn subgraph(
        &self,
        args: application::quran_graph_api::SubgraphArgs,
    ) -> Result<
        application::quran_graph_api::SubgraphOutput,
        application::quran_graph_api::GraphApiError,
    > {
        use application::quran_graph_api::GraphApiError;
        if args.seeds.iter().any(|seed| seed == "denied-node") {
            return Err(GraphApiError::access_denied("restricted assertion scope"));
        }
        let truncated = args.seeds.iter().any(|seed| seed == "truncated-node");
        let start = args.seeds.first().cloned().unwrap_or_default();
        Ok(serde_json::from_value(serde_json::json!({
            "nodes": nodes_json(),
            "edges": edges_json(),
            "truncated": truncated,
            "incomplete_reason": if truncated {
                serde_json::Value::String("node budget 500 exhausted".to_string())
            } else {
                serde_json::Value::Null
            },
            "explanation": explanation_json(&start, None, truncated),
        }))
        .unwrap())
    }

    async fn pattern(
        &self,
        args: application::quran_graph_api::PatternArgs,
    ) -> Result<
        application::quran_graph_api::PatternOutput,
        application::quran_graph_api::GraphApiError,
    > {
        let truncated = args.seeds.iter().any(|seed| seed == "truncated-node");
        let start = args.seeds.first().cloned().unwrap_or_default();
        Ok(serde_json::from_value(serde_json::json!({
            "nodes": nodes_json(),
            "edges": edges_json(),
            "truncated": truncated,
            "incomplete_reason": if truncated {
                serde_json::Value::String("node budget 500 exhausted".to_string())
            } else {
                serde_json::Value::Null
            },
            "explanation": explanation_json(&start, None, truncated),
        }))
        .unwrap())
    }

    async fn root_family(
        &self,
        args: application::quran_graph_api::RootFamilyArgs,
    ) -> Result<
        application::quran_graph_api::RootFamilyOutput,
        application::quran_graph_api::GraphApiError,
    > {
        use application::quran_graph_api::GraphApiError;
        if args.root == "no-dataset" {
            return Err(GraphApiError::Morphology(
                application::quran_morphology::MorphologyToolError::UnavailableDataset {
                    capability: "root family".to_string(),
                },
            ));
        }
        Ok(serde_json::from_value(serde_json::json!({
            "root": args.root,
            "dataset": "test-morph@0.1.0",
            "ayahs": [{"surah": 1, "ayah": 1, "position": 1}],
            "limit": 25,
            "duration_ms": 3,
        }))
        .unwrap())
    }

    async fn snapshot_meta(
        &self,
    ) -> Result<
        application::quran_graph_api::GraphSnapshotMeta,
        application::quran_graph_api::GraphApiError,
    > {
        Ok(fake_snapshot())
    }
}

fn test_state() -> AppState {
    AppState {
        tools: Arc::new(ToolRegistry::new(Arc::new(FakeBackend))),
        api: Arc::new(FakeApi),
        search: Arc::new(FakeSearch),
        lexicon: Arc::new(FakeLexicon),
        graph: Arc::new(FakeGraph),
    }
}

async fn serve_once() -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let handle = tokio::spawn(async move {
        axum::serve(listener, router(test_state())).await.unwrap();
    });
    (addr, handle)
}

async fn post_raw(
    addr: &str,
    path: &str,
    body: &serde_json::Value,
    extra_headers: &[(&str, &str)],
) -> (StatusCode, HeaderMap, Vec<u8>) {
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let payload = serde_json::to_string(body).unwrap();
    let mut request = format!(
        "POST {path} HTTP/1.1\r\nhost: x\r\nconnection: close\r\ncontent-type: application/json\r\ncontent-length: {}\r\n",
        payload.len()
    );
    for (name, value) in extra_headers {
        request.push_str(&format!("{name}: {value}\r\n"));
    }
    request.push_str("\r\n");
    request.push_str(&payload);
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

async fn post_json(addr: &str, path: &str, body: &serde_json::Value) -> (StatusCode, Vec<u8>) {
    let (status, _, body) = post_raw(addr, path, body, &[]).await;
    (status, body)
}

fn body_json(body: &[u8]) -> serde_json::Value {
    serde_json::from_slice(body).expect("response is JSON")
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

/// D-12: the neighbors route returns the read-service result under the
/// versioned Envelope with ETag support, and the reproducibility block pins
/// the answering projection build.
#[tokio::test]
async fn graph_neighbors_route_returns_enveloped_result_with_etag() {
    let (addr, handle) = serve_once().await;
    let (status, headers, body) = post_raw(
        &addr,
        "/api/v1/quran/graph/neighbors",
        &serde_json::json!({"node": "ayah:1:1"}),
        &[],
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let value = body_json(&body);
    assert_envelope(&value);
    assert_eq!(value["data"]["nodes"].as_array().unwrap().len(), 2);
    assert_eq!(value["data"]["edges"][0]["edge"], "NEXT");
    assert_eq!(value["data"]["truncated"], false);
    // D-14: the full explainability payload travels on the result.
    assert_eq!(value["data"]["explanation"]["start"], "ayah:1:1");
    assert_eq!(value["data"]["explanation"]["snapshot"]["projection_id"], "quran-structural-v1");
    assert_eq!(value["data"]["explanation"]["edges"][0]["provenance"]["kind"], "structural");
    // The reproducibility block pins the graph build, not just the edition.
    assert_eq!(value["meta"]["reproducibility"]["tool"], "quran.graph.neighbors");
    assert_eq!(value["meta"]["reproducibility"]["projection_id"], "quran-structural-v1");
    assert!(value["meta"]["reproducibility"]["builder_version"].is_string());
    // ETag support: the tag derives from the edition read context, and a
    // matching conditional request short-circuits the body (T-04-13).
    let etag = headers.get("etag").expect("ETag").to_str().unwrap().to_string();
    assert!(etag.contains("sha256:ab"), "ETag derives from text hash: {etag}");
    let (status, _, _) = post_raw(
        &addr,
        "/api/v1/quran/graph/neighbors",
        &serde_json::json!({"node": "ayah:1:1"}),
        &[("if-none-match", &etag)],
    )
    .await;
    assert_eq!(status, StatusCode::NOT_MODIFIED);
    handle.abort();
}

/// D-12: all three path modes answer under the same route with the
/// CLI's mode selector; an unknown mode is a typed 422 before any I/O.
#[tokio::test]
async fn graph_path_modes_return_enveloped_results() {
    let (addr, handle) = serve_once().await;

    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/graph/path",
        &serde_json::json!({"from": "ayah:1:1", "to": "ayah:1:2", "mode": "reachability"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let value = body_json(&body);
    assert_envelope(&value);
    assert_eq!(value["data"]["reachable"], true);
    assert_eq!(value["data"]["hops"], 1);
    assert_eq!(value["meta"]["reproducibility"]["tool"], "quran.graph.path");

    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/graph/path",
        &serde_json::json!({"from": "ayah:1:1", "to": "ayah:1:2", "mode": "shortest"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let value = body_json(&body);
    assert_envelope(&value);
    assert_eq!(value["data"]["path"]["node_ids"], serde_json::json!(["ayah:1:1", "ayah:1:2"]));

    // The default mode is the CLI's default (`paths`).
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/graph/path",
        &serde_json::json!({"from": "ayah:1:1", "to": "ayah:1:2"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let value = body_json(&body);
    assert_eq!(value["data"]["paths"].as_array().unwrap().len(), 1);
    assert_eq!(
        value["data"]["explanation"]["paths"][0]["node_ids"],
        serde_json::json!(["ayah:1:1", "ayah:1:2"])
    );

    // An unknown mode is a coded 422 from the shared CLI/API parser.
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/graph/path",
        &serde_json::json!({"from": "ayah:1:1", "to": "ayah:1:2", "mode": "sideways"}),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body_json(&body)["error"]["code"], "QAI-GRAPH-0003");
    handle.abort();
}

/// D-12: subgraph and typed patterns answer enveloped with the same node,
/// edge, and explanation shape as neighbors.
#[tokio::test]
async fn graph_subgraph_and_pattern_routes_return_enveloped_results() {
    let (addr, handle) = serve_once().await;
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/graph/subgraph",
        &serde_json::json!({"seeds": ["ayah:1:1"]}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let value = body_json(&body);
    assert_envelope(&value);
    assert_eq!(value["data"]["nodes"].as_array().unwrap().len(), 2);
    assert_eq!(value["data"]["truncated"], false);
    assert_eq!(value["meta"]["reproducibility"]["tool"], "quran.graph.subgraph");

    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/graph/pattern",
        &serde_json::json!({
            "seeds": ["ayah:1:1"],
            "steps": [{"edge": "NEXT"}, {"edge": "CONTAINS", "node_kind": "ayah"}],
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let value = body_json(&body);
    assert_envelope(&value);
    assert_eq!(value["data"]["edges"][0]["edge"], "NEXT");
    assert_eq!(value["meta"]["reproducibility"]["tool"], "quran.graph.pattern");

    // An unknown edge rejects pre-flight with the pattern code (422).
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/graph/pattern",
        &serde_json::json!({"seeds": ["ayah:1:1"], "steps": [{"edge": "NOPE"}]}),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body_json(&body)["error"]["code"], "QAI-GRAPH-0003");
    handle.abort();
}

/// D-12: root-family answers ranked ayahs with dataset attribution, and the
/// reproducibility block names the answering dataset.
#[tokio::test]
async fn graph_root_family_route_returns_ranked_ayahs() {
    let (addr, handle) = serve_once().await;
    let (status, body) =
        post_json(&addr, "/api/v1/quran/graph/root-family", &serde_json::json!({"root": "r-1"}))
            .await;
    assert_eq!(status, StatusCode::OK);
    let value = body_json(&body);
    assert_envelope(&value);
    assert_eq!(value["data"]["dataset"], "test-morph@0.1.0");
    assert_eq!(value["data"]["ayahs"], serde_json::json!([{"surah": 1, "ayah": 1, "position": 1}]));
    assert_eq!(value["meta"]["reproducibility"]["tool"], "quran.graph.root_family");
    assert_eq!(value["meta"]["reproducibility"]["dataset"], "test-morph@0.1.0");
    handle.abort();
}

/// Budget pre-flight violations are 422 with `QAI-GRAPH-0002`, on every
/// route — including the boundary values shared with the CLI.
#[tokio::test]
async fn graph_preflight_violations_are_422() {
    let (addr, handle) = serve_once().await;
    for (path, body) in [
        (
            "/api/v1/quran/graph/neighbors",
            serde_json::json!({"node": "ayah:1:1", "budgets": {"max_hops": 0}}),
        ),
        (
            "/api/v1/quran/graph/path",
            serde_json::json!({"from": "ayah:1:1", "to": "ayah:1:2", "budgets": {"max_nodes": 0}}),
        ),
        (
            "/api/v1/quran/graph/subgraph",
            serde_json::json!({"seeds": ["ayah:1:1"], "budgets": {"max_fanout": 0}}),
        ),
        (
            "/api/v1/quran/graph/pattern",
            serde_json::json!({
                "seeds": ["ayah:1:1"],
                "steps": [{"edge": "NEXT"}],
                "budgets": {"max_edges": 0},
            }),
        ),
    ] {
        let (status, body) = post_json(&addr, path, &body).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{path}");
        let value = body_json(&body);
        assert_eq!(value["error"]["code"], "QAI-GRAPH-0002", "{path}");
        assert!(value.get("data").is_none(), "{path}: a violation carries no data");
    }

    // Boundary values at the edge of the allowed ranges still run (200).
    for (path, body) in [
        (
            "/api/v1/quran/graph/neighbors",
            serde_json::json!({"node": "ayah:1:1", "budgets": {"max_hops": 32, "max_nodes": 1}}),
        ),
        (
            "/api/v1/quran/graph/subgraph",
            serde_json::json!({"seeds": ["ayah:1:1"], "budgets": {"max_hops": 1, "max_nodes": 100000}}),
        ),
    ] {
        let (status, _) = post_json(&addr, path, &body).await;
        assert_eq!(status, StatusCode::OK, "{path}");
    }
    handle.abort();
}

/// Unknown nodes and projections are 404 with their `QAI-GRAPH` codes —
/// never an empty 200 a caller could read as absence.
#[tokio::test]
async fn graph_unknown_nodes_are_404() {
    let (addr, handle) = serve_once().await;
    for (path, body) in [
        ("/api/v1/quran/graph/neighbors", serde_json::json!({"node": "ayah:9:9"})),
        (
            "/api/v1/quran/graph/path",
            serde_json::json!({"from": "ayah:9:9", "to": "ayah:1:2", "mode": "shortest"}),
        ),
    ] {
        let (status, body) = post_json(&addr, path, &body).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
        let value = body_json(&body);
        assert_eq!(value["error"]["code"], "QAI-GRAPH-0004", "{path}");
        assert!(value.get("data").is_none(), "{path}: unknown carries no data");
    }
    handle.abort();
}

/// Truncated expansions stay 200 with `truncated: true` plus a non-empty
/// reason — never an error, never an empty result masking a partial.
#[tokio::test]
async fn graph_truncated_results_are_200_with_reason() {
    let (addr, handle) = serve_once().await;
    for (path, body) in [
        ("/api/v1/quran/graph/neighbors", serde_json::json!({"node": "truncated-node"})),
        ("/api/v1/quran/graph/subgraph", serde_json::json!({"seeds": ["truncated-node"]})),
        (
            "/api/v1/quran/graph/pattern",
            serde_json::json!({"seeds": ["truncated-node"], "steps": [{"edge": "NEXT"}]}),
        ),
    ] {
        let (status, body) = post_json(&addr, path, &body).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        let value = body_json(&body);
        assert_envelope(&value);
        assert_eq!(value["data"]["truncated"], true, "{path}");
        assert!(
            !value["data"]["incomplete_reason"].as_str().unwrap_or_default().is_empty(),
            "{path}: a partial carries its reason"
        );
        assert_eq!(value["data"]["explanation"]["truncated"], true, "{path}");
    }
    handle.abort();
}

/// An unavailable word-root capability is 404 with the morphology
/// diagnostic preserved — never an empty family.
#[tokio::test]
async fn graph_unavailable_dataset_is_404_with_morphology_code() {
    let (addr, handle) = serve_once().await;
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/graph/root-family",
        &serde_json::json!({"root": "no-dataset"}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let value = body_json(&body);
    assert_eq!(value["error"]["code"], "QAI-MORPH-0004");
    assert!(value.get("data").is_none(), "an unavailable capability carries no data");
    handle.abort();
}

/// T-04-11: unknown, truncated, and denied stay distinguishable — 404 for
/// unknown, 200 plus `truncated` for partial, 403 for denied.
#[tokio::test]
async fn graph_denied_stays_distinguishable_from_unknown_and_truncated() {
    let (addr, handle) = serve_once().await;
    for (path, body) in [
        ("/api/v1/quran/graph/neighbors", serde_json::json!({"node": "denied-node"})),
        ("/api/v1/quran/graph/subgraph", serde_json::json!({"seeds": ["denied-node"]})),
    ] {
        let (status, body) = post_json(&addr, path, &body).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{path}");
        let value = body_json(&body);
        assert_eq!(value["error"]["code"], "QAI-GRAPH-0006", "{path}");
        assert!(value.get("data").is_none(), "{path}: denied carries no data");
    }
    handle.abort();
}

/// D-11 fence: no mutation route exists under the graph prefix — build,
/// review, and repair stay CLI-only.
#[tokio::test]
async fn graph_has_no_mutation_route() {
    let (addr, handle) = serve_once().await;
    for path in ["/api/v1/quran/graph/build", "/api/v1/quran/graph/review"] {
        let (status, _) = post_json(&addr, path, &serde_json::json!({})).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path} must not exist");
    }
    handle.abort();
}
