//! Isnad reader service: narrators, rijal sources, identity assertions, chain
//! path traversal (Phase 7).

use std::sync::Arc;

use async_trait::async_trait;
use domain::Timestamp;
use isnad_graph::{NarratorId, NarratorRecord, RijalSource};
use quran_graph::{
    Assertion, AssertionDecision, AssertionKind, GraphStore, PathsResult, ProvenanceLayer,
    QueryBudgets,
};
use sqlx::Row;
use storage::error::StorageError;
use storage_sqlite::SqliteDatabase;

use crate::isnad_graph_store::SqliteIsnadGraphStore;

#[derive(Debug, Clone)]
pub struct IsnadIdentityProposal {
    pub claim: serde_json::Value,
    pub evidence: serde_json::Value,
    pub source_location: String,
    pub provenance_layer: String,
    pub algorithm: Option<String>,
    pub algorithm_version: Option<String>,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum IsnadReaderError {
    #[error("storage failed: {0}")]
    Storage(#[from] StorageError),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("check violation: {0}")]
    CheckViolation(String),
}

impl storage::error::Diagnostic for IsnadReaderError {
    fn code(&self) -> storage::error::DiagnosticCode {
        match self {
            Self::Storage(_) => storage::error::DiagnosticCode::new("QAI-ISNAD", 1),
            Self::NotFound(_) => storage::error::DiagnosticCode::new("QAI-ISNAD", 2),
            Self::InvalidInput(_) => storage::error::DiagnosticCode::new("QAI-ISNAD", 3),
            Self::CheckViolation(_) => storage::error::DiagnosticCode::new("QAI-ISNAD", 4),
        }
    }
    fn summary(&self) -> String {
        self.to_string()
    }
    fn remedy(&self) -> Option<String> {
        Some("Check input and retry.".into())
    }
    fn next_command(&self) -> Option<String> {
        Some("qai isnad --help".into())
    }
}

#[async_trait]
pub trait IsnadReader: Send + Sync {
    async fn get_narrator(&self, id: &str) -> Result<NarratorRecord, IsnadReaderError>;
    async fn get_rijal_sources(
        &self,
        narrator_id: &str,
    ) -> Result<Vec<RijalSource>, IsnadReaderError>;
    async fn trace_paths(
        &self,
        src: &str,
        dst: &str,
        k: usize,
        budgets: QueryBudgets,
    ) -> Result<PathsResult, IsnadReaderError>;
    async fn propose_identity(
        &self,
        proposal: IsnadIdentityProposal,
    ) -> Result<Assertion, IsnadReaderError>;
    async fn accept_identity(&self, id: &str, reviewer: &str) -> Result<(), IsnadReaderError>;
    async fn reject_identity(&self, id: &str, reviewer: &str) -> Result<(), IsnadReaderError>;
    async fn dispute_identity(&self, id: &str, reviewer: &str) -> Result<(), IsnadReaderError>;
    async fn supersede_identity(
        &self,
        id: &str,
        new_proposal: IsnadIdentityProposal,
        reviewer: &str,
    ) -> Result<Assertion, IsnadReaderError>;
}

pub struct IsnadReaderService {
    db: Arc<SqliteDatabase>,
}

impl IsnadReaderService {
    pub fn new(db: Arc<SqliteDatabase>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl IsnadReader for IsnadReaderService {
    async fn get_narrator(&self, id: &str) -> Result<NarratorRecord, IsnadReaderError> {
        let row = sqlx::query("SELECT id, slug, primary_name, name_variants_json, metadata_json, created_at FROM narrators WHERE id = ? OR slug = ?")
            .bind(id)
            .bind(id.strip_prefix("narrator:").unwrap_or(id))
            .fetch_optional(self.db.read_pool())
            .await
            .map_err(|e| IsnadReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        let row = row.ok_or_else(|| IsnadReaderError::NotFound(id.to_string()))?;
        let slug: String = row.get("slug");
        let idstr: String = row.get("id");
        let stable = if idstr.starts_with("narrator:") {
            idstr.clone()
        } else {
            format!("narrator:{}", slug)
        };
        let nid: NarratorId =
            stable.parse().map_err(|e: String| IsnadReaderError::InvalidInput(e))?;
        let variants: String = row.get("name_variants_json");
        let metadata: String = row.get("metadata_json");
        Ok(NarratorRecord {
            id: nid,
            slug,
            primary_name: row.get("primary_name"),
            name_variants: serde_json::from_str(&variants).unwrap_or_default(),
            metadata: serde_json::from_str(&metadata).unwrap_or(serde_json::json!({})),
            created_at: row.get("created_at"),
        })
    }

    async fn get_rijal_sources(
        &self,
        narrator_id: &str,
    ) -> Result<Vec<RijalSource>, IsnadReaderError> {
        let slug = narrator_id.strip_prefix("narrator:").unwrap_or(narrator_id);
        let rows = sqlx::query("SELECT id, narrator_id, scholar_name, source_citation, grading_value, evidence_json, reviewer, decision, decided_at, supersedes_id, provenance_layer, confidence, created_at FROM narrator_rijal_sources WHERE narrator_id = ? OR narrator_id = ?")
            .bind(narrator_id)
            .bind(format!("narrator:{}", slug))
            .fetch_all(self.db.read_pool())
            .await
            .map_err(|e| IsnadReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        let mut out = Vec::new();
        for row in &rows {
            let nid: String = row.get("narrator_id");
            let nid_parsed: NarratorId =
                nid.parse().map_err(|e: String| IsnadReaderError::InvalidInput(e))?;
            let evidence: String = row.get("evidence_json");
            out.push(RijalSource {
                id: row.get("id"),
                narrator_id: nid_parsed,
                scholar_name: row.get("scholar_name"),
                source_citation: row.get("source_citation"),
                grading_value: row.get("grading_value"),
                evidence: serde_json::from_str(&evidence).unwrap_or(serde_json::json!({})),
                reviewer: row.get("reviewer"),
                decision: row.get("decision"),
                decided_at: row.get("decided_at"),
                supersedes_id: row.get("supersedes_id"),
                provenance_layer: row.get("provenance_layer"),
                confidence: row.get("confidence"),
                created_at: row.get("created_at"),
            });
        }
        Ok(out)
    }

    async fn trace_paths(
        &self,
        src: &str,
        dst: &str,
        k: usize,
        mut budgets: QueryBudgets,
    ) -> Result<PathsResult, IsnadReaderError> {
        if k > 0 {
            budgets.max_paths = k;
        }
        budgets.check().map_err(|e| IsnadReaderError::InvalidInput(e.to_string()))?;
        let store = SqliteIsnadGraphStore::open(&self.db).await.map_err(|e| {
            IsnadReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
        })?;
        // need mutable for bounded_paths? method takes &self? bounded_paths takes &self in trait? GraphStore::bounded_paths(&self,...). ok.
        let authz = quran_graph::AuthzScope::all_visible();
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let skipped = store.skipped_edges();
        let mut res = store
            .bounded_paths(src, dst, budgets.max_hops, &budgets, &cancel, &authz)
            .map_err(|e| {
                IsnadReaderError::Storage(StorageError::ConstraintViolation {
                    message: e.to_string(),
                })
            })?;
        // CR-01: edges whose endpoint is not a loaded narrator node are
        // skipped at load; report that rather than returning a silently
        // partial "complete" trace.
        if skipped > 0 && !res.truncated {
            res.incomplete_reason = Some(format!(
                "{skipped} isnad edge(s) skipped: endpoint not in the narrator projection"
            ));
        }
        Ok(res)
    }

    async fn propose_identity(
        &self,
        proposal: IsnadIdentityProposal,
    ) -> Result<Assertion, IsnadReaderError> {
        let id = format!("nia:{}", uuid::Uuid::new_v4());
        let now = chrono_now();
        let layer = validate_proposal(&proposal)?;
        let confidence = proposal.confidence;
        Self::insert_identity_assertion(self.db.write_pool(), &id, &proposal, layer, None, &now)
            .await?;
        Ok(Assertion {
            id,
            kind: AssertionKind::NarratorIdentity,
            claim: proposal.claim,
            evidence: proposal.evidence,
            source_location: proposal.source_location,
            reviewer: None,
            decision: AssertionDecision::Pending,
            decided_at: None,
            supersedes_id: None,
            layer: if layer == "D" { ProvenanceLayer::D } else { ProvenanceLayer::B },
            algorithm: proposal.algorithm,
            algorithm_version: proposal.algorithm_version,
            confidence,
            created_at: now,
        })
    }

    async fn accept_identity(&self, id: &str, reviewer: &str) -> Result<(), IsnadReaderError> {
        self.decide(id, reviewer, "accepted").await
    }
    async fn reject_identity(&self, id: &str, reviewer: &str) -> Result<(), IsnadReaderError> {
        self.decide(id, reviewer, "rejected").await
    }
    async fn dispute_identity(&self, id: &str, reviewer: &str) -> Result<(), IsnadReaderError> {
        self.decide(id, reviewer, "disputed").await
    }
    async fn supersede_identity(
        &self,
        id: &str,
        new_proposal: IsnadIdentityProposal,
        reviewer: &str,
    ) -> Result<Assertion, IsnadReaderError> {
        if reviewer.trim().is_empty() {
            return Err(IsnadReaderError::InvalidInput("reviewer must not be empty".into()));
        }
        // Validate everything BEFORE the first write so a rejected proposal
        // cannot leave the predecessor tombstoned with no successor (T-07-04).
        let layer = validate_proposal(&new_proposal)?;
        let new_id = format!("nia:{}", uuid::Uuid::new_v4());
        let now = chrono_now();

        // One transaction: tombstone predecessor + create the chained
        // successor. Either both land or neither does.
        let mut tx = self.db.write_pool().begin().await.map_err(storage_err)?;
        let decided = sqlx::query(
            "UPDATE narrator_identity_assertions SET decision = 'superseded', reviewer = ?, decided_at = ? WHERE id = ? AND decision = 'pending'",
        )
        .bind(reviewer.trim())
        .bind(&now)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(storage_err)?;
        if decided.rows_affected() == 0 {
            let exists: Option<String> =
                sqlx::query_scalar("SELECT id FROM narrator_identity_assertions WHERE id = ?")
                    .bind(id)
                    .fetch_optional(&mut *tx)
                    .await
                    .unwrap_or(None);
            return Err(if exists.is_some() {
                IsnadReaderError::CheckViolation("already decided".into())
            } else {
                IsnadReaderError::NotFound(id.to_string())
            });
        }
        // supersedes_id is bound in the INSERT — never a follow-up UPDATE.
        Self::insert_identity_assertion(&mut *tx, &new_id, &new_proposal, layer, Some(id), &now)
            .await?;
        tx.commit().await.map_err(storage_err)?;

        let confidence = new_proposal.confidence;
        Ok(Assertion {
            id: new_id,
            kind: AssertionKind::NarratorIdentity,
            claim: new_proposal.claim,
            evidence: new_proposal.evidence,
            source_location: new_proposal.source_location,
            reviewer: None,
            decision: AssertionDecision::Pending,
            decided_at: None,
            supersedes_id: Some(id.to_string()),
            layer: if layer == "D" { ProvenanceLayer::D } else { ProvenanceLayer::B },
            algorithm: new_proposal.algorithm,
            algorithm_version: new_proposal.algorithm_version,
            confidence,
            created_at: now,
        })
    }
}

impl IsnadReaderService {
    /// Single INSERT for a narrator-identity assertion. `supersedes_id` is
    /// bound here (never a follow-up UPDATE) so a supersession is one write.
    async fn insert_identity_assertion(
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
        id: &str,
        proposal: &IsnadIdentityProposal,
        layer: &str,
        supersedes_id: Option<&str>,
        created_at: &str,
    ) -> Result<(), IsnadReaderError> {
        sqlx::query(
            r#"INSERT INTO narrator_identity_assertions
               (id, assertion_kind, claim_json, evidence_json, source_location, reviewer, decision, decided_at, supersedes_id, provenance_layer, algorithm, algorithm_version, confidence, created_at)
               VALUES (?, 'narrator_identity', ?, ?, ?, NULL, 'pending', NULL, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(id)
        .bind(serde_json::to_string(&proposal.claim).unwrap_or_default())
        .bind(serde_json::to_string(&proposal.evidence).unwrap_or_default())
        .bind(&proposal.source_location)
        .bind(supersedes_id)
        .bind(layer)
        .bind(&proposal.algorithm)
        .bind(&proposal.algorithm_version)
        .bind(proposal.confidence)
        .bind(created_at)
        .execute(exec)
        .await
        .map_err(storage_err)?;
        Ok(())
    }

    async fn decide(
        &self,
        id: &str,
        reviewer: &str,
        decision: &str,
    ) -> Result<(), IsnadReaderError> {
        if reviewer.trim().is_empty() {
            return Err(IsnadReaderError::InvalidInput("reviewer must not be empty".into()));
        }
        let now = chrono_now();
        let result = sqlx::query(
            "UPDATE narrator_identity_assertions SET decision = ?, reviewer = ?, decided_at = ? WHERE id = ? AND decision = 'pending'",
        )
        .bind(decision)
        .bind(reviewer.trim())
        .bind(&now)
        .bind(id)
        .execute(self.db.write_pool())
        .await
        .map_err(|e| IsnadReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        if result.rows_affected() == 0 {
            let exists: Option<String> =
                sqlx::query_scalar("SELECT id FROM narrator_identity_assertions WHERE id = ?")
                    .bind(id)
                    .fetch_optional(self.db.read_pool())
                    .await
                    .unwrap_or(None);
            if exists.is_some() {
                return Err(IsnadReaderError::CheckViolation("already decided".into()));
            }
            return Err(IsnadReaderError::NotFound(id.to_string()));
        }
        Ok(())
    }
}

fn chrono_now() -> String {
    Timestamp::now().to_string()
}

fn storage_err(e: sqlx::Error) -> IsnadReaderError {
    IsnadReaderError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
}

/// Effective provenance layer, defaulting anything that is not `D` to `B`.
fn layer_of(provenance_layer: &str) -> &'static str {
    if provenance_layer == "D" { "D" } else { "B" }
}

/// Shared pre-write validation for `propose` and `supersede`: confidence
/// domain (WR-06) plus the D-layer algorithm/version/confidence gate.
fn validate_proposal(proposal: &IsnadIdentityProposal) -> Result<&'static str, IsnadReaderError> {
    let layer = layer_of(&proposal.provenance_layer);
    validate_confidence(proposal.confidence)?;
    if layer == "D"
        && (proposal.algorithm.is_none()
            || proposal.algorithm_version.is_none()
            || proposal.confidence.is_none())
    {
        return Err(IsnadReaderError::InvalidInput(
            "D-layer requires algorithm, version, confidence".into(),
        ));
    }
    Ok(layer)
}

/// WR-06: confidence is attributed research metadata; reject values outside
/// the `0.0..=1.0` domain (and non-finite values) before they reach the DB,
/// whose CHECK only enforces NOT NULL.
fn validate_confidence(confidence: Option<f64>) -> Result<(), IsnadReaderError> {
    if let Some(c) = confidence
        && !(c.is_finite() && (0.0..=1.0).contains(&c))
    {
        return Err(IsnadReaderError::InvalidInput(
            "confidence must be a finite value within 0.0..=1.0".into(),
        ));
    }
    Ok(())
}
