//! Phase 2 — lexical search over derived Quran text (`quran-search`, D2.3).
//!
//! Backend-independent search semantics live here: the [`FullTextIndex`]
//! port plus its query/manifest types. Engine adapters (FTS5 now, Tantivy
//! later) sit below the port; tools, counting, and explanation sit above it.
//!
//! This crate is lexical + morphological only: no LLM, embedding, retrieval,
//! or vector-store dependency, ever (enforced by `cargo xtask arch-check`,
//! AC-P2-36).

pub mod error;
pub mod fts5;
pub mod highlight;
pub mod hit;
pub mod index;
pub mod model;
pub mod regex;
pub mod skeleton;
pub mod tokenizer;

pub use domain::SemVer;
pub use error::{Diagnostic, DiagnosticCode, IndexError, codes};
pub use fts5::Fts5Index;
pub use highlight::apply_markers;
pub use regex::{DFA_SIZE_LIMIT, MAX_PATTERN_LEN, NFA_SIZE_LIMIT, compile_dfa, first_match};
pub use hit::{ScoreExplain, SearchHit, SearchHitParts, Segmentation, Warning};
pub use index::FullTextIndex;
pub use model::{
    CommitStamp, FieldId, Filter, FtsBackend, FtsDoc, FtsHit, FtsIntegrityReport, FtsQuery,
    FtsResults, FtsSchema, FtsStats, IndexManifest, ResultOrder, SearchOpts,
};
pub use skeleton::{BuiltSkeleton, ayah_skeleton, skeletons_for_surah};
pub use tokenizer::{ArTokenizer, INDEXED_FIELDS, TokenizerFamily, profile_for_field};
