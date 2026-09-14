//! Phase 1 — `quran-corpus`: intermediate format, adapters, tokenizer,
//! validator, differ, and importer logic.
//!
//! # quran-corpus
//!
//! Everything between a raw dataset and the canonical tables lives here. The
//! crate never touches an LLM, embedding, retrieval, or vector store
//! (invariant I2, enforced by `cargo xtask arch-check`).
//!
//! ## Contents
//!
//! - [`format`] — the normalized intermediate edition format
//! - [`adapters`] — dataset-shape adapters into the intermediate format
//! - [`tokenize`] — lossless whitespace-preserving tokenization
//! - [`hashing`] — `text_hash` / `structure_hash` / `token_order_hash`
//! - [`unicode`] — Unicode auditor for canonical text
//! - [`validation`] — the QV-001…QV-028 corpus validator
//! - [`differ`] — char-level edition differ
//! - [`import`] — the 13-checkpoint canonical importer
//! - [`error`] — `QAI-QUR-02xx` corpus infrastructure errors

pub mod adapters;
pub mod differ;
pub mod error;
pub mod format;
pub mod hashing;
pub mod import;
pub mod tokenize;
pub mod unicode;
pub mod validation;

pub use adapters::{CsvAdapter, EditionAdapter, JsonAdapter, parse_with_adapter};
pub use differ::{
    AyahChange, ChangeKind, DIFFER_NAME, DIFFER_VERSION, EditionDiff, diff_ayahs,
};
pub use error::{CorpusError, codes};
pub use format::{
    AyahSource, EditionMeta, EditionSource, ExpectedCounts, FORMAT_TAG, FORMAT_VERSION,
    SurahSource, TokenSource, TokenizationPolicy,
};
pub use hashing::{AyahLayout, TokenOrder, structure_hash, tagged, text_hash, token_order_hash};
pub use import::{
    ImportCheckpoint, ImportInput, ImportOptions, ImportOutcome, ImportSuccess, run_import,
};
pub use tokenize::{ComputedToken, TokenizedAyah, reconstruct, tokenize};
pub use unicode::{
    ForbiddenPoint, find_forbidden, first_unexpected, is_expected_code_point, normalization_form,
};
pub use validation::{
    Finding, Outcome, QuranEditionValidator, Severity, VALIDATOR_NAME, VALIDATOR_VERSION,
    ValidationReport, check_file_hash, intermediate_hash, validate_edition,
};
