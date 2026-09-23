//! Graph JSON export (TASK-427 slice): bounded, attribution-preserving output.
//!
//! [`export_json`] serializes identities (nodes), structure (edges),
//! attribution (assertions), versions (manifest), and truncation flags into
//! one `quran-graph-json-v1` document. [`retain_visible`] is the
//! tombstone/authorization hook: callers filter edges through a predicate
//! (e.g. [`assertion_allowlist_predicate`]) before export so restricted
//! evidence can never leak into a document.
//!
//! GraphML is explicitly out of scope for this slice. Rationale: export must
//! stay bounded and tombstone-safe first. A second XML serialization would
//! double the output-surface audit (escaping, unbounded document size,
//! per-viewer filtering semantics) before the JSON contract, the truncation
//! flags, and the authorization hook have proven themselves. Revisit only via
//! an explicit export-format decision (P4-X05).

use std::collections::HashSet;

use crate::model::{Assertion, GraphEdge, GraphNode, ProjectionManifest};

/// Export document format marker.
pub const GRAPH_JSON_FORMAT: &str = "quran-graph-json-v1";

/// Truncation notice attached to every export document.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExportNotice {
    /// Whether the exported collection was cut short by budgets.
    pub truncated: bool,
    /// Why the collection is partial. Always `Some` when `truncated`.
    #[serde(default)]
    pub incomplete_reason: Option<String>,
}

impl ExportNotice {
    /// A complete export.
    pub fn complete() -> Self {
        Self { truncated: false, incomplete_reason: None }
    }

    /// A partial export with an explicit reason.
    pub fn incomplete(reason: impl Into<String>) -> Self {
        Self { truncated: true, incomplete_reason: Some(reason.into()) }
    }
}

/// Export nodes, edges, assertions, and the manifest as a complete
/// `quran-graph-json-v1` document.
pub fn export_json(
    nodes: &[GraphNode],
    edges: &[GraphEdge],
    assertions: &[Assertion],
    manifest: &ProjectionManifest,
) -> serde_json::Value {
    export_json_with_notice(nodes, edges, assertions, manifest, &ExportNotice::complete())
}

/// Export with an explicit truncation notice (mirrors the traversal result
/// the collection came from).
pub fn export_json_with_notice(
    nodes: &[GraphNode],
    edges: &[GraphEdge],
    assertions: &[Assertion],
    manifest: &ProjectionManifest,
    notice: &ExportNotice,
) -> serde_json::Value {
    serde_json::json!({
        "format": GRAPH_JSON_FORMAT,
        "manifest": manifest,
        "nodes": nodes,
        "edges": edges,
        "assertions": assertions,
        "counts": {
            "nodes": nodes.len(),
            "edges": edges.len(),
            "assertions": assertions.len(),
        },
        "truncation": {
            "truncated": notice.truncated,
            "incomplete_reason": notice.incomplete_reason,
        },
    })
}

/// Tombstone/authorization hook: keep the edges accepted by `keep`, drop the
/// rest. All node records pass through untouched — node records carry no
/// assertions, and the edge filter is the authorization boundary. Callers
/// that need orphan-free output can prune unreferenced nodes afterwards.
pub fn retain_visible<F>(
    nodes: &[GraphNode],
    edges: &[GraphEdge],
    keep: F,
) -> (Vec<GraphNode>, Vec<GraphEdge>)
where
    F: Fn(&GraphEdge) -> bool,
{
    (nodes.to_vec(), edges.iter().filter(|e| keep(e)).cloned().collect())
}

/// Build the standard authorization predicate: structural edges (no
/// assertion) plus edges whose assertion is in `visible` are kept.
pub fn assertion_allowlist_predicate(
    visible: &HashSet<String>,
) -> impl Fn(&GraphEdge) -> bool + use<> {
    let visible = visible.clone();
    move |edge: &GraphEdge| match &edge.assertion_id {
        None => true,
        Some(id) => visible.contains(id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        AssertionDecision, AssertionKind, NodeKind, ProjectionStatus, ProvenanceLayer,
    };
    use std::collections::BTreeMap;

    fn sample_manifest() -> ProjectionManifest {
        ProjectionManifest {
            id: "build-1".into(),
            projection_id: "quran-structural-v1".into(),
            builder_version: "structural-v1".into(),
            edition_id: "test-min".into(),
            corpus_generation: 3,
            dataset_versions: BTreeMap::from([("morph".to_string(), "v2".to_string())]),
            dependency_snapshot: BTreeMap::new(),
            status: ProjectionStatus::Active,
            manifest: serde_json::json!({}),
            created_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn export_carries_contract_fields() {
        let nodes = vec![GraphNode::new("ayah:1:1", NodeKind::Ayah, serde_json::json!({}))];
        let edges = vec![GraphEdge::structural(
            "surah:1",
            "CONTAINS",
            "ayah:1:1",
            serde_json::json!({"input_version": "v1"}),
        )];
        let doc = export_json(&nodes, &edges, &[], &sample_manifest());
        assert_eq!(doc["format"], GRAPH_JSON_FORMAT);
        assert_eq!(doc["counts"]["nodes"], 1);
        assert_eq!(doc["counts"]["edges"], 1);
        assert_eq!(doc["truncation"]["truncated"], false);
        assert_eq!(doc["manifest"]["edition_id"], "test-min");
    }

    #[test]
    fn export_with_notice_flags_truncation() {
        let doc = export_json_with_notice(
            &[],
            &[],
            &[],
            &sample_manifest(),
            &ExportNotice::incomplete("node budget exhausted"),
        );
        assert_eq!(doc["truncation"]["truncated"], true);
        assert_eq!(doc["truncation"]["incomplete_reason"], "node budget exhausted");
    }

    #[test]
    fn retain_visible_filters_by_predicate() {
        let nodes = vec![GraphNode::new("a", NodeKind::Ayah, serde_json::Value::Null)];
        let edges = vec![
            GraphEdge::structural("a", "NEXT", "b", serde_json::Value::Null),
            GraphEdge::asserted("a", "CITES", "c", "hidden-1"),
        ];
        let visible = HashSet::new();
        let (kept_nodes, kept_edges) =
            retain_visible(&nodes, &edges, assertion_allowlist_predicate(&visible));
        assert_eq!(kept_nodes.len(), 1);
        assert_eq!(kept_edges.len(), 1);
        assert_eq!(kept_edges[0].edge, "NEXT");
        let _ = (AssertionDecision::Accepted, AssertionKind::Annotation, ProvenanceLayer::B);
    }
}
