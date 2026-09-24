//! Phase 2 — backend-independent full-text types (`plan.md` §4.1, P2-T29).
//!
//! These types are the **port**: index backends (FTS5 now, Tantivy later)
//! implement [`FullTextIndex`](crate::index::FullTextIndex) over them, and no
//! backend-specific query, document address, or schema type may escape the
//! adapter. The query path and the build path share the normalization
//! pipeline, so a query and a document can never disagree (parity-tested at
//! `tests/search/parity.rs`, P2-T32).

use std::collections::BTreeMap;
use std::time::Duration;

use domain::SemVer;
use serde::{Deserialize, Serialize};

/// Index backend selector. Only `Fts5` ships in Phase 2 (DEV-05); the other
/// variants name the extension points so callers never hard-code FTS5 query
/// syntax through this port.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FtsBackend {
    /// SQLite FTS5 virtual tables (Phase-2 implementation backend).
    Fts5,
    /// Tantivy adapter (deferred; see DEV-05).
    Tantivy,
    /// Server-side search (Phase 11+; not embedded).
    OpenSearch,
}

impl std::fmt::Display for FtsBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fts5 => f.write_str("fts5"),
            Self::Tantivy => f.write_str("tantivy"),
            Self::OpenSearch => f.write_str("opensearch"),
        }
    }
}

/// Which normalized profile a text field holds. Field names are stable API:
/// `text_exact`, `text_ws`, `text_marks`, `text_bare`, `text_hamza`,
/// `text_folded`, `text_affix`.
pub type FieldId = String;

/// A typed lexical query. Backends translate this to their native query
/// language; callers never write backend syntax.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FtsQuery {
    /// Single term against one field.
    Term {
        /// Indexed field.
        field: FieldId,
        /// Normalized term.
        term: String,
    },
    /// Positional phrase with slop (`ordered=false` = unordered-near).
    Phrase {
        /// Indexed field.
        field: FieldId,
        /// Normalized terms in order.
        terms: Vec<String>,
        /// Maximum intervening tokens.
        slop: u32,
        /// Whether order is enforced.
        ordered: bool,
    },
    /// Boolean combination.
    Boolean {
        /// All must match.
        must: Vec<FtsQuery>,
        /// At least one should match.
        should: Vec<FtsQuery>,
        /// None may match.
        must_not: Vec<FtsQuery>,
    },
    /// Integer metadata range (surah, juz, page, global index…).
    Range {
        /// Metadata field.
        field: FieldId,
        /// Inclusive lower bound.
        lo: Option<i64>,
        /// Inclusive upper bound.
        hi: Option<i64>,
    },
    /// Bounded regex over an indexed field (I16 guards apply at the tool
    /// layer; the backend additionally enforces its own budgets).
    Regex {
        /// Indexed normalized field (never a raw canonical scan).
        field: FieldId,
        /// DFA-safe pattern.
        pattern: String,
    },
    /// Match everything (count/verify paths).
    All,
}

/// Metadata filters shared by every search tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Filter {
    /// Restrict to surahs.
    Surah(Vec<u16>),
    /// Restrict to a juz range (inclusive).
    JuzRange(u16, u16),
    /// Restrict to pages.
    Page(Vec<u32>),
    /// Restrict to revelation place (`makki`/`madani`).
    RevelationPlace(String),
    /// Restrict to a global ayah-index range (inclusive).
    GlobalRange(u64, u64),
}

/// Result ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResultOrder {
    /// Canonical (surah, ayah) order — the default for research tools.
    CanonicalOrder,
    /// Backend relevance (BM25) order.
    Relevance,
}

/// Search execution options.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchOpts {
    /// Hard-capped result count (absolute ceiling 1000, §40).
    pub limit: u32,
    /// Result offset for paging.
    pub offset: u32,
    /// Metadata filters (AND-combined).
    pub filters: Vec<Filter>,
    /// Ordering.
    pub order: ResultOrder,
    /// Whether to compute highlight ranges.
    pub highlight: bool,
    /// Per-query timeout in milliseconds (default 2000, ceiling 10000).
    pub timeout_ms: u64,
    /// Whether to include the per-hit scoring breakdown.
    pub explain: bool,
}

impl Default for SearchOpts {
    fn default() -> Self {
        Self {
            limit: 100,
            offset: 0,
            filters: Vec::new(),
            order: ResultOrder::CanonicalOrder,
            highlight: false,
            timeout_ms: 2000,
            explain: false,
        }
    }
}

impl SearchOpts {
    /// Validate caps; clamps `limit`/`timeout_ms` to their ceilings.
    #[must_use]
    pub fn normalized(mut self) -> Self {
        self.limit = self.limit.min(1000);
        self.timeout_ms = self.timeout_ms.clamp(1, 10_000);
        self
    }

    /// Borrow as a [`Duration`] for async timeouts.
    #[must_use]
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }
}

/// One indexable retrieval unit (ayah granularity in Phase 2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FtsDoc {
    /// Stable id (`<index-id>:<edition>:<surah>:<ayah>`).
    pub id: String,
    /// Canonical edition id.
    pub edition_id: String,
    /// Surah number.
    pub surah: u16,
    /// Ayah number.
    pub ayah: u16,
    /// Global ayah index.
    pub global_index: u64,
    /// Corpus generation at index time (for atomic activation).
    pub generation: u64,
    /// Juz / page / revelation metadata for filtering.
    pub metadata: BTreeMap<String, String>,
    /// Normalized text fields by [`FieldId`].
    pub fields: BTreeMap<FieldId, String>,
    /// Optional morphology projection fields (`roots`, `lemmas`, `stems`,
    /// `pos_tags`, and `patterns`). They are kept separate from text-profile
    /// fields so lexicon values are never silently normalized as ayah text.
    #[serde(default)]
    pub lexicon_fields: BTreeMap<FieldId, String>,
}

/// One ranked (or canonically ordered) hit with backend evidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FtsHit {
    /// Stable retrieval-unit id (resolves through [`FtsDoc::id`]).
    pub doc_id: String,
    /// Backend score (`None` for canonical-order tools).
    pub score: Option<f32>,
    /// Field that matched.
    pub matched_field: FieldId,
}

/// Search results with an exact total (never estimated).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FtsResults {
    /// Hits in the requested order.
    pub hits: Vec<FtsHit>,
    /// Exact match count from the separate count path.
    pub total_matches: u64,
    /// True when `limit` truncated the hit list.
    pub truncated: bool,
    /// Regex expansion terms (empty unless the query was a regex).
    #[serde(default)]
    pub regex_terms: Vec<String>,
    /// Dictionary terms examined for a regex (0 unless regex).
    #[serde(default)]
    pub terms_examined: u64,
}

/// Index build inputs: generation + every version that must match (I14).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexManifest {
    /// Index identity (`quran.ayah.v1`).
    pub index_id: String,
    /// Index schema version.
    pub schema_version: u32,
    /// Corpus generation indexed.
    pub corpus_generation: u64,
    /// Canonical edition id.
    pub edition_id: String,
    /// Canonical edition version.
    #[serde(with = "semver_serde")]
    pub edition_version: SemVer,
    /// Profile id → ladder version.
    #[serde(default)]
    pub rule_set_versions: BTreeMap<String, SemVer>,
    /// Tokenizer version.
    #[serde(with = "semver_serde")]
    pub tokenizer_version: SemVer,
    /// Morphology dataset slug → version (empty until M4).
    #[serde(default)]
    pub morphology_dataset_versions: BTreeMap<String, SemVer>,
    /// Build timestamp (RFC 3339).
    pub built_at: String,
    /// Documents committed.
    pub doc_count: u64,
    /// Trigram posting rows in `trigram.db` (T36; 0 for pre-T36 generations).
    #[serde(default)]
    pub trigram_postings: u64,
    /// Content hash over the manifest (drift detection).
    pub content_hash: String,
}

/// String-form `SemVer` serde (mirrors `quran-normalization`).
mod semver_serde {
    use domain::SemVer;
    use serde::de::Error as _;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(version: &SemVer, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&version.to_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<SemVer, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(D::Error::custom)
    }
}

/// Index schema: which fields exist (backend creates them).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FtsSchema {
    /// Index identity.
    pub index_id: String,
    /// Schema version.
    pub schema_version: u32,
    /// Text fields (profile field names).
    pub fields: Vec<FieldId>,
}

/// Commit receipt for one build batch or generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitStamp {
    /// Generation committed.
    pub generation: u64,
    /// Documents visible after commit.
    pub doc_count: u64,
    /// Manifest content hash committed.
    pub content_hash: String,
}

/// Backend statistics for `doctor` and the build report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FtsStats {
    /// Backend in use.
    pub backend: FtsBackend,
    /// Documents served.
    pub doc_count: u64,
    /// Generation currently serving.
    pub generation: u64,
}

/// Integrity report from [`FullTextIndex::verify`](crate::index::FullTextIndex::verify).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FtsIntegrityReport {
    /// Whether the index passed all checks.
    pub ok: bool,
    /// Documents counted.
    pub doc_count: u64,
    /// Human-readable findings (empty when `ok`).
    pub findings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opts_clamp_to_ceilings() {
        let opts = SearchOpts { limit: 99_999, timeout_ms: 99_999, ..SearchOpts::default() };
        let opts = opts.normalized();
        assert_eq!(opts.limit, 1000);
        assert_eq!(opts.timeout_ms, 10_000);
        assert_eq!(opts.timeout(), Duration::from_secs(10));
    }

    #[test]
    fn query_and_manifest_round_trip_json() {
        let query = FtsQuery::Phrase {
            field: "text_bare".to_string(),
            terms: vec!["الحمد".to_string(), "لله".to_string()],
            slop: 0,
            ordered: true,
        };
        let json = serde_json::to_string(&query).unwrap();
        let back: FtsQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(back, query);

        let manifest = IndexManifest {
            index_id: "quran.ayah.v1".to_string(),
            schema_version: 1,
            corpus_generation: 7,
            edition_id: "ed-1".to_string(),
            edition_version: SemVer::new(1, 0, 0),
            rule_set_versions: BTreeMap::from([(
                "L3.diacritics".to_string(),
                SemVer::new(1, 0, 0),
            )]),
            tokenizer_version: SemVer::new(1, 0, 0),
            morphology_dataset_versions: BTreeMap::new(),
            built_at: "2026-09-15T00:00:00Z".to_string(),
            doc_count: 6236,
            trigram_postings: 0,
            content_hash: "sha256:ab".to_string(),
        };
        let json = serde_json::to_string(&manifest).unwrap();
        assert!(json.contains("\"edition_version\":\"1.0.0\""), "{json}");
        let back: IndexManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, manifest);
    }
}
