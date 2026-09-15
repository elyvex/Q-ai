//! Phase 2 — normalization rule identity and application contract.
//!
//! Rules are pure, order-significant, composable character transformations
//! (`plan.md` §3.2). Each rule has a stable [`RuleId`], a version, a
//! [`RuleKind`], and a documented mapping table (the tables land with the
//! rule implementations in M1b; the linguist-reviewed catalog is ADR-0204).
//!
//! [`NormalizedText`] pairs a derived string with the [`SpanMap`](crate::span::SpanMap)
//! that maps it back to its input, so provenance is never separated from text.

use serde::{Deserialize, Serialize};

pub use domain::SemVer;

use crate::error::NormalizationError;
use crate::span::SpanMap;

/// Stable rule identifier from the N01–N24 catalog (`plan.md` §3.2).
///
/// Identifiers are append-only: new rules take the next free number; existing
/// numbers are never reused or redefined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleId {
    /// `whitespace_collapse` — runs of whitespace → single U+0020; trim ends.
    N01,
    /// `strip_tatweel` — remove U+0640.
    N02,
    /// `strip_harakat` — remove U+064B–U+0652 + U+0656–U+065F extras.
    N03,
    /// `strip_quranic_marks` — remove U+06D6–U+06ED and related signs.
    N04,
    /// `strip_superscript_alef` — remove U+0670.
    N05,
    /// `normalize_hamza_forms` — fold hamza carriers to bare forms.
    N06,
    /// `normalize_wasla` — U+0671 → U+0627.
    N07,
    /// `normalize_alif_maqsura` — U+0649 → U+064A.
    N08,
    /// `normalize_ta_marbuta` — U+0629 → U+0647.
    N09,
    /// `normalize_persian_codepoints` — fold Persian variants to Arabic.
    N10,
    /// `strip_zero_width_and_bidi` — remove Cf/format controls.
    N11,
    /// `strip_punctuation` — remove Arabic and Latin punctuation.
    N12,
    /// `fold_digits` — Eastern/Arabic-Indic digits → ASCII.
    N13,
    /// `strip_pause_marks` — remove waqf letters.
    N14,
    /// `expand_presentation_forms` — compatibility forms → base sequences.
    N15,
    /// `nfc` — canonical composition (queries only; asserted on canonical).
    N16,
    /// `remove_spaces` — delete U+0020 (skeleton search).
    N17,
    /// `strip_definite_article` — heuristic leading-article strip.
    N18,
    /// `strip_conjunction_prefix` — heuristic leading و/ف strip.
    N19,
    /// `strip_preposition_prefix` — heuristic leading ب/ل/ك strip.
    N20,
    /// `strip_pronoun_suffix` — heuristic trailing-pronoun strip.
    N21,
    /// `dedupe_repeated_letters` — experimental collapse of ≥3 repeats.
    N22,
    /// `transliterate` — reserved for Phase 4 (ADR-0206).
    N23,
    /// `phonetic_key` — reserved, experimental, off by default.
    N24,
}

impl RuleId {
    /// Canonical short id (`"N01"` … `"N24"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::N01 => "N01",
            Self::N02 => "N02",
            Self::N03 => "N03",
            Self::N04 => "N04",
            Self::N05 => "N05",
            Self::N06 => "N06",
            Self::N07 => "N07",
            Self::N08 => "N08",
            Self::N09 => "N09",
            Self::N10 => "N10",
            Self::N11 => "N11",
            Self::N12 => "N12",
            Self::N13 => "N13",
            Self::N14 => "N14",
            Self::N15 => "N15",
            Self::N16 => "N16",
            Self::N17 => "N17",
            Self::N18 => "N18",
            Self::N19 => "N19",
            Self::N20 => "N20",
            Self::N21 => "N21",
            Self::N22 => "N22",
            Self::N23 => "N23",
            Self::N24 => "N24",
        }
    }

    /// Parse a short id back to a rule.
    ///
    /// # Errors
    ///
    /// Returns [`NormalizationError::UnknownRule`] for anything outside
    /// `N01`…`N24`.
    pub fn parse(text: &str) -> Result<Self, NormalizationError> {
        match text {
            "N01" => Ok(Self::N01),
            "N02" => Ok(Self::N02),
            "N03" => Ok(Self::N03),
            "N04" => Ok(Self::N04),
            "N05" => Ok(Self::N05),
            "N06" => Ok(Self::N06),
            "N07" => Ok(Self::N07),
            "N08" => Ok(Self::N08),
            "N09" => Ok(Self::N09),
            "N10" => Ok(Self::N10),
            "N11" => Ok(Self::N11),
            "N12" => Ok(Self::N12),
            "N13" => Ok(Self::N13),
            "N14" => Ok(Self::N14),
            "N15" => Ok(Self::N15),
            "N16" => Ok(Self::N16),
            "N17" => Ok(Self::N17),
            "N18" => Ok(Self::N18),
            "N19" => Ok(Self::N19),
            "N20" => Ok(Self::N20),
            "N21" => Ok(Self::N21),
            "N22" => Ok(Self::N22),
            "N23" => Ok(Self::N23),
            "N24" => Ok(Self::N24),
            other => Err(NormalizationError::UnknownRule { rule: other.to_string() }),
        }
    }

    /// All rule ids in catalog order.
    #[must_use]
    pub const fn all() -> [Self; 24] {
        [
            Self::N01,
            Self::N02,
            Self::N03,
            Self::N04,
            Self::N05,
            Self::N06,
            Self::N07,
            Self::N08,
            Self::N09,
            Self::N10,
            Self::N11,
            Self::N12,
            Self::N13,
            Self::N14,
            Self::N15,
            Self::N16,
            Self::N17,
            Self::N18,
            Self::N19,
            Self::N20,
            Self::N21,
            Self::N22,
            Self::N23,
            Self::N24,
        ]
    }

    /// True for the heuristic rules N18–N22 (trace-labeled per §8.3).
    #[must_use]
    pub const fn is_heuristic(self) -> bool {
        matches!(self, Self::N18 | Self::N19 | Self::N20 | Self::N21 | Self::N22)
    }

    /// True for the reserved rules N23–N24 (not implemented in Phase 2).
    #[must_use]
    pub const fn is_reserved(self) -> bool {
        matches!(self, Self::N23 | Self::N24)
    }
}

impl std::fmt::Display for RuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Whether a rule is an exact code-point transformation or a linguistic
/// approximation that must be surfaced to the user as heuristic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleKind {
    /// Pure code-point/whitespace transformation with a published mapping table.
    Deterministic,
    /// Pattern-based approximation of morphology. Results using it MUST be
    /// labeled heuristic in traces, CLI, API, and tool payloads.
    Heuristic,
}

/// Derived text inseparable from its offset map back to its input.
///
/// Rules consume and produce this type so a bare `String` can never silently
/// lose its provenance inside the pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedText {
    text: String,
    spans: SpanMap,
}

impl NormalizedText {
    /// Wrap untransformed input with the identity map.
    #[must_use]
    pub fn from_plain(text: &str) -> Self {
        let len = text.chars().count() as u32;
        Self { text: text.to_string(), spans: SpanMap::identity(len) }
    }

    /// The derived string.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The offset map back to the input of the last applied rule.
    #[must_use]
    pub fn spans(&self) -> &SpanMap {
        &self.spans
    }

    /// Length in chars.
    #[must_use]
    pub fn len_chars(&self) -> u32 {
        self.spans.derived_len()
    }

    /// True when the derived string is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Assemble derived text with its offset map (rule implementations only).
    ///
    /// In debug builds, panics when the text length disagrees with the map's
    /// derived side; release builds accept it (the map is advisory there).
    pub(crate) fn from_parts(text: String, spans: SpanMap) -> Self {
        debug_assert_eq!(
            text.chars().count() as u32,
            spans.derived_len(),
            "NormalizedText text/map length mismatch"
        );
        Self { text, spans }
    }
}

/// Implementation contract for a single normalization rule (`plan.md` §3.2).
///
/// Rules are pure (`Send + Sync`, no I/O) and total over Unicode input:
/// they must never panic, including on arbitrary strings (fuzz-tested).
pub trait NormalizationRule: Send + Sync {
    /// Stable catalog id.
    fn id(&self) -> RuleId;

    /// Rule implementation version.
    fn version(&self) -> SemVer;

    /// One-line human description of the effect.
    fn description(&self) -> &'static str;

    /// Deterministic fold vs heuristic approximation.
    fn kind(&self) -> RuleKind;

    /// Pure application. Must produce a [`SpanMap`] so offsets stay
    /// reversible (I10); implementations build it with
    /// [`SpanMap::build`](crate::span::SpanMap::build).
    fn apply(&self, input: &NormalizedText) -> NormalizedText;

    /// True when applying twice equals applying once. Property-tested for
    /// every rule that claims it.
    fn is_idempotent(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Diagnostic;

    #[test]
    fn catalog_has_24_stable_ids() {
        let all = RuleId::all();
        assert_eq!(all.len(), 24);
        for (i, id) in all.iter().enumerate() {
            assert_eq!(id.as_str(), format!("N{:02}", i + 1));
            assert_eq!(RuleId::parse(id.as_str()).unwrap(), *id);
        }
    }

    #[test]
    fn unknown_rule_parse_reports_code() {
        let err = RuleId::parse("N99").unwrap_err();
        assert_eq!(err.code(), crate::error::codes::UNKNOWN_RULE);
    }

    #[test]
    fn heuristic_and_reserved_flags_match_plan() {
        for id in RuleId::all() {
            let n: u32 = id.as_str()[1..].parse().unwrap();
            assert_eq!(id.is_heuristic(), (18..=22).contains(&n), "{id}");
            assert_eq!(id.is_reserved(), n >= 23, "{id}");
        }
    }

    #[test]
    fn plain_text_carries_identity_map() {
        let t = NormalizedText::from_plain("بِسْمِ");
        assert_eq!(t.len_chars(), 6);
        assert!(!t.is_empty());
        let span = t.spans().to_canonical(0..6);
        assert_eq!(span.char_range, 0..6);
        assert!(span.exact);
    }
}
