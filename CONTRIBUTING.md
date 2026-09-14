# Contributing to Q-ai

Thanks for contributing. This project enforces integrity by construction; please read the checks
below before opening a pull request.

## Before you start

- Read `AGENTS.md` and `docs/03-plan/phases/phase-00-foundation/plan.md`.
- Run the pre-merge gate locally:

  ```bash
  cargo fmt --all
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
  cargo run -p xtask -- arch-check
  cargo run -p xtask -- migrate-check
  ```

## Pull-request checklist (Definition of Done, PRD §58)

- [ ] Implemented behind an explicit interface; no cross-layer coupling (`arch-check` green)
- [ ] Unit tests + property tests where the state space warrants
- [ ] Integration tests against **real** SQLite (not mocks) where persistence is involved
- [ ] Typed errors implement `Diagnostic` with a remedy **and** next command
- [ ] Observable: tracing spans + metrics registered in the catalog
- [ ] Documented in `docs/architecture/` and referenced from the crate's `//!` docs
- [ ] Configuration validated at load **and** on change
- [ ] Cancellation and timeouts implemented where the operation is long-running
- [ ] Secrets redacted everywhere (secret-leak suite green)
- [ ] Provenance and audit events recorded for **every** mutation
- [ ] Schema versions recorded; migration reversible or explicitly forward-only **with rationale**
- [ ] Access/policy checks present (deny-by-default) even in single-user mode
- [ ] Failure recovery tested (crash, retry, interrupted job)

## Error-code registry rules

- Error codes are **public API**. Never renumber or reuse a code.
- Namespaces: `QAI-CFG-`, `QAI-SEC-`, `QAI-DB-`, `QAI-JOB-`, `QAI-SRC-`, `QAI-PROV-`, `QAI-AUD-`,
  `QAI-CLI-`, `QAI-QUR-`, `QAI-NORM-`, `QAI-IDX-`.
- Every code must be **unique** across the workspace (a test enforces this).
- See `docs/architecture/error-codes.md`.

## Secret-redaction requirements

- Secret values live only in `Secret<T>` (env / OS keychain / age-encrypted file).
- **Never** store secret values in SQLite in Phase 0 — store a `SecretRef` only.
- `Debug`/`Display`/`Serialize` of `Secret<T>` must render `***`; never log `.expose()`.
- The secret-leak sentinel suite (`crates/testkit/tests/secret_leak.rs`) is a **permanent CI gate**
  reused by every future phase.

## Migration append-only policy

- Migrations are numbered, append-only, and checksummed (`migrations/sqlite/NNNN_*.up.sql`).
- **Never** edit an applied migration — add a new one instead.
- `xtask migrate-check` fails CI on checksum drift (`QAI-DB-0003`); regenerate
  `migrations/sqlite/checksums.json` only when adding a new migration.
- Backup/restore uses `VACUUM INTO`, never a raw file copy (ADR-0001 §7).

## Architecture rules

- Dependency directions are enforced by `cargo run -p xtask -- arch-check` against
  `xtask/allowlist.toml`. Adding a forbidden edge fails CI (AC-P0-02).
- The `domain` crate has **no** async runtime or I/O dependencies.
- `unsafe` is forbidden workspace-wide.

## Commits & PRs

- Use clear, conventional commit messages. Do not commit secrets or generated databases.
- Update the task document, phase progress, and `CHANGELOG.md` where appropriate.
