//! `generation_monotonicity` — `corpus_generations.number` never regresses
//! under 50 concurrent writers targeting the same scope (AC-P0-24).

mod common;

use std::sync::Arc;

use storage::Database as _;

const SCOPE: &str = "quran:concurrent";

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn numbers_are_unique_and_contiguous_under_concurrency() {
    let fx = common::fixture().await;
    let path = fx.path.clone();
    let db = Arc::new(fx.db);

    let mut handles = Vec::new();
    for _ in 0..50 {
        let db = db.clone();
        handles.push(tokio::spawn(async move {
            let mut uow = db.write().await.unwrap();
            uow.outbox().allocate_generation(SCOPE, "concurrent").await.unwrap();
            uow.commit().await.unwrap();
        }));
    }
    for handle in handles {
        handle.await.unwrap();
    }

    let numbers = common::generation_numbers(&path, SCOPE).await;
    assert_eq!(numbers.len(), 50, "every writer must allocate exactly once");

    // Strictly increasing and contiguous 1..=50 — no regression, no gaps.
    let expected: Vec<i64> = (1..=50).collect();
    assert_eq!(numbers, expected, "numbers must be monotonic and contiguous");

    // Re-reading the max never regresses.
    let mut uow = db.write().await.unwrap();
    let current = uow.outbox().current_generation(SCOPE).await.unwrap().unwrap();
    assert_eq!(current.number, 50);
    uow.rollback().await.unwrap();
}
