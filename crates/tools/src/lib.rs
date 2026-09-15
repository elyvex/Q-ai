//! Tool result contract (§12) and reproducibility checksums (§12.1).
//!
//! # tools
//!
//! Every tool in every phase conforms to [`ToolResult`]: the exact query, the
//! editions and versions read, canonical references for every result, and a
//! deterministic [`ReproducibilityData`] checksum. Phase 1 covers the
//! deterministic inputs; model/prompt fields arrive in Phase 9.

use std::collections::BTreeMap;

use domain::{Confidence, ContentHash, HashAlgorithm, SemVer};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// One analysis source behind a tool result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisSource {
    /// Source kind (`canonical`, `translation`, `metadata`).
    pub kind: String,
    /// Canonical reference or location.
    pub reference: String,
}

/// Deterministic reproducibility record (§12.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReproducibilityData {
    /// Checksum over the deterministic inputs.
    pub checksum: ContentHash,
    /// Checksum over the exact query.
    pub query_hash: ContentHash,
    /// Tool plan as `name@version` entries.
    pub tool_plan: Vec<String>,
    /// Source versions read.
    pub source_versions: BTreeMap<String, String>,
    /// Edition versions read (`slug` → version).
    pub edition_versions: BTreeMap<String, String>,
    /// Normalization rule set (Phase 2; `None` in Phase 1).
    pub normalization_rule_set: Option<String>,
    /// Retrieval config hash (later phases; `None` in Phase 1).
    pub retrieval_config_hash: Option<ContentHash>,
    /// Model provider (Phase 9; `None` in Phase 1).
    pub model_provider: Option<String>,
    /// Model name (Phase 9; `None` in Phase 1).
    pub model_name: Option<String>,
    /// Prompt version (Phase 9; `None` in Phase 1).
    pub prompt_version: Option<String>,
    /// Corpus generation read from.
    pub corpus_generation: u64,
    /// Whether the result is fully deterministic.
    pub deterministic: bool,
}

/// The tool result envelope every tool returns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolResult<T> {
    /// Tool name (`quran.get_ayah`).
    pub tool_name: String,
    /// Tool version.
    pub tool_version: SemVer,
    /// Exact input (secret-redacted by callers).
    pub query: serde_json::Value,
    /// Normalization rules applied (empty in Phase 1).
    pub normalization_rules: Vec<String>,
    /// Edition id read, when applicable.
    pub edition_id: Option<String>,
    /// Edition version read, when applicable.
    pub edition_version: Option<String>,
    /// Results.
    pub results: T,
    /// Fully-qualified canonical references for every result.
    pub canonical_references: Vec<String>,
    /// Analysis sources behind the results.
    pub analysis_sources: Vec<AnalysisSource>,
    /// Confidence, when the tool reports one.
    pub confidence: Option<Confidence>,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
    /// Wall-clock execution time.
    pub execution_time_ms: f64,
    /// Reproducibility record.
    pub reproducibility: ReproducibilityData,
}

/// Tool errors (`QAI-QUR-0311…0312`).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ToolError {
    /// The tool input failed validation.
    #[error("invalid tool input for {tool}: {detail}")]
    InvalidInput {
        /// Tool name.
        tool: &'static str,
        /// What was wrong.
        detail: String,
    },
    /// The backend failed (machine-readable `code` + human `detail`).
    #[error("tool backend failed: {detail}")]
    Backend {
        /// Namespaced code (`QAI-QUR-*`) delegated from the backend.
        code: String,
        /// What was wrong.
        detail: String,
    },
}

impl ToolError {
    /// Stable code string.
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidInput { .. } => "QAI-QUR-0311",
            Self::Backend { .. } => "QAI-QUR-0312",
        }
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// SHA-256 [`ContentHash`] of canonical JSON bytes.
pub fn content_hash_of(value: &serde_json::Value) -> ContentHash {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    ContentHash { algorithm: HashAlgorithm::Sha256, hex: sha256_hex(&bytes) }
}

/// Build the deterministic reproducibility record for a read-only tool call.
#[allow(clippy::too_many_arguments)]
pub fn reproducibility(
    tool_name: &str,
    tool_version: SemVer,
    query: &serde_json::Value,
    edition_slug: Option<&str>,
    edition_version: Option<&str>,
    source_versions: BTreeMap<String, String>,
    corpus_generation: u64,
) -> ReproducibilityData {
    let query_hash = content_hash_of(query);
    let mut edition_versions = BTreeMap::new();
    if let (Some(slug), Some(version)) = (edition_slug, edition_version) {
        edition_versions.insert(slug.to_string(), version.to_string());
    }
    let checksum = content_hash_of(&serde_json::json!({
        "tool": format!("{tool_name}@{tool_version}"),
        "query_hash": query_hash.hex,
        "editions": edition_versions,
        "sources": source_versions,
        "generation": corpus_generation,
    }));
    ReproducibilityData {
        checksum,
        query_hash,
        tool_plan: vec![format!("{tool_name}@{tool_version}")],
        source_versions,
        edition_versions,
        normalization_rule_set: None,
        retrieval_config_hash: None,
        model_provider: None,
        model_name: None,
        prompt_version: None,
        corpus_generation,
        deterministic: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksums_are_deterministic_and_sensitive() {
        let query = serde_json::json!({"reference": "2:255"});
        let first = reproducibility(
            "quran.get_ayah",
            SemVer::new(1, 0, 0),
            &query,
            Some("hafs-uthmani"),
            Some("1.0.0"),
            BTreeMap::new(),
            7,
        );
        let second = reproducibility(
            "quran.get_ayah",
            SemVer::new(1, 0, 0),
            &query,
            Some("hafs-uthmani"),
            Some("1.0.0"),
            BTreeMap::new(),
            7,
        );
        assert_eq!(first.checksum, second.checksum);
        assert!(first.deterministic);
        let other = reproducibility(
            "quran.get_ayah",
            SemVer::new(1, 0, 0),
            &query,
            Some("hafs-uthmani"),
            Some("1.0.0"),
            BTreeMap::new(),
            8,
        );
        assert_ne!(first.checksum, other.checksum);
    }

    #[test]
    fn tool_error_codes() {
        assert_eq!(
            ToolError::InvalidInput { tool: "t", detail: "d".into() }.code(),
            "QAI-QUR-0311"
        );
        assert_eq!(
            ToolError::Backend { code: "QAI-QUR-0307".into(), detail: "d".into() }.code(),
            "QAI-QUR-0312"
        );
    }
}
