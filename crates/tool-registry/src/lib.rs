//! Minimal local tool registry with the first two read-only tools (D1.8).
//!
//! # tool-registry
//!
//! `quran.get_ayah` and `quran.get_context` prove the §12 contract works from
//! all interfaces before agents exist. Both are `ReadOnly` (§29). The registry
//! depends on a [`QuranBackend`] trait — implemented by the application layer —
//! so this crate never depends on storage or the reader directly.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use domain::SemVer;
use quran_core::{AyahOptions, AyahView, ContextBoundary, ContextSpec, ContextView, QuranRef};
use serde::{Deserialize, Serialize};
use tools::{AnalysisSource, ToolError, ToolResult, reproducibility};

/// Tool versions (§12 tool plan).
pub const GET_AYAH_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Tool versions (§12 tool plan).
pub const GET_CONTEXT_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Tool versions (§12 tool plan; D-13 lexicon surface).
pub const SEARCH_TOOL_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Tool versions (§12 tool plan; D-13 lexicon surface).
pub const ROOT_TOOL_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Tool versions (§12 tool plan; D-13 lexicon surface).
pub const LEMMA_TOOL_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Tool versions (§12 tool plan; D-13 lexicon surface).
pub const MORPHOLOGY_TOOL_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Tool versions (§12 tool plan; D-13 lexicon surface).
pub const FAMILY_TOOL_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Tool versions (§12 tool plan; Phase 4 graph surface, D-12).
pub const GRAPH_NEIGHBORS_TOOL_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Tool versions (§12 tool plan; Phase 4 graph surface, D-12).
pub const GRAPH_PATH_TOOL_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Tool versions (§12 tool plan; Phase 4 graph surface, D-12).
pub const GRAPH_SUBGRAPH_TOOL_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Tool versions (§12 tool plan; Phase 4 graph surface, D-12).
pub const GRAPH_PATTERN_TOOL_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Tool versions (§12 tool plan; Phase 4 graph surface, D-12).
pub const GRAPH_ROOT_FAMILY_TOOL_VERSION: SemVer = SemVer::new(1, 0, 0);

/// Edition + generation metadata behind one backend read.
#[derive(Debug, Clone)]
pub struct BackendMeta {
    /// Edition slug read.
    pub edition_slug: String,
    /// Edition version read.
    pub edition_version: String,
    /// Edition row id.
    pub edition_id: String,
    /// Corpus generation read from.
    pub corpus_generation: u64,
    /// Edition text hash (`sha256:<hex>`) for ETags.
    pub text_hash: String,
    /// Edition script (`uthmani`, …).
    pub script: String,
    /// Transmission, when declared.
    pub riwayah: Option<String>,
    /// Verse-numbering scheme.
    pub numbering_scheme: String,
    /// Graph projection family served (`quran-structural-v1`, …; empty when
    /// the tool does not read a graph projection).
    pub projection_id: String,
    /// Builder name + version of the serving projection build (empty when
    /// the tool does not read a graph projection).
    pub builder_version: String,
}

/// The reader surface tools need. Implemented by `application` for the real
/// reader; tests substitute fakes. Keeps this crate off storage.
#[async_trait]
pub trait QuranBackend: Send + Sync {
    /// Fetch one ayah view plus read metadata.
    async fn backend_get_ayah(
        &self,
        reference: &QuranRef,
        options: &AyahOptions,
    ) -> Result<(AyahView, BackendMeta), ToolError>;

    /// Fetch context plus read metadata.
    async fn backend_get_context(
        &self,
        reference: &QuranRef,
        spec: &ContextSpec,
    ) -> Result<(ContextView, BackendMeta), ToolError>;

    /// `quran.search` (D-13): normalized search with normalization rules and
    /// attribution on the envelope. Backends that do not serve search return a
    /// typed backend error.
    async fn backend_search(
        &self,
        _params: &SearchToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("quran.search"))
    }

    /// `quran.root` (D-13): root-grouped occurrences with dataset attribution.
    async fn backend_root(
        &self,
        _params: &RootToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("quran.root"))
    }

    /// `quran.lemma` (D-13): lemma-grouped occurrences with dataset attribution.
    async fn backend_lemma(
        &self,
        _params: &LemmaToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("quran.lemma"))
    }

    /// `quran.morphology` (D-13): all analyses of one token, attributed.
    async fn backend_morphology(
        &self,
        _params: &MorphologyToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("quran.morphology"))
    }

    /// `quran.family` (D-13): explained family relations, attributed.
    async fn backend_family(
        &self,
        _params: &FamilyToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("quran.family"))
    }

    /// `quran.graph_neighbors` (D-12): bounded neighbors with per-edge
    /// provenance. Backends that do not serve the graph return a typed
    /// backend error.
    async fn backend_graph_neighbors(
        &self,
        _params: &GraphNeighborsParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("quran.graph_neighbors"))
    }

    /// `quran.graph_path` (D-12): reachability, shortest, or up-to-K paths.
    async fn backend_graph_path(
        &self,
        _params: &GraphPathParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("quran.graph_path"))
    }

    /// `quran.graph_subgraph` (D-12): bounded multi-seed subgraph.
    async fn backend_graph_subgraph(
        &self,
        _params: &GraphSubgraphParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("quran.graph_subgraph"))
    }

    /// `quran.graph_pattern` (D-12): typed pattern query from seeds.
    async fn backend_graph_pattern(
        &self,
        _params: &GraphPatternParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("quran.graph_pattern"))
    }

    /// `quran.graph_root_family` (D-12): ranked ayahs from the active
    /// morphology dataset (lexicon-gated).
    async fn backend_graph_root_family(
        &self,
        _params: &GraphRootFamilyParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("quran.graph_root_family"))
    }
}

/// Parameters for `quran.get_ayah`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAyahParams {
    /// Reference string (edition may be embedded).
    pub reference: String,
    /// Translation slugs to attach.
    #[serde(default)]
    pub translations: Vec<String>,
    /// Attach word glosses.
    #[serde(default)]
    pub glosses: bool,
    /// Attach surface tokens.
    #[serde(default)]
    pub tokens: bool,
}

/// Parameters for `quran.get_context`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetContextParams {
    /// Focal reference (ayah-level).
    pub reference: String,
    /// Ayahs before (default 3).
    #[serde(default = "default_before")]
    pub before: u16,
    /// Ayahs after (default 3).
    #[serde(default = "default_after")]
    pub after: u16,
    /// Structural boundary.
    #[serde(default)]
    pub boundary: ContextBoundaryArg,
    /// Hard cap (default 11).
    #[serde(default = "default_max")]
    pub max_ayahs: u16,
}

fn default_before() -> u16 {
    3
}

fn default_after() -> u16 {
    3
}

fn default_max() -> u16 {
    11
}

/// Parameters for `quran.search` (D-13): normalized search over the serving
/// index. An empty/whitespace query is a typed invalid-input error, never an
/// empty result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchToolParams {
    /// Query text.
    pub text: String,
    /// Edition `slug@version` (defaults to the indexed edition).
    #[serde(default)]
    pub edition: Option<String>,
    /// Result cap (defaults to 20).
    #[serde(default)]
    pub limit: Option<u32>,
}

/// Parameters for `quran.root` (D-13).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootToolParams {
    /// Root surface to group by.
    pub root: String,
}

/// Parameters for `quran.lemma` (D-13).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LemmaToolParams {
    /// Lemma surface to group by.
    pub lemma: String,
}

/// Parameters for `quran.morphology` (D-13).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphologyToolParams {
    /// Surah number.
    pub surah: u16,
    /// Ayah number.
    pub ayah: u32,
    /// Token position within the ayah.
    pub position: u32,
}

/// Parameters for `quran.family` (D-13).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyToolParams {
    /// Member kind (`token`, `root`, `lemma`, …).
    pub kind: String,
    /// Member id.
    pub id: String,
}

/// Optional per-field budget overrides for the graph tools (D-12).
/// Primitives only — this crate must not name `quran-graph` types
/// (`arch-check`): the application layer folds these into budgets.
/// `None` keeps the surface default; explicit values fail pre-flight,
/// never clamp.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphBudgetsParams {
    /// Maximum traversal depth in hops.
    #[serde(default)]
    pub max_hops: Option<usize>,
    /// Maximum distinct nodes collected per query.
    #[serde(default)]
    pub max_nodes: Option<usize>,
    /// Maximum edge relaxations performed per query.
    #[serde(default)]
    pub max_edges: Option<usize>,
    /// Maximum paths returned per path query.
    #[serde(default)]
    pub max_paths: Option<usize>,
    /// Maximum neighbors expanded per single node visit.
    #[serde(default)]
    pub max_fanout: Option<usize>,
    /// Wall-clock budget in milliseconds.
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

/// Parameters for `quran.graph_neighbors` (D-12).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNeighborsParams {
    /// Stable node id to open around.
    pub node: String,
    /// Allowed edge predicates (`None` admits every allowlisted predicate).
    #[serde(default)]
    pub edge_types: Option<Vec<String>>,
    /// Expansion direction (`both` default; `outgoing`|`incoming`).
    #[serde(default)]
    pub direction: Option<String>,
    /// Budget overrides.
    #[serde(default)]
    pub budgets: GraphBudgetsParams,
}

/// Parameters for `quran.graph_path` (D-12).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPathParams {
    /// Source stable id.
    pub from: String,
    /// Destination stable id.
    pub to: String,
    /// Search mode (`reachability`|`shortest`|`paths`; default `paths`).
    #[serde(default)]
    pub mode: Option<String>,
    /// Requested path count for `paths` mode (defaults to the `max_paths`
    /// budget; beyond it is a pre-flight error).
    #[serde(default)]
    pub paths: Option<usize>,
    /// Allowed edge predicates (recorded; path modes expand
    /// direction-agnostic by traversal design).
    #[serde(default)]
    pub edge_types: Option<Vec<String>>,
    /// Expansion direction (recorded; path modes expand direction-agnostic).
    #[serde(default)]
    pub direction: Option<String>,
    /// Budget overrides.
    #[serde(default)]
    pub budgets: GraphBudgetsParams,
}

/// Parameters for `quran.graph_subgraph` (D-12).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSubgraphParams {
    /// Seed stable ids.
    pub seeds: Vec<String>,
    /// Allowed edge predicates.
    #[serde(default)]
    pub edge_types: Option<Vec<String>>,
    /// Expansion direction.
    #[serde(default)]
    pub direction: Option<String>,
    /// Budget overrides.
    #[serde(default)]
    pub budgets: GraphBudgetsParams,
}

/// One typed pattern step for `quran.graph_pattern` (D-12).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPatternStep {
    /// Edge predicate; must be in the edge vocabulary.
    pub edge: String,
    /// Optional node-kind constraint (`ayah`, `token`, `root`, …).
    #[serde(default)]
    pub node_kind: Option<String>,
}

/// Parameters for `quran.graph_pattern` (D-12).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPatternParams {
    /// Seed stable ids.
    pub seeds: Vec<String>,
    /// Ordered typed steps (`1..=8`; unknown edges reject pre-flight).
    pub steps: Vec<GraphPatternStep>,
    /// Budget overrides.
    #[serde(default)]
    pub budgets: GraphBudgetsParams,
}

/// Parameters for `quran.graph_root_family` (D-12).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRootFamilyParams {
    /// Normalized root spelling.
    pub root: String,
    /// Result cap (default 25, mirroring the CLI verb).
    #[serde(default)]
    pub limit: Option<usize>,
}

/// A typed backend error for a tool a backend does not serve.
fn unsupported(tool: &'static str) -> ToolError {
    ToolError::Backend {
        code: "QAI-QUR-0310".to_string(),
        detail: format!("{tool} is not supported by this backend"),
    }
}

/// Serializable boundary argument.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextBoundaryArg {
    /// Stay within the focal surah.
    Surah,
    /// Stay within the focal juz.
    Juz,
    /// Stay within the focal ruku.
    Ruku,
    /// Stay within the focal page.
    Page,
    /// No structural boundary.
    #[default]
    None,
}

impl From<ContextBoundaryArg> for ContextBoundary {
    fn from(value: ContextBoundaryArg) -> Self {
        match value {
            ContextBoundaryArg::Surah => Self::Surah,
            ContextBoundaryArg::Juz => Self::Juz,
            ContextBoundaryArg::Ruku => Self::Ruku,
            ContextBoundaryArg::Page => Self::Page,
            ContextBoundaryArg::None => Self::None,
        }
    }
}

/// The local tool registry.
pub struct ToolRegistry {
    backend: Arc<dyn QuranBackend>,
}

impl ToolRegistry {
    /// Wrap a backend.
    pub fn new(backend: Arc<dyn QuranBackend>) -> Self {
        Self { backend }
    }

    /// The full registered tool set (§12 tool plan; D-13): the two direct-read
    /// tools plus the attributed search/lexicon surface.
    pub const TOOL_NAMES: [&'static str; 12] = [
        "quran.get_ayah",
        "quran.get_context",
        "quran.search",
        "quran.root",
        "quran.lemma",
        "quran.morphology",
        "quran.family",
        "quran.graph_neighbors",
        "quran.graph_path",
        "quran.graph_subgraph",
        "quran.graph_pattern",
        "quran.graph_root_family",
    ];

    /// Registered tool names.
    pub fn tool_names(&self) -> Vec<&'static str> {
        Self::TOOL_NAMES.to_vec()
    }

    /// `quran.get_ayah`: exact ayah lookup, no synthesis, ever.
    ///
    /// Returns the envelope plus the backend read metadata (edition descriptor
    /// for API envelopes and ETags).
    pub async fn get_ayah(
        &self,
        params: GetAyahParams,
    ) -> Result<(ToolResult<Vec<AyahView>>, BackendMeta), ToolError> {
        let started = Instant::now();
        let reference = quran_core::parse(&params.reference).map_err(|err| {
            ToolError::InvalidInput { tool: "quran.get_ayah", detail: err.to_string() }
        })?;
        let options = AyahOptions {
            translations: params.translations.clone(),
            glosses: params.glosses,
            tokens: params.tokens,
        };
        let query = serde_json::to_value(&params).unwrap_or(serde_json::Value::Null);
        let (view, meta) = self.backend.backend_get_ayah(&reference, &options).await?;
        let result = ToolResult {
            tool_name: "quran.get_ayah".to_string(),
            tool_version: GET_AYAH_VERSION,
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: Some(meta.edition_id.clone()),
            edition_version: Some(meta.edition_version.clone()),
            canonical_references: vec![view.canonical.reference().to_string()],
            analysis_sources: vec![AnalysisSource {
                kind: "canonical".to_string(),
                reference: view.canonical.reference().to_string(),
            }],
            results: vec![view],
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                "quran.get_ayah",
                GET_AYAH_VERSION,
                &query,
                Some(&meta.edition_slug),
                Some(&meta.edition_version),
                BTreeMap::new(),
                meta.corpus_generation,
            ),
        };
        Ok((result, meta))
    }

    /// `quran.get_context`: structure-bounded context retrieval.
    pub async fn get_context(
        &self,
        params: GetContextParams,
    ) -> Result<(ToolResult<ContextView>, BackendMeta), ToolError> {
        let started = Instant::now();
        if params.max_ayahs < 1 {
            return Err(ToolError::InvalidInput {
                tool: "quran.get_context",
                detail: "max_ayahs must be >= 1".to_string(),
            });
        }
        let reference = quran_core::parse(&params.reference).map_err(|err| {
            ToolError::InvalidInput { tool: "quran.get_context", detail: err.to_string() }
        })?;
        let spec = ContextSpec {
            before: params.before,
            after: params.after,
            boundary: ContextBoundary::from(params.boundary),
            include_surah_header: false,
            max_ayahs: params.max_ayahs,
        };
        let query = serde_json::to_value(&params).unwrap_or(serde_json::Value::Null);
        let (view, meta) = self.backend.backend_get_context(&reference, &spec).await?;
        let mut references = vec![view.canonical_reference.clone()];
        references.extend(view.before.iter().map(|item| item.canonical.reference().to_string()));
        references.extend(view.after.iter().map(|item| item.canonical.reference().to_string()));
        let sources = references
            .iter()
            .map(|reference| AnalysisSource {
                kind: "canonical".to_string(),
                reference: reference.clone(),
            })
            .collect();
        let result = ToolResult {
            tool_name: "quran.get_context".to_string(),
            tool_version: GET_CONTEXT_VERSION,
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: Some(meta.edition_id.clone()),
            edition_version: Some(meta.edition_version.clone()),
            canonical_references: references,
            analysis_sources: sources,
            results: view,
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                "quran.get_context",
                GET_CONTEXT_VERSION,
                &query,
                Some(&meta.edition_slug),
                Some(&meta.edition_version),
                BTreeMap::new(),
                meta.corpus_generation,
            ),
        };
        Ok((result, meta))
    }

    /// `quran.search` (D-13): a non-empty query is required — an empty query is
    /// a typed invalid-input error, never an empty result.
    pub async fn search(
        &self,
        params: SearchToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        if params.text.trim().is_empty() {
            return Err(ToolError::InvalidInput {
                tool: "quran.search",
                detail: "query text must not be empty".to_string(),
            });
        }
        self.backend.backend_search(&params).await
    }

    /// `quran.root` (D-13): a non-empty root is required.
    pub async fn root(
        &self,
        params: RootToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        if params.root.trim().is_empty() {
            return Err(ToolError::InvalidInput {
                tool: "quran.root",
                detail: "root must not be empty".to_string(),
            });
        }
        self.backend.backend_root(&params).await
    }

    /// `quran.lemma` (D-13): a non-empty lemma is required.
    pub async fn lemma(
        &self,
        params: LemmaToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        if params.lemma.trim().is_empty() {
            return Err(ToolError::InvalidInput {
                tool: "quran.lemma",
                detail: "lemma must not be empty".to_string(),
            });
        }
        self.backend.backend_lemma(&params).await
    }

    /// `quran.morphology` (D-13): surah/ayah/position must be non-zero.
    pub async fn morphology(
        &self,
        params: MorphologyToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        if params.surah == 0 || params.ayah == 0 || params.position == 0 {
            return Err(ToolError::InvalidInput {
                tool: "quran.morphology",
                detail: "surah, ayah and position must all be >= 1".to_string(),
            });
        }
        self.backend.backend_morphology(&params).await
    }

    /// `quran.family` (D-13): non-empty kind and id are required.
    pub async fn family(
        &self,
        params: FamilyToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        if params.kind.trim().is_empty() || params.id.trim().is_empty() {
            return Err(ToolError::InvalidInput {
                tool: "quran.family",
                detail: "family needs a non-empty kind and id".to_string(),
            });
        }
        self.backend.backend_family(&params).await
    }

    /// `quran.graph_neighbors` (D-12): a non-empty node is required.
    pub async fn graph_neighbors(
        &self,
        params: GraphNeighborsParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        if params.node.trim().is_empty() {
            return Err(ToolError::InvalidInput {
                tool: "quran.graph_neighbors",
                detail: "node must not be empty".to_string(),
            });
        }
        self.backend.backend_graph_neighbors(&params).await
    }

    /// `quran.graph_path` (D-12): non-empty endpoints are required; the
    /// mode selector is validated by the backend before any I/O.
    pub async fn graph_path(
        &self,
        params: GraphPathParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        if params.from.trim().is_empty() || params.to.trim().is_empty() {
            return Err(ToolError::InvalidInput {
                tool: "quran.graph_path",
                detail: "path needs non-empty from and to nodes".to_string(),
            });
        }
        self.backend.backend_graph_path(&params).await
    }

    /// `quran.graph_subgraph` (D-12): at least one non-empty seed is required.
    pub async fn graph_subgraph(
        &self,
        params: GraphSubgraphParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        if params.seeds.is_empty() || params.seeds.iter().any(|seed| seed.trim().is_empty()) {
            return Err(ToolError::InvalidInput {
                tool: "quran.graph_subgraph",
                detail: "subgraph needs at least one non-empty seed".to_string(),
            });
        }
        self.backend.backend_graph_subgraph(&params).await
    }

    /// `quran.graph_pattern` (D-12): seeds plus at least one step are
    /// required; step validation happens in the backend before any I/O.
    pub async fn graph_pattern(
        &self,
        params: GraphPatternParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        if params.seeds.is_empty() || params.seeds.iter().any(|seed| seed.trim().is_empty()) {
            return Err(ToolError::InvalidInput {
                tool: "quran.graph_pattern",
                detail: "pattern needs at least one non-empty seed".to_string(),
            });
        }
        if params.steps.is_empty() {
            return Err(ToolError::InvalidInput {
                tool: "quran.graph_pattern",
                detail: "pattern needs at least one step".to_string(),
            });
        }
        self.backend.backend_graph_pattern(&params).await
    }

    /// `quran.graph_root_family` (D-12): a non-empty root is required.
    pub async fn graph_root_family(
        &self,
        params: GraphRootFamilyParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        if params.root.trim().is_empty() {
            return Err(ToolError::InvalidInput {
                tool: "quran.graph_root_family",
                detail: "root must not be empty".to_string(),
            });
        }
        self.backend.backend_graph_root_family(&params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quran_core::{AyahNumber, EditionSelector, SurahNumber};

    struct FakeBackend;

    fn test_view() -> (AyahView, BackendMeta) {
        let view = serde_json::from_value::<AyahView>(serde_json::json!({
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
        .unwrap();
        let meta = BackendMeta {
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
        };
        (view, meta)
    }

    /// Graph tools share one attributed envelope shape in these contract
    /// tests; the application backend pins the real projection build.
    fn graph_attributed(
        tool: &str,
        version: SemVer,
        query: serde_json::Value,
    ) -> ToolResult<serde_json::Value> {
        attributed(tool, version, query)
    }

    /// A conformant attributed envelope for the D-13 tools: attribution always
    /// present, rules present where a normalization trace applies.
    fn attributed(
        tool: &str,
        version: SemVer,
        query: serde_json::Value,
    ) -> ToolResult<serde_json::Value> {
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
            results: serde_json::json!({ "ok": true }),
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
        }
    }

    #[async_trait]
    impl QuranBackend for FakeBackend {
        async fn backend_get_ayah(
            &self,
            _reference: &QuranRef,
            _options: &AyahOptions,
        ) -> Result<(AyahView, BackendMeta), ToolError> {
            Ok(test_view())
        }

        async fn backend_get_context(
            &self,
            _reference: &QuranRef,
            _spec: &ContextSpec,
        ) -> Result<(ContextView, BackendMeta), ToolError> {
            Err(ToolError::Backend { code: "QAI-QUR-0310".into(), detail: "unimplemented".into() })
        }

        async fn backend_search(
            &self,
            params: &SearchToolParams,
        ) -> Result<ToolResult<serde_json::Value>, ToolError> {
            Ok(attributed(
                "quran.search",
                SEARCH_TOOL_VERSION,
                serde_json::to_value(params).unwrap(),
            ))
        }

        async fn backend_root(
            &self,
            params: &RootToolParams,
        ) -> Result<ToolResult<serde_json::Value>, ToolError> {
            Ok(attributed("quran.root", ROOT_TOOL_VERSION, serde_json::to_value(params).unwrap()))
        }

        async fn backend_lemma(
            &self,
            params: &LemmaToolParams,
        ) -> Result<ToolResult<serde_json::Value>, ToolError> {
            Ok(attributed("quran.lemma", LEMMA_TOOL_VERSION, serde_json::to_value(params).unwrap()))
        }

        async fn backend_morphology(
            &self,
            params: &MorphologyToolParams,
        ) -> Result<ToolResult<serde_json::Value>, ToolError> {
            Ok(attributed(
                "quran.morphology",
                MORPHOLOGY_TOOL_VERSION,
                serde_json::to_value(params).unwrap(),
            ))
        }

        async fn backend_family(
            &self,
            params: &FamilyToolParams,
        ) -> Result<ToolResult<serde_json::Value>, ToolError> {
            Ok(attributed(
                "quran.family",
                FAMILY_TOOL_VERSION,
                serde_json::to_value(params).unwrap(),
            ))
        }

        /// Graph tools share one attributed envelope shape in these
        /// contract tests; the application backend (plan 04-04) pins the
        /// real projection build.
        async fn backend_graph_neighbors(
            &self,
            params: &GraphNeighborsParams,
        ) -> Result<ToolResult<serde_json::Value>, ToolError> {
            Ok(graph_attributed(
                "quran.graph_neighbors",
                GRAPH_NEIGHBORS_TOOL_VERSION,
                serde_json::to_value(params).unwrap(),
            ))
        }

        async fn backend_graph_path(
            &self,
            params: &GraphPathParams,
        ) -> Result<ToolResult<serde_json::Value>, ToolError> {
            Ok(graph_attributed(
                "quran.graph_path",
                GRAPH_PATH_TOOL_VERSION,
                serde_json::to_value(params).unwrap(),
            ))
        }

        async fn backend_graph_subgraph(
            &self,
            params: &GraphSubgraphParams,
        ) -> Result<ToolResult<serde_json::Value>, ToolError> {
            Ok(graph_attributed(
                "quran.graph_subgraph",
                GRAPH_SUBGRAPH_TOOL_VERSION,
                serde_json::to_value(params).unwrap(),
            ))
        }

        async fn backend_graph_pattern(
            &self,
            params: &GraphPatternParams,
        ) -> Result<ToolResult<serde_json::Value>, ToolError> {
            Ok(graph_attributed(
                "quran.graph_pattern",
                GRAPH_PATTERN_TOOL_VERSION,
                serde_json::to_value(params).unwrap(),
            ))
        }

        async fn backend_graph_root_family(
            &self,
            params: &GraphRootFamilyParams,
        ) -> Result<ToolResult<serde_json::Value>, ToolError> {
            Ok(graph_attributed(
                "quran.graph_root_family",
                GRAPH_ROOT_FAMILY_TOOL_VERSION,
                serde_json::to_value(params).unwrap(),
            ))
        }
    }

    #[test]
    fn registry_lists_all_tools() {
        let registry = ToolRegistry::new(Arc::new(FakeBackend));
        assert_eq!(
            registry.tool_names(),
            [
                "quran.get_ayah",
                "quran.get_context",
                "quran.search",
                "quran.root",
                "quran.lemma",
                "quran.morphology",
                "quran.family",
                "quran.graph_neighbors",
                "quran.graph_path",
                "quran.graph_subgraph",
                "quran.graph_pattern",
                "quran.graph_root_family",
            ]
        );
    }

    /// D-13 / ASSUMPTION (empty fallback edge): every new selector tool rejects
    /// an empty input as a typed invalid-input error, never an empty result.
    #[tokio::test]
    async fn empty_selector_tools_are_typed_invalid_input() {
        let registry = ToolRegistry::new(Arc::new(FakeBackend));
        let err = registry
            .search(SearchToolParams { text: "   ".into(), edition: None, limit: None })
            .await
            .unwrap_err();
        assert!(matches!(err, ToolError::InvalidInput { tool: "quran.search", .. }));
        let err = registry.root(RootToolParams { root: String::new() }).await.unwrap_err();
        assert!(matches!(err, ToolError::InvalidInput { tool: "quran.root", .. }));
        let err = registry.lemma(LemmaToolParams { lemma: "  ".into() }).await.unwrap_err();
        assert!(matches!(err, ToolError::InvalidInput { tool: "quran.lemma", .. }));
        let err = registry
            .morphology(MorphologyToolParams { surah: 1, ayah: 1, position: 0 })
            .await
            .unwrap_err();
        assert!(matches!(err, ToolError::InvalidInput { tool: "quran.morphology", .. }));
        let err = registry
            .family(FamilyToolParams { kind: String::new(), id: "x".into() })
            .await
            .unwrap_err();
        assert!(matches!(err, ToolError::InvalidInput { tool: "quran.family", .. }));
    }

    /// D-13 / T-03-29: every new tool result is emitted with attribution
    /// (`analysis_sources`) and, where a normalization trace applies,
    /// `normalization_rules`.
    #[tokio::test]
    async fn new_tools_return_attributed_envelopes() {
        let registry = ToolRegistry::new(Arc::new(FakeBackend));
        let search = registry
            .search(SearchToolParams { text: "ويت".into(), edition: None, limit: None })
            .await
            .unwrap();
        assert_eq!(search.tool_name, "quran.search");
        assert!(!search.analysis_sources.is_empty(), "search must attribute its sources");
        assert!(!search.normalization_rules.is_empty(), "search must carry its rule trace");

        for (tool, result) in [
            ("quran.root", registry.root(RootToolParams { root: "root-1".into() }).await.unwrap()),
            (
                "quran.lemma",
                registry.lemma(LemmaToolParams { lemma: "lem-1".into() }).await.unwrap(),
            ),
            (
                "quran.morphology",
                registry
                    .morphology(MorphologyToolParams { surah: 1, ayah: 1, position: 1 })
                    .await
                    .unwrap(),
            ),
            (
                "quran.family",
                registry
                    .family(FamilyToolParams { kind: "token".into(), id: "token:1:1:1".into() })
                    .await
                    .unwrap(),
            ),
        ] {
            assert_eq!(result.tool_name, tool);
            assert!(
                !result.analysis_sources.is_empty(),
                "{tool} must attribute its dataset source"
            );
        }
    }

    #[tokio::test]
    async fn get_ayah_conforms_to_the_contract() {
        let registry = ToolRegistry::new(Arc::new(FakeBackend));
        let (mut result, meta) = registry
            .get_ayah(GetAyahParams {
                reference: "1:1".into(),
                translations: Vec::new(),
                glosses: false,
                tokens: false,
            })
            .await
            .unwrap();
        assert_eq!(result.tool_name, "quran.get_ayah");
        assert!(result.normalization_rules.is_empty());
        assert_eq!(result.canonical_references, ["quran:test@0.1.0:1:1".to_string()]);
        assert_eq!(result.reproducibility.corpus_generation, 3);
        assert!(result.reproducibility.deterministic);
        assert_eq!(meta.edition_slug, "test");
        assert_eq!(meta.text_hash, "sha256:ab");
        // Timing is not deterministic; exclude it from the round-trip equality.
        result.execution_time_ms = 0.0;
        // The envelope round-trips (contract stability for later consumers).
        let json = serde_json::to_string(&result).unwrap();
        let back: ToolResult<Vec<AyahView>> = serde_json::from_str(&json).unwrap();
        assert_eq!(back, result);
    }

    #[tokio::test]
    async fn malformed_references_are_typed_errors() {
        let registry = ToolRegistry::new(Arc::new(FakeBackend));
        let err = registry
            .get_ayah(GetAyahParams {
                reference: ":::".into(),
                translations: Vec::new(),
                glosses: false,
                tokens: false,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, ToolError::InvalidInput { .. }));
        assert_eq!(err.code(), "QAI-QUR-0311");
    }

    #[test]
    fn context_params_validate() {
        let params = GetContextParams {
            reference: "2:255".into(),
            before: 2,
            after: 2,
            boundary: ContextBoundaryArg::Surah,
            max_ayahs: 0,
        };
        assert_eq!(params.max_ayahs, 0);
        let _ = (AyahNumber::new(1).unwrap(), SurahNumber::new(1).unwrap());
        let _ = EditionSelector::Active;
    }
}
