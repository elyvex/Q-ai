//! Tool result contract (§12) and reproducibility checksums (§12.1).
//!
//! # tools
//!
//! Every tool in every phase conforms to [`ToolResult`]: the exact query, the
//! editions and versions read, canonical references for every result, and a
//! deterministic [`ReproducibilityData`] checksum. Phase 1 covers the
//! deterministic inputs; model/prompt fields arrive in Phase 9.

use std::collections::BTreeMap;

use domain::{Confidence, ContentHash, HashAlgorithm, HashingError, SemVer};
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
    /// Research checksum (D-14/D-15): domain-separated SHA-256 over tool
    /// identity + normalized parameters + canonical inputs + the result
    /// payload, so identical results recompute to an identical digest on
    /// every surface (UI, API, CLI). Required: no `Default`, every
    /// construction site sets it.
    pub research_checksum: ContentHash,
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

/// Domain tag for the research checksum (ADR-0108 style separation: this
/// digest must never collide with a digest computed for another purpose).
pub const RESEARCH_CHECKSUM_DOMAIN: &str = "qai-research-checksum-v1";

/// Research checksum (D-14): domain-separated SHA-256 over tool identity +
/// normalized parameters + canonical inputs (refs / edition /
/// `corpus_generation`) + the result payload.
///
/// The envelope is serialized with the frozen
/// [`domain::hashing::canonical_json_bytes`] helper (ADR-0006) — reused
/// verbatim, never rewritten — so map/field insertion order never changes
/// the digest. An empty payload still yields a non-null digest over the
/// tool identity and inputs.
///
/// `normalized_params` is the tool's typed params struct serialized to
/// JSON (the caller passes the same `query` value stored on the envelope);
/// `canonical_inputs` names the canonical rows read; `result_payload` is
/// the serialized `results` field.
pub fn research_checksum(
    tool_name: &str,
    tool_version: SemVer,
    normalized_params: &serde_json::Value,
    canonical_inputs: &serde_json::Value,
    result_payload: &serde_json::Value,
) -> Result<ContentHash, HashingError> {
    let canonical = serde_json::json!({
        "domain": RESEARCH_CHECKSUM_DOMAIN,
        "tool": format!("{tool_name}@{tool_version}"),
        "params": normalized_params,
        "inputs": canonical_inputs,
        "payload": result_payload,
    });
    let bytes = domain::hashing::canonical_json_bytes(&canonical)?;
    Ok(ContentHash { algorithm: HashAlgorithm::Sha256, hex: sha256_hex(&bytes) })
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

    fn checksum_fixture() -> (
        &'static str,
        SemVer,
        serde_json::Value,
        serde_json::Value,
        serde_json::Value,
    ) {
        (
            "quran.get_ayah",
            SemVer::new(1, 0, 0),
            serde_json::json!({"reference": "2:255"}),
            serde_json::json!({
                "reference": "2:255",
                "edition_slug": "hafs-uthmani",
                "edition_version": "1.0.0",
                "corpus_generation": 7,
            }),
            serde_json::json!({"arabic_text": "ب"}),
        )
    }

    #[test]
    fn research_checksum_is_deterministic_payload_and_param_sensitive() {
        let (tool, version, params, inputs, payload) = checksum_fixture();
        let first = research_checksum(tool, version, &params, &inputs, &payload).unwrap();
        let second = research_checksum(tool, version, &params, &inputs, &payload).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.algorithm, HashAlgorithm::Sha256);
        assert!(!first.hex.is_empty());

        // A one-field difference in the payload changes the digest.
        let other_payload = serde_json::json!({"arabic_text": "ت"});
        let changed = research_checksum(tool, version, &params, &inputs, &other_payload).unwrap();
        assert_ne!(first, changed);

        // A one-field difference in the normalized params changes the digest.
        let other_params = serde_json::json!({"reference": "2:256"});
        let changed = research_checksum(tool, version, &other_params, &inputs, &payload).unwrap();
        assert_ne!(first, changed);

        // An empty payload still yields a non-null, non-empty digest.
        let empty = research_checksum(tool, version, &params, &inputs, &serde_json::Value::Null)
            .unwrap();
        assert!(!empty.hex.is_empty());
        assert_ne!(first, empty);
    }

    #[test]
    fn research_checksum_is_insensitive_to_field_insertion_order() {
        let (tool, version, params, _, payload) = checksum_fixture();
        // Same keys, different textual order — the frozen canonical-JSON
        // helper sorts map keys (serde_json without `preserve_order` is
        // BTreeMap-backed), so the digest must not move.
        let ordered = serde_json::json!({
            "reference": "2:255",
            "edition_slug": "hafs-uthmani",
            "edition_version": "1.0.0",
            "corpus_generation": 7,
        });
        let reordered: serde_json::Value = serde_json::from_str(
            r#"{"corpus_generation":7,"edition_version":"1.0.0","edition_slug":"hafs-uthmani","reference":"2:255"}"#,
        )
        .unwrap();
        let first = research_checksum(tool, version, &params, &ordered, &payload).unwrap();
        let second = research_checksum(tool, version, &params, &reordered, &payload).unwrap();
        assert_eq!(first, second);
    }
}
