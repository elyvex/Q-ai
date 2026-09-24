//! Phase 2 — query/index tokenizer parity (P2-T32).
//!
//! The query path and the index path share one [`TokenizerFamily`]: these
//! tests lock that wiring. The 5,000-substring loop proves the family is
//! total and stable over hostile Unicode; the end-to-end spot checks prove
//! both paths actually route through it (a bypass on either side fails one
//! direction: diacriticized docs must match bare queries AND bare docs must
//! match diacriticized queries).

use std::collections::BTreeMap;

use quran_normalization::{ProfileRegistry, RuleId};
use quran_search::{
    FieldId, Fts5Index, FtsDoc, FtsQuery, FtsSchema, FullTextIndex, IndexManifest, SearchOpts,
    SemVer, TokenizerFamily,
};

fn v1() -> SemVer {
    SemVer::new(1, 0, 0)
}

fn family() -> TokenizerFamily {
    TokenizerFamily::new(&ProfileRegistry::new(), v1()).unwrap()
}

fn manifest(doc_count: u64) -> IndexManifest {
    IndexManifest {
        index_id: "test.parity.v1".to_string(),
        schema_version: 1,
        corpus_generation: 3,
        edition_id: "ed-1".to_string(),
        edition_version: v1(),
        rule_set_versions: BTreeMap::new(),
        tokenizer_version: v1(),
        morphology_dataset_versions: BTreeMap::new(),
        built_at: "2026-09-15T00:00:00Z".to_string(),
        doc_count,
        trigram_postings: 0,
        content_hash: "sha256:test".to_string(),
    }
}

fn schema() -> FtsSchema {
    FtsSchema {
        index_id: "test.parity.v1".to_string(),
        schema_version: 1,
        fields: quran_search::tokenizer::INDEXED_FIELDS.iter().map(|s| (*s).to_string()).collect(),
    }
}

/// Hostile sample: diacritics, wasla, hamza carriers, tatweel, marks,
/// Persian code points, digits, punctuation, zero-width, presentation
/// forms, spaces, and plain ASCII.
const SAMPLE: &str = "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ، وَٱلْحَمْدُ ک weekend ۱۲۳ ﻻ\u{200B}ﬁﬂ  x  ﭐﻷَرْضِ ۖ test";

/// Deterministic char-boundary substrings of `SAMPLE` (5,000 cases).
fn substrings() -> Vec<String> {
    let chars: Vec<char> = SAMPLE.chars().collect();
    let len = chars.len();
    let mut out = Vec::with_capacity(5000);
    for i in 0..5000 {
        let start = (i * 37) % len;
        let width = (i * 13) % 24;
        let end = (start + width).min(len);
        out.push(chars[start..end].iter().collect());
    }
    out
}

/// The family is total and deterministic over 5,000 hostile substrings on
/// every indexed field, and agrees with the bare pipeline on the primary
/// field (the same instance both paths share).
#[test]
fn family_total_and_stable_over_5000_substrings() {
    let family = family();
    let registry = ProfileRegistry::new();
    let bare = quran_normalization::NormalizationPipeline::for_profile(
        &registry,
        quran_normalization::ProfileId::L3,
        v1(),
    )
    .unwrap();
    for substring in substrings() {
        for field in quran_search::tokenizer::INDEXED_FIELDS {
            let first = family.tokenize(field, &substring).unwrap();
            let second = family.tokenize(field, &substring).unwrap();
            assert_eq!(first, second, "unstable {field} on {substring:?}");
        }
        assert_eq!(
            family.tokenize("text_bare", &substring).unwrap(),
            bare.apply(&substring).0.text(),
            "family disagrees with the shared pipeline on {substring:?}"
        );
    }
}

/// Rule ids behind a field match the ladder (trace assembly upstream).
#[test]
fn field_rule_ids_match_ladder_order() {
    let family = family();
    let ids = family.rule_ids("text_bare");
    assert_eq!(
        ids,
        vec![
            RuleId::N01,
            RuleId::N11,
            RuleId::N16,
            RuleId::N04,
            RuleId::N14,
            RuleId::N03,
            RuleId::N05,
            RuleId::N02,
        ]
    );
    assert!(family.rule_ids("text_nope").is_empty());
}

/// End-to-end in both directions: diacriticized docs match bare queries
/// (index path normalizes) and bare docs match diacriticized queries
/// (query path normalizes). Bypassing either side breaks one direction.
#[tokio::test]
async fn end_to_end_parity_both_directions() {
    let dir = tempfile::tempdir().unwrap();
    // Wasla-free pair: the directions tested here are diacritic folding.
    // (Wasla folding lives at L4; the backend suite covers it per field.)
    let raw_diac = "الرَّحْمَنِ الرَّحِيمِ";
    let raw_bare = "الرحمن الرحيم";
    let mk = |id: &str, raw: &str| {
        let fields: BTreeMap<FieldId, String> = quran_search::tokenizer::INDEXED_FIELDS
            .iter()
            .map(|f| ((*f).to_string(), raw.to_string()))
            .collect();
        FtsDoc {
            id: id.to_string(),
            edition_id: "ed-1".to_string(),
            surah: 1,
            ayah: 1,
            global_index: 1,
            generation: 3,
            metadata: BTreeMap::new(),
            fields,
            lexicon_fields: BTreeMap::new(),
        }
    };
    let index = Fts5Index::stage(dir.path(), 3, manifest(2), family()).await.unwrap();
    index.create(&schema()).await.unwrap();
    index.add_batch(vec![mk("diac", raw_diac), mk("bare", raw_bare)]).await.unwrap();
    index.commit().await.unwrap();

    let bare_query = FtsQuery::Term { field: "text_bare".to_string(), term: raw_bare.to_string() };
    let found = index.search(&bare_query, &SearchOpts::default()).await.unwrap();
    assert_eq!(found.total_matches, 2, "index path must normalize");

    let diac_query = FtsQuery::Term { field: "text_bare".to_string(), term: raw_diac.to_string() };
    let found = index.search(&diac_query, &SearchOpts::default()).await.unwrap();
    assert_eq!(found.total_matches, 2, "query path must normalize");
}
