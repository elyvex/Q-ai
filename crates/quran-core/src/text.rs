//! Text utilities for canonical text handling.
//!
//! # quran_core::text
//!
//! `char_count` in the Quran domain means **grapheme clusters**, not Unicode
//! scalar values (ADR-0104). A canonical ayah's `char_count`, and every stored
//! token offset, are expressed in these units.

use unicode_segmentation::UnicodeSegmentation;

/// Count grapheme clusters in `text`.
///
/// Grapheme clusters are the user-perceived characters: combining marks and
/// their base letter count as one. This is the unit of `Ayah::char_count` and
/// of `Token::char_start`/`char_end`.
pub fn grapheme_count(text: &str) -> u32 {
    text.graphemes(true).count() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_ascii() {
        assert_eq!(grapheme_count(""), 0);
        assert_eq!(grapheme_count("abc"), 3);
        assert_eq!(grapheme_count("a b"), 3);
    }

    #[test]
    fn counts_combining_sequence_as_one() {
        // U+0644 + U+064E (fatha) is one grapheme cluster, not two scalars.
        let text = "\u{0644}\u{064E}";
        assert_eq!(grapheme_count(text), 1);
    }

    #[test]
    fn counts_superscript_alef_sequence_as_one() {
        // A base letter plus a superscript alef forms one grapheme cluster.
        let text = "\u{0627}\u{0670}";
        assert_eq!(grapheme_count(text), 1);
    }
}
