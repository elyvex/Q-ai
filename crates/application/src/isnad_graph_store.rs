//! SQLite-backed [`GraphStore`](quran_graph::GraphStore) read-view for isnad
//! (Phase 7). Loads `narrators` + `isnad_edges` into a `MemGraphStore`
//! under projection family `isnad-narrator-v1`. Tombstoned edges
//! (rejected/superseded assertions, or label 'rejected') are excluded at
//! load time; disputed stays effective.

use std::sync::atomic::AtomicBool;

use quran_graph::{
    Assertion, AssertionDecision, AssertionKind, AuthzScope, BuildInspection, EdgeFilter,
    GraphEdge, GraphError, GraphNode, GraphStore, MemGraphStore, NodeKind, PathsResult,
    ProjectionManifest, ProjectionStatus, ProvenanceLayer, QueryBudgets, TraversalResult,
};
use storage_sqlite::SqliteDatabase;

pub struct SqliteIsnadGraphStore {
    projection_id: String,
    inner: MemGraphStore,
}

impl SqliteIsnadGraphStore {
    pub async fn open(db: &SqliteDatabase) -> Result<Self, GraphError> {
        use sqlx::Row as _;
        let pool = db.read_pool();
        let projection_id = "isnad-narrator-v1".to_string();
        let mut inner = MemGraphStore::new(projection_id.clone());

        // Load assertions from narrator_identity_assertions (effective only).
        let assertion_rows = sqlx::query(
            "SELECT id, assertion_kind, claim_json, evidence_json, source_location,
                    reviewer, decision, decided_at, supersedes_id, provenance_layer,
                    algorithm, algorithm_version, confidence, created_at
             FROM narrator_identity_assertions",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| GraphError::BuildFailed { stage: "open".into(), detail: e.to_string() })?;
        for row in &assertion_rows {
            let decision_str: String = row.get("decision");
            let decision = match decision_str.as_str() {
                "pending" => AssertionDecision::Pending,
                "accepted" => AssertionDecision::Accepted,
                "rejected" => AssertionDecision::Rejected,
                "superseded" => AssertionDecision::Superseded,
                "disputed" => AssertionDecision::Disputed,
                _ => AssertionDecision::Pending,
            };
            // tombstoned filter for assertions themselves: still insert for audit,
            // but MemGraphStore is_edge_visible handles effectiveness.
            let layer_str: String = row.get("provenance_layer");
            let layer = match layer_str.as_str() {
                "D" => ProvenanceLayer::D,
                _ => ProvenanceLayer::B,
            };
            let assertion = Assertion {
                id: row.get("id"),
                kind: AssertionKind::NarratorIdentity,
                claim: parse_json(row.get("claim_json")),
                evidence: parse_json(row.get("evidence_json")),
                source_location: row.get("source_location"),
                reviewer: row.get("reviewer"),
                decision,
                decided_at: row.get("decided_at"),
                supersedes_id: row.get("supersedes_id"),
                layer,
                algorithm: row.get("algorithm"),
                algorithm_version: row.get("algorithm_version"),
                confidence: row.get::<Option<f64>, _>("confidence"),
                created_at: row.get("created_at"),
            };
            inner.insert_assertion(assertion);
        }

        // Load narrators as nodes.
        let narrator_rows = sqlx::query(
            "SELECT id, slug, primary_name, name_variants_json, metadata_json FROM narrators",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| GraphError::BuildFailed { stage: "open".into(), detail: e.to_string() })?;
        let mut nodes = Vec::new();
        for row in &narrator_rows {
            let id: String = row.get("id");
            // stable id is narrator:{slug}; use id column (already stable)
            let slug: String = row.get("slug");
            let stable_id =
                if id.starts_with("narrator:") { id.clone() } else { format!("narrator:{}", slug) };
            let primary_name: String = row.get("primary_name");
            let variants: String = row.get("name_variants_json");
            let metadata: String = row.get("metadata_json");
            let mut attrs = serde_json::json!({
                "slug": slug,
                "primary_name": primary_name,
                "name_variants": serde_json::from_str::<serde_json::Value>(&variants).unwrap_or(serde_json::json!([])),
            });
            if let Ok(m) = serde_json::from_str::<serde_json::Value>(&metadata) {
                attrs["metadata"] = m;
            }
            nodes.push(GraphNode::new(stable_id, NodeKind::Narrator, attrs));
        }
        inner
            .stage_nodes(nodes)
            .map_err(|e| GraphError::BuildFailed { stage: "open".into(), detail: e.to_string() })?;

        // Load isnad_edges, filtering tombstoned.
        let edge_rows = sqlx::query(
            "SELECT id, src_stable_id, edge, dst_stable_id, assertion_id, provenance_layer, confidence, label, algorithm, algorithm_version, reviewer, decided_at, created_at FROM isnad_edges",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| GraphError::BuildFailed { stage: "open".into(), detail: e.to_string() })?;
        let mut edges = Vec::new();
        for row in &edge_rows {
            let label: String = row.get("label");
            if label == "rejected" {
                continue;
            }
            let assertion_id: Option<String> = row.get("assertion_id");
            // check assertion effectiveness
            if let Some(aid) = &assertion_id
                && let Some(a) = inner.get_assertion(aid)
                && !a.is_effective()
            {
                continue;
            }
            let edge_str: String = row.get("edge");
            let src: String = row.get("src_stable_id");
            let dst: String = row.get("dst_stable_id");
            let attrs = serde_json::json!({
                "label": label,
                "confidence": row.get::<Option<f64>,_>("confidence"),
                "provenance_layer": row.get::<String,_>("provenance_layer"),
            });
            let aid_clean = assertion_id.clone().filter(|s| !s.is_empty());
            let ge = GraphEdge { src, edge: edge_str, dst, assertion_id: aid_clean, attrs };
            edges.push(ge);
        }
        inner
            .stage_edges(edges)
            .map_err(|e| GraphError::BuildFailed { stage: "open".into(), detail: e.to_string() })?;

        Ok(Self { projection_id, inner })
    }

    pub fn manifest(&self) -> ProjectionManifest {
        ProjectionManifest {
            id: "isnad-narrator-v1".into(),
            projection_id: self.projection_id.clone(),
            builder_version: "isnad-narrator-v1".into(),
            edition_id: String::new(),
            corpus_generation: 0,
            dataset_versions: Default::default(),
            dependency_snapshot: Default::default(),
            status: ProjectionStatus::Active,
            manifest: serde_json::Value::Null,
            created_at: String::new(),
        }
    }

    pub fn get_assertion(&self, id: &str) -> Option<&Assertion> {
        self.inner.get_assertion(id)
    }
}

impl GraphStore for SqliteIsnadGraphStore {
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
        pattern: &quran_graph::pattern::Pattern,
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
        self.inner.inspect()
    }
    fn clear_staged(&mut self) {
        self.inner.clear_staged();
    }
    fn capabilities(&self) -> Vec<String> {
        self.inner.capabilities()
    }
}

fn parse_json(raw: String) -> serde_json::Value {
    if raw.trim().is_empty() {
        return serde_json::Value::Null;
    }
    serde_json::from_str(&raw).unwrap_or(serde_json::Value::Null)
}
