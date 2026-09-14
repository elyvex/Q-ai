//! Registry of structure validators (Phase 1, P1-T18).
//!
//! # sources::registry
//!
//! Validators are looked up by name (e.g. `quran_edition_v1`). Registration is
//! explicit and fail-closed: an unregistered name resolves to nothing, so no
//! dataset can slip through validation by naming an unknown validator.

use std::collections::HashMap;
use std::sync::Arc;

use super::StructureValidator;

/// Named structure validators, keyed by [`StructureValidator::name`].
#[derive(Default)]
pub struct ValidatorRegistry {
    validators: HashMap<String, Arc<dyn StructureValidator>>,
}

impl ValidatorRegistry {
    /// An empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a validator under its own name, replacing any previous entry.
    pub fn register(&mut self, validator: Arc<dyn StructureValidator>) -> &mut Self {
        self.validators.insert(validator.name().to_string(), validator);
        self
    }

    /// Look up a validator by name.
    pub fn get(&self, name: &str) -> Option<&Arc<dyn StructureValidator>> {
        self.validators.get(name)
    }

    /// Whether a validator is registered under `name`.
    pub fn contains(&self, name: &str) -> bool {
        self.validators.contains_key(name)
    }

    /// The number of registered validators.
    pub fn len(&self) -> usize {
        self.validators.len()
    }

    /// Whether no validator is registered.
    pub fn is_empty(&self) -> bool {
        self.validators.is_empty()
    }

    /// Registered names in sorted order.
    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.validators.keys().cloned().collect();
        names.sort();
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ValidationError, ValidationReport};

    struct DenyAll;

    #[async_trait::async_trait]
    impl StructureValidator for DenyAll {
        async fn validate(
            &self,
            _version: &crate::SourceVersion,
        ) -> Result<ValidationReport, ValidationError> {
            Ok(ValidationReport {
                valid: false,
                validator_name: self.name().to_string(),
                errors: vec![],
                warnings: vec![],
            })
        }

        fn name(&self) -> &str {
            "test.deny_all"
        }
    }

    #[test]
    fn register_lookup_and_names() {
        let mut registry = ValidatorRegistry::new();
        assert!(registry.is_empty());
        assert!(registry.get("test.deny_all").is_none());
        registry.register(Arc::new(DenyAll));
        assert_eq!(registry.len(), 1);
        assert!(registry.contains("test.deny_all"));
        assert_eq!(registry.names(), ["test.deny_all".to_string()]);
    }
}
