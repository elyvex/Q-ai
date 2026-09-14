//! Phase 1 — deterministic reader acceptance: AC-P1-05/06/19 + D1.6.
//!
//! Imports the synthetic edition under a fixed UUID run id (reader-mapped
//! domain types require UUID-parseable ids), activates it under a seeded
//! approval, then exercises lookup, context, divisions, resolve, caching, and
//! a performance smoke.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use application::quran_reader::{
    EditionFilter, QuranReader, QuranReaderService, ReaderError,
};
use domain::{PrincipalId, Timestamp};
use quran_core::{
    AyahOptions, ContextBoundary, ContextSpec, EditionSelector, QuranRef,
};
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use quran_corpus::sha256_hex;
use storage::Database as _;
use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

const BASE_MANIFEST: &str =
    include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const RUN_ID: &str = "11111111-2222-4333-8444-555555555555";
const PRINCIPAL: &str = "00000000-0000-0000-0000-000000000001";
const CREATED_AT: &str = "2026-09-14T00:00:00Z";
const SLUG: &str = "test-edition-min";
const VERSION: &str = "0.1.0";
const LICENSE_JSON: &str = "{\"status\":\"PublicDomain\",\"spdx_id\":null,\"name\":null,\
                              \"url\":null,\"attribution_required\":false,\
                              \"redistribution_allowed\":true,\"export_allowed\":true,\
                              \"notes\":null}";

fn principal() -> PrincipalId {
    PRINCIPAL.parse().unwrap()
}

fn timestamp() -> Timestamp {
    Timestamp::from_ymd_hms(2026, 9, 14, 0, 0, 0).unwrap()
}

async fn active_reader() -> (tempfile::TempDir, Arc<SqliteDatabase>, QuranReaderService, String) {
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
             VALUES ('appr-1', 'quran-edition:{SLUG}@{VERSION}', 'CanonicalChange',
                     '{PRINCIPAL}', '{PRINCIPAL}', 'approved', '{{}}', '{CREATED_AT}', '{CREATED_AT}')"
        ),
    ] {
        sqlx::query(&sql).execute(&seed).await.unwrap();
    }
    seed.close().await;

    let manifest_hash = sha256_hex(BASE_MANIFEST.as_bytes());
    let outcome = run_import(
        &*db,
        &ImportInput {
            run_id: RUN_ID.into(),
            job_id: None,
            source_version_id: "sv-1".into(),
            adapter: "json".into(),
            manifest_text: BASE_MANIFEST.into(),
            declared_manifest_hash: Some(manifest_hash),
            invoked_by: PRINCIPAL.into(),
            license_status: "PublicDomain".into(),
            license_json: LICENSE_JSON.into(),
            created_at: CREATED_AT.into(),
        },
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .expect("import completes");
    assert!(matches!(outcome, ImportOutcome::Completed(_)));
    let generation = application::quran::activate_edition(
        &*db,
        SLUG,
        VERSION,
        &principal(),
        "appr-1",
        &timestamp(),
    )
    .await
    .expect("activation completes");
    assert_eq!(generation, 1);
    let reader = QuranReaderService::new(db.clone());
    (dir, db, reader, path_str)
}

fn plain() -> AyahOptions {
    AyahOptions { translations: Vec::new(), glosses: false, tokens: false }
}

#[tokio::test]
async fn get_ayah_returns_identified_canonical_text() {
    let (_dir, _db, reader, _path) = active_reader().await;
    let reference = quran_core::parse("2:1").unwrap();
    let view = reader.get_ayah(&reference, &plain()).await.unwrap();
    assert_eq!(view.canonical.reference(), "quran:test-edition-min@0.1.0:2:1");
    assert_eq!(view.canonical.edition().slug, SLUG);
    assert_eq!(view.canonical.edition().version.to_string(), VERSION);
    assert_eq!(view.canonical.text_hash().hex.len(), 64);
    assert!(view.canonical.deep_link().contains("test-edition-min@0.1.0/2:1"));
    assert!(view.translations.is_empty());
    assert!(view.tokens.is_none());
}

#[tokio::test]
async fn get_ayah_with_tokens_attaches_surfaces() {
    let (_dir, _db, reader, _path) = active_reader().await;
    let reference = quran_core::parse("1:1").unwrap();
    let options = AyahOptions { translations: Vec::new(), glosses: false, tokens: true };
    let view = reader.get_ayah(&reference, &options).await.unwrap();
    let tokens = view.tokens.expect("tokens requested");
    assert!(!tokens.is_empty());
    let surfaces: Vec<_> = tokens.iter().map(|token| token.surface.as_str()).collect();
    assert_eq!(surfaces.join(" "), view.canonical.arabic_text());
    for (index, token) in tokens.iter().enumerate() {
        assert_eq!(token.position.get(), index as u16 + 1);
    }
}

#[tokio::test]
async fn missing_references_are_typed_errors() {
    let (_dir, _db, reader, _path) = active_reader().await;
    let err = reader.get_ayah(&quran_core::parse("99:1").unwrap(), &plain()).await.unwrap_err();
    assert!(matches!(err, ReaderError::AyahNotFound(_)));
    let err =
        reader.get_ayah(&quran_core::parse("quran:juz:99").unwrap(), &plain()).await.unwrap_err();
    assert!(matches!(err, ReaderError::DivisionNotFound(_)));
    let options = AyahOptions {
        translations: vec!["en-missing".to_string()],
        glosses: false,
        tokens: false,
    };
    let err =
        reader.get_ayah(&quran_core::parse("1:1").unwrap(), &options).await.unwrap_err();
    assert!(matches!(err, ReaderError::TranslationNotFound(_)));
}

#[tokio::test]
async fn ranges_surahs_and_divisions_expand() {
    let (_dir, _db, reader, _path) = active_reader().await;
    let range = reader.get_ayahs(&quran_core::parse("1:1-2:1").unwrap(), &plain()).await.unwrap();
    assert_eq!(range.len(), 4);
    assert_eq!(range[0].canonical.reference(), "quran:test-edition-min@0.1.0:1:1");
    assert_eq!(range[3].canonical.reference(), "quran:test-edition-min@0.1.0:2:1");

    let surah = reader
        .get_ayahs(
            &QuranRef::Surah {
                edition: EditionSelector::Active,
                surah: "3".parse().unwrap(),
            },
            &plain(),
        )
        .await
        .unwrap();
    assert_eq!(surah.len(), 4);

    let juz = reader.get_ayahs(&quran_core::parse("quran:juz:1").unwrap(), &plain()).await.unwrap();
    assert_eq!(juz.len(), 8);
    let juz2 =
        reader.get_ayahs(&quran_core::parse("quran:juz:2").unwrap(), &plain()).await.unwrap();
    assert_eq!(juz2.len(), 6);
}

#[tokio::test]
async fn context_respects_boundaries_and_caps() {
    let (_dir, _db, reader, _path) = active_reader().await;
    let spec = ContextSpec {
        before: 3,
        after: 3,
        boundary: ContextBoundary::Surah,
        include_surah_header: true,
        max_ayahs: 10,
    };
    // Ayah (2,2) is the last of surah 2: one ayah before it, none after (clipped).
    let view =
        reader.get_context(&quran_core::parse("2:2").unwrap(), &spec).await.unwrap();
    assert_eq!(view.focal.canonical.reference(), "quran:test-edition-min@0.1.0:2:2");
    assert!(view.surah.is_some());
    assert_eq!(view.before.len(), 1);
    assert!(view.after.is_empty(), "surah boundary clips the tail");
    assert_eq!(view.global_range, (4, 5));

    let capped = ContextSpec { max_ayahs: 2, ..spec };
    let view = reader.get_context(&quran_core::parse("2:2").unwrap(), &capped).await.unwrap();
    assert_eq!(1 + view.before.len() + view.after.len(), 2);

    // Juz boundary: (3,1) is the first ayah of juz 2 in the fixture.
    let juz_spec = ContextSpec {
        before: 5,
        after: 0,
        boundary: ContextBoundary::Juz,
        include_surah_header: false,
        max_ayahs: 20,
    };
    let view = reader.get_context(&quran_core::parse("3:2").unwrap(), &juz_spec).await.unwrap();
    assert_eq!(view.before.len(), 1, "clipped at the juz-2 start");
}

#[tokio::test]
async fn resolve_parses_and_bounds_checks() {
    let (_dir, _db, reader, _path) = active_reader().await;
    let resolved = reader.resolve("2:255").await.unwrap_err();
    assert!(matches!(resolved, ReaderError::AyahNotFound(_)));
    let resolved = reader.resolve("quran:juz:1").await.unwrap();
    assert_eq!(resolved.canonical, "juz:1");
}

#[tokio::test]
async fn editions_and_surahs_list() {
    let (_dir, _db, reader, _path) = active_reader().await;
    let editions = reader.list_editions(EditionFilter::default()).await.unwrap();
    assert_eq!(editions.len(), 1);
    assert_eq!(editions[0].slug, SLUG);
    let active = reader.get_edition(&EditionSelector::Active).await.unwrap();
    assert_eq!(active.status, quran_core::EditionStatus::Active);
    let surahs = reader.list_surahs(&EditionSelector::Active).await.unwrap();
    assert_eq!(surahs.len(), 5);
    assert_eq!(surahs[0].number.get(), 1);
}

#[tokio::test]
async fn cache_serves_no_stale_text_after_activation() {
    let (_dir, db, reader, path) = active_reader().await;
    let reference = quran_core::parse("1:1").unwrap();
    let before = reader.get_ayah(&reference, &plain()).await.unwrap();
    // Import a corrected version (first ayah extended by one token) and activate it.
    let mut v2_doc: serde_json::Value = serde_json::from_str(BASE_MANIFEST).unwrap();
    v2_doc["edition"]["version"] = serde_json::json!("0.2.0");
    let first = v2_doc["ayahs"][0]["text"].as_str().unwrap().to_string();
    v2_doc["ayahs"][0]["text"] = serde_json::json!(format!("{first} ب"));
    let v2_manifest = serde_json::to_string(&v2_doc).unwrap();
    let run_id = "22222222-3333-4444-8555-666666666666";
    run_import(
        &*db,
        &ImportInput {
            run_id: run_id.into(),
            job_id: None,
            source_version_id: "sv-1".into(),
            adapter: "json".into(),
            manifest_text: v2_manifest,
            declared_manifest_hash: None,
            invoked_by: PRINCIPAL.into(),
            license_status: "PublicDomain".into(),
            license_json: LICENSE_JSON.into(),
            created_at: CREATED_AT.into(),
        },
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .expect("v2 imports");
    // Approval for the corrected version.
    {
        let seed = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(&path)
                    .foreign_keys(true),
            )
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO approvals
                (id, subject_urn, kind, requested_by, decided_by, decision,
                 request_payload, requested_at, decided_at)
             VALUES ('appr-2', 'quran-edition:test-edition-min@0.2.0', 'CanonicalChange',
                     '00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001',
                     'approved', '{}', '2026-09-14T00:00:00Z', '2026-09-14T00:00:00Z')",
        )
        .execute(&seed)
        .await
        .unwrap();
        seed.close().await;
    }
    let generation = application::quran::activate_edition(
        &*db,
        SLUG,
        "0.2.0",
        &principal(),
        "appr-2",
        &timestamp(),
    )
    .await
    .expect("v2 activates");
    assert_eq!(generation, 2);
    let after = reader.get_ayah(&reference, &plain()).await.unwrap();
    assert_ne!(before.canonical.arabic_text(), after.canonical.arabic_text());
    assert!(after.canonical.reference().contains("@0.2.0"));
}

#[tokio::test]
async fn lookup_performance_smoke() {
    let (_dir, _db, reader, _path) = active_reader().await;
    let reference = quran_core::parse("2:1").unwrap();
    let start = Instant::now();
    for _ in 0..2000 {
        reader.get_ayah(&reference, &plain()).await.unwrap();
    }
    let average_ms = start.elapsed().as_secs_f64() * 1000.0 / 2000.0;
    assert!(average_ms < 5.0, "warm average {average_ms:.3} ms over budget");
}
