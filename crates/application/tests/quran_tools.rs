//! Phase 1 — tool contract + citation acceptance: AC-P1-15/21 (P1-T43–T47).
//!
//! `quran.get_ayah` / `quran.get_context` conformance against the real reader,
//! the no-fabrication guarantee, deterministic checksums, and citation
//! resolution + persistence round-trips.

#[path = "common/mod.rs"]
mod common;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use application::quran_forms::{RebuildParams, rebuild_forms};
use application::quran_index::{IndexBuildParams, QURAN_AYAH_INDEX_ID, rebuild_index};
use application::quran_morphology::{
    MorphologyActivateParams, MorphologyImportParams, activate_morphology, dataset_urn,
    run_morphology_import,
};
use application::quran_reader::{QuranReader, QuranReaderService};
use application::quran_tools::{ReaderToolBackend, tool_get_ayah, tool_get_context};
use common::{BASE_MANIFEST, active_reader};
use quran_core::AyahOptions;
use quran_corpus::import::{ImportInput, ImportOptions, ImportProgress, run_import};
use storage::Database as _;
use storage_sqlite::SqliteDatabase;
use tool_registry::{
    FamilyToolParams, GetAyahParams, GetContextParams, LemmaToolParams, MorphologyToolParams,
    RootToolParams, SearchToolParams,
};

fn plain() -> AyahOptions {
    AyahOptions { translations: Vec::new(), glosses: false, tokens: false }
}

#[tokio::test]
async fn get_ayah_tool_conforms_and_cites() {
    let (_dir, _db, reader, _path) = active_reader().await;
    let reader = Arc::new(reader);
    let (result, meta) = tool_get_ayah(
        &reader,
        GetAyahParams {
            reference: "2:1".into(),
            translations: Vec::new(),
            glosses: false,
            tokens: true,
        },
    )
    .await
    .unwrap();
    assert_eq!(meta.edition_slug, "test-edition-min");
    assert_eq!(result.tool_name, "quran.get_ayah");
    assert_eq!(result.results.len(), 1);
    assert_eq!(result.edition_version.as_deref(), Some("0.1.0"));
    assert_eq!(result.canonical_references.len(), 1);
    assert!(result.canonical_references[0].contains("test-edition-min@0.1.0:2:1"));
    assert!(!result.results[0].tokens.as_ref().unwrap().is_empty());
    assert!(result.reproducibility.deterministic);
    assert_eq!(result.reproducibility.corpus_generation, 1);
    // Deterministic: same call, same checksum.
    let (again, _) = tool_get_ayah(
        &reader,
        GetAyahParams {
            reference: "2:1".into(),
            translations: Vec::new(),
            glosses: false,
            tokens: true,
        },
    )
    .await
    .unwrap();
    assert_eq!(result.reproducibility.checksum, again.reproducibility.checksum);
}

#[tokio::test]
async fn get_context_tool_conforms() {
    let (_dir, _db, reader, _path) = active_reader().await;
    let (result, _) = tool_get_context(
        &Arc::new(reader),
        GetContextParams {
            reference: "1:2".into(),
            before: 1,
            after: 1,
            boundary: tool_registry::ContextBoundaryArg::Surah,
            max_ayahs: 5,
        },
    )
    .await
    .unwrap();
    assert_eq!(result.tool_name, "quran.get_context");
    assert_eq!(result.results.before.len(), 1);
    assert_eq!(result.results.after.len(), 1);
    assert_eq!(
        result.canonical_references,
        vec![
            "quran:test-edition-min@0.1.0:1:2".to_string(),
            "quran:test-edition-min@0.1.0:1:1".to_string(),
            "quran:test-edition-min@0.1.0:1:3".to_string(),
        ]
    );
}

#[tokio::test]
async fn no_fabrication_on_missing_references() {
    let (_dir, _db, reader, _path) = active_reader().await;
    let reader = Arc::new(reader);
    // A well-formed but absent reference is a typed backend error, never text.
    let err = tool_get_ayah(
        &reader,
        GetAyahParams {
            reference: "99:1".into(),
            translations: Vec::new(),
            glosses: false,
            tokens: false,
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, tools::ToolError::Backend { .. }));
    // A malformed reference is a typed input error.
    let err = tool_get_ayah(
        &reader,
        GetAyahParams {
            reference: ":::".into(),
            translations: Vec::new(),
            glosses: false,
            tokens: false,
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, tools::ToolError::InvalidInput { .. }));
}

#[tokio::test]
async fn citations_resolve_verify_and_persist() {
    let (_dir, db, reader, _path) = active_reader().await;
    let reader = Arc::new(reader);
    let view = reader.get_ayah(&quran_core::parse("1:2").unwrap(), &plain()).await.unwrap();
    let text = view.canonical.arabic_text().to_string();
    let citation = citations::Citation {
        id: "cit-1".into(),
        kind: citations::CitationKind::Quran,
        canonical_reference: "quran:test-edition-min@0.1.0:1:2".into(),
        quoted_text: text.clone(),
        edition_slug: "test-edition-min".into(),
        edition_version: "0.1.0".into(),
        surah: "1".parse().unwrap(),
        ayah: "2".parse().unwrap(),
    };
    let resolver = application::quran_tools::ReaderCitationSource::resolver(reader.clone());
    let resolved = resolver.resolve(&citation).await.unwrap();
    assert_eq!(resolved.verdict, citations::QuotationVerdict::ExactMatch);
    assert!(resolved.text_hash.as_deref().unwrap_or("").starts_with("sha256:"));
    assert_eq!(resolved.deep_link.as_deref(), Some("/read/test-edition-min@0.1.0/1:2"));

    // Tampered quotation → hard Mismatch with location info.
    let verdict = resolver.verify_quotation(&citation, "tampered text").await.unwrap();
    assert!(matches!(verdict, citations::QuotationVerdict::Mismatch { .. }));

    // Persist + re-read + re-verify (AC-P1-21's later-reverification).
    application::quran_tools::persist_citation(&*db, &resolved, &citation, "1", common::CREATED_AT)
        .await
        .unwrap();
    let mut uow = db.write().await.unwrap();
    let stored = uow.quran().get_citation("cit-1").await.unwrap().unwrap();
    assert_eq!(stored.verdict, "ExactMatch");
    uow.rollback().await.unwrap();
    let reverified = resolver.verify_quotation(&citation, &text).await.unwrap();
    assert_eq!(reverified, citations::QuotationVerdict::ExactMatch);
}

#[tokio::test]
async fn tools_read_the_active_edition_after_activation() {
    // A second import + activation moves what the tools serve (generation 2).
    let (_dir, db, reader, _path) = active_reader().await;
    let reader = Arc::new(reader);
    let mut v2_doc: serde_json::Value = serde_json::from_str(BASE_MANIFEST).unwrap();
    v2_doc["edition"]["version"] = serde_json::json!("0.2.0");
    let first = v2_doc["ayahs"][0]["text"].as_str().unwrap().to_string();
    v2_doc["ayahs"][0]["text"] = serde_json::json!(format!("{first} ب"));
    run_import(
        &*db,
        &ImportInput {
            run_id: "22222222-3333-4444-8555-666666666666".into(),
            job_id: None,
            source_version_id: common::SOURCE_VERSION_ID.into(),
            adapter: "json".into(),
            manifest_text: serde_json::to_string(&v2_doc).unwrap(),
            declared_manifest_hash: None,
            invoked_by: common::PRINCIPAL.into(),
            license_status: "PublicDomain".into(),
            license_json: common::LICENSE_JSON.into(),
            created_at: common::CREATED_AT.into(),
            reference_manifest_text: None,
        },
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .expect("v2 imports");
    let (before, _) = tool_get_ayah(
        &reader,
        GetAyahParams {
            reference: "1:1".into(),
            translations: Vec::new(),
            glosses: false,
            tokens: false,
        },
    )
    .await
    .unwrap();
    assert_eq!(before.reproducibility.corpus_generation, 1);
    // No approval seeded for v2 here: tools keep serving v1 until activation.
    assert!(before.results[0].canonical.reference().contains("@0.1.0"));
}

#[tokio::test]
async fn verify_canonical_quotation_succeeds_on_exact_and_hard_fails_on_mismatch() {
    let (_dir, _db, reader, _path) = active_reader().await;
    let reader = Arc::new(reader);
    let view = reader.get_ayah(&quran_core::parse("1:2").unwrap(), &plain()).await.unwrap();
    let text = view.canonical.arabic_text().to_string();

    // The real fixture text verifies exactly and returns the resolved canonical
    // hash read from the row (never computed from the supplied text).
    let (verdict, hash) = application::quran_tools::verify_canonical_quotation(
        &reader,
        "test-edition-min",
        "0.1.0",
        1,
        2,
        &text,
    )
    .await
    .unwrap();
    assert_eq!(verdict, citations::QuotationVerdict::ExactMatch);
    assert_eq!(hash, format!("sha256:{}", view.canonical.text_hash().hex));
    assert!(verdict.label() == "ExactMatch");

    // A tampered string is a hard failure with a typed, stable code — never a
    // success and never a synthesized hash.
    let err = application::quran_tools::verify_canonical_quotation(
        &reader,
        "test-edition-min",
        "0.1.0",
        1,
        2,
        "tampered text",
    )
    .await
    .unwrap_err();
    assert_eq!(err.code(), "QAI-QUR-0323");

    // A missing location and a missing edition are hard not-found failures.
    let err = application::quran_tools::verify_canonical_quotation(
        &reader,
        "test-edition-min",
        "0.1.0",
        99,
        9,
        &text,
    )
    .await
    .unwrap_err();
    assert_eq!(err.code(), "QAI-QUR-0324");
    let err = application::quran_tools::verify_canonical_quotation(
        &reader,
        "no-such-edition",
        "0.1.0",
        1,
        2,
        &text,
    )
    .await
    .unwrap_err();
    assert_eq!(err.code(), "QAI-QUR-0325");
}

// --- Phase 3 (03-07): the D-13 attributed search/lexicon tool surface -------

const MORPH_SLUG: &str = "test-morph";
const MORPH_VERSION: &str = "0.1.0";
const MORPH_BATCH: &str = "tools-conformance-batch";

/// The fixture's first token surface (a deterministic whole-token query).
async fn first_token_surface(db: &SqliteDatabase) -> String {
    let mut uow = db.write().await.unwrap();
    let active = uow.quran().get_active().await.unwrap().unwrap();
    let ayahs = uow.quran().list_ayahs_range(&active.edition_id, 1, i64::MAX).await.unwrap();
    let mut surface = String::new();
    for ayah in &ayahs {
        let tokens =
            uow.quran().get_tokens(&active.edition_id, ayah.surah, ayah.ayah).await.unwrap();
        if let Some(token) = tokens.first() {
            surface = token.surface.clone();
            break;
        }
    }
    uow.rollback().await.unwrap();
    surface
}

/// Import + activate edition (`active_reader`) + derived forms + a built
/// serving index: the search tool's read context.
async fn searchable_reader()
-> (tempfile::TempDir, Arc<SqliteDatabase>, Arc<QuranReaderService>, std::path::PathBuf) {
    let (dir, db, reader, _path) = active_reader().await;
    rebuild_forms(
        &db,
        &RebuildParams {
            edition_slug: common::SLUG.to_string(),
            edition_version: common::VERSION.to_string(),
            invoked_by: common::PRINCIPAL.to_string(),
            run_tag: "tools-test".to_string(),
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
            edition_slug: common::SLUG.to_string(),
            edition_version: common::VERSION.to_string(),
            invoked_by: common::PRINCIPAL.to_string(),
            run_tag: "tools-index".to_string(),
            data_dir: data_dir.clone(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("index build completes");
    (dir, db, Arc::new(reader), data_dir)
}

/// Seed the approval activation is gated on (the dataset URN subject).
async fn seed_dataset_approval(dir: &tempfile::TempDir) {
    let path = dir.path().join("qai.db");
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(path).foreign_keys(true))
        .await
        .unwrap();
    sqlx::query(&format!(
        "INSERT INTO approvals
            (id, subject_urn, kind, requested_by, decided_by, decision,
             request_payload, requested_at, decided_at)
         VALUES ('appr-morph', '{}', 'CanonicalChange', '{}', '{}', 'approved', '{{}}', '{}', '{}')",
        dataset_urn(MORPH_SLUG, MORPH_VERSION),
        common::PRINCIPAL,
        common::PRINCIPAL,
        common::CREATED_AT,
        common::CREATED_AT,
    ))
    .execute(&pool)
    .await
    .unwrap();
    pool.close().await;
}

/// A full aligned synthetic morphology document (two analyses per token).
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
            for analysis_no in [0u32, 1u32] {
                rows.push(serde_json::json!({
                    "sura_no": ayah.surah,
                    "aya_no": ayah.ayah,
                    "tok_idx": token.position,
                    "analysis_no": analysis_no,
                    "surface_form": token.surface,
                    "lemma_str": format!("lem-{}", index % 9),
                    "root_str": format!("root-{}", index % 3),
                    "stem_str": format!("stem-{}", index % 5),
                    "tag_native": if analysis_no == 0 { "N" } else { "V" },
                    "tag_unified": if analysis_no == 0 { "noun" } else { "verb" },
                    "layer": "B",
                    "state": "imported",
                    "synthetic_test_only": true,
                }));
            }
            index += 1;
        }
    }
    uow.rollback().await.unwrap();
    serde_json::to_string(&rows).unwrap()
}

fn morph_params(document_text: String) -> MorphologyImportParams {
    MorphologyImportParams {
        dataset_slug: MORPH_SLUG.to_string(),
        dataset_version: MORPH_VERSION.to_string(),
        adapter: "json".to_string(),
        document_text,
        edition_slug: common::SLUG.to_string(),
        edition_version: common::VERSION.to_string(),
        invoked_by: common::PRINCIPAL.to_string(),
        batch_id: Some(MORPH_BATCH.to_string()),
        attribution: "synthetic test import (not scholarly data)".to_string(),
        license_status: "PublicDomain".to_string(),
        license_json: r#"{"source_url":"https://example.invalid/qai-synthetic-test-lexicon","capture_date":"2026-09-28","capturer":"qai-test-fixtures","spdx_id":"CC0-1.0","redistribution_allowed":true,"modification_allowed":true,"attribution_required":false}"#.to_string(),
    }
}

/// G-11 / D-13: `quran.search` returns an attributed envelope whose rule trace
/// comes from the served hit (I9), and an empty query is a typed invalid-input
/// error — never an empty result.
#[tokio::test]
async fn search_tool_returns_attributed_envelope() {
    let (_dir, db, reader, data_dir) = searchable_reader().await;
    let registry = ReaderToolBackend::registry_with_index_root(reader, data_dir);
    // The registry ships all twelve `quran.*` tools (two reads + five D-13
    // lexicon/search + five graph); the backend only serves the ones it wires.
    assert_eq!(registry.tool_names().len(), 12);
    assert!(registry.tool_names().contains(&"quran.search"));

    let surface = first_token_surface(&db).await;
    assert!(!surface.is_empty(), "fixture has a token surface");

    // Empty/whitespace query: typed invalid input (the flagged ASSUMPTION edge).
    let err = registry
        .search(SearchToolParams { text: "   ".into(), edition: None, limit: None })
        .await
        .unwrap_err();
    assert!(matches!(err, tools::ToolError::InvalidInput { tool: "quran.search", .. }));

    let result = registry
        .search(SearchToolParams { text: surface, edition: None, limit: None })
        .await
        .unwrap();
    assert_eq!(result.tool_name, "quran.search");
    assert_eq!(result.edition_version.as_deref(), Some("0.1.0"));
    assert!(!result.canonical_references.is_empty(), "hits carry canonical references");
    assert!(!result.analysis_sources.is_empty(), "search attributes its canonical sources");
    assert!(!result.normalization_rules.is_empty(), "L3 search always names the rules it applied");
    assert!(result.reproducibility.deterministic);
}

/// G-11 / SC3: with no active dataset every lexicon tool returns the typed
/// backend error (QAI-MORPH-0004) — never an empty result.
#[tokio::test]
async fn lexicon_tools_fail_closed_without_a_dataset() {
    let (_dir, _db, reader, data_dir) = searchable_reader().await;
    let registry = ReaderToolBackend::registry_with_index_root(reader, data_dir);
    for err in [
        registry.root(RootToolParams { root: "root-1".into() }).await.unwrap_err(),
        registry.lemma(LemmaToolParams { lemma: "lem-1".into() }).await.unwrap_err(),
        registry
            .morphology(MorphologyToolParams { surah: 1, ayah: 1, position: 1 })
            .await
            .unwrap_err(),
        registry
            .family(FamilyToolParams { kind: "token".into(), id: "token:1:1:1".into() })
            .await
            .unwrap_err(),
    ] {
        match err {
            tools::ToolError::Backend { code, .. } => {
                assert_eq!(
                    code, "QAI-MORPH-0004",
                    "unavailable dataset is a typed capability error"
                )
            }
            other => panic!("expected a typed backend error, got {other:?}"),
        }
    }
}

/// G-11 / T-03-29: with an active dataset every lexicon tool returns an
/// attributed envelope (`analysis_sources` names the dataset).
#[tokio::test]
async fn lexicon_tools_are_attributed_with_an_active_dataset() {
    let (dir, db, reader, data_dir) = searchable_reader().await;
    seed_dataset_approval(&dir).await;
    let document = aligned_morph_document(&db).await;
    run_morphology_import(&db, &morph_params(document), &AtomicBool::new(false), |_| {})
        .await
        .expect("morphology import completes");
    activate_morphology(
        &db,
        &MorphologyActivateParams {
            batch_id: MORPH_BATCH.to_string(),
            approval_id: "appr-morph".to_string(),
            invoked_by: common::PRINCIPAL.to_string(),
        },
        &common::principal(),
    )
    .await
    .expect("morphology activation completes");

    let registry = ReaderToolBackend::registry_with_index_root(reader, data_dir);

    let morphology =
        registry.morphology(MorphologyToolParams { surah: 1, ayah: 1, position: 1 }).await.unwrap();
    assert_eq!(morphology.tool_name, "quran.morphology");
    assert_eq!(morphology.analysis_sources[0].kind, "dataset");
    assert!(
        morphology.results.as_array().is_some_and(|rows| !rows.is_empty()),
        "the active dataset serves analyses"
    );
    assert!(!morphology.canonical_references.is_empty());

    let family = registry
        .family(FamilyToolParams { kind: "token".into(), id: "token:1:1:1".into() })
        .await
        .unwrap();
    assert_eq!(family.tool_name, "quran.family");
    assert_eq!(family.analysis_sources[0].kind, "dataset");

    let root = registry.root(RootToolParams { root: "root-0".into() }).await.unwrap();
    assert_eq!(root.tool_name, "quran.root");
    assert!(!root.analysis_sources.is_empty(), "root results attribute their dataset");

    let lemma = registry.lemma(LemmaToolParams { lemma: "lem-0".into() }).await.unwrap();
    assert_eq!(lemma.tool_name, "quran.lemma");
    assert!(!lemma.analysis_sources.is_empty(), "lemma results attribute their dataset");
}

/// D-14/D-15: every `quran.get_ayah` result carries a non-empty,
/// payload-sensitive `research_checksum` — identical calls reproduce it,
/// a different payload moves it.
#[tokio::test]
async fn get_ayah_research_checksum_is_nonempty_and_payload_sensitive() {
    let (_dir, _db, reader, _path) = active_reader().await;
    let reader = Arc::new(reader);
    let plain = GetAyahParams {
        reference: "2:1".into(),
        translations: Vec::new(),
        glosses: false,
        tokens: false,
    };
    let (first, _) = tool_get_ayah(&reader, plain).await.unwrap();
    assert!(!first.research_checksum.hex.is_empty(), "checksum must be non-empty");
    let (again, _) = tool_get_ayah(
        &reader,
        GetAyahParams {
            reference: "2:1".into(),
            translations: Vec::new(),
            glosses: false,
            tokens: false,
        },
    )
    .await
    .unwrap();
    assert_eq!(
        first.research_checksum, again.research_checksum,
        "identical tool + inputs + payload reproduces the checksum"
    );
    // A richer payload (tokens attached) moves the digest.
    let (with_tokens, _) = tool_get_ayah(
        &reader,
        GetAyahParams {
            reference: "2:1".into(),
            translations: Vec::new(),
            glosses: false,
            tokens: true,
        },
    )
    .await
    .unwrap();
    assert!(!with_tokens.research_checksum.hex.is_empty());
    assert_ne!(
        first.research_checksum, with_tokens.research_checksum,
        "a different result payload must change the checksum"
    );
}
