//! Repository trait and row types for the canonical Quran corpus.
//!
//! Stringly-typed rows mirror the Phase-0 convention: the storage layer moves
//! bytes, and typed domain mapping lives above it. Method bodies are stubs
//! returning `Err(StorageError::StorageUnavailable)` until the SQLite
//! implementation provides them.
//!
//! Canonical-write rule (AC-P1-09): this trait exposes **no** row-level
//! canonical insert. The only canonical-write path is [`QuranRepository::activate_edition`]
//! (staging → canonical move plus the pointer flip in one transaction) and
//! [`QuranRepository::rollback_edition`], both of which record the approving
//! identity, plus the human-gated maintenance mutators
//! [`QuranRepository::set_edition_status`] and
//! [`QuranRepository::set_edition_verification`] (status / `verified_*`
//! metadata only — trigger-guarded identity, hashes, and ayah text are
//! unreachable through them). Raw SQL is still blocked by the insert-only triggers.

use async_trait::async_trait;

use crate::error::StorageError;

// ─── Row types ────────────────────────────────────────────────────────────

/// A row in `quran_editions`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuranEditionRow {
    pub id: String,
    pub slug: String,
    pub version: String,
    pub name: String,
    pub script: String,
    pub riwayah: Option<String>,
    pub qiraah: Option<String>,
    pub publisher: Option<String>,
    pub source_url: Option<String>,
    /// Exact upstream identifier, preserved verbatim (ADR-0101, OD-01).
    pub upstream_edition_slug: Option<String>,
    /// Q-ai-internal stable id mapped from the upstream slug.
    pub qai_edition_id: Option<String>,
    /// Explicit primary/default designation (D-07); never redefines the
    /// active-edition pointer.
    pub is_primary: bool,
    pub language: String,
    pub verse_numbering_scheme: String,
    pub basmala_policy: String,
    pub unicode_normalization: String,
    pub license_json: String,
    pub text_hash: String,
    pub structure_hash: String,
    pub token_order_hash: String,
    pub manifest_hash: String,
    pub source_version_id: String,
    pub statistics_json: String,
    pub status: String,
    pub imported_at: String,
    pub verified_at: Option<String>,
    pub verified_by: Option<String>,
    pub verification_method: Option<String>,
    pub activated_at: Option<String>,
    pub deprecated_at: Option<String>,
}

/// The single active-edition pointer row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveEditionRow {
    pub edition_id: String,
    pub corpus_generation: i64,
    pub activated_at: String,
    pub activated_by: String,
    pub approval_id: String,
}

/// A row in `quran_surahs` (and its staging mirror, minus the run id).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurahRow {
    pub edition_id: String,
    pub number: i64,
    pub name_arabic: String,
    pub name_transliteration: Option<String>,
    pub name_translations_json: String,
    pub ayah_count: i64,
    pub revelation_place: Option<String>,
    pub revelation_order: Option<i64>,
    pub basmala: String,
    pub ruku_count: Option<i64>,
    pub metadata_provenance_id: Option<String>,
}

/// A row in `quran_ayahs` (and its staging mirror, minus the run id).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AyahRow {
    pub edition_id: String,
    pub surah: i64,
    pub ayah: i64,
    pub text: String,
    pub text_hash: String,
    pub char_count: i64,
    pub token_count: i64,
    pub global_ayah_index: i64,
    pub juz: Option<i64>,
    pub hizb: Option<i64>,
    pub rub: Option<i64>,
    pub manzil: Option<i64>,
    pub ruku: Option<i64>,
    pub page: Option<i64>,
    pub sajdah: Option<String>,
    pub provenance_id: String,
}

/// A row in `quran_tokens` (and its staging mirror, minus the run id).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenRow {
    pub edition_id: String,
    pub surah: i64,
    pub ayah: i64,
    pub position: i64,
    pub surface: String,
    pub surface_hash: String,
    pub char_start: i64,
    pub char_end: i64,
    pub byte_start: i64,
    pub byte_end: i64,
    pub is_pause_mark: bool,
    pub global_token_index: i64,
}

/// A row in `quran_token_separators` (and its staging mirror).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeparatorRow {
    pub edition_id: String,
    pub surah: i64,
    pub ayah: i64,
    pub after_position: i64,
    pub separator: String,
}

/// A row in `quran_divisions` (and its staging mirror, minus the run id).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DivisionRow {
    pub edition_id: String,
    pub kind: String,
    pub number: i64,
    pub start_surah: i64,
    pub start_ayah: i64,
    pub end_surah: i64,
    pub end_ayah: i64,
    pub start_global: i64,
    pub end_global: i64,
    pub label: Option<String>,
    pub provenance_id: String,
}

/// A staged edition reference: the run holding it plus its edition id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedEditionRef {
    /// Import run id.
    pub run_id: String,
    /// Staged edition id.
    pub edition_id: String,
}

/// A row in `quran_import_runs`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportRunRow {
    /// Run id.
    pub run_id: String,
    pub job_id: Option<String>,
    pub edition_slug: String,
    pub edition_version: String,
    pub adapter: String,
    pub state: String,
    pub created_at: String,
}

/// A row in `validation_reports`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReportRow {
    pub id: String,
    pub subject_urn: String,
    pub validator: String,
    pub validator_version: String,
    pub outcome: String,
    pub fatal_count: i64,
    pub error_count: i64,
    pub warning_count: i64,
    pub findings_json: String,
    pub created_at: String,
}

/// A row in `difference_reports`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DifferenceReportRow {
    pub id: String,
    pub subject_urn: String,
    pub from_version: String,
    pub to_version: String,
    pub differ: String,
    pub differ_version: String,
    pub summary_json: String,
    pub details_json: String,
    pub created_at: String,
}

/// A row in `citations`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitationRow {
    pub id: String,
    pub kind: String,
    pub canonical_reference: String,
    pub source_id: Option<String>,
    pub source_version_id: Option<String>,
    pub edition_ref: Option<String>,
    pub location_json: String,
    pub quoted_text_hash: Option<String>,
    pub ingestion_version: String,
    pub resolved_at: String,
    pub verdict: String,
}

/// A row in `translation_editions`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationEditionRow {
    pub id: String,
    pub slug: String,
    pub version: String,
    pub name: String,
    pub translator: String,
    pub language: String,
    pub aligned_edition_id: String,
    pub numbering_scheme: String,
    pub license_json: String,
    pub trust_level: String,
    pub source_version_id: String,
    pub text_hash: String,
    pub status: String,
    pub imported_at: String,
}

/// A row in `translation_passages`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationPassageRow {
    pub translation_edition_id: String,
    pub surah: i64,
    pub ayah: i64,
    pub text: String,
    pub footnotes_json: String,
    pub provenance_id: String,
}

/// A row in `word_glosses`: one attributed word-level gloss aligned to a
/// canonical token position (Phase 1, P1-T38; migration `0010`).
///
/// Glosses are a separate attributed dataset, never canonical text
/// (principle 5): `gloss_dataset_id` names the `sources` row carrying the
/// dataset's provenance, and `edition_id` names the aligned Arabic edition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordGlossRow {
    pub gloss_dataset_id: String,
    pub edition_id: String,
    pub surah: i64,
    pub ayah: i64,
    /// 1-based token position within the ayah.
    pub position: i64,
    pub language: String,
    pub gloss: String,
    pub provenance_id: String,
}

/// One index pointer row: the serving generation for an index id.
///
/// The pointer is the only mutable Phase-2 catalog row by design; flips
/// happen in one transaction with the build-run state change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexPointerRow {
    /// Index identity (`quran.ayah.v1`).
    pub index_id: String,
    /// Serving build generation.
    pub generation: i64,
    /// Serving manifest as JSON.
    pub manifest_json: String,
    /// Flip timestamp (RFC 3339).
    pub updated_at: String,
    /// Operator principal id.
    pub updated_by: String,
}

/// One index build-run row: staged → verifying → active|failed, with the
/// previous active generation moving to superseded on flip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexBuildRunRow {
    /// Run id (unique per attempt).
    pub id: String,
    /// Index identity.
    pub index_id: String,
    /// Build generation.
    pub generation: i64,
    /// Corpus generation indexed.
    pub corpus_generation: i64,
    /// `staged` | `verifying` | `active` | `failed` | `superseded`.
    pub state: String,
    /// Documents committed.
    pub doc_count: i64,
    /// Manifest content hash.
    pub manifest_hash: String,
    /// Failure detail (failed runs only).
    pub error: Option<String>,
    /// Start timestamp (RFC 3339).
    pub started_at: String,
    /// Finish timestamp (RFC 3339, if finished).
    pub finished_at: Option<String>,
}

/// Derived token-form column for exact-SQL counting (P2-T95).
///
/// Maps 1:1 to `quran_token_forms` columns; the mapping is a Rust-side
/// allowlist so counting queries can never interpolate caller SQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormColumn {
    /// `simple` (`L2.marks`).
    Simple,
    /// `bare` (`L3.diacritics`).
    Bare,
    /// `hamza_folded` (`L4.hamza`).
    HamzaFolded,
    /// `folded` (`L5.codepoints`).
    Folded,
    /// `affix_stripped` (`L7.affix`, heuristic).
    AffixStripped,
}

impl FormColumn {
    /// Physical column name (allowlisted; never caller-supplied).
    #[must_use]
    pub const fn column_name(self) -> &'static str {
        match self {
            Self::Simple => "simple",
            Self::Bare => "bare",
            Self::HamzaFolded => "hamza_folded",
            Self::Folded => "folded",
            Self::AffixStripped => "affix_stripped",
        }
    }

    /// Profile whose pipeline produces this column's values.
    #[must_use]
    pub const fn profile_name(self) -> &'static str {
        match self {
            Self::Simple => "L2.marks",
            Self::Bare => "L3.diacritics",
            Self::HamzaFolded => "L4.hamza",
            Self::Folded => "L5.codepoints",
            Self::AffixStripped => "L7.affix",
        }
    }
}

/// One derived token-form row (migration `0014`, Layer D).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenFormRow {
    /// Canonical edition id.
    pub edition_id: String,
    /// Surah number.
    pub surah: i64,
    /// Ayah number.
    pub ayah: i64,
    /// 1-based token position.
    pub position: i64,
    /// `L2.marks` form.
    pub simple: String,
    /// `L3.diacritics` form.
    pub bare: String,
    /// `L4.hamza` form.
    pub hamza_folded: String,
    /// `L5.codepoints` form.
    pub folded: String,
    /// `L7.affix` form (token-level only).
    pub affix_stripped: String,
    /// Reserved (N23, Phase 4); always `None` in Phase 2.
    pub transliteration: Option<String>,
    /// Reserved (N24, experimental); always `None` in Phase 2.
    pub phonetic: Option<String>,
    /// Producing rule set (e.g. `quran-normalization`).
    pub rule_set_id: String,
    /// Profile ladder version (e.g. `1.0.0`).
    pub rule_set_version: String,
    /// Corpus generation at build time.
    pub corpus_generation: i64,
    /// Layer D provenance record.
    pub provenance_id: String,
}

/// One derived ayah-form row (migration `0014`, Layer D).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AyahFormRow {
    /// Canonical edition id.
    pub edition_id: String,
    /// Surah number.
    pub surah: i64,
    /// Ayah number.
    pub ayah: i64,
    /// `L2.marks` form.
    pub simple: String,
    /// `L3.diacritics` form.
    pub bare: String,
    /// `L4.hamza` form.
    pub hamza_folded: String,
    /// `L5.codepoints` form.
    pub folded: String,
    /// Reserved (N23, Phase 4); always `None` in Phase 2.
    pub transliteration: Option<String>,
    /// Producing rule set (e.g. `quran-normalization`).
    pub rule_set_id: String,
    /// Profile ladder version (e.g. `1.0.0`).
    pub rule_set_version: String,
    /// Corpus generation at build time.
    pub corpus_generation: i64,
    /// Layer D provenance record.
    pub provenance_id: String,
}

/// One skeleton row: an ayah skeleton or a 3-ayah window (migration `0014`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkeletonRow {
    /// Canonical edition id.
    pub edition_id: String,
    /// Surah number (windows never cross a surah boundary).
    pub surah: i64,
    /// Window start ayah (equals `ayah_end` for single-ayah rows).
    pub ayah_start: i64,
    /// Window end ayah (`ayah_start..=ayah_start+2`).
    pub ayah_end: i64,
    /// `L6.skeleton` form.
    pub skeleton: String,
    /// Producing rule set (e.g. `quran-normalization`).
    pub rule_set_id: String,
    /// Profile ladder version (e.g. `1.0.0`).
    pub rule_set_version: String,
    /// Corpus generation at build time.
    pub corpus_generation: i64,
    /// Layer D provenance record.
    pub provenance_id: String,
}

/// One seeded normalization rule (migration `0013`, append-only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizationRuleRow {
    /// Short id (`N01`…`N22`).
    pub rule_id: String,
    /// Implementation version (`MAJOR.MINOR.PATCH`).
    pub version: String,
    /// `deterministic` or `heuristic`.
    pub kind: String,
    /// Human description of the effect.
    pub description: String,
}

/// One normalization profile version (migration `0013`, append-only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizationProfileRow {
    /// Ladder id (`L0.exact`…`L8.fuzzy`).
    pub profile_id: String,
    /// Profile version (`MAJOR.MINOR.PATCH`).
    pub version: String,
    /// PRD §8.2 label.
    pub label: String,
    /// Ordered rule ids as a JSON array.
    pub rules_json: String,
    /// Whether an index field is built from this profile.
    pub indexed: bool,
    /// Whether results must render the heuristic-matched label.
    pub heuristic: bool,
    /// Experimental, off-by-default profiles.
    pub experimental: bool,
}

// ─── Repository trait ─────────────────────────────────────────────────────

/// Repository for the canonical Quran corpus.
///
/// Manages `quran_editions`, `quran_active_edition`, `quran_surahs`,
/// `quran_ayahs`, `quran_tokens`, `quran_token_separators`, `quran_segments`,
/// `quran_divisions`, the `quran_stg_*` staging mirrors, `quran_import_runs`,
/// `validation_reports`, `difference_reports`, `citations`,
/// `translation_editions`, and `translation_passages`, plus the Phase-2
/// normalization catalog (`normalization_rules`, `normalization_profiles`).
#[async_trait]
pub trait QuranRepository: Send + Sync {
    /// Record a new import run.
    async fn insert_import_run(&mut self, _row: ImportRunRow) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Fetch an import run by id.
    async fn get_import_run(&self, _run_id: &str) -> Result<Option<ImportRunRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Find a staged edition by slug and version.
    async fn find_staged_edition(
        &self,
        _slug: &str,
        _version: &str,
    ) -> Result<Option<StagedEditionRef>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Fetch one staged edition row (for staged re-validation).
    async fn get_stg_edition(
        &self,
        _run_id: &str,
        _edition_id: &str,
    ) -> Result<Option<QuranEditionRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List staged surahs for a run ordered by number.
    async fn list_stg_surahs(&self, _run_id: &str) -> Result<Vec<SurahRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List staged divisions for a run.
    async fn list_stg_divisions(&self, _run_id: &str) -> Result<Vec<DivisionRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Set a canonical edition's lifecycle status (human-gated maintenance;
    /// identity and hashes stay trigger-guarded).
    async fn set_edition_status(&mut self, _id: &str, _status: &str) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Stamp editorial verification (`verified_by` / `verified_at` /
    /// `verification_method`) on a canonical edition (P1-T55; OD-02).
    ///
    /// Human-gated maintenance like [`QuranRepository::set_edition_status`]:
    /// only these three columns are updated, so the edition-identity trigger
    /// (`slug`, `version`, hashes) cannot fire and canonical text is untouched.
    /// The reviewer identity itself is an owner act this method records, never
    /// invents — empty reviewer names fail closed.
    async fn set_edition_verification(
        &mut self,
        _id: &str,
        _verified_by: &str,
        _verified_at: &str,
        _verification_method: &str,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Update an import run's state.
    async fn set_import_run_state(
        &mut self,
        _run_id: &str,
        _state: &str,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Delete an import run; staging rows cascade.
    async fn delete_import_run(&mut self, _run_id: &str) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Count staging rows whose run ended `Cancelled` or `Failed`.
    async fn count_staging_orphans(&self) -> Result<i64, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Stage one edition row.
    async fn insert_stg_edition(
        &mut self,
        _run_id: &str,
        _row: QuranEditionRow,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Stage one surah row.
    async fn insert_stg_surah(
        &mut self,
        _run_id: &str,
        _row: SurahRow,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Stage one ayah row.
    async fn insert_stg_ayah(&mut self, _run_id: &str, _row: AyahRow) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Stage one token row.
    async fn insert_stg_token(
        &mut self,
        _run_id: &str,
        _row: TokenRow,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Stage one separator row.
    async fn insert_stg_separator(
        &mut self,
        _run_id: &str,
        _row: SeparatorRow,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Stage one division row.
    async fn insert_stg_division(
        &mut self,
        _run_id: &str,
        _row: DivisionRow,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Count staged ayahs for a run.
    async fn count_stg_ayahs(&self, _run_id: &str) -> Result<i64, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Delete all staging rows for a run, keeping the run record.
    async fn clear_staging(&mut self, _run_id: &str) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List staged ayahs for a run ordered by `(surah, ayah)`.
    async fn list_stg_ayahs(&self, _run_id: &str) -> Result<Vec<AyahRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List staged tokens for one ayah ordered by position.
    async fn list_stg_tokens(
        &self,
        _run_id: &str,
        _edition_id: &str,
        _surah: i64,
        _ayah: i64,
    ) -> Result<Vec<TokenRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List staged separators for one ayah ordered by `after_position`.
    async fn list_stg_separators(
        &self,
        _run_id: &str,
        _edition_id: &str,
        _surah: i64,
        _ayah: i64,
    ) -> Result<Vec<SeparatorRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Activate a staged edition: move staging rows to canonical tables, flip
    /// the active pointer, bump `corpus_generation`, and consume the staging
    /// rows — atomically. Returns the new generation.
    async fn activate_edition(
        &mut self,
        _run_id: &str,
        _edition_id: &str,
        _activated_by: &str,
        _approval_id: &str,
        _activated_at: &str,
    ) -> Result<i64, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Roll back to an existing edition version: flip the pointer and bump the
    /// generation atomically. Returns the new generation.
    async fn rollback_edition(
        &mut self,
        _slug: &str,
        _version: &str,
        _activated_by: &str,
        _approval_id: &str,
        _activated_at: &str,
    ) -> Result<i64, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Fetch an edition by id.
    async fn get_edition(&self, _id: &str) -> Result<Option<QuranEditionRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Fetch an edition by slug and version.
    async fn get_edition_by_slug_version(
        &self,
        _slug: &str,
        _version: &str,
    ) -> Result<Option<QuranEditionRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List all editions ordered by slug and version.
    async fn list_editions(&self) -> Result<Vec<QuranEditionRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Fetch the active-edition pointer, if any.
    async fn get_active(&self) -> Result<Option<ActiveEditionRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Fetch one surah row.
    async fn get_surah(
        &self,
        _edition_id: &str,
        _number: i64,
    ) -> Result<Option<SurahRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List surahs ordered by number.
    async fn list_surahs(&self, _edition_id: &str) -> Result<Vec<SurahRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Fetch one ayah row.
    async fn get_ayah(
        &self,
        _edition_id: &str,
        _surah: i64,
        _ayah: i64,
    ) -> Result<Option<AyahRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Fetch one ayah by global index.
    async fn get_ayah_by_global(
        &self,
        _edition_id: &str,
        _global: i64,
    ) -> Result<Option<AyahRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List ayahs in a global-index range, ordered.
    async fn list_ayahs_range(
        &self,
        _edition_id: &str,
        _start_global: i64,
        _end_global: i64,
    ) -> Result<Vec<AyahRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List an ayah's tokens ordered by position.
    async fn get_tokens(
        &self,
        _edition_id: &str,
        _surah: i64,
        _ayah: i64,
    ) -> Result<Vec<TokenRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List an ayah's separators ordered by `after_position`.
    async fn get_separators(
        &self,
        _edition_id: &str,
        _surah: i64,
        _ayah: i64,
    ) -> Result<Vec<SeparatorRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List divisions of one kind ordered by number.
    async fn list_divisions(
        &self,
        _edition_id: &str,
        _kind: &str,
    ) -> Result<Vec<DivisionRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Count canonical ayahs for hash recomputation.
    async fn count_ayahs(&self, _edition_id: &str) -> Result<i64, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Count canonical tokens for hash recomputation.
    async fn count_tokens(&self, _edition_id: &str) -> Result<i64, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Persist a validation report.
    async fn insert_validation_report(
        &mut self,
        _row: ValidationReportRow,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Fetch a validation report by id.
    async fn get_validation_report(
        &self,
        _id: &str,
    ) -> Result<Option<ValidationReportRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Persist a difference report.
    async fn insert_difference_report(
        &mut self,
        _row: DifferenceReportRow,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Persist a resolved citation.
    async fn insert_citation(&mut self, _row: CitationRow) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Fetch a citation by id.
    async fn get_citation(&self, _id: &str) -> Result<Option<CitationRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List citations by canonical reference.
    async fn list_citations_by_ref(
        &self,
        _canonical_reference: &str,
    ) -> Result<Vec<CitationRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Insert a translation edition.
    async fn insert_translation_edition(
        &mut self,
        _row: TranslationEditionRow,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Insert a translation passage.
    async fn insert_translation_passage(
        &mut self,
        _row: TranslationPassageRow,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Fetch one translation passage.
    async fn get_translation_passage(
        &self,
        _translation_edition_id: &str,
        _surah: i64,
        _ayah: i64,
    ) -> Result<Option<TranslationPassageRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List translation editions ordered by slug and version.
    async fn list_translation_editions(&self) -> Result<Vec<TranslationEditionRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Insert a word gloss (P1-T38; the `word_glosses` PK rejects duplicates).
    async fn insert_word_gloss(&mut self, _row: WordGlossRow) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List an ayah's word glosses ordered by dataset, position, then language
    /// (deterministic serving order for the reader).
    async fn list_word_glosses(
        &self,
        _edition_id: &str,
        _surah: i64,
        _ayah: i64,
    ) -> Result<Vec<WordGlossRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    // ─── Phase 2 — normalization catalog (migration 0013, append-only) ──

    /// List seeded normalization rules ordered by rule id and version.
    async fn list_normalization_rules(&self) -> Result<Vec<NormalizationRuleRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List normalization profile versions ordered by profile id and version.
    async fn list_normalization_profiles(
        &self,
    ) -> Result<Vec<NormalizationProfileRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Fetch one normalization profile version.
    async fn get_normalization_profile(
        &self,
        _profile_id: &str,
        _version: &str,
    ) -> Result<Option<NormalizationProfileRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Register a new normalization profile version (new keys only; the
    /// append-only trigger rejects rewrites of seeded rows).
    async fn insert_normalization_profile(
        &mut self,
        _row: NormalizationProfileRow,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    // ─── Phase 2 — derived forms (migration 0014, Layer D) ────────────

    /// Bulk-insert derived token forms (called by `forms.rebuild`).
    async fn insert_token_forms(&mut self, _rows: Vec<TokenFormRow>) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Bulk-insert derived ayah forms (called by `forms.rebuild`).
    async fn insert_ayah_forms(&mut self, _rows: Vec<AyahFormRow>) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Bulk-insert skeleton rows: ayah skeletons plus 3-ayah windows.
    async fn insert_skeletons(&mut self, _rows: Vec<SkeletonRow>) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List derived token forms for one ayah in position order.
    async fn list_token_forms(
        &self,
        _edition_id: &str,
        _surah: i64,
        _ayah: i64,
    ) -> Result<Vec<TokenFormRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Count token-form rows matching an exact derived value (counting
    /// tools, P2-T95: exact SQL aggregation, never FTS frequencies).
    async fn count_tokens_matching_form(
        &self,
        _edition_id: &str,
        _column: FormColumn,
        _value: &str,
    ) -> Result<i64, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Per-surah counts for an exact derived value (distribution, P2-T96).
    async fn count_tokens_matching_form_by_surah(
        &self,
        _edition_id: &str,
        _column: FormColumn,
        _value: &str,
    ) -> Result<Vec<(i64, i64)>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Distinct derived values with exact counts (hapax/mining, P2-T101).
    async fn list_distinct_forms(
        &self,
        _edition_id: &str,
        _column: FormColumn,
    ) -> Result<Vec<(String, i64)>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// All token-form rows for one edition in canonical order (windows and
    /// collocation statistics, P2-T97/T98; forms are short derived strings).
    async fn list_all_token_forms(
        &self,
        _edition_id: &str,
    ) -> Result<Vec<TokenFormRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Fetch one derived ayah-form row.
    async fn get_ayah_form(
        &self,
        _edition_id: &str,
        _surah: i64,
        _ayah: i64,
    ) -> Result<Option<AyahFormRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List skeleton rows for one surah ordered by window start.
    async fn list_skeletons(
        &self,
        _edition_id: &str,
        _surah: i64,
    ) -> Result<Vec<SkeletonRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Count derived token-form rows for one edition (build verification).
    async fn count_token_forms(&self, _edition_id: &str) -> Result<i64, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Count derived ayah-form rows for one edition (doctor coverage, P2-T105).
    async fn count_ayah_forms(&self, _edition_id: &str) -> Result<i64, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Delete all derived forms for one edition (rebuilds only; canonical
    /// tables are never touched by this method).
    async fn delete_forms_for_edition(&mut self, _edition_id: &str) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    // ─── Phase 2 — index lifecycle (migration 0015) ───────────────────

    /// Fetch the serving pointer for an index id.
    async fn get_index_pointer(
        &self,
        _index_id: &str,
    ) -> Result<Option<IndexPointerRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Insert or replace the serving pointer (flip path only; called inside
    /// the same transaction as the build-run state change).
    async fn upsert_index_pointer(&mut self, _row: IndexPointerRow) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Record a new index build run.
    async fn insert_build_run(&mut self, _row: IndexBuildRunRow) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Update a build run's terminal state (active/failed/superseded).
    async fn set_build_run_state(
        &mut self,
        _id: &str,
        _state: &str,
        _doc_count: i64,
        _manifest_hash: &str,
        _error: Option<&str>,
        _finished_at: &str,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List build runs for an index ordered by generation.
    async fn list_build_runs(
        &self,
        _index_id: &str,
    ) -> Result<Vec<IndexBuildRunRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Highest build generation for an index id (0 when never built).
    async fn max_build_generation(&self, _index_id: &str) -> Result<i64, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Delete one build-run row (retention GC only, P2-T35).
    ///
    /// The generation directory must already be removed from disk; this only
    /// drops the tracking row. Never called for the active generation — the
    /// GC guards that before reaching storage.
    async fn delete_build_run(&mut self, _id: &str) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    // ─── Phase 2 — search result cache (migration 0016) ──────────────

    /// Fetch a cache entry by key (no generation check; the caller validates).
    async fn cache_get(&self, _key: &str) -> Result<Option<SearchCacheRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Insert or replace a cache entry.
    async fn cache_put(&mut self, _row: SearchCacheRow) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Refresh an entry's `last_hit_at` (LRU touch on cache hits).
    async fn cache_touch(&mut self, _key: &str, _at: &str) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Delete one cache entry (generation mismatches, invalidation).
    async fn cache_delete(&mut self, _key: &str) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Entry count and total bytes.
    async fn cache_stats(&self) -> Result<(i64, i64), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Evict least-recently-hit entries until at most `max_bytes` remain.
    /// Returns evicted entries. Single statement, no read-modify-write race.
    async fn cache_enforce_cap(&mut self, _max_bytes: i64) -> Result<u64, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Delete every entry from other generations (wholesale invalidation on
    /// a corpus-generation bump). Returns deleted entries.
    async fn cache_delete_stale(&mut self, _keep_generation: i64) -> Result<u64, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    // ─── Phase 2 — morphology lexicon + staging (migrations 0017/0018) ──

    /// Register a dataset (import path; state starts `staged`).
    async fn upsert_dataset(&mut self, _row: QuranDatasetRow) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Fetch a dataset by slug@version.
    async fn get_dataset(
        &self,
        _slug: &str,
        _version: &str,
    ) -> Result<Option<QuranDatasetRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List all datasets (newest first).
    async fn list_datasets(&self) -> Result<Vec<QuranDatasetRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// The active dataset, if any (activation flips exactly one row).
    async fn active_dataset(&self) -> Result<Option<QuranDatasetRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Set a dataset's state (activation path only, approval-gated above).
    async fn set_dataset_state(&mut self, _id: &str, _state: &str) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Open a staging batch (import path).
    async fn insert_staging_batch(&mut self, _row: StagingBatchRow) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Advance a staging batch's state/checkpoint (cancel-safe resume).
    async fn update_staging_batch(
        &mut self,
        _id: &str,
        _state: &str,
        _checkpoint: &str,
        _finished_at: Option<&str>,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Fetch a staging batch.
    async fn get_staging_batch(&self, _id: &str) -> Result<Option<StagingBatchRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Replace a batch's staging rows (idempotent rewrite for resume).
    async fn replace_staging_rows(
        &mut self,
        _batch_id: &str,
        _rows: Vec<StagedMorphRow>,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List a batch's staging rows.
    async fn list_staging_rows(
        &self,
        _batch_id: &str,
    ) -> Result<Vec<StagedMorphRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Replace a batch's alignment rows (idempotent rewrite for resume).
    async fn replace_alignment_rows(
        &mut self,
        _batch_id: &str,
        _rows: Vec<AlignmentRow>,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List a batch's alignment rows.
    async fn list_alignment(&self, _batch_id: &str) -> Result<Vec<AlignmentRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Replace a batch's validation findings (idempotent rewrite).
    async fn replace_findings(
        &mut self,
        _batch_id: &str,
        _rows: Vec<MorphologyFindingRow>,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List a batch's findings.
    async fn list_findings(
        &self,
        _batch_id: &str,
    ) -> Result<Vec<MorphologyFindingRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Promote staged roots into the lexicon (activation path, one tx).
    async fn insert_roots(&mut self, _rows: Vec<LexiconRootRow>) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Promote staged lemmas into the lexicon (activation path, one tx).
    async fn insert_lemmas(&mut self, _rows: Vec<LexiconLemmaRow>) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Promote staged analyses into the lexicon (activation path, one tx).
    async fn insert_analyses(&mut self, _rows: Vec<TokenAnalysisRow>) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Promote staged morphemes into the lexicon (activation path, one tx).
    async fn insert_morphemes(&mut self, _rows: Vec<MorphemeRow>) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// All analyses for one token across datasets (multi-analysis reads).
    async fn analyses_for_token(
        &self,
        _dataset_id: Option<&str>,
        _edition_id: &str,
        _surah: i64,
        _ayah: i64,
        _position: i64,
    ) -> Result<Vec<TokenAnalysisRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// All analyses for one dataset in canonical location order.
    async fn list_analyses(
        &self,
        _dataset_id: &str,
    ) -> Result<Vec<TokenAnalysisRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Analyses by normalized root within a dataset (root search).
    async fn analyses_for_root(
        &self,
        _dataset_id: &str,
        _root_normalized: &str,
    ) -> Result<Vec<TokenAnalysisRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Analyses by lemma within a dataset (lemma search).
    async fn analyses_for_lemma(
        &self,
        _dataset_id: &str,
        _lemma: &str,
    ) -> Result<Vec<TokenAnalysisRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Roots of a dataset in lemma order (browse).
    async fn list_roots(&self, _dataset_id: &str) -> Result<Vec<LexiconRootRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Lemmas of a dataset in lemma order (browse).
    async fn list_lemmas(&self, _dataset_id: &str) -> Result<Vec<LexiconLemmaRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Record family relations (builders + review promotions).
    async fn insert_family_relations(
        &mut self,
        _rows: Vec<FamilyRelationRow>,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Relations touching one member (either side).
    async fn family_relations_for(
        &self,
        _kind: &str,
        _id: &str,
    ) -> Result<Vec<FamilyRelationRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Enqueue a review item (suggestion flow).
    async fn insert_review_item(&mut self, _row: MorphReviewItemRow) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// List review items by status.
    async fn list_review_items(
        &self,
        _status: &str,
    ) -> Result<Vec<MorphReviewItemRow>, StorageError> {
        Err(StorageError::StorageUnavailable)
    }

    /// Decide a review item (approval-gated above; reviewer recorded).
    async fn decide_review_item(
        &mut self,
        _id: &str,
        _reviewer: &str,
        _decision: &str,
        _decided_at: &str,
    ) -> Result<(), StorageError> {
        Err(StorageError::StorageUnavailable)
    }
}

/// One cached search response (migration `0016`, derived data).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchCacheRow {
    /// Cache key (binds tool, query, profile, filters, paging, generation).
    pub key: String,
    /// Corpus generation served.
    pub generation: i64,
    /// Serialized search-output JSON.
    ///
    /// The payload type lives in `application` (this crate must not depend
    /// on it); the service validates the shape on read and treats garbage
    /// as a miss.
    pub payload_json: String,
    /// Payload size in bytes (eviction accounting).
    pub bytes: i64,
    /// Insert timestamp (RFC 3339).
    pub created_at: String,
    /// Last-hit timestamp (RFC 3339, LRU order).
    pub last_hit_at: String,
}

// ─── Phase 2 — morphology lexicon + staging rows (migrations 0017/0018) ──

/// One morphology dataset (migration `0017`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuranDatasetRow {
    /// Row id (`slug@version` content key).
    pub id: String,
    /// Dataset slug.
    pub slug: String,
    /// Dataset version.
    pub version: String,
    /// Human title.
    pub title: String,
    /// License status + JSON (redistribution gate).
    pub license_status: String,
    /// License detail JSON.
    pub license_json: String,
    /// Required attribution string.
    pub attribution: String,
    /// Root convention identifier.
    pub root_convention: String,
    /// Tagset version.
    pub tagset_version: String,
    /// `staged` | `active` | `superseded`.
    pub state: String,
    /// Insert timestamp (RFC 3339).
    pub created_at: String,
}

/// One import staging batch (migration `0018`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagingBatchRow {
    /// Batch id.
    pub id: String,
    /// Dataset slug/version under import.
    pub dataset_slug: String,
    /// Dataset version under import.
    pub dataset_version: String,
    /// Adapter name (`json` | `csv`).
    pub adapter: String,
    /// Source manifest hash.
    pub source_manifest_hash: String,
    /// Batch state (12-checkpoint lifecycle).
    pub state: String,
    /// Resume checkpoint name.
    pub checkpoint: String,
    /// Start timestamp (RFC 3339).
    pub created_at: String,
    /// Finish timestamp (RFC 3339, if finished).
    pub finished_at: Option<String>,
}

/// One staged token analysis (migration `0018`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedMorphRow {
    /// Row id.
    pub id: String,
    /// Owning batch.
    pub batch_id: String,
    /// `surah:ayah:position` reference.
    pub reference: String,
    /// Surah number.
    pub surah: i64,
    /// Ayah number.
    pub ayah: i64,
    /// Token position.
    pub token_position: i64,
    /// Intermediate-format analysis JSON.
    pub payload_json: String,
    /// Native tags JSON.
    pub native_tags_json: String,
    /// Insert timestamp (RFC 3339).
    pub created_at: String,
}

/// One alignment decision (migration `0018`; never modifies tokens).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlignmentRow {
    /// Row id.
    pub id: String,
    /// Owning batch.
    pub batch_id: String,
    /// `surah:ayah:position:surface` direct key.
    pub direct_key: String,
    /// Canonical edition id aligned to.
    pub edition_id: String,
    /// Surah number.
    pub surah: i64,
    /// Ayah number.
    pub ayah: i64,
    /// Token position.
    pub token_position: i64,
    /// `direct_key` | `table_mapped` | `unmatched`.
    pub alignment_kind: String,
    /// Alignment-table entry hash (auditable mapping).
    pub alignment_hash: String,
    /// Insert timestamp (RFC 3339).
    pub created_at: String,
}

/// One MV validation finding (migration `0018`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MorphologyFindingRow {
    /// Row id.
    pub id: String,
    /// Owning batch.
    pub batch_id: String,
    /// MV rule id (`MV-001`…`MV-018`).
    pub rule_id: String,
    /// `fatal` | `error` | `warn`.
    pub severity: String,
    /// `surah:ayah:position` reference (empty for batch-level).
    pub reference: String,
    /// Human detail.
    pub detail: String,
    /// Insert timestamp (RFC 3339).
    pub created_at: String,
}

/// Shared Layer B/D provenance columns for lexicon rows.
#[derive(Debug, Clone, PartialEq)]
pub struct LexiconProvenance {
    /// `B` (dataset-attested) or `D` (computational).
    pub layer: String,
    /// Algorithm name (D only).
    pub algorithm: Option<String>,
    /// Algorithm version (D only).
    pub algorithm_version: Option<String>,
    /// Confidence in [0,1] (D only).
    pub confidence: Option<f64>,
    /// Reviewer id (human_verified only).
    pub reviewer: Option<String>,
    /// `imported` | `human_verified`.
    pub status: String,
}

/// One lexicon root (migration `0017`; dataset identity preserved).
#[derive(Debug, Clone, PartialEq)]
pub struct LexiconRootRow {
    /// Row id.
    pub id: String,
    /// Owning dataset.
    pub dataset_id: String,
    /// Native root spelling.
    pub root: String,
    /// Normalized root (convention-pinned).
    pub root_normalized: String,
    /// Provenance.
    pub provenance: LexiconProvenance,
    /// Corpus generation at import.
    pub corpus_generation: i64,
    /// Insert timestamp (RFC 3339).
    pub created_at: String,
}

/// One lexicon lemma (migration `0017`).
#[derive(Debug, Clone, PartialEq)]
pub struct LexiconLemmaRow {
    /// Row id.
    pub id: String,
    /// Owning dataset.
    pub dataset_id: String,
    /// Lemma surface.
    pub lemma: String,
    /// Owning root row id (nullable).
    pub root_id: Option<String>,
    /// Unified POS tag.
    pub pos_unified: String,
    /// Verbatim native POS tag.
    pub pos_native: String,
    /// Provenance.
    pub provenance: LexiconProvenance,
    /// Corpus generation at import.
    pub corpus_generation: i64,
    /// Insert timestamp (RFC 3339).
    pub created_at: String,
}

/// One token analysis (migration `0017`; no winner column by design).
#[derive(Debug, Clone, PartialEq)]
pub struct TokenAnalysisRow {
    /// Row id.
    pub id: String,
    /// Owning dataset.
    pub dataset_id: String,
    /// Aligned canonical edition.
    pub edition_id: String,
    /// Surah number.
    pub surah: i64,
    /// Ayah number.
    pub ayah: i64,
    /// Token position.
    pub token_position: i64,
    /// Nth analysis of this token (multi-analysis coexistence).
    pub analysis_index: i64,
    /// Canonical surface.
    pub surface: String,
    /// Lemma row id (nullable).
    pub lemma_id: Option<String>,
    /// Root row id (nullable).
    pub root_id: Option<String>,
    /// Stem surface.
    pub stem: String,
    /// Unified POS tag.
    pub pos_unified: String,
    /// Verbatim native POS tag.
    pub pos_native: String,
    /// Feature JSON.
    pub features_json: String,
    /// Morpheme segment JSON.
    pub segments_json: String,
    /// Provenance.
    pub provenance: LexiconProvenance,
    /// Corpus generation at import.
    pub corpus_generation: i64,
    /// Insert timestamp (RFC 3339).
    pub created_at: String,
}

/// One morpheme (migration `0017`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MorphemeRow {
    /// Row id.
    pub id: String,
    /// Owning analysis.
    pub analysis_id: String,
    /// `prefix` | `stem` | `suffix`.
    pub kind: String,
    /// Morpheme surface.
    pub surface: String,
    /// Feature JSON.
    pub features_json: String,
    /// Insert timestamp (RFC 3339).
    pub created_at: String,
}

/// One word-family relation (migration `0017`; explanation mandatory).
#[derive(Debug, Clone, PartialEq)]
pub struct FamilyRelationRow {
    /// Row id.
    pub id: String,
    /// Relation taxonomy value.
    pub relation: String,
    /// Source member kind (`root` | `lemma` | `token`).
    pub from_kind: String,
    /// Source member id.
    pub from_id: String,
    /// Target member kind.
    pub to_kind: String,
    /// Target member id.
    pub to_id: String,
    /// Mandatory human-readable explanation.
    pub explanation: String,
    /// Owning dataset (nullable for computational suggestions).
    pub dataset_id: Option<String>,
    /// Provenance.
    pub provenance: LexiconProvenance,
    /// `proposed` | `scholar_verified`.
    pub status: String,
    /// Evidence JSON.
    pub evidence_json: String,
    /// Corpus generation.
    pub corpus_generation: i64,
    /// Insert timestamp (RFC 3339).
    pub created_at: String,
}

/// One morphology review-queue item (migration `0017`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MorphReviewItemRow {
    /// Row id.
    pub id: String,
    /// `root_unification` | `family_relation` | `analysis_correction`.
    pub kind: String,
    /// Subject JSON.
    pub subject_json: String,
    /// Evidence JSON.
    pub evidence_json: String,
    /// `pending` | `approved` | `rejected`.
    pub status: String,
    /// Reviewer id (decided items).
    pub reviewer: Option<String>,
    /// Decision timestamp (RFC 3339, decided items).
    pub decided_at: Option<String>,
    /// Insert timestamp (RFC 3339).
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn surah(edition: &str, number: i64) -> SurahRow {
        SurahRow {
            edition_id: edition.into(),
            number,
            name_arabic: format!("s{number}"),
            name_transliteration: None,
            name_translations_json: "{}".into(),
            ayah_count: 0,
            revelation_place: None,
            revelation_order: None,
            basmala: String::new(),
            ruku_count: None,
            metadata_provenance_id: None,
        }
    }

    fn ayah(edition: &str, surah_n: i64, ayah_n: i64) -> AyahRow {
        AyahRow {
            edition_id: edition.into(),
            surah: surah_n,
            ayah: ayah_n,
            text: format!("{surah_n}:{ayah_n}"),
            text_hash: "h".into(),
            char_count: 1,
            token_count: 1,
            global_ayah_index: surah_n * 1000 + ayah_n,
            juz: None,
            hizb: None,
            rub: None,
            manzil: None,
            ruku: None,
            page: None,
            sajdah: None,
            provenance_id: "p".into(),
        }
    }

    fn token(edition: &str, surah_n: i64, ayah_n: i64, pos: i64) -> TokenRow {
        TokenRow {
            edition_id: edition.into(),
            surah: surah_n,
            ayah: ayah_n,
            position: pos,
            surface: format!("t{pos}"),
            surface_hash: "h".into(),
            char_start: pos,
            char_end: pos + 1,
            byte_start: pos,
            byte_end: pos + 1,
            is_pause_mark: false,
            global_token_index: pos,
        }
    }

    fn separator(edition: &str, surah_n: i64, ayah_n: i64, after: i64) -> SeparatorRow {
        SeparatorRow {
            edition_id: edition.into(),
            surah: surah_n,
            ayah: ayah_n,
            after_position: after,
            separator: " ".into(),
        }
    }

    /// Minimal run-scoped staging fake: staging mirrors keyed by run id,
    /// deterministic ordered reads, single gated activation path.
    #[derive(Default)]
    struct FakeQuran {
        calls: Vec<String>,
        stg_surahs: HashMap<String, Vec<SurahRow>>,
        stg_ayahs: HashMap<String, Vec<AyahRow>>,
        stg_tokens: HashMap<(String, String, i64, i64), Vec<TokenRow>>,
        stg_seps: HashMap<(String, String, i64, i64), Vec<SeparatorRow>>,
        validations: Vec<String>,
        activated: Vec<(String, String)>,
    }

    #[async_trait]
    impl QuranRepository for FakeQuran {
        async fn insert_stg_surah(
            &mut self,
            run_id: &str,
            row: SurahRow,
        ) -> Result<(), StorageError> {
            self.calls.push(format!("stage-surah:{run_id}:{}", row.number));
            self.stg_surahs.entry(run_id.into()).or_default().push(row);
            Ok(())
        }
        async fn insert_stg_ayah(
            &mut self,
            run_id: &str,
            row: AyahRow,
        ) -> Result<(), StorageError> {
            self.calls.push(format!("stage-ayah:{run_id}:{}:{}", row.surah, row.ayah));
            self.stg_ayahs.entry(run_id.into()).or_default().push(row);
            Ok(())
        }
        async fn insert_stg_token(
            &mut self,
            run_id: &str,
            row: TokenRow,
        ) -> Result<(), StorageError> {
            self.calls.push(format!("stage-token:{run_id}:{}", row.position));
            self.stg_tokens
                .entry((run_id.into(), row.edition_id.clone(), row.surah, row.ayah))
                .or_default()
                .push(row);
            Ok(())
        }
        async fn insert_stg_separator(
            &mut self,
            run_id: &str,
            row: SeparatorRow,
        ) -> Result<(), StorageError> {
            self.calls.push(format!("stage-sep:{run_id}:{}", row.after_position));
            self.stg_seps
                .entry((run_id.into(), row.edition_id.clone(), row.surah, row.ayah))
                .or_default()
                .push(row);
            Ok(())
        }
        async fn list_stg_surahs(&self, run_id: &str) -> Result<Vec<SurahRow>, StorageError> {
            let mut v = self.stg_surahs.get(run_id).cloned().unwrap_or_default();
            v.sort_by_key(|r| r.number);
            Ok(v)
        }
        async fn list_stg_ayahs(&self, run_id: &str) -> Result<Vec<AyahRow>, StorageError> {
            let mut v = self.stg_ayahs.get(run_id).cloned().unwrap_or_default();
            v.sort_by_key(|r| (r.surah, r.ayah));
            Ok(v)
        }
        async fn list_stg_tokens(
            &self,
            run_id: &str,
            edition_id: &str,
            surah: i64,
            ayah_n: i64,
        ) -> Result<Vec<TokenRow>, StorageError> {
            let mut v = self
                .stg_tokens
                .get(&(run_id.into(), edition_id.into(), surah, ayah_n))
                .cloned()
                .unwrap_or_default();
            v.sort_by_key(|r| r.position);
            Ok(v)
        }
        async fn list_stg_separators(
            &self,
            run_id: &str,
            edition_id: &str,
            surah: i64,
            ayah_n: i64,
        ) -> Result<Vec<SeparatorRow>, StorageError> {
            let mut v = self
                .stg_seps
                .get(&(run_id.into(), edition_id.into(), surah, ayah_n))
                .cloned()
                .unwrap_or_default();
            v.sort_by_key(|r| r.after_position);
            Ok(v)
        }
        async fn count_stg_ayahs(&self, run_id: &str) -> Result<i64, StorageError> {
            Ok(self.stg_ayahs.get(run_id).map(|v| v.len() as i64).unwrap_or(0))
        }
        async fn insert_validation_report(
            &mut self,
            row: ValidationReportRow,
        ) -> Result<(), StorageError> {
            self.calls.push(format!("validate:{}", row.id));
            self.validations.push(row.id);
            Ok(())
        }
        async fn activate_edition(
            &mut self,
            run_id: &str,
            edition_id: &str,
            _activated_by: &str,
            _approval_id: &str,
            _activated_at: &str,
        ) -> Result<i64, StorageError> {
            // Gated path requires staged content; missing run → NotFound,
            // pointer and generation unchanged (caller observes NotFound).
            let has = self.stg_ayahs.get(run_id).map(|v| !v.is_empty()).unwrap_or(false);
            if !has {
                return Err(StorageError::NotFound { urn: run_id.into() });
            }
            self.calls.push(format!("activate:{run_id}:{edition_id}"));
            self.activated.push((run_id.into(), edition_id.into()));
            Ok(1)
        }
    }

    fn validation(id: &str) -> ValidationReportRow {
        ValidationReportRow {
            id: id.into(),
            subject_urn: "urn:run".into(),
            validator: "qv".into(),
            validator_version: "1".into(),
            outcome: "pass".into(),
            fatal_count: 0,
            error_count: 0,
            warning_count: 0,
            findings_json: "[]".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    // T011 (US2, FR-006): staged reads are deterministic and run-scoped.
    #[tokio::test]
    async fn staged_reads_are_deterministic_and_run_scoped() {
        let mut q = FakeQuran::default();
        // Insert out of order into run-A.
        q.insert_stg_surah("run-a", surah("ed", 2)).await.unwrap();
        q.insert_stg_surah("run-a", surah("ed", 1)).await.unwrap();
        q.insert_stg_ayah("run-a", ayah("ed", 2, 1)).await.unwrap();
        q.insert_stg_ayah("run-a", ayah("ed", 1, 2)).await.unwrap();
        q.insert_stg_ayah("run-a", ayah("ed", 1, 1)).await.unwrap();
        q.insert_stg_token("run-a", token("ed", 1, 1, 3)).await.unwrap();
        q.insert_stg_token("run-a", token("ed", 1, 1, 1)).await.unwrap();
        q.insert_stg_token("run-a", token("ed", 1, 1, 2)).await.unwrap();
        q.insert_stg_separator("run-a", separator("ed", 1, 1, 2)).await.unwrap();
        q.insert_stg_separator("run-a", separator("ed", 1, 1, 1)).await.unwrap();
        // Run-B stays empty (run scoping).
        q.insert_stg_surah("run-b", surah("ed", 9)).await.unwrap();

        let surahs = q.list_stg_surahs("run-a").await.unwrap();
        assert_eq!(surahs.iter().map(|r| r.number).collect::<Vec<_>>(), vec![1, 2]);
        let ayahs = q.list_stg_ayahs("run-a").await.unwrap();
        assert_eq!(
            ayahs.iter().map(|r| (r.surah, r.ayah)).collect::<Vec<_>>(),
            vec![(1, 1), (1, 2), (2, 1)]
        );
        let toks = q.list_stg_tokens("run-a", "ed", 1, 1).await.unwrap();
        assert_eq!(toks.iter().map(|r| r.position).collect::<Vec<_>>(), vec![1, 2, 3]);
        let seps = q.list_stg_separators("run-a", "ed", 1, 1).await.unwrap();
        assert_eq!(seps.iter().map(|r| r.after_position).collect::<Vec<_>>(), vec![1, 2]);

        // Run scoping: run-B sees only its own rows.
        assert_eq!(q.list_stg_surahs("run-b").await.unwrap().len(), 1);
        assert_eq!(q.list_stg_ayahs("run-b").await.unwrap().len(), 0);
        assert_eq!(q.count_stg_ayahs("run-a").await.unwrap(), 3);
        assert_eq!(q.count_stg_ayahs("run-b").await.unwrap(), 0);
    }

    // T011 caller order: stage → validate → activate.
    #[tokio::test]
    async fn caller_order_stage_validate_activate() {
        let mut q = FakeQuran::default();
        q.insert_stg_ayah("run-1", ayah("ed-1", 1, 1)).await.unwrap();
        q.insert_validation_report(validation("v-1")).await.unwrap();
        let generation = q
            .activate_edition("run-1", "ed-1", "op", "appr-1", "2026-01-01T00:00:00Z")
            .await
            .unwrap();
        assert_eq!(generation, 1);
        let stage_pos = q.calls.iter().position(|c| c.starts_with("stage-ayah")).unwrap();
        let validate_pos = q.calls.iter().position(|c| c.starts_with("validate:")).unwrap();
        let activate_pos = q.calls.iter().position(|c| c.starts_with("activate:")).unwrap();
        assert!(stage_pos < validate_pos && validate_pos < activate_pos);

        // Activating a missing run fails closed with NotFound.
        let err = q
            .activate_edition("missing", "ed-x", "op", "appr", "2026-01-01T00:00:00Z")
            .await
            .unwrap_err();
        assert!(matches!(err, StorageError::NotFound { .. }));
        assert_eq!(err.code(), "QAI-DB-0002");
    }

    // T012 (US2, FR-005/SC-003): single gated canonical-write path.
    // Audit pinned to this source file: the only canonical-write methods are
    // `activate_edition`, `rollback_edition`, human-gated `set_edition_status`,
    // and human-gated `set_edition_verification` (P1-T55: stamps only the
    // `verified_*` metadata columns, which sit outside the trigger-guarded
    // identity/hash columns, so canonical text can never change through it).
    // Staging (`insert_stg_*`), validation/difference
    // reports, citations, translations, glosses, normalization catalog,
    // Layer-D forms, index pointers/build runs, and search cache are separate
    // attributed/derived datasets that never overwrite canonical Arabic.
    #[test]
    fn canonical_write_surface_is_gated_to_three_mutators() {
        let src = include_str!("quran.rs");
        // Restrict the audit to the contract surface (everything before the
        // test module) so this test's own forbidden-name literals do not
        // self-match via `include_str!`.
        let trait_src = src.split("#[cfg(test)]").next().unwrap_or(src);
        for allowed in [
            "activate_edition",
            "rollback_edition",
            "set_edition_status",
            "set_edition_verification",
        ] {
            assert!(trait_src.contains(allowed), "gated mutator {allowed} must exist");
        }
        // No row-level canonical insert/update/delete may exist.
        // NOTE: names carry a trailing `(` so derived helpers like
        // `insert_ayah_forms` / `insert_token_forms` (Layer-D, allowed) do not
        // false-positive on the `insert_ayah` / `insert_token` prefixes.
        for forbidden in [
            "fn insert_edition(",
            "fn insert_surah(",
            "fn insert_ayah(",
            "fn insert_token(",
            "fn insert_separator(",
            "fn insert_division(",
            "fn update_edition(",
            "fn update_ayah(",
            "fn delete_edition(",
            "fn delete_ayah(",
            "fn write_canonical(",
            "fn insert_canonical(",
        ] {
            assert!(
                !trait_src.contains(forbidden),
                "forbidden canonical-write method {forbidden} must not exist"
            );
        }
    }

    #[tokio::test]
    async fn gated_mutators_fail_closed_without_backend() {
        struct StubQuran;
        #[async_trait]
        impl QuranRepository for StubQuran {}
        let mut q = StubQuran;
        for err in [
            q.activate_edition("r", "e", "op", "a", "t").await.unwrap_err(),
            q.rollback_edition("s", "v", "op", "a", "t").await.unwrap_err(),
            q.set_edition_status("e", "Active").await.unwrap_err(),
            q.set_edition_verification("e", "reviewer", "t", "method").await.unwrap_err(),
        ] {
            assert_eq!(err, StorageError::StorageUnavailable);
            assert_eq!(err.code(), "QAI-DB-0009");
        }
    }
}
