//! Phase 7 (plan 07-04) — isnad-graph conformance tests.

use isnad_graph::{
    Assertion, AssertionDecision, AssertionKind, ConfidenceBand, NarratorId, NarratorRecord,
};
use quran_graph::{
    Assertion as GraphAssertion, AuthzScope, GraphEdge, GraphNode, GraphStore, MemGraphStore,
    NodeKind, ProvenanceLayer, QueryBudgets, is_allowed_edge, up_to_k_paths,
};
use std::sync::atomic::AtomicBool;

#[test]
fn node_kind_serializes_narrator_and_hadith() {
    assert_eq!(NodeKind::Narrator.to_string(), "narrator");
    assert_eq!(NodeKind::Hadith.to_string(), "hadith");
    // serde
    let n = serde_json::to_string(&NodeKind::Narrator).unwrap();
    assert_eq!(n, "\"narrator\"");
    let h = serde_json::to_string(&NodeKind::Hadith).unwrap();
    assert_eq!(h, "\"hadith\"");
}

#[test]
fn assertion_kind_narrator_identity_serializes() {
    let s = serde_json::to_string(&AssertionKind::NarratorIdentity).unwrap();
    assert_eq!(s, "\"narrator_identity\"");
}

#[test]
fn is_allowed_edge_transmits_and_narrates() {
    assert!(is_allowed_edge("TRANSMITS"));
    assert!(is_allowed_edge("NARRATES"));
    assert!(!is_allowed_edge("UNKNOWN_EDGE"));
}

#[test]
fn stage_narrator_nodes_and_edges() {
    let mut store = MemGraphStore::new("isnad-conformance".to_string());
    store
        .stage_nodes(vec![
            GraphNode::new("narrator:a", NodeKind::Narrator, serde_json::json!({})),
            GraphNode::new("narrator:b", NodeKind::Narrator, serde_json::json!({})),
            GraphNode::new("hadith:1", NodeKind::Hadith, serde_json::json!({})),
        ])
        .unwrap();
    store
        .stage_edges(vec![
            GraphEdge::asserted("narrator:a", "TRANSMITS", "narrator:b", "a1"),
            GraphEdge::asserted("narrator:b", "NARRATES", "hadith:1", "a2"),
        ])
        .unwrap();
    let insp = store.inspect();
    assert_eq!(insp.node_count, 3);
    assert_eq!(insp.edge_count, 2);
}

#[test]
fn up_to_k_paths_returns_assertion_id_per_edge() {
    let mut store = MemGraphStore::new("isnad-paths".to_string());
    store
        .stage_nodes(vec![
            GraphNode::new("narrator:a", NodeKind::Narrator, serde_json::json!({})),
            GraphNode::new("narrator:b", NodeKind::Narrator, serde_json::json!({})),
            GraphNode::new("hadith:1", NodeKind::Hadith, serde_json::json!({})),
        ])
        .unwrap();
    // insert assertions
    store.insert_assertion(GraphAssertion {
        id: "a1".to_string(),
        kind: AssertionKind::NarratorIdentity,
        claim: serde_json::json!({}),
        evidence: serde_json::json!({}),
        source_location: "loc".to_string(),
        reviewer: Some("rev".to_string()),
        decision: AssertionDecision::Accepted,
        decided_at: Some("2026-01-01T00:00:00Z".to_string()),
        supersedes_id: None,
        layer: ProvenanceLayer::B,
        algorithm: None,
        algorithm_version: None,
        confidence: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
    });
    store.insert_assertion(GraphAssertion {
        id: "a2".to_string(),
        kind: AssertionKind::NarratorIdentity,
        claim: serde_json::json!({}),
        evidence: serde_json::json!({}),
        source_location: "loc".to_string(),
        reviewer: Some("rev".to_string()),
        decision: AssertionDecision::Accepted,
        decided_at: Some("2026-01-01T00:00:00Z".to_string()),
        supersedes_id: None,
        layer: ProvenanceLayer::B,
        algorithm: None,
        algorithm_version: None,
        confidence: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
    });
    store
        .stage_edges(vec![
            GraphEdge::asserted("narrator:a", "TRANSMITS", "narrator:b", "a1"),
            GraphEdge::asserted("narrator:b", "NARRATES", "hadith:1", "a2"),
        ])
        .unwrap();

    let cancel = AtomicBool::new(false);
    let authz = AuthzScope::all_visible();
    let budgets = QueryBudgets::default();
    let res =
        up_to_k_paths(&store, "narrator:a", "hadith:1", 5, &budgets, &cancel, &authz).unwrap();
    assert!(!res.truncated);
    assert!(!res.paths.is_empty());
    for path in &res.paths {
        for edge in &path.edges {
            assert!(edge.assertion_id.is_some(), "edge must carry assertion_id");
        }
    }
}

#[test]
fn budget_exhaustion_propagates_truncated() {
    let mut store = MemGraphStore::new("isnad-budget".to_string());
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for i in 0..40 {
        nodes.push(GraphNode::new(format!("n{}", i), NodeKind::Narrator, serde_json::json!({})));
    }
    for i in 0..39 {
        edges.push(GraphEdge::structural(
            &format!("n{}", i),
            "TRANSMITS",
            &format!("n{}", i + 1),
            serde_json::json!({}),
        ));
    }
    store.stage_nodes(nodes).unwrap();
    store.stage_edges(edges).unwrap();

    let cancel = AtomicBool::new(false);
    let authz = AuthzScope::all_visible();
    let tight = QueryBudgets { max_nodes: 5, ..QueryBudgets::default() };
    let res = store.bounded_paths("n0", "n39", 40, &tight, &cancel, &authz).unwrap();
    assert!(res.truncated);
    assert!(res.incomplete_reason.is_some());
    assert!(!res.incomplete_reason.as_ref().unwrap().is_empty());
}

#[test]
fn confidence_bands_thresholds() {
    assert_eq!(ConfidenceBand::of(0.95), ConfidenceBand::High);
    assert_eq!(ConfidenceBand::of(0.9), ConfidenceBand::Medium);
    assert_eq!(ConfidenceBand::of(0.7), ConfidenceBand::Medium);
    assert_eq!(ConfidenceBand::of(0.69), ConfidenceBand::Low);
}
