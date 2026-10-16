//! Hadith ingestion pipeline.

pub mod parser;
pub mod validate;

pub use parser::{
    HadithParser, HadithParserImpl, JsonHadithParser, HadithJsonParser, JsonParser, JsonHadith,
    JsonGrading, parse,
};
pub use sources::{Source, ValidationError, ValidationReport};
pub use domain::SourceId;
pub use validate::HadithValidator;

use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct StagedRecord {
    pub source_id: String,
    pub record: ParsedHadith,
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivationError {
    NothingStaged,
    ValidationFailed,
}

#[derive(Default)]
pub struct IngestionPipeline {
    staged: Mutex<HashMap<String, Vec<StagedRecord>>>,
    active: Mutex<HashMap<String, Vec<StagedRecord>>>,
    validator: HadithValidator,
}

impl IngestionPipeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_validator(validator: HadithValidator) -> Self {
        Self { validator, ..Self::default() }
    }

    pub fn discover(&self, source_url: &str) -> Source {
        use domain::Timestamp;
        use sources::SourceContentType;
        let now = Timestamp::now();
        Source {
            id: SourceId::new(),
            title: source_url.to_string(),
            alternate_titles: Vec::new(),
            content_type: SourceContentType::HadithCollection,
            authors: Vec::new(),
            compiler: None,
            translator: None,
            editor: None,
            publisher: None,
            language: None,
            tradition: None,
            school: None,
            identifiers: sources::SourceIdentifiers::default(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn stage<S: ToString>(
        &self,
        source_id: S,
        data: &[ParsedHadith],
    ) -> Vec<StagedRecord> {
        let key = source_id.to_string();
        let mut records = Vec::new();
        for hadith in data {
            records.push(StagedRecord {
                source_id: key.clone(),
                record: hadith.clone(),
                state: "staged".to_string(),
            });
        }
        let mut staged = self.staged.lock().unwrap();
        staged.insert(key, records.clone());
        records
    }

    pub fn validate<S: ToString>(&self, source_id: S) -> ValidationReport {
        let key = source_id.to_string();
        let staged = self.staged.lock().unwrap();
        let records = staged.get(&key).cloned().unwrap_or_default();
        drop(staged);
        let mut errors = Vec::new();
        for record in &records {
            if let Err(e) = self.validator.validate(&record.record) {
                errors.push(e);
            }
        }
        ValidationReport {
            valid: errors.is_empty(),
            validator_name: "hadith".to_string(),
            errors,
            warnings: Vec::new(),
        }
    }

    pub fn activate<S: ToString>(&self, source_id: S) -> Result<(), ActivationError> {
        let key = source_id.to_string();
        let validation = self.validate(&key);
        if !validation.valid {
            return Err(ActivationError::ValidationFailed);
        }
        let mut staged = self.staged.lock().unwrap();
        let records = staged.remove(&key).ok_or(ActivationError::NothingStaged)?;
        drop(staged);
        let mut active = self.active.lock().unwrap();
        active.insert(key, records);
        Ok(())
    }

    pub fn staged<S: ToString>(&self, source_id: S) -> Vec<StagedRecord> {
        let key = source_id.to_string();
        let staged = self.staged.lock().unwrap();
        staged.get(&key).cloned().unwrap_or_default()
    }

    pub fn active<S: ToString>(&self, source_id: S) -> Vec<StagedRecord> {
        let key = source_id.to_string();
        let active = self.active.lock().unwrap();
        active.get(&key).cloned().unwrap_or_default()
    }
}

// ParseError must be defined before parser re-export? It is re-exported from parser; re-export ParsedHadith etc already. ParseError defined here? parser module references crate::ParseError. Need define below? Module parser compiled before? Rust allows forward? Type items order doesn't matter. Define here.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParseError {
    pub message: String,
    pub location: Option<String>,
}

// Parsed types needed by parser module. But parser module references crate::{...}; items must be in crate root. Define before use? Rust resolves across modules; order irrelevant for items.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ParsedHadith {
    pub collection: String,
    pub source_number: String,
    pub matn_text: String,
    pub isnad_text: Option<String>,
    pub narrator_names: Vec<String>,
    pub gradings: Vec<ParsedGrading>,
    #[serde(default)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ParsedGrading {
    pub scholar_name: String,
    pub grading_value: String,
    pub source_citation: String,
    #[serde(default)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    const FIXTURE: &str = r#"[
      {
        "collection": "bukhari",
        "source_number": "bukhari:1",
        "matn_text": "إنما الأعمال بالنيات",
        "isnad_text": "حدثنا فلان",
        "narrator_names": ["فلان"],
        "gradings": [
          {"scholar_name": "al-Bukhari", "grading_value": "sahih", "source_citation": "bukhari#1"}
        ]
      }
    ]"#;

    #[test]
    fn parse_single_and_array() {
        let one = r#"{"collection":"bukhari","source_number":"1","matn_text":"hello"}"#;
        let recs = parse(one).unwrap();
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].source_number, "1");
        let arr = parse(FIXTURE).unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].gradings.len(), 1);
        assert_eq!(arr[0].gradings[0].scholar_name, "al-Bukhari");
        assert!(parse("").unwrap().is_empty());
    }

    #[test]
    fn parse_rejects_consensus_merged() {
        let bad = r#"{"collection":"x","source_number":"1","matn_text":"y","consensus":"true"}"#;
        assert!(parse(bad).is_err());
        let bad2 = r#"[{"collection":"x","source_number":"1","matn_text":"y","gradings":[{"scholar_name":"z","grading_value":"sahih","source_citation":"c","consensus":"merged"}]}]"#;
        assert!(parse(bad2).is_err());
    }

    #[test]
    fn validate_requires_fields() {
        let validator = HadithValidator::default();
        let mut h = ParsedHadith { collection: "bukhari".into(), source_number: "1".into(), matn_text: "m".into(), ..Default::default() };
        assert!(validator.validate(&h).is_ok());

        h.source_number = "   ".into();
        assert!(validator.validate(&h).is_err());
        h.source_number = "1".into();
        h.matn_text = "".into();
        assert!(validator.validate(&h).is_err());
        h.matn_text = "m".into();
        h.collection = "".into();
        assert!(validator.validate(&h).is_err());

        // grading fields
        let mut h2 = ParsedHadith { collection: "bukhari".into(), source_number: "1".into(), matn_text: "m".into(), gradings: vec![ParsedGrading { scholar_name: "".into(), grading_value: "g".into(), source_citation: "c".into(), ..Default::default() }], ..Default::default() };
        assert!(validator.validate(&h2).is_err());
        h2.gradings[0].scholar_name = "s".into();
        h2.gradings[0].source_citation = "".into();
        assert!(validator.validate(&h2).is_err());
    }

    #[test]
    fn validate_with_collections() {
        let v = HadithValidator::with_collections(["bukhari"]);
        let ok = ParsedHadith { collection: "bukhari".into(), source_number: "1".into(), matn_text: "m".into(), ..Default::default() };
        assert!(v.validate(&ok).is_ok());
        let bad = ParsedHadith { collection: "tirmidhi".into(), source_number: "1".into(), matn_text: "m".into(), ..Default::default() };
        assert!(v.validate(&bad).is_err());
    }

    #[test]
    fn validate_rejects_consensus_extra() {
        let v = HadithValidator::default();
        let mut h = ParsedHadith { collection: "bukhari".into(), source_number: "1".into(), matn_text: "m".into(), ..Default::default() };
        h.extra.insert("consensus_grading".into(), serde_json::json!(true));
        assert!(v.validate(&h).is_err());
        h.extra.clear();
        let mut g = ParsedGrading { scholar_name: "s".into(), grading_value: "g".into(), source_citation: "c".into(), ..Default::default() };
        g.extra.insert("majority".into(), serde_json::json!(true));
        h.gradings = vec![g];
        assert!(v.validate(&h).is_err());
    }

    #[test]
    fn pipeline_full_flow() {
        let pipeline = IngestionPipeline::new();
        let src = pipeline.discover("https://example.invalid/hadith");
        let parsed = parse(FIXTURE).unwrap();
        let staged = pipeline.stage(src.id, &parsed);
        assert_eq!(staged.len(), 1);
        assert_eq!(staged[0].state, "staged");
        assert_eq!(pipeline.staged(src.id).len(), 1);

        let report = pipeline.validate(src.id);
        assert!(report.valid, "{:?}", report.errors);

        assert!(pipeline.activate(src.id).is_ok());
        assert!(pipeline.staged(src.id).is_empty());
        assert_eq!(pipeline.active(src.id).len(), 1);
        // re-activate on empty fails
        assert!(pipeline.activate(src.id).is_err());
    }

    #[test]
    fn pipeline_rejects_consensus_grading() {
        let _pipeline = IngestionPipeline::new();
        let bad = r#"[{"collection":"bukhari","source_number":"1","matn_text":"m","consensus":"true"}]"#;
        // parse rejects first
        assert!(parse(bad).is_err());
    }

    #[test]
    fn pipeline_rejects_invalid_collection_when_validator_configured() {
        let v = HadithValidator::with_collections(["bukhari"]);
        let pipeline = IngestionPipeline::with_validator(v);
        let src = pipeline.discover("url");
        let data = vec![ParsedHadith {
            collection: "other".into(),
            source_number: "1".into(),
            matn_text: "m".into(),
            ..Default::default()
        }];
        pipeline.stage(src.id, &data);
        let report = pipeline.validate(src.id);
        assert!(!report.valid);
        assert!(!report.errors.is_empty());
        assert!(pipeline.activate(src.id).is_err());
    }

    #[test]
    fn pipeline_source_number_preserved() {
        let pipeline = IngestionPipeline::new();
        let src = pipeline.discover("url");
        let parsed = parse(r#"[{"collection":"bukhari","source_number":"bukhari:42","matn_text":"m"}]"#).unwrap();
        pipeline.stage(src.id, &parsed);
        pipeline.activate(src.id).unwrap();
        let active = pipeline.active(src.id);
        assert_eq!(active[0].record.source_number, "bukhari:42");
    }

    #[test]
    fn activate_nothing_staged_returns_error() {
        let pipeline = IngestionPipeline::new();
        let src = pipeline.discover("url");
        assert_eq!(pipeline.activate(src.id), Err(ActivationError::NothingStaged));
    }
}
