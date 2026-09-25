//! Phase-2 index/linguistics doctor checks: `qai doctor --indexes` (P2-T105).
//!
//! # application::quran_doctor_indexes
//!
//! Nineteen read-only checks over derived data (normalization catalog,
//! derived forms, FTS indexes, trigram postings, morphology, lexicon, and a
//! data-driven search smoke suite). Read-only by construction: catalog reads
//! run inside units of work that are rolled back, search opens the serving
//! generation read-only, and no write method is invoked.
//!
//! Severity policy: missing optional subsystems are `Skipped`, stale-but-
//! servable state is `Warn`, and only corruption, dangling references, or
//! failed tripwires are `Fail`. A fresh database with nothing built reports
//! `Skipped`, never `Fail`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::quran_doctor::{CheckLevel, QuranDoctorCheck};
use storage::Database as _;
use storage::quran::TokenAnalysisRow;
use storage_sqlite::SqliteDatabase;

/// The 19 Phase-2 check ids, in plan §12 order.
pub const INDEX_CHECK_IDS: [&str; 19] = [
    "quran.normalization.profiles_loaded",
    "quran.normalization.idempotency",
    "quran.normalization.spanmap_sanity",
    "quran.forms.current",
    "quran.forms.coverage",
    "quran.canonical_unchanged",
    "quran.fts.ayah_index",
    "quran.fts.token_index",
    "quran.skeleton.index",
    "quran.index.drift",
    "quran.index.orphans",
    "quran.morphology.dataset_active",
    "quran.morphology.alignment",
    "quran.morphology.coverage",
    "quran.morphology.provenance",
    "quran.morphology.unverified_layer_d",
    "quran.lexicon.integrity",
    "quran.lexicon.root_unification_queue",
    "quran.search.smoke",
];

fn pass(id: &'static str, summary: String) -> QuranDoctorCheck {
    QuranDoctorCheck { id, status: CheckLevel::Pass, summary, remedy: None, next_command: None }
}

fn fail(id: &'static str, summary: String, remedy: &str, next: &str) -> QuranDoctorCheck {
    QuranDoctorCheck {
        id,
        status: CheckLevel::Fail,
        summary,
        remedy: Some(remedy.to_string()),
        next_command: Some(next.to_string()),
    }
}

fn warn(id: &'static str, summary: String, remedy: &str, next: &str) -> QuranDoctorCheck {
    QuranDoctorCheck {
        id,
        status: CheckLevel::Warn,
        summary,
        remedy: Some(remedy.to_string()),
        next_command: Some(next.to_string()),
    }
}

fn skipped(id: &'static str, summary: String, remedy: &str, next: &str) -> QuranDoctorCheck {
    QuranDoctorCheck {
        id,
        status: CheckLevel::Skipped,
        summary,
        remedy: Some(remedy.to_string()),
        next_command: Some(next.to_string()),
    }
}

/// Shared read-only snapshot for the 19 checks.
struct Snapshot {
    edition_id: String,
    edition_slug: String,
    edition_version: String,
    corpus_generation: i64,
    ayahs: Vec<storage::quran::AyahRow>,
    token_total: i64,
    profiles_db: Vec<storage::quran::NormalizationProfileRow>,
    token_forms: i64,
    ayah_forms: i64,
    pointer: Option<storage::quran::IndexPointerRow>,
    runs: Vec<storage::quran::IndexBuildRunRow>,
    datasets: Vec<storage::quran::QuranDatasetRow>,
    active_dataset: Option<storage::quran::QuranDatasetRow>,
    pending_reviews: Vec<storage::quran::MorphReviewItemRow>,
}

impl Snapshot {
    fn has_edition(&self) -> bool {
        !self.edition_id.is_empty()
    }
}

/// Deterministic sampler: every 7th ayah plus first/last, capped (deep = all).
fn sample_ayahs(ayahs: &[storage::quran::AyahRow], deep: bool) -> Vec<&storage::quran::AyahRow> {
    if deep {
        return ayahs.iter().collect();
    }
    let mut out: Vec<&storage::quran::AyahRow> = ayahs
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 7 == 0)
        .map(|(_, row)| row)
        .take(200)
        .collect();
    if let Some(first) = ayahs.first()
        && !out.iter().any(|row| row.surah == first.surah && row.ayah == first.ayah)
    {
        out.push(first);
    }
    if let Some(last) = ayahs.last()
        && !out.iter().any(|row| row.surah == last.surah && row.ayah == last.ayah)
    {
        out.push(last);
    }
    out
}

/// Run all 19 Phase-2 checks. `data_dir` is the index root (`gen-<N>/` lives
/// here); `deep` upgrades samples to full-corpus scans.
pub async fn run_index_checks(
    db: &SqliteDatabase,
    data_dir: &Path,
    deep: bool,
) -> Result<Vec<QuranDoctorCheck>, storage::error::StorageError> {
    let snapshot = load_snapshot(db).await?;
    if !snapshot.has_edition() {
        // No active edition: every check skips. The count invariant (19)
        // holds so surfaces can rely on it.
        return Ok(INDEX_CHECK_IDS
            .iter()
            .map(|id| {
                skipped(
                    id,
                    "no active Arabic edition".to_string(),
                    "import and activate an edition",
                    "qai quran import --help",
                )
            })
            .collect());
    }
    let mut checks = Vec::with_capacity(19);
    checks.push(check_profiles_loaded(&snapshot));
    checks.push(check_idempotency(&snapshot));
    checks.push(check_spanmap_sanity(&snapshot));
    checks.push(check_forms_current(db, &snapshot).await);
    checks.push(check_forms_coverage(&snapshot));
    checks.push(check_canonical_unchanged(db, &snapshot).await);
    checks.push(
        check_fts_pointer(
            db,
            data_dir,
            &snapshot,
            "quran.fts.ayah_index",
            super::quran_index::QURAN_AYAH_INDEX_ID,
            format!("index {} not built", super::quran_index::QURAN_AYAH_INDEX_ID),
            "build an index generation",
        )
        .await,
    );
    checks.push(
        check_fts_pointer(
            db,
            data_dir,
            &snapshot,
            "quran.fts.token_index",
            "quran.token.v1",
            "token-level index quran.token.v1 has no backend yet".to_string(),
            "track the token-index follow-up; the ayah index is the serving index",
        )
        .await,
    );
    checks.push(check_skeleton_index(data_dir, &snapshot).await);
    checks.push(check_index_drift(&snapshot));
    checks.push(check_index_orphans(data_dir, &snapshot));
    checks.push(check_dataset_active(&snapshot));
    let morph = morphology_token_probe(db, &snapshot, deep).await;
    checks.push(check_alignment(&snapshot, &morph));
    checks.push(check_morphology_coverage(&snapshot, &morph));
    checks.push(check_morphology_provenance(&snapshot, &morph));
    checks.push(check_unverified_layer_d(&snapshot, &morph));
    checks.push(check_lexicon_integrity(db, &snapshot, &morph).await);
    checks.push(check_unification_queue(&snapshot));
    checks.push(check_search_smoke(db, data_dir, &snapshot).await);
    debug_assert_eq!(checks.len(), 19);
    debug_assert!(checks.iter().map(|c| c.id).collect::<Vec<_>>() == INDEX_CHECK_IDS);
    Ok(checks)
}

async fn load_snapshot(db: &SqliteDatabase) -> Result<Snapshot, storage::error::StorageError> {
    let mut uow = db.write().await?;
    let empty = Snapshot {
        edition_id: String::new(),
        edition_slug: String::new(),
        edition_version: String::new(),
        corpus_generation: 0,
        ayahs: Vec::new(),
        token_total: 0,
        profiles_db: Vec::new(),
        token_forms: 0,
        ayah_forms: 0,
        pointer: None,
        runs: Vec::new(),
        datasets: Vec::new(),
        active_dataset: None,
        pending_reviews: Vec::new(),
    };
    let Some(active) = uow.quran().get_active().await? else {
        uow.rollback().await?;
        return Ok(empty);
    };
    let Some(edition) = uow.quran().get_edition(&active.edition_id).await? else {
        uow.rollback().await?;
        return Ok(empty);
    };
    let edition_id = edition.id.clone();
    let ayahs = uow.quran().list_ayahs_range(&edition_id, 1, i64::MAX).await?;
    let token_total = uow.quran().count_tokens(&edition_id).await?;
    let profiles_db = uow.quran().list_normalization_profiles().await?;
    let token_forms = uow.quran().count_token_forms(&edition_id).await?;
    let ayah_forms = uow.quran().count_ayah_forms(&edition_id).await?;
    let pointer = uow.quran().get_index_pointer(super::quran_index::QURAN_AYAH_INDEX_ID).await?;
    let runs = uow.quran().list_build_runs(super::quran_index::QURAN_AYAH_INDEX_ID).await?;
    let datasets = uow.quran().list_datasets().await?;
    let active_dataset = uow.quran().active_dataset().await?;
    let pending_reviews = uow.quran().list_review_items("pending").await?;
    uow.rollback().await?;
    Ok(Snapshot {
        edition_id: edition.id.clone(),
        edition_slug: edition.slug.clone(),
        edition_version: edition.version.clone(),
        corpus_generation: active.corpus_generation,
        ayahs,
        token_total,
        profiles_db,
        token_forms,
        ayah_forms,
        pointer,
        runs,
        datasets,
        active_dataset,
        pending_reviews,
    })
}

fn check_profiles_loaded(snapshot: &Snapshot) -> QuranDoctorCheck {
    const ID: &str = "quran.normalization.profiles_loaded";
    const NEXT: &str = "qai quran normalize --list-profiles";
    if snapshot.profiles_db.is_empty() {
        return fail(
            ID,
            "no normalization profiles seeded".to_string(),
            "apply migration 0013 seeds",
            NEXT,
        );
    }
    let registry = super::quran_normalize::builtin_registry();
    let mut bad = Vec::new();
    for row in &snapshot.profiles_db {
        let resolved = quran_normalization::ProfileId::parse(&row.profile_id)
            .map_err(|_| row.profile_id.clone())
            .and_then(|id| {
                row.version
                    .parse::<quran_normalization::SemVer>()
                    .map(|version| (id, version))
                    .map_err(|_| format!("{}@{}", row.profile_id, row.version))
            })
            .map(|(id, version)| registry.get(id, version).is_ok())
            .unwrap_or(false);
        if !resolved {
            bad.push(format!("{}@{}", row.profile_id, row.version));
        }
    }
    if bad.is_empty() {
        pass(ID, format!("{} seeded profiles resolve to known rules", snapshot.profiles_db.len()))
    } else {
        fail(
            ID,
            format!("{} profiles do not resolve: {}", bad.len(), bad.join(", ")),
            "seed/code drift: re-seed or update the rule catalog",
            NEXT,
        )
    }
}

/// (location, text) samples: ayah texts for rule-level probes.
fn sample_texts(snapshot: &Snapshot, cap: usize) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for ayah in sample_ayahs(&snapshot.ayahs, false) {
        out.push((format!("{}:{}", ayah.surah, ayah.ayah), ayah.text.clone()));
        if out.len() >= cap {
            break;
        }
    }
    out
}

fn check_idempotency(snapshot: &Snapshot) -> QuranDoctorCheck {
    const ID: &str = "quran.normalization.idempotency";
    const NEXT: &str = "qai quran normalize --list-profiles";
    let samples = sample_texts(snapshot, 100);
    let mut checked = 0;
    for rule in quran_normalization::rules::all_rules() {
        if !rule.is_idempotent() {
            continue;
        }
        for (location, text) in &samples {
            let once = rule.apply(&quran_normalization::NormalizedText::from_plain(text));
            let twice = rule.apply(&once);
            if once.text() != twice.text() {
                return fail(
                    ID,
                    format!("rule {} not idempotent at {location}", rule.id().as_str()),
                    "fix the rule mapping table; idempotence is a catalog guarantee",
                    NEXT,
                );
            }
        }
        checked += 1;
    }
    pass(ID, format!("{checked} idempotent rules stable over {} samples", samples.len()))
}

fn check_spanmap_sanity(snapshot: &Snapshot) -> QuranDoctorCheck {
    const ID: &str = "quran.normalization.spanmap_sanity";
    const NEXT: &str = "qai quran normalize --list-profiles";
    let registry = super::quran_normalize::builtin_registry();
    let version = quran_normalization::SemVer::new(1, 0, 0);
    let mut checked = 0;
    for profile in [quran_normalization::ProfileId::L3, quran_normalization::ProfileId::L5] {
        let pipeline = match quran_normalization::NormalizationPipeline::for_profile(
            &registry, profile, version,
        ) {
            Ok(pipeline) => pipeline,
            Err(error) => {
                return fail(
                    ID,
                    format!("cannot build {profile} pipeline: {error}"),
                    "re-seed the profile catalog",
                    NEXT,
                );
            }
        };
        for (location, text) in sample_texts(snapshot, 100) {
            let (derived, _) = pipeline.apply(&text);
            // I10 reversibility, per derived char: each maps back to an exact
            // non-empty canonical span that maps forward onto itself. (Full-
            // range hulls legitimately shrink past stripped edges, so the
            // probe is per-char, not whole-text.)
            let dlen = derived.spans().derived_len();
            for i in 0..dlen {
                let span = derived.spans().to_canonical(i..i + 1);
                let back = if span.exact && !span.char_range.is_empty() {
                    derived.spans().to_derived(span.char_range.clone())
                } else {
                    None
                };
                if back.as_ref().is_none_or(|range| !range.contains(&i)) {
                    return fail(
                        ID,
                        format!("{profile} offset round-trip broken at {location} char {i}"),
                        "offset maps must be reversible (I10); fix the rule span construction",
                        NEXT,
                    );
                }
            }
            checked += 1;
        }
    }
    pass(ID, format!("L3/L5 offset round-trips hold over {checked} samples"))
}

fn current_ladder() -> quran_normalization::SemVer {
    let registry = super::quran_normalize::builtin_registry();
    let mut ladder = quran_normalization::SemVer::new(0, 0, 0);
    for id in quran_normalization::ProfileId::all() {
        if let Ok(profile) = registry.latest(id) {
            ladder = ladder.max(profile.version);
        }
    }
    ladder
}

async fn check_forms_current(db: &SqliteDatabase, snapshot: &Snapshot) -> QuranDoctorCheck {
    const ID: &str = "quran.forms.current";
    const NEXT: &str = "qai quran forms rebuild";
    if snapshot.token_total > 0 && snapshot.token_forms == 0 {
        return skipped(
            ID,
            "derived forms not built".to_string(),
            "rebuild derived forms for the active edition",
            NEXT,
        );
    }
    let ladder = current_ladder().to_string();
    let mut checked = 0;
    let mut uow = match db.write().await {
        Ok(uow) => uow,
        Err(error) => {
            return fail(
                ID,
                format!("cannot read derived forms: {error}"),
                "check the database",
                NEXT,
            );
        }
    };
    for ayah in sample_ayahs(&snapshot.ayahs, false).into_iter().take(20) {
        let Ok(tokens) =
            uow.quran().list_token_forms(&snapshot.edition_id, ayah.surah, ayah.ayah).await
        else {
            let _ = uow.rollback().await;
            return fail(
                ID,
                "cannot read derived token forms".to_string(),
                "check the database",
                NEXT,
            );
        };
        for row in &tokens {
            if row.corpus_generation != snapshot.corpus_generation || row.rule_set_version != ladder
            {
                let _ = uow.rollback().await;
                return fail(
                    ID,
                    format!(
                        "stale derived forms at {}:{} (generation {}, rules {})",
                        ayah.surah, ayah.ayah, row.corpus_generation, row.rule_set_version
                    ),
                    "rebuild derived forms after activation or rule changes",
                    NEXT,
                );
            }
            checked += 1;
        }
        if let Ok(Some(row)) =
            uow.quran().get_ayah_form(&snapshot.edition_id, ayah.surah, ayah.ayah).await
            && (row.corpus_generation != snapshot.corpus_generation
                || row.rule_set_version != ladder)
        {
            let _ = uow.rollback().await;
            return fail(
                ID,
                format!(
                    "stale derived ayah form at {}:{} (generation {}, rules {})",
                    ayah.surah, ayah.ayah, row.corpus_generation, row.rule_set_version
                ),
                "rebuild derived forms after activation or rule changes",
                NEXT,
            );
        }
    }
    let _ = uow.rollback().await;
    pass(
        ID,
        format!(
            "{checked} sampled form rows match generation {} + rules {ladder}",
            snapshot.corpus_generation
        ),
    )
}

fn check_forms_coverage(snapshot: &Snapshot) -> QuranDoctorCheck {
    const ID: &str = "quran.forms.coverage";
    const NEXT: &str = "qai quran forms rebuild";
    if snapshot.token_total > 0 && snapshot.token_forms == 0 && snapshot.ayah_forms == 0 {
        return skipped(
            ID,
            "derived forms not built".to_string(),
            "rebuild derived forms for the active edition",
            NEXT,
        );
    }
    let ayah_total = snapshot.ayahs.len() as i64;
    if snapshot.token_forms == snapshot.token_total && snapshot.ayah_forms == ayah_total {
        pass(
            ID,
            format!(
                "every token ({}) and ayah ({ayah_total}) has derived forms",
                snapshot.token_total
            ),
        )
    } else {
        fail(
            ID,
            format!(
                "forms cover {}/{} tokens and {}/{ayah_total} ayahs",
                snapshot.token_forms, snapshot.token_total, snapshot.ayah_forms
            ),
            "rebuild derived forms; partial coverage means stale search fields",
            NEXT,
        )
    }
}

async fn check_canonical_unchanged(db: &SqliteDatabase, snapshot: &Snapshot) -> QuranDoctorCheck {
    const ID: &str = "quran.canonical_unchanged";
    match super::quran_forms::verify_canonical_unchanged(db, &snapshot.edition_id).await {
        Ok(check) if check.unchanged => {
            pass(ID, format!("MV-018: canonical hashes match {}", check.edition_urn))
        }
        Ok(check) => fail(
            ID,
            format!("MV-018: canonical text drifted for {}", check.edition_urn),
            "re-import the edition; canonical rows must never be edited",
            "qai quran validate",
        ),
        Err(error) => fail(
            ID,
            format!("MV-018 probe failed: {error}"),
            "check the database, then re-import if corruption is confirmed",
            "qai quran validate",
        ),
    }
}

async fn check_fts_pointer(
    db: &SqliteDatabase,
    data_dir: &Path,
    _snapshot: &Snapshot,
    id: &'static str,
    index_id: &str,
    absent_summary: String,
    absent_remedy: &str,
) -> QuranDoctorCheck {
    const NEXT: &str = "qai quran index rebuild";
    let mut uow = match db.write().await {
        Ok(uow) => uow,
        Err(error) => {
            return fail(
                id,
                format!("cannot read index catalog: {error}"),
                "check the database",
                NEXT,
            );
        }
    };
    let pointer = match uow.quran().get_index_pointer(index_id).await {
        Ok(pointer) => pointer,
        Err(error) => {
            let _ = uow.rollback().await;
            return fail(
                id,
                format!("cannot read index catalog: {error}"),
                "check the database",
                NEXT,
            );
        }
    };
    let _ = uow.rollback().await;
    let Some(pointer) = pointer else {
        return skipped(id, absent_summary, absent_remedy, NEXT);
    };
    let generation = pointer.generation as u64;
    if !data_dir.join(format!("gen-{generation}")).exists() {
        return fail(
            id,
            format!("index {index_id} pointer dangles: gen-{generation} missing from disk"),
            "rebuild the index; never hand-edit the pointer",
            NEXT,
        );
    }
    let manifest: quran_search::IndexManifest = match serde_json::from_str(&pointer.manifest_json) {
        Ok(manifest) => manifest,
        Err(error) => {
            return fail(
                id,
                format!("index {index_id} manifest corrupt: {error}"),
                "rebuild the index",
                NEXT,
            );
        }
    };
    let registry = super::quran_normalize::builtin_registry();
    let family = match quran_search::TokenizerFamily::new(&registry, manifest.tokenizer_version) {
        Ok(family) => family,
        Err(error) => {
            return fail(
                id,
                format!("index {index_id} tokenizer unusable: {error}"),
                "rebuild the index",
                NEXT,
            );
        }
    };
    let index = match quran_search::Fts5Index::open(data_dir, generation, manifest, family).await {
        Ok(index) => index,
        Err(error) => {
            return fail(
                id,
                format!("index {index_id} gen-{generation} will not open: {error}"),
                "rebuild the index",
                NEXT,
            );
        }
    };
    match quran_search::FullTextIndex::verify(&index).await {
        Ok(report) if report.ok => pass(
            id,
            format!("index {index_id} gen-{generation} healthy: {} docs", report.doc_count),
        ),
        Ok(report) => fail(
            id,
            format!("index {index_id} gen-{generation} FAILED verification: {}", report.doc_count),
            "rebuild the index",
            NEXT,
        ),
        Err(error) => fail(
            id,
            format!("index {index_id} gen-{generation} verification error: {error}"),
            "rebuild the index",
            NEXT,
        ),
    }
}

async fn check_skeleton_index(data_dir: &Path, snapshot: &Snapshot) -> QuranDoctorCheck {
    const ID: &str = "quran.skeleton.index";
    const NEXT: &str = "qai quran index rebuild";
    let Some(pointer) = snapshot.pointer.as_ref() else {
        return skipped(ID, "ayah index not built".to_string(), "build an index generation", NEXT);
    };
    let generation = pointer.generation as u64;
    let path = quran_search::trigram_path(data_dir, generation);
    let mut probes = 0;
    for ayah in sample_ayahs(&snapshot.ayahs, false).into_iter().take(3) {
        let skeleton = quran_search::ayah_skeleton(&ayah.text).skeleton;
        let chars: Vec<char> = skeleton.chars().collect();
        if chars.len() < 10 {
            continue;
        }
        let slice: String = chars[2..10].iter().collect();
        let trigrams = quran_search::trigrams_of(&slice);
        match quran_search::recall(&path, &trigrams).await {
            Ok(Some(set)) => {
                let want = quran_search::SkelAddr {
                    surah: ayah.surah,
                    ayah_start: ayah.ayah,
                    ayah_end: ayah.ayah,
                };
                if !set.contains(&want) {
                    return fail(
                        ID,
                        format!(
                            "trigram recall missed {}:{} in gen-{generation}",
                            ayah.surah, ayah.ayah
                        ),
                        "rebuild the index; postings must recall indexed skeletons",
                        NEXT,
                    );
                }
                probes += 1;
            }
            Ok(None) => {
                return fail(
                    ID,
                    format!("no trigram postings for gen-{generation}"),
                    "rebuild the index with trigram postings",
                    NEXT,
                );
            }
            Err(error) => {
                return fail(
                    ID,
                    format!("trigram recall failed: {error}"),
                    "rebuild the index",
                    NEXT,
                );
            }
        }
    }
    if probes == 0 {
        return skipped(
            ID,
            "no skeleton probes available".to_string(),
            "index an edition with longer ayahs",
            NEXT,
        );
    }
    pass(ID, format!("trigram postings consistent in gen-{generation} ({probes} recall probes)"))
}

fn check_index_drift(snapshot: &Snapshot) -> QuranDoctorCheck {
    const ID: &str = "quran.index.drift";
    const NEXT: &str = "qai quran index rebuild";
    let Some(pointer) = snapshot.pointer.as_ref() else {
        return skipped(ID, "ayah index not built".to_string(), "build an index generation", NEXT);
    };
    let manifest: quran_search::IndexManifest = match serde_json::from_str(&pointer.manifest_json) {
        Ok(manifest) => manifest,
        Err(error) => {
            return fail(
                ID,
                format!("serving manifest corrupt: {error}"),
                "rebuild the index",
                NEXT,
            );
        }
    };
    let mut drifted = Vec::new();
    if manifest.corpus_generation as i64 != snapshot.corpus_generation {
        drifted.push(format!(
            "corpus generation {} served, {} active",
            manifest.corpus_generation, snapshot.corpus_generation
        ));
    }
    let registry = super::quran_normalize::builtin_registry();
    for (profile, version) in &manifest.rule_set_versions {
        let current = registry
            .latest(parse_profile_id(profile))
            .map(|p| p.version.to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        if current != version.to_string() {
            drifted.push(format!("{profile} indexed at {version}, registry at {current}"));
        }
    }
    if drifted.is_empty() {
        pass(
            ID,
            format!(
                "gen-{} inputs match corpus generation {}",
                pointer.generation, snapshot.corpus_generation
            ),
        )
    } else {
        warn(
            ID,
            format!("stale index inputs: {}", drifted.join("; ")),
            "rebuild the index to refresh derived inputs",
            NEXT,
        )
    }
}

fn parse_profile_id(raw: &str) -> quran_normalization::ProfileId {
    quran_normalization::ProfileId::parse(raw).unwrap_or(quran_normalization::ProfileId::L0)
}

fn check_index_orphans(data_dir: &Path, snapshot: &Snapshot) -> QuranDoctorCheck {
    const ID: &str = "quran.index.orphans";
    const NEXT: &str = "qai quran index gc";
    let on_disk: BTreeSet<u64> = match std::fs::read_dir(data_dir) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
            .filter_map(|name| name.strip_prefix("gen-")?.parse::<u64>().ok())
            .collect(),
        Err(_) => BTreeSet::new(),
    };
    let mut expected: BTreeSet<u64> =
        snapshot.runs.iter().map(|run| run.generation as u64).collect();
    if let Some(pointer) = snapshot.pointer.as_ref() {
        expected.insert(pointer.generation as u64);
    }
    let orphans: Vec<u64> = on_disk.difference(&expected).copied().collect();
    if orphans.is_empty() {
        pass(ID, format!("{} on-disk generations all tracked", on_disk.len()))
    } else {
        warn(
            ID,
            format!(
                "orphaned on-disk generations: {}",
                orphans.iter().map(u64::to_string).collect::<Vec<_>>().join(", ")
            ),
            "remove them with retention GC",
            NEXT,
        )
    }
}

fn check_dataset_active(snapshot: &Snapshot) -> QuranDoctorCheck {
    const ID: &str = "quran.morphology.dataset_active";
    const NEXT: &str = "qai quran morphology import";
    if let Some(active) = snapshot.active_dataset.as_ref() {
        return pass(ID, format!("active morphology dataset {}@{}", active.slug, active.version));
    }
    if snapshot.datasets.is_empty() {
        skipped(
            ID,
            "no morphology datasets catalogued (explicit none)".to_string(),
            "import a morphology dataset to enable analysis checks",
            NEXT,
        )
    } else {
        warn(
            ID,
            format!("{} datasets staged but none active", snapshot.datasets.len()),
            "activate exactly one dataset",
            NEXT,
        )
    }
}

/// Token-level morphology probe shared by the alignment/coverage/provenance
/// checks. Reads are bounded (deep = full scan).
struct MorphProbe {
    active: bool,
    error: Option<String>,
    tokens_sampled: usize,
    tokens_covered: usize,
    unmatched_by_surah: BTreeMap<i64, usize>,
    analyses: Vec<TokenAnalysisRow>,
}

async fn morphology_token_probe(
    db: &SqliteDatabase,
    snapshot: &Snapshot,
    deep: bool,
) -> MorphProbe {
    let inactive = MorphProbe {
        active: false,
        error: None,
        tokens_sampled: 0,
        tokens_covered: 0,
        unmatched_by_surah: BTreeMap::new(),
        analyses: Vec::new(),
    };
    let Some(dataset) = snapshot.active_dataset.as_ref() else {
        return inactive;
    };
    let cap = if deep { usize::MAX } else { 2000 };
    let mut probe = MorphProbe {
        active: true,
        error: None,
        tokens_sampled: 0,
        tokens_covered: 0,
        unmatched_by_surah: BTreeMap::new(),
        analyses: Vec::new(),
    };
    let mut uow = match db.write().await {
        Ok(uow) => uow,
        Err(error) => {
            probe.error = Some(error.to_string());
            return probe;
        }
    };
    let ayahs: Vec<(i64, i64)> = snapshot.ayahs.iter().map(|row| (row.surah, row.ayah)).collect();
    for (surah, ayah) in ayahs {
        if probe.tokens_sampled >= cap {
            break;
        }
        let tokens = match uow.quran().get_tokens(&snapshot.edition_id, surah, ayah).await {
            Ok(tokens) => tokens,
            Err(error) => {
                probe.error = Some(error.to_string());
                break;
            }
        };
        for token in tokens {
            if probe.tokens_sampled >= cap {
                break;
            }
            match uow
                .quran()
                .analyses_for_token(
                    Some(&dataset.id),
                    &snapshot.edition_id,
                    surah,
                    ayah,
                    token.position,
                )
                .await
            {
                Ok(rows) => {
                    probe.tokens_sampled += 1;
                    if rows.is_empty() {
                        *probe.unmatched_by_surah.entry(surah).or_insert(0) += 1;
                    } else {
                        probe.tokens_covered += 1;
                        probe.analyses.extend(rows);
                    }
                }
                Err(error) => {
                    probe.error = Some(error.to_string());
                    break;
                }
            }
        }
        if probe.error.is_some() {
            break;
        }
    }
    let _ = uow.rollback().await;
    probe
}

fn check_alignment(_snapshot: &Snapshot, probe: &MorphProbe) -> QuranDoctorCheck {
    const ID: &str = "quran.morphology.alignment";
    const NEXT: &str = "qai quran morphology import";
    if !probe.active {
        return skipped(
            ID,
            "no active morphology dataset".to_string(),
            "import and activate a morphology dataset",
            NEXT,
        );
    }
    if let Some(error) = probe.error.as_ref() {
        return fail(ID, format!("morphology probe failed: {error}"), "check the database", NEXT);
    }
    let unmatched: usize = probe.unmatched_by_surah.values().sum();
    if unmatched == 0 {
        pass(ID, format!("{} sampled tokens all aligned", probe.tokens_sampled))
    } else {
        let mut worst: Vec<(i64, usize)> =
            probe.unmatched_by_surah.iter().map(|(s, n)| (*s, *n)).collect();
        worst.sort_by_key(|b| std::cmp::Reverse(b.1));
        let detail: Vec<String> =
            worst.into_iter().take(3).map(|(s, n)| format!("surah {s}: {n}")).collect();
        fail(
            ID,
            format!("{unmatched} unmatched tokens ({})", detail.join(", ")),
            "activation gates zero unmatched; re-import morphology for the active edition",
            NEXT,
        )
    }
}

fn check_morphology_coverage(_snapshot: &Snapshot, probe: &MorphProbe) -> QuranDoctorCheck {
    const ID: &str = "quran.morphology.coverage";
    const NEXT: &str = "qai quran morphology import";
    if !probe.active {
        return skipped(
            ID,
            "no active morphology dataset".to_string(),
            "import and activate a morphology dataset",
            NEXT,
        );
    }
    if let Some(error) = probe.error.as_ref() {
        return fail(ID, format!("morphology probe failed: {error}"), "check the database", NEXT);
    }
    if probe.tokens_covered == probe.tokens_sampled {
        pass(
            ID,
            format!("{}/{} sampled tokens covered", probe.tokens_covered, probe.tokens_sampled),
        )
    } else {
        fail(
            ID,
            format!("coverage {}/{} sampled tokens", probe.tokens_covered, probe.tokens_sampled),
            "re-import morphology for the active edition",
            NEXT,
        )
    }
}

fn check_morphology_provenance(_snapshot: &Snapshot, probe: &MorphProbe) -> QuranDoctorCheck {
    const ID: &str = "quran.morphology.provenance";
    const NEXT: &str = "qai quran morphology import";
    if !probe.active {
        return skipped(
            ID,
            "no active morphology dataset".to_string(),
            "import and activate a morphology dataset",
            NEXT,
        );
    }
    if let Some(error) = probe.error.as_ref() {
        return fail(ID, format!("morphology probe failed: {error}"), "check the database", NEXT);
    }
    if probe.analyses.is_empty() {
        return warn(
            ID,
            "active dataset holds no analyses".to_string(),
            "re-import morphology for the active edition",
            NEXT,
        );
    }
    let bad = probe
        .analyses
        .iter()
        .filter(|row| row.provenance.layer != "B" && row.provenance.layer != "D")
        .count();
    if bad == 0 {
        pass(ID, format!("{} analyses carry Layer B/D provenance", probe.analyses.len()))
    } else {
        fail(
            ID,
            format!("{bad} analyses lack Layer B/D provenance"),
            "re-import with attributed provenance; unattributed analyses must not serve",
            NEXT,
        )
    }
}

fn check_unverified_layer_d(_snapshot: &Snapshot, probe: &MorphProbe) -> QuranDoctorCheck {
    const ID: &str = "quran.morphology.unverified_layer_d";
    const NEXT: &str = "qai quran morphology import";
    if !probe.active {
        return skipped(
            ID,
            "no active morphology dataset".to_string(),
            "import and activate a morphology dataset",
            NEXT,
        );
    }
    if let Some(error) = probe.error.as_ref() {
        return fail(ID, format!("morphology probe failed: {error}"), "check the database", NEXT);
    }
    if probe.analyses.is_empty() {
        return warn(
            ID,
            "active dataset holds no analyses".to_string(),
            "re-import morphology for the active edition",
            NEXT,
        );
    }
    let pending = probe
        .analyses
        .iter()
        .filter(|row| row.provenance.layer == "D" && row.provenance.status != "human_verified")
        .count();
    pass(ID, format!("{pending} Layer-D analyses await review"))
}

async fn check_lexicon_integrity(
    db: &SqliteDatabase,
    snapshot: &Snapshot,
    probe: &MorphProbe,
) -> QuranDoctorCheck {
    const ID: &str = "quran.lexicon.integrity";
    const NEXT: &str = "qai quran morphology import";
    let Some(dataset) = snapshot.active_dataset.as_ref() else {
        return skipped(
            ID,
            "no active morphology dataset".to_string(),
            "import and activate a morphology dataset",
            NEXT,
        );
    };
    if let Some(error) = probe.error.as_ref() {
        return fail(ID, format!("morphology probe failed: {error}"), "check the database", NEXT);
    }
    let mut uow = match db.write().await {
        Ok(uow) => uow,
        Err(error) => {
            return fail(ID, format!("cannot read lexicon: {error}"), "check the database", NEXT);
        }
    };
    let roots: BTreeSet<String> = match uow.quran().list_roots(&dataset.id).await {
        Ok(rows) => rows.into_iter().map(|row| row.id).collect(),
        Err(error) => {
            let _ = uow.rollback().await;
            return fail(
                ID,
                format!("cannot read lexicon roots: {error}"),
                "check the database",
                NEXT,
            );
        }
    };
    let lemmas: BTreeSet<String> = match uow.quran().list_lemmas(&dataset.id).await {
        Ok(rows) => rows.into_iter().map(|row| row.id).collect(),
        Err(error) => {
            let _ = uow.rollback().await;
            return fail(
                ID,
                format!("cannot read lexicon lemmas: {error}"),
                "check the database",
                NEXT,
            );
        }
    };
    let analyses = if probe.analyses.is_empty() {
        // Deep scans audit the whole dataset, not just the token sample.
        match uow.quran().list_analyses(&dataset.id).await {
            Ok(rows) => rows,
            Err(error) => {
                let _ = uow.rollback().await;
                return fail(
                    ID,
                    format!("cannot read lexicon analyses: {error}"),
                    "check the database",
                    NEXT,
                );
            }
        }
    } else {
        probe.analyses.clone()
    };
    let _ = uow.rollback().await;
    let mut dangling = 0;
    let mut examples = Vec::new();
    for row in &analyses {
        for (label, id) in [("lemma", row.lemma_id.as_ref()), ("root", row.root_id.as_ref())] {
            if let Some(id) = id {
                let known = if label == "lemma" { lemmas.contains(id) } else { roots.contains(id) };
                if !known {
                    dangling += 1;
                    if examples.len() < 3 {
                        examples.push(format!("{label} {id}"));
                    }
                }
            }
        }
    }
    if dangling == 0 {
        pass(ID, format!("{} analyses reference live roots/lemmas", analyses.len()))
    } else {
        fail(
            ID,
            format!("{dangling} dangling references ({})", examples.join(", ")),
            "re-import morphology; analyses must never outlive their lexicon rows",
            NEXT,
        )
    }
}

fn check_unification_queue(snapshot: &Snapshot) -> QuranDoctorCheck {
    const ID: &str = "quran.lexicon.root_unification_queue";
    let pending = snapshot
        .pending_reviews
        .iter()
        .filter(|item| item.kind == "root_unification" && item.status == "pending")
        .count();
    pass(ID, format!("{pending} pending root-unification suggestions"))
}

fn smoke_search_params(text: String) -> super::quran_search::SearchParams {
    super::quran_search::SearchParams {
        text,
        edition: None,
        mode: super::quran_search::MatchMode::WholeToken,
        filters: Vec::new(),
        limit: 100,
        offset: 0,
        explain: false,
        highlight: false,
    }
}

async fn check_search_smoke(
    db: &SqliteDatabase,
    data_dir: &Path,
    snapshot: &Snapshot,
) -> QuranDoctorCheck {
    const ID: &str = "quran.search.smoke";
    const NEXT: &str = "qai quran index rebuild";
    if snapshot.pointer.is_none() {
        return skipped(ID, "ayah index not built".to_string(), "build an index generation", NEXT);
    }
    if snapshot.ayahs.is_empty() {
        return skipped(ID, "corpus has no ayahs".to_string(), "import an edition", NEXT);
    }
    let pinned = |surah: i64, ayah: i64| {
        format!("quran:{}@{}:{surah}:{ayah}", snapshot.edition_slug, snapshot.edition_version)
    };
    // Deterministic probes from corpus content: longest tokens (rarest hits)
    // for exact/normalized, longest ayahs for concatenated skeletons.
    let mut tokens: Vec<(usize, String, i64, i64)> = Vec::new();
    {
        let mut uow = match db.write().await {
            Ok(uow) => uow,
            Err(error) => {
                return fail(
                    ID,
                    format!("cannot read tokens: {error}"),
                    "check the database",
                    NEXT,
                );
            }
        };
        for ayah in sample_ayahs(&snapshot.ayahs, false) {
            match uow.quran().get_tokens(&snapshot.edition_id, ayah.surah, ayah.ayah).await {
                Ok(rows) => {
                    for row in rows {
                        tokens.push((
                            row.surface.chars().count(),
                            row.surface.clone(),
                            ayah.surah,
                            ayah.ayah,
                        ));
                    }
                }
                Err(error) => {
                    let _ = uow.rollback().await;
                    return fail(
                        ID,
                        format!("cannot read tokens: {error}"),
                        "check the database",
                        NEXT,
                    );
                }
            }
        }
        let _ = uow.rollback().await;
    }
    tokens.sort_by(|a, b| b.0.cmp(&a.0).then(a.2.cmp(&b.2)).then(a.3.cmp(&b.3)));
    tokens.dedup_by(|a, b| a.1 == b.1);
    tokens.truncate(4);
    let registry = super::quran_normalize::builtin_registry();
    let l3 = match quran_normalization::NormalizationPipeline::for_profile(
        &registry,
        quran_normalization::ProfileId::L3,
        quran_normalization::SemVer::new(1, 0, 0),
    ) {
        Ok(pipeline) => pipeline,
        Err(error) => {
            return fail(
                ID,
                format!("cannot build L3 pipeline: {error}"),
                "re-seed the profile catalog",
                NEXT,
            );
        }
    };
    let mut probes: Vec<(String, String, String)> = Vec::new();
    for (kind, surface, surah, ayah) in tokens.iter().take(4) {
        let _ = kind;
        let expected = pinned(*surah, *ayah);
        probes.push((format!("exact:{surface}"), surface.clone(), expected.clone()));
        let derived = l3.apply(surface).0;
        if !derived.text().trim().is_empty() {
            probes.push((
                format!("normalized:{}", derived.text()),
                derived.text().to_string(),
                expected,
            ));
        }
    }
    let mut ayahs: Vec<&storage::quran::AyahRow> = snapshot.ayahs.iter().collect();
    ayahs.sort_by_key(|b| std::cmp::Reverse(b.text.chars().count()));
    for ayah in ayahs.iter().take(8) {
        if probes.len() >= 12 {
            break;
        }
        let skeleton = quran_search::ayah_skeleton(&ayah.text).skeleton;
        let chars: Vec<char> = skeleton.chars().collect();
        if chars.len() < 10 {
            continue;
        }
        let slice: String = chars[2..10].iter().collect();
        probes.push((format!("concatenated:{slice}"), slice, pinned(ayah.surah, ayah.ayah)));
    }
    if probes.is_empty() {
        return skipped(
            ID,
            "no smoke probes derivable".to_string(),
            "index an edition with longer ayahs",
            NEXT,
        );
    }
    let mut missed = Vec::new();
    for (index, (label, query, expected)) in probes.iter().enumerate() {
        let tag = format!("P{}", index + 1);
        let hits: Result<Vec<String>, String> = if label.starts_with("exact:") {
            super::quran_search::search_exact(
                db,
                data_dir,
                &smoke_search_params(query.clone()),
                super::quran_search::ExactField::TextExact,
            )
            .await
            .map(|out| out.hits.iter().map(|hit| hit.reference().to_string()).collect())
            .map_err(|error| error.to_string())
        } else if label.starts_with("normalized:") {
            super::quran_search::search_normalized(
                db,
                data_dir,
                &smoke_search_params(query.clone()),
                super::quran_search::NormalizedProfile::Registry(
                    quran_normalization::ProfileId::L3,
                    Some(quran_normalization::SemVer::new(1, 0, 0)),
                ),
            )
            .await
            .map(|out| out.hits.iter().map(|hit| hit.reference().to_string()).collect())
            .map_err(|error| error.to_string())
        } else {
            super::quran_search::search_concatenated(
                db,
                data_dir,
                &smoke_search_params(query.clone()),
                false,
                1,
            )
            .await
            .map(|out| out.hits.iter().map(|hit| hit.reference().to_string()).collect())
            .map_err(|error| error.to_string())
        };
        match hits {
            Ok(refs) if refs.iter().any(|reference| reference == expected) => {}
            Ok(_) => missed.push(format!("{tag} missed {label} (expected {expected})")),
            Err(error) => missed.push(format!("{tag} errored on {label}: {error}")),
        }
    }
    if missed.is_empty() {
        pass(
            ID,
            format!(
                "{}/{} smoke probes hit (exact + normalized + concatenated)",
                probes.len(),
                probes.len()
            ),
        )
    } else {
        fail(
            ID,
            format!("smoke probes missed: {}", missed.join("; ")),
            "tokenizer/profile regression: rebuild the index; if a green build still misses, fix the pipeline",
            NEXT,
        )
    }
}
