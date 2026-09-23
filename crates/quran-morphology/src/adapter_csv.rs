//! Phase 2 — flat CSV adapter (`adapter_csv`, P2-T60).
//!
//! Parses the **flat CSV shape**: one header row plus one record per
//! analysis, with column names deliberately different from BOTH
//! [`IntermediateMorphology`] and the array-shape adapter (proving
//! adapters are per-shape mappings).
//!
//! The CSV dialect is parsed by hand (this crate's dependency set is
//! `serde`/`serde_json`/`thiserror` plus `domain`/`quran-core` only):
//! `,`-separated fields, `"` quoting with `""` escapes, `#` comment lines,
//! blank lines skipped. The `morphs` cell uses the micro-syntax
//! `kind:surface|kind:surface` (e.g. `prefix:ال|stem:علم`).

use std::collections::HashMap;

use crate::dataset::{DatasetRef, IntermediateMorphology, MorphemeSegment, TokenAnalysis};
use crate::error::MorphologyError;

/// Flat CSV column names (all differ from intermediate field names).
const REQUIRED_COLUMNS: [&str; 4] = ["chapter", "verse", "word_n", "token"];

/// Split one CSV line into fields (`"` quoting, `""` escapes).
fn split_csv_line(line: &str) -> Result<Vec<String>, MorphologyError> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    let mut in_quotes = false;
    let mut any_quotes = false;
    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    current.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                current.push(c);
            }
        } else if c == '"' {
            in_quotes = true;
            any_quotes = true;
        } else if c == ',' {
            fields.push(std::mem::take(&mut current));
        } else {
            current.push(c);
        }
    }
    if in_quotes {
        return Err(MorphologyError::ValidationFailed {
            detail: "csv adapter: unterminated quoted field".to_string(),
        });
    }
    // A line that is only quotes with no content still yields one field;
    // that is fine — arity is checked by the caller.
    let _ = any_quotes;
    fields.push(current);
    Ok(fields)
}

/// Parse the `morphs` micro-syntax `kind:surface|kind:surface`.
fn parse_morphs(cell: &str) -> Result<Vec<MorphemeSegment>, MorphologyError> {
    if cell.trim().is_empty() {
        return Ok(Vec::new());
    }
    cell.split('|')
        .map(|part| {
            let (kind, surface) =
                part.split_once(':').ok_or_else(|| MorphologyError::ValidationFailed {
                    detail: format!(
                        "csv adapter: bad morphs cell '{cell}': want 'kind:surface|…' entries"
                    ),
                })?;
            Ok(MorphemeSegment {
                kind: kind.trim().to_string(),
                surface: surface.trim().to_string(),
                features: serde_json::Value::Null,
            })
        })
        .collect()
}

fn parse_u32(cell: &str, column: &str, line_no: usize) -> Result<u32, MorphologyError> {
    cell.trim().parse::<u32>().map_err(|_| MorphologyError::ValidationFailed {
        detail: format!("csv adapter: line {line_no}: column '{column}' is not a u32: '{cell}'"),
    })
}

fn parse_bool_cell(cell: &str) -> bool {
    matches!(cell.trim().to_ascii_lowercase().as_str(), "true" | "1" | "yes")
}

fn parse_optional_f64(
    cell: &str,
    column: &str,
    line_no: usize,
) -> Result<Option<f64>, MorphologyError> {
    if cell.trim().is_empty() {
        return Ok(None);
    }
    cell.trim().parse::<f64>().map_or_else(
        |_| {
            Err(MorphologyError::ValidationFailed {
                detail: format!(
                    "csv adapter: line {line_no}: column '{column}' is not a number: '{cell}'"
                ),
            })
        },
        |v| Ok(Some(v)),
    )
}

/// Parse the flat CSV shape into [`IntermediateMorphology`].
///
/// `dataset` attribution and `edition_id` are supplied by the caller.
/// JSON text (or any non-CSV shape) fails with a typed
/// [`MorphologyError::ValidationFailed`] (missing required columns).
pub fn parse_flat_csv(
    csv: &str,
    dataset: &DatasetRef,
    edition_id: &str,
) -> Result<IntermediateMorphology, MorphologyError> {
    let mut lines: Vec<(usize, &str)> = Vec::new();
    for (index, raw) in csv.lines().enumerate() {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        lines.push((index + 1, line));
    }
    let Some((_, header_line)) = lines.first() else {
        return Err(MorphologyError::ValidationFailed {
            detail: "csv adapter: no header row found".to_string(),
        });
    };
    let header = split_csv_line(header_line)?;
    let columns: HashMap<&str, usize> =
        header.iter().enumerate().map(|(i, name)| (name.trim(), i)).collect();
    let missing: Vec<&str> =
        REQUIRED_COLUMNS.iter().filter(|c| !columns.contains_key(**c)).copied().collect();
    if !missing.is_empty() {
        return Err(MorphologyError::ValidationFailed {
            detail: format!("csv adapter: missing required columns: {}", missing.join(", ")),
        });
    }
    let col = |name: &str| columns.get(name).copied();

    let get = |record: &[String], name: &str| -> String {
        col(name).map_or_else(String::new, |i| record.get(i).cloned().unwrap_or_default())
    };

    let mut analyses = Vec::new();
    let mut synthetic_test_only = false;
    for (line_no, line) in lines.iter().skip(1) {
        let record = split_csv_line(line)?;
        if record.len() != header.len() {
            return Err(MorphologyError::ValidationFailed {
                detail: format!(
                    "csv adapter: line {line_no}: want {} fields, found {}",
                    header.len(),
                    record.len()
                ),
            });
        }
        let segmented = parse_bool_cell(&get(&record, "segmented"));
        let mut features = serde_json::json!({});
        if segmented {
            features["segmented"] = serde_json::Value::Bool(true);
        }
        if parse_bool_cell(&get(&record, "synthetic_test_only")) {
            synthetic_test_only = true;
        }
        analyses.push(TokenAnalysis {
            surah: parse_u32(&get(&record, "chapter"), "chapter", *line_no)?,
            ayah: parse_u32(&get(&record, "verse"), "verse", *line_no)?,
            token_position: parse_u32(&get(&record, "word_n"), "word_n", *line_no)?,
            analysis_index: if col("ana_n").is_some() {
                let cell = get(&record, "ana_n");
                if cell.trim().is_empty() { 0 } else { parse_u32(&cell, "ana_n", *line_no)? }
            } else {
                0
            },
            surface: get(&record, "token"),
            lemma: get(&record, "lemma_cite"),
            root: get(&record, "root_cite"),
            stem: get(&record, "stem_cite"),
            pos_unified: get(&record, "upos"),
            pos_native: get(&record, "msd"),
            features_json: features,
            segments: parse_morphs(&get(&record, "morphs"))?,
            provenance_layer: get(&record, "prov"),
            algorithm: get(&record, "method"),
            algorithm_version: get(&record, "method_v"),
            confidence: parse_optional_f64(&get(&record, "score"), "score", *line_no)?,
            reviewer: get(&record, "reviewer"),
            status: get(&record, "state"),
        });
    }
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

    const FLAT_CSV: &str = concat!(
        "# synthetic_test_only=true: synthetic CSV sample, never scholarly data\n",
        "chapter,verse,word_n,ana_n,token,lemma_cite,root_cite,stem_cite,msd,upos,prov,method,method_v,score,state,reviewer,segmented,morphs,synthetic_test_only\n",
        "2,1,1,0,يَعْلَمُ,عَلِمَ,علم,يعلم,V_impf,Verb,B,,,,imported,,false,,true\n",
    );

    #[test]
    fn parses_flat_csv_with_column_mapping() {
        let doc = parse_flat_csv(
            FLAT_CSV,
            &DatasetRef::new("synthetic-morph-test", "0.1.0"),
            "test-edition-min",
        )
        .expect("parse");
        assert_eq!(doc.analyses.len(), 1);
        let a = &doc.analyses[0];
        assert_eq!((a.surah, a.ayah, a.token_position), (2, 1, 1));
        assert_eq!(a.surface, "يَعْلَمُ");
        assert_eq!(a.root, "علم");
        assert!(doc.synthetic_test_only);
    }

    #[test]
    fn json_shape_fails_typed() {
        let err = parse_flat_csv(
            r#"[{"sura_no": 2, "aya_no": 1}]"#,
            &DatasetRef::new("synthetic-morph-test", "0.1.0"),
            "test-edition-min",
        )
        .expect_err("must fail");
        assert!(matches!(err, MorphologyError::ValidationFailed { .. }), "unexpected: {err:?}");
    }

    #[test]
    fn quoted_fields_and_escapes_parse() {
        let fields = split_csv_line(r#"a,"b,c","d""e",f"#).expect("split");
        assert_eq!(fields, vec!["a", "b,c", r#"d"e"#, "f"]);
    }

    #[test]
    fn bad_morphs_cell_fails_typed() {
        assert!(parse_morphs("prefix-without-colon").is_err());
        assert!(parse_morphs("prefix:ال|stem:علم").is_ok());
    }
}
