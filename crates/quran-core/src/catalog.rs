//! Upstream edition catalog and integrity vocabulary (ADR-0101, ADR-0114).
//!
//! # quran_core::catalog
//!
//! Types for describing **where an edition came from** and **how it is
//! verified**, with script, qiraʾah, riwayah, edition and version kept strictly
//! separate (ADR-0101). These are pure metadata types: they describe upstream
//! material without asserting unverified facts. A field that is not known is
//! [`Qiraah::Unknown`] / [`Riwayah::Unknown`] / `None`, never a guess.
//!
//! Upstream identifiers are preserved verbatim in [`UpstreamEditionSlug`]; a
//! Q-ai-internal id (if any) is a separate mapping, never a rename.

use core::fmt;
use std::str::FromStr;

use domain::{EditionId, HashAlgorithm, Language, LicenseRecord, Timestamp};
use serde::{Deserialize, Serialize};

use crate::enums::Script;

/// A reading (qirāʾah) attributed to an imam, or `unknown`.
///
/// This is deliberately **not** exhaustive: it carries the well-known readings
/// plus [`Qiraah::Other`] for any value the source declares, and
/// [`Qiraah::Unknown`] when the source declares none. Membership in this list
/// is not a claim that Q-ai supports that reading as a corpus.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Qiraah {
    /// ʿĀṣim.
    Asim,
    /// Nāfiʿ.
    Nafi,
    /// Abū ʿAmr.
    AbuAmr,
    /// Ibn Kathīr.
    IbnKathir,
    /// Ibn ʿĀmir.
    IbnAmir,
    /// Ḥamzah.
    Hamzah,
    /// Al-Kisāʾī.
    AlKisai,
    /// Abū Jaʿfar.
    AbuJafar,
    /// Yaʿqūb.
    Yaqub,
    /// Khalaf.
    Khalaf,
    /// The source declares no reading.
    Unknown,
    /// A reading named by the source but not in this list, stored verbatim.
    Other(String),
}

impl Qiraah {
    /// The canonical lowercase spelling, or the verbatim `Other` value.
    pub fn as_str(&self) -> &str {
        match self {
            Qiraah::Asim => "asim",
            Qiraah::Nafi => "nafi",
            Qiraah::AbuAmr => "abu_amr",
            Qiraah::IbnKathir => "ibn_kathir",
            Qiraah::IbnAmir => "ibn_amir",
            Qiraah::Hamzah => "hamzah",
            Qiraah::AlKisai => "al_kisai",
            Qiraah::AbuJafar => "abu_jafar",
            Qiraah::Yaqub => "yaqub",
            Qiraah::Khalaf => "khalaf",
            Qiraah::Unknown => "unknown",
            Qiraah::Other(s) => s,
        }
    }

    /// Whether the value is a known, named reading (not `Unknown`/`Other`).
    pub fn is_known(&self) -> bool {
        !matches!(self, Qiraah::Unknown | Qiraah::Other(_))
    }
}

impl fmt::Display for Qiraah {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Qiraah {
    type Err = core::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "asim" => Qiraah::Asim,
            "nafi" => Qiraah::Nafi,
            "abu_amr" => Qiraah::AbuAmr,
            "ibn_kathir" => Qiraah::IbnKathir,
            "ibn_amir" => Qiraah::IbnAmir,
            "hamzah" => Qiraah::Hamzah,
            "al_kisai" => Qiraah::AlKisai,
            "abu_jafar" => Qiraah::AbuJafar,
            "yaqub" => Qiraah::Yaqub,
            "khalaf" => Qiraah::Khalaf,
            "" | "unknown" => Qiraah::Unknown,
            other => Qiraah::Other(other.to_owned()),
        })
    }
}

impl Serialize for Qiraah {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Qiraah {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Qiraah::from_str(&s).expect("infallible"))
    }
}

/// A transmission (riwāyah) of a reading, or `unknown`.
///
/// Kept separate from [`Qiraah`] per ADR-0101: `Uthmani ⇒ Hafs` is not a valid
/// inference, and neither is `Hafs ⇒ Asim` unless the source states it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Riwayah {
    /// Ḥafṣ ʿan ʿĀṣim.
    Hafs,
    /// Warsh ʿan Nāfiʿ.
    Warsh,
    /// Qālūn ʿan Nāfiʿ.
    Qalun,
    /// Al-Sūsī ʿan Abī ʿAmr.
    AlSusi,
    /// Al-Dūrī ʿan Abī ʿAmr.
    AlDuri,
    /// Al-Bazzī ʿan Ibn Kathīr.
    AlBazzi,
    /// Qunbul ʿan Ibn Kathīr.
    Qunbul,
    /// Shuʿbah ʿan ʿĀṣim.
    Shubah,
    /// The source declares no transmission.
    Unknown,
    /// A transmission named by the source but not in this list, stored verbatim.
    Other(String),
}

impl Riwayah {
    /// The canonical lowercase spelling, or the verbatim `Other` value.
    pub fn as_str(&self) -> &str {
        match self {
            Riwayah::Hafs => "hafs",
            Riwayah::Warsh => "warsh",
            Riwayah::Qalun => "qalun",
            Riwayah::AlSusi => "al_susi",
            Riwayah::AlDuri => "al_duri",
            Riwayah::AlBazzi => "al_bazzi",
            Riwayah::Qunbul => "qunbul",
            Riwayah::Shubah => "shubah",
            Riwayah::Unknown => "unknown",
            Riwayah::Other(s) => s,
        }
    }

    /// Whether the value is a known, named transmission (not `Unknown`/`Other`).
    pub fn is_known(&self) -> bool {
        !matches!(self, Riwayah::Unknown | Riwayah::Other(_))
    }

    /// The reading this transmission belongs to, **only** when established by
    /// well-known attribution; otherwise `None`. This does not consult an
    /// upstream source.
    pub fn qiraah(&self) -> Option<Qiraah> {
        match self {
            Riwayah::Hafs | Riwayah::Shubah => Some(Qiraah::Asim),
            Riwayah::Warsh | Riwayah::Qalun => Some(Qiraah::Nafi),
            Riwayah::AlSusi | Riwayah::AlDuri => Some(Qiraah::AbuAmr),
            Riwayah::AlBazzi | Riwayah::Qunbul => Some(Qiraah::IbnKathir),
            Riwayah::Unknown | Riwayah::Other(_) => None,
        }
    }
}

impl fmt::Display for Riwayah {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Riwayah {
    type Err = core::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "hafs" => Riwayah::Hafs,
            "warsh" => Riwayah::Warsh,
            "qalun" => Riwayah::Qalun,
            "al_susi" => Riwayah::AlSusi,
            "al_duri" => Riwayah::AlDuri,
            "al_bazzi" => Riwayah::AlBazzi,
            "qunbul" => Riwayah::Qunbul,
            "shubah" => Riwayah::Shubah,
            "" | "unknown" => Riwayah::Unknown,
            other => Riwayah::Other(other.to_owned()),
        })
    }
}

impl Serialize for Riwayah {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Riwayah {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Riwayah::from_str(&s).expect("infallible"))
    }
}

/// An upstream edition identifier, preserved exactly as the source spells it.
///
/// Renaming an upstream id is forbidden (ADR-0101); this newtype exists so the
/// external identity survives round-trips unchanged. Validation only rejects
/// empty/path-like values, not naming differences.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UpstreamEditionSlug(String);

/// A malformed upstream edition identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpstreamEditionSlugError;

impl fmt::Display for UpstreamEditionSlugError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("QAI-QUR-0103: invalid upstream edition slug")
    }
}

impl std::error::Error for UpstreamEditionSlugError {}

impl UpstreamEditionSlug {
    /// Validate and preserve an upstream identifier verbatim.
    pub fn try_new(value: impl Into<String>) -> Result<Self, UpstreamEditionSlugError> {
        let value = value.into();
        let valid = !value.is_empty()
            && !value.contains('/')
            && !value.contains('\\')
            && !value.contains("..")
            && value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.');
        if valid { Ok(Self(value)) } else { Err(UpstreamEditionSlugError) }
    }

    /// The identifier exactly as the upstream spells it.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UpstreamEditionSlug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// What kind of content a catalog entry holds (ADR-0101).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditionContentKind {
    /// A (potentially canonical) Quran text.
    QuranText,
    /// A translation of the Quran.
    Translation,
    /// A tafsir/commentary keyed to the Quran.
    Tafsir,
    /// A Latin/other transliteration (not Arabic script).
    Transliteration,
    /// A reference corpus used for comparison only.
    Reference,
    /// An integrity/checksum manifest (hash-only).
    Checksum,
}

/// Provenance/quality flags for imported data (ADR-0101 §data quality).
///
/// These are additive observations, not a single global `trusted` boolean.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataQualityFlag {
    /// Verified by a named human reviewer.
    Verified,
    /// Verified against the named upstream source, not editorially.
    SourceVerified,
    /// Transcribed from images via OCR (may contain errors).
    Ocr,
    /// Machine-transcribed (not OCR).
    MachineTranscription,
    /// Community-contributed source.
    CommunitySource,
    /// Stored in a legacy non-Unicode encoding (e.g. KFGQPC-font code points):
    /// reference-only until a declared conversion to Unicode exists.
    NonUnicode,
    /// The licence is not verified for this item.
    LicenseUnknown,
    /// The provenance chain is not fully known.
    ProvenanceUnknown,
    /// Integrity verified against a checksum manifest.
    ChecksumVerified,
    /// Integrity not yet verified against a checksum manifest.
    ChecksumUnverified,
}

/// Required attribution for an imported data family (ADR-0101).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attribution {
    /// The credit line to display.
    pub text: String,
    /// A URL to show alongside the credit, when the source provides one.
    pub url: Option<String>,
}

/// The scope a checksum manifest covers (ADR-0114).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrityScope {
    /// One hash per verse.
    Verse,
    /// One root hash per surah.
    Surah,
    /// One root hash for the whole edition.
    Edition,
    /// One root hash for a translation (distinct key from the Quran root).
    TranslationRoot,
}

/// A reference to one exact upstream retrieval (reproducibility, ADR-0101).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamSourceRef {
    /// `owner/repository`, e.g. `fawazahmed0/quran-api`.
    pub repository: String,
    /// Commit/tag the data was read at; `None` means not pinned.
    pub revision: Option<String>,
    /// Path within the repository, when applicable.
    pub upstream_path: Option<String>,
    /// When the artifact was retrieved.
    pub retrieved_at: Option<Timestamp>,
}

impl UpstreamSourceRef {
    /// A source reference with no revision or path yet pinned.
    pub fn new(repository: impl Into<String>) -> Self {
        Self {
            repository: repository.into(),
            revision: None,
            upstream_path: None,
            retrieved_at: None,
        }
    }

    /// Whether the source is pinned to an exact revision (reproducible).
    pub fn is_pinned(&self) -> bool {
        self.revision.as_deref().is_some_and(|r| !r.is_empty())
    }
}

/// A descriptor for one upstream edition/translation entry (ADR-0101).
///
/// This is catalog metadata, not canonical text. Unknown facts stay `None` /
/// [`Qiraah::Unknown`] / [`Riwayah::Unknown`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamEditionRef {
    /// Exact upstream identifier, never renamed.
    pub upstream_edition_slug: UpstreamEditionSlug,
    /// The upstream catalog's own object key, when it differs.
    pub upstream_catalog_key: Option<String>,
    /// Q-ai's internal edition id, when one has been assigned.
    pub qai_edition_id: Option<EditionId>,
    /// What the entry holds.
    pub content_kind: EditionContentKind,
    /// Declared orthography, if the source states one.
    pub script: Option<Script>,
    /// Declared reading, if any.
    pub qiraah: Qiraah,
    /// Declared transmission, if any.
    pub riwayah: Riwayah,
    /// Where the assignment above came from (e.g. `upstream-name`), when inferred.
    pub transmission_evidence: Option<String>,
    /// Content language.
    pub language: Language,
    /// Exact retrieval provenance.
    pub source: UpstreamSourceRef,
    /// Quality/provenance flags.
    pub quality: Vec<DataQualityFlag>,
    /// Licence record; `None` means not yet recorded (treat as unknown).
    pub license: Option<LicenseRecord>,
    /// Required attribution, when the source requires one.
    pub attribution: Option<Attribution>,
}

/// A reference to an integrity manifest and what it covers (ADR-0114).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityManifestRef {
    /// Hash algorithm.
    pub algorithm: HashAlgorithm,
    /// The explicit normalization rule applied before hashing.
    pub normalization: String,
    /// What the manifest's hashes cover.
    pub scope: IntegrityScope,
    /// Where the manifest came from.
    pub source: UpstreamSourceRef,
}

impl UpstreamEditionRef {
    /// Whether every declared quality flag set contains `flag`.
    pub fn has_flag(&self, flag: DataQualityFlag) -> bool {
        self.quality.contains(&flag)
    }

    /// Whether this entry may be redistributed: licence known and permissive.
    pub fn redistribution_verified(&self) -> bool {
        self.license.as_ref().is_some_and(|l| l.redistribution_allowed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{HashAlgorithm, LicenseStatus};

    fn source() -> UpstreamSourceRef {
        UpstreamSourceRef {
            repository: "fawazahmed0/quran-api".to_string(),
            revision: Some("47ca096b0976443ba2eab2e45cdf0fb4096a2610".to_string()),
            upstream_path: Some("editions/ara-quranuthmanihaf.json".to_string()),
            retrieved_at: None,
        }
    }

    #[test]
    fn qiraah_and_riwayah_roundtrip_as_strings() {
        assert_eq!(serde_json::to_string(&Qiraah::Asim).unwrap(), "\"asim\"");
        assert_eq!(serde_json::to_string(&Riwayah::Hafs).unwrap(), "\"hafs\"");
        assert_eq!(serde_json::from_str::<Qiraah>("\"asim\"").unwrap(), Qiraah::Asim);
        assert_eq!(serde_json::from_str::<Riwayah>("\"unknown\"").unwrap(), Riwayah::Unknown);
    }

    #[test]
    fn unknown_and_other_are_distinct() {
        assert!(!Qiraah::Unknown.is_known());
        let other: Qiraah = serde_json::from_str("\"aqra\"").unwrap();
        assert_eq!(other, Qiraah::Other("aqra".to_string()));
        assert!(!other.is_known());
        assert_eq!(serde_json::to_string(&other).unwrap(), "\"aqra\"");
    }

    #[test]
    fn known_transmissions_map_to_readings() {
        assert_eq!(Riwayah::Hafs.qiraah(), Some(Qiraah::Asim));
        assert_eq!(Riwayah::Warsh.qiraah(), Some(Qiraah::Nafi));
        assert_eq!(Riwayah::AlSusi.qiraah(), Some(Qiraah::AbuAmr));
        assert_eq!(Riwayah::Unknown.qiraah(), None);
    }

    #[test]
    fn upstream_slug_is_preserved_verbatim() {
        let slug = UpstreamEditionSlug::try_new("ara-quranuthmanihaf").unwrap();
        assert_eq!(slug.as_str(), "ara-quranuthmanihaf");
        assert_eq!(serde_json::to_string(&slug).unwrap(), "\"ara-quranuthmanihaf\"");
        // object-key form with underscores is also a valid, distinct identity
        let key = UpstreamEditionSlug::try_new("ara_quranuthmanihaf").unwrap();
        assert_ne!(key, slug);
    }

    #[test]
    fn upstream_slug_rejects_path_like_values() {
        assert!(UpstreamEditionSlug::try_new("").is_err());
        assert!(UpstreamEditionSlug::try_new("a/b").is_err());
        assert!(UpstreamEditionSlug::try_new("..").is_err());
        assert!(UpstreamEditionSlug::try_new("a b").is_err());
    }

    #[test]
    fn redistribution_requires_known_permission() {
        let mut entry = UpstreamEditionRef {
            upstream_edition_slug: UpstreamEditionSlug::try_new("ara-quranuthmanihaf").unwrap(),
            upstream_catalog_key: Some("ara_quranuthmanihaf".to_string()),
            qai_edition_id: None,
            content_kind: EditionContentKind::QuranText,
            script: Some(Script::Uthmani),
            qiraah: Qiraah::Unknown,
            riwayah: Riwayah::Unknown,
            transmission_evidence: None,
            language: "ar".parse().unwrap(),
            source: source(),
            quality: vec![DataQualityFlag::ProvenanceUnknown],
            license: None,
            attribution: None,
        };
        assert!(!entry.redistribution_verified());
        assert!(entry.has_flag(DataQualityFlag::ProvenanceUnknown));

        entry.license = Some(LicenseRecord {
            status: LicenseStatus::Unknown,
            spdx_id: None,
            name: None,
            url: None,
            attribution_required: false,
            redistribution_allowed: false,
            export_allowed: false,
            notes: None,
        });
        assert!(!entry.redistribution_verified());
    }

    #[test]
    fn source_ref_reports_pinning() {
        assert!(source().is_pinned());
        assert!(!UpstreamSourceRef::new("spqrxi/quranchecksum").is_pinned());
    }

    #[test]
    fn integrity_manifest_keeps_algorithm_and_scope_separate() {
        let manifest = IntegrityManifestRef {
            algorithm: HashAlgorithm::Sha256,
            normalization: "nfc".to_string(),
            scope: IntegrityScope::Verse,
            source: UpstreamSourceRef::new("spqrxi/quranchecksum"),
        };
        assert_eq!(manifest.algorithm, HashAlgorithm::Sha256);
        assert_eq!(manifest.scope, IntegrityScope::Verse);
        let json = serde_json::to_string(&manifest).unwrap();
        assert!(json.contains("\"scope\":\"verse\""));
    }
}
