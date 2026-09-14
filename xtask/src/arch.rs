//! `cargo xtask arch-check` — enforce workspace dependency-direction rules.
//!
//! The rule set lives in `xtask/allowlist.toml` (source of truth: readme §6).
//! A workspace crate may only depend on workspace crates listed in its `allow` set;
//! a crate absent from the allowlist must have no workspace (path) dependencies.
//!
//! `violations()` is pure and unit-tested: the AC-P0-02 mutation test feeds it a
//! synthetic metadata graph containing the forbidden `domain -> cli` edge and asserts
//! the check fails.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

/// Minimal projection of `cargo metadata --format-version 1`.
#[derive(Debug, Deserialize)]
pub struct Metadata {
    /// Filesystem path IDs for workspace members, e.g. "path+file:///...#domain@0.0.0".
    #[serde(rename = "workspace_members")]
    pub workspace_member_ids: Vec<String>,
    #[serde(rename = "workspace_root")]
    pub workspace_root: String,
    pub packages: Vec<Package>,
}

#[derive(Debug, Deserialize)]
pub struct Package {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
}

/// A single dependency entry.
#[derive(Debug, Deserialize)]
pub struct Dependency {
    /// The target package's real name (cargo metadata always exposes the canonical name here,
    /// with the alias in `rename`).
    pub name: String,
    /// Present only for path/workspace dependencies (registry/git deps have a `source`).
    #[serde(default)]
    pub source: Option<String>,
}

/// The parsed allowlist, keyed by crate name.
///
/// A map (rather than Phase-0's hardcoded struct) so later phases can register new
/// crates in `allowlist.toml` without touching this gate. Fail-closed is preserved:
/// a crate absent from the map may depend on no workspace crate.
#[derive(Debug, Default, Deserialize)]
pub struct Allowlist {
    #[serde(flatten)]
    pub crates: HashMap<String, CrateRule>,
}

#[derive(Debug, Default, Deserialize)]
pub struct CrateRule {
    #[serde(default)]
    pub workspace: WorkspaceRule,
}

#[derive(Debug, Default, Deserialize)]
pub struct WorkspaceRule {
    #[serde(default)]
    pub allow: Vec<String>,
}

impl Allowlist {
    /// Returns the allowed workspace-dependency set for a crate, or an empty set if the
    /// crate is not recognized (fail-closed: unrecognized crates may depend on nothing).
    pub fn allowed_workspace_deps(&self, crate_name: &str) -> HashSet<&str> {
        match self.crates.get(crate_name) {
            Some(rule) => rule.workspace.allow.iter().map(String::as_str).collect(),
            None => Default::default(),
        }
    }
}

pub fn parse_allowlist(path: &Path) -> Result<Allowlist> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read allowlist at {}", path.display()))?;
    toml::from_str(&text).with_context(|| "failed to parse allowlist.toml")
}

/// Runs `cargo metadata` for the workspace rooted at `cwd`.
pub fn load_metadata(cwd: &Path) -> Result<Metadata> {
    let out = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1", "--all-features"])
        .current_dir(cwd)
        .output()
        .with_context(|| format!("failed to invoke cargo metadata in {}", cwd.display()))?;
    anyhow::ensure!(
        out.status.success(),
        "cargo metadata failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).context("failed to parse cargo metadata JSON")
}

/// Computes forbidden dependency edges. Pure — no I/O, no side effects.
///
/// Returns human-readable violation strings, one per offending edge.
pub fn violations(meta: &Metadata, allowlist: &Allowlist) -> Vec<String> {
    // Map each package id -> package (workspace members only).
    let member_ids: HashSet<&str> = meta.workspace_member_ids.iter().map(String::as_str).collect();
    let mut packages_by_id: HashMap<&str, &Package> = HashMap::new();
    for pkg in &meta.packages {
        if member_ids.contains(pkg.id.as_str()) {
            packages_by_id.insert(pkg.id.as_str(), pkg);
        }
    }

    let mut found = Vec::new();

    for pkg in packages_by_id.values() {
        let allowed = allowlist.allowed_workspace_deps(&pkg.name);
        let mut allowed_path_deps: HashSet<&str> = HashSet::new();
        for dep in &pkg.dependencies {
            // A workspace (path) dependency has `source == None`.
            if dep.source.is_none() {
                // cargo metadata exposes the canonical target package name in `name`.
                allowed_path_deps.insert(dep.name.as_str());
            }
        }
        let mut offending: Vec<String> = allowed_path_deps
            .iter()
            .filter(|target| !allowed.contains(**target))
            .map(|target| target.to_string())
            .collect();
        offending.sort();
        for target in offending {
            found.push(format!(
                "{} -> {} : forbidden dependency edge (not in allowlist)",
                pkg.name, target
            ));
        }
    }

    found
}

/// Entry point for `cargo xtask arch-check`.
pub fn run() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let meta = load_metadata(&cwd)?;

    // The allowlist lives beside this source, at <root>/xtask/allowlist.toml.
    let allowlist_path = Path::new(&meta.workspace_root).join("xtask/allowlist.toml");
    let allowlist = parse_allowlist(&allowlist_path)?;

    let bad = violations(&meta, &allowlist);
    if bad.is_empty() {
        println!("arch-check: OK — no forbidden dependency edges.");
        Ok(())
    } else {
        eprintln!("arch-check: {} forbidden edge(s):", bad.len());
        for line in &bad {
            eprintln!("  {line}");
        }
        anyhow::bail!("arch-check: FAIL — dependency-direction contracts violated (AC-P0-02).");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allowlist() -> Allowlist {
        let toml = r#"
            [domain]
            workspace = { allow = [] }
            [cli]
            workspace = { allow = ["application", "config", "observability"] }
        "#;
        toml::from_str(toml).unwrap()
    }

    fn package(id: &str, name: &str, deps: Vec<(&str, Option<&str>)>) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "name": name,
            "dependencies": deps
                .into_iter()
                .map(|(n, src)| serde_json::json!({ "name": n, "source": src }))
                .collect::<Vec<_>>(),
        })
    }

    fn meta(members: Vec<&str>, packages: Vec<serde_json::Value>) -> Metadata {
        let members: Vec<String> = members.into_iter().map(String::from).collect();
        let m = serde_json::json!({
            "workspace_members": members,
            "workspace_root": "/tmp/qai",
            "packages": packages,
        });
        serde_json::from_str(&m.to_string()).unwrap()
    }

    fn meta_with_forbidden_edge() -> Metadata {
        meta(
            vec!["#domain@0.0.0", "#cli@0.0.0"],
            vec![
                package("#domain@0.0.0", "domain", vec![("cli", None)]), // forbidden path edge
                package(
                    "#cli@0.0.0",
                    "cli",
                    vec![("serde", Some("registry+https://github.com/rust-lang/crates.io-index"))],
                ),
            ],
        )
    }

    fn meta_compliant() -> Metadata {
        meta(
            vec!["#domain@0.0.0", "#cli@0.0.0"],
            vec![
                // registry dep only -> legal; no path edges at all
                package(
                    "#domain@0.0.0",
                    "domain",
                    vec![("serde", Some("registry+https://github.com/rust-lang/crates.io-index"))],
                ),
                package(
                    "#cli@0.0.0",
                    "cli",
                    vec![("serde", Some("registry+https://github.com/rust-lang/crates.io-index"))],
                ),
            ],
        )
    }

    #[test]
    fn forbidden_edge_is_detected() {
        // AC-P0-02 mutation: domain -> cli must be flagged.
        let bad = violations(&meta_with_forbidden_edge(), &allowlist());
        assert!(
            bad.iter().any(|v| v.starts_with("domain -> cli")),
            "expected domain -> cli to be flagged, got: {bad:?}"
        );
    }

    #[test]
    fn registry_only_graph_is_clean() {
        // A graph with only registry deps and no path edges must pass.
        let bad = violations(&meta_compliant(), &allowlist());
        assert!(bad.is_empty(), "expected no violations, got: {bad:?}");
    }

    #[test]
    fn unrecognized_crate_fails_closed() {
        // A crate not in the allowlist (tui) with ANY path dependency is flagged;
        // a path dep in cli NOT in {application,config,observability} is flagged.
        let meta = meta(
            vec!["#domain@0.0.0", "#cli@0.0.0", "#tui@0.0.0"],
            vec![
                package("#domain@0.0.0", "domain", vec![("cli", None)]),
                package("#cli@0.0.0", "cli", vec![("tui", None)]),
                package("#tui@0.0.0", "tui", vec![("domain", None)]),
            ],
        );
        let bad = violations(&meta, &allowlist());
        assert!(bad.iter().any(|v| v.starts_with("cli -> tui")), "got: {bad:?}");
        assert!(bad.iter().any(|v| v.starts_with("tui -> domain")), "got: {bad:?}");
    }

    #[test]
    fn newly_registered_crate_uses_its_listed_edges() {
        // Phase-1 regression: a crate added only via allowlist.toml (no struct
        // change here) honours exactly its own allow set.
        let toml = r#"
            [quran-core]
            workspace = { allow = ["domain"] }
        "#;
        let allowlist: Allowlist = toml::from_str(toml).unwrap();
        let meta_allowed = meta(
            vec!["#quran-core@0.0.0", "#domain@0.0.0"],
            vec![
                package("#quran-core@0.0.0", "quran-core", vec![("domain", None)]),
                package("#domain@0.0.0", "domain", vec![]),
            ],
        );
        let bad = violations(&meta_allowed, &allowlist);
        assert!(bad.is_empty(), "expected no violations, got: {bad:?}");

        let meta_forbidden = meta(
            vec!["#quran-core@0.0.0", "#storage@0.0.0"],
            vec![
                package("#quran-core@0.0.0", "quran-core", vec![("storage", None)]),
                package("#storage@0.0.0", "storage", vec![]),
            ],
        );
        let bad = violations(&meta_forbidden, &allowlist);
        assert!(bad.iter().any(|v| v.starts_with("quran-core -> storage")), "got: {bad:?}");
    }
}
