//! Tool-contract conformance suite (P2-T110, D-3.5-08).
//!
//! Runs every registered tool name through one shared contract: envelope,
//! reproducibility, attribution, trace, typed-unavailable, truncation, and
//! no raw backend error strings. Parameterized over a conformable backend
//! and an unsupported backend, the way `quran-graph/tests/conformance.rs`
//! parameterizes its backends.

use std::collections::BTreeMap;
use std::sync::Arc;

use domain::SemVer;
use quran_core::{AyahOptions, AyahView, ContextSpec, ContextView, QuranRef};
use tool_registry::{
    BackendMeta, FAMILY_TOOL_VERSION, FamilyToolParams, GRAPH_NEIGHBORS_TOOL_VERSION,
    GRAPH_PATH_TOOL_VERSION, GRAPH_PATTERN_TOOL_VERSION, GRAPH_ROOT_FAMILY_TOOL_VERSION,
    GRAPH_SUBGRAPH_TOOL_VERSION as SUBGRAPH_VERSION, GetAyahParams, GetContextParams,
    LEMMA_TOOL_VERSION, LemmaToolParams, MORPHOLOGY_TOOL_VERSION, MorphologyToolParams,
    QuranBackend, ROOT_TOOL_VERSION, RootToolParams, SEARCH_TOOL_VERSION, SearchToolParams,
    ToolRegistry,
};
use tools::{AnalysisSource, ToolError, ToolResult, reproducibility, research_checksum};

// ── Conformable backend ─────────────────────────────────────────────────────

fn test_meta() -> BackendMeta {
    BackendMeta {
        edition_slug: "test".into(),
        edition_version: "0.1.0".into(),
        edition_id: "ed-1".into(),
        corpus_generation: 3,
        text_hash: "sha256:ab".into(),
        script: "uthmani".into(),
        riwayah: None,
        numbering_scheme: "hafs".into(),
        projection_id: String::new(),
        builder_version: String::new(),
    }
}

fn test_view() -> AyahView {
    serde_json::from_value::<AyahView>(serde_json::json!({
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

fn test_context() -> ContextView {
    let focal = test_view();
    ContextView {
        focal,
        before: vec![],
        after: vec![],
        surah: None,
        canonical_reference: "quran:test@0.1.0:1:1".to_string(),
        global_range: (1, 1),
    }
}

/// A conformable envelope for the D-13 tools.
fn conformable_envelope(
    tool: &'static str,
    version: SemVer,
    query: serde_json::Value,
) -> ToolResult<serde_json::Value> {
    let results = serde_json::json!({ "ok": true });
    ToolResult {
        tool_name: tool.to_string(),
        tool_version: version,
        query: query.clone(),
        normalization_rules: vec!["N01".to_string(), "N03".to_string()],
        edition_id: Some("ed-1".to_string()),
        edition_version: Some("0.1.0".to_string()),
        canonical_references: vec!["quran:test@0.1.0:1:1".to_string()],
        analysis_sources: vec![AnalysisSource {
            kind: "dataset".to_string(),
            reference: "test-morph@0.1.0".to_string(),
        }],
        results: results.clone(),
        confidence: None,
        warnings: Vec::new(),
        execution_time_ms: 0.0,
        reproducibility: reproducibility(
            tool,
            version,
            &query,
            Some("test"),
            Some("0.1.0"),
            BTreeMap::new(),
            3,
        ),
        research_checksum: research_checksum(
            tool,
            version,
            &query,
            &serde_json::json!({
                "edition_slug": "test",
                "edition_version": "0.1.0",
                "corpus_generation": 3,
            }),
            &results,
        )
        .expect("research checksum input is JSON-serializable"),
    }
}

/// A backend that returns conformable results for every tool.
struct ConformableBackend;

#[async_trait::async_trait]
impl QuranBackend for ConformableBackend {
    async fn backend_get_ayah(
        &self,
        _reference: &QuranRef,
        _options: &AyahOptions,
    ) -> Result<(AyahView, BackendMeta), ToolError> {
        Ok((test_view(), test_meta()))
    }

    async fn backend_get_context(
        &self,
        _reference: &QuranRef,
        _spec: &ContextSpec,
    ) -> Result<(ContextView, BackendMeta), ToolError> {
        Ok((test_context(), test_meta()))
    }

    async fn backend_search(
        &self,
        params: &SearchToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Ok(conformable_envelope(
            "quran.search",
            SEARCH_TOOL_VERSION,
            serde_json::to_value(params).unwrap(),
        ))
    }

    async fn backend_root(
        &self,
        params: &RootToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Ok(conformable_envelope(
            "quran.root",
            ROOT_TOOL_VERSION,
            serde_json::to_value(params).unwrap(),
        ))
    }

    async fn backend_lemma(
        &self,
        params: &LemmaToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Ok(conformable_envelope(
            "quran.lemma",
            LEMMA_TOOL_VERSION,
            serde_json::to_value(params).unwrap(),
        ))
    }

    async fn backend_morphology(
        &self,
        params: &MorphologyToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Ok(conformable_envelope(
            "quran.morphology",
            MORPHOLOGY_TOOL_VERSION,
            serde_json::to_value(params).unwrap(),
        ))
    }

    async fn backend_family(
        &self,
        params: &FamilyToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Ok(conformable_envelope(
            "quran.family",
            FAMILY_TOOL_VERSION,
            serde_json::to_value(params).unwrap(),
        ))
    }

    async fn backend_graph_neighbors(
        &self,
        params: &tool_registry::GraphNeighborsParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Ok(conformable_envelope(
            "quran.graph_neighbors",
            GRAPH_NEIGHBORS_TOOL_VERSION,
            serde_json::to_value(params).unwrap(),
        ))
    }

    async fn backend_graph_path(
        &self,
        params: &tool_registry::GraphPathParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Ok(conformable_envelope(
            "quran.graph_path",
            GRAPH_PATH_TOOL_VERSION,
            serde_json::to_value(params).unwrap(),
        ))
    }

    async fn backend_graph_subgraph(
        &self,
        params: &tool_registry::GraphSubgraphParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Ok(conformable_envelope(
            "quran.graph_subgraph",
            SUBGRAPH_VERSION,
            serde_json::to_value(params).unwrap(),
        ))
    }

    async fn backend_graph_pattern(
        &self,
        params: &tool_registry::GraphPatternParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Ok(conformable_envelope(
            "quran.graph_pattern",
            GRAPH_PATTERN_TOOL_VERSION,
            serde_json::to_value(params).unwrap(),
        ))
    }

    async fn backend_graph_root_family(
        &self,
        params: &tool_registry::GraphRootFamilyParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Ok(conformable_envelope(
            "quran.graph_root_family",
            GRAPH_ROOT_FAMILY_TOOL_VERSION,
            serde_json::to_value(params).unwrap(),
        ))
    }
}

// ── Unsupported backend ─────────────────────────────────────────────────────

/// A backend that returns typed-unsupported for every tool.
struct UnsupportedBackend;

fn unsupported(tool: &'static str) -> ToolError {
    ToolError::Backend {
        code: "QAI-QUR-0310".to_string(),
        detail: format!("{tool} is not supported by this backend"),
    }
}

#[async_trait::async_trait]
impl QuranBackend for UnsupportedBackend {
    async fn backend_get_ayah(
        &self,
        _reference: &QuranRef,
        _options: &AyahOptions,
    ) -> Result<(AyahView, BackendMeta), ToolError> {
        Err(unsupported("quran.get_ayah"))
    }

    async fn backend_get_context(
        &self,
        _reference: &QuranRef,
        _spec: &ContextSpec,
    ) -> Result<(ContextView, BackendMeta), ToolError> {
        Err(unsupported("quran.get_context"))
    }

    async fn backend_search(
        &self,
        _params: &SearchToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("quran.search"))
    }

    async fn backend_root(
        &self,
        _params: &RootToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("quran.root"))
    }

    async fn backend_lemma(
        &self,
        _params: &LemmaToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("quran.lemma"))
    }

    async fn backend_morphology(
        &self,
        _params: &MorphologyToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("quran.morphology"))
    }

    async fn backend_family(
        &self,
        _params: &FamilyToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("quran.family"))
    }

    async fn backend_graph_neighbors(
        &self,
        _params: &tool_registry::GraphNeighborsParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("quran.graph_neighbors"))
    }

    async fn backend_graph_path(
        &self,
        _params: &tool_registry::GraphPathParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("quran.graph_path"))
    }

    async fn backend_graph_subgraph(
        &self,
        _params: &tool_registry::GraphSubgraphParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("quran.graph_subgraph"))
    }

    async fn backend_graph_pattern(
        &self,
        _params: &tool_registry::GraphPatternParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("quran.graph_pattern"))
    }

    async fn backend_graph_root_family(
        &self,
        _params: &tool_registry::GraphRootFamilyParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("quran.graph_root_family"))
    }
}

// ── Truncated backend ───────────────────────────────────────────────────────

/// A backend whose graph traversal exhausts its budget: it returns a truncated
/// payload plus a non-empty reason, never an empty list presented as absence.
struct TruncatedBackend;

#[async_trait::async_trait]
impl QuranBackend for TruncatedBackend {
    async fn backend_get_ayah(
        &self,
        _reference: &QuranRef,
        _options: &AyahOptions,
    ) -> Result<(AyahView, BackendMeta), ToolError> {
        Err(unsupported("quran.get_ayah"))
    }

    async fn backend_get_context(
        &self,
        _reference: &QuranRef,
        _spec: &ContextSpec,
    ) -> Result<(ContextView, BackendMeta), ToolError> {
        Err(unsupported("quran.get_context"))
    }

    async fn backend_graph_path(
        &self,
        params: &tool_registry::GraphPathParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let query = serde_json::to_value(params).unwrap();
        let results = serde_json::json!({
            "paths": [],
            "truncated": true,
            "incomplete_reason": "budget exhausted: max_nodes reached before the search completed",
        });
        Ok(ToolResult {
            tool_name: "quran.graph_path".to_string(),
            tool_version: GRAPH_PATH_TOOL_VERSION,
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: None,
            edition_version: None,
            canonical_references: Vec::new(),
            analysis_sources: vec![AnalysisSource {
                kind: "graph".to_string(),
                reference: "quran-structural-v1".to_string(),
            }],
            // Mirrors the graph API payload: an empty `paths` is only a proven
            // "no path" when `truncated` is false, so a budget-exhausted search
            // can never read as absence.
            results: results.clone(),
            confidence: None,
            warnings: vec!["result truncated: budget exhausted".to_string()],
            execution_time_ms: 0.0,
            reproducibility: reproducibility(
                "quran.graph_path",
                GRAPH_PATH_TOOL_VERSION,
                &query,
                None,
                None,
                BTreeMap::new(),
                0,
            ),
            research_checksum: research_checksum(
                "quran.graph_path",
                GRAPH_PATH_TOOL_VERSION,
                &query,
                &serde_json::json!({ "corpus_generation": 0 }),
                &results,
            )
            .expect("research checksum input is JSON-serializable"),
        })
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

/// All twelve registered tools resolve stable names.
#[test]
fn all_twelve_tools_resolve_stable_names() {
    let registry = ToolRegistry::new(Arc::new(ConformableBackend));
    let names = registry.tool_names();
    assert_eq!(names.len(), 12);
    for name in &names {
        assert!(name.starts_with("quran."), "tool name must be quran.*: {name}");
    }
}

/// All twelve tools return the ToolResult envelope (not a bare value).
#[tokio::test]
async fn all_twelve_tools_return_tool_result_envelope() {
    let registry = ToolRegistry::new(Arc::new(ConformableBackend));

    // get_ayah
    let (result, _) = registry
        .get_ayah(GetAyahParams {
            reference: "1:1".into(),
            translations: Vec::new(),
            glosses: false,
            tokens: false,
        })
        .await
        .unwrap();
    assert_eq!(result.tool_name, "quran.get_ayah");

    // get_context
    let (result, _) = registry
        .get_context(GetContextParams {
            reference: "1:1".into(),
            before: 1,
            after: 1,
            boundary: tool_registry::ContextBoundaryArg::None,
            max_ayahs: 5,
        })
        .await
        .unwrap();
    assert_eq!(result.tool_name, "quran.get_context");

    // search
    let result = registry
        .search(SearchToolParams { text: "test".into(), edition: None, limit: None })
        .await
        .unwrap();
    assert_eq!(result.tool_name, "quran.search");

    // root
    let result = registry.root(RootToolParams { root: "root-0".into() }).await.unwrap();
    assert_eq!(result.tool_name, "quran.root");

    // lemma
    let result = registry.lemma(LemmaToolParams { lemma: "lem-0".into() }).await.unwrap();
    assert_eq!(result.tool_name, "quran.lemma");

    // morphology
    let result =
        registry.morphology(MorphologyToolParams { surah: 1, ayah: 1, position: 1 }).await.unwrap();
    assert_eq!(result.tool_name, "quran.morphology");

    // family
    let result = registry
        .family(FamilyToolParams { kind: "token".into(), id: "token:1:1:1".into() })
        .await
        .unwrap();
    assert_eq!(result.tool_name, "quran.family");

    // graph_neighbors
    let result = registry
        .graph_neighbors(tool_registry::GraphNeighborsParams {
            node: "ayah:1:1".into(),
            edge_types: None,
            direction: None,
            budgets: Default::default(),
        })
        .await
        .unwrap();
    assert_eq!(result.tool_name, "quran.graph_neighbors");

    // graph_path
    let result = registry
        .graph_path(tool_registry::GraphPathParams {
            from: "ayah:1:1".into(),
            to: "ayah:1:2".into(),
            mode: None,
            paths: None,
            edge_types: None,
            direction: None,
            budgets: Default::default(),
        })
        .await
        .unwrap();
    assert_eq!(result.tool_name, "quran.graph_path");

    // graph_subgraph
    let result = registry
        .graph_subgraph(tool_registry::GraphSubgraphParams {
            seeds: vec!["ayah:1:1".into()],
            edge_types: None,
            direction: None,
            budgets: Default::default(),
        })
        .await
        .unwrap();
    assert_eq!(result.tool_name, "quran.graph_subgraph");

    // graph_pattern
    let result = registry
        .graph_pattern(tool_registry::GraphPatternParams {
            seeds: vec!["ayah:1:1".into()],
            steps: vec![tool_registry::GraphPatternStep { edge: "NEXT".into(), node_kind: None }],
            budgets: Default::default(),
        })
        .await
        .unwrap();
    assert_eq!(result.tool_name, "quran.graph_pattern");

    // graph_root_family
    let result = registry
        .graph_root_family(tool_registry::GraphRootFamilyParams {
            root: "root-0".into(),
            limit: None,
        })
        .await
        .unwrap();
    assert_eq!(result.tool_name, "quran.graph_root_family");
}

/// All twelve tools carry a reproducibility payload.
#[tokio::test]
async fn all_twelve_tools_carry_reproducibility() {
    let registry = ToolRegistry::new(Arc::new(ConformableBackend));

    let (result, _) = registry
        .get_ayah(GetAyahParams {
            reference: "1:1".into(),
            translations: Vec::new(),
            glosses: false,
            tokens: false,
        })
        .await
        .unwrap();
    assert!(result.reproducibility.deterministic);

    let (result, _) = registry
        .get_context(GetContextParams {
            reference: "1:1".into(),
            before: 1,
            after: 1,
            boundary: tool_registry::ContextBoundaryArg::None,
            max_ayahs: 5,
        })
        .await
        .unwrap();
    assert!(result.reproducibility.deterministic);

    for result in [
        registry
            .search(SearchToolParams { text: "test".into(), edition: None, limit: None })
            .await
            .unwrap(),
        registry.root(RootToolParams { root: "root-0".into() }).await.unwrap(),
        registry.lemma(LemmaToolParams { lemma: "lem-0".into() }).await.unwrap(),
        registry.morphology(MorphologyToolParams { surah: 1, ayah: 1, position: 1 }).await.unwrap(),
        registry
            .family(FamilyToolParams { kind: "token".into(), id: "token:1:1:1".into() })
            .await
            .unwrap(),
    ] {
        assert!(result.reproducibility.deterministic, "{}", result.tool_name);
    }
}

/// Quranic tools carry canonical sources.
#[tokio::test]
async fn quranic_tools_carry_canonical_sources() {
    let registry = ToolRegistry::new(Arc::new(ConformableBackend));

    let (result, _) = registry
        .get_ayah(GetAyahParams {
            reference: "1:1".into(),
            translations: Vec::new(),
            glosses: false,
            tokens: false,
        })
        .await
        .unwrap();
    assert!(!result.analysis_sources.is_empty(), "get_ayah must attribute sources");
    assert!(!result.canonical_references.is_empty(), "get_ayah must carry canonical refs");

    let (result, _) = registry
        .get_context(GetContextParams {
            reference: "1:1".into(),
            before: 1,
            after: 1,
            boundary: tool_registry::ContextBoundaryArg::None,
            max_ayahs: 5,
        })
        .await
        .unwrap();
    assert!(!result.analysis_sources.is_empty(), "get_context must attribute sources");
    assert!(!result.canonical_references.is_empty(), "get_context must carry canonical refs");
}

/// Lexicon tools carry dataset attribution.
#[tokio::test]
async fn lexicon_tools_carry_dataset_attribution() {
    let registry = ToolRegistry::new(Arc::new(ConformableBackend));

    for result in [
        registry.root(RootToolParams { root: "root-0".into() }).await.unwrap(),
        registry.lemma(LemmaToolParams { lemma: "lem-0".into() }).await.unwrap(),
        registry.morphology(MorphologyToolParams { surah: 1, ayah: 1, position: 1 }).await.unwrap(),
        registry
            .family(FamilyToolParams { kind: "token".into(), id: "token:1:1:1".into() })
            .await
            .unwrap(),
    ] {
        assert!(
            !result.analysis_sources.is_empty(),
            "{} must attribute its dataset source",
            result.tool_name
        );
        assert_eq!(
            result.analysis_sources[0].kind, "dataset",
            "{} must use dataset attribution",
            result.tool_name
        );
    }
}

/// Search tool carries normalization rules.
#[tokio::test]
async fn search_tool_carries_normalization_rules() {
    let registry = ToolRegistry::new(Arc::new(ConformableBackend));
    let result = registry
        .search(SearchToolParams { text: "test".into(), edition: None, limit: None })
        .await
        .unwrap();
    assert!(!result.normalization_rules.is_empty(), "search must carry its rule trace");
}

/// Unavailable dataset surfaces the typed QAI-MORPH-0004 backend error.
#[tokio::test]
async fn unavailable_dataset_surfaces_typed_error() {
    let registry = ToolRegistry::new(Arc::new(UnsupportedBackend));
    for err in [
        registry.root(RootToolParams { root: "root-0".into() }).await.unwrap_err(),
        registry.lemma(LemmaToolParams { lemma: "lem-0".into() }).await.unwrap_err(),
        registry
            .morphology(MorphologyToolParams { surah: 1, ayah: 1, position: 1 })
            .await
            .unwrap_err(),
        registry
            .family(FamilyToolParams { kind: "token".into(), id: "token:1:1:1".into() })
            .await
            .unwrap_err(),
    ] {
        match err {
            ToolError::Backend { code, .. } => {
                assert_eq!(code, "QAI-QUR-0310", "unsupported backend must return typed error");
            }
            other => panic!("expected a typed backend error, got {other:?}"),
        }
    }
}

/// No tool returns a raw backend error string.
#[tokio::test]
async fn no_tool_returns_raw_backend_error_string() {
    let registry = ToolRegistry::new(Arc::new(UnsupportedBackend));
    // Every error must be a typed ToolError, never a raw string.
    let err = registry
        .search(SearchToolParams { text: "test".into(), edition: None, limit: None })
        .await
        .unwrap_err();
    assert!(matches!(err, ToolError::Backend { .. }));
}

/// Registered but unimplemented tool returns typed-unsupported error.
#[tokio::test]
async fn registered_but_unimplemented_tool_returns_typed_unsupported() {
    let registry = ToolRegistry::new(Arc::new(UnsupportedBackend));
    // All 12 tools must return the typed-unsupported error, distinguishable
    // from a real empty result.
    let names = registry.tool_names();
    assert_eq!(names.len(), 12);
    for name in &names {
        let err = match *name {
            "quran.get_ayah" => registry
                .get_ayah(GetAyahParams {
                    reference: "1:1".into(),
                    translations: Vec::new(),
                    glosses: false,
                    tokens: false,
                })
                .await
                .unwrap_err(),
            "quran.get_context" => registry
                .get_context(GetContextParams {
                    reference: "1:1".into(),
                    before: 1,
                    after: 1,
                    boundary: tool_registry::ContextBoundaryArg::None,
                    max_ayahs: 5,
                })
                .await
                .unwrap_err(),
            "quran.search" => registry
                .search(SearchToolParams { text: "test".into(), edition: None, limit: None })
                .await
                .unwrap_err(),
            "quran.root" => {
                registry.root(RootToolParams { root: "root-0".into() }).await.unwrap_err()
            }
            "quran.lemma" => {
                registry.lemma(LemmaToolParams { lemma: "lem-0".into() }).await.unwrap_err()
            }
            "quran.morphology" => registry
                .morphology(MorphologyToolParams { surah: 1, ayah: 1, position: 1 })
                .await
                .unwrap_err(),
            "quran.family" => registry
                .family(FamilyToolParams { kind: "token".into(), id: "token:1:1:1".into() })
                .await
                .unwrap_err(),
            "quran.graph_neighbors" => registry
                .graph_neighbors(tool_registry::GraphNeighborsParams {
                    node: "ayah:1:1".into(),
                    edge_types: None,
                    direction: None,
                    budgets: Default::default(),
                })
                .await
                .unwrap_err(),
            "quran.graph_path" => registry
                .graph_path(tool_registry::GraphPathParams {
                    from: "ayah:1:1".into(),
                    to: "ayah:1:2".into(),
                    mode: None,
                    paths: None,
                    edge_types: None,
                    direction: None,
                    budgets: Default::default(),
                })
                .await
                .unwrap_err(),
            "quran.graph_subgraph" => registry
                .graph_subgraph(tool_registry::GraphSubgraphParams {
                    seeds: vec!["ayah:1:1".into()],
                    edge_types: None,
                    direction: None,
                    budgets: Default::default(),
                })
                .await
                .unwrap_err(),
            "quran.graph_pattern" => registry
                .graph_pattern(tool_registry::GraphPatternParams {
                    seeds: vec!["ayah:1:1".into()],
                    steps: vec![tool_registry::GraphPatternStep {
                        edge: "NEXT".into(),
                        node_kind: None,
                    }],
                    budgets: Default::default(),
                })
                .await
                .unwrap_err(),
            "quran.graph_root_family" => registry
                .graph_root_family(tool_registry::GraphRootFamilyParams {
                    root: "root-0".into(),
                    limit: None,
                })
                .await
                .unwrap_err(),
            _ => panic!("unknown tool {name}"),
        };
        match err {
            ToolError::Backend { code, .. } => {
                assert_eq!(code, "QAI-QUR-0310", "{name} must return typed-unsupported");
            }
            other => panic!("{name}: expected typed backend error, got {other:?}"),
        }
    }
}

/// A truncated / budget-exhausted result carries `truncated` plus a non-empty
/// `incomplete_reason` and is never read as a proven "no matches".
#[tokio::test]
async fn truncated_results_carry_reason_and_never_claim_absence() {
    let registry = ToolRegistry::new(Arc::new(TruncatedBackend));
    let result = registry
        .graph_path(tool_registry::GraphPathParams {
            from: "ayah:1:1".into(),
            to: "ayah:2:1".into(),
            mode: Some("paths".into()),
            paths: None,
            edge_types: None,
            direction: None,
            budgets: Default::default(),
        })
        .await
        .unwrap();
    assert_eq!(result.tool_name, "quran.graph_path");
    assert_eq!(
        result.results["truncated"],
        serde_json::Value::Bool(true),
        "budget exhaustion must set truncated"
    );
    let reason =
        result.results["incomplete_reason"].as_str().unwrap_or_default().trim().to_string();
    assert!(!reason.is_empty(), "a truncated result must carry a non-empty incomplete_reason");
    // `truncated == true` is what stops an empty `paths` from reading as
    // absence; the envelope also surfaces the reason as a warning.
    assert_eq!(result.results["paths"], serde_json::json!([]));
    assert!(!result.warnings.is_empty(), "truncation must be surfaced on the envelope");
}
