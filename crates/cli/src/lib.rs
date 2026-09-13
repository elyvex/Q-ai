pub mod commands;
pub mod db;
pub mod doctor;
pub mod exit_code;
pub mod server_stub;

use clap::Parser;
use commands::Cli;

/// Q-ai CLI entry point.
pub fn run() -> i32 {
    let cli = Cli::parse();

    let format = if cli.json {
        observability::Format::Json
    } else {
        observability::Format::Text
    };
    let _guard = observability::init(format);

    commands::dispatch(cli)
}
