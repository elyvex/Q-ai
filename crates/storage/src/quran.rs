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
//! identity. Raw SQL is still blocked by the insert-only triggers.

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
