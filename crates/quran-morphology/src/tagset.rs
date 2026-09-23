//! Phase 2 — unified morphological tagset (`tagset`, ADR-0215).
//!
//! Every analysis row carries BOTH the dataset-native tag (verbatim,
//! immutable) and the unified tag (via a versioned [`TagMapping`]). The
//! mapping table is itself a reviewed artifact; unmappable native tags map
//! to [`UnifiedTag::Unmapped`] with the native string preserved — never to
//! a guessed nearest tag. Linguist sign-off of the tagset and its mappings
//! is pending; nothing here claims scholarly authority.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Unified part-of-speech tags for cross-dataset queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnifiedTag {
    /// Verb.
    Verb,
    /// Noun.
    Noun,
    /// Particle.
    Particle,
    /// Adjective.
    Adjective,
    /// Pronoun.
    Pronoun,
    /// Adverb.
    Adverb,
    /// Numeral.
    Numeral,
    /// Interjection.
    Interjection,
    /// Explicitly unmapped: the native tag is preserved, nothing guessed.
    Unmapped,
}

/// A versioned native-to-unified tag mapping for one dataset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagMapping {
    /// Mapping version (bumps ride dataset versions).
    pub version: String,
    table: HashMap<String, UnifiedTag>,
}

impl TagMapping {
    /// Build a mapping from native-tag / unified-tag pairs.
    pub fn new(
        version: impl Into<String>,
        pairs: impl IntoIterator<Item = (impl Into<String>, UnifiedTag)>,
    ) -> Self {
        Self {
            version: version.into(),
            table: pairs.into_iter().map(|(k, v)| (k.into(), v)).collect(),
        }
    }

    /// Mapping version.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Look up one native tag.
    pub fn get(&self, native: &str) -> Option<UnifiedTag> {
        self.table.get(native).copied()
    }

    /// Number of mapped native tags (coverage reporting input).
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Whether the mapping holds no entries.
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
}

/// Map a dataset-native tag to its unified tag, preserving the native tag
/// verbatim alongside.
///
/// Unmappable natives (including the empty string) yield
/// [`UnifiedTag::Unmapped`] with the native preserved — never a guess.
pub fn map_tag(native: &str, mapping: &TagMapping) -> (UnifiedTag, String) {
    let unified = mapping.get(native).unwrap_or(UnifiedTag::Unmapped);
    (unified, native.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mapping() -> TagMapping {
        TagMapping::new("test-tags-0.1", [("V_perf", UnifiedTag::Verb), ("N", UnifiedTag::Noun)])
    }

    #[test]
    fn known_tags_map() {
        let (tag, preserved) = map_tag("V_perf", &mapping());
        assert_eq!(tag, UnifiedTag::Verb);
        assert_eq!(preserved, "V_perf");
    }

    #[test]
    fn unknown_tags_are_unmapped_with_native_preserved() {
        let (tag, preserved) = map_tag("X_mystery", &mapping());
        assert_eq!(tag, UnifiedTag::Unmapped);
        assert_eq!(preserved, "X_mystery");
    }

    #[test]
    fn empty_native_is_unmapped_never_guessed() {
        let (tag, preserved) = map_tag("", &mapping());
        assert_eq!(tag, UnifiedTag::Unmapped);
        assert!(preserved.is_empty());
    }
}
