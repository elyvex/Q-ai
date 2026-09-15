//! Phase 2 — `NormalizationPipeline`: one shared instance for query and index.
//!
//! The query path and the index path must never disagree, so both normalize
//! through the same pipeline object built from a frozen [`Profile`] (or an
//! explicit adhoc rule list). The pipeline returns derived text **with** its
//! [`NormalizationTrace`]; there is no method that returns one without the
//! other.

use sha2::{Digest, Sha256};

use crate::error::NormalizationError;
use crate::profile::{Profile, ProfileId, ProfileRegistry};
use crate::rule::{NormalizationRule, NormalizedText, RuleId, SemVer};
use crate::rules::by_id;
use crate::trace::{NormalizationTrace, RuleApplication};

/// A frozen, reusable normalization pipeline: ordered rules + trace metadata.
pub struct NormalizationPipeline {
    profile_label: String,
    rules: Vec<PipelineRule>,
}

struct PipelineRule {
    id: RuleId,
    version: SemVer,
    kind: crate::rule::RuleKind,
    rule: Box<dyn NormalizationRule>,
}

impl std::fmt::Debug for NormalizationPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NormalizationPipeline")
            .field("profile_label", &self.profile_label)
            .field("rules", &self.rule_ids())
            .finish()
    }
}

impl NormalizationPipeline {
    /// Build a pipeline from a registered profile version.
    ///
    /// # Errors
    ///
    /// Propagates [`NormalizationError::UnknownProfile`] for unregistered
    /// `(id, version)` pairs, and [`NormalizationError::UnknownRule`] if a
    /// profile ever names a rule with no implementation (reserved N23/N24).
    pub fn for_profile(
        registry: &ProfileRegistry,
        id: ProfileId,
        version: SemVer,
    ) -> Result<Self, NormalizationError> {
        let profile: &Profile = registry.get(id, version)?;
        Self::from_rule_ids(&profile.trace_label(), &profile.rules)
    }

    /// Build an adhoc pipeline from an explicit rule list (never both a
    /// profile and a rule list, per §5.2).
    ///
    /// The trace label is `adhoc:<sha256[..12]>` over the joined rule ids, so
    /// identical ad-hoc sets always share a label and different sets never do.
    ///
    /// # Errors
    ///
    /// Returns [`NormalizationError::UnknownRule`] for ids with no
    /// implementation (reserved N23/N24).
    pub fn adhoc(rule_ids: &[RuleId]) -> Result<Self, NormalizationError> {
        let joined = rule_ids.iter().map(|id| id.as_str()).collect::<Vec<_>>().join(",");
        let mut hasher = Sha256::new();
        hasher.update(joined.as_bytes());
        let digest = hex12(&hasher.finalize());
        Self::from_rule_ids(&format!("adhoc:{digest}"), rule_ids)
    }

    fn from_rule_ids(label: &str, rule_ids: &[RuleId]) -> Result<Self, NormalizationError> {
        let mut rules = Vec::with_capacity(rule_ids.len());
        for id in rule_ids {
            let rule = by_id(*id)
                .ok_or_else(|| NormalizationError::UnknownRule { rule: id.to_string() })?;
            rules.push(PipelineRule { id: *id, version: rule.version(), kind: rule.kind(), rule });
        }
        Ok(Self { profile_label: label.to_string(), rules })
    }

    /// The trace label this pipeline stamps (`profile@version` or `adhoc:…`).
    #[must_use]
    pub fn profile_label(&self) -> &str {
        &self.profile_label
    }

    /// Ordered rule ids in this pipeline.
    #[must_use]
    pub fn rule_ids(&self) -> Vec<RuleId> {
        self.rules.iter().map(|r| r.id).collect()
    }

    /// Normalize text, returning derived text **and** its trace together.
    pub fn apply(&self, text: &str) -> (NormalizedText, NormalizationTrace) {
        let mut current = NormalizedText::from_plain(text);
        let mut applications = Vec::with_capacity(self.rules.len());
        for entry in &self.rules {
            current = entry.rule.apply(&current);
            applications.push(RuleApplication {
                rule: entry.id,
                version: entry.version,
                kind: entry.kind,
            });
        }
        let trace = NormalizationTrace::new(self.profile_label.clone(), applications)
            .expect("pipeline labels are never blank by construction");
        (current, trace)
    }
}

/// Lowercase hex of the digest's first 6 bytes (12 chars).
fn hex12(digest: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(12);
    for byte in digest.iter().take(6) {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0F) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Diagnostic;
    use crate::profile::ProfileRegistry;

    fn registry() -> ProfileRegistry {
        ProfileRegistry::new()
    }

    #[test]
    fn pipeline_matches_manual_chain() {
        let v = SemVer::new(1, 0, 0);
        let pipeline = NormalizationPipeline::for_profile(&registry(), ProfileId::L3, v).unwrap();
        assert_eq!(pipeline.profile_label(), "L3.diacritics@1.0.0");
        let (text, trace) = pipeline.apply("بِسْمِ ٱللَّهِ");
        assert_eq!(text.text(), "بسم ٱلله");
        assert_eq!(
            trace.rule_ids(),
            vec![
                RuleId::N01,
                RuleId::N11,
                RuleId::N16,
                RuleId::N04,
                RuleId::N14,
                RuleId::N03,
                RuleId::N05,
                RuleId::N02,
            ]
        );
        assert!(!trace.contains_heuristic_rules);
    }

    #[test]
    fn l7_pipeline_flags_heuristics() {
        let v = SemVer::new(1, 0, 0);
        let pipeline = NormalizationPipeline::for_profile(&registry(), ProfileId::L7, v).unwrap();
        // N19 strips the conjunction, N21 the pronoun; N18 ran before N19 so
        // the article (exposed only after و-removal) survives the single pass.
        let (text, trace) = pipeline.apply("والكتابه");
        assert_eq!(text.text(), "الكتاب");
        assert!(trace.contains_heuristic_rules);
        assert_eq!(trace.profile, "L7.affix@1.0.0");
    }

    #[test]
    fn adhoc_labels_are_stable_and_distinct() {
        let a = NormalizationPipeline::adhoc(&[RuleId::N01, RuleId::N03]).unwrap();
        let b = NormalizationPipeline::adhoc(&[RuleId::N01, RuleId::N03]).unwrap();
        let c = NormalizationPipeline::adhoc(&[RuleId::N03, RuleId::N01]).unwrap();
        assert_eq!(a.profile_label(), b.profile_label());
        assert_ne!(a.profile_label(), c.profile_label());
        assert!(a.profile_label().starts_with("adhoc:"));
        assert_eq!(a.profile_label().len(), "adhoc:".len() + 12);
        let (_, trace) = a.apply("بِسْمِ");
        assert_eq!(trace.profile, a.profile_label());
        assert_eq!(trace.rule_ids(), vec![RuleId::N01, RuleId::N03]);
    }

    #[test]
    fn unknown_profile_and_reserved_rules_error() {
        let err =
            NormalizationPipeline::for_profile(&registry(), ProfileId::L3, SemVer::new(9, 9, 9))
                .unwrap_err();
        assert_eq!(err.code(), crate::error::codes::UNKNOWN_PROFILE);
        let err = NormalizationPipeline::adhoc(&[RuleId::N23]).unwrap_err();
        assert_eq!(err.code(), crate::error::codes::UNKNOWN_RULE);
    }
}
