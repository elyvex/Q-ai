//! Side-by-side grading disagreement assembly.
//!
//! # application::disagreement

use std::sync::Arc;

use crate::hadith_reader::HadithReader;
use async_trait::async_trait;
use hadith_core::{GradingRecord, HadithId, ProvenanceLayer};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use storage::error::StorageError;

pub type GradeLabel = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GradingView {
    pub scholar_name: String,
    pub grading_value: String,
    pub source_citation: String,
    pub confidence: Option<f64>,
    pub label: GradeLabel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisagreementView {
    pub hadith_id: HadithId,
    pub gradings: Vec<GradingView>,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum DisagreementError {
    #[error("storage failed: {0}")]
    Storage(#[from] StorageError),
    #[error("source record not found: {0}")]
    SourceNotFound(String),
    #[error("invalid id: {0}")]
    InvalidId(String),
}

impl storage::error::Diagnostic for DisagreementError {
    fn code(&self) -> storage::error::DiagnosticCode {
        match self {
            Self::Storage(_) => storage::error::DiagnosticCode::new("QAI-DIS", 1),
            Self::SourceNotFound(_) => storage::error::DiagnosticCode::new("QAI-DIS", 2),
            Self::InvalidId(_) => storage::error::DiagnosticCode::new("QAI-DIS", 3),
        }
    }
    fn summary(&self) -> String {
        self.to_string()
    }
    fn remedy(&self) -> Option<String> {
        Some("Check the hadith id and database state.".to_string())
    }
    fn next_command(&self) -> Option<String> {
        Some("qai hadith --help".to_string())
    }
}

#[async_trait]
pub trait DisagreementSource: Send + Sync {
    async fn fetch_gradings(
        &self,
        hadith_id: &HadithId,
    ) -> Result<Vec<GradingRecord>, DisagreementError>;
}

pub struct DisagreementService {
    source: Arc<dyn DisagreementSource>,
}

impl DisagreementService {
    pub fn new<S>(source: Arc<S>) -> Self
    where
        S: DisagreementSource + 'static,
    {
        Self { source }
    }

    pub async fn assemble<H>(&self, hadith_id: H) -> Result<DisagreementView, DisagreementError>
    where
        H: AsHadithId,
    {
        let hadith_id = hadith_id.into_id();
        let gradings = self.source.fetch_gradings(&hadith_id).await?;
        let mut views: Vec<GradingView> = gradings
            .into_iter()
            .map(|g| {
                let label = if g.provenance_layer == ProvenanceLayer::D || g.confidence.is_some() {
                    "uncertain".to_string()
                } else {
                    "confirmed".to_string()
                };
                GradingView {
                    scholar_name: g.scholar_name,
                    grading_value: g.grading_value,
                    source_citation: g.source_citation,
                    confidence: g.confidence,
                    label,
                }
            })
            .collect();
        views.sort_by(|a, b| a.scholar_name.cmp(&b.scholar_name));
        Ok(DisagreementView { hadith_id, gradings: views })
    }
}

pub trait AsHadithId {
    fn into_id(self) -> HadithId;
}
impl AsHadithId for HadithId {
    fn into_id(self) -> HadithId {
        self
    }
}
impl AsHadithId for &HadithId {
    fn into_id(self) -> HadithId {
        *self
    }
}
impl AsHadithId for String {
    fn into_id(self) -> HadithId {
        self.parse().unwrap_or_default()
    }
}
impl AsHadithId for &str {
    fn into_id(self) -> HadithId {
        self.parse().unwrap_or_default()
    }
}

#[async_trait]
impl DisagreementSource for storage_sqlite::SqliteDatabase {
    async fn fetch_gradings(
        &self,
        hadith_id: &HadithId,
    ) -> Result<Vec<GradingRecord>, DisagreementError> {
        let rows = sqlx::query(
            r#"SELECT id, hadith_id, scholar_name, grading_value, source_citation, evidence_json,
                      reviewer, decision, decided_at, supersedes_id, provenance_layer, confidence, created_at
               FROM hadith_gradings WHERE hadith_id = ?"#,
        )
        .bind(hadith_id.to_string())
        .fetch_all(self.read_pool())
        .await
        .map_err(|e| DisagreementError::Storage(StorageError::ConstraintViolation { message: e.to_string() }))?;
        let mut out = Vec::with_capacity(rows.len());
        for row in &rows {
            let decided_at: Option<String> = row.get(8);
            let supersedes_id: Option<String> = row.get(9);
            let provenance: String = row.get(10);
            let confidence: Option<f64> = row.get(11);
            let created_at: String = row.get(12);
            out.push(GradingRecord {
                id: row
                    .get::<String, _>(0)
                    .parse()
                    .map_err(|_| DisagreementError::InvalidId(row.get::<String, _>(0)))?,
                hadith_id: row
                    .get::<String, _>(1)
                    .parse()
                    .map_err(|_| DisagreementError::InvalidId(row.get::<String, _>(1)))?,
                scholar_name: row.get(2),
                grading_value: row.get(3),
                source_citation: row.get(4),
                evidence_json: serde_json::from_str(&row.get::<String, _>(5))
                    .unwrap_or(serde_json::Value::Null),
                reviewer: row.get(6),
                decision: row
                    .get::<String, _>(7)
                    .parse()
                    .map_err(|e| DisagreementError::SourceNotFound(e))?,
                decided_at: match decided_at {
                    Some(s) => {
                        Some(s.parse().map_err(|_| DisagreementError::InvalidId(s.clone()))?)
                    }
                    None => None,
                },
                supersedes_id: supersedes_id
                    .as_deref()
                    .map(|s| s.parse())
                    .transpose()
                    .map_err(|e: String| DisagreementError::InvalidId(e))?,
                provenance_layer: provenance
                    .parse()
                    .map_err(|e: String| DisagreementError::SourceNotFound(e))?,
                confidence,
                created_at: created_at
                    .parse()
                    .map_err(|_| DisagreementError::InvalidId(created_at.clone()))?,
            });
        }
        Ok(out)
    }
}

#[async_trait]
impl DisagreementSource for Arc<storage_sqlite::SqliteDatabase> {
    async fn fetch_gradings(
        &self,
        hadith_id: &HadithId,
    ) -> Result<Vec<GradingRecord>, DisagreementError> {
        (**self).fetch_gradings(hadith_id).await
    }
}

#[async_trait]
impl DisagreementSource for crate::hadith_reader::HadithReaderService {
    async fn fetch_gradings(
        &self,
        hadith_id: &HadithId,
    ) -> Result<Vec<GradingRecord>, DisagreementError> {
        let gradings = self.get_gradings_for_hadith(hadith_id).await.map_err(|e| {
            DisagreementError::Storage(StorageError::ConstraintViolation { message: e.to_string() })
        })?;
        Ok(gradings)
    }
}

#[async_trait]
impl DisagreementSource for Arc<crate::hadith_reader::HadithReaderService> {
    async fn fetch_gradings(
        &self,
        hadith_id: &HadithId,
    ) -> Result<Vec<GradingRecord>, DisagreementError> {
        (**self).fetch_gradings(hadith_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn view_roundtrips_serde() {
        let view = DisagreementView {
            hadith_id: HadithId::new(),
            gradings: vec![GradingView {
                scholar_name: "Al-Albani".to_string(),
                grading_value: "sahih".to_string(),
                source_citation: "Silsilah".to_string(),
                confidence: None,
                label: "confirmed".to_string(),
            }],
        };
        let json = serde_json::to_string(&view).unwrap();
        let back: DisagreementView = serde_json::from_str(&json).unwrap();
        assert_eq!(back, view);
    }
}
