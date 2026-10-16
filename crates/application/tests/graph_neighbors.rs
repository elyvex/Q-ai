//! Phase 4 tracer — structural projection build into SQLite, manifest
//! inspect, bounded neighbors, and per-hit pinned canonical quotation
//! (plan 04-01, D-01/D-02/D-10/D-11/D-14).
//!
//! Against real SQLite in a tempdir with a real imported + activated edition:
//! the structural projection builds from the active edition, the manifest
//! pins `quran-structural-v1` with the active corpus generation, neighbors
//! around an ayah are bounded and deterministic, every ayah hit re-verifies
//! byte-identical through the canonical reader, unknown nodes report typed
//! `NodeNotFound`, and one node pair carries multiple attributed assertions
//! (plus a hidden tombstoned one) without constraint violation.

mod common;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use application::quran_cli::{GraphBudgets, cmd_graph_neighbors};
use application::quran_graph_api::{
    FileGraphBackend, GraphApiError, GraphApiService, GraphBackend as _, NeighborsArgs, ReadOptions,
};
use application::quran_graph_build::{
    ANNOTATED_PROJECTION_ID, collect_structural_input, publish_annotated_build,
    publish_structural_build,
};
use application::quran_graph_store::SqliteGraphStore;
use application::quran_graph_tools::GraphToolBackend;
use application::quran_reader::{QuranReader, QuranReaderService};
use application::quran_tools::verify_canonical_quotation;
use quran_graph::{
    AuthzScope, EdgeFilter, GraphError, GraphStore, NodeKind, QueryBudgets,
    STRUCTURAL_PROJECTION_ID,
};
use serde_json::{Value, json};

const CONCEPT_SEED: &str = include_str!("../../../fixtures/quran/graph/concept-seed-v1.json");

/// Neighbor-view depth every annotated leg runs at, so canonicalized parity
/// compares reads that asked the same question.
const HOPS: usize = 2;

/// One selector-bearing read-options value for the file-backed legs: the
/// annotated family at the shared budgets.
fn annotated_options() -> ReadOptions {
    ReadOptions {
        budgets: QueryBudgets { max_hops: HOPS, ..QueryBudgets::default() },
        projection: Some(ANNOTATED_PROJECTION_ID.to_string()),
        ..ReadOptions::default()
    }
}

fn annotated_tool_budgets() -> tool_registry::GraphBudgetsParams {
    tool_registry::GraphBudgetsParams {
        max_hops: Some(HOPS),
        ..tool_registry::GraphBudgetsParams::default()
    }
}

/// Strip the presentational-only duration wherever it appears: the one field
/// excluded from cross-surface parity.
fn canonical(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.remove("duration_ms");
            for nested in map.values_mut() {
                canonical(nested);
            }
        }
        Value::Array(items) => {
            for nested in items {
                canonical(nested);
            }
        }
        _ => {}
    }
}

fn budgets() -> QueryBudgets {
    QueryBudgets::default()
}

fn open_scope() -> (AtomicBool, AuthzScope) {
    (AtomicBool::new(false), AuthzScope::all_visible())
}

async fn active_store() -> (
    tempfile::TempDir,
    Arc<storage_sqlite::SqliteDatabase>,
    Arc<QuranReaderService>,
    String,
    SqliteGraphStore,
) {
    let (dir, db, reader, path_str) = common::active_reader().await;
    let collected = collect_structural_input(&db)
        .await
        .expect("collect from active edition")
        .expect("the common harness activates test-edition-min, so input collection succeeds");
    let report =
        publish_structural_build(&path_str, &collected).await.expect("structural build publishes");
    assert_eq!(report.manifest.projection_id, STRUCTURAL_PROJECTION_ID);
    assert_eq!(
        report.manifest.corpus_generation, 1,
        "generation stamp comes from the active edition row"
    );
    assert!(report.node_count > 0 && report.edge_count > 0);
    let store = SqliteGraphStore::open_active(&db, STRUCTURAL_PROJECTION_ID)
        .await
        .expect("active projection opens");
    assert_eq!(store.manifest().projection_id, STRUCTURAL_PROJECTION_ID);
    assert_eq!(store.manifest().corpus_generation, 1);
    let inspection = store.inspect();
    assert_eq!(inspection.node_count, report.node_count);
    assert_eq!(inspection.edge_count, report.edge_count);
    (dir, db, Arc::new(reader), path_str, store)
}

#[tokio::test]
async fn structural_build_inspect_neighbors_canonical_quotation() {
    let (_dir, _db, reader, _path, store) = active_store().await;
    let (cancel, authz) = open_scope();

    // Bounded neighbor view around a verse: complete, deterministic.
    let first = store
        .neighbors("ayah:1:1", &EdgeFilter::any(), &budgets(), &cancel, &authz)
        .expect("neighbors open around ayah:1:1");
    assert!(!first.truncated, "the fixture neighborhood fits default budgets");
    assert!(!first.nodes.is_empty() && !first.edges.is_empty());
    let second = store
        .neighbors("ayah:1:1", &EdgeFilter::any(), &budgets(), &cancel, &authz)
        .expect("neighbors reopen");
    assert_eq!(first, second, "equal budgets plus equal visible input order equally");

    // Every ayah hit links to a pinned canonical ref that re-verifies
    // byte-identical through the reader (never from graph record text).
    let mut ayah_hits = 0;
    for node in &first.nodes {
        if node.kind != NodeKind::Ayah {
            continue;
        }
        ayah_hits += 1;
        let (surah, ayah) = parse_ayah(node);
        let reference = quran_core::QuranRef::Ayah {
            edition: quran_core::EditionSelector::Pinned {
                slug: common::SLUG.to_string(),
                version: common::VERSION.parse().expect("fixture version parses"),
            },
            surah: quran_core::SurahNumber::new(surah).expect("valid surah"),
            ayah: quran_core::AyahNumber::new(ayah).expect("valid ayah"),
        };
        let view = reader
            .get_ayah(&reference, &quran_core::AyahOptions::default())
            .await
            .expect("ayah hit resolves through the reader");
        assert_eq!(
            view.canonical.reference(),
            format!("quran:{}@{}:{surah}:{ayah}", common::SLUG, common::VERSION),
            "the hit links to its pinned canonical ref"
        );
        let text = view.canonical.arabic_text().to_string();
        let (verdict, text_hash) =
            verify_canonical_quotation(&reader, common::SLUG, common::VERSION, surah, ayah, &text)
                .await
                .expect("quotation re-verifies");
        assert!(
            matches!(verdict, citations::QuotationVerdict::ExactMatch),
            "re-verification is exact: {verdict:?}"
        );
        assert_eq!(
            text_hash,
            format!("sha256:{}", view.canonical.text_hash().hex),
            "hash resolves, never synthesized"
        );
    }
    assert!(ayah_hits >= 2, "the view carries ayah hits to pin (found {ayah_hits})");

    // Unknown nodes are typed NodeNotFound, never empty-complete.
    let (cancel, authz) = open_scope();
    let err = store
        .neighbors("ayah:99:99", &EdgeFilter::any(), &budgets(), &cancel, &authz)
        .expect_err("unknown node is an error");
    assert!(
        matches!(err, GraphError::NodeNotFound { .. }),
        "unknown nodes report NodeNotFound: {err}"
    );
    assert_eq!(
        quran_graph::Diagnostic::code(&err).to_string(),
        "QAI-GRAPH-0004",
        "unknown nodes carry QAI-GRAPH-0004"
    );
}

/// SC1 concept leg (gap G-04-SC1): the concept/entity neighborhood is
/// reachable, but only through the annotated projection — and the selector
/// that reaches it is the same one on every surface.
///
/// Proves: the structural default hides the concept node (typed
/// `NodeNotFound`, never empty-complete); the annotated selector reveals it
/// identically through the read service, the request-scoped file backend,
/// the `quran.graph_neighbors` tool, and the CLI verb; every seed edge keeps
/// the seed's attribution assertion and no concept/entity node claims
/// reviewed scholarship; an unknown family is a typed `UnknownProjection`
/// before any traversal.
#[tokio::test]
async fn concept_neighborhood_is_reachable_only_through_the_annotated_projection() {
    let (_dir, _db, _reader, path_str, _structural) = active_store().await;
    let report = publish_annotated_build(&path_str, CONCEPT_SEED)
        .await
        .expect("the annotated projection publishes");
    assert_eq!(report.manifest.projection_id, ANNOTATED_PROJECTION_ID);

    // (a) The default read scope is unchanged: the concept node is out of the
    // structural family, as a typed error.
    let pinned = GraphApiService::open(&path_str).await.expect("structural opens");
    let err = pinned
        .neighbors(NeighborsArgs {
            node: "concept:mercy".to_string(),
            options: ReadOptions::default(),
        })
        .await
        .expect_err("concept:mercy is out of the structural default scope");
    assert!(
        matches!(err, GraphApiError::Graph(GraphError::NodeNotFound { .. })),
        "the selector-less read stays a typed NodeNotFound: {err}"
    );
    drop(pinned);

    // (b) The annotated family reaches it, keep its pending attribution, and
    // never claims reviewed scholarship.
    let annotated = GraphApiService::open_for_projection(&path_str, ANNOTATED_PROJECTION_ID)
        .await
        .expect("the annotated projection opens");
    let direct = annotated
        .neighbors(NeighborsArgs {
            node: "concept:mercy".to_string(),
            options: annotated_options(),
        })
        .await
        .expect("the concept neighborhood opens under the annotated selector");
    assert!(!direct.nodes.is_empty() && !direct.edges.is_empty(), "the seed neighborhood is real");
    assert_eq!(direct.explanation.snapshot.projection_id, ANNOTATED_PROJECTION_ID);
    for edge in &direct.edges {
        assert_eq!(
            edge.assertion_id.as_deref(),
            Some(report.attribution_assertion_id.as_str()),
            "seed edges stay seed-attributed: {edge:?}"
        );
    }
    for node in &direct.nodes {
        if matches!(node.kind, NodeKind::Concept | NodeKind::Entity) {
            assert_eq!(
                node.attrs.get("reviewed_by").and_then(Value::as_str),
                Some("pending-scholar"),
                "no concept/entity node claims reviewed scholarship: {node:?}"
            );
        }
    }
    let mut service_payload = serde_json::to_value(&direct).expect("the direct read serializes");
    canonical(&mut service_payload);

    // (c) The request-scoped file backend (the shared HTTP/tool path)
    // resolves the same selector to the same canonicalized payload.
    let backend = FileGraphBackend::structural(&path_str);
    let over_backend = backend
        .neighbors(NeighborsArgs {
            node: "concept:mercy".to_string(),
            options: annotated_options(),
        })
        .await
        .expect("the file backend reads the annotated family per request");
    let mut via_backend = serde_json::to_value(&over_backend).expect("the backend read serializes");
    canonical(&mut via_backend);
    assert_eq!(via_backend, service_payload, "the file backend matches the direct service read");

    // (d) The tool surface shares the one resolution point.
    let registry = GraphToolBackend::registry(Arc::new(FileGraphBackend::structural(&path_str)));
    let tool = registry
        .graph_neighbors(tool_registry::GraphNeighborsParams {
            node: "concept:mercy".to_string(),
            projection: Some(ANNOTATED_PROJECTION_ID.to_string()),
            edge_types: None,
            direction: None,
            budgets: annotated_tool_budgets(),
        })
        .await
        .expect("the neighbors tool forwards the selector");
    let mut tool_results = tool.results.clone();
    canonical(&mut tool_results);
    assert_eq!(tool_results, service_payload, "the tool payload matches the direct read");
    assert_eq!(
        tool.reproducibility.source_versions.get("graph_projection").map(String::as_str),
        Some(ANNOTATED_PROJECTION_ID),
        "the tool's reproducibility block pins the answering family"
    );

    // (e) The CLI verb resolves the same selector through the same point.
    let cli = cmd_graph_neighbors(
        &path_str,
        None,
        true,
        "concept:mercy",
        HOPS,
        &GraphBudgets::default(),
        Some(ANNOTATED_PROJECTION_ID),
    )
    .await;
    assert_eq!(cli.exit, 0, "CLI neighbors succeeds: {}", cli.human);
    let mut cli_projection = json!({
        "nodes": cli.json["nodes"],
        "edges": cli.json["edges"],
        "truncated": cli.json["truncated"],
        "incomplete_reason": cli.json["incomplete_reason"],
        "explanation": cli.json["explanation"],
    });
    canonical(&mut cli_projection);
    assert_eq!(cli_projection, service_payload, "the CLI shares the one resolution point");

    // (f) An unknown or unbuilt family fails before traversal: a typed
    // UnknownProjection, never an empty success.
    let err = FileGraphBackend::structural(&path_str)
        .neighbors(NeighborsArgs {
            node: "concept:mercy".to_string(),
            options: ReadOptions {
                projection: Some("quran-not-a-family".to_string()),
                ..ReadOptions::default()
            },
        })
        .await
        .expect_err("an unbuilt family is UnknownProjection");
    assert!(
        matches!(err, GraphApiError::Graph(GraphError::UnknownProjection { .. })),
        "an unknown family is typed UnknownProjection: {err}"
    );
}

#[tokio::test]
async fn one_triple_carries_multiple_attributed_assertions() {
    let (_dir, db, _reader, path_str, store) = active_store().await;
    let (cancel, authz) = open_scope();
    let view = store
        .neighbors("ayah:1:1", &EdgeFilter::any(), &budgets(), &cancel, &authz)
        .expect("neighbors open");
    let anchor = view.edges.first().expect("the fixture neighborhood has edges").clone();
    let row_id = store.projection_row_id().to_string();
    let edition_id = store.manifest().edition_id.clone();
    drop(store);

    // Two accepted attributions plus one tombstoned rejection on one triple.
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
    for (suffix, decision) in [("m1", "accepted"), ("m2", "accepted"), ("tomb", "rejected")] {
        let aid = format!("tracer-{suffix}");
        sqlx::query(
            "INSERT INTO graph_assertions
               (id, projection_row_id, projection_id, edition_id, dataset_scope,
                assertion_kind, claim_json, evidence_json, source_location,
                reviewer, decision, decided_at,
                provenance_layer, created_at)
             VALUES (?, ?, ?, ?, 'tracer',
                     'annotation', '{}', '{}', 'tracer-seed',
                     'tracer', ?, '2026-09-28T00:00:00Z',
                     'B', '2026-09-28T00:00:00Z')",
        )
        .bind(&aid)
        .bind(&row_id)
        .bind(STRUCTURAL_PROJECTION_ID)
        .bind(&edition_id)
        .bind(decision)
        .execute(&pool)
        .await
        .expect("assertion inserts");
        sqlx::query(
            "INSERT INTO graph_edges
               (id, projection_row_id, src_stable_id, edge, dst_stable_id,
                assertion_id, budgets_json, attrs_json, created_at)
             VALUES (?, ?, ?, ?, ?, ?, '{}', '{}', '2026-09-28T00:00:00Z')",
        )
        .bind(format!("tracer-edge-{suffix}"))
        .bind(&row_id)
        .bind(&anchor.src)
        .bind(&anchor.edge)
        .bind(&anchor.dst)
        .bind(&aid)
        .execute(&pool)
        .await
        .expect("second attribution on one triple inserts without constraint error");
    }
    pool.close().await;

    let reopened = SqliteGraphStore::open(&db, &row_id).await.expect("projection reopens");
    let (cancel, authz) = open_scope();
    let again = reopened
        .neighbors(&anchor.src, &EdgeFilter::any(), &budgets(), &cancel, &authz)
        .expect("neighbors reopen");
    let attributed: Vec<&str> = again
        .edges
        .iter()
        .filter_map(|edge| edge.assertion_id.as_deref())
        .filter(|aid| aid.starts_with("tracer-"))
        .collect();
    assert!(
        attributed.contains(&"tracer-m1") && attributed.contains(&"tracer-m2"),
        "both attributions traverse: {attributed:?}"
    );
    assert!(
        !attributed.contains(&"tracer-tomb"),
        "the tombstoned rejection stays hidden: {attributed:?}"
    );
}

fn parse_ayah(node: &quran_graph::GraphNode) -> (u16, u32) {
    let rest =
        node.stable_id.strip_prefix("ayah:").expect("ayah-kind nodes carry ayah: stable IDs");
    let mut parts = rest.split(':');
    let surah: u16 = parts.next().expect("surah part").parse().expect("surah parses");
    let ayah: u32 = parts.next().expect("ayah part").parse().expect("ayah parses");
    assert!(parts.next().is_none(), "ayah stable IDs are triples");
    (surah, ayah)
}
