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
        /// Attach word glosses (optional attributed dataset).
        #[arg(long, default_value_t = false)]
        glosses: bool,
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
    /// Parse an upstream `editions.json` catalog into edition metadata (no
    /// database, no text import; licences stay unknown).
    Catalog {
        /// Path to the `editions.json` file.
        catalog: String,
        /// Upstream commit/tag the file was read at (unpinned when absent).
        #[arg(long)]
        revision: Option<String>,
        /// Write the full JSON report here.
        #[arg(long)]
        report: Option<String>,
        /// Directory holding `chapterverse/` + `linebyline/` text mirrors to
        /// describe per edition (see `fixtures/upstream/README.md`).
        #[arg(long)]
        database: Option<String>,
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
    /// Word-gloss dataset management (optional attributed path).
    Gloss {
        #[command(subcommand)]
        action: GlossAction,
    },
    /// Derived-form management (search indexes build on these).
    Forms {
        #[command(subcommand)]
        action: FormsAction,
    },
    /// Search-index management.
    Index {
        #[command(subcommand)]
        action: IndexAction,
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
    /// Search the canonical corpus (exact, normalized, phrase, concatenated, regex).
    Search {
        /// Search options (boxed: the full flag set dwarfs other variants).
        #[command(flatten)]
        args: Box<QuranSearchArgs>,
    },
    /// Exact counting & discovery tools (read-only; rule-relative).
    Count {
        #[command(subcommand)]
        action: CountAction,
    },
    /// Morphology datasets: import (staging) and activate (approval-gated).
    Morphology {
        #[command(subcommand)]
        action: MorphologyAction,
    },
    /// Knowledge-graph operations (read-only inspection + build).
    Graph {
        #[command(subcommand)]
        action: GraphAction,
    },
}

/// `qai quran search` flags (P2-T52): all five lexical tools plus filters.
#[derive(clap::Args, Debug, Clone)]
pub struct QuranSearchArgs {
    /// Query text (or regex pattern with `--regex`).
    pub text: String,
    /// `quran.search_exact`: no linguistic expansion beyond the field profile.
    #[arg(long, default_value_t = false)]
    pub exact: bool,
    /// `quran.search_phrase`: ordered/near/unordered phrase search.
    #[arg(long, default_value_t = false)]
    pub phrase: bool,
    /// `quran.search_concatenated`: space-insensitive skeleton search.
    #[arg(long, default_value_t = false)]
    pub concatenated: bool,
    /// `quran.search_regex`: DFA-bounded regex over an indexed field.
    #[arg(long, default_value_t = false)]
    pub regex: bool,
    /// Edition `slug@version` (defaults to the indexed edition).
    #[arg(long)]
    pub edition: Option<String>,
    /// Exact-search field (`text_exact`|`text_ws`); regex field with `--regex`.
    #[arg(long)]
    pub field: Option<String>,
    /// Token match mode (`whole_token`|`substring`|`ayah_prefix`).
    #[arg(long, default_value = "whole_token")]
    pub match_mode: String,
    /// Registry profile (`L3.diacritics`, optionally `@version`-pinned).
    #[arg(long)]
    pub profile: Option<String>,
    /// Explicit rule list (`N01,N03`); never with `--profile`.
    #[arg(long)]
    pub rules: Option<String>,
    /// Phrase mode (`ordered_exact`|`ordered_near`|`unordered_near`).
    #[arg(long, default_value = "ordered_exact")]
    pub phrase_mode: String,
    /// Max intervening tokens for `*_near` phrase modes.
    #[arg(long, default_value_t = 0)]
    pub slop: u32,
    /// Allow 3-ayah window matches (concatenated only).
    #[arg(long, default_value_t = false)]
    pub cross_ayah: bool,
    /// Max ayahs per window match (concatenated only).
    #[arg(long, default_value_t = 3)]
    pub max_ayah_span: u32,
    /// Surah filter (`1,2,3`).
    #[arg(long)]
    pub surah: Option<String>,
    /// Juz filter (`2` or `1-5`).
    #[arg(long)]
    pub juz: Option<String>,
    /// Page filter (`3,4`).
    #[arg(long)]
    pub page: Option<String>,
    /// Revelation-place filter (`makki`|`madani`).
    #[arg(long)]
    pub revelation_place: Option<String>,
    /// Global ayah-index filter (`10-99`).
    #[arg(long)]
    pub global_range: Option<String>,
    /// Result cap (ceiling 1000).
    #[arg(long, default_value_t = 20)]
    pub limit: u32,
    /// Result offset.
    #[arg(long, default_value_t = 0)]
    pub offset: u32,
    /// Relevance order with per-hit BM25 breakdowns.
    #[arg(long, default_value_t = false)]
    pub explain: bool,
    /// Wrap hit spans in `<b>` display markers.
    #[arg(long, default_value_t = false)]
    pub highlight: bool,
    /// Regex wall-clock budget in ms (regex only, ceiling 10000).
    #[arg(long, default_value_t = 3000)]
    pub timeout_ms: u64,
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
    /// Record editorial verification (`verified_by`) under a human approval
    /// (P1-T55; the reviewer identity itself is an owner act, OD-02).
    Verify {
        /// `slug@version`.
        edition: String,
        /// Reviewer name (recorded in `verified_by`; never invented).
        #[arg(long)]
        reviewer: String,
        /// Comparison method (recorded in `verification_method`).
        #[arg(long)]
        method: String,
    },
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

/// Word-gloss subcommands.
#[derive(Subcommand)]
pub enum GlossAction {
    /// Import a word-gloss manifest.
    Import {
        /// Manifest path.
        manifest: String,
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

/// Index subcommands.
#[derive(Subcommand)]
pub enum IndexAction {
    /// Build an index generation and atomically activate it.
    Rebuild {
        /// Index id (default `quran.ayah.v1`).
        #[arg(long)]
        index: Option<String>,
        /// Edition `slug@version` (default: active edition).
        #[arg(long)]
        edition: Option<String>,
    },
    /// Verify the serving generation of an index.
    Verify {
        /// Index id (default `quran.ayah.v1`).
        #[arg(long)]
        index: Option<String>,
    },
    /// Enforce generation retention (default: keep active + previous).
    Gc {
        /// Index id (default `quran.ayah.v1`).
        #[arg(long)]
        index: Option<String>,
        /// Generations to retain, newest-first including active (default 2).
        #[arg(long, default_value_t = 2)]
        keep: usize,
    },
}

/// Exact counting & discovery subcommands (P2-T94…T103).
#[derive(Subcommand)]
pub enum CountAction {
    /// Exact frequency of a target under a profile (rules block included).
    Frequency {
        /// Target text to count.
        target: String,
        /// Counting profile (`L3.diacritics` default; L2/L4/L5/L7).
        #[arg(long, default_value = "L3.diacritics")]
        profile: String,
    },
    /// Frequency partitioned by surah with partition provenance.
    Distribution {
        /// Target text.
        target: String,
        /// Counting profile.
        #[arg(long, default_value = "L3.diacritics")]
        profile: String,
    },
    /// Per-surah first/last occurrence + interval (disclaimer included).
    Occurrences {
        /// Target text.
        target: String,
        /// Counting profile.
        #[arg(long, default_value = "L3.diacritics")]
        profile: String,
    },
    /// Hapax legomena under a profile (rule-relative, profile stated).
    Hapax {
        /// Counting profile.
        #[arg(long, default_value = "L3.diacritics")]
        profile: String,
        /// Result cap.
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    /// Co-occurrence within a token window (cross-ayah flags included).
    Cooccurrence {
        /// Target text.
        target: String,
        /// Counting profile.
        #[arg(long, default_value = "L3.diacritics")]
        profile: String,
        /// Window radius in tokens.
        #[arg(long, default_value_t = 3)]
        window: usize,
        /// Result cap.
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
    /// Association measures (PMI/LLR/t-score) with a min-count floor.
    Collocation {
        /// Target text.
        target: String,
        /// Counting profile.
        #[arg(long, default_value = "L3.diacritics")]
        profile: String,
        /// Window radius in tokens.
        #[arg(long, default_value_t = 3)]
        window: usize,
        /// Result cap.
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
    /// Numeric report (checksum + no-interpretation note).
    NumericReport {
        /// Target text.
        target: String,
        /// Counting profile.
        #[arg(long, default_value = "L3.diacritics")]
        profile: String,
    },
    /// Prove a zero count under stated rules (disclaimer included).
    MissingForm {
        /// Target text expected to be absent.
        target: String,
        /// Counting profile.
        #[arg(long, default_value = "L3.diacritics")]
        profile: String,
    },
    /// Near-duplicate passages (MinHash candidates + exact verify).
    NearDuplicates {
        /// Jaccard threshold in [0,1].
        #[arg(long, default_value_t = 0.8)]
        threshold: f64,
        /// Result cap.
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
}

/// Morphology dataset subcommands (import/activate separation).
#[derive(Subcommand)]
pub enum MorphologyAction {
    /// Import an intermediate document into staging (never activates).
    Import {
        /// Path to the intermediate JSON/CSV document.
        #[arg(long)]
        file: String,
        /// Dataset slug.
        #[arg(long)]
        dataset: String,
        /// Dataset version.
        #[arg(long)]
        version: String,
        /// Adapter (`json`|`csv`).
        #[arg(long, default_value = "json")]
        adapter: String,
        /// Edition `slug@version` to align against (must be active).
        #[arg(long)]
        edition: String,
        /// Attribution string recorded on the dataset.
        #[arg(long, default_value = "")]
        attribution: String,
        /// Resume/replace a specific batch id.
        #[arg(long)]
        batch: Option<String>,
    },
    /// Activate a staged batch (approval-gated; cannot run in the importer).
    Activate {
        /// Staged batch id.
        #[arg(long)]
        batch: String,
        /// Granted approval id covering the dataset URN.
        #[arg(long)]
        approval: String,
    },
    /// List registered datasets and their states.
    Datasets,
    /// Show all analyses of one token with attribution.
    Token {
        /// Edition `slug@version`.
        #[arg(long)]
        edition: String,
        /// Surah number.
        #[arg(long)]
        surah: i64,
        /// Ayah number.
        #[arg(long)]
        ayah: i64,
        /// 1-based token position.
        #[arg(long, default_value_t = 1)]
        position: i64,
    },
    /// Compare competing analyses of one token (verdicts, no winner).
    Compare {
        /// Edition `slug@version`.
        #[arg(long)]
        edition: String,
        /// Surah number.
        #[arg(long)]
        surah: i64,
        /// Ayah number.
        #[arg(long)]
        ayah: i64,
        /// 1-based token position.
        #[arg(long, default_value_t = 1)]
        position: i64,
    },
    /// Root search (grouped occurrences with dataset attribution).
    Root {
        /// Normalized root spelling.
        root: String,
    },
    /// Lemma search (grouped occurrences).
    Lemma {
        /// Lemma spelling.
        lemma: String,
    },
    /// Affix search: dataset backend when active, else the labeled L7 path.
    Affix {
        /// Affix text.
        affix: String,
        /// Profile (`L7.affix` selects the heuristic backend).
        #[arg(long, default_value = "L7.affix")]
        profile: String,
    },
}

/// Knowledge-graph subcommands (build/inspect/root-family/export).
#[derive(Subcommand)]
pub enum GraphAction {
    /// Build the structural projection from the active edition (in-memory).
    Build {
        /// Write the built projection as JSON to this path (optional).
        #[arg(long)]
        out: Option<String>,
    },
    /// Inspect the projection manifest and node/edge counts from a file.
    Inspect {
        /// Projection JSON written by `graph build --out`.
        #[arg(long)]
        file: String,
    },
    /// Bounded neighbors of a node (budgeted, explicit truncation).
    Neighbors {
        /// Projection JSON file.
        #[arg(long)]
        file: String,
        /// Stable node id (e.g. `ayah:1:1`).
        #[arg(long)]
        node: String,
        /// Max hops from the node.
        #[arg(long, default_value_t = 1)]
        hops: usize,
    },
    /// Bounded path search between two nodes (budgeted).
    Path {
        /// Projection JSON file.
        #[arg(long)]
        file: String,
        /// Source stable id.
        #[arg(long)]
        from: String,
        /// Target stable id.
        #[arg(long)]
        to: String,
        /// Max hops.
        #[arg(long, default_value_t = 4)]
        hops: usize,
    },
    /// Root-family ranked ayahs (lexicon-gated; needs an active dataset).
    RootFamily {
        /// Normalized root spelling.
        root: String,
        /// Result cap.
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
    /// Export the projection as Graph JSON (identities, versions, truncation).
    Export {
        /// Projection JSON file.
        #[arg(long)]
        file: String,
        /// Output path (defaults to stdout).
        #[arg(long)]
        out: Option<String>,
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
        QuranAction::Get { reference, translations, tokens, glosses } => {
            application::quran_cli::cmd_get(
                db_path,
                &reference,
                translations.as_deref(),
                tokens,
                glosses,
            )
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
            EditionAction::Verify { edition, reviewer, method } => {
                confirm(
                    "verify",
                    &edition,
                    yes,
                    application::quran_cli::cmd_edition_verify(
                        db_path, &edition, &reviewer, &method,
                    ),
                )
                .await
            }
        },
        QuranAction::Import { manifest, adapter, dry_run } => {
            application::quran_cli::cmd_import(db_path, &manifest, &adapter, dry_run).await
        }
        QuranAction::Validate { target, report } => {
            application::quran_cli::cmd_validate(db_path, &target, report.as_deref()).await
        }
        QuranAction::Catalog { catalog, revision, report, database } => {
            application::quran_cli::cmd_catalog(
                &catalog,
                revision.as_deref(),
                report.as_deref(),
                database.as_deref(),
            )
            .await
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
        QuranAction::Gloss { action } => match action {
            GlossAction::Import { manifest } => {
                application::quran_cli::cmd_gloss_import(db_path, &manifest).await
            }
        },
        QuranAction::Forms { action } => match action {
            FormsAction::Rebuild { edition } => {
                application::quran_cli::cmd_forms_rebuild(db_path, &edition).await
            }
        },
        QuranAction::Index { action } => match action {
            IndexAction::Rebuild { index, edition } => {
                application::quran_cli::cmd_index_rebuild(
                    db_path,
                    index.as_deref(),
                    edition.as_deref(),
                )
                .await
            }
            IndexAction::Verify { index } => {
                application::quran_cli::cmd_index_verify(db_path, index.as_deref()).await
            }
            IndexAction::Gc { index, keep } => {
                application::quran_cli::cmd_index_gc(db_path, index.as_deref(), keep).await
            }
        },
        QuranAction::Count { action } => match action {
            CountAction::Frequency { target, profile } => {
                application::quran_cli::cmd_count_frequency(db_path, &target, &profile).await
            }
            CountAction::Distribution { target, profile } => {
                application::quran_cli::cmd_count_distribution(db_path, &target, &profile).await
            }
            CountAction::Occurrences { target, profile } => {
                application::quran_cli::cmd_count_occurrences(db_path, &target, &profile).await
            }
            CountAction::Hapax { profile, limit } => {
                application::quran_cli::cmd_count_hapax(db_path, &profile, *limit).await
            }
            CountAction::Cooccurrence { target, profile, window, limit } => {
                application::quran_cli::cmd_count_cooccurrence(
                    db_path, &target, &profile, *window, *limit,
                )
                .await
            }
            CountAction::Collocation { target, profile, window, limit } => {
                application::quran_cli::cmd_count_collocation(
                    db_path, &target, &profile, *window, *limit,
                )
                .await
            }
            CountAction::NumericReport { target, profile } => {
                application::quran_cli::cmd_count_numeric_report(db_path, &target, &profile).await
            }
            CountAction::MissingForm { target, profile } => {
                application::quran_cli::cmd_count_missing_form(db_path, &target, &profile).await
            }
            CountAction::NearDuplicates { threshold, limit } => {
                application::quran_cli::cmd_count_near_duplicates(db_path, *threshold, *limit).await
            }
        },
        QuranAction::Morphology { action } => match action {
            MorphologyAction::Import { file, dataset, version, adapter, edition, attribution, batch } => {
                application::quran_cli::cmd_morphology_import(
                    db_path,
                    &file,
                    &dataset,
                    &version,
                    &adapter,
                    &edition,
                    &attribution,
                    batch.as_deref(),
                )
                .await
            }
            MorphologyAction::Activate { batch, approval } => {
                application::quran_cli::cmd_morphology_activate(db_path, &batch, &approval).await
            }
            MorphologyAction::Datasets => {
                application::quran_cli::cmd_morphology_datasets(db_path).await
            }
            MorphologyAction::Token { edition, surah, ayah, position } => {
                application::quran_cli::cmd_morphology_token(
                    db_path, &edition, *surah, *ayah, *position,
                )
                .await
            }
            MorphologyAction::Compare { edition, surah, ayah, position } => {
                application::quran_cli::cmd_morphology_compare(
                    db_path, &edition, *surah, *ayah, *position,
                )
                .await
            }
            MorphologyAction::Root { root } => {
                application::quran_cli::cmd_morphology_root(db_path, &root).await
            }
            MorphologyAction::Lemma { lemma } => {
                application::quran_cli::cmd_morphology_lemma(db_path, &lemma).await
            }
            MorphologyAction::Affix { affix, profile } => {
                application::quran_cli::cmd_morphology_affix(db_path, &affix, &profile).await
            }
        },
        QuranAction::Graph { action } => match action {
            GraphAction::Build { out } => {
                application::quran_cli::cmd_graph_build(db_path, out.as_deref()).await
            }
            GraphAction::Inspect { file } => {
                application::quran_cli::cmd_graph_inspect(&file).await
            }
            GraphAction::Neighbors { file, node, hops } => {
                application::quran_cli::cmd_graph_neighbors(&file, &node, *hops).await
            }
            GraphAction::Path { file, from, to, hops } => {
                application::quran_cli::cmd_graph_path(&file, &from, &to, *hops).await
            }
            GraphAction::RootFamily { root, limit } => {
                application::quran_cli::cmd_graph_root_family(db_path, &root, *limit).await
            }
            GraphAction::Export { file, out } => {
                application::quran_cli::cmd_graph_export(&file, out.as_deref()).await
            }
        },
        QuranAction::Search { args } => {
            let QuranSearchArgs {
                text,
                exact,
                phrase,
                concatenated,
                regex,
                edition,
                field,
                match_mode,
                profile,
                rules,
                phrase_mode,
                slop,
                cross_ayah,
                max_ayah_span,
                surah,
                juz,
                page,
                revelation_place,
                global_range,
                limit,
                offset,
                explain,
                highlight,
                timeout_ms,
            } = *args;
            let tools =
                [exact, phrase, concatenated, regex].iter().filter(|&&selected| selected).count();
            if tools > 1 {
                use crate::exit_code;
                eprintln!("error: use only one of --exact, --phrase, --concatenated, --regex");
                return exit_code::USAGE;
            }
            let mode = if regex {
                application::quran_cli::SearchCliMode::Regex
            } else if phrase {
                application::quran_cli::SearchCliMode::Phrase
            } else if concatenated {
                application::quran_cli::SearchCliMode::Concatenated
            } else if exact {
                application::quran_cli::SearchCliMode::Exact
            } else if phrase_mode != "ordered_exact" || slop != 0 {
                application::quran_cli::SearchCliMode::Phrase
            } else {
                application::quran_cli::SearchCliMode::Normalized
            };
            application::quran_cli::cmd_search(
                db_path,
                &application::quran_cli::SearchCliOptions {
                    text,
                    mode,
                    edition,
                    field,
                    match_mode,
                    profile,
                    rules,
                    phrase_mode,
                    slop,
                    allow_cross_ayah: cross_ayah,
                    max_ayah_span,
                    surah,
                    juz,
                    page,
                    revelation_place,
                    global_range,
                    limit,
                    offset,
                    explain,
                    highlight,
                    timeout_ms,
                },
            )
            .await
        }
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
