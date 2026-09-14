//! Dataset adapters: real-world shapes into [`EditionSource`].
//!
//! # quran_corpus::adapters
//!
//! Each adapter declares the `source_id` pattern it handles. Adding a new
//! dataset means adding an adapter, never a core change (§56). The JSON
//! adapter consumes the normalized intermediate format directly; the CSV
//! adapter proves the trait works for a genuinely different shape by taking
//! its edition metadata as a sidecar.

use serde::Deserialize;

use crate::error::CorpusError;
use crate::format::{
    AyahSource, EditionMeta, EditionSource, ExpectedCounts, SurahSource, TokenizationPolicy,
};
use quran_core::enums::SajdahKind;

/// Converts one dataset shape into the normalized intermediate format.
pub trait EditionAdapter: Send + Sync {
    /// Stable adapter name used by `qai quran import --adapter`.
    fn name(&self) -> &'static str;

    /// The `source_id` pattern this adapter handles.
    fn source_id_pattern(&self) -> &'static str;

    /// Parse raw dataset text into an [`EditionSource`].
    fn parse(&self, input: &str) -> Result<EditionSource, CorpusError>;
}

/// Adapter for dataset shape 1: the normalized intermediate JSON itself.
pub struct JsonAdapter;

impl JsonAdapter {
    /// The registered adapter name.
    pub const NAME: &str = "json";
}

impl EditionAdapter for JsonAdapter {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn source_id_pattern(&self) -> &'static str {
        "qai.quran.edition json (*.json)"
    }

    fn parse(&self, input: &str) -> Result<EditionSource, CorpusError> {
        let source: EditionSource = serde_json::from_str(input).map_err(|err| {
            CorpusError::InvalidFormat { detail: format!("not an edition source document: {err}") }
        })?;
        source.validate()?;
        Ok(source)
    }
}

/// One row of the ayah-rows CSV shape.
///
/// Headers (exact): `surah,ayah,text,juz,hizb,rub,manzil,ruku,page,sajdah`.
/// Empty cells decode to `None`; `sajdah` accepts `recommended`/`obligatory`.
#[derive(Debug, Deserialize)]
struct CsvRow {
    surah: u16,
    ayah: u32,
    text: String,
    juz: Option<u16>,
    hizb: Option<u16>,
    rub: Option<u16>,
    manzil: Option<u16>,
    ruku: Option<u16>,
    page: Option<u32>,
    sajdah: Option<String>,
}

/// Adapter for dataset shape 2: ayah rows as CSV plus sidecar edition metadata.
pub struct CsvAdapter {
    edition: EditionMeta,
    surahs: Vec<SurahSource>,
    expected: ExpectedCounts,
    tokenization: TokenizationPolicy,
}

impl CsvAdapter {
    /// Build a CSV adapter with the sidecar metadata the row shape cannot carry.
    pub fn new(
        edition: EditionMeta,
        surahs: Vec<SurahSource>,
        expected: ExpectedCounts,
        tokenization: TokenizationPolicy,
    ) -> Self {
        Self { edition, surahs, expected, tokenization }
    }

    /// The registered adapter name.
    pub const NAME: &str = "csv";
}

impl EditionAdapter for CsvAdapter {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn source_id_pattern(&self) -> &'static str {
        "ayah rows CSV (*.csv) with sidecar edition metadata"
    }

    fn parse(&self, input: &str) -> Result<EditionSource, CorpusError> {
        let fail = |detail: String| CorpusError::AdapterFailed { adapter: Self::NAME, detail };
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .from_reader(input.as_bytes());
        let mut ayahs = Vec::new();
        for (index, row) in reader.deserialize::<CsvRow>().enumerate() {
            let row: CsvRow = row.map_err(|err| fail(format!("row {}: {err}", index + 1)))?;
            let sajdah = match row.sajdah.as_deref().map(str::trim) {
                None | Some("") => None,
                Some("recommended") => Some(SajdahKind::Recommended),
                Some("obligatory") => Some(SajdahKind::Obligatory),
                Some(other) => {
                    return Err(fail(format!(
                        "row {}: unknown sajdah `{other}` (expected recommended|obligatory)",
                        index + 1
                    )));
                }
            };
            ayahs.push(AyahSource {
                surah: row.surah,
                ayah: row.ayah,
                text: row.text,
                juz: row.juz,
                hizb: row.hizb,
                rub: row.rub,
                manzil: row.manzil,
                ruku: row.ruku,
                page: row.page,
                sajdah,
                tokens: None,
            });
        }
        let source = EditionSource {
            format: crate::format::FORMAT_TAG.to_string(),
            format_version: crate::format::FORMAT_VERSION,
            edition: self.edition.clone(),
            expected: self.expected.clone(),
            surahs: self.surahs.clone(),
            ayahs,
            tokenization: self.tokenization.clone(),
        };
        source.validate().map_err(|err| fail(err.to_string()))?;
        Ok(source)
    }
}

/// Parse with a statically registered adapter (`json`).
///
/// Adapters needing sidecar metadata (like CSV) are constructed directly.
pub fn parse_with_adapter(name: &str, input: &str) -> Result<EditionSource, CorpusError> {
    match name {
        JsonAdapter::NAME => JsonAdapter.parse(input),
        _ => Err(CorpusError::UnknownAdapter { name: name.to_string() }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::tests::sample;
    use quran_core::error::Diagnostic;

    #[test]
    fn json_adapter_roundtrips_the_sample() {
        let json = serde_json::to_string(&sample()).unwrap();
        let back = JsonAdapter.parse(&json).unwrap();
        assert_eq!(back, sample());
        let via_registry = parse_with_adapter("json", &json).unwrap();
        assert_eq!(via_registry, sample());
    }

    #[test]
    fn json_adapter_rejects_garbage_and_unknown_names() {
        let err = JsonAdapter.parse("{not json").unwrap_err();
        assert_eq!(err.code().to_string(), "QAI-QUR-0201");
        let err = parse_with_adapter("xml", "{}").unwrap_err();
        assert_eq!(err.code().to_string(), "QAI-QUR-0202");
    }

    #[test]
    fn csv_adapter_parses_rows_with_sidecar_meta() {
        let sample = sample();
        let adapter = CsvAdapter::new(
            sample.edition.clone(),
            sample.surahs.clone(),
            sample.expected.clone(),
            sample.tokenization.clone(),
        );
        let csv = "surah,ayah,text,juz,hizb,rub,manzil,ruku,page,sajdah\n\
                   1,1,ب ت ث,1,1,1,1,1,1,\n\
                   1,2,ن ي م,1,1,1,1,1,1,recommended\n";
        let parsed = adapter.parse(csv).unwrap();
        assert_eq!(parsed.ayahs.len(), 2);
        assert_eq!(parsed.ayahs[0].text, "ب ت ث");
        assert_eq!(parsed.ayahs[1].sajdah, Some(SajdahKind::Recommended));
        assert_eq!(parsed.edition.slug, "test-edition-min");
    }

    #[test]
    fn csv_adapter_rejects_bad_sajdah_and_bad_rows() {
        let sample = sample();
        let adapter = CsvAdapter::new(
            sample.edition.clone(),
            sample.surahs.clone(),
            sample.expected.clone(),
            sample.tokenization.clone(),
        );
        let bad = "surah,ayah,text,juz,hizb,rub,manzil,ruku,page,sajdah\n1,1,x,,,,,,,,maybe\n";
        let err = adapter.parse(bad).unwrap_err();
        assert_eq!(err.code().to_string(), "QAI-QUR-0203");
        let bad_header = "surah,ayah\n1,1\n";
        assert!(adapter.parse(bad_header).is_err());
    }
}
