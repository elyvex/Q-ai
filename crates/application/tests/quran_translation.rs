//! Phase 1 — translation import acceptance (P1-T36, D1.4).
//!
//! Exercises `application::quran::import_translations` end to end against the
//! synthetic edition: attributed (principle 5) storage, structural alignment
//! (the aligned edition and every named ayah must exist), and the rejections
//! that keep an unaligned or unattributed translation out of the corpus.

#[path = "common/mod.rs"]
mod common;

use application::quran::import_translations;
use application::quran_reader::QuranReader as _;
use common::{active_reader, principal, timestamp};
use quran_core::AyahOptions;
use storage::Database as _;

/// True when `value` is the tagged SHA-256 storage form `sha256:<64 lowercase hex>`.
fn is_tagged_sha256(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn manifest(aligned: &str, translator: &str, passages: &[(u16, u32, &str)]) -> String {
    manifest_with_license(aligned, translator, None, passages)
}

/// Build a translation manifest, optionally declaring an SPDX license id.
fn manifest_with_license(
    aligned: &str,
    translator: &str,
    spdx_id: Option<&str>,
    passages: &[(u16, u32, &str)],
) -> String {
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
    let spdx = match spdx_id {
        Some(id) => serde_json::to_string(id).unwrap(),
        None => "null".to_string(),
    };
    format!(
        "{{\"translation\":{{\"slug\":\"en-test\",\"version\":\"1.0.0\",\"name\":\"Test English\",\
         \"translator\":\"{translator}\",\"language\":\"en\",\"aligned_edition\":\"{aligned}\",\
         \"numbering_scheme\":\"hafs\",\"spdx_id\":{spdx}}},\"passages\":[{passages}]}}"
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
    let hash_ascending =
        uow.quran().list_translation_editions().await.unwrap()[0].text_hash.clone();
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

/// A declared translation license is persisted verbatim: the SPDX identifier is
/// preserved character-for-character, never synthesized and never inferred from
/// the translator, publisher, or repository (QC-09 tail, D-04, T-02-24).
#[tokio::test]
async fn import_translations_persists_the_declared_license_verbatim() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let text = manifest_with_license(
        "test-edition-min@0.1.0",
        "Test Translator",
        Some("CC-BY-4.0"),
        &[(1, 1, "licensed one one")],
    );
    import_translations(
        &*db,
        &text,
        "12345678-1234-1234-1234-123456789abc",
        &principal(),
        &timestamp(),
    )
    .await
    .expect("declared-license manifest imports");

    let mut uow = db.write().await.unwrap();
    let editions = uow.quran().list_translation_editions().await.unwrap();
    assert_eq!(editions.len(), 1);
    let license: serde_json::Value =
        serde_json::from_str(&editions[0].license_json).expect("license_json is JSON");
    assert_eq!(
        license["spdx_id"].as_str(),
        Some("CC-BY-4.0"),
        "the declared identifier must be stored character-for-character"
    );
    assert_ne!(
        license["status"].as_str(),
        Some("Unknown"),
        "a declared license must not read back as undeclared"
    );
    assert_eq!(license["redistribution_allowed"].as_bool(), Some(false));
    assert_eq!(license["export_allowed"].as_bool(), Some(false));
}

/// An undeclared translation license stays explicitly `Unknown` with no invented
/// permission flags (QC-09 tail, D-04, T-02-24).
#[tokio::test]
async fn import_translations_keeps_an_undeclared_license_unknown() {
    let (_dir, db, _reader, _path) = active_reader().await;
    let text = manifest_with_license(
        "test-edition-min@0.1.0",
        "Test Translator",
        None,
        &[(1, 1, "unlicensed one one")],
    );
    import_translations(
        &*db,
        &text,
        "12345678-1234-1234-1234-123456789abc",
        &principal(),
        &timestamp(),
    )
    .await
    .expect("undeclared-license manifest imports");

    let mut uow = db.write().await.unwrap();
    let editions = uow.quran().list_translation_editions().await.unwrap();
    let license: serde_json::Value =
        serde_json::from_str(&editions[0].license_json).expect("license_json is JSON");
    assert_eq!(
        license["status"].as_str(),
        Some("Unknown"),
        "an undeclared license must stay explicitly unknown"
    );
    assert!(license["spdx_id"].is_null());
    assert_eq!(
        license["redistribution_allowed"].as_bool(),
        Some(false),
        "no redistribution permission may be invented"
    );
    assert_eq!(license["export_allowed"].as_bool(), Some(false));
}

/// Layer separation is enforced by construction and proven here: canonical and
/// translation rows live in **separate tables** (`quran_*` canonical rows versus
/// `translation_*` rows), are carried by **different types** (`QuranQuotation`
/// versus `AttributedTranslation`), and meet only in `AyahView` — where the
/// canonical slot holds the canonical quotation (canonical Arabic + canonical
/// hash) and a translation can only ride alongside it as an attributed sidecar.
///
/// This negative test proves the boundary on a real imported translation at the
/// read surface and scans `quran-core`'s view module for any canonical
/// constructor that accepts a translator/language pair (D-04, ADR-0112, T-02-22).
#[tokio::test]
async fn translation_cannot_reach_a_canonical_slot() {
    let (_dir, db, reader, _path) = active_reader().await;
    let translation_text = "translated words one one";
    let text = manifest("test-edition-min@0.1.0", "Test Translator", &[(1, 1, translation_text)]);
    import_translations(
        &*db,
        &text,
        "12345678-1234-1234-1234-123456789abc",
        &principal(),
        &timestamp(),
    )
    .await
    .expect("translation imports");

    // Serve the canonical ayah with the translation attached.
    let options =
        AyahOptions { translations: vec!["en-test".to_string()], glosses: false, tokens: false };
    let view = reader.get_ayah(&quran_core::parse("1:1").unwrap(), &options).await.unwrap();

    // The canonical slot is the canonical quotation: canonical Arabic and the
    // stored canonical per-ayah hash — never the translation text.
    let mut uow = db.write().await.unwrap();
    let edition = uow
        .quran()
        .get_edition_by_slug_version("test-edition-min", "0.1.0")
        .await
        .unwrap()
        .unwrap();
    let canonical_row = uow.quran().get_ayah(&edition.id, 1, 1).await.unwrap().unwrap();
    assert_eq!(view.canonical.arabic_text(), canonical_row.text);
    assert_ne!(view.canonical.arabic_text(), translation_text);
    assert_eq!(quran_corpus::tagged(view.canonical.text_hash()), canonical_row.text_hash);
    assert_eq!(view.canonical.edition().slug, "test-edition-min");
    // The canonical quotation's only translation channel is an identity
    // reference (`TranslationRef`), which carries no text at all.
    assert!(view.canonical.translation().is_none());

    // The translation text appears only in the attributed sidecar, with its
    // translator and edition reference.
    assert_eq!(view.translations.len(), 1);
    let sidecar = &view.translations[0];
    assert_eq!(sidecar.text(), translation_text);
    assert_eq!(sidecar.translator(), "Test Translator");
    assert_eq!(sidecar.edition_ref(), "en-test@1.0.0");

    // Serialized, the layers stay distinct: the canonical object has no field
    // that can hold the translation text, and the translation is a sibling.
    let json = serde_json::to_value(&view).unwrap();
    assert_eq!(json["canonical"]["arabic_text"].as_str(), Some(canonical_row.text.as_str()));
    assert_eq!(json["translations"][0]["text"].as_str(), Some(translation_text));
    assert!(json["canonical"].get("text").is_none());
    assert_ne!(json["canonical"]["arabic_text"], json["translations"][0]["text"]);

    // The type boundary is not a comment: scan the canonical view module for a
    // public constructor that takes a translator/language pair and could return
    // a canonical quotation. The translation's own guarded constructor is the
    // only such signature today, and it returns a translation.
    assert_no_canonical_constructor_takes_translation(include_str!("../../quran-core/src/view.rs"));
}

/// Scan `quran-core`'s view module for a public constructor that takes a
/// translator/language pair and could return a canonical quotation.
fn assert_no_canonical_constructor_takes_translation(src: &str) {
    for chunk in src.split("pub fn ").skip(1) {
        let signature_end = chunk.find('{').unwrap_or(chunk.len());
        let signature = &chunk[..signature_end];
        if signature.contains("translator")
            && (signature.contains("language") || signature.contains("Language"))
        {
            assert!(
                !signature.contains("QuranQuotation"),
                "a canonical constructor accepts a translator/language pair: {signature}"
            );
        }
    }
    assert!(
        !src.contains("QuranQuotation::new"),
        "the canonical view module must not build a QuranQuotation from anything"
    );
}
