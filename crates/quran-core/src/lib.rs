//! Phase 1 — `quran-core`: the pure canonical Quran domain.
//!
//! # quran-core
//!
//! Types, the reference grammar, and `QuranQuotation` — the vocabulary every
//! other Phase-1 crate operates on. **No I/O, no async, no `sqlx`, no model
//! dependency** (invariant I2). The canonical read path must never require an
//! LLM, embedding, or vector store.
//!
//! ## Contents
//!
//! - [`error`] — `QAI-QUR-nnnn` errors and the Phase-1 `Diagnostic` contract
//! - [`numbers`] — validated `SurahNumber`, `AyahNumber`, `TokenPosition`
//! - [`enums`] — script, numbering, basmala, Unicode, status and selector enums
//! - [`edition`] — `QuranEdition`, `EditionStatistics`
//! - [`structure`] — `Surah`, `Ayah`, `Segment`, `Token`
//! - [`quotation`] — `QuranQuotation`, the only type for quoted canonical text
//! - [`text`] — grapheme utilities
//!
//! ## Constraint (enforced by `cargo xtask arch-check`)
//!
//! `quran-core` may depend only on `{domain, serde, thiserror}` plus Unicode
//! crates (`unicode-segmentation`). It must never pull in `storage`, `sqlx`,
//! `tokio`, `llm`, `embeddings`, `retrieval`, or any vector store.

pub mod edition;
pub mod enums;
pub mod error;
pub mod numbers;
pub mod quotation;
pub mod structure;
pub mod text;

pub use edition::{EditionStatistics, QuranEdition};
pub use enums::{
    BasmalaPolicy, EditionSelector, EditionStatus, NumberingScheme, RevelationPlace, SajdahKind,
    Script, SegmentKind, UnicodeForm,
};
pub use error::{Diagnostic, DiagnosticCode, QuranError, codes};
pub use numbers::{AyahNumber, SurahNumber, TokenPosition};
pub use quotation::{EditionRef, QuotationParts, QuranQuotation, TranslationRef};
pub use structure::{Ayah, Segment, Surah, Token};
pub use text::grapheme_count;
