//! Hadith source format parsers.

use serde::Deserialize;
use std::collections::BTreeMap;

use crate::{ParseError, ParsedGrading, ParsedHadith};

const FORBIDDEN_KEYS: &[&str] = &["consensus", "majority"];

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonHadith {
    #[serde(
        alias = "collection",
        alias = "collection_name",
        alias = "collection_slug",
        alias = "name",
        alias = "slug"
    )]
    pub collection: String,
    #[serde(alias = "source_number", alias = "hadith_number", alias = "number", alias = "id")]
    pub source_number: String,
    #[serde(alias = "matn_text", alias = "matn", alias = "text", alias = "hadith_text")]
    pub matn_text: String,
    #[serde(alias = "isnad_text", alias = "isnad", alias = "chain", alias = "sanad")]
    pub isnad_text: Option<String>,
    #[serde(alias = "narrator_names", alias = "narrators", alias = "transmitters", default)]
    pub narrator_names: Vec<String>,
    #[serde(alias = "gradings", alias = "grading", alias = "gradings_list", default)]
    pub gradings: Vec<JsonGrading>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonGrading {
    #[serde(alias = "scholar_name", alias = "scholar", alias = "grader")]
    pub scholar_name: String,
    #[serde(alias = "grading_value", alias = "grade", alias = "value", alias = "grading")]
    pub grading_value: String,
    #[serde(alias = "source_citation", alias = "citation", alias = "source", alias = "reference")]
    pub source_citation: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct HadithParserImpl;

impl HadithParserImpl {
    pub fn parse(input: &str) -> Result<Vec<ParsedHadith>, ParseError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }
        // Reject consensus-merged gradings anywhere in the input.
        reject_forbidden_keys(trimmed)?;

        let mut out = Vec::new();
        if trimmed.starts_with('[') {
            let arr: Vec<JsonHadith> = serde_json::from_str(trimmed)
                .map_err(|e| ParseError { message: e.to_string(), location: None })?;
            for row in arr {
                out.push(row.into_parsed()?);
            }
        } else {
            let obj: JsonHadith = serde_json::from_str(trimmed)
                .map_err(|e| ParseError { message: e.to_string(), location: None })?;
            out.push(obj.into_parsed()?);
        }
        Ok(out)
    }
}

impl JsonHadith {
    fn into_parsed(self) -> Result<ParsedHadith, ParseError> {
        let gradings =
            self.gradings.into_iter().map(|g| g.into_parsed()).collect::<Result<Vec<_>, _>>()?;
        Ok(ParsedHadith {
            collection: self.collection,
            source_number: self.source_number,
            matn_text: self.matn_text,
            isnad_text: self.isnad_text,
            narrator_names: self.narrator_names,
            gradings,
            extra: self.extra,
        })
    }
}

impl JsonGrading {
    fn into_parsed(self) -> Result<ParsedGrading, ParseError> {
        Ok(ParsedGrading {
            scholar_name: self.scholar_name,
            grading_value: self.grading_value,
            source_citation: self.source_citation,
            extra: self.extra,
        })
    }
}

fn reject_forbidden_keys(input: &str) -> Result<(), ParseError> {
    let value: serde_json::Value = serde_json::from_str(input)
        .map_err(|e| ParseError { message: e.to_string(), location: None })?;
    if contains_forbidden(&value) {
        return Err(ParseError {
            message: "consensus-merged grading fields are not allowed".to_string(),
            location: None,
        });
    }
    Ok(())
}

fn contains_forbidden(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(map) => {
            for (key, v) in map {
                let lower = key.to_lowercase();
                if FORBIDDEN_KEYS.iter().any(|f| lower.contains(f)) {
                    return true;
                }
                if contains_forbidden(v) {
                    return true;
                }
            }
            false
        }
        serde_json::Value::Array(items) => items.iter().any(contains_forbidden),
        _ => false,
    }
}

// Trait alias holder.
pub trait HadithParser {
    fn parse(input: &str) -> Result<Vec<ParsedHadith>, ParseError>;
}

impl HadithParser for HadithParserImpl {
    fn parse(input: &str) -> Result<Vec<ParsedHadith>, ParseError> {
        HadithParserImpl::parse(input)
    }
}

// Convenience free function and aliases for likely parser names used by tests.
pub fn parse(input: &str) -> Result<Vec<ParsedHadith>, ParseError> {
    HadithParserImpl::parse(input)
}

// Type aliases for common parser names.
pub type JsonHadithParser = HadithParserImpl;
pub type HadithJsonParser = HadithParserImpl;
pub type JsonParser = HadithParserImpl;
