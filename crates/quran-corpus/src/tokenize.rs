//! Lossless whitespace-preserving tokenization (ADR-0105).
//!
//! # quran_corpus::tokenize
//!
//! The tokenizer splits ayah text into surface tokens while recording the
//! exact separator between every pair, so reconstruction is byte-identical:
//! `reconstruct(tokenize(text)) == text` always.
//!
//! Rules:
//! - Unicode whitespace runs are separators, recorded exactly.
//! - Quranic annotation signs (U+06D6..=U+06ED: waqf marks, end-of-ayah,
//!   sajdah, rub-el-hizb) form their own tokens (`is_pause_mark`).
//! - A mark that does not start a new grapheme cluster (a combining mark glued
//!   to a preceding letter) stays attached to its word token, so every token
//!   always spans at least one grapheme cluster and offsets stay valid.
//! - Everything else accumulates into word tokens.
//!
//! Morphological segmentation is Phase 2 and separate; this layer never alters
//! canonical order or content.

use unicode_segmentation::UnicodeSegmentation;

/// A Quranic annotation sign: waqf marks, end-of-ayah, sajdah, rub-el-hizb.
fn is_pause_mark_scalar(ch: char) -> bool {
    matches!(ch, '\u{06D6}'..='\u{06ED}')
}

/// One computed token with offsets into the source ayah text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputedToken {
    /// 1-based position within the ayah (as `u32`; the importer validates the
    /// `u16` range when building rows, so tokenization itself cannot fail).
    pub position: u32,
    /// Surface form, byte-identical to `text[byte_start..byte_end]`.
    pub surface: String,
    /// Grapheme-cluster start offset.
    pub char_start: u32,
    /// Grapheme-cluster end offset (exclusive, always `> char_start`).
    pub char_end: u32,
    /// Byte start offset.
    pub byte_start: u32,
    /// Byte end offset (exclusive, always `> byte_start`).
    pub byte_end: u32,
    /// Whether this token is a standalone annotation sign.
    pub is_pause_mark: bool,
}

/// Tokenization output: tokens plus the exact separators around them.
///
/// `separators` has `tokens.len() + 1` entries: `separators[0]` is the leading
/// text before token 1, `separators[i]` follows token `i`, and the last entry
/// is the trailing text.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TokenizedAyah {
    /// Tokens in canonical order.
    pub tokens: Vec<ComputedToken>,
    /// Exact separators; `separators.len() == tokens.len() + 1`.
    pub separators: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ClusterKind {
    Separator,
    Word,
    Mark,
}

/// Split ayah text into surface tokens with lossless separators.
pub fn tokenize(text: &str) -> TokenizedAyah {
    let mut tokens = Vec::new();
    let mut separators = vec![String::new()];
    let mut current = String::new();
    let mut current_start_byte = 0_u32;
    let mut current_start_cluster = 0_u32;
    let mut current_kind: Option<ClusterKind> = None;
    let mut cluster_index = 0_u32;
    let mut position = 0_u32;

    for (byte, cluster) in text.grapheme_indices(true) {
        let kind = if cluster.chars().all(|ch| ch.is_whitespace()) {
            ClusterKind::Separator
        } else if cluster.chars().all(is_pause_mark_scalar) {
            ClusterKind::Mark
        } else {
            ClusterKind::Word
        };
        match kind {
            ClusterKind::Separator => {
                separators.last_mut().expect("at least one separator").push_str(cluster);
            }
            _ => {
                if current_kind.is_some_and(|current| current != kind) {
                    position += 1;
                    tokens.push(ComputedToken {
                        position,
                        surface: std::mem::take(&mut current),
                        char_start: current_start_cluster,
                        char_end: cluster_index,
                        byte_start: current_start_byte,
                        byte_end: byte as u32,
                        is_pause_mark: current_kind == Some(ClusterKind::Mark),
                    });
                    separators.push(String::new());
                    current_kind = None;
                }
                if current_kind.is_none() {
                    current_start_byte = byte as u32;
                    current_start_cluster = cluster_index;
                    current_kind = Some(kind);
                }
                current.push_str(cluster);
            }
        }
        cluster_index += 1;
    }
    if current_kind.is_some() {
        position += 1;
        tokens.push(ComputedToken {
            position,
            surface: current,
            char_start: current_start_cluster,
            char_end: cluster_index,
            byte_start: current_start_byte,
            byte_end: text.len() as u32,
            is_pause_mark: current_kind == Some(ClusterKind::Mark),
        });
        separators.push(String::new());
    }
    TokenizedAyah { tokens, separators }
}

/// Rebuild ayah text from tokens and separators, byte-for-byte.
///
/// Lenient about lengths: missing entries are treated as empty.
pub fn reconstruct(tokens: &[ComputedToken], separators: &[String]) -> String {
    let mut out = String::new();
    for (index, token) in tokens.iter().enumerate() {
        if let Some(separator) = separators.get(index) {
            out.push_str(separator);
        }
        out.push_str(&token.surface);
    }
    if let Some(trailing) = separators.get(tokens.len()) {
        out.push_str(trailing);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_words_and_records_separators() {
        let tokenized = tokenize("ب ت  ث");
        let surfaces: Vec<_> =
            tokenized.tokens.iter().map(|token| token.surface.as_str()).collect();
        assert_eq!(surfaces, ["ب", "ت", "ث"]);
        assert_eq!(tokenized.separators, ["", " ", "  ", ""]);
        assert_eq!(reconstruct(&tokenized.tokens, &tokenized.separators), "ب ت  ث");
    }

    #[test]
    fn pause_marks_become_their_own_tokens() {
        let tokenized = tokenize("ب ۝ ت");
        assert_eq!(tokenized.tokens.len(), 3);
        assert!(tokenized.tokens[1].is_pause_mark);
        assert_eq!(tokenized.tokens[1].surface, "۝");
        assert!(!tokenized.tokens[0].is_pause_mark);
        assert_eq!(reconstruct(&tokenized.tokens, &tokenized.separators), "ب ۝ ت");
    }

    #[test]
    fn attached_mark_stays_with_its_word() {
        // U+06D6 is a combining mark: it joins the preceding letter's cluster.
        let text = "ب\u{6D6} ت";
        let tokenized = tokenize(text);
        assert_eq!(tokenized.tokens.len(), 2);
        assert!(!tokenized.tokens[0].is_pause_mark);
        assert_eq!(reconstruct(&tokenized.tokens, &tokenized.separators), text);
        for token in &tokenized.tokens {
            assert!(token.char_end > token.char_start);
            assert!(token.byte_end > token.byte_start);
        }
    }

    #[test]
    fn offsets_slice_back_to_surfaces() {
        let text = "ٱللَّهِ ۝ رَبِّ";
        let tokenized = tokenize(text);
        for token in &tokenized.tokens {
            let (start, end) = (token.byte_start as usize, token.byte_end as usize);
            assert_eq!(&text[start..end], token.surface);
            assert!(token.char_end > token.char_start);
        }
        let positions: Vec<_> = tokenized.tokens.iter().map(|t| t.position).collect();
        assert_eq!(positions, [1, 2, 3]);
        assert_eq!(tokenized.separators.len(), tokenized.tokens.len() + 1);
    }

    #[test]
    fn empty_and_whitespace_only_are_lossless() {
        for text in ["", " ", "   "] {
            let tokenized = tokenize(text);
            assert!(tokenized.tokens.is_empty());
            assert_eq!(reconstruct(&tokenized.tokens, &tokenized.separators), text);
        }
    }
}
