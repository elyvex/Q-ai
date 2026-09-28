# Phase 03 — Deferred Items

Out-of-scope discoveries logged during execution (SCOPE BOUNDARY: pre-existing
failures in unrelated files are not auto-fixed).

## From plan 03-01 (executed 2026-09-28)

### 1. `cargo test -p cli --test quran` — stale trycmd snapshots after the enqueue-only import refactor (pre-existing, NOT caused by 03-01)

- **Command:** `cargo test -p cli --test quran`
- **Result:** 9 of 13 snapshot tests fail; e.g. `quran_search_snapshots` expects
  `imported test-edition-min@0.1.0 to Staged` but the binary now prints
  `queued test-edition-min@0.1.0 import as job <id> (state: Queued)`.
- **Cause:** commit `c2d6b68 feat(01-04): make quran import enqueue-only with queued job result`
  changed `application::quran::run_import_job` → `enqueue_import_job` and the CLI
  wording, but did **not** update `crates/cli/tests/quran/*.trycmd`. The snapshots
  are stale relative to the committed behaviour.
- **Non-causation proof:** reverting the 03-01 doc-only change to
  `crates/quran-normalization/src/rules/mod.rs` and re-running
  `cargo test -p cli --test quran quran_search_snapshots` reproduces the identical
  failure. A doc comment cannot change CLI runtime output.
- **Owner:** the 01-04 enqueue-only plan (or a follow-up) must refresh the trycmd
  snapshots to the queued/host contract.
- **Not fixed here:** outside plan 03-01's `files_modified` and outside the
  rule-doc correction's blast radius.

## From plan 03-03 (executed 2026-09-28)

### 1. `cargo fmt --check` is red for two committed 03-01 test files (pre-existing, NOT caused by 03-03)

- **Files:** `crates/application/tests/alpha_smoke.rs`,
  `crates/application/tests/canonical_display_identity.rs`
- **Finding:** running `cargo fmt -p application` (edition-2024 `rustfmt.toml`)
  reformats these two committed files, so they are not currently rustfmt-clean in
  CI terms. They are unrelated to plan 03-03 and were reverted here rather than
  fixed.
- **Owner:** a formatting-sweep follow-up (or the phase gate) should run
  `cargo fmt -p application` on the 03-01 artifacts.
- **Not fixed here:** outside plan 03-03's `files_modified`.

### Status note (03-01 item 1, now resolved)

`cargo test -p cli --test quran` is **green (13/13)** on the 03-03 base commit
`0afc4d1`; the stale-snapshot failure recorded above was refreshed by the later
`01-04`/`03-02` snapshot commits.

