//! Phase 4 projection builders: word-root (dataset-gated) plus annotated
//! (seed-driven) with rebuild preservation (plan 04-02, D-02/D-03/D-04/D-08).
//!
//! Against real SQLite in a tempdir: the word-root projection builds only
//! from the active attributed morphology dataset (typed
//! `UnavailableDataset` otherwise, never heuristics); the annotated
//! projection consumes `concept-seed-v1.json` into concept/entity nodes plus
//! `MENTIONS_CONCEPT`/`REFERS_TO` edges and refs-only `TRANSLATES` coverage
//! edges; and a delete-plus-rebuild cycle preserves every authority and
//! review-history row (AC-P4-03).
//!
//! OD-11 (morphology dataset/license) and OD-12 (linguist) stay explicit
//! BLOCKED owner gates: word-root production acceptance waits owner
//! ratification P4-X01 through P4-X05, and the seed stays mechanics-only.

mod common;

use std::sync::atomic::AtomicBool;

use application::quran_graph_annotations::{ProposeInput, accept, get_assertion, propose};
use application::quran_graph_build::{
    ANNOTATED_PROJECTION_ID, WORDROOT_CAPABILITY, WORDROOT_PROJECTION_ID,
    clear_projection_adjacency, collect_structural_input, collect_wordroot_input,
    publish_annotated_build, publish_structural_build, publish_wordroot_build,
};
use application::quran_graph_store::SqliteGraphStore;
use quran_graph::{AuthzScope, EdgeFilter, GraphStore, QueryBudgets};
use storage::Database as _;

const REVIEWER: &str = "pending-scholar";
const OPERATOR: &str = common::PRINCIPAL;

const CONCEPT_SEED: &str = include_str!("../../../fixtures/quran/graph/concept-seed-v1.json");

async fn active_harness()
-> (tempfile::TempDir, std::sync::Arc<storage_sqlite::SqliteDatabase>, String) {
    let (dir, db, _reader, path_str) = common::active_reader().await;
    (dir, db, path_str)
}

async fn active_edition_id(db: &storage_sqlite::SqliteDatabase) -> String {
    let mut uow = db.write().await.expect("uow opens");
    let active = uow.quran().get_active().await.expect("active reads").expect("edition active");
    let id = active.edition_id.clone();
    uow.rollback().await.expect("read rolls back");
    id
}

fn lexicon_provenance() -> storage::quran::LexiconProvenance {
    storage::quran::LexiconProvenance {
        layer: "B".to_string(),
        algorithm: None,
        algorithm_version: None,
        confidence: None,
        reviewer: None,
        status: "imported".to_string(),
    }
}

/// Seed one synthetic morphology dataset (`test-lexicon`, active) with two
/// roots, two lemmas, and competing analyses on one token (never merged).
async fn seed_synthetic_dataset(db: &storage_sqlite::SqliteDatabase, edition_id: &str) -> String {
    let dataset_id = "test-lexicon@0.1.0".to_string();
    let mut uow = db.write().await.expect("uow opens");
    uow.quran()
        .upsert_dataset(storage::quran::QuranDatasetRow {
            id: dataset_id.clone(),
            slug: "test-lexicon".to_string(),
            version: "0.1.0".to_string(),
            title: "Synthetic test lexicon (non-canonical)".to_string(),
            license_status: "Unspecified".to_string(),
            license_json: "{}".to_string(),
            attribution: "synthetic test lexicon by phase-4-tracer".to_string(),
            root_convention: "test".to_string(),
            tagset_version: "test-v1".to_string(),
            state: "staged".to_string(),
            created_at: "2026-09-28T00:00:00Z".to_string(),
        })
        .await
        .expect("dataset upserts");
    for (id, root) in [("root-ktb", "ktb"), ("root-slm", "slm")] {
        uow.quran()
            .insert_roots(vec![storage::quran::LexiconRootRow {
                id: id.to_string(),
                dataset_id: dataset_id.clone(),
                root: root.to_string(),
                root_normalized: root.to_string(),
                provenance: lexicon_provenance(),
                corpus_generation: 1,
                created_at: "2026-09-28T00:00:00Z".to_string(),
            }])
            .await
            .expect("root inserts");
    }
    for (id, lemma, root_id) in
        [("lemma-kataba", "kataba", Some("root-ktb")), ("lemma-salam", "salam", Some("root-slm"))]
    {
        uow.quran()
            .insert_lemmas(vec![storage::quran::LexiconLemmaRow {
                id: id.to_string(),
                dataset_id: dataset_id.clone(),
                lemma: lemma.to_string(),
                root_id: root_id.map(str::to_string),
                pos_unified: "verb".to_string(),
                pos_native: "V".to_string(),
                provenance: lexicon_provenance(),
                corpus_generation: 1,
                created_at: "2026-09-28T00:00:00Z".to_string(),
            }])
            .await
            .expect("lemma inserts");
    }
    // Token 1:1:1 carries TWO competing analyses (ktb vs slm); token 1:1:2
    // carries one. Competing analyses coexist: no winner column exists.
    let analyses = [
        ("analysis-1", 1, 1, 1, 0, Some("lemma-kataba"), Some("root-ktb")),
        ("analysis-2", 1, 1, 1, 1, Some("lemma-salam"), Some("root-slm")),
        ("analysis-3", 1, 1, 2, 0, Some("lemma-kataba"), Some("root-ktb")),
    ];
    for (id, surah, ayah, position, index, lemma_id, root_id) in analyses {
        uow.quran()
            .insert_analyses(vec![storage::quran::TokenAnalysisRow {
                id: id.to_string(),
                dataset_id: dataset_id.clone(),
                edition_id: edition_id.to_string(),
                surah,
                ayah,
                token_position: position,
                analysis_index: index,
                surface: format!("surface-{position}"),
                lemma_id: lemma_id.map(str::to_string),
                root_id: root_id.map(str::to_string),
                stem: String::new(),
                pos_unified: "verb".to_string(),
                pos_native: "V".to_string(),
                features_json: "{}".to_string(),
                segments_json: "[]".to_string(),
                provenance: lexicon_provenance(),
                corpus_generation: 1,
                created_at: "2026-09-28T00:00:00Z".to_string(),
            }])
            .await
            .expect("analysis inserts");
    }
    uow.quran().set_dataset_state(&dataset_id, "active").await.expect("dataset activates");
    uow.commit().await.expect("seed commits");
    dataset_id
}

#[tokio::test]
async fn wordroot_build_from_synthetic_dataset() {
    let (_dir, db, path_str) = active_harness().await;
    let edition_id = active_edition_id(&db).await;
    let dataset_id = seed_synthetic_dataset(&db, &edition_id).await;

    let input = collect_wordroot_input(&db).await.expect("active dataset collects");
    assert_eq!(input.dataset_id, dataset_id);
    assert!(input.synthetic_test_only, "synthetic datasets stay labeled");
    assert_eq!(input.roots.len(), 2);
    assert_eq!(input.analyses.len(), 3);

    let report = publish_wordroot_build(&path_str, &input).await.expect("word-root publishes");
    assert_eq!(report.manifest.projection_id, WORDROOT_PROJECTION_ID);
    assert!(!report.attribution_assertion_id.is_empty());
    assert!(report.node_count >= 6, "roots, lemmas, and tokens stage: {}", report.node_count);
    assert!(report.edge_count > 0);

    // Every HAS_ROOT/HAS_LEMMA/SAME_ROOT_AS/SAME_LEMMA_AS edge references
    // the dataset attribution assertion...
    let store = SqliteGraphStore::open_active(&db, WORDROOT_PROJECTION_ID)
        .await
        .expect("word-root projection opens");
    let cancel = AtomicBool::new(false);
    let authz = AuthzScope::all_visible();
    let view = store
        .subgraph(
            &["token:1:1:1".to_string(), "token:1:1:2".to_string()],
            &QueryBudgets::default(),
            &cancel,
            &authz,
        )
        .expect("subgraph opens");
    assert!(!view.edges.is_empty());
    for edge in &view.edges {
        assert_eq!(
            edge.assertion_id.as_deref(),
            Some(report.attribution_assertion_id.as_str()),
            "word-root edges stay dataset-attributed: {edge:?}"
        );
        assert!(
            ["HAS_ROOT", "HAS_LEMMA", "SAME_ROOT_AS", "SAME_LEMMA_AS"]
                .contains(&edge.edge.as_str()),
            "word-root vocabulary only: {}",
            edge.edge
        );
    }

    // ...and competing analyses are never merged into a winner: token 1:1:1
    // links BOTH roots.
    let roots: Vec<&str> = view
        .edges
        .iter()
        .filter(|edge| edge.src == "token:1:1:1" && edge.edge == "HAS_ROOT")
        .map(|edge| edge.dst.as_str())
        .collect();
    assert!(
        roots.contains(&"root:ktb") && roots.contains(&"root:slm"),
        "both readings link: {roots:?}"
    );

    // The attribution assertion is pending (traversable, labeled — never a
    // verified claim) with the dataset attribution carried verbatim.
    let attribution = get_assertion(&path_str, &report.attribution_assertion_id)
        .await
        .expect("attribution reads");
    assert_eq!(attribution.decision, quran_graph::AssertionDecision::Pending);
    assert!(
        attribution
            .claim
            .get("synthetic_test_only")
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
    );
    assert_eq!(
        attribution.claim.get("dataset").and_then(|value| value.as_str()),
        Some(dataset_id.as_str())
    );

    // No canonical surfaces leak into the projection bytes.
    let doc = serde_json::to_value(store.inspect()).expect("inspect serializes");
    assert!(!serde_json::to_string(&doc).expect("serializes").contains("surface-1"));
}

#[tokio::test]
async fn od11_dataset_license_blocked_wordroot_requires_active_dataset() {
    // OD-11 BLOCKED (decisions-needed.md: "Which morphology dataset, under
    // what license (ADR-0203)?" — unanswered): with no active dataset the
    // word-root build fails with the typed UnavailableDataset diagnostic
    // naming the capability (QAI-MORPH-0004; CLI exit 5, HTTP 404 conventions
    // reserved), never an empty projection or heuristic roots.
    let (_dir, db, _path) = active_harness().await;
    let err = collect_wordroot_input(&db).await.expect_err("no dataset is a typed error");
    match &err {
        application::quran_morphology::MorphologyToolError::UnavailableDataset { capability } => {
            assert_eq!(capability, WORDROOT_CAPABILITY, "capability named: {capability}");
        }
        other => panic!("expected UnavailableDataset, got {other}"),
    }
    use storage::error::Diagnostic as _;
    assert_eq!(err.code().to_string(), "QAI-MORPH-0004");
    assert_eq!(
        err.to_string(),
        format!(
            "no active morphology dataset for {WORDROOT_CAPABILITY}; import and activate one first"
        )
    );
}

#[tokio::test]
async fn od12_linguist_blocked_annotated_seed_stays_mechanics_only() {
    // OD-12 BLOCKED (decisions-needed.md: normalization catalog plus engaged
    // linguist (ADR-0204) — unanswered): the annotated build stages the
    // synthetic seed as mechanics only — every node synthetic-labeled with
    // pending-scholar attribution, every edge referencing the pending
    // seed-attribution assertion, zero translation edges without a
    // registered translation edition.
    let (_dir, _db, path_str) = active_harness().await;
    let report =
        publish_annotated_build(&path_str, CONCEPT_SEED).await.expect("annotated publishes");
    assert_eq!(report.manifest.projection_id, ANNOTATED_PROJECTION_ID);
    assert!(report.node_count >= 14 + 7, "ayahs plus seed nodes stage: {}", report.node_count);
    assert_eq!(
        report.edge_count, 5,
        "seed links stage with no translations: {}",
        report.edge_count
    );
    assert_eq!(report.rederived, 0);

    let db = storage_sqlite::SqliteDatabase::new(&path_str, 4, true).await.expect("db opens");
    let store = SqliteGraphStore::open_active(&db, ANNOTATED_PROJECTION_ID)
        .await
        .expect("annotated projection opens");
    let cancel = AtomicBool::new(false);
    let authz = AuthzScope::all_visible();
    let view = store
        .neighbors("concept:mercy", &EdgeFilter::any(), &QueryBudgets::default(), &cancel, &authz)
        .expect("concept neighbors open");
    assert!(!view.edges.is_empty());
    for edge in &view.edges {
        assert_eq!(
            edge.assertion_id.as_deref(),
            Some(report.attribution_assertion_id.as_str()),
            "seed edges stay seed-attributed: {edge:?}"
        );
    }
    for node in &view.nodes {
        if node.kind == quran_graph::NodeKind::Concept || node.kind == quran_graph::NodeKind::Entity
        {
            assert_eq!(
                node.attrs.get("reviewed_by").and_then(|value| value.as_str()),
                Some("pending-scholar"),
                "no node claims reviewed scholarship: {node:?}"
            );
        }
    }
    let attribution = get_assertion(&path_str, &report.attribution_assertion_id)
        .await
        .expect("attribution reads");
    assert_eq!(attribution.decision, quran_graph::AssertionDecision::Pending);
}

/// Register one translation edition plus a passage carrying unmistakable
/// text, proving translation edges are refs-only (ADR-0112).
async fn seed_translation_with_passage(
    db: &storage_sqlite::SqliteDatabase,
    edition_id: &str,
) -> String {
    let slug = "test-translation-en".to_string();
    let mut uow = db.write().await.expect("uow opens");
    uow.provenance()
        .insert(storage::repository::ProvenanceRecord {
            id: "prov-test-translation".to_string(),
            layer: "publisher_metadata".to_string(),
            subject_urn: "urn:qai:test:translation".to_string(),
            attribution_kind: "dataset".to_string(),
            attribution_json: "{}".to_string(),
            source_version_id: Some(common::SOURCE_VERSION_ID.to_string()),
            trust_level: "ImportedUnverified".to_string(),
            verification_status: "unverified".to_string(),
            confidence: None,
            versions_json: "{}".to_string(),
            created_by: common::PRINCIPAL.to_string(),
        })
        .await
        .expect("provenance inserts");
    uow.quran()
        .insert_translation_edition(storage::quran::TranslationEditionRow {
            id: "translation-test-en".to_string(),
            slug: slug.clone(),
            version: "0.1.0".to_string(),
            name: "Synthetic test translation".to_string(),
            translator: "phase-4-tracer".to_string(),
            language: "en".to_string(),
            aligned_edition_id: edition_id.to_string(),
            numbering_scheme: "hafs".to_string(),
            license_json: "{}".to_string(),
            trust_level: "ImportedUnverified".to_string(),
            source_version_id: common::SOURCE_VERSION_ID.to_string(),
            text_hash: "sha256:00".to_string(),
            status: "staged".to_string(),
            imported_at: "2026-09-28T00:00:00Z".to_string(),
        })
        .await
        .expect("translation edition inserts");
    uow.quran()
        .insert_translation_passage(storage::quran::TranslationPassageRow {
            translation_edition_id: "translation-test-en".to_string(),
            surah: 1,
            ayah: 1,
            text: "UNIQUE-PASSAGE-TEXT-NEVER-IN-GRAPH-7f3a".to_string(),
            footnotes_json: "[]".to_string(),
            provenance_id: "prov-test-translation".to_string(),
        })
        .await
        .expect("passage inserts");
    uow.commit().await.expect("translation seed commits");
    slug
}

#[tokio::test]
async fn annotated_build_emits_refs_only_translation_edges() {
    let (_dir, db, path_str) = active_harness().await;
    let edition_id = active_edition_id(&db).await;
    let slug = seed_translation_with_passage(&db, &edition_id).await;

    let report =
        publish_annotated_build(&path_str, CONCEPT_SEED).await.expect("annotated publishes");
    let translates = report.edge_count - 5 - report.rederived;
    assert!(translates >= 14, "TRANSLATES coverage edges stage: {translates}");

    let db = storage_sqlite::SqliteDatabase::new(&path_str, 4, true).await.expect("db opens");
    let store = SqliteGraphStore::open_active(&db, ANNOTATED_PROJECTION_ID)
        .await
        .expect("annotated projection opens");
    let cancel = AtomicBool::new(false);
    let authz = AuthzScope::all_visible();
    let view = store
        .neighbors("ayah:1:1", &EdgeFilter::any(), &QueryBudgets::default(), &cancel, &authz)
        .expect("ayah neighbors open");
    assert!(
        view.edges
            .iter()
            .any(|edge| edge.edge == "TRANSLATES"
                && edge.dst == format!("translation-edition:{slug}")),
        "translation edges point at existing translation edition refs"
    );

    // Refs-only (ADR-0112): passage text never reaches projection bytes.
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&path_str))
        .await
        .expect("pool opens");
    let blobs: Vec<String> = sqlx::query_scalar(
        "SELECT attrs_json FROM graph_nodes UNION ALL SELECT attrs_json FROM graph_edges",
    )
    .fetch_all(&pool)
    .await
    .expect("payloads read");
    pool.close().await;
    let joined = blobs.join("\n");
    assert!(
        !joined.contains("UNIQUE-PASSAGE-TEXT-NEVER-IN-GRAPH-7f3a"),
        "no translation text in projection records"
    );
}

fn review_propose(
    id: &str,
    projection: &str,
    edition_id: &str,
    src: &str,
    edge: &str,
    dst: &str,
) -> ProposeInput {
    ProposeInput {
        id: Some(id.to_string()),
        kind: quran_graph::AssertionKind::Annotation,
        src: src.to_string(),
        edge: edge.to_string(),
        dst: dst.to_string(),
        evidence: serde_json::json!({"source": "graph-build-test"}),
        source_id: "graph-build-test".to_string(),
        source_location: "graph_build.rs".to_string(),
        author: REVIEWER.to_string(),
        invoked_by: OPERATOR.to_string(),
        projection_id: projection.to_string(),
        edition_id: edition_id.to_string(),
        dataset_scope: String::new(),
    }
}

async fn authority_snapshot(path_str: &str) -> Vec<(String, String)> {
    // Every authority row with its decision: the AC-P4-03 survival set.
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(path_str))
        .await
        .expect("pool opens");
    let mut rows: Vec<(String, String)> =
        sqlx::query_as("SELECT id, decision FROM graph_assertions ORDER BY id")
            .fetch_all(&pool)
            .await
            .expect("authority reads");
    rows.sort();
    pool.close().await;
    rows
}

#[tokio::test]
async fn rebuild_preserves_authority_and_review_history_ac_p4_03() {
    let (_dir, db, path_str) = active_harness().await;
    // Authority scopes by the canonical edition id (the same identity the
    // builders pin), so re-derivation finds the rows after rebuild.
    let edition_id = active_edition_id(&db).await;

    // Populate: annotated build (seed attribution authority) plus a
    // human-reviewed annotation in the same scope.
    let first =
        publish_annotated_build(&path_str, CONCEPT_SEED).await.expect("annotated publishes");
    propose(
        &path_str,
        review_propose(
            "ac-p4-03-note",
            ANNOTATED_PROJECTION_ID,
            &edition_id,
            "ayah:1:1",
            "PARALLELS",
            "ayah:1:2",
        ),
    )
    .await
    .expect("annotation proposes");
    accept(
        &path_str,
        application::quran_graph_annotations::DecideInput {
            id: "ac-p4-03-note".to_string(),
            reviewer: REVIEWER.to_string(),
            decided_at: None,
            invoked_by: OPERATOR.to_string(),
        },
    )
    .await
    .expect("annotation accepts");
    let before = authority_snapshot(&path_str).await;
    assert!(before.iter().any(|(id, _)| id == "ac-p4-03-note"));
    assert!(before.iter().any(|(id, _)| id == &first.attribution_assertion_id));

    // Delete adjacency plus republish: the wipe touches only nodes/edges.
    let (nodes_wiped, edges_wiped) =
        clear_projection_adjacency(&path_str, &first.build_row_id).await.expect("adjacency wipes");
    assert!(nodes_wiped > 0 && edges_wiped > 0);
    let wiped = authority_snapshot(&path_str).await;
    assert_eq!(before, wiped, "adjacency wipe never discards authority or review history");

    let second =
        publish_annotated_build(&path_str, CONCEPT_SEED).await.expect("annotated republishes");
    assert_ne!(second.build_row_id, first.build_row_id, "republish mints a fresh build row");
    let after = authority_snapshot(&path_str).await;
    for (id, decision) in &before {
        assert!(
            after.contains(&(id.clone(), decision.clone())),
            "authority row {id} ({decision}) survives delete plus rebuild: {after:?}"
        );
    }

    // The accepted annotation re-authorizes traversal in the new build
    // (re-derived from authority, history intact).
    let kept = get_assertion(&path_str, "ac-p4-03-note").await.expect("annotation retained");
    assert_eq!(kept.decision, quran_graph::AssertionDecision::Accepted);
    let db = storage_sqlite::SqliteDatabase::new(&path_str, 4, true).await.expect("db opens");
    let store = SqliteGraphStore::open_active(&db, ANNOTATED_PROJECTION_ID)
        .await
        .expect("rebuilt projection opens");
    let cancel = AtomicBool::new(false);
    let authz = AuthzScope::all_visible();
    let view = store
        .neighbors("ayah:1:1", &EdgeFilter::any(), &QueryBudgets::default(), &cancel, &authz)
        .expect("neighbors open");
    assert!(
        view.edges.iter().any(|edge| edge.assertion_id.as_deref() == Some("ac-p4-03-note")),
        "accepted annotations re-derive into rebuilt adjacency"
    );
    assert!(second.rederived >= 1, "re-derivation counted: {}", second.rederived);
}

#[tokio::test]
async fn structural_build_unchanged_beside_new_builders() {
    // The tracer publish framework is reused unchanged: a structural build
    // still publishes exactly its built nodes and edges.
    let (_dir, db, path_str) = active_harness().await;
    let collected =
        collect_structural_input(&db).await.expect("collects").expect("active edition present");
    let report = publish_structural_build(&path_str, &collected).await.expect("publishes");
    assert_eq!(report.node_count, report.built.nodes.len());
    assert_eq!(report.edge_count, report.built.edges.len());
}
