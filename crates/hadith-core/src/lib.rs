//! Hadith domain types (Phase 6).
//!
//! # hadith_core
//!
//! Source-aware hadith records following the Quran corpus pattern: collection,
//! edition, and source numbering preserved verbatim. Matn/isnad separation where
//! the source supports it. Per-scholar grading records that never merge into
//! false consensus.

pub mod collection;
pub mod grading;
pub mod record;

pub use collection::{HadithCollection, HadithCollectionId, Tradition};
pub use grading::{GradingId, GradingRecord, ProvenanceLayer, ReviewDecision};
pub use record::{HadithId, HadithRecord, RecordStatus};
