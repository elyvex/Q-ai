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

use application::quran_graph_build::{collect_structural_input, publish_structural_build};
use application::quran_graph_store::SqliteGraphStore;
use application::quran_reader::{QuranReader, QuranReaderService};
use application::quran_tools::verify_canonical_quotation;
use quran_graph::{
    AuthzScope, EdgeFilter, GraphError, GraphStore, NodeKind, QueryBudgets,
    STRUCTURAL_PROJECTION_ID,
};

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
