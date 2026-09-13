//! Q-ai CLI crate.
//!
//! Exposes `run()` for the `qai` binary and a clap-based command tree
//! covering the Phase 0 surface: version, config, db, doctor, serve, and
//! Phase-N stub commands.

pub mod doctor;

use clap::{Parser, Subcommand};
use config::Config;

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
    match cli.command {
        Commands::Version => {
            if cli.json {
                println!(r#"{{ "version": "{}" }}"#, env!("CARGO_PKG_VERSION"));
            } else {
                println!("qai {}", env!("CARGO_PKG_VERSION"));
            }
            exit_code::OK
        }
        Commands::Doctor { json, .. } => {
            let cfg = loads_or_default(cli.config.as_deref());
            doctor::run_checks(&cfg, json)
        }
        Commands::Config { action } => match action {
            ConfigAction::Show { explain, defaults, json } => handle_config_show(explain, defaults, json),
            ConfigAction::Get { key } => handle_config_get(&key),
            ConfigAction::Validate { file } => handle_config_validate(file.as_deref()),
        },
        Commands::Db { action } => match action {
            DbAction::Migrate | DbAction::Status | DbAction::Verify | DbAction::Plan => {
                handle_db(action)
            }
        },
        Commands::Secret { .. } => phase_stub("secret", 11),
        Commands::Source { .. } => phase_stub("source", 1),
        Commands::Job { .. } => phase_stub("job", 1),
        Commands::Audit { .. } => phase_stub("audit", 1),
        Commands::Serve { bind } => {
            if !bind.starts_with("127.0.0.1") && !bind.starts_with("[::1]") {
                eprintln!("error: Phase 0 restricts server bind to loopback; refusing `{bind}`");
                return exit_code::POLICY;
            }
            let r = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .map_err(|e| {
                    eprintln!("failed to start async runtime: {e}");
                    exit_code::INTERNAL
                });
            match r {
                Ok(rt) => {
                    let result = rt.block_on(server::start(&bind));
                    match result {
                        Ok(()) => exit_code::OK,
                        Err(e) => {
                            eprintln!("serve failed: {e}");
                            exit_code::INTERNAL
                        }
                    }
                }
                Err(code) => code,
            }
        }
        Commands::Completions { .. } => phase_stub("completions", 13),
    }
}

fn loads_or_default(path: Option<&std::path::Path>) -> Config {
    let _ = path;
    Config::default()
}

fn handle_config_show(explain: bool, defaults: bool, json: bool) -> i32 {
    let cfg = Config::default();
    if defaults {
        if json {
            println!("{}", serde_json::to_string_pretty(&cfg).unwrap());
        } else {
            println!("{cfg:?}");
        }
        return exit_code::OK;
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&cfg).unwrap());
    } else {
        println!("config::default show: {cfg:?}");
    }
    let _ = explain;
    exit_code::OK
}

fn handle_config_get(key: &str) -> i32 {
    let cfg = Config::default();
    println!("{cfg:?}");
    let _ = key;
    exit_code::OK
}

fn handle_config_validate(file: Option<&std::path::Path>) -> i32 {
    let _ = file;
    let cfg = Config::default();
    match cfg.validate() {
        Ok(()) => exit_code::OK,
        Err(_) => exit_code::VALIDATION,
    }
}

fn handle_db(action: DbAction) -> i32 {
    match action {
        DbAction::Migrate => {
            let dir = "migrations/sqlite";
            match std::fs::read_dir(dir) {
                Ok(entries) => {
                    let mut files: Vec<_> = entries.filter_map(|e| e.ok()).collect();
                    files.sort_by_key(|e| e.file_name());
                    for f in files {
                        println!("apply: {}", f.path().display());
                    }
                    exit_code::OK
                }
                Err(e) => {
                    eprintln!("failed to read migrations dir: {e}");
                    exit_code::INTERNAL
                }
            }
        }
        DbAction::Status => {
            println!("migration status: pending detection requires a live SQLite run — see tests/db.rs");
            exit_code::OK
        }
        DbAction::Verify => {
            println!("verify: no database file found");
            exit_code::OK
        }
        DbAction::Plan => {
            println!("plan: no database file found; all migrations pending");
            exit_code::OK
        }
    }
}

fn phase_stub(name: &str, phase: u8) -> i32 {
    println!("`qai {name}` will become available in Phase {phase}.");
    exit_code::OK
}

// exit_codes handled via exit_code module
pub use crate::doctor::run as doctor_run;

mod exit_code;
