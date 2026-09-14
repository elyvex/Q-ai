//! `QuranQuotation` — the only type allowed to represent quoted canonical text.
//!
//! # quran_core::quotation
//!
//! Invariant I6: every returned quotation carries the edition id, version, and
//! content hash. `QuranQuotation` therefore has no constructor that omits them —
//! construction goes through [`QuotationParts`], whose `edition` and `text_hash`
//! fields are mandatory.
//!
//! Principle 5: a translation can never occupy the canonical slot. It is carried
//! only as an [`TranslationRef`] alongside, and a translation reference requires a
//! non-empty named translator.

use domain::{ContentHash, Language, SemVer};
use serde::{Deserialize, Serialize};

use crate::enums::Script;
use crate::error::QuranError;
use crate::numbers::{AyahNumber, SurahNumber};

/// Identity of the edition a quotation came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditionRef {
    /// Edition slug.
    pub slug: String,
    /// Exact edition version.
    pub version: SemVer,
    /// Script of the edition.
    pub script: Script,
    /// Transmission, if declared.
    pub riwayah: Option<String>,
}

/// A reference to a translation *alongside* canonical text (never instead of it).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranslationRef {
    /// Translation edition slug.
    pub slug: String,
    /// Named human translator.
    pub translator: String,
    /// Translation language.
    pub language: Language,
}

impl TranslationRef {
    /// Validate the principle-5 requirement: a non-empty named translator.
    pub fn validate(&self) -> Result<(), QuranError> {
        if self.translator.trim().is_empty() {
            return Err(QuranError::MissingTranslator);
        }
        Ok(())
    }
}

/// The validated inputs to a [`QuranQuotation`].
///
/// `edition` and `text_hash` are mandatory, so a quotation cannot exist without
/// the identity and hash required by invariant I6.
#[derive(Debug, Clone)]
pub struct QuotationParts {
    /// Fully-qualified canonical reference, e.g. `quran:hafs-uthmani@1.0.0:2:255`.
    pub reference: String,
    /// Surah number.
    pub surah_number: SurahNumber,
    /// Arabic surah name.
    pub surah_name_arabic: String,
    /// Transliterated surah name, if any.
    pub surah_name_translit: Option<String>,
    /// Inclusive ayah range of this quotation.
    pub ayah_range: (AyahNumber, AyahNumber),
    /// Canonical Arabic text.
    pub arabic_text: String,
    /// Hash of `arabic_text`.
    pub text_hash: ContentHash,
    /// Edition identity (mandatory).
    pub edition: EditionRef,
    /// Optional translation carried alongside.
    pub translation: Option<TranslationRef>,
    /// Deep link for the web reader.
    pub deep_link: String,
    /// Page number, if known.
    pub page: Option<u32>,
    /// Juz number, if known.
    pub juz: Option<u16>,
}

/// A quotation of canonical Arabic text with full, non-optional provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuranQuotation {
    reference: String,
    surah_number: SurahNumber,
    surah_name_arabic: String,
    surah_name_translit: Option<String>,
    ayah_range: (AyahNumber, AyahNumber),
    arabic_text: String,
    text_hash: ContentHash,
    edition: EditionRef,
    translation: Option<TranslationRef>,
    deep_link: String,
    page: Option<u32>,
    juz: Option<u16>,
}

impl QuranQuotation {
    /// Construct a quotation, validating the mandatory provenance and text.
    pub fn new(parts: QuotationParts) -> Result<Self, QuranError> {
        if parts.reference.trim().is_empty() {
            return Err(QuranError::EmptyField { field: "reference" });
        }
        if parts.arabic_text.trim().is_empty() {
            return Err(QuranError::EmptyField { field: "arabic_text" });
        }
        if parts.deep_link.trim().is_empty() {
            return Err(QuranError::EmptyField { field: "deep_link" });
        }
        if parts.ayah_range.0 > parts.ayah_range.1 {
            return Err(QuranError::InvalidAyahNumber);
        }
        if let Some(translation) = &parts.translation {
            translation.validate()?;
        }
        Ok(Self {
            reference: parts.reference,
            surah_number: parts.surah_number,
            surah_name_arabic: parts.surah_name_arabic,
            surah_name_translit: parts.surah_name_translit,
            ayah_range: parts.ayah_range,
            arabic_text: parts.arabic_text,
            text_hash: parts.text_hash,
            edition: parts.edition,
            translation: parts.translation,
            deep_link: parts.deep_link,
            page: parts.page,
            juz: parts.juz,
        })
    }

    /// The fully-qualified canonical reference.
    pub fn reference(&self) -> &str {
        &self.reference
    }

    /// The surah number.
    pub fn surah_number(&self) -> SurahNumber {
        self.surah_number
    }

    /// The Arabic surah name.
    pub fn surah_name_arabic(&self) -> &str {
        &self.surah_name_arabic
    }

    /// The transliterated surah name, if any.
    pub fn surah_name_translit(&self) -> Option<&str> {
        self.surah_name_translit.as_deref()
    }

    /// The inclusive ayah range.
    pub fn ayah_range(&self) -> (AyahNumber, AyahNumber) {
        self.ayah_range
    }

    /// The canonical Arabic text.
    pub fn arabic_text(&self) -> &str {
        &self.arabic_text
    }

    /// The content hash of the canonical text.
    pub fn text_hash(&self) -> &ContentHash {
        &self.text_hash
    }

    /// The edition identity.
    pub fn edition(&self) -> &EditionRef {
        &self.edition
    }

    /// The accompanying translation, if any.
    pub fn translation(&self) -> Option<&TranslationRef> {
        self.translation.as_ref()
    }

    /// The deep link.
    pub fn deep_link(&self) -> &str {
        &self.deep_link
    }

    /// The page number, if known.
    pub fn page(&self) -> Option<u32> {
        self.page
    }

    /// The juz number, if known.
    pub fn juz(&self) -> Option<u16> {
        self.juz
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Diagnostic;
    use domain::HashAlgorithm;

    fn edition_ref() -> EditionRef {
        EditionRef {
            slug: "hafs-uthmani".to_string(),
            version: SemVer::new(1, 0, 0),
            script: Script::Uthmani,
            riwayah: Some("Hafs".to_string()),
        }
    }

    fn parts(text: &str) -> QuotationParts {
        QuotationParts {
            reference: "quran:hafs-uthmani@1.0.0:2:255".to_string(),
            surah_number: SurahNumber::new(2).unwrap(),
            surah_name_arabic: "البقرة".to_string(),
            surah_name_translit: Some("Al-Baqarah".to_string()),
            ayah_range: (AyahNumber::new(255).unwrap(), AyahNumber::new(255).unwrap()),
            arabic_text: text.to_string(),
            text_hash: ContentHash::try_new(HashAlgorithm::Sha256, "ab".repeat(32)).unwrap(),
            edition: edition_ref(),
            translation: None,
            deep_link: "/read/hafs-uthmani@1.0.0/2:255".to_string(),
            page: Some(42),
            juz: Some(3),
        }
    }

    #[test]
    fn quotation_requires_and_exposes_provenance() {
        let q = QuranQuotation::new(parts("اللَّهُ لَا إِلَٰهَ إِلَّا هُوَ")).unwrap();
        assert_eq!(q.edition().version, SemVer::new(1, 0, 0));
        assert_eq!(q.text_hash().hex.len(), 64);
        assert_eq!(q.deep_link(), "/read/hafs-uthmani@1.0.0/2:255");
        assert_eq!(q.ayah_range().0, q.ayah_range().1);
    }

    #[test]
    fn empty_text_is_rejected() {
        let err = QuranQuotation::new(parts("   ")).unwrap_err();
        assert_eq!(err.code().to_string(), "QAI-QUR-0009");
    }

    #[test]
    fn translation_as_canonical_is_impossible_and_requires_translator() {
        // A translation can only be attached as a side reference, and it must be attributed.
        let mut p = parts("text");
        p.translation = Some(TranslationRef {
            slug: "en-sahih".to_string(),
            translator: "   ".to_string(),
            language: "en".parse().unwrap(),
        });
        let err = QuranQuotation::new(p).unwrap_err();
        assert_eq!(err.code().to_string(), "QAI-QUR-0011");

        let mut ok = parts("text");
        ok.translation = Some(TranslationRef {
            slug: "en-sahih".to_string(),
            translator: "Muhammad Muhsin Khan".to_string(),
            language: "en".parse().unwrap(),
        });
        let q = QuranQuotation::new(ok).unwrap();
        assert_eq!(q.translation().unwrap().translator, "Muhammad Muhsin Khan");
    }

    #[test]
    fn quotation_roundtrips_serde() {
        let q = QuranQuotation::new(parts("نص")).unwrap();
        let json = serde_json::to_string(&q).unwrap();
        let back: QuranQuotation = serde_json::from_str(&json).unwrap();
        assert_eq!(back, q);
    }
}
