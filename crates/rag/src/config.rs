//! RAG project configuration (TOML).

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_version() -> String {
    "1".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingCfg {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_dimension", alias = "dim")]
    pub dimension: usize,
}

fn default_provider() -> String {
    "local-hash".to_string()
}
fn default_dimension() -> usize {
    128
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankerCfg {
    #[serde(default = "default_reranker", alias = "name")]
    pub provider: String,
}

fn default_reranker() -> String {
    "score".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterCfg {
    #[serde(default = "default_router", alias = "provider")]
    pub name: String,
}

fn default_router() -> String {
    "hybrid-rrf".to_string()
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
            project: ProjectMeta { name: "default".to_string(), version: default_version() },
            embedding: EmbeddingCfg { provider: default_provider(), dimension: default_dimension() },
            reranker: RerankerCfg { provider: default_reranker() },
            router: RouterCfg { name: default_router() },
            sources: Vec::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("toml parse: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("{field}: {message}")]
    Validation { field: String, message: String },
    #[error("{field}: path traversal not allowed: {path}")]
    PathTraversal { field: String, path: String },
}

pub type RagConfigError = ConfigError;

impl Default for EmbeddingCfg {
    fn default() -> Self { Self { provider: default_provider(), dimension: default_dimension() } }
}
impl Default for RerankerCfg {
    fn default() -> Self { Self { provider: default_reranker() } }
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
        // project.name
        if self.project.name.trim().is_empty() {
            return Err(ConfigError::Validation {
                field: "project.name".to_string(),
                message: "must not be empty".to_string(),
            });
        }
        // embedding.provider
        if self.embedding.provider != "local-hash" {
            return Err(ConfigError::Validation {
                field: "embedding.provider".to_string(),
                message: format!("unknown provider '{}' (allowed: local-hash)", self.embedding.provider),
            });
        }
        if self.embedding.dimension == 0 || self.embedding.dimension > 4096 {
            return Err(ConfigError::Validation {
                field: "embedding.dimension".to_string(),
                message: format!("must be > 0 and <= 4096, got {}", self.embedding.dimension),
            });
        }
        // reranker.provider
        if self.reranker.provider != "score" {
            return Err(ConfigError::Validation {
                field: "reranker.provider".to_string(),
                message: format!("unknown provider '{}' (allowed: score)", self.reranker.provider),
            });
        }
        // router.name
        if self.router.name != "hybrid-rrf" && self.router.name != "default" {
            return Err(ConfigError::Validation {
                field: "router.name".to_string(),
                message: format!("unknown router '{}' (allowed: hybrid-rrf)", self.router.name),
            });
        }
        // sources
        for (i, src) in self.sources.iter().enumerate() {
            let field = format!("sources[{}].path", i);
            if src.path.trim().is_empty() {
                return Err(ConfigError::Validation {
                    field: field.clone(),
                    message: "must not be empty".to_string(),
                });
            }
            // relative check
            if std::path::Path::new(&src.path).is_absolute() {
                return Err(ConfigError::PathTraversal { field: field.clone(), path: src.path.clone() });
            }
            for seg in src.path.split('/') {
                if seg == ".." || src.path.contains("..") && seg == ".." {
                    // already
                }
            }
            // path traversal segments
            for seg in src.path.replace('\\', "/").split('/') {
                if seg == ".." {
                    return Err(ConfigError::PathTraversal { field: field.clone(), path: src.path.clone() });
                }
            }
            if src.path.contains("..") {
                // catch traversal even if not exact segment
                // but allow filenames with ..? plan says reject .. segments; contains check safer
                // However names like "a..b" would be rejected; acceptable
                // Actually only reject if contains "/../" or starts "../" or ends "/.."
                // Simpler: reject any ".." substring as plan original did
                return Err(ConfigError::PathTraversal { field: field.clone(), path: src.path.clone() });
            }
        }
        Ok(())
    }

    pub fn to_toml_string(&self) -> String {
        toml::to_string(self).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_config() {
        let s = r#"
[project]
name = "demo"
version = "1"

[embedding]
provider = "local-hash"
dimension = 128

[reranker]
provider = "score"

[router]
name = "hybrid-rrf"

[[sources]]
path = "./docs"
glob = "**/*.md"
"#;
        let cfg = RagConfig::from_toml_str(s).unwrap();
        assert_eq!(cfg.project.name, "demo");
        assert_eq!(cfg.embedding.dimension, 128);
    }

    #[test]
    fn empty_name_invalid() {
        let s = r#"
[project]
name = ""
"#;
        assert!(RagConfig::from_toml_str(s).is_err());
    }

    #[test]
    fn unknown_provider_invalid() {
        let s = r#"
[project]
name = "x"

[embedding]
provider = "openai"
"#;
        let err = RagConfig::from_toml_str(s).unwrap_err();
        assert!(err.to_string().contains("embedding.provider"));
    }

    #[test]
    fn path_traversal_invalid() {
        let s = r#"
[project]
name = "x"

[[sources]]
path = "../secret"
"#;
        let err = RagConfig::from_toml_str(s).unwrap_err();
        assert!(err.to_string().to_lowercase().contains("traversal") || err.to_string().contains("sources[0].path"));
    }

    #[test]
    fn dimension_bounds() {
        let s = r#"
[project]
name = "x"

[embedding]
dimension = 0
"#;
        assert!(RagConfig::from_toml_str(s).is_err());
        let s = r#"
[project]
name = "x"

[embedding]
dimension = 5000
"#;
        assert!(RagConfig::from_toml_str(s).is_err());
    }
}