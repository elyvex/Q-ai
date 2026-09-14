//! Corpus hashing: `text_hash` / `structure_hash` / `token_order_hash` (ADR-0108).
//!
//! # quran_corpus::hashing
//!
//! The recipe is frozen before the first import: changing it later invalidates
//! every stored hash. All three hashes are SHA-256 (`ContentHash` carries the
//! algorithm tag so a future algorithm never reinterprets old digests).
//!
//! Recipes (byte-exact):
//! - `text_hash`: `feed("qai-text-hash-v1")`, `feed(slug)`, `feed(version)`,
//!   then `feed(ayah_text)` for every ayah in `(surah, ayah)` order, where
//!   `feed(x)` is `len(x) as u64 LE || x`. Length-prefixing keeps
//!   `["ab", "c"]` distinct from `["a", "bc"]`.
//! - `structure_hash`: SHA-256 over one canonical string,
//!   `v1|{slug}|{version}|S{s}:{count},…|A{s}:{a}:{j}:{h}:{r}:{m}:{rk}:{p}:{sj};…`
//!   with `-` for absent divisions and `0|1|2` for sajdah
//!   (none|recommended|obligatory). Slugs and versions cannot contain the
//!   separators by construction.
//! - `token_order_hash`: `feed("qai-token-order-hash-v1")`, `feed(slug)`,
//!   `feed(version)`, then per token in global order `surah u16 LE || ayah u32
//!   LE || position u32 LE` plus `feed(surface)`.

use domain::{ContentHash, HashAlgorithm};
use quran_core::enums::SajdahKind;
use sha2::{Digest, Sha256};

fn feed(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn finish(hasher: Sha256) -> ContentHash {
    ContentHash { algorithm: HashAlgorithm::Sha256, hex: format!("{:x}", hasher.finalize()) }
}

/// SHA-256 hex digest of bytes (manifest hashing, CLI-declared hashes).
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Render a hash in storage form (`sha256:<hex>`), matching D0.6.
pub fn tagged(hash: &ContentHash) -> String {    let algorithm = match hash.algorithm {
        HashAlgorithm::Sha256 => "sha256",
        HashAlgorithm::Blake3 => "blake3",
    };
    format!("{algorithm}:{}", hash.hex)
}

/// Hash over the canonical ayah text stream in `(surah, ayah)` order.
pub fn text_hash(slug: &str, version: &str, texts_in_order: &[&str]) -> ContentHash {
    let mut hasher = Sha256::new();
    feed(&mut hasher, b"qai-text-hash-v1");
    feed(&mut hasher, slug.as_bytes());
    feed(&mut hasher, version.as_bytes());
    for text in texts_in_order {
        feed(&mut hasher, text.as_bytes());
    }
    finish(hasher)
}

/// One ayah's structural layout for [`structure_hash`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AyahLayout {
    /// Surah number.
    pub surah: u16,
    /// Ayah number.
    pub ayah: u32,
    /// Juz number, if known.
    pub juz: Option<u16>,
    /// Hizb number, if known.
    pub hizb: Option<u16>,
    /// Rubʿ number, if known.
    pub rub: Option<u16>,
    /// Manzil number, if known.
    pub manzil: Option<u16>,
    /// Ruku number, if known.
    pub ruku: Option<u16>,
    /// Page number, if known.
    pub page: Option<u32>,
    /// Sajdah kind, if any.
    pub sajdah: Option<SajdahKind>,
}

/// Hash over the structural skeleton: surah numbering plus per-ayah divisions.
pub fn structure_hash(
    slug: &str,
    version: &str,
    surahs: &[(u16, u16)],
    ayahs: &[AyahLayout],
) -> ContentHash {
    fn opt(value: Option<impl std::fmt::Display>) -> String {
        value.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string())
    }
    let mut canonical = format!("v1|{slug}|{version}|S");
    for (number, count) in surahs {
        canonical.push_str(&format!("{number}:{count},"));
    }
    canonical.push('|');
    canonical.push('A');
    for ayah in ayahs {
        let sajdah = match ayah.sajdah {
            None => 0,
            Some(SajdahKind::Recommended) => 1,
            Some(SajdahKind::Obligatory) => 2,
        };
        canonical.push_str(&format!(
            "{}:{}:{}:{}:{}:{}:{}:{}:{};",
            ayah.surah,
            ayah.ayah,
            opt(ayah.juz),
            opt(ayah.hizb),
            opt(ayah.rub),
            opt(ayah.manzil),
            opt(ayah.ruku),
            opt(ayah.page),
            sajdah
        ));
    }
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    finish(hasher)
}

/// One token's order identity for [`token_order_hash`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenOrder<'a> {
    /// Surah number.
    pub surah: u16,
    /// Ayah number.
    pub ayah: u32,
    /// 1-based position.
    pub position: u32,
    /// Surface form.
    pub surface: &'a str,
}

/// Hash over token identities and positions in global order.
pub fn token_order_hash(slug: &str, version: &str, tokens: &[TokenOrder<'_>]) -> ContentHash {
    let mut hasher = Sha256::new();
    feed(&mut hasher, b"qai-token-order-hash-v1");
    feed(&mut hasher, slug.as_bytes());
    feed(&mut hasher, version.as_bytes());
    for token in tokens {
        hasher.update(token.surah.to_le_bytes());
        hasher.update(token.ayah.to_le_bytes());
        hasher.update(token.position.to_le_bytes());
        feed(&mut hasher, token.surface.as_bytes());
    }
    finish(hasher)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_are_deterministic_and_tagged() {
        let first = text_hash("s", "1.0.0", &["a", "b"]);
        let second = text_hash("s", "1.0.0", &["a", "b"]);
        assert_eq!(first, second);
        assert_eq!(first.algorithm, HashAlgorithm::Sha256);
        assert_eq!(first.hex.len(), 64);
    }

    #[test]
    fn any_change_flips_the_digest() {
        let base = text_hash("s", "1.0.0", &["a", "b"]);
        assert_ne!(base, text_hash("s", "1.0.0", &["a", "c"]));
        assert_ne!(base, text_hash("s", "1.0.0", &["b", "a"]));
        assert_ne!(base, text_hash("other", "1.0.0", &["a", "b"]));
        assert_ne!(base, text_hash("s", "2.0.0", &["a", "b"]));
        // Length-prefixing defeats boundary-shift collisions.
        assert_ne!(text_hash("s", "1.0.0", &["ab", "c"]), text_hash("s", "1.0.0", &["a", "bc"]));
    }

    #[test]
    fn recipes_are_domain_separated() {
        let text = text_hash("s", "1.0.0", &["x"]);
        let structure = structure_hash(
            "s",
            "1.0.0",
            &[(1, 1)],
            &[AyahLayout {
                surah: 1,
                ayah: 1,
                juz: None,
                hizb: None,
                rub: None,
                manzil: None,
                ruku: None,
                page: None,
                sajdah: None,
            }],
        );
        let order = token_order_hash(
            "s",
            "1.0.0",
            &[TokenOrder { surah: 1, ayah: 1, position: 1, surface: "x" }],
        );
        assert_ne!(text.hex, structure.hex);
        assert_ne!(text.hex, order.hex);
        assert_ne!(structure.hex, order.hex);
    }

    #[test]
    fn structure_hash_covers_layout_changes() {
        let layout = |juz| AyahLayout {
            surah: 1,
            ayah: 1,
            juz,
            hizb: None,
            rub: None,
            manzil: None,
            ruku: None,
            page: None,
            sajdah: None,
        };
        let base = structure_hash("s", "1.0.0", &[(1, 1)], &[layout(Some(1))]);
        assert_ne!(base, structure_hash("s", "1.0.0", &[(1, 1)], &[layout(Some(2))]));
        assert_ne!(base, structure_hash("s", "1.0.0", &[(1, 2)], &[layout(Some(1))]));
    }
}
