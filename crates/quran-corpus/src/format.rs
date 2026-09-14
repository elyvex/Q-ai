//! The normalized intermediate edition format (`qai.quran.edition` v1).
//!
//! # quran_corpus::format
//!
//! Adapters convert real-world dataset shapes into [`EditionSource`]; the
//! importer and validator only ever see this type. Field names match
//! `docs/schemas/quran-edition-source.v1.schema.json` exactly.
//!
//! `AyahSource::tokens` is optional: adapters that ship pre-tokenized data may
//! supply tokens, but they are **verified, never trusted** — the importer
//! always recomputes tokenization and the validator checks consistency
//! (QV-010…QV-012, QV-024).

use domain::{Language, SemVer};
use quran_core::enums::{
    BasmalaPolicy, NumberingScheme, RevelationPlace, SajdahKind, Script, UnicodeForm, is_valid_slug,
};
use serde::{Deserialize, Serialize};

use crate::error::CorpusError;

/// The format tag every edition source must declare.
pub const FORMAT_TAG: &str = "qai.quran.edition";
/// The only intermediate-format version Phase 1 accepts.
pub const FORMAT_VERSION: u32 = 1;

/// A normalized Quran edition ready for validation and import.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditionSource {
    /// Must be [`FORMAT_TAG`].
    pub format: String,
    /// Must be [`FORMAT_VERSION`].
    pub format_version: u32,
    /// Edition identity and policy metadata.
    pub edition: EditionMeta,
    /// Declared counts the validator checks the corpus against.
    pub expected: ExpectedCounts,
    /// Surah metadata rows.
    pub surahs: Vec<SurahSource>,
    /// Canonical ayah rows.
    pub ayahs: Vec<AyahSource>,
    /// How the source was tokenized (informational; recomputed on import).
    pub tokenization: TokenizationPolicy,
}

/// Identity and policy metadata of the edition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditionMeta {
    /// URL/CLI-safe slug, e.g. `hafs-uthmani`.
    pub slug: String,
    /// Human-readable name.
    pub name: String,
    /// Writing system.
    pub script: Script,
    /// Transmission, if declared.
    pub riwayah: Option<String>,
    /// Reading, if declared.
    pub qiraah: Option<String>,
    /// Content language (`ar` for a canonical Arabic edition).
    pub language: Language,
    /// Verse-numbering scheme.
    pub verse_numbering_scheme: NumberingScheme,
    /// Declared normalization form of the text.
    pub unicode_normalization: UnicodeForm,
    /// Basmala policy.
    pub basmala_policy: BasmalaPolicy,
    /// Publisher, if any.
    pub publisher: Option<String>,
    /// Edition version.
    pub version: SemVer,
}

/// Declared counts and the optional reference corpus for comparison (QV-015).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpectedCounts {
    /// Expected number of surahs.
    pub surah_count: u16,
    /// Expected total ayahs.
    pub ayah_count: u32,
    /// Reference corpus id for independent comparison, if configured.
    pub reference_corpus_id: Option<String>,
    /// Expected reference text hash, if configured.
    pub reference_text_hash: Option<String>,
}

/// One surah metadata row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurahSource {
    /// Surah number.
    pub number: u16,
    /// Arabic name.
    pub name_arabic: String,
    /// Transliterated name, if provided.
    pub name_transliteration: Option<String>,
    /// Declared ayah count for this surah.
    pub ayah_count: u16,
    /// Meccan or Medinan, if provided.
    pub revelation_place: Option<RevelationPlace>,
    /// Revelation order, if provided.
    pub revelation_order: Option<u16>,
    /// Basmala policy for this surah.
    pub basmala: BasmalaPolicy,
    /// Ruku count, if provided.
    pub ruku_count: Option<u16>,
}

/// One ayah row with its structural metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AyahSource {
    /// Surah number.
    pub surah: u16,
    /// Ayah number.
    pub ayah: u32,
    /// Canonical text.
    pub text: String,
    /// Juz number, if known.
    pub juz: Option<u16>,
    /// Hizb number, if known.
    pub hizb: Option<u16>,
    /// Rubʿ number, if known.
    pub rub: Option<u16>,
    /// Manzil number, if known.
    pub manzil: Option<u16>,
    /// Ruku number, if known.
    pub ruku: Option<u16>,
    /// Page number, if the edition paginates.
    pub page: Option<u32>,
    /// Sajdah kind, if this ayah carries one.
    pub sajdah: Option<SajdahKind>,
    /// Adapter-supplied tokens (verified, never trusted).
    #[serde(default)]
    pub tokens: Option<Vec<TokenSource>>,
}

/// One adapter-supplied token with offsets into the ayah text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenSource {
    /// 1-based position within the ayah.
    pub position: u16,
    /// Surface form.
    pub surface: String,
    /// Grapheme start offset.
    pub char_start: u32,
    /// Grapheme end offset (exclusive).
    pub char_end: u32,
    /// Byte start offset.
    pub byte_start: u32,
    /// Byte end offset (exclusive).
    pub byte_end: u32,
}

/// How the source claims to have been tokenized (informational).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenizationPolicy {
    /// Tokenization strategy, e.g. `whitespace_preserving`.
    pub strategy: String,
    /// Separator policy, e.g. `record_exact`.
    pub separator_policy: String,
}

impl EditionSource {
    /// Parse-level checks: tag, version, slug, and non-empty tables.
    ///
    /// Content rules (counts, Unicode, token order, hashes) belong to the
    /// validator (M5), not to parsing.
    pub fn validate(&self) -> Result<(), CorpusError> {
        if self.format != FORMAT_TAG || self.format_version != FORMAT_VERSION {
            return Err(CorpusError::UnsupportedFormat {
                found: format!("{} v{}", self.format, self.format_version),
            });
        }
        if !is_valid_slug(&self.edition.slug) {
            return Err(CorpusError::InvalidFormat {
                detail: format!("edition slug `{}` is invalid", self.edition.slug),
            });
        }
        if self.edition.name.trim().is_empty() {
            return Err(CorpusError::InvalidFormat { detail: "edition name is empty".into() });
        }
        if self.surahs.is_empty() {
            return Err(CorpusError::InvalidFormat { detail: "edition has no surahs".into() });
        }
        if self.ayahs.is_empty() {
            return Err(CorpusError::InvalidFormat { detail: "edition has no ayahs".into() });
        }
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn sample() -> EditionSource {
        EditionSource {
            format: FORMAT_TAG.to_string(),
            format_version: FORMAT_VERSION,
            edition: EditionMeta {
                slug: "test-edition-min".to_string(),
                name: "Synthetic test edition".to_string(),
                script: Script::Uthmani,
                riwayah: None,
                qiraah: None,
                language: "ar".parse().unwrap(),
                verse_numbering_scheme: NumberingScheme::Hafs,
                unicode_normalization: UnicodeForm::Nfc,
                basmala_policy: BasmalaPolicy::PerSurah,
                publisher: None,
                version: SemVer::new(0, 1, 0),
            },
            expected: ExpectedCounts {
                surah_count: 1,
                ayah_count: 1,
                reference_corpus_id: None,
                reference_text_hash: None,
            },
            surahs: vec![SurahSource {
                number: 1,
                name_arabic: "تجريبي".to_string(),
                name_transliteration: Some("Test".to_string()),
                ayah_count: 1,
                revelation_place: Some(RevelationPlace::Makki),
                revelation_order: Some(1),
                basmala: BasmalaPolicy::CountedAsFirstAyah,
                ruku_count: Some(1),
            }],
            ayahs: vec![AyahSource {
                surah: 1,
                ayah: 1,
                text: "ب ت ث".to_string(),
                juz: Some(1),
                hizb: Some(1),
                rub: Some(1),
                manzil: Some(1),
                ruku: Some(1),
                page: Some(1),
                sajdah: None,
                tokens: None,
            }],
            tokenization: TokenizationPolicy {
                strategy: "whitespace_preserving".to_string(),
                separator_policy: "record_exact".to_string(),
            },
        }
    }

    #[test]
    fn sample_validates_and_roundtrips() {
        let source = sample();
        source.validate().unwrap();
        let json = serde_json::to_string(&source).unwrap();
        let back: EditionSource = serde_json::from_str(&json).unwrap();
        assert_eq!(back, source);
    }

    #[test]
    fn wrong_tag_version_slug_or_empty_tables_rejected() {
        let mut bad = sample();
        bad.format = "other".to_string();
        assert!(bad.validate().is_err());
        let mut bad = sample();
        bad.format_version = 99;
        assert!(bad.validate().is_err());
        let mut bad = sample();
        bad.edition.slug = "Bad Slug".to_string();
        assert!(bad.validate().is_err());
        let mut bad = sample();
        bad.surahs.clear();
        assert!(bad.validate().is_err());
        let mut bad = sample();
        bad.ayahs.clear();
        assert!(bad.validate().is_err());
    }
}
