//! Upstream edition-catalog adapter: `fawazahmed0/quran-api` `editions.json`
//! into [`UpstreamEditionRef`] metadata (ADR-0101).
//!
//! # quran_corpus::upstream_catalog
//!
//! This adapter ingests **catalog metadata only** — identifiers, authorship,
//! language, direction, source links, and provenance comments. It never fetches
//! or vendors edition text: every produced record carries `license: None`
//! (unknown) and the [`DataQualityFlag::LicenseUnknown`] flag, so no caller can
//! mistake a catalog entry for redistribution permission.
//!
//! Verified upstream facts (pinned revision, key shapes, inventory counts) are
//! recorded in `docs/02-architecture/upstream-sources.md` §1; the repository
//! roles and revisions live in [`sources::upstream`]. Update both together when
//! the pin moves.
//!
//! ## Honesty rules enforced here
//!
//! - The upstream `name` (hyphenated) is preserved verbatim as
//!   `upstream_edition_slug`; the object key (underscored) as
//!   `upstream_catalog_key`. Nothing is renamed.
//! - The catalog carries **no** `qiraah` / `riwayah` / `script` field, so the
//!   adapter records [`Qiraah::Unknown`] / [`Riwayah::Unknown`] / `script: None`
//!   unless the upstream slug itself names a transmission — and then only with
//!   `transmission_evidence` set to `inferred-from-upstream-name:<fragment>`.
//!   `Uthmani ⇒ Hafs` is never inferred.
//! - `qiraah` is **never** back-filled from a riwayah (no `Hafs ⇒ Asim`); the
//!   well-known mapping in [`Riwayah::qiraah`] is query-side knowledge, not a
//!   source claim.
//! - Content-kind classification treats `ara-*` tafsir and transliteration
//!   entries as non-text; `ara-jalaladdinalmah` is classified from its author
//!   names and flagged [`DataQualityFlag::ProvenanceUnknown`] because the
//!   upstream declares no `type` field.
//! - Non-Unicode editions (`*nonun`, `ara-qurannastaleeqn`) are flagged
//!   [`DataQualityFlag::NonUnicode`]: reference-only until a declared Unicode
//!   conversion exists.
//! - The caller supplies the upstream `revision` the bytes were read at; when
//!   it is unknown the record stays explicitly unpinned (`revision: None`).

use quran_core::catalog::{
    Attribution, DataQualityFlag, EditionContentKind, Qiraah, Riwayah, UpstreamEditionRef,
    UpstreamEditionSlug, UpstreamSourceRef,
};
use quran_core::enums::Script;

use crate::error::CorpusError;

/// Adapter name used in [`CorpusError::AdapterFailed`] reports.
pub const NAME: &str = "quran-api-catalog";

/// Upstream repository identity recorded on every produced record.
pub const REPOSITORY: &str = "fawazahmed0/quran-api";

/// Path of the catalog file inside the upstream repository.
pub const CATALOG_PATH: &str = "editions.json";

/// One raw entry of `editions.json`. All upstream values arrive as strings;
/// empty strings decode to `None` (the catalog leaves `source` and `comments`
/// empty for many translations).
///
/// The upstream `language` display name (e.g. `"Arabic"`) is intentionally not
/// ingested: it is a free-text label, while the object-key `<lang>_` prefix is
/// the stable language identity. Serde ignores the undeclared field.
#[derive(Debug, Clone, serde::Deserialize)]
struct RawEntry {
    name: Option<String>,
    author: Option<String>,
    direction: Option<String>,
    source: Option<String>,
    comments: Option<String>,
    link: Option<String>,
    linkmin: Option<String>,
}

impl RawEntry {
    fn text(value: &Option<String>) -> Option<String> {
        value.as_ref().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
    }
}

/// Slugs (hyphenated upstream `name`) the catalog declares as tafsir in the
/// slug itself.
const SLUG_DECLARES_TAFSIR: &[&str] = &["ara-sirajtafseer", "ara-sirajtafseernod"];

/// Upstream slug whose tafsir status is inferred from its author names
/// (Tafsīr al-Jalālayn). The upstream declares no `type` field, so this
/// classification must be confirmed before the entry feeds any text pipeline.
const INFERRED_TAFSIR_SLUG: &str = "ara-jalaladdinalmah";

/// `(slug fragment, transmission)` pairs read from upstream edition names.
/// Fragments are matched against the hyphenated slug; every hit is recorded
/// with `transmission_evidence`, never as an upstream-declared fact.
const NAMED_TRANSMISSIONS: &[(&str, Riwayah)] = &[
    ("uthmanihaf", Riwayah::Hafs),
    ("warsh", Riwayah::Warsh),
    ("qaloon", Riwayah::Qalun),
    ("soosi", Riwayah::AlSusi),
    ("doori", Riwayah::AlDuri),
    ("bazzi", Riwayah::AlBazzi),
    ("qumbul", Riwayah::Qunbul),
    ("shouba", Riwayah::Shubah),
];

/// Parse an `editions.json` document into catalog metadata records.
///
/// `revision` is the upstream commit/tag the bytes were read at (`None` when
/// unknown — the records then stay explicitly unpinned for reproducibility).
/// Entry order follows the document order; callers must not rely on it beyond
/// determinism for a fixed input.
pub fn parse_catalog(
    input: &str,
    revision: Option<&str>,
) -> Result<Vec<UpstreamEditionRef>, CorpusError> {
    let fail = |detail: String| CorpusError::AdapterFailed { adapter: NAME, detail };
    let value: serde_json::Value =
        serde_json::from_str(input).map_err(|err| fail(format!("not a JSON document: {err}")))?;
    let object =
        value.as_object().ok_or_else(|| fail("catalog must be a JSON object".to_string()))?;
    if object.is_empty() {
        return Err(fail("catalog is empty".to_string()));
    }
    let mut entries = Vec::with_capacity(object.len());
    for (key, raw) in object {
        let entry: RawEntry = serde_json::from_value(raw.clone())
            .map_err(|err| fail(format!("entry `{key}`: {err}")))?;
        entries.push(convert_entry(key, &entry, revision)?);
    }
    Ok(entries)
}

/// Validate `direction` without storing it: [`UpstreamEditionRef`] currently
/// carries no direction field (follow-up: add `direction`/script metadata to
/// the catalog type if surfaces need it).
fn check_direction(
    key: &str,
    direction: Option<&str>,
    fail: &impl Fn(String) -> CorpusError,
) -> Result<(), CorpusError> {
    match direction {
        Some("rtl") | Some("ltr") => Ok(()),
        other => Err(fail(format!(
            "entry `{key}`: unexpected direction `{}` (expected rtl|ltr)",
            other.unwrap_or("<missing>")
        ))),
    }
}

fn convert_entry(
    key: &str,
    raw: &RawEntry,
    revision: Option<&str>,
) -> Result<UpstreamEditionRef, CorpusError> {
    let fail = |detail: String| CorpusError::AdapterFailed { adapter: NAME, detail };

    let slug_text =
        RawEntry::text(&raw.name).ok_or_else(|| fail(format!("entry `{key}`: missing `name`")))?;
    let slug = UpstreamEditionSlug::try_new(slug_text.clone())
        .map_err(|_| fail(format!("entry `{key}`: invalid upstream slug `{slug_text}`")))?;

    // The language tag comes from the catalog key's `<lang>_` prefix (ISO
    // 639-2/3 codes, valid BCP-47 primary subtags). A key without that prefix
    // carries no language identity and fails closed.
    if !key.contains('_') {
        return Err(fail(format!(
            "entry `{key}`: key carries no `<lang>_` prefix for the language tag"
        )));
    }
    let prefix = key.split('_').next().filter(|p| !p.is_empty()).ok_or_else(|| {
        fail(format!("entry `{key}`: key carries no `<lang>_` prefix for the language tag"))
    })?;
    let language: domain::Language = prefix
        .parse()
        .map_err(|_| fail(format!("entry `{key}`: invalid language tag `{prefix}`")))?;

    check_direction(key, raw.direction.as_deref(), &fail)?;

    let content_kind = classify_content(&slug_text, key);
    let (riwayah, transmission_evidence) = infer_transmission(&slug_text);
    let quality = quality_flags(&slug_text, key, raw);

    let author = RawEntry::text(&raw.author);
    let link = RawEntry::text(&raw.link).or_else(|| RawEntry::text(&raw.linkmin));
    let attribution = author.map(|text| Attribution { text, url: link });
    // The upstream-declared `source` URL (e.g. qurancomplex.gov.sa) has no
    // field on UpstreamEditionRef yet — `source` there is the retrieval
    // provenance (repo + revision + path). Follow-up: add an
    // `upstream_declared_source` field if surfaces need it; until then the
    // value is deliberately dropped here rather than misfiled.
    let _declared_source_url = RawEntry::text(&raw.source);

    Ok(UpstreamEditionRef {
        upstream_edition_slug: slug,
        upstream_catalog_key: Some(key.to_string()),
        qai_edition_id: None,
        content_kind,
        script: script_for(&slug_text, content_kind),
        qiraah: Qiraah::Unknown,
        riwayah,
        transmission_evidence,
        language,
        source: UpstreamSourceRef {
            repository: REPOSITORY.to_string(),
            revision: revision.filter(|r| !r.trim().is_empty()).map(str::to_string),
            upstream_path: Some(CATALOG_PATH.to_string()),
            retrieved_at: None,
        },
        quality,
        license: None,
        attribution,
    })
}

/// Classify an entry without consulting any field the upstream does not
/// declare. `key` (object key) and the hyphenated slug carry the same prefix;
/// transliteration detection uses the key because `-la` suffixes are part of
/// the slug too — both spellings are checked for robustness.
fn classify_content(slug: &str, key: &str) -> EditionContentKind {
    if SLUG_DECLARES_TAFSIR.contains(&slug) || slug == INFERRED_TAFSIR_SLUG {
        return EditionContentKind::Tafsir;
    }
    let lower_key = key.to_lowercase();
    let lower_slug = slug.to_lowercase();
    let is_transliteration = ["_la", "_lad", "phonetic", "-la", "-lad"]
        .iter()
        .any(|frag| lower_key.contains(frag) || lower_slug.contains(frag));
    if is_transliteration {
        return EditionContentKind::Transliteration;
    }
    if key.starts_with("ara_") {
        EditionContentKind::QuranText
    } else {
        EditionContentKind::Translation
    }
}

/// Read a transmission from the upstream slug name only. Returns `Unknown`
/// with no evidence when the slug names none.
fn infer_transmission(slug: &str) -> (Riwayah, Option<String>) {
    let lower = slug.to_lowercase();
    for (fragment, riwayah) in NAMED_TRANSMISSIONS {
        if lower.contains(fragment) {
            return (riwayah.clone(), Some(format!("inferred-from-upstream-name:{fragment}")));
        }
    }
    (Riwayah::Unknown, None)
}

/// The catalog declares no `script` field, so script stays `None` everywhere
/// for now — including `uthmani`-named slugs, where assigning Uthmani would be
/// an inference, not a source fact. (Follow-up: adopt per-edition script only
/// from an explicit upstream field or a verified source statement.)
fn script_for(_slug: &str, _kind: EditionContentKind) -> Option<Script> {
    None
}

/// Quality/provenance flags parsed from the entry. Every record starts from
/// unknown-safety: licence, text provenance, and checksum state are all
/// unverified for catalog-only ingestion.
fn quality_flags(slug: &str, key: &str, raw: &RawEntry) -> Vec<DataQualityFlag> {
    let mut flags = vec![
        DataQualityFlag::LicenseUnknown,
        DataQualityFlag::ProvenanceUnknown,
        DataQualityFlag::ChecksumUnverified,
    ];
    let comments = raw.comments.as_deref().unwrap_or("").to_lowercase();
    if comments.contains("ocr") {
        flags.push(DataQualityFlag::Ocr);
    }
    let lower_key = key.to_lowercase();
    if lower_key.contains("nonun")
        || slug.to_lowercase().contains("nonun")
        || lower_key.contains("nastaleeqn")
    {
        flags.push(DataQualityFlag::NonUnicode);
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;
    use quran_core::catalog::EditionContentKind;

    const SAMPLE: &str = r#"{
        "ara_quranuthmanihaf": {
            "name": "ara-quranuthmanihaf",
            "author": "Quran Uthmani Hafs",
            "language": "Arabic",
            "direction": "rtl",
            "source": "https://qurancomplex.gov.sa/",
            "comments": "Version 13, use Uthmani fonts",
            "link": "https://cdn.example/ara-quranuthmanihaf.json",
            "linkmin": "https://cdn.example/ara-quranuthmanihaf.min.json"
        },
        "ara_quranwarsh": {
            "name": "ara-quranwarsh",
            "author": "Quran Warsh",
            "language": "Arabic",
            "direction": "rtl",
            "source": "https://qurancomplex.gov.sa/",
            "comments": "Version 8",
            "link": "https://cdn.example/ara-quranwarsh.json",
            "linkmin": "https://cdn.example/ara-quranwarsh.min.json"
        },
        "eng_abdelhaleem": {
            "name": "eng-abdelhaleem",
            "author": "Abdel Haleem",
            "language": "English",
            "direction": "ltr",
            "source": "",
            "comments": "",
            "link": "https://cdn.example/eng-abdelhaleem.json",
            "linkmin": "https://cdn.example/eng-abdelhaleem.min.json"
        }
    }"#;

    fn parsed() -> Vec<UpstreamEditionRef> {
        parse_catalog(SAMPLE, Some("47ca096b0976443ba2eab2e45cdf0fb4096a2610")).unwrap()
    }

    #[test]
    fn slugs_keys_and_revision_survive_verbatim() {
        let entries = parsed();
        assert_eq!(entries.len(), 3);
        let hafs = &entries[0];
        assert_eq!(hafs.upstream_edition_slug.as_str(), "ara-quranuthmanihaf");
        assert_eq!(hafs.upstream_catalog_key.as_deref(), Some("ara_quranuthmanihaf"));
        assert_eq!(hafs.qai_edition_id, None);
        assert_eq!(hafs.language.as_str(), "ara");
        assert_eq!(hafs.source.repository, REPOSITORY);
        assert_eq!(
            hafs.source.revision.as_deref(),
            Some("47ca096b0976443ba2eab2e45cdf0fb4096a2610")
        );
        assert_eq!(hafs.source.upstream_path.as_deref(), Some(CATALOG_PATH));
    }

    #[test]
    fn transmission_is_inferred_with_evidence_and_qiraah_stays_unknown() {
        let entries = parsed();
        let hafs = &entries[0];
        assert_eq!(hafs.riwayah, Riwayah::Hafs);
        assert_eq!(
            hafs.transmission_evidence.as_deref(),
            Some("inferred-from-upstream-name:uthmanihaf")
        );
        // No Hafs ⇒ Asim back-fill: qiraah is unknown until a source states it.
        assert_eq!(hafs.qiraah, Qiraah::Unknown);
        assert_eq!(entries[1].riwayah, Riwayah::Warsh);
        // Translations name no transmission.
        assert_eq!(entries[2].riwayah, Riwayah::Unknown);
        assert_eq!(entries[2].transmission_evidence, None);
    }

    #[test]
    fn script_is_never_inferred_from_the_slug() {
        for entry in parsed() {
            assert_eq!(entry.script, None);
        }
    }

    #[test]
    fn content_kinds_split_text_translation_tafsir_and_transliteration() {
        let entries = parsed();
        assert_eq!(entries[0].content_kind, EditionContentKind::QuranText);
        assert_eq!(entries[2].content_kind, EditionContentKind::Translation);

        let tafsir = r#"{"ara_sirajtafseer": {
            "name": "ara-sirajtafseer", "author": "Siraj", "language": "Arabic",
            "direction": "rtl", "source": "https://quranenc.com", "comments": "",
            "link": "https://cdn.example/x.json", "linkmin": "https://cdn.example/x.min.json"}}"#;
        let entries = parse_catalog(tafsir, None).unwrap();
        assert_eq!(entries[0].content_kind, EditionContentKind::Tafsir);

        let latin = r#"{"ara_quran_la": {
            "name": "ara-quran-la", "author": "Transliteration", "language": "Arabic",
            "direction": "ltr", "source": "", "comments": "",
            "link": "https://cdn.example/x.json", "linkmin": "https://cdn.example/x.min.json"}}"#;
        let entries = parse_catalog(latin, None).unwrap();
        assert_eq!(entries[0].content_kind, EditionContentKind::Transliteration);
    }

    #[test]
    fn jalaladdinalmah_is_tafsir_with_unknown_provenance() {
        let raw = r#"{"ara_jalaladdinalmah": {
            "name": "ara-jalaladdinalmah", "author": "Jalal Ad Din Al Mahalli & As Suyuti",
            "language": "Arabic", "direction": "rtl", "source": "https://tanzil.net",
            "comments": "", "link": "https://cdn.example/x.json",
            "linkmin": "https://cdn.example/x.min.json"}}"#;
        let entries = parse_catalog(raw, None).unwrap();
        // Classified from the author names (Tafsir al-Jalalayn), never as bare
        // Quran text — and still flagged until the classification is confirmed.
        assert_eq!(entries[0].content_kind, EditionContentKind::Tafsir);
        assert!(entries[0].has_flag(DataQualityFlag::ProvenanceUnknown));
    }

    #[test]
    fn non_unicode_editions_are_reference_only() {
        let raw = r#"{"ara_qurandoorinonun": {
            "name": "ara-qurandoorinonun", "author": "Quran Doori Non Unicode",
            "language": "Arabic", "direction": "rtl", "source": "https://qurancomplex.gov.sa/",
            "comments": "needs KFGQPC fonts", "link": "https://cdn.example/x.json",
            "linkmin": "https://cdn.example/x.min.json"}}"#;
        let entries = parse_catalog(raw, None).unwrap();
        assert!(entries[0].has_flag(DataQualityFlag::NonUnicode));
        assert_eq!(entries[0].riwayah, Riwayah::AlDuri);
    }

    #[test]
    fn ocr_comments_set_the_ocr_flag() {
        let raw = r#"{"eng_someocr": {
            "name": "eng-someocr", "author": "Someone", "language": "English",
            "direction": "ltr", "source": "", "comments": "OCR-derived from print",
            "link": "https://cdn.example/x.json", "linkmin": "https://cdn.example/x.min.json"}}"#;
        let entries = parse_catalog(raw, None).unwrap();
        assert!(entries[0].has_flag(DataQualityFlag::Ocr));
    }

    #[test]
    fn licence_is_always_unknown_so_redistribution_is_never_verified() {
        for entry in parsed() {
            assert_eq!(entry.license, None);
            assert!(entry.has_flag(DataQualityFlag::LicenseUnknown));
            assert!(!entry.redistribution_verified());
        }
    }

    #[test]
    fn attribution_keeps_author_and_link_verbatim() {
        let entries = parsed();
        let haleem = &entries[2];
        let attribution = haleem.attribution.as_ref().unwrap();
        assert_eq!(attribution.text, "Abdel Haleem");
        assert_eq!(attribution.url.as_deref(), Some("https://cdn.example/eng-abdelhaleem.json"));
    }

    #[test]
    fn unknown_revision_stays_explicitly_unpinned() {
        let entries = parse_catalog(SAMPLE, None).unwrap();
        assert_eq!(entries[0].source.revision, None);
        assert!(!entries[0].source.is_pinned());
    }

    #[test]
    fn malformed_catalogs_fail_closed() {
        let err = parse_catalog("{not json", None).unwrap_err();
        assert!(matches!(err, CorpusError::AdapterFailed { .. }));
        let err = parse_catalog("[1,2]", None).unwrap_err();
        assert!(matches!(err, CorpusError::AdapterFailed { .. }));
        let err = parse_catalog("{}", None).unwrap_err();
        assert!(matches!(err, CorpusError::AdapterFailed { .. }));
        // Missing name.
        let err = parse_catalog(
            r#"{"eng_x": {"author": "A", "language": "English", "direction": "ltr",
                "source": "", "comments": "",
                "link": "https://cdn.example/x.json", "linkmin": ""}}"#,
            None,
        )
        .unwrap_err();
        assert!(matches!(err, CorpusError::AdapterFailed { .. }));
        // Bad direction.
        let err = parse_catalog(
            r#"{"eng_x": {"name": "eng-x", "author": "A", "language": "English",
                "direction": "sideways", "source": "", "comments": "",
                "link": "https://cdn.example/x.json", "linkmin": ""}}"#,
            None,
        )
        .unwrap_err();
        assert!(matches!(err, CorpusError::AdapterFailed { .. }));
        // Key without a language prefix.
        let err = parse_catalog(
            r#"{"noprefix": {"name": "eng-x", "author": "A", "language": "English",
                "direction": "ltr", "source": "", "comments": "",
                "link": "https://cdn.example/x.json", "linkmin": ""}}"#,
            None,
        )
        .unwrap_err();
        assert!(matches!(err, CorpusError::AdapterFailed { .. }));
    }
}
