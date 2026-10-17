use application::{rag_reader::SqliteRagReader, rag_search::RagSearchService, rag_tools::RagToolBackend};
use embeddings::EmbeddingProvider;
use std::sync::Arc;
use storage_sqlite::SqliteDatabase;
use tool_registry::RagBackend;

#[tokio::test]
async fn rag_tracer_e2e() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("qai.db");
    let path_str = path.to_str().unwrap().to_string();
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    storage_sqlite::migrate::apply_migrations(&path_str, &repo_root).await.unwrap();
    let db = Arc::new(SqliteDatabase::new(&path_str, 4, true).await.unwrap());
    let reader = Arc::new(SqliteRagReader::new(db.clone()));
    let embedder: Arc<dyn EmbeddingProvider> = Arc::new(embeddings::LocalHashEmbedding::new(32));
    let reranker: Arc<dyn reranking::Reranker> = Arc::new(reranking::ScoreReranker);
    let search = Arc::new(RagSearchService::new(db.clone(), embedder, reranker));
    let backend = RagToolBackend::new(reader.clone(), search);

    let config_toml = r#"
[project]
name = "test"

[embedding]
provider = "local-hash"
dim = 32

[reranker]
name = "score"

[router]
name = "default"
"#;
    let res = backend.backend_create_project("test", config_toml).await.unwrap();
    assert_eq!(res.results["name"], "test");

    let docs = serde_json::json!([{
        "source_id": "src-1",
        "path": "a.txt",
        "title": "Doc",
        "text": "hello world",
        "metadata": {}
    }]);
    let imp = backend.backend_import("test", docs.as_array().unwrap()).await.unwrap();
    assert_eq!(imp.results["imported"], 1);

    let q = backend.backend_query("test", "hello").await.unwrap();
    let hits = q.results["hits"].as_array().unwrap();
    assert!(!hits.is_empty(), "expected hits");
    let rd = &q.results["route_decision"];
    assert!(!rd["checksum"].as_str().unwrap().is_empty());
}
