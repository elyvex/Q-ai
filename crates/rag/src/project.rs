//! Project lifecycle helpers.

use crate::config::RagConfig;

/// A created project descriptor.
#[derive(Debug, Clone)]
pub struct ProjectDescriptor {
    pub id: String,
    pub name: String,
    pub config: RagConfig,
}

impl ProjectDescriptor {
    pub fn new(name: &str, config: RagConfig) -> Self {
        Self { id: uuid::Uuid::new_v4().to_string(), name: name.to_string(), config }
    }
}
