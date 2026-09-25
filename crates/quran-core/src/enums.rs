//! Canonical Quran enumerations.
//!
//! # quran_core::enums
//!
//! Script, numbering, basmala, Unicode-form, status, segment and edition
//! selector enums. All are serde `snake_case` so the JSON edition source format
//! and the API envelope agree on spelling.

use domain::SemVer;
use serde::{Deserialize, Serialize};

/// The writing system of an edition (ADR-0101).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Script {
    /// Uthmani orthography.
    Uthmani,
    /// Simplified (Imlaei) orthography.
    ImlaeiSimple,
    /// Any other declared script.
    Other(String),
}

/// The verse-numbering scheme an edition follows (ADR-0103).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumberingScheme {
    /// The Hafs numbering (6236 ayahs).
    Hafs,
    /// The Kufi numbering.
    Kufi,
    /// A named custom scheme.
    Custom(String),
}

/// How the basmala relates to ayah numbering in an edition (ADR-0110, §13.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BasmalaPolicy {
    /// The basmala is ayah 1 of the surah (as in Al-Fatihah).
    CountedAsFirstAyah,
    /// The basmala precedes the surah and is not numbered.
    UnnumberedHeader,
    /// There is no basmala (e.g. At-Tawbah).
    Absent,
    /// A per-surah override table supplied by the edition.
    PerSurah,
}

/// The declared Unicode normalization form of stored canonical text (ADR-0104).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnicodeForm {
    /// Canonical composition (expected).
    Nfc,
    /// Canonical decomposition.
    Nfd,
    /// Compatibility composition.
    Nfkc,
    /// Compatibility decomposition.
    Nfkd,
}

/// Whether a surah is Meccan or Medinan (publisher metadata, Layer B).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevelationPlace {
    /// Revealed in Mecca.
    Makki,
    /// Revealed in Medina.
    Madani,
}

/// The kind of sajdah marked in an ayah.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SajdahKind {
    /// A recommended prostration.
    Recommended,
    /// An obligatory prostration.
    Obligatory,
}

/// Lifecycle status of an edition (mirrors the SQL CHECK constraint set).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditionStatus {
    /// Imported into staging, not yet activated.
    Staged,
    /// Validated and approved, not yet active.
    Approved,
    /// The currently active edition version.
    Active,
    /// Superseded by a newer active version.
    Deprecated,
    /// Rejected by validation or editorial review.
    Quarantined,
}

/// A grouping of tokens inside an ayah (display/analysis only, never reorders).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentKind {
    /// A clause.
    Clause,
    /// A pause (waqf) mark.
    PauseMark,
    /// A basmala contained in the ayah.
    Basmala,
    /// Any other declared segment kind.
    Other,
}

/// How an edition is selected for a lookup (ADR-0102).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditionSelector {
    /// Resolve via the active-edition pointer.
    Active,
    /// Resolve the explicitly flagged primary/default edition (D-07).
    ///
    /// "Primary default" is not "only edition": this never falls back to the
    /// active-edition pointer. A store with no flagged edition — or with more
    /// than one — is a typed error, so a caller cannot believe it read "the
    /// default" when none is uniquely declared.
    Primary,
    /// Resolve a specific slug at its active version.
    Slug(String),
    /// Resolve an exact, pinned slug@version for reproducible research (§12.1).
    Pinned {
        /// The edition slug.
        slug: String,
        /// The exact edition version.
        version: SemVer,
    },
}

/// Canonical-structure boundaries that context retrieval never crosses (§11.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextBoundary {
    /// Stay within the focal surah.
    Surah,
    /// Stay within the focal juz.
    Juz,
    /// Stay within the focal ruku.
    Ruku,
    /// Stay within the focal page.
    Page,
    /// No structural boundary (global order only).
    None,
}

/// Validate an edition slug against the frozen grammar: `ALPHA (ALPHA|DIGIT|'-')*`.
///
/// Slugs are lowercase by convention (`hafs-uthmani`), which is what the storage
/// layer and deep links assume.
pub fn is_valid_slug(slug: &str) -> bool {
    let mut bytes = slug.bytes();
    match bytes.next() {
        Some(b) if b.is_ascii_lowercase() => {}
        _ => return false,
    }
    bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_grammar() {
        assert!(is_valid_slug("hafs-uthmani"));
        assert!(is_valid_slug("a"));
        assert!(!is_valid_slug(""));
        assert!(!is_valid_slug("1st"));
        assert!(!is_valid_slug("-x"));
        assert!(!is_valid_slug("Upper"));
        assert!(!is_valid_slug("has space"));
        assert!(!is_valid_slug("under_score"));
    }

    #[test]
    fn enums_serialize_snake_case() {
        assert_eq!(serde_json::to_string(&BasmalaPolicy::PerSurah).unwrap(), "\"per_surah\"");
        assert_eq!(serde_json::to_string(&UnicodeForm::Nfc).unwrap(), "\"nfc\"");
        assert_eq!(serde_json::to_string(&EditionStatus::Active).unwrap(), "\"active\"");
        assert_eq!(serde_json::to_string(&EditionSelector::Active).unwrap(), "\"active\"");
    }

    #[test]
    fn pinned_selector_carries_version() {
        let sel =
            EditionSelector::Pinned { slug: "hafs-uthmani".into(), version: SemVer::new(1, 0, 0) };
        let json = serde_json::to_string(&sel).unwrap();
        assert!(json.contains("hafs-uthmani"));
        assert!(json.contains("1.0.0"));
    }
}
