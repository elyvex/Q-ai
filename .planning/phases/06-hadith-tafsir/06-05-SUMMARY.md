---
phase: 06-hadith-tafsir
plan: 05
subsystem: integration
tags: [hadith, tafsir, verse-links, ingestion, cross-surface, arch-check]
requirements-completed: [REQ-hadith-corpus, REQ-tafsir-comparative]
status: complete
actuals:
  tokens: 30000
  tasks: 3
  commits: 2
  plan_head_before: 84eaed89f91354238bd82dcfe944744e914a09f6
  plan_head_after: 546b832b5aff61362a7b5c08c7e6b7f1c1e7eeff
---

# Phase 06 Plan 05: SC3 verse-links read surface + ingestion tests summary

**Closed the two gaps from 06-VERIFICATION.md: user-facing verse navigation (HTTP + CLI) and hadith-ingestion arch-check + smoke tests.**

## Accomplishments

- Added `crates/cli/src/links.rs` with `qai links verse <surah> <ayah>` wired through `VerseLinkService::get_links_for_verse` (include_uncertain flag).
- Wired `AppState.verse_links: Option<Arc<VerseLinkService>>` in `crates/cli/src/lib.rs` (constructed from the serve DB), and rewrote `crates/server/src/api.rs::verse_links_handler` to consult it (returning confirmed-only by default, or all links when `include_uncertain=true`).
- Updated server tests (`api`, `citation_open`, `graph`) and `crates/cli/tests/parity_cross_surface.rs` for the new `verse_links` field.
- Added hadith-ingestion crate tests (10 tests: parser, validation, discover→stage→validate→activate smoke + consensus-merged grading rejection).
- Arch-check already green from the allowlist fix (84eaed8); re-run confirms zero forbidden edges.

## Commits

- 5e3362c: feat(06-05): wire SC3 verse-links read surface (HTTP + CLI)
- 546b832: test(06-05): hadith ingestion pipeline coverage (parse, validate, activate)

## Verification

- `cargo build -p cli -p server` passed
- `cargo test -p hadith-ingestion` passed (10 tests)
- `cargo test -p server` passed (11 tests)
- `cargo xtask arch-check` passed

## Known Deferrals

- HTTP/TUI parity three-leg coverage for hadith/tafsir tool rows in `crates/cli/tests/parity_cross_surface.rs` remains a manual/human verification item (test-coverage only; behavior attested by shared dispatch + research_checksum shape). Deferred to Phase 8/11.
- Ingestion pipeline remains in-memory; SQLite persistence deferred (per 06-04 contract).

## Self-Check: PASSED
EOF