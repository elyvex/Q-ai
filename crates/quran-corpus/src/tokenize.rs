//! Lossless whitespace-preserving tokenization (ADR-0105).
//!
//! # quran_corpus::tokenize
//!
//! The tokenizer splits ayah text into surface tokens while recording the
//! exact separator between every pair, so reconstruction is byte-identical:
//! `reconstruct(tokenize(text)) == text` always.
//!
//! Rules:
//! - Unicode whitespace runs are separators, recorded exactly, and always end
//!   the open token (each separator row sits `after_position` of a token).
//! - Quranic annotation signs (U+06D6..=U+06ED: waqf marks, end-of-ayah,
//!   sajdah, rub-el-hizb) form their own tokens (`is_pause_mark`), except a
//!   mark glued to a preceding word character stays attached to that word.
//! - Everything else accumulates into word tokens.
//! - Offsets are mapped back onto grapheme clusters, so every token spans at
//!   least one cluster and all offsets are valid boundaries.
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
enum CharKind {
    Separator,
    Word,
    Mark,
}

fn char_kind(ch: char) -> CharKind {
    if ch.is_whitespace() {
        CharKind::Separator
    } else if is_pause_mark_scalar(ch) {
        CharKind::Mark
    } else {
        CharKind::Word
    }
}

/// The cluster index containing `byte` (`starts` holds cluster start bytes).
fn cluster_of(starts: &[usize], byte: usize) -> u32 {
    starts.partition_point(|&start| start <= byte) as u32 - 1
}

/// Split ayah text into surface tokens with lossless separators.
///
/// Splitting is per-character (whitespace ends tokens; a mark following a word
/// character attaches to it; otherwise marks form their own tokens), while
/// offsets are expressed in grapheme clusters so they stay valid boundaries.
pub fn tokenize(text: &str) -> TokenizedAyah {
    let starts: Vec<usize> = text.grapheme_indices(true).map(|(byte, _)| byte).collect();
    let mut tokens = Vec::new();
    let mut separators = vec![String::new()];
    let mut current = String::new();
    let mut current_start_byte = 0_usize;
    let mut current_start_cluster = 0_u32;
    let mut current_is_mark = false;
    let mut position = 0_u32;

    let mut flush = |current: &mut String,
                     tokens: &mut Vec<ComputedToken>,
                     separators: &mut Vec<String>,
                     position: &mut u32,
                     start_byte: usize,
                     start_cluster: u32,
                     is_mark: bool| {
        if current.is_empty() {
            return;
        }
        *position += 1;
        let byte_end = start_byte + current.len();
        tokens.push(ComputedToken {
            position: *position,
            surface: std::mem::take(current),
            char_start: start_cluster,
            char_end: cluster_of(&starts, byte_end - 1) + 1,
            byte_start: start_byte as u32,
            byte_end: byte_end as u32,
            is_pause_mark: is_mark,
        });
        separators.push(String::new());
    };

    for (byte, ch) in text.char_indices() {
        match char_kind(ch) {
            CharKind::Separator => {
                flush(
                    &mut current,
                    &mut tokens,
                    &mut separators,
                    &mut position,
                    current_start_byte,
                    current_start_cluster,
                    current_is_mark,
                );
                separators.last_mut().expect("at least one separator").push(ch);
            }
            CharKind::Mark => {
                if !current.is_empty() && !current_is_mark {
                    flush(
                        &mut current,
                        &mut tokens,
                        &mut separators,
                        &mut position,
                        current_start_byte,
                        current_start_cluster,
                        current_is_mark,
                    );
                }
                if current.is_empty() {
                    current_start_byte = byte;
                    current_start_cluster = cluster_of(&starts, byte);
                    current_is_mark = true;
                }
                current.push(ch);
            }
            CharKind::Word => {
                if !current.is_empty() && current_is_mark {
                    flush(
                        &mut current,
                        &mut tokens,
                        &mut separators,
                        &mut position,
                        current_start_byte,
                        current_start_cluster,
                        current_is_mark,
                    );
                }
                if current.is_empty() {
                    current_start_byte = byte;
                    current_start_cluster = cluster_of(&starts, byte);
                    current_is_mark = false;
                }
                current.push(ch);
            }
        }
    }
    flush(
        &mut current,
        &mut tokens,
        &mut separators,
        &mut position,
        current_start_byte,
        current_start_cluster,
        current_is_mark,
    );
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
