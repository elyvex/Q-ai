//! RAG project configuration (TOML).

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingCfg {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_dim")]
    pub dim: usize,
}

fn default_provider() -> String {
    "local-hash".to_string()
}
fn default_dim() -> usize {
    128
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankerCfg {
    #[serde(default = "default_reranker")]
    pub name: String,
}

fn default_reranker() -> String {
    "score".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterCfg {
    #[serde(default = "default_router")]
    pub name: String,
}

fn default_router() -> String {
    "default".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCfg {
    pub path: String,
    #[serde(default = "default_glob")]
    pub glob: String,
}

fn default_glob() -> String {
    "**/*".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    pub project: ProjectMeta,
    #[serde(default)]
    pub embedding: EmbeddingCfg,
    #[serde(default)]
    pub reranker: RerankerCfg,
    #[serde(default)]
    pub router: RouterCfg,
    #[serde(default)]
    pub sources: Vec<SourceCfg>,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            project: ProjectMeta { name: "default".to_string() },
            embedding: EmbeddingCfg { provider: default_provider(), dim: default_dim() },
            reranker: RerankerCfg { name: default_reranker() },
            router: RouterCfg { name: default_router() },
            sources: Vec::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("toml parse: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("path traversal not allowed: {0}")]
    PathTraversal(String),
}

impl Default for EmbeddingCfg {
    fn default() -> Self { Self { provider: default_provider(), dim: default_dim() } }
}
impl Default for RerankerCfg {
    fn default() -> Self { Self { name: default_reranker() } }
}
impl Default for RouterCfg {
    fn default() -> Self { Self { name: default_router() } }
}
impl Default for SourceCfg {
    fn default() -> Self { Self { path: String::new(), glob: default_glob() } }
}

impl RagConfig {
    pub fn from_toml_str(s: &str) -> Result<Self, ConfigError> {
        let cfg: Self = toml::from_str(s)?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        for src in &self.sources {
            if src.path.contains("..") {
                return Err(ConfigError::PathTraversal(src.path.clone()));
            }
            for seg in src.path.split('/') {
                if seg == ".." {
                    return Err(ConfigError::PathTraversal(src.path.clone()));
                }
            }
        }
        Ok(())
    }

    pub fn to_toml_string(&self) -> String {
        toml::to_string(self).unwrap_or_default()
    }
}
