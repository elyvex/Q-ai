//! Phase 4 read services: all path modes, subgraph, typed patterns, and
//! root-family over SQLite with the full explainability payload (plan 04-03,
//! D-09/D-10/D-14).
//!
//! Against real SQLite in a tempdir with an imported + activated edition and
//! a structural projection: reachability, min-hop, and up-to-K paths each
//! carry per-edge provenance; unknown pattern edges are typed rejections;
//! unknown subgraph seeds are complete-empty; root-family results carry
//! dataset attribution (typed unavailability without a dataset); truncated
//! searches refuse no-path claims; and K beyond max_paths is a pre-flight
//! error (T-04-07).

mod common;

use application::quran_graph_api::{
    Explanation, GraphApiError, GraphApiService, GraphBackend, NeighborsArgs, PathArgs, PathsArgs,
    PatternArgs, ProvenanceExplanation, ReadOptions, RootFamilyArgs, SubgraphArgs,
};
use application::quran_graph_build::{collect_structural_input, publish_structural_build};
use application::quran_morphology::MorphologyToolError;
use quran_graph::{
    AuthzScope, Direction, EdgeFilter, GraphError, Pattern, PatternStep, QueryBudgets,
    STRUCTURAL_PROJECTION_ID,
};

fn options() -> ReadOptions {
    ReadOptions::default()
}

async fn active_service() -> (tempfile::TempDir, GraphApiService, String) {
    let (dir, db, _reader, path_str) = common::active_reader().await;
    let collected = collect_structural_input(&db)
        .await
        .expect("collect from active edition")
        .expect("the harness activates test-edition-min, so collection succeeds");
    publish_structural_build(&path_str, &collected).await.expect("structural build publishes");
    drop(db);
    let service = GraphApiService::open(&path_str).await.expect("read service opens");
    assert_eq!(service.manifest().projection_id, STRUCTURAL_PROJECTION_ID);
    assert_eq!(service.manifest().corpus_generation, 1);
    (dir, service, path_str)
}

/// Seed two accepted attributions plus one tombstoned rejection as parallel
/// `PARALLELS` edges on one ayah pair, plus one isolated node with no edges.
async fn seed_parallel_assertions(path_str: &str, row_id: &str, edition_id: &str) {
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
    for (suffix, decision) in [("par-1", "accepted"), ("par-2", "accepted"), ("tomb", "rejected")] {
        let aid = format!("paths-{suffix}");
        sqlx::query(
            "INSERT INTO graph_assertions
               (id, projection_row_id, projection_id, edition_id, dataset_scope,
                assertion_kind, claim_json, evidence_json, source_location,
                reviewer, decision, decided_at,
                provenance_layer, created_at)
             VALUES (?, ?, ?, ?, 'paths-seed',
                     'annotation', '{}', '{\"source\": \"paths-seed\"}', 'paths-seed:row-1',
                     'tracer-scholar', ?, '2026-09-28T00:00:00Z',
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
        .bind(format!("paths-edge-{suffix}"))
        .bind(row_id)
        .bind(&aid)
        .execute(&pool)
        .await
        .expect("parallel attribution inserts");
    }
    sqlx::query(
        "INSERT INTO graph_nodes
           (id, projection_row_id, node_kind, stable_id, attrs_json, created_at)
         VALUES ('orphan-node-1', ?, 'annotation', 'annotation:orphan', '{}',
                 '2026-09-28T00:00:00Z')",
    )
    .bind(row_id)
    .execute(&pool)
    .await
    .expect("isolated node inserts");
    pool.close().await;
}

/// Every result carries the snapshot identity, the unrestricted scope
/// descriptor, and a truncation flag consistent with its reason.
fn assert_explained(explanation: &Explanation) {
    assert_eq!(explanation.snapshot.projection_id, STRUCTURAL_PROJECTION_ID);
    assert_eq!(explanation.snapshot.builder_version, "structural-v1");
    assert_eq!(explanation.snapshot.corpus_generation, 1);
    assert_eq!(explanation.applied_filters.authz, "unrestricted");
    assert_eq!(
        explanation.truncated,
        explanation.incomplete_reason.is_some(),
        "truncation always carries a reason"
    );
}

#[tokio::test]
async fn reachability_check_reports_hops_with_provenance() {
    let (_dir, service, _path) = active_service().await;

    // Reachable pair: edition containment is one hop.
    let proven = service
        .reachability(PathArgs {
            from: "surah:1".to_string(),
            to: "ayah:1:1".to_string(),
            options: options(),
        })
        .await
        .expect("reachability answers");
    assert_eq!(proven.reachable, Some(true));
    assert_eq!(proven.hops, Some(1));
    assert!(!proven.explanation.truncated);
    assert_eq!(proven.explanation.start.as_deref(), Some("surah:1"));
    assert_eq!(proven.explanation.end.as_deref(), Some("ayah:1:1"));
    assert_explained(&proven.explanation);

    // An unknown destination is proven absence (the BFS simply never
    // arrives); an unknown source is a typed NodeNotFound, never an empty
    // answer.
    let absent = service
        .reachability(PathArgs {
            from: "ayah:1:1".to_string(),
            to: "ayah:99:99".to_string(),
            options: options(),
        })
        .await
        .expect("reachability answers");
    assert_eq!(absent.reachable, Some(false));
    assert_eq!(absent.hops, None);
    assert!(!absent.explanation.truncated);
    let err = service
        .reachability(PathArgs {
            from: "ayah:99:99".to_string(),
            to: "ayah:1:1".to_string(),
            options: options(),
        })
        .await
        .expect_err("unknown source is an error");
    assert!(
        matches!(err, GraphApiError::Graph(GraphError::NodeNotFound { .. })),
        "unknown nodes report NodeNotFound: {err}"
    );
}

#[tokio::test]
async fn shortest_path_is_min_hop_with_per_edge_provenance() {
    let (_dir, service, _path) = active_service().await;

    // ayah:1:1 -> ayah:1:3 is two hops through surah:1 (NEXT never crosses
    // the surah, so no shorter route exists).
    let found = service
        .shortest_path(PathArgs {
            from: "ayah:1:1".to_string(),
            to: "ayah:1:3".to_string(),
            options: options(),
        })
        .await
        .expect("shortest answers");
    let path = found.path.expect("the pair connects");
    assert_eq!(path.node_ids, vec!["ayah:1:1", "surah:1", "ayah:1:3"]);
    assert_eq!(path.edges.len(), 2);
    assert!(!found.explanation.truncated);
    // Every traversed edge carries structural provenance with the corpus
    // input version (never bare edges).
    let explained = found.explanation.paths.first().expect("explained path");
    assert_eq!(explained.node_ids, path.node_ids);
    for edge in &explained.edges {
        match &edge.provenance {
            ProvenanceExplanation::Structural { input_version } => {
                assert!(input_version.is_some(), "structural edges stamp input_version: {edge:?}");
            }
            other => panic!("structural route carries structural provenance: {other:?}"),
        }
    }
    assert_explained(&found.explanation);
}

#[tokio::test]
async fn up_to_k_paths_rank_shortest_first_with_stable_tiebreak() {
    let (_dir, service, _path) = active_service().await;

    // token:1:1:1 -> token:1:1:3 has two 2-hop routes: the NEXT chain
    // through token:1:1:2 and the containment route through ayah:1:1.
    // Equal length sorts by stable node-ID order: ayah:1:1 first.
    let ranked = service
        .paths(PathsArgs {
            from: "token:1:1:1".to_string(),
            to: "token:1:1:3".to_string(),
            k: 2,
            options: options(),
        })
        .await
        .expect("up-to-K answers");
    assert!(!ranked.truncated);
    assert_eq!(ranked.paths.len(), 2, "both 2-hop routes rank: {:?}", ranked.paths);
    assert_eq!(
        ranked.paths[0].node_ids,
        vec!["token:1:1:1", "ayah:1:1", "token:1:1:3"],
        "stable-ID tie-break orders ayah:1:1 before token:1:1:2"
    );
    assert_eq!(ranked.paths[1].node_ids, vec!["token:1:1:1", "token:1:1:2", "token:1:1:3"]);
    // The explained copies mirror the ranked paths edge-for-edge.
    assert_eq!(ranked.explanation.paths.len(), 2);
    for explained in &ranked.explanation.paths {
        assert_eq!(explained.edges.len(), explained.node_ids.len() - 1);
    }
    assert_explained(&ranked.explanation);
}

#[tokio::test]
async fn parallel_asserted_edges_keep_their_own_provenance() {
    let (_dir, service, path_str) = active_service().await;
    let row_id = service.manifest().id.clone();
    let edition_id = service.manifest().edition_id.clone();
    seed_parallel_assertions(&path_str, &row_id, &edition_id).await;
    let service = GraphApiService::open(&path_str).await.expect("service reopens");

    let ranked = service
        .paths(PathsArgs {
            from: "ayah:1:1".to_string(),
            to: "ayah:1:2".to_string(),
            k: 10,
            options: options(),
        })
        .await
        .expect("up-to-K answers");
    assert!(!ranked.truncated);
    // Direct NEXT, two parallel PARALLELS attributions, the 2-hop
    // containment route through surah:1, and longer structural detours.
    let direct: Vec<_> = ranked.paths.iter().filter(|path| path.edges.len() == 1).collect();
    assert_eq!(direct.len(), 3, "parallel edges never collapse: {:?}", ranked.paths);
    assert!(
        ranked.paths.iter().any(|path| path.node_ids == vec!["ayah:1:1", "surah:1", "ayah:1:2"]),
        "the 2-hop containment route ranks too: {:?}",
        ranked.paths
    );
    let mut asserted_ids: Vec<&str> = Vec::new();
    let mut structural_seen = false;
    for explained in &ranked.explanation.paths {
        assert_eq!(explained.node_ids.first().map(String::as_str), Some("ayah:1:1"));
        assert_eq!(explained.node_ids.last().map(String::as_str), Some("ayah:1:2"));
        for edge in &explained.edges {
            match &edge.provenance {
                ProvenanceExplanation::Asserted { id, reviewer, decision, decided_at, .. } => {
                    asserted_ids.push(id);
                    assert_eq!(reviewer.as_deref(), Some("tracer-scholar"));
                    assert_eq!(decision, "accepted");
                    assert!(decided_at.is_some(), "decision timestamp travels: {edge:?}");
                }
                ProvenanceExplanation::Structural { .. } => {
                    structural_seen = true;
                }
                ProvenanceExplanation::UnknownAssertion { id } => {
                    panic!("seeded assertions resolve: {id}");
                }
            }
        }
    }
    asserted_ids.sort_unstable();
    assert_eq!(asserted_ids, vec!["paths-par-1", "paths-par-2"]);
    assert!(structural_seen, "the structural NEXT route still traverses");
}

#[tokio::test]
async fn isolated_nodes_prove_absence_while_unknown_nodes_error() {
    let (_dir, service, path_str) = active_service().await;
    let row_id = service.manifest().id.clone();
    let edition_id = service.manifest().edition_id.clone();
    seed_parallel_assertions(&path_str, &row_id, &edition_id).await;
    let service = GraphApiService::open(&path_str).await.expect("service reopens");

    // The isolated node resolves but connects to nothing: proven absence.
    let absent = service
        .reachability(PathArgs {
            from: "annotation:orphan".to_string(),
            to: "ayah:1:1".to_string(),
            options: options(),
        })
        .await
        .expect("reachability answers");
    assert_eq!(absent.reachable, Some(false));
    assert_eq!(absent.hops, None);
    assert!(!absent.explanation.truncated);

    let absent_path = service
        .shortest_path(PathArgs {
            from: "annotation:orphan".to_string(),
            to: "ayah:1:1".to_string(),
            options: options(),
        })
        .await
        .expect("shortest answers");
    assert!(absent_path.path.is_none());
    assert!(absent_path.no_path_proven(), "complete-empty proves absence");

    let absent_ranked = service
        .paths(PathsArgs {
            from: "annotation:orphan".to_string(),
            to: "ayah:1:1".to_string(),
            k: 3,
            options: options(),
        })
        .await
        .expect("up-to-K answers");
    assert!(absent_ranked.paths.is_empty());
    assert!(!absent_ranked.truncated);
    assert!(absent_ranked.no_path_proven(), "complete-empty only when not truncated");
}

#[tokio::test]
async fn neighbors_and_subgraph_carry_explainability() {
    let (_dir, service, _path) = active_service().await;

    let view = service
        .neighbors(NeighborsArgs { node: "ayah:1:1".to_string(), options: options() })
        .await
        .expect("neighbors open");
    assert!(!view.truncated);
    assert!(!view.nodes.is_empty() && !view.edges.is_empty());
    assert_eq!(view.explanation.start.as_deref(), Some("ayah:1:1"));
    assert_eq!(view.explanation.nodes.len(), view.nodes.len());
    assert_eq!(view.explanation.edges.len(), view.edges.len());
    assert_eq!(view.explanation.applied_filters.direction, "both");
    assert!(view.explanation.applied_filters.edge_types.is_none(), "default filter admits all");
    assert_explained(&view.explanation);

    // Unknown subgraph seeds are skipped: all-unknown is complete-empty,
    // never an error and never a truncation.
    let empty = service
        .subgraph(SubgraphArgs { seeds: vec!["nope:1".to_string()], options: options() })
        .await
        .expect("subgraph answers");
    assert!(empty.nodes.is_empty() && empty.edges.is_empty());
    assert!(!empty.truncated);
    assert!(empty.incomplete_reason.is_none());

    // A known seed plus an unknown one still extracts the bounded subgraph.
    let mixed = service
        .subgraph(SubgraphArgs {
            seeds: vec!["ayah:1:1".to_string(), "nope:1".to_string()],
            options: options(),
        })
        .await
        .expect("subgraph answers");
    assert!(!mixed.truncated);
    assert!(!mixed.nodes.is_empty() && !mixed.edges.is_empty());
    assert_explained(&mixed.explanation);
}

#[tokio::test]
async fn pattern_allowlist_rejection_happens_before_io() {
    let (_dir, service, _path) = active_service().await;

    // Unknown edges are typed rejections, never silent empty results.
    for edge in ["PRECEDES", "HAS ROOT", "NEXT; DROP TABLE x"] {
        let err = service
            .pattern(PatternArgs {
                pattern: Pattern::new(vec![PatternStep::edge(edge)]),
                seeds: vec!["ayah:1:1".to_string()],
                options: options(),
            })
            .await
            .expect_err("unknown edge rejects");
        assert!(
            matches!(err, GraphApiError::Graph(GraphError::PatternRejected { .. })),
            "unknown edge `{edge}` is PatternRejected: {err}"
        );
    }

    // An allowlisted pattern executes: NEXT from ayah:1:1 reaches ayah:1:2.
    let matched = service
        .pattern(PatternArgs {
            pattern: Pattern::new(vec![PatternStep::edge("NEXT")]),
            seeds: vec!["ayah:1:1".to_string()],
            options: options(),
        })
        .await
        .expect("allowlisted pattern executes");
    assert!(!matched.truncated);
    assert!(
        matched.nodes.iter().any(|node| node.stable_id == "ayah:1:2"),
        "NEXT reaches ayah:1:2: {:?}",
        matched.nodes.iter().map(|node| &node.stable_id).collect::<Vec<_>>()
    );
    assert_explained(&matched.explanation);
}

#[tokio::test]
async fn truncated_searches_refuse_no_path_claims() {
    let (_dir, service, _path) = active_service().await;

    // A one-node budget exhausts mid-enumeration: partial, with a non-empty
    // reason, and no proven absence even with zero paths collected.
    let tight = ReadOptions {
        budgets: QueryBudgets { max_nodes: 1, ..QueryBudgets::default() },
        ..ReadOptions::default()
    };
    let partial = service
        .paths(PathsArgs {
            from: "ayah:1:1".to_string(),
            to: "ayah:2:1".to_string(),
            k: 5,
            options: tight,
        })
        .await
        .expect("truncated enumeration still answers");
    assert!(partial.truncated);
    assert!(
        partial.incomplete_reason.as_deref().is_some_and(|reason| !reason.is_empty()),
        "every partial carries a reason"
    );
    assert!(
        !partial.no_path_proven(),
        "a truncated search refuses the no-path claim even with zero paths"
    );

    let unknown = service
        .reachability(PathArgs {
            from: "ayah:1:1".to_string(),
            to: "ayah:2:1".to_string(),
            options: ReadOptions {
                budgets: QueryBudgets { max_nodes: 1, ..QueryBudgets::default() },
                ..ReadOptions::default()
            },
        })
        .await
        .expect("truncated reachability still answers");
    assert!(unknown.explanation.truncated);
    assert_eq!(unknown.reachable, None, "truncated reachability is unknown, never absent");
    assert_eq!(unknown.hops, None);

    // Relaxed-edge counting is exact: a one-edge budget exhausts with the
    // edge-budget reason (precision probe).
    let edge_tight = ReadOptions {
        budgets: QueryBudgets { max_edges: 1, ..QueryBudgets::default() },
        ..ReadOptions::default()
    };
    let edge_partial = service
        .paths(PathsArgs {
            from: "ayah:1:1".to_string(),
            to: "ayah:1:3".to_string(),
            k: 5,
            options: edge_tight,
        })
        .await
        .expect("edge-truncated enumeration still answers");
    assert!(edge_partial.truncated);
    assert!(
        edge_partial.incomplete_reason.as_deref().is_some_and(|reason| reason.contains("edge")),
        "edge exhaustion names the edge budget: {:?}",
        edge_partial.incomplete_reason
    );
}

#[tokio::test]
async fn k_beyond_max_paths_is_a_preflight_error() {
    let (_dir, service, _path) = active_service().await;

    // Default max_paths is 10: requesting 11 is a typed pre-flight error
    // (T-04-07), never a silent cap.
    let err = service
        .paths(PathsArgs {
            from: "ayah:1:1".to_string(),
            to: "ayah:1:2".to_string(),
            k: 11,
            options: options(),
        })
        .await
        .expect_err("K beyond max_paths errors");
    assert!(
        matches!(err, GraphApiError::Graph(GraphError::BudgetExceeded { .. })),
        "K beyond range is BudgetExceeded: {err}"
    );

    // Out-of-range budgets are pre-flight errors on every mode.
    let bad = ReadOptions {
        budgets: QueryBudgets { max_hops: 0, ..QueryBudgets::default() },
        ..ReadOptions::default()
    };
    let err = service
        .neighbors(NeighborsArgs { node: "ayah:1:1".to_string(), options: bad.clone() })
        .await
        .expect_err("neighbors pre-flights");
    assert!(
        matches!(err, GraphApiError::Graph(GraphError::BudgetExceeded { .. })),
        "out-of-range budgets are BudgetExceeded: {err}"
    );
    let err = service
        .subgraph(SubgraphArgs { seeds: vec!["ayah:1:1".to_string()], options: bad })
        .await
        .expect_err("subgraph pre-flights");
    assert!(
        matches!(err, GraphApiError::Graph(GraphError::BudgetExceeded { .. })),
        "out-of-range budgets are BudgetExceeded: {err}"
    );
}

#[tokio::test]
async fn root_family_carries_dataset_attribution_or_typed_unavailability() {
    // Without a dataset the read is the typed unavailable error, never an
    // empty family a caller could read as "no such root".
    let (_dir, service, _path) = active_service().await;
    let err = service
        .root_family(RootFamilyArgs { root: "ktb".to_string(), limit: 25 })
        .await
        .expect_err("no dataset is unavailable");
    assert!(
        matches!(err, GraphApiError::Morphology(MorphologyToolError::UnavailableDataset { .. })),
        "word-root reads gate on the active dataset: {err}"
    );

    // With the synthetic dataset active, ranked ayahs carry its identity.
    let (dir, db, _reader, path_str) = common::active_reader().await;
    let edition_id = active_edition_id(&db).await;
    seed_synthetic_dataset(&db, &edition_id).await;
    let collected = collect_structural_input(&db)
        .await
        .expect("collect from active edition")
        .expect("collection succeeds");
    drop(db);
    publish_structural_build(&path_str, &collected).await.expect("structural build publishes");
    let service = GraphApiService::open(&path_str).await.expect("read service opens");
    let family = service
        .root_family(RootFamilyArgs { root: "ktb".to_string(), limit: 25 })
        .await
        .expect("root family answers");
    assert_eq!(family.dataset, "test-lexicon@0.1.0");
    assert_eq!(family.limit, 25);
    let ayahs: Vec<(i64, i64)> = family.ayahs.iter().map(|hit| (hit.surah, hit.ayah)).collect();
    assert_eq!(ayahs, vec![(1, 1), (1, 2)], "ranked, deduplicated per ayah: {ayahs:?}");
    drop(dir);
}

#[tokio::test]
async fn budgets_and_filters_shape_explainability() {
    let (_dir, service, _path) = active_service().await;

    // A restricted scope plus a one-predicate filter is named in the applied
    // filters without leaking hidden assertion IDs.
    let scoped = ReadOptions {
        filter: EdgeFilter::one("NEXT", Direction::Outgoing),
        authz: AuthzScope::restricted(["ann-hidden".to_string()]),
        ..ReadOptions::default()
    };
    let view = service
        .neighbors(NeighborsArgs { node: "ayah:1:1".to_string(), options: scoped })
        .await
        .expect("scoped neighbors open");
    assert_eq!(view.explanation.applied_filters.edge_types, Some(vec!["NEXT".to_string()]));
    assert_eq!(view.explanation.applied_filters.direction, "outgoing");
    assert_eq!(view.explanation.applied_filters.authz, "restricted(1 assertions)");
    let serialized = serde_json::to_string(&view.explanation).expect("explanation serializes");
    assert!(!serialized.contains("ann-hidden"), "hidden IDs never leak into the payload");
    // The restricted scope hides nothing structural here, so the NEXT edge
    // still traverses.
    assert!(view.edges.iter().any(|edge| edge.edge == "NEXT"));
}

async fn active_edition_id(db: &storage_sqlite::SqliteDatabase) -> String {
    use storage::Database as _;
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

/// Seed the synthetic morphology dataset (`test-lexicon`, active) with the
/// `ktb` root over tokens 1:1:1 and 1:1:2 (mirrors graph_build.rs).
async fn seed_synthetic_dataset(db: &storage_sqlite::SqliteDatabase, edition_id: &str) -> String {
    use storage::Database as _;
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
    uow.quran()
        .insert_roots(vec![storage::quran::LexiconRootRow {
            id: "root-ktb".to_string(),
            dataset_id: dataset_id.clone(),
            root: "ktb".to_string(),
            root_normalized: "ktb".to_string(),
            provenance: lexicon_provenance(),
            corpus_generation: 1,
            created_at: "2026-09-28T00:00:00Z".to_string(),
        }])
        .await
        .expect("root inserts");
    uow.quran()
        .insert_lemmas(vec![
            storage::quran::LexiconLemmaRow {
                id: "lemma-kataba".to_string(),
                dataset_id: dataset_id.clone(),
                lemma: "kataba".to_string(),
                root_id: Some("root-ktb".to_string()),
                pos_unified: "verb".to_string(),
                pos_native: "V".to_string(),
                provenance: lexicon_provenance(),
                corpus_generation: 1,
                created_at: "2026-09-28T00:00:00Z".to_string(),
            },
            storage::quran::LexiconLemmaRow {
                id: "lemma-salam".to_string(),
                dataset_id: dataset_id.clone(),
                lemma: "salam".to_string(),
                root_id: None,
                pos_unified: "verb".to_string(),
                pos_native: "V".to_string(),
                provenance: lexicon_provenance(),
                corpus_generation: 1,
                created_at: "2026-09-28T00:00:00Z".to_string(),
            },
        ])
        .await
        .expect("lemma inserts");
    let analyses = [
        ("analysis-1", 1, 1, 1, 0, Some("lemma-kataba"), Some("root-ktb")),
        ("analysis-3", 1, 2, 1, 0, Some("lemma-kataba"), Some("root-ktb")),
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
