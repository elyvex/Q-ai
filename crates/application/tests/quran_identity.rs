//! Phase 2 (plan 02-01) — edition identity acceptance.
//!
//! A manifest-declared upstream identity (`upstream_edition_slug`,
//! `qai_edition_id`) and explicit primary/default designation (`is_primary`)
//! survive import → staging → activation and are readable through the
//! canonical row, the reader view, and the application-level
//! `application::quran_cli::cmd_edition_show` surface. An undeclared manifest
//! leaves every value absent (NULL / `None` / `false`); nothing is invented.
//!
//! The `qai` binary trycmd snapshot for the same surface is owned by plan
//! 02-03; this suite asserts the surface at the application level.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use application::quran::activate_edition;
use application::quran_cli::cmd_edition_show;
use application::quran_reader::{QuranReader, QuranReaderService};
use domain::{PrincipalId, SemVer, Timestamp};
use quran_core::EditionSelector;
use quran_corpus::EditionSource;
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use quran_corpus::sha256_hex;
use storage::Database as _;
use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

const BASE_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const RUN_ID: &str = "11111111-2222-4333-8444-555555555555";
const PRINCIPAL: &str = "00000000-0000-0000-0000-000000000001";
const CREATED_AT: &str = "2026-09-14T00:00:00Z";
const SLUG: &str = "test-edition-min";
const VERSION: &str = "0.1.0";
const V1_URN: &str = "quran-edition:test-edition-min@0.1.0";
const V2_URN: &str = "quran-edition:test-edition-min@0.2.0";
const DECLARED_UPSTREAM: &str = "ara-QuranUthmanihaf";
const DECLARED_QAI_ID: &str = "qai-hafs-uthmani";
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

fn pinned() -> EditionSelector {
    EditionSelector::Pinned {
        slug: SLUG.to_string(),
        version: VERSION.parse::<SemVer>().unwrap(),
    }
}

fn input(run_id: &str, manifest: &str) -> ImportInput {
    ImportInput {
        run_id: run_id.into(),
        job_id: None,
        source_version_id: "12345678-1234-1234-1234-123456789abc".into(),
        adapter: "json".into(),
        manifest_text: manifest.into(),
        declared_manifest_hash: Some(sha256_hex(manifest.as_bytes())),
        invoked_by: PRINCIPAL.into(),
        license_status: "PublicDomain".into(),
        license_json: LICENSE_JSON.into(),
        created_at: CREATED_AT.into(),
    }
}

/// Build a manifest from the base fixture with the identity fields overridden.
fn identity_manifest(upstream: Option<&str>, qai_id: Option<&str>, is_primary: bool) -> String {
    let mut source: EditionSource = serde_json::from_str(BASE_MANIFEST).unwrap();
    source.edition.upstream_edition_slug = upstream.map(str::to_string);
    source.edition.qai_edition_id = qai_id.map(str::to_string);
    source.edition.is_primary = is_primary;
    serde_json::to_string(&source).unwrap()
}

/// Migrated, seeded temp database (FK parents + approvals for v1 and v2).
async fn migrated_db() -> (tempfile::TempDir, Arc<SqliteDatabase>, String) {
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
         VALUES ('12345678-1234-1234-1234-123456789abc', 'src-1', '0.1.0', 1, 'Staged',
                 'ImportedUnverified', 'PublicDomain', '{}', '2026-09-14T00:00:00Z')"
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
             VALUES ('appr-2', '{V2_URN}', 'CanonicalChange', '{PRINCIPAL}', '{PRINCIPAL}',
                     'approved', '{{}}', '{CREATED_AT}', '{CREATED_AT}')"
        ),
    ] {
        sqlx::query(&sql).execute(&seed).await.unwrap();
    }
    seed.close().await;
    (dir, db, path_str)
}

/// Import `manifest` under `run_id` and activate it.
async fn import_and_activate(db: &SqliteDatabase, run_id: &str, manifest: &str, approval: &str) {
    let outcome = run_import(
        db,
        &input(run_id, manifest),
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .expect("import completes");
    assert!(matches!(outcome, ImportOutcome::Completed(_)));
    activate_edition(db, SLUG, VERSION, &principal(), approval, &timestamp())
        .await
        .expect("activation completes");
}

#[tokio::test]
async fn declared_identity_and_primary_survive_to_the_operator_surface() {
    let (_dir, db, path) = migrated_db().await;
    let manifest = identity_manifest(Some(DECLARED_UPSTREAM), Some(DECLARED_QAI_ID), true);
    import_and_activate(&db, RUN_ID, &manifest, "appr-1").await;

    // Canonical row: the declared values are carried byte-for-byte (no trim,
    // case fold, or normalization of the identity string).
    let mut uow = db.write().await.unwrap();
    let row = uow
        .quran()
        .get_edition_by_slug_version(SLUG, VERSION)
        .await
        .unwrap()
        .expect("canonical edition row");
    assert_eq!(row.upstream_edition_slug.as_deref(), Some(DECLARED_UPSTREAM));
    assert_eq!(row.qai_edition_id.as_deref(), Some(DECLARED_QAI_ID));
    assert!(row.is_primary, "declared primary flag must persist");
    uow.rollback().await.unwrap();

    // Reader view: the same values ride through the canonical `QuranEdition`.
    let reader = QuranReaderService::new(db.clone());
    let edition = reader.get_edition(&pinned()).await.unwrap();
    assert_eq!(edition.upstream_edition_slug.as_deref(), Some(DECLARED_UPSTREAM));
    assert_eq!(edition.qai_edition_id.as_deref(), Some(DECLARED_QAI_ID));
    assert!(edition.is_primary);

    // Operator surface: human + `--json` carry the declared identity/marker.
    let show = cmd_edition_show(&path, &format!("{SLUG}@{VERSION}"), false, false).await;
    assert_eq!(show.exit, 0, "edition show must succeed: {}", show.human);
    assert!(show.human.contains(&format!("upstream: {DECLARED_UPSTREAM}")), "{}", show.human);
    assert!(show.human.contains(&format!("qai-id: {DECLARED_QAI_ID}")), "{}", show.human);
    assert!(show.human.contains("primary: true"), "{}", show.human);
    assert_eq!(show.json["upstream_edition_slug"], DECLARED_UPSTREAM);
    assert_eq!(show.json["qai_edition_id"], DECLARED_QAI_ID);
    assert_eq!(show.json["is_primary"], true);
}

#[tokio::test]
async fn undeclared_identity_stays_absent() {
    let (_dir, db, path) = migrated_db().await;
    import_and_activate(&db, RUN_ID, BASE_MANIFEST, "appr-1").await;

    let mut uow = db.write().await.unwrap();
    let row = uow
        .quran()
        .get_edition_by_slug_version(SLUG, VERSION)
        .await
        .unwrap()
        .expect("canonical edition row");
    assert_eq!(row.upstream_edition_slug, None);
    assert_eq!(row.qai_edition_id, None);
    assert!(!row.is_primary);
    uow.rollback().await.unwrap();

    let reader = QuranReaderService::new(db.clone());
    let edition = reader.get_edition(&pinned()).await.unwrap();
    assert_eq!(edition.upstream_edition_slug, None);
    assert_eq!(edition.qai_edition_id, None);
    assert!(!edition.is_primary);

    let show = cmd_edition_show(&path, &format!("{SLUG}@{VERSION}"), false, false).await;
    assert_eq!(show.exit, 0);
    assert!(!show.human.contains("upstream:"), "{}", show.human);
    assert!(!show.human.contains("qai-id:"), "{}", show.human);
    assert!(!show.human.contains("primary:"), "{}", show.human);
    assert!(show.json["upstream_edition_slug"].is_null());
    assert!(show.json["qai_edition_id"].is_null());
    assert_eq!(show.json["is_primary"], false);
}
