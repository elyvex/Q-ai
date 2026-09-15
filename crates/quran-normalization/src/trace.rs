//! Phase 2 — `NormalizationTrace` (I9).
//!
//! Every search result reports the exact ordered rule set applied. The trace
//! is therefore a **required** field of every match further down the stack:
//! [`NormalizationTrace::new`] rejects an empty profile label, mirroring how
//! Phase 1 gives `QuranQuotation` no constructor without provenance. M3 adds
//! the compile-time guard on `SearchHit`; this module provides the value type
//! and its invariants.

use serde::{Deserialize, Serialize};

use crate::error::NormalizationError;
use crate::rule::{RuleId, RuleKind, SemVer};

/// One applied rule inside a trace, in application order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleApplication {
    /// Which rule ran.
    pub rule: RuleId,
    /// The implementation version that ran.
    #[serde(with = "crate::semver_serde")]
    pub version: SemVer,
    /// Deterministic fold vs heuristic approximation.
    pub kind: RuleKind,
}

/// The exact ordered rule set that produced a derived result (I9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizationTrace {
    /// Profile label, e.g. `"L3.diacritics@1.0.0"` or `"adhoc:<sha12>"`.
    /// Never empty — enforced by [`Self::new`].
    pub profile: String,
    /// Rules in the order they ran.
    pub rules_applied: Vec<RuleApplication>,
    /// True when any applied rule is heuristic (N18–N22). UIs render the
    /// mandatory "matched using heuristic …" label from this flag.
    pub contains_heuristic_rules: bool,
}

impl NormalizationTrace {
    /// Build a trace, deriving the heuristic flag from the rule list.
    ///
    /// # Errors
    ///
    /// Returns [`NormalizationError::EmptyProfile`] when `profile` is blank:
    /// an unlabeled result cannot be verified against any rule set.
    pub fn new(
        profile: impl Into<String>,
        rules_applied: Vec<RuleApplication>,
    ) -> Result<Self, NormalizationError> {
        let profile = profile.into();
        if profile.trim().is_empty() {
            return Err(NormalizationError::EmptyProfile);
        }
        let contains_heuristic_rules = rules_applied.iter().any(|r| r.kind == RuleKind::Heuristic);
        Ok(Self { profile, rules_applied, contains_heuristic_rules })
    }

    /// Ordered rule ids, for snapshot assertions and display.
    #[must_use]
    pub fn rule_ids(&self) -> Vec<RuleId> {
        self.rules_applied.iter().map(|r| r.rule).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Diagnostic;

    fn app(rule: RuleId, kind: RuleKind) -> RuleApplication {
        RuleApplication { rule, version: SemVer::new(1, 0, 0), kind }
    }

    #[test]
    fn heuristic_flag_derives_from_kinds() {
        let det = NormalizationTrace::new(
            "L3.diacritics@1.0.0",
            vec![
                app(RuleId::N01, RuleKind::Deterministic),
                app(RuleId::N03, RuleKind::Deterministic),
            ],
        )
        .unwrap();
        assert!(!det.contains_heuristic_rules);
        assert_eq!(det.rule_ids(), vec![RuleId::N01, RuleId::N03]);

        let heu = NormalizationTrace::new(
            "L7.affix@1.0.0",
            vec![app(RuleId::N03, RuleKind::Deterministic), app(RuleId::N18, RuleKind::Heuristic)],
        )
        .unwrap();
        assert!(heu.contains_heuristic_rules);
    }

    #[test]
    fn empty_profile_is_rejected() {
        let err = NormalizationTrace::new("  ", vec![]).unwrap_err();
        assert_eq!(err.code(), crate::error::codes::EMPTY_PROFILE);
    }

    #[test]
    fn trace_round_trips_json() {
        let trace = NormalizationTrace::new(
            "L3.diacritics@1.0.0",
            vec![app(RuleId::N03, RuleKind::Deterministic)],
        )
        .unwrap();
        let json = serde_json::to_string(&trace).unwrap();
        assert!(json.contains("\"profile\":\"L3.diacritics@1.0.0\""), "{json}");
        assert!(json.contains("\"version\":\"1.0.0\""), "{json}");
        let back: NormalizationTrace = serde_json::from_str(&json).unwrap();
        assert_eq!(back, trace);
    }
}
