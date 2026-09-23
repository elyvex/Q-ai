//! Phase 2 — counting & discovery tools (Sprint 2.6a, P2-T94…T103, D2.9).
//!
//! Anti-numerology discipline (ADR-0211): every numeric output carries a
//! complete [`CountingRules`] block; aggregation is exact SQL over stored
//! derived forms (never FTS frequencies); `numeric_report` contains no
//! interpretive commentary; `interval_analysis` and `missing_expected_form`
//! emit fixed disclaimers verbatim. Counts are rule-relative: `hapax`
//! states its profile prominently.
//!
//! Morphology-gated targets (roots, lemmas, multi-analysis modes) return
//! typed [`CountingError::UnavailableDataset`] naming the missing capability
//! (AC-P2-01 fallback) — never guessed data.

use std::collections::{BTreeMap, BTreeSet};

use storage::Database as _;
use storage::error::StorageError;
use storage::quran::FormColumn;
use storage_sqlite::SqliteDatabase;

/// Error namespace for counting tools.
pub const CODE_PREFIX: &str = "QAI-CNT";

/// How competing analyses are handled in a count (ADR-0211).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MultiAnalysisHandling {
    /// Single-source forms only (current capability: stored derived forms).
    SingleSource,
    /// All dataset analyses, one vote each (needs an active lexicon).
    AllAnalyses,
    /// One vote per token regardless of analysis count (needs lexicon).
    OneVotePerToken,
}

/// Complete, reproducible counting rules (mandatory on every numeric output).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CountingRules {
    /// Normalization profile id (e.g. `L3.diacritics`).
    pub profile: String,
    /// Profile ladder version.
    pub profile_version: String,
    /// Dataset slugs consulted (`["stored-forms"]` until a lexicon activates).
    pub datasets: Vec<String>,
    /// Multi-analysis handling mode.
    pub multi_analysis_handling: MultiAnalysisHandling,
    /// Window definition for co-occurrence (`token`/`ayah` + radius).
    pub window: Option<String>,
    /// Exclusions applied (e.g. stop lists — empty in v1).
    pub exclusions: Vec<String>,
}

impl CountingRules {
    /// Canonical serialization (checksum input; field order fixed by struct).
    #[must_use]
    pub fn canonical_json(&self) -> String {
        serde_json::to_string(self).expect("CountingRules serializes")
    }
}

/// Counting failures. Codes live in `QAI-CNT-*` (append-only numbering).
#[derive(Debug, Clone, thiserror::Error)]
pub enum CountingError {
    /// Storage failure.
    #[error("counting storage failed: {0}")]
    Storage(String),
    /// Unknown normalization profile.
    #[error("unknown counting profile: {0}")]
    UnknownProfile(String),
    /// Profile has no countable token-form column (L0/L1/L6/L8).
    #[error("profile {0} has no countable token column; use L2-L5 (L7 heuristic)")]
    ProfileNotCountable(String),
    /// Empty target.
    #[error("count target must not be empty")]
    EmptyTarget,
    /// Morphology-gated capability (no active lexicon dataset).
    #[error("dataset unavailable for {capability}: import and activate a morphology dataset first")]
    UnavailableDataset {
        /// Capability needing a dataset (e.g. `root frequency`).
        capability: String,
    },
}

impl CountingError {
    fn storage(error: StorageError) -> Self {
        Self::Storage(error.to_string())
    }
}

impl storage::error::Diagnostic for CountingError {
    fn code(&self) -> storage::error::DiagnosticCode {
        let number = match self {
            Self::Storage(_) => 1,
            Self::UnknownProfile(_) => 2,
            Self::ProfileNotCountable(_) => 3,
            Self::EmptyTarget => 4,
            Self::UnavailableDataset { .. } => 5,
        };
        storage::error::DiagnosticCode::new(CODE_PREFIX, number)
    }

    fn summary(&self) -> String {
        self.to_string()
    }

    fn remedy(&self) -> Option<String> {
        Some(match self {
            Self::Storage(_) => "Check the database and retry.".to_string(),
            Self::UnknownProfile(_) => "Use a registered profile (L2/L3/L4/L5/L7).".to_string(),
            Self::ProfileNotCountable(_) => {
                "Count under L3.diacritics (or L2/L4/L5) instead.".to_string()
            }
            Self::EmptyTarget => "Pass a non-empty target string.".to_string(),
            Self::UnavailableDataset { .. } => {
                "Run `qai quran morphology import`, then `activate`.".to_string()
            }
        })
    }

    fn next_command(&self) -> Option<String> {
        Some("qai doctor --quran".to_string())
    }
}

/// Project a token-form row onto a countable column.
fn form_value(form: &storage::quran::TokenFormRow, column: FormColumn) -> &str {
    match column {
        FormColumn::Simple => &form.simple,
        FormColumn::Bare => &form.bare,
        FormColumn::HamzaFolded => &form.hamza_folded,
        FormColumn::Folded => &form.folded,
        FormColumn::AffixStripped => &form.affix_stripped,
    }
}

/// Fixed disclaimer for interval analysis (verbatim, snapshot-tested).
pub const INTERVAL_DISCLAIMER: &str =
    "Intervals describe ayah positions only; no chronological or interpretive claim is made.";
/// Fixed disclaimer for missing-expected-form reports (verbatim).
pub const MISSING_FORM_DISCLAIMER: &str = "Absence under these counting rules is not evidence of absence under other rules; see the CountingRules block.";
/// Mandatory no-interpretation note on numeric reports.
pub const NO_INTERPRETATION_NOTE: &str =
    "Numeric output only; no interpretive commentary is provided.";

fn countable_column(profile: &str) -> Result<FormColumn, CountingError> {
    match profile {
        "L2.marks" => Ok(FormColumn::Simple),
        "L3.diacritics" => Ok(FormColumn::Bare),
        "L4.hamza" => Ok(FormColumn::HamzaFolded),
        "L5.codepoints" => Ok(FormColumn::Folded),
        "L7.affix" => Ok(FormColumn::AffixStripped),
        other => {
            if quran_normalization::ProfileId::parse(other).is_ok() {
                Err(CountingError::ProfileNotCountable(other.to_string()))
            } else {
                Err(CountingError::UnknownProfile(other.to_string()))
            }
        }
    }
}

/// Resolve the active edition id.
async fn active_edition_id(db: &SqliteDatabase) -> Result<String, CountingError> {
    let mut uow = db.write().await.map_err(CountingError::storage)?;
    let active = uow.quran().get_active().await.map_err(CountingError::storage)?;
    uow.rollback().await.map_err(CountingError::storage)?;
    active.map(|a| a.edition_id).ok_or_else(|| CountingError::Storage("no active edition".into()))
}

/// Normalize a target through a profile pipeline (shared code path, R6).
fn normalize_target(profile: &str, target: &str) -> Result<String, CountingError> {
    if target.trim().is_empty() {
        return Err(CountingError::EmptyTarget);
    }
    let registry = crate::quran_normalize::builtin_registry();
    let id = quran_normalization::ProfileId::parse(profile)
        .map_err(|_| CountingError::UnknownProfile(profile.to_string()))?;
    let version = registry
        .latest(id)
        .map_err(|_| CountingError::UnknownProfile(profile.to_string()))?
        .version;
    let pipe = quran_normalization::NormalizationPipeline::for_profile(&registry, id, version)
        .map_err(|_| CountingError::UnknownProfile(profile.to_string()))?;
    Ok(pipe.apply(target).0.text().to_string())
}

fn rules_for(profile: &str, window: Option<String>) -> Result<CountingRules, CountingError> {
    let registry = crate::quran_normalize::builtin_registry();
    let id = quran_normalization::ProfileId::parse(profile)
        .map_err(|_| CountingError::UnknownProfile(profile.to_string()))?;
    let version = registry
        .latest(id)
        .map_err(|_| CountingError::UnknownProfile(profile.to_string()))?
        .version;
    Ok(CountingRules {
        profile: profile.to_string(),
        profile_version: version.to_string(),
        datasets: vec!["stored-forms".to_string()],
        multi_analysis_handling: MultiAnalysisHandling::SingleSource,
        window,
        exclusions: Vec::new(),
    })
}

fn checksum(canonical: &str) -> String {
    format!("sha256:{}", quran_corpus::sha256_hex(canonical.as_bytes()))
}

/// Exact token frequency (P2-T95): SQL `COUNT(*)` over the profile column.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FrequencyReport {
    /// Normalized target actually counted.
    pub target: String,
    /// Mandatory rules block.
    pub rules: CountingRules,
    /// Exact total.
    pub count: u64,
    /// Per-surah breakdown (surah → count), sparse.
    pub by_surah: BTreeMap<i64, u64>,
    /// Checksum over the canonical report (reproducibility).
    pub checksum: String,
}

/// `quran.frequency` — exact aggregation for one surface target.
pub async fn frequency(
    db: &SqliteDatabase,
    target: &str,
    profile: &str,
) -> Result<FrequencyReport, CountingError> {
    let column = countable_column(profile)?;
    let rules = rules_for(profile, None)?;
    let normalized = normalize_target(profile, target)?;
    let edition_id = active_edition_id(db).await?;
    let (count, by_surah) = {
        let mut uow = db.write().await.map_err(CountingError::storage)?;
        let count = uow
            .quran()
            .count_tokens_matching_form(&edition_id, column, &normalized)
            .await
            .map_err(CountingError::storage)?;
        let parts = uow
            .quran()
            .count_tokens_matching_form_by_surah(&edition_id, column, &normalized)
            .await
            .map_err(CountingError::storage)?;
        uow.rollback().await.map_err(CountingError::storage)?;
        (count, parts)
    };
    let by_surah: BTreeMap<i64, u64> =
        by_surah.into_iter().map(|(s, c)| (s, c.max(0) as u64)).collect();
    let canonical =
        format!("{}|{}|{}|{}", normalized, rules.canonical_json(), count.max(0), by_surah.len());
    Ok(FrequencyReport {
        target: normalized,
        rules,
        count: count.max(0) as u64,
        by_surah,
        checksum: checksum(&canonical),
    })
}

/// Partition distribution with provenance (P2-T96).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DistributionReport {
    /// Target frequency report (rules included).
    pub frequency: FrequencyReport,
    /// Partition provenance (what each partition key means).
    pub partition_provenance: String,
    /// Single-source warning (morphology pending: no cross-dataset view).
    pub warnings: Vec<String>,
}

/// `quran.distribution` — frequency partitioned by surah.
pub async fn distribution(
    db: &SqliteDatabase,
    target: &str,
    profile: &str,
) -> Result<DistributionReport, CountingError> {
    let frequency = frequency(db, target, profile).await?;
    Ok(DistributionReport {
        frequency,
        partition_provenance: "partitions keyed by surah number (canonical structure)".to_string(),
        warnings: vec![
            "single-source counts over stored derived forms; cross-dataset distributions need an active lexicon".to_string(),
        ],
    })
}

/// Hapax legomena under a profile (P2-T101): forms occurring exactly once.
/// The profile is part of every entry point (rule-relativity, AC-P2-30).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HapaxReport {
    /// Profile the counts are relative to (prominent by contract).
    pub profile: String,
    /// Rules block.
    pub rules: CountingRules,
    /// (form, 1) pairs, ordered by form.
    pub hapax: Vec<(String, u64)>,
}

/// `quran.hapax_search`.
pub async fn hapax_search(
    db: &SqliteDatabase,
    profile: &str,
    limit: usize,
) -> Result<HapaxReport, CountingError> {
    let column = countable_column(profile)?;
    let rules = rules_for(profile, None)?;
    let edition_id = active_edition_id(db).await?;
    let mut uow = db.write().await.map_err(CountingError::storage)?;
    let distinct = uow
        .quran()
        .list_distinct_forms(&edition_id, column)
        .await
        .map_err(CountingError::storage)?;
    uow.rollback().await.map_err(CountingError::storage)?;
    let mut hapax: Vec<(String, u64)> =
        distinct.into_iter().filter(|(_, c)| *c == 1).map(|(f, _)| (f, 1)).collect();
    hapax.sort();
    hapax.truncate(limit.max(1));
    Ok(HapaxReport { profile: profile.to_string(), rules, hapax })
}

/// Numeric report: checksum + rules + no interpretation (P2-T100).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NumericReport {
    /// Frequency payload.
    pub frequency: FrequencyReport,
    /// Mandatory no-interpretation note (verbatim).
    pub note: String,
}

/// `quran.numeric_report`.
pub async fn numeric_report(
    db: &SqliteDatabase,
    target: &str,
    profile: &str,
) -> Result<NumericReport, CountingError> {
    Ok(NumericReport {
        frequency: frequency(db, target, profile).await?,
        note: NO_INTERPRETATION_NOTE.to_string(),
    })
}

/// First/last occurrence + interval summary (P2-T99).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OccurrenceSpan {
    /// First (surah, ayah) in canonical order, if any.
    pub first: Option<(i64, i64)>,
    /// Last (surah, ayah) in canonical order, if any.
    pub last: Option<(i64, i64)>,
    /// Ayah-span between first and last (inclusive count), if any.
    pub ayah_span: Option<u64>,
    /// Rules block.
    pub rules: CountingRules,
    /// Mandatory disclaimer (verbatim).
    pub disclaimer: String,
}

/// First/last occurrence via ordered token-form scan.
pub async fn first_last_occurrence(
    db: &SqliteDatabase,
    target: &str,
    profile: &str,
) -> Result<OccurrenceSpan, CountingError> {
    let column = countable_column(profile)?;
    let rules = rules_for(profile, None)?;
    let normalized = normalize_target(profile, target)?;
    let edition_id = active_edition_id(db).await?;
    let mut uow = db.write().await.map_err(CountingError::storage)?;
    let forms =
        uow.quran().list_all_token_forms(&edition_id).await.map_err(CountingError::storage)?;
    uow.rollback().await.map_err(CountingError::storage)?;
    let mut first: Option<(i64, i64)> = None;
    let mut last: Option<(i64, i64)> = None;
    for form in &forms {
        if form_value(form, column) == normalized {
            let loc = (form.surah, form.ayah);
            if first.is_none() {
                first = Some(loc);
            }
            last = Some(loc);
        }
    }
    // Ayah span: count distinct ayahs between first and last inclusive.
    let ayah_span = match (first, last) {
        (Some(_), Some(_)) => {
            let ayahs: BTreeSet<(i64, i64)> = forms
                .iter()
                .filter(|f| {
                    let loc = (f.surah, f.ayah);
                    Some(loc) >= first && Some(loc) <= last
                })
                .map(|f| (f.surah, f.ayah))
                .collect();
            Some(ayahs.len() as u64)
        }
        _ => None,
    };
    Ok(OccurrenceSpan {
        first,
        last,
        ayah_span,
        rules,
        disclaimer: INTERVAL_DISCLAIMER.to_string(),
    })
}

/// Alias with the interval disclaimer foregrounded (same computation).
pub async fn interval_analysis(
    db: &SqliteDatabase,
    target: &str,
    profile: &str,
) -> Result<OccurrenceSpan, CountingError> {
    first_last_occurrence(db, target, profile).await
}

/// Missing-expected-form report (P2-T103): zero counts with disclaimer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MissingFormReport {
    /// Normalized target with zero matches.
    pub target: String,
    /// Rules block proving the zero.
    pub rules: CountingRules,
    /// Always 0 (structural: only constructed on zero).
    pub count: u64,
    /// Mandatory disclaimer (verbatim).
    pub disclaimer: String,
}

/// `quran.missing_expected_form`: proves a zero under stated rules.
pub async fn missing_expected_form(
    db: &SqliteDatabase,
    target: &str,
    profile: &str,
) -> Result<MissingFormReport, CountingError> {
    let report = frequency(db, target, profile).await?;
    if report.count != 0 {
        return Err(CountingError::Storage(format!(
            "target occurs {} time(s); missing_expected_form needs a zero",
            report.count
        )));
    }
    Ok(MissingFormReport {
        target: report.target,
        rules: report.rules,
        count: 0,
        disclaimer: MISSING_FORM_DISCLAIMER.to_string(),
    })
}

/// Co-occurrence within token/ayah windows (P2-T97).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CooccurrenceHit {
    /// Co-occurring form.
    pub form: String,
    /// Co-occurrence count (windows containing both).
    pub count: u64,
    /// True when the pair spans an ayah boundary somewhere.
    pub spans_ayah_boundary: bool,
}

/// `quran.cooccurrence`: token-window co-occurrence over bare-form order.
pub async fn cooccurrence(
    db: &SqliteDatabase,
    target: &str,
    profile: &str,
    window_tokens: usize,
    limit: usize,
) -> Result<(CountingRules, Vec<CooccurrenceHit>), CountingError> {
    let column = countable_column(profile)?;
    let rules = rules_for(profile, Some(format!("token:{}", window_tokens.max(1))))?;
    let normalized = normalize_target(profile, target)?;
    let edition_id = active_edition_id(db).await?;
    let mut uow = db.write().await.map_err(CountingError::storage)?;
    let forms =
        uow.quran().list_all_token_forms(&edition_id).await.map_err(CountingError::storage)?;
    uow.rollback().await.map_err(CountingError::storage)?;
    let stream: Vec<(String, i64, i64)> =
        forms.iter().map(|f| (form_value(f, column).to_string(), f.surah, f.ayah)).collect();
    let radius = window_tokens.max(1);
    let mut counts: BTreeMap<String, (u64, bool)> = BTreeMap::new();
    for (i, (form, _, _)) in stream.iter().enumerate() {
        if form != &normalized {
            continue;
        }
        let lo = i.saturating_sub(radius);
        let hi = (i + radius + 1).min(stream.len());
        for (form_j, surah_j, ayah_j) in &stream[lo..hi] {
            if form_j == &normalized {
                continue;
            }
            let entry = counts.entry(form_j.clone()).or_insert((0, false));
            entry.0 += 1;
            let (_, surah_i, ayah_i) = stream[i];
            if (*surah_j, *ayah_j) != (surah_i, ayah_i) {
                entry.1 = true;
            }
        }
    }
    let mut hits: Vec<CooccurrenceHit> = counts
        .into_iter()
        .map(|(form, (count, spans_ayah_boundary))| CooccurrenceHit {
            form,
            count,
            spans_ayah_boundary,
        })
        .collect();
    hits.sort_by(|a, b| b.count.cmp(&a.count).then(a.form.cmp(&b.form)));
    hits.truncate(limit.max(1));
    Ok((rules, hits))
}

/// Collocation association scores (P2-T98): PMI + LLR + t-score.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CollocationHit {
    /// Candidate form.
    pub form: String,
    /// Joint count within the window.
    pub observed: u64,
    /// Expected joint count under independence.
    pub expected: f64,
    /// Pointwise mutual information (bits).
    pub pmi: f64,
    /// Dunning log-likelihood ratio (G²).
    pub llr: f64,
    /// t-score.
    pub t_score: f64,
}

/// Minimum joint count for a collocation candidate (floor, plan §8).
pub const COLLOCATION_MIN_COUNT: u64 = 3;

/// `quran.collocation`: association measures over token windows.
pub async fn collocation(
    db: &SqliteDatabase,
    target: &str,
    profile: &str,
    window_tokens: usize,
    limit: usize,
) -> Result<(CountingRules, Vec<CollocationHit>), CountingError> {
    let (rules, co) = cooccurrence(db, target, profile, window_tokens, usize::MAX).await?;
    let column = countable_column(profile)?;
    let edition_id = active_edition_id(db).await?;
    let mut uow = db.write().await.map_err(CountingError::storage)?;
    let forms =
        uow.quran().list_all_token_forms(&edition_id).await.map_err(CountingError::storage)?;
    uow.rollback().await.map_err(CountingError::storage)?;
    let total = forms.len() as f64;
    if total == 0.0 {
        return Ok((rules, Vec::new()));
    }
    let mut unigram: BTreeMap<String, f64> = BTreeMap::new();
    for form in &forms {
        *unigram.entry(form_value(form, column).to_string()).or_insert(0.0) += 1.0;
    }
    let window_mass = co.iter().map(|h| h.count as f64).sum::<f64>();
    let mut hits = Vec::new();
    for hit in &co {
        if hit.count < COLLOCATION_MIN_COUNT {
            continue;
        }
        let observed = hit.count as f64;
        let candidate_count = unigram.get(&hit.form).copied().unwrap_or(0.0);
        // Expected joint windows under independence, scaled by window mass:
        // E = window_mass * P(candidate) with P from unigram rates.
        let expected = (window_mass / total.max(1.0) * candidate_count).max(1e-9);
        let pmi = (observed / expected).log2();
        let t_score = (observed - expected) / observed.sqrt().max(1e-9);
        // Dunning G² over {candidate, ¬candidate} × {in-window, out}.
        let o11 = observed;
        let o12 = candidate_count - observed;
        let o21 = window_mass - observed;
        let o22 = total - candidate_count - o21;
        let llr = 2.0
            * (log_term(o11, expected)
                + log_term(o12, (o12 + o22).max(1e-9) * candidate_count / total.max(1e-9))
                + log_term(
                    o21,
                    (o21 + o22).max(1e-9) * (total - candidate_count) / total.max(1e-9),
                )
                + log_term(
                    o22,
                    (o12 + o22).max(1e-9) * (total - candidate_count) / total.max(1e-9),
                ));
        hits.push(CollocationHit {
            form: hit.form.clone(),
            observed: hit.count,
            expected,
            pmi,
            llr: llr.max(0.0),
            t_score,
        });
    }
    hits.sort_by(|a, b| {
        b.llr.partial_cmp(&a.llr).unwrap_or(std::cmp::Ordering::Equal).then(a.form.cmp(&b.form))
    });
    hits.truncate(limit.max(1));
    Ok((rules, hits))
}

fn log_term(observed: f64, expected: f64) -> f64 {
    if observed <= 0.0 || expected <= 0.0 { 0.0 } else { observed * (observed / expected).ln() }
}

/// Near-duplicate passage pair (P2-T102): MinHash candidate + exact verify.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NearDuplicateHit {
    /// First ayah (surah, ayah).
    pub first: (i64, i64),
    /// Second ayah (surah, ayah).
    pub second: (i64, i64),
    /// Exact Jaccard over char-5-shingle sets.
    pub jaccard: f64,
    /// Shared shingle count.
    pub shared: usize,
}

/// MinHash signatures (64 deterministic hashes: sha256(seed || shingle)).
fn minhash_signature(shingles: &BTreeSet<String>) -> [u64; 64] {
    let mut sig = [u64::MAX; 64];
    for shingle in shingles {
        for (i, slot) in sig.iter_mut().enumerate() {
            let digest = quran_corpus::sha256_hex(format!("{i}:{shingle}").as_bytes());
            let value = u64::from_str_radix(&digest[..16], 16).unwrap_or(u64::MAX);
            *slot = (*slot).min(value);
        }
    }
    sig
}

fn char_shingles(text: &str, k: usize) -> BTreeSet<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < k {
        return BTreeSet::from([text.to_string()]);
    }
    chars.windows(k).map(|w| w.iter().collect()).collect()
}

/// `quran.near_duplicate_passages`: MinHash candidates, exact Jaccard verify.
pub async fn near_duplicate_passages(
    db: &SqliteDatabase,
    threshold: f64,
    limit: usize,
) -> Result<(CountingRules, Vec<NearDuplicateHit>), CountingError> {
    let rules = rules_for("L6.skeleton", None)
        .map_err(|_| CountingError::ProfileNotCountable("L6.skeleton".to_string()))?;
    let _ = &rules;
    // L6 counting has no token column; operate on stored skeletons directly
    // (ayah-level rows only) with rules naming the skeleton derivation.
    let rules = CountingRules {
        profile: "L6.skeleton".to_string(),
        profile_version: rules.profile_version.clone(),
        datasets: vec!["stored-skeletons".to_string()],
        multi_analysis_handling: MultiAnalysisHandling::SingleSource,
        window: None,
        exclusions: Vec::new(),
    };
    let edition_id = active_edition_id(db).await?;
    let surahs = {
        let mut uow = db.write().await.map_err(CountingError::storage)?;
        let surahs = uow.quran().list_surahs(&edition_id).await.map_err(CountingError::storage)?;
        uow.rollback().await.map_err(CountingError::storage)?;
        surahs
    };
    let mut ayahs: Vec<((i64, i64), String)> = Vec::new();
    for surah in &surahs {
        let mut uow = db.write().await.map_err(CountingError::storage)?;
        let skeletons = uow
            .quran()
            .list_skeletons(&edition_id, surah.number)
            .await
            .map_err(CountingError::storage)?;
        uow.rollback().await.map_err(CountingError::storage)?;
        for row in skeletons {
            if row.ayah_start == row.ayah_end && !row.skeleton.is_empty() {
                ayahs.push(((surah.number, row.ayah_start), row.skeleton));
            }
        }
    }
    let shingle_sets: Vec<BTreeSet<String>> =
        ayahs.iter().map(|(_, s)| char_shingles(s, 5)).collect();
    let sigs: Vec<[u64; 64]> = shingle_sets.iter().map(minhash_signature).collect();
    let mut hits = Vec::new();
    for i in 0..ayahs.len() {
        for j in (i + 1)..ayahs.len() {
            // MinHash prefilter: ≥ half the signature slots agree.
            let agree = sigs[i].iter().zip(sigs[j].iter()).filter(|(a, b)| a == b).count();
            if agree < 32 {
                continue;
            }
            // Exact verify: Jaccard over full shingle sets.
            let inter = shingle_sets[i].intersection(&shingle_sets[j]).count();
            let union = shingle_sets[i].union(&shingle_sets[j]).count().max(1);
            let jaccard = inter as f64 / union as f64;
            if jaccard >= threshold {
                hits.push(NearDuplicateHit {
                    first: ayahs[i].0,
                    second: ayahs[j].0,
                    jaccard,
                    shared: inter,
                });
            }
        }
    }
    hits.sort_by(|a, b| {
        b.jaccard
            .partial_cmp(&a.jaccard)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.first.cmp(&b.first))
            .then(a.second.cmp(&b.second))
    });
    hits.truncate(limit.max(1));
    Ok((rules, hits))
}

/// Morphology-gated stubs: root/lemma counting needs an active lexicon.
/// Each names the missing capability (AC-P2-01 fallback, never guessed data).
pub async fn root_frequency(
    _db: &SqliteDatabase,
    _root: &str,
) -> Result<FrequencyReport, CountingError> {
    Err(CountingError::UnavailableDataset { capability: "root frequency".to_string() })
}

/// Lemma frequency (lexicon-gated).
pub async fn lemma_frequency(
    _db: &SqliteDatabase,
    _lemma: &str,
) -> Result<FrequencyReport, CountingError> {
    Err(CountingError::UnavailableDataset { capability: "lemma frequency".to_string() })
}

/// Unusual-usage mining (lexicon-gated: needs analysis distributions).
pub async fn unusual_usage(
    _db: &SqliteDatabase,
    _target: &str,
    _profile: &str,
) -> Result<NumericReport, CountingError> {
    Err(CountingError::UnavailableDataset { capability: "unusual usage mining".to_string() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_carry_cnt_prefix() {
        use storage::error::Diagnostic as _;
        for err in [
            CountingError::Storage("x".into()),
            CountingError::UnknownProfile("x".into()),
            CountingError::ProfileNotCountable("x".into()),
            CountingError::EmptyTarget,
            CountingError::UnavailableDataset { capability: "x".into() },
        ] {
            assert!(err.code().to_string().starts_with("QAI-CNT-"), "{}", err.code());
            assert!(err.remedy().is_some());
            assert!(err.next_command().is_some());
        }
    }

    #[test]
    fn disclaimers_verbatim() {
        assert!(INTERVAL_DISCLAIMER.contains("no chronological"));
        assert!(MISSING_FORM_DISCLAIMER.contains("not evidence of absence"));
        assert_eq!(
            NO_INTERPRETATION_NOTE,
            "Numeric output only; no interpretive commentary is provided."
        );
    }

    #[test]
    fn countable_columns_match_plan() {
        assert_eq!(countable_column("L3.diacritics").unwrap(), FormColumn::Bare);
        assert_eq!(countable_column("L5.codepoints").unwrap(), FormColumn::Folded);
        assert!(matches!(countable_column("L0.exact"), Err(CountingError::ProfileNotCountable(_))));
        assert!(matches!(countable_column("L9.nope"), Err(CountingError::UnknownProfile(_))));
    }

    #[test]
    fn minhash_signature_stable_and_sensitive() {
        let a: BTreeSet<String> = ["abcde", "bcdef"].iter().map(|s| s.to_string()).collect();
        let b = a.clone();
        let mut c = a.clone();
        c.insert("zzzzz".to_string());
        assert_eq!(minhash_signature(&a), minhash_signature(&b));
        assert_ne!(minhash_signature(&a), minhash_signature(&c));
    }
}
