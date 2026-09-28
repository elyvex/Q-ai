//! Staged-batch structural projection builder with fenced publish (Phase 4).
//!
//! [`collect_structural_input`] assembles the pure builder input from the
//! active edition's canonical rows through the `storage` repository seam
//! (read-only: the unit of work is rolled back). [`publish_structural_build`]
//! reserves a `graph_projections` row in `building` state plus a
//! `graph_build_progress` cursor, stages nodes then edges in batches through
//! the port's rejection rules (empty stable IDs, non-vocabulary predicates,
//! dangling endpoints), then verifies counts, dangling-freedom, and
//! canonical-unchanged inside one transaction that also flips the active
//! pointer while superseding the previous active row.
//!
//! Graph tables have no `storage` repository seam, so graph writes go through
//! a dedicated short-lived sqlx pool on the same database file (WAL +
//! busy-timeout, mirroring the test-seeding precedent). Reads of canonical
//! rows stay on the `Database`/`UnitOfWork` seam. A future storage graph
//! repository can replace the pool without touching the port logic.

use quran_graph::{
    AyahInput, BuiltProjection, GraphError, ProjectionManifest, ProjectionStatus, StructuralInput,
    SurahInput, TokenInput, build_structural, is_allowed_edge,
};
use storage::Database as _;
use storage_sqlite::SqliteDatabase;

/// Structural input plus the active-edition identity it was read from.
pub struct CollectedStructural {
    /// Canonical edition id the input covers.
    pub edition_id: String,
    /// Corpus generation read from the active pointer.
    pub corpus_generation: i64,
    /// Pure builder input (refs and numbers only; no canonical text).
    pub input: StructuralInput,
}

/// Publish report for one structural build.
pub struct StructuralBuildReport {
    /// Manifest of the newly activated build row.
    pub manifest: ProjectionManifest,
    /// The activated `graph_projections` build row id.
    pub build_row_id: String,
    /// Staged node count.
    pub node_count: usize,
    /// Staged edge count.
    pub edge_count: usize,
    /// The built projection (for `--out` documents; identical to staged).
    pub built: BuiltProjection,
}

/// Assemble the structural input from the active edition's canonical rows.
///
/// Returns `Ok(None)` when no edition is active (the caller reports
/// not-found); storage failures are build failures at stage `collect`.
pub async fn collect_structural_input(
    db: &SqliteDatabase,
) -> Result<Option<CollectedStructural>, GraphError> {
    fn storage(error: storage::error::StorageError) -> GraphError {
        GraphError::BuildFailed { stage: "collect".to_string(), detail: error.to_string() }
    }
    let mut uow = db.write().await.map_err(storage)?;
    let active = uow.quran().get_active().await.map_err(storage)?;
    let Some(active) = active else {
        let _ = uow.rollback().await;
        return Ok(None);
    };
    let edition_id = active.edition_id.clone();
    let corpus_generation = active.corpus_generation;
    let surahs = uow.quran().list_surahs(&edition_id).await.map_err(storage)?;
    let ayahs = uow.quran().list_ayahs_range(&edition_id, 1, i64::MAX).await.map_err(storage)?;
    let mut structural_ayahs = Vec::new();
    let mut tokens = Vec::new();
    for ayah in &ayahs {
        structural_ayahs.push(AyahInput {
            surah: ayah.surah as u32,
            ayah: ayah.ayah as u32,
            text: String::new(),
        });
        let rows =
            uow.quran().get_tokens(&edition_id, ayah.surah, ayah.ayah).await.map_err(storage)?;
        for token in rows {
            tokens.push(TokenInput {
                surah: ayah.surah as u32,
                ayah: ayah.ayah as u32,
                position: token.position as u32,
                surface: String::new(),
            });
        }
    }
    let _ = uow.rollback().await;

    Ok(Some(CollectedStructural {
        edition_id: edition_id.clone(),
        corpus_generation,
        input: StructuralInput {
            edition_id,
            input_version: format!("corpus-generation-{corpus_generation}"),
            surahs: surahs.iter().map(|s| SurahInput { number: s.number as u32 }).collect(),
            ayahs: structural_ayahs,
            tokens,
            divisions: Vec::new(),
        },
    }))
}

/// Build the structural projection and publish it as the active row.
///
/// Reserve → stage (batches) → verify → pointer flip runs in one database
/// transaction: a crash or a failed check commits nothing, and the previous
/// active row is retained (superseded) for single-step rollback.
pub async fn publish_structural_build(
    db_path: &str,
    collected: &CollectedStructural,
) -> Result<StructuralBuildReport, GraphError> {
    fn storage(stage: &str, error: sqlx::Error) -> GraphError {
        GraphError::BuildFailed { stage: stage.to_string(), detail: error.to_string() }
    }

    let built = build_structural(&collected.input);
    let node_count = built.nodes.len();
    let edge_count = built.edges.len();

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(db_path)
                .create_if_missing(false)
                .foreign_keys(true)
                .busy_timeout(std::time::Duration::from_secs(5)),
        )
        .await
        .map_err(|error| storage("open", error))?;

    let short = uuid::Uuid::new_v4().to_string();
    let build_row_id =
        format!("build-{}-{}-{}", collected.edition_id, collected.corpus_generation, &short[..8]);
    let at = domain::Timestamp::now().to_string();
    let snapshot =
        serde_json::json!({"canonical": format!("generation-{}", collected.corpus_generation)});

    let mut tx = pool.begin().await.map_err(|error| storage("reserve", error))?;

    sqlx::query(
        "INSERT INTO graph_projections
           (id, projection_id, builder_version, edition_id, corpus_generation,
            dataset_versions_json, dependency_snapshot_json, status, manifest_json, created_at)
         VALUES (?, ?, ?, ?, ?, '{}', ?, 'building', '{}', ?)",
    )
    .bind(&build_row_id)
    .bind(quran_graph::STRUCTURAL_PROJECTION_ID)
    .bind(quran_graph::STRUCTURAL_BUILDER_VERSION)
    .bind(&collected.edition_id)
    .bind(collected.corpus_generation)
    .bind(snapshot.to_string())
    .bind(&at)
    .execute(&mut *tx)
    .await
    .map_err(|error| storage("reserve", error))?;

    sqlx::query(
        "INSERT INTO graph_build_progress
           (build_id, projection_row_id, stage, cursor_json, updated_at)
         VALUES (?, ?, 'reserve', '{}', ?)",
    )
    .bind(&build_row_id)
    .bind(&build_row_id)
    .bind(&at)
    .execute(&mut *tx)
    .await
    .map_err(|error| storage("reserve", error))?;

    // Stage nodes in batches, enforcing the port's rejection rule.
    for (batch_no, batch) in built.nodes.chunks(500).enumerate() {
        for (offset, node) in batch.iter().enumerate() {
            if node.stable_id.trim().is_empty() {
                return Err(GraphError::BuildFailed {
                    stage: "stage_nodes".to_string(),
                    detail: "refused node with empty stable ID".to_string(),
                });
            }
            let id = format!("{build_row_id}:n:{}:{}", batch_no * 500 + offset, node.stable_id);
            sqlx::query(
                "INSERT INTO graph_nodes
                   (id, projection_row_id, node_kind, stable_id, attrs_json, created_at)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(&build_row_id)
            .bind(node.kind.to_string())
            .bind(&node.stable_id)
            .bind(node.attrs.to_string())
            .bind(&at)
            .execute(&mut *tx)
            .await
            .map_err(|error| storage("stage_nodes", error))?;
        }
    }

    // Stage edges in batches through the port's rejection rules: allowlisted
    // predicates only, both endpoints already staged (zero dangling edges).
    {
        use std::collections::BTreeSet;
        let staged: BTreeSet<&str> =
            built.nodes.iter().map(|node| node.stable_id.as_str()).collect();
        for (batch_no, batch) in built.edges.chunks(500).enumerate() {
            for (offset, edge) in batch.iter().enumerate() {
                if !is_allowed_edge(&edge.edge) {
                    return Err(GraphError::PatternRejected {
                        detail: format!("cannot stage unknown edge '{}'", edge.edge),
                    });
                }
                if !staged.contains(edge.src.as_str()) {
                    return Err(GraphError::BuildFailed {
                        stage: "stage_edges".to_string(),
                        detail: format!("dangling edge source '{}'", edge.src),
                    });
                }
                if !staged.contains(edge.dst.as_str()) {
                    return Err(GraphError::BuildFailed {
                        stage: "stage_edges".to_string(),
                        detail: format!("dangling edge destination '{}'", edge.dst),
                    });
                }
                let id = format!("{build_row_id}:e:{}", batch_no * 500 + offset);
                sqlx::query(
                    "INSERT INTO graph_edges
                       (id, projection_row_id, src_stable_id, edge, dst_stable_id,
                        assertion_id, budgets_json, attrs_json, created_at)
                     VALUES (?, ?, ?, ?, ?, ?, '{}', ?, ?)",
                )
                .bind(&id)
                .bind(&build_row_id)
                .bind(&edge.src)
                .bind(&edge.edge)
                .bind(&edge.dst)
                .bind(edge.assertion_id.as_deref())
                .bind(edge.attrs.to_string())
                .bind(&at)
                .execute(&mut *tx)
                .await
                .map_err(|error| storage("stage_edges", error))?;
            }
        }
    }

    // Verify inside the transaction: counts, no dangling edges, and the
    // canonical pointer unchanged since collection.
    let stored_nodes: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM graph_nodes WHERE projection_row_id = ?")
            .bind(&build_row_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|error| storage("verify", error))?;
    let stored_edges: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM graph_edges WHERE projection_row_id = ?")
            .bind(&build_row_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|error| storage("verify", error))?;
    if stored_nodes as usize != node_count || stored_edges as usize != edge_count {
        return Err(GraphError::BuildFailed {
            stage: "verify".to_string(),
            detail: format!(
                "staged counts ({stored_nodes} nodes, {stored_edges} edges) differ from built \
                 ({node_count} nodes, {edge_count} edges)"
            ),
        });
    }
    let dangling: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM graph_edges e WHERE e.projection_row_id = ?
         AND (NOT EXISTS (SELECT 1 FROM graph_nodes n
                          WHERE n.projection_row_id = e.projection_row_id
                            AND n.stable_id = e.src_stable_id)
              OR NOT EXISTS (SELECT 1 FROM graph_nodes n
                             WHERE n.projection_row_id = e.projection_row_id
                               AND n.stable_id = e.dst_stable_id))",
    )
    .bind(&build_row_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|error| storage("verify", error))?;
    if dangling != 0 {
        return Err(GraphError::BuildFailed {
            stage: "verify".to_string(),
            detail: format!("{dangling} dangling edge(s) in staged projection"),
        });
    }
    let pointer: Option<(String, i64)> =
        sqlx::query_as("SELECT edition_id, corpus_generation FROM quran_active_edition")
            .fetch_optional(&mut *tx)
            .await
            .map_err(|error| storage("verify", error))?;
    match pointer {
        Some((edition_id, generation))
            if edition_id == collected.edition_id && generation == collected.corpus_generation => {}
        _ => {
            return Err(GraphError::BuildFailed {
                stage: "verify".to_string(),
                detail: "canonical pointer changed during the build; collection is stale"
                    .to_string(),
            });
        }
    }

    // Fenced publish: supersede the previous active row and flip the pointer
    // in the same commit.
    sqlx::query(
        "UPDATE graph_projections SET status = 'superseded'
         WHERE projection_id = ? AND status = 'active'",
    )
    .bind(quran_graph::STRUCTURAL_PROJECTION_ID)
    .execute(&mut *tx)
    .await
    .map_err(|error| storage("publish", error))?;
    let manifest_payload = serde_json::json!({
        "source": "quran_graph_build.publish_structural_build",
        "node_count": node_count,
        "edge_count": edge_count,
    });
    sqlx::query("UPDATE graph_projections SET status = 'active', manifest_json = ? WHERE id = ?")
        .bind(manifest_payload.to_string())
        .bind(&build_row_id)
        .execute(&mut *tx)
        .await
        .map_err(|error| storage("publish", error))?;
    let cursor = serde_json::json!({"node_count": node_count, "edge_count": edge_count});
    sqlx::query(
        "UPDATE graph_build_progress SET stage = 'published', cursor_json = ?, updated_at = ?
         WHERE build_id = ?",
    )
    .bind(cursor.to_string())
    .bind(&at)
    .bind(&build_row_id)
    .execute(&mut *tx)
    .await
    .map_err(|error| storage("publish", error))?;
    tx.commit().await.map_err(|error| storage("publish", error))?;
    pool.close().await;

    Ok(StructuralBuildReport {
        manifest: ProjectionManifest {
            id: build_row_id.clone(),
            projection_id: quran_graph::STRUCTURAL_PROJECTION_ID.to_string(),
            builder_version: quran_graph::STRUCTURAL_BUILDER_VERSION.to_string(),
            edition_id: collected.edition_id.clone(),
            corpus_generation: collected.corpus_generation.max(0) as u64,
            dataset_versions: std::collections::BTreeMap::new(),
            dependency_snapshot: snapshot
                .as_object()
                .map(|map| {
                    map.iter()
                        .map(|(key, value)| {
                            (key.clone(), value.as_str().unwrap_or_default().to_string())
                        })
                        .collect()
                })
                .unwrap_or_default(),
            status: ProjectionStatus::Active,
            manifest: manifest_payload,
            created_at: at,
        },
        build_row_id,
        node_count,
        edge_count,
        built,
    })
}

// ─── Word-root and annotated projections (plan 04-02, D-02/D-03/D-04/D-08) ──

/// Projection family ID for the word-root graph.
pub const WORDROOT_PROJECTION_ID: &str = "quran-wordroot-v1";
/// Builder version stamped into word-root projection rows.
pub const WORDROOT_BUILDER_VERSION: &str = "wordroot-v1";
/// Projection family ID for the annotated (concept/entity) graph.
pub const ANNOTATED_PROJECTION_ID: &str = "quran-annotated-v1";
/// Builder version stamped into annotated projection rows.
pub const ANNOTATED_BUILDER_VERSION: &str = "annotated-v1";
/// Authority scope marking seed-derived assertions.
pub const CONCEPT_SEED_SCOPE: &str = "concept-seed-v1";

/// Capability name carried by the word-root unavailable-dataset diagnostic.
pub const WORDROOT_CAPABILITY: &str = "word-root graph build";

/// One lexicon root used by the word-root builder.
#[derive(Debug, Clone)]
pub struct WordRoot {
    /// Lexicon row id (join key, never staged).
    pub id: String,
    /// Normalized root spelling (node identity).
    pub normalized: String,
}

/// One lexicon lemma used by the word-root builder.
#[derive(Debug, Clone)]
pub struct WordLemma {
    /// Lexicon row id (join key, never staged).
    pub id: String,
    /// Lemma spelling (node identity).
    pub lemma: String,
}

/// One token analysis used by the word-root builder (no winner elected:
/// every analysis emits its own edges; identical triples dedup).
#[derive(Debug, Clone)]
pub struct WordAnalysis {
    /// Surah number.
    pub surah: u32,
    /// Ayah number.
    pub ayah: u32,
    /// Token position.
    pub position: u32,
    /// Lemma row id, when the analysis carries one.
    pub lemma_id: Option<String>,
    /// Root row id, when the analysis carries one.
    pub root_id: Option<String>,
}

/// Word-root builder input: the active morphology dataset at a version.
#[derive(Debug, Clone)]
pub struct WordRootInput {
    /// Active dataset row id (`slug@version` content key).
    pub dataset_id: String,
    /// Dataset slug.
    pub dataset_slug: String,
    /// Dataset version.
    pub dataset_version: String,
    /// Dataset attribution string (copied verbatim into the attribution
    /// assertion; never merged or reinterpreted).
    pub dataset_attribution: String,
    /// Mechanical synthetic labeling: true when the dataset slug starts with
    /// `test-` or the attribution names synthetic data (OD-11 BLOCKED:
    /// production word-root links need owner dataset/license sign-off).
    pub synthetic_test_only: bool,
    /// Active edition id at collection (alignment reference; empty when no
    /// edition is active — word-root builds never require canonical data).
    pub edition_id: String,
    /// Corpus generation at collection (0 when no edition is active).
    pub corpus_generation: i64,
    /// Lexicon roots of the dataset.
    pub roots: Vec<WordRoot>,
    /// Lexicon lemmas of the dataset.
    pub lemmas: Vec<WordLemma>,
    /// Token analyses of the dataset.
    pub analyses: Vec<WordAnalysis>,
}

/// Collect the word-root input from the ACTIVE morphology dataset only
/// (D-03 gate, I11/I12/I13).
///
/// No active dataset returns the typed [`MorphologyToolError`]
/// `UnavailableDataset` naming the capability (QAI-MORPH-0004; CLI exit 5,
/// HTTP 404 conventions reserved for later surfaces) — never an empty
/// projection, never a heuristic fallback, never guessed roots.
pub async fn collect_wordroot_input(
    db: &SqliteDatabase,
) -> Result<WordRootInput, crate::quran_morphology::MorphologyToolError> {
    use crate::quran_morphology::MorphologyToolError as E;
    fn storage(error: storage::error::StorageError) -> E {
        E::Storage(error.to_string())
    }
    let mut uow = db.write().await.map_err(storage)?;
    let dataset = uow
        .quran()
        .active_dataset()
        .await
        .map_err(storage)?
        .ok_or_else(|| E::UnavailableDataset { capability: WORDROOT_CAPABILITY.to_string() })?;
    let active = uow.quran().get_active().await.map_err(storage)?;
    let roots = uow.quran().list_roots(&dataset.id).await.map_err(storage)?;
    let lemmas = uow.quran().list_lemmas(&dataset.id).await.map_err(storage)?;
    let analyses = uow.quran().list_analyses(&dataset.id).await.map_err(storage)?;
    let _ = uow.rollback().await;
    let synthetic_test_only = dataset.slug.starts_with("test-")
        || dataset.attribution.to_lowercase().contains("synthetic");
    Ok(WordRootInput {
        dataset_id: dataset.id.clone(),
        dataset_slug: dataset.slug.clone(),
        dataset_version: dataset.version.clone(),
        dataset_attribution: dataset.attribution.clone(),
        synthetic_test_only,
        edition_id: active.as_ref().map(|row| row.edition_id.clone()).unwrap_or_default(),
        corpus_generation: active.map(|row| row.corpus_generation).unwrap_or(0),
        roots: roots
            .into_iter()
            .map(|row| WordRoot { id: row.id, normalized: row.root_normalized })
            .collect(),
        lemmas: lemmas.into_iter().map(|row| WordLemma { id: row.id, lemma: row.lemma }).collect(),
        analyses: analyses
            .into_iter()
            .map(|row| WordAnalysis {
                surah: row.surah.max(0) as u32,
                ayah: row.ayah.max(0) as u32,
                position: row.token_position.max(0) as u32,
                lemma_id: row.lemma_id,
                root_id: row.root_id,
            })
            .collect(),
    })
}

/// Build word-root nodes and edges (pure, deterministic).
///
/// Nodes: `root:<normalized>`, `lemma:<lemma>`, `token:<s>:<a>:<p>`
/// (refs-only: numbers plus dataset identity, never surfaces). Edges:
/// `HAS_ROOT`/`HAS_LEMMA` per analysis plus `SAME_ROOT_AS`/`SAME_LEMMA_AS`
/// chains linking consecutive same-root/lemma tokens in canonical order
/// (bounded chains, not cliques). Every edge references the dataset
/// attribution assertion; identical triples dedup (a storage constraint, not
/// a winner election — competing analyses coexist in the dataset).
pub fn build_wordroot(
    input: &WordRootInput,
    attribution_assertion_id: &str,
) -> (Vec<quran_graph::GraphNode>, Vec<quran_graph::GraphEdge>) {
    use std::collections::{BTreeMap, HashMap};
    let root_by_id: HashMap<&str, &str> =
        input.roots.iter().map(|root| (root.id.as_str(), root.normalized.as_str())).collect();
    let lemma_by_id: HashMap<&str, &str> =
        input.lemmas.iter().map(|lemma| (lemma.id.as_str(), lemma.lemma.as_str())).collect();

    let mut nodes: BTreeMap<String, quran_graph::GraphNode> = BTreeMap::new();
    let mut edges: BTreeMap<(String, String, String), quran_graph::GraphEdge> = BTreeMap::new();
    let mut link = |src: String, edge: &str, dst: String| {
        edges.insert(
            (src.clone(), edge.to_string(), dst.clone()),
            quran_graph::GraphEdge {
                src,
                edge: edge.to_string(),
                dst,
                assertion_id: Some(attribution_assertion_id.to_string()),
                attrs: serde_json::json!({"dataset": input.dataset_id}),
            },
        );
    };
    for root in input.roots.iter().map(|root| root.normalized.as_str()) {
        let id = format!("root:{root}");
        nodes.insert(
            id.clone(),
            quran_graph::GraphNode::new(
                id,
                quran_graph::NodeKind::Root,
                serde_json::json!({"root": root, "dataset": input.dataset_id}),
            ),
        );
    }
    for lemma in input.lemmas.iter().map(|lemma| lemma.lemma.as_str()) {
        let id = format!("lemma:{lemma}");
        nodes.insert(
            id.clone(),
            quran_graph::GraphNode::new(
                id,
                quran_graph::NodeKind::Lemma,
                serde_json::json!({"lemma": lemma, "dataset": input.dataset_id}),
            ),
        );
    }
    // Token order is canonical (surah, ayah, position): chains are stable.
    let mut ordered: Vec<&WordAnalysis> = input.analyses.iter().collect();
    ordered.sort_by_key(|analysis| (analysis.surah, analysis.ayah, analysis.position));
    let mut root_chains: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    let mut lemma_chains: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for analysis in ordered {
        let token_id = format!("token:{}:{}:{}", analysis.surah, analysis.ayah, analysis.position);
        nodes.insert(
            token_id.clone(),
            quran_graph::GraphNode::new(
                token_id.clone(),
                quran_graph::NodeKind::Token,
                serde_json::json!({
                    "surah": analysis.surah,
                    "ayah": analysis.ayah,
                    "position": analysis.position,
                    "dataset": input.dataset_id,
                }),
            ),
        );
        if let Some(root_id) = analysis.root_id.as_deref()
            && let Some(root) = root_by_id.get(root_id)
        {
            link(token_id.clone(), "HAS_ROOT", format!("root:{root}"));
            root_chains.entry(root_id).or_default().push(token_id.clone());
        }
        if let Some(lemma_id) = analysis.lemma_id.as_deref()
            && let Some(lemma) = lemma_by_id.get(lemma_id)
        {
            link(token_id.clone(), "HAS_LEMMA", format!("lemma:{lemma}"));
            lemma_chains.entry(lemma_id).or_default().push(token_id.clone());
        }
    }
    for chain in root_chains.values() {
        for pair in chain.windows(2) {
            link(pair[0].clone(), "SAME_ROOT_AS", pair[1].clone());
        }
    }
    for chain in lemma_chains.values() {
        for pair in chain.windows(2) {
            link(pair[0].clone(), "SAME_LEMMA_AS", pair[1].clone());
        }
    }
    (nodes.into_values().collect(), edges.into_values().collect())
}

// ─── Shared publish helpers (reserve/stage/verify/flip) ───

fn build_storage(stage: &str, error: sqlx::Error) -> GraphError {
    GraphError::BuildFailed { stage: stage.to_string(), detail: error.to_string() }
}

async fn open_build_pool(db_path: &str) -> Result<sqlx::sqlite::SqlitePool, GraphError> {
    sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(db_path)
                .create_if_missing(false)
                .foreign_keys(true)
                .busy_timeout(std::time::Duration::from_secs(5)),
        )
        .await
        .map_err(|error| build_storage("open", error))
}

/// Reserve a `graph_projections` row in `building` state plus its progress
/// cursor. Returns the build row id and timestamp.
async fn reserve_build_row(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    projection_id: &str,
    builder_version: &str,
    edition_id: &str,
    corpus_generation: i64,
    dataset_versions: &std::collections::BTreeMap<String, String>,
    dependency_snapshot: &serde_json::Value,
) -> Result<(String, String), GraphError> {
    let short = uuid::Uuid::new_v4().to_string();
    let build_row_id = format!("build-{projection_id}-{corpus_generation}-{}", &short[..8]);
    let at = domain::Timestamp::now().to_string();
    sqlx::query(
        "INSERT INTO graph_projections
            (id, projection_id, builder_version, edition_id, corpus_generation,
             dataset_versions_json, dependency_snapshot_json, status, manifest_json, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, 'building', '{}', ?)",
    )
    .bind(&build_row_id)
    .bind(projection_id)
    .bind(builder_version)
    .bind(edition_id)
    .bind(corpus_generation)
    .bind(serde_json::to_value(dataset_versions).unwrap_or_default().to_string())
    .bind(dependency_snapshot.to_string())
    .bind(&at)
    .execute(&mut **tx)
    .await
    .map_err(|error| build_storage("reserve", error))?;
    sqlx::query(
        "INSERT INTO graph_build_progress
            (build_id, projection_row_id, stage, cursor_json, updated_at)
         VALUES (?, ?, 'reserve', '{}', ?)",
    )
    .bind(&build_row_id)
    .bind(&build_row_id)
    .bind(&at)
    .execute(&mut **tx)
    .await
    .map_err(|error| build_storage("reserve", error))?;
    Ok((build_row_id, at))
}

/// Stage nodes enforcing the port's rejection rule (no empty stable IDs).
async fn stage_nodes_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    build_row_id: &str,
    nodes: &[quran_graph::GraphNode],
    at: &str,
) -> Result<(), GraphError> {
    for (batch_no, batch) in nodes.chunks(500).enumerate() {
        for (offset, node) in batch.iter().enumerate() {
            if node.stable_id.trim().is_empty() {
                return Err(GraphError::BuildFailed {
                    stage: "stage_nodes".to_string(),
                    detail: "refused node with empty stable ID".to_string(),
                });
            }
            let id = format!("{build_row_id}:n:{}:{}", batch_no * 500 + offset, node.stable_id);
            sqlx::query(
                "INSERT INTO graph_nodes
                    (id, projection_row_id, node_kind, stable_id, attrs_json, created_at)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(build_row_id)
            .bind(node.kind.to_string())
            .bind(&node.stable_id)
            .bind(node.attrs.to_string())
            .bind(at)
            .execute(&mut **tx)
            .await
            .map_err(|error| build_storage("stage_nodes", error))?;
        }
    }
    Ok(())
}

/// Stage edges enforcing the port's rejection rules: allowlisted predicates
/// only, both endpoints already staged (zero dangling edges). `OR IGNORE`
/// keeps rebuild re-derivation idempotent under the asserted-edge unique
/// index.
async fn stage_edges_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    build_row_id: &str,
    edges: &[quran_graph::GraphEdge],
    at: &str,
) -> Result<(), GraphError> {
    use std::collections::BTreeSet;
    let staged_rows: Vec<String> =
        sqlx::query_scalar("SELECT stable_id FROM graph_nodes WHERE projection_row_id = ?")
            .bind(build_row_id)
            .fetch_all(&mut **tx)
            .await
            .map_err(|error| build_storage("stage_edges", error))?;
    let staged: BTreeSet<String> = staged_rows.into_iter().collect();
    for (batch_no, batch) in edges.chunks(500).enumerate() {
        for (offset, edge) in batch.iter().enumerate() {
            if !is_allowed_edge(&edge.edge) {
                return Err(GraphError::PatternRejected {
                    detail: format!("cannot stage unknown edge '{}'", edge.edge),
                });
            }
            if !staged.contains(&edge.src) {
                return Err(GraphError::BuildFailed {
                    stage: "stage_edges".to_string(),
                    detail: format!("dangling edge source '{}'", edge.src),
                });
            }
            if !staged.contains(&edge.dst) {
                return Err(GraphError::BuildFailed {
                    stage: "stage_edges".to_string(),
                    detail: format!("dangling edge destination '{}'", edge.dst),
                });
            }
            let id = format!("{build_row_id}:e:{}", batch_no * 500 + offset);
            sqlx::query(
                "INSERT OR IGNORE INTO graph_edges
                    (id, projection_row_id, src_stable_id, edge, dst_stable_id,
                     assertion_id, budgets_json, attrs_json, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, '{}', ?, ?)",
            )
            .bind(&id)
            .bind(build_row_id)
            .bind(&edge.src)
            .bind(&edge.edge)
            .bind(&edge.dst)
            .bind(edge.assertion_id.as_deref())
            .bind(edge.attrs.to_string())
            .bind(at)
            .execute(&mut **tx)
            .await
            .map_err(|error| build_storage("stage_edges", error))?;
        }
    }
    Ok(())
}

/// Verify staged counts plus dangling-freedom inside the transaction.
async fn verify_staged(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    build_row_id: &str,
    node_count: usize,
    edge_count: usize,
) -> Result<(), GraphError> {
    let stored_nodes: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM graph_nodes WHERE projection_row_id = ?")
            .bind(build_row_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(|error| build_storage("verify", error))?;
    let stored_edges: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM graph_edges WHERE projection_row_id = ?")
            .bind(build_row_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(|error| build_storage("verify", error))?;
    if stored_nodes as usize != node_count || stored_edges as usize != edge_count {
        return Err(GraphError::BuildFailed {
            stage: "verify".to_string(),
            detail: format!(
                "staged counts ({stored_nodes} nodes, {stored_edges} edges) differ from built \
                 ({node_count} nodes, {edge_count} edges)"
            ),
        });
    }
    let dangling: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM graph_edges e WHERE e.projection_row_id = ?
         AND (NOT EXISTS (SELECT 1 FROM graph_nodes n
                          WHERE n.projection_row_id = e.projection_row_id
                            AND n.stable_id = e.src_stable_id)
              OR NOT EXISTS (SELECT 1 FROM graph_nodes n
                             WHERE n.projection_row_id = e.projection_row_id
                               AND n.stable_id = e.dst_stable_id))",
    )
    .bind(build_row_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|error| build_storage("verify", error))?;
    if dangling != 0 {
        return Err(GraphError::BuildFailed {
            stage: "verify".to_string(),
            detail: format!("{dangling} dangling edge(s) in staged projection"),
        });
    }
    Ok(())
}

/// Fenced publish: supersede the previous active row of the family and flip
/// the new row to `active` in the same commit.
async fn flip_to_active(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    projection_id: &str,
    build_row_id: &str,
    manifest_payload: &serde_json::Value,
    cursor: &serde_json::Value,
    at: &str,
) -> Result<(), GraphError> {
    sqlx::query(
        "UPDATE graph_projections SET status = 'superseded'
         WHERE projection_id = ? AND status = 'active'",
    )
    .bind(projection_id)
    .execute(&mut **tx)
    .await
    .map_err(|error| build_storage("publish", error))?;
    sqlx::query("UPDATE graph_projections SET status = 'active', manifest_json = ? WHERE id = ?")
        .bind(manifest_payload.to_string())
        .bind(build_row_id)
        .execute(&mut **tx)
        .await
        .map_err(|error| build_storage("publish", error))?;
    sqlx::query(
        "UPDATE graph_build_progress SET stage = 'published', cursor_json = ?, updated_at = ?
         WHERE build_id = ?",
    )
    .bind(cursor.to_string())
    .bind(at)
    .bind(build_row_id)
    .execute(&mut **tx)
    .await
    .map_err(|error| build_storage("publish", error))?;
    Ok(())
}

/// Manifest assembly for one published build row.
struct ManifestParts {
    build_row_id: String,
    projection_id: &'static str,
    builder_version: &'static str,
    edition_id: String,
    corpus_generation: i64,
    dataset_versions: std::collections::BTreeMap<String, String>,
    dependency_snapshot: serde_json::Value,
    manifest_payload: serde_json::Value,
    created_at: String,
}

fn build_manifest(parts: ManifestParts) -> ProjectionManifest {
    ProjectionManifest {
        id: parts.build_row_id,
        projection_id: parts.projection_id.to_string(),
        builder_version: parts.builder_version.to_string(),
        edition_id: parts.edition_id,
        corpus_generation: parts.corpus_generation.max(0) as u64,
        dataset_versions: parts.dataset_versions,
        dependency_snapshot: parts
            .dependency_snapshot
            .as_object()
            .map(|map| {
                map.iter()
                    .map(|(key, value)| {
                        (key.clone(), value.as_str().unwrap_or_default().to_string())
                    })
                    .collect()
            })
            .unwrap_or_default(),
        status: ProjectionStatus::Active,
        manifest: parts.manifest_payload,
        created_at: parts.created_at,
    }
}

/// One dataset/seed attribution assertion write.
struct AttributionWrite {
    id: String,
    build_row_id: String,
    projection_id: &'static str,
    edition_id: String,
    dataset_scope: String,
    claim: serde_json::Value,
    source_location: String,
    at: String,
}

/// Insert one dataset/seed attribution assertion (decision pending, layer
/// `B`): every word-root/annotated edge references it, so the dataset or
/// seed stays attributed even though the edges traverse while pending.
async fn insert_attribution_assertion(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    write: AttributionWrite,
) -> Result<(), GraphError> {
    sqlx::query(
        "INSERT INTO graph_assertions
            (id, projection_row_id, projection_id, edition_id, dataset_scope,
             assertion_kind, claim_json, evidence_json, source_location,
             reviewer, decision, decided_at, supersedes_id,
             provenance_layer, algorithm, algorithm_version, confidence, created_at)
         VALUES (?, ?, ?, ?, ?, 'import', ?, '{}', ?, NULL, 'pending', NULL, NULL,
                 'B', NULL, NULL, NULL, ?)",
    )
    .bind(&write.id)
    .bind(&write.build_row_id)
    .bind(write.projection_id)
    .bind(&write.edition_id)
    .bind(&write.dataset_scope)
    .bind(write.claim.to_string())
    .bind(&write.source_location)
    .bind(&write.at)
    .execute(&mut **tx)
    .await
    .map_err(|error| build_storage("stage_assertion", error))?;
    Ok(())
}

/// Re-derive adjacency for every effective assertion in scope (AC-P4-03).
///
/// After an adjacency wipe plus republish, accepted/pending/disputed
/// assertions authorize traversal again without losing their review history:
/// edges stage from the claim triple (`review-<assertion_id>` ids, `OR
/// IGNORE` idempotent) only when both endpoints exist in the new build.
/// Rows whose claim lacks a triple or whose endpoints are gone are skipped
/// and counted — never dangling, never a build failure.
pub async fn stage_effective_assertion_edges(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    projection_id: &str,
    edition_id: &str,
    build_row_id: &str,
    at: &str,
) -> Result<(usize, usize), GraphError> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT id, claim_json FROM graph_assertions
         WHERE projection_id = ? AND edition_id = ?
           AND decision IN ('pending', 'accepted', 'disputed')
         ORDER BY COALESCE(decided_at, ''), id",
    )
    .bind(projection_id)
    .bind(edition_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|error| build_storage("rederive", error))?;
    let (mut staged, mut skipped) = (0usize, 0usize);
    for (assertion_id, claim_json) in rows {
        let claim: serde_json::Value = serde_json::from_str(&claim_json).unwrap_or_default();
        let triple = claim
            .get("src")
            .and_then(|value| value.as_str())
            .zip(claim.get("edge").and_then(|value| value.as_str()))
            .and_then(|(src, edge)| {
                claim.get("dst").and_then(|value| value.as_str()).map(|dst| (src, edge, dst))
            });
        let Some((src, edge, dst)) = triple else {
            skipped += 1;
            continue;
        };
        if !is_allowed_edge(edge) {
            skipped += 1;
            continue;
        }
        let mut endpoints = 0i64;
        for endpoint in [src, dst] {
            let present: Option<String> = sqlx::query_scalar(
                "SELECT stable_id FROM graph_nodes
                 WHERE projection_row_id = ? AND stable_id = ? LIMIT 1",
            )
            .bind(build_row_id)
            .bind(endpoint)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|error| build_storage("rederive", error))?;
            if present.is_some() {
                endpoints += 1;
            }
        }
        if endpoints != 2 {
            skipped += 1;
            continue;
        }
        let changed = sqlx::query(
            "INSERT OR IGNORE INTO graph_edges
                (id, projection_row_id, src_stable_id, edge, dst_stable_id,
                 assertion_id, budgets_json, attrs_json, created_at)
             VALUES (?, ?, ?, ?, ?, ?, '{}', '{}', ?)",
        )
        .bind(format!("review-{assertion_id}"))
        .bind(build_row_id)
        .bind(src)
        .bind(edge)
        .bind(dst)
        .bind(&assertion_id)
        .bind(at)
        .execute(&mut **tx)
        .await
        .map_err(|error| build_storage("rederive", error))?
        .rows_affected();
        if changed == 1 {
            staged += 1;
        }
    }
    Ok((staged, skipped))
}

/// Drop adjacency (nodes plus edges) of one build row, never authority
/// (AC-P4-03).
///
/// Rebuilds wipe and republish adjacency; `graph_assertions` rows (plus
/// their provenance, audit, and outbox history) survive untouched, keyed by
/// (`projection_id`, `edition_id`, `dataset_scope`).
pub async fn clear_projection_adjacency(
    db_path: &str,
    build_row_id: &str,
) -> Result<(u64, u64), GraphError> {
    let pool = open_build_pool(db_path).await?;
    let mut tx = pool.begin().await.map_err(|error| build_storage("clear", error))?;
    let edges = sqlx::query("DELETE FROM graph_edges WHERE projection_row_id = ?")
        .bind(build_row_id)
        .execute(&mut *tx)
        .await
        .map_err(|error| build_storage("clear", error))?
        .rows_affected();
    let nodes = sqlx::query("DELETE FROM graph_nodes WHERE projection_row_id = ?")
        .bind(build_row_id)
        .execute(&mut *tx)
        .await
        .map_err(|error| build_storage("clear", error))?
        .rows_affected();
    tx.commit().await.map_err(|error| build_storage("clear", error))?;
    pool.close().await;
    Ok((nodes, edges))
}

/// Word-root publish report.
pub struct WordRootBuildReport {
    /// Manifest of the newly activated build row.
    pub manifest: ProjectionManifest,
    /// The activated `graph_projections` build row id.
    pub build_row_id: String,
    /// Staged node count (including re-derived review nodes: none — review
    /// edges reuse staged nodes, so this equals built nodes).
    pub node_count: usize,
    /// Staged edge count (built edges plus re-derived effective edges).
    pub edge_count: usize,
    /// Dataset attribution assertion id referenced by every built edge.
    pub attribution_assertion_id: String,
    /// Re-derived effective assertion edges staged from authority.
    pub rederived: usize,
    /// Authority rows skipped during re-derivation (no triple or missing
    /// endpoints).
    pub rederived_skipped: usize,
}

/// Publish the word-root projection through the fenced pipeline (D-03).
///
/// Reserve → stage (nodes, asserted edges, re-derived scope edges) → verify
/// (counts plus no-dangling) → pointer flip, one transaction. No
/// canonical-unchanged check: word-root derives from the morphology dataset,
/// not the corpus; the pinned edition generation is an alignment reference
/// only.
pub async fn publish_wordroot_build(
    db_path: &str,
    input: &WordRootInput,
) -> Result<WordRootBuildReport, GraphError> {
    let pool = open_build_pool(db_path).await?;
    let mut tx = pool.begin().await.map_err(|error| build_storage("reserve", error))?;
    let snapshot = serde_json::json!({
        "canonical": format!("generation-{}", input.corpus_generation),
        "dataset": format!("{}@{}", input.dataset_slug, input.dataset_version),
    });
    let mut dataset_versions = std::collections::BTreeMap::new();
    dataset_versions.insert(input.dataset_slug.clone(), input.dataset_version.clone());
    let (build_row_id, at) = reserve_build_row(
        &mut tx,
        WORDROOT_PROJECTION_ID,
        WORDROOT_BUILDER_VERSION,
        &input.edition_id,
        input.corpus_generation,
        &dataset_versions,
        &snapshot,
    )
    .await?;
    let attribution_id = format!("{build_row_id}-attribution");
    insert_attribution_assertion(
        &mut tx,
        AttributionWrite {
            id: attribution_id.clone(),
            build_row_id: build_row_id.clone(),
            projection_id: WORDROOT_PROJECTION_ID,
            edition_id: input.edition_id.clone(),
            dataset_scope: input.dataset_id.clone(),
            claim: serde_json::json!({
                "dataset": input.dataset_id,
                "dataset_slug": input.dataset_slug,
                "dataset_version": input.dataset_version,
                "attribution": input.dataset_attribution,
                "synthetic_test_only": input.synthetic_test_only,
                "provenance": {
                    "source_id": input.dataset_id,
                    "source_location": format!("morphology dataset {}@{}", input.dataset_slug, input.dataset_version),
                    "author_or_algorithm": input.dataset_attribution,
                    "version": input.dataset_version,
                    "confidence": null,
                    "verification_status": "unverified",
                    "created_at": at,
                },
            }),
            source_location: format!(
                "morphology dataset {}@{}",
                input.dataset_slug, input.dataset_version
            ),
            at: at.clone(),
        },
    )
    .await?;
    let (nodes, edges) = build_wordroot(input, &attribution_id);
    stage_nodes_tx(&mut tx, &build_row_id, &nodes, &at).await?;
    stage_edges_tx(&mut tx, &build_row_id, &edges, &at).await?;
    let (rederived, rederived_skipped) = stage_effective_assertion_edges(
        &mut tx,
        WORDROOT_PROJECTION_ID,
        &input.edition_id,
        &build_row_id,
        &at,
    )
    .await?;
    let node_count = nodes.len();
    let edge_count = edges.len() + rederived;
    verify_staged(&mut tx, &build_row_id, node_count, edge_count).await?;
    let manifest_payload = serde_json::json!({
        "source": "quran_graph_build.publish_wordroot_build",
        "node_count": node_count,
        "edge_count": edge_count,
        "dataset": input.dataset_id,
        "synthetic_test_only": input.synthetic_test_only,
    });
    let cursor = serde_json::json!({
        "node_count": node_count,
        "edge_count": edge_count,
        "rederived": rederived,
        "rederived_skipped": rederived_skipped,
    });
    flip_to_active(&mut tx, WORDROOT_PROJECTION_ID, &build_row_id, &manifest_payload, &cursor, &at)
        .await?;
    tx.commit().await.map_err(|error| build_storage("publish", error))?;
    pool.close().await;
    Ok(WordRootBuildReport {
        manifest: build_manifest(ManifestParts {
            build_row_id: build_row_id.clone(),
            projection_id: WORDROOT_PROJECTION_ID,
            builder_version: WORDROOT_BUILDER_VERSION,
            edition_id: input.edition_id.clone(),
            corpus_generation: input.corpus_generation,
            dataset_versions,
            dependency_snapshot: snapshot,
            manifest_payload,
            created_at: at,
        }),
        build_row_id,
        node_count,
        edge_count,
        attribution_assertion_id: attribution_id,
        rederived,
        rederived_skipped,
    })
}

// ─── Annotated projection (D-04/D-05, refs-only) ───

/// One concept/entity node in the annotated seed.
#[derive(Debug, Clone)]
pub struct AnnotatedNode {
    /// Stable ID (`concept:*` or `entity:*`).
    pub id: String,
    /// Display label (metadata only; never canonical text).
    pub label: String,
    /// `theme`, `person`, or `place`.
    pub kind: String,
    /// Attribution carried verbatim from the seed.
    pub reviewed_by: String,
}

/// One seed link: a curated edge between staged nodes.
#[derive(Debug, Clone)]
pub struct AnnotatedLink {
    /// Edge source stable ID.
    pub src: String,
    /// Allowlisted edge predicate.
    pub edge: String,
    /// Edge destination stable ID.
    pub dst: String,
    /// Supporting evidence.
    pub evidence: serde_json::Value,
    /// Source location within the seed.
    pub source_location: String,
}

/// Parsed annotated seed (`concept-seed-v1.json`).
#[derive(Debug, Clone)]
pub struct AnnotatedSeed {
    /// Seed version (must be `v1`).
    pub version: String,
    /// Edition scope the seed covers.
    pub edition_scope: String,
    /// The seed declares itself synthetic-only (OD-12 BLOCKED: production
    /// seeds require scholar curation).
    pub synthetic_test_only: bool,
    /// Concept plus entity nodes.
    pub nodes: Vec<AnnotatedNode>,
    /// Curated links.
    pub links: Vec<AnnotatedLink>,
}

/// Parse and shape-check the annotated seed (unknown predicates and unknown
/// endpoints are typed rejections, never staged).
pub fn parse_annotated_seed(seed_json: &str) -> Result<AnnotatedSeed, GraphError> {
    let seed: serde_json::Value = serde_json::from_str(seed_json).map_err(|error| {
        GraphError::PatternRejected { detail: format!("concept seed is not valid JSON: {error}") }
    })?;
    if seed.get("format").and_then(|value| value.as_str()) != Some("qai.graph.concept-seed-v1") {
        return Err(GraphError::PatternRejected {
            detail: "concept seed must carry format qai.graph.concept-seed-v1".to_string(),
        });
    }
    let version = seed.get("version").and_then(|value| value.as_str()).unwrap_or_default();
    if version != "v1" {
        return Err(GraphError::PatternRejected {
            detail: format!("unsupported concept seed version '{version}'"),
        });
    }
    let mut nodes = Vec::new();
    for (section, kind) in [("concepts", "theme"), ("persons", "person"), ("places", "place")] {
        let items = seed.get(section).and_then(|value| value.as_array()).ok_or_else(|| {
            GraphError::PatternRejected {
                detail: format!("concept seed section '{section}' must be a list"),
            }
        })?;
        for item in items {
            let id = item.get("id").and_then(|value| value.as_str()).unwrap_or_default();
            if id.trim().is_empty() {
                return Err(GraphError::PatternRejected {
                    detail: format!("concept seed '{section}' entry needs a stable id"),
                });
            }
            nodes.push(AnnotatedNode {
                id: id.to_string(),
                label: item
                    .get("label")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string(),
                kind: kind.to_string(),
                reviewed_by: item
                    .get("attribution")
                    .and_then(|attribution| attribution.get("reviewed_by"))
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string(),
            });
        }
    }
    let known: std::collections::BTreeSet<&str> =
        nodes.iter().map(|node| node.id.as_str()).collect();
    let mut links = Vec::new();
    for (index, item) in seed
        .get("links")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        let src = item.get("src").and_then(|value| value.as_str()).unwrap_or_default();
        let edge = item.get("edge").and_then(|value| value.as_str()).unwrap_or_default();
        let dst = item.get("dst").and_then(|value| value.as_str()).unwrap_or_default();
        if !is_allowed_edge(edge) {
            return Err(GraphError::PatternRejected {
                detail: format!("concept seed links[{index}] carries unknown edge '{edge}'"),
            });
        }
        if !dst.starts_with("concept:") && !dst.starts_with("entity:") || !known.contains(dst) {
            return Err(GraphError::BuildFailed {
                stage: "collect".to_string(),
                detail: format!("concept seed links[{index}] points at unknown node '{dst}'"),
            });
        }
        if src.trim().is_empty() || dst.trim().is_empty() {
            return Err(GraphError::BuildFailed {
                stage: "collect".to_string(),
                detail: format!("concept seed links[{index}] needs non-empty endpoints"),
            });
        }
        links.push(AnnotatedLink {
            src: src.to_string(),
            edge: edge.to_string(),
            dst: dst.to_string(),
            evidence: item.get("evidence").cloned().unwrap_or_default(),
            source_location: item
                .get("source_location")
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string(),
        });
    }
    Ok(AnnotatedSeed {
        version: version.to_string(),
        edition_scope: seed
            .get("edition_scope")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        synthetic_test_only: seed
            .get("synthetic_test_only")
            .and_then(|value| value.as_bool())
            .unwrap_or(false),
        nodes,
        links,
    })
}

/// Build annotated nodes and edges (pure, deterministic).
///
/// Nodes: every edition ayah (refs-only `{surah, ayah}`, enabling seed-link
/// and coverage endpoints), every seed concept/entity (synthetic labeling
/// carried verbatim), plus one `translation-edition:<slug>` node per
/// registered translation edition. Edges: seed links plus `TRANSLATES`
/// coverage edges (ayah → translation-edition node) for every ayah in scope
/// times every registered translation. Translation edges point at existing
/// translation edition refs and never carry text (ADR-0112): the builder
/// reads translation *editions*, never *passages*.
pub fn build_annotated(
    seed: &AnnotatedSeed,
    ayahs: &[(u32, u32)],
    translation_slugs: &[String],
    attribution_assertion_id: &str,
) -> (Vec<quran_graph::GraphNode>, Vec<quran_graph::GraphEdge>) {
    use std::collections::BTreeMap;
    let mut nodes: BTreeMap<String, quran_graph::GraphNode> = BTreeMap::new();
    let mut edges: BTreeMap<(String, String, String), quran_graph::GraphEdge> = BTreeMap::new();
    let mut link = |src: String, edge: &str, dst: String| {
        edges.insert(
            (src.clone(), edge.to_string(), dst.clone()),
            quran_graph::GraphEdge {
                src,
                edge: edge.to_string(),
                dst,
                assertion_id: Some(attribution_assertion_id.to_string()),
                attrs: serde_json::json!({"seed": CONCEPT_SEED_SCOPE}),
            },
        );
    };
    for (surah, ayah) in ayahs {
        let id = format!("ayah:{surah}:{ayah}");
        nodes.insert(
            id.clone(),
            quran_graph::GraphNode::new(
                id,
                quran_graph::NodeKind::Ayah,
                serde_json::json!({"surah": surah, "ayah": ayah}),
            ),
        );
    }
    for node in &seed.nodes {
        let kind = if node.kind == "theme" {
            quran_graph::NodeKind::Concept
        } else {
            quran_graph::NodeKind::Entity
        };
        nodes.insert(
            node.id.clone(),
            quran_graph::GraphNode::new(
                node.id.clone(),
                kind,
                serde_json::json!({
                    "label": node.label,
                    "kind": node.kind,
                    "reviewed_by": node.reviewed_by,
                    "synthetic_test_only": seed.synthetic_test_only,
                }),
            ),
        );
    }
    for slug in translation_slugs {
        let id = format!("translation-edition:{slug}");
        nodes.insert(
            id.clone(),
            quran_graph::GraphNode::new(
                id,
                quran_graph::NodeKind::Entity,
                serde_json::json!({"translation_slug": slug, "kind": "translation-edition"}),
            ),
        );
    }
    let ayah_ids: Vec<String> =
        ayahs.iter().map(|(surah, ayah)| format!("ayah:{surah}:{ayah}")).collect();
    for item in &seed.links {
        link(item.src.clone(), &item.edge, item.dst.clone());
    }
    for ayah_id in &ayah_ids {
        for slug in translation_slugs {
            link(ayah_id.clone(), "TRANSLATES", format!("translation-edition:{slug}"));
        }
    }
    (nodes.into_values().collect(), edges.into_values().collect())
}

/// Annotated publish report.
pub struct AnnotatedBuildReport {
    /// Manifest of the newly activated build row.
    pub manifest: ProjectionManifest,
    /// The activated `graph_projections` build row id.
    pub build_row_id: String,
    /// Staged node count.
    pub node_count: usize,
    /// Staged edge count (seed edges plus coverage plus re-derived).
    pub edge_count: usize,
    /// Seed attribution assertion id referenced by every seed edge.
    pub attribution_assertion_id: String,
    /// Re-derived effective assertion edges staged from authority.
    pub rederived: usize,
    /// Authority rows skipped during re-derivation.
    pub rederived_skipped: usize,
}

/// Publish the annotated projection through the fenced pipeline (D-04/D-05).
///
/// Consumes the concept seed plus the active edition's ayah refs plus the
/// registered translation editions (refs only). Seed-link ayah endpoints
/// must exist in the active edition; translation slugs must resolve to
/// registered translation editions (the builder lists them — unknown refs
/// cannot be constructed).
pub async fn publish_annotated_build(
    db_path: &str,
    seed_json: &str,
) -> Result<AnnotatedBuildReport, GraphError> {
    let seed = parse_annotated_seed(seed_json)?;
    let db = SqliteDatabase::new(db_path, 4, true).await.map_err(|error| {
        GraphError::BuildFailed { stage: "open".to_string(), detail: error.to_string() }
    })?;
    let collected = collect_structural_input(&db).await?;
    let Some(collected) = collected else {
        return Err(GraphError::BuildFailed {
            stage: "collect".to_string(),
            detail: "no active edition; import one first".to_string(),
        });
    };
    let ayahs: Vec<(u32, u32)> = {
        let mut keys: std::collections::BTreeSet<(u32, u32)> =
            collected.input.ayahs.iter().map(|ayah| (ayah.surah, ayah.ayah)).collect();
        for token in &collected.input.tokens {
            keys.insert((token.surah, token.ayah));
        }
        keys.into_iter().collect()
    };
    // Seed-link ayah endpoints must exist in the edition scope.
    for (index, item) in seed.links.iter().enumerate() {
        if let Some(rest) = item.src.strip_prefix("ayah:")
            && let [surah, ayah] = rest.split(':').collect::<Vec<_>>()[..]
        {
            let endpoint = (surah.parse::<u32>().unwrap_or(0), ayah.parse::<u32>().unwrap_or(0));
            if !ayahs.contains(&endpoint) {
                return Err(GraphError::BuildFailed {
                    stage: "collect".to_string(),
                    detail: format!(
                        "concept seed links[{index}] ayah '{}' is outside the edition scope",
                        item.src
                    ),
                });
            }
        } else if item.src.starts_with("ayah:") {
            return Err(GraphError::BuildFailed {
                stage: "collect".to_string(),
                detail: format!(
                    "concept seed links[{index}] carries an unparseable ayah '{}'",
                    item.src
                ),
            });
        }
    }
    let translation_slugs = {
        fn collect(error: storage::error::StorageError) -> GraphError {
            GraphError::BuildFailed { stage: "collect".to_string(), detail: error.to_string() }
        }
        let mut uow = db.write().await.map_err(collect)?;
        let editions = uow.quran().list_translation_editions().await.map_err(collect)?;
        let _ = uow.rollback().await;
        let mut slugs: Vec<String> = editions.into_iter().map(|row| row.slug).collect();
        slugs.sort();
        slugs.dedup();
        slugs
    };

    let pool = open_build_pool(db_path).await?;
    let mut tx = pool.begin().await.map_err(|error| build_storage("reserve", error))?;
    let snapshot = serde_json::json!({
        "canonical": format!("generation-{}", collected.corpus_generation),
        "seed": CONCEPT_SEED_SCOPE,
    });
    let mut dataset_versions = std::collections::BTreeMap::new();
    dataset_versions.insert(CONCEPT_SEED_SCOPE.to_string(), seed.version.clone());
    let (build_row_id, at) = reserve_build_row(
        &mut tx,
        ANNOTATED_PROJECTION_ID,
        ANNOTATED_BUILDER_VERSION,
        &collected.edition_id,
        collected.corpus_generation,
        &dataset_versions,
        &snapshot,
    )
    .await?;
    let attribution_id = format!("{build_row_id}-attribution");
    insert_attribution_assertion(
        &mut tx,
        AttributionWrite {
            id: attribution_id.clone(),
            build_row_id: build_row_id.clone(),
            projection_id: ANNOTATED_PROJECTION_ID,
            edition_id: collected.edition_id.clone(),
            dataset_scope: CONCEPT_SEED_SCOPE.to_string(),
            claim: serde_json::json!({
                "seed": CONCEPT_SEED_SCOPE,
                "seed_version": seed.version,
                "edition_scope": seed.edition_scope,
                "synthetic_test_only": seed.synthetic_test_only,
                "translations": translation_slugs,
                "provenance": {
                    "source_id": CONCEPT_SEED_SCOPE,
                    "source_location": "fixtures/quran/graph/concept-seed-v1.json",
                    "author_or_algorithm": "phase-4-tracer",
                    "version": seed.version,
                    "confidence": null,
                    "verification_status": "unverified",
                    "created_at": at,
                },
            }),
            source_location: "fixtures/quran/graph/concept-seed-v1.json".to_string(),
            at: at.clone(),
        },
    )
    .await?;
    let (nodes, edges) = build_annotated(&seed, &ayahs, &translation_slugs, &attribution_id);
    stage_nodes_tx(&mut tx, &build_row_id, &nodes, &at).await?;
    stage_edges_tx(&mut tx, &build_row_id, &edges, &at).await?;
    let (rederived, rederived_skipped) = stage_effective_assertion_edges(
        &mut tx,
        ANNOTATED_PROJECTION_ID,
        &collected.edition_id,
        &build_row_id,
        &at,
    )
    .await?;
    let node_count = nodes.len();
    let edge_count = edges.len() + rederived;
    verify_staged(&mut tx, &build_row_id, node_count, edge_count).await?;
    // The canonical pointer must not move under the build (same fenced
    // guarantee as the structural pipeline).
    let pointer: Option<(String, i64)> =
        sqlx::query_as("SELECT edition_id, corpus_generation FROM quran_active_edition")
            .fetch_optional(&mut *tx)
            .await
            .map_err(|error| build_storage("verify", error))?;
    match pointer {
        Some((edition_id, generation))
            if edition_id == collected.edition_id && generation == collected.corpus_generation => {}
        _ => {
            return Err(GraphError::BuildFailed {
                stage: "verify".to_string(),
                detail: "canonical pointer changed during the build; collection is stale"
                    .to_string(),
            });
        }
    }
    let manifest_payload = serde_json::json!({
        "source": "quran_graph_build.publish_annotated_build",
        "node_count": node_count,
        "edge_count": edge_count,
        "seed": CONCEPT_SEED_SCOPE,
        "synthetic_test_only": seed.synthetic_test_only,
    });
    let cursor = serde_json::json!({
        "node_count": node_count,
        "edge_count": edge_count,
        "rederived": rederived,
        "rederived_skipped": rederived_skipped,
    });
    flip_to_active(
        &mut tx,
        ANNOTATED_PROJECTION_ID,
        &build_row_id,
        &manifest_payload,
        &cursor,
        &at,
    )
    .await?;
    tx.commit().await.map_err(|error| build_storage("publish", error))?;
    pool.close().await;
    Ok(AnnotatedBuildReport {
        manifest: build_manifest(ManifestParts {
            build_row_id: build_row_id.clone(),
            projection_id: ANNOTATED_PROJECTION_ID,
            builder_version: ANNOTATED_BUILDER_VERSION,
            edition_id: collected.edition_id.clone(),
            corpus_generation: collected.corpus_generation,
            dataset_versions,
            dependency_snapshot: snapshot,
            manifest_payload,
            created_at: at,
        }),
        build_row_id,
        node_count,
        edge_count,
        attribution_assertion_id: attribution_id,
        rederived,
        rederived_skipped,
    })
}
