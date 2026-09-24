//! Phase 2 — array-shape JSON adapter (`adapter_json`, P2-T60).
//!
//! Parses the **array shape**: a top-level JSON array of per-token rows
//! whose field names deliberately differ from [`IntermediateMorphology`]
//! (proving the adapter layer, not the wire format, owns the mapping).
//! Together with [`crate::adapter_csv`] this demonstrates adapter
//! extensibility: new dataset shapes add an adapter, never a schema edit.
//!
//! The array shape carries no provenance header, so the adapter marks its
//! output `synthetic_test_only` whenever any input row bears that label —
//! synthetic fixtures can never be mistaken for dataset ground truth.

use serde::Deserialize;

use crate::dataset::{DatasetRef, IntermediateMorphology, MorphemeSegment, TokenAnalysis};
use crate::error::MorphologyError;

/// One array-shape row. Field names are intentionally non-canonical.
#[derive(Debug, Deserialize)]
struct ArrayRow {
    sura_no: u32,
    aya_no: u32,
    tok_idx: u32,
    #[serde(default)]
    analysis_no: u32,
    #[serde(default)]
    surface_form: String,
    #[serde(default)]
    lemma_str: String,
    #[serde(default)]
    root_str: String,
    #[serde(default)]
    stem_str: String,
    #[serde(default)]
    tag_native: String,
    #[serde(default)]
    tag_unified: String,
    #[serde(default)]
    layer: String,
    #[serde(default)]
    algo: String,
    #[serde(default)]
    algo_ver: String,
    #[serde(default)]
    conf: Option<f64>,
    #[serde(default)]
    state: String,
    #[serde(default)]
    reviewer: String,
    #[serde(default)]
    segmented: bool,
    /// Optional dataset-supplied morphological pattern label.
    #[serde(default)]
    pattern: String,
    /// Optional dataset-supplied verb-form label.
    #[serde(default)]
    verb_form: String,
    #[serde(default)]
    morphs: Vec<MorphRow>,
    #[serde(default)]
    synthetic_test_only: bool,
}

/// One array-shape morpheme segment (`part`/`text` instead of
/// `kind`/`surface`).
#[derive(Debug, Deserialize)]
struct MorphRow {
    #[serde(default)]
    part: String,
    #[serde(default)]
    text: String,
}

/// Parse the array shape into [`IntermediateMorphology`].
///
/// `dataset` attribution and `edition_id` are supplied by the caller (the
/// shape carries neither); structural failures surface as
/// [`MorphologyError::ValidationFailed`].
pub fn parse_array_shape(
    json: &str,
    dataset: &DatasetRef,
    edition_id: &str,
) -> Result<IntermediateMorphology, MorphologyError> {
    let rows: Vec<ArrayRow> =
        serde_json::from_str(json).map_err(|err| MorphologyError::ValidationFailed {
            detail: format!("array-shape adapter: JSON array of rows expected: {err}"),
        })?;
    let synthetic_test_only = rows.iter().any(|r| r.synthetic_test_only);
    let analyses = rows
        .into_iter()
        .map(|r| {
            let mut features = serde_json::json!({});
            if r.segmented {
                features["segmented"] = serde_json::Value::Bool(true);
            }
            if !r.pattern.is_empty() {
                features["pattern"] = serde_json::Value::String(r.pattern);
            }
            if !r.verb_form.is_empty() {
                features["verb_form"] = serde_json::Value::String(r.verb_form);
            }
            TokenAnalysis {
                surah: r.sura_no,
                ayah: r.aya_no,
                token_position: r.tok_idx,
                analysis_index: r.analysis_no,
                surface: r.surface_form,
                lemma: r.lemma_str,
                root: r.root_str,
                stem: r.stem_str,
                pos_unified: r.tag_unified,
                pos_native: r.tag_native,
                features_json: features,
                segments: r
                    .morphs
                    .into_iter()
                    .map(|m| MorphemeSegment {
                        kind: m.part,
                        surface: m.text,
                        features: serde_json::Value::Null,
                    })
                    .collect(),
                provenance_layer: r.layer,
                algorithm: r.algo,
                algorithm_version: r.algo_ver,
                confidence: r.conf,
                reviewer: r.reviewer,
                status: r.state,
            }
        })
        .collect();
    Ok(IntermediateMorphology {
        dataset: dataset.clone(),
        edition_id: edition_id.to_string(),
        synthetic_test_only,
        analyses,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARRAY_JSON: &str = r#"[
        {"sura_no": 2, "aya_no": 1, "tok_idx": 1, "surface_form": "كَتَبَ",
         "lemma_str": "كَتَبَ", "root_str": "كتب", "stem_str": "كتب",
         "tag_native": "V_perf", "tag_unified": "Verb",
         "layer": "B", "state": "imported", "synthetic_test_only": true}
    ]"#;

    #[test]
    fn parses_array_shape_with_field_mapping() {
        let doc = parse_array_shape(
            ARRAY_JSON,
            &DatasetRef::new("synthetic-morph-test", "0.1.0"),
            "test-edition-min",
        )
        .expect("parse");
        assert_eq!(doc.analyses.len(), 1);
        let a = &doc.analyses[0];
        assert_eq!((a.surah, a.ayah, a.token_position), (2, 1, 1));
        assert_eq!(a.surface, "كَتَبَ");
        assert_eq!(a.pos_native, "V_perf");
        assert!(doc.synthetic_test_only);
    }

    #[test]
    fn rejects_non_array_input_as_typed_error() {
        let err = parse_array_shape(
            "chapter,verse\n1,1\n",
            &DatasetRef::new("synthetic-morph-test", "0.1.0"),
            "test-edition-min",
        )
        .expect_err("must fail");
        assert!(matches!(err, MorphologyError::ValidationFailed { .. }));
    }
}
