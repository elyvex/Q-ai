//! Phase 2 — normalization profiles L0–L8 and the append-only registry.
//!
//! A **profile** is an ordered, versioned rule list (`plan.md` §3.3). Profiles
//! — not individual rules — are what users select, what indexes are built
//! from, and what appears in `reproducibility.normalization_rule_set`.
//!
//! Profiles are **append-only**: changing a rule list requires a new version,
//! a full rebuild, and a drift report — never an in-place edit. The registry
//! enforces this by rejecting any re-registration of an existing
//! `(id, version)` with different rules
//! ([`NormalizationError::ProfileImmutable`]).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::NormalizationError;
use crate::rule::{RuleId, SemVer};

/// Profile identifier from the §8.2 strictness ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ProfileId {
    /// Exact canonical (identity).
    L0,
    /// Exact after whitespace normalization.
    L1,
    /// Ignore Quranic marks.
    L2,
    /// Ignore diacritics (primary search field).
    L3,
    /// Normalize hamza/alif.
    L4,
    /// Normalize Arabic/Persian code points (most permissive indexed).
    L5,
    /// Space-insensitive skeleton.
    L6,
    /// Morphological heuristic affix search (token-level only).
    L7,
    /// Fuzzy spelling search (experimental, off by default, query-time only).
    L8,
}

impl ProfileId {
    /// Canonical id string (`"L0.exact"` … `"L8.fuzzy"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::L0 => "L0.exact",
            Self::L1 => "L1.ws",
            Self::L2 => "L2.marks",
            Self::L3 => "L3.diacritics",
            Self::L4 => "L4.hamza",
            Self::L5 => "L5.codepoints",
            Self::L6 => "L6.skeleton",
            Self::L7 => "L7.affix",
            Self::L8 => "L8.fuzzy",
        }
    }

    /// Parse a canonical id string.
    ///
    /// # Errors
    ///
    /// Returns [`NormalizationError::UnknownProfile`] for anything outside
    /// `L0.exact`…`L8.fuzzy`.
    pub fn parse(text: &str) -> Result<Self, NormalizationError> {
        match text {
            "L0.exact" => Ok(Self::L0),
            "L1.ws" => Ok(Self::L1),
            "L2.marks" => Ok(Self::L2),
            "L3.diacritics" => Ok(Self::L3),
            "L4.hamza" => Ok(Self::L4),
            "L5.codepoints" => Ok(Self::L5),
            "L6.skeleton" => Ok(Self::L6),
            "L7.affix" => Ok(Self::L7),
            "L8.fuzzy" => Ok(Self::L8),
            other => Err(NormalizationError::UnknownProfile { profile: other.to_string() }),
        }
    }

    /// All profile ids in ladder order.
    #[must_use]
    pub const fn all() -> [Self; 9] {
        [Self::L0, Self::L1, Self::L2, Self::L3, Self::L4, Self::L5, Self::L6, Self::L7, Self::L8]
    }
}

impl std::fmt::Display for ProfileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A versioned, immutable rule list with its index policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    /// Which ladder rung.
    pub id: ProfileId,
    /// Profile version. Bumped (never edited) on any rule-list change.
    #[serde(with = "crate::semver_serde")]
    pub version: SemVer,
    /// Ordered rule list. Empty only for `L0.exact`.
    pub rules: Vec<RuleId>,
    /// PRD §8.2 label.
    pub label: String,
    /// Whether an index field is built from this profile.
    pub indexed: bool,
    /// Whether results must render the heuristic-matched label.
    pub heuristic: bool,
    /// Experimental, off-by-default profiles (L8 only in v1).
    pub experimental: bool,
}

impl Profile {
    /// Trace label for this profile, e.g. `"L3.diacritics@1.0.0"`.
    #[must_use]
    pub fn trace_label(&self) -> String {
        format!("{}@{}", self.id, self.version)
    }
}

/// Append-only registry of normalization profiles.
#[derive(Debug, Clone)]
pub struct ProfileRegistry {
    profiles: BTreeMap<(ProfileId, SemVer), Profile>,
}

impl ProfileRegistry {
    /// Registry seeded with the v1 ladder L0–L8 (`plan.md` §3.3).
    #[must_use]
    pub fn new() -> Self {
        let mut registry = Self { profiles: BTreeMap::new() };
        for profile in builtin_profiles() {
            // Built-ins are distinct keys by construction.
            registry.profiles.insert((profile.id, profile.version), profile);
        }
        registry
    }

    /// Look up a profile version.
    ///
    /// # Errors
    ///
    /// Returns [`NormalizationError::UnknownProfile`] when the id is unknown
    /// or the exact version is not registered (callers must not fall back
    /// silently to another version).
    pub fn get(&self, id: ProfileId, version: SemVer) -> Result<&Profile, NormalizationError> {
        self.profiles.get(&(id, version)).ok_or_else(|| NormalizationError::UnknownProfile {
            profile: format!("{id}@{version}"),
        })
    }

    /// Latest registered version of a profile id.
    ///
    /// # Errors
    ///
    /// Returns [`NormalizationError::UnknownProfile`] for unregistered ids.
    pub fn latest(&self, id: ProfileId) -> Result<&Profile, NormalizationError> {
        self.profiles
            .range((id, SemVer::new(0, 0, 0))..=(id, SemVer::new(u64::MAX, u64::MAX, u64::MAX)))
            .next_back()
            .map(|(_, p)| p)
            .ok_or_else(|| NormalizationError::UnknownProfile { profile: id.to_string() })
    }

    /// Register a new profile version.
    ///
    /// # Errors
    ///
    /// Returns [`NormalizationError::ProfileImmutable`] when `(id, version)`
    /// is already registered with a **different** rule list. Re-registering
    /// the identical profile is a no-op returning `false`; a fresh version
    /// returns `true`.
    pub fn register(&mut self, profile: Profile) -> Result<bool, NormalizationError> {
        let key = (profile.id, profile.version);
        if let Some(existing) = self.profiles.get(&key) {
            if existing.rules != profile.rules {
                return Err(NormalizationError::ProfileImmutable {
                    profile: profile.trace_label(),
                });
            }
            return Ok(false);
        }
        self.profiles.insert(key, profile);
        Ok(true)
    }

    /// Number of registered `(id, version)` entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    /// True when no profile is registered (never for a seeded registry).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }
}

impl Default for ProfileRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// The v1 ladder from `plan.md` §3.3, all at `1.0.0`.
fn builtin_profiles() -> Vec<Profile> {
    use RuleId::{
        N01, N02, N03, N04, N05, N06, N07, N08, N09, N10, N11, N12, N13, N14, N15, N16, N17, N18,
        N19, N20, N21,
    };
    let v = SemVer::new(1, 0, 0);
    // Cumulative ladders: each rung extends the previous rule list.
    let l1 = vec![N01, N11, N16];
    let l2 = [l1.clone(), vec![N04, N14]].concat();
    let l3 = [l2.clone(), vec![N03, N05, N02]].concat();
    let l4 = [l3.clone(), vec![N07, N06, N08]].concat();
    let l5 = [l4.clone(), vec![N10, N13, N09, N15]].concat();
    let l6 = [l5.clone(), vec![N12, N17]].concat();
    let l7 = [l5.clone(), vec![N18, N19, N20, N21]].concat();
    // L8 shares the L5 rule list; fuzziness is query-time Levenshtein, not a rule.
    let l8 = l5.clone();
    let profile = |id: ProfileId,
                   rules: Vec<RuleId>,
                   label: &str,
                   indexed: bool,
                   heuristic: bool,
                   experimental: bool| {
        Profile {
            id,
            version: v,
            rules,
            label: label.to_string(),
            indexed,
            heuristic,
            experimental,
        }
    };
    vec![
        profile(ProfileId::L0, vec![], "Exact canonical", true, false, false),
        profile(ProfileId::L1, l1, "Exact after whitespace normalization", true, false, false),
        profile(ProfileId::L2, l2, "Ignore Quranic marks", true, false, false),
        profile(ProfileId::L3, l3, "Ignore diacritics", true, false, false),
        profile(ProfileId::L4, l4, "Normalize hamza/alif", true, false, false),
        profile(ProfileId::L5, l5, "Normalize Arabic/Persian code points", true, false, false),
        profile(ProfileId::L6, l6, "Space-insensitive skeleton", true, false, false),
        profile(ProfileId::L7, l7, "Morphological (heuristic affix) search", true, true, false),
        profile(ProfileId::L8, l8, "Fuzzy spelling search", false, false, true),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Diagnostic;

    #[test]
    fn ladder_ids_parse_and_display() {
        for id in ProfileId::all() {
            assert_eq!(ProfileId::parse(id.as_str()).unwrap(), id);
        }
        assert_eq!(ProfileId::L3.to_string(), "L3.diacritics");
        assert!(ProfileId::parse("L9.fuzzy").is_err());
    }

    #[test]
    fn builtin_rule_lists_match_plan() {
        use RuleId::*;
        let registry = ProfileRegistry::new();
        assert_eq!(registry.len(), 9);
        let v = SemVer::new(1, 0, 0);
        let rules = |id: ProfileId| registry.get(id, v).unwrap().rules.clone();
        assert_eq!(rules(ProfileId::L0), vec![]);
        assert_eq!(rules(ProfileId::L1), vec![N01, N11, N16]);
        assert_eq!(rules(ProfileId::L2), vec![N01, N11, N16, N04, N14]);
        assert_eq!(rules(ProfileId::L3), vec![N01, N11, N16, N04, N14, N03, N05, N02]);
        assert_eq!(
            rules(ProfileId::L4),
            vec![N01, N11, N16, N04, N14, N03, N05, N02, N07, N06, N08]
        );
        let l5 = rules(ProfileId::L5);
        assert_eq!(&l5[..11], rules(ProfileId::L4));
        assert_eq!(&l5[11..], vec![N10, N13, N09, N15]);
        let l6 = rules(ProfileId::L6);
        assert_eq!(&l6[..l5.len()], l5);
        assert_eq!(&l6[l5.len()..], vec![N12, N17]);
        let l7 = rules(ProfileId::L7);
        assert_eq!(&l7[..l5.len()], l5);
        assert_eq!(&l7[l5.len()..], vec![N18, N19, N20, N21]);
        assert_eq!(rules(ProfileId::L8), l5);
    }

    #[test]
    fn flags_match_plan() {
        let registry = ProfileRegistry::new();
        let v = SemVer::new(1, 0, 0);
        assert!(registry.get(ProfileId::L7, v).unwrap().heuristic);
        assert!(!registry.get(ProfileId::L3, v).unwrap().heuristic);
        assert!(registry.get(ProfileId::L8, v).unwrap().experimental);
        assert!(!registry.get(ProfileId::L8, v).unwrap().indexed);
        assert!(registry.get(ProfileId::L0, v).unwrap().indexed);
    }

    #[test]
    fn registry_is_append_only() {
        let mut registry = ProfileRegistry::new();
        let v = SemVer::new(1, 0, 0);
        // Identical re-registration is a no-op.
        let same = registry.get(ProfileId::L3, v).unwrap().clone();
        assert_eq!(registry.register(same).unwrap(), false);
        // Same key, different rules: rejected, original untouched.
        let mut tampered = registry.get(ProfileId::L3, v).unwrap().clone();
        tampered.rules.push(RuleId::N12);
        let err = registry.register(tampered).unwrap_err();
        assert_eq!(err.code(), crate::error::codes::PROFILE_IMMUTABLE);
        assert_eq!(registry.get(ProfileId::L3, v).unwrap().rules.len(), 8);
        // A new version with different rules is accepted.
        let mut v2 = registry.get(ProfileId::L3, v).unwrap().clone();
        v2.version = SemVer::new(2, 0, 0);
        v2.rules.push(RuleId::N12);
        assert_eq!(registry.register(v2).unwrap(), true);
        assert_eq!(registry.latest(ProfileId::L3).unwrap().version, SemVer::new(2, 0, 0));
    }

    #[test]
    fn unknown_profile_versions_never_fall_back() {
        let registry = ProfileRegistry::new();
        let err = registry.get(ProfileId::L3, SemVer::new(9, 9, 9)).unwrap_err();
        assert_eq!(err.code(), crate::error::codes::UNKNOWN_PROFILE);
    }
}
