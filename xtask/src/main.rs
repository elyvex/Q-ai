//! `cargo xtask` — Q-ai build tooling (D0.1).
//!
//! Subcommands:
//!   arch-check    — enforce workspace dependency-direction rules (AC-P0-02)
//!   ci            — run the whole pre-merge gate
//!   migrate-check — enforce migrations are append-only and checksums stable
//!   gen-schema    — emit JSON Schemas deterministically into `docs/schemas/`
//!
//! Invoked via `cargo xtask <cmd>` (aliased by an `.cargo/config.toml` `alias.xtask`
//! so it runs as a normal workspace command).

use clap::{Parser, Subcommand};
use std::process::ExitCode;

mod arch;
mod ci;
mod migrate;
mod schema;

#[derive(Parser)]
#[command(name = "xtask", about = "Q-ai build tooling")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Enforce workspace dependency-direction rules; exit non-zero on a forbidden edge.
    ArchCheck,
    /// Run the full pre-merge gate (fmt, clippy, test, deny, arch, migrate).
    Ci,
    /// Verify migrations are append-only and checksums are unchanged.
    MigrateCheck {
        /// Path to the migrations directory (default: ./migrations/sqlite).
        #[arg(long, value_name = "DIR")]
        dir: Option<std::path::PathBuf>,
    },
    /// Emit JSON Schemas into docs/schemas (idempotent).
    GenSchema,
}

fn run(cli: Cli) -> Result<(), anyhow::Error> {
    match cli.command {
        Commands::ArchCheck => arch::run(),
        Commands::Ci => ci::run(),
        Commands::MigrateCheck { dir } => migrate::run(dir.as_deref()),
        Commands::GenSchema => schema::run(),
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e:#}");
            ExitCode::FAILURE
        }
    }
}
