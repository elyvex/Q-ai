---
phase: 08-multi-rag-platform
plan: 06
subsystem: validation
tags: [parity, owner-gates, deferrals, validation-map, phase-gate]
requires:
  - plan: 08-05
    provides: tombstoning propagation
provides:
  - Cross-surface parity (rag_parity_service passes)
  - Owner-gate + deferral ledgers
  - Validation map
  - Full phase gate green
affects: []
tech-stack:
  added: []
  patterns: [parity trycmd, validation map]
key-files:
  created:
    - crates/cli/tests/rag_parity.trycmd
    - crates/cli/tests/rag_parity.rs
    - crates/cli/tests/rag_parity_runner.rs
    - .planning/phases/08-multi-rag-platform/08-VALIDATION.md
    - docs/05-followups/phase-08-owner-gates.md
    - docs/05-followups/phase-08-deferrals.md
  modified: []
key-decisions:
  - "Phase 8 introduces no new owner gates; ADR-0701/0301 inherited undecided"
  - "PG/Qdrant adapters, remote providers, multi-tenant deferred"
patterns-established:
  - "Validation map follows Phase 3/7 format"
requirements-completed: [REQ-multi-rag, REQ-generic-rag-platform]
---

# 08-06 SUMMARY — Cross-surface parity, owner gates, deferrals, validation, phase gate

Cross-surface parity test passes (`rag_parity_service`). Owner-gate and deferral
ledgers written. Validation map complete. Migrate-check (38 migrations) and
arch-check pass. Phase 8 workspace tests for changed crates green.
