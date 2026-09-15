//! M2 entry gate (DEV-05): the FTS5-first `FullTextIndex` backend requires
//! FTS5 compiled into the bundled SQLite (`libsqlite3-sys`). This test fails
//! loudly — instead of a mysterious backend failure later — when that
//! assumption breaks, and smoke-tests virtual-table indexing plus `MATCH`
//! retrieval over Arabic content.

/// FTS5 must be a compile option of the linked SQLite.
#[tokio::test]
async fn fts5_is_compiled_in() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fts5-probe.db");
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&path)
                .create_if_missing(true),
        )
        .await
        .unwrap();
    let used: i64 =
        sqlx::query_scalar("SELECT sqlite_compileoption_used('ENABLE_FTS5')")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(used, 1, "SQLite must be built with FTS5 (M2 FTS5-first backend)");
    pool.close().await;
}

/// Virtual tables index and `MATCH` retrieves Arabic content.
#[tokio::test]
async fn fts5_indexes_and_matches_arabic() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fts5-smoke.db");
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&path)
                .create_if_missing(true),
        )
        .await
        .unwrap();
    sqlx::query("CREATE VIRTUAL TABLE probe_fts USING fts5(text, tokenize='unicode61')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO probe_fts (text) VALUES (?)")
        .bind("بِسْمِ اللَّهِ")
        .execute(&pool)
        .await
        .unwrap();
    let hits: Vec<String> =
        sqlx::query_scalar("SELECT text FROM probe_fts WHERE probe_fts MATCH ?")
            .bind("اللَّهِ")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(hits, vec!["بِسْمِ اللَّهِ".to_string()]);
    // Highlight introspection (needed later for highlight ranges) works.
    // Note: `offsets()` is context-restricted in current SQLite; `highlight()`
    // is the supported path and is what M3 highlighting will use.
    let marked: String =
        sqlx::query_scalar("SELECT highlight(probe_fts, 0, '[', ']') FROM probe_fts WHERE probe_fts MATCH ?")
            .bind("اللَّهِ")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(marked.contains('[') && marked.contains(']'), "highlight() must mark the match");
    pool.close().await;
}
