//! Isnad typed tools (Phase 7): IsnadBackend implementation.

use std::sync::Arc;

use async_trait::async_trait;
use domain::SemVer;
use serde_json::json;
use tool_registry::{IdentityDecisionParams, IdentityProposalParams, IsnadBackend, TraceParams};
use tools::{AnalysisSource, ToolError, ToolResult, reproducibility, research_checksum};

use crate::isnad_reader::{IsnadIdentityProposal, IsnadReader};
use storage::error::Diagnostic;

pub struct IsnadToolBackend {
    reader: Arc<dyn IsnadReader>,
}

impl IsnadToolBackend {
    pub fn new(reader: Arc<dyn IsnadReader>) -> Self {
        Self { reader }
    }
}

fn proposal_from_params(p: &IdentityProposalParams) -> IsnadIdentityProposal {
    IsnadIdentityProposal {
        claim: p.claim.clone(),
        evidence: p.evidence.clone(),
        source_location: p.source_location.clone(),
        provenance_layer: p.provenance_layer.clone(),
        algorithm: p.algorithm.clone(),
        algorithm_version: p.algorithm_version.clone(),
        confidence: p.confidence,
    }
}

#[async_trait]
impl IsnadBackend for IsnadToolBackend {
    async fn backend_get_narrator(
        &self,
        id: &str,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let started = std::time::Instant::now();
        let narrator = self.reader.get_narrator(id).await.map_err(|e| ToolError::Backend {
            code: e.code().to_string(),
            detail: e.to_string(),
        })?;
        let rijal = self.reader.get_rijal_sources(id).await.unwrap_or_default();
        let results = json!({ "narrator": narrator, "rijal_sources": rijal });
        let payload = results.clone();
        let query = json!({ "id": id });
        Ok(ToolResult {
            tool_name: "isnad.narrator".to_string(),
            tool_version: SemVer::new(1, 0, 0),
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: None,
            edition_version: None,
            canonical_references: vec![id.to_string()],
            analysis_sources: vec![AnalysisSource {
                kind: "isnad".to_string(),
                reference: id.to_string(),
            }],
            results,
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                "isnad.narrator",
                SemVer::new(1, 0, 0),
                &query,
                None,
                None,
                std::collections::BTreeMap::new(),
                0,
            ),
            research_checksum: research_checksum(
                "isnad.narrator",
                SemVer::new(1, 0, 0),
                &query,
                &json!({}),
                &payload,
            )
            .expect("checksum"),
        })
    }
    async fn backend_get_rijal(
        &self,
        narrator_id: &str,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let started = std::time::Instant::now();
        let rijal = self.reader.get_rijal_sources(narrator_id).await.map_err(|e| {
            ToolError::Backend { code: e.code().to_string(), detail: e.to_string() }
        })?;
        let results = json!({ "rijal_sources": rijal });
        let payload = results.clone();
        let query = json!({ "narrator_id": narrator_id });
        Ok(ToolResult {
            tool_name: "isnad.rijal".to_string(),
            tool_version: SemVer::new(1, 0, 0),
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: None,
            edition_version: None,
            canonical_references: vec![narrator_id.to_string()],
            analysis_sources: vec![AnalysisSource {
                kind: "isnad".to_string(),
                reference: narrator_id.to_string(),
            }],
            results,
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                "isnad.rijal",
                SemVer::new(1, 0, 0),
                &query,
                None,
                None,
                std::collections::BTreeMap::new(),
                0,
            ),
            research_checksum: research_checksum(
                "isnad.rijal",
                SemVer::new(1, 0, 0),
                &query,
                &json!({}),
                &payload,
            )
            .expect("checksum"),
        })
    }
    async fn backend_trace(
        &self,
        params: &TraceParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let started = std::time::Instant::now();
        let k = params.k.unwrap_or(10);
        let budgets = quran_graph::QueryBudgets::default();
        let res =
            self.reader.trace_paths(&params.from, &params.to, k, budgets).await.map_err(|e| {
                ToolError::Backend { code: e.code().to_string(), detail: e.to_string() }
            })?;
        let results = serde_json::to_value(&res).unwrap_or(serde_json::Value::Null);
        let payload = results.clone();
        let query = serde_json::to_value(params).unwrap_or(serde_json::Value::Null);
        Ok(ToolResult {
            tool_name: "isnad.trace".to_string(),
            tool_version: SemVer::new(1, 0, 0),
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: None,
            edition_version: None,
            canonical_references: vec![params.from.clone(), params.to.clone()],
            analysis_sources: vec![AnalysisSource {
                kind: "isnad".to_string(),
                reference: "trace".to_string(),
            }],
            results,
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                "isnad.trace",
                SemVer::new(1, 0, 0),
                &query,
                None,
                None,
                std::collections::BTreeMap::new(),
                0,
            ),
            research_checksum: research_checksum(
                "isnad.trace",
                SemVer::new(1, 0, 0),
                &query,
                &json!({}),
                &payload,
            )
            .expect("checksum"),
        })
    }
    async fn backend_propose_identity(
        &self,
        params: &IdentityProposalParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let started = std::time::Instant::now();
        let assertion =
            self.reader.propose_identity(proposal_from_params(params)).await.map_err(|e| {
                ToolError::Backend { code: e.code().to_string(), detail: e.to_string() }
            })?;
        let results = serde_json::to_value(&assertion).unwrap_or(serde_json::Value::Null);
        let payload = results.clone();
        let query = serde_json::to_value(params).unwrap_or(serde_json::Value::Null);
        Ok(ToolResult {
            tool_name: "isnad.identity_propose".to_string(),
            tool_version: SemVer::new(1, 0, 0),
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: None,
            edition_version: None,
            canonical_references: vec![],
            analysis_sources: vec![AnalysisSource {
                kind: "isnad".to_string(),
                reference: "identity".to_string(),
            }],
            results,
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                "isnad.identity_propose",
                SemVer::new(1, 0, 0),
                &query,
                None,
                None,
                std::collections::BTreeMap::new(),
                0,
            ),
            research_checksum: research_checksum(
                "isnad.identity_propose",
                SemVer::new(1, 0, 0),
                &query,
                &json!({}),
                &payload,
            )
            .expect("checksum"),
        })
    }
    async fn backend_accept_identity(
        &self,
        params: &IdentityDecisionParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        self.reader.accept_identity(&params.id, &params.reviewer).await.map_err(|e| {
            ToolError::Backend { code: e.code().to_string(), detail: e.to_string() }
        })?;
        self.make_decision_result("isnad.identity_accept", params)
    }
    async fn backend_reject_identity(
        &self,
        params: &IdentityDecisionParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        self.reader.reject_identity(&params.id, &params.reviewer).await.map_err(|e| {
            ToolError::Backend { code: e.code().to_string(), detail: e.to_string() }
        })?;
        self.make_decision_result("isnad.identity_reject", params)
    }
    async fn backend_dispute_identity(
        &self,
        params: &IdentityDecisionParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        self.reader.dispute_identity(&params.id, &params.reviewer).await.map_err(|e| {
            ToolError::Backend { code: e.code().to_string(), detail: e.to_string() }
        })?;
        self.make_decision_result("isnad.identity_dispute", params)
    }
    async fn backend_supersede_identity(
        &self,
        id: &str,
        new_proposal: &IdentityProposalParams,
        reviewer: &str,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let assertion = self
            .reader
            .supersede_identity(id, proposal_from_params(new_proposal), reviewer)
            .await
            .map_err(|e| ToolError::Backend {
                code: e.code().to_string(),
                detail: e.to_string(),
            })?;
        let results = serde_json::to_value(&assertion).unwrap_or(serde_json::Value::Null);
        let payload = results.clone();
        let query = json!({ "id": id, "reviewer": reviewer });
        let started = std::time::Instant::now();
        Ok(ToolResult {
            tool_name: "isnad.identity_supersede".to_string(),
            tool_version: SemVer::new(1, 0, 0),
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: None,
            edition_version: None,
            canonical_references: vec![],
            analysis_sources: vec![AnalysisSource {
                kind: "isnad".to_string(),
                reference: "identity".to_string(),
            }],
            results,
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                "isnad.identity_supersede",
                SemVer::new(1, 0, 0),
                &query,
                None,
                None,
                std::collections::BTreeMap::new(),
                0,
            ),
            research_checksum: research_checksum(
                "isnad.identity_supersede",
                SemVer::new(1, 0, 0),
                &query,
                &json!({}),
                &payload,
            )
            .expect("checksum"),
        })
    }
}

impl IsnadToolBackend {
    fn make_decision_result(
        &self,
        name: &str,
        params: &IdentityDecisionParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        let started = std::time::Instant::now();
        let results = json!({ "id": params.id, "decision": name });
        let payload = results.clone();
        let query = serde_json::to_value(params).unwrap_or(serde_json::Value::Null);
        Ok(ToolResult {
            tool_name: name.to_string(),
            tool_version: SemVer::new(1, 0, 0),
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: None,
            edition_version: None,
            canonical_references: vec![params.id.clone()],
            analysis_sources: vec![AnalysisSource {
                kind: "isnad".to_string(),
                reference: params.id.clone(),
            }],
            results,
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                name,
                SemVer::new(1, 0, 0),
                &query,
                None,
                None,
                std::collections::BTreeMap::new(),
                0,
            ),
            research_checksum: research_checksum(
                name,
                SemVer::new(1, 0, 0),
                &query,
                &json!({}),
                &payload,
            )
            .expect("checksum"),
        })
    }
}
