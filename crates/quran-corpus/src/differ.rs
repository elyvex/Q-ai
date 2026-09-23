//! Edition differ: char-level `DifferenceReport` content (ADR-0109, P1-T27).
//!
//! # quran_corpus::differ
//!
//! Ayahs are aligned by `(surah, ayah)` identity; text that differs is
//! diffed at the character level with `similar`, reporting changed ranges in
//! new-text character offsets for reviewer usability. Metadata differences
//! (name, script, numbering) fold into a single boolean — reviewers read
//! changed ayahs, not field lists.

use domain::SemVer;
use serde::{Deserialize, Serialize};
use similar::{DiffTag, TextDiff};

/// The differ name recorded on difference reports.
pub const DIFFER_NAME: &str = "quran_edition";
/// The differ version recorded on difference reports.
pub const DIFFER_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Classification vocabulary version (ADR-0114 typed comparison, P1-T26 Tier-2).
pub const CLASSIFICATION_VOCABULARY: &str = "ADR-0114-v1";

/// Operand-kind-aware comparison taxonomy (ADR-0114 §Decision, P1-T26 Tier-2).
///
/// Every comparison declares its operand kinds. Comparing incompatible
/// representations requires an explicit opt-in naming the kind; the result is
/// never labeled "corruption".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonKind {
    /// Canonical edition vs the same edition at the same version.
    Integrity,
    /// Canonical edition vs the same edition at another version (ADR-0109).
    #[default]
    Version,
    /// Canonical edition vs an alternate edition / other riwayah.
    Readings,
    /// Canonical text vs a translation.
    Translation,
    /// Canonical text vs a reference corpus (QV-015).
    Reference,
    /// Text vs a checksum manifest.
    Checksum,
}

impl ComparisonKind {
    /// Stable string for reports and reproducibility payloads.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Integrity => "integrity",
            Self::Version => "version",
            Self::Readings => "readings",
            Self::Translation => "translation",
            Self::Reference => "reference",
            Self::Checksum => "checksum",
        }
    }
}

/// Closed difference classification (ADR-0114 §Decision.2, P1-T26 Tier-2).
///
/// A difference that cannot be classified at or below the level that produced
/// it is [`DifferenceClass::UnknownDifference`] — never silently forced into
/// a benign class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DifferenceClass {
    /// No difference.
    Same,
    /// Texts agree after whitespace normalization only.
    NormalizationOnly,
    /// Same reading, different orthographic carrier.
    OrthographicDifference,
    /// Different script representation.
    ScriptDifference,
    /// Different transmission of a recognized reading.
    RiwayahDifference,
    /// Different identified edition.
    EditionDifference,
    /// Same characters, different token boundaries.
    TokenizationDifference,
    /// Text vs translation rendering.
    TranslationDifference,
    /// Unclassifiable; stays visible, never a silent pass.
    #[default]
    UnknownDifference,
}

/// Operand record for reproducibility (ADR-0114: every difference report
/// records operand kinds + normalization + editions/versions).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComparisonOperands {
    /// Comparison kind (operand kinds).
    pub kind: ComparisonKind,
    /// Left operand identity (e.g. `quran-edition:slug@version`).
    pub left: String,
    /// Right operand identity.
    pub right: String,
    /// Normalization applied before comparison (empty for exact Tier-1).
    pub normalization_applied: Vec<String>,
    /// Classification vocabulary version.
    pub vocabulary: String,
}

impl ComparisonOperands {
    /// Exact Tier-1 operand record (no normalization applied).
    pub fn exact(kind: ComparisonKind, left: &str, right: &str) -> Self {
        Self {
            kind,
            left: left.to_string(),
            right: right.to_string(),
            normalization_applied: Vec::new(),
            vocabulary: CLASSIFICATION_VOCABULARY.to_string(),
        }
    }
}

/// Classify one ayah-level difference (ADR-0114 §Decision.2).
///
/// Presence differences (one side absent) are [`DifferenceClass::EditionDifference`];
/// whitespace-only text differences are [`DifferenceClass::NormalizationOnly`];
/// anything else is [`DifferenceClass::UnknownDifference`] — the conservative
/// default the ADR requires. Cross-reading attribution (riwayah/orthographic)
/// needs edition metadata the differ does not carry, so this function never
/// claims those classes from bare strings.
pub fn classify_difference(old: Option<&str>, new: Option<&str>) -> DifferenceClass {
    match (old, new) {
        (None, None) => DifferenceClass::Same,
        (None, Some(_)) | (Some(_), None) => DifferenceClass::EditionDifference,
        (Some(old_text), Some(new_text)) => {
            if old_text == new_text {
                DifferenceClass::Same
            } else if whitespace_normalized_equal(old_text, new_text) {
                DifferenceClass::NormalizationOnly
            } else {
                DifferenceClass::UnknownDifference
            }
        }
    }
}

/// Whitespace-insensitive equality: same token sequence, any spacing.
fn whitespace_normalized_equal(first: &str, second: &str) -> bool {
    first.split_whitespace().collect::<Vec<_>>() == second.split_whitespace().collect::<Vec<_>>()
}

/// How one ayah changed between versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    /// Present only in the new version.
    Added,
    /// Present only in the old version.
    Removed,
    /// Present in both with different text.
    Changed,
}

/// One ayah-level change with character ranges into the new text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AyahChange {
    /// Surah number.
    pub surah: u16,
    /// Ayah number.
    pub ayah: u32,
    /// Change kind.
    pub kind: ChangeKind,
    /// Old text (absent for `Added`).
    pub old_text: Option<String>,
    /// New text (absent for `Removed`).
    pub new_text: Option<String>,
    /// Changed `(start, end)` character ranges in the new text.
    pub changed_ranges: Vec<(u32, u32)>,
    /// ADR-0114 classification of this change (Tier-2; defaults to
    /// `unknown_difference` for legacy reports).
    #[serde(default)]
    pub classification: DifferenceClass,
}

/// The full edition difference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditionDiff {
    /// Added ayahs.
    pub added: u32,
    /// Removed ayahs.
    pub removed: u32,
    /// Changed ayahs.
    pub changed: u32,
    /// Unchanged ayahs.
    pub unchanged: u32,
    /// Whether edition-level metadata differed.
    pub metadata_changed: bool,
    /// Per-ayah changes in `(surah, ayah)` order.
    pub changes: Vec<AyahChange>,
    /// Operand-kind-aware comparison kind (Tier-2; legacy reports read as `version`).
    #[serde(default)]
    pub comparison_kind: ComparisonKind,
    /// Normalization applied before comparison (empty for exact Tier-1).
    #[serde(default)]
    pub normalization_applied: Vec<String>,
}

impl EditionDiff {
    /// Whether the two versions are identical.
    pub fn is_empty(&self) -> bool {
        self.added == 0 && self.removed == 0 && self.changed == 0 && !self.metadata_changed
    }
}

fn changed_ranges(old_text: &str, new_text: &str) -> Vec<(u32, u32)> {
    TextDiff::from_chars(old_text, new_text)
        .ops()
        .iter()
        .filter_map(|op| match op.tag() {
            DiffTag::Equal => None,
            _ => Some((op.new_range().start as u32, op.new_range().end as u32)),
        })
        .collect()
}

/// Diff two ayah lists aligned by `(surah, ayah)` identity.
///
/// Same-edition version comparison (ADR-0109): the typed Tier-2 default.
/// Use [`diff_ayahs_typed`] to name another operand kind explicitly.
pub fn diff_ayahs(
    old: &[(u16, u32, String)],
    new: &[(u16, u32, String)],
    metadata_changed: bool,
) -> EditionDiff {
    diff_ayahs_typed(old, new, metadata_changed, ComparisonKind::Version, Vec::new())
}

/// Typed diff: same alignment as [`diff_ayahs`], with the ADR-0114 operand
/// kind and the applied normalization recorded on the report for
/// reproducibility. Every change carries its [`DifferenceClass`]; the Tier-1
/// exact path records no normalization and classifies text changes
/// conservatively (`unknown_difference` unless whitespace-only).
pub fn diff_ayahs_typed(
    old: &[(u16, u32, String)],
    new: &[(u16, u32, String)],
    metadata_changed: bool,
    kind: ComparisonKind,
    normalization_applied: Vec<String>,
) -> EditionDiff {
    use std::collections::BTreeMap;
    let old_map: BTreeMap<(u16, u32), &str> =
        old.iter().map(|(s, a, t)| ((*s, *a), t.as_str())).collect();
    let new_map: BTreeMap<(u16, u32), &str> =
        new.iter().map(|(s, a, t)| ((*s, *a), t.as_str())).collect();
    let mut diff = EditionDiff {
        added: 0,
        removed: 0,
        changed: 0,
        unchanged: 0,
        metadata_changed,
        changes: Vec::new(),
        comparison_kind: kind,
        normalization_applied,
    };
    let mut keys: Vec<(u16, u32)> = old_map.keys().chain(new_map.keys()).copied().collect();
    keys.sort_unstable();
    keys.dedup();
    for key in keys {
        match (old_map.get(&key), new_map.get(&key)) {
            (None, Some(new_text)) => {
                diff.added += 1;
                diff.changes.push(AyahChange {
                    surah: key.0,
                    ayah: key.1,
                    kind: ChangeKind::Added,
                    old_text: None,
                    new_text: Some((*new_text).to_string()),
                    changed_ranges: vec![(0, new_text.chars().count() as u32)],
                    classification: DifferenceClass::EditionDifference,
                });
            }
            (Some(_), None) => {
                diff.removed += 1;
                diff.changes.push(AyahChange {
                    surah: key.0,
                    ayah: key.1,
                    kind: ChangeKind::Removed,
                    old_text: None,
                    new_text: None,
                    changed_ranges: Vec::new(),
                    classification: DifferenceClass::EditionDifference,
                });
            }
            (Some(old_text), Some(new_text)) if old_text != new_text => {
                diff.changed += 1;
                diff.changes.push(AyahChange {
                    surah: key.0,
                    ayah: key.1,
                    kind: ChangeKind::Changed,
                    old_text: Some((*old_text).to_string()),
                    new_text: Some((*new_text).to_string()),
                    changed_ranges: changed_ranges(old_text, new_text),
                    classification: classify_difference(Some(old_text), Some(new_text)),
                });
            }
            _ => {
                diff.unchanged += 1;
            }
        }
    }
    diff
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ayahs(rows: &[(u16, u32, &str)]) -> Vec<(u16, u32, String)> {
        rows.iter().map(|(s, a, t)| (*s, *a, (*t).to_string())).collect()
    }

    #[test]
    fn identical_lists_have_no_changes() {
        let list = ayahs(&[(1, 1, "ب"), (1, 2, "ت")]);
        let diff = diff_ayahs(&list, &list.clone(), false);
        assert!(diff.is_empty());
        assert_eq!(diff.unchanged, 2);
    }

    #[test]
    fn added_removed_changed_are_counted_with_ranges() {
        let old = ayahs(&[(1, 1, "ب ت"), (1, 2, "gone"), (2, 1, "same")]);
        let new = ayahs(&[(1, 1, "ب ث"), (1, 3, "new"), (2, 1, "same")]);
        let diff = diff_ayahs(&old, &new, false);
        assert_eq!((diff.added, diff.removed, diff.changed, diff.unchanged), (1, 1, 1, 1));
        assert!(!diff.is_empty());
        let changed = diff.changes.iter().find(|c| c.kind == ChangeKind::Changed).unwrap();
        assert_eq!((changed.surah, changed.ayah), (1, 1));
        assert!(!changed.changed_ranges.is_empty());
        let added = diff.changes.iter().find(|c| c.kind == ChangeKind::Added).unwrap();
        assert_eq!(added.new_text.as_deref(), Some("new"));
        let removed = diff.changes.iter().find(|c| c.kind == ChangeKind::Removed).unwrap();
        assert!(removed.new_text.is_none());
    }

    #[test]
    fn metadata_flag_survives_identical_text() {
        let list = ayahs(&[(1, 1, "ب")]);
        let diff = diff_ayahs(&list, &list.clone(), true);
        assert!(!diff.is_empty());
        assert!(diff.metadata_changed);
    }

    #[test]
    fn presence_changes_classify_as_edition_difference() {
        assert_eq!(classify_difference(None, Some("ب")), DifferenceClass::EditionDifference);
        assert_eq!(classify_difference(Some("ب"), None), DifferenceClass::EditionDifference);
        assert_eq!(classify_difference(None, None), DifferenceClass::Same);
        assert_eq!(classify_difference(Some("ب"), Some("ب")), DifferenceClass::Same);
    }

    #[test]
    fn whitespace_only_changes_classify_as_normalization_only() {
        assert_eq!(
            classify_difference(Some("ب ت"), Some("ب  ت")),
            DifferenceClass::NormalizationOnly
        );
        assert_eq!(
            classify_difference(Some("ب ت"), Some("ب ث")),
            DifferenceClass::UnknownDifference
        );
    }

    #[test]
    fn vocabulary_serializes_to_the_adr_literals() {
        for (class, literal) in [
            (DifferenceClass::Same, "\"same\""),
            (DifferenceClass::NormalizationOnly, "\"normalization_only\""),
            (DifferenceClass::OrthographicDifference, "\"orthographic_difference\""),
            (DifferenceClass::ScriptDifference, "\"script_difference\""),
            (DifferenceClass::RiwayahDifference, "\"riwayah_difference\""),
            (DifferenceClass::EditionDifference, "\"edition_difference\""),
            (DifferenceClass::TokenizationDifference, "\"tokenization_difference\""),
            (DifferenceClass::TranslationDifference, "\"translation_difference\""),
            (DifferenceClass::UnknownDifference, "\"unknown_difference\""),
        ] {
            assert_eq!(serde_json::to_string(&class).unwrap(), literal);
        }
        for (kind, literal) in [
            (ComparisonKind::Integrity, "\"integrity\""),
            (ComparisonKind::Version, "\"version\""),
            (ComparisonKind::Readings, "\"readings\""),
            (ComparisonKind::Translation, "\"translation\""),
            (ComparisonKind::Reference, "\"reference\""),
            (ComparisonKind::Checksum, "\"checksum\""),
        ] {
            assert_eq!(serde_json::to_string(&kind).unwrap(), literal);
            assert_eq!(
                format!("{:?}", serde_json::from_str::<ComparisonKind>(literal).unwrap()),
                format!("{:?}", kind)
            );
        }
    }

    #[test]
    fn typed_diff_records_kind_normalization_and_classifications() {
        let old = ayahs(&[(1, 1, "ب ت"), (1, 2, "gone"), (2, 1, "same")]);
        let new = ayahs(&[(1, 1, "ب  ت"), (1, 3, "new"), (2, 1, "same")]);
        let diff = diff_ayahs_typed(
            &old,
            &new,
            false,
            ComparisonKind::Reference,
            vec!["whitespace".to_string()],
        );
        assert_eq!((diff.added, diff.removed, diff.changed, diff.unchanged), (1, 1, 1, 1));
        assert_eq!(diff.comparison_kind, ComparisonKind::Reference);
        assert_eq!(diff.normalization_applied, vec!["whitespace".to_string()]);
        let changed = diff.changes.iter().find(|c| c.kind == ChangeKind::Changed).unwrap();
        assert_eq!(changed.classification, DifferenceClass::NormalizationOnly);
        let added = diff.changes.iter().find(|c| c.kind == ChangeKind::Added).unwrap();
        assert_eq!(added.classification, DifferenceClass::EditionDifference);
        let removed = diff.changes.iter().find(|c| c.kind == ChangeKind::Removed).unwrap();
        assert_eq!(removed.classification, DifferenceClass::EditionDifference);
    }

    #[test]
    fn legacy_reports_without_tier2_fields_still_deserialize() {
        let legacy = serde_json::json!({
            "added": 0, "removed": 0, "changed": 0, "unchanged": 1,
            "metadata_changed": false, "changes": [],
        });
        let diff: EditionDiff = serde_json::from_value(legacy).unwrap();
        assert_eq!(diff.comparison_kind, ComparisonKind::Version);
        assert!(diff.normalization_applied.is_empty());
        assert!(diff.is_empty());
    }

    #[test]
    fn exact_operand_record_carries_kind_and_vocabulary() {
        let operands =
            ComparisonOperands::exact(ComparisonKind::Reference, "left@1.0.0", "right@1.0.0");
        assert!(operands.normalization_applied.is_empty());
        assert_eq!(operands.vocabulary, CLASSIFICATION_VOCABULARY);
        let roundtrip: ComparisonOperands =
            serde_json::from_str(&serde_json::to_string(&operands).unwrap()).unwrap();
        assert_eq!(roundtrip, operands);
    }
}
