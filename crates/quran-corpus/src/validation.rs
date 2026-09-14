//! Corpus validator: QV-001…QV-028 (D1.4).
//!
//! # quran_corpus::validation
//!
//! [`validate_edition`] runs every content rule with no fail-fast: a partial
//! report is useless for editorial review. Severity semantics:
//! - `Fatal` — the import cannot reach `Staged`.
//! - `Error` — may stage, but cannot be approved without an explicit override
//!   recorded in the approval.
//! - `Warning` / `Info` — shown in the report.
//!
//! Rules that need pipeline state live with the importer (M7) and are marked
//! below: QV-013 has [`check_file_hash`], QV-014 has [`intermediate_hash`],
//! QV-015 skips when no reference corpus is configured (never silently
//! passes), QV-024 is recomputed from stored rows, QV-025/026 hold by
//! construction (the importer assigns provenance; the columns are `NOT NULL`),
//! and QV-028 is a Phase-2 hook that passes vacuously in v1.

use std::collections::BTreeMap;

use domain::{ContentHash, HashAlgorithm, SemVer, canonical_json_bytes};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::adapters::EditionAdapter;
use crate::adapters::JsonAdapter;
use crate::error::CorpusError;
use crate::format::EditionSource;
use crate::tokenize::{ComputedToken, reconstruct, tokenize};
use crate::unicode::{find_forbidden, first_unexpected, normalization_form};

/// The validator name registered in the `sources` validator registry.
pub const VALIDATOR_NAME: &str = "quran_edition_v1";
/// The validator version recorded on every report.
pub const VALIDATOR_VERSION: SemVer = SemVer::new(1, 0, 0);

/// Finding severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Shown in the report.
    Info,
    /// Shown in the report.
    Warning,
    /// May stage; approval needs an explicit override.
    Error,
    /// Cannot reach `Staged`.
    Fatal,
}

/// One rule finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// Rule id, e.g. `QV-005`.
    pub rule_id: String,
    /// Severity.
    pub severity: Severity,
    /// Where it failed, e.g. `surah 2 ayah 255` or `edition`.
    pub location: String,
    /// Human-readable description.
    pub message: String,
}

impl Finding {
    /// Build a finding.
    pub fn new(
        rule_id: impl Into<String>,
        severity: Severity,
        location: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            rule_id: rule_id.into(),
            severity,
            location: location.into(),
            message: message.into(),
        }
    }
}

/// Overall report outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// No findings above `Info`.
    Pass,
    /// Warnings present, no `Error`/`Fatal`.
    PassWithWarnings,
    /// At least one `Error` or `Fatal`.
    Fail,
}

/// Machine-readable validation report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    /// What was validated, e.g. `quran-edition:test-edition-min@0.1.0`.
    pub subject_urn: String,
    /// Validator name ([`VALIDATOR_NAME`]).
    pub validator: String,
    /// Validator version.
    pub validator_version: SemVer,
    /// Overall outcome.
    pub outcome: Outcome,
    /// Number of `Fatal` findings.
    pub fatal_count: u32,
    /// Number of `Error` findings.
    pub error_count: u32,
    /// Number of `Warning` findings.
    pub warning_count: u32,
    /// Every finding, in rule order.
    pub findings: Vec<Finding>,
}

impl ValidationReport {
    /// Whether any `Fatal` finding exists.
    pub fn has_fatal(&self) -> bool {
        self.fatal_count > 0
    }

    /// Whether any finding has the given rule id.
    pub fn has_rule(&self, rule_id: &str) -> bool {
        self.findings.iter().any(|finding| finding.rule_id == rule_id)
    }

    fn finish(subject_urn: String, findings: Vec<Finding>) -> Self {
        let fatal_count = findings.iter().filter(|f| f.severity == Severity::Fatal).count() as u32;
        let error_count = findings.iter().filter(|f| f.severity == Severity::Error).count() as u32;
        let warning_count =
            findings.iter().filter(|f| f.severity == Severity::Warning).count() as u32;
        let outcome = if fatal_count > 0 || error_count > 0 {
            Outcome::Fail
        } else if warning_count > 0 {
            Outcome::PassWithWarnings
        } else {
            Outcome::Pass
        };
        Self {
            subject_urn,
            validator: VALIDATOR_NAME.to_string(),
            validator_version: VALIDATOR_VERSION,
            outcome,
            fatal_count,
            error_count,
            warning_count,
            findings,
        }
    }
}

fn supplied_as_computed(tokens: &[crate::format::TokenSource]) -> Vec<ComputedToken> {
    tokens
        .iter()
        .map(|token| ComputedToken {
            position: u32::from(token.position),
            surface: token.surface.clone(),
            char_start: token.char_start,
            char_end: token.char_end,
            byte_start: token.byte_start,
            byte_end: token.byte_end,
            is_pause_mark: false,
        })
        .collect()
}

/// Validate an edition source against QV-001…QV-012, QV-016…QV-023, and QV-027.
///
/// Pipeline-state rules (QV-013…015, QV-024…026, QV-028) are documented in the
/// module docs and enforced by the importer, not here.
pub fn validate_edition(source: &EditionSource) -> ValidationReport {
    let mut findings: Vec<Finding> = Vec::new();
    let edition_slug = source.edition.slug.clone();
    let subject_urn = format!("quran-edition:{edition_slug}@{}", source.edition.version);

    // QV-027: the canonical import path accepts Arabic editions only.
    if source.edition.language.as_str() != "ar" {
        findings.push(Finding::new(
            "QV-027",
            Severity::Error,
            "edition",
            format!(
                "language `{}` is not Arabic; a translation cannot be imported as a canonical edition",
                source.edition.language
            ),
        ));
    }

    // QV-001: surah count matches the manifest expectation.
    if source.surahs.len() != usize::from(source.expected.surah_count) {
        findings.push(Finding::new(
            "QV-001",
            Severity::Fatal,
            "edition",
            format!(
                "surah count is {}, manifest expects {}",
                source.surahs.len(),
                source.expected.surah_count
            ),
        ));
    }

    // QV-002: surah numbers are exactly 1..=N.
    {
        let mut numbers: Vec<u16> = source.surahs.iter().map(|s| s.number).collect();
        numbers.sort_unstable();
        let expected: Vec<u16> = (1..=numbers.len() as u16).collect();
        if numbers != expected {
            findings.push(Finding::new(
                "QV-002",
                Severity::Fatal,
                "edition",
                format!("surah numbers are not exactly 1..={}: {numbers:?}", numbers.len()),
            ));
        }
    }

    let declared: BTreeMap<u16, u16> =
        source.surahs.iter().map(|s| (s.number, s.ayah_count)).collect();
    let mut actual_count: BTreeMap<u16, usize> = BTreeMap::new();
    for ayah in &source.ayahs {
        *actual_count.entry(ayah.surah).or_insert(0) += 1;
    }

    // QV-003: each surah's ayah count equals its declared count.
    for surah in &source.surahs {
        let actual = actual_count.get(&surah.number).copied().unwrap_or(0);
        if actual != usize::from(surah.ayah_count) {
            findings.push(Finding::new(
                "QV-003",
                Severity::Fatal,
                format!("surah {}", surah.number),
                format!(
                    "surah {} declares {} ayahs, found {actual}",
                    surah.number, surah.ayah_count
                ),
            ));
        }
    }
    for surah in actual_count.keys() {
        if !declared.contains_key(surah) {
            findings.push(Finding::new(
                "QV-003",
                Severity::Fatal,
                format!("surah {surah}"),
                format!("ayahs reference surah {surah} with no metadata row"),
            ));
        }
    }

    // QV-004: total ayah count matches the manifest expectation.
    if source.ayahs.len() != source.expected.ayah_count as usize {
        findings.push(Finding::new(
            "QV-004",
            Severity::Fatal,
            "edition",
            format!(
                "ayah count is {}, manifest expects {}",
                source.ayahs.len(),
                source.expected.ayah_count
            ),
        ));
    }

    // QV-005: ayah numbers within each surah are exactly 1..=declared.
    {
        let mut by_surah: BTreeMap<u16, Vec<u32>> = BTreeMap::new();
        for ayah in &source.ayahs {
            by_surah.entry(ayah.surah).or_default().push(ayah.ayah);
        }
        for (surah, mut numbers) in by_surah {
            numbers.sort_unstable();
            let declared_count = declared.get(&surah).copied().unwrap_or(0) as usize;
            let expected: Vec<u32> = (1..=declared_count as u32).collect();
            if numbers != expected {
                findings.push(Finding::new(
                    "QV-005",
                    Severity::Fatal,
                    format!("surah {surah}"),
                    format!("ayah numbers are not exactly 1..={declared_count}: {numbers:?}"),
                ));
            }
        }
    }

    // Tokenize once; token rules and the round-trip guard reuse this.
    let tokenized: Vec<crate::tokenize::TokenizedAyah> =
        source.ayahs.iter().map(|ayah| tokenize(&ayah.text)).collect();

    for (ayah, computed) in source.ayahs.iter().zip(tokenized.iter()) {
        let location = format!("surah {} ayah {}", ayah.surah, ayah.ayah);

        // QV-006: no empty or whitespace-only ayah text.
        if ayah.text.trim().is_empty() {
            findings.push(Finding::new(
                "QV-006",
                Severity::Fatal,
                location.clone(),
                "ayah text is empty or whitespace-only".to_string(),
            ));
            continue;
        }

        // QV-007: text is in the declared normalization form.
        let declared_form = source.edition.unicode_normalization;
        if normalization_form(&ayah.text) != Some(declared_form) {
            findings.push(Finding::new(
                "QV-007",
                Severity::Fatal,
                location.clone(),
                format!("text is not in the declared {declared_form:?} form"),
            ));
        }

        // QV-008: no forbidden code points.
        let forbidden = find_forbidden(&ayah.text);
        if let Some(first) = forbidden.first() {
            findings.push(Finding::new(
                "QV-008",
                Severity::Fatal,
                location.clone(),
                format!(
                    "forbidden U+{:04X} ({}) at byte {}{}",
                    first.character as u32,
                    first.reason,
                    first.byte_offset,
                    if forbidden.len() > 1 {
                        format!(" ({} total)", forbidden.len())
                    } else {
                        String::new()
                    }
                ),
            ));
        }

        // QV-009: code points are within expected blocks.
        if let Some((byte, ch)) = first_unexpected(&ayah.text) {
            findings.push(Finding::new(
                "QV-009",
                Severity::Error,
                location.clone(),
                format!("U+{:04X} at byte {byte} is outside the expected blocks", ch as u32),
            ));
        }

        // QV-011a: computed tokenization round-trips byte-identically.
        if reconstruct(&computed.tokens, &computed.separators) != ayah.text {
            findings.push(Finding::new(
                "QV-011",
                Severity::Fatal,
                location.clone(),
                "computed tokens do not reconstruct the ayah text".to_string(),
            ));
        }

        // Supplied-token rules (verified, never trusted).
        if let Some(supplied) = ayah.tokens.as_deref() {
            // QV-010: positions are 1..=k in row order.
            let ordered = supplied
                .iter()
                .enumerate()
                .all(|(index, token)| token.position as usize == index + 1);
            if !ordered {
                findings.push(Finding::new(
                    "QV-010",
                    Severity::Fatal,
                    location.clone(),
                    "supplied token positions are not 1..=k in order".to_string(),
                ));
            }

            // QV-011b: supplied tokens reconstruct the ayah text.
            let supplied_computed = supplied_as_computed(supplied);
            if reconstruct(&supplied_computed, &computed.separators) != ayah.text {
                findings.push(Finding::new(
                    "QV-011",
                    Severity::Fatal,
                    location.clone(),
                    "supplied tokens do not reconstruct the ayah text".to_string(),
                ));
            }

            // QV-012: supplied offsets are valid boundaries slicing to surfaces.
            let graphemes =
                unicode_segmentation::UnicodeSegmentation::graphemes(ayah.text.as_str(), true)
                    .count() as u32;
            for token in supplied {
                let token_location = format!("{} token {}", location, token.position);
                let bytes_ok = ayah.text.get(token.byte_start as usize..token.byte_end as usize)
                    == Some(token.surface.as_str());
                let chars_ok = token.char_end > token.char_start && token.char_end <= graphemes;
                if !bytes_ok || !chars_ok {
                    findings.push(Finding::new(
                        "QV-012",
                        Severity::Fatal,
                        token_location,
                        format!(
                            "offsets [{}, {}) / [{}, {}) do not slice to the surface",
                            token.byte_start, token.byte_end, token.char_start, token.char_end
                        ),
                    ));
                }
            }
        }
    }

    // QV-016: juz values are contiguous and cover the corpus (where present).
    {
        let juz: Vec<u16> = source.ayahs.iter().filter_map(|a| a.juz).collect();
        if !juz.is_empty() {
            let mut bad: Option<String> = None;
            if juz[0] != 1 {
                bad = Some(format!("juz coverage starts at {}, not 1", juz[0]));
            } else {
                for pair in juz.windows(2) {
                    if pair[1] != pair[0] && pair[1] != pair[0] + 1 {
                        bad = Some(format!("juz jumps from {} to {}", pair[0], pair[1]));
                        break;
                    }
                }
            }
            if let Some(message) = bad {
                findings.push(Finding::new("QV-016", Severity::Error, "edition", message));
            }
        }
    }

    // QV-017: hizb/rub/manzil consistency with juz (where all are present).
    {
        let mut notes: Vec<String> = Vec::new();
        let mut by_juz: BTreeMap<u16, Vec<&crate::format::AyahSource>> = BTreeMap::new();
        for ayah in source.ayahs.iter().filter(|a| a.juz.is_some()) {
            by_juz.entry(ayah.juz.unwrap_or(0)).or_default().push(ayah);
        }
        for (juz, group) in &by_juz {
            let hizb: Vec<u16> = group.iter().filter_map(|a| a.hizb).collect();
            if hizb.windows(2).any(|w| w[1] < w[0]) {
                notes.push(format!("hizb decreases within juz {juz}"));
            }
            let manzil: Vec<u16> = group.iter().filter_map(|a| a.manzil).collect();
            if manzil.windows(2).any(|w| w[1] < w[0]) {
                notes.push(format!("manzil decreases within juz {juz}"));
            }
            let mut by_hizb: BTreeMap<u16, Vec<u16>> = BTreeMap::new();
            for ayah in group.iter().filter(|a| a.hizb.is_some()) {
                by_hizb.entry(ayah.hizb.unwrap_or(0)).or_default().extend(ayah.rub);
            }
            for (hizb, rub) in &by_hizb {
                if rub.windows(2).any(|w| w[1] < w[0]) {
                    notes.push(format!("rub decreases within juz {juz} hizb {hizb}"));
                }
            }
        }
        for note in notes {
            findings.push(Finding::new("QV-017", Severity::Warning, "edition", note));
        }
    }

    // QV-018: pages are monotonic non-decreasing by global ayah order.
    {
        let pages: Vec<u32> = source.ayahs.iter().filter_map(|a| a.page).collect();
        if pages.windows(2).any(|w| w[1] < w[0]) {
            findings.push(Finding::new(
                "QV-018",
                Severity::Error,
                "edition",
                "page numbers decrease along the global ayah order".to_string(),
            ));
        }
    }

    // QV-019: sajdah markers vs the manifest expectation (v1 declares none).
    {
        let sajdah_count = source.ayahs.iter().filter(|a| a.sajdah.is_some()).count();
        if sajdah_count > 0 {
            findings.push(Finding::new(
                "QV-019",
                Severity::Info,
                "edition",
                format!(
                    "{sajdah_count} sajdah markers present; v1 declares no expectation to compare"
                ),
            ));
        }
    }

    // QV-020: basmala policy stated per surah and consistent when edition-wide.
    if source.edition.basmala_policy != quran_core::enums::BasmalaPolicy::PerSurah {
        for surah in &source.surahs {
            if surah.basmala != source.edition.basmala_policy {
                findings.push(Finding::new(
                    "QV-020",
                    Severity::Error,
                    format!("surah {}", surah.number),
                    "surah basmala policy disagrees with the edition-wide policy".to_string(),
                ));
            }
        }
    }

    // QV-021: revelation place present per surah.
    for surah in source.surahs.iter().filter(|s| s.revelation_place.is_none()) {
        findings.push(Finding::new(
            "QV-021",
            Severity::Warning,
            format!("surah {}", surah.number),
            "revelation place is missing (Layer-B metadata)".to_string(),
        ));
    }

    // QV-022: duplicate ayah text within a surah (legitimate repetition exists).
    {
        let mut seen: BTreeMap<(u16, &str), u32> = BTreeMap::new();
        for ayah in &source.ayahs {
            if let Some(first) = seen.insert((ayah.surah, ayah.text.as_str()), ayah.ayah) {
                findings.push(Finding::new(
                    "QV-022",
                    Severity::Info,
                    format!("surah {}", ayah.surah),
                    format!(
                        "ayah {} repeats the text of ayah {first} (whitelist legitimate refrains)",
                        ayah.ayah
                    ),
                ));
            }
        }
    }

    // QV-023: file order is strictly increasing by (surah, ayah).
    {
        let mut previous = (0_u16, 0_u32);
        for ayah in &source.ayahs {
            if (ayah.surah, ayah.ayah) <= previous {
                findings.push(Finding::new(
                    "QV-023",
                    Severity::Fatal,
                    format!("surah {} ayah {}", ayah.surah, ayah.ayah),
                    "ayahs are not in strictly increasing (surah, ayah) order".to_string(),
                ));
                break;
            }
            previous = (ayah.surah, ayah.ayah);
        }
    }

    ValidationReport::finish(subject_urn, findings)
}

/// QV-013: compare a declared file hash against observed bytes.
///
/// Returns `None` on match, else the `Fatal` finding. The importer runs this
/// at checkpoint 2 for every declared file.
pub fn check_file_hash(location: &str, declared_hex: &str, observed: &[u8]) -> Option<Finding> {
    let mut hasher = Sha256::new();
    hasher.update(observed);
    let observed_hex = format!("{:x}", hasher.finalize());
    if observed_hex == declared_hex.to_lowercase() {
        None
    } else {
        Some(Finding::new(
            "QV-013",
            Severity::Fatal,
            location,
            "declared file hash does not match the observed hash".to_string(),
        ))
    }
}

/// QV-014 helper: hash of the intermediate document itself.
///
/// The importer hashes before staging and compares against the hash recomputed
/// from stored rows, proving the round trip is stable.
pub fn intermediate_hash(source: &EditionSource) -> Result<ContentHash, CorpusError> {
    let bytes = canonical_json_bytes(source)
        .map_err(|err| CorpusError::InvalidFormat { detail: err.to_string() })?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(ContentHash { algorithm: HashAlgorithm::Sha256, hex: format!("{:x}", hasher.finalize()) })
}

/// A `sources::StructureValidator` running this validator over a held document.
///
/// The Phase-0 registry passes only a `SourceVersion`; the importer holds the
/// bytes, so this bridge carries the document it validates.
pub struct QuranEditionValidator {
    document: String,
}

impl QuranEditionValidator {
    /// The registered validator name.
    pub const NAME: &str = VALIDATOR_NAME;

    /// Validate this edition document when the registry invokes the validator.
    pub fn for_document(document: impl Into<String>) -> Self {
        Self { document: document.into() }
    }

    fn run(&self) -> ValidationReport {
        match JsonAdapter.parse(&self.document) {
            Ok(source) => validate_edition(&source),
            Err(err) => ValidationReport::finish(
                "quran-edition:unparseable".to_string(),
                vec![Finding::new(
                    "QV-000",
                    Severity::Fatal,
                    "document",
                    format!("document is not a parseable edition source: {err}"),
                )],
            ),
        }
    }
}

#[async_trait::async_trait]
impl sources::StructureValidator for QuranEditionValidator {
    async fn validate(
        &self,
        _version: &sources::SourceVersion,
    ) -> Result<sources::ValidationReport, sources::ValidationError> {
        let report = self.run();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        for finding in &report.findings {
            let mapped = sources::ValidationError {
                code: finding.rule_id.to_string(),
                message: finding.message.clone(),
                location: Some(finding.location.clone()),
            };
            match finding.severity {
                Severity::Fatal | Severity::Error => errors.push(mapped),
                Severity::Warning | Severity::Info => warnings.push(mapped),
            }
        }
        Ok(sources::ValidationReport {
            valid: report.outcome == Outcome::Pass || report.outcome == Outcome::PassWithWarnings,
            validator_name: Self::NAME.to_string(),
            errors,
            warnings,
        })
    }

    fn name(&self) -> &str {
        Self::NAME
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::EditionAdapter;

    const BASE: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");

    fn base_source() -> EditionSource {
        JsonAdapter.parse(BASE).expect("base fixture parses")
    }

    #[test]
    fn base_fixture_has_no_fatal_or_error() {
        let report = validate_edition(&base_source());
        assert!(
            !report.has_fatal() && report.error_count == 0,
            "base must be clean: {:?}",
            report.findings
        );
    }

    #[test]
    fn outcome_counts_match_findings() {
        let report = validate_edition(&base_source());
        let fatal = report.findings.iter().filter(|f| f.severity == Severity::Fatal).count() as u32;
        let error = report.findings.iter().filter(|f| f.severity == Severity::Error).count() as u32;
        assert_eq!((report.fatal_count, report.error_count), (fatal, error));
    }

    #[test]
    fn intermediate_hash_is_stable() {
        let source = base_source();
        assert_eq!(intermediate_hash(&source).unwrap(), intermediate_hash(&source).unwrap());
    }

    #[test]
    fn file_hash_check_detects_tampering() {
        assert!(check_file_hash("data", &"0".repeat(64), b"tampered").is_some());
        let mut hasher = Sha256::new();
        hasher.update(b"tampered");
        let hex = format!("{:x}", hasher.finalize());
        assert!(check_file_hash("data", &hex, b"tampered").is_none());
    }
}
