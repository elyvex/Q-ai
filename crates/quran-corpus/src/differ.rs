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
pub fn diff_ayahs(
    old: &[(u16, u32, String)],
    new: &[(u16, u32, String)],
    metadata_changed: bool,
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
}
