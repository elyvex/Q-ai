//! Versioned dataset loading and validation (P2-T108, D-3.5-07).
//!
//! A dataset is a JSONL file with a header row carrying a `version` field,
//! a `reviewed_by` field, and a `dataset_class` field. The loader refuses an
//! unknown or missing version so a metric regression is always attributable
//! to a known dataset revision.

use serde::Deserialize;
use std::path::Path;

/// A dataset header row (the first line of a dataset file).
#[derive(Debug, Clone, Deserialize)]
pub struct DatasetHeader {
    /// Must be `true`.
    pub header: bool,
    /// Dataset version (must match the expected version).
    pub version: u32,
    /// Review state (always `pending-linguist` until OD-12 closes).
    pub reviewed_by: String,
    /// Dataset class (always `synthetic_test_only` for fixture data).
    pub dataset_class: String,
}

/// A loaded and validated dataset.
#[derive(Debug, Clone)]
pub struct Dataset {
    /// Dataset version.
    pub version: u32,
    /// Review state.
    pub reviewed_by: String,
    /// Dataset class.
    pub dataset_class: String,
    /// Data rows (everything after the header).
    pub rows: Vec<serde_json::Value>,
}

/// Errors that can occur when loading a dataset.
#[derive(Debug, thiserror::Error)]
pub enum DatasetError {
    /// The dataset file does not exist.
    #[error("dataset file not found: {0}")]
    NotFound(String),
    /// The header row is missing or invalid.
    #[error("dataset header missing or invalid: {0}")]
    InvalidHeader(String),
    /// The dataset version does not match the expected version.
    #[error("unknown dataset version: expected {expected}, got {actual}")]
    UnknownVersion { expected: u32, actual: u32 },
    /// A data row failed to parse.
    #[error("dataset row {index} failed to parse: {source}")]
    RowParseError { index: usize, source: serde_json::Error },
}

/// Load and validate a versioned dataset file.
///
/// The file must exist, have a valid header row with `header: true`, and
/// carry the expected `version`. Returns the parsed dataset or a typed error.
pub fn load_dataset(path: &Path, expected_version: u32) -> Result<Dataset, DatasetError> {
    let text = std::fs::read_to_string(path)
        .map_err(|_| DatasetError::NotFound(path.display().to_string()))?;
    let mut lines = text.lines();
    let header: DatasetHeader = serde_json::from_str(
        lines.next().ok_or_else(|| DatasetError::InvalidHeader("empty file".to_string()))?,
    )
    .map_err(|e| DatasetError::InvalidHeader(e.to_string()))?;

    if !header.header {
        return Err(DatasetError::InvalidHeader("header flag must be true".to_string()));
    }
    if header.version != expected_version {
        return Err(DatasetError::UnknownVersion {
            expected: expected_version,
            actual: header.version,
        });
    }

    let mut rows = Vec::new();
    for (i, line) in lines.enumerate() {
        let row: serde_json::Value = serde_json::from_str(line)
            .map_err(|e| DatasetError::RowParseError { index: i, source: e })?;
        rows.push(row);
    }

    Ok(Dataset {
        version: header.version,
        reviewed_by: header.reviewed_by,
        dataset_class: header.dataset_class,
        rows,
    })
}
