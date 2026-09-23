//! No-backend-language guard: formatted outputs and public API names never
//! carry backend query-language fragments. The needles below name what is
//! banned; they appear here only as assertions, never in crate output.

use quran_graph::{
    AuthzScope, Diagnostic, ExportNotice, GraphEdge, GraphError, GraphNode, GraphStore,
    MinHopsOutcome, NodeKind, PathsResult, Pattern, PatternStep, ProjectionManifest,
    ProjectionStatus, QueryBudgets, TraversalResult,
};

/// Fragments no output or public name may contain. `match (` covers raw
/// selection syntax; the language names cover engine-specific exposure.
fn needles() -> [&'static str; 3] {
    ["cypher", "datalog", "match ("]
}

fn assert_clean(label: &str, text: &str) {
    let lower = text.to_lowercase();
    for needle in needles() {
        assert!(!lower.contains(needle), "{label} leaks banned fragment {needle:?}: {text:?}");
    }
}

fn sample_nodes() -> (Vec<GraphNode>, Vec<GraphEdge>) {
    (
        vec![
            GraphNode::new("ayah:1:1", NodeKind::Ayah, serde_json::json!({"surah": 1})),
            GraphNode::new("concept:x", NodeKind::Concept, serde_json::Value::Null),
        ],
        vec![
            GraphEdge::structural("surah:1", "CONTAINS", "ayah:1:1", serde_json::json!({})),
            GraphEdge::asserted("ayah:1:1", "MENTIONS_CONCEPT", "concept:x", "a-1"),
        ],
    )
}

fn sample_manifest() -> ProjectionManifest {
    ProjectionManifest {
        id: "build-1".into(),
        projection_id: "quran-structural-v1".into(),
        builder_version: "structural-v1".into(),
        edition_id: "test-min".into(),
        corpus_generation: 1,
        dataset_versions: Default::default(),
        dependency_snapshot: Default::default(),
        status: ProjectionStatus::Active,
        manifest: serde_json::Value::Null,
        created_at: String::new(),
    }
}

#[test]
fn outputs_carry_no_backend_language() {
    let (nodes, edges) = sample_nodes();
    let pattern = Pattern::new(vec![
        PatternStep::edge_to("MENTIONS_CONCEPT", NodeKind::Concept),
        PatternStep::edge("NEXT"),
    ]);

    let errors = [
        GraphError::UnknownProjection { projection: "p".into() },
        GraphError::BudgetExceeded { detail: "budgets tight".into() },
        GraphError::PatternRejected { detail: "unknown edge".into() },
        GraphError::NodeNotFound { stable_id: "n".into() },
        GraphError::BuildFailed { stage: "stage".into(), detail: "bad".into() },
        GraphError::AuthzDenied { detail: "denied".into() },
    ];
    for err in &errors {
        assert_clean("GraphError Debug", &format!("{err:?}"));
        assert_clean("GraphError Display", &format!("{err}"));
        assert_clean("GraphError human", &err.render_human());
    }

    let result = TraversalResult {
        nodes: nodes.clone(),
        edges: edges.clone(),
        truncated: true,
        incomplete_reason: Some("node budget exhausted".into()),
        expanded_nodes: 3,
        expanded_edges: 2,
    };
    assert_clean("TraversalResult Debug", &format!("{result:?}"));
    assert_clean("TraversalResult JSON", &serde_json::to_string(&result).unwrap());

    let paths = PathsResult::complete(Vec::new(), 0, 0);
    assert_clean("PathsResult Debug", &format!("{paths:?}"));

    let hops = MinHopsOutcome {
        hops: Some(2),
        truncated: false,
        incomplete_reason: None,
        expanded_nodes: 2,
    };
    assert_clean("MinHopsOutcome Debug", &format!("{hops:?}"));

    assert_clean("QueryBudgets Debug", &format!("{:?}", QueryBudgets::default()));
    assert_clean("Pattern Debug", &format!("{pattern:?}"));
    assert_clean("Pattern JSON", &serde_json::to_string(&pattern).unwrap());
    assert_clean("NodeKind Display", &format!("{}", NodeKind::Ayah));
    assert_clean("Manifest JSON", &serde_json::to_string(&sample_manifest()).unwrap());
    assert_clean("AuthzScope Debug", &format!("{:?}", AuthzScope::restricted(["a-1".to_string()])));
    assert_clean(
        "ExportNotice Debug",
        &format!("{:?}", ExportNotice::incomplete("edge budget exhausted")),
    );
    let doc = quran_graph::export_json(&nodes, &edges, &[], &sample_manifest());
    assert_clean("export JSON", &serde_json::to_string(&doc).unwrap());
}

#[test]
fn public_api_names_carry_no_backend_language() {
    // Every public type, function, and capability string in this crate.
    let names = [
        "GraphError",
        "Diagnostic",
        "DiagnosticCode",
        "codes",
        "GraphNode",
        "GraphEdge",
        "NodeKind",
        "EDGE_VOCABULARY",
        "is_allowed_edge",
        "Assertion",
        "AssertionKind",
        "AssertionDecision",
        "ProvenanceLayer",
        "ProjectionManifest",
        "ProjectionStatus",
        "QueryBudgets",
        "TraversalResult",
        "GraphPath",
        "PathsResult",
        "Direction",
        "EdgeFilter",
        "AuthzScope",
        "BuildInspection",
        "cancel_requested",
        "GraphStore",
        "resolve_node",
        "neighbors",
        "bounded_paths",
        "subgraph",
        "pattern_query",
        "stage_nodes",
        "stage_edges",
        "inspect",
        "clear_staged",
        "capabilities",
        "MemGraphStore",
        "reachable",
        "bounded_subgraph",
        "min_hops",
        "up_to_k_paths",
        "MinHopsOutcome",
        "Pattern",
        "PatternStep",
        "PATTERN_SIZE_BUDGET",
        "validate_pattern",
        "execute_pattern",
        "StructuralInput",
        "BuiltProjection",
        "build_structural",
        "STRUCTURAL_BUILDER_VERSION",
        "STRUCTURAL_PROJECTION_ID",
        "GRAPH_JSON_FORMAT",
        "ExportNotice",
        "export_json",
        "export_json_with_notice",
        "retain_visible",
        "assertion_allowlist_predicate",
    ];
    for name in names {
        assert_clean("public API name", name);
    }
    let store = quran_graph::MemGraphStore::new("caps");
    for cap in store.capabilities() {
        assert_clean("capability string", &cap);
    }
}
