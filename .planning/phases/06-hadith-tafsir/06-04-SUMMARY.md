---
phase: 06-hadith-tafsir
plan: 04
subsystem: ingestion
tags: [hadith, tafsir, ingestion, cross-surface, cli, http-api, parity]
requirements-completed: [REQ-hadith-corpus, REQ-tafsir-comparative]
status: complete
actuals:
  tokens: 45000
  tasks: 3
  commits: 3
  plan_head_before: b78b84d19eba9b6a31a80ce2c225c1955de6b2e0
  plan_head_after: 6c7d86d7d6ec95434484d86bdd9e60208185c65c
---

# Phase 06 Plan 04: Cross-surface parity, CLI/API routes, and ingestion pipeline Summary

**Cross-surface hadith/tafsir tooling completed with ingestion pipeline, CLI commands, and HTTP API routes.**

## Accomplishments

- Implemented `hadith-ingestion` parser/validator/pipeline (discover → stage → validate → activate)
- Added `qai hadith` and `qai tafsir` CLI commands calling the application service layer
- Added `/api/v1/hadith/*`, `/api/v1/tafsir/*`, and `/api/v1/verse/{surah}/{ayah}/links` routes
- Preserved canonical-text mutability and no-consensus-grading constraints

## Commits

- 0ab664c: feat(06-04): hadith/tafsir ingestion pipeline
- bf8a651: feat(06-04): CLI commands for hadith and tafsir
- 6c7d86d: feat(06-04): HTTP API routes for hadith/tafsir and verse links

## Verification

- `cargo test -p hadith-ingestion` passed
- `cargo test -p cli` passed
- `cargo test -p server` passed
- `cargo test -p cli --test parity_cross_surface` passed

## Known Stubs

- `verse_links_handler` returns an empty list until `AppState` gains a database handle; deferred to keep API compilation stable.

## Self-Check: PASSED
EOF