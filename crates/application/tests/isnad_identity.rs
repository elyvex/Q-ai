//! Phase 7 (plan 07-04) — identity workflow + supersession + disputed tests.

use std::sync::Arc;

use application::isnad_graph_store::SqliteIsnadGraphStore;
use application::isnad_reader::{IsnadIdentityProposal, IsnadReader, IsnadReaderService};
use isnad_graph::ConfidenceBand;
use quran_graph::{AssertionDecision, GraphStore, QueryBudgets};
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

fn proposal() -> IsnadIdentityProposal {
    IsnadIdentityProposal {
        claim: serde_json::json!({"name": "X"}),
        evidence: serde_json::json!({}),
        source_location: "loc".into(),
        provenance_layer: "B".into(),
        algorithm: None,
        algorithm_version: None,
        confidence: None,
    }
}

#[tokio::test]
async fn propose_pending_reviewer_null() {
    let (_dir, db) = migrated_db().await;
    let svc = IsnadReaderService::new(db.clone());
    let a = svc.propose_identity(proposal()).await.unwrap();
    assert_eq!(a.decision, AssertionDecision::Pending);
    assert!(a.reviewer.is_none());
    assert!(a.decided_at.is_none());
}

#[tokio::test]
async fn accept_sets_reviewer_and_decided_at() {
    let (_dir, db) = migrated_db().await;
    let svc = IsnadReaderService::new(db.clone());
    let a = svc.propose_identity(proposal()).await.unwrap();
    svc.accept_identity(&a.id, "rev-1").await.unwrap();

    let row: (String, Option<String>, Option<String>) =
        sqlx::query_as("SELECT decision, reviewer, decided_at FROM narrator_identity_assertions WHERE id = ?")
            .bind(&a.id)
            .fetch_one(db.read_pool())
            .await
            .unwrap();
    assert_eq!(row.0, "accepted");
    assert_eq!(row.1.as_deref(), Some("rev-1"));
    assert!(row.2.is_some());
}

#[tokio::test]
async fn reject_retained_not_deleted() {
    let (_dir, db) = migrated_db().await;
    let svc = IsnadReaderService::new(db.clone());
    let a = svc.propose_identity(proposal()).await.unwrap();
    svc.reject_identity(&a.id, "rev-1").await.unwrap();
    let exists: Option<String> =
        sqlx::query_scalar("SELECT id FROM narrator_identity_assertions WHERE id = ?")
            .bind(&a.id)
            .fetch_optional(db.read_pool())
            .await
            .unwrap();
    assert!(exists.is_some());
}

#[tokio::test]
async fn dispute_stays_effective() {
    let (_dir, db) = migrated_db().await;
    let svc = IsnadReaderService::new(db.clone());
    let a = svc.propose_identity(proposal()).await.unwrap();
    svc.dispute_identity(&a.id, "rev-1").await.unwrap();

    let store = SqliteIsnadGraphStore::open(&db).await.unwrap();
    let assertion = store.get_assertion(&a.id).unwrap();
    assert!(assertion.is_effective());
    assert!(!assertion.is_tombstoned());
}

#[tokio::test]
async fn supersede_creates_new_referencing_old_old_retained() {
    let (_dir, db) = migrated_db().await;
    let svc = IsnadReaderService::new(db.clone());
    let old = svc.propose_identity(proposal()).await.unwrap();
    let new = svc.supersede_identity(&old.id, proposal(), "rev-1").await.unwrap();
    assert_eq!(new.supersedes_id, Some(old.id.clone()));

    // old retained + superseded
    let row: (String,) =
        sqlx::query_as("SELECT decision FROM narrator_identity_assertions WHERE id = ?")
            .bind(&old.id)
            .fetch_one(db.read_pool())
            .await
            .unwrap();
    assert_eq!(row.0, "superseded");

    // new retained
    let exists: Option<String> =
        sqlx::query_scalar("SELECT id FROM narrator_identity_assertions WHERE id = ?")
            .bind(&new.id)
            .fetch_optional(db.read_pool())
            .await
            .unwrap();
    assert!(exists.is_some());
}

#[tokio::test]
async fn supersession_chain_resolves_latest() {
    let (_dir, db) = migrated_db().await;
    let svc = IsnadReaderService::new(db.clone());
    let v1 = svc.propose_identity(proposal()).await.unwrap();
    let v2 = svc.supersede_identity(&v1.id, proposal(), "rev-1").await.unwrap();
    // v2 currently pending; supersede requires pending. Mark? v2 is pending so ok.
    let v3 = svc.supersede_identity(&v2.id, proposal(), "rev-1").await.unwrap();

    // v1, v2 superseded; v3 not
    for id in [&v1.id, &v2.id] {
        let row: (String,) =
            sqlx::query_as("SELECT decision FROM narrator_identity_assertions WHERE id = ?")
                .bind(id)
                .fetch_one(db.read_pool())
                .await
                .unwrap();
        assert_eq!(row.0, "superseded");
    }
    let row: (String,) =
        sqlx::query_as("SELECT decision FROM narrator_identity_assertions WHERE id = ?")
            .bind(&v3.id)
            .fetch_one(db.read_pool())
            .await
            .unwrap();
    assert_ne!(row.0, "superseded");
    assert_eq!(v3.supersedes_id.as_deref(), Some(v2.id.as_str()));
}

#[tokio::test]
async fn traversal_excludes_tombstoned_disputed_effective() {
    let (_dir, db) = migrated_db().await;
    // narrators
    for id in ["narrator:s", "narrator:t1", "narrator:t2", "narrator:t3"] {
        let slug = id.strip_prefix("narrator:").unwrap();
        sqlx::query(
            "INSERT INTO narrators (id, slug, primary_name, name_variants_json, metadata_json, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(slug)
        .bind(slug)
        .bind("[]")
        .bind("{}")
        .bind("2026-01-01T00:00:00Z")
        .execute(db.write_pool())
        .await
        .unwrap();
    }

    let svc = IsnadReaderService::new(db.clone());
    let rejected = svc.propose_identity(proposal()).await.unwrap();
    svc.reject_identity(&rejected.id, "rev").await.unwrap();
    let disputed = svc.propose_identity(proposal()).await.unwrap();
    svc.dispute_identity(&disputed.id, "rev").await.unwrap();
    let accepted = svc.propose_identity(proposal()).await.unwrap();
    svc.accept_identity(&accepted.id, "rev").await.unwrap();

    // edges
    for (aid, dst) in [(&rejected.id, "narrator:t1"), (&disputed.id, "narrator:t2"), (&accepted.id, "narrator:t3")] {
        sqlx::query(
            "INSERT INTO isnad_edges (id, src_stable_id, edge, dst_stable_id, assertion_id, provenance_layer, confidence, label, algorithm, algorithm_version, reviewer, decided_at, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(format!("edge-{}", dst))
        .bind("narrator:s")
        .bind("TRANSMITS")
        .bind(dst)
        .bind(aid)
        .bind("B")
        .bind(Option::<f64>::None)
        .bind("uncertain")
        .bind(Option::<String>::None)
        .bind(Option::<String>::None)
        .bind(Option::<String>::None)
        .bind(Option::<String>::None)
        .bind("2026-01-01T00:00:00Z")
        .execute(db.write_pool())
        .await
        .unwrap();
    }

    use quran_graph::{AuthzScope, EdgeFilter, Direction};
    let store = SqliteIsnadGraphStore::open(&db).await.unwrap();
    let budgets = QueryBudgets::default();
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let authz = AuthzScope::all_visible();
    let view = store.neighbors("narrator:s", &EdgeFilter::one("TRANSMITS", Direction::Outgoing), &budgets, &cancel, &authz).unwrap();
    // rejected edge excluded, disputed+accepted included
    assert_eq!(view.edges.len(), 2);
    assert!(view.edges.iter().all(|e| e.dst != "narrator:t1"));
}

#[tokio::test]
async fn budget_exhaustion_truncated() {
    let (_dir, db) = migrated_db().await;
    // chain of narrators
    let n = 20;
    for i in 0..n {
        let id = format!("narrator:n{}", i);
        let slug = format!("n{}", i);
        sqlx::query(
            "INSERT INTO narrators (id, slug, primary_name, name_variants_json, metadata_json, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&slug)
        .bind(&slug)
        .bind("[]")
        .bind("{}")
        .bind("2026-01-01T00:00:00Z")
        .execute(db.write_pool())
        .await
        .unwrap();
    }
    for i in 0..(n - 1) {
        sqlx::query(
            "INSERT INTO isnad_edges (id, src_stable_id, edge, dst_stable_id, assertion_id, provenance_layer, confidence, label, algorithm, algorithm_version, reviewer, decided_at, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(format!("edge-{}", i))
        .bind(format!("narrator:n{}", i))
        .bind("TRANSMITS")
        .bind(format!("narrator:n{}", i + 1))
        .bind(Option::<String>::None)
        .bind("B")
        .bind(Option::<f64>::None)
        .bind("uncertain")
        .bind(Option::<String>::None)
        .bind(Option::<String>::None)
        .bind(Option::<String>::None)
        .bind(Option::<String>::None)
        .bind("2026-01-01T00:00:00Z")
        .execute(db.write_pool())
        .await
        .unwrap();
    }

    let svc = IsnadReaderService::new(db.clone());
    let budgets = QueryBudgets { max_nodes: 3, ..QueryBudgets::default() };
    let res = svc.trace_paths("narrator:n0", "narrator:n19", 10, budgets).await.unwrap();
    assert!(res.truncated);
    assert!(res.incomplete_reason.is_some());
    assert!(!res.incomplete_reason.as_ref().unwrap().is_empty());
}

#[tokio::test]
async fn empty_path_complete() {
    let (_dir, db) = migrated_db().await;
    for id in ["narrator:a", "narrator:b"] {
        let slug = id.strip_prefix("narrator:").unwrap();
        sqlx::query(
            "INSERT INTO narrators (id, slug, primary_name, name_variants_json, metadata_json, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(slug)
        .bind(slug)
        .bind("[]")
        .bind("{}")
        .bind("2026-01-01T00:00:00Z")
        .execute(db.write_pool())
        .await
        .unwrap();
    }
    let svc = IsnadReaderService::new(db.clone());
    let res = svc.trace_paths("narrator:a", "narrator:b", 10, QueryBudgets::default()).await.unwrap();
    assert!(!res.truncated);
    assert!(res.paths.is_empty());
}

#[tokio::test]
async fn confidence_bands_thresholds() {
    assert_eq!(ConfidenceBand::of(0.95), ConfidenceBand::High);
    assert_eq!(ConfidenceBand::of(0.9), ConfidenceBand::Medium);
    assert_eq!(ConfidenceBand::of(0.7), ConfidenceBand::Medium);
    assert_eq!(ConfidenceBand::of(0.69), ConfidenceBand::Low);
}
