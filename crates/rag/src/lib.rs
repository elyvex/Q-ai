//! Multi-RAG platform core types.

pub mod config;
pub mod hybrid;
pub mod project;
pub mod tombstone;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub use config::{RagConfig, ConfigError, RagConfigError, ProjectMeta, EmbeddingCfg, RerankerCfg, RouterCfg, SourceCfg};
pub use project::ProjectDescriptor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceStatus {
    Active,
    Deactivated,
    Tombstoned,
}

impl SourceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Deactivated => "deactivated",
            Self::Tombstoned => "tombstoned",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "active" => Some(Self::Active),
            "deactivated" => Some(Self::Deactivated),
            "tombstoned" => Some(Self::Tombstoned),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RagProject {
    pub id: String,
    pub name: String,
    pub config: RagConfig,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct RagSource {
    pub id: String,
    pub project_id: String,
    pub path: String,
    pub glob: String,
    pub status: SourceStatus,
}

#[derive(Debug, Clone)]
pub struct RagDocument {
    pub id: String,
    pub project_id: String,
    pub source_id: String,
    pub path: String,
    pub title: Option<String>,
    pub text: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct RagHit {
    pub doc_id: String,
    pub score: f32,
    pub source: String,
    pub rank: usize,
}

#[derive(Debug, Clone)]
pub struct RouteDecision {
    pub router: String,
    pub tools: Vec<String>,
    pub sources: Vec<String>,
    pub scores: Vec<(String, f32)>,
    pub final_ranking: Vec<String>,
    pub checksum: String,
    pub candidates: usize,
    pub merged: usize,
    pub reranked: usize,
    pub excluded: usize,
}

impl RouteDecision {
    pub fn compute_checksum(router: &str, query: &str, ids: &[String]) -> String {
        let mut h = Sha256::new();
        h.update(router.as_bytes());
        h.update(query.as_bytes());
        for id in ids {
            h.update(id.as_bytes());
        }
        hex(&h.finalize())
    }
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

/// RagBackend trait is defined in tool-registry; re-export not needed.

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_status_roundtrip() {
        assert_eq!(SourceStatus::from_str("active"), Some(SourceStatus::Active));
    }
}
