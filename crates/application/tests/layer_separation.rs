//! Layer-separation + prohibition source-scan backstop (05-07, SC5).
//!
//! Automated guard for ADR-0112/D-04/D-12: no canonical-slot component
//! accepts a translation/annotation, no dataset text is injected as HTML,
//! no remote URL or bundled font is referenced, and the TUI spawns no
//! process and speaks no HTTP. Follows the `answer_path_ledger.rs`
//! follows the `answer_path_ledger.rs` scanning precedent (`include_str!`
//! + text walk; directory walk at runtime for `web/` and `crates/tui/`).
//!
//! # Ceiling (honest limits)
//!
//! This is a **guard, not a proof**: it detects the listed literal markers.
//! A brand-new violation shape matching none of them must still be caught by
//! review. Each predicate below carries an inline self-check proving it
//! fires on a violating snippet.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn read_tree(dir: &Path, extensions: &[&str]) -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let entries = match std::fs::read_dir(&current) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if extensions.iter().any(|ext| path.extension().is_some_and(|e| e == *ext)) {
                if let Ok(text) = std::fs::read_to_string(&path) {
                    out.push((path, text));
                }
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Canonical-slot guard: the canonical component's props must not declare a
/// translation/annotation field (`name:` or `name?:`).
fn canonical_props_clean(source: &str) -> bool {
    for marker in ["translation", "annotation"] {
        for line in source.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") || trimmed.starts_with('*') || trimmed.starts_with("/*") {
                continue;
            }
            let lower = trimmed.to_lowercase();
            if lower.starts_with(marker)
                && lower[marker.len()..].trim_start().starts_with([':', '?'])
            {
                return false;
            }
            if lower.contains(&format!("{marker}:")) || lower.contains(&format!("{marker}?:")) {
                // Field-shaped occurrence outside comments (e.g. inside the
                // props interface). Destructured uses (`translation.text`)
                // contain `translation.` not `translation:` — allowed.
                if !lower.contains(&format!("{marker}."))
                    || lower.contains(&format!("{marker}:"))
                {
                    return false;
                }
            }
        }
    }
    true
}

/// Non-canonical components must never wear the canonical token class.
fn no_canonical_class(source: &str) -> bool {
    !source.contains("ayah-canonical")
}

/// No HTML injection of dataset text.
fn no_inner_html(source: &str) -> bool {
    !source.contains("dangerouslySetInnerHTML")
}

/// No remote URL: `http(s)://` outside localhost/127.0.0.1, no `//cdn`.
fn no_remote_url(source: &str) -> bool {
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with('*') {
            continue;
        }
        let lower = trimmed.to_lowercase();
        if lower.contains("//cdn") {
            return false;
        }
        for marker in ["https://", "http://"] {
            let mut rest = lower.as_str();
            while let Some(at) = rest.find(marker) {
                let after = &rest[at + marker.len()..];
                if !(after.starts_with("localhost") || after.starts_with("127.0.0.1")) {
                    return false;
                }
                rest = after;
            }
        }
    }
    true
}

/// No bundled webfont: `@font-face` with a `url(` source (OD-04).
fn no_bundled_font(source: &str) -> bool {
    let lower = source.to_lowercase();
    let mut rest = lower.as_str();
    while let Some(at) = rest.find("@font-face") {
        let block = &rest[at..];
        let end = block.find('}').map_or(block.len(), |i| i + 1);
        if block[..end].contains("url(") {
            return false;
        }
        rest = &block[end..];
    }
    true
}

/// TUI purity (D-12): no subprocess, no HTTP client.
///
/// The subprocess marker is the path-qualified `process::Command` (the shape
/// the tree uses); a bare `Command::new` is the palette's own constructor
/// and must not trip the scan.
fn tui_pure(source: &str) -> bool {
    for marker in [
        "process::Command",
        "reqwest::",
        "hyper::",
        "HttpClient",
        "http_client",
        "ureq::",
    ] {
        if source.contains(marker) {
            return false;
        }
    }
    true
}

#[test]
fn spa_canonical_slot_has_no_translation_prop() {
    let path = root().join("web/src/components/AyahColumn.tsx");
    let source = std::fs::read_to_string(&path).expect("AyahColumn.tsx exists");
    assert!(canonical_props_clean(&source), "AyahColumn declares a translation/annotation prop");
}

#[test]
fn spa_non_canonical_components_wear_no_canonical_class() {
    for file in ["web/src/components/TranslationPanel.tsx", "web/src/components/WordInspector.tsx"] {
        let source = std::fs::read_to_string(root().join(file)).expect("component exists");
        assert!(no_canonical_class(&source), "{file} uses the canonical token class");
    }
}

#[test]
fn spa_layer_tokens_define_all_three_families() {
    let source =
        std::fs::read_to_string(root().join("web/src/tokens/layers.css")).expect("layers.css exists");
    for family in ["--layer-canonical-", "--layer-translation-", "--layer-annotation-"] {
        assert!(source.contains(family), "layers.css lacks {family}");
    }
}

#[test]
fn spa_has_no_html_injection() {
    let files = read_tree(&root().join("web/src"), &["tsx", "ts"]);
    assert!(!files.is_empty(), "web/src has sources to scan");
    for (path, source) in &files {
        assert!(no_inner_html(source), "{} injects HTML", path.display());
    }
}

#[test]
fn spa_has_no_remote_url_or_bundled_font() {
    let files = read_tree(&root().join("web/src"), &["tsx", "ts", "css", "html"]);
    assert!(!files.is_empty(), "web/src has sources to scan");
    for (path, source) in &files {
        assert!(no_remote_url(source), "{} references a remote URL", path.display());
        assert!(no_bundled_font(source), "{} bundles a webfont", path.display());
    }
}

#[test]
fn tui_spawns_no_process_and_speaks_no_http() {
    let files = read_tree(&root().join("crates/tui/src"), &["rs"]);
    assert!(!files.is_empty(), "crates/tui/src has sources to scan");
    for (path, source) in &files {
        assert!(tui_pure(source), "{} spawns a process or speaks HTTP", path.display());
    }
}

#[test]
fn scan_predicates_catch_violations() {
    // Each predicate fires on a violating snippet and passes a clean one —
    // the self-check proving the scan is not vacuous.
    assert!(!canonical_props_clean("interface P {\n translation?: string\n}"));
    assert!(canonical_props_clean("// no translation prop here\nconst x = 1;"));
    assert!(!no_canonical_class("<p className=\"ayah-canonical\">"));
    assert!(!no_inner_html("<div dangerouslySetInnerHTML={{__html: t}} />"));
    assert!(!no_remote_url("import x from 'https://cdn.example/x.js'"));
    assert!(no_remote_url("// proxy to http://localhost:8737 in dev"));
    assert!(!no_bundled_font("@font-face { src: url(font.woff2); }"));
    assert!(!tui_pure("std::process::Command::new(\"qai\")"));
    assert!(tui_pure("let x = 1;"));
}
