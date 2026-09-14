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
//! - [`error`] — `QAI-QUR-02xx` corpus infrastructure errors

pub mod adapters;
pub mod error;
pub mod format;

pub use adapters::{CsvAdapter, EditionAdapter, JsonAdapter, parse_with_adapter};
pub use error::{CorpusError, codes};
pub use format::{
    AyahSource, EditionMeta, EditionSource, ExpectedCounts, FORMAT_TAG, FORMAT_VERSION,
    SurahSource, TokenSource, TokenizationPolicy,
};
