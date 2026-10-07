//! Typed graph agent tools with attributed envelopes (Phase 4, D-12).
//!
//! # application::quran_graph_tools
//!
//! Implements `tool_registry::QuranBackend` graph methods over the
//! [`GraphBackend`](crate::quran_graph_api::GraphBackend) read trait, so the
//! same service answers the CLI, HTTP, and tool surfaces. Tools stay
//! read-only — no mutation tool exists in this phase (D-11 scope fence).
//! Every result carries the full [`ToolResult`] envelope with
//! canonical references, analysis sources, and a reproducibility checksum
//! pinning the graph build (projection id plus builder version plus corpus
//! generation) alongside the edition.
//!
//! Layer-D suggestion edges never appear as verified results: any traversed
//! edge whose assertion is still `pending` is labeled in the envelope
//! `warnings` with its algorithm attribution (T-04-12).

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use domain::SemVer;
use tool_registry::{
    GRAPH_NEIGHBORS_TOOL_VERSION, GRAPH_PATH_TOOL_VERSION, GRAPH_PATTERN_TOOL_VERSION,
    GRAPH_ROOT_FAMILY_TOOL_VERSION, GRAPH_SUBGRAPH_TOOL_VERSION, GraphNeighborsParams,
    GraphPathParams, GraphPatternParams, GraphRootFamilyParams, GraphSubgraphParams, QuranBackend,
    ToolRegistry,
};
use tools::{AnalysisSource, ToolError, ToolResult, reproducibility, research_checksum};

use crate::quran_graph_api::{
    BudgetPatch, Explanation, GraphApiError, GraphBackend as GraphReadBackend, GraphSnapshotMeta,
    NeighborsArgs, PathArgs, PathMode, PathsArgs, PatternArgs, PatternStepRequest, RootFamilyArgs,
    SubgraphArgs, parse_path_mode, parse_pattern_steps, read_options,
};

/// Map a graph read failure to a typed tool backend error, preserving the
/// namespaced `QAI-GRAPH-*` (or `QAI-MORPH-*`) code for HTTP mapping.
fn graph_tool_error(error: GraphApiError) -> ToolError {
    use storage::error::Diagnostic as _;
    ToolError::Backend { code: error.code().to_string(), detail: error.to_string() }
}

impl From<&tool_registry::GraphBudgetsParams> for BudgetPatch {
    fn from(params: &tool_registry::GraphBudgetsParams) -> Self {
        Self {
            max_hops: params.max_hops,
            max_nodes: params.max_nodes,
            max_edges: params.max_edges,
            max_paths: params.max_paths,
            max_fanout: params.max_fanout,
            timeout_ms: params.timeout_ms,
        }
    }
}

/// One `graph` analysis source pinning the answering projection build.
fn graph_source(projection_id: &str, builder_version: &str) -> Vec<AnalysisSource> {
    vec![AnalysisSource {
        kind: "graph".to_string(),
        reference: format!("{projection_id}@{builder_version}"),
    }]
}

/// Source versions pinning the graph build for the reproducibility
/// checksum: the projection family plus the builder that produced it.
fn graph_source_versions(projection_id: &str, builder_version: &str) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("graph_projection".to_string(), projection_id.to_string()),
        ("graph_builder".to_string(), builder_version.to_string()),
    ])
}

/// Canonical references for traversed nodes: `ayah:<surah>:<ayah>` stable
/// IDs pin to fully-qualified `quran:<slug>@<version>` references; every
/// other node kind travels as its stable ID (never dropped, never
/// invented).
fn explanation_references(explanation: &Explanation, meta: &GraphSnapshotMeta) -> Vec<String> {
    let mut ids = explanation.nodes.clone();
    for path in &explanation.paths {
        ids.extend(path.node_ids.iter().cloned());
    }
    ids.sort();
    ids.dedup();
    ids.iter()
        .map(|id| match id.strip_prefix("ayah:") {
            Some(rest) => {
                let mut parts = rest.split(':');
                match (parts.next(), parts.next(), parts.next()) {
                    (Some(surah), Some(ayah), None) => format!(
                        "quran:{}@{}:{surah}:{ayah}",
                        meta.edition_slug, meta.edition_version
                    ),
                    _ => id.clone(),
                }
            }
            None => id.clone(),
        })
        .collect()
}

/// Pending-suggestion labels (T-04-12): every traversed edge whose
/// assertion is still `pending` is reported with its algorithm attribution,
/// so unverified suggestions are never presented as verified scholarship.
fn suggestion_warnings(explanation: &Explanation) -> Vec<String> {
    use crate::quran_graph_api::ProvenanceExplanation;
    let mut warnings = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let edges =
        explanation.edges.iter().chain(explanation.paths.iter().flat_map(|path| path.edges.iter()));
    for edge in edges {
        if let ProvenanceExplanation::Asserted { decision, algorithm, algorithm_version, .. } =
            &edge.provenance
            && decision == "pending"
            && seen.insert((edge.src.clone(), edge.edge.clone(), edge.dst.clone()))
        {
            let attribution = match (algorithm, algorithm_version) {
                (Some(name), Some(version)) => format!("algorithm {name} v{version}"),
                (Some(name), None) => format!("algorithm {name}"),
                _ => "computational suggestion".to_string(),
            };
            warnings.push(format!(
                "suggestion edge {} -{}-> {} is pending human review ({attribution}); not verified scholarship",
                edge.src, edge.edge, edge.dst
            ));
        }
    }
    warnings.sort();
    warnings
}

/// Assemble the full tool envelope over a read-service payload.
#[allow(clippy::too_many_arguments)]
fn envelope(
    tool: &str,
    version: SemVer,
    query: &serde_json::Value,
    meta: &GraphSnapshotMeta,
    source_versions: BTreeMap<String, String>,
    canonical_references: Vec<String>,
    analysis_sources: Vec<AnalysisSource>,
    results: serde_json::Value,
    warnings: Vec<String>,
    started: Instant,
) -> ToolResult<serde_json::Value> {
    ToolResult {
        tool_name: tool.to_string(),
        tool_version: version,
        query: query.clone(),
        normalization_rules: Vec::new(),
        edition_id: if meta.edition_id.is_empty() { None } else { Some(meta.edition_id.clone()) },
        edition_version: Some(meta.edition_version.clone()),
        canonical_references,
        analysis_sources,
        results: results.clone(),
        confidence: None,
        warnings,
        execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
        reproducibility: reproducibility(
            tool,
            version,
            query,
            Some(&meta.edition_slug),
            Some(&meta.edition_version),
            source_versions.clone(),
            meta.corpus_generation,
        ),
        research_checksum: research_checksum(
            tool,
            version,
            query,
            &serde_json::json!({
                "edition_slug": meta.edition_slug,
                "edition_version": meta.edition_version,
                "corpus_generation": meta.corpus_generation,
                "sources": source_versions,
            }),
            &results,
        )
        .expect("research checksum input is JSON-serializable"),
    }
}

/// The graph tool backend over any [`GraphReadBackend`] (the live
/// [`GraphApiService`](crate::quran_graph_api::GraphApiService) or a fake
/// in contract tests).
pub struct GraphToolBackend {
    backend: Arc<dyn GraphReadBackend>,
}

impl GraphToolBackend {
    /// Wrap a read backend.
    pub fn new(backend: Arc<dyn GraphReadBackend>) -> Self {
        Self { backend }
    }

    /// A registry serving the five graph tools over the read backend.
    pub fn registry(backend: Arc<dyn GraphReadBackend>) -> ToolRegistry {
        ToolRegistry::new(Arc::new(Self::new(backend)))
    }
}

#[async_trait]
impl QuranBackend for GraphToolBackend {
    async fn backend_get_ayah(
        &self,
        _reference: &quran_core::QuranRef,
        _options: &quran_core::AyahOptions,
    ) -> Result<(quran_core::AyahView, tool_registry::BackendMeta), ToolError> {
        Err(ToolError::Backend {
            code: "QAI-QUR-0310".into(),
            detail: "quran.get_ayah is not served by the graph tool backend".to_string(),
        })
    }

    async fn backend_get_context(
        &self,
        _reference: &quran_core::QuranRef,
        _spec: &quran_core::ContextSpec,
    ) -> Result<(quran_core::ContextView, tool_registry::BackendMeta), ToolError> {
        Err(ToolError::Backend {
            code: "QAI-QUR-0310".into(),
            detail: "quran.get_context is not served by the graph tool backend".to_string(),
        })
    }

    /// `quran.graph_neighbors`: bounded neighbors with per-edge provenance.
    async fn backend_graph_neighbors(
        &self,
        params: &GraphNeighborsParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let started = Instant::now();
        let meta = self.backend.snapshot_meta().await.map_err(graph_tool_error)?;
        // CLI parity: neighbors opens one hop by default (`--hops 1`).
        let options = read_options(
            BudgetPatch::from(&params.budgets),
            1,
            params.edge_types.clone(),
            params.direction.as_deref(),
        )
        .map_err(graph_tool_error)?;
        let output = self
            .backend
            .neighbors(NeighborsArgs { node: params.node.clone(), options })
            .await
            .map_err(graph_tool_error)?;
        let query = serde_json::to_value(params).unwrap_or(serde_json::Value::Null);
        let snapshot = &output.explanation.snapshot;
        Ok(envelope(
            "quran.graph_neighbors",
            GRAPH_NEIGHBORS_TOOL_VERSION,
            &query,
            &meta,
            graph_source_versions(&snapshot.projection_id, &snapshot.builder_version),
            explanation_references(&output.explanation, &meta),
            graph_source(&snapshot.projection_id, &snapshot.builder_version),
            serde_json::to_value(&output).unwrap_or(serde_json::Value::Null),
            suggestion_warnings(&output.explanation),
            started,
        ))
    }

    /// `quran.graph_path`: reachability, shortest, or up-to-K paths under
    /// the CLI's mode selector.
    async fn backend_graph_path(
        &self,
        params: &GraphPathParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let started = Instant::now();
        let meta = self.backend.snapshot_meta().await.map_err(graph_tool_error)?;
        let mode = parse_path_mode(params.mode.as_deref()).map_err(graph_tool_error)?;
        let options = read_options(
            BudgetPatch::from(&params.budgets),
            4,
            params.edge_types.clone(),
            params.direction.as_deref(),
        )
        .map_err(graph_tool_error)?;
        // CLI parity: `--paths` defaults to the `max_paths` budget.
        let k = params.paths.unwrap_or(options.budgets.max_paths);
        let query = serde_json::to_value(params).unwrap_or(serde_json::Value::Null);
        let (output_value, explanation) = match mode {
            PathMode::Reachability => {
                let output = self
                    .backend
                    .reachability(PathArgs {
                        from: params.from.clone(),
                        to: params.to.clone(),
                        options,
                    })
                    .await
                    .map_err(graph_tool_error)?;
                let value = serde_json::to_value(&output).unwrap_or(serde_json::Value::Null);
                (value, output.explanation)
            }
            PathMode::Shortest => {
                let output = self
                    .backend
                    .shortest_path(PathArgs {
                        from: params.from.clone(),
                        to: params.to.clone(),
                        options,
                    })
                    .await
                    .map_err(graph_tool_error)?;
                let value = serde_json::to_value(&output).unwrap_or(serde_json::Value::Null);
                (value, output.explanation)
            }
            PathMode::Paths => {
                let output = self
                    .backend
                    .paths(PathsArgs {
                        from: params.from.clone(),
                        to: params.to.clone(),
                        k,
                        options,
                    })
                    .await
                    .map_err(graph_tool_error)?;
                let value = serde_json::to_value(&output).unwrap_or(serde_json::Value::Null);
                (value, output.explanation)
            }
        };
        let snapshot = &explanation.snapshot;
        Ok(envelope(
            "quran.graph_path",
            GRAPH_PATH_TOOL_VERSION,
            &query,
            &meta,
            graph_source_versions(&snapshot.projection_id, &snapshot.builder_version),
            explanation_references(&explanation, &meta),
            graph_source(&snapshot.projection_id, &snapshot.builder_version),
            output_value,
            suggestion_warnings(&explanation),
            started,
        ))
    }

    /// `quran.graph_subgraph`: bounded multi-seed subgraph.
    async fn backend_graph_subgraph(
        &self,
        params: &GraphSubgraphParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let started = Instant::now();
        let meta = self.backend.snapshot_meta().await.map_err(graph_tool_error)?;
        let options = read_options(
            BudgetPatch::from(&params.budgets),
            4,
            params.edge_types.clone(),
            params.direction.as_deref(),
        )
        .map_err(graph_tool_error)?;
        let output = self
            .backend
            .subgraph(SubgraphArgs { seeds: params.seeds.clone(), options })
            .await
            .map_err(graph_tool_error)?;
        let query = serde_json::to_value(params).unwrap_or(serde_json::Value::Null);
        let snapshot = &output.explanation.snapshot;
        Ok(envelope(
            "quran.graph_subgraph",
            GRAPH_SUBGRAPH_TOOL_VERSION,
            &query,
            &meta,
            graph_source_versions(&snapshot.projection_id, &snapshot.builder_version),
            explanation_references(&output.explanation, &meta),
            graph_source(&snapshot.projection_id, &snapshot.builder_version),
            serde_json::to_value(&output).unwrap_or(serde_json::Value::Null),
            suggestion_warnings(&output.explanation),
            started,
        ))
    }

    /// `quran.graph_pattern`: typed pattern query from seeds.
    async fn backend_graph_pattern(
        &self,
        params: &GraphPatternParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let started = Instant::now();
        let meta = self.backend.snapshot_meta().await.map_err(graph_tool_error)?;
        let pattern = parse_pattern_steps(
            params
                .steps
                .iter()
                .map(|step| PatternStepRequest {
                    edge: step.edge.clone(),
                    node_kind: step.node_kind.clone(),
                })
                .collect(),
        )
        .map_err(graph_tool_error)?;
        let options = read_options(BudgetPatch::from(&params.budgets), 4, None, None)
            .map_err(graph_tool_error)?;
        let output = self
            .backend
            .pattern(PatternArgs { pattern, seeds: params.seeds.clone(), options })
            .await
            .map_err(graph_tool_error)?;
        let query = serde_json::to_value(params).unwrap_or(serde_json::Value::Null);
        let snapshot = &output.explanation.snapshot;
        Ok(envelope(
            "quran.graph_pattern",
            GRAPH_PATTERN_TOOL_VERSION,
            &query,
            &meta,
            graph_source_versions(&snapshot.projection_id, &snapshot.builder_version),
            explanation_references(&output.explanation, &meta),
            graph_source(&snapshot.projection_id, &snapshot.builder_version),
            serde_json::to_value(&output).unwrap_or(serde_json::Value::Null),
            suggestion_warnings(&output.explanation),
            started,
        ))
    }

    /// `quran.graph_root_family`: ranked ayahs from the active morphology
    /// dataset, attributed per dataset (no attribution means no result).
    async fn backend_graph_root_family(
        &self,
        params: &GraphRootFamilyParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let started = Instant::now();
        let meta = self.backend.snapshot_meta().await.map_err(graph_tool_error)?;
        // CLI parity: the verb caps at 25 ranked ayahs by default.
        let limit = params.limit.unwrap_or(25);
        let output = self
            .backend
            .root_family(RootFamilyArgs { root: params.root.clone(), limit })
            .await
            .map_err(graph_tool_error)?;
        let query = serde_json::to_value(params).unwrap_or(serde_json::Value::Null);
        let canonical_references: Vec<String> = output
            .ayahs
            .iter()
            .map(|hit| {
                format!(
                    "quran:{}@{}:{}:{}",
                    meta.edition_slug, meta.edition_version, hit.surah, hit.ayah
                )
            })
            .collect();
        Ok(envelope(
            "quran.graph_root_family",
            GRAPH_ROOT_FAMILY_TOOL_VERSION,
            &query,
            &meta,
            BTreeMap::from([("graph_dataset".to_string(), output.dataset.clone())]),
            canonical_references,
            vec![AnalysisSource { kind: "dataset".to_string(), reference: output.dataset.clone() }],
            serde_json::to_value(&output).unwrap_or(serde_json::Value::Null),
            Vec::new(),
            started,
        ))
    }
}
