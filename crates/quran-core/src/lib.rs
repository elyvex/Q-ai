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
//! - [`catalog`] — upstream edition metadata, qiraʾah/riwayah, integrity refs
//! - [`structure`] — `Surah`, `Ayah`, `Segment`, `Token`
//! - [`reference`] — the frozen reference grammar (parser + serializer, ADR-0102)
//! - [`quotation`] — `QuranQuotation`, the only type for quoted canonical text
//! - [`text`] — grapheme utilities
//!
//! ## Constraint (enforced by `cargo xtask arch-check`)
//!
//! `quran-core` may depend only on `{domain, serde, thiserror}` plus Unicode
//! crates (`unicode-segmentation`). It must never pull in `storage`, `sqlx`,
//! `tokio`, `llm`, `embeddings`, `retrieval`, or any vector store.

pub mod catalog;
pub mod edition;
pub mod enums;
pub mod error;
pub mod numbers;
pub mod quotation;
pub mod reference;
pub mod structure;
pub mod text;
pub mod view;

pub use catalog::{
    Attribution, DataQualityFlag, EditionContentKind, IntegrityManifestRef, IntegrityScope, Qiraah,
    Riwayah, UpstreamEditionRef, UpstreamEditionSlug, UpstreamEditionSlugError, UpstreamSourceRef,
};
pub use edition::{EditionStatistics, QuranEdition};
pub use enums::{
    BasmalaPolicy, ContextBoundary, EditionSelector, EditionStatus, NumberingScheme,
    RevelationPlace, SajdahKind, Script, SegmentKind, UnicodeForm, is_valid_slug,
};
pub use error::{Diagnostic, DiagnosticCode, QuranError, codes};
pub use numbers::{AyahNumber, SurahNumber, TokenPosition};
pub use quotation::{EditionRef, QuotationParts, QuranQuotation, TranslationRef};
pub use reference::{
    DivisionKind, QuranRef, ResolvedRef, canonical_form, parse, resolve, serialize,
};
pub use structure::{Ayah, Segment, Surah, Token};
pub use text::grapheme_count;
pub use view::{
    AttributedGloss, AttributedTranslation, AyahLocation, AyahOptions, AyahView, ContextSpec,
    ContextView,
};
