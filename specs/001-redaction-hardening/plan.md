# Implementation Plan: Global Redaction Tracing Layer + Sentinel Suite

**Branch**: `001-redaction-hardening` | **Date**: 2026-09-15 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/001-redaction-hardening/spec.md`

## Summary

P0-T16 extends secret-leak protection from type-level (`Secret<T>`) and
audit-level redaction to four new emission surfaces — traced log fields,
diagnostic renderers, `config show`, and doctor JSON — behind one shared
`domain::redaction` helper plus a redacting `FormatFields` wrapper on the
`tracing_subscriber::fmt` layer, switched by the existing (currently dead)
`logging.redact_secrets` flag. Approach per [research.md](research.md):
leaf-crate helper, one justified allowlist line (`observability → domain`),
behavior-preserving `audit` delegation, additive `init_with_options` API.

## Technical Context

**Language/Version**: Rust, workspace `resolver = "2"`, edition 2024
(`rust-toolchain.toml`).

**Primary Dependencies**: `tracing` + `tracing-subscriber` (fmt layer),
`serde_json` (already a `domain` dep), `config` (`Secret<T>`,
`SecretStore`, `logging.redact_secrets`), `audit`
(`redact_audit_value`), `cli` render paths.

**Storage**: N/A — pure in-memory transformation, no migrations, no new
persisted state.

**Testing**: `cargo test --workspace`; sentinel suite extended in
`crates/testkit/tests/secret_leak.rs`; unit tests in `domain`,
`observability`, `audit`; trycmd/schema gates unchanged.

**Target Platform**: local-first `qai` CLI + library crates (same binary).

**Project Type**: Rust workspace feature across `domain`, `observability`,
`audit`, `application`, `cli` (+ `testkit` suite).

**Performance Goals**: secret-free hot paths unchanged — `redact_text`
returns borrowed input when clean (zero-alloc); field-name check is a
small substring scan per recorded field; no per-event allocation blowup.

**Constraints**: `unsafe_code = "forbid"`; `clippy -D warnings`; fmt
clean; `arch-check` green WITH the one-line allowlist amendment;
`migrate-check` untouched; doctor JSON schema (`additionalProperties:
false`) — structure-preserving redaction only; no new external
dependencies (manual pattern scan, no regex crate).

**Scale/Scope**: 4 emission surfaces (US1–US3), 6 new sentinel tests,
1 allowlist line, 1 additive constructor; OTLP span scrubbing explicitly
deferred to follow-up (spec Assumptions).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- I (Canonical integrity): PASS — no canonical paths touched.
- II (Layered trust/provenance): PASS — strengthens Layer E separation;
  no provenance schema changes.
- III (Traceability/reproducibility): PASS — no tool contracts changed;
  doctor schema shape preserved by construction.
- IV (Scholarly honesty): N/A — no scholarly content.
- V (Test-first/gates): PASS — suite-first per FR-006/007; full gate
  sweep in quickstart §6.
- VI (Local-first/deny-by-default): PASS — default-on redaction, flag is
  an opt-out escape hatch; no network/egress changes; no secret values
  leave the process in tests (sentinel is fake).
- VII (Simplicity/architecture): CONDITIONAL PASS — the single
  `observability → domain` allowlist addition is a leaf-ward edge with
  zero cycle risk, recorded with justification here (research D2), not
  silent. `cli` consumes via `application` re-export (composition root).
  No new crates, no new external deps.

Post-design re-check: no new violations introduced beyond the one
declared amendment. GATE OPEN.

## Project Structure

### Documentation (this feature)

```text
specs/001-redaction-hardening/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
│   └── redaction-api.md
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/
├── domain/src/redaction.rs          # NEW: shared helper (Rule A/B/C, marker)
├── domain/src/diagnostic.rs         # MOD: render-time redaction
├── domain/src/lib.rs                # MOD: pub mod redaction
├── audit/src/lib.rs                 # MOD: delegate to domain::redaction
├── observability/src/lib.rs         # MOD: InitOptions + redacting FormatFields
├── observability/Cargo.toml         # MOD: + domain path dep
├── application/src/lib.rs           # MOD: re-export + thread flag in run()
├── cli/src/lib.rs                   # MOD: config-show redact-then-print
├── cli/src/doctor.rs                # MOD: doctor JSON redact-then-print
├── testkit/tests/secret_leak.rs     # MOD: six new sentinel tests
└── xtask/allowlist.toml             # MOD: one justified line (observability → domain)

tests/ — workspace suites above; no new harness.
```

**Structure Decision**: Rust workspace, existing crates only — no new
crate (VII simplicity; helper belongs in the `domain` leaf). Single
project layout; paths above are exact per contracts.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|---|---|---|
| `observability → domain` allowlist addition | Tracing layer must share the exact key matcher + marker with audit/diagnostics | Duplication re-creates the matcher drift this feature closes; placing the helper in `observability` strands `domain::Diagnostic` and `audit` (wrong-direction edges) |
