//! Phase 1 — translation import acceptance (P1-T36, D1.4).
//!
//! Exercises `application::quran::import_translations` end to end against the
//! synthetic edition: attributed (principle 5) storage, structural alignment
//! (the aligned edition and every named ayah must exist), and the rejections
//! that keep an unaligned or unattributed translation out of the corpus.

#[path = "common/mod.rs"]
mod common;

use application::quran::import_translations;
use common::{active_reader, principal, timestamp};
use storage::Database as _;

/// True when `value` is the tagged SHA-256 storage form `sha256:<64 lowercase hex>`.
fn is_tagged_sha256(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn manifest(aligned: &str, translator: &str, passages: &[(u16, u32, &str)]) -> String {
    let passages = passages
        .iter()
        .map(|(surah, ayah, text)| {
            format!(
                "{{\"surah\":{surah},\"ayah\":{ayah},\"text\":{}}}",
                serde_json::to_string(text).unwrap()
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"translation\":{{\"slug\":\"en-test\",\"version\":\"1.0.0\",\"name\":\"Test English\",\
         \"translator\":\"{translator}\",\"language\":\"en\",\"aligned_edition\":\"{aligned}\",\
         \"numbering_scheme\":\"hafs\",\"spdx_id\":null}},\"passages\":[{passages}]}}"
    )
}

/// Valid imports persist an attributed edition and its passages; read-back goes
/// through the public repository API.
#[tokio::test]
async fn import_translations_persists_attributed_edition_and_passages() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let text = manifest(
        "test-edition-min@0.1.0",
        "Test Translator",
        &[(1, 1, "one one"), (1, 2, "one two"), (2, 1, "two one")],
    );
    let id = import_translations(
        &*db,
        &text,
        "12345678-1234-1234-1234-123456789abc",
        &principal(),
        &timestamp(),
    )
    .await
    .expect("valid translation imports");
    assert_eq!(id, "tr-en-test-1.0.0");

    let mut uow = db.write().await.unwrap();
    let editions = uow.quran().list_translation_editions().await.unwrap();
    assert_eq!(editions.len(), 1);
    let edition = &editions[0];
    assert_eq!(edition.slug, "en-test");
    assert_eq!(edition.version, "1.0.0");
    assert_eq!(edition.translator, "Test Translator");
    assert_eq!(edition.language, "en");
    assert_eq!(edition.status, "Staged");
    // A real content hash: the additive `qai-translation-hash-v1` recipe in the
    // tagged storage form. It is never empty and never a canonical `text_hash`.
    assert!(
        is_tagged_sha256(&edition.text_hash),
        "translation text_hash must be `sha256:<64 hex>`, got `{}`",
        edition.text_hash
    );
    // The aligned edition is the canonical `test-edition-min@0.1.0` row.
    let aligned = uow
        .quran()
        .get_edition_by_slug_version("test-edition-min", "0.1.0")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(edition.aligned_edition_id, aligned.id);

    let passage = uow.quran().get_translation_passage(&id, 1, 1).await.unwrap().unwrap();
    assert_eq!(passage.text, "one one");
    assert!(!passage.provenance_id.is_empty());
    let missing = uow.quran().get_translation_passage(&id, 3, 1).await.unwrap();
    assert!(missing.is_none(), "passages are stored only where declared");
}

/// The content hash is computed over the passages sorted by `(surah, ayah)`, so
/// the manifest's passage order cannot change the digest (QC-08, T-02-25).
#[tokio::test]
async fn translation_hash_is_independent_of_manifest_passage_order() {
    let ascending = manifest(
        "test-edition-min@0.1.0",
        "Test Translator",
        &[(1, 1, "one one"), (1, 2, "one two"), (2, 1, "two one")],
    );
    let shuffled = manifest(
        "test-edition-min@0.1.0",
        "Test Translator",
        &[(2, 1, "two one"), (1, 2, "one two"), (1, 1, "one one")],
    );

    let (_dir_a, db_a, _reader_a, _path_a) = active_reader().await;
    import_translations(
        &*db_a,
        &ascending,
        "12345678-1234-1234-1234-123456789abc",
        &principal(),
        &timestamp(),
    )
    .await
    .expect("ascending manifest imports");
    let mut uow = db_a.write().await.unwrap();
    let hash_ascending = uow.quran().list_translation_editions().await.unwrap()[0].text_hash.clone();
    drop(uow);

    let (_dir_b, db_b, _reader_b, _path_b) = active_reader().await;
    import_translations(
        &*db_b,
        &shuffled,
        "12345678-1234-1234-1234-123456789abc",
        &principal(),
        &timestamp(),
    )
    .await
    .expect("shuffled manifest imports");
    let mut uow = db_b.write().await.unwrap();
    let hash_shuffled = uow.quran().list_translation_editions().await.unwrap()[0].text_hash.clone();

    assert!(is_tagged_sha256(&hash_ascending), "got `{hash_ascending}`");
    assert_eq!(
        hash_ascending, hash_shuffled,
        "the hash must be independent of the manifest's passage order"
    );
}

#[tokio::test]
async fn import_translations_rejects_malformed_manifest() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let err = import_translations(
        &*db,
        "{not json",
        "12345678-1234-1234-1234-123456789abc",
        &principal(),
        &timestamp(),
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("manifest is not valid"), "{err}");
}

#[tokio::test]
async fn import_translations_requires_attribution() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let text = manifest("test-edition-min@0.1.0", "   ", &[(1, 1, "text")]);
    let err = import_translations(
        &*db,
        &text,
        "12345678-1234-1234-1234-123456789abc",
        &principal(),
        &timestamp(),
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("non-empty translator"), "{err}");
}

#[tokio::test]
async fn import_translations_requires_slug_at_version_alignment() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let text = manifest("test-edition-min", "Test Translator", &[(1, 1, "text")]);
    let err = import_translations(
        &*db,
        &text,
        "12345678-1234-1234-1234-123456789abc",
        &principal(),
        &timestamp(),
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("slug@version"), "{err}");
}

#[tokio::test]
async fn import_translations_rejects_unknown_aligned_edition() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let text = manifest("nope@9.9.9", "Test Translator", &[(1, 1, "text")]);
    let err = import_translations(
        &*db,
        &text,
        "12345678-1234-1234-1234-123456789abc",
        &principal(),
        &timestamp(),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, application::quran::ActivationError::NotStaged { .. }), "{err}");
}

#[tokio::test]
async fn import_translations_rejects_empty_passage_text() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let text = manifest("test-edition-min@0.1.0", "Test Translator", &[(1, 1, "   ")]);
    let err = import_translations(
        &*db,
        &text,
        "12345678-1234-1234-1234-123456789abc",
        &principal(),
        &timestamp(),
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("empty translation"), "{err}");
}

#[tokio::test]
async fn import_translations_rejects_ayah_absent_from_aligned_edition() {
    let (_dir, db, _reader, _path) = active_reader().await;
    // Surah 1 has three ayahs in the synthetic edition; 1:9 does not exist.
    let text = manifest("test-edition-min@0.1.0", "Test Translator", &[(1, 9, "out of range")]);
    let err = import_translations(
        &*db,
        &text,
        "12345678-1234-1234-1234-123456789abc",
        &principal(),
        &timestamp(),
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("has no ayah 1:9"), "{err}");
}

#[tokio::test]
async fn import_translations_rejects_duplicate_passage() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let text =
        manifest("test-edition-min@0.1.0", "Test Translator", &[(1, 1, "first"), (1, 1, "second")]);
    let err = import_translations(
        &*db,
        &text,
        "12345678-1234-1234-1234-123456789abc",
        &principal(),
        &timestamp(),
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("duplicate translation passage"), "{err}");
}

#[tokio::test]
async fn import_translations_is_atomic_on_rejection() {
    let (_dir, db, _reader, _path) = active_reader().await;
    // A valid first passage followed by an unknown ayah: nothing may persist.
    let text =
        manifest("test-edition-min@0.1.0", "Test Translator", &[(1, 1, "ok"), (2, 9, "bad")]);
    let err = import_translations(
        &*db,
        &text,
        "12345678-1234-1234-1234-123456789abc",
        &principal(),
        &timestamp(),
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("has no ayah 2:9"), "{err}");

    let mut uow = db.write().await.unwrap();
    let editions = uow.quran().list_translation_editions().await.unwrap();
    assert!(editions.is_empty(), "rejected import must not leave a partial edition");
}
