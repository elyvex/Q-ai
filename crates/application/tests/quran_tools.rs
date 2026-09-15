//! Phase 1 — tool contract + citation acceptance: AC-P1-15/21 (P1-T43–T47).
//!
//! `quran.get_ayah` / `quran.get_context` conformance against the real reader,
//! the no-fabrication guarantee, deterministic checksums, and citation
//! resolution + persistence round-trips.

#[path = "common/mod.rs"]
mod common;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use application::quran_reader::QuranReader;
use application::quran_tools::{tool_get_ayah, tool_get_context};
use common::{BASE_MANIFEST, active_reader};
use quran_core::AyahOptions;
use quran_corpus::import::{ImportInput, ImportOptions, ImportProgress, run_import};
use storage::Database as _;
use tool_registry::{GetAyahParams, GetContextParams};

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
