//! Hadith-specific validation rules.

use crate::{ParsedHadith, ValidationError};
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct HadithValidator {
    known_collections: HashSet<String>,
}

impl HadithValidator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_collections<I, S>(collections: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self { known_collections: collections.into_iter().map(Into::into).collect() }
    }

    pub fn validate(&self, hadith: &ParsedHadith) -> Result<(), ValidationError> {
        if hadith.source_number.trim().is_empty() {
            return Err(ValidationError {
                code: "QAI-HAD-ING-0001".to_string(),
                message: "source_number is required".to_string(),
                location: None,
            });
        }
        if hadith.matn_text.trim().is_empty() {
            return Err(ValidationError {
                code: "QAI-HAD-ING-0002".to_string(),
                message: "matn_text is required".to_string(),
                location: None,
            });
        }
        // Consensus-merged grading values are forbidden.
        for key in hadith.extra.keys() {
            let lower = key.to_lowercase();
            if lower.contains("consensus") || lower.contains("majority") {
                return Err(ValidationError {
                    code: "QAI-HAD-ING-0003".to_string(),
                    message: "consensus-merged grading fields are not allowed".to_string(),
                    location: None,
                });
            }
        }
        for grading in &hadith.gradings {
            if grading.scholar_name.trim().is_empty() {
                return Err(ValidationError {
                    code: "QAI-HAD-ING-0004".to_string(),
                    message: "grading scholar_name is required".to_string(),
                    location: None,
                });
            }
            if grading.source_citation.trim().is_empty() {
                return Err(ValidationError {
                    code: "QAI-HAD-ING-0005".to_string(),
                    message: "grading source_citation is required".to_string(),
                    location: None,
                });
            }
            for key in grading.extra.keys() {
                let lower = key.to_lowercase();
                if lower.contains("consensus") || lower.contains("majority") {
                    return Err(ValidationError {
                        code: "QAI-HAD-ING-0003".to_string(),
                        message: "consensus-merged grading fields are not allowed".to_string(),
                        location: None,
                    });
                }
            }
        }
        // Collection check: when known collections are registered, require existence.
        if !self.known_collections.is_empty()
            && !self.known_collections.contains(&hadith.collection)
        {
            return Err(ValidationError {
                code: "QAI-HAD-ING-0006".to_string(),
                message: format!("collection not found: {}", hadith.collection),
                location: None,
            });
        }
        // When no collection registry is provided, still reject empty collection.
        if hadith.collection.trim().is_empty() {
            return Err(ValidationError {
                code: "QAI-HAD-ING-0006".to_string(),
                message: "collection is required".to_string(),
                location: None,
            });
        }
        Ok(())
    }

    // Associated-style static helper for callers that use HadithValidator::validate(hadith)
    pub fn validate_hadith(hadith: &ParsedHadith) -> Result<(), ValidationError> {
        HadithValidator::default().validate(hadith)
    }
}

// Static trait-like methods on a zero-sized validator.
impl HadithValidator {
    pub fn validate_static(hadith: &ParsedHadith) -> Result<(), ValidationError> {
        HadithValidator::default().validate(hadith)
    }
}
