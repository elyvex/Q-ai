//! Phase 1 — persistence acceptance: AC-P1-08/09 (P1-T12–T14, T24).
//!
//! Against real SQLite in a tempdir, never a mock:
//! - insert-only triggers abort raw `UPDATE`/`DELETE` with `QAI-QUR-0001…0005`;
//! - staging tables stay writable and cascade on run deletion;
//! - activation moves staging → canonical, flips the pointer, and bumps
//!   `corpus_generation` atomically; rollback restores a prior version.
//!
//! AC-P1-09's API-surface half (no public canonical-write path except
//! activation-with-approval) holds structurally: `QuranRepository` exposes no
//! row-level canonical insert, and is reviewed as such; the trigger half is
//! automated here.

use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use storage::Database;
use storage::quran::{
    AyahRow, CitationRow, DifferenceReportRow, ImportRunRow, QuranEditionRow, SeparatorRow,
    SurahRow, TokenRow, TranslationEditionRow, TranslationPassageRow, ValidationReportRow,
    WordGlossRow,
};
use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

fn now() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

async fn migrated_db() -> (tempfile::TempDir, SqliteDatabase, String) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("qai.db");
    let path = db_path.to_str().unwrap().to_string();
    let repo_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    storage_sqlite::migrate::apply_migrations(&path, &repo_root).await.unwrap();
    let seed = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(&path).foreign_keys(true))
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO principals (id, kind, display_name, created_at)
         VALUES ('principal', 'local_user', 'Test', ?)",
    )
    .bind(now())
    .execute(&seed)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO sources (id, title, content_type, created_at, updated_at)
         VALUES ('src-1', 'Test source', 'quran_edition', ?, ?)",
    )
    .bind(now())
    .bind(now())
    .execute(&seed)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO source_versions
            (id, source_id, version, schema_version, state, trust_level,
             license_status, license_json, created_at)
         VALUES ('sv-1', 'src-1', '0.1.0', 1, 'Staged', 'ImportedUnverified',
                 'PublicDomain', '{}', ?)",
    )
    .bind(now())
    .execute(&seed)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO provenance_records
            (id, layer, subject_urn, attribution_kind, attribution_json,
             source_version_id, trust_level, verification_status, versions_json,
             created_at, created_by)
         VALUES ('prov-1', 'canonical_source', 'test', 'dataset', '{}', 'sv-1',
                 'CanonicalVerified', 'unverified', '{}', ?, 'principal')",
    )
    .bind(now())
    .execute(&seed)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO approvals
            (id, subject_urn, kind, requested_by, decision, request_payload, requested_at)
         VALUES ('appr-1', 'test', 'canonical_change', 'principal', 'approved', '{}', ?)",
    )
    .bind(now())
    .execute(&seed)
    .await
    .unwrap();
    seed.close().await;
    let db = SqliteDatabase::new(&path, 4, true).await.unwrap();
    (dir, db, path)
}

fn edition_row(id: &str, slug: &str, version: &str, status: &str) -> QuranEditionRow {
    QuranEditionRow {
        id: id.to_string(),
        slug: slug.to_string(),
        version: version.to_string(),
        name: "Test".to_string(),
        script: "uthmani".to_string(),
        riwayah: None,
        qiraah: None,
        publisher: None,
        source_url: None,
        language: "ar".to_string(),
        verse_numbering_scheme: "hafs".to_string(),
        basmala_policy: "per_surah".to_string(),
        unicode_normalization: "nfc".to_string(),
        license_json: "{}".to_string(),
        text_hash: "sha256:aa".to_string(),
        structure_hash: "sha256:bb".to_string(),
        token_order_hash: "sha256:cc".to_string(),
        manifest_hash: "sha256:dd".to_string(),
        source_version_id: "sv-1".to_string(),
        statistics_json: "{}".to_string(),
        status: status.to_string(),
        imported_at: now(),
        verified_at: None,
        verified_by: None,
        verification_method: None,
        activated_at: None,
        deprecated_at: None,
    }
}

fn surah_row(edition: &str, number: i64, ayah_count: i64) -> SurahRow {
    SurahRow {
        edition_id: edition.to_string(),
        number,
        name_arabic: "تجريبي".to_string(),
        name_transliteration: None,
        name_translations_json: "{}".to_string(),
        ayah_count,
        revelation_place: Some("makki".to_string()),
        revelation_order: Some(number),
        basmala: "absent".to_string(),
        ruku_count: None,
        metadata_provenance_id: Some("prov-1".to_string()),
    }
}

fn ayah_row(edition: &str, surah: i64, ayah: i64, global: i64) -> AyahRow {
    AyahRow {
        edition_id: edition.to_string(),
        surah,
        ayah,
        text: "ب ت".to_string(),
        text_hash: "sha256:ee".to_string(),
        char_count: 3,
        token_count: 2,
        global_ayah_index: global,
        juz: Some(1),
        hizb: None,
        rub: None,
        manzil: None,
        ruku: None,
        page: None,
        sajdah: None,
        provenance_id: "prov-1".to_string(),
    }
}

fn token_row(edition: &str, surah: i64, ayah: i64, position: i64, global: i64) -> TokenRow {
    TokenRow {
        edition_id: edition.to_string(),
        surah,
        ayah,
        position,
        surface: if position == 1 { "ب".to_string() } else { "ت".to_string() },
        surface_hash: "sha256:ff".to_string(),
        char_start: position - 1,
        char_end: position,
        byte_start: (position - 1) * 2,
        byte_end: position * 2,
        is_pause_mark: false,
        global_token_index: global,
    }
}

async fn stage_run(
    db: &SqliteDatabase,
    run_id: &str,
    edition_id: &str,
    slug: &str,
    version: &str,
) -> QuranEditionRow {
    let mut uow = db.write().await.unwrap();
    uow.quran()
        .insert_import_run(ImportRunRow {
            run_id: run_id.to_string(),
            job_id: None,
            edition_slug: slug.to_string(),
            edition_version: version.to_string(),
            adapter: "json".to_string(),
            state: "Staged".to_string(),
            created_at: now(),
        })
        .await
        .unwrap();
    let edition = edition_row(edition_id, slug, version, "Staged");
    uow.quran().insert_stg_edition(run_id, edition.clone()).await.unwrap();
    uow.quran().insert_stg_surah(run_id, surah_row(edition_id, 1, 1)).await.unwrap();
    uow.quran().insert_stg_ayah(run_id, ayah_row(edition_id, 1, 1, 1)).await.unwrap();
    for position in [1, 2] {
        uow.quran()
            .insert_stg_token(run_id, token_row(edition_id, 1, 1, position, position))
            .await
            .unwrap();
    }
    for (after, separator) in [(0, ""), (1, " "), (2, "")] {
        uow.quran()
            .insert_stg_separator(
                run_id,
                SeparatorRow {
                    edition_id: edition_id.to_string(),
                    surah: 1,
                    ayah: 1,
                    after_position: after,
                    separator: separator.to_string(),
                },
            )
            .await
            .unwrap();
    }
    uow.commit().await.unwrap();
    edition
}

#[tokio::test]
async fn canonical_triggers_abort_raw_writes_with_codes() {
    let (_dir, db, path) = migrated_db().await;
    stage_run(&db, "run-1", "ed-1", "test", "0.1.0").await;
    let mut uow = db.write().await.unwrap();
    let generation =
        uow.quran().activate_edition("run-1", "ed-1", "principal", "appr-1", &now()).await.unwrap();
    assert_eq!(generation, 1);
    uow.commit().await.unwrap();

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(&path).foreign_keys(true))
        .await
        .unwrap();
    for (sql, code) in [
        ("UPDATE quran_ayahs SET text = 'x' WHERE edition_id = 'ed-1'", "QAI-QUR-0001"),
        ("DELETE FROM quran_ayahs WHERE edition_id = 'ed-1'", "QAI-QUR-0003"),
        ("UPDATE quran_tokens SET surface = 'x' WHERE edition_id = 'ed-1'", "QAI-QUR-0004"),
        ("DELETE FROM quran_tokens WHERE edition_id = 'ed-1'", "QAI-QUR-0005"),
        ("UPDATE quran_editions SET text_hash = 'x' WHERE id = 'ed-1'", "QAI-QUR-0002"),
    ] {
        let err = sqlx::query(sql).execute(&pool).await.unwrap_err();
        let message = err.to_string();
        assert!(message.contains(code), "{sql} must abort with {code}, got: {message}");
    }
    pool.close().await;
}

#[tokio::test]
async fn canonical_tables_declare_the_trigger_set() {
    let (_dir, _db, path) = migrated_db().await;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(&path).read_only(true))
        .await
        .unwrap();
    let rows = sqlx::query("SELECT name FROM sqlite_master WHERE type = 'trigger'")
        .fetch_all(&pool)
        .await
        .unwrap();
    let names: Vec<String> = rows.iter().map(|r| r.get("name")).collect();
    for expected in [
        "trg_ayah_no_update",
        "trg_ayah_no_delete",
        "trg_token_no_update",
        "trg_token_no_delete",
        "trg_edition_immutable_hashes",
    ] {
        assert!(names.iter().any(|n| n == expected), "missing trigger {expected}");
    }
    pool.close().await;
}

#[tokio::test]
async fn staging_tables_are_writable_and_cascade() {
    let (_dir, db, _path) = migrated_db().await;
    stage_run(&db, "run-1", "ed-1", "test", "0.1.0").await;

    let mut uow = db.write().await.unwrap();
    assert_eq!(uow.quran().count_stg_ayahs("run-1").await.unwrap(), 1);
    // Staging rows accept UPDATE/DELETE: no triggers there by design.
    uow.quran().set_import_run_state("run-1", "Cancelled").await.unwrap();
    let run = uow.quran().get_import_run("run-1").await.unwrap().unwrap();
    assert_eq!(run.state, "Cancelled");
    uow.quran().delete_import_run("run-1").await.unwrap();
    assert_eq!(uow.quran().count_stg_ayahs("run-1").await.unwrap(), 0);
    assert!(uow.quran().get_import_run("run-1").await.unwrap().is_none());
    uow.commit().await.unwrap();
}

#[tokio::test]
async fn activation_requires_a_staged_run() {
    let (_dir, db, _path) = migrated_db().await;
    let mut uow = db.write().await.unwrap();
    uow.quran()
        .insert_import_run(ImportRunRow {
            run_id: "run-x".to_string(),
            job_id: None,
            edition_slug: "test".to_string(),
            edition_version: "0.1.0".to_string(),
            adapter: "json".to_string(),
            state: "Running".to_string(),
            created_at: now(),
        })
        .await
        .unwrap();
    let err = uow
        .quran()
        .activate_edition("run-x", "ed-x", "principal", "appr-1", &now())
        .await
        .unwrap_err();
    assert!(matches!(err, storage::StorageError::ConstraintViolation { .. }));
    let err = uow
        .quran()
        .activate_edition("missing", "ed-x", "principal", "appr-1", &now())
        .await
        .unwrap_err();
    assert!(matches!(err, storage::StorageError::NotFound { .. }));
    uow.rollback().await.unwrap();
}

#[tokio::test]
async fn activation_moves_rows_flips_pointer_and_bumps_generation() {
    let (_dir, db, _path) = migrated_db().await;
    stage_run(&db, "run-1", "ed-1", "test", "0.1.0").await;

    let mut uow = db.write().await.unwrap();
    assert_eq!(
        uow.quran().activate_edition("run-1", "ed-1", "principal", "appr-1", &now()).await.unwrap(),
        1
    );
    let edition = uow.quran().get_edition("ed-1").await.unwrap().unwrap();
    assert_eq!(edition.status, "Active");
    let active = uow.quran().get_active().await.unwrap().unwrap();
    assert_eq!((active.edition_id.as_str(), active.corpus_generation), ("ed-1", 1));
    let ayah = uow.quran().get_ayah("ed-1", 1, 1).await.unwrap().unwrap();
    assert_eq!(ayah.text, "ب ت");
    assert_eq!(uow.quran().get_tokens("ed-1", 1, 1).await.unwrap().len(), 2);
    assert_eq!(uow.quran().get_separators("ed-1", 1, 1).await.unwrap().len(), 3);
    assert_eq!(uow.quran().count_ayahs("ed-1").await.unwrap(), 1);
    assert_eq!(uow.quran().count_tokens("ed-1").await.unwrap(), 2);
    // Staging is consumed by activation.
    assert!(uow.quran().get_import_run("run-1").await.unwrap().is_none());
    uow.commit().await.unwrap();

    // A second edition deprecates the first and bumps the generation.
    stage_run(&db, "run-2", "ed-2", "test", "0.2.0").await;
    let mut uow = db.write().await.unwrap();
    assert_eq!(
        uow.quran().activate_edition("run-2", "ed-2", "principal", "appr-1", &now()).await.unwrap(),
        2
    );
    assert_eq!(uow.quran().get_edition("ed-1").await.unwrap().unwrap().status, "Deprecated");
    let active = uow.quran().get_active().await.unwrap().unwrap();
    assert_eq!((active.edition_id.as_str(), active.corpus_generation), ("ed-2", 2));
    assert_eq!(
        uow.quran().get_edition_by_slug_version("test", "0.1.0").await.unwrap().unwrap().id,
        "ed-1"
    );
    assert_eq!(uow.quran().list_editions().await.unwrap().len(), 2);
    uow.commit().await.unwrap();
}

#[tokio::test]
async fn rollback_restores_a_prior_version() {
    let (_dir, db, _path) = migrated_db().await;
    stage_run(&db, "run-1", "ed-1", "test", "0.1.0").await;
    let mut uow = db.write().await.unwrap();
    uow.quran().activate_edition("run-1", "ed-1", "principal", "appr-1", &now()).await.unwrap();
    uow.commit().await.unwrap();
    stage_run(&db, "run-2", "ed-2", "test", "0.2.0").await;
    let mut uow = db.write().await.unwrap();
    uow.quran().activate_edition("run-2", "ed-2", "principal", "appr-1", &now()).await.unwrap();
    uow.commit().await.unwrap();

    let mut uow = db.write().await.unwrap();
    assert_eq!(
        uow.quran().rollback_edition("test", "0.1.0", "principal", "appr-1", &now()).await.unwrap(),
        3
    );
    // ed-1 (test@0.1.0) was Deprecated; it is Active again at generation 3.
    let active = uow.quran().get_active().await.unwrap().unwrap();
    assert_eq!((active.edition_id.as_str(), active.corpus_generation), ("ed-1", 3));
    // Rolling back to the already-Active version is a conflict; unknown is not found.
    let err = uow
        .quran()
        .rollback_edition("test", "0.1.0", "principal", "appr-1", &now())
        .await
        .unwrap_err();
    assert!(matches!(err, storage::StorageError::Conflict));
    let err = uow
        .quran()
        .rollback_edition("test", "9.9.9", "principal", "appr-1", &now())
        .await
        .unwrap_err();
    assert!(matches!(err, storage::StorageError::NotFound { .. }));
    uow.commit().await.unwrap();
}

#[tokio::test]
async fn reads_cover_lookups_ranges_and_divisions() {
    let (_dir, db, _path) = migrated_db().await;
    stage_run(&db, "run-1", "ed-1", "test", "0.1.0").await;
    let mut uow = db.write().await.unwrap();
    uow.quran().activate_edition("run-1", "ed-1", "principal", "appr-1", &now()).await.unwrap();

    assert!(uow.quran().get_surah("ed-1", 1).await.unwrap().is_some());
    assert!(uow.quran().get_surah("ed-1", 99).await.unwrap().is_none());
    assert_eq!(uow.quran().list_surahs("ed-1").await.unwrap().len(), 1);
    assert!(uow.quran().get_ayah_by_global("ed-1", 1).await.unwrap().is_some());
    assert_eq!(uow.quran().list_ayahs_range("ed-1", 1, 1).await.unwrap().len(), 1);
    assert!(uow.quran().list_ayahs_range("ed-1", 2, 9).await.unwrap().is_empty());
    assert!(uow.quran().list_divisions("ed-1", "juz").await.unwrap().is_empty());
    uow.rollback().await.unwrap();
}

#[tokio::test]
async fn reports_citations_and_translations_roundtrip() {
    let (_dir, db, _path) = migrated_db().await;
    stage_run(&db, "run-1", "ed-1", "test", "0.1.0").await;
    let mut uow = db.write().await.unwrap();
    uow.quran().activate_edition("run-1", "ed-1", "principal", "appr-1", &now()).await.unwrap();
    uow.quran()
        .insert_validation_report(ValidationReportRow {
            id: "vr-1".to_string(),
            subject_urn: "quran-edition:test@0.1.0".to_string(),
            validator: "quran_edition_v1".to_string(),
            validator_version: "1.0.0".to_string(),
            outcome: "pass".to_string(),
            fatal_count: 0,
            error_count: 0,
            warning_count: 0,
            findings_json: "[]".to_string(),
            created_at: now(),
        })
        .await
        .unwrap();
    assert!(uow.quran().get_validation_report("vr-1").await.unwrap().is_some());
    uow.quran()
        .insert_difference_report(DifferenceReportRow {
            id: "dr-1".to_string(),
            subject_urn: "quran-edition:test".to_string(),
            from_version: "0.1.0".to_string(),
            to_version: "0.2.0".to_string(),
            differ: "quran_edition".to_string(),
            differ_version: "1.0.0".to_string(),
            summary_json: "{}".to_string(),
            details_json: "[]".to_string(),
            created_at: now(),
        })
        .await
        .unwrap();
    uow.quran()
        .insert_citation(CitationRow {
            id: "cit-1".to_string(),
            kind: "quran".to_string(),
            canonical_reference: "quran:test@0.1.0:1:1".to_string(),
            source_id: Some("src-1".to_string()),
            source_version_id: Some("sv-1".to_string()),
            edition_ref: Some("test@0.1.0".to_string()),
            location_json: "{}".to_string(),
            quoted_text_hash: Some("sha256:ee".to_string()),
            ingestion_version: "1".to_string(),
            resolved_at: now(),
            verdict: "ExactMatch".to_string(),
        })
        .await
        .unwrap();
    assert!(uow.quran().get_citation("cit-1").await.unwrap().is_some());
    assert_eq!(uow.quran().list_citations_by_ref("quran:test@0.1.0:1:1").await.unwrap().len(), 1);
    // Principle 5 at the DB layer: an unattributed translation is rejected.
    uow.quran()
        .insert_translation_edition(TranslationEditionRow {
            id: "tr-1".to_string(),
            slug: "en-test".to_string(),
            version: "1.0.0".to_string(),
            name: "Test".to_string(),
            translator: "Some One".to_string(),
            language: "en".to_string(),
            aligned_edition_id: "ed-1".to_string(),
            numbering_scheme: "hafs".to_string(),
            license_json: "{}".to_string(),
            trust_level: "ImportedUnverified".to_string(),
            source_version_id: "sv-1".to_string(),
            text_hash: "sha256:ab".to_string(),
            status: "Staged".to_string(),
            imported_at: now(),
        })
        .await
        .unwrap();
    uow.quran()
        .insert_translation_passage(TranslationPassageRow {
            translation_edition_id: "tr-1".to_string(),
            surah: 1,
            ayah: 1,
            text: "test words".to_string(),
            footnotes_json: "[]".to_string(),
            provenance_id: "prov-1".to_string(),
        })
        .await
        .unwrap();
    let passage = uow.quran().get_translation_passage("tr-1", 1, 1).await.unwrap().unwrap();
    assert_eq!(passage.text, "test words");
    assert_eq!(uow.quran().list_translation_editions().await.unwrap().len(), 1);
    let mut bad = TranslationEditionRow {
        id: "tr-bad".to_string(),
        slug: "en-bad".to_string(),
        version: "1.0.0".to_string(),
        name: "Bad".to_string(),
        translator: "   ".to_string(),
        language: "en".to_string(),
        aligned_edition_id: "ed-1".to_string(),
        numbering_scheme: "hafs".to_string(),
        license_json: "{}".to_string(),
        trust_level: "ImportedUnverified".to_string(),
        source_version_id: "sv-1".to_string(),
        text_hash: "sha256:ab".to_string(),
        status: "Staged".to_string(),
        imported_at: now(),
    };
    let err = uow.quran().insert_translation_edition(bad.clone()).await.unwrap_err();
    assert!(matches!(err, storage::StorageError::ConstraintViolation { .. }));
    bad.id = "tr-2".to_string();
    bad.translator = "Some One".to_string();
    bad.aligned_edition_id = "missing-edition".to_string();
    let err = uow.quran().insert_translation_edition(bad).await.unwrap_err();
    assert!(matches!(err, storage::StorageError::ConstraintViolation { .. }));
    // A passage for an unknown translation edition is rejected too.
    let err = uow
        .quran()
        .insert_translation_passage(TranslationPassageRow {
            translation_edition_id: "missing-tr".to_string(),
            surah: 1,
            ayah: 1,
            text: "test".to_string(),
            footnotes_json: "[]".to_string(),
            provenance_id: "prov-1".to_string(),
        })
        .await
        .unwrap_err();
    assert!(matches!(err, storage::StorageError::ConstraintViolation { .. }));
    uow.rollback().await.unwrap();
}

#[tokio::test]
async fn word_glosses_roundtrip_in_deterministic_order() {
    let (_dir, db, _path) = migrated_db().await;
    let mut uow = db.write().await.unwrap();
    for dataset in ["gloss-a", "gloss-b"] {
        uow.sources()
            .insert_source(storage::repository::SourceRow {
                id: dataset.to_string(),
                title: dataset.to_string(),
                content_type: "quran_gloss".to_string(),
                language: Some("en".to_string()),
                created_at: now(),
            })
            .await
            .unwrap();
    }
    // Out-of-order inserts come back ordered by (dataset, position, language).
    for (dataset, position, language, gloss) in [
        ("gloss-b", 2, "en", "second"),
        ("gloss-a", 2, "en", "second-a"),
        ("gloss-a", 1, "en", "first"),
        ("gloss-a", 1, "ar", "first-ar"),
    ] {
        uow.quran()
            .insert_word_gloss(WordGlossRow {
                gloss_dataset_id: dataset.to_string(),
                edition_id: "ed-1".to_string(),
                surah: 1,
                ayah: 1,
                position,
                language: language.to_string(),
                gloss: gloss.to_string(),
                provenance_id: "prov-1".to_string(),
            })
            .await
            .unwrap();
    }
    let rows = uow.quran().list_word_glosses("ed-1", 1, 1).await.unwrap();
    assert_eq!(rows.len(), 4);
    let keys: Vec<_> =
        rows.iter().map(|r| (r.gloss_dataset_id.clone(), r.position, r.language.clone())).collect();
    assert_eq!(
        keys,
        vec![
            ("gloss-a".to_string(), 1, "ar".to_string()),
            ("gloss-a".to_string(), 1, "en".to_string()),
            ("gloss-a".to_string(), 2, "en".to_string()),
            ("gloss-b".to_string(), 2, "en".to_string()),
        ]
    );
    // Glosses are namespaced by edition: another edition sees none.
    assert!(uow.quran().list_word_glosses("ed-2", 1, 1).await.unwrap().is_empty());
    // Duplicate (dataset, edition, surah, ayah, position, language) is rejected.
    let err = uow
        .quran()
        .insert_word_gloss(WordGlossRow {
            gloss_dataset_id: "gloss-a".to_string(),
            edition_id: "ed-1".to_string(),
            surah: 1,
            ayah: 1,
            position: 1,
            language: "en".to_string(),
            gloss: "dup".to_string(),
            provenance_id: "prov-1".to_string(),
        })
        .await
        .unwrap_err();
    assert!(matches!(err, storage::StorageError::Conflict));
    // Unknown dataset and unknown provenance are rejected (attribution FKs).
    for (dataset, provenance) in [("missing-src", "prov-1"), ("gloss-a", "missing-prov")] {
        let err = uow
            .quran()
            .insert_word_gloss(WordGlossRow {
                gloss_dataset_id: dataset.to_string(),
                edition_id: "ed-1".to_string(),
                surah: 1,
                ayah: 2,
                position: 1,
                language: "en".to_string(),
                gloss: "x".to_string(),
                provenance_id: provenance.to_string(),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, storage::StorageError::ConstraintViolation { .. }));
    }
    uow.rollback().await.unwrap();
}
