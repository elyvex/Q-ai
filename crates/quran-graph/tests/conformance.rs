//! Backend-agnostic conformance suite (TASK-409 slice, extended plan 04-01):
//! neighbors, paths, subgraph, patterns, budget exhaustion (truncated, never
//! empty), and authorization-filtered intermediates excluded from paths *and*
//! counts.
//!
//! Every fixture runs against both [`MemGraphStore`] and the
//! application-layer [`SqliteGraphStore`](application::quran_graph_store::SqliteGraphStore)
//! through the backend-generic [`Backend`] harness, so SQLite-backed reads
//! prove the same bounded, authorized, deterministic behavior as the
//! reference backend. SQLite-only cases pin determinism under
//! insertion-order variation and the `NodeNotFound` versus
//! truncated-empty distinction.

use std::collections::HashSet;
use std::sync::atomic::AtomicBool;

use application::quran_graph_store::SqliteGraphStore;
use quran_graph::{
    Assertion, AssertionDecision, AssertionKind, AuthzScope, Diagnostic as _, Direction,
    EdgeFilter, GraphEdge, GraphError, GraphNode, GraphStore, MemGraphStore, NodeKind, Pattern,
    PatternStep, ProvenanceLayer, QueryBudgets,
};

fn open() -> (AtomicBool, AuthzScope, QueryBudgets) {
    (AtomicBool::new(false), AuthzScope::all_visible(), QueryBudgets::default())
}

fn assertion(id: &str, decision: AssertionDecision) -> Assertion {
    Assertion {
        id: id.to_string(),
        kind: AssertionKind::ConceptLink,
        claim: serde_json::json!({"concept": "mercy"}),
        evidence: serde_json::json!({"source": "annotator-a"}),
        source_location: "dataset-v1:row-7".to_string(),
        reviewer: Some("reviewer-1".to_string()),
        decision,
        decided_at: Some("2026-01-02T00:00:00Z".to_string()),
        supersedes_id: None,
        layer: ProvenanceLayer::B,
        algorithm: None,
        algorithm_version: None,
        confidence: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
    }
}

fn node(id: &str, kind: NodeKind) -> GraphNode {
    GraphNode::new(id, kind, serde_json::Value::Null)
}

/// Backend-neutral fixture: node/edge/assertion sets both backends stage.
struct FixtureParts {
    projection_id: String,
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    assertions: Vec<Assertion>,
}

/// Triangle: `s -> mid -> t` via asserted edges plus a direct structural
/// `s -> t` edge, so authz tests can prove the hidden route vanishes.
fn triangle_parts() -> FixtureParts {
    FixtureParts {
        projection_id: "conformance".to_string(),
        nodes: vec![
            node("s", NodeKind::Ayah),
            node("mid", NodeKind::Ayah),
            node("t", NodeKind::Ayah),
            node("c1", NodeKind::Concept),
        ],
        edges: vec![
            GraphEdge::asserted("s", "NEXT", "mid", "a-link"),
            GraphEdge::asserted("mid", "NEXT", "t", "a-link"),
            GraphEdge::structural("s", "NEXT", "t", serde_json::Value::Null),
            GraphEdge::asserted("s", "MENTIONS_CONCEPT", "c1", "a-hidden"),
            GraphEdge::asserted("mid", "MENTIONS_CONCEPT", "c1", "a-tomb"),
        ],
        assertions: vec![
            assertion("a-link", AssertionDecision::Accepted),
            assertion("a-hidden", AssertionDecision::Accepted),
            assertion("a-tomb", AssertionDecision::Rejected),
        ],
    }
}

/// One backend instance under test plus any guards it must hold open.
enum Backend {
    Mem(MemGraphStore),
    // Boxed: the snapshot store dwarfs the reference backend, and only two
    // instances ever exist (clippy::large_enum_variant).
    // The tempdir guard is never read by construction — it keeps the
    // database file alive for the test body (dead_code).
    Sqlite(Box<SqliteGraphStore>, #[allow(dead_code)] tempfile::TempDir),
}

impl Backend {
    fn name(&self) -> &'static str {
        match self {
            Self::Mem(_) => "mem",
            Self::Sqlite(_, _) => "sqlite",
        }
    }

    fn store(&self) -> &dyn GraphStore {
        match self {
            Self::Mem(store) => store,
            Self::Sqlite(store, _) => store.as_ref(),
        }
    }

    fn store_mut(&mut self) -> &mut dyn GraphStore {
        match self {
            Self::Mem(store) => store,
            Self::Sqlite(store, _) => store.as_mut(),
        }
    }

    fn assertion_present(&self, id: &str) -> bool {
        match self {
            Self::Mem(store) => store.get_assertion(id).is_some(),
            Self::Sqlite(store, _) => store.get_assertion(id).is_some(),
        }
    }
}

fn mem_backend(parts: &FixtureParts) -> MemGraphStore {
    let mut store = MemGraphStore::new(parts.projection_id.clone());
    for record in &parts.assertions {
        store.insert_assertion(record.clone());
    }
    store.stage_nodes(parts.nodes.clone()).expect("fixture nodes stage");
    store.stage_edges(parts.edges.clone()).expect("fixture edges stage");
    store
}

const CONFORMANCE_ROW_ID: &str = "conf-triangle-row";
const CONFORMANCE_AT: &str = "2026-09-28T00:00:00Z";

async fn sqlite_backend(parts: &FixtureParts) -> (SqliteGraphStore, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("qai.db");
    let path_str = path.to_str().expect("utf8 path").to_string();
    let repo_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    storage_sqlite::migrate::apply_migrations(&path_str, &repo_root).await.expect("migrate");
    let db = storage_sqlite::SqliteDatabase::new(&path_str, 4, true).await.expect("open database");

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&path_str)
                .create_if_missing(false)
                .foreign_keys(true)
                .busy_timeout(std::time::Duration::from_secs(5)),
        )
        .await
        .expect("seed pool opens");
    sqlx::query(
        "INSERT INTO graph_projections
           (id, projection_id, builder_version, edition_id, corpus_generation,
            dataset_versions_json, dependency_snapshot_json, status, manifest_json, created_at)
         VALUES (?, ?, 'conformance-v1', 'test-min', 0, '{}', '{}', 'active', '{}', ?)",
    )
    .bind(CONFORMANCE_ROW_ID)
    .bind(&parts.projection_id)
    .bind(CONFORMANCE_AT)
    .execute(&pool)
    .await
    .expect("projection row inserts");
    for (index, n) in parts.nodes.iter().enumerate() {
        sqlx::query(
            "INSERT INTO graph_nodes
               (id, projection_row_id, node_kind, stable_id, attrs_json, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(format!("conf-n-{index}"))
        .bind(CONFORMANCE_ROW_ID)
        .bind(n.kind.to_string())
        .bind(&n.stable_id)
        .bind(n.attrs.to_string())
        .bind(CONFORMANCE_AT)
        .execute(&pool)
        .await
        .expect("node inserts");
    }
    for record in &parts.assertions {
        sqlx::query(
            "INSERT INTO graph_assertions
               (id, projection_row_id, projection_id, edition_id, dataset_scope,
                assertion_kind, claim_json, evidence_json, source_location,
                reviewer, decision, decided_at, supersedes_id,
                provenance_layer, algorithm, algorithm_version, confidence, created_at)
             VALUES (?, ?, ?, 'test-min', 'conformance',
                     ?, ?, ?, ?,
                     ?, ?, ?, ?,
                     ?, ?, ?, ?, ?)",
        )
        .bind(&record.id)
        .bind(CONFORMANCE_ROW_ID)
        .bind(&parts.projection_id)
        .bind(assertion_kind_str(record.kind))
        .bind(record.claim.to_string())
        .bind(record.evidence.to_string())
        .bind(&record.source_location)
        .bind(record.reviewer.as_deref())
        .bind(decision_str(record.decision))
        .bind(record.decided_at.as_deref())
        .bind(record.supersedes_id.as_deref())
        .bind(match record.layer {
            ProvenanceLayer::B => "B",
            ProvenanceLayer::D => "D",
        })
        .bind(record.algorithm.as_deref())
        .bind(record.algorithm_version.as_deref())
        .bind(record.confidence)
        .bind(&record.created_at)
        .execute(&pool)
        .await
        .expect("assertion inserts");
    }
    for (index, edge) in parts.edges.iter().enumerate() {
        sqlx::query(
            "INSERT INTO graph_edges
               (id, projection_row_id, src_stable_id, edge, dst_stable_id,
                assertion_id, budgets_json, attrs_json, created_at)
             VALUES (?, ?, ?, ?, ?, ?, '{}', ?, ?)",
        )
        .bind(format!("conf-e-{index}"))
        .bind(CONFORMANCE_ROW_ID)
        .bind(&edge.src)
        .bind(&edge.edge)
        .bind(&edge.dst)
        .bind(edge.assertion_id.as_deref())
        .bind(edge.attrs.to_string())
        .bind(CONFORMANCE_AT)
        .execute(&pool)
        .await
        .expect("edge inserts");
    }
    pool.close().await;

    let store = SqliteGraphStore::open(&db, CONFORMANCE_ROW_ID).await.expect("projection opens");
    (store, dir)
}

fn assertion_kind_str(kind: AssertionKind) -> &'static str {
    match kind {
        AssertionKind::Annotation => "annotation",
        AssertionKind::ConceptLink => "concept_link",
        AssertionKind::EntityLink => "entity_link",
        AssertionKind::FamilyLink => "family_link",
        AssertionKind::Import => "import",
    }
}

fn decision_str(decision: AssertionDecision) -> &'static str {
    match decision {
        AssertionDecision::Pending => "pending",
        AssertionDecision::Accepted => "accepted",
        AssertionDecision::Rejected => "rejected",
        AssertionDecision::Superseded => "superseded",
        AssertionDecision::Disputed => "disputed",
    }
}

/// Both backends over the same fixture, in backend-name order for
/// `--nocapture` listings.
async fn backends(parts: &FixtureParts) -> Vec<Backend> {
    let mem = Backend::Mem(mem_backend(parts));
    let (sqlite, dir) = sqlite_backend(parts).await;
    vec![mem, Backend::Sqlite(Box::new(sqlite), dir)]
}

#[tokio::test]
async fn resolve_and_neighbors_basics() {
    let parts = triangle_parts();
    for backend in backends(&parts).await {
        println!("backend: {}", backend.name());
        let store = backend.store();
        let (cancel, authz, budgets) = open();
        let found = store.resolve_node("s", &budgets, &cancel, &authz).unwrap();
        assert_eq!(found.map(|n| n.kind), Some(NodeKind::Ayah));
        assert!(store.resolve_node("ghost", &budgets, &cancel, &authz).unwrap().is_none());

        let all = store.neighbors("s", &EdgeFilter::any(), &budgets, &cancel, &authz).unwrap();
        assert!(!all.truncated);
        // s -> mid, s -> t, s -> c1 = 3 edges; center + 3 counterparts = 4 nodes.
        assert_eq!(all.edges.len(), 3);
        assert_eq!(all.nodes.len(), 4);

        let out_next = store
            .neighbors(
                "s",
                &EdgeFilter::one("NEXT", Direction::Outgoing),
                &budgets,
                &cancel,
                &authz,
            )
            .unwrap();
        assert_eq!(out_next.edges.len(), 2);

        let incoming = store
            .neighbors(
                "t",
                &EdgeFilter::one("NEXT", Direction::Incoming),
                &budgets,
                &cancel,
                &authz,
            )
            .unwrap();
        assert_eq!(incoming.edges.len(), 2);
    }
}

#[tokio::test]
async fn bounded_paths_are_deterministic() {
    let parts = triangle_parts();
    for backend in backends(&parts).await {
        println!("backend: {}", backend.name());
        let store = backend.store();
        let (cancel, authz, budgets) = open();
        let first = store.bounded_paths("s", "t", 4, &budgets, &cancel, &authz).unwrap();
        let second = store.bounded_paths("s", "t", 4, &budgets, &cancel, &authz).unwrap();
        assert!(!first.truncated);
        assert_eq!(first, second);
        // Direct edge plus the two-hop route via mid.
        assert_eq!(first.paths.len(), 2);
        assert_eq!(first.paths[0].node_ids, vec!["s".to_string(), "t".to_string()]);
        assert_eq!(
            first.paths[1].node_ids,
            vec!["s".to_string(), "mid".to_string(), "t".to_string()]
        );
    }
}

#[tokio::test]
async fn subgraph_and_pattern_cover_seeds() {
    let parts = triangle_parts();
    for backend in backends(&parts).await {
        println!("backend: {}", backend.name());
        let store = backend.store();
        let (cancel, authz, budgets) = open();
        let sub = store.subgraph(&["s".to_string()], &budgets, &cancel, &authz).unwrap();
        assert!(!sub.truncated);
        let ids: HashSet<&str> = sub.nodes.iter().map(|n| n.stable_id.as_str()).collect();
        assert!(
            ids.contains("s") && ids.contains("mid") && ids.contains("t") && ids.contains("c1")
        );

        let pattern =
            Pattern::new(vec![PatternStep::edge_to("MENTIONS_CONCEPT", NodeKind::Concept)]);
        let matched =
            store.pattern_query(&pattern, &["s".to_string()], &budgets, &cancel, &authz).unwrap();
        assert!(!matched.truncated);
        assert!(matched.nodes.iter().any(|n| n.stable_id == "c1"));
    }
}

fn chain_parts() -> FixtureParts {
    let ids: Vec<String> = (0..50).map(|i| format!("n{i}")).collect();
    FixtureParts {
        projection_id: "long-chain".to_string(),
        nodes: ids.iter().map(|id| node(id, NodeKind::Ayah)).collect(),
        edges: ids
            .windows(2)
            .map(|w| GraphEdge::structural(&w[0], "NEXT", &w[1], serde_json::Value::Null))
            .collect(),
        assertions: Vec::new(),
    }
}

#[tokio::test]
async fn budget_exhaustion_is_truncated_not_empty() {
    let parts = chain_parts();
    for backend in backends(&parts).await {
        println!("backend: {}", backend.name());
        let store = backend.store();
        let (cancel, authz, _) = open();
        let tight = QueryBudgets { max_nodes: 5, ..QueryBudgets::default() };

        let sub = store.subgraph(&["n0".to_string()], &tight, &cancel, &authz).unwrap();
        assert!(sub.truncated, "exhausted budgets must truncate");
        assert!(sub.incomplete_reason.is_some());

        let paths = store.bounded_paths("n0", "n49", 60, &tight, &cancel, &authz).unwrap();
        assert!(paths.truncated, "a far target under tight budgets is incomplete, not absent");
        assert!(!paths.incomplete_reason.as_deref().unwrap_or_default().is_empty());
    }
}

#[tokio::test]
async fn authz_hides_intermediates_from_paths_and_counts() {
    let parts = triangle_parts();
    for backend in backends(&parts).await {
        println!("backend: {}", backend.name());
        let store = backend.store();
        let (cancel, _, budgets) = open();
        // Scope sees the direct structural edge and the hidden concept link, but
        // not the a-link route through mid.
        let scope = AuthzScope::restricted(["a-hidden".to_string()]);

        let paths = store.bounded_paths("s", "t", 4, &budgets, &cancel, &scope).unwrap();
        assert!(!paths.truncated);
        assert_eq!(paths.paths.len(), 1, "only the structural route may show");
        assert_eq!(paths.paths[0].node_ids, vec!["s".to_string(), "t".to_string()]);

        let nbrs = store.neighbors("s", &EdgeFilter::any(), &budgets, &cancel, &scope).unwrap();
        assert!(!nbrs.truncated);
        // s->mid hidden; s->t structural + s->c1 visible = 2 edges.
        assert_eq!(nbrs.edges.len(), 2);
        assert!(nbrs.edges.iter().all(|e| e.dst != "mid"));

        let sub = store.subgraph(&["s".to_string()], &budgets, &cancel, &scope).unwrap();
        assert!(!sub.truncated);
        let ids: HashSet<&str> = sub.nodes.iter().map(|n| n.stable_id.as_str()).collect();
        assert!(!ids.contains("mid"), "nodes reachable only via hidden edges vanish");
        assert!(ids.contains("c1"));

        // Tombstoned assertions hide their edges even under full visibility.
        let (_, full, _) = open();
        let mid_nbrs =
            store.neighbors("mid", &EdgeFilter::any(), &budgets, &cancel, &full).unwrap();
        assert!(
            mid_nbrs.edges.iter().all(|e| e.assertion_id.as_deref() != Some("a-tomb")),
            "rejected assertions never authorize traversal"
        );
    }
}

#[tokio::test]
async fn out_of_range_budgets_are_rejected_before_execution() {
    // ADR-0217 rule 1: a budget outside its valid range is an error, never a
    // clamped-and-truncated result and never a silent empty answer. Every port
    // operation must reject it, including operations that would otherwise
    // return an empty-but-complete result.
    let parts = triangle_parts();
    for backend in backends(&parts).await {
        println!("backend: {}", backend.name());
        let store = backend.store();
        let (cancel, authz, _) = open();
        // `max_hops = 0` and `max_nodes = 0` are both below the accepted ranges.
        let invalid = QueryBudgets { max_hops: 0, max_nodes: 0, ..QueryBudgets::default() };
        let pattern =
            Pattern::new(vec![PatternStep::edge_to("MENTIONS_CONCEPT", NodeKind::Concept)]);

        let err = store.resolve_node("s", &invalid, &cancel, &authz).unwrap_err();
        assert!(matches!(err, GraphError::BudgetExceeded { .. }), "resolve: {err}");

        let err = store.neighbors("s", &EdgeFilter::any(), &invalid, &cancel, &authz).unwrap_err();
        assert!(matches!(err, GraphError::BudgetExceeded { .. }), "neighbors: {err}");

        let err = store.bounded_paths("s", "t", 4, &invalid, &cancel, &authz).unwrap_err();
        assert!(matches!(err, GraphError::BudgetExceeded { .. }), "paths: {err}");

        let err = store.subgraph(&["s".to_string()], &invalid, &cancel, &authz).unwrap_err();
        assert!(matches!(err, GraphError::BudgetExceeded { .. }), "subgraph: {err}");

        let err = store
            .pattern_query(&pattern, &["s".to_string()], &invalid, &cancel, &authz)
            .unwrap_err();
        assert!(matches!(err, GraphError::BudgetExceeded { .. }), "pattern: {err}");

        // A single out-of-range field is enough; the rest of the budget stays legal.
        for bad in [
            QueryBudgets { max_paths: 0, ..QueryBudgets::default() },
            QueryBudgets { max_edges: 0, ..QueryBudgets::default() },
            QueryBudgets { max_fanout: 0, ..QueryBudgets::default() },
            QueryBudgets { timeout_ms: 0, ..QueryBudgets::default() },
        ] {
            assert!(
                store.neighbors("s", &EdgeFilter::any(), &bad, &cancel, &authz).is_err(),
                "out-of-range budget must be rejected: {bad:?}"
            );
        }
    }
}

#[tokio::test]
async fn pattern_queries_apply_authz_during_expansion() {
    // The authz suite above covers neighbors/paths/subgraph; pattern queries
    // expand a typed predicate chain and must filter the same way, otherwise a
    // restricted scope could enumerate hidden concepts through a pattern.
    let parts = triangle_parts();
    for backend in backends(&parts).await {
        println!("backend: {}", backend.name());
        let store = backend.store();
        let (cancel, _, budgets) = open();
        let pattern =
            Pattern::new(vec![PatternStep::edge_to("MENTIONS_CONCEPT", NodeKind::Concept)]);

        let full = store
            .pattern_query(
                &pattern,
                &["s".to_string()],
                &budgets,
                &cancel,
                &AuthzScope::all_visible(),
            )
            .unwrap();
        assert!(full.nodes.iter().any(|n| n.stable_id == "c1"));

        // `a-hidden` is the only accepted assertion behind `s -> c1`.
        let scope = AuthzScope::restricted(["a-link".to_string()]);
        let restricted =
            store.pattern_query(&pattern, &["s".to_string()], &budgets, &cancel, &scope).unwrap();
        assert!(!restricted.truncated, "a policy-filtered empty result is complete, not truncated");
        assert!(
            !restricted.nodes.iter().any(|n| n.stable_id == "c1"),
            "hidden concepts must not leak through a pattern query"
        );
    }
}

#[tokio::test]
async fn build_inspect_and_capabilities() {
    let parts = FixtureParts {
        projection_id: "proj-1".to_string(),
        nodes: Vec::new(),
        edges: Vec::new(),
        assertions: vec![assertion("a-keep", AssertionDecision::Accepted)],
    };
    for backend in backends(&parts).await {
        println!("backend: {}", backend.name());
        let mut backend = backend;
        let staged = backend.store_mut();
        staged.stage_nodes(vec![node("a", NodeKind::Ayah), node("b", NodeKind::Ayah)]).unwrap();
        staged
            .stage_edges(vec![GraphEdge::structural("a", "NEXT", "b", serde_json::Value::Null)])
            .unwrap();
        let report = staged.inspect();
        assert_eq!(report.projection_id, "proj-1");
        assert_eq!((report.node_count, report.edge_count), (2, 1));
        let caps = staged.capabilities();
        for want in ["neighbors", "bounded_paths", "subgraph", "pattern_query", "authz_filter"] {
            assert!(caps.contains(&want.to_string()), "missing capability {want}");
        }
        staged.clear_staged();
        let cleared = staged.inspect();
        assert_eq!((cleared.node_count, cleared.edge_count), (0, 0));
    }
    // Authority records survive staging clears on every backend (checked
    // after the loop so the sqlite tempdir guard stays alive).
    for backend in backends(&parts).await {
        assert!(
            backend.assertion_present("a-keep"),
            "clearing a projection never discards scholarly history"
        );
    }
}

#[tokio::test]
async fn disputed_assertions_authorize_traversal() {
    // D-07: a disputed assertion is live disagreement, not a tombstone —
    // its edges traverse under full and listed scopes alike.
    let parts = FixtureParts {
        projection_id: "disputed".to_string(),
        nodes: vec![node("x", NodeKind::Ayah), node("y", NodeKind::Ayah)],
        edges: vec![GraphEdge::asserted("x", "MENTIONS_CONCEPT", "y", "d1")],
        assertions: vec![assertion("d1", AssertionDecision::Disputed)],
    };
    assert!(assertion("d1", AssertionDecision::Disputed).is_effective());
    for backend in backends(&parts).await {
        println!("backend: {}", backend.name());
        let store = backend.store();
        let (cancel, _, budgets) = open();
        let full = store
            .neighbors("x", &EdgeFilter::any(), &budgets, &cancel, &AuthzScope::all_visible())
            .unwrap();
        assert!(!full.truncated);
        assert!(
            full.edges.iter().any(|e| e.assertion_id.as_deref() == Some("d1")),
            "disputed assertions authorize traversal"
        );
        let scope = AuthzScope::restricted(["d1".to_string()]);
        let listed = store.neighbors("x", &EdgeFilter::any(), &budgets, &cancel, &scope).unwrap();
        assert!(
            listed.edges.iter().any(|e| e.assertion_id.as_deref() == Some("d1")),
            "listed disputed assertions stay visible"
        );
    }
}

#[tokio::test]
async fn node_not_found_vs_truncated_empty() {
    // Empty-graph, unknown-node, no-path-complete, and truncated-empty are
    // four distinct typed outcomes: unknown seeds resolve to NodeNotFound
    // while unknown subgraph seeds skip to complete-empty.
    let parts = triangle_parts();
    for backend in backends(&parts).await {
        println!("backend: {}", backend.name());
        let store = backend.store();
        let (cancel, authz, budgets) = open();

        let err =
            store.neighbors("ghost", &EdgeFilter::any(), &budgets, &cancel, &authz).unwrap_err();
        assert!(matches!(err, GraphError::NodeNotFound { .. }), "neighbors: {err}");
        assert_eq!(err.code().to_string(), "QAI-GRAPH-0004");

        let err = store.bounded_paths("ghost", "t", 4, &budgets, &cancel, &authz).unwrap_err();
        assert!(matches!(err, GraphError::NodeNotFound { .. }), "paths: {err}");

        // Unknown subgraph seeds skip to a complete empty result.
        let empty = store.subgraph(&["ghost".to_string()], &budgets, &cancel, &authz).unwrap();
        assert!(!empty.truncated, "unknown seeds are complete-empty, never truncated");
        assert!(empty.nodes.is_empty() && empty.edges.is_empty());
        assert!(empty.incomplete_reason.is_none());

        // No path within budget is complete-empty, not an error.
        let none = store
            .subgraph(&["c1".to_string()], &budgets, &cancel, &AuthzScope::all_visible())
            .unwrap();
        assert!(!none.truncated);
    }
}

#[tokio::test]
async fn sqlite_deterministic_under_insertion_order() {
    // SQLite-only: scrambled physical insertion order still yields the
    // reference ordering — frontiers sort by stable ID, result edges by
    // (src, edge, dst).
    let parts = triangle_parts();
    let mut scrambled = triangle_parts();
    scrambled.edges.reverse();
    scrambled.nodes.reverse();

    let mem = mem_backend(&parts);
    let (ordered, _guard) = sqlite_backend(&parts).await;
    let (shuffled, _guard) = sqlite_backend(&scrambled).await;
    let (cancel, authz, budgets) = open();

    let a = ordered.neighbors("s", &EdgeFilter::any(), &budgets, &cancel, &authz).unwrap();
    let b = shuffled.neighbors("s", &EdgeFilter::any(), &budgets, &cancel, &authz).unwrap();
    assert_eq!(a, b, "insertion order must not move SQLite results");
    let c = mem.neighbors("s", &EdgeFilter::any(), &budgets, &cancel, &authz).unwrap();
    assert_eq!(a.edges, c.edges, "SQLite ordering matches the reference backend");
    assert_eq!(
        a.nodes.iter().map(|n| &n.stable_id).collect::<Vec<_>>(),
        c.nodes.iter().map(|n| &n.stable_id).collect::<Vec<_>>()
    );

    let pa = ordered.bounded_paths("s", "t", 4, &budgets, &cancel, &authz).unwrap();
    let pb = shuffled.bounded_paths("s", "t", 4, &budgets, &cancel, &authz).unwrap();
    assert_eq!(pa, pb, "path order is insertion-independent");
    println!("backend: sqlite");
}

const CONCEPT_SEED: &str = include_str!("../../../fixtures/quran/graph/concept-seed-v1.json");
const ANNOTATION_GOLDENS: &str =
    include_str!("../../../fixtures/quran/graph/annotation-goldens.json");

#[tokio::test]
async fn seed_fixtures_validate() {
    // Both fixtures parse as JSON with version plus synthetic labeling; the
    // goldens exercise the full suggest/accept/reject/correct/supersede plus
    // dispute lifecycle with layer-D algorithm fields where required.
    let seed: serde_json::Value = serde_json::from_str(CONCEPT_SEED).expect("seed parses");
    assert_eq!(seed["version"], "v1");
    assert_eq!(seed["synthetic_test_only"], true);
    assert_eq!(seed["edition_scope"], "test-min");
    for entry in ["concepts", "persons", "places"] {
        let items = seed[entry].as_array().expect("seed section is a list");
        assert!(!items.is_empty(), "seed section {entry} is non-empty");
        for item in items {
            assert_eq!(item["synthetic_test_only"], true, "every entry is synthetic-labeled");
            assert!(
                item["attribution"]["reviewed_by"].as_str().unwrap_or_default().contains("pending"),
                "no entry claims reviewed scholarship: {item}"
            );
        }
    }

    let goldens: serde_json::Value =
        serde_json::from_str(ANNOTATION_GOLDENS).expect("goldens parse");
    assert_eq!(goldens["version"], "v1");
    assert_eq!(goldens["synthetic_test_only"], true);
    let rows = goldens["assertions"].as_array().expect("assertion rows list");
    let decisions: HashSet<&str> = rows.iter().filter_map(|row| row["decision"].as_str()).collect();
    for want in ["pending", "accepted", "rejected", "superseded", "disputed"] {
        assert!(decisions.contains(want), "lifecycle covers {want}: {decisions:?}");
    }
    let by_id: std::collections::HashMap<&str, &serde_json::Value> =
        rows.iter().filter_map(|row| row["id"].as_str().map(|id| (id, row))).collect();
    let corrected = by_id["gold-correct-new"];
    assert_eq!(corrected["supersedes"], "gold-correct-old");
    assert_eq!(by_id["gold-correct-old"]["decision"], "superseded");
    for row in rows {
        if row["provenance_layer"] == "D" {
            assert!(
                row["algorithm"].is_string()
                    && row["algorithm_version"].is_string()
                    && row["confidence"].is_number(),
                "layer-D rows carry algorithm fields: {row}"
            );
        }
        if row["decision"] != "pending" {
            assert!(
                row["reviewer"].is_string() && row["decided_at"].is_string(),
                "decided rows carry reviewer plus timestamp: {row}"
            );
        }
    }
    println!("fixtures: concept-seed-v1 + annotation-goldens valid");
}
