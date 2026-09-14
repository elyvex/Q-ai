//! `cargo xtask ci` — the full pre-merge gate (AC-P0-01).
//!
//! Runs, in order: fmt check, clippy -D warnings, test workspace, deny (if installed),
//! arch-check, migrate-check, adr-lint, gen-schema diff. Any non-zero exit stops the gate.

use anyhow::{Context, Result, bail};
use std::process::Command;

fn run_cmd(cmd: &mut Command) -> Result<()> {
    let status = cmd.status().with_context(|| format!("failed to spawn {cmd:?}"))?;
    anyhow::ensure!(status.success(), "command failed: {cmd:?}");
    Ok(())
}

pub fn run() -> Result<()> {
    let root = std::env::current_dir()?;

    println!("── qai ci 1/9 · fmt ──");
    run_cmd(Command::new("cargo").args(["fmt", "--all", "--", "--check"]).current_dir(&root))?;

    println!("── qai ci 2/9 · clippy ──");
    run_cmd(
        Command::new("cargo")
            .args(["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])
            .current_dir(&root),
    )?;

    println!("── qai ci 3/9 · test ──");
    run_cmd(Command::new("cargo").args(["test", "--workspace"]).current_dir(&root))?;

    println!("── qai ci 4/9 · deny ──");
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

    println!("── qai ci 5/9 · arch-check ──");
    run_cmd(
        Command::new("cargo").args(["run", "-p", "xtask", "--", "arch-check"]).current_dir(&root),
    )?;

    println!("── qai ci 6/9 · migrate-check ──");
    run_cmd(
        Command::new("cargo")
            .args(["run", "-p", "xtask", "--", "migrate-check"])
            .current_dir(&root),
    )?;

    println!("── qai ci 7/9 · adr-lint ──");
    run_cmd(
        Command::new("cargo").args(["run", "-p", "xtask", "--", "adr-lint"]).current_dir(&root),
    )?;

    println!("── qai ci 8/9 · gen-schema diff ──");
    run_cmd(
        Command::new("cargo").args(["run", "-p", "xtask", "--", "gen-schema"]).current_dir(&root),
    )?;
    run_cmd(Command::new("git").args(["diff", "--exit-code", "docs/schemas/"]).current_dir(&root))?;

    println!("── qai ci 9/9 · doctor schema ──");
    // Build the binary, migrate a scratch database, and validate `doctor --json`.
    run_cmd(Command::new("cargo").args(["build", "--bin", "qai", "-p", "cli"]).current_dir(&root))?;
    let scratch = std::env::temp_dir().join(format!("qai-ci-doctor-{}", std::process::id()));
    let scratch_str = scratch.to_string_lossy().to_string();
    let qai = root.join("target/debug/qai");
    run_cmd(
        Command::new(&qai).args(["--data-dir", &scratch_str, "db", "migrate"]).current_dir(&root),
    )?;
    let doctor_json = scratch.join("doctor.json");
    let out = Command::new(&qai)
        .args(["--data-dir", &scratch_str, "doctor", "--json"])
        .current_dir(&root)
        .output()
        .context("run qai doctor --json")?;
    anyhow::ensure!(out.status.success(), "qai doctor --json failed");
    std::fs::write(&doctor_json, &out.stdout)
        .with_context(|| format!("write {}", doctor_json.display()))?;
    run_cmd(
        Command::new("cargo")
            .args([
                "run",
                "-q",
                "-p",
                "xtask",
                "--",
                "validate",
                doctor_json.to_string_lossy().as_ref(),
                "docs/schemas/doctor.v1.schema.json",
            ])
            .current_dir(&root),
    )?;
    let _ = std::fs::remove_dir_all(&scratch);

    println!("── qai ci: OK ──");
    Ok(())
}
