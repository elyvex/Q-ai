//! Citation resolver v1 + quotation verification + deep links (D1.9).
//!
//! # citations
//!
//! A citation names an ayah in an identified edition; the resolver checks the
//! location exists and the quoted text matches, returning a [`QuotationVerdict`].
//! `Mismatch` and `LocationNotFound` are hard failures on answer paths — the
//! mechanism Phase 9's verifier reuses. v1 verifies single ayahs; ranges and
//! other locators report `LocationNotFound` (documented limitation, safe
//! direction).
//!
//! The resolver reads through a [`CitationSource`] trait (implemented by the
//! application layer), so this crate never touches storage.

use std::sync::Arc;

use async_trait::async_trait;
use domain::{ContentHash, HashAlgorithm};
use quran_core::{AyahNumber, QuranRef, SurahNumber};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// What a citation points at (v1: Quran ayahs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CitationKind {
    /// A Quran ayah quotation.
    Quran,
}

/// A citation of one ayah in an identified edition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Citation {
    /// Citation id.
    pub id: String,
    /// Citation kind.
    pub kind: CitationKind,
    /// Fully-qualified canonical reference.
    pub canonical_reference: String,
    /// Quoted text under verification.
    pub quoted_text: String,
    /// Edition slug.
    pub edition_slug: String,
    /// Edition version.
    pub edition_version: String,
    /// Surah number.
    pub surah: SurahNumber,
    /// Ayah number.
    pub ayah: AyahNumber,
}

/// The quotation verification verdict (§35.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuotationVerdict {
    /// Byte-identical match.
    ExactMatch,
    /// Matches after whitespace normalization.
    MatchAfterWhitespaceNormalization,
    /// Matches after explicitly listed normalization rules.
    MatchAfterDeclaredNormalization {
        /// Rule ids applied.
        rules: Vec<String>,
    },
    /// Does not match; first difference and the canonical hash.
    Mismatch {
        /// First differing character index.
        first_difference_at: u32,
        /// SHA-256 hex of the canonical text.
        expected_hash: String,
    },
    /// The location does not resolve.
    LocationNotFound,
    /// The edition does not exist.
    EditionNotFound,
    /// Access denied (reserved; v1 never denies in single-user mode).
    AccessDenied,
}

/// A resolved citation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedCitation {
    /// Citation id.
    pub citation_id: String,
    /// Verdict.
    pub verdict: QuotationVerdict,
    /// Canonical text hash (`sha256:<hex>`).
    pub text_hash: Option<String>,
    /// Deep link for the reader.
    pub deep_link: Option<String>,
}

/// Citation errors (infrastructure failures; verdicts cover content outcomes).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CitationError {
    /// The canonical reference failed to parse.
    #[error("unparseable canonical reference `{reference}`")]
    InvalidReference {
        /// The offending reference.
        reference: String,
    },
    /// The backend failed.
    #[error("citation backend failed: {detail}")]
    Backend {
        /// What was wrong.
        detail: String,
    },
}

impl CitationError {
    /// Stable code string.
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidReference { .. } => "QAI-QUR-0321",
            Self::Backend { .. } => "QAI-QUR-0322",
        }
    }
}

/// The corpus surface the resolver needs. Implemented by `application`.
#[async_trait]
pub trait CitationSource: Send + Sync {
    /// Whether the edition exists.
    async fn edition_exists(&self, slug: &str, version: &str) -> Result<bool, CitationError>;
    /// Canonical ayah text, or `None` when the location is absent.
    async fn fetch_ayah_text(
        &self,
        slug: &str,
        version: &str,
        surah: u16,
        ayah: u32,
    ) -> Result<Option<String>, CitationError>;
}

/// Render a reader deep link.
pub fn deep_link(slug: &str, version: &str, surah: u16, ayah: u32) -> String {
    format!("/read/{slug}@{version}/{surah}:{ayah}")
}

/// Render the stored citation URN.
pub fn citation_urn(slug: &str, version: &str, surah: u16, ayah: u32) -> String {
    format!("qai://quran/{slug}@{version}/{surah}:{ayah}")
}

fn sha256_hex(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn whitespace_normalized(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn first_difference(left: &str, right: &str) -> u32 {
    left.chars()
        .zip(right.chars())
        .position(|(a, b)| a != b)
        .map(|index| index as u32)
        .unwrap_or_else(|| left.chars().count().min(right.chars().count()) as u32)
}

/// Citation resolver v1.
pub struct CitationResolver {
    source: Arc<dyn CitationSource>,
}

impl CitationResolver {
    /// Wrap a source.
    pub fn new(source: Arc<dyn CitationSource>) -> Self {
        Self { source }
    }

    /// Resolve a citation: source exists, location resolves, quotation matches,
    /// edition exists, hash and version available (§35.3).
    pub async fn resolve(&self, citation: &Citation) -> Result<ResolvedCitation, CitationError> {
        let parsed = quran_core::parse(&citation.canonical_reference).map_err(|_| {
            CitationError::InvalidReference { reference: citation.canonical_reference.clone() }
        })?;
        // v1 verifies single ayahs; anything else is a hard LocationNotFound.
        let (surah, ayah) = match parsed {
            QuranRef::Ayah { surah, ayah, .. } => (surah.get(), ayah.get()),
            _ => {
                return Ok(ResolvedCitation {
                    citation_id: citation.id.clone(),
                    verdict: QuotationVerdict::LocationNotFound,
                    text_hash: None,
                    deep_link: None,
                });
            }
        };
        if !self.source.edition_exists(&citation.edition_slug, &citation.edition_version).await? {
            return Ok(ResolvedCitation {
                citation_id: citation.id.clone(),
                verdict: QuotationVerdict::EditionNotFound,
                text_hash: None,
                deep_link: None,
            });
        }
        let canonical = self
            .source
            .fetch_ayah_text(&citation.edition_slug, &citation.edition_version, surah, ayah)
            .await?;
        let Some(canonical) = canonical else {
            return Ok(ResolvedCitation {
                citation_id: citation.id.clone(),
                verdict: QuotationVerdict::LocationNotFound,
                text_hash: None,
                deep_link: None,
            });
        };
        Ok(ResolvedCitation {
            citation_id: citation.id.clone(),
            verdict: Self::verdict(&canonical, &citation.quoted_text),
            text_hash: Some(format!("sha256:{}", sha256_hex(&canonical))),
            deep_link: Some(deep_link(
                &citation.edition_slug,
                &citation.edition_version,
                surah,
                ayah,
            )),
        })
    }

    /// Verify a quoted string against a citation's location.
    pub async fn verify_quotation(
        &self,
        citation: &Citation,
        quoted: &str,
    ) -> Result<QuotationVerdict, CitationError> {
        let mut probe = citation.clone();
        probe.quoted_text = quoted.to_string();
        Ok(self.resolve(&probe).await?.verdict)
    }

    /// Re-verify a stored citation against the current corpus text.
    ///
    /// Compares the live text hash against the stored hash — no quoted text
    /// needed. This is the later-reverification half of AC-P1-21.
    pub async fn resolve_stored(
        &self,
        stored: &StoredCitation,
    ) -> Result<ResolvedCitation, CitationError> {
        let parsed = quran_core::parse(&stored.canonical_reference).map_err(|_| {
            CitationError::InvalidReference {
                reference: stored.canonical_reference.clone(),
            }
        })?;
        if !matches!(parsed, QuranRef::Ayah { .. }) {
            return Ok(ResolvedCitation {
                citation_id: stored.id.clone(),
                verdict: QuotationVerdict::LocationNotFound,
                text_hash: None,
                deep_link: None,
            });
        }
        if !self.source.edition_exists(&stored.edition_slug, &stored.edition_version).await? {
            return Ok(ResolvedCitation {
                citation_id: stored.id.clone(),
                verdict: QuotationVerdict::EditionNotFound,
                text_hash: None,
                deep_link: None,
            });
        }
        let current = self
            .source
            .fetch_ayah_text(
                &stored.edition_slug,
                &stored.edition_version,
                stored.surah,
                stored.ayah,
            )
            .await?;
        let Some(current) = current else {
            return Ok(ResolvedCitation {
                citation_id: stored.id.clone(),
                verdict: QuotationVerdict::LocationNotFound,
                text_hash: None,
                deep_link: None,
            });
        };
        let current_hash = format!("sha256:{}", sha256_hex(&current));
        let verdict = if current_hash == stored.quoted_text_hash {
            QuotationVerdict::ExactMatch
        } else {
            QuotationVerdict::Mismatch {
                first_difference_at: 0,
                expected_hash: sha256_hex(&current),
            }
        };
        Ok(ResolvedCitation {
            citation_id: stored.id.clone(),
            verdict,
            text_hash: Some(current_hash),
            deep_link: Some(deep_link(
                &stored.edition_slug,
                &stored.edition_version,
                stored.surah,
                stored.ayah,
            )),
        })
    }

    fn verdict(canonical: &str, quoted: &str) -> QuotationVerdict {
        if quoted == canonical {
            QuotationVerdict::ExactMatch
        } else if whitespace_normalized(quoted) == whitespace_normalized(canonical) {
            QuotationVerdict::MatchAfterWhitespaceNormalization
        } else {
            QuotationVerdict::Mismatch {
                first_difference_at: first_difference(canonical, quoted),
                expected_hash: sha256_hex(canonical),
            }
        }
    }
}

/// A persisted citation for later re-verification (§12.1, §21.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredCitation {
    /// Citation id.
    pub id: String,
    /// Fully-qualified canonical reference.
    pub canonical_reference: String,
    /// Stored content hash (`sha256:<hex>`).
    pub quoted_text_hash: String,
    /// Edition slug.
    pub edition_slug: String,
    /// Edition version.
    pub edition_version: String,
    /// Surah number.
    pub surah: u16,
    /// Ayah number.
    pub ayah: u32,
}

/// The content hash helper shared with stored citations.
pub fn content_hash_of(text: &str) -> ContentHash {
    ContentHash { algorithm: HashAlgorithm::Sha256, hex: sha256_hex(text) }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeSource;

    #[async_trait]
    impl CitationSource for FakeSource {
        async fn edition_exists(&self, slug: &str, _version: &str) -> Result<bool, CitationError> {
            Ok(slug == "test")
        }

        async fn fetch_ayah_text(
            &self,
            _slug: &str,
            _version: &str,
            surah: u16,
            ayah: u32,
        ) -> Result<Option<String>, CitationError> {
            Ok((surah == 1 && ayah == 1).then(|| "ب ت".to_string()))
        }
    }

    fn citation(quoted: &str) -> Citation {
        Citation {
            id: "cit-1".into(),
            kind: CitationKind::Quran,
            canonical_reference: "quran:test@0.1.0:1:1".into(),
            quoted_text: quoted.into(),
            edition_slug: "test".into(),
            edition_version: "0.1.0".into(),
            surah: SurahNumber::new(1).unwrap(),
            ayah: AyahNumber::new(1).unwrap(),
        }
    }

    fn resolver() -> CitationResolver {
        CitationResolver::new(Arc::new(FakeSource))
    }

    #[tokio::test]
    async fn exact_match_resolves_with_hash_and_link() {
        let resolved = resolver().resolve(&citation("ب ت")).await.unwrap();
        assert_eq!(resolved.verdict, QuotationVerdict::ExactMatch);
        assert!(resolved.text_hash.as_deref().unwrap_or("").starts_with("sha256:"));
        assert_eq!(resolved.deep_link.as_deref(), Some("/read/test@0.1.0/1:1"));
    }

    #[tokio::test]
    async fn whitespace_variants_and_mismatches() {
        let resolved = resolver().resolve(&citation("ب  ت")).await.unwrap();
        assert_eq!(resolved.verdict, QuotationVerdict::MatchAfterWhitespaceNormalization);
        let resolved = resolver().resolve(&citation("ب ث")).await.unwrap();
        assert!(matches!(
            resolved.verdict,
            QuotationVerdict::Mismatch { first_difference_at: 2, .. }
        ));
    }

    #[tokio::test]
    async fn missing_edition_location_and_unparseable() {
        let mut missing_edition = citation("ب ت");
        missing_edition.edition_slug = "nope".into();
        let resolved = resolver().resolve(&missing_edition).await.unwrap();
        assert_eq!(resolved.verdict, QuotationVerdict::EditionNotFound);

        let mut missing_ayah = citation("ب ت");
        missing_ayah.canonical_reference = "quran:test@0.1.0:9:9".into();
        let resolved = resolver().resolve(&missing_ayah).await.unwrap();
        assert_eq!(resolved.verdict, QuotationVerdict::LocationNotFound);

        let mut bad_ref = citation("ب ت");
        bad_ref.canonical_reference = ":::".into();
        let err = resolver().resolve(&bad_ref).await.unwrap_err();
        assert_eq!(err.code(), "QAI-QUR-0321");

        // Non-ayah locators are hard LocationNotFound in v1.
        let mut range = citation("ب ت");
        range.canonical_reference = "quran:test@0.1.0:1:1-1:2".into();
        let resolved = resolver().resolve(&range).await.unwrap();
        assert_eq!(resolved.verdict, QuotationVerdict::LocationNotFound);
    }

    #[test]
    fn links_render() {
        assert_eq!(deep_link("s", "1.0.0", 2, 255), "/read/s@1.0.0/2:255");
        assert_eq!(citation_urn("s", "1.0.0", 2, 255), "qai://quran/s@1.0.0/2:255");
    }
}
