//! Phase 3 — alpha end-to-end evidence (plan 03-08, D-03 / SC1–SC5).
//!
//! One `#[tokio::test]` walks the entire D-03 alpha on the synthetic fixture
//! through the REAL services:
//!
//! migrate → import edition → activate → `rebuild_forms` → `rebuild_index` →
//! import + activate a morphology dataset with valid license evidence →
//! `search_exact` / `search_normalized` / `search_phrase` /
//! `search_concatenated` / `search_regex` → `morphology_for_token` →
//! `root_search` / `lemma_search` → `word_family` → `frequency` / `distribution`
//! / `cooccurrence` / `root_frequency` / `lemma_frequency`.
//!
//! Every step surfaces its contract in the assertion messages so a reviewer sees
//! the alpha evidence in one run:
//!
//! - every search hit carries the mandatory normalization trace (I9), the exact
//!   canonical span (I10), a pinned reference, and a quotation that is
//!   byte-identical to the canonical row (SC5, verified quotation);
//! - concatenated hits additionally carry a non-empty segmentation (SC2);
//! - every lexicon result carries dataset attribution (SC3);
//! - every numeric report carries a complete `CountingRules` block, and the
//!   lexicon reports name the active dataset (I15 / SC4).
//!
//! **Synthetic only.** The fixture is `synthetic: true` and the lexicon is
//! `synthetic_test_only`; this proves mechanical behavior, never scholarly
//! ground truth. The linguist/dataset ratification stays BLOCKED (OD-11/OD-12,
//! `docs/05-followups/phase-03-owner-gates.md`).

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use application::quran::activate_edition;
use application::quran_counting::{
    CountingRules, MultiAnalysisHandling, cooccurrence, distribution, frequency, lemma_frequency,
    root_frequency,
};
use application::quran_forms::{RebuildParams, rebuild_forms};
use application::quran_index::{IndexBuildParams, QURAN_AYAH_INDEX_ID, rebuild_index};
use application::quran_morphology::{
    MorphologyActivateParams, MorphologyImportParams, activate_morphology, build_affix_relations,
    build_derived_relations, build_inflectional_relations, build_same_form_relations,
    build_same_lemma_relations, build_same_root_relations, build_same_stem_relations, dataset_urn,
    lemma_search, morphology_for_token, root_search, run_morphology_import, word_family,
};
use application::quran_reader::{QuranReader, QuranReaderService};
use application::quran_search::{
    ExactField, MatchMode, NormalizedProfile, PhraseMode, RateLimiter, SearchOutput, SearchParams,
    search_concatenated, search_exact, search_normalized, search_phrase, search_regex,
};
use domain::{PrincipalId, Timestamp};
use quran_core::{AyahNumber, AyahOptions, EditionSelector, QuranRef, SurahNumber};
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use quran_corpus::sha256_hex;
use quran_normalization::{ProfileId, SemVer};
use storage::Database as _;
use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

const BASE_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const PRINCIPAL: &str = "00000000-0000-0000-0000-000000000001";
const CREATED_AT: &str = "2026-09-14T00:00:00Z";
const V1_URN: &str = "quran-edition:test-edition-min@0.1.0";
const SLUG: &str = "test-edition-min";
const VERSION: &str = "0.1.0";
const MORPH_SLUG: &str = "test-morph";
const MORPH_VERSION: &str = "0.1.0";
/// The import run id doubles as the canonical `edition_id`; the reader parses it
/// as a UUID, so it must be a valid UUID (the `alpha_smoke` harness convention).
const RUN_ID: &str = "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee";

/// The typed family relation kinds (ADR-0210); every built relation must be one.
const TYPED_RELATIONS: [&str; 7] =
    ["same_form", "same_lemma", "same_stem", "same_root", "derived", "inflectional", "affix"];

fn principal() -> PrincipalId {
    PRINCIPAL.parse().unwrap()
}

fn timestamp() -> Timestamp {
    Timestamp::from_ymd_hms(2026, 9, 14, 0, 0, 0).unwrap()
}

/// The whole alpha chain up to (and including) an active synthetic lexicon.
///
/// Harness pattern (proven across 03-01…03-07): tempdir + repo migrations +
/// `SqliteDatabase::new(path, 4, true)` + raw seed rows (principal, source,
/// source-version, edition approval, dataset approval), then the real services.
async fn alpha_db() -> (tempfile::TempDir, Arc<SqliteDatabase>, std::path::PathBuf) {
    let dir = tempdir().unwrap();
    let path = dir.path().join("qai.db");
    let path_str = path.to_str().unwrap().to_string();
    let repo_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    storage_sqlite::migrate::apply_migrations(&path_str, &repo_root).await.unwrap();
    let db = Arc::new(SqliteDatabase::new(&path_str, 4, true).await.unwrap());
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

    // 1. import + activate the edition.
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

    // 2. forms rebuild → index rebuild.
    rebuild_forms(
        &db,
        &RebuildParams {
            edition_slug: SLUG.to_string(),
            edition_version: VERSION.to_string(),
            invoked_by: PRINCIPAL.to_string(),
            run_tag: "alpha-e2e-forms".to_string(),
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
            run_tag: "alpha-e2e-index".to_string(),
            data_dir: data_dir.clone(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("index build completes");

    // 3. morphology import + approval-gated activation with license evidence.
    let doc = lexicon_document(&db).await;
    run_morphology_import(&db, &morph_import_params(doc), &AtomicBool::new(false), |_| {})
        .await
        .expect("synthetic lexicon import completes");
    activate_morphology(
        &db,
        &MorphologyActivateParams {
            batch_id: "batch-alpha-e2e".to_string(),
            approval_id: "appr-morph".to_string(),
            invoked_by: PRINCIPAL.to_string(),
        },
        &principal(),
    )
    .await
    .expect("synthetic lexicon activates");

    // 4. build every typed family relation the SC3 surface reads.
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

/// Build an aligned array-shape synthetic lexicon over real fixture tokens.
///
/// Two competing analyses per token (multi-analysis reads, no winner), with
/// every typed family key populated (`k % 9` lemma, `k % 3` root — 9 is a
/// multiple of 3 so a lemma maps to exactly one root, satisfying
/// `quran_lemmas` UNIQUE(dataset_id, lemma)), plus declared prefix/suffix.
/// Every row is `synthetic_test_only: true`.
async fn lexicon_document(db: &SqliteDatabase) -> String {
    let mut uow = db.write().await.unwrap();
    let active = uow.quran().get_active().await.unwrap().unwrap();
    let ayahs = uow.quran().list_ayahs_range(&active.edition_id, 1, i64::MAX).await.unwrap();
    let mut rows = Vec::new();
    let mut index = 0usize;
    for ayah in &ayahs {
        let tokens =
            uow.quran().get_tokens(&active.edition_id, ayah.surah, ayah.ayah).await.unwrap();
        for token in &tokens {
            let root = format!("root-{}", index % 3);
            let lemma = format!("lem-{}", index % 9);
            let stem = format!("stem-{}", index % 5);
            let prefix = format!("pre-{}", index % 4);
            let suffix = format!("suf-{}", index % 3);
            for analysis_no in [0u32, 1u32] {
                rows.push(serde_json::json!({
                    "sura_no": ayah.surah,
                    "aya_no": ayah.ayah,
                    "tok_idx": token.position,
                    "analysis_no": analysis_no,
                    "surface_form": token.surface,
                    "lemma_str": lemma,
                    "root_str": root,
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

fn morph_import_params(document_text: String) -> MorphologyImportParams {
    MorphologyImportParams {
        dataset_slug: MORPH_SLUG.to_string(),
        dataset_version: MORPH_VERSION.to_string(),
        adapter: "json".to_string(),
        document_text,
        edition_slug: SLUG.to_string(),
        edition_version: VERSION.to_string(),
        invoked_by: PRINCIPAL.to_string(),
        batch_id: Some("batch-alpha-e2e".to_string()),
        attribution: "synthetic test import (not scholarly data)".to_string(),
        license_status: "PublicDomain".to_string(),
        // Gate 6 (D-07): mandatory capture fields + `redistribution_allowed`.
        license_json: r#"{"source_url":"https://example.invalid/qai-synthetic-test-lexicon","capture_date":"2026-09-28","capturer":"qai-test-fixtures","spdx_id":"CC0-1.0","redistribution_allowed":true,"modification_allowed":true,"attribution_required":false}"#.to_string(),
    }
}

fn params(text: &str, mode: MatchMode) -> SearchParams {
    SearchParams {
        text: text.to_string(),
        edition: None,
        mode,
        filters: vec![],
        limit: 100,
        offset: 0,
        explain: false,
        highlight: false,
    }
}

/// Assert the full result contract on one search output (trace, canonical span,
/// pinned reference, byte-identical verified quotation — SC5).
async fn assert_search_contract(mode: &str, out: &SearchOutput, reader: &QuranReaderService) {
    assert!(
        out.total_matches >= 1,
        "alpha[{mode}]: expected >= 1 hit, got total_matches={} hits={}",
        out.total_matches,
        out.hits.len()
    );
    assert!(!out.hits.is_empty(), "alpha[{mode}]: hits present alongside total_matches");
    for hit in &out.hits {
        let trace = hit.explanation();
        assert!(
            !trace.profile.is_empty(),
            "alpha[{mode}]: hit {} must carry the mandatory trace (I9)",
            hit.reference()
        );
        if !trace.profile.starts_with("L0") {
            // L0.exact legitimately applies no rules; every other profile must
            // disclose its ordered rule set (I9).
            assert!(
                !trace.rules_applied.is_empty(),
                "alpha[{mode}]: hit {} must disclose its ordered rule set (I9); trace={trace:?}",
                hit.reference()
            );
        }
        let text = hit.quotation().arabic_text();
        let bytes = hit.byte_range().unwrap_or_else(|| {
            panic!("alpha[{mode}]: hit {} span must map to bytes (I10)", hit.reference())
        });
        assert!(
            bytes.start < bytes.end,
            "alpha[{mode}]: hit {} canonical span must be non-empty",
            hit.reference()
        );
        assert!(
            (bytes.end as usize) <= text.len(),
            "alpha[{mode}]: hit {} span must lie inside the canonical text",
            hit.reference()
        );
        assert!(
            hit.reference().starts_with(&format!("quran:{SLUG}@{VERSION}:")),
            "alpha[{mode}]: hit reference must be a pinned canonical reference, got {}",
            hit.reference()
        );

        // SC5: independently resolve the canonical row and pin byte identity.
        let q = hit.quotation();
        let reference = QuranRef::Ayah {
            edition: EditionSelector::Pinned {
                slug: SLUG.to_string(),
                version: SemVer::new(0, 1, 0),
            },
            surah: SurahNumber::new(q.surah_number().get()).expect("valid surah"),
            ayah: AyahNumber::new(q.ayah_range().0.get()).expect("valid ayah"),
        };
        let view = reader
            .get_ayah(&reference, &AyahOptions::default())
            .await
            .expect("canonical ayah resolves");
        assert_eq!(
            text.as_bytes(),
            view.canonical.arabic_text().as_bytes(),
            "alpha[{mode}]: hit {} displayed text must be byte-identical to the canonical row (SC5)",
            hit.reference()
        );
        assert_eq!(
            q.text_hash(),
            view.canonical.text_hash(),
            "alpha[{mode}]: hit {} must carry the canonical stored hash, never recomputed",
            hit.reference()
        );
    }
}

/// A numeric report must carry a complete `CountingRules`, and (for lexicon
/// reports) name the active dataset.
fn assert_rules_complete(label: &str, rules: &CountingRules, expected_dataset: Option<&str>) {
    assert!(!rules.profile.is_empty(), "alpha[{label}]: rules.profile must be set");
    assert!(!rules.profile_version.is_empty(), "alpha[{label}]: rules.profile_version must be set");
    assert!(
        !rules.datasets.is_empty(),
        "alpha[{label}]: rules.datasets must name the consulted source(s)"
    );
    if let Some(dataset) = expected_dataset {
        assert!(
            rules.datasets.iter().any(|d| d == dataset),
            "alpha[{label}]: rules.datasets must name the active dataset {dataset}, got {:?}",
            rules.datasets
        );
    }
}

/// The whole D-03 alpha, one command.
#[tokio::test]
async fn alpha_end_to_end_synthetic() {
    let (_dir, db, data_dir) = alpha_db().await;
    let reader = QuranReaderService::new(db.clone());
    let dataset = format!("{MORPH_SLUG}@{MORPH_VERSION}");
    let version = SemVer::new(1, 0, 0);
    let limiter = RateLimiter::new(10_000);

    // ── SC1: exact + normalized search ──────────────────────────────────────
    let exact =
        search_exact(&db, &data_dir, &params("أوَيت", MatchMode::Substring), ExactField::TextExact)
            .await
            .expect("search_exact runs");
    assert_search_contract("exact", &exact, &reader).await;

    let normalized = search_normalized(
        &db,
        &data_dir,
        &params("أويت", MatchMode::Substring),
        NormalizedProfile::Registry(ProfileId::L3, Some(version)),
    )
    .await
    .expect("search_normalized runs");
    assert_search_contract("normalized", &normalized, &reader).await;

    // ── SC2: phrase + concatenated (spaceless) ──────────────────────────────
    let phrase = search_phrase(
        &db,
        &data_dir,
        &params("أوَيت تَساكُن", MatchMode::WholeToken),
        NormalizedProfile::Registry(ProfileId::L0, Some(version)),
        PhraseMode::OrderedExact,
        0,
    )
    .await
    .expect("search_phrase runs");
    assert_search_contract("phrase", &phrase, &reader).await;

    let concatenated =
        search_concatenated(&db, &data_dir, &params("اويتتساكن", MatchMode::Substring), false, 3)
            .await
            .expect("search_concatenated runs");
    assert_search_contract("concatenated", &concatenated, &reader).await;
    for hit in &concatenated.hits {
        assert!(
            !hit.segmentation().is_empty(),
            "alpha[concatenated]: hit {} must carry an explainable segmentation (SC2)",
            hit.reference()
        );
        for part in hit.segmentation() {
            assert!(
                !part.query_part.is_empty() && part.canonical_token >= 1,
                "alpha[concatenated]: every segment must name a query part + 1-based token"
            );
        }
    }

    // ── regex search ────────────────────────────────────────────────────────
    let regex = search_regex(
        &db,
        &data_dir,
        &params("^أو", MatchMode::WholeToken),
        "text_exact",
        "^أو",
        PRINCIPAL,
        &limiter,
        3000,
    )
    .await
    .expect("search_regex runs");
    assert_search_contract("regex", &regex, &reader).await;

    // ── SC3: token morphology, root/lemma search, word family ───────────────
    let (morph_dataset, analyses) =
        morphology_for_token(&db, SLUG, VERSION, 1, 1, 1).await.expect("morphology_for_token runs");
    assert_eq!(morph_dataset, dataset, "alpha[morphology]: dataset attribution");
    assert!(
        !analyses.is_empty(),
        "alpha[morphology]: an active dataset must serve >= 1 analysis for token 1:1:1"
    );
    let analyses_json = serde_json::to_string(&analyses).unwrap();
    for banned in ["winner", "is_correct", "is_primary", "selected"] {
        assert!(
            !analyses_json.contains(banned),
            "alpha[morphology]: competing analyses must never elect a '{banned}'"
        );
    }

    let (root_dataset, root_occurrences) =
        root_search(&db, "root-0").await.expect("root_search runs");
    assert_eq!(root_dataset, dataset, "alpha[root_search]: dataset attribution");
    assert!(!root_occurrences.is_empty(), "alpha[root_search]: root-0 must have occurrences");
    assert!(
        root_occurrences.iter().all(|o| o.dataset == dataset),
        "alpha[root_search]: every occurrence must carry the active dataset"
    );

    let (lemma_dataset, lemma_occurrences) =
        lemma_search(&db, "lem-0").await.expect("lemma_search runs");
    assert_eq!(lemma_dataset, dataset, "alpha[lemma_search]: dataset attribution");
    assert!(!lemma_occurrences.is_empty(), "alpha[lemma_search]: lem-0 must have occurrences");
    assert!(
        lemma_occurrences.iter().all(|o| o.dataset == dataset),
        "alpha[lemma_search]: every occurrence must carry the active dataset"
    );

    let (family_dataset, members) =
        word_family(&db, "token", "token:1:1:1").await.expect("word_family runs");
    assert_eq!(family_dataset, dataset, "alpha[word_family]: dataset attribution");
    assert!(!members.is_empty(), "alpha[word_family]: token:1:1:1 must have family members");
    for member in &members {
        assert!(
            TYPED_RELATIONS.contains(&member.relation.as_str()),
            "alpha[word_family]: relation {} must be a typed relation",
            member.relation
        );
        assert!(
            !member.explanation.trim().is_empty(),
            "alpha[word_family]: every relation must carry an explanation"
        );
        assert_eq!(
            member.dataset.as_deref(),
            Some(dataset.as_str()),
            "alpha[word_family]: every member must carry dataset attribution"
        );
    }

    // ── SC4: frequency / distribution / co-occurrence (stored forms) ────────
    let freq = frequency(&db, "كن", "L3.diacritics").await.expect("frequency runs");
    assert!(freq.count > 0, "alpha[frequency]: the target must occur in the fixture");
    assert_rules_complete("frequency", &freq.rules, None);
    assert_eq!(
        freq.by_surah.values().sum::<u64>(),
        freq.count,
        "alpha[frequency]: breakdown sums to the total"
    );

    let dist = distribution(&db, "كن", "L3.diacritics").await.expect("distribution runs");
    assert_rules_complete("distribution", &dist.frequency.rules, None);
    assert!(!dist.warnings.is_empty(), "alpha[distribution]: single-source warning required");

    let (cooc_rules, cooc_hits) =
        cooccurrence(&db, "كن", "L3.diacritics", 3, 10).await.expect("cooccurrence runs");
    assert_rules_complete("cooccurrence", &cooc_rules, None);
    assert_eq!(
        cooc_rules.window.as_deref(),
        Some("token:3"),
        "alpha[cooccurrence]: window declared"
    );
    assert!(cooc_hits.len() <= 10, "alpha[cooccurrence]: respect the limit");

    // ── SC4: lexicon root/lemma frequency (active dataset) ─────────────────
    let root_freq =
        root_frequency(&db, "root-0", "L3.diacritics", MultiAnalysisHandling::AllAnalyses)
            .await
            .expect("root_frequency runs over the active lexicon");
    assert!(
        root_freq.count > 0,
        "alpha[root_frequency]: an active lexicon must yield a non-zero count"
    );
    assert_rules_complete("root_frequency", &root_freq.rules, Some(&dataset));
    assert_eq!(
        root_freq.by_surah.values().sum::<u64>(),
        root_freq.count,
        "alpha[root_frequency]: breakdown sums to the total"
    );

    let lemma_freq =
        lemma_frequency(&db, "lem-0", "L3.diacritics", MultiAnalysisHandling::AllAnalyses)
            .await
            .expect("lemma_frequency runs over the active lexicon");
    assert!(
        lemma_freq.count > 0,
        "alpha[lemma_frequency]: an active lexicon must yield a non-zero count"
    );
    assert_rules_complete("lemma_frequency", &lemma_freq.rules, Some(&dataset));
    assert_eq!(
        lemma_freq.by_surah.values().sum::<u64>(),
        lemma_freq.count,
        "alpha[lemma_frequency]: breakdown sums to the total"
    );
}
