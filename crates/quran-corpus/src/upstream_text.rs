//! Upstream text readers: `quran-api` `database/chapterverse` and
//! `database/linebyline` files (ADR-0101).
//!
//! # quran_corpus::upstream_text
//!
//! Every upstream text file has the same outer shape, measured across all 984
//! vendored files (`fixtures/upstream/README.md`):
//!
//! ```text
//! <content lines…>
//! {
//!     "name": "<slug>", …  (10-line JSON metadata trailer)
//! }
//! ```
//!
//! - `chapterverse`: one `surah|ayah|text` row per verse (6236 in every
//!   vendored file). Bare non-row lines would be collected as [`Annotation`]s —
//!   none occur in the mirror, but the path is tested and kept so a future
//!   upstream change fails safe instead of misfiling notes as verses.
//! - `linebyline`: bare ordered text lines (6236 in every vendored file,
//!   empties preserved).
//!
//! Intra-verse carriage returns (`\r`, used as a separator by four vendored
//! translations) are kept verbatim and counted (`cr_lines`); canonical import
//! validation (QV-008) rejects control points at promotion time.
//!
//! These readers produce **validated raw material**, not canonical editions:
//! promotion into the corpus still goes through the staged importer,
//! validation, and editorial sign-off. Licence state of every file is unknown
//! (see the fixtures README); parsing a file grants no redistribution rights.

use crate::error::CorpusError;

/// Adapter name used in [`CorpusError::AdapterFailed`] reports.
pub const NAME: &str = "quran-api-text";

/// Which upstream text layout a file uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextFormat {
    /// `surah|ayah|text` rows.
    ChapterVerse,
    /// Bare ordered text lines.
    LineByLine,
}

/// One validated `surah|ayah|text` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterVerseLine {
    /// Surah number (1–114).
    pub surah: u16,
    /// Ayah number (≥ 1).
    pub ayah: u32,
    /// Verse text, verbatim (never empty/whitespace-only).
    pub text: String,
    /// 1-based line number in the source file.
    pub line_no: usize,
}

/// A non-verse content line (translator note) kept out of the verse stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    /// 1-based line number in the source file.
    pub line_no: usize,
    /// Note text, verbatim.
    pub text: String,
}

/// The JSON metadata trailer every upstream text file carries.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct TrailerMeta {
    /// Upstream slug; must equal the file's `<slug>.txt` stem.
    pub name: String,
    /// Upstream author credit (may be empty).
    #[serde(default)]
    pub author: String,
    /// Upstream language label (free text, may be empty).
    #[serde(default)]
    pub language: String,
    /// `rtl` or `ltr` (may be empty on old entries).
    #[serde(default)]
    pub direction: String,
    /// Upstream-declared source (may be empty).
    #[serde(default)]
    pub source: String,
    /// Provenance/quality comments (may be empty).
    #[serde(default)]
    pub comments: String,
    /// Canonical download link (may be empty).
    #[serde(default)]
    pub link: String,
    /// Minified download link (may be empty).
    #[serde(default)]
    pub linkmin: String,
}

/// A parsed `chapterverse` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterVerseFile {
    /// Upstream slug (file stem, cross-checked against the trailer).
    pub slug: String,
    /// Validated verse rows in file order.
    pub verses: Vec<ChapterVerseLine>,
    /// Non-verse translator-note lines in file order.
    pub annotations: Vec<Annotation>,
    /// Content lines containing a carriage return (`\r`): four vendored
    /// translations use it as an intra-verse separator. Kept verbatim here;
    /// canonical import validation (QV-008) rejects control points later.
    pub cr_lines: usize,
    /// The file's metadata trailer.
    pub trailer: TrailerMeta,
}

/// A parsed `linebyline` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineByLineFile {
    /// Upstream slug (file stem, cross-checked against the trailer).
    pub slug: String,
    /// Ordered text lines, verbatim (empty lines preserved).
    pub lines: Vec<String>,
    /// Content lines containing a carriage return (`\r`), kept verbatim.
    pub cr_lines: usize,
    /// The file's metadata trailer.
    pub trailer: TrailerMeta,
}

/// Content lines with their 1-based file line numbers.
type ContentLines<'a> = Vec<(usize, &'a str)>;

/// Split raw file text into `(content lines, trailer)`. The trailer starts at
/// the first line that is exactly `{`; anything after it must parse as JSON.
fn split_trailer<'a>(
    slug: &str,
    input: &'a str,
) -> Result<(ContentLines<'a>, TrailerMeta), CorpusError> {
    let fail = |detail: String| CorpusError::AdapterFailed { adapter: NAME, detail };
    let lines: Vec<&str> = input.split('\n').collect();
    let start = lines.iter().position(|line| *line == "{").ok_or_else(|| {
        fail(format!("`{slug}`: no JSON metadata trailer (no line that is exactly `{{`)"))
    })?;
    let trailer: TrailerMeta = serde_json::from_str(&lines[start..].join("\n"))
        .map_err(|err| fail(format!("`{slug}`: malformed metadata trailer: {err}")))?;
    if trailer.name != slug {
        return Err(fail(format!(
            "`{slug}`: trailer names `{}` (expected the file slug)",
            trailer.name
        )));
    }
    let content =
        lines[..start].iter().enumerate().map(|(index, line)| (index + 1, *line)).collect();
    Ok((content, trailer))
}

/// Parse a `chapterverse` file: `surah|ayah|text` rows in strictly increasing
/// `(surah, ayah)` order, surah in 1–114, non-empty text. Bare lines that are
/// not verse rows become [`Annotation`]s.
pub fn parse_chapterverse(slug: &str, input: &str) -> Result<ChapterVerseFile, CorpusError> {
    let fail = |detail: String| CorpusError::AdapterFailed { adapter: NAME, detail };
    let (content, trailer) = split_trailer(slug, input)?;
    let mut verses = Vec::new();
    let mut annotations = Vec::new();
    let mut previous: Option<(u16, u32)> = None;
    for (line_no, line) in content {
        let mut parts = line.splitn(3, '|');
        match (parts.next(), parts.next(), parts.next()) {
            (Some(surah_text), Some(ayah_text), Some(text)) => {
                let surah: u16 = surah_text.parse().map_err(|_| {
                    fail(format!("`{slug}` line {line_no}: bad surah `{surah_text}`"))
                })?;
                let ayah: u32 = ayah_text.parse().map_err(|_| {
                    fail(format!("`{slug}` line {line_no}: bad ayah `{ayah_text}`"))
                })?;
                if !(1..=114).contains(&surah) {
                    return Err(fail(format!(
                        "`{slug}` line {line_no}: surah {surah} outside 1–114"
                    )));
                }
                if ayah == 0 || text.trim().is_empty() {
                    return Err(fail(format!(
                        "`{slug}` line {line_no}: empty ayah number or empty text"
                    )));
                }
                if let Some((prev_surah, prev_ayah)) = previous
                    && (surah, ayah) <= (prev_surah, prev_ayah)
                {
                    return Err(fail(format!(
                        "`{slug}` line {line_no}: `{surah}|{ayah}` is not after \
                         `{prev_surah}|{prev_ayah}`"
                    )));
                }
                previous = Some((surah, ayah));
                verses.push(ChapterVerseLine { surah, ayah, text: text.to_string(), line_no });
            }
            _ => annotations.push(Annotation { line_no, text: line.to_string() }),
        }
    }
    if verses.is_empty() {
        return Err(fail(format!("`{slug}`: no verse rows before the trailer")));
    }
    let cr_lines = verses.iter().filter(|row| row.text.contains('\r')).count();
    Ok(ChapterVerseFile { slug: slug.to_string(), verses, annotations, cr_lines, trailer })
}

/// Parse a `linebyline` file: ordered text lines kept verbatim (empty lines
/// preserved — one translation uses blank lines as paragraph spacing).
pub fn parse_linebyline(slug: &str, input: &str) -> Result<LineByLineFile, CorpusError> {
    let fail = |detail: String| CorpusError::AdapterFailed { adapter: NAME, detail };
    let (content, trailer) = split_trailer(slug, input)?;
    if content.is_empty() {
        return Err(fail(format!("`{slug}`: no text lines before the trailer")));
    }
    let cr_lines = content.iter().filter(|(_, line)| line.contains('\r')).count();
    Ok(LineByLineFile {
        slug: slug.to_string(),
        lines: content.into_iter().map(|(_, line)| line.to_string()).collect(),
        cr_lines,
        trailer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAPTERVERSE: &str = "1|1|Bismillah\n1|2|Praise\n2|1|Alif\n{\n    \"name\": \"eng-x\",\n    \"author\": \"A\",\n    \"language\": \"English\",\n    \"direction\": \"ltr\",\n    \"source\": \"\",\n    \"comments\": \"\",\n    \"link\": \"\",\n    \"linkmin\": \"\"\n}";

    const LINEBYLINE: &str = "Bismillah\n\nPraise\n{\n    \"name\": \"eng-x\",\n    \"author\": \"A\",\n    \"language\": \"English\",\n    \"direction\": \"ltr\",\n    \"source\": \"\",\n    \"comments\": \"\",\n    \"link\": \"\",\n    \"linkmin\": \"\"\n}";

    #[test]
    fn chapterverse_parses_rows_in_order() {
        let file = parse_chapterverse("eng-x", CHAPTERVERSE).unwrap();
        assert_eq!(file.slug, "eng-x");
        assert_eq!(file.verses.len(), 3);
        assert_eq!(file.verses[0].surah, 1);
        assert_eq!(file.verses[2].ayah, 1);
        assert_eq!(file.verses[2].line_no, 3);
        assert!(file.annotations.is_empty());
        assert_eq!(file.cr_lines, 0);
        assert_eq!(file.trailer.author, "A");
    }

    #[test]
    fn chapterverse_keeps_notes_out_of_the_verse_stream() {
        let input = "1|1|Bismillah\nA translator note\n1|2|Praise\n{\n    \"name\": \"eng-x\"\n}";
        let file = parse_chapterverse("eng-x", input).unwrap();
        assert_eq!(file.verses.len(), 2);
        assert_eq!(file.annotations.len(), 1);
        assert_eq!(file.annotations[0].line_no, 2);
        assert_eq!(file.annotations[0].text, "A translator note");
    }

    #[test]
    fn chapterverse_rejects_bad_rows_and_bad_trailers() {
        // Non-increasing reference.
        let bad = "1|2|Praise\n1|1|Bismillah\n{\n    \"name\": \"eng-x\"\n}";
        assert!(parse_chapterverse("eng-x", bad).is_err());
        // Surah out of range.
        let bad = "115|1|Nope\n{\n    \"name\": \"eng-x\"\n}";
        assert!(parse_chapterverse("eng-x", bad).is_err());
        // Empty text.
        let bad = "1|1|   \n{\n    \"name\": \"eng-x\"\n}";
        assert!(parse_chapterverse("eng-x", bad).is_err());
        // Missing trailer.
        assert!(parse_chapterverse("eng-x", "1|1|Bismillah\n").is_err());
        // Trailer names another slug.
        let bad = "1|1|Bismillah\n{\n    \"name\": \"eng-other\"\n}";
        assert!(parse_chapterverse("eng-x", bad).is_err());
        // No verse rows at all.
        assert!(parse_chapterverse("eng-x", "{\n    \"name\": \"eng-x\"\n}").is_err());
    }

    #[test]
    fn linebyline_preserves_order_and_empties() {
        let file = parse_linebyline("eng-x", LINEBYLINE).unwrap();
        assert_eq!(file.lines, vec!["Bismillah".to_string(), String::new(), "Praise".to_string()]);
        assert_eq!(file.cr_lines, 0);
        assert_eq!(file.trailer.direction, "ltr");
        assert!(parse_linebyline("eng-x", "{\n    \"name\": \"eng-x\"\n}").is_err());
        assert!(parse_linebyline("eng-x", "Bismillah\n").is_err());
    }

    #[test]
    fn carriage_returns_are_counted_but_kept_verbatim() {
        let input = "1|1|A\rb\n{\n    \"name\": \"eng-x\"\n}";
        let file = parse_chapterverse("eng-x", input).unwrap();
        assert_eq!(file.cr_lines, 1);
        assert_eq!(file.verses[0].text, "A\rb");
        let file = parse_linebyline("eng-x", input).unwrap();
        assert_eq!(file.cr_lines, 1);
    }
}
