//! Unicode auditor for canonical text (ADR-0104).
//!
//! # quran_corpus::unicode
//!
//! Decides whether stored text is corrupt: normalization-form detection,
//! forbidden code points (controls, BOM, bidi overrides, zero-width joiners,
//! private-use, noncharacters), and the expected-block check behind QV-009.
//!
//! Unassigned code points cannot be detected without Unicode tables and are
//! therefore not checked here; everything else on the QV-008 list is exact.

use quran_core::enums::UnicodeForm;

/// Detect the normalization form of `text`, if it is purely one form.
pub fn normalization_form(text: &str) -> Option<UnicodeForm> {
    if unicode_normalization::is_nfc(text) {
        Some(UnicodeForm::Nfc)
    } else if unicode_normalization::is_nfd(text) {
        Some(UnicodeForm::Nfd)
    } else if unicode_normalization::is_nfkc(text) {
        Some(UnicodeForm::Nfkc)
    } else if unicode_normalization::is_nfkd(text) {
        Some(UnicodeForm::Nfkd)
    } else {
        None
    }
}

/// One forbidden code point found in text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForbiddenPoint {
    /// Byte offset of the character.
    pub byte_offset: u32,
    /// The offending character.
    pub character: char,
    /// Machine-readable reason.
    pub reason: &'static str,
}

fn forbidden_reason(ch: char) -> Option<&'static str> {
    let value = ch as u32;
    if value <= 0x1F || (0x7F..=0x9F).contains(&value) {
        Some("control-character")
    } else if ch == '\u{FEFF}' {
        Some("byte-order-mark")
    } else if matches!(ch, '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
    {
        Some("bidi-control")
    } else if matches!(ch, '\u{200C}' | '\u{200D}') {
        Some("zero-width-joiner")
    } else if matches!(ch, '\u{E000}'..='\u{F8FF}' | '\u{F0000}'..='\u{FFFFD}' | '\u{100000}'..='\u{10FFFD}')
    {
        Some("private-use")
    } else if matches!(ch, '\u{FDD0}'..='\u{FDEF}')
        || (value & 0xFFFF) >= 0xFFFE && value <= 0x10FFFF
    {
        Some("noncharacter")
    } else {
        None
    }
}

/// Find every forbidden code point in `text`, in order.
pub fn find_forbidden(text: &str) -> Vec<ForbiddenPoint> {
    let mut points = Vec::new();
    for (byte_offset, ch) in text.char_indices() {
        if let Some(reason) = forbidden_reason(ch) {
            points.push(ForbiddenPoint { byte_offset: byte_offset as u32, character: ch, reason });
        }
    }
    points
}

/// Whether `ch` is in an expected block for canonical Quran text: the Arabic
/// blocks (0600–06FF, 0750–077F, 0870–089F, 08A0–08FF, 10EFD–10EFF) or the
/// plain space separating tokens. Anything else is reported by QV-009 with
/// its location; presentation forms are deliberately excluded.
pub fn is_expected_code_point(ch: char) -> bool {
    ch == ' '
        || matches!(ch,
            '\u{0600}'..='\u{06FF}'
            | '\u{0750}'..='\u{077F}'
            | '\u{0870}'..='\u{089F}'
            | '\u{08A0}'..='\u{08FF}'
            | '\u{10EFD}'..='\u{10EFF}')
}

/// The first out-of-block code point in `text`, if any.
pub fn first_unexpected(text: &str) -> Option<(u32, char)> {
    text.char_indices()
        .find(|(_, ch)| !is_expected_code_point(*ch))
        .map(|(byte, ch)| (byte as u32, ch))
}

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_normalization::UnicodeNormalization;

    #[test]
    fn detects_normalization_forms() {
        assert_eq!(normalization_form("بسم"), Some(UnicodeForm::Nfc));
        let nfd: String = "أ".nfd().collect();
        assert_ne!(nfd, "أ");
        assert_eq!(normalization_form(&nfd), Some(UnicodeForm::Nfd));
    }

    #[test]
    fn finds_each_forbidden_class() {
        assert!(find_forbidden("بسم").is_empty());
        let bom = find_forbidden("\u{FEFF}ب");
        assert_eq!(bom.len(), 1);
        assert_eq!(bom[0].reason, "byte-order-mark");
        let bidi = find_forbidden("ب\u{202E}ت");
        assert_eq!(bidi.len(), 1);
        assert_eq!(bidi[0].reason, "bidi-control");
        let zwj = find_forbidden("ب\u{200D}ت");
        assert_eq!(zwj.len(), 1);
        assert_eq!(zwj[0].character, '\u{200D}');
        let control = find_forbidden("ب\tt");
        assert_eq!(control.len(), 1);
        assert_eq!(control[0].reason, "control-character");
        let pua = find_forbidden("ب\u{E000}ت");
        assert_eq!(pua[0].reason, "private-use");
        // Offsets are byte offsets into the original text.
        assert_eq!(bidi[0].byte_offset, 2);
    }

    #[test]
    fn block_check_accepts_arabic_and_space_only() {
        assert!(is_expected_code_point('ب'));
        assert!(is_expected_code_point('أ'));
        assert!(is_expected_code_point(' '));
        assert!(is_expected_code_point('۝'));
        assert!(!is_expected_code_point('a'));
        assert_eq!(first_unexpected("ب a"), Some((3, 'a')));
        assert_eq!(first_unexpected("بسم"), None);
    }
}
