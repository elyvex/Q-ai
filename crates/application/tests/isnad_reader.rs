//! Phase 7 (plan 07-04) — isnad reader DB CHECK + narrator round-trip tests.

use std::sync::Arc;

use application::isnad_reader::{IsnadReader, IsnadReaderService};
use isnad_graph::{ConfidenceBand, NarratorId, NarratorRecord};
use storage_sqlite::SqliteDatabase;
use tempfile::tempdir;

async fn migrated_db() -> (tempfile::TempDir, Arc<SqliteDatabase>) {
    let dir = tempdir().unwrap();
    let path = dir.path().join("qai.db");
    let path_str = path.to_str().unwrap().to_string();
    let repo_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    storage_sqlite::migrate::apply_migrations(&path_str, &repo_root).await.unwrap();
    let db = Arc::new(SqliteDatabase::new(&path_str, 4, true).await.unwrap());
    (dir, db)
}

#[tokio::test]
async fn narrator_round_trip_with_variants() {
    let (_dir, db) = migrated_db().await;
    sqlx::query(
        "INSERT INTO narrators (id, slug, primary_name, name_variants_json, metadata_json, created_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind("narrator:abu-hurayra")
    .bind("abu-hurayra")
    .bind("Abu Hurayra")
    .bind(r#"["Abu Huraira","Abu Hurairah"]"#)
    .bind("{}")
    .bind("2026-01-01T00:00:00Z")
    .execute(db.write_pool())
    .await
    .unwrap();

    let svc = IsnadReaderService::new(db.clone());
    let rec = svc.get_narrator("abu-hurayra").await.unwrap();
    assert_eq!(rec.id.to_string(), "narrator:abu-hurayra");
    assert_eq!(rec.primary_name, "Abu Hurayra");
    assert_eq!(rec.name_variants, vec!["Abu Huraira".to_string(), "Abu Hurairah".to_string()]);
    // stable id form round-trips
    let parsed: NarratorId = "narrator:abu-hurayra".parse().unwrap();
    assert_eq!(parsed, rec.id);
}

#[tokio::test]
async fn rijal_sources_per_scholar_not_merged() {
    let (_dir, db) = migrated_db().await;
    sqlx::query(
        "INSERT INTO narrators (id, slug, primary_name, name_variants_json, metadata_json, created_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind("narrator:x")
    .bind("x")
    .bind("X")
    .bind("[]")
    .bind("{}")
    .bind("2026-01-01T00:00:00Z")
    .execute(db.write_pool())
    .await
    .unwrap();

    for (scholar, cite) in [("Ibn Hajar", "Tahdhib al-Tahdhib"), ("al-Dhahabi", "Mizan al-I'tidal")] {
        sqlx::query(
            "INSERT INTO narrator_rijal_sources (id, narrator_id, scholar_name, source_citation, grading_value, evidence_json, reviewer, decision, decided_at, supersedes_id, provenance_layer, confidence, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(format!("rs-{}", scholar))
        .bind("narrator:x")
        .bind(scholar)
        .bind(cite)
        .bind(Some("thiqa"))
        .bind("{}")
        .bind(Option::<String>::None)
        .bind("pending")
        .bind(Option::<String>::None)
        .bind(Option::<String>::None)
        .bind("B")
        .bind(Option::<f64>::None)
        .bind("2026-01-01T00:00:00Z")
        .execute(db.write_pool())
        .await
        .unwrap();
    }

    let svc = IsnadReaderService::new(db.clone());
    let sources = svc.get_rijal_sources("narrator:x").await.unwrap();
    assert_eq!(sources.len(), 2);
    let scholars: Vec<&str> = sources.iter().map(|s| s.scholar_name.as_str()).collect();
    assert!(scholars.contains(&"Ibn Hajar"));
    assert!(scholars.contains(&"al-Dhahabi"));
    for s in &sources {
        assert!(!s.source_citation.is_empty());
    }
    // no consensus field in serialized form
    let json = serde_json::to_string(&sources).unwrap();
    assert!(!json.contains("consensus"));
}

#[tokio::test]
async fn empty_rijal_returns_zero() {
    let (_dir, db) = migrated_db().await;
    sqlx::query(
        "INSERT INTO narrators (id, slug, primary_name, name_variants_json, metadata_json, created_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind("narrator:empty")
    .bind("empty")
    .bind("Empty")
    .bind("[]")
    .bind("{}")
    .bind("2026-01-01T00:00:00Z")
    .execute(db.write_pool())
    .await
    .unwrap();

    let svc = IsnadReaderService::new(db.clone());
    let sources = svc.get_rijal_sources("narrator:empty").await.unwrap();
    assert_eq!(sources.len(), 0);
}

#[tokio::test]
async fn db_check_rejects_rijal_accepted_without_reviewer() {
    let (_dir, db) = migrated_db().await;
    sqlx::query(
        "INSERT INTO narrators (id, slug, primary_name, name_variants_json, metadata_json, created_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind("narrator:x")
    .bind("x")
    .bind("X")
    .bind("[]")
    .bind("{}")
    .bind("2026-01-01T00:00:00Z")
    .execute(db.write_pool())
    .await
    .unwrap();

    let res = sqlx::query(
        "INSERT INTO narrator_rijal_sources (id, narrator_id, scholar_name, source_citation, grading_value, evidence_json, reviewer, decision, decided_at, supersedes_id, provenance_layer, confidence, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("rs-bad")
    .bind("narrator:x")
    .bind("Scholar")
    .bind("Cite")
    .bind(Option::<String>::None)
    .bind("{}")
    .bind(Option::<String>::None) // reviewer NULL
    .bind("accepted")
    .bind(Some("2026-01-01T00:00:00Z"))
    .bind(Option::<String>::None)
    .bind("B")
    .bind(Option::<f64>::None)
    .bind("2026-01-01T00:00:00Z")
    .execute(db.write_pool())
    .await;
    assert!(res.is_err(), "CHECK should reject accepted without reviewer");
}

#[tokio::test]
async fn db_check_rejects_isnad_edge_confirmed_without_reviewer() {
    let (_dir, db) = migrated_db().await;
    let res = sqlx::query(
        "INSERT INTO isnad_edges (id, src_stable_id, edge, dst_stable_id, assertion_id, provenance_layer, confidence, label, algorithm, algorithm_version, reviewer, decided_at, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("edge-bad")
    .bind("narrator:a")
    .bind("TRANSMITS")
    .bind("narrator:b")
    .bind(Option::<String>::None)
    .bind("B")
    .bind(Option::<f64>::None)
    .bind("confirmed")
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind(Option::<String>::None) // reviewer NULL
    .bind(Option::<String>::None)
    .bind("2026-01-01T00:00:00Z")
    .execute(db.write_pool())
    .await;
    assert!(res.is_err(), "CHECK should reject confirmed without reviewer");
}

#[tokio::test]
async fn db_check_rejects_identity_accepted_without_decided_at() {
    let (_dir, db) = migrated_db().await;
    let res = sqlx::query(
        "INSERT INTO narrator_identity_assertions (id, assertion_kind, claim_json, evidence_json, source_location, reviewer, decision, decided_at, supersedes_id, provenance_layer, algorithm, algorithm_version, confidence, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("nia-bad")
    .bind("narrator_identity")
    .bind("{}")
    .bind("{}")
    .bind("")
    .bind(Some("reviewer"))
    .bind("accepted")
    .bind(Option::<String>::None) // decided_at NULL
    .bind(Option::<String>::None)
    .bind("B")
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind(Option::<f64>::None)
    .bind("2026-01-01T00:00:00Z")
    .execute(db.write_pool())
    .await;
    assert!(res.is_err(), "CHECK should reject accepted without decided_at");
}

#[tokio::test]
async fn d_layer_gate_requires_fields() {
    let (_dir, db) = migrated_db().await;
    let svc = IsnadReaderService::new(db.clone());

    // missing algorithm/version/confidence
    let prop = application::isnad_reader::IsnadIdentityProposal {
        claim: serde_json::json!({}),
        evidence: serde_json::json!({}),
        source_location: "loc".into(),
        provenance_layer: "D".into(),
        algorithm: None,
        algorithm_version: None,
        confidence: None,
    };
    assert!(svc.propose_identity(prop).await.is_err());

    // algorithm present but confidence null
    let prop = application::isnad_reader::IsnadIdentityProposal {
        claim: serde_json::json!({}),
        evidence: serde_json::json!({}),
        source_location: "loc".into(),
        provenance_layer: "D".into(),
        algorithm: Some("algo".into()),
        algorithm_version: Some("1.0".into()),
        confidence: None,
    };
    assert!(svc.propose_identity(prop).await.is_err());

    // all present -> ok
    let prop = application::isnad_reader::IsnadIdentityProposal {
        claim: serde_json::json!({}),
        evidence: serde_json::json!({}),
        source_location: "loc".into(),
        provenance_layer: "D".into(),
        algorithm: Some("algo".into()),
        algorithm_version: Some("1.0".into()),
        confidence: Some(0.9),
    };
    assert!(svc.propose_identity(prop).await.is_ok());

    // confidence band thresholds
    assert_eq!(ConfidenceBand::of(0.95), ConfidenceBand::High);
    assert_eq!(ConfidenceBand::of(0.9), ConfidenceBand::Medium);
    assert_eq!(ConfidenceBand::of(0.7), ConfidenceBand::Medium);
    assert_eq!(ConfidenceBand::of(0.69), ConfidenceBand::Low);
}
