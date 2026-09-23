//! Phase 2 — word-family relations (`family`, ADR-0210).
//!
//! Relations are typed, explained, and attributed. `explanation` is
//! mandatory on every member; cross-dataset computational suggestions are
//! **opt-in only**, carry a confidence floor, always bear
//! [`SUGGESTION_LABEL`], and promote to scholar-verified status exclusively
//! through the [`ReviewPromotion`] queue flow (reviewer + timestamp +
//! evidence recorded).

use serde::{Deserialize, Serialize};

use crate::dataset::TokenAnalysis;
use crate::error::MorphologyError;

/// Word-family relation taxonomy (mirrors the `0017`
/// `word_family_relations.relation` domain).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FamilyRelation {
    /// Same surface form.
    SameForm,
    /// Same lemma.
    SameLemma,
    /// Same stem.
    SameStem,
    /// Same root.
    SameRoot,
    /// Derivational link.
    Derived,
    /// Inflectional link.
    Inflectional,
    /// Affixation link.
    Affix,
    /// Unreviewed computational suggestion (never scholarship).
    ComputationalSuggestion,
}

/// Snake-case relation name matching the `0017` CHECK domain.
pub fn relation_name(relation: FamilyRelation) -> &'static str {
    match relation {
        FamilyRelation::SameForm => "same_form",
        FamilyRelation::SameLemma => "same_lemma",
        FamilyRelation::SameStem => "same_stem",
        FamilyRelation::SameRoot => "same_root",
        FamilyRelation::Derived => "derived",
        FamilyRelation::Inflectional => "inflectional",
        FamilyRelation::Affix => "affix",
        FamilyRelation::ComputationalSuggestion => "computational_suggestion",
    }
}

/// One endpoint of a family relation.
///
/// `kind` is one of `root` / `lemma` / `token` (mirrors the `0017`
/// `from_kind` / `to_kind` domains).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FamilyMember {
    /// Member id (application key).
    pub id: String,
    /// Member kind (`root`, `lemma`, or `token`).
    pub kind: String,
    /// Mandatory human-readable explanation of the membership.
    pub explanation: String,
}

impl FamilyMember {
    /// Build a member; rejects empty explanations (and empty ids / unknown
    /// kinds) with [`MorphologyError::ReviewViolation`].
    pub fn new(
        id: impl Into<String>,
        kind: impl Into<String>,
        explanation: impl Into<String>,
    ) -> Result<Self, MorphologyError> {
        let (id, kind, explanation) = (id.into(), kind.into(), explanation.into());
        if id.trim().is_empty() {
            return Err(MorphologyError::ReviewViolation {
                detail: "family member id must be non-empty".to_string(),
            });
        }
        if !matches!(kind.as_str(), "root" | "lemma" | "token") {
            return Err(MorphologyError::ReviewViolation {
                detail: format!(
                    "family member kind '{kind}' unknown: want 'root', 'lemma', or 'token'"
                ),
            });
        }
        if explanation.trim().is_empty() {
            return Err(MorphologyError::ReviewViolation {
                detail: "family member explanation is mandatory and must be non-empty".to_string(),
            });
        }
        Ok(Self { id, kind, explanation })
    }
}

/// Build a human-readable diff of two analyses under a relation.
///
/// Shared non-empty fields are reported as shared; differing fields are
/// listed with both values, so a reader sees exactly what the relation
/// claims and what it does not.
pub fn explain_relation(
    left: &TokenAnalysis,
    right: &TokenAnalysis,
    relation: FamilyRelation,
) -> String {
    let left_ref = format!("{}:{}:{}", left.surah, left.ayah, left.token_position);
    let right_ref = format!("{}:{}:{}", right.surah, right.ayah, right.token_position);
    let fields = [
        ("surface", left.surface.as_str(), right.surface.as_str()),
        ("lemma", left.lemma.as_str(), right.lemma.as_str()),
        ("root", left.root.as_str(), right.root.as_str()),
        ("stem", left.stem.as_str(), right.stem.as_str()),
        ("pos_native", left.pos_native.as_str(), right.pos_native.as_str()),
        ("pos_unified", left.pos_unified.as_str(), right.pos_unified.as_str()),
    ];
    let mut shared = Vec::new();
    let mut differing = Vec::new();
    for (name, l, r) in fields {
        if l == r {
            if !l.is_empty() {
                shared.push(format!("{name} '{l}'"));
            }
        } else {
            differing.push(format!("{name} ('{l}' vs '{r}')"));
        }
    }
    let mut out = format!(
        "{} links {left_ref} ('{}') with {right_ref} ('{}')",
        relation_name(relation),
        left.surface,
        right.surface
    );
    if shared.is_empty() {
        out.push_str("; shared: (none stated)");
    } else {
        out.push_str(&format!("; shared: {}", shared.join(", ")));
    }
    if differing.is_empty() {
        out.push_str("; differing: (none — identical rows)");
    } else {
        out.push_str(&format!("; differing: {}", differing.join(", ")));
    }
    out
}

/// Mandatory label on every computational suggestion: it is not verified
/// scholarship.
pub const SUGGESTION_LABEL: &str = "not verified scholarship — computational suggestion";

/// Minimum confidence for a computational suggestion to be issuable.
pub const SUGGESTION_CONFIDENCE_FLOOR: f64 = 0.50;

/// An unreviewed computational family suggestion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComputationalSuggestion {
    /// Suggestion id (review-queue key).
    pub suggestion_id: String,
    /// Source member id.
    pub from_id: String,
    /// Target member id.
    pub to_id: String,
    /// Suggested relation.
    pub relation: FamilyRelation,
    /// Issuing confidence in [floor, 1].
    pub confidence: f64,
    /// Always [`SUGGESTION_LABEL`].
    pub label: String,
}

/// Issue a computational suggestion.
///
/// Requires explicit `opt_in` and a confidence within
/// `[SUGGESTION_CONFIDENCE_FLOOR, 1]`; the label is always attached.
/// Suggestions never auto-verify — see [`ReviewPromotion`].
pub fn suggest_computational(
    suggestion_id: impl Into<String>,
    from_id: impl Into<String>,
    to_id: impl Into<String>,
    confidence: f64,
    opt_in: bool,
) -> Result<ComputationalSuggestion, MorphologyError> {
    if !opt_in {
        return Err(MorphologyError::ReviewViolation {
            detail: "computational suggestions require explicit opt-in".to_string(),
        });
    }
    if !(SUGGESTION_CONFIDENCE_FLOOR..=1.0).contains(&confidence) {
        return Err(MorphologyError::ReviewViolation {
            detail: format!(
                "suggestion confidence {confidence} below floor {SUGGESTION_CONFIDENCE_FLOOR} or above 1"
            ),
        });
    }
    Ok(ComputationalSuggestion {
        suggestion_id: suggestion_id.into(),
        from_id: from_id.into(),
        to_id: to_id.into(),
        relation: FamilyRelation::ComputationalSuggestion,
        confidence,
        label: SUGGESTION_LABEL.to_string(),
    })
}

/// A review-queue promotion decision: the only path from `proposed` to
/// scholar-verified (mirrors the `0017` `review_queue` decided rows).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewPromotion {
    /// Suggestion being decided.
    pub suggestion_id: String,
    /// Reviewer id (mandatory).
    pub reviewer: String,
    /// Decision timestamp, RFC 3339 (mandatory).
    pub decided_at: String,
    /// Displayed evidence object (mandatory, must be non-null).
    pub evidence: serde_json::Value,
}

impl ReviewPromotion {
    /// Record a promotion decision; rejects missing reviewer, timestamp,
    /// or evidence with [`MorphologyError::ReviewViolation`].
    pub fn new(
        suggestion_id: impl Into<String>,
        reviewer: impl Into<String>,
        decided_at: impl Into<String>,
        evidence: serde_json::Value,
    ) -> Result<Self, MorphologyError> {
        let (suggestion_id, reviewer, decided_at) =
            (suggestion_id.into(), reviewer.into(), decided_at.into());
        if reviewer.trim().is_empty() {
            return Err(MorphologyError::ReviewViolation {
                detail: "review promotion requires a reviewer".to_string(),
            });
        }
        if decided_at.trim().is_empty() {
            return Err(MorphologyError::ReviewViolation {
                detail: "review promotion requires a decision timestamp".to_string(),
            });
        }
        if evidence.is_null() {
            return Err(MorphologyError::ReviewViolation {
                detail: "review promotion requires non-null evidence".to_string(),
            });
        }
        Ok(Self { suggestion_id, reviewer, decided_at, evidence })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(surah: u32, ayah: u32, position: u32, surface: &str, lemma: &str) -> TokenAnalysis {
        TokenAnalysis {
            surah,
            ayah,
            token_position: position,
            analysis_index: 0,
            surface: surface.to_string(),
            lemma: lemma.to_string(),
            root: "كتب".to_string(),
            stem: "كتب".to_string(),
            pos_unified: "Verb".to_string(),
            pos_native: "V_perf".to_string(),
            features_json: serde_json::json!({}),
            segments: Vec::new(),
            provenance_layer: "B".to_string(),
            algorithm: String::new(),
            algorithm_version: String::new(),
            confidence: None,
            reviewer: String::new(),
            status: "imported".to_string(),
        }
    }

    #[test]
    fn explanation_is_mandatory() {
        assert!(FamilyMember::new("m1", "token", "").is_err());
        assert!(FamilyMember::new("m1", "token", "  ").is_err());
        assert!(FamilyMember::new("m1", "token", "shares root كتب").is_ok());
    }

    #[test]
    fn member_kind_is_checked() {
        assert!(FamilyMember::new("m1", "ayah", "x").is_err());
        assert!(FamilyMember::new("m1", "root", "x").is_ok());
    }

    #[test]
    fn explain_relation_diffs_features() {
        let text = explain_relation(
            &token(2, 1, 1, "كَتَبَ", "كَتَبَ"),
            &token(2, 1, 2, "كِتَاب", "كِتَاب"),
            FamilyRelation::SameRoot,
        );
        assert!(text.contains("same_root"), "{text}");
        assert!(text.contains("shared"), "{text}");
        assert!(text.contains("differing"), "{text}");
        assert!(text.contains("lemma"), "{text}");
    }

    #[test]
    fn suggestions_require_opt_in_and_floor() {
        assert!(suggest_computational("s", "a", "b", 0.9, false).is_err());
        assert!(suggest_computational("s", "a", "b", 0.1, true).is_err());
        let ok = suggest_computational("s", "a", "b", 0.9, true).expect("issuable");
        assert_eq!(ok.label, SUGGESTION_LABEL);
        assert_eq!(ok.relation, FamilyRelation::ComputationalSuggestion);
    }

    #[test]
    fn promotion_requires_reviewer_timestamp_evidence() {
        assert!(ReviewPromotion::new("s", "", "t", serde_json::json!({})).is_err());
        assert!(ReviewPromotion::new("s", "r", "", serde_json::json!({})).is_err());
        assert!(ReviewPromotion::new("s", "r", "t", serde_json::Value::Null).is_err());
        assert!(
            ReviewPromotion::new(
                "s",
                "r",
                "2026-01-01T00:00:00Z",
                serde_json::json!({"note": "x"})
            )
            .is_ok()
        );
    }

    #[test]
    fn relation_names_match_schema_domain() {
        assert_eq!(relation_name(FamilyRelation::SameRoot), "same_root");
        assert_eq!(
            relation_name(FamilyRelation::ComputationalSuggestion),
            "computational_suggestion"
        );
    }
}
