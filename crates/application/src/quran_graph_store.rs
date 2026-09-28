//! SQLite-backed [`GraphStore`](quran_graph::GraphStore) read-view (Phase 4, D4.1).
//!
//! [`SqliteGraphStore`] pins one `graph_projections` build row and serves the
//! port from a snapshot loaded through parameterized SQL. Traversal itself
//! delegates to the reference expansion logic, so SQLite-backed queries have
//! byte-identical budget, authorization, cancellation, and ordering semantics
//! to [`quran_graph::MemGraphStore`] by construction: pre-flight
//! [`quran_graph::QueryBudgets::check`], per-batch cancel, in-expansion
//! authz/tombstone filtering, stable-ID ordering, typed incompletes, and
//! `NodeNotFound` for unknown stable IDs.
//!
//! Why a snapshot instead of per-hop SQL: the port's query methods are
//! synchronous while SQL access is async, and blocking an executor inside a
//! query method is unsound. Loading the pinned projection once (single
//! projection row, so bounded by the same budgets that built it) keeps the
//! sync port shape unchanged for every caller. Batched frontier SQL stays an
//! allowed future optimization behind this same type.
//!
//! Durable staging lives in [`crate::quran_graph_build`], not here:
//! `stage_nodes`/`stage_edges` buffer into the snapshot (validating the same
//! rejection rules) without persisting. `clear_staged` drops adjacency and
//! keeps authority records, mirroring the reference backend.

use std::sync::atomic::AtomicBool;

use quran_graph::pattern::Pattern;
use quran_graph::{
    Assertion, AssertionDecision, AssertionKind, AuthzScope, BuildInspection, EdgeFilter,
    GraphEdge, GraphError, GraphNode, GraphStore, MemGraphStore, NodeKind, PathsResult,
    ProjectionManifest, ProjectionStatus, ProvenanceLayer, QueryBudgets, TraversalResult,
};
use storage_sqlite::SqliteDatabase;

/// SQLite-backed graph read-view over one pinned projection build row.
pub struct SqliteGraphStore {
    projection_id: String,
    projection_row_id: String,
    manifest: ProjectionManifest,
    inner: MemGraphStore,
}

impl SqliteGraphStore {
    /// Open the projection build row `projection_row_id`.
    ///
    /// Unknown row ids are [`GraphError::UnknownProjection`]; corrupt rows
    /// (unknown kinds, decisions, layers, unparseable payloads, dangling
    /// edges, non-vocabulary predicates) are [`GraphError::BuildFailed`].
    /// Reads go through the read-only pool and never mutate.
    pub async fn open(db: &SqliteDatabase, projection_row_id: &str) -> Result<Self, GraphError> {
        use sqlx::Row as _;
        fn storage(stage: &str, error: sqlx::Error) -> GraphError {
            GraphError::BuildFailed { stage: stage.to_string(), detail: error.to_string() }
        }

        let pool = db.read_pool();
        let row = sqlx::query(
            "SELECT id, projection_id, builder_version, edition_id, corpus_generation,
                    dataset_versions_json, dependency_snapshot_json, status,
                    manifest_json, created_at
             FROM graph_projections WHERE id = ?",
        )
        .bind(projection_row_id)
        .fetch_optional(pool)
        .await
        .map_err(|error| storage("open", error))?
        .ok_or_else(|| GraphError::UnknownProjection {
            projection: projection_row_id.to_string(),
        })?;

        let projection_id: String = row.get("projection_id");
        let manifest = ProjectionManifest {
            id: row.get("id"),
            projection_id: projection_id.clone(),
            builder_version: row.get("builder_version"),
            edition_id: row.get("edition_id"),
            corpus_generation: row.get::<i64, _>("corpus_generation").max(0) as u64,
            dataset_versions: parse_map(row.get("dataset_versions_json"))?,
            dependency_snapshot: parse_map(row.get("dependency_snapshot_json"))?,
            status: parse_status(row.get("status"))?,
            manifest: parse_json(row.get("manifest_json"))?,
            created_at: row.get("created_at"),
        };

        let node_rows = sqlx::query(
            "SELECT stable_id, node_kind, attrs_json FROM graph_nodes
             WHERE projection_row_id = ? ORDER BY stable_id",
        )
        .bind(projection_row_id)
        .fetch_all(pool)
        .await
        .map_err(|error| storage("open", error))?;
        let mut nodes = Vec::with_capacity(node_rows.len());
        for node in &node_rows {
            nodes.push(GraphNode::new(
                node.get::<String, _>("stable_id"),
                parse_kind(node.get("node_kind"))?,
                parse_json(node.get("attrs_json"))?,
            ));
        }

        // Ordered by edge then destination for deterministic loads; the
        // delegated expansion re-sorts anyway.
        let edge_rows = sqlx::query(
            "SELECT src_stable_id, edge, dst_stable_id, assertion_id, attrs_json
             FROM graph_edges WHERE projection_row_id = ?
             ORDER BY edge, dst_stable_id",
        )
        .bind(projection_row_id)
        .fetch_all(pool)
        .await
        .map_err(|error| storage("open", error))?;
        let mut edges = Vec::with_capacity(edge_rows.len());
        for edge in &edge_rows {
            let mut record = GraphEdge {
                src: edge.get("src_stable_id"),
                edge: edge.get("edge"),
                dst: edge.get("dst_stable_id"),
                assertion_id: edge.get("assertion_id"),
                attrs: parse_json(edge.get("attrs_json"))?,
            };
            if record.assertion_id.as_deref().is_some_and(str::is_empty) {
                record.assertion_id = None;
            }
            edges.push(record);
        }

        let assertion_rows = sqlx::query(
            "SELECT id, assertion_kind, claim_json, evidence_json, source_location,
                    reviewer, decision, decided_at, supersedes_id, provenance_layer,
                    algorithm, algorithm_version, confidence, created_at
             FROM graph_assertions WHERE projection_row_id = ? ORDER BY id",
        )
        .bind(projection_row_id)
        .fetch_all(pool)
        .await
        .map_err(|error| storage("open", error))?;
        let mut assertions = Vec::with_capacity(assertion_rows.len());
        for record in &assertion_rows {
            assertions.push(Assertion {
                id: record.get("id"),
                kind: parse_assertion_kind(record.get("assertion_kind"))?,
                claim: parse_json(record.get("claim_json"))?,
                evidence: parse_json(record.get("evidence_json"))?,
                source_location: record.get("source_location"),
                reviewer: record.get("reviewer"),
                decision: parse_decision(record.get("decision"))?,
                decided_at: record.get("decided_at"),
                supersedes_id: record.get("supersedes_id"),
                layer: parse_layer(record.get("provenance_layer"))?,
                algorithm: record.get("algorithm"),
                algorithm_version: record.get("algorithm_version"),
                confidence: record.get::<Option<f64>, _>("confidence"),
                created_at: record.get("created_at"),
            });
        }

        let mut inner = MemGraphStore::new(projection_id.clone());
        for assertion in assertions {
            inner.insert_assertion(assertion);
        }
        inner.stage_nodes(nodes).map_err(|error| GraphError::BuildFailed {
            stage: "open".to_string(),
            detail: format!("stored nodes failed staging validation: {error}"),
        })?;
        inner.stage_edges(edges).map_err(|error| GraphError::BuildFailed {
            stage: "open".to_string(),
            detail: format!("stored edges failed staging validation: {error}"),
        })?;

        Ok(Self {
            projection_id,
            projection_row_id: projection_row_id.to_string(),
            manifest,
            inner,
        })
    }

    /// Open the `active` build row of projection family `projection_id`.
    ///
    /// No active row is [`GraphError::UnknownProjection`], never an empty
    /// projection: absence of a build must not read as an empty graph.
    pub async fn open_active(db: &SqliteDatabase, projection_id: &str) -> Result<Self, GraphError> {
        use sqlx::Row as _;
        let pool = db.read_pool();
        let row = sqlx::query(
            "SELECT id FROM graph_projections
             WHERE projection_id = ? AND status = 'active'
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(projection_id)
        .fetch_optional(pool)
        .await
        .map_err(|error| GraphError::BuildFailed {
            stage: "open_active".to_string(),
            detail: error.to_string(),
        })?;
        match row {
            Some(record) => Self::open(db, &record.get::<String, _>("id")).await,
            None => Err(GraphError::UnknownProjection { projection: projection_id.to_string() }),
        }
    }

    /// The pinned build's manifest (identity, versions, generation stamp).
    pub fn manifest(&self) -> &ProjectionManifest {
        &self.manifest
    }

    /// The pinned `graph_projections` build row id.
    pub fn projection_row_id(&self) -> &str {
        &self.projection_row_id
    }

    /// Look up an authority record by ID (mirrors the reference backend).
    pub fn get_assertion(&self, id: &str) -> Option<&Assertion> {
        self.inner.get_assertion(id)
    }
}

impl GraphStore for SqliteGraphStore {
    fn resolve_node(
        &self,
        stable_id: &str,
        budgets: &QueryBudgets,
        cancel: &AtomicBool,
        authz: &AuthzScope,
    ) -> Result<Option<GraphNode>, GraphError> {
        self.inner.resolve_node(stable_id, budgets, cancel, authz)
    }

    fn neighbors(
        &self,
        stable_id: &str,
        filter: &EdgeFilter,
        budgets: &QueryBudgets,
        cancel: &AtomicBool,
        authz: &AuthzScope,
    ) -> Result<TraversalResult, GraphError> {
        self.inner.neighbors(stable_id, filter, budgets, cancel, authz)
    }

    fn bounded_paths(
        &self,
        src: &str,
        dst: &str,
        max_hops: usize,
        budgets: &QueryBudgets,
        cancel: &AtomicBool,
        authz: &AuthzScope,
    ) -> Result<PathsResult, GraphError> {
        self.inner.bounded_paths(src, dst, max_hops, budgets, cancel, authz)
    }

    fn subgraph(
        &self,
        seeds: &[String],
        budgets: &QueryBudgets,
        cancel: &AtomicBool,
        authz: &AuthzScope,
    ) -> Result<TraversalResult, GraphError> {
        self.inner.subgraph(seeds, budgets, cancel, authz)
    }

    fn pattern_query(
        &self,
        pattern: &Pattern,
        seeds: &[String],
        budgets: &QueryBudgets,
        cancel: &AtomicBool,
        authz: &AuthzScope,
    ) -> Result<TraversalResult, GraphError> {
        self.inner.pattern_query(pattern, seeds, budgets, cancel, authz)
    }

    fn stage_nodes(&mut self, nodes: Vec<GraphNode>) -> Result<(), GraphError> {
        self.inner.stage_nodes(nodes)
    }

    fn stage_edges(&mut self, edges: Vec<GraphEdge>) -> Result<(), GraphError> {
        self.inner.stage_edges(edges)
    }

    fn inspect(&self) -> BuildInspection {
        BuildInspection { projection_id: self.projection_id.clone(), ..self.inner.inspect() }
    }

    fn clear_staged(&mut self) {
        self.inner.clear_staged();
    }

    fn capabilities(&self) -> Vec<String> {
        self.inner.capabilities()
    }
}

fn build_failed(stage: &str, detail: String) -> GraphError {
    GraphError::BuildFailed { stage: stage.to_string(), detail }
}

fn parse_json(raw: String) -> Result<serde_json::Value, GraphError> {
    if raw.trim().is_empty() {
        return Ok(serde_json::Value::Null);
    }
    serde_json::from_str(&raw)
        .map_err(|error| build_failed("open", format!("unparseable JSON payload: {error}")))
}

fn parse_map(raw: String) -> Result<std::collections::BTreeMap<String, String>, GraphError> {
    if raw.trim().is_empty() {
        return Ok(std::collections::BTreeMap::new());
    }
    serde_json::from_str(&raw)
        .map_err(|error| build_failed("open", format!("unparseable JSON map: {error}")))
}

fn parse_kind(raw: String) -> Result<NodeKind, GraphError> {
    match raw.as_str() {
        "edition" => Ok(NodeKind::Edition),
        "surah" => Ok(NodeKind::Surah),
        "ayah" => Ok(NodeKind::Ayah),
        "token" => Ok(NodeKind::Token),
        "division" => Ok(NodeKind::Division),
        "root" => Ok(NodeKind::Root),
        "lemma" => Ok(NodeKind::Lemma),
        "concept" => Ok(NodeKind::Concept),
        "entity" => Ok(NodeKind::Entity),
        "annotation" => Ok(NodeKind::Annotation),
        other => Err(build_failed("open", format!("unknown node kind '{other}'"))),
    }
}

fn parse_assertion_kind(raw: String) -> Result<AssertionKind, GraphError> {
    match raw.as_str() {
        "annotation" => Ok(AssertionKind::Annotation),
        "concept_link" => Ok(AssertionKind::ConceptLink),
        "entity_link" => Ok(AssertionKind::EntityLink),
        "family_link" => Ok(AssertionKind::FamilyLink),
        "import" => Ok(AssertionKind::Import),
        other => Err(build_failed("open", format!("unknown assertion kind '{other}'"))),
    }
}

fn parse_decision(raw: String) -> Result<AssertionDecision, GraphError> {
    match raw.as_str() {
        "pending" => Ok(AssertionDecision::Pending),
        "accepted" => Ok(AssertionDecision::Accepted),
        "rejected" => Ok(AssertionDecision::Rejected),
        "superseded" => Ok(AssertionDecision::Superseded),
        "disputed" => Ok(AssertionDecision::Disputed),
        other => Err(build_failed("open", format!("unknown assertion decision '{other}'"))),
    }
}

fn parse_layer(raw: String) -> Result<ProvenanceLayer, GraphError> {
    match raw.as_str() {
        "B" => Ok(ProvenanceLayer::B),
        "D" => Ok(ProvenanceLayer::D),
        other => Err(build_failed("open", format!("unknown provenance layer '{other}'"))),
    }
}

fn parse_status(raw: String) -> Result<ProjectionStatus, GraphError> {
    match raw.as_str() {
        "building" => Ok(ProjectionStatus::Building),
        "active" => Ok(ProjectionStatus::Active),
        "superseded" => Ok(ProjectionStatus::Superseded),
        other => Err(build_failed("open", format!("unknown projection status '{other}'"))),
    }
}
