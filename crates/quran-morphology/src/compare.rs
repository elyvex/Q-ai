//! Phase 2 — analysis comparison (`compare`, ADR-0209).
//!
//! [`compare`] returns per-field [`FieldVerdict`]s over two stated
//! analyses and never synthesizes. **Compile-time note:** [`FieldVerdict`]
//! serializes to exactly `{"field", "verdict"}` — there is no
//! resolution/synthesis/winner field, and a test pins that field list so
//! one cannot be added silently.

use serde::{Deserialize, Serialize};

use crate::dataset::TokenAnalysis;

/// Per-field comparison verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Verdict {
    /// Both sides state the same value.
    Identical,
    /// Both sides state non-empty values that differ through a documented
    /// compatibility path (same native tag behind differing unified tags,
    /// or root/lemma equality after diacritic stripping).
    CompatibleVariant,
    /// Both sides state irreconcilable values.
    Conflicting,
    /// Exactly one side states a value.
    OnlyInOne,
}

// NOTE (ADR-0209, AC-P2-18): this struct must never gain a
// resolution/synthesis/winner field. The serde field-list test in
// `tests/family_policy.rs` fails the build if one appears.
/// One field's verdict between two analyses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldVerdict {
    /// Compared field name.
    pub field: String,
    /// The verdict for that field.
    pub verdict: Verdict,
}

/// Strip Arabic diacritics (tashkeel + superscript alef) for the
/// root/lemma compatibility path. Letters are never altered.
fn strip_diacritics(text: &str) -> String {
    text.chars().filter(|c| !matches!(*c, '\u{064B}'..='\u{0652}' | '\u{0670}')).collect()
}

fn classify_text(field: &str, left: &str, right: &str, compatible: bool) -> FieldVerdict {
    let verdict = if left == right {
        Verdict::Identical
    } else if left.is_empty() || right.is_empty() {
        Verdict::OnlyInOne
    } else if compatible {
        Verdict::CompatibleVariant
    } else {
        Verdict::Conflicting
    };
    FieldVerdict { field: field.to_string(), verdict }
}

/// Compare two analyses field by field.
///
/// Compared fields: `surface`, `lemma`, `root`, `stem`, `pos_native`,
/// `pos_unified`, `features`. Compatibility paths:
/// `pos_unified` values differing behind identical non-empty native tags,
/// and `root`/`lemma` values equal after diacritic stripping.
pub fn compare(left: &TokenAnalysis, right: &TokenAnalysis) -> Vec<FieldVerdict> {
    let natives_equal = !left.pos_native.is_empty() && left.pos_native == right.pos_native;
    vec![
        classify_text("surface", &left.surface, &right.surface, false),
        classify_text(
            "lemma",
            &left.lemma,
            &right.lemma,
            strip_diacritics(&left.lemma) == strip_diacritics(&right.lemma),
        ),
        classify_text(
            "root",
            &left.root,
            &right.root,
            strip_diacritics(&left.root) == strip_diacritics(&right.root),
        ),
        classify_text("stem", &left.stem, &right.stem, false),
        classify_text("pos_native", &left.pos_native, &right.pos_native, false),
        classify_text("pos_unified", &left.pos_unified, &right.pos_unified, natives_equal),
        FieldVerdict {
            field: "features".to_string(),
            verdict: if left.features_json == right.features_json {
                Verdict::Identical
            } else if left.features_json.is_null() || right.features_json.is_null() {
                Verdict::OnlyInOne
            } else {
                Verdict::Conflicting
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> TokenAnalysis {
        TokenAnalysis {
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
        }
    }

    fn verdict_of(verdicts: &[FieldVerdict], field: &str) -> Verdict {
        verdicts.iter().find(|v| v.field == field).expect("field present").verdict
    }

    #[test]
    fn identical_rows_compare_identical() {
        let a = base();
        assert!(compare(&a, &a).iter().all(|v| v.verdict == Verdict::Identical));
    }

    #[test]
    fn empty_side_is_only_in_one() {
        let (mut a, b) = (base(), base());
        a.lemma.clear();
        assert_eq!(verdict_of(&compare(&a, &b), "lemma"), Verdict::OnlyInOne);
    }

    #[test]
    fn same_native_behind_different_unified_is_compatible_variant() {
        let (mut a, b) = (base(), base());
        a.pos_unified = "Verb".to_string();
        let mut c = b.clone();
        let _ = &b;
        c.pos_unified = "Particle".to_string();
        // natives identical on both sides
        assert_eq!(verdict_of(&compare(&a, &c), "pos_unified"), Verdict::CompatibleVariant);
    }

    #[test]
    fn unrelated_values_conflict() {
        let (mut a, b) = (base(), base());
        a.root = "كتب".to_string();
        let mut c = b.clone();
        let _ = &b;
        c.root = "علم".to_string();
        assert_eq!(verdict_of(&compare(&a, &c), "root"), Verdict::Conflicting);
    }
}
