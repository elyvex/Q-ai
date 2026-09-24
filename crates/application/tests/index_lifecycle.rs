//! Phase 2 — index lifecycle acceptance (P2-T35 retention GC, P2-T37 crash
//! matrix, P2-T38 rebuild gate, T36 posting/query parity).
//!
//! Against real SQLite in tempdirs with a real imported + activated edition,
//! real derived forms, and real built generations: retention enforcement
//! (active + previous survive, older generations lose dir + row, search keeps
//! serving), cancellation at every build stage (pointer unchanged, retry
//! succeeds), the cold-rebuild timing gate, and posting-index vs scan parity
//! for concatenated search.

use std::sync::atomic::AtomicBool;
use std::time::Instant;

use application::quran::activate_edition;
use application::quran_forms::{RebuildParams, rebuild_forms};
use application::quran_index::{
    GcParams, IndexBuildError, IndexBuildParams, QURAN_AYAH_INDEX_ID, RollbackParams, gc_index,
    rebuild_index, rollback_index_single_step,
};
use application::quran_search::{MatchMode, SearchParams, search_concatenated};
use domain::{PrincipalId, Timestamp};
use quran_corpus::import::{ImportInput, ImportOptions, ImportOutcome, ImportProgress, run_import};
use quran_corpus::sha256_hex;
use storage::Database as _;
use storage::error::Diagnostic as _;
use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

const BASE_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const PRINCIPAL: &str = "00000000-0000-0000-0000-000000000001";
const CREATED_AT: &str = "2026-09-14T00:00:00Z";
const V1_URN: &str = "quran-edition:test-edition-min@0.1.0";

fn principal() -> PrincipalId {
    PRINCIPAL.parse().unwrap()
}

fn timestamp() -> Timestamp {
    Timestamp::from_ymd_hms(2026, 9, 14, 0, 0, 0).unwrap()
}

async fn migrated_db() -> (tempfile::TempDir, SqliteDatabase) {
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
    ] {
        sqlx::query(&sql).execute(&seed).await.unwrap();
    }
    seed.close().await;
    (dir, db)
}

async fn ready_db() -> (tempfile::TempDir, SqliteDatabase) {
    let (dir, db) = migrated_db().await;
    let outcome = run_import(
        &db,
        &ImportInput {
            run_id: "run-lifecycle-1".into(),
            job_id: None,
            source_version_id: "sv-1".into(),
            adapter: "json".into(),
            manifest_text: BASE_MANIFEST.into(),
            declared_manifest_hash: Some(sha256_hex(BASE_MANIFEST.as_bytes())),
            invoked_by: PRINCIPAL.into(),
            license_status: "PublicDomain".into(),
            license_json: "{}".into(),
            created_at: CREATED_AT.into(),
        },
        &ImportOptions::default(),
        &AtomicBool::new(false),
        ImportProgress::new(),
    )
    .await
    .expect("fixture import completes");
    assert!(matches!(outcome, ImportOutcome::Completed(_)));
    activate_edition(&db, "test-edition-min", "0.1.0", &principal(), "appr-1", &timestamp())
        .await
        .expect("fixture activation completes");
    rebuild_forms(
        &db,
        &RebuildParams {
            edition_slug: "test-edition-min".to_string(),
            edition_version: "0.1.0".to_string(),
            invoked_by: PRINCIPAL.to_string(),
            run_tag: "lifecycle-test".to_string(),
        },
        &AtomicBool::new(false),
        |_| {},
    )
    .await
    .expect("forms rebuild completes");
    (dir, db)
}

fn params(data_dir: &std::path::Path, tag: &str) -> IndexBuildParams {
    IndexBuildParams {
        index_id: QURAN_AYAH_INDEX_ID.to_string(),
        edition_slug: "test-edition-min".to_string(),
        edition_version: "0.1.0".to_string(),
        invoked_by: PRINCIPAL.to_string(),
        run_tag: tag.to_string(),
        data_dir: data_dir.to_path_buf(),
    }
}

async fn build(db: &SqliteDatabase, data_dir: &std::path::Path, tag: &str) {
    rebuild_index(db, &params(data_dir, tag), &AtomicBool::new(false), |_| {})
        .await
        .expect("index build completes");
}

async fn pointer_generation(db: &SqliteDatabase) -> Option<i64> {
    let mut uow = db.write().await.unwrap();
    let pointer = uow.quran().get_index_pointer(QURAN_AYAH_INDEX_ID).await.unwrap();
    uow.rollback().await.unwrap();
    pointer.map(|p| p.generation)
}

fn gc_params(data_dir: &std::path::Path, keep: usize) -> GcParams {
    GcParams {
        index_id: QURAN_AYAH_INDEX_ID.to_string(),
        keep,
        invoked_by: PRINCIPAL.to_string(),
        data_dir: data_dir.to_path_buf(),
    }
}

fn concat_params(text: &str) -> SearchParams {
    SearchParams {
        text: text.to_string(),
        edition: None,
        mode: MatchMode::WholeToken,
        filters: vec![],
        limit: 100,
        offset: 0,
        explain: false,
        highlight: false,
    }
}

/// Derive a spaceless concatenated query from a real fixture ayah: the L6
/// skeleton slice always matches its own ayah (mechanics, not mushaf goldens).
async fn skeleton_query(db: &SqliteDatabase, surah: i64, ayah: i64) -> String {
    let edition_id = {
        let mut uow = db.write().await.unwrap();
        let active = uow.quran().get_active().await.unwrap().expect("active edition");
        uow.rollback().await.unwrap();
        active.edition_id
    };
    let text = {
        let mut uow = db.write().await.unwrap();
        let row =
            uow.quran().get_ayah(&edition_id, surah, ayah).await.unwrap().expect("fixture ayah");
        uow.rollback().await.unwrap();
        row.text
    };
    let registry = application::quran_normalize::builtin_registry();
    let v = quran_normalization::SemVer::new(1, 0, 0);
    let pipe = quran_normalization::NormalizationPipeline::for_profile(
        &registry,
        quran_normalization::ProfileId::L6,
        v,
    )
    .unwrap();
    let skel = pipe.apply(&text).0;
    let chars: Vec<char> = skel.text().chars().collect();
    assert!(chars.len() >= 10, "fixture skeleton long enough for trigram probes");
    chars[2..10].iter().collect()
}

/// P2-T35: three generations → `keep = 2` removes gen-1 (dir + row), keeps
/// gen-2/gen-3, and the pointer still serves gen-3.
#[tokio::test]
async fn gc_keep_two_removes_oldest_and_serves() {
    let (dir, db) = ready_db().await;
    let data_dir = dir.path().join("index");
    build(&db, &data_dir, "g1").await;
    build(&db, &data_dir, "g2").await;
    build(&db, &data_dir, "g3").await;
    assert_eq!(pointer_generation(&db).await, Some(3));

    let report =
        gc_index(&db, &gc_params(&data_dir, 2), &AtomicBool::new(false)).await.expect("gc runs");
    assert_eq!(report.active_generation, Some(3));
    assert_eq!(report.kept, vec![3, 2]);
    assert_eq!(report.removed.len(), 1);
    assert_eq!(report.removed[0].generation, 1);
    assert!(report.removed[0].run_id_deleted.is_some());
    assert_eq!(report.removed[0].docs_removed, 14);

    assert!(!data_dir.join("gen-1").exists());
    assert!(data_dir.join("gen-2").exists());
    assert!(data_dir.join("gen-3").exists());
    assert_eq!(pointer_generation(&db).await, Some(3));

    let mut uow = db.write().await.unwrap();
    let runs = uow.quran().list_build_runs(QURAN_AYAH_INDEX_ID).await.unwrap();
    uow.rollback().await.unwrap();
    assert_eq!(runs.len(), 2);
    assert!(runs.iter().all(|r| r.generation == 2 || r.generation == 3));

    // The serving generation still answers search after GC.
    let query = skeleton_query(&db, 1, 1).await;
    let out = search_concatenated(&db, &data_dir, &concat_params(&query), false, 1).await.unwrap();
    assert!(out.total_matches > 0, "post-GC search must serve");
}

/// P2-T35: `keep = 1` retains only the active generation; `keep = 0`
/// clamps to 1 and still protects the pointer.
#[tokio::test]
async fn gc_keep_one_and_zero_clamp() {
    let (dir, db) = ready_db().await;
    let data_dir = dir.path().join("index");
    build(&db, &data_dir, "g1").await;
    build(&db, &data_dir, "g2").await;

    let report =
        gc_index(&db, &gc_params(&data_dir, 1), &AtomicBool::new(false)).await.expect("gc runs");
    assert_eq!(report.kept, vec![2]);
    assert_eq!(report.removed.len(), 1);
    assert!(!data_dir.join("gen-1").exists());
    assert_eq!(pointer_generation(&db).await, Some(2));

    let report =
        gc_index(&db, &gc_params(&data_dir, 0), &AtomicBool::new(false)).await.expect("gc runs");
    assert_eq!(report.kept, vec![2]);
    assert!(report.removed.is_empty());
    assert_eq!(pointer_generation(&db).await, Some(2));
}

/// P2-T35: GC on a never-built index is a no-op, not an error; GC is
/// idempotent (second pass removes nothing).
#[tokio::test]
async fn gc_empty_and_idempotent() {
    let (dir, db) = ready_db().await;
    let data_dir = dir.path().join("index");
    let report =
        gc_index(&db, &gc_params(&data_dir, 2), &AtomicBool::new(false)).await.expect("gc runs");
    assert!(report.active_generation.is_none());
    assert!(report.kept.is_empty() && report.removed.is_empty());

    build(&db, &data_dir, "g1").await;
    build(&db, &data_dir, "g2").await;
    gc_index(&db, &gc_params(&data_dir, 2), &AtomicBool::new(false)).await.unwrap();
    let again =
        gc_index(&db, &gc_params(&data_dir, 2), &AtomicBool::new(false)).await.expect("gc reruns");
    assert!(again.removed.is_empty());
}

/// P2-T35: cancelled GC stops early and never touches the active generation.
#[tokio::test]
async fn gc_cancel_leaves_active_untouched() {
    let (dir, db) = ready_db().await;
    let data_dir = dir.path().join("index");
    build(&db, &data_dir, "g1").await;
    build(&db, &data_dir, "g2").await;
    let cancel = AtomicBool::new(true);
    let err = gc_index(&db, &gc_params(&data_dir, 1), &cancel).await.unwrap_err();
    assert!(matches!(err, IndexBuildError::Cancelled));
    assert_eq!(pointer_generation(&db).await, Some(2));
    assert!(data_dir.join("gen-2").exists());
}

fn rollback_params(data_dir: &std::path::Path) -> RollbackParams {
    RollbackParams {
        index_id: QURAN_AYAH_INDEX_ID.to_string(),
        invoked_by: PRINCIPAL.to_string(),
        data_dir: data_dir.to_path_buf(),
    }
}

async fn run_states(db: &SqliteDatabase) -> Vec<(i64, String)> {
    let mut uow = db.write().await.unwrap();
    let mut states: Vec<(i64, String)> = uow
        .quran()
        .list_build_runs(QURAN_AYAH_INDEX_ID)
        .await
        .unwrap()
        .into_iter()
        .map(|run| (run.generation, run.state))
        .collect();
    uow.rollback().await.unwrap();
    states.sort_unstable();
    states
}

/// P2-T35: single-step rollback flips the pointer back to the retained
/// previous generation, swaps run states atomically, and keeps serving.
#[tokio::test]
async fn rollback_restores_previous_generation_and_serves() {
    let (dir, db) = ready_db().await;
    let data_dir = dir.path().join("index");
    build(&db, &data_dir, "g1").await;
    build(&db, &data_dir, "g2").await;
    assert_eq!(pointer_generation(&db).await, Some(2));

    let report =
        rollback_index_single_step(&db, &rollback_params(&data_dir)).await.expect("rollback runs");
    assert_eq!(report.index_id, QURAN_AYAH_INDEX_ID);
    assert_eq!((report.from_generation, report.to_generation), (2, 1));
    assert_eq!(pointer_generation(&db).await, Some(1));
    assert_eq!(
        run_states(&db).await,
        vec![(1, "active".to_string()), (2, "superseded".to_string())]
    );
    // Both generations stay on disk; the restored one answers search.
    assert!(data_dir.join("gen-1").exists());
    assert!(data_dir.join("gen-2").exists());
    let query = skeleton_query(&db, 1, 1).await;
    let out = search_concatenated(&db, &data_dir, &concat_params(&query), false, 1).await.unwrap();
    assert!(out.total_matches > 0, "post-rollback search must serve");
}

/// P2-T35: a second rollback does not ping-pong forward — only strictly
/// older generations are rollback targets, so the pointer stays put.
#[tokio::test]
async fn rollback_twice_fails_closed() {
    let (dir, db) = ready_db().await;
    let data_dir = dir.path().join("index");
    build(&db, &data_dir, "g1").await;
    build(&db, &data_dir, "g2").await;
    rollback_index_single_step(&db, &rollback_params(&data_dir)).await.unwrap();

    let err = rollback_index_single_step(&db, &rollback_params(&data_dir)).await.unwrap_err();
    assert!(matches!(err, IndexBuildError::NoPreviousGeneration { .. }));
    assert_eq!(err.code().to_string(), "QAI-IDX-0008");
    assert_eq!(pointer_generation(&db).await, Some(1));
    assert_eq!(
        run_states(&db).await,
        vec![(1, "active".to_string()), (2, "superseded".to_string())]
    );
}

/// P2-T35: with a single generation there is nothing to undo; the pointer
/// is untouched.
#[tokio::test]
async fn rollback_without_previous_fails_closed() {
    let (dir, db) = ready_db().await;
    let data_dir = dir.path().join("index");
    build(&db, &data_dir, "g1").await;

    let err = rollback_index_single_step(&db, &rollback_params(&data_dir)).await.unwrap_err();
    assert!(matches!(err, IndexBuildError::NoPreviousGeneration { .. }));
    assert_eq!(pointer_generation(&db).await, Some(1));
}

/// P2-T35: when the previous directory is gone (GC eviction or manual
/// deletion), rollback refuses rather than pointing at a missing generation.
#[tokio::test]
async fn rollback_after_eviction_fails_closed() {
    let (dir, db) = ready_db().await;
    let data_dir = dir.path().join("index");
    build(&db, &data_dir, "g1").await;
    build(&db, &data_dir, "g2").await;
    std::fs::remove_dir_all(data_dir.join("gen-1")).unwrap();

    let err = rollback_index_single_step(&db, &rollback_params(&data_dir)).await.unwrap_err();
    assert!(matches!(err, IndexBuildError::PreviousGenerationEvicted { .. }));
    assert_eq!(err.code().to_string(), "QAI-IDX-0009");
    assert_eq!(pointer_generation(&db).await, Some(2));
    assert_eq!(
        run_states(&db).await,
        vec![(1, "superseded".to_string()), (2, "active".to_string())]
    );
}

/// P2-T37: cancellation at every build stage leaves the pointer unchanged
/// (no partial index ever serves) and the retry succeeds.
#[tokio::test]
async fn crash_matrix_cancel_at_each_stage_never_serves_partial() {
    // Note: the per-batch `build` checkpoint fires only on corpora larger
    // than one batch (500 ayahs); on the fixture it never fires, so the
    // `build` kill trips at `staged` instead — the identical code path
    // (cancel check inside the batch loop, mid-build, partial staging
    // never activated).
    for stage in ["resolved", "mv018-pre", "staged", "build", "built", "trigram", "verified"] {
        let (dir, db) = ready_db().await;
        let data_dir = dir.path().join("index");
        let trip_at = if stage == "build" { "staged" } else { stage };
        let cancel = AtomicBool::new(false);
        let hit_stage = std::sync::Mutex::new(false);
        let result =
            rebuild_index(&db, &params(&data_dir, &format!("kill-{stage}")), &cancel, |seen| {
                if seen == trip_at {
                    *hit_stage.lock().unwrap() = true;
                    cancel.store(true, std::sync::atomic::Ordering::SeqCst);
                }
            })
            .await;
        let err = match result {
            Err(err) => err,
            Ok(report) => panic!(
                "stage {stage}: build succeeded despite cancellation (gen {})",
                report.generation
            ),
        };
        assert!(
            *hit_stage.lock().unwrap(),
            "stage {stage} must be reached before cancellation trips"
        );
        assert!(
            matches!(err, IndexBuildError::Cancelled),
            "stage {stage}: expected Cancelled, got {err:?}"
        );
        assert_eq!(pointer_generation(&db).await, None, "stage {stage}: pointer must stay empty");

        // Retry without cancellation activates cleanly (staging is rebuilt).
        // Generation is monotonic and the run-row insert precedes every
        // cancel check, so each killed attempt consumes gen-1 and the retry
        // is always gen-2 — never a reuse, never a partial serve.
        let report =
            rebuild_index(&db, &params(&data_dir, "retry"), &AtomicBool::new(false), |_| {})
                .await
                .expect("retry after cancel succeeds");
        assert_eq!(report.generation, 2, "stage {stage}: retry generation");
        assert_eq!(pointer_generation(&db).await, Some(2));
    }
}

/// P2-T37: cancelling a second build keeps the previous generation serving.
#[tokio::test]
async fn crash_matrix_cancel_during_rebuild_keeps_previous_serving() {
    let (dir, db) = ready_db().await;
    let data_dir = dir.path().join("index");
    build(&db, &data_dir, "g1").await;
    let cancel = AtomicBool::new(false);
    let err = rebuild_index(&db, &params(&data_dir, "kill-g2"), &cancel, |seen| {
        if seen == "built" {
            cancel.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    })
    .await
    .unwrap_err();
    assert!(matches!(err, IndexBuildError::Cancelled));
    assert_eq!(pointer_generation(&db).await, Some(1));
    assert!(data_dir.join("gen-1").exists());
    // Gen-1 still serves search.
    let query = skeleton_query(&db, 1, 1).await;
    let out = search_concatenated(&db, &data_dir, &concat_params(&query), false, 1).await.unwrap();
    assert!(out.total_matches > 0);
}

/// P2-T38: cold rebuild timing gate (< 6 min total). The fixture rebuilds in
/// seconds; the assertion locks the CI gate shape and records the elapsed
/// time for the full-corpus runbook (T38 follow-up on licensed data).
#[tokio::test]
async fn cold_rebuild_completes_within_gate() {
    let (dir, db) = ready_db().await;
    let data_dir = dir.path().join("index");
    let started = Instant::now();
    let report = rebuild_index(&db, &params(&data_dir, "timed"), &AtomicBool::new(false), |_| {})
        .await
        .expect("timed build completes");
    let elapsed = started.elapsed();
    eprintln!(
        "cold rebuild: {} docs in {:.2}s (gate: < 360s; trigram postings: {})",
        report.doc_count,
        elapsed.as_secs_f64(),
        report.trigram_postings
    );
    assert!(elapsed.as_secs() < 360, "rebuild exceeded the 6-minute gate: {elapsed:?}");
    assert_eq!(report.doc_count, 14);
    assert!(report.trigram_postings > 0, "T36 postings must be built");
    assert!(data_dir.join("gen-1").join("trigram.db").exists());
}

/// T36 parity: concatenated search returns identical references through the
/// posting index and through the scan fallback (trigram file removed).
#[tokio::test]
async fn concatenated_postings_match_scan_fallback() {
    let (dir, db) = ready_db().await;
    let data_dir = dir.path().join("index");
    build(&db, &data_dir, "g1").await;
    assert!(data_dir.join("gen-1").join("trigram.db").exists());

    let queries = vec![
        skeleton_query(&db, 1, 1).await,
        skeleton_query(&db, 2, 1).await,
        skeleton_query(&db, 3, 2).await,
    ];
    assert!(queries.iter().all(|q| q.chars().count() >= 8));
    let mut posted: Vec<Vec<String>> = Vec::new();
    for query in &queries {
        let out =
            search_concatenated(&db, &data_dir, &concat_params(query), false, 1).await.unwrap();
        assert!(!out.hits.is_empty(), "derived query {query:?} must match its own ayah");
        posted.push(out.hits.iter().map(|h| h.reference().to_string()).collect());
    }
    // Remove the posting file: the reader must fall back to the scan.
    std::fs::remove_file(data_dir.join("gen-1").join("trigram.db")).unwrap();
    for (query, before) in queries.iter().zip(posted.iter()) {
        let out =
            search_concatenated(&db, &data_dir, &concat_params(query), false, 1).await.unwrap();
        let after: Vec<String> = out.hits.iter().map(|h| h.reference().to_string()).collect();
        assert_eq!(&after, before, "postings/scan parity failed for {query:?}");
    }
}
