//! `cargo xtask adr-lint` — validate Phase-0 ADRs (AC-P0-19).
//!
//! Every ADR-0001 … ADR-0012 must exist, be `Accepted`, and contain all §48
//! fields — including the **religious-source** and **licensing** implications.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

/// The Phase-0 ADR numbers that must exist.
pub const REQUIRED_ADRS: [&str; 12] = [
    "0001", "0002", "0003", "0004", "0005", "0006", "0007", "0008", "0009", "0010", "0011", "0012",
];

/// Substrings each ADR must contain (§48 template fields).
pub const REQUIRED_SECTIONS: [&str; 11] = [
    "Status: Accepted",
    "## Context",
    "Options",
    "## Decision",
    "Accuracy",
    "Religious-Source",
    "Licensing",
    "Security",
    "Operational",
    "Migration Strategy",
    "Reversal Cost",
];

const ADR_DIR: &str = "docs/02-architecture/decisions";

/// Return the missing §48 sections for one ADR's content.
pub fn missing_sections(content: &str) -> Vec<&'static str> {
    REQUIRED_SECTIONS.iter().copied().filter(|section| !content.contains(section)).collect()
}

/// Find `ADR-00NN-*.md` for a given number.
fn adr_path(dir: &Path, number: &str) -> Option<PathBuf> {
    let prefix = format!("ADR-{number}-");
    std::fs::read_dir(dir).ok()?.flatten().map(|e| e.path()).find(|p| {
        p.file_name().and_then(|s| s.to_str()).map(|n| n.starts_with(&prefix)).unwrap_or(false)
    })
}

/// Entry point for `cargo xtask adr-lint`.
pub fn run() -> Result<()> {
    let root = std::env::current_dir()?;
    let dir = root.join(ADR_DIR);
    if !dir.exists() {
        bail!("adr-lint: ADR directory not found at {}", dir.display());
    }

    let mut problems = Vec::new();
    for number in REQUIRED_ADRS {
        let Some(path) = adr_path(&dir, number) else {
            problems.push(format!("ADR-{number}: file missing"));
            continue;
        };
        let content =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let missing = missing_sections(&content);
        if !missing.is_empty() {
            problems.push(format!(
                "ADR-{number} ({}): missing {}",
                path.file_name().and_then(|s| s.to_str()).unwrap_or("?"),
                missing.join(", ")
            ));
        }
    }

    if problems.is_empty() {
        println!("adr-lint: OK — {} ADRs present, Accepted, and complete.", REQUIRED_ADRS.len());
        Ok(())
    } else {
        for p in &problems {
            eprintln!("adr-lint: {p}");
        }
        bail!("adr-lint: {} problem(s)", problems.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_adr_has_no_missing_sections() {
        let mut content = String::new();
        for section in REQUIRED_SECTIONS {
            content.push_str(section);
            content.push('\n');
        }
        assert!(missing_sections(&content).is_empty());
    }

    #[test]
    fn incomplete_adr_reports_missing_fields() {
        let content = "Status: Accepted\n## Context\n";
        let missing = missing_sections(content);
        assert!(missing.contains(&"Religious-Source"));
        assert!(missing.contains(&"Reversal Cost"));
    }
}
