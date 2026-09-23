//! Phase 2 — morphology intermediate format (`dataset`).
//!
//! The [`IntermediateMorphology`] struct is the single hand-off shape every
//! adapter produces and every validator/alignment step consumes. Field
//! mapping onto migration `0017` (`quran_datasets` / `quran_token_analyses`)
//! columns:
//!
//! | Intermediate field | `0017` column(s) |
//! |---|---|
//! | `dataset.slug` / `dataset.version` | `quran_datasets.slug` / `.version` |
//! | `edition_id` | `quran_token_analyses.edition_id` |
//! | `surah` / `ayah` / `token_position` / `analysis_index` | same-named columns |
//! | `surface` / `stem` | `surface` / `stem` |
//! | `lemma` / `root` | resolved to `lemma_id` / `root_id` at import |
//! | `pos_unified` / `pos_native` | `pos_unified` / `pos_native` |
//! | `features_json` | `features_json` |
//! | `segments` | `segments_json` (+ `quran_morphemes` rows at import) |
//! | `provenance_layer` / `algorithm` / `algorithm_version` / `confidence` | same-named columns |
//! | `reviewer` / `status` | `reviewer` / `status` |
//!
//! There is deliberately NO winner/authoritative column anywhere here
//! (ADR-0209): competing analyses coexist as separate rows distinguished by
//! `analysis_index` with dataset attribution.

use serde::{Deserialize, Serialize};

use crate::error::MorphologyError;

/// A dataset reference: slug + version. Versions are opaque strings owned
/// by the application layer; this crate only checks presence/consistency.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DatasetRef {
    /// Dataset slug (e.g. `"synthetic-morph-test"`).
    pub slug: String,
    /// Dataset version (e.g. `"0.1.0"`).
    pub version: String,
}

impl DatasetRef {
    /// Build a dataset reference.
    pub fn new(slug: impl Into<String>, version: impl Into<String>) -> Self {
        Self { slug: slug.into(), version: version.into() }
    }
}

/// One token of the application token inventory: `(surah, ayah,
/// token_position, surface)`.
///
/// Alignment and validation take the inventory **by reference** and never
/// modify tokens; they only read it.
pub type InventoryToken = (u32, u32, u32, String);

/// One morpheme segment. `kind` maps onto the `quran_morphemes.kind`
/// domain (`prefix` / `stem` / `suffix`); anything else is an MV-017 finding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MorphemeSegment {
    /// Segment kind (`prefix`, `stem`, `suffix`).
    #[serde(default)]
    pub kind: String,
    /// Segment surface text.
    #[serde(default)]
    pub surface: String,
    /// Segment-level feature object.
    #[serde(default)]
    pub features: serde_json::Value,
}

/// One token analysis row (maps onto one `quran_token_analyses` row).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenAnalysis {
    /// Surah number (1–114; range checked by MV-003).
    pub surah: u32,
    /// Ayah number (≥ 1; checked by MV-003).
    pub ayah: u32,
    /// 1-based token position within the ayah.
    pub token_position: u32,
    /// 0-based index distinguishing competing analyses of one token.
    #[serde(default)]
    pub analysis_index: u32,
    /// Surface token text (must be non-empty: MV-002).
    #[serde(default)]
    pub surface: String,
    /// Lemma string (resolved to `lemma_id` at import).
    #[serde(default)]
    pub lemma: String,
    /// Root string (resolved to `root_id` at import).
    #[serde(default)]
    pub root: String,
    /// Stem string.
    #[serde(default)]
    pub stem: String,
    /// Unified POS tag (via the versioned tag mapping).
    #[serde(default)]
    pub pos_unified: String,
    /// Dataset-native POS tag, verbatim.
    #[serde(default)]
    pub pos_native: String,
    /// Feature object (maps onto `features_json`; must be an object: MV-016).
    #[serde(default = "default_features")]
    pub features_json: serde_json::Value,
    /// Morpheme segments (ride `segments_json`; MV-008 / MV-017).
    #[serde(default)]
    pub segments: Vec<MorphemeSegment>,
    /// Provenance layer (`B` or `D`; MV-014).
    #[serde(default)]
    pub provenance_layer: String,
    /// Layer-D algorithm name (required for `D`: MV-010).
    #[serde(default)]
    pub algorithm: String,
    /// Layer-D algorithm version (required for `D`: MV-010).
    #[serde(default)]
    pub algorithm_version: String,
    /// Layer-D confidence in [0, 1] (required for `D`: MV-010; MV-009).
    #[serde(default)]
    pub confidence: Option<f64>,
    /// Reviewer id (required for `human_verified`: MV-011).
    #[serde(default)]
    pub reviewer: String,
    /// Row status (`imported` / `human_verified`; MV-015).
    #[serde(default)]
    pub status: String,
}

/// Default `features_json`: the empty object (a valid MV-016 value).
fn default_features() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

/// The intermediate morphology document: dataset meta plus token analyses.
///
/// `synthetic_test_only` marks synthetic test fixtures so they can never be
/// mistaken for dataset ground truth or scholarly data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntermediateMorphology {
    /// Dataset attribution for every row in this document.
    pub dataset: DatasetRef,
    /// Edition the token references belong to.
    #[serde(default)]
    pub edition_id: String,
    /// `true` for synthetic test fixtures (never scholarly ground truth).
    #[serde(default)]
    pub synthetic_test_only: bool,
    /// Token analyses (multi-analysis: one token may appear several times
    /// with distinct `analysis_index` values).
    pub analyses: Vec<TokenAnalysis>,
}

impl IntermediateMorphology {
    /// Stable reference string for one analysis: `surah:ayah:position#index`.
    pub fn reference_of(analysis: &TokenAnalysis) -> String {
        format!(
            "{}:{}:{}#{}",
            analysis.surah, analysis.ayah, analysis.token_position, analysis.analysis_index
        )
    }
}

/// Parse an intermediate morphology JSON document.
///
/// Structural JSON failures surface as
/// [`MorphologyError::ValidationFailed`]; per-rule problems are reported by
/// the [`crate::validate`] module, not here.
pub fn load_intermediate(json: &str) -> Result<IntermediateMorphology, MorphologyError> {
    serde_json::from_str(json).map_err(|err| MorphologyError::ValidationFailed {
        detail: format!("intermediate morphology JSON does not parse: {err}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clean_analysis() -> TokenAnalysis {
        TokenAnalysis {
            surah: 2,
            ayah: 1,
            token_position: 1,
            analysis_index: 0,
            surface: "كَتَبَ".to_string(),
            lemma: "كَتَبَ".to_string(),
            root: "كتب".to_string(),
            stem: "كتب".to_string(),
            pos_unified: "Verb".to_string(),
            pos_native: "V_perf".to_string(),
            features_json: default_features(),
            segments: Vec::new(),
            provenance_layer: "B".to_string(),
            algorithm: String::new(),
            algorithm_version: String::new(),
            confidence: None,
            reviewer: String::new(),
            status: "imported".to_string(),
        }
    }

    #[test]
    fn load_round_trip_preserves_all_columns() {
        let doc = IntermediateMorphology {
            dataset: DatasetRef::new("synthetic-morph-test", "0.1.0"),
            edition_id: "test-edition-min".to_string(),
            synthetic_test_only: true,
            analyses: vec![clean_analysis()],
        };
        let json = serde_json::to_string(&doc).expect("serialize");
        let back = load_intermediate(&json).expect("parse");
        assert_eq!(doc, back);
    }

    #[test]
    fn load_rejects_structural_garbage_as_validation_failed() {
        let err = load_intermediate("{not json").expect_err("must fail");
        assert!(matches!(err, MorphologyError::ValidationFailed { .. }));
    }

    #[test]
    fn reference_format_is_stable() {
        let a = clean_analysis();
        assert_eq!(IntermediateMorphology::reference_of(&a), "2:1:1#0");
    }
}
