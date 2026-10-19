//! Phase 8 RAG cross-surface parity (service leg smoke).

use application::rag_tools::RagToolBackend;
use std::sync::Arc;
use storage_sqlite::SqliteDatabase;
use tool_registry::RagBackend;

#[tokio::test]
async fn rag_parity_service() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("qai.db");
    let path_str = path.to_str().unwrap().to_string();
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    storage_sqlite::migrate::apply_migrations(&path_str, &repo_root).await.unwrap();
    let db = Arc::new(SqliteDatabase::new(&path_str, 4, true).await.unwrap());
    let backend = RagToolBackend::from_db(db);

    let config_toml = r#"
[project]
name = "test"

[embedding]
provider = "local-hash"
dim = 32

[reranker]
name = "score"

[router]
name = "hybrid-rrf"
"#;
    backend.backend_create_project("test", config_toml).await.unwrap();
    let docs = serde_json::json!([
        {"source_id":"s1","path":"a.txt","title":"A","text":"hello world","metadata":{}},
        {"source_id":"s1","path":"b.txt","title":"B","text":"hello rust","metadata":{}},
        {"source_id":"s2","path":"c.txt","title":"C","text":"goodbye world","metadata":{}}
    ]);
    backend.backend_import("test", docs.as_array().unwrap()).await.unwrap();

    let r1 = backend.backend_query("test", "hello").await.unwrap();
    let r2 = backend.backend_query("test", "hello").await.unwrap();
    assert_eq!(r1.research_checksum, r2.research_checksum);
    let c1 = r1.results["route_decision"]["checksum"].as_str().unwrap();
    let c2 = r2.results["route_decision"]["checksum"].as_str().unwrap();
    assert_eq!(c1, c2);
    assert!(!c1.is_empty());
}
