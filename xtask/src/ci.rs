//! `cargo xtask ci` — the full pre-merge gate (AC-P0-01).
//!
//! Runs, in order: fmt check, clippy -D warnings, test workspace, deny (if installed),
//! arch-check, migrate-check, gen-schema diff. Any non-zero exit stops the gate.

use anyhow::{bail, Context, Result};
use std::process::Command;

fn run_cmd(cmd: &mut Command) -> Result<()> {
    let status = cmd.status().with_context(|| format!("failed to spawn {cmd:?}"))?;
    anyhow::ensure!(status.success(), "command failed: {cmd:?}");
    Ok(())
}

pub fn run() -> Result<()> {
    let root = std::env::current_dir()?;

    println!("── qai ci 1/7 · fmt ──");
    run_cmd(Command::new("cargo").args(["fmt", "--all", "--", "--check"]).current_dir(&root))?;

    println!("── qai ci 2/7 · clippy ──");
    run_cmd(Command::new("cargo")
        .args(["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])
        .current_dir(&root))?;

    println!("── qai ci 3/7 · test ──");
    run_cmd(Command::new("cargo").args(["test", "--workspace"]).current_dir(&root))?;

    println!("── qai ci 4/7 · deny ──");
    match Command::new("cargo-deny").arg("check").current_dir(&root).status() {
        Ok(status) if status.success() => {}
        Ok(status) if status.code() == Some(2) => {
            // code 2 from deny can mean "policy issue" — treat as failure
            bail!("cargo-deny check failed with {status}");
        }
        Ok(_) => bail!("cargo-deny check failed"),
        Err(_) => {
            // deny not installed -> warn, do not block (it is enforced in CI).
            println!("WARN: cargo-deny not installed; skipping (enforced in CI).");
        }
    }

    println!("── qai ci 5/7 · arch-check ──");
    run_cmd(Command::new("cargo")
        .args(["run", "-p", "xtask", "--", "arch-check"])
        .current_dir(&root))?;

    println!("── qai ci 6/7 · migrate-check ──");
    run_cmd(Command::new("cargo")
        .args(["run", "-p", "xtask", "--", "migrate-check"])
        .current_dir(&root))?;

    println!("── qai ci 7/7 · gen-schema diff ──");
    run_cmd(Command::new("cargo")
        .args(["run", "-p", "xtask", "--", "gen-schema"])
        .current_dir(&root))?;
    run_cmd(Command::new("git").args(["diff", "--exit-code", "docs/schemas/"]).current_dir(&root))?;

    println!("── qai ci: OK ──");
    Ok(())
}
