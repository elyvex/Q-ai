//! Phase 2 — versioned Arabic normalization (`quran-normalization`, D2.1).
//!
//! Normalization produces **derived** search copies of canonical text and
//! never mutates it (I8). Every transformation emits a [`SpanMap`](span::SpanMap)
//! so matches map back to exact canonical character ranges (I10), and every
//! search result will carry the ordered rule set actually applied (I9, via
//! `NormalizationTrace` in M1b).
//!
//! This crate is pure logic: no I/O, no async runtime, no database. It depends
//! only on `domain`-adjacent primitives plus `serde`/`thiserror`, and must
//! never gain a dependency on `llm`, `embeddings`, `retrieval`, or any
//! vector-store crate (enforced by `cargo xtask arch-check`, AC-P2-36).

pub mod error;
pub mod rule;
pub mod rules;
pub mod span;

pub use error::{Diagnostic, DiagnosticCode, NormalizationError, codes};
pub use rule::{NormalizationRule, NormalizedText, RuleId, RuleKind};
pub use rules::all_rules;
pub use span::{CanonicalSpan, SpanMap, SpanSegment};
