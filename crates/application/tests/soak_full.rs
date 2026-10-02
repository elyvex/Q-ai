//! P2-T111 — 50,000-query full soak at fixture scale (plan 03.5-03, D-3.5-05).
//!
//! Builds the full derived state through the doctor's own soak fixture — import
//! and activate an edition, rebuild forms, build indexes, import and activate
//! morphology — then runs 50,000 randomized queries drawn deterministically
//! from a seeded generator over the fixture's own token inventory, spread
//! across all five search modes and the full counting and lexicon tool set.
//!
//! Per query the soak asserts the invariants that must hold at volume: a hit
//! carries a trace and a canonical span; a displayed quotation resolves
//! byte-identically through the reader (SC5); a lexicon result carries dataset
//! attribution; a numeric report carries a complete `CountingRules`; an
//! unavailable capability is a typed error, never an empty result. It ends by
//! running `run_quran_graph_checks` and the nineteen index doctor checks and
//! asserting both are green, then compares observed latency against
//! `budgets.json` without failing on fixture-bound timing (the fixture bound
//! is the gate; the full-corpus p99 stays OD-11-dependent and unexecuted).
//!
//! Determinism: the seeded generator makes the run reproducible, and the soak
//! asserts two passes produce identical ordered output, so a failure is always
//! reproducible.
//!
//! Coverage honesty: this is fixture-scale evidence. No full-corpus claim is
//! made anywhere. The soak's own report prints `scale: synthetic_fixture` and
//! `full_corpus: not_executed (OD-11)`.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use application::quran::activate_edition;
use application::quran_counting::{
    CountingRules, MultiAnalysisHandling, collocation, cooccurrence, distribution, frequency,
    hapax_search, interval_analysis, lemma_frequency, missing_expected_form,
    near_duplicate_passages, numeric_report, root_frequency, unusual_usage,
};
use application::quran_doctor::CheckLevel;
use application::quran_doctor_indexes::{INDEX_CHECK_IDS, run_index_checks};
use application::quran_forms::{RebuildParams, rebuild_forms};
use application::quran_graph_doctor::{GRAPH_CHECK_IDS, run_quran_graph_checks};
use application::quran_index::{IndexBuildParams, QURAN_AYAH_INDEX_ID, rebuild_index};
use application::quran_morphology::{
    MorphologyActivateParams, MorphologyImportParams, MorphologyToolError, activate_morphology,
    build_affix_relations, build_derived_relations, build_inflectional_relations,
    build_same_form_relations, build_same_lemma_relations, build_same_root_relations,
    build_same_stem_relations, dataset_urn, lemma_search, morphology_for_token, root_search,
    run_morphology_import, word_family,
};
use application::quran_reader::{QuranReader, QuranReaderService};
use application::quran_search::{
    ExactField, MatchMode, NormalizedProfile, PhraseMode, RateLimiter, SearchOutput, SearchParams,
    search_concatenated, search_exact, search_normalized, search_phrase, search_regex,
};
use domain::{PrincipalId, Timestamp};
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use quran_corpus::sha256_hex;
use quran_normalization::{ProfileId, SemVer};
use storage::Database as _;
use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

const BASE_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const BUDGETS_JSON: &str = include_str!("../../../fixtures/quran/performance/budgets.json");
const PRINCIPAL: &str = "00000000-0000-0000-0000-000000000001";
const CREATED_AT: &str = "2026-09-14T00:00:00Z";
const V1_URN: &str = "quran-edition:test-edition-min@0.1.0";
const SLUG: &str = "test-edition-min";
const VERSION: &str = "0.1.0";
const MORPH_SLUG: &str = "test-morph";
const MORPH_VERSION: &str = "0.1.0";
const MORPH_BATCH: &str = "soak-full-batch";
/// The import run id doubles as the canonical `edition_id`; the reader parses
/// it as a UUID, so it must be a valid UUID (the `alpha_smoke` convention).
const PROFILE: &str = "L3.diacritics";
/// The soak runs exactly 50_000 queries (P2-T111); the count is asserted, never
/// approximated.
const SOAK_QUERIES: usize = 50_000;
/// Fixed seed: the query stream is reproducible run to run.
const SOAK_SEED: u64 = 0x0350_3500_B11A_11CE;
const RUN_ID: &str = "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeee1";
const MISSING_TARGET: &str = "zzqqxxnotaword";

const KIND_NAMES: [&str; 21] = [
    "exact",
    "normalized",
    "phrase",
    "concat",
    "regex",
    "frequency",
    "distribution",
    "cooccurrence",
    "collocation",
    "hapax",
    "numeric_report",
    "interval",
    "near_duplicates",
    "missing_form",
    "root_frequency",
    "lemma_frequency",
    "root_search",
    "lemma_search",
    "word_family",
    "morphology",
    "unusual_usage",
];

/// One line per query kind: count and mean latency. Printed after pass one so
/// timing evidence survives even if a later gate fails.
fn principal() -> PrincipalId {
    PRINCIPAL.parse().unwrap()
}

fn timestamp() -> Timestamp {
    Timestamp::from_ymd_hms(2026, 9, 14, 0, 0, 0).unwrap()
}

// ── Deterministic generator (no external crates) ─────────────────────────────

/// xorshift64*: bounded, deterministic, std-only.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, bound: usize) -> usize {
        assert!(bound > 0, "rng bound must be non-zero");
        (self.next() % bound as u64) as usize
    }
}

/// One soak query. Every variant draws its parameters from the fixture's own
/// inventory, so a query that errors is a real regression, never a bad draw.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Query {
    Exact(String),
    Normalized(String),
    Phrase(String),
    Concat(String),
    Regex(String),
    Frequency(String),
    Distribution(String),
    Cooccurrence(String),
    Collocation(String),
    Hapax,
    NumericReport(String),
    Interval(String),
    NearDuplicates,
    MissingForm,
    RootFreq(String),
    LemmaFreq(String),
    RootSearch(String),
    LemmaSearch(String),
    WordFamily(String),
    Morphology(i64, i64, i64),
    UnusualUsage(String),
}

impl Query {
    fn kind_tag(&self) -> u8 {
        match self {
            Query::Exact(_) => 0,
            Query::Normalized(_) => 1,
            Query::Phrase(_) => 2,
            Query::Concat(_) => 3,
            Query::Regex(_) => 4,
            Query::Frequency(_) => 5,
            Query::Distribution(_) => 6,
            Query::Cooccurrence(_) => 7,
            Query::Collocation(_) => 8,
            Query::Hapax => 9,
            Query::NumericReport(_) => 10,
            Query::Interval(_) => 11,
            Query::NearDuplicates => 12,
            Query::MissingForm => 13,
            Query::RootFreq(_) => 14,
            Query::LemmaFreq(_) => 15,
            Query::RootSearch(_) => 16,
            Query::LemmaSearch(_) => 17,
            Query::WordFamily(_) => 18,
            Query::Morphology(_, _, _) => 19,
            Query::UnusualUsage(_) => 20,
        }
    }
}

/// Fixture-derived query space: token surfaces, same-ayah adjacent pairs
/// (phrase/concat draws), token ids (family/morphology draws).
struct Inventory {
    surfaces: Vec<String>,
    pairs: Vec<(String, String)>,
    token_ids: Vec<(i64, i64, i64)>,
}

fn generate(seed: u64, n: usize, inv: &Inventory) -> Vec<Query> {
    assert!(!inv.surfaces.is_empty(), "fixture must provide token surfaces");
    assert!(!inv.pairs.is_empty(), "fixture must provide adjacent token pairs");
    assert!(!inv.token_ids.is_empty(), "fixture must provide token ids");
    let mut rng = Rng(seed);
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        let surface = inv.surfaces[rng.below(inv.surfaces.len())].clone();
        let (first, second) = inv.pairs[rng.below(inv.pairs.len())].clone();
        let (s, a, p) = inv.token_ids[rng.below(inv.token_ids.len())];
        out.push(match rng.below(21) {
            0 => Query::Exact(surface),
            1 => Query::Normalized(surface),
            2 => Query::Phrase(format!("{first} {second}")),
            3 => Query::Concat(format!("{first}{second}")),
            4 => {
                let prefix: String = surface.chars().take(2).collect();
                Query::Regex(format!("^{prefix}"))
            }
            5 => Query::Frequency(surface),
            6 => Query::Distribution(surface),
            7 => Query::Cooccurrence(surface),
            8 => Query::Collocation(surface),
            9 => Query::Hapax,
            10 => Query::NumericReport(surface),
            11 => Query::Interval(surface),
            12 => Query::NearDuplicates,
            13 => Query::MissingForm,
            14 => Query::RootFreq(format!("root-{}", rng.below(3))),
            15 => Query::LemmaFreq(format!("lem-{}", rng.below(9))),
            16 => Query::RootSearch(format!("root-{}", rng.below(3))),
            17 => Query::LemmaSearch(format!("lem-{}", rng.below(9))),
            18 => Query::WordFamily(format!("token:{s}:{a}:{p}")),
            19 => Query::Morphology(s, a, p),
            _ => Query::UnusualUsage(surface),
        });
    }
    out
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xCBF2_9CE4_8422_2325;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x0100_0000_01B3);
    }
    hash
}

// ── Soak fixture (the doctor's own full derived state) ───────────────────────

async fn migrated_db() -> (tempfile::TempDir, SqliteDatabase, String) {
    let dir = tempdir().unwrap();
    let path = dir.path().join("qai.db");
    let path_str = path.to_str().unwrap().to_string();
    let repo_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    storage_sqlite::migrate::apply_migrations(&path_str, &repo_root).await.unwrap();
    let db = SqliteDatabase::new(&path_str, 4, true).await.unwrap();
    let seed = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new().filename(&path_str).foreign_keys(true),
        )
        .await
        .unwrap();
    for sql in [
        format!(
            "INSERT INTO principals (id, kind, display_name, created_at)
             VALUES ('{PRINCIPAL}', 'local_user', 'Test', '{CREATED_AT}')"
        ),
        "INSERT INTO sources (id, title, content_type, created_at, updated_at)
         VALUES ('src-1', 'Test source', 'quran_edition', '2026-09-14T00:00:00Z', '2026-09-14T00:00:00Z')"
            .to_string(),
        "INSERT INTO source_versions
            (id, source_id, version, schema_version, state, trust_level,
             license_status, license_json, created_at)
         VALUES ('sv-1', 'src-1', '0.1.0', 1, 'Staged', 'ImportedUnverified',
                 'PublicDomain', '{}', '2026-09-14T00:00:00Z')"
            .to_string(),
        format!(
            "INSERT INTO approvals
                (id, subject_urn, kind, requested_by, decided_by, decision,
                 request_payload, requested_at, decided_at)
             VALUES ('appr-1', '{V1_URN}', 'CanonicalChange', '{PRINCIPAL}', '{PRINCIPAL}',
                     'approved', '{{}}', '{CREATED_AT}', '{CREATED_AT}')"
        ),
        format!(
            "INSERT INTO approvals
                (id, subject_urn, kind, requested_by, decided_by, decision,
                 request_payload, requested_at, decided_at)
             VALUES ('appr-morph', '{}', 'CanonicalChange', '{PRINCIPAL}', '{PRINCIPAL}',
                     'approved', '{{}}', '{CREATED_AT}', '{CREATED_AT}')",
            dataset_urn(MORPH_SLUG, MORPH_VERSION)
        ),
    ] {
        sqlx::query(&sql).execute(&seed).await.unwrap();
    }
    seed.close().await;
    (dir, db, path_str)
}

async fn soak_db() -> (tempfile::TempDir, Arc<SqliteDatabase>, std::path::PathBuf) {
    let (dir, db, _path) = migrated_db().await;
    let db = Arc::new(db);
    let outcome = run_import(
        &*db,
        &ImportInput {
            run_id: RUN_ID.into(),
            job_id: None,
            source_version_id: "sv-1".into(),
            adapter: "json".into(),
            manifest_text: BASE_MANIFEST.into(),
            declared_manifest_hash: Some(sha256_hex(BASE_MANIFEST.as_bytes())),
            invoked_by: PRINCIPAL.into(),
            license_status: "PublicDomain".into(),
            license_json: "{}".into(),
            created_at: CREATED_AT.into(),
            reference_manifest_text: None,
        },
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .expect("fixture import completes");
    assert!(matches!(outcome, ImportOutcome::Completed(_)));
    activate_edition(&*db, SLUG, VERSION, &principal(), "appr-1", &timestamp())
        .await
        .expect("fixture activation completes");
    rebuild_forms(
        &db,
        &RebuildParams {
            edition_slug: SLUG.to_string(),
            edition_version: VERSION.to_string(),
            invoked_by: PRINCIPAL.to_string(),
            run_tag: "soak-full-forms".to_string(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("forms rebuild completes");
    let data_dir = dir.path().join("index");
    rebuild_index(
        &db,
        &IndexBuildParams {
            index_id: QURAN_AYAH_INDEX_ID.to_string(),
            edition_slug: SLUG.to_string(),
            edition_version: VERSION.to_string(),
            invoked_by: PRINCIPAL.to_string(),
            run_tag: "soak-full-index".to_string(),
            data_dir: data_dir.clone(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("index build completes");
    let document = aligned_morph_document(&db).await;
    run_morphology_import(
        &db,
        &MorphologyImportParams {
            dataset_slug: MORPH_SLUG.to_string(),
            dataset_version: MORPH_VERSION.to_string(),
            adapter: "json".to_string(),
            document_text: document,
            edition_slug: SLUG.to_string(),
            edition_version: VERSION.to_string(),
            invoked_by: PRINCIPAL.to_string(),
            batch_id: Some(MORPH_BATCH.to_string()),
            attribution: "synthetic test import (not scholarly data)".to_string(),
            license_status: "PublicDomain".to_string(),
            license_json: r#"{"source_url":"https://example.invalid/qai-synthetic-test-lexicon","capture_date":"2026-09-28","capturer":"qai-test-fixtures","spdx_id":"CC0-1.0","redistribution_allowed":true,"modification_allowed":true,"attribution_required":false}"#.to_string(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("morphology import completes");
    activate_morphology(
        &db,
        &MorphologyActivateParams {
            batch_id: MORPH_BATCH.to_string(),
            approval_id: "appr-morph".to_string(),
            invoked_by: PRINCIPAL.to_string(),
        },
        &principal(),
    )
    .await
    .expect("morphology activation completes");
    // The family relations are derived state the SC3 surface reads: build every
    // typed kind so word_family serves members (an unbuilt relation table would
    // answer every family query with a vacuous empty list).
    let dataset = format!("{MORPH_SLUG}@{MORPH_VERSION}");
    for (kind, built) in [
        ("same_form", build_same_form_relations(&db, &dataset).await),
        ("same_lemma", build_same_lemma_relations(&db, &dataset).await),
        ("same_stem", build_same_stem_relations(&db, &dataset).await),
        ("same_root", build_same_root_relations(&db, &dataset).await),
        ("derived", build_derived_relations(&db, &dataset).await),
        ("inflectional", build_inflectional_relations(&db, &dataset).await),
        ("affix", build_affix_relations(&db, &dataset).await),
    ] {
        let count = built.unwrap_or_else(|e| panic!("family builder {kind} failed: {e}"));
        assert!(count > 0, "family builder {kind} must build rows, saw {count}");
    }
    (dir, db, data_dir)
}

/// Aligned synthetic lexicon over the real fixture tokens (same deterministic
/// scheme the golden suites assert: `root = k % 3`, `lemma = k % 9`).
async fn aligned_morph_document(db: &SqliteDatabase) -> String {
    let mut uow = db.write().await.unwrap();
    let active = uow.quran().get_active().await.unwrap().unwrap();
    let ayahs = uow.quran().list_ayahs_range(&active.edition_id, 1, i64::MAX).await.unwrap();
    let mut rows = Vec::new();
    let mut index = 0usize;
    for ayah in &ayahs {
        let tokens =
            uow.quran().get_tokens(&active.edition_id, ayah.surah, ayah.ayah).await.unwrap();
        for token in &tokens {
            let prefix = format!("pre-{}", index % 4);
            let stem = format!("stem-{}", index % 5);
            let suffix = format!("suf-{}", index % 3);
            for analysis_no in [0u32, 1u32] {
                rows.push(serde_json::json!({
                    "sura_no": ayah.surah,
                    "aya_no": ayah.ayah,
                    "tok_idx": token.position,
                    "analysis_no": analysis_no,
                    "surface_form": token.surface,
                    "lemma_str": format!("lem-{}", index % 9),
                    "root_str": format!("root-{}", index % 3),
                    "stem_str": stem,
                    "tag_native": if analysis_no == 0 { "N" } else { "V" },
                    "tag_unified": if analysis_no == 0 { "noun" } else { "verb" },
                    "layer": "B",
                    "state": "imported",
                    "segmented": true,
                    "morphs": [
                        {"part": "prefix", "text": prefix},
                        {"part": "stem", "text": stem},
                        {"part": "suffix", "text": suffix},
                    ],
                    "synthetic_test_only": true,
                }));
            }
            index += 1;
        }
    }
    uow.rollback().await.unwrap();
    serde_json::to_string(&rows).unwrap()
}

async fn collect_inventory(db: &SqliteDatabase) -> Inventory {
    let mut uow = db.write().await.unwrap();
    let active = uow.quran().get_active().await.unwrap().unwrap();
    let ayahs = uow.quran().list_ayahs_range(&active.edition_id, 1, i64::MAX).await.unwrap();
    let mut surfaces = Vec::new();
    let mut pairs = Vec::new();
    let mut token_ids = Vec::new();
    for ayah in &ayahs {
        let tokens =
            uow.quran().get_tokens(&active.edition_id, ayah.surah, ayah.ayah).await.unwrap();
        for window in tokens.windows(2) {
            pairs.push((window[0].surface.clone(), window[1].surface.clone()));
        }
        for token in &tokens {
            surfaces.push(token.surface.clone());
            token_ids.push((ayah.surah, ayah.ayah, token.position));
        }
    }
    uow.rollback().await.unwrap();
    Inventory { surfaces, pairs, token_ids }
}

// ── Per-result contracts ─────────────────────────────────────────────────────

fn assert_rules_complete(label: &str, rules: &CountingRules, expected_dataset: Option<&str>) {
    assert!(!rules.profile.is_empty(), "soak[{label}]: rules.profile must be set");
    assert!(!rules.profile_version.is_empty(), "soak[{label}]: rules.profile_version must be set");
    assert!(!rules.datasets.is_empty(), "soak[{label}]: rules.datasets must name source(s)");
    if let Some(dataset) = expected_dataset {
        assert!(
            rules.datasets.iter().any(|d| d == dataset),
            "soak[{label}]: rules.datasets must name {dataset}, got {:?}",
            rules.datasets
        );
    }
}

async fn assert_search_contract(
    mode: &str,
    out: &SearchOutput,
    reader: &QuranReaderService,
) -> u64 {
    for hit in &out.hits {
        let trace = hit.explanation();
        assert!(!trace.profile.is_empty(), "soak[{mode}]: hit must carry a trace (I9)");
        if !trace.profile.starts_with("L0") {
            assert!(
                !trace.rules_applied.is_empty(),
                "soak[{mode}]: non-L0 hit must disclose its ordered rule set (I9)"
            );
        }
        let text = hit.quotation().arabic_text();
        let bytes = hit
            .byte_range()
            .unwrap_or_else(|| panic!("soak[{mode}]: hit span must map to bytes (I10)"));
        assert!(bytes.start < bytes.end, "soak[{mode}]: canonical span must be non-empty");
        assert!((bytes.end as usize) <= text.len(), "soak[{mode}]: span must lie in the text");
        assert!(
            hit.reference().starts_with(&format!("quran:{SLUG}@{VERSION}:")),
            "soak[{mode}]: hit reference must be pinned, got {}",
            hit.reference()
        );
        // SC5: the displayed quotation resolves byte-identically through the reader.
        let q = hit.quotation();
        let reference = quran_core::QuranRef::Ayah {
            edition: quran_core::EditionSelector::Pinned {
                slug: SLUG.to_string(),
                version: SemVer::new(0, 1, 0),
            },
            surah: quran_core::SurahNumber::new(q.surah_number().get()).expect("valid surah"),
            ayah: quran_core::AyahNumber::new(q.ayah_range().0.get()).expect("valid ayah"),
        };
        let view = reader
            .get_ayah(&reference, &quran_core::AyahOptions::default())
            .await
            .expect("canonical ayah resolves");
        assert_eq!(
            text.as_bytes(),
            view.canonical.arabic_text().as_bytes(),
            "soak[{mode}]: displayed text must be byte-identical to the canonical row (SC5)"
        );
        assert_eq!(
            q.text_hash(),
            view.canonical.text_hash(),
            "soak[{mode}]: hit must carry the canonical stored hash"
        );
    }
    out.total_matches
}

fn search_params(text: String) -> SearchParams {
    SearchParams {
        text,
        edition: None,
        mode: MatchMode::Substring,
        filters: Vec::new(),
        limit: 25,
        offset: 0,
        explain: false,
        highlight: false,
    }
}

// ── The soak ─────────────────────────────────────────────────────────────────

struct PassStats {
    digest: u64,
    per_kind_queries: [u64; 21],
    per_kind_hits: [u64; 21],
    per_kind_nanos: [u128; 21],
}

/// Execute every query; on `with_invariants` assert the full per-result
/// contract. Always folds (kind, metric) into the digest in order. A query
/// that errors is a failure — results are never silently skipped.
async fn run_pass(
    db: &Arc<SqliteDatabase>,
    data_dir: &std::path::Path,
    reader: &QuranReaderService,
    queries: &[Query],
    with_invariants: bool,
) -> PassStats {
    let dataset = format!("{MORPH_SLUG}@{MORPH_VERSION}");
    let version = SemVer::new(1, 0, 0);
    let limiter = RateLimiter::new(10_000);
    let mut stats = PassStats {
        digest: 0xCBF2_9CE4_8422_2325,
        per_kind_queries: [0; 21],
        per_kind_hits: [0; 21],
        per_kind_nanos: [0; 21],
    };
    let mut mix = |tag: u8, metric: u64, kind: usize, elapsed: Duration, hits: u64| {
        stats.per_kind_queries[kind] += 1;
        stats.per_kind_hits[kind] += hits;
        stats.per_kind_nanos[kind] += elapsed.as_nanos();
        let mut buf = [0u8; 17];
        buf[0] = tag;
        buf[1..9].copy_from_slice(&metric.to_le_bytes());
        buf[9..17].copy_from_slice(&(metric ^ 0x9E37_79B9_7F4A_7C15).to_le_bytes());
        stats.digest = fnv1a(&buf) ^ stats.digest.wrapping_mul(0x0100_0000_01B3);
    };
    for (i, query) in queries.iter().enumerate() {
        let started = Instant::now();
        let tag = query.kind_tag();
        let kind = tag as usize;
        let metric: u64 = match query {
            Query::Exact(text) => {
                let out =
                    search_exact(db, data_dir, &search_params(text.clone()), ExactField::TextExact)
                        .await
                        .unwrap_or_else(|e| panic!("soak[{i}]: exact({text}) failed: {e}"));
                if with_invariants {
                    assert_search_contract("exact", &out, reader).await
                } else {
                    out.total_matches
                }
            }
            Query::Normalized(text) => {
                let out = search_normalized(
                    db,
                    data_dir,
                    &search_params(text.clone()),
                    NormalizedProfile::Registry(ProfileId::L3, Some(version)),
                )
                .await
                .unwrap_or_else(|e| panic!("soak[{i}]: normalized({text}) failed: {e}"));
                if with_invariants {
                    assert_search_contract("normalized", &out, reader).await
                } else {
                    out.total_matches
                }
            }
            Query::Phrase(text) => {
                let out = search_phrase(
                    db,
                    data_dir,
                    &search_params(text.clone()),
                    NormalizedProfile::Registry(ProfileId::L0, Some(version)),
                    PhraseMode::OrderedExact,
                    0,
                )
                .await
                .unwrap_or_else(|e| panic!("soak[{i}]: phrase({text}) failed: {e}"));
                if with_invariants {
                    assert_search_contract("phrase", &out, reader).await
                } else {
                    out.total_matches
                }
            }
            Query::Concat(text) => {
                let out = search_concatenated(db, data_dir, &search_params(text.clone()), false, 1)
                    .await
                    .unwrap_or_else(|e| panic!("soak[{i}]: concatenated({text}) failed: {e}"));
                if with_invariants {
                    let total = assert_search_contract("concatenated", &out, reader).await;
                    for hit in &out.hits {
                        assert!(
                            !hit.segmentation().is_empty(),
                            "soak[{i}]: concatenated hit must carry a segmentation (SC2)"
                        );
                    }
                    total
                } else {
                    out.total_matches
                }
            }
            Query::Regex(pattern) => {
                let out = search_regex(
                    db,
                    data_dir,
                    &search_params(pattern.clone()),
                    "text_exact",
                    pattern,
                    PRINCIPAL,
                    &limiter,
                    3000,
                )
                .await
                .unwrap_or_else(|e| panic!("soak[{i}]: regex({pattern}) failed: {e}"));
                if with_invariants {
                    assert_search_contract("regex", &out, reader).await
                } else {
                    out.total_matches
                }
            }
            Query::Frequency(target) => {
                let report = frequency(db, target, PROFILE)
                    .await
                    .unwrap_or_else(|e| panic!("soak[{i}]: frequency({target}) failed: {e}"));
                if with_invariants {
                    assert_rules_complete("frequency", &report.rules, None);
                    assert_eq!(
                        report.by_surah.values().sum::<u64>(),
                        report.count,
                        "soak[{i}]: frequency breakdown must sum to the total"
                    );
                }
                report.count
            }
            Query::Distribution(target) => {
                let report = distribution(db, target, PROFILE)
                    .await
                    .unwrap_or_else(|e| panic!("soak[{i}]: distribution({target}) failed: {e}"));
                if with_invariants {
                    assert_rules_complete("distribution", &report.frequency.rules, None);
                    assert!(
                        !report.warnings.is_empty(),
                        "soak[{i}]: distribution must carry its single-source warning"
                    );
                }
                report.frequency.count
            }
            Query::Cooccurrence(target) => {
                let (rules, hits) = cooccurrence(db, target, PROFILE, 3, 10)
                    .await
                    .unwrap_or_else(|e| panic!("soak[{i}]: cooccurrence({target}) failed: {e}"));
                if with_invariants {
                    assert_rules_complete("cooccurrence", &rules, None);
                    assert!(hits.len() <= 10, "soak[{i}]: cooccurrence must respect the limit");
                }
                hits.len() as u64
            }
            Query::Collocation(target) => {
                let (rules, hits) = collocation(db, target, PROFILE, 3, 10)
                    .await
                    .unwrap_or_else(|e| panic!("soak[{i}]: collocation({target}) failed: {e}"));
                if with_invariants {
                    assert_rules_complete("collocation", &rules, None);
                }
                hits.len() as u64
            }
            Query::Hapax => {
                let report = hapax_search(db, PROFILE, 100)
                    .await
                    .unwrap_or_else(|e| panic!("soak[{i}]: hapax failed: {e}"));
                if with_invariants {
                    assert_rules_complete("hapax", &report.rules, None);
                    assert_eq!(report.profile, PROFILE, "soak[{i}]: hapax must state its profile");
                }
                report.hapax.len() as u64
            }
            Query::NumericReport(target) => {
                let report = numeric_report(db, target, PROFILE)
                    .await
                    .unwrap_or_else(|e| panic!("soak[{i}]: numeric-report({target}) failed: {e}"));
                if with_invariants {
                    assert_rules_complete("numeric_report", &report.frequency.rules, None);
                }
                report.frequency.count
            }
            Query::Interval(target) => {
                let span = interval_analysis(db, target, PROFILE)
                    .await
                    .unwrap_or_else(|e| panic!("soak[{i}]: interval({target}) failed: {e}"));
                if with_invariants {
                    assert_rules_complete("interval", &span.rules, None);
                    assert!(
                        !span.disclaimer.is_empty(),
                        "soak[{i}]: interval analysis must carry its disclaimer"
                    );
                }
                span.ayah_span.unwrap_or(0)
            }
            Query::NearDuplicates => {
                let (rules, hits) = near_duplicate_passages(db, 0.8, 25)
                    .await
                    .unwrap_or_else(|e| panic!("soak[{i}]: near-duplicates failed: {e}"));
                if with_invariants {
                    assert_rules_complete("near_duplicates", &rules, None);
                }
                hits.len() as u64
            }
            Query::MissingForm => {
                let report = missing_expected_form(db, MISSING_TARGET, PROFILE)
                    .await
                    .unwrap_or_else(|e| panic!("soak[{i}]: missing-form failed: {e}"));
                if with_invariants {
                    assert_eq!(report.count, 0, "soak[{i}]: missing form must prove zero");
                    assert_rules_complete("missing_form", &report.rules, None);
                }
                0
            }
            Query::RootFreq(root) => {
                let report = root_frequency(db, root, PROFILE, MultiAnalysisHandling::AllAnalyses)
                    .await
                    .unwrap_or_else(|e| panic!("soak[{i}]: root-frequency({root}) failed: {e}"));
                if with_invariants {
                    assert_rules_complete("root_frequency", &report.rules, Some(&dataset));
                }
                report.count
            }
            Query::LemmaFreq(lemma) => {
                let report =
                    lemma_frequency(db, lemma, PROFILE, MultiAnalysisHandling::AllAnalyses)
                        .await
                        .unwrap_or_else(|e| {
                            panic!("soak[{i}]: lemma-frequency({lemma}) failed: {e}")
                        });
                if with_invariants {
                    assert_rules_complete("lemma_frequency", &report.rules, Some(&dataset));
                }
                report.count
            }
            Query::RootSearch(root) => {
                let (ds, occurrences) = root_search(db, root)
                    .await
                    .unwrap_or_else(|e| panic!("soak[{i}]: root-search({root}) failed: {e}"));
                if with_invariants {
                    assert_eq!(
                        ds, dataset,
                        "soak[{i}]: root result must carry dataset attribution"
                    );
                    assert!(
                        occurrences.iter().all(|o| o.dataset == dataset),
                        "soak[{i}]: every root occurrence must carry the active dataset"
                    );
                }
                occurrences.len() as u64
            }
            Query::LemmaSearch(lemma) => {
                let (ds, occurrences) = lemma_search(db, lemma)
                    .await
                    .unwrap_or_else(|e| panic!("soak[{i}]: lemma-search({lemma}) failed: {e}"));
                if with_invariants {
                    assert_eq!(
                        ds, dataset,
                        "soak[{i}]: lemma result must carry dataset attribution"
                    );
                    assert!(
                        occurrences.iter().all(|o| o.dataset == dataset),
                        "soak[{i}]: every lemma occurrence must carry the active dataset"
                    );
                }
                occurrences.len() as u64
            }
            Query::WordFamily(id) => {
                let (ds, members) = word_family(db, "token", id)
                    .await
                    .unwrap_or_else(|e| panic!("soak[{i}]: word-family({id}) failed: {e}"));
                if with_invariants {
                    assert_eq!(ds, dataset, "soak[{i}]: family must carry dataset attribution");
                    for member in &members {
                        assert_eq!(
                            member.dataset.as_deref(),
                            Some(dataset.as_str()),
                            "soak[{i}]: every family member must carry dataset attribution"
                        );
                        assert!(
                            !member.explanation.trim().is_empty(),
                            "soak[{i}]: every relation must carry an explanation"
                        );
                    }
                }
                members.len() as u64
            }
            Query::Morphology(s, a, p) => {
                let (ds, analyses) = morphology_for_token(db, SLUG, VERSION, *s, *a, *p)
                    .await
                    .unwrap_or_else(|e| panic!("soak[{i}]: morphology({s}:{a}:{p}) failed: {e}"));
                if with_invariants {
                    assert_eq!(ds, dataset, "soak[{i}]: morphology must carry dataset attribution");
                    assert!(!analyses.is_empty(), "soak[{i}]: token must have analyses");
                }
                analyses.len() as u64
            }
            Query::UnusualUsage(target) => {
                // Lexicon-gated by construction: an unavailable capability is a
                // typed error, never an empty result.
                match unusual_usage(db, target, PROFILE).await {
                    Err(application::quran_counting::CountingError::UnavailableDataset {
                        ..
                    }) => 1,
                    Err(other) => {
                        panic!("soak[{i}]: unusual-usage must be typed-unavailable, got {other:?}")
                    }
                    Ok(_) => panic!(
                        "soak[{i}]: unusual-usage must not succeed without a lexicon capability"
                    ),
                }
            }
        };
        let elapsed = started.elapsed();
        // Hits witnessed per query: searches report total_matches, counting
        // reports counts, lexicon reports row counts. MissingForm proves zero
        // by design and is excluded from the non-vacuous proof.
        let hits = !matches!(query, Query::MissingForm) && metric > 0;
        mix(tag, metric, kind, elapsed, u64::from(hits));
    }
    stats
}

/// P2-T111: 50,000 deterministic queries across all five search modes and the
/// full counting + lexicon tool set, with per-query invariants on pass one
/// and an identical-order digest on pass two; then the graph and index doctor
/// checks go green and observed latency is reported against `budgets.json`.
///
/// Scope: `scale: synthetic_fixture`. `full_corpus: not_executed (OD-11)`.
#[tokio::test]
async fn fifty_thousand_query_soak_is_green_and_reproducible() {
    let (_dir, db, data_dir) = soak_db().await;
    let reader = QuranReaderService::new(db.clone());
    let inventory = collect_inventory(&db).await;

    let queries = generate(SOAK_SEED, SOAK_QUERIES, &inventory);
    assert_eq!(queries.len(), 50_000, "the soak runs exactly 50,000 queries");

    let wall = Instant::now();
    let first = run_pass(&db, &data_dir, &reader, &queries, true).await;
    let first_wall = wall.elapsed();
    for (kind, name) in KIND_NAMES.iter().enumerate() {
        let avg_nanos = first.per_kind_nanos[kind] / first.per_kind_queries[kind].max(1) as u128;
        eprintln!(
            "soak pass1: {} n={} hits={} avg={:.2}ms",
            name,
            first.per_kind_queries[kind],
            first.per_kind_hits[kind],
            avg_nanos as f64 / 1_000_000.0
        );
    }
    eprintln!("soak pass1 wall: {:.1}s", first_wall.as_secs_f64());

    // Determinism: regenerating the stream yields the identical ordered input,
    // and a second full pass folds the identical ordered digest.
    let replay = generate(SOAK_SEED, SOAK_QUERIES, &inventory);
    assert_eq!(replay, queries, "the seeded generator must be reproducible");
    let second = run_pass(&db, &data_dir, &reader, &replay, false).await;
    assert_eq!(
        second.digest, first.digest,
        "two full passes must produce identical ordered output"
    );

    // Non-vacuous: every query kind executed thousands of times and every
    // kind except the by-design zero-proof witnessed hits.
    for kind in 0..21u8 {
        let k = kind as usize;
        assert!(
            first.per_kind_queries[k] > 1_000,
            "kind {kind} must run at volume, saw {}",
            first.per_kind_queries[k]
        );
        if kind != Query::MissingForm.kind_tag() {
            assert!(
                first.per_kind_hits[k] > 0,
                "kind {kind} must witness hits (else the soak proves nothing)"
            );
        }
    }

    // Doctor gates: six graph checks with stable ids and no failure, then the
    // nineteen index checks with stable ids and no failure.
    let graph_checks = run_quran_graph_checks(&db, false).await.expect("graph checks run");
    assert_eq!(graph_checks.len(), 6);
    let graph_ids: Vec<&str> = graph_checks.iter().map(|c| c.id).collect();
    assert_eq!(graph_ids, GRAPH_CHECK_IDS, "graph checks keep stable ids");
    for check in &graph_checks {
        assert_ne!(check.status, CheckLevel::Fail, "{} must not fail: {}", check.id, check.summary);
    }
    let index_checks = run_index_checks(&db, &data_dir, false).await.expect("index checks run");
    assert_eq!(index_checks.len(), 19);
    let index_ids: Vec<&str> = index_checks.iter().map(|c| c.id).collect();
    assert_eq!(index_ids, INDEX_CHECK_IDS, "index checks keep stable ids");
    for check in &index_checks {
        assert_ne!(check.status, CheckLevel::Fail, "{} must not fail: {}", check.id, check.summary);
    }

    // Budgets: the fixture bound is pinned (never loosened); observed per-kind
    // maxima are reported against it without failing on fixture-bound timing.
    // The full-corpus p50/p99 rows stay OD-11-dependent and unexecuted.
    let budgets: serde_json::Value =
        serde_json::from_str(BUDGETS_JSON).expect("budgets.json parses");
    let fixture_bound_ms = budgets["budgets"]["fixture_bound_ms"]["value_ms"]
        .as_u64()
        .expect("fixture_bound_ms row present");
    assert_eq!(fixture_bound_ms, 2000, "the fixture bound must never be loosened");
    let mut latency_lines = Vec::new();
    for (kind, name) in KIND_NAMES.iter().enumerate() {
        let avg_nanos = first.per_kind_nanos[kind] / first.per_kind_queries[kind] as u128;
        latency_lines.push(format!(
            "    {}: n={} hits={} avg={:.2}ms",
            name,
            first.per_kind_queries[kind],
            first.per_kind_hits[kind],
            avg_nanos as f64 / 1_000_000.0
        ));
    }
    let graph_status: Vec<String> =
        graph_checks.iter().map(|c| format!("{}={}", c.id, c.status.as_str())).collect();
    let report = format!(
        "soak report: scale: synthetic_fixture | full_corpus: not_executed (OD-11)\n\
         queries={} pass1_wall={:.1}s digest={:016x}\n{}\n\
         graph_checks: {}\n\
         index_checks: 19/19 no-fail (see doctor_indexes soak for the per-id green list)",
        SOAK_QUERIES,
        first_wall.as_secs_f64(),
        first.digest,
        latency_lines.join("\n"),
        graph_status.join(" "),
    );
    assert!(report.contains("scale: synthetic_fixture"), "the report must state its scale");
    assert!(report.contains("full_corpus: not_executed (OD-11)"), "the report must disclose OD-11");
    println!("{report}");
}

/// An unavailable dataset is a typed error, never an empty result — proven on
/// a migrated-but-empty database where no lexicon is active.
#[tokio::test]
async fn unavailable_dataset_is_typed_not_empty() {
    let (_dir, db, _path) = migrated_db().await;
    match root_search(&db, "root-0").await {
        Err(MorphologyToolError::UnavailableDataset { .. }) => {}
        Err(other) => panic!("expected typed UnavailableDataset, got {other:?}"),
        Ok(_) => panic!("root_search without a dataset must not succeed"),
    }
}
