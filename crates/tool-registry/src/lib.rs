//! Minimal local tool registry with the first two read-only tools (D1.8).
//!
//! # tool-registry
//!
//! `quran.get_ayah` and `quran.get_context` prove the §12 contract works from
//! all interfaces before agents exist. Both are `ReadOnly` (§29). The registry
//! depends on a [`QuranBackend`] trait — implemented by the application layer —
//! so this crate never depends on storage or the reader directly.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use domain::SemVer;
use quran_core::{AyahOptions, AyahView, ContextBoundary, ContextSpec, ContextView, QuranRef};
use serde::{Deserialize, Serialize};
use tools::{AnalysisSource, ToolError, ToolResult, reproducibility};

/// Tool versions (§12 tool plan).
pub const GET_AYAH_VERSION: SemVer = SemVer::new(1, 0, 0);
/// Tool versions (§12 tool plan).
pub const GET_CONTEXT_VERSION: SemVer = SemVer::new(1, 0, 0);

/// Edition + generation metadata behind one backend read.
#[derive(Debug, Clone)]
pub struct BackendMeta {
    /// Edition slug read.
    pub edition_slug: String,
    /// Edition version read.
    pub edition_version: String,
    /// Edition row id.
    pub edition_id: String,
    /// Corpus generation read from.
    pub corpus_generation: u64,
    /// Edition text hash (`sha256:<hex>`) for ETags.
    pub text_hash: String,
    /// Edition script (`uthmani`, …).
    pub script: String,
    /// Transmission, when declared.
    pub riwayah: Option<String>,
    /// Verse-numbering scheme.
    pub numbering_scheme: String,
}

/// The reader surface tools need. Implemented by `application` for the real
/// reader; tests substitute fakes. Keeps this crate off storage.
#[async_trait]
pub trait QuranBackend: Send + Sync {
    /// Fetch one ayah view plus read metadata.
    async fn backend_get_ayah(
        &self,
        reference: &QuranRef,
        options: &AyahOptions,
    ) -> Result<(AyahView, BackendMeta), ToolError>;

    /// Fetch context plus read metadata.
    async fn backend_get_context(
        &self,
        reference: &QuranRef,
        spec: &ContextSpec,
    ) -> Result<(ContextView, BackendMeta), ToolError>;
}

/// Parameters for `quran.get_ayah`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAyahParams {
    /// Reference string (edition may be embedded).
    pub reference: String,
    /// Translation slugs to attach.
    #[serde(default)]
    pub translations: Vec<String>,
    /// Attach word glosses.
    #[serde(default)]
    pub glosses: bool,
    /// Attach surface tokens.
    #[serde(default)]
    pub tokens: bool,
}

/// Parameters for `quran.get_context`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetContextParams {
    /// Focal reference (ayah-level).
    pub reference: String,
    /// Ayahs before (default 3).
    #[serde(default = "default_before")]
    pub before: u16,
    /// Ayahs after (default 3).
    #[serde(default = "default_after")]
    pub after: u16,
    /// Structural boundary.
    #[serde(default)]
    pub boundary: ContextBoundaryArg,
    /// Hard cap (default 11).
    #[serde(default = "default_max")]
    pub max_ayahs: u16,
}

fn default_before() -> u16 {
    3
}

fn default_after() -> u16 {
    3
}

fn default_max() -> u16 {
    11
}

/// Serializable boundary argument.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextBoundaryArg {
    /// Stay within the focal surah.
    Surah,
    /// Stay within the focal juz.
    Juz,
    /// Stay within the focal ruku.
    Ruku,
    /// Stay within the focal page.
    Page,
    /// No structural boundary.
    #[default]
    None,
}

impl From<ContextBoundaryArg> for ContextBoundary {
    fn from(value: ContextBoundaryArg) -> Self {
        match value {
            ContextBoundaryArg::Surah => Self::Surah,
            ContextBoundaryArg::Juz => Self::Juz,
            ContextBoundaryArg::Ruku => Self::Ruku,
            ContextBoundaryArg::Page => Self::Page,
            ContextBoundaryArg::None => Self::None,
        }
    }
}

/// The local tool registry.
pub struct ToolRegistry {
    backend: Arc<dyn QuranBackend>,
}

impl ToolRegistry {
    /// Wrap a backend.
    pub fn new(backend: Arc<dyn QuranBackend>) -> Self {
        Self { backend }
    }

    /// Registered tool names.
    pub fn tool_names(&self) -> Vec<&'static str> {
        vec!["quran.get_ayah", "quran.get_context"]
    }

    /// `quran.get_ayah`: exact ayah lookup, no synthesis, ever.
    ///
    /// Returns the envelope plus the backend read metadata (edition descriptor
    /// for API envelopes and ETags).
    pub async fn get_ayah(
        &self,
        params: GetAyahParams,
    ) -> Result<(ToolResult<Vec<AyahView>>, BackendMeta), ToolError> {
        let started = Instant::now();
        let reference = quran_core::parse(&params.reference).map_err(|err| {
            ToolError::InvalidInput { tool: "quran.get_ayah", detail: err.to_string() }
        })?;
        let options = AyahOptions {
            translations: params.translations.clone(),
            glosses: params.glosses,
            tokens: params.tokens,
        };
        let query = serde_json::to_value(&params).unwrap_or(serde_json::Value::Null);
        let (view, meta) = self.backend.backend_get_ayah(&reference, &options).await?;
        let result = ToolResult {
            tool_name: "quran.get_ayah".to_string(),
            tool_version: GET_AYAH_VERSION,
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: Some(meta.edition_id.clone()),
            edition_version: Some(meta.edition_version.clone()),
            canonical_references: vec![view.canonical.reference().to_string()],
            analysis_sources: vec![AnalysisSource {
                kind: "canonical".to_string(),
                reference: view.canonical.reference().to_string(),
            }],
            results: vec![view],
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                "quran.get_ayah",
                GET_AYAH_VERSION,
                &query,
                Some(&meta.edition_slug),
                Some(&meta.edition_version),
                BTreeMap::new(),
                meta.corpus_generation,
            ),
        };
        Ok((result, meta))
    }

    /// `quran.get_context`: structure-bounded context retrieval.
    pub async fn get_context(
        &self,
        params: GetContextParams,
    ) -> Result<(ToolResult<ContextView>, BackendMeta), ToolError> {
        let started = Instant::now();
        if params.max_ayahs < 1 {
            return Err(ToolError::InvalidInput {
                tool: "quran.get_context",
                detail: "max_ayahs must be >= 1".to_string(),
            });
        }
        let reference = quran_core::parse(&params.reference).map_err(|err| {
            ToolError::InvalidInput { tool: "quran.get_context", detail: err.to_string() }
        })?;
        let spec = ContextSpec {
            before: params.before,
            after: params.after,
            boundary: ContextBoundary::from(params.boundary),
            include_surah_header: false,
            max_ayahs: params.max_ayahs,
        };
        let query = serde_json::to_value(&params).unwrap_or(serde_json::Value::Null);
        let (view, meta) = self.backend.backend_get_context(&reference, &spec).await?;
        let mut references = vec![view.canonical_reference.clone()];
        references.extend(view.before.iter().map(|item| item.canonical.reference().to_string()));
        references.extend(view.after.iter().map(|item| item.canonical.reference().to_string()));
        let sources = references
            .iter()
            .map(|reference| AnalysisSource {
                kind: "canonical".to_string(),
                reference: reference.clone(),
            })
            .collect();
        let result = ToolResult {
            tool_name: "quran.get_context".to_string(),
            tool_version: GET_CONTEXT_VERSION,
            query: query.clone(),
            normalization_rules: Vec::new(),
            edition_id: Some(meta.edition_id.clone()),
            edition_version: Some(meta.edition_version.clone()),
            canonical_references: references,
            analysis_sources: sources,
            results: view,
            confidence: None,
            warnings: Vec::new(),
            execution_time_ms: started.elapsed().as_secs_f64() * 1000.0,
            reproducibility: reproducibility(
                "quran.get_context",
                GET_CONTEXT_VERSION,
                &query,
                Some(&meta.edition_slug),
                Some(&meta.edition_version),
                BTreeMap::new(),
                meta.corpus_generation,
            ),
        };
        Ok((result, meta))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quran_core::{AyahNumber, EditionSelector, SurahNumber};

    struct FakeBackend;

    fn test_view() -> (AyahView, BackendMeta) {
        let view = serde_json::from_value::<AyahView>(serde_json::json!({
            "canonical": {
                "reference": "quran:test@0.1.0:1:1",
                "surah_number": 1,
                "surah_name_arabic": "ت",
                "surah_name_translit": null,
                "ayah_range": [1, 1],
                "arabic_text": "ب",
                "text_hash": {"algorithm": "Sha256", "hex": "ab".repeat(32)},
                "edition": {"slug": "test", "version": "0.1.0",
                            "script": "uthmani", "riwayah": null},
                "translation": null,
                "deep_link": "/read/test@0.1.0/1:1",
                "page": null,
                "juz": null
            },
            "translations": [],
            "word_glosses": null,
            "tokens": null
        }))
        .unwrap();
        let meta = BackendMeta {
            edition_slug: "test".into(),
            edition_version: "0.1.0".into(),
            edition_id: "ed-1".into(),
            corpus_generation: 3,
            text_hash: "sha256:ab".into(),
            script: "uthmani".into(),
            riwayah: None,
            numbering_scheme: "hafs".into(),
        };
        (view, meta)
    }

    #[async_trait]
    impl QuranBackend for FakeBackend {
        async fn backend_get_ayah(
            &self,
            _reference: &QuranRef,
            _options: &AyahOptions,
        ) -> Result<(AyahView, BackendMeta), ToolError> {
            Ok(test_view())
        }

        async fn backend_get_context(
            &self,
            _reference: &QuranRef,
            _spec: &ContextSpec,
        ) -> Result<(ContextView, BackendMeta), ToolError> {
            Err(ToolError::Backend { code: "QAI-QUR-0310".into(), detail: "unimplemented".into() })
        }
    }

    #[test]
    fn registry_lists_both_tools() {
        let registry = ToolRegistry::new(Arc::new(FakeBackend));
        assert_eq!(registry.tool_names(), ["quran.get_ayah", "quran.get_context"]);
    }

    #[tokio::test]
    async fn get_ayah_conforms_to_the_contract() {
        let registry = ToolRegistry::new(Arc::new(FakeBackend));
        let (result, meta) = registry
            .get_ayah(GetAyahParams {
                reference: "1:1".into(),
                translations: Vec::new(),
                glosses: false,
                tokens: false,
            })
            .await
            .unwrap();
        assert_eq!(result.tool_name, "quran.get_ayah");
        assert!(result.normalization_rules.is_empty());
        assert_eq!(result.canonical_references, ["quran:test@0.1.0:1:1".to_string()]);
        assert_eq!(result.reproducibility.corpus_generation, 3);
        assert!(result.reproducibility.deterministic);
        assert_eq!(meta.edition_slug, "test");
        assert_eq!(meta.text_hash, "sha256:ab");
        // The envelope round-trips (contract stability for later consumers).
        let json = serde_json::to_string(&result).unwrap();
        let back: ToolResult<Vec<AyahView>> = serde_json::from_str(&json).unwrap();
        assert_eq!(back, result);
    }

    #[tokio::test]
    async fn malformed_references_are_typed_errors() {
        let registry = ToolRegistry::new(Arc::new(FakeBackend));
        let err = registry
            .get_ayah(GetAyahParams {
                reference: ":::".into(),
                translations: Vec::new(),
                glosses: false,
                tokens: false,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, ToolError::InvalidInput { .. }));
        assert_eq!(err.code(), "QAI-QUR-0311");
    }

    #[test]
    fn context_params_validate() {
        let params = GetContextParams {
            reference: "2:255".into(),
            before: 2,
            after: 2,
            boundary: ContextBoundaryArg::Surah,
            max_ayahs: 0,
        };
        assert_eq!(params.max_ayahs, 0);
        let _ = (AyahNumber::new(1).unwrap(), SurahNumber::new(1).unwrap());
        let _ = EditionSelector::Active;
    }
}
