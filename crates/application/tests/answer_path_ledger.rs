//! Answer-path ledger guard (G-02-3).
//!
//! The D-15 answer-path exemption ledger is a point-in-time snapshot that drifted
//! once already: every `file:line` anchor went stale, and a new emitting path
//! (tool `quran.search`, added by plan 03-07) was never dispositioned. This test
//! turns the ledger into a checked list that is resolved against the source tree,
//! so a rename/drift of a recorded path fails the build, and a *new* emitting
//! handler that uses one of the three known emission markers also fails the build.
//!
//! It follows the precedent of
//! `crates/application/tests/quran_translation.rs#translation_cannot_reach_a_canonical_slot`,
//! reading sources with `include_str!` and scanning them as text.
//!
//! # Ceiling (honest limits)
//!
//! This is a **guard, not a proof**. It detects emitting paths that use one of the
//! three literal markers `.arabic_text()`, `search_response(&headers,`, or
//! `ok_envelope(result.results`. A brand-new emission *shape* that matches none of
//! them — for example a hand-built `serde_json::json!` text field like
//! `tokens_handler` — is enumerated in [`LEDGER`] but cannot be detected literally
//! by the scan; adding such a handler must still be caught by the review
//! checklist. The ledger's completeness claim is correspondingly framed in
//! `docs/06-progress/phase-02-evidence.md` §3.

use std::collections::BTreeSet;

const API: &str = "crates/server/src/api.rs";
const CLI: &str = "crates/application/src/quran_cli.rs";
const TOOLS: &str = "crates/application/src/quran_tools.rs";

/// The machine-readable answer-path ledger: `(file, symbol, kind)`.
///
/// `kind` is the symbol's actual definition form in the tree (`async fn` for the
/// handler/CLI/tool functions, `struct` for the two helper types). The two
/// `struct` entries are helper types — the tool backend and the citation-source
/// doc anchor — recorded for drift protection, not emitting functions.
const LEDGER: &[(&str, &str, &str)] = &[
    (API, "surah_handler", "async fn"),
    (API, "divisions_handler", "async fn"),
    (API, "search_exact_handler", "async fn"),
    (API, "search_normalized_handler", "async fn"),
    (API, "search_phrase_handler", "async fn"),
    (API, "search_concatenated_handler", "async fn"),
    (API, "search_regex_handler", "async fn"),
    (API, "ayahs_handler", "async fn"),
    (API, "context_handler", "async fn"),
    (API, "tokens_handler", "async fn"),
    (API, "debug_reader_handler", "async fn"),
    (CLI, "cmd_get", "async fn"),
    (CLI, "cmd_context", "async fn"),
    (CLI, "cmd_surah", "async fn"),
    (CLI, "cmd_division", "async fn"),
    (CLI, "cmd_search", "async fn"),
    (CLI, "cmd_verify_quotation", "async fn"),
    (TOOLS, "verify_canonical_quotation", "async fn"),
    (TOOLS, "ReaderToolBackend", "struct"),
    (TOOLS, "ReaderCitationSource", "struct"),
];

/// Resolve a ledger file label to its source text.
fn source_for(file: &str) -> &'static str {
    if file == API {
        include_str!("../../server/src/api.rs")
    } else if file == CLI {
        include_str!("../src/quran_cli.rs")
    } else if file == TOOLS {
        include_str!("../src/quran_tools.rs")
    } else {
        panic!("unknown ledger file `{file}`");
    }
}

/// Kind-aware resolution: does `source` define `symbol` with the declared form?
fn resolve(source: &str, symbol: &str, kind: &str) -> bool {
    source.contains(&format!("{kind} {symbol}"))
}

/// Return the enclosing `async fn` name for every occurrence of `marker` in
/// `source`. Splits the source on `async fn ` so each marker is attributed to the
/// function whose body contains it; repeats a name once per occurrence.
fn scan_marker(source: &str, marker: &str) -> Vec<String> {
    let mut found = Vec::new();
    for chunk in source.split("async fn ").skip(1) {
        let occurrences = chunk.matches(marker).count();
        if occurrences == 0 {
            continue;
        }
        let name: String =
            chunk.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
        for _ in 0..occurrences {
            found.push(name.clone());
        }
    }
    found
}

/// The recorded symbols for one file.
fn ledger_symbols(file: &str) -> BTreeSet<&'static str> {
    LEDGER.iter().filter(|entry| entry.0 == file).map(|entry| entry.1).collect()
}

#[test]
fn every_ledger_symbol_resolves_under_its_declared_kind() {
    for &(file, symbol, kind) in LEDGER {
        assert!(
            resolve(source_for(file), symbol, kind),
            "ledger symbol `{symbol}` ({kind}) does not resolve in {file}"
        );
    }
}

#[test]
fn emitting_markers_resolve_to_recorded_paths() {
    let recorded = ledger_symbols(API);
    let markers = [".arabic_text()", "search_response(&headers,", "ok_envelope(result.results"];
    let expected = [3usize, 5usize, 2usize];
    for (marker, expected) in markers.iter().zip(expected) {
        let hits = scan_marker(source_for(API), marker);
        assert_eq!(hits.len(), expected, "occurrence count for `{marker}` changed");
        for name in hits {
            assert!(
                recorded.contains(name.as_str()),
                "emitting marker `{marker}` in `{name}` is not in the ledger"
            );
        }
    }
}

#[test]
fn cli_search_quotation_extraction_is_inside_cmd_search() {
    let hits = scan_marker(source_for(CLI), ".get(\"quotation\")");
    assert_eq!(hits, vec!["cmd_search".to_string()]);
}

#[test]
fn tool_search_path_is_present() {
    assert!(source_for(TOOLS).contains("\"quran.search\""));
}

#[test]
fn scanner_reports_an_unlisted_emitting_handler() {
    let synthetic = "async fn new_handler(state: State) -> Response { \
                     let body = view.canonical.arabic_text(); body }";
    let hits = scan_marker(synthetic, ".arabic_text()");
    assert_eq!(hits, vec!["new_handler".to_string()]);
    let recorded = ledger_symbols(API);
    assert!(!recorded.contains(hits[0].as_str()), "self-test handler must be unlisted");
}

#[test]
fn kind_aware_resolution_rejects_a_mismatched_definition() {
    let synthetic = "struct cmd_get { }";
    assert!(resolve(synthetic, "cmd_get", "struct"));
    assert!(!resolve(synthetic, "cmd_get", "async fn"));
}
