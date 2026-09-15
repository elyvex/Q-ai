//! Phase 1 — word-gloss import acceptance (P1-T38, D1.4).
//!
//! Exercises `application::quran::import_glosses` end to end against the
//! synthetic edition: the dataset must be catalogued, every gloss must land
//! on a real token of the aligned edition, and rejections leave nothing
//! behind.

#[path = "common/mod.rs"]
mod common;

use application::quran::import_glosses;
use common::{active_reader, principal, timestamp};
use storage::Database as _;

fn manifest(dataset: &str, aligned: &str, entries: &[(u16, u32, u32, &str, &str)]) -> String {
    let entries = entries
        .iter()
        .map(|(surah, ayah, position, language, gloss)| {
            format!(
                "{{\"surah\":{surah},\"ayah\":{ayah},\"position\":{position},\
                 \"language\":{},\"gloss\":{}}}",
                serde_json::to_string(language).unwrap(),
                serde_json::to_string(gloss).unwrap()
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"dataset\":{{\"id\":{},\"aligned_edition\":{}}},\"glosses\":[{entries}]}}",
        serde_json::to_string(dataset).unwrap(),
        serde_json::to_string(aligned).unwrap()
    )
}

/// `src-1` is seeded by the harness, so it doubles as a gloss dataset here.
#[tokio::test]
async fn import_glosses_persists_token_aligned_rows() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let text = manifest(
        "src-1",
        "test-edition-min@0.1.0",
        &[(1, 1, 1, "en", "one one"), (1, 1, 2, "en", "one two"), (2, 1, 1, "en", "two one")],
    );
    let count =
        import_glosses(&*db, &text, &principal(), &timestamp()).await.expect("valid import");
    assert_eq!(count, 3);

    let mut uow = db.write().await.unwrap();
    let aligned = uow
        .quran()
        .get_edition_by_slug_version("test-edition-min", "0.1.0")
        .await
        .unwrap()
        .unwrap();
    let rows = uow.quran().list_word_glosses(&aligned.id, 1, 1).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].gloss_dataset_id, "src-1");
    assert_eq!(rows[0].position, 1);
    assert_eq!(rows[0].language, "en");
    assert_eq!(rows[0].gloss, "one one");
    assert!(!rows[0].provenance_id.is_empty());
    let rows = uow.quran().list_word_glosses(&aligned.id, 2, 1).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].gloss, "two one");
}

#[tokio::test]
async fn import_glosses_rejects_malformed_manifest() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let err = import_glosses(&*db, "{not json", &principal(), &timestamp()).await.unwrap_err();
    assert!(err.to_string().contains("manifest is not valid"), "{err}");
}

#[tokio::test]
async fn import_glosses_requires_a_dataset_id() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let text = manifest("  ", "test-edition-min@0.1.0", &[(1, 1, 1, "en", "x")]);
    let err = import_glosses(&*db, &text, &principal(), &timestamp()).await.unwrap_err();
    assert!(err.to_string().contains("non-empty id"), "{err}");
}

#[tokio::test]
async fn import_glosses_requires_a_catalogued_dataset() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let text = manifest("missing-src", "test-edition-min@0.1.0", &[(1, 1, 1, "en", "x")]);
    let err = import_glosses(&*db, &text, &principal(), &timestamp()).await.unwrap_err();
    assert!(err.to_string().contains("unknown gloss dataset"), "{err}");
}

#[tokio::test]
async fn import_glosses_requires_slug_at_version_alignment() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let text = manifest("src-1", "test-edition-min", &[(1, 1, 1, "en", "x")]);
    let err = import_glosses(&*db, &text, &principal(), &timestamp()).await.unwrap_err();
    assert!(err.to_string().contains("slug@version"), "{err}");
}

#[tokio::test]
async fn import_glosses_rejects_unknown_aligned_edition() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let text = manifest("src-1", "nope@9.9.9", &[(1, 1, 1, "en", "x")]);
    let err = import_glosses(&*db, &text, &principal(), &timestamp()).await.unwrap_err();
    assert!(matches!(err, application::quran::ActivationError::NotStaged { .. }), "{err}");
}

#[tokio::test]
async fn import_glosses_rejects_empty_gloss_and_bad_language() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let text = manifest("src-1", "test-edition-min@0.1.0", &[(1, 1, 1, "en", "   ")]);
    let err = import_glosses(&*db, &text, &principal(), &timestamp()).await.unwrap_err();
    assert!(err.to_string().contains("empty gloss"), "{err}");

    let text = manifest("src-1", "test-edition-min@0.1.0", &[(1, 1, 1, "en us", "x")]);
    let err = import_glosses(&*db, &text, &principal(), &timestamp()).await.unwrap_err();
    assert!(err.to_string().contains("bad language"), "{err}");
}

#[tokio::test]
async fn import_glosses_rejects_positions_outside_the_token_range() {
    let (_dir, db, _reader, _path) = active_reader().await;
    // Ayah 1:1 has five tokens; position 0 and 6 are both invalid.
    for position in [0, 6] {
        let text = manifest("src-1", "test-edition-min@0.1.0", &[(1, 1, position, "en", "x")]);
        let err = import_glosses(&*db, &text, &principal(), &timestamp()).await.unwrap_err();
        assert!(err.to_string().contains("position"), "{err}");
    }
}

#[tokio::test]
async fn import_glosses_rejects_ayah_absent_from_aligned_edition() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let text = manifest("src-1", "test-edition-min@0.1.0", &[(1, 9, 1, "en", "x")]);
    let err = import_glosses(&*db, &text, &principal(), &timestamp()).await.unwrap_err();
    assert!(err.to_string().contains("has no ayah 1:9"), "{err}");
}

#[tokio::test]
async fn import_glosses_rejects_duplicate_entry() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let text = manifest(
        "src-1",
        "test-edition-min@0.1.0",
        &[(1, 1, 1, "en", "first"), (1, 1, 1, "en", "second")],
    );
    let err = import_glosses(&*db, &text, &principal(), &timestamp()).await.unwrap_err();
    assert!(err.to_string().contains("duplicate gloss"), "{err}");
}

#[tokio::test]
async fn import_glosses_is_atomic_on_rejection() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let text = manifest(
        "src-1",
        "test-edition-min@0.1.0",
        &[(1, 1, 1, "en", "ok"), (1, 1, 9, "en", "bad")],
    );
    let err = import_glosses(&*db, &text, &principal(), &timestamp()).await.unwrap_err();
    assert!(err.to_string().contains("position"), "{err}");

    let mut uow = db.write().await.unwrap();
    let aligned = uow
        .quran()
        .get_edition_by_slug_version("test-edition-min", "0.1.0")
        .await
        .unwrap()
        .unwrap();
    let rows = uow.quran().list_word_glosses(&aligned.id, 1, 1).await.unwrap();
    assert!(rows.is_empty(), "rejected import must not leave partial glosses");
}
