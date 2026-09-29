//! Phase 4 graph tools: registry names, attributed envelopes with
//! projection-pinned reproducibility, service-identical payloads, typed
//! errors preserving `QAI-GRAPH` codes, and pending-suggestion labeling
//! (plan 04-04, D-12/T-04-12).
//!
//! Against real SQLite in a tempdir with an imported + activated edition
//! and a structural projection: the five graph tools answer through the
//! same read backend the CLI and HTTP surfaces use, so payloads agree by
//! construction and the tests pin the wiring (envelopes, attribution,
//! reproducibility, warnings).

mod common;

use std::sync::Arc;

use application::quran_graph_api::{
    BudgetPatch, GraphApiService, GraphBackend, NeighborsArgs, PathArgs, SubgraphArgs, read_options,
};
use application::quran_graph_build::{collect_structural_input, publish_structural_build};
use application::quran_graph_tools::GraphToolBackend;
use domain::SemVer;
use tool_registry::{
    GRAPH_NEIGHBORS_TOOL_VERSION, GRAPH_PATH_TOOL_VERSION, GRAPH_PATTERN_TOOL_VERSION,
    GRAPH_ROOT_FAMILY_TOOL_VERSION, GRAPH_SUBGRAPH_TOOL_VERSION, ToolRegistry,
};
use tools::ToolError;

async fn live_registry() -> (tempfile::TempDir, GraphApiService, ToolRegistry) {
    let (dir, db, _reader, path_str) = common::active_reader().await;
    let collected = collect_structural_input(&db)
        .await
        .expect("collect from active edition")
        .expect("the harness activates test-edition-min, so collection succeeds");
    publish_structural_build(&path_str, &collected).await.expect("structural build publishes");
    drop(db);
    let service = GraphApiService::open(&path_str).await.expect("read service opens");
    let registry = GraphToolBackend::registry(Arc::new(
        GraphApiService::open(&path_str).await.expect("tool backend opens"),
    ));
    (dir, service, registry)
}

/// Twelve registered names with their own SemVer 1.0.0 version consts, and
/// no mutation tool in the set (D-11 fence).
#[tokio::test]
async fn graph_tools_register_twelve_names_with_version_consts() {
    let (_dir, _service, registry) = live_registry().await;
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
        ]
    );
    for version in [
        GRAPH_NEIGHBORS_TOOL_VERSION,
        GRAPH_PATH_TOOL_VERSION,
        GRAPH_SUBGRAPH_TOOL_VERSION,
        GRAPH_PATTERN_TOOL_VERSION,
        GRAPH_ROOT_FAMILY_TOOL_VERSION,
    ] {
        assert_eq!(version, SemVer::new(1, 0, 0));
    }
    for name in registry.tool_names() {
        for banned in [
            "propose", "suggest", "accept", "reject", "dispute", "correct", "review", "build",
            "repair",
        ] {
            assert!(!name.contains(banned), "no mutation tool in the registry: {name}");
        }
    }
}

/// The neighbors envelope carries canonical references, analysis sources,
/// and a projection-pinned reproducibility checksum — over the exact
/// read-service payload for fixed inputs.
#[tokio::test]
async fn graph_neighbors_envelope_is_attributed_and_service_identical() {
    let (_dir, service, registry) = live_registry().await;
    let params = tool_registry::GraphNeighborsParams {
        node: "ayah:1:1".to_string(),
        edge_types: None,
        direction: None,
        budgets: tool_registry::GraphBudgetsParams::default(),
    };
    let tool = registry.graph_neighbors(params).await.expect("neighbors tool answers");
    assert_eq!(tool.tool_name, "quran.graph_neighbors");
    assert_eq!(tool.tool_version, SemVer::new(1, 0, 0));
    assert!(!tool.canonical_references.is_empty(), "every result names its sources");
    assert!(
        tool.canonical_references.iter().all(|reference| !reference.trim().is_empty()),
        "no empty references: {:?}",
        tool.canonical_references
    );
    // Ayah hits pin to fully-qualified references; every other node kind
    // travels as its stable ID (never dropped, never invented).
    for pinned in ["quran:test-edition-min@0.1.0:1:1", "quran:test-edition-min@0.1.0:1:2"] {
        assert!(
            tool.canonical_references.iter().any(|reference| reference == pinned),
            "pinned ayah ref travels: {:?}",
            tool.canonical_references
        );
    }
    assert!(
        tool.canonical_references.iter().any(|reference| reference == "surah:1"),
        "non-ayah nodes keep their stable IDs: {:?}",
        tool.canonical_references
    );
    assert_eq!(tool.analysis_sources.len(), 1);
    assert_eq!(tool.analysis_sources[0].kind, "graph");
    assert!(
        tool.analysis_sources[0].reference.starts_with("quran-structural-v1@"),
        "the graph source pins the answering build: {}",
        tool.analysis_sources[0].reference
    );
    // Reproducibility pins the graph build alongside the edition.
    assert!(tool.reproducibility.deterministic);
    assert_eq!(
        tool.reproducibility.source_versions.get("graph_projection").map(String::as_str),
        Some("quran-structural-v1")
    );
    assert!(
        tool.reproducibility
            .source_versions
            .get("graph_builder")
            .map(String::as_str)
            .unwrap_or_default()
            .starts_with("structural-v1"),
        "builder version travels: {:?}",
        tool.reproducibility.source_versions
    );
    assert!(tool.edition_version.is_some());

    // Identical payloads to the read service for fixed inputs: the direct
    // call uses the same folded budgets the tool applies (neighbors opens
    // one hop by default, mirroring the CLI verb).
    let direct = service
        .neighbors(NeighborsArgs {
            node: "ayah:1:1".to_string(),
            options: read_options(BudgetPatch::default(), 1, None, None)
                .expect("default options validate"),
        })
        .await
        .expect("direct service answers");
    let expected = serde_json::to_value(&direct).expect("service output serializes");
    let mut tool_results = tool.results.clone();
    let mut expected_results = expected;
    // Duration milliseconds are presentational only: excluded from parity.
    tool_results["explanation"]["duration_ms"] = serde_json::Value::Null;
    expected_results["explanation"]["duration_ms"] = serde_json::Value::Null;
    assert_eq!(tool_results, expected_results);

    // The checksum is deterministic: same inputs reproduce it exactly.
    let again = registry
        .graph_neighbors(tool_registry::GraphNeighborsParams {
            node: "ayah:1:1".to_string(),
            edge_types: None,
            direction: None,
            budgets: tool_registry::GraphBudgetsParams::default(),
        })
        .await
        .expect("neighbors tool answers again");
    assert_eq!(tool.reproducibility.checksum, again.reproducibility.checksum);
}

/// Path modes, subgraph, and patterns answer attributed envelopes over
/// their service payloads; root-family carries dataset attribution (or the
/// typed unavailability when no dataset is active).
#[tokio::test]
async fn graph_path_subgraph_pattern_and_root_family_envelopes() {
    let (_dir, service, registry) = live_registry().await;

    let path = registry
        .graph_path(tool_registry::GraphPathParams {
            from: "ayah:1:1".to_string(),
            to: "ayah:1:2".to_string(),
            mode: Some("shortest".to_string()),
            paths: None,
            edge_types: None,
            direction: None,
            budgets: tool_registry::GraphBudgetsParams::default(),
        })
        .await
        .expect("path tool answers");
    assert_eq!(path.tool_name, "quran.graph_path");
    assert!(!path.canonical_references.is_empty());
    assert_eq!(path.analysis_sources[0].kind, "graph");
    let direct = service
        .shortest_path(PathArgs {
            from: "ayah:1:1".to_string(),
            to: "ayah:1:2".to_string(),
            options: read_options(BudgetPatch::default(), 4, None, None)
                .expect("default options validate"),
        })
        .await
        .expect("direct shortest answers");
    let mut tool_results = path.results.clone();
    let mut expected_results = serde_json::to_value(&direct).expect("shortest serializes");
    tool_results["explanation"]["duration_ms"] = serde_json::Value::Null;
    expected_results["explanation"]["duration_ms"] = serde_json::Value::Null;
    assert_eq!(tool_results, expected_results, "path payloads match modulo duration");

    let subgraph = registry
        .graph_subgraph(tool_registry::GraphSubgraphParams {
            seeds: vec!["ayah:1:1".to_string()],
            edge_types: None,
            direction: None,
            budgets: tool_registry::GraphBudgetsParams::default(),
        })
        .await
        .expect("subgraph tool answers");
    assert_eq!(subgraph.tool_name, "quran.graph_subgraph");
    let direct = service
        .subgraph(SubgraphArgs {
            seeds: vec!["ayah:1:1".to_string()],
            options: read_options(BudgetPatch::default(), 4, None, None)
                .expect("default options validate"),
        })
        .await
        .expect("direct subgraph answers");
    let mut tool_results = subgraph.results.clone();
    let mut expected_results = serde_json::to_value(&direct).expect("subgraph serializes");
    tool_results["explanation"]["duration_ms"] = serde_json::Value::Null;
    expected_results["explanation"]["duration_ms"] = serde_json::Value::Null;
    assert_eq!(tool_results, expected_results);

    let pattern = registry
        .graph_pattern(tool_registry::GraphPatternParams {
            seeds: vec!["ayah:1:1".to_string()],
            steps: vec![tool_registry::GraphPatternStep {
                edge: "NEXT".to_string(),
                node_kind: None,
            }],
            budgets: tool_registry::GraphBudgetsParams::default(),
        })
        .await
        .expect("pattern tool answers");
    assert_eq!(pattern.tool_name, "quran.graph_pattern");
    assert_eq!(
        pattern.reproducibility.source_versions.get("graph_projection").map(String::as_str),
        Some("quran-structural-v1")
    );

    // K beyond max_paths is a pre-flight backend error through the tool.
    let err = registry
        .graph_path(tool_registry::GraphPathParams {
            from: "ayah:1:1".to_string(),
            to: "ayah:1:2".to_string(),
            mode: Some("paths".to_string()),
            paths: Some(10_000),
            edge_types: None,
            direction: None,
            budgets: tool_registry::GraphBudgetsParams::default(),
        })
        .await
        .expect_err("K beyond max_paths is a pre-flight error");
    assert!(
        matches!(&err, ToolError::Backend { code, .. } if code == "QAI-GRAPH-0002"),
        "unexpected error: {err:?}"
    );

    match registry
        .graph_root_family(tool_registry::GraphRootFamilyParams {
            root: "r-1".to_string(),
            limit: None,
        })
        .await
    {
        Ok(family) => {
            assert_eq!(family.tool_name, "quran.graph_root_family");
            assert_eq!(family.analysis_sources[0].kind, "dataset");
            assert!(
                !family.analysis_sources[0].reference.is_empty(),
                "lexicon results attribute their dataset"
            );
        }
        Err(ToolError::Backend { code, .. }) => {
            assert_eq!(code, "QAI-MORPH-0004", "no dataset is the typed gate, never empty");
        }
        Err(other) => panic!("unexpected root-family error: {other:?}"),
    }
}

/// Tool errors preserve the namespaced codes: unknown nodes 404-class,
/// pre-flight violations 422-class, and empty selectors are registry-level
/// invalid input.
#[tokio::test]
async fn graph_tool_errors_preserve_namespaced_codes() {
    let (_dir, _service, registry) = live_registry().await;
    let err = registry
        .graph_neighbors(tool_registry::GraphNeighborsParams {
            node: "ayah:9:9".to_string(),
            edge_types: None,
            direction: None,
            budgets: tool_registry::GraphBudgetsParams::default(),
        })
        .await
        .expect_err("unknown node is a typed error");
    assert!(
        matches!(&err, ToolError::Backend { code, .. } if code == "QAI-GRAPH-0004"),
        "unexpected error: {err:?}"
    );

    let err = registry
        .graph_neighbors(tool_registry::GraphNeighborsParams {
            node: "ayah:1:1".to_string(),
            edge_types: None,
            direction: None,
            budgets: tool_registry::GraphBudgetsParams {
                max_hops: Some(0),
                ..tool_registry::GraphBudgetsParams::default()
            },
        })
        .await
        .expect_err("out-of-range budget is a pre-flight error");
    assert!(
        matches!(&err, ToolError::Backend { code, .. } if code == "QAI-GRAPH-0002"),
        "unexpected error: {err:?}"
    );

    let err = registry
        .graph_path(tool_registry::GraphPathParams {
            from: "ayah:1:1".to_string(),
            to: "ayah:1:2".to_string(),
            mode: Some("sideways".to_string()),
            paths: None,
            edge_types: None,
            direction: None,
            budgets: tool_registry::GraphBudgetsParams::default(),
        })
        .await
        .expect_err("unknown mode is a typed rejection");
    assert!(
        matches!(&err, ToolError::Backend { code, .. } if code == "QAI-GRAPH-0003"),
        "unexpected error: {err:?}"
    );

    let err = registry
        .graph_neighbors(tool_registry::GraphNeighborsParams {
            node: "   ".to_string(),
            edge_types: None,
            direction: None,
            budgets: tool_registry::GraphBudgetsParams::default(),
        })
        .await
        .expect_err("empty selectors never reach the backend");
    assert!(matches!(err, ToolError::InvalidInput { .. }), "unexpected error: {err:?}");
}

/// Seed one layer-D pending suggestion edge plus one accepted edge on the
/// same pair: the suggestion is labeled pending with its algorithm
/// attribution in the envelope warnings — never presented as verified.
async fn seed_suggestion(path_str: &str, row_id: &str, edition_id: &str) {
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
    // (suffix, decision, layer, reviewer, decided_at, algorithm triple)
    let rows = [
        ("sug", "pending", "D", None, None, Some(("test-suggester", "0.1.0", 0.5))),
        ("acc", "accepted", "B", Some("tracer-scholar"), Some("2026-09-28T00:00:00Z"), None),
    ];
    for (suffix, decision, layer, reviewer, decided_at, algorithm) in rows {
        let aid = format!("tools-{suffix}");
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
             VALUES (?, ?, ?, ?, 'tools-seed',
                     'annotation', '{}', '{\"source\": \"tools-seed\"}', 'tools-seed:row-1',
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
        .bind(format!("tools-edge-{suffix}"))
        .bind(row_id)
        .bind(&aid)
        .execute(&pool)
        .await
        .expect("suggestion edge inserts");
    }
    pool.close().await;
}

#[tokio::test]
async fn graph_suggestion_results_are_labeled_pending_in_warnings() {
    let (dir, service, _registry) = live_registry().await;
    let path_str = dir.path().join("qai.db").to_str().unwrap().to_string();
    let row_id = service.manifest().id.clone();
    let edition_id = service.manifest().edition_id.clone();
    seed_suggestion(&path_str, &row_id, &edition_id).await;
    // The store snapshots on open: reopen so the seeded rows are visible.
    let service = GraphApiService::open(&path_str).await.expect("service reopens");
    let registry = GraphToolBackend::registry(Arc::new(
        GraphApiService::open(&path_str).await.expect("tool backend opens"),
    ));
    let tool = registry
        .graph_neighbors(tool_registry::GraphNeighborsParams {
            node: "ayah:1:1".to_string(),
            edge_types: None,
            direction: None,
            budgets: tool_registry::GraphBudgetsParams::default(),
        })
        .await
        .expect("neighbors tool answers");
    assert_eq!(tool.warnings.len(), 1, "one pending suggestion is labeled: {:?}", tool.warnings);
    assert!(
        tool.warnings[0].contains("pending human review"),
        "pending label: {}",
        tool.warnings[0]
    );
    assert!(
        tool.warnings[0].contains("test-suggester"),
        "algorithm attribution: {}",
        tool.warnings[0]
    );
    // The suggestion is pending in the explanation too — never verified.
    let decisions: Vec<String> = tool.results["explanation"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["provenance"]["kind"] == "asserted")
        .filter_map(|edge| edge["provenance"]["decision"].as_str().map(str::to_string))
        .collect();
    assert!(
        decisions.iter().any(|decision| decision == "pending"),
        "a pending assertion travels: {}",
        tool.results["explanation"]["edges"]
    );
    assert!(
        decisions.iter().all(|decision| decision != "verified"),
        "nothing claims a verified status it was never granted"
    );
    drop(service);
}
