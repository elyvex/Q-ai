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
        projection_id: String::new(),
        builder_version: String::new(),
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
        } else if id == "cit-mismatch" {
            // A persisted citation whose stored verdict is a hard failure: the
            // live canonical hash no longer matches the stored baseline.
            Ok(citations::ResolvedCitation {
                citation_id: id.into(),
                verdict: citations::QuotationVerdict::Mismatch {
                    first_difference_at: 0,
                    expected_hash: "ab".repeat(32),
                },
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

struct FakeSearch;

fn empty_search_output(rule_set: &str) -> application::quran_search::SearchOutput {
    application::quran_search::SearchOutput {
        hits: Vec::new(),
        total_matches: 0,
        truncated: false,
        rule_set: rule_set.to_string(),
        generation: 1,
        warnings: Vec::new(),
        regex_report: None,
    }
}

#[async_trait::async_trait]
impl application::quran_search_api::SearchBackend for FakeSearch {
    async fn search_exact(
        &self,
        _args: application::quran_search_api::ExactArgs,
    ) -> Result<application::quran_search::SearchOutput, application::quran_search::SearchError>
    {
        Ok(empty_search_output("L0.exact@1.0.0"))
    }

    async fn search_normalized(
        &self,
        _args: application::quran_search_api::NormalizedArgs,
    ) -> Result<application::quran_search::SearchOutput, application::quran_search::SearchError>
    {
        Ok(empty_search_output("L3.diacritics@1.0.0"))
    }

    async fn search_phrase(
        &self,
        _args: application::quran_search_api::PhraseArgs,
    ) -> Result<application::quran_search::SearchOutput, application::quran_search::SearchError>
    {
        Ok(empty_search_output("L3.diacritics@1.0.0"))
    }

    async fn search_concatenated(
        &self,
        _args: application::quran_search_api::ConcatenatedArgs,
    ) -> Result<application::quran_search::SearchOutput, application::quran_search::SearchError>
    {
        Ok(empty_search_output("L6.skeleton@1.0.0"))
    }

    async fn search_regex(
        &self,
        args: application::quran_search_api::RegexArgs,
    ) -> Result<application::quran_search::SearchOutput, application::quran_search::SearchError>
    {
        if args.pattern == ".*x" {
            return Err(application::quran_search_api::reject(
                "leading `.*` is rejected; anchor the pattern",
            ));
        }
        let mut output = empty_search_output("L3.diacritics@1.0.0");
        output.regex_report = Some(application::quran_search::RegexReport {
            pattern: args.pattern.clone(),
            field: args.field.clone(),
            terms_matched: Vec::new(),
            terms_examined: 0,
        });
        Ok(output)
    }
}

/// Lexicon backend stub: the sentinel selectors stand in for the two dataset
/// states a real deployment reaches — a registered synthetic lexicon and no
/// active dataset at all (`UnavailableDataset`, never an empty result).
struct FakeLexicon;

fn lexicon_frequency_report(
    target: &str,
    mode: application::quran_counting::MultiAnalysisHandling,
    count: u64,
) -> application::quran_counting::FrequencyReport {
    application::quran_counting::FrequencyReport {
        target: target.to_string(),
        rules: application::quran_counting::CountingRules {
            profile: application::quran_lexicon_api::DEFAULT_PROFILE.to_string(),
            profile_version: "1.0.0".to_string(),
            datasets: vec!["test-morph@0.1.0".to_string()],
            multi_analysis_handling: mode,
            window: None,
            exclusions: Vec::new(),
        },
        count,
        by_surah: std::collections::BTreeMap::from([(1, count)]),
        checksum: format!("sha256:{}", "ab".repeat(32)),
    }
}

#[async_trait::async_trait]
impl application::quran_lexicon_api::LexiconBackend for FakeLexicon {
    async fn word_family(
        &self,
        args: application::quran_lexicon_api::FamilyArgs,
    ) -> Result<
        Vec<application::quran_morphology::FamilyMemberView>,
        application::quran_lexicon_api::LexiconApiError,
    > {
        use application::quran_lexicon_api::LexiconApiError;
        match args.id.as_str() {
            // The no-active-dataset path: a typed capability error, never an
            // empty relation list (T-03-21).
            "token:1:1:1" => {
                Err(LexiconApiError::UnavailableDataset { capability: "word family".to_string() })
            }
            _ => Ok(vec![application::quran_morphology::FamilyMemberView {
                id: "token:1:1:2".to_string(),
                kind: "token".to_string(),
                relation: "same_root".to_string(),
                explanation: "shares root r-1 with the queried token".to_string(),
                dataset: Some("test-morph@0.1.0".to_string()),
            }]),
        }
    }

    async fn root_frequency(
        &self,
        args: application::quran_lexicon_api::RootFrequencyArgs,
    ) -> Result<
        application::quran_counting::FrequencyReport,
        application::quran_lexicon_api::LexiconApiError,
    > {
        use application::quran_lexicon_api::LexiconApiError;
        if args.root == "tst-gone" {
            return Err(LexiconApiError::UnavailableDataset {
                capability: "root frequency".to_string(),
            });
        }
        Ok(lexicon_frequency_report(&args.root, args.mode, 64))
    }

    async fn lemma_frequency(
        &self,
        args: application::quran_lexicon_api::LemmaFrequencyArgs,
    ) -> Result<
        application::quran_counting::FrequencyReport,
        application::quran_lexicon_api::LexiconApiError,
    > {
        use application::quran_lexicon_api::LexiconApiError;
        if args.lemma == "tst-gone" {
            return Err(LexiconApiError::UnavailableDataset {
                capability: "lemma frequency".to_string(),
            });
        }
        Ok(lexicon_frequency_report(&args.lemma, args.mode, 127))
    }

    async fn count_frequency(
        &self,
        target: &str,
        _profile: &str,
    ) -> Result<
        application::quran_counting::FrequencyReport,
        application::quran_lexicon_api::LexiconApiError,
    > {
        use application::quran_counting::{CountingRules, MultiAnalysisHandling};
        Ok(application::quran_counting::FrequencyReport {
            target: target.to_string(),
            rules: CountingRules {
                profile: "L3.diacritics".to_string(),
                profile_version: "1.0.0".to_string(),
                datasets: vec!["stored-forms".to_string()],
                multi_analysis_handling: MultiAnalysisHandling::SingleSource,
                window: None,
                exclusions: vec![],
            },
            count: 42,
            by_surah: std::collections::BTreeMap::from([(1, 42)]),
            checksum: "sha256:fake".to_string(),
        })
    }

    async fn count_distribution(
        &self,
        target: &str,
        profile: &str,
    ) -> Result<
        application::quran_counting::DistributionReport,
        application::quran_lexicon_api::LexiconApiError,
    > {
        let freq = self.count_frequency(target, profile).await?;
        Ok(application::quran_counting::DistributionReport {
            frequency: freq,
            partition_provenance: "partitions keyed by surah number".to_string(),
            warnings: vec![],
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
        use application::quran_counting::{
            CountingRules, INTERVAL_DISCLAIMER, MultiAnalysisHandling,
        };
        Ok(application::quran_counting::OccurrenceSpan {
            first: Some((1, 1)),
            last: Some((2, 5)),
            ayah_span: Some(10),
            rules: CountingRules {
                profile: "L3.diacritics".to_string(),
                profile_version: "1.0.0".to_string(),
                datasets: vec!["stored-forms".to_string()],
                multi_analysis_handling: MultiAnalysisHandling::SingleSource,
                window: None,
                exclusions: vec![],
            },
            disclaimer: INTERVAL_DISCLAIMER.to_string(),
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
        use application::quran_counting::{CountingRules, MultiAnalysisHandling};
        Ok(application::quran_counting::HapaxReport {
            profile: "L3.diacritics".to_string(),
            rules: CountingRules {
                profile: "L3.diacritics".to_string(),
                profile_version: "1.0.0".to_string(),
                datasets: vec!["stored-forms".to_string()],
                multi_analysis_handling: MultiAnalysisHandling::SingleSource,
                window: None,
                exclusions: vec![],
            },
            hapax: vec![("fake-hapax".to_string(), 1)],
        })
    }

    async fn count_cooccurrence(
        &self,
        _target: &str,
        _profile: &str,
        _window: usize,
        _limit: usize,
    ) -> Result<
        (
            application::quran_counting::CountingRules,
            Vec<application::quran_counting::CooccurrenceHit>,
        ),
        application::quran_lexicon_api::LexiconApiError,
    > {
        use application::quran_counting::{CountingRules, MultiAnalysisHandling};
        Ok((
            CountingRules {
                profile: "L3.diacritics".to_string(),
                profile_version: "1.0.0".to_string(),
                datasets: vec!["stored-forms".to_string()],
                multi_analysis_handling: MultiAnalysisHandling::SingleSource,
                window: Some("token:3".to_string()),
                exclusions: vec![],
            },
            vec![],
        ))
    }

    async fn count_collocation(
        &self,
        _target: &str,
        _profile: &str,
        _window: usize,
        _limit: usize,
    ) -> Result<
        (
            application::quran_counting::CountingRules,
            Vec<application::quran_counting::CollocationHit>,
        ),
        application::quran_lexicon_api::LexiconApiError,
    > {
        use application::quran_counting::{CountingRules, MultiAnalysisHandling};
        Ok((
            CountingRules {
                profile: "L3.diacritics".to_string(),
                profile_version: "1.0.0".to_string(),
                datasets: vec!["stored-forms".to_string()],
                multi_analysis_handling: MultiAnalysisHandling::SingleSource,
                window: Some("token:3".to_string()),
                exclusions: vec![],
            },
            vec![],
        ))
    }

    async fn count_numeric_report(
        &self,
        target: &str,
        profile: &str,
    ) -> Result<
        application::quran_counting::NumericReport,
        application::quran_lexicon_api::LexiconApiError,
    > {
        let freq = self.count_frequency(target, profile).await?;
        Ok(application::quran_counting::NumericReport {
            frequency: freq,
            note: application::quran_counting::NO_INTERPRETATION_NOTE.to_string(),
        })
    }

    async fn count_missing_form(
        &self,
        target: &str,
        profile: &str,
    ) -> Result<
        application::quran_counting::MissingFormReport,
        application::quran_lexicon_api::LexiconApiError,
    > {
        let freq = self.count_frequency(target, profile).await?;
        Ok(application::quran_counting::MissingFormReport {
            target: freq.target,
            rules: freq.rules,
            count: 0,
            disclaimer: application::quran_counting::MISSING_FORM_DISCLAIMER.to_string(),
        })
    }

    async fn count_near_duplicates(
        &self,
        _threshold: f64,
        _limit: usize,
    ) -> Result<
        (
            application::quran_counting::CountingRules,
            Vec<application::quran_counting::NearDuplicateHit>,
        ),
        application::quran_lexicon_api::LexiconApiError,
    > {
        use application::quran_counting::{CountingRules, MultiAnalysisHandling};
        Ok((
            CountingRules {
                profile: "L6.skeleton".to_string(),
                profile_version: "1.0.0".to_string(),
                datasets: vec!["stored-skeletons".to_string()],
                multi_analysis_handling: MultiAnalysisHandling::SingleSource,
                window: None,
                exclusions: vec![],
            },
            vec![],
        ))
    }

    async fn count_interval(
        &self,
        target: &str,
        profile: &str,
    ) -> Result<
        application::quran_counting::OccurrenceSpan,
        application::quran_lexicon_api::LexiconApiError,
    > {
        self.count_occurrences(target, profile).await
    }

    async fn count_unusual_usage(
        &self,
        target: &str,
        profile: &str,
    ) -> Result<
        application::quran_counting::NumericReport,
        application::quran_lexicon_api::LexiconApiError,
    > {
        self.count_numeric_report(target, profile).await
    }
}

fn test_state() -> AppState {
    AppState {
        tools: Arc::new(ToolRegistry::new(Arc::new(FakeBackend))),
        api: Arc::new(FakeApi),
        search: Arc::new(FakeSearch),
        lexicon: Arc::new(FakeLexicon),
        graph: Arc::new(DisabledGraph),
    }
}

/// Graph backend stub for the API contract tests: the graph routes are
/// pinned by the dedicated `tests/graph.rs` suite, so this state refuses
/// every graph read with a typed rejection (never a panic, never empty).
struct DisabledGraph;

#[async_trait::async_trait]
impl application::quran_graph_api::GraphBackend for DisabledGraph {
    async fn neighbors(
        &self,
        _args: application::quran_graph_api::NeighborsArgs,
    ) -> Result<
        application::quran_graph_api::NeighborsOutput,
        application::quran_graph_api::GraphApiError,
    > {
        Err(application::quran_graph_api::GraphApiError::rejected(
            "graph backend disabled in API contract tests",
        ))
    }

    async fn reachability(
        &self,
        _args: application::quran_graph_api::PathArgs,
    ) -> Result<
        application::quran_graph_api::ReachabilityOutput,
        application::quran_graph_api::GraphApiError,
    > {
        Err(application::quran_graph_api::GraphApiError::rejected(
            "graph backend disabled in API contract tests",
        ))
    }

    async fn shortest_path(
        &self,
        _args: application::quran_graph_api::PathArgs,
    ) -> Result<
        application::quran_graph_api::ShortestOutput,
        application::quran_graph_api::GraphApiError,
    > {
        Err(application::quran_graph_api::GraphApiError::rejected(
            "graph backend disabled in API contract tests",
        ))
    }

    async fn paths(
        &self,
        _args: application::quran_graph_api::PathsArgs,
    ) -> Result<
        application::quran_graph_api::PathsOutput,
        application::quran_graph_api::GraphApiError,
    > {
        Err(application::quran_graph_api::GraphApiError::rejected(
            "graph backend disabled in API contract tests",
        ))
    }

    async fn subgraph(
        &self,
        _args: application::quran_graph_api::SubgraphArgs,
    ) -> Result<
        application::quran_graph_api::SubgraphOutput,
        application::quran_graph_api::GraphApiError,
    > {
        Err(application::quran_graph_api::GraphApiError::rejected(
            "graph backend disabled in API contract tests",
        ))
    }

    async fn pattern(
        &self,
        _args: application::quran_graph_api::PatternArgs,
    ) -> Result<
        application::quran_graph_api::PatternOutput,
        application::quran_graph_api::GraphApiError,
    > {
        Err(application::quran_graph_api::GraphApiError::rejected(
            "graph backend disabled in API contract tests",
        ))
    }

    async fn root_family(
        &self,
        _args: application::quran_graph_api::RootFamilyArgs,
    ) -> Result<
        application::quran_graph_api::RootFamilyOutput,
        application::quran_graph_api::GraphApiError,
    > {
        Err(application::quran_graph_api::GraphApiError::rejected(
            "graph backend disabled in API contract tests",
        ))
    }

    async fn snapshot_meta(
        &self,
    ) -> Result<
        application::quran_graph_api::GraphSnapshotMeta,
        application::quran_graph_api::GraphApiError,
    > {
        Err(application::quran_graph_api::GraphApiError::rejected(
            "graph backend disabled in API contract tests",
        ))
    }
}

async fn serve_once() -> (String, tokio::task::JoinHandle<()>) {
    serve_router(router(test_state())).await
}

async fn serve_router(app: axum::Router) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
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
        "/api/v1/meta",
        "/api/v1/quran/editions",
        "/api/v1/quran/editions/{slug}",
        "/api/v1/quran/surahs",
        "/api/v1/quran/surahs/{number}",
        "/api/v1/quran/ayahs/{reference}",
        "/api/v1/quran/context/{reference}",
        "/api/v1/quran/tool/{name}",
        "/api/v1/quran/divisions/{kind}/{number}",
        "/api/v1/quran/tokens/{reference}",
        "/api/v1/quran/resolve",
        "/api/v1/quran/citations/{id}",
        "/api/v1/quran/normalization/preview",
        "/api/v1/quran/normalization/profiles",
        "/api/v1/quran/search/exact",
        "/api/v1/quran/search/normalized",
        "/api/v1/quran/search/phrase",
        "/api/v1/quran/search/concatenated",
        "/api/v1/quran/search/regex",
        "/api/v1/quran/graph/neighbors",
        "/api/v1/quran/graph/path",
        "/api/v1/quran/graph/subgraph",
        "/api/v1/quran/graph/pattern",
        "/api/v1/quran/graph/root-family",
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
        "research_checksum",
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
        "research_checksum",
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
async fn server_rejects_non_loopback_before_binding() {
    for addr in ["0.0.0.0:0", "[::]:0", "192.168.1.10:0"] {
        let result = server::api::serve(addr, test_state()).await;
        assert!(matches!(result, Err(server::ServerError::NonLoopback(_))), "{addr}");
    }
    for addr in ["localhost:invalid", "127.0.0.1.example:8737"] {
        assert!(matches!(
            server::api::serve(addr, test_state()).await,
            Err(server::ServerError::InvalidAddr(_))
        ));
    }
}

#[tokio::test]
async fn metadata_endpoint_returns_only_public_build_information() {
    let (addr, handle) = serve_once().await;
    let (status, headers, body) = get(&addr, "/api/v1/meta", &[]).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers.get("content-type").unwrap(), "application/json; charset=utf-8");
    let value = body_json(&body);
    assert_envelope(&value);
    assert_eq!(value["api_version"], "v1");
    assert_eq!(
        value["data"],
        serde_json::json!({
            "name": "Q-ai",
            "version": env!("CARGO_PKG_VERSION"),
        })
    );
    assert!(headers.get("etag").is_none());
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
async fn get_ayah_research_checksum_matches_service() {
    use tool_registry::GetAyahParams;
    let (addr, handle) = serve_once().await;
    let (status, _, body) = get(&addr, "/api/v1/quran/ayahs/1:1", &[]).await;
    assert_eq!(status, StatusCode::OK);
    let value = body_json(&body);
    let http_checksum =
        value["meta"]["research_checksum"].as_str().expect("meta.research_checksum is a string");
    assert!(!http_checksum.is_empty(), "meta.research_checksum must be non-empty");
    assert!(
        http_checksum.starts_with("sha256:"),
        "checksum renders as sha256:<hex>: {http_checksum}"
    );
    // The HTTP leg projects the typed ToolResult checksum (T-05-02: single
    // source, never recomputed in the HTTP layer) — so it equals the
    // checksum the service returns for the same call.
    let registry = ToolRegistry::new(Arc::new(FakeBackend));
    let (result, _) = registry
        .get_ayah(GetAyahParams {
            reference: "1:1".into(),
            translations: Vec::new(),
            glosses: false,
            tokens: false,
        })
        .await
        .unwrap();
    assert_eq!(http_checksum, format!("sha256:{}", result.research_checksum.hex));
    handle.abort();
}

#[tokio::test]
async fn tool_route_serves_registered_tools_with_matching_checksum() {
    use tool_registry::GetAyahParams;
    let (addr, handle) = serve_once().await;
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/tool/quran.get_ayah",
        &serde_json::json!({"reference": "1:1"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let value = body_json(&body);
    // The payload is the registry-produced ToolResult verbatim, so its
    // checksum rides in both `data` and `meta` with the same value.
    let data_hex =
        value["data"]["research_checksum"]["hex"].as_str().expect("data carries the checksum");
    assert!(!data_hex.is_empty());
    assert_eq!(value["meta"]["research_checksum"], format!("sha256:{data_hex}"));
    // …and it equals the checksum the same registry call produces.
    let registry = ToolRegistry::new(Arc::new(FakeBackend));
    let (result, _) = registry
        .get_ayah(GetAyahParams {
            reference: "1:1".into(),
            translations: Vec::new(),
            glosses: false,
            tokens: false,
        })
        .await
        .unwrap();
    assert_eq!(data_hex, result.research_checksum.hex);
    // Unknown tool names are typed 4xx errors, never a 200 and never a panic.
    let (status, _) =
        post_json(&addr, "/api/v1/quran/tool/quran.nope", &serde_json::json!({})).await;
    assert!(status.is_client_error(), "unknown tool must be 4xx, got {status}");
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
async fn stored_citation_hard_failure_is_not_a_success_envelope() {
    let (addr, handle) = serve_once().await;
    // A persisted citation whose stored verdict is a hard failure must return a
    // typed error, never a 200 envelope that re-serves an untrusted verdict
    // (T-02-18).
    let (status, _, body) = get(&addr, "/api/v1/quran/citations/cit-mismatch", &[]).await;
    assert_ne!(status, StatusCode::OK, "a hard-failing stored verdict must not be 200");
    let value = body_json(&body);
    assert_eq!(value["error"]["code"], "QAI-QUR-0323");
    assert!(value.get("data").is_none(), "no success envelope for a failed verdict");
    handle.abort();
}

#[tokio::test]
async fn debug_reader_without_font_declares_local_only_stack() {
    let (addr, handle) = serve_once().await;
    let (_, headers, body) = get(&addr, "/debug/read/test/1", &[]).await;
    assert_eq!(headers.get("content-type").unwrap(), "text/html; charset=utf-8");
    let text = String::from_utf8_lossy(&body).into_owned();
    assert!(text.contains("font-family"));
    // OD-04 system stack: locally installed fonts only, nothing bundled.
    assert!(text.contains("\"Amiri Quran\""));
    assert!(text.contains("KFGQPC Uthmanic Script HAFS"));
    assert!(!text.contains("@font-face"), "no @font-face unless a font is wired");
    assert!(!text.contains("http"), "no remote font fetch from the debug reader");
    handle.abort();
}

#[tokio::test]
async fn debug_reader_uses_local_font_asset_when_wired() {
    let font: Vec<u8> = vec![0x77, 0x4F, 0x46, 0x32, 0x00, 0x01, 0x02, 0x03];
    let app = server::api::router_with_debug_font(test_state(), font.clone());
    let (addr, handle) = serve_router(app).await;
    let (_, headers, body) = get(&addr, "/debug/assets/reader.woff2", &[]).await;
    assert_eq!(headers.get("content-type").unwrap(), "font/woff2");
    assert_eq!(headers.get("cache-control").unwrap(), "no-store");
    assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
    assert_eq!(body, font);
    let (_, _, body) = get(&addr, "/debug/read/test/1", &[]).await;
    let text = String::from_utf8_lossy(&body).into_owned();
    assert!(text.contains("@font-face"));
    assert!(text.contains("/debug/assets/reader.woff2"));
    assert!(text.contains("font-family:\"Qai Debug Reader\""));
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
    request.push_str(&format!("\r\n{payload}"));
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).await.unwrap();
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    let status_line = head.lines().next().unwrap_or("");
    let status: u16 = status_line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
    let mut headers = HeaderMap::new();
    for line in head.lines().skip(1) {
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

#[tokio::test]
async fn search_exact_returns_envelope_with_reproducibility() {
    let (addr, handle) = serve_once().await;
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/search/exact",
        &serde_json::json!({"text": "بسم", "limit": 5}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let value = body_json(&body);
    assert_envelope(&value);
    assert_eq!(value["data"]["total_matches"], 0);
    assert_eq!(value["data"]["truncated"], false);
    assert_eq!(value["data"]["rule_set"], "L0.exact@1.0.0");
    assert_eq!(value["meta"]["reproducibility"]["tool"], "quran.search_exact");
    assert_eq!(value["meta"]["reproducibility"]["rule_set"], "L0.exact@1.0.0");
    assert_eq!(value["meta"]["reproducibility"]["generation"], 1);
    handle.abort();
}

#[tokio::test]
async fn search_endpoints_reject_bad_values_with_coded_errors() {
    let (addr, handle) = serve_once().await;
    // Unknown match mode is a 400 with the IDX namespace.
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/search/normalized",
        &serde_json::json!({"text": "x", "match_mode": "nope"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body_json(&body)["error"]["code"], "QAI-IDX-0002");

    // Profile and rules together are rejected, never silently preferred.
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/search/normalized",
        &serde_json::json!({"text": "x", "profile": "L3.diacritics", "rules": "N01"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body_json(&body)["error"]["code"], "QAI-IDX-0002");

    // Empty query text is a 400, never a match-everything.
    let (status, _) =
        post_json(&addr, "/api/v1/quran/search/phrase", &serde_json::json!({"text": "  "})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Unknown phrase mode is a 400.
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/search/phrase",
        &serde_json::json!({"text": "x", "phrase_mode": "diagonal"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body_json(&body)["error"]["code"], "QAI-IDX-0002");
    handle.abort();
}

#[tokio::test]
async fn search_regex_reports_provenance_and_guards() {
    let (addr, handle) = serve_once().await;
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/search/regex",
        &serde_json::json!({"pattern": "^ا?ل?رحم", "field": "text_bare"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let value = body_json(&body);
    assert_envelope(&value);
    assert_eq!(value["data"]["regex_report"]["pattern"], "^ا?ل?رحم");
    assert_eq!(value["data"]["regex_report"]["field"], "text_bare");
    assert_eq!(value["meta"]["reproducibility"]["tool"], "quran.search_regex");

    // Guard rejection is a coded 400, not a silent empty result.
    let (status, body) =
        post_json(&addr, "/api/v1/quran/search/regex", &serde_json::json!({"pattern": ".*x"}))
            .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body_json(&body)["error"]["code"], "QAI-IDX-0002");
    handle.abort();
}

#[tokio::test]
async fn search_sse_streams_hits_then_terminal_totals() {
    let (addr, handle) = serve_once().await;
    let (status, headers, body) = post_raw(
        &addr,
        "/api/v1/quran/search/concatenated",
        &serde_json::json!({"text": "بسمالله"}),
        &[("accept", "text/event-stream")],
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers.get("content-type").unwrap(), "text/event-stream");
    let text = String::from_utf8_lossy(&body).into_owned();
    assert!(text.contains("event: totals"), "terminal totals event missing: {text}");
    let totals = text.split("event: totals\ndata: ").nth(1).expect("totals event carries data");
    let totals: serde_json::Value = serde_json::from_str(totals.trim()).unwrap();
    assert_eq!(totals["total_matches"], 0);
    assert_eq!(totals["rule_set"], "L6.skeleton@1.0.0");

    // Without the SSE accept header the same endpoint serves JSON.
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/search/concatenated",
        &serde_json::json!({"text": "بسمالله"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_envelope(&body_json(&body));
    handle.abort();
}

// ─── Lexicon routes (SC3/SC4, G-02/G-10, D-10/D-13) ────────────────────

#[tokio::test]
async fn lexicon_family_route_returns_attributed_relations() {
    let (addr, handle) = serve_once().await;
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/family",
        &serde_json::json!({"kind": "token", "id": "token:1:1:2"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let value = body_json(&body);
    assert_envelope(&value);
    // SC3: typed relations with a mandatory explanation AND dataset
    // attribution on every member (T-03-19).
    let relations = value["data"].as_array().expect("family relations array");
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0]["relation"], "same_root");
    assert_eq!(relations[0]["kind"], "token");
    assert_eq!(relations[0]["dataset"], "test-morph@0.1.0");
    assert!(
        !relations[0]["explanation"].as_str().unwrap_or_default().trim().is_empty(),
        "explanation is mandatory: {}",
        relations[0]
    );
    assert_eq!(value["meta"]["reproducibility"]["tool"], "quran.word_family");
    handle.abort();
}

#[tokio::test]
async fn lexicon_family_route_never_returns_an_empty_envelope() {
    let (addr, handle) = serve_once().await;
    // No active dataset is a typed capability error (QAI-MORPH-0004's HTTP
    // analogue), never a 200 with an empty relation list (T-03-21).
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/family",
        &serde_json::json!({"kind": "token", "id": "token:1:1:1"}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let value = body_json(&body);
    assert_eq!(value["error"]["code"], "QAI-LEX-0004");
    assert!(value.get("data").is_none(), "an unavailable capability carries no data: {value}");

    // An empty selector is a coded 400 before any storage read (T-03-18).
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/family",
        &serde_json::json!({"kind": "  ", "id": "token:1:1:2"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body_json(&body)["error"]["code"], "QAI-LEX-0002");
    handle.abort();
}

#[tokio::test]
async fn lexicon_count_routes_return_the_counting_rules_block() {
    let (addr, handle) = serve_once().await;
    // SC4 at the HTTP level: the report carries the CountingRules block
    // (dataset attribution + the requested multi-analysis handling) and a
    // reproducibility checksum, never a bare number.
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/count/root-frequency",
        &serde_json::json!({"root": "r-1", "mode": "all-analyses"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let value = body_json(&body);
    assert_envelope(&value);
    assert_eq!(value["data"]["rules"]["datasets"][0], "test-morph@0.1.0");
    assert_eq!(value["data"]["rules"]["multi_analysis_handling"], "AllAnalyses");
    assert_eq!(value["data"]["rules"]["profile"], "L3.diacritics");
    assert_eq!(value["data"]["count"], 64);
    assert!(!value["data"]["checksum"].as_str().unwrap_or_default().is_empty());
    assert_eq!(value["meta"]["reproducibility"]["tool"], "quran.count.root_frequency");

    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/count/lemma-frequency",
        &serde_json::json!({"lemma": "lem-1"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let value = body_json(&body);
    assert_eq!(value["data"]["rules"]["datasets"][0], "test-morph@0.1.0");
    // The default mode is the CLI's default (single-source) convention.
    assert_eq!(value["data"]["rules"]["multi_analysis_handling"], "SingleSource");
    assert_eq!(value["data"]["count"], 127);

    // An unknown mode is a coded 400 from the shared CLI/API parser.
    let (status, body) = post_json(
        &addr,
        "/api/v1/quran/count/root-frequency",
        &serde_json::json!({"root": "r-1", "mode": "sideways"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body_json(&body)["error"]["code"], "QAI-LEX-0002");
    handle.abort();
}

#[tokio::test]
async fn lexicon_count_routes_are_typed_when_the_dataset_is_unavailable() {
    let (addr, handle) = serve_once().await;
    for path in ["/api/v1/quran/count/root-frequency", "/api/v1/quran/count/lemma-frequency"] {
        let body = if path.ends_with("root-frequency") {
            serde_json::json!({"root": "tst-gone"})
        } else {
            serde_json::json!({"lemma": "tst-gone"})
        };
        let (status, body) = post_json(&addr, path, &body).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path} must not answer 200 without a dataset");
        let value = body_json(&body);
        assert_eq!(value["error"]["code"], "QAI-LEX-0004", "{path}");
        assert!(value.get("data").is_none(), "{path}: no empty 200 envelope");
    }
    handle.abort();
}
