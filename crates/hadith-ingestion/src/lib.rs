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
