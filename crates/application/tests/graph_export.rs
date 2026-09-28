//! Phase 4 export: Graph JSON v1 with policy filter plus static DOT/SVG
//! rendering (plan 04-03, D-02/D-13).
//!
//! Against real SQLite in a tempdir with an imported + activated edition
//! and a structural projection: assertions travel with kept edges,
//! restricted and tombstoned IDs are absent from the serialized bytes,
//! truncated exports carry the incomplete notice with a reason, and DOT/SVG
//! outputs contain the exported node and edge identities with a visible
//! truncation banner when partial.

mod common;

use application::quran_graph_build::{collect_structural_input, publish_structural_build};
use application::quran_graph_export::{
    assemble_export, export_document, hop_layers, load_projection_sets, render_dot, render_svg,
};
use quran_graph::{
    AuthzScope, ExportNotice, GRAPH_JSON_FORMAT, GraphError, STRUCTURAL_PROJECTION_ID,
};

async fn active_db() -> (tempfile::TempDir, std::sync::Arc<storage_sqlite::SqliteDatabase>, String)
{
    let (dir, db, _reader, path_str) = common::active_reader().await;
    let collected = collect_structural_input(&db)
        .await
        .expect("collect from active edition")
        .expect("the harness activates test-edition-min, so collection succeeds");
    publish_structural_build(&path_str, &collected).await.expect("structural build publishes");
    (dir, db, path_str)
}

async fn row_and_edition(db: &storage_sqlite::SqliteDatabase) -> (String, String) {
    let store =
        application::quran_graph_store::SqliteGraphStore::open_active(db, STRUCTURAL_PROJECTION_ID)
            .await
            .expect("active projection opens");
    (store.manifest().id.clone(), store.manifest().edition_id.clone())
}

/// Seed one assertion plus its PARALLELS edge on ayah:1:1 -> ayah:1:2.
async fn seed_asserted_edge(
    path_str: &str,
    row_id: &str,
    edition_id: &str,
    suffix: &str,
    decision: &str,
) -> String {
    let aid = format!("export-{suffix}");
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
    sqlx::query(
        "INSERT INTO graph_assertions
           (id, projection_row_id, projection_id, edition_id, dataset_scope,
            assertion_kind, claim_json, evidence_json, source_location,
            reviewer, decision, decided_at,
            provenance_layer, created_at)
         VALUES (?, ?, ?, ?, 'export-seed',
                 'annotation', '{\"relation\": \"PARALLELS\"}',
                 '{\"source\": \"export-seed\"}', 'export-seed:row-1',
                 'export-scholar', ?, '2026-09-28T00:00:00Z',
                 'B', '2026-09-28T00:00:00Z')",
    )
    .bind(&aid)
    .bind(row_id)
    .bind(STRUCTURAL_PROJECTION_ID)
    .bind(edition_id)
    .bind(decision)
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
    .bind(format!("export-edge-{suffix}"))
    .bind(row_id)
    .bind(&aid)
    .execute(&pool)
    .await
    .expect("asserted edge inserts");
    pool.close().await;
    aid
}

#[tokio::test]
async fn assertions_travel_with_kept_edges() {
    let (_dir, db, path_str) = active_db().await;
    let (row_id, edition_id) = row_and_edition(&db).await;
    let kept = seed_asserted_edge(&path_str, &row_id, &edition_id, "keep", "accepted").await;
    drop(db);

    let pool_db =
        storage_sqlite::SqliteDatabase::new(&path_str, 4, true).await.expect("db reopens");
    let sets = load_projection_sets(&pool_db, &row_id).await.expect("sets load");
    assert_eq!(sets.manifest.projection_id, STRUCTURAL_PROJECTION_ID);
    assert!(!sets.nodes.is_empty() && !sets.edges.is_empty());

    let doc = assemble_export(&sets, &AuthzScope::all_visible(), &ExportNotice::complete());
    assert_eq!(doc["format"], GRAPH_JSON_FORMAT);
    // The asserted edge ships with its assertion record: reviewer,
    // decision, timestamp, and evidence travel together.
    let edges = doc["edges"].as_array().cloned().unwrap_or_default();
    assert!(
        edges.iter().any(|edge| edge["assertion_id"] == kept.as_str()),
        "kept edge ships: {edges:?}"
    );
    let assertions = doc["assertions"].as_array().cloned().unwrap_or_default();
    let record = assertions
        .iter()
        .find(|record| record["id"] == kept.as_str())
        .expect("assertion travels with its edge");
    assert_eq!(record["reviewer"], "export-scholar");
    assert_eq!(record["decision"], "accepted");
    assert_eq!(record["decided_at"], "2026-09-28T00:00:00Z");
    assert!(record["evidence"].is_object(), "evidence travels with the claim");
    assert_eq!(doc["counts"]["edges"], edges.len());
    assert_eq!(doc["counts"]["assertions"], assertions.len());
    assert_eq!(doc["truncation"]["truncated"], false);
}

#[tokio::test]
async fn restricted_and_tombstoned_ids_are_absent_from_bytes() {
    let (_dir, db, path_str) = active_db().await;
    let (row_id, edition_id) = row_and_edition(&db).await;
    let kept = seed_asserted_edge(&path_str, &row_id, &edition_id, "keep", "accepted").await;
    let dropped = seed_asserted_edge(&path_str, &row_id, &edition_id, "drop", "accepted").await;
    let tomb = seed_asserted_edge(&path_str, &row_id, &edition_id, "tomb", "rejected").await;
    drop(db);

    let pool_db =
        storage_sqlite::SqliteDatabase::new(&path_str, 4, true).await.expect("db reopens");
    let sets = load_projection_sets(&pool_db, &row_id).await.expect("sets load");

    // Tombstoned assertions hide even under the unrestricted scope.
    let open = assemble_export(&sets, &AuthzScope::all_visible(), &ExportNotice::complete());
    let serialized = serde_json::to_string(&open).expect("export serializes");
    assert!(!serialized.contains(&tomb), "tombstoned IDs never reach export bytes");
    assert!(serialized.contains(&kept), "effective assertions travel");
    assert!(serialized.contains(&dropped), "unrestricted scope keeps effective edges");

    // A restricted scope keeps structural edges plus the listed assertion
    // only: the dropped ID vanishes from edges, assertions, and every
    // stray string field.
    let scope = AuthzScope::restricted([kept.clone()]);
    let doc = assemble_export(&sets, &scope, &ExportNotice::complete());
    let serialized = serde_json::to_string(&doc).expect("export serializes");
    assert!(!serialized.contains(&dropped), "restricted IDs never reach export bytes");
    assert!(!serialized.contains(&tomb), "tombstoned IDs stay hidden under restriction");
    assert!(serialized.contains(&kept), "listed assertions travel");
    // Structural edges (no assertion) always survive the filter.
    assert!(
        doc["edges"]
            .as_array()
            .is_some_and(|edges| { edges.iter().any(|edge| edge["assertion_id"].is_null()) }),
        "structural edges pass the filter: {doc}"
    );
}

#[tokio::test]
async fn truncated_exports_carry_the_incomplete_notice() {
    let (_dir, db, _path) = active_db().await;
    let (row_id, _edition) = row_and_edition(&db).await;
    let sets = load_projection_sets(&db, &row_id).await.expect("sets load");

    let doc = assemble_export(
        &sets,
        &AuthzScope::all_visible(),
        &ExportNotice::incomplete("node budget 7 exhausted"),
    );
    assert_eq!(doc["format"], GRAPH_JSON_FORMAT);
    assert_eq!(doc["truncation"]["truncated"], true);
    assert_eq!(doc["truncation"]["incomplete_reason"], "node budget 7 exhausted");
    assert!(doc["manifest"]["projection_id"] == STRUCTURAL_PROJECTION_ID);

    // Unknown rows are typed errors, never empty documents.
    let err = load_projection_sets(&db, "no-such-row").await.expect_err("unknown row errors");
    assert!(
        matches!(err, GraphError::UnknownProjection { .. }),
        "unknown rows are UnknownProjection: {err}"
    );
}

#[tokio::test]
async fn export_document_loads_and_assembles_in_one_step() {
    let (_dir, db, path_str) = active_db().await;
    let (row_id, edition_id) = row_and_edition(&db).await;
    let kept = seed_asserted_edge(&path_str, &row_id, &edition_id, "keep", "accepted").await;
    let doc = export_document(&db, &row_id, &AuthzScope::all_visible(), &ExportNotice::complete())
        .await
        .expect("document assembles");
    let serialized = serde_json::to_string(&doc).expect("export serializes");
    assert!(serialized.contains(&kept));
    assert_eq!(doc["format"], GRAPH_JSON_FORMAT);
}

#[tokio::test]
async fn dot_and_svg_render_identities_with_banner_when_partial() {
    use quran_graph::{GraphEdge, GraphNode, NodeKind};
    let nodes = vec![
        GraphNode::new("ayah:1:1", NodeKind::Ayah, serde_json::Value::Null),
        GraphNode::new("ayah:1:2", NodeKind::Ayah, serde_json::Value::Null),
        GraphNode::new("surah:1", NodeKind::Surah, serde_json::Value::Null),
    ];
    let edges = vec![
        GraphEdge::structural("surah:1", "CONTAINS", "ayah:1:1", serde_json::Value::Null),
        GraphEdge::asserted("ayah:1:1", "PARALLELS", "ayah:1:2", "export-keep"),
    ];

    // Hop layering is deterministic: the seed opens layer zero, arrivals
    // sort by stable ID within each layer.
    let layers = hop_layers(&nodes, &edges, "ayah:1:1");
    assert_eq!(layers[0], vec!["ayah:1:1".to_string()]);
    assert_eq!(layers[1], vec!["ayah:1:2".to_string(), "surah:1".to_string()]);

    // Complete renders carry identities with no banner.
    let dot = render_dot(&nodes, &edges, &ExportNotice::complete());
    assert!(dot.contains("digraph"), "{dot}");
    assert!(dot.contains("ayah:1:1"), "{dot}");
    assert!(dot.contains("PARALLELS"), "{dot}");
    assert!(!dot.contains("truncated"), "complete renders carry no banner: {dot}");
    let svg = render_svg(&nodes, &edges, "ayah:1:1", &ExportNotice::complete());
    assert!(svg.contains("<svg"), "{svg}");
    assert!(svg.contains("ayah:1:2"), "{svg}");
    assert!(svg.contains("PARALLELS"), "{svg}");
    assert!(!svg.contains("truncated"), "complete renders carry no banner");

    // Partial renders banner the reason verbatim in both formats.
    let notice = ExportNotice::incomplete("edge budget 3 exhausted");
    let dot = render_dot(&nodes, &edges, &notice);
    assert!(dot.contains("ayah:1:1"), "identities survive truncation: {dot}");
    assert!(dot.contains("truncated: edge budget 3 exhausted"), "{dot}");
    let svg = render_svg(&nodes, &edges, "ayah:1:1", &notice);
    assert!(svg.contains("ayah:1:1"), "identities survive truncation");
    assert!(svg.contains("truncated: edge budget 3 exhausted"), "{svg}");
}
