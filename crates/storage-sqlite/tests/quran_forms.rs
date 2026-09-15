//! Phase 2 — derived-forms tables (migration `0014`): repo acceptance.
//!
//! Against real SQLite in a tempdir: empty reads work, orphan writes fail
//! closed on the canonical FK, and edition-scoped deletes are total. Full
//! build round-trips (with canonical parents) belong to the `forms.rebuild`
//! suite (P2-T26), which imports a real edition first.

use storage::Database as _;
use storage::error::StorageError;
use storage::quran::{AyahFormRow, SkeletonRow, TokenFormRow};

mod common;

fn token_form() -> TokenFormRow {
    TokenFormRow {
        edition_id: "ed-missing".to_string(),
        surah: 1,
        ayah: 1,
        position: 1,
        simple: "ا".to_string(),
        bare: "ا".to_string(),
        hamza_folded: "ا".to_string(),
        folded: "ا".to_string(),
        affix_stripped: "ا".to_string(),
        transliteration: None,
        phonetic: None,
        rule_set_id: "quran-normalization".to_string(),
        rule_set_version: "1.0.0".to_string(),
        corpus_generation: 1,
        provenance_id: "prov-missing".to_string(),
    }
}

fn ayah_form() -> AyahFormRow {
    AyahFormRow {
        edition_id: "ed-missing".to_string(),
        surah: 1,
        ayah: 1,
        simple: "ا".to_string(),
        bare: "ا".to_string(),
        hamza_folded: "ا".to_string(),
        folded: "ا".to_string(),
        transliteration: None,
        rule_set_id: "quran-normalization".to_string(),
        rule_set_version: "1.0.0".to_string(),
        corpus_generation: 1,
        provenance_id: "prov-missing".to_string(),
    }
}

fn skeleton() -> SkeletonRow {
    SkeletonRow {
        edition_id: "ed-missing".to_string(),
        surah: 1,
        ayah_start: 1,
        ayah_end: 3,
        skeleton: "ا".to_string(),
        rule_set_id: "quran-normalization".to_string(),
        rule_set_version: "1.0.0".to_string(),
        corpus_generation: 1,
        provenance_id: "prov-missing".to_string(),
    }
}

/// Empty reads work on a freshly migrated database.
#[tokio::test]
async fn empty_forms_read_clean() {
    let fx = common::fixture().await;
    let mut uow = fx.db.write().await.unwrap();
    assert!(uow.quran().list_token_forms("ed", 1, 1).await.unwrap().is_empty());
    assert!(uow.quran().get_ayah_form("ed", 1, 1).await.unwrap().is_none());
    assert!(uow.quran().list_skeletons("ed", 1).await.unwrap().is_empty());
    assert_eq!(uow.quran().count_token_forms("ed").await.unwrap(), 0);
    uow.quran().delete_forms_for_edition("ed").await.unwrap();
}

/// Orphan writes fail closed: derived rows cannot exist without canonical
/// parents (I8 — the FK, not application discipline, enforces this).
#[tokio::test]
async fn orphan_forms_are_rejected() {
    let fx = common::fixture().await;
    let mut uow = fx.db.write().await.unwrap();
    let err = uow.quran().insert_token_forms(vec![token_form()]).await.unwrap_err();
    assert!(matches!(err, StorageError::ConstraintViolation { .. }), "{err:?}");
    let err = uow.quran().insert_ayah_forms(vec![ayah_form()]).await.unwrap_err();
    assert!(matches!(err, StorageError::ConstraintViolation { .. }), "{err:?}");
    let err = uow.quran().insert_skeletons(vec![skeleton()]).await.unwrap_err();
    assert!(matches!(err, StorageError::ConstraintViolation { .. }), "{err:?}");
}
