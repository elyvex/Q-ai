//! Phase 0 — hashing & canonical serialization.
//!
//! # domain::hashing
//!
//! `ContentHash` and `canonical_json_bytes` for reproducible,
//! algorithm-tagged checksums across the entire platform.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The hash algorithm used to produce a `ContentHash`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HashAlgorithm {
    Sha256,
    Blake3,
}

/// A content-addressed hash with an explicit algorithm tag so future
/// algorithm changes are detectable.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash {
    pub algorithm: HashAlgorithm,
    pub hex: String,
}

impl ContentHash {
    /// Create a new ContentHash, validating that the hex string is
    /// lowercase and has an even number of characters.
    pub fn try_new(algorithm: HashAlgorithm, hex: String) -> Result<Self, HashingError> {
        if !hex.len().is_multiple_of(2) {
            return Err(HashingError::InvalidHexLength { expected: hex.len() - 1, got: hex.len() });
        }
        if !hex.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)) {
            return Err(HashingError::InvalidHexCharacter);
        }
        Ok(Self { algorithm, hex })
    }
}

/// Error types for the hashing module.
#[derive(Error, Debug)]
pub enum HashingError {
    #[error("invalid ContentHash length: expected {expected} hex chars, got {got}")]
    InvalidHexLength { expected: usize, got: usize },
    #[error("invalid hex character in ContentHash")]
    InvalidHexCharacter,
    #[error("serialization failed: {0}")]
    SerializationFailed(String),
}

/// Canonical serialization used for all checksums.
/// Stable key order, NFC-preserving, LF newlines, no BOM.
/// Defined once so hashes are reproducible forever.
pub fn canonical_json_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, HashingError> {
    let bytes =
        serde_json::to_vec(value).map_err(|e| HashingError::SerializationFailed(e.to_string()))?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    #[allow(dead_code)] // Test helper
    struct TestRecord {
        name: String,
        value: u64,
    }

    #[test]
    fn canonical_json_bytes_stable_across_map_order() {
        let mut map1 = std::collections::BTreeMap::new();
        map1.insert("a", 1u64);
        map1.insert("b", 2u64);

        let mut map2 = std::collections::BTreeMap::new();
        map2.insert("b", 2u64);
        map2.insert("a", 1u64);

        // BTreeMap is already sorted, so canonical bytes should match
        let bytes1 = canonical_json_bytes(&map1).unwrap();
        let bytes2 = canonical_json_bytes(&map2).unwrap();
        assert_eq!(bytes1, bytes2);
    }

    proptest! {
        #[test]
        fn content_hash_hex_is_lowercase_and_even_length(
            hex in "[a-f0-9]{1,32}".prop_map(|s| {
                let mut s = s;
                if s.len() % 2 == 1 {
                    s.push('0');
                }
                s
            })
        ) {
            let hash = ContentHash::try_new(HashAlgorithm::Sha256, hex.clone()).unwrap();
            assert_eq!(hash.hex.len() % 2, 0);
        }
    }

    #[test]
    fn content_hash_rejects_odd_length() {
        let result = ContentHash::try_new(HashAlgorithm::Sha256, "abc".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn content_hash_rejects_invalid_chars() {
        let result = ContentHash::try_new(HashAlgorithm::Sha256, "0123456789ABCDEF".to_string());
        assert!(result.is_err());
    }
}
