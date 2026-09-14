//! The canonical structure hierarchy: Edition → Surah → Ayah → Segment → Token.
//!
//! # quran_core::structure
//!
//! Phase 1 stores only the **surface** token. Lemma/root/morphology arrive in
//! Phase 2 and attach to token positions — the canonical text is never
//! re-tokenized (see `acceptance.md` §7).
//!
//! Round-trip guarantee (tested in `quran-corpus`): for every ayah,
//! `join(tokens_by_position, recorded_separators) == ayah.text` byte-for-byte,
//! and `ayah.text[byte_start..byte_end] == token.surface`.

use std::collections::BTreeMap;

use domain::{ContentHash, EditionId, Language, ProvenanceId};
use serde::{Deserialize, Serialize};

use crate::enums::{BasmalaPolicy, RevelationPlace, SajdahKind, SegmentKind};
use crate::numbers::{AyahNumber, SurahNumber, TokenPosition};

/// A surah with its metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Surah {
    /// Owning edition.
    pub edition_id: EditionId,
    /// Surah number, `1..=114`.
    pub number: SurahNumber,
    /// Arabic name.
    pub name_arabic: String,
    /// Transliterated name, if provided.
    pub name_transliteration: Option<String>,
    /// Name translations, keyed by language.
    pub name_translations: BTreeMap<Language, String>,
    /// Number of ayahs in this surah for this edition.
    pub ayah_count: u16,
    /// Meccan or Medinan (Layer-B metadata, never canonical).
    pub revelation_place: Option<RevelationPlace>,
    /// Revelation order, if provided.
    pub revelation_order: Option<u16>,
    /// Basmala policy for this surah.
    pub basmala: BasmalaPolicy,
    /// Number of ruku divisions, if provided.
    pub ruku_count: Option<u16>,
    /// Provenance of the non-canonical metadata above.
    pub metadata_provenance: ProvenanceId,
}

/// One canonical ayah.
///
/// `text` is the canonical surface text of the owning edition version; it is
/// immutable once written (trigger `QAI-QUR-0001`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ayah {
    /// Owning edition.
    pub edition_id: EditionId,
    /// Surah number.
    pub surah: SurahNumber,
    /// Ayah number within the surah (1-based).
    pub number: AyahNumber,
    /// Canonical text, exactly as in the approved source.
    pub text: String,
    /// Hash of `text`.
    pub text_hash: ContentHash,
    /// Grapheme-cluster count of `text`.
    pub char_count: u32,
    /// Number of tokens.
    pub token_count: u16,
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
    /// 1-based index across the whole edition (interval math, Phase 2).
    pub global_ayah_index: u32,
    /// Canonical provenance of this row (the importing source version).
    pub provenance: ProvenanceId,
}

/// A grouping of tokens inside an ayah for display/analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Segment {
    /// Owning edition.
    pub edition_id: EditionId,
    /// Surah number.
    pub surah: SurahNumber,
    /// Ayah number.
    pub ayah: AyahNumber,
    /// Segment index within the ayah, 1-based.
    pub index: u16,
    /// Segment kind.
    pub kind: SegmentKind,
    /// Inclusive token position range.
    pub token_range: (u16, u16),
    /// Provenance (Layer B or C, never canonical).
    pub provenance: ProvenanceId,
}

/// A canonical surface token and its offsets within the ayah text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    /// Owning edition.
    pub edition_id: EditionId,
    /// Surah number.
    pub surah: SurahNumber,
    /// Ayah number.
    pub ayah: AyahNumber,
    /// 1-based position within the ayah.
    pub position: TokenPosition,
    /// Canonical surface form.
    pub surface: String,
    /// Hash of `surface`.
    pub surface_hash: ContentHash,
    /// Grapheme start offset within the ayah text.
    pub char_start: u32,
    /// Grapheme end offset (exclusive).
    pub char_end: u32,
    /// Byte start offset within the ayah text.
    pub byte_start: u32,
    /// Byte end offset (exclusive).
    pub byte_end: u32,
    /// Whether the token is a pause (waqf) mark.
    pub is_pause_mark: bool,
    /// 1-based index across the whole edition.
    pub global_token_index: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::HashAlgorithm;

    fn hash(tag: &str) -> ContentHash {
        ContentHash::try_new(HashAlgorithm::Sha256, tag.repeat(32)).unwrap()
    }

    #[test]
    fn surah_holds_language_keyed_names() {
        let mut names = BTreeMap::new();
        names.insert("en".parse::<Language>().unwrap(), "The Opening".to_string());
        names.insert("fa".parse::<Language>().unwrap(), "فاتحه".to_string());
        let surah = Surah {
            edition_id: EditionId::new(),
            number: SurahNumber::new(1).unwrap(),
            name_arabic: "الفاتحة".to_string(),
            name_transliteration: Some("Al-Fatihah".to_string()),
            name_translations: names,
            ayah_count: 7,
            revelation_place: Some(RevelationPlace::Makki),
            revelation_order: Some(5),
            basmala: BasmalaPolicy::CountedAsFirstAyah,
            ruku_count: Some(1),
            metadata_provenance: ProvenanceId::new(),
        };
        assert_eq!(surah.name_translations.len(), 2);
        assert!(serde_json::to_string(&surah).unwrap().contains("Al-Fatihah"));
    }

    #[test]
    fn token_offsets_are_ordered() {
        let token = Token {
            edition_id: EditionId::new(),
            surah: SurahNumber::new(1).unwrap(),
            ayah: AyahNumber::new(1).unwrap(),
            position: TokenPosition::new(1).unwrap(),
            surface: "بسم".to_string(),
            surface_hash: hash("aa"),
            char_start: 0,
            char_end: 3,
            byte_start: 0,
            byte_end: 6,
            is_pause_mark: false,
            global_token_index: 1,
        };
        assert!(token.byte_end > token.byte_start);
        assert!(token.char_end > token.char_start);
    }
}
