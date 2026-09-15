//! Phase 2 — `ar_*` tokenizers over the shared normalization pipeline
//! (P2-T31).
//!
//! FTS5 cannot host custom C tokenizers, so the `ar_*` family is implemented
//! in Rust **in front of** FTS5 (DEV-05 annex to ADR-0201): each tokenizer
//! pairs one indexed field with one [`NormalizationPipeline`], and both the
//! index path (documents) and the query path (terms) normalize through the
//! same instances. A query and a document can never disagree; the
//! 5,000-substring parity test (`tests/search/parity.rs`, P2-T32) locks the
//! wiring, not just the function.
//!
//! Field → profile mapping (plan §4.2):
//!
//! | Field | Profile |
//! |---|---|
//! | `text_exact` | `L0.exact` (identity) |
//! | `text_ws` | `L1.ws` |
//! | `text_marks` | `L2.marks` |
//! | `text_bare` | `L3.diacritics` (primary search field) |
//! | `text_hamza` | `L4.hamza` |
//! | `text_folded` | `L5.codepoints` (most permissive indexed) |
//! | `text_affix` | `L7.affix` (token-level only) |

use std::collections::BTreeMap;

use quran_normalization::{NormalizationPipeline, ProfileId, ProfileRegistry, RuleId, SemVer};

use crate::error::IndexError;
use crate::model::FieldId;

/// All indexed text fields in a stable order.
pub const INDEXED_FIELDS: [&str; 7] =
    ["text_exact", "text_ws", "text_marks", "text_bare", "text_hamza", "text_folded", "text_affix"];

/// Profile behind each indexed field.
#[must_use]
pub fn profile_for_field(field: &str) -> Option<ProfileId> {
    match field {
        "text_exact" => Some(ProfileId::L0),
        "text_ws" => Some(ProfileId::L1),
        "text_marks" => Some(ProfileId::L2),
        "text_bare" => Some(ProfileId::L3),
        "text_hamza" => Some(ProfileId::L4),
        "text_folded" => Some(ProfileId::L5),
        "text_affix" => Some(ProfileId::L7),
        _ => None,
    }
}

/// One `ar_*` tokenizer: a field name bound to its normalization pipeline.
///
/// Both paths call [`ArTokenizer::tokenize`]; there is deliberately no
/// separate query normalizer.
#[derive(Debug)]
pub struct ArTokenizer {
    field: FieldId,
    profile: ProfileId,
    pipeline: NormalizationPipeline,
}

impl ArTokenizer {
    /// Build the tokenizer for `field` from a profile registry.
    ///
    /// # Errors
    ///
    /// Returns [`IndexError::BuildFailed`] for unknown fields (a backend
    /// wiring bug, never user input) and propagates unknown-profile errors.
    pub fn for_field(
        registry: &ProfileRegistry,
        field: &str,
        version: SemVer,
    ) -> Result<Self, IndexError> {
        let profile = profile_for_field(field).ok_or_else(|| IndexError::BuildFailed {
            stage: "tokenizer".to_string(),
            detail: format!("unknown indexed field '{field}'"),
        })?;
        let pipeline =
            NormalizationPipeline::for_profile(registry, profile, version).map_err(|err| {
                IndexError::BuildFailed { stage: "tokenizer".to_string(), detail: err.to_string() }
            })?;
        Ok(Self { field: field.to_string(), profile, pipeline })
    }

    /// Indexed field this tokenizer serves.
    #[must_use]
    pub fn field(&self) -> &str {
        &self.field
    }

    /// Profile this tokenizer normalizes with.
    #[must_use]
    pub fn profile(&self) -> ProfileId {
        self.profile
    }

    /// Trace label stamped on indexed content (`profile@version`).
    #[must_use]
    pub fn trace_label(&self) -> &str {
        self.pipeline.profile_label()
    }

    /// Normalize text exactly as the index path does.
    #[must_use]
    pub fn tokenize(&self, text: &str) -> String {
        self.pipeline.apply(text).0.text().to_string()
    }
}

/// The full `ar_*` family, keyed by field, sharing one registry.
///
/// Construct once per build (and once per query server) and hand the same
/// instance to both paths.
#[derive(Debug)]
pub struct TokenizerFamily {
    tokenizers: BTreeMap<FieldId, ArTokenizer>,
    version: SemVer,
}

impl TokenizerFamily {
    /// Build every `ar_*` tokenizer at one ladder version.
    ///
    /// # Errors
    ///
    /// Propagates unknown-profile errors for the requested version.
    pub fn new(registry: &ProfileRegistry, version: SemVer) -> Result<Self, IndexError> {
        let mut tokenizers = BTreeMap::new();
        for field in INDEXED_FIELDS {
            let tokenizer = ArTokenizer::for_field(registry, field, version)?;
            tokenizers.insert(field.to_string(), tokenizer);
        }
        Ok(Self { tokenizers, version })
    }

    /// Ladder version backing the family.
    #[must_use]
    pub fn version(&self) -> SemVer {
        self.version
    }

    /// Tokenizer for one field, if it is an indexed text field.
    #[must_use]
    pub fn get(&self, field: &str) -> Option<&ArTokenizer> {
        self.tokenizers.get(field)
    }

    /// Normalize `text` for `field`, following the index path exactly.
    ///
    /// # Errors
    ///
    /// Returns [`IndexError::BuildFailed`] for non-indexed fields.
    pub fn tokenize(&self, field: &str, text: &str) -> Result<String, IndexError> {
        self.get(field).map(|tokenizer| tokenizer.tokenize(text)).ok_or_else(|| {
            IndexError::BuildFailed {
                stage: "tokenizer".to_string(),
                detail: format!("unknown indexed field '{field}'"),
            }
        })
    }

    /// Ordered rule ids backing `field` (for trace assembly upstream).
    #[must_use]
    pub fn rule_ids(&self, field: &str) -> Vec<RuleId> {
        self.get(field).map(|t| t.pipeline.rule_ids()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn family() -> TokenizerFamily {
        let registry = ProfileRegistry::new();
        TokenizerFamily::new(&registry, SemVer::new(1, 0, 0)).unwrap()
    }

    #[test]
    fn every_indexed_field_resolves() {
        let family = family();
        for field in INDEXED_FIELDS {
            assert!(family.get(field).is_some(), "{field}");
        }
        assert!(family.get("text_nope").is_none());
        assert!(family.tokenize("text_nope", "x").is_err());
    }

    #[test]
    fn bare_field_strips_diacritics_like_the_ladder() {
        let family = family();
        assert_eq!(family.tokenize("text_bare", "بِسْمِ").unwrap(), "بسم");
        assert_eq!(family.tokenize("text_exact", "بِسْمِ").unwrap(), "بِسْمِ");
        assert_eq!(family.tokenize("text_folded", "ٱلرَّحْمَٰنِ").unwrap(), "الرحمن");
    }

    #[test]
    fn trace_labels_pin_profile_versions() {
        let family = family();
        assert_eq!(family.get("text_bare").unwrap().trace_label(), "L3.diacritics@1.0.0");
        assert_eq!(family.get("text_affix").unwrap().trace_label(), "L7.affix@1.0.0");
    }
}
