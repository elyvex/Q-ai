# Specification Quality Checklist: cli command tree

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-09-17
**Feature**: [spec.md](./spec.md)

## Content Quality

- [x] No implementation details (languages, frameworks, APIs) — EXCEPTION JUSTIFIED: reverse specification of an implemented crate; references grounded in read skeletons (`lib.rs`) and symbol greps (`quran.rs`, `doctor.rs`, `exit_code.rs`).
- [x] Focused on user value and business needs — operators: one binary, scriptable exits, safe lifecycle, honest stubs.
- [x] Written for non-technical stakeholders — scenarios in plain language.
- [x] All mandatory sections completed.

## Requirement Completeness

- [x] No [NEEDS CLARIFICATION] markers remain.
- [x] Requirements are testable and unambiguous — FRs name enums/functions plus gates (16 tests, trycmd, live smoke).
- [x] Success criteria are measurable — counts and refusal guarantees quoted from 2026-09-17 state.
- [x] Success criteria are technology-agnostic — outcomes stated (refused, zero rows changed, parses clean).
- [x] All acceptance scenarios are defined.
- [x] Edge cases are identified — stubs named, `config get` wart flagged (not hidden), missing search CLI stated.
- [x] Scope is clearly bounded — unbuilt surfaces explicitly listed as stubs.
- [x] Dependencies and assumptions identified — `QAI_DATA_DIR` isolation, live-smoke evidence, stale doc counts.

## Feature Readiness

- [x] All functional requirements have clear acceptance criteria — via SC-001…SC-004.
- [x] User scenarios cover primary flows — exits, lifecycle, doctor.
- [x] Feature meets measurable outcomes defined in Success Criteria.
- [x] No implementation details leak into specification — beyond the justified crate-spec exception.

## Notes

- Invariant check 2026-09-17: I5/I7 (`--yes` gates), VII (read-only doctor, loopback serve), VI (render-then-scrub) all hold. `config get` raw-Debug wart recorded as follow-up, not documented as safe. No `QAI-CLI-` codes invented. No fabricated content. No Tantivy claims.
- Pre-spec gate evidence: `cargo test -p cli --lib` 16 passed; `arch-check` OK.
- Ready for final spec in the loop (`server` 011).
