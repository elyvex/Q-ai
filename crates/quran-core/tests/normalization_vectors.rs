//! A7 — normalization test-vector files: shape gate (OD-12).
//!
//! Vector files live at `tests/vectors/normalization/<RULE>-v<VERSION>.json`
//! with rows `{input_codepoints, expected_output_codepoints, note}` where the
//! codepoint arrays hold `"U+XXXX"` strings. This suite validates the *shape*
//! only (valid ids, ≥20 vectors per rule, well-formed codepoints, non-empty
//! notes) without depending on any normalization implementation — `quran-core`
//! must stay free of normalization edges (I2). Rule *application* of these
//! vectors is tested in `quran-normalization` (`tests/normalization_vectors.rs`).
//!
//! CI fails if a rule ships without its vector file: stabilizing a rule means
//! adding its file here (see `VECTOR_COVERED` in the `quran-normalization`
//! suite).

use std::path::PathBuf;

const VECTORS_DIR: &str = "tests/vectors/normalization";
const MIN_VECTORS_PER_RULE: usize = 20;

fn vectors_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(VECTORS_DIR)
}

fn parse_codepoint(text: &str) -> Option<char> {
    let hex = text.strip_prefix("U+")?;
    let value = u32::from_str_radix(hex, 16).ok()?;
    if !(0..=0x10FFFF).contains(&value) {
        return None;
    }
    char::from_u32(value)
}

fn check_vector_file(path: &std::path::Path) {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_else(|| panic!("non-UTF8 vector filename: {}", path.display()));
    let (rule, version) = stem
        .split_once("-v")
        .unwrap_or_else(|| panic!("vector file must be <RULE>-v<VERSION>.json: {stem}"));
    assert!(
        rule.len() == 3 && rule.starts_with('N') && rule[1..].chars().all(|c| c.is_ascii_digit()),
        "rule id must look like N02, got: {rule}"
    );
    assert!(
        version.split('.').count() >= 2
            && version.split('.').all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())),
        "version must look like 1 (major.minor…), got: {version}"
    );
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|_| panic!("cannot read {}", path.display()));
    let rows: Vec<serde_json::Value> = serde_json::from_str(&text)
        .unwrap_or_else(|_| panic!("{} is not a JSON array", path.display()));
    assert!(
        rows.len() >= MIN_VECTORS_PER_RULE,
        "{stem}: {} vectors, need ≥ {MIN_VECTORS_PER_RULE}",
        rows.len()
    );
    for (index, row) in rows.iter().enumerate() {
        for key in ["input_codepoints", "expected_output_codepoints"] {
            let codes = row
                .get(key)
                .and_then(|v| v.as_array())
                .unwrap_or_else(|| panic!("{stem}[{index}]: missing array `{key}`"));
            for code in codes {
                let code = code
                    .as_str()
                    .unwrap_or_else(|| panic!("{stem}[{index}]: non-string codepoint in `{key}`"));
                assert!(
                    parse_codepoint(code).is_some(),
                    "{stem}[{index}]: malformed codepoint `{code}` in `{key}`"
                );
            }
        }
        let note = row
            .get("note")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("{stem}[{index}]: missing string `note`"));
        assert!(!note.trim().is_empty(), "{stem}[{index}]: empty `note`");
    }
}

#[test]
fn normalization_vector_files_are_well_formed() {
    let dir = vectors_dir();
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|_| panic!("missing vectors dir: {}", dir.display()))
        .map(|entry| entry.expect("read vectors dir").path())
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    assert!(!files.is_empty(), "no normalization vector files in {}", dir.display());
    files.sort();
    for path in &files {
        check_vector_file(path);
    }
}
