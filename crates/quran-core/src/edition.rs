//! The Quran edition model (PRD §7.2).
//!
//! # quran_core::edition
//!
//! An edition is one versioned, identified reading of the Quran. Different
//! readings/editions are never merged (invariant I3): `id` is part of every
//! canonical key and every returned record.

use domain::{ContentHash, EditionId, Language, LicenseRecord, SemVer, SourceVersionId, Timestamp};
use serde::{Deserialize, Serialize};

use crate::enums::{BasmalaPolicy, EditionStatus, NumberingScheme, Script, UnicodeForm};

/// Counts computed at import and stored with the edition (D1.3).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditionStatistics {
    /// Number of surahs.
    pub surah_count: u16,
    /// Total ayahs.
    pub ayah_count: u32,
    /// Total surface tokens.
    pub token_count: u64,
    /// Distinct surface forms.
    pub distinct_surface_forms: u64,
    /// Number of juz divisions.
    pub juz_count: u16,
    /// Number of pages, when the edition defines pagination.
    pub page_count: Option<u32>,
    /// Number of sajdah markers.
    pub sajdah_count: u16,
}

/// A versioned edition of the Quran.
///
/// The identity/hash fields (`text_hash`, `structure_hash`, `token_order_hash`,
/// `slug`, `version`) are immutable once written (trigger `QAI-QUR-0002`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuranEdition {
    /// Stable edition identity.
    pub id: EditionId,
    /// URL/CLI-safe slug, e.g. `hafs-uthmani`.
    pub slug: String,
    /// Human-readable name.
    pub name: String,
    /// Writing system.
    pub script: Script,
    /// Transmission (e.g. `Hafs`).
    pub riwayah: Option<String>,
    /// Reading (e.g. `Asim`).
    pub qiraah: Option<String>,
    /// Publisher, if any.
    pub publisher: Option<String>,
    /// Source URL, if any.
    pub source_url: Option<String>,
    /// Exact upstream identifier, preserved verbatim (ADR-0101, OD-01).
    pub upstream_edition_slug: Option<String>,
    /// Q-ai-internal stable id mapped from the upstream slug.
    pub qai_edition_id: Option<String>,
    /// Explicit primary/default designation (D-07); "primary default" is not
    /// "only edition" and never redefines the active-edition pointer.
    pub is_primary: bool,
    /// Licensing record (PRD §38).
    pub license: LicenseRecord,
    /// Content language (always `ar` for the canonical text).
    pub language: Language,
    /// Verse-numbering scheme (ADR-0103).
    pub verse_numbering_scheme: NumberingScheme,
    /// Basmala policy (ADR-0110).
    pub basmala_policy: BasmalaPolicy,
    /// Declared normalization form of stored text (ADR-0104).
    pub unicode_normalization: UnicodeForm,
    /// Hash over the canonical ayah text stream.
    pub text_hash: ContentHash,
    /// Hash over the structural skeleton.
    pub structure_hash: ContentHash,
    /// Hash over token ids/positions.
    pub token_order_hash: ContentHash,
    /// Hash of the source manifest that produced this edition.
    pub manifest_hash: ContentHash,
    /// Edition version.
    pub version: SemVer,
    /// The source version this edition was imported from.
    pub source_version_id: SourceVersionId,
    /// Import timestamp.
    pub imported_at: Timestamp,
    /// Editorial verification timestamp, if verified.
    pub verified_at: Option<Timestamp>,
    /// Named human reviewer who signed off the text, if any.
    pub verified_by: Option<String>,
    /// Method used for verification, if any.
    pub verification_method: Option<String>,
    /// Lifecycle status.
    pub status: EditionStatus,
    /// Computed counts.
    pub statistics: EditionStatistics,
}

impl QuranEdition {
    /// Whether this edition may be served as canonical text.
    pub fn is_readable(&self) -> bool {
        matches!(self.status, EditionStatus::Active | EditionStatus::Deprecated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{HashAlgorithm, LicenseStatus};

    fn content_hash(byte: &str) -> ContentHash {
        ContentHash::try_new(HashAlgorithm::Sha256, byte.repeat(32)).unwrap()
    }

    fn sample_edition() -> QuranEdition {
        QuranEdition {
            id: EditionId::new(),
            slug: "hafs-uthmani".to_string(),
            name: "Quran — Hafs, Uthmani script".to_string(),
            script: Script::Uthmani,
            riwayah: Some("Hafs".to_string()),
            qiraah: Some("Asim".to_string()),
            publisher: None,
            source_url: None,
            upstream_edition_slug: None,
            qai_edition_id: None,
            is_primary: false,
            license: LicenseRecord {
                status: LicenseStatus::PublicDomain,
                spdx_id: None,
                name: None,
                url: None,
                attribution_required: false,
                redistribution_allowed: true,
                export_allowed: true,
                notes: None,
            },
            language: "ar".parse().unwrap(),
            verse_numbering_scheme: NumberingScheme::Hafs,
            basmala_policy: BasmalaPolicy::PerSurah,
            unicode_normalization: UnicodeForm::Nfc,
            text_hash: content_hash("ab"),
            structure_hash: content_hash("cd"),
            token_order_hash: content_hash("ef"),
            manifest_hash: content_hash("12"),
            version: SemVer::new(1, 0, 0),
            source_version_id: SourceVersionId::new(),
            imported_at: Timestamp::from_ymd_hms(2026, 1, 15, 0, 0, 0).unwrap(),
            verified_at: None,
            verified_by: None,
            verification_method: None,
            status: EditionStatus::Staged,
            statistics: EditionStatistics::default(),
        }
    }

    #[test]
    fn edition_roundtrips_serde() {
        let edition = sample_edition();
        let json = serde_json::to_string(&edition).unwrap();
        assert!(json.contains("\"script\":\"uthmani\""));
        assert!(json.contains("\"status\":\"staged\""));
        let back: QuranEdition = serde_json::from_str(&json).unwrap();
        assert_eq!(back, edition);
    }

    #[test]
    fn readability_depends_on_status() {
        let mut edition = sample_edition();
        assert!(!edition.is_readable());
        edition.status = EditionStatus::Active;
        assert!(edition.is_readable());
        edition.status = EditionStatus::Deprecated;
        assert!(edition.is_readable());
        edition.status = EditionStatus::Quarantined;
        assert!(!edition.is_readable());
    }

    #[test]
    fn statistics_default_is_zeroed() {
        let stats = EditionStatistics::default();
        assert_eq!(stats.surah_count, 0);
        assert_eq!(stats.ayah_count, 0);
        assert_eq!(stats.page_count, None);
    }
}
