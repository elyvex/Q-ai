//! `cargo xtask ci` — the full pre-merge gate (AC-P0-01).
//!
//! Runs, in order: fmt check, clippy -D warnings, SPA build, test workspace,
//! deny (if installed), parity, arch-check, migrate-check, adr-lint,
//! gen-schema diff, doctor schema. Any non-zero exit stops the gate.
//!
//! Dev loop: run `qai serve` on 127.0.0.1:8737 and `npm run dev -- --port …`
//! in `web/` with the Vite dev server proxying `/api` to the local serve
//! port; the committed build (`npm run build` in the SPA step below) is what
//! ships embedded.

use anyhow::{Context, Result, bail};
use std::process::Command;

fn run_cmd(cmd: &mut Command) -> Result<()> {
    let status = cmd.status().with_context(|| format!("failed to spawn {cmd:?}"))?;
    anyhow::ensure!(status.success(), "command failed: {cmd:?}");
    Ok(())
}

pub fn run() -> Result<()> {
    let root = std::env::current_dir()?;

    println!("── qai ci 1/11 · fmt ──");
    run_cmd(Command::new("cargo").args(["fmt", "--all", "--", "--check"]).current_dir(&root))?;

    println!("── qai ci 2/11 · clippy ──");
    run_cmd(
        Command::new("cargo")
            .args(["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])
            .current_dir(&root),
    )?;

    println!("── qai ci 3/11 · spa build ──");
    // Phase 5 (D-03): the committed SPA builds here so every later Rust step
    // (notably the spa_fallback contract test) runs against a populated
    // embed. Fails loudly without Node/npm — the CI workflow provides Node.
    // `web/dist` stays gitignored; `allow_missing` keeps plain `cargo build`
    // working outside this gate.
    run_cmd(Command::new("npm").args(["ci"]).current_dir(root.join("web")))?;
    run_cmd(Command::new("npm").args(["run", "build"]).current_dir(root.join("web")))?;

    println!("── qai ci 4/11 · test ──");
    run_cmd(Command::new("cargo").args(["test", "--workspace"]).current_dir(&root))?;

    println!("── qai ci 5/11 · deny ──");
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

    println!("── qai ci 6/11 · parity ──");
    // Phase 5 (D-16/SC3): the cross-surface parity gate as an explicit step —
    // all 12 tools × service/HTTP/CLI legs, byte-identical payloads.
    run_cmd(
        Command::new("cargo")
            .args(["test", "-p", "cli", "--test", "parity_cross_surface"])
            .current_dir(&root),
    )?;

    println!("── qai ci 7/11 · arch-check ──");
    run_cmd(
        Command::new("cargo").args(["run", "-p", "xtask", "--", "arch-check"]).current_dir(&root),
    )?;

    println!("── qai ci 8/11 · migrate-check ──");
    run_cmd(
        Command::new("cargo")
            .args(["run", "-p", "xtask", "--", "migrate-check"])
            .current_dir(&root),
    )?;

    println!("── qai ci 9/11 · adr-lint ──");
    run_cmd(
        Command::new("cargo").args(["run", "-p", "xtask", "--", "adr-lint"]).current_dir(&root),
    )?;

    println!("── qai ci 10/11 · gen-schema diff ──");
    run_cmd(
        Command::new("cargo").args(["run", "-p", "xtask", "--", "gen-schema"]).current_dir(&root),
    )?;
    run_cmd(Command::new("git").args(["diff", "--exit-code", "docs/schemas/"]).current_dir(&root))?;

    println!("── qai ci 11/11 · doctor schema ──");
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
