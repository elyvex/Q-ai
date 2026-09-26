//! Q-ai CLI crate.
//!
//! Exposes `run()` for the `qai` binary and a clap-based command tree
//! covering the Phase 0 surface: version, config, db, doctor, serve, and
//! Phase-N stub commands.

pub mod doctor;
pub mod exit_code;
pub mod quran;

use clap::{Parser, Subcommand};
use config::{Config, OriginMap, ValueOrigin};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Parser)]
#[command(name = "qai", about = "Q-ai research platform CLI", version)]
pub struct Cli {
    /// Path to the configuration file.
    #[arg(long, global = true)]
    pub config: Option<std::path::PathBuf>,
    /// Data directory override.
    #[arg(long, global = true)]
    pub data_dir: Option<String>,
    /// Log level override.
    #[arg(long, global = true)]
    pub log_level: Option<String>,
    /// Output machine-readable JSON.
    #[arg(long, global = true)]
    pub json: bool,
    /// Suppress non-essential output.
    #[arg(long, global = true)]
    pub quiet: bool,
    /// Assume yes for destructive prompts.
    #[arg(long, global = true)]
    pub yes: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Short status summary.
    Status,
    /// Print the version.
    Version,
    /// Run diagnostic checks (read-only).
    Doctor {
        /// Output as JSON.
        #[arg(long)]
        json: bool,
        /// Only core checks.
        #[arg(long)]
        core: bool,
        /// Only source checks.
        #[arg(long)]
        sources: bool,
        /// Only index checks.
        #[arg(long)]
        indexes: bool,
        /// Only model checks.
        #[arg(long)]
        models: bool,
        /// Only tool checks.
        #[arg(long)]
        tools: bool,
        /// Quran corpus checks.
        #[arg(long)]
        quran: bool,
        /// Full-corpus deep mode for Quran checks.
        #[arg(long)]
        deep: bool,
        /// Print the repair plan without executing.
        #[arg(long)]
        repair_preview: bool,
    },
    /// Configuration management.
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Database management.
    Db {
        #[command(subcommand)]
        action: DbAction,
    },
    /// Secret management (Phase 11).
    Secret {
        #[command(subcommand)]
        action: SecretAction,
    },
    /// Source catalog management (Phase 1).
    Source {
        #[command(subcommand)]
        action: SourceAction,
    },
    /// Background job management (Phase 1).
    Job {
        #[command(subcommand)]
        action: JobAction,
    },
    /// Audit log inspection (Phase 1).
    Audit {
        #[command(subcommand)]
        action: AuditAction,
    },
    /// Start the health server.
    Serve {
        /// Bind address (must be loopback in Phase 0).
        #[arg(long, default_value = "127.0.0.1:8737")]
        bind: String,
    },
    /// Quran corpus: lookup, import, validation, activation.
    Quran {
        #[command(subcommand)]
        action: quran::QuranAction,
    },
    /// Shell completions (Phase 1).
    Completions {
        /// Shell to generate completions for.
        shell: String,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show the effective configuration.
    Show {
        /// Explain where each value came from.
        #[arg(long)]
        explain: bool,
        /// Show defaults only.
        #[arg(long)]
        defaults: bool,
        /// Output as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Get a specific config key.
    Get { key: String },
    /// Validate the config file.
    Validate {
        #[arg(long)]
        file: Option<std::path::PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum DbAction {
    /// Apply pending migrations.
    Migrate,
    /// Show migration status.
    Status,
    /// Verify applied migration checksums.
    Verify,
    /// Print the migration plan without applying.
    Plan,
    /// Take a consistent backup (`VACUUM INTO`).
    Backup {
        /// Destination path for the backup file.
        path: String,
    },
    /// Restore from a backup (Phase 1+; requires --yes).
    Restore {
        /// Backup file to restore from.
        path: String,
        /// Confirm the destructive restore.
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum SecretAction {
    /// List secret references.
    List,
    /// Set a secret value.
    Set { r#ref: String },
    /// Delete a secret.
    Delete { r#ref: String },
}

#[derive(Subcommand)]
pub enum SourceAction {
    /// List sources.
    List,
    /// Show a source.
    Show { id: String },
    /// Import a manifest.
    Import { manifest_path: String },
}

#[derive(Subcommand)]
pub enum JobAction {
    /// List jobs.
    List,
    /// Show a job.
    Show { id: String },
    /// Cancel a job.
    Cancel { id: String },
}

#[derive(Subcommand)]
pub enum AuditAction {
    /// List audit events.
    List,
    /// Verify the audit chain.
    Verify,
}

/// Dispatch a parsed command, returning the process exit code.
pub fn dispatch(cli: Cli) -> i32 {
    let (cfg, origins) = match load_config(&cli) {
        Ok(pair) => pair,
        Err(err) => return report_config_error(&err, cli.config.as_deref(), cli.json),
    };
    match cli.command {
        Commands::Status => {
            println!("Q-ai: ready (Phase 0)");
            exit_code::OK
        }
        Commands::Version => {
            if cli.json {
                println!(r#"{{ "version": "{}" }}"#, env!("CARGO_PKG_VERSION"));
            } else {
                println!("qai {}", env!("CARGO_PKG_VERSION"));
            }
            exit_code::OK
        }
        Commands::Doctor { json, repair_preview, quran, deep, indexes, .. } => {
            let mut probe = block_on(application::db::probe_database(&cfg));
            // Collect the applied-migration checksum state read-only so the
            // `database.checksum_current` check can name `qai db verify` (D-05).
            if probe.reachable {
                let dir = migrations_dir();
                probe.migration_checksum_ok =
                    match block_on(application::db::verify_migrations(&cfg, &dir)) {
                        Ok(report) => Some(report.valid),
                        Err(_) => Some(false),
                    };
            }
            if repair_preview {
                doctor::run_checks(&cfg, &probe, false, true)
            } else if indexes {
                doctor::run_index_checks(&cfg, json || cli.json, deep)
            } else {
                let (doc, human, code) = doctor::doctor_report(&cfg, &probe, quran, deep);
                if json || cli.json {
                    println!("{}", serde_json::to_string_pretty(&doc).unwrap());
                } else {
                    print!("{human}");
                }
                code
            }
        }
        Commands::Config { action } => match action {
            ConfigAction::Show { explain, defaults, json } => {
                handle_config_show(&cfg, &origins, explain, defaults, json || cli.json)
            }
            ConfigAction::Get { key } => handle_config_get(&cfg, &origins, &key, cli.json),
            ConfigAction::Validate { file } => {
                handle_config_validate(&cfg, file.as_deref(), cli.json)
            }
        },
        Commands::Db { action } => handle_db(action, &cfg, cli.json),
        Commands::Secret { action } => handle_secret(action, cli.json),
        Commands::Source { action } => {
            let path = cfg.storage.sqlite.path.clone();
            handle_source(action, &path, cli.json)
        }
        Commands::Job { action } => {
            let path = cfg.storage.sqlite.path.clone();
            handle_job(action, &path, cli.json)
        }
        Commands::Audit { action: AuditAction::Verify } => {
            let path = cfg.storage.sqlite.path.clone();
            render_audit_verify(&path, cli.json)
        }
        Commands::Audit { action: AuditAction::List } => {
            let path = cfg.storage.sqlite.path.clone();
            match block_on(application::db::list_audit_events(&path)) {
                Ok(page) => {
                    print_catalog(&page, cli.json, "no audit events");
                    exit_code::OK
                }
                Err(e) => {
                    eprintln!("audit list failed: {e}");
                    exit_code::GENERIC
                }
            }
        }
        Commands::Serve { bind } => {
            if !server::is_loopback(&bind) {
                eprintln!("error: Phase 0 restricts server bind to loopback; refusing `{bind}`");
                return exit_code::POLICY;
            }
            let r = tokio::runtime::Builder::new_multi_thread().enable_all().build().map_err(|e| {
                eprintln!("failed to start async runtime: {e}");
                exit_code::INTERNAL
            });
            match r {
                Ok(rt) => {
                    let db_path = cfg.storage.sqlite.path.clone();
                    let result = rt.block_on(async {
                        let reader = std::sync::Arc::new(
                            application::quran_cli::open_reader(&db_path).await.map_err(|e| {
                                eprintln!("cannot open database: {e}");
                                exit_code::INTERNAL
                            })?,
                        );
                        let tools = std::sync::Arc::new(
                            application::quran_tools::ReaderToolBackend::registry(reader.clone()),
                        );
                        let api = std::sync::Arc::new(server::api::ReaderBackend::new(reader));
                        let search = std::sync::Arc::new(
                            application::quran_search_api::SearchApiService::open(&db_path)
                                .await
                                .map_err(|e| {
                                eprintln!("cannot open search backend: {e}");
                                exit_code::INTERNAL
                            })?,
                        );
                        server::api::serve(&bind, server::api::AppState { tools, api, search })
                            .await
                            .map_err(|e| {
                                eprintln!("serve failed: {e}");
                                exit_code::INTERNAL
                            })
                    });
                    match result {
                        Ok(()) => exit_code::OK,
                        Err(code) => code,
                    }
                }
                Err(code) => code,
            }
        }
        Commands::Completions { shell } => handle_completions(&shell),
        Commands::Quran { action } => {
            let db_path = cfg.storage.sqlite.path.clone();
            quran::handle_quran(action, &db_path, cli.json, cli.yes)
        }
    }
}

/// Locate the migrations directory: explicit CWD layout first, then the
/// compile-time workspace layout (covers tests and installed binaries run
/// outside the repo root).
fn migrations_dir() -> std::path::PathBuf {
    let from_cwd = std::path::PathBuf::from("migrations/sqlite");
    if from_cwd.is_dir() {
        return from_cwd;
    }
    let from_manifest =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    if from_manifest.is_dir() {
        return from_manifest;
    }
    from_cwd
}

/// Build the complete global CLI override map (D-07): `--data-dir` updates the
/// effective `app.data_dir`, `storage.sqlite.path`, and `storage.objects.root`
/// (each recorded with a CLI origin), and `--log-level` updates `logging.level`.
fn cli_overrides(cli: &Cli) -> BTreeMap<String, String> {
    let mut overrides = BTreeMap::new();
    if let Some(dir) = &cli.data_dir {
        overrides.insert("app.data_dir".to_string(), dir.clone());
        overrides.insert("storage.sqlite.path".to_string(), format!("{dir}/qai.db"));
        overrides.insert("storage.objects.root".to_string(), format!("{dir}/objects"));
    }
    if let Some(level) = &cli.log_level {
        overrides.insert("logging.level".to_string(), level.clone());
    }
    overrides
}

/// Load the effective `(Config, OriginMap)` exactly once (D-01, D-03, D-07).
///
/// Delegates the four-layer merge to `Config::load` (CLI > env > file >
/// defaults); never reconstructs defaults in a handler. The legacy
/// single-underscore `QAI_DATA_DIR` env is honored only when no `--data-dir`
/// flag was supplied and is attributed to that env var.
fn load_config(cli: &Cli) -> Result<(Config, OriginMap), config::ConfigError> {
    let (mut cfg, mut origins) = Config::load(cli.config.as_ref(), "QAI", &cli_overrides(cli))?;
    if cli.data_dir.is_none()
        && let Ok(dir) = std::env::var("QAI_DATA_DIR")
        && !dir.is_empty()
    {
        cfg.app.data_dir = dir.clone();
        cfg.storage.sqlite.path = format!("{dir}/qai.db");
        cfg.storage.objects.root = format!("{dir}/objects");
        let origin = ValueOrigin::Env("QAI_DATA_DIR".to_string());
        origins.insert("app.data_dir", origin.clone());
        origins.insert("storage.sqlite.path", origin.clone());
        origins.insert("storage.objects.root", origin);
    }
    Ok((cfg, origins))
}

/// Render `config show` output with secrets scrubbed (001-redaction-hardening,
/// US3/T021): JSON goes through `redact_json_value` (structure-preserving),
/// Debug text through `redact_text`. Routed via `application::redaction` so
/// `cli` gains no new workspace edge (`arch-check`).
fn render_config_show(cfg: &Config, json: bool) -> String {
    if json {
        let mut value = serde_json::to_value(cfg).unwrap_or(serde_json::Value::Null);
        application::redaction::redact_json_value(&mut value);
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| "null".to_string())
    } else {
        application::redaction::redact_text(&format!("{cfg:?}")).into_owned()
    }
}

/// One redacted JSON envelope carrying the effective configuration and the
/// dotted-key origin map (D-07). Redaction is applied to the whole document
/// before it can reach stdout.
fn config_envelope(cfg: &Config, origins: &OriginMap) -> serde_json::Value {
    let value = serde_json::to_value(cfg).unwrap_or(serde_json::Value::Null);
    let origin_obj: BTreeMap<String, String> =
        origins.iter().map(|(key, origin)| (key.clone(), origin.to_string())).collect();
    let mut envelope = serde_json::json!({ "config": value, "origins": origin_obj });
    application::redaction::redact_json_value(&mut envelope);
    envelope
}

/// Resolve a dotted config key (e.g. `server.port`) against an effective
/// serialized configuration value.
fn value_at_dotted<'a>(value: &'a serde_json::Value, key: &str) -> Option<&'a serde_json::Value> {
    let mut current = value;
    for segment in key.split('.') {
        current = current.get(segment)?;
    }
    Some(current)
}

/// Flatten a serialized effective configuration into dotted-key leaves.
fn flatten_effective(
    value: &serde_json::Value,
    prefix: String,
    out: &mut BTreeMap<String, serde_json::Value>,
) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, nested) in map {
                let dotted =
                    if prefix.is_empty() { key.clone() } else { format!("{prefix}.{key}") };
                flatten_effective(nested, dotted, out);
            }
        }
        _ => {
            out.insert(prefix, value.clone());
        }
    }
}

/// Render the human `--explain` view: one `key = value (origin)` line per
/// effective leaf in stable dotted-key order (D-07). Redaction runs over the
/// serialized document before any line can reach stdout.
fn render_config_explain(cfg: &Config, origins: &OriginMap) -> String {
    let mut value = serde_json::to_value(cfg).unwrap_or(serde_json::Value::Null);
    application::redaction::redact_json_value(&mut value);
    let mut flat = BTreeMap::new();
    flatten_effective(&value, String::new(), &mut flat);
    let mut out = String::new();
    for (key, leaf) in &flat {
        let origin =
            origins.get(key).map(ToString::to_string).unwrap_or_else(|| "unknown".to_string());
        out.push_str(&format!("{key} = {leaf} ({origin})\n"));
    }
    out
}

// ── Config diagnostic contract (D-07) ──────────────────────────────────

fn config_error_code(err: &config::ConfigError) -> &'static str {
    use config::ConfigError as E;
    match err {
        E::FileRead { .. } => "QAI-CFG-0001",
        E::Parse(_) => "QAI-CFG-0002",
        E::Validation(_) => "QAI-CFG-0003",
        E::InterpolationCycle(_) => "QAI-CFG-0004",
        E::MissingKey(_) => "QAI-CFG-0005",
    }
}

fn config_error_remedy(err: &config::ConfigError) -> &'static str {
    use config::ConfigError as E;
    match err {
        E::FileRead { .. } => "check the configuration file path and permissions, then retry",
        E::Parse(_) => "fix the TOML syntax reported above, then re-run validation",
        E::Validation(_) => "correct the reported configuration key, then re-run validation",
        E::InterpolationCycle(_) => "remove the circular `${...}` interpolation reference",
        E::MissingKey(_) => "supply the missing configuration key in the file, environment, or CLI",
    }
}

/// Emit a config diagnostic with the same code, remedy, and runnable next
/// command in human and JSON modes, and map to the centralized exit code.
fn report_config_error(err: &config::ConfigError, file: Option<&Path>, json: bool) -> i32 {
    let code = config_error_code(err);
    let summary = err.to_string();
    let remedy = config_error_remedy(err);
    let next_command = match file {
        Some(path) => match err {
            config::ConfigError::Parse(_) | config::ConfigError::Validation(_) => {
                format!("qai config show --file {} --json", path.display())
            }
            _ => format!("qai config validate --file {}", path.display()),
        },
        None => "qai config validate".to_string(),
    };
    if json {
        println!(
            "{}",
            serde_json::json!({
                "code": code,
                "summary": summary,
                "remedy": remedy,
                "next_command": next_command,
            })
        );
    } else {
        eprintln!("[{code}] {summary}");
        eprintln!("      remedy: {remedy}");
        eprintln!("      next: {next_command}");
    }
    exit_code::from_config_error(err)
}

fn handle_config_show(
    cfg: &Config,
    origins: &OriginMap,
    explain: bool,
    defaults: bool,
    json: bool,
) -> i32 {
    if defaults {
        let cfg = Config::default();
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&config_envelope(&cfg, &OriginMap::new()))
                    .unwrap_or_else(|_| "null".to_string())
            );
        } else {
            println!("config::default show: {}", render_config_show(&cfg, false));
        }
        return exit_code::OK;
    }
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&config_envelope(cfg, origins))
                .unwrap_or_else(|_| "null".to_string())
        );
    } else if explain {
        print!("{}", render_config_explain(cfg, origins));
    } else {
        println!("config show: {}", render_config_show(cfg, false));
    }
    exit_code::OK
}

fn handle_config_get(cfg: &Config, origins: &OriginMap, key: &str, json: bool) -> i32 {
    let value = serde_json::to_value(cfg).unwrap_or(serde_json::Value::Null);
    match value_at_dotted(&value, key) {
        Some(found) => {
            // Redact before emission: keyed lookups can address secret-shaped
            // leaves (e.g. `logging.file`) directly (D-07, T-01-CFG).
            let mut found = found.clone();
            application::redaction::redact_json_value(&mut found);
            let origin = origins.get(key).map(ToString::to_string);
            if json {
                println!("{}", serde_json::json!({ "key": key, "value": found, "origin": origin }));
            } else {
                println!("{key} = {found}");
                if let Some(origin) = origin {
                    println!("  origin: {origin}");
                }
            }
            exit_code::OK
        }
        None => {
            let summary = format!("unknown effective configuration key `{key}`");
            let remedy = "choose a valid dotted key; list the effective keys and origins";
            let next_command = "qai config show --explain";
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "code": "QAI-CFG-0006",
                        "summary": summary,
                        "remedy": remedy,
                        "next_command": next_command,
                    })
                );
            } else {
                eprintln!("[QAI-CFG-0006] {summary}");
                eprintln!("      remedy: {remedy}");
                eprintln!("      next: {next_command}");
            }
            exit_code::NOT_FOUND
        }
    }
}

fn handle_config_validate(cfg: &Config, file: Option<&Path>, json: bool) -> i32 {
    let candidate = match file {
        Some(path) => match load_config_file(path) {
            Ok(cfg) => cfg,
            Err(err) => return report_config_error(&err, Some(path), json),
        },
        None => cfg.clone(),
    };
    match candidate.validate() {
        Ok(()) => {
            if json {
                println!("{}", serde_json::json!({ "valid": true }));
            } else {
                println!("configuration valid");
            }
            exit_code::OK
        }
        Err(err) => report_config_error(&err, file, json),
    }
}

/// Load a named config file through the single `Config::load` merge engine.
fn load_config_file(path: &Path) -> Result<Config, config::ConfigError> {
    let path_buf = path.to_path_buf();
    Config::load(Some(&path_buf), "QAI", &BTreeMap::new()).map(|(cfg, _)| cfg)
}

/// Render `qai audit verify` through one typed diagnostic contract (D-12).
///
/// The persisted [`application::audit_bridge::AuditVerificationReport`] is the
/// single authority: no verifier logic is reimplemented here. Human and JSON
/// carry the same `code`, `summary`, `remedy`, `next_command`, `gaps`, and
/// `tampered_sequences`, and every invalid chain maps to the centralized
/// `exit_code::VALIDATION`.
fn render_audit_verify(path: &str, json: bool) -> i32 {
    match block_on(application::audit_bridge::verify_persisted_audit(path)) {
        Ok(report) => {
            if report.valid {
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "valid": true,
                            "checked_events": report.checked_events,
                            "gaps": report.gaps,
                            "tampered_sequences": report.tampered_sequences,
                        })
                    );
                } else {
                    println!("audit chain valid: {} events checked", report.checked_events);
                }
                exit_code::OK
            } else if let Some(diagnosis) =
                application::audit_bridge::diagnose_invalid_audit(&report)
            {
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "valid": false,
                            "code": diagnosis.code,
                            "summary": diagnosis.summary,
                            "remedy": diagnosis.remedy,
                            "next_command": diagnosis.next_command,
                            "gaps": report.gaps,
                            "tampered_sequences": report.tampered_sequences,
                        })
                    );
                } else {
                    eprintln!("[{}] {}", diagnosis.code, diagnosis.summary);
                    eprintln!("      remedy: {}", diagnosis.remedy);
                    eprintln!("      next: {}", diagnosis.next_command);
                    eprintln!(
                        "      gaps: {:?}; tampered sequences: {:?}",
                        report.gaps, report.tampered_sequences
                    );
                }
                exit_code::VALIDATION
            } else {
                eprintln!(
                    "audit verification produced an inconsistent report; re-run `qai audit verify`"
                );
                exit_code::INTERNAL
            }
        }
        Err(_) => {
            if json {
                println!(
                    r#"{{"valid":false,"error":"audit verification could not complete","remedy":"check database availability, migrations, and audit record integrity"}}"#
                );
            } else {
                eprintln!(
                    "audit verification could not complete; check database availability, migrations, and audit record integrity"
                );
            }
            exit_code::GENERIC
        }
    }
}

fn handle_db(action: DbAction, cfg: &Config, json: bool) -> i32 {
    let migrations_dir = migrations_dir();
    match action {
        DbAction::Migrate => {
            match block_on(application::db::migrate_database(cfg, &migrations_dir)) {
                Ok(version) => {
                    if json {
                        println!("{}", serde_json::json!({"schema_version": version}));
                    } else {
                        println!("migrations applied; schema version {version}");
                    }
                    exit_code::OK
                }
                Err(e) => {
                    eprintln!("migrate failed: {e}");
                    exit_code::INTERNAL
                }
            }
        }
        DbAction::Status | DbAction::Plan => {
            match block_on(application::db::migration_status(cfg, &migrations_dir)) {
                Ok(status) => {
                    if json {
                        println!(
                            "{}",
                            serde_json::json!({
                                "applied_version": status.applied_version,
                                "latest_on_disk": status.latest_on_disk,
                                "pending": status.pending,
                                "current": status.current,
                            })
                        );
                    } else {
                        println!(
                            "applied: v{}  latest: v{}  pending: {:?}",
                            status.applied_version, status.latest_on_disk, status.pending
                        );
                    }
                    exit_code::OK
                }
                Err(e) => {
                    eprintln!("status failed: {e}");
                    exit_code::INTERNAL
                }
            }
        }
        DbAction::Verify => {
            match block_on(application::db::verify_migrations(cfg, &migrations_dir)) {
                Ok(report) => {
                    if !report.valid {
                        eprintln!("checksum mismatch at version(s): {:?}", report.mismatches);
                        return exit_code::VALIDATION;
                    }
                    if json {
                        println!(
                            "{}",
                            serde_json::json!({
                                "valid": report.valid,
                                "mismatches": report.mismatches,
                            })
                        );
                    } else {
                        println!("migration checksums OK");
                    }
                    exit_code::OK
                }
                Err(e) => {
                    eprintln!("verify failed: {e}");
                    exit_code::INTERNAL
                }
            }
        }
        DbAction::Backup { path } => match block_on(application::db::backup_database(cfg, &path)) {
            Ok(()) => {
                println!("backup written to {path}");
                exit_code::OK
            }
            Err(e) => {
                eprintln!("backup failed: {e}");
                exit_code::INTERNAL
            }
        },
        DbAction::Restore { path, yes } => {
            if !yes {
                eprintln!("refusing to restore without --yes");
                return exit_code::USAGE;
            }
            match block_on(application::db::restore_database(cfg, &path, &migrations_dir)) {
                Ok(()) => {
                    println!("restored from {path} (previous database kept as .pre-restore)");
                    exit_code::OK
                }
                Err(e) => {
                    eprintln!("restore failed: {e}");
                    exit_code::INTERNAL
                }
            }
        }
    }
}

/// Run a future to completion on a small current-thread runtime.
fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    match tokio::runtime::Builder::new_current_thread().enable_all().build() {
        Ok(rt) => rt.block_on(fut),
        Err(e) => {
            eprintln!("failed to start async runtime: {e}");
            std::process::exit(exit_code::INTERNAL);
        }
    }
}

/// Print a catalog page: JSON document with `--json`, one human-readable
/// line per item otherwise. Payloads are already redacted by the application
/// layer; this function never touches secrets.
fn print_catalog(page: &serde_json::Value, json: bool, empty_hint: &str) {
    if json {
        println!("{page}");
        return;
    }
    let items = page["items"].as_array().cloned().unwrap_or_default();
    if items.is_empty() {
        println!("{empty_hint}");
    }
    for item in &items {
        println!("{}", catalog_line(item));
    }
    if page["truncated"] == true {
        eprintln!("warning: listing truncated to 1000 items; refine with `show <id>`");
    }
}

/// One-line human summary for a catalog item. Matches on whichever
/// descriptor fields the item carries (source / job / audit shapes).
fn catalog_line(item: &serde_json::Value) -> String {
    let id = item["id"].as_str().unwrap_or("?");
    if let Some(action) = item["action"].as_str() {
        let sequence = item["sequence"].as_u64().unwrap_or(0);
        let outcome = item["outcome"].as_str().unwrap_or("?");
        return format!("#{sequence} {action} [{outcome}] {id}");
    }
    if let Some(kind) = item["kind"].as_str() {
        let state = item["state"].as_str().unwrap_or("?");
        let attempts = item["attempts"].as_u64().unwrap_or(0);
        return format!("{id} {kind} [{state}] attempts={attempts}");
    }
    let title = item["title"].as_str().unwrap_or("?");
    let content_type = item["content_type"].as_str().unwrap_or("?");
    format!("{id} {title} ({content_type})")
}

fn storage_error_code(err: &application::db::CatalogError) -> i32 {
    match err {
        application::db::CatalogError::NotFound { .. } => exit_code::NOT_FOUND,
        application::db::CatalogError::MigrationRequired { .. }
        | application::db::CatalogError::MigrationChecksumMismatch { .. } => exit_code::VALIDATION,
        _ => exit_code::GENERIC,
    }
}

fn handle_source(action: SourceAction, path: &str, json: bool) -> i32 {
    match action {
        SourceAction::List => match block_on(application::db::list_sources(path)) {
            Ok(page) => {
                print_catalog(&page, json, "no sources");
                exit_code::OK
            }
            Err(e) => {
                eprintln!("source list failed: {e}");
                storage_error_code(&e)
            }
        },
        SourceAction::Show { id } => match block_on(application::db::get_source(path, &id)) {
            Ok(item) => {
                if json {
                    println!("{item}");
                } else {
                    println!("{}", catalog_line(&item));
                    if let Some(versions) = item["versions"].as_array() {
                        for version in versions {
                            let version_id = version["version"].as_str().unwrap_or("?");
                            let state = version["state"].as_str().unwrap_or("?");
                            println!("  {version_id} [{state}]");
                        }
                    }
                }
                exit_code::OK
            }
            Err(e) => {
                eprintln!("source show failed: {e}");
                storage_error_code(&e)
            }
        },
        SourceAction::Import { .. } => {
            eprintln!(
                "refusing `qai source import`: cataloged import requires manifest validation, \
                 approval preconditions, and same-transaction audit (Phase 1); \
                 use `qai quran import` for corpus editions"
            );
            exit_code::USAGE
        }
    }
}

fn handle_job(action: JobAction, path: &str, json: bool) -> i32 {
    match action {
        JobAction::List => match block_on(application::db::list_jobs(path)) {
            Ok(page) => {
                print_catalog(&page, json, "no jobs");
                exit_code::OK
            }
            Err(e) => {
                eprintln!("job list failed: {e}");
                storage_error_code(&e)
            }
        },
        JobAction::Show { id } => match block_on(application::db::get_job(path, &id)) {
            Ok(item) => {
                if json {
                    println!("{item}");
                } else {
                    println!("{}", catalog_line(&item));
                }
                exit_code::OK
            }
            Err(e) => {
                eprintln!("job show failed: {e}");
                storage_error_code(&e)
            }
        },
        JobAction::Cancel { .. } => {
            eprintln!(
                "refusing `qai job cancel`: cancellation must coordinate with the worker \
                 lease and record an audit event in the same transaction (Phase 1)"
            );
            exit_code::USAGE
        }
    }
}

fn handle_secret(action: SecretAction, json: bool) -> i32 {
    use config::{EnvSecretStore, SecretStore as _};
    match action {
        SecretAction::List => {
            let refs = block_on(EnvSecretStore.list_refs());
            match refs {
                Ok(refs) => {
                    let names: Vec<String> = refs.iter().map(ToString::to_string).collect();
                    if json {
                        println!("{}", serde_json::json!({"refs": names}));
                    } else if names.is_empty() {
                        println!("no secret references");
                    } else {
                        for name in &names {
                            println!("{name}");
                        }
                    }
                    exit_code::OK
                }
                Err(e) => {
                    eprintln!("secret list failed: {e}");
                    exit_code::GENERIC
                }
            }
        }
        SecretAction::Set { .. } => {
            eprintln!(
                "refusing `qai secret set`: durable secret writes live behind the \
                 Phase 11 secret-management surface, not the Phase 0 chassis"
            );
            exit_code::USAGE
        }
        SecretAction::Delete { .. } => {
            eprintln!(
                "refusing `qai secret delete`: durable secret writes live behind the \
                 Phase 11 secret-management surface, not the Phase 0 chassis"
            );
            exit_code::USAGE
        }
    }
}

fn handle_completions(shell: &str) -> i32 {
    const COMMANDS: &[&str] = &[
        "status",
        "version",
        "doctor",
        "config",
        "db",
        "secret",
        "source",
        "job",
        "audit",
        "serve",
        "quran",
        "completions",
    ];
    let names = COMMANDS.join(" ");
    match shell {
        "bash" => {
            println!(
                "_qai_completions() {{ COMPREPLY=($(compgen -W \"{names}\" -- \"${{COMP_WORDS[COMP_CWORD]}}\")); }}\n\
                 complete -F _qai_completions qai"
            );
            exit_code::OK
        }
        "zsh" => {
            println!("#compdef qai\n_qai() {{ _arguments '1:command:({names})' }}\n_qai \"$@\"");
            exit_code::OK
        }
        "fish" => {
            println!("complete -c qai -f -n '__fish_use_subcommand' -a '{names}'");
            exit_code::OK
        }
        "powershell" | "elvish" => {
            println!("# qai {shell} completions (static Phase 0 set): {names}");
            exit_code::OK
        }
        other => {
            eprintln!(
                "unsupported shell `{other}`; expected bash, zsh, fish, powershell, or elvish"
            );
            exit_code::USAGE
        }
    }
}

// exit codes and dispatch complete

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn serve_defaults_to_loopback() {
        let cli = Cli::try_parse_from(["qai", "serve"]).unwrap();
        match cli.command {
            Commands::Serve { bind } => {
                assert!(
                    bind.starts_with("127.0.0.1") || bind.starts_with("[::1]"),
                    "serve must default to loopback, got `{bind}`"
                );
            }
            _ => panic!("expected Serve command"),
        }
    }

    #[test]
    fn default_config_binds_loopback_and_validates() {
        let cfg = Config::default();
        assert_eq!(cfg.server.bind, "127.0.0.1");
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn non_loopback_bind_without_tls_fails_validation() {
        let mut cfg = Config::default();
        cfg.server.bind = "0.0.0.0".into();
        cfg.server.tls = "disabled".into();
        let err = cfg.validate().unwrap_err();
        assert!(err.to_string().contains("0.0.0.0"), "error must name the offending bind: {err}");
    }

    /// US3/T021: `config show` render path scrubs credential-shaped values
    /// (Rule C userinfo here) in both JSON and text modes while preserving
    /// non-secret surroundings byte-for-byte.
    #[test]
    fn config_show_scrubs_credential_shaped_values() {
        const SENTINEL: &str = "SENTINEL_9f3c__DO_NOT_LEAK";
        let mut cfg = Config::default();
        cfg.storage.sqlite.path = format!("postgresql://admin:{SENTINEL}@localhost:5432/qai");
        for json in [false, true] {
            let out = render_config_show(&cfg, json);
            assert!(!out.contains(SENTINEL), "config show (json={json}) leaked: {out}");
            assert!(out.contains("***REDACTED***"), "config show (json={json}) lost marker: {out}");
            assert!(
                out.contains("localhost:5432"),
                "config show (json={json}) mangled host: {out}"
            );
        }
        // JSON stays a parseable object with non-secret leaves intact.
        let doc: serde_json::Value =
            serde_json::from_str(&render_config_show(&cfg, true)).expect("valid JSON");
        assert_eq!(doc["server"]["bind"], "127.0.0.1");
        // Default config carries no credential shapes: redaction is a no-op
        // on normal output (FR-008).
        let defaults = Config::default();
        assert_eq!(render_config_show(&defaults, false), format!("{defaults:?}"));
    }
}
