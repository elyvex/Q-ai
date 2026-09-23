//! Mechanical generator for the normalization golden set (`fixtures/quran/normalization/pairs.jsonl`).
//!
//! Run: `cargo run -p quran-normalization --example gen_goldens`
//!
//! The generator feeds deterministic input combinations through the real
//! [`NormalizationPipeline`] and records input → output per profile. The
//! output is a **regression lock**, not a linguist-reviewed oracle: the
//! header records `reviewed_by: pending-linguist`, and P2-T11 (signed
//! 2,000-pair set, ADR-0204) stays open until a qualified linguist reviews
//! the expectations. Independent spec coverage lives in the hand-written
//! rule mapping-table tests (`src/rules/n*.rs`) and `tests/deterministic_rules.rs`.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use quran_normalization::pipeline::NormalizationPipeline;
use quran_normalization::profile::{ProfileId, ProfileRegistry};
use quran_normalization::rule::RuleId;

const PROFILES: [ProfileId; 6] =
    [ProfileId::L0, ProfileId::L1, ProfileId::L2, ProfileId::L3, ProfileId::L4, ProfileId::L5];

/// Base words exercising every rule family.
const BASES: &[&str] = &[
    "الحمد",
    "بِسْمِ",
    "ٱللَّهِ",
    "ٱلرَّحْمَـٰنِ",
    "أإآؤئء",
    "مؤمن",
    "کیه",
    "الله",
    "صلوة",
    "آية ١٢٣",
    "قال: ربنا!",
    "ب\u{200c}س\u{200d}م",
    "ﷲ",
    "طه",
    "والكتابه",
    "للبيت",
    "الكتاب",
];

/// Decorations combined with bases to build the input space.
const DECORATIONS: &[&str] = &["", "ِ", "ٌ", "ـ", "  ", "،", "\u{200e}", "۝", "  ٱلْحَمْدُ  "];

fn build_inputs() -> Vec<String> {
    let mut set = BTreeSet::new();
    for base in BASES {
        for deco in DECORATIONS {
            set.insert(format!("{base}{deco}"));
            set.insert(format!("{deco}{base}"));
        }
    }
    // Multi-word joins (phrase shapes) from ordered base pairs.
    for pair in BASES.windows(2) {
        set.insert(pair.join(" "));
        set.insert(pair.join("  "));
    }
    // Longer compositions to reach the target count deterministically.
    let bases_joined = BASES.join(" ");
    set.insert(bases_joined.clone());
    for (i, base) in BASES.iter().enumerate() {
        set.insert(format!("{bases_joined} {base} {i}"));
    }
    set.into_iter().collect()
}

fn main() {
    let registry = ProfileRegistry::new();
    let v = quran_normalization::SemVer::new(1, 0, 0);
    let inputs = build_inputs();
    let per_profile = 2000_usize / PROFILES.len();
    let mut rows: Vec<String> = Vec::with_capacity(2001);
    let header = serde_json::json!({
        "header": true,
        "version": 1,
        "generated_by": "crates/quran-normalization/examples/gen_goldens.rs",
        "generated_at": "2026-09-23",
        "profiles": PROFILES.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
        "profile_version": "1.0.0",
        "reviewed_by": "pending-linguist",
        "reviewed_at": serde_json::Value::Null,
        "note": "Mechanical regression lock from the v1 pipeline; NOT linguist-signed. P2-T11 stays open until linguist review (P2-X02).",
    });
    rows.push(header.to_string());

    let mut count = 0;
    'outer: for cycle in 0.. {
        for input in &inputs {
            // Cycle suffixes keep inputs distinct across cycles while
            // staying deterministic (no RNG seed to manage).
            let text = if cycle == 0 { input.clone() } else { format!("{input} {cycle}") };
            for profile in PROFILES {
                let pipe =
                    NormalizationPipeline::for_profile(&registry, profile, v).expect("builtin");
                let (out, trace) = pipe.apply(&text);
                let rules: Vec<&str> =
                    trace.rules_applied.iter().map(|r| r.rule.as_str()).collect();
                let mut profile_name = String::new();
                write!(profile_name, "{profile}").unwrap();
                let row = serde_json::json!({
                    "input": text,
                    "profile": profile_name,
                    "profile_version": "1.0.0",
                    "expected_output": out.text(),
                    "expected_rules_applied": rules,
                });
                rows.push(row.to_string());
                count += 1;
                if count >= 2000 {
                    break 'outer;
                }
            }
        }
    }
    assert_eq!(count, 2000, "golden set must hold exactly 2,000 pairs");

    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/quran/normalization/pairs.jsonl");
    std::fs::create_dir_all(fixtures.parent().unwrap()).unwrap();
    std::fs::write(&fixtures, rows.join("\n") + "\n").unwrap();
    eprintln!("wrote {} rows to {}", rows.len(), fixtures.display());

    // Sanity: every profile contributed equally; every RuleId parses.
    for line in rows.iter().skip(1) {
        let row: serde_json::Value = serde_json::from_str(line).unwrap();
        for rule in row["expected_rules_applied"].as_array().unwrap() {
            RuleId::parse(rule.as_str().unwrap()).unwrap();
        }
    }
}
