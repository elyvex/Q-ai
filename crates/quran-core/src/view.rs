//! Read views: attributed translations, ayah views, context (D1.6).
//!
//! # quran_core::view
//!
//! Principle 5 is structural here: `AyahView.canonical` is a `QuranQuotation`
//! (Arabic only) and there is no variant in which a translation can occupy the
//! canonical slot. `AttributedTranslation` has no constructor without a
//! non-empty translator and an edition reference.

use domain::{Language, LicenseRecord};
use serde::{Deserialize, Serialize};

use crate::enums::ContextBoundary;
use crate::error::QuranError;
use crate::numbers::{AyahNumber, SurahNumber};
use crate::quotation::QuranQuotation;
use crate::structure::{Surah, Token};

/// A translation carried *alongside* canonical text, never instead of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributedTranslation {
    translator: String,
    edition_ref: String,
    language: Language,
    text: String,
    license: LicenseRecord,
}

impl AttributedTranslation {
    /// Construct an attributed translation (principle-5 guard).
    pub fn new(
        translator: String,
        edition_ref: String,
        language: Language,
        text: String,
        license: LicenseRecord,
    ) -> Result<Self, QuranError> {
        if translator.trim().is_empty() {
            return Err(QuranError::MissingTranslator);
        }
        if edition_ref.trim().is_empty() {
            return Err(QuranError::EmptyField { field: "edition_ref" });
        }
        Ok(Self { translator, edition_ref, language, text, license })
    }

    /// The named human translator.
    pub fn translator(&self) -> &str {
        &self.translator
    }

    /// The translation edition reference (`slug@version`).
    pub fn edition_ref(&self) -> &str {
        &self.edition_ref
    }

    /// The translation language.
    pub fn language(&self) -> &Language {
        &self.language
    }

    /// The translated text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The translation license.
    pub fn license(&self) -> &LicenseRecord {
        &self.license
    }
}

/// An optional word-level gloss, explicitly a separate attributed dataset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributedGloss {
    /// Gloss dataset id.
    pub dataset: String,
    /// Gloss language.
    pub language: Language,
    /// The gloss text.
    pub gloss: String,
}

/// What to fetch alongside an ayah.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AyahOptions {
    /// Translation edition slugs to attach.
    pub translations: Vec<String>,
    /// Attach word glosses.
    pub glosses: bool,
    /// Attach surface tokens.
    pub tokens: bool,
}

/// One ayah as served: canonical Arabic plus attributed extras.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AyahView {
    /// Canonical Arabic quotation with edition identity and hash.
    pub canonical: QuranQuotation,
    /// Requested translations, each attributed.
    pub translations: Vec<AttributedTranslation>,
    /// Word glosses, when requested and available.
    pub word_glosses: Option<Vec<AttributedGloss>>,
    /// Surface tokens, when requested.
    pub tokens: Option<Vec<Token>>,
}

/// How far context extends around a focal ayah.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSpec {
    /// Ayahs before the focal ayah.
    pub before: u16,
    /// Ayahs after the focal ayah.
    pub after: u16,
    /// Context never crosses this boundary.
    pub boundary: ContextBoundary,
    /// Include the surah header record.
    pub include_surah_header: bool,
    /// Hard cap on total ayahs (§40 memory-bounded results).
    pub max_ayahs: u16,
}

/// Ayahs around a focal ayah, bounded by canonical structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextView {
    /// The focal ayah.
    pub focal: AyahView,
    /// Ayahs before (ascending order).
    pub before: Vec<AyahView>,
    /// Ayahs after (ascending order).
    pub after: Vec<AyahView>,
    /// Surah header, when requested.
    pub surah: Option<Surah>,
    /// Canonical reference of the focal ayah.
    pub canonical_reference: String,
    /// Global index range covered, inclusive.
    pub global_range: (u32, u32),
}

/// A resolved `(surah, ayah)` location with its global index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AyahLocation {
    /// Surah number.
    pub surah: SurahNumber,
    /// Ayah number.
    pub ayah: AyahNumber,
    /// 1-based global index.
    pub global: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Diagnostic;
    use domain::LicenseStatus;

    fn license() -> LicenseRecord {
        LicenseRecord {
            status: LicenseStatus::PublicDomain,
            spdx_id: None,
            name: None,
            url: None,
            attribution_required: false,
            redistribution_allowed: true,
            export_allowed: true,
            notes: None,
        }
    }

    #[test]
    fn translation_requires_translator_and_edition() {
        let err = AttributedTranslation::new(
            "   ".into(),
            "en-sahih@1.0.0".into(),
            "en".parse().unwrap(),
            "text".into(),
            license(),
        )
        .unwrap_err();
        assert_eq!(err.code().to_string(), "QAI-QUR-0011");
        let err = AttributedTranslation::new(
            "Someone".into(),
            "  ".into(),
            "en".parse().unwrap(),
            "text".into(),
            license(),
        )
        .unwrap_err();
        assert_eq!(err.code().to_string(), "QAI-QUR-0009");
    }

    #[test]
    fn attributed_translation_exposes_attribution() {
        let translation = AttributedTranslation::new(
            "Someone".into(),
            "en-sahih@1.0.0".into(),
            "en".parse().unwrap(),
            "text".into(),
            license(),
        )
        .unwrap();
        assert_eq!(translation.translator(), "Someone");
        assert_eq!(translation.edition_ref(), "en-sahih@1.0.0");
    }
}
