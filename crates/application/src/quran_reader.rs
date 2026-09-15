//! Deterministic, LLM-free Quran lookup with caching (D1.6, P1-T32–T35).
//!
//! # application::quran_reader
//!
//! [`QuranReader`] serves exact canonical text by stable reference, with
//! context by canonical structure (never arbitrary chunking). Reads go through
//! the unit of work in read-only fashion; a dedicated read-pool path is future
//! work (the single-writer pool serializes concurrent readers in v1).
//!
//! Caching (ADR-0113): an `lru` cache keyed by
//! `(edition_id, version, corpus_generation, ref, options)` — wholesale
//! invalidation on generation change, so no stale text is ever served after an
//! activation. Correctness over hit rate.

use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use domain::{Language, SemVer};
use lru::LruCache;
use quran_core::{
    AttributedGloss, AttributedTranslation, AyahLocation, AyahNumber, AyahOptions, AyahView,
    BasmalaPolicy, ContextBoundary, ContextSpec, ContextView, EditionRef, EditionSelector,
    EditionStatus, NumberingScheme, QuotationParts, QuranEdition, QuranQuotation, QuranRef,
    ResolvedRef, RevelationPlace, Script, Surah, SurahNumber, Token, TokenPosition, UnicodeForm,
};
use storage::Database as _;
use storage::error::StorageError;
use storage::quran::{QuranEditionRow, SurahRow, TokenRow};
use storage_sqlite::SqliteDatabase;

/// Reader errors (`QAI-QUR-0306…0310`; reference errors keep their `01xx` codes).
#[derive(Debug, Clone, thiserror::Error)]
pub enum ReaderError {
    /// The reference string failed to parse (code delegated to the grammar error).
    #[error("invalid reference: {0}")]
    InvalidReference(#[from] quran_core::QuranError),
    /// No such edition (or no active edition for `Active`/bare-slug selectors).
    #[error("edition not found: {0}")]
    EditionNotFound(String),
    /// The location is outside the edition.
    #[error("ayah not found: {0}")]
    AyahNotFound(String),
    /// A requested translation is missing, misaligned, or unattributed.
    #[error("translation not found: {0}")]
    TranslationNotFound(String),
    /// No such division.
    #[error("division not found: {0}")]
    DivisionNotFound(String),
    /// Storage failure.
    #[error("storage failed: {0}")]
    Storage(#[from] StorageError),
}

impl storage::error::Diagnostic for ReaderError {
    fn code(&self) -> storage::error::DiagnosticCode {
        match self {
            Self::InvalidReference(inner) => {
                let rendered = quran_corpus::error::QuranDiagnostic::code(inner).to_string();
                let number =
                    rendered.rsplit('-').next().and_then(|digits| digits.parse().ok()).unwrap_or(0);
                storage::error::DiagnosticCode::new("QAI-QUR", number)
            }
            Self::EditionNotFound(_) => storage::error::DiagnosticCode::new("QAI-QUR", 306),
            Self::AyahNotFound(_) => storage::error::DiagnosticCode::new("QAI-QUR", 307),
            Self::TranslationNotFound(_) => storage::error::DiagnosticCode::new("QAI-QUR", 308),
            Self::DivisionNotFound(_) => storage::error::DiagnosticCode::new("QAI-QUR", 309),
            Self::Storage(_) => storage::error::DiagnosticCode::new("QAI-QUR", 310),
        }
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn remedy(&self) -> Option<String> {
        Some(
            match self {
                Self::InvalidReference(_) => "Rewrite the reference (e.g. `quran:2:255`).",
                Self::EditionNotFound(_) => "Import and activate the edition first.",
                Self::AyahNotFound(_) => "Check the surah and ayah numbers for this edition.",
                Self::TranslationNotFound(_) => "Import an aligned, attributed translation first.",
                Self::DivisionNotFound(_) => "Check the division kind and number.",
                Self::Storage(_) => "Check the database and retry.",
            }
            .to_string(),
        )
    }

    fn next_command(&self) -> Option<String> {
        Some("qai quran resolve --help".to_string())
    }
}

fn storage_broken(detail: String) -> ReaderError {
    ReaderError::Storage(StorageError::ConstraintViolation { message: detail })
}

fn parse_tagged(raw: &str) -> Result<domain::ContentHash, ReaderError> {
    let (algorithm, hex) =
        raw.split_once(':').ok_or_else(|| storage_broken(format!("bad hash `{raw}`")))?;
    let algorithm = match algorithm {
        "sha256" => domain::HashAlgorithm::Sha256,
        "blake3" => domain::HashAlgorithm::Blake3,
        _ => return Err(storage_broken(format!("unknown hash algorithm in `{raw}`"))),
    };
    domain::ContentHash::try_new(algorithm, hex.to_string())
        .map_err(|err| storage_broken(format!("bad hash `{raw}`: {err}")))
}

fn parse_edition_id(raw: &str) -> Result<domain::EditionId, ReaderError> {
    raw.parse::<domain::EditionId>()
        .map_err(|_| storage_broken(format!("edition id `{raw}` is not a UUID")))
}

fn parse_provenance_id(raw: &str) -> Result<domain::ProvenanceId, ReaderError> {
    raw.parse::<domain::ProvenanceId>()
        .map_err(|_| storage_broken(format!("provenance id `{raw}` is not a UUID")))
}

fn parse_script(raw: &str) -> Script {
    match raw {
        "uthmani" => Script::Uthmani,
        "imlaei_simple" => Script::ImlaeiSimple,
        other => match other.strip_prefix("other:") {
            Some(name) => Script::Other(name.to_string()),
            None => Script::Other(other.to_string()),
        },
    }
}

fn parse_numbering(raw: &str) -> NumberingScheme {
    match raw {
        "hafs" => NumberingScheme::Hafs,
        "kufi" => NumberingScheme::Kufi,
        other => NumberingScheme::Custom(other.to_string()),
    }
}

fn parse_basmala(raw: &str) -> Result<BasmalaPolicy, ReaderError> {
    match raw {
        "counted_as_first_ayah" => Ok(BasmalaPolicy::CountedAsFirstAyah),
        "unnumbered_header" => Ok(BasmalaPolicy::UnnumberedHeader),
        "absent" => Ok(BasmalaPolicy::Absent),
        "per_surah" => Ok(BasmalaPolicy::PerSurah),
        _ => Err(storage_broken(format!("unknown basmala policy `{raw}`"))),
    }
}

fn parse_unicode(raw: &str) -> Result<UnicodeForm, ReaderError> {
    match raw {
        "nfc" => Ok(UnicodeForm::Nfc),
        "nfd" => Ok(UnicodeForm::Nfd),
        "nfkc" => Ok(UnicodeForm::Nfkc),
        "nfkd" => Ok(UnicodeForm::Nfkd),
        _ => Err(storage_broken(format!("unknown unicode form `{raw}`"))),
    }
}

fn parse_status(raw: &str) -> Result<EditionStatus, ReaderError> {
    match raw {
        "Staged" => Ok(EditionStatus::Staged),
        "Approved" => Ok(EditionStatus::Approved),
        "Active" => Ok(EditionStatus::Active),
        "Deprecated" => Ok(EditionStatus::Deprecated),
        "Quarantined" => Ok(EditionStatus::Quarantined),
        _ => Err(storage_broken(format!("unknown edition status `{raw}`"))),
    }
}

fn parse_place(raw: &str) -> Result<RevelationPlace, ReaderError> {
    match raw {
        "makki" => Ok(RevelationPlace::Makki),
        "madani" => Ok(RevelationPlace::Madani),
        _ => Err(storage_broken(format!("unknown revelation place `{raw}`"))),
    }
}

fn map_edition(row: &QuranEditionRow) -> Result<QuranEdition, ReaderError> {
    Ok(QuranEdition {
        id: parse_edition_id(&row.id)?,
        slug: row.slug.clone(),
        name: row.name.clone(),
        script: parse_script(&row.script),
        riwayah: row.riwayah.clone(),
        qiraah: row.qiraah.clone(),
        publisher: row.publisher.clone(),
        source_url: row.source_url.clone(),
        license: serde_json::from_str(&row.license_json)
            .map_err(|err| storage_broken(format!("bad license record: {err}")))?,
        language: row
            .language
            .parse::<Language>()
            .map_err(|_| storage_broken(format!("bad language `{}`", row.language)))?,
        verse_numbering_scheme: parse_numbering(&row.verse_numbering_scheme),
        basmala_policy: parse_basmala(&row.basmala_policy)?,
        unicode_normalization: parse_unicode(&row.unicode_normalization)?,
        text_hash: parse_tagged(&row.text_hash)?,
        structure_hash: parse_tagged(&row.structure_hash)?,
        token_order_hash: parse_tagged(&row.token_order_hash)?,
        manifest_hash: parse_tagged(&row.manifest_hash)?,
        version: row
            .version
            .parse::<SemVer>()
            .map_err(|_| storage_broken(format!("bad version `{}`", row.version)))?,
        source_version_id: row
            .source_version_id
            .parse::<domain::SourceVersionId>()
            .map_err(|_| storage_broken("bad source version id".to_string()))?,
        imported_at: row
            .imported_at
            .parse::<domain::Timestamp>()
            .map_err(|_| storage_broken("bad imported_at".to_string()))?,
        verified_at: row
            .verified_at
            .as_deref()
            .map(|raw| {
                raw.parse::<domain::Timestamp>()
                    .map_err(|_| storage_broken("bad verified_at".to_string()))
            })
            .transpose()?,
        verified_by: row.verified_by.clone(),
        verification_method: row.verification_method.clone(),
        status: parse_status(&row.status)?,
        statistics: serde_json::from_str(&row.statistics_json)
            .map_err(|err| storage_broken(format!("bad statistics: {err}")))?,
    })
}

fn map_surah(row: &SurahRow) -> Result<Surah, ReaderError> {
    let names: BTreeMap<Language, String> = serde_json::from_str(&row.name_translations_json)
        .map_err(|err| storage_broken(format!("bad name translations: {err}")))?;
    Ok(Surah {
        edition_id: parse_edition_id(&row.edition_id)?,
        number: SurahNumber::new(row.number as u16)
            .map_err(|_| storage_broken(format!("bad surah number {}", row.number)))?,
        name_arabic: row.name_arabic.clone(),
        name_transliteration: row.name_transliteration.clone(),
        name_translations: names,
        ayah_count: row.ayah_count as u16,
        revelation_place: row.revelation_place.as_deref().map(parse_place).transpose()?,
        revelation_order: row.revelation_order.map(|value| value as u16),
        basmala: parse_basmala(&row.basmala)?,
        ruku_count: row.ruku_count.map(|value| value as u16),
        metadata_provenance: parse_provenance_id(
            row.metadata_provenance_id.as_deref().unwrap_or(""),
        )?,
    })
}

fn map_token(row: &TokenRow) -> Result<Token, ReaderError> {
    Ok(Token {
        edition_id: parse_edition_id(&row.edition_id)?,
        surah: SurahNumber::new(row.surah as u16)
            .map_err(|_| storage_broken(format!("bad surah number {}", row.surah)))?,
        ayah: AyahNumber::new(row.ayah as u32)
            .map_err(|_| storage_broken(format!("bad ayah number {}", row.ayah)))?,
        position: TokenPosition::new(row.position as u16)
            .map_err(|_| storage_broken(format!("bad token position {}", row.position)))?,
        surface: row.surface.clone(),
        surface_hash: parse_tagged(&row.surface_hash)?,
        char_start: row.char_start as u32,
        char_end: row.char_end as u32,
        byte_start: row.byte_start as u32,
        byte_end: row.byte_end as u32,
        is_pause_mark: row.is_pause_mark,
        global_token_index: row.global_token_index as u64,
    })
}

/// An edition resolved for reading, with its generation.
#[derive(Debug, Clone)]
pub struct ResolvedEdition {
    /// Edition row id.
    pub id: String,
    /// Slug.
    pub slug: String,
    /// Version string.
    pub version: String,
    /// Current corpus generation (cache key component).
    pub generation: i64,
}

/// Filter for listing editions.
#[derive(Debug, Clone, Default)]
pub struct EditionFilter {
    /// Only this status.
    pub status: Option<EditionStatus>,
    /// Only this language tag.
    pub language: Option<String>,
}

/// Deterministic, LLM-free Quran lookup (plan D1.6).
#[async_trait::async_trait]
pub trait QuranReader: Send + Sync {
    /// List editions, optionally filtered.
    async fn list_editions(&self, filter: EditionFilter) -> Result<Vec<QuranEdition>, ReaderError>;
    /// Fetch one edition by selector.
    async fn get_edition(&self, selector: &EditionSelector) -> Result<QuranEdition, ReaderError>;
    /// List surahs of the selected edition.
    async fn list_surahs(&self, selector: &EditionSelector) -> Result<Vec<Surah>, ReaderError>;
    /// Fetch one ayah view.
    async fn get_ayah(
        &self,
        reference: &QuranRef,
        options: &AyahOptions,
    ) -> Result<AyahView, ReaderError>;
    /// Fetch ayah views for a range/surah/division reference.
    async fn get_ayahs(
        &self,
        reference: &QuranRef,
        options: &AyahOptions,
    ) -> Result<Vec<AyahView>, ReaderError>;
    /// Context around a focal ayah, bounded by canonical structure.
    async fn get_context(
        &self,
        reference: &QuranRef,
        spec: &ContextSpec,
    ) -> Result<ContextView, ReaderError>;
    /// Surface tokens for an ayah-level reference.
    async fn get_tokens(&self, reference: &QuranRef) -> Result<Vec<Token>, ReaderError>;
    /// Parse a reference and bounds-check it against the active edition.
    async fn resolve(&self, text: &str) -> Result<ResolvedRef, ReaderError>;
}

/// Cache size: ayah views are small; 1024 entries cover repeated reads.
const CACHE_CAPACITY: usize = 1024;

/// The reader service.
pub struct QuranReaderService {
    db: Arc<SqliteDatabase>,
    cache: Mutex<LruCache<String, AyahView>>,
}

impl QuranReaderService {
    /// Wrap a database handle.
    pub fn new(db: Arc<SqliteDatabase>) -> Self {
        Self {
            db,
            cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(CACHE_CAPACITY).expect("nonzero cache"),
            )),
        }
    }

    /// The underlying database handle.
    pub fn database(&self) -> &SqliteDatabase {
        &self.db
    }

    fn cache_key(edition: &ResolvedEdition, reference: &QuranRef, options: &AyahOptions) -> String {
        let options_hash = serde_json::to_string(options).unwrap_or_default();
        format!(
            "{}|{}|{}|{}|{options_hash}",
            edition.id,
            edition.version,
            edition.generation,
            quran_core::serialize(reference)
        )
    }

    async fn resolve_edition(
        &self,
        selector: &EditionSelector,
    ) -> Result<ResolvedEdition, ReaderError> {
        let mut uow = self.db.write().await.map_err(ReaderError::from)?;
        let pointer = uow.quran().get_active().await.map_err(ReaderError::from)?;
        let generation = pointer.as_ref().map(|row| row.corpus_generation).unwrap_or(0);
        let row = match selector {
            EditionSelector::Active => match pointer.as_ref() {
                Some(active) => uow
                    .quran()
                    .get_edition(&active.edition_id)
                    .await
                    .map_err(ReaderError::from)?
                    .ok_or_else(|| ReaderError::EditionNotFound("active".to_string()))?,
                None => return Err(ReaderError::EditionNotFound("active".to_string())),
            },
            EditionSelector::Slug(slug) => {
                let active_id = pointer.map(|row| row.edition_id);
                let mut found = None;
                // The slug resolves at its active version: it must be the pointer's edition.
                if let Some(id) = active_id {
                    let candidate =
                        uow.quran().get_edition(&id).await.map_err(ReaderError::from)?;
                    if candidate.as_ref().is_some_and(|row| &row.slug == slug) {
                        found = candidate;
                    }
                }
                found.ok_or_else(|| ReaderError::EditionNotFound(slug.clone()))?
            }
            EditionSelector::Pinned { slug, version } => uow
                .quran()
                .get_edition_by_slug_version(slug, &version.to_string())
                .await
                .map_err(ReaderError::from)?
                .ok_or_else(|| ReaderError::EditionNotFound(format!("{slug}@{version}")))?,
        };
        uow.rollback().await.map_err(ReaderError::from)?;
        Ok(ResolvedEdition {
            id: row.id.clone(),
            slug: row.slug.clone(),
            version: row.version.clone(),
            generation,
        })
    }

    async fn ayah_location(
        &self,
        edition_id: &str,
        surah: SurahNumber,
        ayah: AyahNumber,
    ) -> Result<AyahLocation, ReaderError> {
        let mut uow = self.db.write().await.map_err(ReaderError::from)?;
        let row = uow
            .quran()
            .get_ayah(edition_id, i64::from(surah.get()), i64::from(ayah.get()))
            .await
            .map_err(ReaderError::from)?
            .ok_or_else(|| ReaderError::AyahNotFound(format!("{surah}:{ayah}")))?;
        uow.rollback().await.map_err(ReaderError::from)?;
        Ok(AyahLocation { surah, ayah, global: row.global_ayah_index as u32 })
    }

    async fn build_view(
        &self,
        edition: &ResolvedEdition,
        location: AyahLocation,
        options: &AyahOptions,
    ) -> Result<AyahView, ReaderError> {
        let mut uow = self.db.write().await.map_err(ReaderError::from)?;
        let edition_row = uow
            .quran()
            .get_edition(&edition.id)
            .await
            .map_err(ReaderError::from)?
            .ok_or_else(|| ReaderError::EditionNotFound(edition.id.clone()))?;
        let surah_row = uow
            .quran()
            .get_surah(&edition.id, i64::from(location.surah.get()))
            .await
            .map_err(ReaderError::from)?
            .ok_or_else(|| ReaderError::AyahNotFound(format!("surah {}", location.surah)))?;
        let ayah_row = uow
            .quran()
            .get_ayah(&edition.id, i64::from(location.surah.get()), i64::from(location.ayah.get()))
            .await
            .map_err(ReaderError::from)?
            .ok_or_else(|| {
                ReaderError::AyahNotFound(format!("{}:{}", location.surah, location.ayah))
            })?;
        let surah = map_surah(&surah_row)?;
        let reference = format!(
            "quran:{}@{}:{}:{}",
            edition.slug, edition.version, location.surah, location.ayah
        );
        let canonical = QuranQuotation::new(QuotationParts {
            reference: reference.clone(),
            surah_number: location.surah,
            surah_name_arabic: surah.name_arabic.clone(),
            surah_name_translit: surah.name_transliteration.clone(),
            ayah_range: (location.ayah, location.ayah),
            arabic_text: ayah_row.text.clone(),
            text_hash: parse_tagged(&ayah_row.text_hash)?,
            edition: EditionRef {
                slug: edition.slug.clone(),
                version: edition_row
                    .version
                    .parse::<SemVer>()
                    .map_err(|_| storage_broken("bad edition version".to_string()))?,
                script: parse_script(&edition_row.script),
                riwayah: edition_row.riwayah.clone(),
            },
            translation: None,
            deep_link: format!(
                "/read/{}@{}/{}:{}",
                edition.slug, edition.version, location.surah, location.ayah
            ),
            page: ayah_row.page.map(|value| value as u32),
            juz: ayah_row.juz.map(|value| value as u16),
        })?;

        let mut translations = Vec::new();
        for slug in &options.translations {
            translations.push(self.translation_for(&mut uow, &edition_row, slug, &location).await?);
        }
        let tokens = if options.tokens {
            let rows = uow
                .quran()
                .get_tokens(
                    &edition.id,
                    i64::from(location.surah.get()),
                    i64::from(location.ayah.get()),
                )
                .await
                .map_err(ReaderError::from)?;
            let mut mapped = Vec::with_capacity(rows.len());
            for row in &rows {
                mapped.push(map_token(row)?);
            }
            Some(mapped)
        } else {
            None
        };
        // Word glosses are an optional attributed dataset keyed by the served
        // edition: only glosses aligned to this edition are returned, each
        // carrying its dataset id (principle 5).
        let word_glosses = if options.glosses {
            let rows = uow
                .quran()
                .list_word_glosses(
                    &edition.id,
                    i64::from(location.surah.get()),
                    i64::from(location.ayah.get()),
                )
                .await
                .map_err(ReaderError::from)?;
            let mut glosses = Vec::with_capacity(rows.len());
            for row in &rows {
                glosses.push(AttributedGloss {
                    dataset: row.gloss_dataset_id.clone(),
                    language: row.language.parse::<Language>().map_err(|_| {
                        storage_broken(format!("bad gloss language `{}`", row.language))
                    })?,
                    gloss: row.gloss.clone(),
                });
            }
            if glosses.is_empty() { None } else { Some(glosses) }
        } else {
            None
        };
        uow.rollback().await.map_err(ReaderError::from)?;
        Ok(AyahView { canonical, translations, word_glosses, tokens })
    }

    async fn translation_for(
        &self,
        uow: &mut Box<dyn storage::UnitOfWork>,
        edition_row: &QuranEditionRow,
        slug: &str,
        location: &AyahLocation,
    ) -> Result<AttributedTranslation, ReaderError> {
        let editions = uow.quran().list_translation_editions().await.map_err(ReaderError::from)?;
        let mut candidates: Vec<_> = editions.iter().filter(|row| row.slug == slug).collect();
        candidates.sort_by(|a, b| a.version.cmp(&b.version));
        let translation = candidates.into_iter().next_back().ok_or_else(|| {
            ReaderError::TranslationNotFound(format!("translation edition `{slug}`"))
        })?;
        if translation.aligned_edition_id != edition_row.id {
            return Err(ReaderError::TranslationNotFound(format!(
                "`{slug}` is not aligned to this edition"
            )));
        }
        let passage = uow
            .quran()
            .get_translation_passage(
                &translation.id,
                i64::from(location.surah.get()),
                i64::from(location.ayah.get()),
            )
            .await
            .map_err(ReaderError::from)?
            .ok_or_else(|| {
                ReaderError::TranslationNotFound(format!(
                    "`{slug}` has no passage for {}:{}",
                    location.surah, location.ayah
                ))
            })?;
        let language = translation
            .language
            .parse::<Language>()
            .map_err(|_| storage_broken(format!("bad language `{}`", translation.language)))?;
        let license: domain::LicenseRecord = serde_json::from_str(&translation.license_json)
            .map_err(|err| storage_broken(format!("bad translation license: {err}")))?;
        let attributed = AttributedTranslation::new(
            translation.translator.clone(),
            format!("{}@{}", translation.slug, translation.version),
            language,
            passage.text.clone(),
            license,
        )?;
        Ok(attributed)
    }

    /// Expand a reference to focal locations (single ayah each).
    async fn expand(
        &self,
        edition: &ResolvedEdition,
        reference: &QuranRef,
    ) -> Result<Vec<AyahLocation>, ReaderError> {
        match reference {
            QuranRef::Edition { .. } => {
                Err(ReaderError::AyahNotFound("edition references need a locator".to_string()))
            }
            QuranRef::Surah { surah, .. } => {
                let mut uow = self.db.write().await.map_err(ReaderError::from)?;
                let row = uow
                    .quran()
                    .get_surah(&edition.id, i64::from(surah.get()))
                    .await
                    .map_err(ReaderError::from)?
                    .ok_or_else(|| ReaderError::AyahNotFound(format!("surah {surah}")))?;
                uow.rollback().await.map_err(ReaderError::from)?;
                let mut locations = Vec::new();
                for ayah in 1..=row.ayah_count as u32 {
                    let number = AyahNumber::new(ayah)
                        .map_err(|_| ReaderError::AyahNotFound(format!("{surah}:{ayah}")))?;
                    locations.push(self.ayah_location(&edition.id, *surah, number).await?);
                }
                Ok(locations)
            }
            QuranRef::Ayah { surah, ayah, .. } => {
                Ok(vec![self.ayah_location(&edition.id, *surah, *ayah).await?])
            }
            QuranRef::AyahRange { start, end, .. } => {
                let (start_surah, start_ayah) = *start;
                let (end_surah, end_ayah) = *end;
                let first = self.ayah_location(&edition.id, start_surah, start_ayah).await?;
                let last = self.ayah_location(&edition.id, end_surah, end_ayah).await?;
                if last.global < first.global {
                    return Err(ReaderError::AyahNotFound("empty range".to_string()));
                }
                let mut uow = self.db.write().await.map_err(ReaderError::from)?;
                let rows = uow
                    .quran()
                    .list_ayahs_range(&edition.id, i64::from(first.global), i64::from(last.global))
                    .await
                    .map_err(ReaderError::from)?;
                uow.rollback().await.map_err(ReaderError::from)?;
                let mut locations = Vec::new();
                for row in &rows {
                    locations.push(AyahLocation {
                        surah: SurahNumber::new(row.surah as u16).map_err(|_| {
                            ReaderError::AyahNotFound(format!("surah {}", row.surah))
                        })?,
                        ayah: AyahNumber::new(row.ayah as u32)
                            .map_err(|_| ReaderError::AyahNotFound(format!("ayah {}", row.ayah)))?,
                        global: row.global_ayah_index as u32,
                    });
                }
                Ok(locations)
            }
            QuranRef::Token { surah, ayah, .. } => {
                Ok(vec![self.ayah_location(&edition.id, *surah, *ayah).await?])
            }
            QuranRef::Division { kind, number, .. } => {
                let keyword = kind.keyword();
                let mut uow = self.db.write().await.map_err(ReaderError::from)?;
                let divisions = uow
                    .quran()
                    .list_divisions(&edition.id, keyword)
                    .await
                    .map_err(ReaderError::from)?;
                uow.rollback().await.map_err(ReaderError::from)?;
                let division = divisions
                    .iter()
                    .find(|row| row.number == i64::from(*number))
                    .ok_or_else(|| ReaderError::DivisionNotFound(format!("{keyword}:{number}")))?;
                let mut uow = self.db.write().await.map_err(ReaderError::from)?;
                let rows = uow
                    .quran()
                    .list_ayahs_range(&edition.id, division.start_global, division.end_global)
                    .await
                    .map_err(ReaderError::from)?;
                uow.rollback().await.map_err(ReaderError::from)?;
                let mut locations = Vec::new();
                for row in &rows {
                    locations.push(AyahLocation {
                        surah: SurahNumber::new(row.surah as u16).map_err(|_| {
                            ReaderError::AyahNotFound(format!("surah {}", row.surah))
                        })?,
                        ayah: AyahNumber::new(row.ayah as u32)
                            .map_err(|_| ReaderError::AyahNotFound(format!("ayah {}", row.ayah)))?,
                        global: row.global_ayah_index as u32,
                    });
                }
                Ok(locations)
            }
        }
    }
}

#[async_trait]
impl QuranReader for QuranReaderService {
    async fn list_editions(&self, filter: EditionFilter) -> Result<Vec<QuranEdition>, ReaderError> {
        let mut uow = self.db.write().await.map_err(ReaderError::from)?;
        let rows = uow.quran().list_editions().await.map_err(ReaderError::from)?;
        uow.rollback().await.map_err(ReaderError::from)?;
        let mut editions = Vec::new();
        for row in &rows {
            let edition = map_edition(row)?;
            if let Some(status) = filter.status
                && edition.status != status
            {
                continue;
            }
            if let Some(language) = &filter.language
                && edition.language.to_string() != *language
            {
                continue;
            }
            editions.push(edition);
        }
        Ok(editions)
    }

    async fn get_edition(&self, selector: &EditionSelector) -> Result<QuranEdition, ReaderError> {
        let resolved = self.resolve_edition(selector).await?;
        let mut uow = self.db.write().await.map_err(ReaderError::from)?;
        let row = uow
            .quran()
            .get_edition(&resolved.id)
            .await
            .map_err(ReaderError::from)?
            .ok_or_else(|| ReaderError::EditionNotFound(resolved.id.clone()))?;
        uow.rollback().await.map_err(ReaderError::from)?;
        map_edition(&row)
    }

    async fn list_surahs(&self, selector: &EditionSelector) -> Result<Vec<Surah>, ReaderError> {
        let resolved = self.resolve_edition(selector).await?;
        let mut uow = self.db.write().await.map_err(ReaderError::from)?;
        let rows = uow.quran().list_surahs(&resolved.id).await.map_err(ReaderError::from)?;
        uow.rollback().await.map_err(ReaderError::from)?;
        rows.iter().map(map_surah).collect()
    }

    async fn get_ayah(
        &self,
        reference: &QuranRef,
        options: &AyahOptions,
    ) -> Result<AyahView, ReaderError> {
        let edition = self.resolve_edition(reference.edition()).await?;
        let key = Self::cache_key(&edition, reference, options);
        if let Some(cached) =
            self.cache.lock().map(|mut cache| cache.get(&key).cloned()).unwrap_or(None)
        {
            return Ok(cached);
        }
        let locations = self.expand(&edition, reference).await?;
        let first = locations
            .into_iter()
            .next()
            .ok_or_else(|| ReaderError::AyahNotFound(quran_core::serialize(reference)))?;
        let view = self.build_view(&edition, first, options).await?;
        if let Ok(mut cache) = self.cache.lock() {
            cache.put(key, view.clone());
        }
        Ok(view)
    }

    async fn get_ayahs(
        &self,
        reference: &QuranRef,
        options: &AyahOptions,
    ) -> Result<Vec<AyahView>, ReaderError> {
        let edition = self.resolve_edition(reference.edition()).await?;
        let mut views = Vec::new();
        for location in self.expand(&edition, reference).await? {
            views.push(self.build_view(&edition, location, options).await?);
        }
        Ok(views)
    }

    async fn get_context(
        &self,
        reference: &QuranRef,
        spec: &ContextSpec,
    ) -> Result<ContextView, ReaderError> {
        let edition = self.resolve_edition(reference.edition()).await?;
        let focal = match reference {
            QuranRef::Ayah { surah, ayah, .. } | QuranRef::Token { surah, ayah, .. } => {
                self.ayah_location(&edition.id, *surah, *ayah).await?
            }
            QuranRef::AyahRange { start, .. } => {
                self.ayah_location(&edition.id, start.0, start.1).await?
            }
            QuranRef::Surah { .. } | QuranRef::Division { .. } | QuranRef::Edition { .. } => {
                return Err(ReaderError::AyahNotFound(
                    "context needs an ayah-level reference; use get_ayahs for ranges".to_string(),
                ));
            }
        };
        // Boundary as an inclusive global range around the focal ayah.
        let mut uow = self.db.write().await.map_err(ReaderError::from)?;
        let (mut low, mut high) = (focal.global, focal.global);
        match spec.boundary {
            ContextBoundary::None => {}
            ContextBoundary::Surah => {
                let row = uow
                    .quran()
                    .get_surah(&edition.id, i64::from(focal.surah.get()))
                    .await
                    .map_err(ReaderError::from)?
                    .ok_or_else(|| ReaderError::AyahNotFound(format!("surah {}", focal.surah)))?;
                let first = uow
                    .quran()
                    .get_ayah(&edition.id, i64::from(focal.surah.get()), 1)
                    .await
                    .map_err(ReaderError::from)?
                    .ok_or_else(|| ReaderError::AyahNotFound("empty surah".to_string()))?;
                low = first.global_ayah_index as u32;
                high = low + row.ayah_count as u32 - 1;
            }
            ContextBoundary::Juz | ContextBoundary::Ruku | ContextBoundary::Page => {
                let kind = match spec.boundary {
                    ContextBoundary::Juz => "juz",
                    ContextBoundary::Ruku => "ruku",
                    _ => "page",
                };
                let divisions = uow
                    .quran()
                    .list_divisions(&edition.id, kind)
                    .await
                    .map_err(ReaderError::from)?;
                let division = divisions
                    .iter()
                    .find(|row| {
                        focal.global >= row.start_global as u32
                            && focal.global <= row.end_global as u32
                    })
                    .ok_or_else(|| {
                        ReaderError::AyahNotFound(format!("ayah outside every {kind}"))
                    })?;
                low = division.start_global as u32;
                high = division.end_global as u32;
            }
        }
        uow.rollback().await.map_err(ReaderError::from)?;

        let take_before = (focal.global - low).min(u32::from(spec.before));
        let take_after = (high - focal.global).min(u32::from(spec.after));
        let mut before = Vec::new();
        if take_before > 0 {
            let mut uow = self.db.write().await.map_err(ReaderError::from)?;
            let rows = uow
                .quran()
                .list_ayahs_range(
                    &edition.id,
                    i64::from(focal.global - take_before),
                    i64::from(focal.global - 1),
                )
                .await
                .map_err(ReaderError::from)?;
            uow.rollback().await.map_err(ReaderError::from)?;
            for row in &rows {
                before.push(
                    self.build_view(
                        &edition,
                        AyahLocation {
                            surah: SurahNumber::new(row.surah as u16).map_err(|_| {
                                ReaderError::AyahNotFound(format!("surah {}", row.surah))
                            })?,
                            ayah: AyahNumber::new(row.ayah as u32).map_err(|_| {
                                ReaderError::AyahNotFound(format!("ayah {}", row.ayah))
                            })?,
                            global: row.global_ayah_index as u32,
                        },
                        &AyahOptions::default(),
                    )
                    .await?,
                );
            }
        }
        let mut after = Vec::new();
        if take_after > 0 {
            let mut uow = self.db.write().await.map_err(ReaderError::from)?;
            let rows = uow
                .quran()
                .list_ayahs_range(
                    &edition.id,
                    i64::from(focal.global + 1),
                    i64::from(focal.global + take_after),
                )
                .await
                .map_err(ReaderError::from)?;
            uow.rollback().await.map_err(ReaderError::from)?;
            for row in &rows {
                after.push(
                    self.build_view(
                        &edition,
                        AyahLocation {
                            surah: SurahNumber::new(row.surah as u16).map_err(|_| {
                                ReaderError::AyahNotFound(format!("surah {}", row.surah))
                            })?,
                            ayah: AyahNumber::new(row.ayah as u32).map_err(|_| {
                                ReaderError::AyahNotFound(format!("ayah {}", row.ayah))
                            })?,
                            global: row.global_ayah_index as u32,
                        },
                        &AyahOptions::default(),
                    )
                    .await?,
                );
            }
        }
        // Hard cap: trim after first, then before.
        let mut total = 1 + before.len() + after.len();
        let cap = usize::from(spec.max_ayahs.max(1));
        while total > cap && !after.is_empty() {
            after.pop();
            total -= 1;
        }
        while total > cap && !before.is_empty() {
            before.remove(0);
            total -= 1;
        }
        let low_global =
            before.first().map(|_| focal.global - before.len() as u32).unwrap_or(focal.global);
        let high_global = focal.global + after.len() as u32;
        let focal_view = self.build_view(&edition, focal, &AyahOptions::default()).await?;
        let surah = if spec.include_surah_header {
            let mut uow = self.db.write().await.map_err(ReaderError::from)?;
            let row = uow
                .quran()
                .get_surah(&edition.id, i64::from(focal.surah.get()))
                .await
                .map_err(ReaderError::from)?;
            uow.rollback().await.map_err(ReaderError::from)?;
            row.map(|row| map_surah(&row)).transpose()?
        } else {
            None
        };
        Ok(ContextView {
            canonical_reference: focal_view.canonical.reference().to_string(),
            focal: focal_view,
            before,
            after,
            surah,
            global_range: (low_global, high_global),
        })
    }

    async fn get_tokens(&self, reference: &QuranRef) -> Result<Vec<Token>, ReaderError> {
        let edition = self.resolve_edition(reference.edition()).await?;
        let (surah, ayah) = match reference {
            QuranRef::Ayah { surah, ayah, .. } | QuranRef::Token { surah, ayah, .. } => {
                (*surah, *ayah)
            }
            _ => {
                return Err(ReaderError::AyahNotFound(
                    "tokens need an ayah-level reference".to_string(),
                ));
            }
        };
        // Bounds-check first so missing ayahs report cleanly.
        self.ayah_location(&edition.id, surah, ayah).await?;
        let mut uow = self.db.write().await.map_err(ReaderError::from)?;
        let rows = uow
            .quran()
            .get_tokens(&edition.id, i64::from(surah.get()), i64::from(ayah.get()))
            .await
            .map_err(ReaderError::from)?;
        uow.rollback().await.map_err(ReaderError::from)?;
        rows.iter().map(map_token).collect()
    }

    async fn resolve(&self, text: &str) -> Result<ResolvedRef, ReaderError> {
        let parsed = quran_core::parse(text)?;
        // Bounds-check against the active edition.
        let edition = self.resolve_edition(&EditionSelector::Active).await?;
        let locations = self.expand(&edition, &parsed).await?;
        if locations.is_empty() {
            return Err(ReaderError::AyahNotFound(text.to_string()));
        }
        Ok(quran_core::resolve(text)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_changes_with_generation() {
        let first = ResolvedEdition {
            id: "ed-1".into(),
            slug: "s".into(),
            version: "1.0.0".into(),
            generation: 1,
        };
        let second = ResolvedEdition { generation: 2, ..first.clone() };
        let reference = QuranRef::Surah {
            edition: EditionSelector::Active,
            surah: SurahNumber::new(1).unwrap(),
        };
        let options = AyahOptions::default();
        assert_ne!(
            QuranReaderService::cache_key(&first, &reference, &options),
            QuranReaderService::cache_key(&second, &reference, &options)
        );
    }
}
