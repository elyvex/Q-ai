//! Tool and citation backends over the deterministic reader (D1.8, D1.9).
//!
//! # application::quran_tools
//!
//! Implements `tool_registry::QuranBackend` and `citations::CitationSource`
//! for [`QuranReaderService`](crate::quran_reader::QuranReaderService), plus
//! citation persistence through the `citations` table. Tools stay read-only;
//! every result carries edition identity, canonical references, and a
//! deterministic reproducibility checksum.

use std::sync::Arc;

use async_trait::async_trait;
use quran_core::{
    AyahNumber, AyahOptions, AyahView, ContextSpec, ContextView, QuranRef, SurahNumber,
};
use storage::Database as _;
use tool_registry::{BackendMeta, GetAyahParams, GetContextParams, QuranBackend, ToolRegistry};
use tools::{ToolError, ToolResult};

use crate::quran_reader::{QuranReader, QuranReaderService, ReaderError};

/// Map a reader error to a tool error (codes delegated for HTTP mapping).
pub fn to_tool_error(error: ReaderError) -> ToolError {
    use storage::error::Diagnostic;
    ToolError::Backend { code: error.code().to_string(), detail: error.to_string() }
}

fn tool_error(error: ReaderError) -> ToolError {
    to_tool_error(error)
}

async fn backend_meta(
    reader: &QuranReaderService,
    reference: &QuranRef,
) -> Result<BackendMeta, ToolError> {
    let edition = reader.get_edition(reference.edition()).await.map_err(tool_error)?;
    let mut uow = reader.database().write().await.map_err(|err| ToolError::Backend {
        code: "QAI-QUR-0310".into(),
        detail: err.to_string(),
    })?;
    let generation = uow.quran().get_active().await.map_err(|err| ToolError::Backend {
        code: "QAI-QUR-0310".into(),
        detail: err.to_string(),
    })?;
    uow.rollback().await.map_err(|err| ToolError::Backend {
        code: "QAI-QUR-0310".into(),
        detail: err.to_string(),
    })?;
    Ok(BackendMeta {
        edition_version: edition.version.to_string(),
        edition_slug: edition.slug.clone(),
        edition_id: edition.id.to_string(),
        corpus_generation: generation.map(|row| row.corpus_generation as u64).unwrap_or(0),
        text_hash: edition.text_hash.hex.clone(),
        script: match &edition.script {
            quran_core::Script::Uthmani => "uthmani".to_string(),
            quran_core::Script::ImlaeiSimple => "imlaei_simple".to_string(),
            quran_core::Script::Other(name) => format!("other:{name}"),
        },
        riwayah: edition.riwayah.clone(),
        numbering_scheme: match &edition.verse_numbering_scheme {
            quran_core::NumberingScheme::Hafs => "hafs".to_string(),
            quran_core::NumberingScheme::Kufi => "kufi".to_string(),
            quran_core::NumberingScheme::Custom(name) => name.clone(),
        },
    })
}

/// The tool backend: [`QuranReaderService`] behind the registry trait.
pub struct ReaderToolBackend {
    reader: Arc<QuranReaderService>,
}

impl ReaderToolBackend {
    /// Wrap a reader.
    pub fn new(reader: Arc<QuranReaderService>) -> Self {
        Self { reader }
    }

    /// A registry wired to the reader.
    pub fn registry(reader: Arc<QuranReaderService>) -> ToolRegistry {
        ToolRegistry::new(Arc::new(Self::new(reader)))
    }
}

#[async_trait]
impl QuranBackend for ReaderToolBackend {
    async fn backend_get_ayah(
        &self,
        reference: &QuranRef,
        options: &AyahOptions,
    ) -> Result<(AyahView, BackendMeta), ToolError> {
        let view = self.reader.get_ayah(reference, options).await.map_err(tool_error)?;
        let meta = backend_meta(&self.reader, reference).await?;
        Ok((view, meta))
    }

    async fn backend_get_context(
        &self,
        reference: &QuranRef,
        spec: &ContextSpec,
    ) -> Result<(ContextView, BackendMeta), ToolError> {
        let view = self.reader.get_context(reference, spec).await.map_err(tool_error)?;
        let meta = backend_meta(&self.reader, reference).await?;
        Ok((view, meta))
    }
}

/// The citation source: [`QuranReaderService`] behind the resolver trait.
pub struct ReaderCitationSource {
    reader: Arc<QuranReaderService>,
}

impl ReaderCitationSource {
    /// Wrap a reader.
    pub fn new(reader: Arc<QuranReaderService>) -> Self {
        Self { reader }
    }

    /// A resolver wired to the reader.
    pub fn resolver(reader: Arc<QuranReaderService>) -> citations::CitationResolver {
        citations::CitationResolver::new(Arc::new(Self::new(reader)))
    }
}

#[async_trait]
impl citations::CitationSource for ReaderCitationSource {
    async fn edition_exists(
        &self,
        slug: &str,
        version: &str,
    ) -> Result<bool, citations::CitationError> {
        use quran_core::EditionSelector;
        let selector = EditionSelector::Pinned {
            slug: slug.to_string(),
            version: version.parse().map_err(|_| citations::CitationError::Backend {
                detail: format!("bad version `{version}`"),
            })?,
        };
        match self.reader.get_edition(&selector).await {
            Ok(_) => Ok(true),
            Err(ReaderError::EditionNotFound(_)) => Ok(false),
            Err(err) => Err(citations::CitationError::Backend { detail: err.to_string() }),
        }
    }

    async fn fetch_ayah_text(
        &self,
        slug: &str,
        version: &str,
        surah: u16,
        ayah: u32,
    ) -> Result<Option<String>, citations::CitationError> {
        use quran_core::{AyahNumber, EditionSelector, SurahNumber};
        let reference = QuranRef::Ayah {
            edition: EditionSelector::Pinned {
                slug: slug.to_string(),
                version: version.parse().map_err(|_| citations::CitationError::Backend {
                    detail: format!("bad version `{version}`"),
                })?,
            },
            surah: SurahNumber::new(surah).map_err(|_| citations::CitationError::Backend {
                detail: format!("bad surah {surah}"),
            })?,
            ayah: AyahNumber::new(ayah).map_err(|_| citations::CitationError::Backend {
                detail: format!("bad ayah {ayah}"),
            })?,
        };
        match self.reader.get_ayah(&reference, &AyahOptions::default()).await {
            Ok(view) => Ok(Some(view.canonical.arabic_text().to_string())),
            Err(ReaderError::AyahNotFound(_)) | Err(ReaderError::EditionNotFound(_)) => Ok(None),
            Err(err) => Err(citations::CitationError::Backend { detail: err.to_string() }),
        }
    }
}

/// Persist a resolved citation for later re-verification (§12.1, §21.2).
pub async fn persist_citation(
    db: &dyn storage::Database,
    resolved: &citations::ResolvedCitation,
    citation: &citations::Citation,
    ingestion_version: &str,
    at: &str,
) -> Result<(), storage::error::StorageError> {
    let mut uow = db.write().await?;
    uow.quran()
        .insert_citation(storage::quran::CitationRow {
            id: citation.id.clone(),
            kind: "quran".to_string(),
            canonical_reference: citation.canonical_reference.clone(),
            source_id: None,
            source_version_id: None,
            edition_ref: Some(format!("{}@{}", citation.edition_slug, citation.edition_version)),
            location_json: serde_json::json!({
                "surah": citation.surah.get(),
                "ayah": citation.ayah.get(),
            })
            .to_string(),
            quoted_text_hash: resolved.text_hash.clone(),
            ingestion_version: ingestion_version.to_string(),
            resolved_at: at.to_string(),
            verdict: verdict_str(&resolved.verdict),
        })
        .await?;
    uow.commit().await
}

fn verdict_str(verdict: &citations::QuotationVerdict) -> String {
    verdict.label().to_string()
}

/// Verify an externally supplied quotation against the canonical source of
/// truth, returning the verdict and the RESOLVED canonical text hash.
///
/// This is the one shared verifying implementation behind every quoted-text
/// answer path (the `qai quran verify-quotation` CLI verb and the HTTP path).
/// The citation is built with the pinned `slug@version`, so the edition is
/// never inferred from the supplied text (ADR-0111). The returned hash is the
/// one read from the canonical row by the reader-backed resolver — it is never
/// recomputed from the supplied text, so a caller can never present a hash it
/// synthesized (T-02-19). A hard-failure verdict is mapped through
/// [`citations::require_exact`], so `Mismatch`/`LocationNotFound`/
/// `EditionNotFound`/`AccessDenied` return a typed error instead of a success.
///
/// `verify_quotation` delegates to the same `resolve` path used here; `resolve`
/// is called directly because it also yields the resolved canonical hash in a
/// single fetch.
pub async fn verify_canonical_quotation(
    reader: &Arc<QuranReaderService>,
    edition_slug: &str,
    edition_version: &str,
    surah: u16,
    ayah: u32,
    text: &str,
) -> Result<(citations::QuotationVerdict, String), citations::CitationError> {
    let citation = citations::Citation {
        id: "verify-quotation".to_string(),
        kind: citations::CitationKind::Quran,
        canonical_reference: format!("quran:{edition_slug}@{edition_version}:{surah}:{ayah}"),
        quoted_text: text.to_string(),
        edition_slug: edition_slug.to_string(),
        edition_version: edition_version.to_string(),
        surah: SurahNumber::new(surah).map_err(|_| citations::CitationError::Backend {
            detail: format!("bad surah {surah}"),
        })?,
        ayah: AyahNumber::new(ayah).map_err(|_| citations::CitationError::Backend {
            detail: format!("bad ayah {ayah}"),
        })?,
    };
    let resolver = ReaderCitationSource::resolver(reader.clone());
    let resolved = resolver.resolve(&citation).await?;
    citations::require_exact(&resolved.verdict)?;
    let text_hash = resolved.text_hash.ok_or_else(|| citations::CitationError::Backend {
        detail: "resolved citation carried no canonical hash".to_string(),
    })?;
    Ok((resolved.verdict, text_hash))
}

/// Convenience: run `quran.get_ayah` against a reader.
pub async fn tool_get_ayah(
    reader: &Arc<QuranReaderService>,
    params: GetAyahParams,
) -> Result<(ToolResult<Vec<AyahView>>, tool_registry::BackendMeta), ToolError> {
    ReaderToolBackend::registry(reader.clone()).get_ayah(params).await
}

/// Convenience: run `quran.get_context` against a reader.
pub async fn tool_get_context(
    reader: &Arc<QuranReaderService>,
    params: GetContextParams,
) -> Result<(ToolResult<quran_core::ContextView>, tool_registry::BackendMeta), ToolError> {
    ReaderToolBackend::registry(reader.clone()).get_context(params).await
}
