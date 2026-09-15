//! Phase 2 — unified search result assembly (P2-T40).
//!
//! Every search tool returns [`SearchHit`] through the single constructor
//! [`SearchHit::new`]: a pinned canonical reference, a validated
//! [`QuranQuotation`], the exact canonical span (I10), the matched tokens,
//! the score explanation, the mandatory [`NormalizationTrace`] (I9), and
//! advisory warnings. There is no constructor that omits the trace —
//! mirroring how Phase 1 gives `QuranQuotation` no constructor without
//! provenance.

use std::ops::Range;

use domain::{ContentHash, HashAlgorithm};
use quran_core::{
    AyahNumber, EditionRef, EditionSelector, QuotationParts, QuranQuotation, QuranRef, SurahNumber,
    canonical_form,
};
use quran_normalization::{CanonicalSpan, NormalizationTrace};
use serde::{Deserialize, Serialize};

use crate::error::IndexError;
use crate::model::{FieldId, FtsBackend, ResultOrder};

/// Per-hit scoring breakdown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreExplain {
    /// Engine that scored the hit.
    pub backend: FtsBackend,
    /// Field that matched.
    pub field: FieldId,
    /// Raw backend score (BM25 rank).
    pub bm25: f32,
    /// Index size the score was computed against (IDF context).
    pub corpus_docs: u64,
    /// Ordering the score was produced for.
    pub order: ResultOrder,
}

impl ScoreExplain {
    /// Build an explanation for one backend hit.
    #[must_use]
    pub fn for_hit(
        backend: FtsBackend,
        field: FieldId,
        bm25: f32,
        corpus_docs: u64,
        order: ResultOrder,
    ) -> Self {
        Self { backend, field, bm25, corpus_docs, order }
    }
}

/// Advisory attached to a hit (drift warnings, heuristic labels, …).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Warning {
    /// Machine-readable code (`QAI-IDX-0101` for staleness, …).
    pub code: String,
    /// Human-readable message.
    pub message: String,
}

impl Warning {
    /// A stale-index warning: the serving generation drifted from inputs.
    /// Queries still work; the warning is mandatory, repair is manual.
    #[must_use]
    pub fn stale_index(detail: impl Into<String>) -> Self {
        Self { code: crate::error::codes::STALE_INDEX.to_string(), message: detail.into() }
    }

    /// A free-form advisory with an explicit code.
    #[must_use]
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self { code: code.into(), message: message.into() }
    }
}

/// One segment of a concatenated match: which query part each canonical
/// token explains (plan §4.4 segmentation explanation, required by §8.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Segmentation {
    /// Slice of the (spaceless) query skeleton explained by this token.
    pub query_part: String,
    /// 1-based canonical token position.
    pub canonical_token: u16,
    /// Canonical token surface.
    pub canonical_surface: String,
}

/// Canonical context for assembling one hit.
///
/// Tools gather this from readers and repositories (edition/surah rows plus
/// the matched ayah text); [`SearchHit::new`] validates and derives
/// everything else. Field types stay primitive (`u16`/`u32`/`String`) so
/// callers never pre-validate: validation is the constructor's job, and its
/// failures are typed [`IndexError::InvalidHit`].
#[derive(Debug, Clone)]
pub struct SearchHitParts {
    /// Edition identity (slug, version, script, riwayah).
    pub edition: EditionRef,
    /// Surah number (`1..=114`, validated).
    pub surah_number: u16,
    /// Surah name in Arabic.
    pub surah_name_arabic: String,
    /// Transliterated surah name, if known.
    pub surah_name_translit: Option<String>,
    /// Ayah number (validated).
    pub ayah: u32,
    /// Canonical ayah text (the span indexes into this).
    pub arabic_text: String,
    /// Tagged ayah text hash (`sha256:…`, as stored).
    pub text_hash: String,
    /// Page number, if known.
    pub page: Option<u32>,
    /// Juz number, if known.
    pub juz: Option<u16>,
    /// Canonical character span of the match (I10).
    pub span: CanonicalSpan,
    /// Matched token positions (1-based, non-empty).
    pub matched_tokens: Vec<u16>,
    /// Backend score (`None` for canonical-order tools).
    pub score: Option<f32>,
    /// Scoring breakdown (`None` for canonical-order tools).
    pub score_explain: Option<ScoreExplain>,
    /// Exact ordered rule set applied (I9 — mandatory, never optional).
    pub explanation: NormalizationTrace,
    /// Concatenated-match segmentation (empty for all other tools).
    pub segmentation: Vec<Segmentation>,
    /// Advisories.
    pub warnings: Vec<Warning>,
}

/// One unified search hit (plan §5.6).
///
/// Private fields + validating constructor: a hit cannot exist without its
/// trace, its quotation, or its span — the same discipline Phase 1 applies
/// to `QuranQuotation`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchHit {
    reference: String,
    deep_link: String,
    quotation: QuranQuotation,
    canonical_span: CanonicalSpan,
    matched_tokens: Vec<u16>,
    score: Option<f32>,
    score_explain: Option<ScoreExplain>,
    explanation: NormalizationTrace,
    segmentation: Vec<Segmentation>,
    warnings: Vec<Warning>,
}

impl SearchHit {
    /// Assemble a hit from canonical context, validating every invariant.
    ///
    /// # Errors
    ///
    /// Returns [`IndexError::InvalidHit`] when tokens are empty, the span
    /// exceeds the text, numbers are out of range, the hash tag is malformed,
    /// or the quotation itself rejects the parts.
    pub fn new(parts: SearchHitParts) -> Result<Self, IndexError> {
        if parts.matched_tokens.is_empty() {
            return Err(IndexError::InvalidHit {
                detail: "matched_tokens must not be empty".to_string(),
            });
        }
        let char_len = parts.arabic_text.chars().count() as u32;
        if parts.span.char_range.end > char_len {
            return Err(IndexError::InvalidHit {
                detail: format!(
                    "span {}..{} exceeds text length {char_len}",
                    parts.span.char_range.start, parts.span.char_range.end
                ),
            });
        }
        let surah_number = SurahNumber::new(parts.surah_number)
            .map_err(|err| IndexError::InvalidHit { detail: format!("bad surah number: {err}") })?;
        let ayah = AyahNumber::new(parts.ayah)
            .map_err(|err| IndexError::InvalidHit { detail: format!("bad ayah number: {err}") })?;
        let reference = canonical_form(&QuranRef::Ayah {
            edition: EditionSelector::Pinned {
                slug: parts.edition.slug.clone(),
                version: parts.edition.version,
            },
            surah: surah_number,
            ayah,
        })
        .ok_or_else(|| IndexError::InvalidHit {
            detail: "reference is not pinned ayah-level".to_string(),
        })?;
        let quotation = QuranQuotation::new(QuotationParts {
            reference: reference.clone(),
            surah_number,
            surah_name_arabic: parts.surah_name_arabic.clone(),
            surah_name_translit: parts.surah_name_translit.clone(),
            ayah_range: (ayah, ayah),
            arabic_text: parts.arabic_text.clone(),
            text_hash: parse_tagged(&parts.text_hash)?,
            edition: parts.edition.clone(),
            translation: None,
            deep_link: format!(
                "/read/{}@{}/{}:{}",
                parts.edition.slug, parts.edition.version, parts.surah_number, parts.ayah
            ),
            page: parts.page,
            juz: parts.juz,
        })
        .map_err(|err| IndexError::InvalidHit { detail: format!("bad quotation: {err}") })?;
        Ok(Self {
            reference,
            deep_link: quotation.deep_link().to_string(),
            quotation,
            canonical_span: parts.span,
            matched_tokens: parts.matched_tokens,
            score: parts.score,
            score_explain: parts.score_explain,
            explanation: parts.explanation,
            segmentation: parts.segmentation,
            warnings: parts.warnings,
        })
    }

    /// The fully-qualified pinned canonical reference.
    #[must_use]
    pub fn reference(&self) -> &str {
        &self.reference
    }

    /// The deep link.
    #[must_use]
    pub fn deep_link(&self) -> &str {
        &self.deep_link
    }

    /// The validated quotation (edition identity + hash).
    #[must_use]
    pub fn quotation(&self) -> &QuranQuotation {
        &self.quotation
    }

    /// The exact canonical character span.
    #[must_use]
    pub fn canonical_span(&self) -> &CanonicalSpan {
        &self.canonical_span
    }

    /// Byte range of the span inside the canonical text, if mappable.
    #[must_use]
    pub fn byte_range(&self) -> Option<Range<u32>> {
        self.canonical_span.byte_range_in(self.quotation.arabic_text())
    }

    /// Matched token positions.
    #[must_use]
    pub fn matched_tokens(&self) -> &[u16] {
        &self.matched_tokens
    }

    /// Backend score, if any.
    #[must_use]
    pub fn score(&self) -> Option<f32> {
        self.score
    }

    /// Scoring breakdown, if any.
    #[must_use]
    pub fn score_explain(&self) -> Option<&ScoreExplain> {
        self.score_explain.as_ref()
    }

    /// The mandatory normalization trace.
    #[must_use]
    pub fn explanation(&self) -> &NormalizationTrace {
        &self.explanation
    }

    /// Concatenated-match segmentation (empty for all other tools).
    #[must_use]
    pub fn segmentation(&self) -> &[Segmentation] {
        &self.segmentation
    }

    /// Advisories.
    #[must_use]
    pub fn warnings(&self) -> &[Warning] {
        &self.warnings
    }
}

/// Parse a stored tagged hash (`sha256:…` / `blake3:…`).
fn parse_tagged(tagged: &str) -> Result<ContentHash, IndexError> {
    let (algorithm, hex) = tagged.split_once(':').ok_or_else(|| IndexError::InvalidHit {
        detail: format!("malformed text hash {tagged:?}"),
    })?;
    let algorithm = match algorithm {
        "sha256" => HashAlgorithm::Sha256,
        "blake3" => HashAlgorithm::Blake3,
        _ => {
            return Err(IndexError::InvalidHit {
                detail: format!("unknown hash algorithm in {tagged:?}"),
            });
        }
    };
    ContentHash::try_new(algorithm, hex.to_string())
        .map_err(|_| IndexError::InvalidHit { detail: format!("malformed text hash {tagged:?}") })
}

/// Test + tooling helper: minimal valid parts for one ayah.
#[cfg(test)]
pub(crate) fn sample_parts() -> SearchHitParts {
    use domain::SemVer;
    use quran_core::Script;
    SearchHitParts {
        edition: EditionRef {
            slug: "hafs-uthmani".to_string(),
            version: SemVer::new(1, 0, 0),
            script: Script::Uthmani,
            riwayah: Some("Hafs".to_string()),
        },
        surah_number: 1,
        surah_name_arabic: "الفاتحة".to_string(),
        surah_name_translit: Some("Al-Fatihah".to_string()),
        ayah: 1,
        arabic_text: "بِسْمِ ٱللَّهِ".to_string(),
        text_hash: format!("sha256:{}", "ab".repeat(32)),
        page: Some(1),
        juz: Some(1),
        span: CanonicalSpan { char_range: 0..12, exact: true },
        matched_tokens: vec![1, 2],
        score: Some(-0.5),
        score_explain: Some(ScoreExplain::for_hit(
            FtsBackend::Fts5,
            "text_bare".to_string(),
            -0.5,
            6236,
            ResultOrder::Relevance,
        )),
        explanation: NormalizationTrace::new(
            "L3.diacritics@1.0.0",
            vec![quran_normalization::RuleApplication {
                rule: quran_normalization::RuleId::N03,
                version: SemVer::new(1, 0, 0),
                kind: quran_normalization::RuleKind::Deterministic,
            }],
        )
        .expect("sample trace is valid"),
        segmentation: vec![Segmentation {
            query_part: "بسمالله".to_string(),
            canonical_token: 1,
            canonical_surface: "بِسْمِ".to_string(),
        }],
        warnings: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Diagnostic;

    #[test]
    fn assembly_derives_reference_quotation_and_link() {
        let hit = SearchHit::new(sample_parts()).unwrap();
        assert_eq!(hit.reference(), "quran:hafs-uthmani@1.0.0:1:1");
        assert_eq!(hit.deep_link(), "/read/hafs-uthmani@1.0.0/1:1");
        assert_eq!(hit.quotation().arabic_text(), "بِسْمِ ٱللَّهِ");
        assert_eq!(hit.matched_tokens(), &[1, 2]);
        assert_eq!(hit.score(), Some(-0.5));
        assert!(hit.score_explain().is_some());
        assert_eq!(hit.explanation().profile, "L3.diacritics@1.0.0");
        assert!(hit.warnings().is_empty());
        // Byte range slices back to canonical text (chars 0..12, i.e.
        // everything but the final kasra the span excludes).
        let bytes = hit.byte_range().unwrap();
        let text = hit.quotation().arabic_text();
        let sliced = &text.as_bytes()[bytes.start as usize..bytes.end as usize];
        let expected: String = text.chars().take(12).collect();
        assert_eq!(sliced, expected.as_bytes());
    }

    #[test]
    fn construction_rejects_traceless_and_spurious_hits() {
        // Empty tokens: no hit without a match.
        let mut parts = sample_parts();
        parts.matched_tokens.clear();
        assert!(SearchHit::new(parts).is_err());

        // Span past the text: unverifiable citation.
        let mut parts = sample_parts();
        parts.span = CanonicalSpan { char_range: 0..99, exact: false };
        let err = SearchHit::new(parts).unwrap_err();
        assert_eq!(err.code(), crate::error::codes::INVALID_HIT);

        // Bad numbers, bad hashes: typed rejections, never panics.
        let mut parts = sample_parts();
        parts.surah_number = 999;
        assert!(SearchHit::new(parts).is_err());
        let mut parts = sample_parts();
        parts.text_hash = "not-a-hash".to_string();
        assert!(SearchHit::new(parts).is_err());
    }

    #[test]
    fn stale_warning_uses_idx_0101() {
        let warning = Warning::stale_index("profile moved to 2.0.0");
        assert_eq!(warning.code, "QAI-IDX-0101");
    }

    #[test]
    fn hit_round_trips_json() {
        let hit = SearchHit::new(sample_parts()).unwrap();
        let json = serde_json::to_string(&hit).unwrap();
        assert!(json.contains("quran:hafs-uthmani@1.0.0:1:1"), "{json}");
        assert!(json.contains("L3.diacritics@1.0.0"), "{json}");
        let back: SearchHit = serde_json::from_str(&json).unwrap();
        assert_eq!(back, hit);
    }
}
