//! Phase 2 — normalization seed parity + repo acceptance (M1c, P2-T19).
//!
//! Against real SQLite in a tempdir: the migration seed must equal the code
//! (any drift fails here, never silently in production), the repo reads it
//! back, new versions insert, and the append-only trigger rejects rewrites.

use application::quran_normalize;
use quran_normalization::{ProfileId, ProfileRegistry, RuleId, SemVer};
use storage::Database as _;
use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

async fn migrated_db() -> (tempfile::TempDir, String, SqliteDatabase) {
    let dir = tempdir().unwrap();
    let path = dir.path().join("qai.db");
    let path_str = path.to_str().unwrap().to_string();
    let repo_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    storage_sqlite::migrate::apply_migrations(&path_str, &repo_root).await.unwrap();
    let db = SqliteDatabase::new(&path_str, 4, true).await.unwrap();
    (dir, path_str, db)
}

async fn rows(
    db: &SqliteDatabase,
) -> (Vec<storage::quran::NormalizationRuleRow>, Vec<storage::quran::NormalizationProfileRow>) {
    let mut uow = db.write().await.unwrap();
    let repo = uow.quran();
    let rules = repo.list_normalization_rules().await.unwrap();
    let profiles = repo.list_normalization_profiles().await.unwrap();
    (rules, profiles)
}

/// The migration seed equals the code: 22 rules with matching kinds and
/// descriptions, 9 profiles with matching ladders, labels, and flags.
#[tokio::test]
async fn seed_matches_code() {
    let (_dir, _path, db) = migrated_db().await;
    let (rule_rows, profile_rows) = rows(&db).await;

    let mut expected_rules = quran_normalize::all_rule_metas();
    expected_rules.sort_by_key(|m| m.id.as_str());
    let mut actual_rules = rule_rows.clone();
    actual_rules.sort_by_key(|r| r.rule_id.clone());
    assert_eq!(actual_rules.len(), expected_rules.len());
    for (actual, expected) in actual_rules.iter().zip(expected_rules.iter()) {
        assert_eq!(actual.rule_id, expected.id.as_str());
        assert_eq!(actual.version, expected.version.to_string());
        assert_eq!(actual.kind, if expected.heuristic { "heuristic" } else { "deterministic" });
        assert_eq!(actual.description, expected.description);
    }

    let registry = ProfileRegistry::new();
    let v = SemVer::new(1, 0, 0);
    assert_eq!(profile_rows.len(), 9);
    for row in &profile_rows {
        let id = ProfileId::parse(&row.profile_id).unwrap();
        let profile = registry.get(id, v).unwrap();
        assert_eq!(row.version, v.to_string());
        assert_eq!(row.label, profile.label);
        let stored: Vec<String> = serde_json::from_str(&row.rules_json).unwrap();
        let coded: Vec<String> = profile.rules.iter().map(|r| r.as_str().to_string()).collect();
        assert_eq!(stored, coded, "seed drift for {}", row.profile_id);
        assert_eq!(row.indexed, profile.indexed);
        assert_eq!(row.heuristic, profile.heuristic);
        assert_eq!(row.experimental, profile.experimental);
    }
}

/// The CLI path (registry rebuilt from seeded rows) normalizes identically
/// to the built-in path (server preview path).
#[tokio::test]
async fn db_registry_matches_builtin_path() {
    let (_dir, _path, db) = migrated_db().await;
    let (_, profile_rows) = rows(&db).await;
    let db_registry = quran_normalize::registry_from_rows(&profile_rows).unwrap();
    let builtin = quran_normalize::builtin_registry();

    for text in ["بِسْمِ ٱللَّهِ", "والكتب", "plain ascii", ""] {
        for id in [ProfileId::L0, ProfileId::L3, ProfileId::L5, ProfileId::L6, ProfileId::L7] {
            let from_db = quran_normalize::preview(&db_registry, text, id, None).unwrap();
            let from_builtin = quran_normalize::preview(&builtin, text, id, None).unwrap();
            assert_eq!(from_db.output, from_builtin.output, "{id} {text:?}");
            assert_eq!(from_db.trace, from_builtin.trace, "{id} {text:?}");
        }
    }
    // Adhoc parity as well.
    let ids = [RuleId::N01, RuleId::N03, RuleId::N18];
    let adhoc = quran_normalize::preview_adhoc("والكتب", &ids).unwrap();
    assert!(adhoc.profile.starts_with("adhoc:"));
    assert!(adhoc.trace.contains_heuristic_rules);
}

/// Profile lookup by version, new-version insert, and trigger enforcement.
#[tokio::test]
async fn profile_lifecycle_and_append_only_trigger() {
    let (_dir, path, db) = migrated_db().await;
    let mut uow = db.write().await.unwrap();

    let found = uow.quran().get_normalization_profile("L3.diacritics", "1.0.0").await.unwrap();
    assert!(found.is_some());
    let missing = uow.quran().get_normalization_profile("L3.diacritics", "9.9.9").await.unwrap();
    assert!(missing.is_none());

    // A new version inserts fine.
    let mut v2 = found.unwrap();
    v2.version = "2.0.0".to_string();
    uow.quran().insert_normalization_profile(v2).await.unwrap();
    let listed = uow.quran().list_normalization_profiles().await.unwrap();
    assert_eq!(listed.len(), 10);
    drop(uow);

    // Rewriting a seeded row is rejected by the append-only trigger.
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite:{path}"))
        .await
        .unwrap();
    let err = sqlx::query(
        "UPDATE normalization_profiles SET label = 'tampered'
         WHERE profile_id = 'L3.diacritics' AND version = '1.0.0'",
    )
    .execute(&pool)
    .await
    .unwrap_err();
    assert!(err.to_string().contains("QAI-NORM-0003"), "{err}");
    pool.close().await;
}
