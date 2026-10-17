---
phase: 07-isnad-narrator-research
plan: 02
subsystem: isnad-narrator-research
tags: [cli, isnad, narrator, identity]
requirements: [REQ-isnad-narrator-graph]
status: complete
key-files:
  created:
    - crates/cli/src/isnad.rs
  modified:
    - crates/cli/src/lib.rs
    - crates/cli/Cargo.toml
    - crates/server/src/api.rs
metrics:
  tasks: 2
  commits: 1
---

# Phase 7 Plan 02: CLI Isnad Commands Summary

Implemented `qai isnad` command tree (narrator/rijal/trace/identity propose/accept/reject/dispute/supersede) mirroring hadith/links CLI pattern, with CommandOutput human/json split and ToolResult envelopes carrying research_checksum. Wired IsnadAction into CLI command tree and dispatch. Fixed pre-existing server isnad dispatch compilation (String match, ? in Result helper) to allow cli build.

## Deviations from Plan

- Supersede CLI uses service-generated new assertion (proposal fields optional in practice); `--new-id` accepted per plan but new assertion is minted by backend.
- Fixed server/src/api.rs run_isnad_dispatch bad-params handling (Result-returning helper) and cli serve arm ordering to compile.

## Self-Check: PASSED
