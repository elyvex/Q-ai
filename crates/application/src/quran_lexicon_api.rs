//! Lexicon service surface for the HTTP API (SC3/SC4, G-02/G-10, D-10/D-13).
//!
//! # application::quran_lexicon_api
//!
//! Thin dispatch over [`crate::quran_morphology::word_family`] and
//! [`crate::quran_counting`]'s root/lemma frequency. Routing lives here so
//! `server` and `cli` gain no new workspace edges (`arch-check`): both crates
//! already depend on `application`. No canonical access happens here beyond
//! what the underlying read-only lexicon services do — every relation and
//! every count carries dataset attribution by construction, and a missing
//! dataset stays a typed [`LexiconApiError::UnavailableDataset`] (never an
//! empty 200 envelope a caller could read as "no such family").

use std::sync::Arc;

use async_trait::async_trait;
use storage_sqlite::SqliteDatabase;

use crate::quran_counting::{self, CountingError, FrequencyReport, MultiAnalysisHandling};
use crate::quran_morphology::{self, FamilyMemberView, MorphologyToolError};

/// Diagnostic namespace for the lexicon route family.
pub const CODE_PREFIX: &str = "QAI-LEX";

/// Arguments for `quran.word_family` (SC3/G-02).
#[derive(Debug, Clone)]
pub struct FamilyArgs {
    /// Member kind (`token`).
    pub kind: String,
    /// Member id (`token:<surah>:<ayah>:<position>`).
    pub id: String,
}

impl FamilyArgs {
    /// Validate the caller's member selector; an empty or whitespace-only
    /// kind/id is a typed rejection before any storage read (T-03-18).
    ///
    /// # Errors
    ///
    /// Returns [`LexiconApiError::InvalidInput`] when either part is empty.
    pub fn new(kind: impl Into<String>, id: impl Into<String>) -> Result<Self, LexiconApiError> {
        let (kind, id) = (kind.into(), id.into());
        if kind.trim().is_empty() || id.trim().is_empty() {
            return Err(LexiconApiError::InvalidInput {
                detail: "family needs a non-empty kind and id".to_string(),
            });
        }
        Ok(Self { kind, id })
    }
}

/// Arguments for `quran.count.root_frequency` (SC4/G-10).
#[derive(Debug, Clone)]
pub struct RootFrequencyArgs {
    /// Root as stored by the active dataset (never re-normalized here).
    pub root: String,
    /// Counting profile recorded in the rules block (`L3.diacritics` default).
    pub profile: String,
    /// Multi-analysis handling (defaults to `single-source`).
    pub mode: MultiAnalysisHandling,
}

impl RootFrequencyArgs {
    /// Validate the root and default the profile/mode selectors.
    ///
    /// # Errors
    ///
    /// Returns [`LexiconApiError::InvalidInput`] for an empty root or an
    /// unknown profile/mode spelling.
    pub fn new(
        root: impl Into<String>,
        profile: Option<&str>,
        mode: Option<&str>,
    ) -> Result<Self, LexiconApiError> {
        let root = root.into();
        if root.trim().is_empty() {
            return Err(LexiconApiError::InvalidInput {
                detail: "root frequency needs a non-empty root".to_string(),
            });
        }
        Ok(Self {
            root,
            profile: profile.unwrap_or(DEFAULT_PROFILE).to_string(),
            mode: parse_multi_analysis_mode(mode)?,
        })
    }
}

/// Arguments for `quran.count.lemma_frequency` (SC4/G-10).
#[derive(Debug, Clone)]
pub struct LemmaFrequencyArgs {
    /// Lemma as stored by the active dataset.
    pub lemma: String,
    /// Counting profile recorded in the rules block (`L3.diacritics` default).
    pub profile: String,
    /// Multi-analysis handling (defaults to `single-source`).
    pub mode: MultiAnalysisHandling,
}

impl LemmaFrequencyArgs {
    /// Validate the lemma and default the profile/mode selectors.
    ///
    /// # Errors
    ///
    /// Returns [`LexiconApiError::InvalidInput`] for an empty lemma or an
    /// unknown profile/mode spelling.
    pub fn new(
        lemma: impl Into<String>,
        profile: Option<&str>,
        mode: Option<&str>,
    ) -> Result<Self, LexiconApiError> {
        let lemma = lemma.into();
        if lemma.trim().is_empty() {
            return Err(LexiconApiError::InvalidInput {
                detail: "lemma frequency needs a non-empty lemma".to_string(),
            });
        }
        Ok(Self {
            lemma,
            profile: profile.unwrap_or(DEFAULT_PROFILE).to_string(),
            mode: parse_multi_analysis_mode(mode)?,
        })
    }
}

/// Default counting profile for the lexicon count routes.
pub const DEFAULT_PROFILE: &str = "L3.diacritics";

/// Resolve the `mode` selector through the same parser the CLI `--mode` flag
/// uses, so an API count and a CLI count describe the same convention.
fn parse_multi_analysis_mode(raw: Option<&str>) -> Result<MultiAnalysisHandling, LexiconApiError> {
    crate::quran_cli::parse_multi_analysis_mode(raw.unwrap_or("single-source"))
        .map_err(|detail| LexiconApiError::InvalidInput { detail })
}

/// Lexicon route failures. Codes live in `QAI-LEX-*` (append-only numbering).
#[derive(Debug, Clone, thiserror::Error)]
pub enum LexiconApiError {
    /// Storage failure.
    #[error("lexicon storage failed: {0}")]
    Storage(String),
    /// Caller supplied an unusable value.
    #[error("{detail}")]
    InvalidInput {
        /// Operator-facing detail.
        detail: String,
    },
    /// The active dataset does not expose the requested capability.
    #[error("{detail}")]
    Unsupported {
        /// Operator-facing detail.
        detail: String,
    },
    /// No active dataset for the requested capability (AC-P2-01 fallback).
    #[error("no active morphology dataset for {capability}; import and activate one first")]
    UnavailableDataset {
        /// Capability needing a dataset.
        capability: String,
    },
}

impl storage::error::Diagnostic for LexiconApiError {
    fn code(&self) -> storage::error::DiagnosticCode {
        let number = match self {
            Self::Storage(_) => 1,
            Self::InvalidInput { .. } => 2,
            Self::Unsupported { .. } => 3,
            Self::UnavailableDataset { .. } => 4,
        };
        storage::error::DiagnosticCode::new(CODE_PREFIX, number)
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn remedy(&self) -> Option<String> {
        Some(match self {
            Self::Storage(_) => "Check the database and retry.".to_string(),
            Self::InvalidInput { .. } => {
                "Send a non-empty selector and a known profile/mode.".to_string()
            }
            Self::Unsupported { .. } => {
                "Use a dataset that supplies the requested capability; do not infer it.".to_string()
            }
            Self::UnavailableDataset { .. } => {
                "Run `qai quran morphology import`, then `activate`.".to_string()
            }
        })
    }

    fn next_command(&self) -> Option<String> {
        Some("qai quran morphology --help".to_string())
    }
}

impl From<MorphologyToolError> for LexiconApiError {
    fn from(error: MorphologyToolError) -> Self {
        match &error {
            MorphologyToolError::UnavailableDataset { capability } => {
                Self::UnavailableDataset { capability: capability.clone() }
            }
            MorphologyToolError::UnavailablePatternField { .. } => {
                Self::Unsupported { detail: error.to_string() }
            }
            MorphologyToolError::Storage(_) | MorphologyToolError::Morphology(_) => {
                Self::Storage(error.to_string())
            }
        }
    }
}

impl From<CountingError> for LexiconApiError {
    fn from(error: CountingError) -> Self {
        match &error {
            CountingError::UnavailableDataset { capability } => {
                Self::UnavailableDataset { capability: capability.clone() }
            }
            CountingError::Storage(_) => Self::Storage(error.to_string()),
            CountingError::UnknownProfile(_) | CountingError::EmptyTarget => {
                Self::InvalidInput { detail: error.to_string() }
            }
            CountingError::ProfileNotCountable(_) => {
                Self::Unsupported { detail: error.to_string() }
            }
        }
    }
}

/// Read-only lexicon backend. Implemented by [`LexiconApiService`]; faked in
/// `server` contract tests.
#[async_trait]
pub trait LexiconBackend: Send + Sync {
    /// `quran.word_family` (SC3/G-02): typed, explained, dataset-attributed
    /// relations for one lexicon member.
    async fn word_family(&self, args: FamilyArgs)
    -> Result<Vec<FamilyMemberView>, LexiconApiError>;
    /// `quran.count.root_frequency` (SC4/G-10): exact rules-blocked count.
    async fn root_frequency(
        &self,
        args: RootFrequencyArgs,
    ) -> Result<FrequencyReport, LexiconApiError>;
    /// `quran.count.lemma_frequency` (SC4/G-10): the lemma analogue.
    async fn lemma_frequency(
        &self,
        args: LemmaFrequencyArgs,
    ) -> Result<FrequencyReport, LexiconApiError>;
    /// `quran.count.frequency`: exact token frequency under a profile.
    async fn count_frequency(
        &self,
        target: &str,
        profile: &str,
    ) -> Result<FrequencyReport, LexiconApiError>;
    /// `quran.count.distribution`: frequency partitioned by surah.
    async fn count_distribution(
        &self,
        target: &str,
        profile: &str,
    ) -> Result<quran_counting::DistributionReport, LexiconApiError>;
    /// `quran.count.occurrences`: first/last occurrence + interval.
    async fn count_occurrences(
        &self,
        target: &str,
        profile: &str,
    ) -> Result<quran_counting::OccurrenceSpan, LexiconApiError>;
    /// `quran.count.hapax`: hapax legomena under a profile.
    async fn count_hapax(
        &self,
        profile: &str,
        limit: usize,
    ) -> Result<quran_counting::HapaxReport, LexiconApiError>;
    /// `quran.count.cooccurrence`: token-window co-occurrence.
    async fn count_cooccurrence(
        &self,
        target: &str,
        profile: &str,
        window: usize,
        limit: usize,
    ) -> Result<(quran_counting::CountingRules, Vec<quran_counting::CooccurrenceHit>), LexiconApiError>;
    /// `quran.count.collocation`: association measures.
    async fn count_collocation(
        &self,
        target: &str,
        profile: &str,
        window: usize,
        limit: usize,
    ) -> Result<(quran_counting::CountingRules, Vec<quran_counting::CollocationHit>), LexiconApiError>;
    /// `quran.count.numeric_report`: checksum + no-interpretation note.
    async fn count_numeric_report(
        &self,
        target: &str,
        profile: &str,
    ) -> Result<quran_counting::NumericReport, LexiconApiError>;
    /// `quran.count.missing_form`: proves a zero under stated rules.
    async fn count_missing_form(
        &self,
        target: &str,
        profile: &str,
    ) -> Result<quran_counting::MissingFormReport, LexiconApiError>;
    /// `quran.count.near_duplicates`: MinHash candidates + exact verify.
    async fn count_near_duplicates(
        &self,
        threshold: f64,
        limit: usize,
    ) -> Result<(quran_counting::CountingRules, Vec<quran_counting::NearDuplicateHit>), LexiconApiError>;
    /// `quran.count.interval`: interval analysis between occurrences.
    async fn count_interval(
        &self,
        target: &str,
        profile: &str,
    ) -> Result<quran_counting::OccurrenceSpan, LexiconApiError>;
    /// `quran.count.unusual_usage`: unusual-usage mining (lexicon-gated).
    async fn count_unusual_usage(
        &self,
        target: &str,
        profile: &str,
    ) -> Result<quran_counting::NumericReport, LexiconApiError>;
}

/// Live backend over a database file.
pub struct LexiconApiService {
    db: Arc<SqliteDatabase>,
}

impl LexiconApiService {
    /// Open the database at `db_path`.
    ///
    /// # Errors
    ///
    /// Returns a message when the database cannot be opened.
    pub async fn open(db_path: &str) -> Result<Self, String> {
        let db = SqliteDatabase::new(db_path, 4, true).await.map_err(|err| err.to_string())?;
        Ok(Self { db: Arc::new(db) })
    }
}

#[async_trait]
impl LexiconBackend for LexiconApiService {
    async fn word_family(
        &self,
        args: FamilyArgs,
    ) -> Result<Vec<FamilyMemberView>, LexiconApiError> {
        // The same service function the CLI's `qai quran family` calls, so the
        // API and CLI results cannot drift (G-02).
        let (_dataset, relations) =
            quran_morphology::word_family(&self.db, &args.kind, &args.id).await?;
        Ok(relations)
    }

    async fn root_frequency(
        &self,
        args: RootFrequencyArgs,
    ) -> Result<FrequencyReport, LexiconApiError> {
        Ok(quran_counting::root_frequency(&self.db, &args.root, &args.profile, args.mode).await?)
    }

    async fn lemma_frequency(
        &self,
        args: LemmaFrequencyArgs,
    ) -> Result<FrequencyReport, LexiconApiError> {
        Ok(quran_counting::lemma_frequency(&self.db, &args.lemma, &args.profile, args.mode).await?)
    }

    async fn count_frequency(
        &self,
        target: &str,
        profile: &str,
    ) -> Result<FrequencyReport, LexiconApiError> {
        Ok(quran_counting::frequency(&self.db, target, profile).await?)
    }

    async fn count_distribution(
        &self,
        target: &str,
        profile: &str,
    ) -> Result<quran_counting::DistributionReport, LexiconApiError> {
        Ok(quran_counting::distribution(&self.db, target, profile).await?)
    }

    async fn count_occurrences(
        &self,
        target: &str,
        profile: &str,
    ) -> Result<quran_counting::OccurrenceSpan, LexiconApiError> {
        Ok(quran_counting::first_last_occurrence(&self.db, target, profile).await?)
    }

    async fn count_hapax(
        &self,
        profile: &str,
        limit: usize,
    ) -> Result<quran_counting::HapaxReport, LexiconApiError> {
        Ok(quran_counting::hapax_search(&self.db, profile, limit).await?)
    }

    async fn count_cooccurrence(
        &self,
        target: &str,
        profile: &str,
        window: usize,
        limit: usize,
    ) -> Result<(quran_counting::CountingRules, Vec<quran_counting::CooccurrenceHit>), LexiconApiError> {
        Ok(quran_counting::cooccurrence(&self.db, target, profile, window, limit).await?)
    }

    async fn count_collocation(
        &self,
        target: &str,
        profile: &str,
        window: usize,
        limit: usize,
    ) -> Result<(quran_counting::CountingRules, Vec<quran_counting::CollocationHit>), LexiconApiError> {
        Ok(quran_counting::collocation(&self.db, target, profile, window, limit).await?)
    }

    async fn count_numeric_report(
        &self,
        target: &str,
        profile: &str,
    ) -> Result<quran_counting::NumericReport, LexiconApiError> {
        Ok(quran_counting::numeric_report(&self.db, target, profile).await?)
    }

    async fn count_missing_form(
        &self,
        target: &str,
        profile: &str,
    ) -> Result<quran_counting::MissingFormReport, LexiconApiError> {
        Ok(quran_counting::missing_expected_form(&self.db, target, profile).await?)
    }

    async fn count_near_duplicates(
        &self,
        threshold: f64,
        limit: usize,
    ) -> Result<(quran_counting::CountingRules, Vec<quran_counting::NearDuplicateHit>), LexiconApiError> {
        Ok(quran_counting::near_duplicate_passages(&self.db, threshold, limit).await?)
    }

    async fn count_interval(
        &self,
        target: &str,
        profile: &str,
    ) -> Result<quran_counting::OccurrenceSpan, LexiconApiError> {
        Ok(quran_counting::interval_analysis(&self.db, target, profile).await?)
    }

    async fn count_unusual_usage(
        &self,
        target: &str,
        profile: &str,
    ) -> Result<quran_counting::NumericReport, LexiconApiError> {
        Ok(quran_counting::unusual_usage(&self.db, target, profile).await?)
    }
}
