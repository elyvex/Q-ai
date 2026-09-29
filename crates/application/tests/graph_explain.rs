//! Phase 4 cross-surface parity and explainability contract (plan 04-04,
//! D-12/D-14).
//!
//! Over one shared fixture with fixed seeded inputs, the CLI JSON output,
//! the HTTP handler output (via a faked backend — the handler places backend
//! outputs verbatim under `Envelope.data`, pinned per route by the server
//! `graph.rs` data-shape cases), and the tool result payload agree on
//! neighbors, each path mode, subgraph, and patterns: canonicalized JSON
//! equality with only duration milliseconds excluded. Every compared result
//! carries the full explainability payload, and a truncated query reports
//! `truncated: true` with a non-empty reason on all three surfaces rather
//! than diverging.
//!
//! Adds no production code: this file only drives the three surfaces.

mod common;

use std::sync::Arc;

use application::quran_cli::{self, GraphBudgets};
use application::quran_graph_api::{
    BudgetPatch, GraphApiService, GraphBackend, NeighborsArgs, PathArgs, PathsArgs, PatternArgs,
    PatternStepRequest, SubgraphArgs, parse_pattern_steps, read_options,
};
use application::quran_graph_build::{collect_structural_input, publish_structural_build};
use application::quran_graph_tools::GraphToolBackend;
use serde_json::Value;

const HOPS: usize = 2;

/// The shared fixed budgets every surface runs: explicit on all inputs so
/// defaults can never smuggle divergence into the comparison.
fn cli_budgets() -> GraphBudgets {
    GraphBudgets {
        max_nodes: Some(500),
        max_edges: Some(2000),
        max_paths: Some(10),
        max_fanout: Some(128),
        timeout_ms: Some(5000),
    }
}

fn patch() -> BudgetPatch {
    BudgetPatch {
        max_hops: Some(HOPS),
        max_nodes: Some(500),
        max_edges: Some(2000),
        max_paths: Some(10),
        max_fanout: Some(128),
        timeout_ms: Some(5000),
    }
}

fn tool_budgets() -> tool_registry::GraphBudgetsParams {
    tool_registry::GraphBudgetsParams {
        max_hops: Some(HOPS),
        max_nodes: Some(500),
        max_edges: Some(2000),
        max_paths: Some(10),
        max_fanout: Some(128),
        timeout_ms: Some(5000),
    }
}

/// One shared fixture: imported + activated edition, structural projection,
/// plus one accepted and one pending-suggestion assertion on the same ayah
/// pair so asserted provenance (with confidence) travels on every surface.
async fn shared_fixture() -> (tempfile::TempDir, String) {
    let (dir, db, _reader, path_str) = common::active_reader().await;
    let collected = collect_structural_input(&db)
        .await
        .expect("collect from active edition")
        .expect("the harness activates test-edition-min, so collection succeeds");
    publish_structural_build(&path_str, &collected).await.expect("structural build publishes");
    drop(db);
    let service = GraphApiService::open(&path_str).await.expect("read service opens");
    seed_assertions(&path_str, &service.manifest().id, &service.manifest().edition_id).await;
    drop(service);
    (dir, path_str)
}

async fn seed_assertions(path_str: &str, row_id: &str, edition_id: &str) {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(path_str)
                .create_if_missing(false)
                .foreign_keys(true)
                .busy_timeout(std::time::Duration::from_secs(5)),
        )
        .await
        .expect("seed pool opens");
    let rows = [
        ("sug", "pending", "D", None, None, Some(("parity-suggester", "0.2.0", 0.5))),
        ("acc", "accepted", "B", Some("tracer-scholar"), Some("2026-09-28T00:00:00Z"), None),
    ];
    for (suffix, decision, layer, reviewer, decided_at, algorithm) in rows {
        let aid = format!("parity-{suffix}");
        let (algorithm, algorithm_version, confidence): (
            Option<String>,
            Option<String>,
            Option<f64>,
        ) = match algorithm {
            Some((name, version, confidence)) => {
                (Some(name.to_string()), Some(version.to_string()), Some(confidence))
            }
            None => (None, None, None),
        };
        sqlx::query(
            "INSERT INTO graph_assertions
               (id, projection_row_id, projection_id, edition_id, dataset_scope,
                assertion_kind, claim_json, evidence_json, source_location,
                reviewer, decision, decided_at,
                provenance_layer, algorithm, algorithm_version, confidence, created_at)
             VALUES (?, ?, ?, ?, 'parity-seed',
                     'annotation', '{}', '{\"source\": \"parity-seed\"}', 'parity-seed:row-1',
                     ?, ?, ?,
                     ?, ?, ?, ?, '2026-09-28T00:00:00Z')",
        )
        .bind(&aid)
        .bind(row_id)
        .bind(quran_graph::STRUCTURAL_PROJECTION_ID)
        .bind(edition_id)
        .bind(reviewer)
        .bind(decision)
        .bind(decided_at)
        .bind(layer)
        .bind(algorithm)
        .bind(algorithm_version)
        .bind(confidence)
        .execute(&pool)
        .await
        .expect("assertion inserts");
        sqlx::query(
            "INSERT INTO graph_edges
               (id, projection_row_id, src_stable_id, edge, dst_stable_id,
                assertion_id, budgets_json, attrs_json, created_at)
             VALUES (?, ?, 'ayah:1:1', 'PARALLELS', 'ayah:1:2',
                     ?, '{}', '{}', '2026-09-28T00:00:00Z')",
        )
        .bind(format!("parity-edge-{suffix}"))
        .bind(row_id)
        .bind(&aid)
        .execute(&pool)
        .await
        .expect("assertion edge inserts");
    }
    pool.close().await;
}

/// The faked HTTP backend: delegates to the live read service, proving the
/// `GraphBackend` trait surface the handlers program against carries the
/// same types. The HTTP leg of the comparison is the serialized backend
/// output — exactly the value the handler places under `Envelope.data`.
struct ParityBackend {
    service: GraphApiService,
}

#[async_trait::async_trait]
impl GraphBackend for ParityBackend {
    async fn neighbors(
        &self,
        args: NeighborsArgs,
    ) -> Result<
        application::quran_graph_api::NeighborsOutput,
        application::quran_graph_api::GraphApiError,
    > {
        self.service.neighbors(args).await
    }

    async fn reachability(
        &self,
        args: PathArgs,
    ) -> Result<
        application::quran_graph_api::ReachabilityOutput,
        application::quran_graph_api::GraphApiError,
    > {
        self.service.reachability(args).await
    }

    async fn shortest_path(
        &self,
        args: PathArgs,
    ) -> Result<
        application::quran_graph_api::ShortestOutput,
        application::quran_graph_api::GraphApiError,
    > {
        self.service.shortest_path(args).await
    }

    async fn paths(
        &self,
        args: PathsArgs,
    ) -> Result<
        application::quran_graph_api::PathsOutput,
        application::quran_graph_api::GraphApiError,
    > {
        self.service.paths(args).await
    }

    async fn subgraph(
        &self,
        args: SubgraphArgs,
    ) -> Result<
        application::quran_graph_api::SubgraphOutput,
        application::quran_graph_api::GraphApiError,
    > {
        self.service.subgraph(args).await
    }

    async fn pattern(
        &self,
        args: PatternArgs,
    ) -> Result<
        application::quran_graph_api::PatternOutput,
        application::quran_graph_api::GraphApiError,
    > {
        self.service.pattern(args).await
    }

    async fn root_family(
        &self,
        args: application::quran_graph_api::RootFamilyArgs,
    ) -> Result<
        application::quran_graph_api::RootFamilyOutput,
        application::quran_graph_api::GraphApiError,
    > {
        self.service.root_family(args).await
    }

    async fn snapshot_meta(
        &self,
    ) -> Result<
        application::quran_graph_api::GraphSnapshotMeta,
        application::quran_graph_api::GraphApiError,
    > {
        self.service.snapshot_meta().await
    }
}

/// Strip the presentational-only duration everywhere it appears (the one
/// field the plan excludes from parity: wall-clock milliseconds legitimately
/// differ across surfaces and calls).
fn strip_duration(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.remove("duration_ms");
            for nested in map.values_mut() {
                strip_duration(nested);
            }
        }
        Value::Array(items) => {
            for nested in items {
                strip_duration(nested);
            }
        }
        _ => {}
    }
}

fn canonical(mut value: Value) -> Value {
    strip_duration(&mut value);
    value
}

/// D-14 on one explanation payload: start/end nodes, ordered traversed
/// path, edge types with per-edge provenance (including confidence on
/// asserted edges), applied filters matching the fixed inputs, snapshot
/// build identity, completion status with reason, and duration.
///
/// `expect_edges` is false for reachability (a min-hop proof by design,
/// never a path render); `expect_asserted` is false where the fixed steps
/// or tie-breaks traverse structural edges only.
fn assert_explained(
    explanation: &Value,
    start: Option<&str>,
    end: Option<&str>,
    expect_edges: bool,
    expect_asserted: bool,
) {
    // Start/end nodes travel on every result (null where the read has none).
    assert!(explanation.get("start").is_some(), "start travels: {explanation}");
    assert!(explanation.get("end").is_some(), "end travels: {explanation}");
    if let Some(expected) = start {
        assert_eq!(explanation["start"], expected, "start node");
    }
    if let Some(expected) = end {
        assert_eq!(explanation["end"], expected, "end node");
    }
    // Ordered traversed path: node IDs plus explained paths.
    assert!(explanation["nodes"].is_array(), "ordered node IDs: {explanation}");
    assert!(explanation["paths"].is_array(), "explained paths: {explanation}");
    // Edge types with per-edge provenance on every traversed edge (top
    // level plus path renders — path modes split edges across both).
    let mut edges: Vec<&Value> = explanation["edges"].as_array().unwrap().iter().collect();
    for path in explanation["paths"].as_array().unwrap() {
        edges.extend(path["edges"].as_array().unwrap().iter());
    }
    if expect_edges {
        assert!(!edges.is_empty(), "traversed edges travel: {explanation}");
    }
    let mut saw_asserted = false;
    for edge in &edges {
        assert!(edge["edge"].is_string(), "edge type: {edge}");
        assert!(edge["provenance"].is_object(), "per-edge provenance: {edge}");
        if edge["provenance"]["kind"] == "asserted" {
            saw_asserted = true;
            for key in ["id", "decision", "algorithm", "algorithm_version", "confidence"] {
                assert!(
                    edge["provenance"].get(key).is_some(),
                    "asserted provenance carries {key}: {edge}"
                );
            }
        }
    }
    if expect_asserted {
        assert!(saw_asserted, "asserted provenance travels: {explanation}");
        // The suggestion's confidence value round-trips through every surface.
        let confidences: Vec<&Value> = edges
            .iter()
            .filter(|edge| edge["provenance"]["kind"] == "asserted")
            .map(|edge| &edge["provenance"]["confidence"])
            .collect();
        assert!(
            confidences.iter().any(|confidence| *confidence == &Value::from(0.5)),
            "suggestion confidence travels: {explanation}"
        );
    }
    // Applied filters name exactly the fixed inputs (never hidden IDs).
    assert_eq!(explanation["applied_filters"]["budgets"]["max_hops"], HOPS);
    assert_eq!(explanation["applied_filters"]["direction"], "both");
    assert_eq!(explanation["applied_filters"]["authz"], "unrestricted");
    // Snapshot build identity pins the answering build.
    assert_eq!(explanation["snapshot"]["projection_id"], "quran-structural-v1");
    assert!(explanation["snapshot"]["builder_version"].is_string());
    assert!(explanation["snapshot"]["corpus_generation"].is_number());
    // Completion status with reason: truncation always carries one.
    assert!(explanation["truncated"].is_boolean());
    assert_eq!(
        explanation["truncated"],
        Value::Bool(!explanation["incomplete_reason"].is_null()),
        "truncation always carries a reason: {explanation}"
    );
    // Duration travels (excluded from cross-surface comparison only).
    assert!(explanation["duration_ms"].is_u64(), "duration travels: {explanation}");
}

fn parity_options() -> application::quran_graph_api::ReadOptions {
    read_options(patch(), HOPS, None, None).expect("fixed options validate")
}

#[tokio::test]
async fn graph_parity_neighbors_across_cli_http_and_tools() {
    let (_dir, path_str) = shared_fixture().await;
    let service = GraphApiService::open(&path_str).await.expect("service opens");
    let backend = ParityBackend {
        service: GraphApiService::open(&path_str).await.expect("fake backend opens"),
    };
    let registry = GraphToolBackend::registry(Arc::new(
        GraphApiService::open(&path_str).await.expect("tool backend opens"),
    ));

    // CLI leg: the db verb with the fixed inputs.
    let cli =
        quran_cli::cmd_graph_neighbors(&path_str, None, true, "ayah:1:1", HOPS, &cli_budgets())
            .await;
    assert_eq!(cli.exit, 0, "CLI neighbors succeeds: {}", cli.human);
    let cli_projection = serde_json::json!({
        "nodes": cli.json["nodes"],
        "edges": cli.json["edges"],
        "truncated": cli.json["truncated"],
        "incomplete_reason": cli.json["incomplete_reason"],
        "explanation": cli.json["explanation"],
    });

    // HTTP leg: the serialized backend output the handler wraps verbatim.
    let http = serde_json::to_value(
        backend
            .neighbors(NeighborsArgs { node: "ayah:1:1".to_string(), options: parity_options() })
            .await
            .expect("fake backend answers"),
    )
    .expect("neighbors output serializes");

    // Tools leg: the result payload inside the tool envelope.
    let tool = registry
        .graph_neighbors(tool_registry::GraphNeighborsParams {
            node: "ayah:1:1".to_string(),
            edge_types: None,
            direction: None,
            budgets: tool_budgets(),
        })
        .await
        .expect("neighbors tool answers");

    assert_eq!(
        canonical(cli_projection.clone()),
        canonical(http.clone()),
        "CLI and HTTP agree on neighbors"
    );
    assert_eq!(
        canonical(http.clone()),
        canonical(tool.results.clone()),
        "HTTP and tools agree on neighbors"
    );
    assert_explained(&cli_projection["explanation"], Some("ayah:1:1"), None, true, true);
    assert_explained(&http["explanation"], Some("ayah:1:1"), None, true, true);
    assert_explained(&tool.results["explanation"], Some("ayah:1:1"), None, true, true);
    drop(service);
}

#[tokio::test]
async fn graph_parity_path_modes_across_cli_http_and_tools() {
    let (_dir, path_str) = shared_fixture().await;
    let backend = ParityBackend {
        service: GraphApiService::open(&path_str).await.expect("fake backend opens"),
    };
    let registry = GraphToolBackend::registry(Arc::new(
        GraphApiService::open(&path_str).await.expect("tool backend opens"),
    ));

    for mode in ["reachability", "shortest", "paths"] {
        let cli = quran_cli::cmd_graph_path(
            &path_str,
            None,
            true,
            "ayah:1:1",
            "ayah:1:2",
            HOPS,
            mode,
            Some(10),
            &cli_budgets(),
        )
        .await;
        assert_eq!(cli.exit, 0, "CLI path {mode} succeeds: {}", cli.human);

        let (cli_projection, http, tool_results) = match mode {
            "reachability" => {
                let cli_projection = serde_json::json!({
                    "reachable": cli.json["reachable"],
                    "hops": cli.json["hops"],
                    "explanation": cli.json["explanation"],
                });
                let http = serde_json::to_value(
                    backend
                        .reachability(PathArgs {
                            from: "ayah:1:1".to_string(),
                            to: "ayah:1:2".to_string(),
                            options: parity_options(),
                        })
                        .await
                        .expect("fake backend answers"),
                )
                .expect("reachability serializes");
                let tool = registry
                    .graph_path(tool_registry::GraphPathParams {
                        from: "ayah:1:1".to_string(),
                        to: "ayah:1:2".to_string(),
                        mode: Some(mode.to_string()),
                        paths: Some(10),
                        edge_types: None,
                        direction: None,
                        budgets: tool_budgets(),
                    })
                    .await
                    .expect("path tool answers");
                (cli_projection, http, tool.results)
            }
            "shortest" => {
                let cli_projection = serde_json::json!({
                    "path": cli.json["path"],
                    "explanation": cli.json["explanation"],
                });
                let http = serde_json::to_value(
                    backend
                        .shortest_path(PathArgs {
                            from: "ayah:1:1".to_string(),
                            to: "ayah:1:2".to_string(),
                            options: parity_options(),
                        })
                        .await
                        .expect("fake backend answers"),
                )
                .expect("shortest serializes");
                let tool = registry
                    .graph_path(tool_registry::GraphPathParams {
                        from: "ayah:1:1".to_string(),
                        to: "ayah:1:2".to_string(),
                        mode: Some(mode.to_string()),
                        paths: Some(10),
                        edge_types: None,
                        direction: None,
                        budgets: tool_budgets(),
                    })
                    .await
                    .expect("path tool answers");
                (cli_projection, http, tool.results)
            }
            _ => {
                let cli_projection = serde_json::json!({
                    "paths": cli.json["paths"],
                    "truncated": cli.json["truncated"],
                    "incomplete_reason": cli.json["incomplete_reason"],
                    "explanation": cli.json["explanation"],
                });
                let http = serde_json::to_value(
                    backend
                        .paths(PathsArgs {
                            from: "ayah:1:1".to_string(),
                            to: "ayah:1:2".to_string(),
                            k: 10,
                            options: parity_options(),
                        })
                        .await
                        .expect("fake backend answers"),
                )
                .expect("paths serialize");
                let tool = registry
                    .graph_path(tool_registry::GraphPathParams {
                        from: "ayah:1:1".to_string(),
                        to: "ayah:1:2".to_string(),
                        mode: Some(mode.to_string()),
                        paths: Some(10),
                        edge_types: None,
                        direction: None,
                        budgets: tool_budgets(),
                    })
                    .await
                    .expect("path tool answers");
                (cli_projection, http, tool.results)
            }
        };
        assert_eq!(
            canonical(cli_projection.clone()),
            canonical(http.clone()),
            "CLI and HTTP agree on path {mode}"
        );
        assert_eq!(
            canonical(http.clone()),
            canonical(tool_results.clone()),
            "HTTP and tools agree on path {mode}"
        );
        let (expect_edges, expect_asserted) = match mode {
            "reachability" => (false, false),
            "shortest" => (true, false),
            _ => (true, true),
        };
        assert_explained(
            &cli_projection["explanation"],
            Some("ayah:1:1"),
            Some("ayah:1:2"),
            expect_edges,
            expect_asserted,
        );
        assert_explained(
            &http["explanation"],
            Some("ayah:1:1"),
            Some("ayah:1:2"),
            expect_edges,
            expect_asserted,
        );
        assert_explained(
            &tool_results["explanation"],
            Some("ayah:1:1"),
            Some("ayah:1:2"),
            expect_edges,
            expect_asserted,
        );
    }
}

#[tokio::test]
async fn graph_parity_subgraph_and_pattern_across_cli_http_and_tools() {
    let (_dir, path_str) = shared_fixture().await;
    let backend = ParityBackend {
        service: GraphApiService::open(&path_str).await.expect("fake backend opens"),
    };
    let registry = GraphToolBackend::registry(Arc::new(
        GraphApiService::open(&path_str).await.expect("tool backend opens"),
    ));
    let seeds = vec!["ayah:1:1".to_string()];

    let cli = quran_cli::cmd_graph_subgraph(&path_str, &seeds, HOPS, &cli_budgets()).await;
    assert_eq!(cli.exit, 0, "CLI subgraph succeeds: {}", cli.human);
    let cli_projection = serde_json::json!({
        "nodes": cli.json["nodes"],
        "edges": cli.json["edges"],
        "truncated": cli.json["truncated"],
        "incomplete_reason": cli.json["incomplete_reason"],
        "explanation": cli.json["explanation"],
    });
    let http = serde_json::to_value(
        backend
            .subgraph(SubgraphArgs { seeds: seeds.clone(), options: parity_options() })
            .await
            .expect("fake backend answers"),
    )
    .expect("subgraph serializes");
    let tool = registry
        .graph_subgraph(tool_registry::GraphSubgraphParams {
            seeds: seeds.clone(),
            edge_types: None,
            direction: None,
            budgets: tool_budgets(),
        })
        .await
        .expect("subgraph tool answers");
    assert_eq!(canonical(cli_projection.clone()), canonical(http.clone()), "CLI and HTTP agree");
    assert_eq!(canonical(http.clone()), canonical(tool.results.clone()), "HTTP and tools agree");
    assert_explained(&cli_projection["explanation"], Some("ayah:1:1"), None, true, true);
    assert_explained(&http["explanation"], Some("ayah:1:1"), None, true, true);
    assert_explained(&tool.results["explanation"], Some("ayah:1:1"), None, true, true);

    let steps = vec!["NEXT".to_string()];
    let cli = quran_cli::cmd_graph_pattern(&path_str, &seeds, &steps, HOPS, &cli_budgets()).await;
    assert_eq!(cli.exit, 0, "CLI pattern succeeds: {}", cli.human);
    let cli_projection = serde_json::json!({
        "nodes": cli.json["nodes"],
        "edges": cli.json["edges"],
        "truncated": cli.json["truncated"],
        "incomplete_reason": cli.json["incomplete_reason"],
        "explanation": cli.json["explanation"],
    });
    let http = serde_json::to_value(
        backend
            .pattern(PatternArgs {
                pattern: parse_pattern_steps(vec![PatternStepRequest {
                    edge: "NEXT".to_string(),
                    node_kind: None,
                }])
                .expect("NEXT validates"),
                seeds: seeds.clone(),
                options: parity_options(),
            })
            .await
            .expect("fake backend answers"),
    )
    .expect("pattern serializes");
    let tool = registry
        .graph_pattern(tool_registry::GraphPatternParams {
            seeds: seeds.clone(),
            steps: vec![tool_registry::GraphPatternStep {
                edge: "NEXT".to_string(),
                node_kind: None,
            }],
            budgets: tool_budgets(),
        })
        .await
        .expect("pattern tool answers");
    assert_eq!(canonical(cli_projection.clone()), canonical(http.clone()), "CLI and HTTP agree");
    assert_eq!(canonical(http.clone()), canonical(tool.results.clone()), "HTTP and tools agree");
    // NEXT-only steps traverse structural edges: explained, but no asserted
    // provenance on this leg.
    assert_explained(&cli_projection["explanation"], Some("ayah:1:1"), None, true, false);
    assert_explained(&http["explanation"], Some("ayah:1:1"), None, true, false);
    assert_explained(&tool.results["explanation"], Some("ayah:1:1"), None, true, false);
}

/// A truncated query reports `truncated: true` with a non-empty reason on
/// all three surfaces — never an error, never an empty result masking a
/// partial, and never diverging.
#[tokio::test]
async fn graph_parity_truncated_reports_unanimously() {
    let (_dir, path_str) = shared_fixture().await;
    let backend = ParityBackend {
        service: GraphApiService::open(&path_str).await.expect("fake backend opens"),
    };
    let registry = GraphToolBackend::registry(Arc::new(
        GraphApiService::open(&path_str).await.expect("tool backend opens"),
    ));
    let tight_cli = GraphBudgets {
        max_nodes: Some(1),
        max_edges: Some(2000),
        max_paths: Some(10),
        max_fanout: Some(128),
        timeout_ms: Some(5000),
    };
    let tight_patch = BudgetPatch {
        max_hops: Some(HOPS),
        max_nodes: Some(1),
        max_edges: Some(2000),
        max_paths: Some(10),
        max_fanout: Some(128),
        timeout_ms: Some(5000),
    };
    let tight_tools = tool_registry::GraphBudgetsParams {
        max_hops: Some(HOPS),
        max_nodes: Some(1),
        max_edges: Some(2000),
        max_paths: Some(10),
        max_fanout: Some(128),
        timeout_ms: Some(5000),
    };
    let tight_options =
        read_options(tight_patch, HOPS, None, None).expect("tight options validate");

    let cli =
        quran_cli::cmd_graph_neighbors(&path_str, None, true, "ayah:1:1", HOPS, &tight_cli).await;
    assert_eq!(cli.exit, 0, "truncation is success with a flag: {}", cli.human);
    let cli_projection = serde_json::json!({
        "nodes": cli.json["nodes"],
        "edges": cli.json["edges"],
        "truncated": cli.json["truncated"],
        "incomplete_reason": cli.json["incomplete_reason"],
        "explanation": cli.json["explanation"],
    });
    let http = serde_json::to_value(
        backend
            .neighbors(NeighborsArgs { node: "ayah:1:1".to_string(), options: tight_options })
            .await
            .expect("fake backend answers"),
    )
    .expect("neighbors output serializes");
    let tool = registry
        .graph_neighbors(tool_registry::GraphNeighborsParams {
            node: "ayah:1:1".to_string(),
            edge_types: None,
            direction: None,
            budgets: tight_tools,
        })
        .await
        .expect("neighbors tool answers");

    for (surface, value) in [("CLI", &cli_projection), ("HTTP", &http), ("tools", &tool.results)] {
        assert_eq!(value["truncated"], true, "{surface} reports truncation");
        assert!(
            !value["incomplete_reason"].as_str().unwrap_or_default().is_empty(),
            "{surface} carries the reason"
        );
        assert_eq!(value["explanation"]["truncated"], true, "{surface} explanation agrees");
    }
    assert_eq!(
        canonical(cli_projection.clone()),
        canonical(http.clone()),
        "CLI and HTTP agree on the partial"
    );
    assert_eq!(
        canonical(http.clone()),
        canonical(tool.results.clone()),
        "HTTP and tools agree on the partial"
    );
}
