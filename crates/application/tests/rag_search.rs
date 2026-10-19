use application::rag_reader::RagReader;
use application::{rag_reader::SqliteRagReader, rag_search::RagSearchService, rag_tools::RagToolBackend};
use embeddings::EmbeddingProvider;
use std::collections::BTreeMap;
use std::sync::Arc;
use storage_sqlite::SqliteDatabase;
use tool_registry::RagBackend;

async fn setup() -> (Arc<SqliteDatabase>, Arc<SqliteRagReader>, Arc<RagSearchService>) {
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
    // keep dir alive by leaking? tempdir drops -> file deleted. Use Box::leak to keep.
    std::mem::forget(dir);
    (db, reader, search)
}

#[tokio::test]
async fn rag_search_reproducible() {
    let (_db, reader, search) = setup().await;
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
    reader.create_project("test", config_toml).await.unwrap();

    // import 3 docs via reader? Use insert_document requires source. Use backend_import easier.
    let backend = RagToolBackend::new(reader.clone(), search.clone());
    let docs = serde_json::json!([
        {"source_id":"s1","path":"a.txt","title":"A","text":"hello world","metadata":{"tag":"x"}},
        {"source_id":"s1","path":"b.txt","title":"B","text":"hello rust","metadata":{"tag":"y"}},
        {"source_id":"s2","path":"c.txt","title":"C","text":"goodbye world","metadata":{}}
    ]);
    backend.backend_import("test", docs.as_array().unwrap()).await.unwrap();

    let (h1, r1) = search.query("test", "hello", BTreeMap::new(), 10usize).await.unwrap();
    let (h2, r2) = search.query("test", "hello", BTreeMap::new(), 10usize).await.unwrap();
    let ids1: Vec<String> = h1.iter().map(|h| h.doc_id.clone()).collect();
    let ids2: Vec<String> = h2.iter().map(|h| h.doc_id.clone()).collect();
    assert_eq!(ids1, ids2);
    let scores1: Vec<f32> = h1.iter().map(|h| h.score).collect();
    let scores2: Vec<f32> = h2.iter().map(|h| h.score).collect();
    assert_eq!(scores1, scores2);
    assert_eq!(r1.checksum, r2.checksum);
    assert!(!h1.is_empty());
}

#[tokio::test]
async fn rag_search_metadata_filter() {
    let (_db, reader, search) = setup().await;
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
    reader.create_project("test", config_toml).await.unwrap();
    let backend = RagToolBackend::new(reader.clone(), search.clone());
    let docs = serde_json::json!([
        {"source_id":"s1","path":"a.txt","title":"A","text":"hello world","metadata":{"tag":"x"}},
        {"source_id":"s1","path":"b.txt","title":"B","text":"hello rust","metadata":{"tag":"y"}}
    ]);
    backend.backend_import("test", docs.as_array().unwrap()).await.unwrap();

    let mut f = BTreeMap::new();
    f.insert("tag".to_string(), "x".to_string());
    let (hits, _) = search.query("test", "hello", f, 10usize).await.unwrap();
    for h in &hits {
        // ensure filtered doc with tag y excluded? We can't check metadata here, but count should be 1
    }
    assert_eq!(hits.len(), 1);
}

#[tokio::test]
async fn rag_search_nonactive_excluded() {
    let (_db, reader, search) = setup().await;
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
    reader.create_project("test", config_toml).await.unwrap();
    let backend = RagToolBackend::new(reader.clone(), search.clone());
    let docs = serde_json::json!([
        {"source_id":"s1","path":"a.txt","title":"A","text":"hello world","metadata":{}}
    ]);
    backend.backend_import("test", docs.as_array().unwrap()).await.unwrap();
    // deactivate source
    backend.backend_source_status("test", "s1", "deactivated").await.unwrap();
    let (hits, _) = search.query("test", "hello", BTreeMap::new(), 10usize).await.unwrap();
    assert!(hits.is_empty(), "expected no hits, got {:?}", hits);
}