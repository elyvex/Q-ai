//! Phase 2 — morphology validation rules MV-001…MV-018 (`validate`).
//!
//! Each rule is `{ id, severity, description }` plus a pure check function
//! over `(intermediate, inventory)`. Severity semantics:
//!
//! - `Fatal`: blocks import (the staging gate requires zero `Fatal`s).
//! - `Error`: must be fixed before activation but does not halt staging.
//! - `Warn`: advisory; recorded but never blocking.
//!
//! MV-018 is a marker rule: this crate holds no canonical store, so the
//! canonical-text agreement check is **deferred to the build-time MV-018
//! verifier** owned by the application layer. The rule therefore always
//! emits one `Warn` finding documenting the deferral; it never passes or
//! fails canonically here.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::dataset::{IntermediateMorphology, InventoryToken, TokenAnalysis};

/// Finding severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Blocks import.
    Fatal,
    /// Must be fixed before activation.
    Error,
    /// Advisory only.
    Warn,
}

/// One validation rule descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationRule {
    /// Stable rule id (`MV-001`…`MV-018`).
    pub id: &'static str,
    /// Rule severity.
    pub severity: Severity,
    /// Human-readable description of what the rule checks.
    pub description: &'static str,
}

/// The MV-001…MV-018 catalog, in id order.
pub const RULES: [ValidationRule; 18] = [
    ValidationRule {
        id: "MV-001",
        severity: Severity::Fatal,
        description: "dataset reference must name a non-empty slug and version",
    },
    ValidationRule {
        id: "MV-002",
        severity: Severity::Error,
        description: "token surface must be non-empty",
    },
    ValidationRule {
        id: "MV-003",
        severity: Severity::Fatal,
        description: "reference must use surah 1-114 and ayah >= 1",
    },
    ValidationRule {
        id: "MV-004",
        severity: Severity::Error,
        description: "token position must be >= 1 and within the inventory ayah length",
    },
    ValidationRule {
        id: "MV-005",
        severity: Severity::Error,
        description: "no duplicate (surah, ayah, position, analysis_index)",
    },
    ValidationRule {
        id: "MV-006",
        severity: Severity::Error,
        description: "lemma and root must be jointly present or jointly absent",
    },
    ValidationRule {
        id: "MV-007",
        severity: Severity::Warn,
        description: "native POS tag present without a unified mapping output",
    },
    ValidationRule {
        id: "MV-008",
        severity: Severity::Error,
        description: "rows claiming segmentation must carry non-empty segments",
    },
    ValidationRule {
        id: "MV-009",
        severity: Severity::Error,
        description: "confidence, when present, must lie in [0, 1]",
    },
    ValidationRule {
        id: "MV-010",
        severity: Severity::Fatal,
        description: "Layer-D rows must carry algorithm, algorithm_version, and confidence",
    },
    ValidationRule {
        id: "MV-011",
        severity: Severity::Fatal,
        description: "human_verified rows must name a reviewer",
    },
    ValidationRule {
        id: "MV-012",
        severity: Severity::Error,
        description: "root spelling must not be blank whitespace",
    },
    ValidationRule {
        id: "MV-013",
        severity: Severity::Error,
        description: "a unified tag must not claim a mapping with an empty native tag",
    },
    ValidationRule {
        id: "MV-014",
        severity: Severity::Error,
        description: "provenance_layer must be 'B' or 'D'",
    },
    ValidationRule {
        id: "MV-015",
        severity: Severity::Error,
        description: "status must be 'imported' or 'human_verified'",
    },
    ValidationRule {
        id: "MV-016",
        severity: Severity::Error,
        description: "features_json must be a JSON object",
    },
    ValidationRule {
        id: "MV-017",
        severity: Severity::Error,
        description: "every segment needs a valid kind and a non-empty surface",
    },
    ValidationRule {
        id: "MV-018",
        severity: Severity::Warn,
        description: "canonical agreement is deferred to the build-time MV-018 verifier",
    },
];

/// One rule finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// Rule id (`MV-001`…`MV-018`).
    pub rule_id: &'static str,
    /// Finding severity (copies the rule severity).
    pub severity: Severity,
    /// Affected reference (`surah:ayah:position#index`, or `""` for
    /// document-level findings).
    pub reference: String,
    /// Human-readable detail.
    pub detail: String,
}

fn finding(rule: &ValidationRule, analysis: Option<&TokenAnalysis>, detail: String) -> Finding {
    Finding {
        rule_id: rule.id,
        severity: rule.severity,
        reference: analysis.map(IntermediateMorphology::reference_of).unwrap_or_default(),
        detail,
    }
}

fn rule(id: &str) -> &'static ValidationRule {
    RULES.iter().find(|r| r.id == id).expect("MV rule id must exist in RULES")
}

/// MV-001: dataset reference must name a non-empty slug and version.
pub fn check_mv_001(intermediate: &IntermediateMorphology, _: &[InventoryToken]) -> Vec<Finding> {
    let r = rule("MV-001");
    if intermediate.dataset.slug.trim().is_empty() || intermediate.dataset.version.trim().is_empty()
    {
        vec![finding(
            r,
            None,
            format!(
                "unknown dataset reference '{}:{}': slug and version must both be non-empty",
                intermediate.dataset.slug, intermediate.dataset.version
            ),
        )]
    } else {
        Vec::new()
    }
}

/// MV-002: token surface must be non-empty.
pub fn check_mv_002(intermediate: &IntermediateMorphology, _: &[InventoryToken]) -> Vec<Finding> {
    let r = rule("MV-002");
    intermediate
        .analyses
        .iter()
        .filter(|a| a.surface.is_empty())
        .map(|a| finding(r, Some(a), "empty surface text".to_string()))
        .collect()
}

/// MV-003: reference must use surah 1–114 and ayah ≥ 1.
pub fn check_mv_003(intermediate: &IntermediateMorphology, _: &[InventoryToken]) -> Vec<Finding> {
    let r = rule("MV-003");
    intermediate
        .analyses
        .iter()
        .filter(|a| !(1..=114).contains(&a.surah) || a.ayah < 1)
        .map(|a| {
            finding(
                r,
                Some(a),
                format!(
                    "bad reference surah={} ayah={}: want surah 1-114, ayah >= 1",
                    a.surah, a.ayah
                ),
            )
        })
        .collect()
}

/// MV-004: token position must be ≥ 1 and within the inventory ayah length.
///
/// "Within" is evaluated only when the inventory contains the
/// `(surah, ayah)` pair, so unknown ayahs report MV-003/MV-004 exactly once
/// at most and never double-fire.
pub fn check_mv_004(
    intermediate: &IntermediateMorphology,
    inventory: &[InventoryToken],
) -> Vec<Finding> {
    let r = rule("MV-004");
    let mut max_by_ayah: HashMap<(u32, u32), u32> = HashMap::new();
    for token in inventory {
        let entry = max_by_ayah.entry((token.0, token.1)).or_insert(0);
        *entry = (*entry).max(token.2);
    }
    intermediate
        .analyses
        .iter()
        .filter(|a| {
            a.token_position < 1
                || max_by_ayah.get(&(a.surah, a.ayah)).is_some_and(|max| a.token_position > *max)
        })
        .map(|a| {
            finding(
                r,
                Some(a),
                format!(
                    "token position {} out of range for {}:{}",
                    a.token_position, a.surah, a.ayah
                ),
            )
        })
        .collect()
}

/// MV-005: no duplicate (surah, ayah, position, analysis_index).
pub fn check_mv_005(intermediate: &IntermediateMorphology, _: &[InventoryToken]) -> Vec<Finding> {
    let r = rule("MV-005");
    let mut seen: HashSet<(u32, u32, u32, u32)> = HashSet::new();
    intermediate
        .analyses
        .iter()
        .filter(|a| !seen.insert((a.surah, a.ayah, a.token_position, a.analysis_index)))
        .map(|a| finding(r, Some(a), "duplicate analysis index".to_string()))
        .collect()
}

/// MV-006: lemma and root must be jointly present or jointly absent.
pub fn check_mv_006(intermediate: &IntermediateMorphology, _: &[InventoryToken]) -> Vec<Finding> {
    let r = rule("MV-006");
    intermediate
        .analyses
        .iter()
        .filter(|a| a.lemma.is_empty() != a.root.is_empty())
        .map(|a| finding(r, Some(a), "empty lemma with non-empty root, or vice versa".to_string()))
        .collect()
}

/// MV-007: native POS tag present without a unified mapping output.
pub fn check_mv_007(intermediate: &IntermediateMorphology, _: &[InventoryToken]) -> Vec<Finding> {
    let r = rule("MV-007");
    intermediate
        .analyses
        .iter()
        .filter(|a| !a.pos_native.is_empty() && a.pos_unified.is_empty())
        .map(|a| {
            finding(
                r,
                Some(a),
                format!("native tag '{}' has no unified mapping output", a.pos_native),
            )
        })
        .collect()
}

/// MV-008: rows claiming segmentation must carry non-empty segments.
pub fn check_mv_008(intermediate: &IntermediateMorphology, _: &[InventoryToken]) -> Vec<Finding> {
    let r = rule("MV-008");
    intermediate
        .analyses
        .iter()
        .filter(|a| {
            a.features_json.get("segmented").is_some_and(|v| v.as_bool().unwrap_or(false))
                && a.segments.is_empty()
        })
        .map(|a| {
            finding(r, Some(a), "features claim segmentation but segments are empty".to_string())
        })
        .collect()
}

/// MV-009: confidence, when present, must lie in [0, 1].
pub fn check_mv_009(intermediate: &IntermediateMorphology, _: &[InventoryToken]) -> Vec<Finding> {
    let r = rule("MV-009");
    intermediate
        .analyses
        .iter()
        .filter(|a| a.confidence.is_some_and(|c| !(0.0..=1.0).contains(&c)))
        .map(|a| {
            finding(
                r,
                Some(a),
                format!("confidence {:?} outside [0, 1]", a.confidence.unwrap_or(f64::NAN)),
            )
        })
        .collect()
}

/// MV-010: Layer-D rows must carry algorithm, algorithm_version, confidence.
pub fn check_mv_010(intermediate: &IntermediateMorphology, _: &[InventoryToken]) -> Vec<Finding> {
    let r = rule("MV-010");
    intermediate
        .analyses
        .iter()
        .filter(|a| {
            a.provenance_layer == "D"
                && (a.algorithm.is_empty()
                    || a.algorithm_version.is_empty()
                    || a.confidence.is_none())
        })
        .map(|a| {
            finding(r, Some(a), "Layer-D row missing algorithm/version/confidence".to_string())
        })
        .collect()
}

/// MV-011: `human_verified` rows must name a reviewer.
pub fn check_mv_011(intermediate: &IntermediateMorphology, _: &[InventoryToken]) -> Vec<Finding> {
    let r = rule("MV-011");
    intermediate
        .analyses
        .iter()
        .filter(|a| a.status == "human_verified" && a.reviewer.trim().is_empty())
        .map(|a| finding(r, Some(a), "human_verified without reviewer".to_string()))
        .collect()
}

/// MV-012: root spelling must not be blank whitespace.
///
/// A fully empty root means "unanalyzed" and is legal (see MV-006);
/// whitespace-only roots are corrupt spellings.
pub fn check_mv_012(intermediate: &IntermediateMorphology, _: &[InventoryToken]) -> Vec<Finding> {
    let r = rule("MV-012");
    intermediate
        .analyses
        .iter()
        .filter(|a| !a.root.is_empty() && a.root.trim().is_empty())
        .map(|a| finding(r, Some(a), "blank root spelling".to_string()))
        .collect()
}

/// MV-013: a unified tag must not claim a mapping with an empty native tag.
///
/// (`Unmapped` is the honest marker for unmappable natives and is exempt:
/// it claims no successful mapping.)
pub fn check_mv_013(intermediate: &IntermediateMorphology, _: &[InventoryToken]) -> Vec<Finding> {
    let r = rule("MV-013");
    intermediate
        .analyses
        .iter()
        .filter(|a| {
            !a.pos_unified.is_empty() && a.pos_unified != "Unmapped" && a.pos_native.is_empty()
        })
        .map(|a| {
            finding(
                r,
                Some(a),
                format!("unified tag '{}' present with empty native tag", a.pos_unified),
            )
        })
        .collect()
}

/// MV-014: `provenance_layer` must be `B` or `D` (mirrors the `0017` CHECK).
pub fn check_mv_014(intermediate: &IntermediateMorphology, _: &[InventoryToken]) -> Vec<Finding> {
    let r = rule("MV-014");
    intermediate
        .analyses
        .iter()
        .filter(|a| a.provenance_layer != "B" && a.provenance_layer != "D")
        .map(|a| {
            finding(
                r,
                Some(a),
                format!("unknown provenance_layer '{}': want 'B' or 'D'", a.provenance_layer),
            )
        })
        .collect()
}

/// MV-015: `status` must be `imported` or `human_verified` (mirrors `0017`).
pub fn check_mv_015(intermediate: &IntermediateMorphology, _: &[InventoryToken]) -> Vec<Finding> {
    let r = rule("MV-015");
    intermediate
        .analyses
        .iter()
        .filter(|a| a.status != "imported" && a.status != "human_verified")
        .map(|a| {
            finding(
                r,
                Some(a),
                format!("unknown status '{}': want 'imported' or 'human_verified'", a.status),
            )
        })
        .collect()
}

/// MV-016: `features_json` must be a JSON object.
pub fn check_mv_016(intermediate: &IntermediateMorphology, _: &[InventoryToken]) -> Vec<Finding> {
    let r = rule("MV-016");
    intermediate
        .analyses
        .iter()
        .filter(|a| !a.features_json.is_object())
        .map(|a| finding(r, Some(a), "features_json must be a JSON object".to_string()))
        .collect()
}

/// MV-017: every segment needs a valid kind and a non-empty surface.
pub fn check_mv_017(intermediate: &IntermediateMorphology, _: &[InventoryToken]) -> Vec<Finding> {
    let r = rule("MV-017");
    let mut out = Vec::new();
    for a in &intermediate.analyses {
        for (index, segment) in a.segments.iter().enumerate() {
            let kind_ok = matches!(segment.kind.as_str(), "prefix" | "stem" | "suffix");
            if !kind_ok || segment.surface.is_empty() {
                out.push(finding(
                    r,
                    Some(a),
                    format!(
                        "bad segment #{index}: kind '{}', empty surface: {}",
                        segment.kind,
                        segment.surface.is_empty()
                    ),
                ));
            }
        }
    }
    out
}

/// MV-018 (marker): canonical agreement is deferred to build-time.
///
/// This crate holds no canonical store, so it cannot verify canonical
/// agreement. It always emits one `Warn` documenting the deferral; the
/// application build-time verifier owns the real check.
pub fn check_mv_018(intermediate: &IntermediateMorphology, _: &[InventoryToken]) -> Vec<Finding> {
    let r = rule("MV-018");
    vec![Finding {
        rule_id: r.id,
        severity: r.severity,
        reference: String::new(),
        detail: format!(
            "deferred to build-time MV-018: {} analyse(s) require canonical agreement verification by the application verifier",
            intermediate.analyses.len()
        ),
    }]
}

/// Run all MV rules in id order and collect every finding.
pub fn validate(
    intermediate: &IntermediateMorphology,
    inventory: &[InventoryToken],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(check_mv_001(intermediate, inventory));
    findings.extend(check_mv_002(intermediate, inventory));
    findings.extend(check_mv_003(intermediate, inventory));
    findings.extend(check_mv_004(intermediate, inventory));
    findings.extend(check_mv_005(intermediate, inventory));
    findings.extend(check_mv_006(intermediate, inventory));
    findings.extend(check_mv_007(intermediate, inventory));
    findings.extend(check_mv_008(intermediate, inventory));
    findings.extend(check_mv_009(intermediate, inventory));
    findings.extend(check_mv_010(intermediate, inventory));
    findings.extend(check_mv_011(intermediate, inventory));
    findings.extend(check_mv_012(intermediate, inventory));
    findings.extend(check_mv_013(intermediate, inventory));
    findings.extend(check_mv_014(intermediate, inventory));
    findings.extend(check_mv_015(intermediate, inventory));
    findings.extend(check_mv_016(intermediate, inventory));
    findings.extend(check_mv_017(intermediate, inventory));
    findings.extend(check_mv_018(intermediate, inventory));
    findings
}

/// Whether any finding is `Fatal` (blocks import).
pub fn has_fatal(findings: &[Finding]) -> bool {
    findings.iter().any(|f| f.severity == Severity::Fatal)
}

/// Whether any finding is `Fatal` or `Error` (blocks activation).
pub fn has_blocking(findings: &[Finding]) -> bool {
    findings.iter().any(|f| f.severity == Severity::Fatal || f.severity == Severity::Error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::DatasetRef;

    fn clean_doc() -> IntermediateMorphology {
        IntermediateMorphology {
            dataset: DatasetRef::new("synthetic-morph-test", "0.1.0"),
            edition_id: String::new(),
            synthetic_test_only: true,
            analyses: vec![TokenAnalysis {
                surah: 2,
                ayah: 1,
                token_position: 1,
                analysis_index: 0,
                surface: "كَتَبَ".to_string(),
                lemma: "كَتَبَ".to_string(),
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
            }],
        }
    }

    fn inventory() -> Vec<InventoryToken> {
        vec![(2, 1, 1, "كَتَبَ".to_string()), (2, 1, 2, "الْعِلْمُ".to_string())]
    }

    #[test]
    fn catalog_holds_exactly_18_rules_in_order() {
        assert_eq!(RULES.len(), 18);
        for (index, r) in RULES.iter().enumerate() {
            assert_eq!(r.id, format!("MV-{:03}", index + 1));
        }
    }

    #[test]
    fn clean_document_has_no_fatal_or_error() {
        let findings = validate(&clean_doc(), &inventory());
        assert!(!has_fatal(&findings));
        assert!(!has_blocking(&findings));
    }

    #[test]
    fn mv018_marker_always_fires_as_warn() {
        let findings = check_mv_018(&clean_doc(), &inventory());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "MV-018");
        assert_eq!(findings[0].severity, Severity::Warn);
        assert!(findings[0].detail.contains("deferred to build-time MV-018"));
    }

    #[test]
    fn fatal_rules_block_import() {
        let mut doc = clean_doc();
        doc.dataset.slug.clear();
        let findings = validate(&doc, &inventory());
        assert!(has_fatal(&findings));
        assert!(findings.iter().any(|f| f.rule_id == "MV-001"));
    }

    #[test]
    fn layer_d_without_provenance_is_fatal() {
        let mut doc = clean_doc();
        doc.analyses[0].provenance_layer = "D".to_string();
        let findings = validate(&doc, &inventory());
        assert!(findings.iter().any(|f| f.rule_id == "MV-010"));
        assert!(has_fatal(&findings));
    }

    #[test]
    fn mv004_ignores_ayahs_absent_from_inventory() {
        // Surah 115 is MV-003's problem; MV-004 must stay silent so each
        // adversarial fixture reports exactly one non-marker rule.
        let mut doc = clean_doc();
        doc.analyses[0].surah = 115;
        let findings = check_mv_004(&doc, &inventory());
        assert!(findings.is_empty());
        assert_eq!(check_mv_003(&doc, &inventory()).len(), 1);
    }
}
