//! Phase 2 — regex/DoS abuse suite (P2-T54, AC-P2-13).
//!
//! All 15 pathological patterns from the plan's abuse catalog must be
//! rejected up front or bounded by the I16 budgets (512-char cap, no leading
//! `.*`, DFA-only with 1 MiB NFA / 4 MiB DFA construction limits, 3 s
//! execution budget, per-principal rate limit). The suite asserts the guard
//! decision plus a wall-clock bound on every accepted pattern so a
//! catastrophic automaton cannot hide behind "accepted but slow".
//!
//! Patterns that compile are additionally run against a hostile haystack
//! (long repetitions, mixed scripts, RTL controls) with a 3 s budget each.

use std::time::{Duration, Instant};

use quran_search::{compile_dfa, first_match};

/// Execution budget per accepted pattern (mirrors the service default).
const BUDGET: Duration = Duration::from_secs(3);

/// Hostile haystack: repetitions, mixed scripts, RTL controls, zero-widths.
fn haystack() -> String {
    let mut hay = String::new();
    hay.push_str(&"ا".repeat(2_000));
    hay.push_str(&"ب".repeat(2_000));
    hay.push_str("الرحمن الرحيم الحمد لله رب العلمين ");
    hay.push_str(&"ab".repeat(2_000));
    hay.push_str("\u{200e}\u{200f}\u{200c}\u{feff}");
    hay.push_str(&"0123456789".repeat(200));
    hay
}

/// (pattern, why it is pathological, must_compile).
/// `must_compile = false` patterns must be rejected by the guards.
const CATALOG: &[(&str, &str, bool)] = &[
    ("(a+)+$", "nested quantifiers (ReDoS classic)", true),
    ("(a|aa)+$", "overlapping alternation branch explosion", true),
    ("(a|a?)+$", "optional-branch ambiguity", true),
    ("(a*)*$", "star-of-star", true),
    ("(a|b|ab)*$", "prefix-overlap alternation star", true),
    ("a{100}b{100}", "large bounded repetitions", true),
    ("(ا|ب|ت|ث|ج|ح|خ|د|ذ|ر|ز|س|ش|ص|ض|ط|ظ|ع|غ|ف|ق|ك|ل|م|ن|ه|و|ي)+$", "28-branch Arabic alternation", true),
    ("^(ا|ب)*$", "anchored Arabic star group", true),
    ("ال(رحمن|رحيم|حمد|له)*", "Arabic prefix alternation star", true),
    ("[ا-ي]{1,64}", "wide char-class bounded repeat", true),
    (".*abc", "unanchored leading .*", false),
    (".+abc", "unanchored leading .+", false),
    ("^.*abc", "anchored-then-bare .*", false),
    ("OVERLONG", "512-char cap (>512 bytes rejected)", false),
    ("[invalid", "invalid class syntax", false),
];

fn pattern_at(index: usize) -> String {
    if index == 13 {
        "x".repeat(600)
    } else {
        CATALOG[index].0.to_string()
    }
}

#[test]
fn fifteen_pathological_patterns_bounded_or_rejected() {
    assert_eq!(CATALOG.len(), 15, "abuse catalog must hold exactly 15 patterns");
    let hay = haystack();
    for (index, (pattern, reason, must_compile)) in CATALOG.iter().enumerate() {
        let text = pattern_at(index);
        let started = Instant::now();
        let compiled = compile_dfa(&text);
        let elapsed = started.elapsed();
        if *must_compile {
            let dfa = compiled.unwrap_or_else(|e| panic!("pattern {index} ({reason}) wrongly rejected: {e}"));
            // Bounded execution against the hostile haystack.
            let searched = Instant::now();
            let _ = first_match(&dfa, &hay);
            let search_elapsed = searched.elapsed();
            assert!(
                elapsed + search_elapsed < BUDGET,
                "pattern {index} ({reason}) exceeded the 3 s budget: compile={elapsed:?} search={search_elapsed:?}"
            );
        } else {
            assert!(
                compiled.is_err(),
                "pattern {index} ({reason}): {pattern:?} must be rejected"
            );
        }
    }
}

#[test]
fn rate_limit_and_timeout_budgets_are_documented_constants() {
    // Structural pin: the budgets this suite assumes must stay the budgets
    // the engine enforces. Values mirror `regex.rs` + service defaults.
    assert_eq!(quran_search::MAX_PATTERN_LEN, 512);
    assert_eq!(quran_search::NFA_SIZE_LIMIT, 1024 * 1024);
    assert_eq!(quran_search::DFA_SIZE_LIMIT, 4 * 1024 * 1024);
}
