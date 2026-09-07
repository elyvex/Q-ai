//! `cargo xtask` — build tooling for Q-ai (D0.1).
//!
//! Subcommands: `arch-check`, `ci`, `migrate-check`, `gen-schema`.
//! Implemented fully in P0-T02; this is the dispatch skeleton so the
//! workspace builds and `cargo xtask` is discoverable from day one.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("arch-check") => {
            eprintln!("arch-check: not yet implemented (P0-T02)");
            ExitCode::from(2)
        }
        Some("ci") => {
            eprintln!("ci: not yet implemented (P0-T02)");
            ExitCode::from(2)
        }
        Some("migrate-check") => {
            eprintln!("migrate-check: not yet implemented (P0-T02)");
            ExitCode::from(2)
        }
        Some("gen-schema") => {
            eprintln!("gen-schema: not yet implemented (P0-T02)");
            ExitCode::from(2)
        }
        Some(other) => {
            eprintln!("unknown xtask command: {other}");
            eprintln!("usage: cargo xtask <arch-check|ci|migrate-check|gen-schema>");
            ExitCode::from(2)
        }
        None => {
            eprintln!("usage: cargo xtask <arch-check|ci|migrate-check|gen-schema>");
            ExitCode::from(2)
        }
    }
}
