//! `qai quran …` — canonical corpus commands (D1.10, P1-T48/T49).
//!
//! Argument parsing and output formatting live here; every database access
//! goes through `application::quran_cli` free functions so this crate never
//! touches storage directly. Human output avoids volatile ids (snapshot
//! determinism); `--json` emits full structures.

use clap::Subcommand;

use crate::exit_code;

/// Quran corpus commands.
#[derive(Subcommand)]
pub enum QuranAction {
    /// Print one ayah with its canonical reference.
    Get {
        /// Reference (`2:255`, `quran:2:255-257`, …).
        reference: String,
        /// Translation slugs (comma-separated).
        #[arg(long)]
        translations: Option<String>,
        /// Attach surface tokens.
        #[arg(long, default_value_t = false)]
        tokens: bool,
    },
    /// Print context around a focal ayah.
    Context {
        /// Focal reference.
        reference: String,
        /// Ayahs before.
        #[arg(long, default_value_t = 3)]
        before: u16,
        /// Ayahs after.
        #[arg(long, default_value_t = 3)]
        after: u16,
        /// Structural boundary.
        #[arg(long, default_value = "surah")]
        boundary: String,
    },
    /// Print a whole surah.
    Surah {
        /// Surah number.
        number: u16,
        /// Only metadata, no ayahs.
        #[arg(long, default_value_t = false)]
        metadata: bool,
    },
    /// Print one division.
    Division {
        /// Division kind (`juz`, `hizb`, `rub`, `manzil`, `page`, `ruku`, `sajdah`).
        kind: String,
        /// Division number.
        number: u32,
    },
    /// Parse and bounds-check a reference.
    Resolve {
        /// Reference string.
        reference: String,
    },
    /// Edition management.
    Edition {
        #[command(subcommand)]
        action: EditionAction,
    },
    /// Import an edition manifest to `Staged` (runs the `quran.import` job inline).
    Import {
        /// Manifest path.
        manifest: String,
        /// Adapter (`json`).
        #[arg(long, default_value = "json")]
        adapter: String,
        /// Validate only; write nothing.
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
    /// Validate a manifest file or a staged `slug@version`.
    Validate {
        /// Manifest path or `slug@version`.
        target: String,
        /// Write the report JSON here.
        #[arg(long)]
        report: Option<String>,
    },
    /// Show the difference between two versions.
    Diff {
        /// Edition slug.
        edition: String,
        /// From version.
        #[arg(long)]
        from: String,
        /// To version.
        #[arg(long)]
        to: String,
        /// Output format (`text`, `json`, `unified`).
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Activate a staged edition (human approval required).
    Activate {
        /// `slug@version`.
        edition: String,
    },
    /// Roll back to a prior edition version (human approval required).
    Rollback {
        /// Edition slug.
        edition: String,
        /// Target version.
        #[arg(long)]
        to: String,
    },
    /// Deprecate a canonical edition version (human approval required).
    Deprecate {
        /// `slug@version`.
        edition: String,
    },
    /// Recompute and compare stored hashes.
    Hashes {
        /// `slug@version`.
        edition: String,
    },
    /// Translation management.
    Translation {
        #[command(subcommand)]
        action: TranslationAction,
    },
    /// Derived-form management (search indexes build on these).
    Forms {
        #[command(subcommand)]
        action: FormsAction,
    },
    /// Normalize text through a profile or adhoc rule list (no canonical reads).
    Normalize {
        /// Text to normalize.
        text: Option<String>,
        /// Profile (`L3.diacritics`, optionally `@version`-pinned).
        #[arg(long)]
        profile: Option<String>,
        /// Explicit rule list (`N01,N03,N06`); never with `--profile`.
        #[arg(long)]
        rules: Option<String>,
        /// Show the rule-by-rule transformation with offset notes.
        #[arg(long, default_value_t = false)]
        explain: bool,
        /// List seeded profiles and exit.
        #[arg(long, default_value_t = false)]
        list_profiles: bool,
        /// Show one rule (`N06`) and exit.
        #[arg(long)]
        show_rule: Option<String>,
    },
}

/// Edition subcommands.
#[derive(Subcommand)]
pub enum EditionAction {
    /// List editions.
    List,
    /// Show one edition with statistics and hashes.
    Show {
        /// Slug, or `slug@version`.
        edition: String,
        /// Show statistics.
        #[arg(long, default_value_t = false)]
        statistics: bool,
        /// Show hashes.
        #[arg(long, default_value_t = false)]
        hashes: bool,
    },
    /// Print the active edition pointer.
    Active,
}

/// Translation subcommands.
#[derive(Subcommand)]
pub enum TranslationAction {
    /// List translation editions.
    List,
    /// Import a translation manifest.
    Import {
        /// Manifest path.
        manifest: String,
    },
    /// Show one translation edition.
    Show {
        /// Slug.
        slug: String,
    },
}

/// Derived-form subcommands.
#[derive(Subcommand)]
pub enum FormsAction {
    /// Rebuild derived token/ayah forms and skeletons for an edition
    /// (MV-018 verified before and after; canonical text untouched).
    Rebuild {
        /// `slug@version` (must be the active edition).
        edition: String,
    },
}

/// Dispatch a Quran command. `db_path` selects the SQLite file.
pub fn handle_quran(action: QuranAction, db_path: &str, json: bool, yes: bool) -> i32 {
    let runtime = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(err) => {
            eprintln!("failed to start async runtime: {err}");
            return exit_code::INTERNAL;
        }
    };
    runtime.block_on(handle_quran_async(action, db_path, json, yes))
}

async fn handle_quran_async(action: QuranAction, db_path: &str, json: bool, yes: bool) -> i32 {
    let output = match action {
        QuranAction::Get { reference, translations, tokens } => {
            application::quran_cli::cmd_get(db_path, &reference, translations.as_deref(), tokens)
                .await
        }
        QuranAction::Context { reference, before, after, boundary } => {
            application::quran_cli::cmd_context(db_path, &reference, before, after, &boundary).await
        }
        QuranAction::Surah { number, metadata } => {
            application::quran_cli::cmd_surah(db_path, number, metadata).await
        }
        QuranAction::Division { kind, number } => {
            application::quran_cli::cmd_division(db_path, &kind, number).await
        }
        QuranAction::Resolve { reference } => {
            application::quran_cli::cmd_resolve(db_path, &reference).await
        }
        QuranAction::Edition { action } => match action {
            EditionAction::List => application::quran_cli::cmd_edition_list(db_path).await,
            EditionAction::Show { edition, statistics, hashes } => {
                application::quran_cli::cmd_edition_show(db_path, &edition, statistics, hashes)
                    .await
            }
            EditionAction::Active => application::quran_cli::cmd_edition_active(db_path).await,
        },
        QuranAction::Import { manifest, adapter, dry_run } => {
            application::quran_cli::cmd_import(db_path, &manifest, &adapter, dry_run).await
        }
        QuranAction::Validate { target, report } => {
            application::quran_cli::cmd_validate(db_path, &target, report.as_deref()).await
        }
        QuranAction::Diff { edition, from, to, format } => {
            application::quran_cli::cmd_diff(db_path, &edition, &from, &to, &format).await
        }
        QuranAction::Activate { edition } => {
            confirm(
                "activate",
                &edition,
                yes,
                application::quran_cli::cmd_activate(db_path, &edition),
            )
            .await
        }
        QuranAction::Rollback { edition, to } => {
            confirm(
                &format!("rollback {edition}"),
                &to,
                yes,
                application::quran_cli::cmd_rollback(db_path, &edition, &to),
            )
            .await
        }
        QuranAction::Deprecate { edition } => {
            confirm(
                "deprecate",
                &edition,
                yes,
                application::quran_cli::cmd_deprecate(db_path, &edition),
            )
            .await
        }
        QuranAction::Hashes { edition } => {
            application::quran_cli::cmd_hashes(db_path, &edition).await
        }
        QuranAction::Translation { action } => match action {
            TranslationAction::List => application::quran_cli::cmd_translation_list(db_path).await,
            TranslationAction::Import { manifest } => {
                application::quran_cli::cmd_translation_import(db_path, &manifest).await
            }
            TranslationAction::Show { slug } => {
                application::quran_cli::cmd_translation_show(db_path, &slug).await
            }
        },
        QuranAction::Forms { action } => match action {
            FormsAction::Rebuild { edition } => {
                application::quran_cli::cmd_forms_rebuild(db_path, &edition).await
            }
        },
        QuranAction::Normalize { text, profile, rules, explain, list_profiles, show_rule } => {
            if list_profiles {
                application::quran_cli::cmd_normalize_list_profiles(db_path).await
            } else if let Some(rule) = show_rule {
                application::quran_cli::cmd_normalize_show_rule(db_path, &rule).await
            } else {
                application::quran_cli::cmd_normalize(
                    db_path,
                    text.as_deref(),
                    profile.as_deref(),
                    rules.as_deref(),
                    explain,
                )
                .await
            }
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&output.json).unwrap_or_default());
    } else {
        // Human output ends with exactly one newline: command handlers may
        // append one themselves, so strip a single trailing newline before the
        // printer adds it back.
        let human = output.human.strip_suffix('\n').unwrap_or(&output.human);
        println!("{human}");
    }
    output.exit
}

async fn confirm(
    verb: &str,
    target: &str,
    yes: bool,
    run: impl std::future::Future<Output = application::quran_cli::CommandOutput>,
) -> application::quran_cli::CommandOutput {
    use std::io::IsTerminal;
    if !yes {
        if !std::io::stdin().is_terminal() {
            eprintln!("refusing to {verb} `{target}` without --yes on a non-terminal");
            return application::quran_cli::CommandOutput::err(
                exit_code::POLICY,
                format!("refusing to {verb} without --yes"),
            );
        }
        eprintln!("Type APPROVE to {verb} `{target}`: ");
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).is_err() || line.trim() != "APPROVE" {
            return application::quran_cli::CommandOutput::err(
                exit_code::CANCELLED,
                "not approved".to_string(),
            );
        }
    }
    run.await
}
