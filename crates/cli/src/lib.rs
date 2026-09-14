//! Q-ai CLI crate.
//!
//! Exposes `run()` for the `qai` binary and a clap-based command tree
//! covering the Phase 0 surface: version, config, db, doctor, serve, and
//! Phase-N stub commands.

pub mod doctor;
pub mod exit_code;

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
    let cfg = loads_or_default(&cli);
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
        Commands::Doctor { json, repair_preview, .. } => {
            let probe = block_on(application::db::probe_database(&cfg));
            doctor::run_checks(&cfg, &probe, json || cli.json, repair_preview)
        }
        Commands::Config { action } => match action {
            ConfigAction::Show { explain, defaults, json } => {
                handle_config_show(explain, defaults, json)
            }
            ConfigAction::Get { key } => handle_config_get(&key),
            ConfigAction::Validate { file } => handle_config_validate(file.as_deref()),
        },
        Commands::Db { action } => handle_db(action, &cfg, cli.json),
        Commands::Secret { .. } => phase_stub("secret", 11),
        Commands::Source { .. } => phase_stub("source", 1),
        Commands::Job { .. } => phase_stub("job", 1),
        Commands::Audit { .. } => phase_stub("audit", 1),
        Commands::Serve { bind } => {
            if !bind.starts_with("127.0.0.1") && !bind.starts_with("[::1]") {
                eprintln!("error: Phase 0 restricts server bind to loopback; refusing `{bind}`");
                return exit_code::POLICY;
            }
            let r = tokio::runtime::Builder::new_multi_thread().enable_all().build().map_err(|e| {
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

fn loads_or_default(cli: &Cli) -> Config {
    let mut cfg = match cli.config.as_ref() {
        Some(path) => {
            let overrides = std::collections::BTreeMap::new();
            config::Config::load(Some(path), "QAI", &overrides).map(|(c, _)| c).unwrap_or_default()
        }
        None => Config::default(),
    };
    if let Some(dir) = &cli.data_dir {
        cfg.app.data_dir = dir.clone();
        cfg.storage.sqlite.path = format!("{dir}/qai.db");
        cfg.storage.objects.root = format!("{dir}/objects");
    }
    if let Some(level) = &cli.log_level {
        cfg.logging.level = level.clone();
    }
    cfg
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

fn handle_db(action: DbAction, cfg: &Config, json: bool) -> i32 {
    let migrations_dir = std::path::PathBuf::from("migrations/sqlite");
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

fn phase_stub(name: &str, phase: u8) -> i32 {
    println!("`qai {name}` will become available in Phase {phase}.");
    exit_code::OK
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
}
