//! `domain::SemVer` with a stable string serde representation.
//!
//! `SemVer` has no serde impls in `domain` (kept dependency-light there), so
//! this module provides `serialize`/`deserialize` for `#[serde(with = …)]`
//! use on version fields. The wire form is always `"MAJOR.MINOR.PATCH"`.

use domain::SemVer;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serializer};

/// Serialize a [`SemVer`] as `"MAJOR.MINOR.PATCH"`.
pub fn serialize<S: Serializer>(version: &SemVer, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&version.to_string())
}

/// Deserialize a [`SemVer`] from `"MAJOR.MINOR.PATCH"`.
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<SemVer, D::Error> {
    let text = String::deserialize(deserializer)?;
    text.parse().map_err(D::Error::custom)
}
