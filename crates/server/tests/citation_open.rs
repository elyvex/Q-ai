//! 05-08 citation-open gate (D-17/ADR-0111): opening a deep link
//! re-verifies server-side; a mismatch hard-fails (typed error, never 200).

use std::sync::Arc;

use server::api::{AppState, QuranApiBackend, router};
use tool_registry::{BackendMeta, QuranBackend, ToolRegistry};

struct FakeReaderBackend;

#[async_trait::async_trait]
impl QuranBackend for FakeReaderBackend {
    async fn backend_get_ayah(
        &self,
        _reference: &quran_core::QuranRef,
        _options: &quran_core::AyahOptions,
    ) -> Result<(quran_core::AyahView, BackendMeta), tools::ToolError> {
        unimplemented!()
    }
    async fn backend_get_context(
        &self,
        _reference: &quran_core::QuranRef,
        _spec: &quran_core::ContextSpec,
    ) -> Result<(quran_core::ContextView, BackendMeta), tools::ToolError> {
        unimplemented!()
    }
}

struct FakeApi;

#[async_trait::async_trait]
impl QuranApiBackend for FakeApi {
    async fn api_editions(&self) -> Result<Vec<serde_json::Value>, tools::ToolError> {
        unimplemented!()
    }
    async fn api_edition(&self, _slug: &str) -> Result<serde_json::Value, tools::ToolError> {
        unimplemented!()
    }
    async fn api_surahs(
        &self,
        _edition: &str,
    ) -> Result<Vec<quran_core::Surah>, tools::ToolError> {
        unimplemented!()
    }
    async fn api_surah(
        &self,
        _edition: &str,
        _surah: u16,
        _options: &quran_core::AyahOptions,
    ) -> Result<(quran_core::Surah, Vec<quran_core::AyahView>), tools::ToolError> {
        unimplemented!()
    }
    async fn api_division(
        &self,
        _edition: &str,
        _kind: &str,
        _number: u32,
        _options: &quran_core::AyahOptions,
    ) -> Result<Vec<quran_core::AyahView>, tools::ToolError> {
        unimplemented!()
    }
    async fn api_tokens(
        &self,
        _reference: &str,
    ) -> Result<Vec<quran_core::Token>, tools::ToolError> {
        unimplemented!()
    }
    async fn api_citation(&self, id: &str) -> Result<citations::ResolvedCitation, tools::ToolError> {
        match id {
            "cite-ok" => Ok(citations::ResolvedCitation {
                citation_id: id.to_string(),
                verdict: citations::QuotationVerdict::ExactMatch,
                text_hash: Some("sha256:00".to_string()),
                deep_link: Some(citations::deep_link("test", "0.1.0", 1, 1)),
                urn: Some(citations::citation_urn("test", "0.1.0", 1, 1)),
            }),
            "cite-bad" => Ok(citations::ResolvedCitation {
                citation_id: id.to_string(),
                verdict: citations::QuotationVerdict::Mismatch {
                    first_difference_at: 0,
                    expected_hash: "00".to_string(),
                },
                text_hash: None,
                deep_link: None,
                urn: None,
            }),
            _ => Err(tools::ToolError::Backend {
                code: "QAI-QUR-0322".to_string(),
                detail: format!("citation `{id}` not found"),
            }),
        }
    }
}

fn state() -> AppState {
    let reader = Arc::new(FakeReaderBackend);
    AppState {
        tools: Arc::new(ToolRegistry::new(reader)),
        api: Arc::new(FakeApi),
        search: Arc::new(StubSearch),
        lexicon: Arc::new(StubLexicon),
        graph: Arc::new(StubGraph),
    }
}

async fn get(path: &str) -> (axum::http::StatusCode, serde_json::Value) {
    use tower::ServiceExt as _;
    let request = axum::http::Request::builder()
        .uri(path)
        .body(axum::body::Body::empty())
        .unwrap();
    let response = router(state()).oneshot(request).await.unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();
    (status, value)
}

#[tokio::test]
async fn open_verified_citation_returns_urn_and_deep_link() {
    let (status, body) = get("/api/v1/quran/citations/cite-ok").await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let data = &body["data"];
    assert_eq!(data["citation_id"], "cite-ok");
    // The URN comes from the frozen helper — never hand-rolled client-side.
    assert_eq!(data["urn"], citations::citation_urn("test", "0.1.0", 1, 1));
    assert_eq!(data["urn"], "qai://quran/test@0.1.0/1:1");
    assert_eq!(data["deep_link"], "/read/test@0.1.0/1:1");
}

#[tokio::test]
async fn open_tampered_citation_hard_fails() {
    let (status, body) = get("/api/v1/quran/citations/cite-bad").await;
    assert_ne!(status, axum::http::StatusCode::OK, "mismatch is a hard failure, never 200");
    assert!(body.get("error").is_some(), "typed error body, got {body}");
}

#[tokio::test]
async fn open_missing_citation_is_not_found() {
    let (status, _) = get("/api/v1/quran/citations/cite-gone").await;
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
}

use application::{quran_graph_api::*, quran_lexicon_api::*, quran_search_api::*};
use application::{quran_counting::FrequencyReport, quran_morphology::FamilyMemberView, quran_search::{SearchError, SearchOutput}};
struct StubSearch;
#[async_trait::async_trait]
impl application::quran_search_api::SearchBackend for StubSearch {
    async fn search_exact(&self, _args: ExactArgs) -> Result<SearchOutput, SearchError> { unimplemented!() }
    async fn search_normalized(&self, _args: NormalizedArgs) -> Result<SearchOutput, SearchError> { unimplemented!() }
    async fn search_phrase(&self, _args: PhraseArgs) -> Result<SearchOutput, SearchError> { unimplemented!() }
    async fn search_concatenated(&self, _args: ConcatenatedArgs) -> Result<SearchOutput, SearchError> { unimplemented!() }
    async fn search_regex(&self, _args: RegexArgs) -> Result<SearchOutput, SearchError> { unimplemented!() }
}

struct StubLexicon;
#[async_trait::async_trait]
impl application::quran_lexicon_api::LexiconBackend for StubLexicon {
    async fn word_family(&self, _args: FamilyArgs) -> Result<Vec<FamilyMemberView>, LexiconApiError> { unimplemented!() }
    async fn root_frequency(&self, _args: RootFrequencyArgs) -> Result<FrequencyReport, LexiconApiError> { unimplemented!() }
    async fn lemma_frequency(&self, _args: LemmaFrequencyArgs) -> Result<FrequencyReport, LexiconApiError> { unimplemented!() }
    async fn count_frequency(&self, _target: &str, _profile: &str) -> Result<FrequencyReport, LexiconApiError> { unimplemented!() }
    async fn count_distribution(&self, _target: &str, _profile: &str) -> Result<application::quran_counting::DistributionReport, LexiconApiError> { unimplemented!() }
    async fn count_occurrences(&self, _target: &str, _profile: &str) -> Result<application::quran_counting::OccurrenceSpan, LexiconApiError> { unimplemented!() }
    async fn count_hapax(&self, _profile: &str, _limit: usize) -> Result<application::quran_counting::HapaxReport, LexiconApiError> { unimplemented!() }
    async fn count_cooccurrence(&self, _target: &str, _profile: &str, _window: usize, _limit: usize) -> Result<(application::quran_counting::CountingRules, Vec<application::quran_counting::CooccurrenceHit>), LexiconApiError> { unimplemented!() }
    async fn count_collocation(&self, _target: &str, _profile: &str, _window: usize, _limit: usize) -> Result<(application::quran_counting::CountingRules, Vec<application::quran_counting::CollocationHit>), LexiconApiError> { unimplemented!() }
    async fn count_numeric_report(&self, _target: &str, _profile: &str) -> Result<application::quran_counting::NumericReport, LexiconApiError> { unimplemented!() }
    async fn count_missing_form(&self, _target: &str, _profile: &str) -> Result<application::quran_counting::MissingFormReport, LexiconApiError> { unimplemented!() }
    async fn count_near_duplicates(&self, _threshold: f64, _limit: usize) -> Result<(application::quran_counting::CountingRules, Vec<application::quran_counting::NearDuplicateHit>), LexiconApiError> { unimplemented!() }
    async fn count_interval(&self, _target: &str, _profile: &str) -> Result<application::quran_counting::OccurrenceSpan, LexiconApiError> { unimplemented!() }
    async fn count_unusual_usage(&self, _target: &str, _profile: &str) -> Result<application::quran_counting::NumericReport, LexiconApiError> { unimplemented!() }
}

struct StubGraph;
#[async_trait::async_trait]
impl application::quran_graph_api::GraphBackend for StubGraph {
    async fn neighbors(&self, _args: NeighborsArgs) -> Result<NeighborsOutput, GraphApiError> { unimplemented!() }
    async fn reachability(&self, _args: PathArgs) -> Result<ReachabilityOutput, GraphApiError> { unimplemented!() }
    async fn shortest_path(&self, _args: PathArgs) -> Result<ShortestOutput, GraphApiError> { unimplemented!() }
    async fn paths(&self, _args: PathsArgs) -> Result<PathsOutput, GraphApiError> { unimplemented!() }
    async fn subgraph(&self, _args: SubgraphArgs) -> Result<SubgraphOutput, GraphApiError> { unimplemented!() }
    async fn pattern(&self, _args: PatternArgs) -> Result<PatternOutput, GraphApiError> { unimplemented!() }
    async fn root_family(&self, _args: RootFamilyArgs) -> Result<RootFamilyOutput, GraphApiError> { unimplemented!() }
    async fn snapshot_meta(&self) -> Result<GraphSnapshotMeta, GraphApiError> { unimplemented!() }
}
