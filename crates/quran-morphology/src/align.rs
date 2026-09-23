//! Phase 2 — token alignment (`align`).
//!
//! Alignment links intermediate rows to the application token inventory
//! **without ever modifying tokens**: the inventory is taken by shared
//! reference and this module returns a report.
//!
//! - [`DirectKey`] (`surah:ayah:position:surface`) requires a 100% key
//!   match against the inventory.
//! - Anything else with a matching `(surah, ayah, position)` but a
//!   differing surface goes through the hashed, auditable
//!   [`AlignmentTable`] (`table_mapped`).
//! - Positions absent from the inventory are `unmatched`, counted per
//!   surah in [`AlignmentReport::unmatched_by_surah`].
//!
//! The table hash is FNV-1a/64 over the mapping triple: a deterministic
//! audit checksum, not a cryptographic commitment. The application layer
//! may additionally store a content hash at import.

use std::collections::{HashMap, HashSet};

use crate::dataset::{IntermediateMorphology, InventoryToken};
use crate::error::MorphologyError;

/// A direct alignment key: `surah:ayah:position:surface`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectKey(String);

impl DirectKey {
    /// Build a direct key from its components.
    pub fn new(surah: u32, ayah: u32, position: u32, surface: &str) -> Self {
        Self(format!("{surah}:{ayah}:{position}:{surface}"))
    }

    /// The key string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One non-direct mapping: the inventory surface that was found for a
/// position whose intermediate surface differs, with an audit hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlignmentEntry {
    /// The direct key that failed the 100% match.
    pub direct_key: String,
    /// Surface present in the inventory at this position.
    pub expected_surface: String,
    /// Surface present in the intermediate row.
    pub found_surface: String,
    /// FNV-1a/64 audit hash over `(expected, found, direct_key)`.
    pub alignment_hash: String,
}

/// Hashed, auditable map for non-direct matches.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AlignmentTable {
    entries: HashMap<String, AlignmentEntry>,
}

impl AlignmentTable {
    /// Empty table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of mapped entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the table holds no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Look up one mapping by direct key.
    pub fn get(&self, direct_key: &str) -> Option<&AlignmentEntry> {
        self.entries.get(direct_key)
    }

    /// Iterate over all mappings.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &AlignmentEntry)> {
        self.entries.iter()
    }
}

/// The outcome of aligning one intermediate document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AlignmentReport {
    /// Rows with a 100% direct-key match.
    pub matched: usize,
    /// Rows resolved through the alignment table.
    pub table_mapped: usize,
    /// The auditable table of non-direct mappings.
    pub table: AlignmentTable,
    /// Unmatched rows per surah (gates import approval upstream).
    pub unmatched_by_surah: HashMap<u32, usize>,
}

impl AlignmentReport {
    /// Total unmatched rows across all surahs.
    pub fn unmatched_total(&self) -> usize {
        self.unmatched_by_surah.values().sum()
    }

    /// Fail-closed gate: any unmatched row blocks the import step.
    pub fn require_fully_aligned(&self) -> Result<(), MorphologyError> {
        if self.unmatched_total() == 0 {
            Ok(())
        } else {
            let mut per_surah: Vec<(u32, usize)> =
                self.unmatched_by_surah.iter().map(|(s, n)| (*s, *n)).collect();
            per_surah.sort_unstable();
            Err(MorphologyError::AlignmentFailed {
                detail: format!("unmatched tokens remain: {per_surah:?}"),
            })
        }
    }
}

/// Deterministic FNV-1a/64 audit hash, hex-encoded.
fn fnv1a_hex(input: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// Align an intermediate document against the token inventory.
///
/// The inventory is read-only; the returned report owns all outcomes.
pub fn align(
    intermediate: &IntermediateMorphology,
    inventory: &[InventoryToken],
) -> AlignmentReport {
    let mut exact: HashSet<&str> = HashSet::new();
    let mut by_position: HashMap<(u32, u32, u32), &str> = HashMap::new();
    // Borrowed-key maps: keys borrow from `inventory`, values are read during
    // the loop below, so no mutation of the caller's tokens can occur.
    let mut key_store: Vec<String> = Vec::with_capacity(inventory.len());
    for token in inventory {
        let (surah, ayah, position, surface) = token;
        key_store.push(DirectKey::new(*surah, *ayah, *position, surface).0);
    }
    for (token, key) in inventory.iter().zip(key_store.iter()) {
        exact.insert(key.as_str());
        by_position.insert((token.0, token.1, token.2), token.3.as_str());
    }

    let mut report = AlignmentReport::default();
    for analysis in &intermediate.analyses {
        let key = DirectKey::new(
            analysis.surah,
            analysis.ayah,
            analysis.token_position,
            &analysis.surface,
        );
        if exact.contains(key.as_str()) {
            report.matched += 1;
        } else if let Some(expected) =
            by_position.get(&(analysis.surah, analysis.ayah, analysis.token_position))
        {
            report.table_mapped += 1;
            let hash_input = format!("{expected}\u{1f}{}\u{1f}{}", analysis.surface, key.as_str());
            report.table.entries.insert(
                key.0.clone(),
                AlignmentEntry {
                    direct_key: key.0.clone(),
                    expected_surface: (*expected).to_string(),
                    found_surface: analysis.surface.clone(),
                    alignment_hash: fnv1a_hex(&hash_input),
                },
            );
        } else {
            *report.unmatched_by_surah.entry(analysis.surah).or_insert(0) += 1;
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::{DatasetRef, TokenAnalysis};

    fn analysis(surah: u32, ayah: u32, position: u32, surface: &str) -> TokenAnalysis {
        TokenAnalysis {
            surah,
            ayah,
            token_position: position,
            analysis_index: 0,
            surface: surface.to_string(),
            lemma: "كَتَبَ".to_string(),
            root: "كتب".to_string(),
            stem: String::new(),
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

    fn doc(rows: Vec<TokenAnalysis>) -> IntermediateMorphology {
        IntermediateMorphology {
            dataset: DatasetRef::new("synthetic-morph-test", "0.1.0"),
            edition_id: String::new(),
            synthetic_test_only: true,
            analyses: rows,
        }
    }

    #[test]
    fn direct_key_format_is_stable() {
        assert_eq!(DirectKey::new(2, 1, 3, "كَتَبَ").as_str(), "2:1:3:كَتَبَ");
    }

    #[test]
    fn exact_match_counts_as_matched() {
        let inventory = vec![(2_u32, 1_u32, 1_u32, "كَتَبَ".to_string())];
        let report = align(&doc(vec![analysis(2, 1, 1, "كَتَبَ")]), &inventory);
        assert_eq!(report.matched, 1);
        assert_eq!(report.table_mapped, 0);
        assert_eq!(report.unmatched_total(), 0);
        assert!(report.require_fully_aligned().is_ok());
    }

    #[test]
    fn surface_mismatch_goes_through_hashed_table() {
        let inventory = vec![(2_u32, 1_u32, 1_u32, "كَتَبَ".to_string())];
        let report = align(&doc(vec![analysis(2, 1, 1, "كِتَاب")]), &inventory);
        assert_eq!(report.matched, 0);
        assert_eq!(report.table_mapped, 1);
        let entry = report.table.get("2:1:1:كِتَاب").expect("table entry recorded");
        assert_eq!(entry.expected_surface, "كَتَبَ");
        assert_eq!(entry.found_surface, "كِتَاب");
        assert_eq!(entry.alignment_hash.len(), 16);
    }

    #[test]
    fn unknown_position_is_unmatched_per_surah_and_blocks_gate() {
        let inventory = vec![(2_u32, 1_u32, 1_u32, "كَتَبَ".to_string())];
        let report = align(&doc(vec![analysis(2, 1, 9, "قَالَ")]), &inventory);
        assert_eq!(report.unmatched_by_surah.get(&2), Some(&1));
        let err = report.require_fully_aligned().expect_err("must block");
        assert!(matches!(err, MorphologyError::AlignmentFailed { .. }));
    }
}
