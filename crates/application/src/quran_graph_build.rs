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
