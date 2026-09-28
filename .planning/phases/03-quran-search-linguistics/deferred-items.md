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

## From plan 03-04 (executed 2026-09-28)

### 1. `affix_search` typed error wording is imprecise when a dataset IS active

- **File:** `crates/application/src/quran_morphology.rs` (`affix_search`,
  `MorphologyToolError::UnavailableDataset`)
- **Finding:** with an active dataset whose morpheme index is not yet built,
  `affix_search` now fails closed with `UnavailableDataset { capability:
  "affix/morpheme index" }` (code `QAI-MORPH-0004`), whose `Display` reads
  "no active morphology dataset for affix/morpheme index; import and activate
  one first". A dataset **is** active in that branch, so the first clause
  misleads. The plan pinned this exact variant/code (G-09/T-03-16) rather than
  introducing a new error code, so it was implemented as specified.
- **Owner:** the SC3 surface plan (03-05, `qai quran family`/morphology
  surfaces) should consider a dedicated "capability not yet built on the active
  dataset" message (a new `QAI-MORPH-*` code, or a reworded capability string),
  coordinated with the Diagnostic code map.
- **Not fixed here:** changing the variant or code map would exceed plan 03-04's
  declared scope and its pinned test assertions.

### 2. `cargo fmt --check` remains red for the two 03-01 test files (pre-existing, still NOT caused by 03-04)

- **Files:** `crates/application/tests/alpha_smoke.rs`,
  `crates/application/tests/canonical_display_identity.rs`
- **Finding:** unchanged from the 03-03 entry above. Plan 03-04 ran `rustfmt`
  only on its own two changed files, so these unrelated files were left
  untouched and the CI-level fmt redness persists.
- **Owner:** a formatting-sweep follow-up (or the phase gate).
- **Not fixed here:** outside plan 03-04's `files_modified`.

## Status note (03-03 item 1, still open at 03-04)

Plan 03-04 did not touch the 03-01 test artifacts; item 2 above restates the
open fmt redness so the phase gate still sees it.

## From plan 03-05 (executed 2026-09-28)

### 1. `cargo test -p cli --test doctor_json` fails on the audit-tamper case (pre-existing, NOT caused by 03-05)

- **Test:** `audit_verify_rejects_corrupt_chain_without_modifying_database`
- **Result:** `report["tampered_sequences"]` is `Null` where the test expects
  `[<event_count>]`, after the test drops `trg_audit_no_update` and rewrites the
  tail row's `chain_hash` + `reason`. The corrupt run still exits 3 with
  `valid: false`, so the tamper is detected but not reported under
  `tampered_sequences`.
- **Non-causation proof:** the same test fails identically on a read-only
  `git archive` snapshot of the pre-03-05 plan base
  (`2d5878a790cef6375ef441ae59456e99db558cb9`) with none of 03-05's changes
  present. 03-05 touches `quran_cli.rs` (additive `cmd_family`), a new
  `quran_lexicon_api` module, `server` routes, and CLI serve wiring — none of
  which is on the `qai audit verify` path.
- **Owner:** the audit-verification surface (`crates/application/src/audit_bridge.rs`
  `diagnose_invalid_audit` / `verify_persisted_audit` and the audit triggers).
- **Not fixed here:** outside plan 03-05's `files_modified` and outside the
  lexicon surface entirely.

### 2. `docs/08-api/quran-v1-openapi.json` does not document the three new lexicon routes (follow-up)

- **Finding:** `POST /api/v1/quran/family`, `POST /api/v1/quran/count/root-frequency`
  and `POST /api/v1/quran/count/lemma-frequency` are served (and contract-tested
  in `crates/server/tests/api.rs`) but absent from the published v1 OpenAPI
  document. `openapi_spec_covers_every_route` asserts a fixed path list, so it
  stays green and does not catch the drift.
- **Owner:** the API-docs surface; add the three paths plus their
  request/response schemas so the served routes and the spec agree again.
- **Not fixed here:** `docs/08-api/quran-v1-openapi.json` is outside plan 03-05's
  `files_modified`.

### 3. The CLI word-family snapshot exercises the typed-unavailability path only (scope note, not a defect)

- **Finding:** `crates/cli/tests/quran/family_s2.trycmd` pins the command tree,
  the typed `UnavailableDataset` (`QAI-MORPH-0004`, exit 5) and the usage guard.
  It does not pin an attributed relation list, because a morphology dataset can
  only be activated through an approval row and no CLI path mints one — the
  trycmd harness would have to seed the approval + synthetic lexicon in Rust.
  The attributed branch is covered instead by `crates/application/tests/family_goldens.rs`
  (03-04, 154 curated families through `word_family`) and by the new
  `lexicon_family_route_returns_attributed_relations` server contract test.
- **Owner:** nobody urgently; if a CLI-level attributed snapshot is wanted, add
  a seeding helper to `crates/cli/tests/quran.rs` (approval row + synthetic
  lexicon document) as a follow-up.


