//! Hand-written recursive-descent parser for the frozen reference grammar.
//!
//! The parser works on borrowed `&str` slices only; the only allocations are the
//! owned edition slug (in the output) and error details. It never panics on
//! user input: every malformed reference yields a `QAI-QUR-01xx` error.

use domain::SemVer;

use crate::enums::{EditionSelector, is_valid_slug};
use crate::error::{DiagnosticCode, QuranError, codes};
use crate::numbers::{AyahNumber, SurahNumber, TokenPosition};
use crate::reference::{DivisionKind, QuranRef};

/// Maximum accepted reference length in bytes (input hygiene, not grammar).
const MAX_REFERENCE_LEN: usize = 256;

/// Maximum `:`-separated segments accepted (grammar needs at most 5).
const MAX_SEGMENTS: usize = 6;

fn fail(code: DiagnosticCode, input: &str, detail: impl Into<String>) -> QuranError {
    QuranError::InvalidReference { code, input: input.to_string(), detail: detail.into() }
}

/// Strict ASCII decimal without sign, whitespace, or leading `+`/`-`.
fn parse_number(field: &str) -> Option<u64> {
    if field.is_empty() {
        return None;
    }
    let mut value: u64 = 0;
    for byte in field.bytes() {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(u64::from(byte - b'0'))?;
    }
    Some(value)
}

fn surah_of(field: &str, input: &str) -> Result<SurahNumber, QuranError> {
    let raw = parse_number(field).unwrap_or(0);
    SurahNumber::new(raw.try_into().unwrap_or(0)).map_err(|_| {
        fail(codes::INVALID_SURAH, input, format!("`{field}` is not a surah number (1..=114)"))
    })
}

fn ayah_of(field: &str, input: &str) -> Result<AyahNumber, QuranError> {
    let raw = parse_number(field).unwrap_or(0);
    AyahNumber::new(raw.try_into().unwrap_or(0)).map_err(|_| {
        fail(codes::INVALID_AYAH, input, format!("`{field}` is not a valid ayah number"))
    })
}

fn position_of(field: &str, input: &str) -> Result<TokenPosition, QuranError> {
    let raw = parse_number(field).unwrap_or(0);
    TokenPosition::new(raw.try_into().unwrap_or(0)).map_err(|_| {
        fail(
            codes::INVALID_REFERENCE_POSITION,
            input,
            format!("`{field}` is not a valid token position"),
        )
    })
}

fn division_kind(field: &str) -> Option<DivisionKind> {
    match field {
        "juz" => Some(DivisionKind::Juz),
        "hizb" => Some(DivisionKind::Hizb),
        "rub" => Some(DivisionKind::Rub),
        "manzil" => Some(DivisionKind::Manzil),
        "page" => Some(DivisionKind::Page),
        "ruku" => Some(DivisionKind::Ruku),
        "sajdah" => Some(DivisionKind::Sajdah),
        _ => None,
    }
}

fn edition_of(field: &str, input: &str) -> Result<EditionSelector, QuranError> {
    if field.is_empty() {
        return Err(fail(codes::UNEXPECTED_INPUT, input, "empty edition segment"));
    }
    match field.split_once('@') {
        None => {
            if !is_valid_slug(field) {
                return Err(fail(
                    codes::INVALID_EDITION,
                    input,
                    format!("`{field}` is not a valid edition slug"),
                ));
            }
            Ok(EditionSelector::Slug(field.to_string()))
        }
        Some((slug, version)) => {
            if !is_valid_slug(slug) {
                return Err(fail(
                    codes::INVALID_EDITION,
                    input,
                    format!("`{slug}` is not a valid edition slug"),
                ));
            }
            let version = version.parse::<SemVer>().map_err(|_| {
                fail(
                    codes::INVALID_EDITION_VERSION,
                    input,
                    format!("`@{version}` is not a MAJOR.MINOR.PATCH version"),
                )
            })?;
            Ok(EditionSelector::Pinned { slug: slug.to_string(), version })
        }
    }
}

fn division(
    edition: EditionSelector,
    kind: DivisionKind,
    field: &str,
    input: &str,
) -> Result<QuranRef, QuranError> {
    let number = parse_number(field)
        .and_then(|value| value.try_into().ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| {
            fail(
                codes::INVALID_DIVISION_NUMBER,
                input,
                format!("`{field}` is not a positive division number"),
            )
        })?;
    Ok(QuranRef::Division { edition, kind, number })
}

fn parse_range(
    edition: EditionSelector,
    locator: &[&str],
    input: &str,
) -> Result<QuranRef, QuranError> {
    let malformed = || {
        fail(codes::INVALID_RANGE, input, "ranges look like `2:255-257` or `2:255-3:2`".to_string())
    };
    if locator.len() < 2 || locator.len() > 3 || locator[0].contains('-') {
        return Err(malformed());
    }
    let dash = locator[1].find('-').ok_or_else(malformed)?;
    let (left, right) = (&locator[1][..dash], &locator[1][dash + 1..]);
    if left.is_empty() || right.is_empty() || right.contains('-') {
        return Err(malformed());
    }
    let (end_surah_field, end_ayah_field) = if locator.len() == 2 {
        (locator[0], right)
    } else {
        if locator[2].contains('-') || locator[2].is_empty() {
            return Err(malformed());
        }
        (right, locator[2])
    };
    let start = (surah_of(locator[0], input)?, ayah_of(left, input)?);
    let end = (surah_of(end_surah_field, input)?, ayah_of(end_ayah_field, input)?);
    if (end.0.get(), end.1.get()) < (start.0.get(), start.1.get()) {
        return Err(fail(codes::RANGE_ORDER, input, "a range must not end before it starts"));
    }
    Ok(QuranRef::AyahRange { edition, start, end })
}

fn parse_locator(
    edition: EditionSelector,
    locator: &[&str],
    input: &str,
) -> Result<QuranRef, QuranError> {
    if let Some(kind) = division_kind(locator[0]) {
        match locator.len() {
            2 => return division(edition, kind, locator[1], input),
            1 => {
                return Err(fail(
                    codes::INVALID_DIVISION_NUMBER,
                    input,
                    format!("`{}` needs a number, e.g. `quran:{}:1`", locator[0], locator[0]),
                ));
            }
            _ => {
                return Err(fail(
                    codes::UNEXPECTED_INPUT,
                    input,
                    format!("unexpected segments after `{}`", locator[0]),
                ));
            }
        }
    }
    if locator.iter().any(|field| field.contains('-')) {
        return parse_range(edition, locator, input);
    }
    match locator.len() {
        1 => Ok(QuranRef::Surah { edition, surah: surah_of(locator[0], input)? }),
        2 => Ok(QuranRef::Ayah {
            edition,
            surah: surah_of(locator[0], input)?,
            ayah: ayah_of(locator[1], input)?,
        }),
        3 => Ok(QuranRef::Token {
            edition,
            surah: surah_of(locator[0], input)?,
            ayah: ayah_of(locator[1], input)?,
            position: position_of(locator[2], input)?,
        }),
        4 if locator[2] == "token" => Ok(QuranRef::Token {
            edition,
            surah: surah_of(locator[0], input)?,
            ayah: ayah_of(locator[1], input)?,
            position: position_of(locator[3], input)?,
        }),
        _ => Err(fail(codes::UNEXPECTED_INPUT, input, "too many `:` segments for a reference")),
    }
}

/// Parse a reference string into a [`QuranRef`].
///
/// Never panics; every malformed input yields a `QAI-QUR-01xx` error.
pub fn parse(input: &str) -> Result<QuranRef, QuranError> {
    let text = input.trim();
    if text.is_empty() {
        return Err(fail(codes::REFERENCE_EMPTY, input, "the reference is empty"));
    }
    if text.len() > MAX_REFERENCE_LEN {
        return Err(fail(
            codes::REFERENCE_TOO_LONG,
            input,
            format!("references longer than {MAX_REFERENCE_LEN} bytes are rejected"),
        ));
    }
    if !text.is_ascii() {
        return Err(fail(
            codes::UNEXPECTED_INPUT,
            input,
            "references use ASCII digits and keywords only",
        ));
    }
    let body = text.strip_prefix("quran:").unwrap_or(text);
    if body.is_empty() {
        return Err(fail(codes::MISSING_LOCATOR, input, "nothing follows `quran:`"));
    }
    let mut fields: [&str; MAX_SEGMENTS] = [""; MAX_SEGMENTS];
    let mut count = 0_usize;
    for part in body.split(':') {
        if count == MAX_SEGMENTS {
            return Err(fail(
                codes::UNEXPECTED_INPUT,
                input,
                "too many `:` segments for a reference",
            ));
        }
        fields[count] = part;
        count += 1;
    }
    if fields[0].is_empty() {
        return Err(fail(codes::UNEXPECTED_INPUT, input, "empty segment before the first `:`"));
    }
    if let Some(kind) = division_kind(fields[0]) {
        match count {
            2 => return division(EditionSelector::Active, kind, fields[1], input),
            1 => {
                return Err(fail(
                    codes::INVALID_DIVISION_NUMBER,
                    input,
                    format!("`{}` needs a number, e.g. `quran:{}:1`", fields[0], fields[0]),
                ));
            }
            _ => {}
        }
    }
    if fields[0].as_bytes().first().is_some_and(u8::is_ascii_digit) {
        parse_locator(EditionSelector::Active, &fields[..count], input)
    } else {
        let edition = edition_of(fields[0], input)?;
        if count == 1 {
            return Err(fail(
                codes::MISSING_LOCATOR,
                input,
                format!("edition `{}` needs a locator, e.g. `quran:{}:1:1`", fields[0], fields[0]),
            ));
        }
        parse_locator(edition, &fields[1..count], input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Diagnostic;

    fn ayah(s: u16, a: u32) -> QuranRef {
        QuranRef::Ayah {
            edition: EditionSelector::Active,
            surah: SurahNumber::new(s).unwrap(),
            ayah: AyahNumber::new(a).unwrap(),
        }
    }

    #[test]
    fn documented_examples_parse() {
        assert_eq!(parse("quran:1:1").unwrap(), ayah(1, 1));
        assert_eq!(parse("2:255").unwrap(), ayah(2, 255));
        assert_eq!(
            parse("quran:hafs-uthmani@1.0.0:112").unwrap(),
            QuranRef::Surah {
                edition: EditionSelector::Pinned {
                    slug: "hafs-uthmani".into(),
                    version: SemVer::new(1, 0, 0)
                },
                surah: SurahNumber::new(112).unwrap(),
            }
        );
        assert_eq!(
            parse("quran:2:255:token:3").unwrap(),
            QuranRef::Token {
                edition: EditionSelector::Active,
                surah: SurahNumber::new(2).unwrap(),
                ayah: AyahNumber::new(255).unwrap(),
                position: TokenPosition::new(3).unwrap(),
            }
        );
        assert_eq!(parse("2:255:3").unwrap(), parse("quran:2:255:token:3").unwrap());
        assert_eq!(
            parse("quran:2:255-257").unwrap(),
            QuranRef::AyahRange {
                edition: EditionSelector::Active,
                start: (SurahNumber::new(2).unwrap(), AyahNumber::new(255).unwrap()),
                end: (SurahNumber::new(2).unwrap(), AyahNumber::new(257).unwrap()),
            }
        );
        assert_eq!(
            parse("quran:2:255-3:2").unwrap(),
            QuranRef::AyahRange {
                edition: EditionSelector::Active,
                start: (SurahNumber::new(2).unwrap(), AyahNumber::new(255).unwrap()),
                end: (SurahNumber::new(3).unwrap(), AyahNumber::new(2).unwrap()),
            }
        );
        assert_eq!(
            parse("quran:hafs-uthmani:36:1-12").unwrap(),
            QuranRef::AyahRange {
                edition: EditionSelector::Slug("hafs-uthmani".into()),
                start: (SurahNumber::new(36).unwrap(), AyahNumber::new(1).unwrap()),
                end: (SurahNumber::new(36).unwrap(), AyahNumber::new(12).unwrap()),
            }
        );
        assert_eq!(
            parse("quran:juz:30").unwrap(),
            QuranRef::Division {
                edition: EditionSelector::Active,
                kind: DivisionKind::Juz,
                number: 30,
            }
        );
        assert_eq!(
            parse("quran:page:604").unwrap(),
            QuranRef::Division {
                edition: EditionSelector::Active,
                kind: DivisionKind::Page,
                number: 604,
            }
        );
        assert_eq!(
            parse("quran:sajdah:1").unwrap(),
            QuranRef::Division {
                edition: EditionSelector::Active,
                kind: DivisionKind::Sajdah,
                number: 1,
            }
        );
    }

    #[test]
    fn malformed_inputs_return_coded_errors_never_panic() {
        let cases = [
            ("", "QAI-QUR-0100"),
            ("   ", "QAI-QUR-0100"),
            ("quran:", "QAI-QUR-0112"),
            ("quran", "QAI-QUR-0112"),
            ("quran:115:1", "QAI-QUR-0103"),
            ("0:1", "QAI-QUR-0103"),
            ("2:0", "QAI-QUR-0104"),
            ("2:255:0", "QAI-QUR-0105"),
            ("2:255:token", "QAI-QUR-0105"),
            ("quran:juz", "QAI-QUR-0108"),
            ("quran:juz:0", "QAI-QUR-0108"),
            ("quran:2:255-", "QAI-QUR-0106"),
            ("quran:2:257-2:255", "QAI-QUR-0111"),
            ("quran:Bad_Slug:1:1", "QAI-QUR-0101"),
            ("quran:x@1.2:1", "QAI-QUR-0102"),
            ("2:3:Token:4", "QAI-QUR-0109"),
            ("٢:٢٥٥", "QAI-QUR-0109"),
            (":2:255", "QAI-QUR-0109"),
            ("2::255", "QAI-QUR-0104"),
        ];
        for (input, code) in cases {
            let err = parse(input).unwrap_err();
            assert_eq!(err.code().to_string(), code, "input `{input}`");
        }
    }

    #[test]
    fn surah_and_ayah_zero_rejected() {
        assert!(parse("114:0").is_err());
        assert!(parse("114").is_ok());
        assert!(parse("1:1-1:1").is_ok());
    }
}
