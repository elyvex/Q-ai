//! Phase 2 — morphology lexicon types (`quran-morphology`, D2.6–D2.8).
//!
//! This crate is **pure logic + serde types**: intermediate format,
//! adapters, alignment, MV validation, analysis policy, comparison, word
//! families, and the unified tagset. It bundles **no dataset** and performs
//! no I/O, no database access, and no async work.
//!
//! The test lexicon under `fixtures/quran/lexicon/` is **synthetic and
//! labeled** (`synthetic_test_only`); it is never dataset ground truth or
//! scholarly data. **Linguist sign-off is pending** for the tagset, root
//! conventions, and family relations — nothing in this crate claims
//! otherwise.
//!
//! The crate must never gain a dependency on `llm`, `embeddings`,
//! `retrieval`, or any vector-store crate (AC-P2-36).

pub mod adapter_csv;
pub mod adapter_json;
pub mod align;
pub mod compare;
pub mod dataset;
pub mod error;
pub mod family;
pub mod policy;
pub mod tagset;
pub mod validate;

pub use adapter_csv::parse_flat_csv;
pub use adapter_json::parse_array_shape;
pub use align::{AlignmentEntry, AlignmentReport, AlignmentTable, DirectKey, align};
pub use compare::{FieldVerdict, Verdict, compare};
pub use dataset::{
    DatasetRef, IntermediateMorphology, InventoryToken, MorphemeSegment, TokenAnalysis,
    load_intermediate,
};
pub use error::{Diagnostic, DiagnosticCode, MorphologyError, codes};
pub use family::{
    ComputationalSuggestion, FamilyMember, FamilyRelation, ReviewPromotion,
    SUGGESTION_CONFIDENCE_FLOOR, SUGGESTION_LABEL, explain_relation, relation_name,
    suggest_computational,
};
pub use policy::{AnalysisPolicy, PolicyAnalysis, PolicyView, apply_policy};
pub use tagset::{TagMapping, UnifiedTag, map_tag};
pub use validate::{Finding, RULES, Severity, ValidationRule, has_blocking, has_fatal, validate};
