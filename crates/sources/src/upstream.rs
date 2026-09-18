//! Registry of known upstream data/reference repositories (ADR-0101).
//!
//! # sources::upstream
//!
//! Metadata about the external repositories Q-ai reads from. This is a
//! **catalog of sources**, not a connector: it records the verified facts an
//! adapter needs (role, repository licence, data-redistribution posture, pinned
//! revision) without depending on any upstream's internal schema.
//!
//! The hard rule this registry exists to enforce: a repository's software
//! licence does **not** grant rights to the data it contains. Repository
//! licence and [`UpstreamRepository::data_redistribution`] are separate fields.
//!
//! Verified facts and revisions are recorded in
//! `docs/02-architecture/upstream-sources.md`; update both together.

use serde::{Deserialize, Serialize};

/// The role an upstream repository plays in Q-ai.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpstreamRole {
    /// Edition/translation catalog and identifiers.
    EditionCatalog,
    /// Relational/database modeling and divisions reference.
    DatabaseReference,
    /// Integrity/checksum manifest design and reference.
    IntegrityManifest,
}

/// The licence covering the repository's software/packaging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryLicense {
    /// Unlicense (public domain), as in `fawazahmed0/quran-api`.
    PublicDomainUnlicense,
    /// MIT, as in `gaitco/quran-database` and `spqrxi/quranchecksum`.
    Mit,
    /// Not verified.
    Unknown,
}

/// Whether the *contained data text* may be redistributed.
///
/// This is deliberately independent of [`RepositoryLicense`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataRedistribution {
    /// Every item must be cleared individually; not verified.
    PerItemUnverified,
    /// The repository stores hashes only (no text to redistribute).
    HashOnlyNoText,
    /// Some items are redistributable and some are not; mixed.
    MixedPerItem,
    /// Not verified.
    Unknown,
}

/// A known upstream repository, with only verified metadata recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamRepository {
    /// Stable internal key (never an upstream identifier).
    pub id: &'static str,
    /// Upstream `owner/repository`.
    pub repository: &'static str,
    /// Canonical URL.
    pub url: &'static str,
    /// Role in Q-ai.
    pub role: UpstreamRole,
    /// Licence of the repository software/packaging.
    pub repository_license: RepositoryLicense,
    /// Redistribution posture of the contained data text.
    pub data_redistribution: DataRedistribution,
    /// The exact commit/tag the recorded facts were verified at.
    pub verified_revision: Option<&'static str>,
    /// Date the facts were verified (`YYYY-MM-DD`).
    pub verified_at: &'static str,
    /// Short, factual note (no inferred licence claims).
    pub notes: &'static str,
}

/// The repositories verified on 2026-09-18 (see `upstream-sources.md`).
pub const KNOWN_UPSTREAMS: &[UpstreamRepository] = &[
    UpstreamRepository {
        id: "quran-api",
        repository: "fawazahmed0/quran-api",
        url: "https://github.com/fawazahmed0/quran-api",
        role: UpstreamRole::EditionCatalog,
        repository_license: RepositoryLicense::PublicDomainUnlicense,
        data_redistribution: DataRedistribution::PerItemUnverified,
        verified_revision: Some("47ca096b0976443ba2eab2e45cdf0fb4096a2610"),
        verified_at: "2026-09-18",
        notes: "editions.json catalog; 492 entries / 98 languages / 33 ara-* at the pinned revision; per-translation licence undeclared upstream",
    },
    UpstreamRepository {
        id: "quran-database",
        repository: "gaitco/quran-database",
        url: "https://github.com/gaitco/quran-database",
        role: UpstreamRole::DatabaseReference,
        repository_license: RepositoryLicense::Mit,
        data_redistribution: DataRedistribution::MixedPerItem,
        verified_revision: Some("4e0cb3414fa3993666a1bf910de45a585ab314d2"),
        verified_at: "2026-09-18",
        notes: "6,236 ayahs / 134 editions / 835,624 ayah_edition rows; translations carry their own terms; rukus dataset is source-terms restricted",
    },
    UpstreamRepository {
        id: "quranchecksum",
        repository: "spqrxi/quranchecksum",
        url: "https://github.com/spqrxi/quranchecksum",
        role: UpstreamRole::IntegrityManifest,
        repository_license: RepositoryLicense::Mit,
        data_redistribution: DataRedistribution::HashOnlyNoText,
        verified_revision: Some("954244e634e7b9d0bfd9f501a7eb859ffff68db1"),
        verified_at: "2026-09-18",
        notes: "verse-level SHA-256 / NFC manifest for Uthmani Hafs an Asim; translation manifests are hash-only",
    },
];

/// Look up a known upstream by its stable internal [`UpstreamRepository::id`].
///
/// Fail-closed: an unknown id resolves to `None`.
pub fn get(id: &str) -> Option<&'static UpstreamRepository> {
    KNOWN_UPSTREAMS.iter().find(|u| u.id == id)
}

/// All known upstreams whose [`UpstreamRepository::role`] is `role`.
pub fn by_role(role: UpstreamRole) -> impl Iterator<Item = &'static UpstreamRepository> {
    KNOWN_UPSTREAMS.iter().filter(move |u| u.role == role)
}

/// Whether a repository licence grants rights to the data text.
///
/// Always `false` for an open repository whose data is per-item or unverified;
/// callers must consult the per-item licence record instead.
pub fn repository_license_covers_data(u: &UpstreamRepository) -> bool {
    matches!(u.data_redistribution, DataRedistribution::HashOnlyNoText)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_the_three_known_upstreams() {
        assert_eq!(KNOWN_UPSTREAMS.len(), 3);
        let ids: Vec<&str> = KNOWN_UPSTREAMS.iter().map(|u| u.id).collect();
        assert_eq!(ids, ["quran-api", "quran-database", "quranchecksum"]);
    }

    #[test]
    fn revisions_are_pinned_and_match_the_recorded_facts() {
        for u in KNOWN_UPSTREAMS {
            assert!(
                u.verified_revision.is_some_and(|r| r.len() == 40),
                "{} must be pinned to a full commit",
                u.id
            );
            assert_eq!(u.verified_at, "2026-09-18");
        }
        assert_eq!(
            get("quran-api").unwrap().verified_revision,
            Some("47ca096b0976443ba2eab2e45cdf0fb4096a2610")
        );
    }

    #[test]
    fn open_repository_license_does_not_imply_data_rights() {
        let api = get("quran-api").unwrap();
        assert_eq!(api.repository_license, RepositoryLicense::PublicDomainUnlicense);
        assert_eq!(api.data_redistribution, DataRedistribution::PerItemUnverified);
        assert!(!repository_license_covers_data(api));

        let sum = get("quranchecksum").unwrap();
        assert!(repository_license_covers_data(sum));
    }

    #[test]
    fn unknown_id_resolves_to_nothing() {
        assert!(get("not-a-source").is_none());
    }

    #[test]
    fn role_filter_is_exact() {
        let integrity: Vec<&str> = by_role(UpstreamRole::IntegrityManifest).map(|u| u.id).collect();
        assert_eq!(integrity, ["quranchecksum"]);
    }
}
