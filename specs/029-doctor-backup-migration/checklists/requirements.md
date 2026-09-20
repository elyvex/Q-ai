# Specification Quality Checklist: Doctor, Backup/Restore, and Checksumed Migration Discipline

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-09-18
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

## Requirement Completeness

- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Success criteria are technology-agnostic (no implementation details)
- [x] All acceptance scenarios are defined
- [x] Edge cases are identified
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

## Feature Readiness

- [x] All functional requirements have clear acceptance criteria
- [x] User scenarios cover primary flows
- [x] Feature meets measurable outcomes defined in Success Criteria
- [x] No implementation details leak into specification

## Notes

- Reverse specification of implemented doctor / backup-restore / migration discipline; code is source of truth (`crates/cli/src/doctor.rs`, `crates/application/src/quran_doctor.rs`, `crates/application/src/db.rs`, `crates/storage-sqlite/src/migrate.rs`, `migrations/sqlite/0001`–`0016` + `checksums.json`, `xtask/src/migrate.rs`, `xtask/src/arch.rs`). No clarifications needed — behavior pinned by code.
- Known code-truth asymmetries pinned as-is (not clarifications): `database.integrity_check` remedy points at `qai db backup` while recovery requires restore; `.qai-probe` filesystem writes are the only doctor-adjacent writes and never touch the database; down files exist only for `0001`–`0006`.
- Validation pass 1: all items pass. Ready for `/speckit-clarify` (optional) or `/speckit-plan`.
