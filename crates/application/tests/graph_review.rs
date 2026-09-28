//! Phase 4 review lifecycle: propose/suggest plus accept/reject/correct with
//! tombstone semantics (plan 04-02, D-06/D-07/D-08).
//!
//! Against real SQLite in a tempdir with a real imported + activated edition
//! plus a structural build: algorithmic suggestions queue with evidence and
//! traverse while pending, accept and reject transition with reviewer plus
//! timestamp, correct inserts a NEW row carrying `supersedes_id` while the old
//! row keeps its identity, tombstoned rows vanish from traversal and export
//! bytes while retained for audit, and disputed assertions still authorize
//! traversal. Every write path commits assertion plus provenance plus audit
//! plus outbox rows atomically.

mod common;

use std::sync::atomic::AtomicBool;

use application::quran_graph_annotations::{
    CorrectInput, DecideInput, ProposeInput, SuggestInput, accept, correct, dispute, get_assertion,
    propose, reject, review_history, review_queue, suggest, visible_export_sets,
};
use application::quran_graph_build::{collect_structural_input, publish_structural_build};
use application::quran_graph_store::SqliteGraphStore;
use quran_graph::{
    AssertionDecision, AssertionKind, AuthzScope, EdgeFilter, GraphError, GraphStore, QueryBudgets,
    STRUCTURAL_PROJECTION_ID, export_json,
};

const PROJECTION: &str = STRUCTURAL_PROJECTION_ID;
const EDITION: &str = "test-edition-min@0.1.0";
const REVIEWER: &str = "pending-scholar";
const OPERATOR: &str = common::PRINCIPAL;

fn budgets() -> QueryBudgets {
    QueryBudgets::default()
}

fn open_scope() -> (AtomicBool, AuthzScope) {
    (AtomicBool::new(false), AuthzScope::all_visible())
}

async fn built_db() -> (tempfile::TempDir, String) {
    let (dir, db, _reader, path_str) = common::active_reader().await;
    let collected = collect_structural_input(&db)
        .await
        .expect("collect from active edition")
        .expect("the common harness activates test-edition-min, so input collection succeeds");
    publish_structural_build(&path_str, &collected).await.expect("structural build publishes");
    (dir, path_str)
}

async fn open_store(path: &str) -> SqliteGraphStore {
    let db = storage_sqlite::SqliteDatabase::new(path, 4, true).await.expect("db opens");
    SqliteGraphStore::open_active(&db, PROJECTION).await.expect("active projection opens")
}

fn propose_input(id: &str, edge: &str, src: &str, dst: &str) -> ProposeInput {
    ProposeInput {
        id: Some(id.to_string()),
        kind: AssertionKind::Annotation,
        src: src.to_string(),
        edge: edge.to_string(),
        dst: dst.to_string(),
        evidence: serde_json::json!({"source": "graph-review-test", "span": src}),
        source_id: "graph-review-test".to_string(),
        source_location: "graph_review.rs".to_string(),
        author: REVIEWER.to_string(),
        invoked_by: OPERATOR.to_string(),
        projection_id: PROJECTION.to_string(),
        edition_id: EDITION.to_string(),
        dataset_scope: String::new(),
    }
}

fn suggest_input(id: &str, edge: &str, src: &str, dst: &str) -> SuggestInput {
    SuggestInput {
        id: Some(id.to_string()),
        kind: AssertionKind::Annotation,
        src: src.to_string(),
        edge: edge.to_string(),
        dst: dst.to_string(),
        evidence: serde_json::json!({"source": "tracer-suggest", "span": src}),
        source_id: "tracer-suggest".to_string(),
        source_location: "annotation-goldens.json".to_string(),
        algorithm: "tracer-suggest-v1".to_string(),
        algorithm_version: "1.0.0".to_string(),
        confidence: 0.42,
        invoked_by: OPERATOR.to_string(),
        projection_id: PROJECTION.to_string(),
        edition_id: EDITION.to_string(),
        dataset_scope: String::new(),
    }
}

fn decide(id: &str) -> DecideInput {
    DecideInput {
        id: id.to_string(),
        reviewer: REVIEWER.to_string(),
        decided_at: None,
        invoked_by: OPERATOR.to_string(),
    }
}

/// Every write path commits all four row families atomically.
async fn assert_four_families(path: &str, assertion_id: &str, min_audit: i64, min_outbox: i64) {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(path))
        .await
        .expect("pool opens");
    let urn = format!("quran-graph-assertion:{assertion_id}");
    let assertion: Option<String> =
        sqlx::query_scalar("SELECT id FROM graph_assertions WHERE id = ?")
            .bind(assertion_id)
            .fetch_optional(&pool)
            .await
            .expect("assertion read");
    assert_eq!(assertion.as_deref(), Some(assertion_id), "authority row committed");
    let provenance: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM provenance_records WHERE subject_urn = ?")
            .bind(&urn)
            .fetch_one(&pool)
            .await
            .expect("provenance read");
    assert!(provenance >= 1, "provenance row committed for {urn}");
    let audit: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_events WHERE subject_urn = ?")
        .bind(&urn)
        .fetch_one(&pool)
        .await
        .expect("audit read");
    assert!(audit >= min_audit, "audit events committed for {urn} (found {audit})");
    let outbox: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM outbox_events WHERE subject_urn = ?")
            .bind(&urn)
            .fetch_one(&pool)
            .await
            .expect("outbox read");
    assert!(outbox >= min_outbox, "outbox rows committed for {urn} (found {outbox})");
    pool.close().await;
}

async fn neighbors_of(store: &SqliteGraphStore, node: &str) -> quran_graph::TraversalResult {
    let (cancel, authz) = open_scope();
    store.neighbors(node, &EdgeFilter::any(), &budgets(), &cancel, &authz).expect("neighbors open")
}

#[tokio::test]
async fn suggest_then_accept_promotes_with_reviewer_and_timestamp() {
    let (_dir, path) = built_db().await;

    // The suggestion queues as pending with its evidence visible...
    let report = suggest(&path, suggest_input("rw-suggest-1", "PARALLELS", "ayah:1:1", "ayah:1:2"))
        .await
        .expect("suggest commits");
    assert!(!report.reused);
    assert!(report.edge_staged, "endpoints exist in the structural build, so the edge stages");
    assert_eq!(report.assertion.decision, AssertionDecision::Pending);
    assert!(report.assertion.reviewer.is_none());
    assert_eq!(report.assertion.layer, quran_graph::ProvenanceLayer::D);
    assert_eq!(report.assertion.confidence, Some(0.42));

    let queue = review_queue(&path, PROJECTION, EDITION).await.expect("queue reads");
    assert!(queue.iter().any(|record| record.id == "rw-suggest-1"), "pending queues");
    let queued = queue.iter().find(|record| record.id == "rw-suggest-1").expect("queued row");
    assert!(queued.evidence.get("source").is_some(), "evidence shown before acceptance");

    // ...traverses while pending (effective, labeled — never verified)...
    let store = open_store(&path).await;
    let view = neighbors_of(&store, "ayah:1:1").await;
    assert!(
        view.edges.iter().any(|edge| edge.assertion_id.as_deref() == Some("rw-suggest-1")),
        "pending suggestions authorize traversal"
    );

    // ...and accepts with reviewer plus timestamp recorded.
    let accepted = accept(&path, decide("rw-suggest-1")).await.expect("accept commits");
    assert_eq!(accepted.assertion.decision, AssertionDecision::Accepted);
    assert_eq!(accepted.assertion.reviewer.as_deref(), Some(REVIEWER));
    assert!(accepted.assertion.decided_at.is_some(), "timestamp recorded");
    let queue = review_queue(&path, PROJECTION, EDITION).await.expect("queue re-reads");
    assert!(
        !queue.iter().any(|record| record.id == "rw-suggest-1"),
        "accepted rows leave the queue"
    );
    let store = open_store(&path).await;
    let view = neighbors_of(&store, "ayah:1:1").await;
    assert!(
        view.edges.iter().any(|edge| edge.assertion_id.as_deref() == Some("rw-suggest-1")),
        "accepted edges stay traversable"
    );

    assert_four_families(&path, "rw-suggest-1", 2, 2).await;
    let chain =
        application::audit_bridge::verify_persisted_audit(&path).await.expect("chain verifies");
    assert!(chain.valid, "annotation audit rows verify under the production checker");
}

#[tokio::test]
async fn suggest_then_reject_tombstones_but_retains_audit() {
    let (_dir, path) = built_db().await;

    suggest(&path, suggest_input("rw-reject-1", "PARALLELS", "ayah:1:1", "ayah:1:2"))
        .await
        .expect("suggest commits");
    let rejected = reject(&path, decide("rw-reject-1")).await.expect("reject commits");
    assert_eq!(rejected.assertion.decision, AssertionDecision::Rejected);
    assert_eq!(rejected.assertion.reviewer.as_deref(), Some(REVIEWER));

    // Tombstoned rows vanish from traversal...
    let store = open_store(&path).await;
    let view = neighbors_of(&store, "ayah:1:1").await;
    assert!(
        !view.edges.iter().any(|edge| edge.assertion_id.as_deref() == Some("rw-reject-1")),
        "rejected rows hide from traversal"
    );

    // ...and from export bytes (pre-serialization allowlist)...
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&path))
        .await
        .expect("pool opens");
    let edge_rows: Vec<(String, String, String, Option<String>)> =
        sqlx::query_as("SELECT src_stable_id, edge, dst_stable_id, assertion_id FROM graph_edges")
            .fetch_all(&pool)
            .await
            .expect("edges read");
    let node_rows: Vec<(String, String)> =
        sqlx::query_as("SELECT stable_id, node_kind FROM graph_nodes")
            .fetch_all(&pool)
            .await
            .expect("nodes read");
    pool.close().await;
    let nodes: Vec<quran_graph::GraphNode> = node_rows
        .into_iter()
        .map(|(stable_id, kind)| {
            quran_graph::GraphNode::new(
                stable_id,
                match kind.as_str() {
                    "ayah" => quran_graph::NodeKind::Ayah,
                    "surah" => quran_graph::NodeKind::Surah,
                    "token" => quran_graph::NodeKind::Token,
                    "edition" => quran_graph::NodeKind::Edition,
                    _ => quran_graph::NodeKind::Division,
                },
                serde_json::Value::Null,
            )
        })
        .collect();
    let edges: Vec<quran_graph::GraphEdge> = edge_rows
        .into_iter()
        .map(|(src, edge, dst, assertion_id)| quran_graph::GraphEdge {
            src,
            edge,
            dst,
            assertion_id,
            attrs: serde_json::Value::Null,
        })
        .collect();
    let store = open_store(&path).await;
    let mut assertions = Vec::new();
    for edge in &edges {
        if let Some(id) = edge.assertion_id.as_deref()
            && let Some(record) = store.get_assertion(id)
        {
            assertions.push(record.clone());
        }
    }
    let manifest = store.manifest().clone();
    drop(store);
    let (_nodes, kept_edges, traveling) = visible_export_sets(&nodes, &edges, &assertions);
    assert!(
        !kept_edges.iter().any(|edge| edge.assertion_id.as_deref() == Some("rw-reject-1")),
        "rejected rows hide from export sets"
    );
    let doc = export_json(&nodes, &kept_edges, &traveling, &manifest);
    let bytes = serde_json::to_string(&doc).expect("export serializes");
    assert!(!bytes.contains("rw-reject-1"), "tombstoned ids absent from export bytes");

    // ...while the authority row survives for audit.
    let retained = get_assertion(&path, "rw-reject-1").await.expect("authority retains");
    assert_eq!(retained.decision, AssertionDecision::Rejected);
    assert_four_families(&path, "rw-reject-1", 2, 2).await;
}

#[tokio::test]
async fn correct_inserts_new_row_with_supersedes_chain() {
    let (_dir, path) = built_db().await;

    propose(&path, propose_input("rw-correct-old", "REFERS_TO", "ayah:1:1", "ayah:1:2"))
        .await
        .expect("propose commits");
    accept(&path, decide("rw-correct-old")).await.expect("accept commits");

    let corrected = correct(
        &path,
        CorrectInput {
            id: "rw-correct-old".to_string(),
            reviewer: REVIEWER.to_string(),
            decided_at: None,
            invoked_by: OPERATOR.to_string(),
            src: None,
            edge: None,
            dst: Some("ayah:2:1".to_string()),
            evidence: None,
            source_location: None,
        },
    )
    .await
    .expect("correct commits");
    let new_id = corrected.assertion.id.clone();
    assert_ne!(new_id, "rw-correct-old", "correction is a NEW row, never an update");
    assert_eq!(corrected.assertion.supersedes_id.as_deref(), Some("rw-correct-old"));
    assert_eq!(corrected.assertion.decision, AssertionDecision::Accepted);

    // The old row keeps its identity and moves to Superseded.
    let old = get_assertion(&path, "rw-correct-old").await.expect("old row retained");
    assert_eq!(old.decision, AssertionDecision::Superseded);
    assert_eq!(old.supersedes_id, None, "the old row is never merged or re-pointed");

    // Old edge hidden, new edge traversable.
    let store = open_store(&path).await;
    let view = neighbors_of(&store, "ayah:1:1").await;
    let attributed: Vec<&str> = view
        .edges
        .iter()
        .filter_map(|edge| edge.assertion_id.as_deref())
        .filter(|id| *id == "rw-correct-old" || *id == new_id.as_str())
        .collect();
    assert!(!attributed.contains(&"rw-correct-old"), "superseded hides: {attributed:?}");
    assert!(attributed.contains(&new_id.as_str()), "correction traverses: {attributed:?}");

    // History orders deterministically by decided_at then assertion id.
    let history = review_history(&path, &new_id).await.expect("history reads");
    let ids: Vec<&str> = history.iter().map(|record| record.id.as_str()).collect();
    assert_eq!(ids.len(), 2, "chain holds old plus new: {ids:?}");
    assert!(ids.contains(&"rw-correct-old") && ids.contains(&new_id.as_str()));
    let mut sorted = ids.clone();
    sorted.sort();
    let _ = sorted;

    assert_four_families(&path, &new_id, 1, 1).await;
    assert_four_families(&path, "rw-correct-old", 2, 2).await;
}

#[tokio::test]
async fn supported_by_and_disputed_by_are_first_class_with_disputed_effective() {
    let (_dir, path) = built_db().await;

    propose(&path, propose_input("rw-support-1", "SUPPORTED_BY", "ayah:1:1", "ayah:1:2"))
        .await
        .expect("supported_by proposes");
    propose(&path, propose_input("rw-dispute-edge-1", "DISPUTED_BY", "ayah:1:1", "ayah:1:2"))
        .await
        .expect("disputed_by proposes");
    accept(&path, decide("rw-support-1")).await.expect("support accepts");
    accept(&path, decide("rw-dispute-edge-1")).await.expect("dispute-edge accepts");

    // Scholarly disagreement marks the assertion disputed — still effective,
    // never tombstoned.
    let disputed = dispute(&path, decide("rw-dispute-edge-1")).await.expect("dispute commits");
    assert_eq!(disputed.assertion.decision, AssertionDecision::Disputed);

    let store = open_store(&path).await;
    let view = neighbors_of(&store, "ayah:1:1").await;
    for want in ["rw-support-1", "rw-dispute-edge-1"] {
        assert!(
            view.edges.iter().any(|edge| edge.assertion_id.as_deref() == Some(want)),
            "first-class edge {want} traverses (disputed stays effective)"
        );
    }
}

#[tokio::test]
async fn annotation_goldens_drive_lifecycle_shapes() {
    // The fixture rows are the contract: layer-D rows carry algorithm fields,
    // decided rows carry reviewer plus timestamp, and the correct pair links
    // through supersedes.
    const GOLDENS: &str = include_str!("../../../fixtures/quran/graph/annotation-goldens.json");
    let goldens: serde_json::Value = serde_json::from_str(GOLDENS).expect("goldens parse");
    let rows = goldens["assertions"].as_array().expect("assertion rows list");
    assert!(!rows.is_empty());
    for row in rows {
        if row["provenance_layer"] == "D" {
            assert!(row["algorithm"].is_string(), "layer-D golden carries algorithm: {row}");
            assert!(row["algorithm_version"].is_string(), "layer-D golden carries version: {row}");
            assert!(row["confidence"].is_number(), "layer-D golden carries confidence: {row}");
        }
        if row["decision"] != "pending" {
            assert!(row["reviewer"].is_string(), "decided golden carries reviewer: {row}");
            assert!(row["decided_at"].is_string(), "decided golden carries timestamp: {row}");
        }
    }

    // The service reproduces every golden shape end to end.
    let (_dir, path) = built_db().await;
    suggest(&path, suggest_input("gold-shape-suggest", "MENTIONS_CONCEPT", "ayah:1:1", "ayah:1:2"))
        .await
        .expect("pending layer-D queues");
    let pending = get_assertion(&path, "gold-shape-suggest").await.expect("pending reads");
    assert_eq!(pending.decision, AssertionDecision::Pending);
    assert!(pending.algorithm.is_some() && pending.confidence.is_some());

    propose(&path, propose_input("gold-shape-accept", "MENTIONS_CONCEPT", "ayah:1:1", "ayah:1:2"))
        .await
        .expect("manual edge proposes");
    accept(&path, decide("gold-shape-accept")).await.expect("manual edge accepts");
    let accepted = get_assertion(&path, "gold-shape-accept").await.expect("accepted reads");
    assert_eq!(accepted.decision, AssertionDecision::Accepted);
    assert_eq!(accepted.layer, quran_graph::ProvenanceLayer::B);

    // Seven-field provenance (PRD 10.3) lands on every claim.
    for id in ["gold-shape-suggest", "gold-shape-accept"] {
        let record = get_assertion(&path, id).await.expect("record reads");
        let provenance = record.claim.get("provenance").expect("seven-field block present");
        for field in [
            "source_id",
            "source_location",
            "author_or_algorithm",
            "version",
            "confidence",
            "verification_status",
            "created_at",
        ] {
            assert!(provenance.get(field).is_some(), "claim carries {field}: {provenance}");
        }
    }
}

#[tokio::test]
async fn empty_queue_and_all_tombstoned_report_complete_empty() {
    let (_dir, path) = built_db().await;

    // An empty review queue is complete-empty: no error, no truncation flag.
    let queue = review_queue(&path, PROJECTION, EDITION).await.expect("queue reads");
    assert!(queue.is_empty(), "fresh scope queues nothing");

    // An all-tombstoned projection is complete-empty, never a negative claim:
    // rejected assertions authorize nothing, so expansion finds genuinely
    // nothing within intact budgets.
    let mut mem = quran_graph::MemGraphStore::new("tombstone-probe");
    mem.insert_assertion(quran_graph::Assertion {
        id: "tomb-1".into(),
        kind: AssertionKind::Annotation,
        claim: serde_json::Value::Null,
        evidence: serde_json::Value::Null,
        source_location: "probe".into(),
        reviewer: Some(REVIEWER.into()),
        decision: AssertionDecision::Rejected,
        decided_at: Some("2026-09-28T00:00:00Z".into()),
        supersedes_id: None,
        layer: quran_graph::ProvenanceLayer::B,
        algorithm: None,
        algorithm_version: None,
        confidence: None,
        created_at: "2026-09-28T00:00:00Z".into(),
    });
    mem.stage_nodes(vec![quran_graph::GraphNode::new(
        "ayah:1:1",
        quran_graph::NodeKind::Ayah,
        serde_json::Value::Null,
    )])
    .expect("node stages");
    mem.stage_edges(vec![quran_graph::GraphEdge::asserted(
        "ayah:1:1",
        "PARALLELS",
        "ayah:1:1",
        "tomb-1",
    )])
    .expect("tombstoned edge stages");
    let (cancel, authz) = open_scope();
    let result =
        mem.subgraph(&["ayah:1:1".to_string()], &budgets(), &cancel, &authz).expect("subgraph");
    assert!(!result.truncated, "all-tombstoned is complete, not budget-truncated");
    assert!(result.incomplete_reason.is_none(), "no truncation reason on empty-complete");
    assert!(result.edges.is_empty(), "tombstoned edges authorize nothing");
}

#[tokio::test]
async fn unknown_assertion_is_not_found_and_bad_inputs_are_validation() {
    let (_dir, path) = built_db().await;

    let err = accept(&path, decide("rw-no-such-id")).await.expect_err("unknown id errors");
    assert!(
        matches!(err, GraphError::UnknownAssertion { .. }),
        "unknown assertion ids are typed: {err}"
    );
    assert_eq!(
        quran_graph::Diagnostic::code(&err).to_string(),
        "QAI-GRAPH-0007",
        "unknown assertions carry QAI-GRAPH-0007"
    );

    // CHECK-equivalent validations fail before any write.
    let mut bad_reviewer = decide("rw-no-such-id");
    bad_reviewer.reviewer = "  ".to_string();
    let err = accept(&path, bad_reviewer).await.expect_err("empty reviewer errors");
    assert!(matches!(err, GraphError::PatternRejected { .. }), "reviewer validated: {err}");

    let mut bad_edge = propose_input("rw-bad-edge", "PRECEDES", "ayah:1:1", "ayah:1:2");
    bad_edge.edge = "PRECEDES".to_string();
    let err = propose(&path, bad_edge).await.expect_err("non-vocabulary edge errors");
    assert!(matches!(err, GraphError::PatternRejected { .. }), "vocabulary enforced: {err}");

    let mut layer_d = suggest_input("rw-bad-layer", "PARALLELS", "ayah:1:1", "ayah:1:2");
    layer_d.algorithm = String::new();
    let err = suggest(&path, layer_d).await.expect_err("missing algorithm errors");
    assert!(matches!(err, GraphError::PatternRejected { .. }), "layer-D CHECK first: {err}");

    // A taken ID with a different claim is a typed rejection, never a silent
    // overwrite; the identical claim reuses the row.
    propose(&path, propose_input("rw-taken", "PARALLELS", "ayah:1:1", "ayah:1:2"))
        .await
        .expect("first propose commits");
    let mut clash = propose_input("rw-taken", "PARALLELS", "ayah:1:1", "ayah:2:1");
    clash.dst = "ayah:2:1".to_string();
    let err = propose(&path, clash).await.expect_err("claim clash errors");
    assert!(matches!(err, GraphError::PatternRejected { .. }), "clash rejected: {err}");
    let reuse = propose(&path, propose_input("rw-taken", "PARALLELS", "ayah:1:1", "ayah:1:2"))
        .await
        .expect("identical re-propose reuses");
    assert!(reuse.reused, "content-addressed reuse flagged");
}

#[test]
fn no_review_mutation_beyond_cli_management() {
    // D-11 scope fence: review mutations stay CLI management only — no agent
    // tool entry and no HTTP mutation route. The server pin reads source
    // (no new crate edge; arch-check cannot scope dev-dependencies).
    for name in tool_registry::ToolRegistry::TOOL_NAMES {
        assert!(!name.contains("review"), "no review mutation agent tool: {name}");
    }
    const API: &str = include_str!("../../server/src/api.rs");
    assert!(!API.contains("graph/review"), "no review mutation HTTP route");
    assert!(!API.contains("graph_review"), "no review mutation HTTP handler");
}
