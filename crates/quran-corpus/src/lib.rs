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
//! - [`error`] — `QAI-QUR-02xx` corpus infrastructure errors

pub mod adapters;
pub mod error;
pub mod format;
pub mod hashing;
pub mod tokenize;
pub mod unicode;

pub use adapters::{CsvAdapter, EditionAdapter, JsonAdapter, parse_with_adapter};
pub use error::{CorpusError, codes};
pub use format::{
    AyahSource, EditionMeta, EditionSource, ExpectedCounts, SurahSource, TokenSource,
    TokenizationPolicy, FORMAT_TAG, FORMAT_VERSION,
};
pub use hashing::{AyahLayout, TokenOrder, structure_hash, text_hash, token_order_hash};
pub use tokenize::{ComputedToken, TokenizedAyah, reconstruct, tokenize};
pub use unicode::{ForbiddenPoint, first_unexpected, find_forbidden, is_expected_code_point, normalization_form};
