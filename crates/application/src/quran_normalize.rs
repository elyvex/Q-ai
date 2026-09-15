//! Phase 2 — normalization services shared by CLI, server, and tools (M1c).
//!
//! Two definition sources exist by design:
//!
//! - [`builtin_registry`] — the in-code v1 ladder, used by the server
//!   preview endpoint (which has no database handle) and as the parity
//!   reference.
//! - [`registry_from_rows`] — the same ladder rebuilt from seeded database
//!   rows, used by the CLI so every `normalize` call proves the seed.
//!
//! `tests/normalization_seed.rs` binds the two: any drift between the
//! migration seed and the code fails the build.

use quran_normalization::{
    ProfileRegistry,
    rules::{all_rules, by_id, heuristic_rules},
};

/// Re-exported so the CLI and server share these types without a direct
/// dependency on the normalization crate (arch rule AC-P2-36).
pub use quran_normalization::{
    NormalizationPipeline, NormalizationTrace, Profile, ProfileId, RuleId, SemVer, StepOutput,
    error::{Diagnostic as NormalizationDiagnostic, NormalizationError},
};

/// Preview output: derived text with its trace (and per-rule steps).
#[derive(Debug, Clone)]
pub struct Preview {
    /// Input text as given.
    pub input: String,
    /// Trace label (`profile@version` or `adhoc:…`).
    pub profile: String,
    /// Derived output text.
    pub output: String,
    /// Exact ordered rule set applied (I9).
    pub trace: NormalizationTrace,
    /// Intermediate text after each rule, in order.
    pub steps: Vec<StepOutput>,
}

/// Rule metadata for `--show-rule` (implementation facts from code).
#[derive(Debug, Clone)]
pub struct RuleMeta {
    /// Short id (`N06`).
    pub id: RuleId,
    /// Snake-case name (`normalize_hamza_forms`).
    pub name: &'static str,
    /// Implementation version.
    pub version: SemVer,
    /// Human description.
    pub description: &'static str,
    /// Deterministic vs heuristic.
    pub heuristic: bool,
    /// Reserved for a later phase (N23/N24).
    pub reserved: bool,
}

/// The in-code v1 ladder (parity reference for the migration seed).
#[must_use]
pub fn builtin_registry() -> ProfileRegistry {
    ProfileRegistry::new()
}

/// Built-in profiles in ladder order (for the catalog endpoint).
#[must_use]
pub fn builtin_registry_profiles() -> Vec<Profile> {
    let registry = ProfileRegistry::new();
    let v = SemVer::new(1, 0, 0);
    ProfileId::all().iter().filter_map(|id| registry.get(*id, v).ok().cloned()).collect()
}

/// Parse a `--profile` value: `L3.diacritics` (latest) or
/// `L3.diacritics@1.0.0` (pinned).
///
/// # Errors
///
/// Returns [`NormalizationError::UnknownProfile`] for unknown ids or
/// malformed versions.
pub fn parse_profile_spec(raw: &str) -> Result<(ProfileId, Option<SemVer>), NormalizationError> {
    match raw.split_once('@') {
        Some((id, version)) => {
            let id = ProfileId::parse(id)?;
            let version: SemVer = version
                .parse()
                .map_err(|_| NormalizationError::UnknownProfile { profile: raw.to_string() })?;
            Ok((id, Some(version)))
        }
        None => Ok((ProfileId::parse(raw)?, None)),
    }
}

/// Parse a `--rules` value: comma-separated rule ids (`N01,N03,N06`).
///
/// # Errors
///
/// Returns [`NormalizationError::UnknownRule`] for unknown ids.
pub fn parse_rule_list(raw: &str) -> Result<Vec<RuleId>, NormalizationError> {
    raw.split(',').map(str::trim).filter(|s| !s.is_empty()).map(RuleId::parse).collect()
}

/// Preview text through a registry profile (server path: built-in registry).
///
/// # Errors
///
/// Propagates unknown-profile errors; never falls back silently.
pub fn preview(
    registry: &ProfileRegistry,
    text: &str,
    id: ProfileId,
    version: Option<SemVer>,
) -> Result<Preview, NormalizationError> {
    let profile: &Profile = match version {
        Some(version) => registry.get(id, version)?,
        None => registry.latest(id)?,
    };
    let pipeline = NormalizationPipeline::for_profile(registry, profile.id, profile.version)?;
    Ok(assemble(text, &pipeline))
}

/// Preview text through an explicit rule list (adhoc pipeline).
///
/// # Errors
///
/// Returns [`NormalizationError::UnknownRule`] for unimplemented ids.
pub fn preview_adhoc(text: &str, rule_ids: &[RuleId]) -> Result<Preview, NormalizationError> {
    Ok(assemble(text, &NormalizationPipeline::adhoc(rule_ids)?))
}

fn assemble(text: &str, pipeline: &NormalizationPipeline) -> Preview {
    let (derived, trace, steps) = pipeline.apply_detailed(text);
    Preview {
        input: text.to_string(),
        profile: pipeline.profile_label().to_string(),
        output: derived.text().to_string(),
        trace,
        steps,
    }
}

/// Rebuild a [`ProfileRegistry`] from seeded database rows (CLI path).
///
/// Lets the CLI prove the migration seed on every call: the pipeline runs
/// from stored definitions, while implementations come from code. Any
/// seed/code drift surfaces through `tests/normalization_seed.rs`, not
/// through silent behavior change.
///
/// # Errors
///
/// Returns [`NormalizationError::UnknownProfile`] for rows naming unknown
/// profile ids, [`NormalizationError::UnknownRule`] for rows naming unknown
/// rule ids, and [`NormalizationError::InvalidMapping`] for malformed rows.
pub fn registry_from_rows(
    rows: &[storage::quran::NormalizationProfileRow],
) -> Result<ProfileRegistry, NormalizationError> {
    let mut registry = ProfileRegistry::new();
    for row in rows {
        let id = ProfileId::parse(&row.profile_id)?;
        let version: SemVer =
            row.version.parse().map_err(|_| NormalizationError::InvalidMapping {
                detail: format!("bad version in profile row {}", row.profile_id),
            })?;
        let rules: Vec<RuleId> = serde_json::from_str::<Vec<String>>(&row.rules_json)
            .map_err(|_| NormalizationError::InvalidMapping {
                detail: format!("bad rules_json in profile row {}", row.profile_id),
            })?
            .iter()
            .map(|s| RuleId::parse(s))
            .collect::<Result<Vec<_>, _>>()?;
        // Every stored rule must have an implementation behind it.
        for rule in &rules {
            by_id(*rule)
                .ok_or_else(|| NormalizationError::UnknownRule { rule: rule.to_string() })?;
        }
        let profile = Profile {
            id,
            version,
            rules,
            label: row.label.clone(),
            indexed: row.indexed,
            heuristic: row.heuristic,
            experimental: row.experimental,
        };
        // Seeded rows collide with the built-in keys by design; an
        // identical re-register is a no-op, a conflicting one errors.
        registry.register(profile)?;
    }
    Ok(registry)
}

/// Pick the latest version rows for one profile id from a listed set.
#[must_use]
pub fn latest_row(
    rows: &[storage::quran::NormalizationProfileRow],
    id: ProfileId,
) -> Option<&storage::quran::NormalizationProfileRow> {
    rows.iter()
        .filter(|r| r.profile_id == id.as_str())
        .max_by_key(|r| r.version.parse::<SemVer>().ok())
}

/// Implementation metadata for every rule id with an implementation.
#[must_use]
pub fn rule_meta(id: RuleId) -> Option<RuleMeta> {
    let rule = by_id(id)?;
    Some(RuleMeta {
        id,
        name: id.name(),
        version: rule.version(),
        description: rule.description(),
        heuristic: id.is_heuristic(),
        reserved: false,
    })
}

/// Metadata for all implemented rules in catalog order.
#[must_use]
pub fn all_rule_metas() -> Vec<RuleMeta> {
    all_rules()
        .into_iter()
        .chain(heuristic_rules())
        .map(|r| RuleMeta {
            id: r.id(),
            name: r.id().name(),
            version: r.version(),
            description: r.description(),
            heuristic: r.id().is_heuristic(),
            reserved: false,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_spec_parsing() {
        assert_eq!(parse_profile_spec("L3.diacritics").unwrap(), (ProfileId::L3, None));
        assert_eq!(
            parse_profile_spec("L3.diacritics@1.0.0").unwrap(),
            (ProfileId::L3, Some(SemVer::new(1, 0, 0)))
        );
        assert!(parse_profile_spec("L9.nope").is_err());
        assert!(parse_profile_spec("L3.diacritics@x").is_err());
    }

    #[test]
    fn rule_list_parsing() {
        assert_eq!(
            parse_rule_list("N01,N03, N06").unwrap(),
            vec![RuleId::N01, RuleId::N03, RuleId::N06]
        );
        assert!(parse_rule_list("N01,N99").is_err());
        assert!(parse_rule_list("").unwrap().is_empty());
    }

    #[test]
    fn preview_matches_pipeline_trace() {
        let registry = builtin_registry();
        let out = preview(&registry, "بِسْمِ", ProfileId::L3, None).unwrap();
        assert_eq!(out.output, "بسم");
        assert_eq!(out.profile, "L3.diacritics@1.0.0");
        assert_eq!(out.trace.profile, out.profile);
        assert_eq!(out.steps.len(), out.trace.rules_applied.len());
        // Steps walk the ladder in order, ending at the output.
        assert_eq!(out.steps.last().map(|s| s.text.as_str()), Some("بسم"));
    }

    #[test]
    fn rule_metas_cover_implemented_catalog() {
        let metas = all_rule_metas();
        assert_eq!(metas.len(), 22);
        assert!(metas.iter().all(|m| !m.reserved));
        assert_eq!(metas.iter().filter(|m| m.heuristic).count(), 5);
        assert!(rule_meta(RuleId::N23).is_none());
    }
}
