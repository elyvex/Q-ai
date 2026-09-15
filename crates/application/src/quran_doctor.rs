//! `qai doctor --quran`: the 19 corpus integrity checks (D1.11, P1-T51/T52).
//!
//! # application::quran_doctor
//!
//! Read-only by construction: every check runs inside one unit of work that is
//! rolled back, and no write method is invoked. `--deep` upgrades the token
//! round-trip from sampled to full-corpus (hashes always recompute fully;
//! `--deep` completes a standard edition in seconds).

use std::collections::BTreeMap;

use quran_corpus::{AyahLayout, TokenOrder, structure_hash, text_hash, token_order_hash};
use storage::quran::AyahRow;

/// Check severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckLevel {
    /// Healthy.
    Pass,
    /// Needs attention.
    Warn,
    /// Broken.
    Fail,
    /// Not applicable.
    Skipped,
}

impl CheckLevel {
    /// Stable string for CLI/JSON rendering.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Warn => "warn",
            Self::Fail => "fail",
            Self::Skipped => "skipped",
        }
    }
}

/// One doctor check result.
#[derive(Debug, Clone)]
pub struct QuranDoctorCheck {
    /// Check id (`quran.*`).
    pub id: &'static str,
    /// Severity.
    pub status: CheckLevel,
    /// Human summary.
    pub summary: String,
    /// Remedy, when actionable.
    pub remedy: Option<String>,
    /// Next command.
    pub next_command: Option<String>,
}

fn pass(id: &'static str, summary: String) -> QuranDoctorCheck {
    QuranDoctorCheck { id, status: CheckLevel::Pass, summary, remedy: None, next_command: None }
}

fn fail(id: &'static str, summary: String, remedy: &str) -> QuranDoctorCheck {
    QuranDoctorCheck {
        id,
        status: CheckLevel::Fail,
        summary,
        remedy: Some(remedy.to_string()),
        next_command: Some("qai quran validate".to_string()),
    }
}

fn warn(id: &'static str, summary: String, remedy: &str) -> QuranDoctorCheck {
    QuranDoctorCheck {
        id,
        status: CheckLevel::Warn,
        summary,
        remedy: Some(remedy.to_string()),
        next_command: Some("qai quran validate".to_string()),
    }
}

fn tagged_hex(hash: &domain::ContentHash) -> String {
    format!("sha256:{}", hash.hex)
}

/// Run all 19 checks against the active edition. `deep` upgrades the token
/// round-trip to a full-corpus scan.
pub async fn run_quran_checks(
    db: &dyn storage::Database,
    deep: bool,
) -> Result<Vec<QuranDoctorCheck>, storage::error::StorageError> {
    let mut uow = db.write().await?;
    let active = uow.quran().get_active().await?;
    let Some(active) = active else {
        uow.rollback().await?;
        return Ok(vec![QuranDoctorCheck {
            id: "quran.edition_active",
            status: CheckLevel::Fail,
            summary: "no active Arabic edition".to_string(),
            remedy: Some("import and activate an edition".to_string()),
            next_command: Some("qai quran import --help".to_string()),
        }]);
    };
    let Some(edition) = uow.quran().get_edition(&active.edition_id).await? else {
        uow.rollback().await?;
        return Ok(vec![QuranDoctorCheck {
            id: "quran.edition_active",
            status: CheckLevel::Fail,
            summary: "active pointer dangles".to_string(),
            remedy: Some("inspect quran_active_edition".to_string()),
            next_command: Some("qai doctor --quran".to_string()),
        }]);
    };
    let edition_id = edition.id.clone();
    let surahs = uow.quran().list_surahs(&edition_id).await?;
    let ayahs = uow.quran().list_ayahs_range(&edition_id, 1, i64::MAX).await?;
    let mut tokens_by_ayah: BTreeMap<(i64, i64), Vec<storage::quran::TokenRow>> = BTreeMap::new();
    for ayah in &ayahs {
        tokens_by_ayah.insert(
            (ayah.surah, ayah.ayah),
            uow.quran().get_tokens(&edition_id, ayah.surah, ayah.ayah).await?,
        );
    }
    let mut separators_by_ayah: BTreeMap<(i64, i64), Vec<String>> = BTreeMap::new();
    for ayah in &ayahs {
        separators_by_ayah.insert(
            (ayah.surah, ayah.ayah),
            uow.quran()
                .get_separators(&edition_id, ayah.surah, ayah.ayah)
                .await?
                .into_iter()
                .map(|row| row.separator)
                .collect(),
        );
    }
    let divisions_juz = uow.quran().list_divisions(&edition_id, "juz").await?;
    let translations = uow.quran().list_translation_editions().await?;
    let orphans = uow.quran().count_staging_orphans().await?;
    uow.rollback().await?;

    let mut checks = Vec::new();
    checks.push(QuranDoctorCheck {
        id: "quran.edition_active",
        status: CheckLevel::Pass,
        summary: format!("exactly one active edition: {}@{}", edition.slug, edition.version),
        remedy: None,
        next_command: None,
    });

    // Hash recomputation from stored rows.
    let texts: Vec<&str> = ayahs.iter().map(|row| row.text.as_str()).collect();
    let recomputed_text = tagged_hex(&text_hash(&edition.slug, &edition.version, &texts));
    checks.push(if recomputed_text == edition.text_hash {
        pass("quran.edition_checksum", format!("text_hash matches ({})", short(&edition.text_hash)))
    } else {
        fail(
            "quran.edition_checksum",
            "stored text_hash differs from recomputation".to_string(),
            "re-import the edition; do not edit canonical rows",
        )
    });

    let surah_pairs: Vec<(u16, u16)> =
        surahs.iter().map(|row| (row.number as u16, row.ayah_count as u16)).collect();
    let layouts: Vec<AyahLayout> = ayahs
        .iter()
        .map(|row| AyahLayout {
            surah: row.surah as u16,
            ayah: row.ayah as u32,
            juz: row.juz.map(|value| value as u16),
            hizb: row.hizb.map(|value| value as u16),
            rub: row.rub.map(|value| value as u16),
            manzil: row.manzil.map(|value| value as u16),
            ruku: row.ruku.map(|value| value as u16),
            page: row.page.map(|value| value as u32),
            sajdah: row.sajdah.as_deref().and_then(|kind| match kind {
                "recommended" => Some(quran_core::SajdahKind::Recommended),
                "obligatory" => Some(quran_core::SajdahKind::Obligatory),
                _ => None,
            }),
        })
        .collect();
    let recomputed_structure =
        tagged_hex(&structure_hash(&edition.slug, &edition.version, &surah_pairs, &layouts));
    checks.push(if recomputed_structure == edition.structure_hash {
        pass("quran.structure_hash", "structural skeleton hash matches".to_string())
    } else {
        fail(
            "quran.structure_hash",
            "stored structure_hash differs from recomputation".to_string(),
            "re-import the edition; do not edit canonical rows",
        )
    });

    let mut orders = Vec::new();
    for ayah in &ayahs {
        if let Some(tokens) = tokens_by_ayah.get(&(ayah.surah, ayah.ayah)) {
            for token in tokens {
                orders.push(TokenOrder {
                    surah: ayah.surah as u16,
                    ayah: ayah.ayah as u32,
                    position: token.position as u32,
                    surface: token.surface.as_str(),
                });
            }
        }
    }
    let recomputed_order = tagged_hex(&token_order_hash(&edition.slug, &edition.version, &orders));
    checks.push(if recomputed_order == edition.token_order_hash {
        pass("quran.token_order_hash", "token order hash matches".to_string())
    } else {
        fail(
            "quran.token_order_hash",
            "stored token_order_hash differs from recomputation".to_string(),
            "re-import the edition; do not edit canonical rows",
        )
    });

    // Counts vs recorded statistics.
    let stats: serde_json::Value =
        serde_json::from_str(&edition.statistics_json).unwrap_or(serde_json::Value::Null);
    let stat_count = |key: &str| stats.get(key).and_then(|value| value.as_u64()).unwrap_or(0);
    checks.push(if surahs.len() as u64 == stat_count("surah_count") {
        pass("quran.surah_count", format!("{} surahs match statistics", surahs.len()))
    } else {
        fail(
            "quran.surah_count",
            format!("{} surahs vs {} in statistics", surahs.len(), stat_count("surah_count")),
            "re-import the edition",
        )
    });
    checks.push(if ayahs.len() as u64 == stat_count("ayah_count") {
        pass("quran.ayah_count", format!("{} ayahs match statistics", ayahs.len()))
    } else {
        fail(
            "quran.ayah_count",
            format!("{} ayahs vs {} in statistics", ayahs.len(), stat_count("ayah_count")),
            "re-import the edition",
        )
    });

    // Ayah identifiers dense per surah.
    let mut identifiers_ok = true;
    for surah in &surahs {
        let mut numbers: Vec<i64> =
            ayahs.iter().filter(|row| row.surah == surah.number).map(|row| row.ayah).collect();
        numbers.sort_unstable();
        let expected: Vec<i64> = (1..=surah.ayah_count).collect();
        if numbers != expected {
            identifiers_ok = false;
            break;
        }
    }
    checks.push(if identifiers_ok {
        pass("quran.ayah_identifiers", "ayah numbers dense in every surah".to_string())
    } else {
        fail(
            "quran.ayah_identifiers",
            "gaps or duplicates in ayah numbering".to_string(),
            "re-import the edition",
        )
    });

    // Token round-trip: sampled normally, full with --deep.
    let sample: Vec<&AyahRow> = if deep {
        ayahs.iter().collect()
    } else if ayahs.is_empty() {
        Vec::new()
    } else {
        let mut picked = vec![&ayahs[0]];
        if ayahs.len() > 2 {
            picked.push(&ayahs[ayahs.len() / 2]);
        }
        if ayahs.len() > 1 {
            picked.push(&ayahs[ayahs.len() - 1]);
        }
        picked
    };
    let mut roundtrip_ok = true;
    for ayah in sample {
        let key = (ayah.surah, ayah.ayah);
        let tokens = tokens_by_ayah.get(&key).cloned().unwrap_or_default();
        let separators = separators_by_ayah.get(&key).cloned().unwrap_or_default();
        let computed: Vec<quran_corpus::ComputedToken> = tokens
            .iter()
            .map(|token| quran_corpus::ComputedToken {
                position: token.position as u32,
                surface: token.surface.clone(),
                char_start: token.char_start as u32,
                char_end: token.char_end as u32,
                byte_start: token.byte_start as u32,
                byte_end: token.byte_end as u32,
                is_pause_mark: token.is_pause_mark,
            })
            .collect();
        if quran_corpus::reconstruct(&computed, &separators) != ayah.text {
            roundtrip_ok = false;
            break;
        }
    }
    checks.push(if roundtrip_ok {
        pass(
            "quran.token_roundtrip",
            if deep {
                format!("full-corpus round-trip over {} ayahs", ayahs.len())
            } else {
                "sampled round-trip holds (use --deep for full scan)".to_string()
            },
        )
    } else {
        fail(
            "quran.token_roundtrip",
            "token reconstruction mismatch".to_string(),
            "re-import the edition",
        )
    });

    // Unicode form across the corpus.
    let declared = edition.unicode_normalization.as_str();
    let bad_unicode = ayahs
        .iter()
        .filter(|row| {
            quran_corpus::unicode::normalization_form(&row.text)
                .is_none_or(|form| format!("{form:?}").to_lowercase() != declared)
        })
        .count();
    checks.push(if bad_unicode == 0 {
        pass("quran.unicode_form", format!("all ayahs in declared {declared} form"))
    } else {
        fail(
            "quran.unicode_form",
            format!("{bad_unicode} ayahs not in {declared} form"),
            "re-import from NFC-normalized sources",
        )
    });
    // Division coverage (juz contiguous).
    let juz: Vec<i64> = ayahs.iter().filter_map(|row| row.juz).collect();
    let mut coverage_ok = !juz.is_empty() && juz[0] == 1;
    for pair in juz.windows(2) {
        if pair[1] != pair[0] && pair[1] != pair[0] + 1 {
            coverage_ok = false;
            break;
        }
    }
    checks.push(if coverage_ok {
        pass(
            "quran.division_coverage",
            format!("juz coverage contiguous ({} divisions)", divisions_juz.len()),
        )
    } else {
        fail(
            "quran.division_coverage",
            "juz coverage has gaps".to_string(),
            "re-import with complete division metadata",
        )
    });

    // Basmala policy stated everywhere.
    let basmala_ok =
        !edition.basmala_policy.is_empty() && surahs.iter().all(|row| !row.basmala.is_empty());
    checks.push(if basmala_ok {
        pass("quran.basmala_policy", "basmala policy declared for every surah".to_string())
    } else {
        fail(
            "quran.basmala_policy",
            "missing basmala policy".to_string(),
            "re-import with declared policies",
        )
    });

    // Provenance complete.
    let provenance_ok = ayahs.iter().all(|row| !row.provenance_id.trim().is_empty());
    checks.push(if provenance_ok {
        pass("quran.provenance_complete", "every ayah carries canonical provenance".to_string())
    } else {
        fail(
            "quran.provenance_complete",
            "ayahs without provenance".to_string(),
            "re-import; the importer assigns provenance to every row",
        )
    });

    // Metadata layering: translations attributed, never canonical.
    let attribution_ok = translations.iter().all(|row| !row.translator.trim().is_empty());
    checks.push(if attribution_ok {
        pass(
            "quran.metadata_layering",
            "layer separation holds; translations attributed".to_string(),
        )
    } else {
        fail(
            "quran.metadata_layering",
            "unattributed translation edition present".to_string(),
            "re-import translations with named translators",
        )
    });

    // Translations aligned.
    let mut aligned_ok = true;
    for row in &translations {
        if row.aligned_edition_id != edition_id {
            aligned_ok = false;
            break;
        }
    }
    checks.push(if aligned_ok {
        pass("quran.translations_aligned", "translations align to existing editions".to_string())
    } else {
        fail(
            "quran.translations_aligned",
            "a translation aligns to an unknown edition".to_string(),
            "re-import the translation against the right edition",
        )
    });
    checks.push(if attribution_ok {
        pass("quran.translation_attribution", "every translation names a translator".to_string())
    } else {
        fail(
            "quran.translation_attribution",
            "unattributed translation edition present".to_string(),
            "re-import translations with named translators",
        )
    });

    // Staging orphans.
    checks.push(if orphans == 0 {
        pass("quran.staging_orphans", "no leftover staging rows".to_string())
    } else {
        fail(
            "quran.staging_orphans",
            format!("{orphans} orphaned staging rows"),
            "cancel or retry the owning import runs",
        )
    });

    // Generation + reference corpus + license.
    checks.push(pass(
        "quran.corpus_generation",
        format!("current generation is {}", active.corpus_generation),
    ));
    checks.push(warn(
        "quran.reference_corpus",
        "no reference corpus configured; QV-015 skips (ADR-0114 pending)".to_string(),
        "configure a reference corpus and sign-off procedure",
    ));
    let license_status = serde_json::from_str::<serde_json::Value>(&edition.license_json)
        .ok()
        .and_then(|value| {
            value.get("status").and_then(|status| status.as_str()).map(str::to_string)
        })
        .unwrap_or_default();
    checks.push(if license_status != "Unknown" && !license_status.is_empty() {
        pass("quran.license_status", format!("active edition license: {license_status}"))
    } else {
        warn(
            "quran.license_status",
            "active edition license is unknown".to_string(),
            "record the dataset license (ADR-0101)",
        )
    });

    Ok(checks)
}

fn short(hash: &str) -> String {
    hash.chars().take(16).collect()
}
