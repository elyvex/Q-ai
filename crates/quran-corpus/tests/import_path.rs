//! D-13(c) / QC-06: the importer has no code path to canonical tables.
//!
//! This is a source audit, not a runtime probe: the importer never holds an
//! `ApprovalToken`, never constructs a `CanonicalWriter`, never calls an
//! activation/rollback mutator, and never issues a canonical-table INSERT.
//! Its writes are staging rows, run state, and validation/difference reports,
//! and its terminal run state is `Staged`.
//!
//! Comments and doc-comments are stripped before scanning so a prose mention
//! (`import.rs` documents that it holds no token) is not mistaken for a call.

/// The importer source with comment-only lines removed.
fn importer_without_comments() -> String {
    include_str!("../src/import.rs")
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn importer_holds_no_approval_token_or_canonical_writer() {
    let src = importer_without_comments();
    for forbidden in ["ApprovalToken", "CanonicalWriter"] {
        assert!(
            !src.contains(forbidden),
            "the importer must not reference {forbidden}; it stages only"
        );
    }
}

#[test]
fn importer_never_calls_an_activation_or_rollback_mutator() {
    let src = importer_without_comments();
    for forbidden in [
        "activate_edition",
        "rollback_edition",
        "set_edition_status",
        "set_edition_verification",
    ] {
        assert!(!src.contains(forbidden), "the importer must never call {forbidden}");
    }
}

#[test]
fn importer_never_inserts_into_a_canonical_table() {
    let src = importer_without_comments();
    for forbidden in [
        "fn insert_edition(",
        "fn insert_surah(",
        "fn insert_ayah(",
        "fn insert_token(",
        "fn insert_separator(",
        "fn insert_division(",
        "fn write_canonical(",
        "fn insert_canonical(",
        "INSERT INTO quran_editions",
        "INSERT INTO quran_surahs",
        "INSERT INTO quran_ayahs",
        "INSERT INTO quran_tokens",
        "INSERT INTO quran_token_separators",
        "INSERT INTO quran_segments",
        "INSERT INTO quran_divisions",
        "INSERT INTO translation_editions",
        "INSERT INTO translation_passages",
        "INSERT INTO word_glosses",
    ] {
        assert!(!src.contains(forbidden), "forbidden canonical write vocabulary: {forbidden}");
    }
}

#[test]
fn importer_only_uses_staging_writes_and_ends_staged() {
    let src = importer_without_comments();
    // Positive control: the audit scans real importer code, which does write
    // staging rows through the `insert_stg_*` surface.
    assert!(
        src.contains("insert_stg_edition("),
        "the importer should write staging rows through insert_stg_*"
    );
    assert!(
        src.contains("set_import_run_state(&run_id, \"Staged\")"),
        "the importer's terminal run state must be Staged"
    );
    assert!(
        !src.contains("\"Active\""),
        "the importer must never mark a run Active; activation is a separate transaction"
    );
}
