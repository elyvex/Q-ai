//! 05-03 cross-surface parity gate (D-16/SC3): every registered research
//! tool — all 12 `ToolRegistry::TOOL_NAMES` — driven through three genuinely
//! distinct legs (shared service dispatch, the generic HTTP typed-tool route,
//! the `qai` CLI binary) over one seeded fixture, asserting byte-identical
//! normalized payloads and identical `research_checksum` values.
//!
//! The comparison unit is the serialized `ToolResult` minus wall-clock
//! fields — never the HTTP `Envelope`, never CLI output against CLI output.

use std::sync::{Arc, atomic::AtomicBool};

use application::quran_forms::{RebuildParams, rebuild_forms};
use application::quran_graph_api::FileGraphBackend;
use application::quran_graph_build::{collect_structural_input, publish_structural_build};
use application::quran_graph_tools::GraphToolBackend;
use application::quran_index::{
    IndexBuildParams, QURAN_AYAH_INDEX_ID, index_root_for_db, rebuild_index,
};
use application::quran_lexicon_api::LexiconApiService;
use application::quran_morphology::{
    MorphologyActivateParams, MorphologyImportParams, activate_morphology, dataset_urn,
    run_morphology_import,
};
use application::quran_search_api::SearchApiService;
use application::quran_tools::{ReaderToolBackend, dispatch_registered_tool};
use domain::hashing::canonical_json_bytes;
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use storage::Database as _;
use storage_sqlite::SqliteDatabase;
use tower::ServiceExt as _;

const MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const SLUG: &str = "test-edition-min";
const VERSION: &str = "0.1.0";
const PRINCIPAL: &str = "00000000-0000-0000-0000-000000000001";
const CREATED_AT: &str = "2026-09-14T00:00:00Z";
const SOURCE_VERSION_ID: &str = "12345678-1234-1234-1234-123456789abc";
const MORPH_SLUG: &str = "test-morph";
const MORPH_VERSION: &str = "0.1.0";
const MORPH_BATCH: &str = "parity-batch";
const LICENSE_JSON: &str = "{\"status\":\"PublicDomain\",\"spdx_id\":null,\"name\":null,\
                              \"url\":null,\"attribution_required\":false,\
                              \"redistribution_allowed\":true,\"export_allowed\":true,\
                              \"notes\":null}";

/// One parity row: canonical tool name + representative params valid
/// against the seeded fixture.
struct Row {
    name: &'static str,
    params: serde_json::Value,
}

fn fixtures(search_surface: &str) -> Vec<Row> {
    vec![
        Row { name: "quran.get_ayah", params: serde_json::json!({"reference": "2:1"}) },
        Row { name: "quran.get_context", params: serde_json::json!({"reference": "2:1"}) },
        Row { name: "quran.search", params: serde_json::json!({"text": search_surface}) },
        Row { name: "quran.root", params: serde_json::json!({"root": "root-0"}) },
        Row { name: "quran.lemma", params: serde_json::json!({"lemma": "lem-0"}) },
        Row {
            name: "quran.morphology",
            params: serde_json::json!({"surah": 1, "ayah": 1, "position": 1}),
        },
        Row {
            name: "quran.family",
            params: serde_json::json!({"kind": "token", "id": "token:1:1:1"}),
        },
        Row { name: "quran.graph_neighbors", params: serde_json::json!({"node": "ayah:1:1"}) },
        Row {
            name: "quran.graph_path",
            params: serde_json::json!({"from": "ayah:1:1", "to": "ayah:1:2"}),
        },
        Row { name: "quran.graph_subgraph", params: serde_json::json!({"seeds": ["ayah:1:1"]}) },
        Row {
            name: "quran.graph_pattern",
            params: serde_json::json!({"seeds": ["ayah:1:1"], "steps": [{"edge": "NEXT"}]}),
        },
        Row { name: "quran.graph_root_family", params: serde_json::json!({"root": "root-0"}) },
    ]
}

/// Strip wall-clock fields (`execution_time_ms`, any `duration_ms`) and
/// serialize through the frozen canonical-JSON recipe: byte-identical
/// inputs yield byte-identical outputs regardless of field order.
fn normalize(value: &serde_json::Value) -> Vec<u8> {
    fn scrub(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                map.remove("execution_time_ms");
                map.remove("duration_ms");
                for field in map.values_mut() {
                    scrub(field);
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    scrub(item);
                }
            }
            _ => {}
        }
    }
    let mut owned = value.clone();
    scrub(&mut owned);
    canonical_json_bytes(&owned).expect("fixture JSON serializes")
}

fn checksum_hex(result: &serde_json::Value) -> String {
    result
        .get("research_checksum")
        .and_then(|checksum| checksum.get("hex"))
        .and_then(|hex| hex.as_str())
        .unwrap_or_default()
        .to_string()
}

struct Seed {
    _dir: tempfile::TempDir,
    dir: std::path::PathBuf,
    db_path: String,
    surface: String,
}

fn principal() -> domain::PrincipalId {
    PRINCIPAL.parse().unwrap()
}

fn timestamp() -> domain::Timestamp {
    domain::Timestamp::from_ymd_hms(2026, 9, 14, 0, 0, 0).unwrap()
}

/// Seed one scratch database exactly as the application acceptance suites
/// do: migrated, canonical import + activation, derived forms, serving
/// index, morphology import + activation, structural graph projection.
async fn seed() -> Seed {
    let dir = tempfile::tempdir().unwrap();
    let db_file = dir.path().join("qai.db");
    let db_path = db_file.to_str().unwrap().to_string();
    let repo_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    storage_sqlite::migrate::apply_migrations(&db_path, &repo_root).await.unwrap();
    let db = Arc::new(SqliteDatabase::new(&db_path, 4, true).await.unwrap());

    let setup = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new().filename(&db_path).foreign_keys(true),
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
        format!(
            "INSERT INTO source_versions
            (id, source_id, version, schema_version, state, trust_level,
             license_status, license_json, created_at)
         VALUES ('{SOURCE_VERSION_ID}', 'src-1', '0.1.0', 1, 'Staged', 'ImportedUnverified',
                 'PublicDomain', '{{}}', '2026-09-14T00:00:00Z')"
        ),
        format!(
            "INSERT INTO approvals
                (id, subject_urn, kind, requested_by, decided_by, decision,
                 request_payload, requested_at, decided_at)
             VALUES ('appr-1', 'quran-edition:{SLUG}@{VERSION}', 'CanonicalChange',
                     '{PRINCIPAL}', '{PRINCIPAL}', 'approved', '{{}}', '{CREATED_AT}', '{CREATED_AT}')"
        ),
        format!(
            "INSERT INTO approvals
                (id, subject_urn, kind, requested_by, decided_by, decision,
                 request_payload, requested_at, decided_at)
             VALUES ('appr-morph', '{}', 'CanonicalChange', '{PRINCIPAL}', '{PRINCIPAL}',
                     'approved', '{{}}', '{CREATED_AT}', '{CREATED_AT}')",
            dataset_urn(MORPH_SLUG, MORPH_VERSION),
        ),
    ] {
        sqlx::query(&sql).execute(&setup).await.unwrap();
    }
    setup.close().await;

    let outcome = run_import(
        &*db,
        &ImportInput {
            run_id: "22222222-3333-4444-8555-666666666666".into(),
            job_id: None,
            source_version_id: SOURCE_VERSION_ID.into(),
            adapter: "json".into(),
            manifest_text: MANIFEST.into(),
            declared_manifest_hash: Some(quran_corpus::sha256_hex(MANIFEST.as_bytes())),
            invoked_by: PRINCIPAL.into(),
            license_status: "PublicDomain".into(),
            license_json: LICENSE_JSON.into(),
            created_at: CREATED_AT.into(),
            reference_manifest_text: None,
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

    rebuild_forms(
        &db,
        &RebuildParams {
            edition_slug: SLUG.to_string(),
            edition_version: VERSION.to_string(),
            invoked_by: PRINCIPAL.to_string(),
            run_tag: "parity-forms".to_string(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("forms rebuild completes");
    rebuild_index(
        &db,
        &IndexBuildParams {
            index_id: QURAN_AYAH_INDEX_ID.to_string(),
            edition_slug: SLUG.to_string(),
            edition_version: VERSION.to_string(),
            invoked_by: PRINCIPAL.to_string(),
            run_tag: "parity-index".to_string(),
            data_dir: dir.path().join("index"),
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

    let collected = collect_structural_input(&db)
        .await
        .expect("collect from active edition")
        .expect("active edition present");
    publish_structural_build(&db_path, &collected).await.expect("structural build publishes");

    let surface = first_token_surface(&db).await;
    assert!(!surface.is_empty(), "fixture has a token surface");
    Seed { _dir: dir, dir: dir_path(&db_path), db_path, surface }
}

fn dir_path(db_path: &str) -> std::path::PathBuf {
    std::path::Path::new(db_path).parent().unwrap().to_path_buf()
}

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

/// Two analyses per token over the active edition (the quran_tools recipe).
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

struct Legs {
    reader_registry: tool_registry::ToolRegistry,
    graph_registry: tool_registry::ToolRegistry,
    state: server::api::AppState,
    seed: Seed,
}

async fn legs() -> Legs {
    let seed = seed().await;
    let reader =
        Arc::new(application::quran_cli::open_reader(&seed.db_path).await.expect("reader opens"));
    let reader_registry = ReaderToolBackend::registry_with_index_root(
        reader.clone(),
        index_root_for_db(&seed.db_path),
    );
    let graph_registry =
        GraphToolBackend::registry(Arc::new(FileGraphBackend::structural(&seed.db_path)));
    let state = server::api::AppState {
        tools: Arc::new(ReaderToolBackend::registry_with_index_root(
            reader.clone(),
            index_root_for_db(&seed.db_path),
        )),
        api: Arc::new(server::api::ReaderBackend::new(reader)),
        search: Arc::new(
            SearchApiService::open(&seed.db_path).await.expect("search backend opens"),
        ),
        lexicon: Arc::new(
            LexiconApiService::open(&seed.db_path).await.expect("lexicon backend opens"),
        ),
        graph: Arc::new(FileGraphBackend::structural(&seed.db_path)),
    };
    Legs { reader_registry, graph_registry, state, seed }
}

/// Service leg: the shared dispatcher over real backends.
async fn service_leg(legs: &Legs, row: &Row) -> serde_json::Value {
    dispatch_registered_tool(
        &legs.reader_registry,
        &legs.graph_registry,
        row.name,
        row.params.clone(),
    )
    .await
    .unwrap_or_else(|err| panic!("service leg fails for {}: {err:?}", row.name))
}

/// HTTP leg: the generic typed-tool route over the same handles.
async fn http_leg(legs: &Legs, row: &Row) -> serde_json::Value {
    let request = axum::http::Request::builder()
        .method(axum::http::Method::POST)
        .uri(format!("/api/v1/quran/tool/{}", row.name))
        .header("content-type", "application/json")
        .body(axum::body::Body::from(row.params.to_string()))
        .unwrap();
    let response = server::api::router(legs.state.clone()).oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        axum::http::StatusCode::OK,
        "HTTP leg fails for {}: {}",
        row.name,
        response.status()
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let envelope: serde_json::Value = serde_json::from_slice(&body).expect("envelope is JSON");
    envelope.get("data").cloned().expect("envelope carries data")
}

/// CLI leg: the built binary against the seeded `--data-dir`.
fn cli_leg(legs: &Legs, row: &Row) -> serde_json::Value {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_qai"))
        .args(["--data-dir", legs.seed.dir.to_str().unwrap()])
        .args(["quran", "tool", row.name, "--params", &row.params.to_string(), "--json"])
        .output()
        .expect("spawn qai");
    assert!(
        output.status.success(),
        "CLI leg fails for {}: {}",
        row.name,
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("CLI emits JSON ToolResult")
}

#[test]
fn fixtures_cover_all_tool_names() {
    let rows = fixtures("surface");
    let mut names: Vec<&str> = rows.iter().map(|row| row.name).collect();
    names.sort_unstable();
    let mut expected = tool_registry::ToolRegistry::TOOL_NAMES.to_vec();
    expected.sort_unstable();
    assert_eq!(names, expected, "one FIXTURES row per registered tool");
}

#[test]
fn normalize_drops_timing_and_detects_perturbation() {
    let value = serde_json::json!({
        "tool_name": "quran.get_ayah",
        "execution_time_ms": 12.5,
        "results": {"explanation": {"duration_ms": 3}},
        "research_checksum": {"hex": "abc"},
    });
    let clean = normalize(&value);
    let text = String::from_utf8(clean.clone()).unwrap();
    assert!(!text.contains("execution_time_ms"), "wall-clock field stripped");
    assert!(!text.contains("duration_ms"), "nested timing stripped");
    assert!(text.contains("abc"), "content survives normalization");

    let mut perturbed = value.clone();
    perturbed["results"] = serde_json::json!({"other": true});
    assert_ne!(normalize(&perturbed), clean, "perturbed payload compares different");
}

#[tokio::test]
async fn service_and_http_legs_match() {
    let legs = legs().await;
    let rows = fixtures(&legs.seed.surface);
    for row in &rows {
        let service = service_leg(&legs, row).await;
        let http = http_leg(&legs, row).await;
        assert_eq!(
            normalize(&service),
            normalize(&http),
            "service vs HTTP payload differs for {}",
            row.name
        );
        assert_eq!(
            checksum_hex(&service),
            checksum_hex(&http),
            "service vs HTTP checksum differs for {}",
            row.name
        );
        assert!(!checksum_hex(&service).is_empty(), "checksum present for {}", row.name);
    }
}

#[tokio::test]
async fn cli_leg_matches() {
    let legs = legs().await;
    let rows = fixtures(&legs.seed.surface);
    for row in &rows {
        let service = service_leg(&legs, row).await;
        let cli = cli_leg(&legs, row);
        assert_eq!(
            normalize(&service),
            normalize(&cli),
            "service vs CLI payload differs for {}",
            row.name
        );
        assert_eq!(
            checksum_hex(&service),
            checksum_hex(&cli),
            "service vs CLI checksum differs for {}",
            row.name
        );
    }
}

#[tokio::test]
async fn empty_query_is_typed_error_on_every_leg() {
    let legs = legs().await;
    let params = serde_json::json!({"text": "   "});
    let err = dispatch_registered_tool(
        &legs.reader_registry,
        &legs.graph_registry,
        "quran.search",
        params.clone(),
    )
    .await
    .unwrap_err();
    assert!(
        matches!(err, tools::ToolError::InvalidInput { .. }),
        "service leg types the empty query: {err:?}"
    );

    let request = axum::http::Request::builder()
        .method(axum::http::Method::POST)
        .uri("/api/v1/quran/tool/quran.search")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(params.to_string()))
        .unwrap();
    let response = server::api::router(legs.state.clone()).oneshot(request).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_qai"))
        .args(["--data-dir", legs.seed.dir.to_str().unwrap()])
        .args(["quran", "tool", "quran.search", "--params", &params.to_string(), "--json"])
        .output()
        .expect("spawn qai");
    assert!(!output.status.success(), "CLI leg rejects the empty query");
}
