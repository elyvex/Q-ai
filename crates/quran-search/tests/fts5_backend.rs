//! Phase 2 — FTS5 backend acceptance (P2-T30, real FTS, tempdirs, no mocks).
//!
//! Every suite here runs the real engine: stage → add_batch → commit →
//! search/count/stats/verify/delete. The normalization suites prove both
//! paths route through the shared family (a bypass on either side fails
//! them by construction).

use std::collections::BTreeMap;

use quran_search::{
    CommitStamp, Diagnostic as _, FieldId, Filter, FtsBackend, FtsDoc, FtsQuery, FtsSchema,
    FullTextIndex, Fts5Index, IndexManifest, ResultOrder, SearchOpts, SemVer, TokenizerFamily,
};
use quran_normalization::{ProfileId, ProfileRegistry};

fn v1() -> SemVer {
    SemVer::new(1, 0, 0)
}

fn rule_versions() -> BTreeMap<String, SemVer> {
    [
        ProfileId::L0,
        ProfileId::L1,
        ProfileId::L2,
        ProfileId::L3,
        ProfileId::L4,
        ProfileId::L5,
        ProfileId::L7,
    ]
    .iter()
    .map(|id| (id.as_str().to_string(), v1()))
    .collect()
}

fn manifest(doc_count: u64) -> IndexManifest {
    IndexManifest {
        index_id: "test.ayah.v1".to_string(),
        schema_version: 1,
        corpus_generation: 7,
        edition_id: "ed-1".to_string(),
        edition_version: v1(),
        rule_set_versions: rule_versions(),
        tokenizer_version: v1(),
        morphology_dataset_versions: BTreeMap::new(),
        built_at: "2026-09-15T00:00:00Z".to_string(),
        doc_count,
        content_hash: "sha256:test".to_string(),
    }
}

fn family() -> TokenizerFamily {
    TokenizerFamily::new(&ProfileRegistry::new(), v1()).unwrap()
}

/// One ayah doc with `raw` under every text field (the adapter normalizes).
fn doc(id: &str, surah: u16, ayah: u16, global: u64, raw: &str) -> FtsDoc {
    let fields: BTreeMap<FieldId, String> = [
        "text_exact",
        "text_ws",
        "text_marks",
        "text_bare",
        "text_hamza",
        "text_folded",
        "text_affix",
    ]
    .iter()
    .map(|f| ((*f).to_string(), raw.to_string()))
    .collect();
    FtsDoc {
        id: id.to_string(),
        edition_id: "ed-1".to_string(),
        surah,
        ayah,
        global_index: global,
        generation: 7,
        metadata: BTreeMap::from([
            ("juz".to_string(), "1".to_string()),
            ("page".to_string(), "1".to_string()),
            ("revelation".to_string(), "makki".to_string()),
        ]),
        fields,
    }
}

fn schema() -> FtsSchema {
    FtsSchema {
        index_id: "test.ayah.v1".to_string(),
        schema_version: 1,
        fields: [
            "text_exact",
            "text_ws",
            "text_marks",
            "text_bare",
            "text_hamza",
            "text_folded",
            "text_affix",
        ]
        .iter()
        .map(|s| (*s).to_string())
        .collect(),
    }
}

async fn staged(
    docs: Vec<FtsDoc>,
) -> (tempfile::TempDir, Fts5Index, IndexManifest, CommitStamp) {
    let dir = tempfile::tempdir().unwrap();
    let manifest = manifest(docs.len() as u64);
    let index = Fts5Index::stage(dir.path(), manifest.clone(), family()).await.unwrap();
    index.create(&schema()).await.unwrap();
    index.add_batch(docs).await.unwrap();
    let stamp = index.commit().await.unwrap();
    assert_eq!(stamp.doc_count, manifest.doc_count);
    assert_eq!(stamp.generation, 7);
    (dir, index, manifest, stamp)
}

fn term(field: &str, term: &str) -> FtsQuery {
    FtsQuery::Term { field: field.to_string(), term: term.to_string() }
}

#[tokio::test]
async fn term_round_trip_count_stats_verify() {
    // Note: wasla (ٱ) folds only at L4, so bare-field data below uses bare
    // alef; wasla coverage lives on `text_hamza` in `normalization_on_both_paths`.
    let docs = vec![
        doc("d1", 1, 1, 1, "بِسْمِ اللَّهِ"),
        doc("d2", 1, 2, 2, "ٱلْحَمْدُ لِلَّهِ"),
        doc("d3", 112, 1, 100, "قُلْ هُوَ اللَّهُ أَحَدٌ"),
    ];
    let (_dir, index, manifest, _) = staged(docs).await;
    assert_eq!(index.backend(), FtsBackend::Fts5);
    assert_eq!(index.manifest(), manifest);

    let results = index.search(&term("text_bare", "الله"), &SearchOpts::default()).await.unwrap();
    assert_eq!(results.total_matches, 2);
    assert_eq!(results.hits.len(), 2);
    assert!(!results.truncated);
    // Canonical order: 1:1 before 112:1.
    assert_eq!(results.hits[0].doc_id, "d1");
    assert_eq!(results.hits[1].doc_id, "d3");
    assert!(results.hits[0].score.is_none(), "canonical order carries no score");

    let count = index.count(&term("text_bare", "الله")).await.unwrap();
    assert_eq!(count, 2);

    let stats = index.stats().await.unwrap();
    assert_eq!(stats.doc_count, 3);
    assert_eq!(stats.generation, 7);

    let report = index.verify().await.unwrap();
    assert!(report.ok, "{:?}", report.findings);
    assert_eq!(report.doc_count, 3);
}

/// Index path AND query path normalize: diacriticized docs match bare
/// queries, and bare docs match diacriticized queries. Bypassing either
/// side breaks one direction. Wasla folds at L4, so the wasla pair runs on
/// `text_hamza` while the diacritic pair runs on `text_bare`.
#[tokio::test]
async fn normalization_on_both_paths() {
    let docs = vec![
        doc("diac", 1, 1, 1, "الرَّحْمَنِ"),
        doc("bare", 1, 2, 2, "الرحمن"),
        doc("wasla", 1, 3, 3, "ٱلرَّحْمَٰنِ"),
    ];
    let (_dir, index, _, _) = staged(docs).await;

    // Query normalized (diacritics in query still match).
    let found = index.search(&term("text_bare", "الرَّحْمَنِ"), &SearchOpts::default()).await.unwrap();
    assert_eq!(found.total_matches, 2);
    // Documents normalized (bare query matches diacriticized doc).
    let found = index.search(&term("text_bare", "الرحمن"), &SearchOpts::default()).await.unwrap();
    assert_eq!(found.total_matches, 2);
    // Exact field does NOT fold: the bare query misses the diacriticized doc.
    let found = index.search(&term("text_exact", "الرحمن"), &SearchOpts::default()).await.unwrap();
    assert_eq!(found.total_matches, 1);
    assert_eq!(found.hits[0].doc_id, "bare");
    // Wasla-insensitivity lives one rung up, on the hamza field.
    let found = index.search(&term("text_hamza", "الرحمن"), &SearchOpts::default()).await.unwrap();
    assert_eq!(found.total_matches, 3);
    // A term normalizing to empty matches nothing (never everything).
    let found = index.search(&term("text_bare", "ً"), &SearchOpts::default()).await.unwrap();
    assert_eq!(found.total_matches, 0);
}

#[tokio::test]
async fn phrase_boolean_all_and_filters() {
    let docs = vec![
        doc("d1", 1, 1, 1, "الحمد لله رب العالمين"),
        doc("d2", 1, 2, 2, "الرحمن الرحيم"),
        doc("d3", 2, 1, 10, "الحمد لله كثيرا"),
    ];
    let (_dir, index, _, _) = staged(docs).await;

    let phrase = FtsQuery::Phrase {
        field: "text_bare".to_string(),
        terms: vec!["الحمد".to_string(), "لله".to_string()],
        slop: 0,
        ordered: true,
    };
    let found = index.search(&phrase, &SearchOpts::default()).await.unwrap();
    assert_eq!(found.total_matches, 2);

    let boolean = FtsQuery::Boolean {
        must: vec![term("text_bare", "الحمد")],
        should: vec![],
        must_not: vec![term("text_bare", "كثيرا")],
    };
    let found = index.search(&boolean, &SearchOpts::default()).await.unwrap();
    assert_eq!(found.total_matches, 1);
    assert_eq!(found.hits[0].doc_id, "d1");

    let found = index.search(&FtsQuery::All, &SearchOpts::default()).await.unwrap();
    assert_eq!(found.total_matches, 3);

    // Metadata filters compose with the match.
    let opts = SearchOpts { filters: vec![Filter::Surah(vec![2])], ..SearchOpts::default() };
    let found = index.search(&FtsQuery::All, &opts).await.unwrap();
    assert_eq!(found.total_matches, 1);
    assert_eq!(found.hits[0].doc_id, "d3");

    // Limit truncates and reports it; relevance order carries scores.
    let opts = SearchOpts {
        limit: 1,
        order: ResultOrder::Relevance,
        ..SearchOpts::default()
    };
    let found = index.search(&FtsQuery::All, &opts).await.unwrap();
    assert_eq!(found.hits.len(), 1);
    assert!(found.truncated);
    assert!(found.hits[0].score.is_some());
}

#[tokio::test]
async fn regex_guards_and_expansion() {
    let docs = vec![doc("d1", 1, 1, 1, "الرحمن الرحيم"), doc("d2", 1, 2, 2, "الحمد لله")];
    let (_dir, index, _, _) = staged(docs).await;

    let anchored = FtsQuery::Regex { field: "text_bare".to_string(), pattern: "^ا?ل?رحم".to_string() };
    let found = index.search(&anchored, &SearchOpts::default()).await.unwrap();
    assert_eq!(found.total_matches, 1);
    assert_eq!(found.hits[0].doc_id, "d1");

    // Leading .* is rejected with an actionable error.
    let wild = FtsQuery::Regex { field: "text_bare".to_string(), pattern: ".*رحم".to_string() };
    let err = index.search(&wild, &SearchOpts::default()).await.unwrap_err();
    assert_eq!(err.code(), quran_search::codes::QUERY_REJECTED);

    // Over-long patterns are rejected before compiling.
    let long = FtsQuery::Regex { field: "text_bare".to_string(), pattern: "ا".repeat(600) };
    let err = index.search(&long, &SearchOpts::default()).await.unwrap_err();
    assert_eq!(err.code(), quran_search::codes::QUERY_REJECTED);

    // Non-text fields are rejected.
    let bad = FtsQuery::Regex { field: "surah".to_string(), pattern: "^1".to_string() };
    let err = index.search(&bad, &SearchOpts::default()).await.unwrap_err();
    assert_eq!(err.code(), quran_search::codes::QUERY_REJECTED);
}

#[tokio::test]
async fn tokenizer_mismatch_and_generation_lifecycle() {
    let dir = tempfile::tempdir().unwrap();
    // Unknown ladder versions fail at family construction.
    let err = TokenizerFamily::new(&ProfileRegistry::new(), SemVer::new(9, 9, 9)).unwrap_err();
    assert_eq!(err.code(), quran_search::codes::BUILD_FAILED);
    // Family version disagreeing with the manifest refuses to stage.
    let mut mismatched = manifest(0);
    mismatched.tokenizer_version = SemVer::new(9, 9, 9);
    let err = Fts5Index::stage(dir.path(), mismatched, family()).await.unwrap_err();
    assert_eq!(err.code(), quran_search::codes::MANIFEST_MISMATCH);

    // Deleting a missing generation is a no-op zero.
    let index = Fts5Index::stage(dir.path(), manifest(1), family()).await.unwrap();
    index.create(&schema()).await.unwrap();
    index.add_batch(vec![doc("d1", 1, 1, 1, "نص")]).await.unwrap();
    index.commit().await.unwrap();
    assert_eq!(index.delete_by_generation(6).await.unwrap(), 0);
    assert_eq!(index.delete_by_generation(7).await.unwrap(), 1);
    assert!(!dir.path().join("gen-7").exists());
    // Opening a deleted generation fails typed.
    let err = Fts5Index::open(dir.path(), manifest(1), family()).await.unwrap_err();
    assert_eq!(err.code(), quran_search::codes::BUILD_FAILED);
}
