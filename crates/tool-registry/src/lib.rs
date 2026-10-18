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
use tools::{AnalysisSource, ToolError, ToolResult, reproducibility, research_checksum};

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
pub const GRAPH_NEIGHBORS_TOOL_VERSION: SemVer = SemVer::new(1, 1, 0);
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

// ============================================================================
// Hadith tools (Phase 6)
// ============================================================================

/// Tool versions for hadith tools (Phase 6).
pub const HADITH_GET_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Tool versions for hadith search (Phase 6).
pub const HADITH_SEARCH_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Tool versions for hadith gradings (Phase 6).
pub const HADITH_GRADINGS_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Tool versions for tafsir get (Phase 6, plan 06-02).
pub const TAFSIR_GET_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Tool versions for tafsir search (Phase 6, plan 06-02).
pub const TAFSIR_SEARCH_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Tool versions for scripture get (Phase 9, plan 09-01).
pub const SCRIPTURE_GET_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Tool versions for scripture search (Phase 9, plan 09-02).
pub const SCRIPTURE_SEARCH_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Tool versions for scripture translations (Phase 9, plan 09-02).
pub const SCRIPTURE_TRANSLATIONS_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Tool versions for scripture editions (Phase 9, plan 09-02).
pub const SCRIPTURE_EDITIONS_VERSION: SemVer = SemVer::new(1, 0, 0);

/// A hadith reference: collection slug + source number.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HadithRef {
    /// Collection slug (e.g., "bukhari-arabic").
    pub collection_slug: String,
    /// Source number, preserved verbatim (e.g., "bukhari:1234").
    pub source_number: String,
}

impl HadithRef {
    /// Parse a reference string like "bukhari-arabic:bukhari:1234".
    pub fn parse(s: &str) -> Option<Self> {
        let (slug, number) = s.split_once(':')?;
        if slug.is_empty() || number.is_empty() {
            return None;
        }
        Some(Self { collection_slug: slug.to_string(), source_number: number.to_string() })
    }
}

impl std::fmt::Display for HadithRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.collection_slug, self.source_number)
    }
}

/// A full hadith view with matn, isnad, and gradings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HadithView {
    /// The hadith record id.
    pub id: String,
    /// Collection slug.
    pub collection_slug: String,
    /// Source number (preserved verbatim).
    pub source_number: String,
    /// Canonical matn text.
    pub matn_text: String,
    /// Isnad text, if separated.
    pub isnad_text: Option<String>,
    /// Narrator names.
    pub narrator_names: Vec<String>,
    /// Per-scholar grading records.
    pub gradings: Vec<GradingRecord>,
}

/// A grading record in the tool result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradingRecord {
    /// The grading record id.
    pub id: String,
    /// Scholar name.
    pub scholar_name: String,
    /// Grading value.
    pub grading_value: String,
    /// Source citation.
    pub source_citation: String,
    /// Review decision.
    pub decision: String,
    /// Reviewer, if any.
    pub reviewer: Option<String>,
    /// Provenance layer.
    pub provenance_layer: String,
    /// Confidence, if any.
    pub confidence: Option<f64>,
}

/// Parameters for `hadith.get`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHadithParams {
    /// Reference string in format `collection_slug:source_number`.
    pub reference: String,
}

/// Parameters for `hadith.search`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HadithSearchToolParams {
    /// Query text.
    pub query: String,
    /// Optional collection filter.
    #[serde(default)]
    pub collection_slug: Option<String>,
    /// Result cap.
    #[serde(default)]
    pub limit: Option<u32>,
}

/// The hadith backend trait: implemented by `application` for the real reader.
#[async_trait]
pub trait HadithBackend: Send + Sync {
    /// Fetch one hadith view by reference.
    async fn backend_get_hadith(
        &self,
        reference: &HadithRef,
    ) -> Result<(HadithView, BackendMeta), ToolError>;

    /// Search hadith records.
    async fn backend_search_hadith(
        &self,
        _params: &HadithSearchToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(ToolError::Backend {
            code: "QAI-HAD-0310".to_string(),
            detail: "hadith.search is not supported by this backend".to_string(),
        })
    }

    /// Fetch grading records for a hadith.
    async fn backend_get_gradings(&self, hadith_id: &str) -> Result<Vec<GradingRecord>, ToolError>;
}

// ============================================================================
// Tafsir tools (Phase 6, plan 06-02)
// ============================================================================

/// A tafsir reference: work slug + optional location inside the work.
///
/// String form is `work-slug[:chapter]` — e.g. `tafsir-tabari:2:255`, where
/// the remainder after the first `:` is the chapter verbatim (it may itself
/// contain `:`). Volume/page are set programmatically, not parsed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TafsirRef {
    /// Work slug (e.g., "tafsir-tabari").
    pub work_slug: String,
    /// Volume number, when the work is multi-volume.
    #[serde(default)]
    pub volume: Option<u32>,
    /// Page number, when the work is paginated.
    #[serde(default)]
    pub page: Option<u32>,
    /// Chapter (e.g. surah name or number), when the work is chaptered.
    #[serde(default)]
    pub chapter: Option<String>,
}

impl TafsirRef {
    /// Parse a reference string like "tafsir-tabari" or "tafsir-tabari:2:255".
    pub fn parse(s: &str) -> Option<Self> {
        match s.split_once(':') {
            Some((slug, chapter)) => {
                if slug.is_empty() || chapter.is_empty() {
                    return None;
                }
                Some(Self {
                    work_slug: slug.to_string(),
                    volume: None,
                    page: None,
                    chapter: Some(chapter.to_string()),
                })
            }
            None => {
                if s.is_empty() {
                    return None;
                }
                Some(Self { work_slug: s.to_string(), volume: None, page: None, chapter: None })
            }
        }
    }
}

impl std::fmt::Display for TafsirRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.work_slug)?;
        if let Some(chapter) = &self.chapter {
            write!(f, ":{chapter}")?;
        }
        Ok(())
    }
}

/// A full tafsir view with text, work, author, and dimensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TafsirView {
    /// The tafsir record id.
    pub id: String,
    /// Work slug.
    pub work_slug: String,
    /// Work name.
    pub work_name: String,
    /// Work author.
    pub author: String,
    /// Volume number, if any.
    pub volume: Option<u32>,
    /// Page number, if any.
    pub page: Option<u32>,
    /// Chapter, if any.
    pub chapter: Option<String>,
    /// Canonical tafsir text.
    pub text: String,
    /// Fully qualified reference (`work_slug:chapter` or `work_slug:volume:page`).
    pub canonical_reference: String,
}

/// Parameters for `tafsir.get`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTafsirParams {
    /// Reference string in format `work_slug[:chapter]`.
    pub reference: String,
}

/// Parameters for `tafsir.search`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TafsirSearchToolParams {
    /// Query text.
    pub query: String,
    /// Optional work filter.
    #[serde(default)]
    pub work_slug: Option<String>,
    /// Result cap.
    #[serde(default)]
    pub limit: Option<u32>,
}

/// The tafsir backend trait: implemented by `application` for the real reader.
#[async_trait]
pub trait TafsirBackend: Send + Sync {
    /// Fetch one tafsir view by reference.
    async fn backend_get_tafsir(
        &self,
        reference: &TafsirRef,
    ) -> Result<(TafsirView, BackendMeta), ToolError>;

    /// Search tafsir records.
    async fn backend_search_tafsir(
        &self,
        _params: &TafsirSearchToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(ToolError::Backend {
            code: "QAI-TAF-0310".to_string(),
            detail: "tafsir.search is not supported by this backend".to_string(),
        })
    }
}

// ============================================================================
// Scripture tools (Phase 9, plan 09-01)
// ============================================================================

/// Re-export of the scripture domain view.
pub use scripture::ScriptureView;

/// A scripture reference: edition + book + chapter + verse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptureRef {
    /// Edition slug or id.
    pub edition: String,
    /// Book slug.
    pub book: String,
    /// Chapter number.
    pub chapter: u32,
    /// Verse number.
    pub verse: u32,
}

impl ScriptureRef {
    /// Parse a reference string `edition:book:chapter:verse`.
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 4 {
            return None;
        }
        let chapter: u32 = parts[2].parse().ok()?;
        let verse: u32 = parts[3].parse().ok()?;
        if chapter == 0 || verse == 0 || parts[0].is_empty() || parts[1].is_empty() {
            return None;
        }
        Some(Self {
            edition: parts[0].to_string(),
            book: parts[1].to_string(),
            chapter,
            verse,
        })
    }
}

impl std::fmt::Display for ScriptureRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}:{}", self.edition, self.book, self.chapter, self.verse)
    }
}

/// Parameters for `scripture.get`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetScriptureParams {
    /// Reference string in format `edition:book:chapter:verse`.
    pub reference: String,
}

/// Parameters for `scripture.search` (placeholder).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptureSearchToolParams {
    /// Query text.
    pub query: String,
    /// Optional edition filter.
    #[serde(default)]
    pub edition: Option<String>,
    /// Result cap.
    #[serde(default)]
    pub limit: Option<u32>,
}

/// The scripture backend trait: implemented by `application`.
#[async_trait]
pub trait ScriptureBackend: Send + Sync {
    /// Fetch one scripture view by reference.
    async fn backend_get_scripture(
        &self,
        reference: &ScriptureRef,
    ) -> Result<(ScriptureView, BackendMeta), ToolError>;

    /// Search scripture (default unsupported).
    async fn backend_search_scripture(
        &self,
        _params: &ScriptureSearchToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(ToolError::Backend {
            code: "QAI-SCR-0310".to_string(),
            detail: "scripture.search is not supported by this backend".to_string(),
        })
    }

    /// List editions (default unsupported).
    async fn backend_list_editions(&self) -> Result<serde_json::Value, ToolError> {
        Err(ToolError::Backend {
            code: "QAI-SCR-0310".to_string(),
            detail: "scripture list editions is not supported by this backend".to_string(),
        })
    }

    /// Get a translation (default unsupported).
    async fn backend_get_translation(&self, _verse_id: &str, _language: &str) -> Result<serde_json::Value, ToolError> {
        Err(ToolError::Backend {
            code: "QAI-SCR-0310".to_string(),
            detail: "scripture translation is not supported by this backend".to_string(),
        })
    }

    /// Parallel passages (default unsupported).
    async fn backend_parallel_passages(&self, _quran_ref: &str) -> Result<serde_json::Value, ToolError> {
        Err(ToolError::Backend {
            code: "QAI-SCR-0310".to_string(),
            detail: "scripture parallel passages is not supported by this backend".to_string(),
        })
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
    /// Projection family to read (default `quran-structural-v1`, the
    /// structural read scope); an unknown or unbuilt family is a typed
    /// `QAI-GRAPH-0001` error, never an empty success.
    #[serde(default)]
    pub projection: Option<String>,
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
    hadith_backend: Option<Arc<dyn HadithBackend>>,
    tafsir_backend: Option<Arc<dyn TafsirBackend>>,
    scripture_backend: Option<Arc<dyn ScriptureBackend>>,
    isnad_backend: Option<Arc<dyn IsnadBackend>>,
    rag_backend: Option<Arc<dyn RagBackend>>,
}

impl ToolRegistry {
    /// Wrap a backend.
    pub fn new(backend: Arc<dyn QuranBackend>) -> Self {
        Self { backend, hadith_backend: None, tafsir_backend: None, scripture_backend: None, isnad_backend: None, rag_backend: None }
    }

    /// Wrap a backend with a hadith backend.
    pub fn with_hadith(
        backend: Arc<dyn QuranBackend>,
        hadith_backend: Arc<dyn HadithBackend>,
    ) -> Self {
        Self {
            backend,
            hadith_backend: Some(hadith_backend),
            tafsir_backend: None,
            scripture_backend: None,
            isnad_backend: None,
            rag_backend: None,
        }
    }

    /// Set the hadith backend after construction.
    pub fn set_hadith_backend(&mut self, hadith_backend: Arc<dyn HadithBackend>) {
        self.hadith_backend = Some(hadith_backend);
    }

    /// Wrap a backend with an isnad backend.
    pub fn with_isnad(
        backend: Arc<dyn QuranBackend>,
        isnad_backend: Arc<dyn IsnadBackend>,
    ) -> Self {
        Self {
            backend,
            hadith_backend: None,
            tafsir_backend: None,
            scripture_backend: None,
            isnad_backend: Some(isnad_backend),
            rag_backend: None,
        }
    }

    /// Set the isnad backend after construction.
    pub fn set_isnad_backend(&mut self, isnad_backend: Arc<dyn IsnadBackend>) {
        self.isnad_backend = Some(isnad_backend);
    }

    pub fn isnad_backend(&self) -> Option<Arc<dyn IsnadBackend>> {
        self.isnad_backend.clone()
    }

    /// Wrap a backend with a RAG backend.
    pub fn with_rag(
        backend: Arc<dyn QuranBackend>,
        rag_backend: Arc<dyn RagBackend>,
    ) -> Self {
        Self {
            backend,
            hadith_backend: None,
            tafsir_backend: None,
            scripture_backend: None,
            isnad_backend: None,
            rag_backend: Some(rag_backend),
        }
    }

    /// Set the RAG backend after construction.
    pub fn set_rag_backend(&mut self, rag_backend: Arc<dyn RagBackend>) {
        self.rag_backend = Some(rag_backend);
    }

    pub fn rag_backend(&self) -> Option<Arc<dyn RagBackend>> {
        self.rag_backend.clone()
    }

    pub async fn rag_create_project(
        &self,
        name: &str,
        config_toml: &str,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let b = self.rag_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-RAG-0310".into(),
            detail: "rag backend not configured".into(),
        })?;
        b.backend_create_project(name, config_toml).await
    }

    pub async fn rag_import(
        &self,
        project: &str,
        docs: &[serde_json::Value],
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let b = self.rag_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-RAG-0310".into(),
            detail: "rag backend not configured".into(),
        })?;
        b.backend_import(project, docs).await
    }

    pub async fn rag_query(
        &self,
        project: &str,
        query: &str,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let b = self.rag_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-RAG-0310".into(),
            detail: "rag backend not configured".into(),
        })?;
        b.backend_query(project, query).await
    }

    pub async fn rag_debug(
        &self,
        project: &str,
        query: &str,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let b = self.rag_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-RAG-0310".into(),
            detail: "rag backend not configured".into(),
        })?;
        b.backend_debug(project, query).await
    }

    pub async fn rag_source_status(
        &self,
        project: &str,
        source: &str,
        status: &str,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let b = self.rag_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-RAG-0310".into(),
            detail: "rag backend not configured".into(),
        })?;
        b.backend_source_status(project, source, status).await
    }

    /// Wrap a backend with a tafsir backend.
    pub fn with_tafsir(
        backend: Arc<dyn QuranBackend>,
        tafsir_backend: Arc<dyn TafsirBackend>,
    ) -> Self {
        Self {
            backend,
            hadith_backend: None,
            tafsir_backend: Some(tafsir_backend),
            scripture_backend: None,
            isnad_backend: None,
            rag_backend: None,
        }
    }

    /// Set the tafsir backend after construction.
    pub fn set_tafsir_backend(&mut self, tafsir_backend: Arc<dyn TafsirBackend>) {
        self.tafsir_backend = Some(tafsir_backend);
    }

    /// Wrap a backend with a scripture backend.
    pub fn with_scripture(
        backend: Arc<dyn QuranBackend>,
        scripture_backend: Arc<dyn ScriptureBackend>,
    ) -> Self {
        Self {
            backend,
            hadith_backend: None,
            tafsir_backend: None,
            scripture_backend: Some(scripture_backend),
            isnad_backend: None,
            rag_backend: None,
        }
    }

    /// Set the scripture backend after construction.
    pub fn set_scripture_backend(&mut self, scripture_backend: Arc<dyn ScriptureBackend>) {
        self.scripture_backend = Some(scripture_backend);
    }

    pub fn scripture_backend(&self) -> Option<Arc<dyn ScriptureBackend>> {
        self.scripture_backend.clone()
    }

    /// The full registered tool set (§12 tool plan; D-13; Phase 6 hadith + tafsir tools).
    pub const TOOL_NAMES: [&'static str; 21] = [
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
        "hadith.get",
        "hadith.search",
        "hadith.gradings",
        "tafsir.get",
        "tafsir.search",
        "scripture.get",
        "scripture.search",
        "scripture.translations",
        "scripture.editions",
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
        let canonical = view.canonical.reference().to_string();
        let results = vec![view];
        let payload = serde_json::to_value(&results).unwrap_or(serde_json::Value::Null);
        let result = ToolResult {
            tool_name: "quran.get_ayah".to_string(),
            tool_version: GET_AYAH_VERSION,
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: Some(meta.edition_id.clone()),
            edition_version: Some(meta.edition_version.clone()),
            canonical_references: vec![canonical.clone()],
            analysis_sources: vec![AnalysisSource {
                kind: "canonical".to_string(),
                reference: canonical,
            }],
            results,
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
            research_checksum: research_checksum(
                "quran.get_ayah",
                GET_AYAH_VERSION,
                &query,
                &serde_json::json!({
                    "reference": params.reference,
                    "edition_slug": meta.edition_slug,
                    "edition_version": meta.edition_version,
                    "corpus_generation": meta.corpus_generation,
                }),
                &payload,
            )
            .expect("research checksum input is JSON-serializable"),
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
        let payload = serde_json::to_value(&view).unwrap_or(serde_json::Value::Null);
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
            research_checksum: research_checksum(
                "quran.get_context",
                GET_CONTEXT_VERSION,
                &query,
                &serde_json::json!({
                    "reference": params.reference,
                    "edition_slug": meta.edition_slug,
                    "edition_version": meta.edition_version,
                    "corpus_generation": meta.corpus_generation,
                }),
                &payload,
            )
            .expect("research checksum input is JSON-serializable"),
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

    /// `hadith.get`: fetch one hadith view by reference.
    pub async fn get_hadith(
        &self,
        params: GetHadithParams,
    ) -> Result<(ToolResult<HadithView>, BackendMeta), ToolError> {
        let hadith_backend = self.hadith_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-HAD-0310".to_string(),
            detail: "hadith backend not configured".to_string(),
        })?;
        let started = Instant::now();
        let reference =
            HadithRef::parse(&params.reference).ok_or_else(|| ToolError::InvalidInput {
                tool: "hadith.get",
                detail: "reference must be in format `collection_slug:source_number`".to_string(),
            })?;
        let query = serde_json::to_value(&params).unwrap_or(serde_json::Value::Null);
        let (view, meta) = hadith_backend.backend_get_hadith(&reference).await?;
        let canonical = format!("{}:{}", view.collection_slug, view.source_number);
        let payload = serde_json::to_value(&view).unwrap_or(serde_json::Value::Null);
        let result = ToolResult {
            tool_name: "hadith.get".to_string(),
            tool_version: HADITH_GET_VERSION,
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: Some(meta.edition_id.clone()),
            edition_version: Some(meta.edition_version.clone()),
            canonical_references: vec![canonical.clone()],
            analysis_sources: vec![AnalysisSource {
                kind: "hadith".to_string(),
                reference: canonical,
            }],
            results: view,
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                "hadith.get",
                HADITH_GET_VERSION,
                &query,
                Some(&meta.edition_slug),
                Some(&meta.edition_version),
                BTreeMap::new(),
                meta.corpus_generation,
            ),
            research_checksum: research_checksum(
                "hadith.get",
                HADITH_GET_VERSION,
                &query,
                &serde_json::json!({
                    "reference": params.reference,
                    "edition_slug": meta.edition_slug,
                    "edition_version": meta.edition_version,
                    "corpus_generation": meta.corpus_generation,
                }),
                &payload,
            )
            .expect("research checksum input is JSON-serializable"),
        };
        Ok((result, meta))
    }

    /// `hadith.search`: normalized search over hadith records.
    pub async fn search_hadith(
        &self,
        params: HadithSearchToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        if params.query.trim().is_empty() {
            return Err(ToolError::InvalidInput {
                tool: "hadith.search",
                detail: "query text must not be empty".to_string(),
            });
        }
        let hadith_backend = self.hadith_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-HAD-0310".to_string(),
            detail: "hadith backend not configured".to_string(),
        })?;
        hadith_backend.backend_search_hadith(&params).await
    }

    /// `hadith.gradings`: fetch grading records for a hadith.
    pub async fn get_gradings(
        &self,
        hadith_id: &str,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let hadith_backend = self.hadith_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-HAD-0310".to_string(),
            detail: "hadith backend not configured".to_string(),
        })?;
        let started = Instant::now();
        let query = serde_json::json!({ "hadith_id": hadith_id });
        let gradings = hadith_backend.backend_get_gradings(hadith_id).await?;
        let results = serde_json::to_value(&gradings).unwrap_or(serde_json::Value::Null);
        let payload = results.clone();
        let canonical = vec![hadith_id.to_string()];
        let result = ToolResult {
            tool_name: "hadith.gradings".to_string(),
            tool_version: HADITH_GRADINGS_VERSION,
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: None,
            edition_version: None,
            canonical_references: canonical,
            analysis_sources: vec![AnalysisSource {
                kind: "hadith".to_string(),
                reference: hadith_id.to_string(),
            }],
            results,
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                "hadith.gradings",
                HADITH_GRADINGS_VERSION,
                &query,
                None,
                None,
                BTreeMap::new(),
                0,
            ),
            research_checksum: research_checksum(
                "hadith.gradings",
                HADITH_GRADINGS_VERSION,
                &query,
                &serde_json::json!({}),
                &payload,
            )
            .expect("research checksum input is JSON-serializable"),
        };
        Ok(result)
    }

    /// `tafsir.get`: fetch one tafsir view by reference.
    pub async fn get_tafsir(
        &self,
        params: GetTafsirParams,
    ) -> Result<(ToolResult<TafsirView>, BackendMeta), ToolError> {
        let tafsir_backend = self.tafsir_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-TAF-0310".to_string(),
            detail: "tafsir backend not configured".to_string(),
        })?;
        let started = Instant::now();
        let reference =
            TafsirRef::parse(&params.reference).ok_or_else(|| ToolError::InvalidInput {
                tool: "tafsir.get",
                detail: "reference must be in format `work_slug[:chapter]`".to_string(),
            })?;
        let query = serde_json::to_value(&params).unwrap_or(serde_json::Value::Null);
        let (view, meta) = tafsir_backend.backend_get_tafsir(&reference).await?;
        let canonical = view.canonical_reference.clone();
        let payload = serde_json::to_value(&view).unwrap_or(serde_json::Value::Null);
        let result = ToolResult {
            tool_name: "tafsir.get".to_string(),
            tool_version: TAFSIR_GET_VERSION,
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: Some(meta.edition_id.clone()),
            edition_version: Some(meta.edition_version.clone()),
            canonical_references: vec![canonical.clone()],
            analysis_sources: vec![AnalysisSource {
                kind: "tafsir".to_string(),
                reference: canonical,
            }],
            results: view,
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                "tafsir.get",
                TAFSIR_GET_VERSION,
                &query,
                Some(&meta.edition_slug),
                Some(&meta.edition_version),
                BTreeMap::new(),
                meta.corpus_generation,
            ),
            research_checksum: research_checksum(
                "tafsir.get",
                TAFSIR_GET_VERSION,
                &query,
                &serde_json::json!({
                    "reference": params.reference,
                    "edition_slug": meta.edition_slug,
                    "edition_version": meta.edition_version,
                    "corpus_generation": meta.corpus_generation,
                }),
                &payload,
            )
            .expect("research checksum input is JSON-serializable"),
        };
        Ok((result, meta))
    }

    /// `scripture.get`: fetch one scripture view by reference.
    pub async fn get_scripture(
        &self,
        params: GetScriptureParams,
    ) -> Result<(ToolResult<ScriptureView>, BackendMeta), ToolError> {
        let scripture_backend = self.scripture_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-SCR-0310".to_string(),
            detail: "scripture backend not configured".to_string(),
        })?;
        let started = Instant::now();
        let reference =
            ScriptureRef::parse(&params.reference).ok_or_else(|| ToolError::InvalidInput {
                tool: "scripture.get",
                detail: "reference must be in format `edition:book:chapter:verse`".to_string(),
            })?;
        let query = serde_json::to_value(&params).unwrap_or(serde_json::Value::Null);
        let (view, meta) = scripture_backend.backend_get_scripture(&reference).await?;
        let canonical = view.verse.id.to_string();
        let payload = serde_json::to_value(&view).unwrap_or(serde_json::Value::Null);
        let result = ToolResult {
            tool_name: "scripture.get".to_string(),
            tool_version: SCRIPTURE_GET_VERSION,
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: Some(meta.edition_id.clone()),
            edition_version: Some(meta.edition_version.clone()),
            canonical_references: vec![canonical.clone()],
            analysis_sources: vec![AnalysisSource {
                kind: "scripture".to_string(),
                reference: canonical,
            }],
            results: view,
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                "scripture.get",
                SCRIPTURE_GET_VERSION,
                &query,
                Some(&meta.edition_slug),
                Some(&meta.edition_version),
                BTreeMap::new(),
                meta.corpus_generation,
            ),
            research_checksum: research_checksum(
                "scripture.get",
                SCRIPTURE_GET_VERSION,
                &query,
                &serde_json::json!({
                    "reference": params.reference,
                    "edition_slug": meta.edition_slug,
                    "edition_version": meta.edition_version,
                    "corpus_generation": meta.corpus_generation,
                }),
                &payload,
            )
            .expect("research checksum input is JSON-serializable"),
        };
        Ok((result, meta))
    }

    /// `scripture.search`: normalized search over scripture verses.
    pub async fn search_scripture(
        &self,
        params: ScriptureSearchToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        if params.query.trim().is_empty() {
            return Err(ToolError::InvalidInput {
                tool: "scripture.search",
                detail: "query text must not be empty".to_string(),
            });
        }
        let scripture_backend = self.scripture_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-SCR-0310".to_string(),
            detail: "scripture backend not configured".to_string(),
        })?;
        scripture_backend.backend_search_scripture(&params).await
    }

    /// `scripture.translations`: fetch translations for a verse.
    pub async fn get_scripture_translations(
        &self,
        verse_id: String,
        language: Option<String>,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let scripture_backend = self.scripture_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-SCR-0310".to_string(),
            detail: "scripture backend not configured".to_string(),
        })?;
        let lang = language.unwrap_or_default();
        let value = scripture_backend.backend_get_translation(&verse_id, &lang).await?;
        // Wrap raw value into ToolResult? backend returns raw Value, not ToolResult. 
        // To keep typed tool parity, return via backend directly is not ToolResult. 
        // But ToolRegistry methods should return ToolResult. Hmm.
        Ok(ToolResult {
            tool_name: "scripture.translations".to_string(),
            tool_version: SCRIPTURE_TRANSLATIONS_VERSION,
            query: serde_json::json!({"verse_id": verse_id, "language": lang}),
            normalization_rules: Vec::new(),
            edition_id: None,
            edition_version: None,
            results: value.clone(),
            canonical_references: vec![verse_id.clone()],
            analysis_sources: vec![AnalysisSource { kind: "scripture".to_string(), reference: verse_id }],
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: 0.0,
            reproducibility: reproducibility("scripture.translations", SCRIPTURE_TRANSLATIONS_VERSION, &serde_json::json!({}), None, None, BTreeMap::new(), 0),
            research_checksum: research_checksum("scripture.translations", SCRIPTURE_TRANSLATIONS_VERSION, &serde_json::json!({}), &serde_json::json!({}), &value).expect("checksum"),
        })
    }

    /// `scripture.editions`: list editions.
    pub async fn list_scripture_editions(
        &self,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let scripture_backend = self.scripture_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-SCR-0310".to_string(),
            detail: "scripture backend not configured".to_string(),
        })?;
        let value = scripture_backend.backend_list_editions().await?;
        Ok(ToolResult {
            tool_name: "scripture.editions".to_string(),
            tool_version: SCRIPTURE_EDITIONS_VERSION,
            query: serde_json::json!({}),
            normalization_rules: Vec::new(),
            edition_id: None,
            edition_version: None,
            results: value.clone(),
            canonical_references: Vec::new(),
            analysis_sources: Vec::new(),
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: 0.0,
            reproducibility: reproducibility("scripture.editions", SCRIPTURE_EDITIONS_VERSION, &serde_json::json!({}), None, None, BTreeMap::new(), 0),
            research_checksum: research_checksum("scripture.editions", SCRIPTURE_EDITIONS_VERSION, &serde_json::json!({}), &serde_json::json!({}), &value).expect("checksum"),
        })
    }

    /// `tafsir.search`: normalized search over tafsir records.
    pub async fn search_tafsir(
        &self,
        params: TafsirSearchToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        if params.query.trim().is_empty() {
            return Err(ToolError::InvalidInput {
                tool: "tafsir.search",
                detail: "query text must not be empty".to_string(),
            });
        }
        let tafsir_backend = self.tafsir_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-TAF-0310".to_string(),
            detail: "tafsir backend not configured".to_string(),
        })?;
        tafsir_backend.backend_search_tafsir(&params).await
    }

    /// `isnad.narrator`: fetch a narrator record + rijal sources.
    pub async fn get_narrator(&self, id: &str) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let b = self.isnad_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-ISNAD-0310".into(),
            detail: "isnad backend not configured".into(),
        })?;
        b.backend_get_narrator(id).await
    }
    pub async fn get_rijal(
        &self,
        narrator_id: &str,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let b = self.isnad_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-ISNAD-0310".into(),
            detail: "isnad backend not configured".into(),
        })?;
        b.backend_get_rijal(narrator_id).await
    }
    pub async fn trace(
        &self,
        params: TraceParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let b = self.isnad_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-ISNAD-0310".into(),
            detail: "isnad backend not configured".into(),
        })?;
        b.backend_trace(&params).await
    }
    pub async fn propose_identity(
        &self,
        params: IdentityProposalParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let b = self.isnad_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-ISNAD-0310".into(),
            detail: "isnad backend not configured".into(),
        })?;
        b.backend_propose_identity(&params).await
    }
    pub async fn accept_identity(
        &self,
        params: IdentityDecisionParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let b = self.isnad_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-ISNAD-0310".into(),
            detail: "isnad backend not configured".into(),
        })?;
        b.backend_accept_identity(&params).await
    }
    pub async fn reject_identity(
        &self,
        params: IdentityDecisionParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let b = self.isnad_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-ISNAD-0310".into(),
            detail: "isnad backend not configured".into(),
        })?;
        b.backend_reject_identity(&params).await
    }
    pub async fn dispute_identity(
        &self,
        params: IdentityDecisionParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let b = self.isnad_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-ISNAD-0310".into(),
            detail: "isnad backend not configured".into(),
        })?;
        b.backend_dispute_identity(&params).await
    }
    pub async fn supersede_identity(
        &self,
        id: &str,
        new_proposal: IdentityProposalParams,
        reviewer: &str,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let b = self.isnad_backend.as_ref().ok_or_else(|| ToolError::Backend {
            code: "QAI-ISNAD-0310".into(),
            detail: "isnad backend not configured".into(),
        })?;
        b.backend_supersede_identity(id, &new_proposal, reviewer).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quran_core::{AyahNumber, EditionSelector, SurahNumber};

    #[test]
    fn mutating_isnad_tools_are_classified() {
        // WR-08: the mutation gate keys off this predicate, so it must name
        // exactly the INSERT/UPDATE isnad tools and nothing read-only.
        for name in [
            "isnad.identity_propose",
            "isnad.identity_accept",
            "isnad.identity_reject",
            "isnad.identity_dispute",
            "isnad.identity_supersede",
        ] {
            assert!(is_mutating_isnad_tool(name), "{name} must be classified mutating");
        }
        for name in
            ["isnad.narrator", "isnad.rijal", "isnad.trace", "quran.get_ayah", "hadith.get", "nope"]
        {
            assert!(!is_mutating_isnad_tool(name), "{name} must not be classified mutating");
        }
    }

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
        let results = serde_json::json!({ "ok": true });
        let payload = serde_json::to_value(&results).unwrap_or(serde_json::Value::Null);
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
            results,
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
                &payload,
            )
            .expect("research checksum input is JSON-serializable"),
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
                "hadith.get",
                "hadith.search",
                "hadith.gradings",
                "tafsir.get",
                "tafsir.search",
                "scripture.get",
                "scripture.search",
                "scripture.translations",
                "scripture.editions",
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

// ============================================================================
// Isnad tools (Phase 7)
// ============================================================================

/// Parameters for `isnad.narrator`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetNarratorParams {
    pub id: String,
}

/// Parameters for `isnad.trace`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceParams {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub k: Option<usize>,
}

/// Parameters for proposing a narrator identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityProposalParams {
    pub claim: serde_json::Value,
    pub evidence: serde_json::Value,
    #[serde(default)]
    pub source_location: String,
    pub provenance_layer: String,
    #[serde(default)]
    pub algorithm: Option<String>,
    #[serde(default)]
    pub algorithm_version: Option<String>,
    #[serde(default)]
    pub confidence: Option<f64>,
}

/// Parameters for an identity decision (accept/reject/dispute).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityDecisionParams {
    pub id: String,
    pub reviewer: String,
}

/// The isnad backend trait.
#[async_trait]
pub trait IsnadBackend: Send + Sync {
    async fn backend_get_narrator(
        &self,
        id: &str,
    ) -> Result<ToolResult<serde_json::Value>, ToolError>;
    async fn backend_get_rijal(
        &self,
        narrator_id: &str,
    ) -> Result<ToolResult<serde_json::Value>, ToolError>;
    async fn backend_trace(
        &self,
        params: &TraceParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError>;
    async fn backend_propose_identity(
        &self,
        params: &IdentityProposalParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError>;
    async fn backend_accept_identity(
        &self,
        params: &IdentityDecisionParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError>;
    async fn backend_reject_identity(
        &self,
        params: &IdentityDecisionParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError>;
    async fn backend_dispute_identity(
        &self,
        params: &IdentityDecisionParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError>;
    async fn backend_supersede_identity(
        &self,
        id: &str,
        new_proposal: &IdentityProposalParams,
        reviewer: &str,
    ) -> Result<ToolResult<serde_json::Value>, ToolError>;
}

/// Isnad tool names.
pub const ISNAD_TOOL_NAMES: [&str; 6] = [
    "isnad.narrator",
    "isnad.rijal",
    "isnad.trace",
    "isnad.identity_propose",
    "isnad.identity_accept",
    "isnad.identity_reject",
];

/// Isnad tools that write durable state (INSERT/UPDATE). The read-only isnad
/// tools (`isnad.narrator`, `isnad.rijal`, `isnad.trace`) are excluded.
///
/// WR-08: the generic HTTP tool route is unauthenticated, so the server uses
/// this to deny every mutating isnad tool unless explicitly opted in
/// (`policy.tool_execution_default = "allow"`), rather than exposing
/// INSERT/UPDATE capabilities on a read-only route by default.
pub fn is_mutating_isnad_tool(name: &str) -> bool {
    matches!(
        name,
        "isnad.identity_propose"
            | "isnad.identity_accept"
            | "isnad.identity_reject"
            | "isnad.identity_dispute"
            | "isnad.identity_supersede"
    )
}

// ============================================================================
// RAG tools (Phase 8)
// ============================================================================

/// The RAG backend trait: implemented by `application` for the real reader.
#[async_trait]
pub trait RagBackend: Send + Sync {
    async fn backend_create_project(
        &self,
        name: &str,
        config_toml: &str,
    ) -> Result<ToolResult<serde_json::Value>, ToolError>;
    async fn backend_import(
        &self,
        project: &str,
        docs: &[serde_json::Value],
    ) -> Result<ToolResult<serde_json::Value>, ToolError>;
    async fn backend_query(
        &self,
        project: &str,
        query: &str,
    ) -> Result<ToolResult<serde_json::Value>, ToolError>;
    async fn backend_debug(
        &self,
        project: &str,
        query: &str,
    ) -> Result<ToolResult<serde_json::Value>, ToolError>;
    async fn backend_source_status(
        &self,
        project: &str,
        source: &str,
        status: &str,
    ) -> Result<ToolResult<serde_json::Value>, ToolError>;
}

/// RAG tool names.
pub const RAG_TOOL_NAMES: [&str; 9] = [
    "rag.create_project",
    "rag.import",
    "rag.query",
    "rag.debug",
    "rag.source_status",
    "rag.source_activate",
    "rag.source_deactivate",
    "rag.source_tombstone",
    "rag.create",
];

/// RAG tools that write durable state.
pub fn is_mutating_rag_tool(name: &str) -> bool {
    matches!(
        name,
        "rag.create_project"
            | "rag.create"
            | "rag.import"
            | "rag.source_status"
            | "rag.source_activate"
            | "rag.source_deactivate"
            | "rag.source_tombstone"
    )
}
