# Feature Specification: cli command tree

**Feature Branch**: `010-cli-tree`

**Created**: 2026-09-17

**Status**: Draft

**Input**: Module-level specification of the implemented `cli` crate (crates/cli/src/{lib,main,quran,doctor,exit_code}.rs + `qai` binary), derived from code truth on 2026-09-17. Reverse specification of what exists today. Code is the source of truth over plan docs.

**Constitution compliance**: `.specify/memory/constitution.md` v1.0.2, Principles I (I5/I7: destructive verbs require `--yes`), V (trycmd acceptance), VI (render-then-scrub on `config show`/`doctor`, loopback-only serve), VII (doctor never mutates).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - One binary, public exit codes (Priority: P1)

Operators run `qai` for everything: `db migrate/status/verify/backup`, `quran` lifecycle + reads, `config`, `doctor`, `serve`, `audit verify`. Every invocation ends in a public exit code — 0 ok · 1 generic · 2 usage · 3 validation · 4 policy · 5 not found · 6 conflict/state · 7 cancelled · 70 internal — mapped from typed errors (`from_config_error`, `from_io_error`).

**Why this priority**: Exit codes are public API for scripts and harnesses. An unmapped error path returning 0 on failure (or 1 on usage errors) breaks automation silently.

**Independent Test**: `cargo test -p cli --lib` green (16 passed on 2026-09-17); trycmd acceptance `quran/read_flow.trycmd` (migrate→import→activate→reads→translations→v2→validate→diff→rollback→hashes→error exits 5/6).

**Acceptance Scenarios**:

1. **Given** `qai quran get 99:99` (or unknown edition), **When** run, **Then** exit is 5 with a Diagnostic body, not a stack trace.
2. **Given** `qai quran activate …` without `--yes`, **When** run, **Then** exit is 4 (policy) and nothing mutates.

---

### User Story 2 - Full Quran lifecycle from the terminal (Priority: P1)

`qai quran import/validate/diff/activate/rollback/deprecate/hashes` manage editions; `get/context/surah/division/resolve` read them; `translation/gloss` serve attributed companions; `forms rebuild`, `index rebuild/verify`, and `normalize` (profiles/rules/explain/preview) drive the derived layer. Reads support `--json`; destructive verbs require `--yes`.

**Why this priority**: This is the developer-stage primary interface (no Web GUI/TUI yet). If a lifecycle step is missing here, it is missing for operators.

**Independent Test**: `read_flow.trycmd` exercises the full loop including v2 diff and rollback; `quran_forms`/`quran_index` paths covered via application services.

**Acceptance Scenarios**:

1. **Given** a fresh `QAI_DATA_DIR`, **When** running migrate→import→activate→`get 1:1 --json`, **Then** each step exits 0 and the JSON carries edition + version + hash.
2. **Given** `qai quran normalize --show-rule N06`, **When** run, **Then** the rule definition prints without touching any data.

---

### User Story 3 - Read-only, secret-safe doctor (Priority: P2)

`qai doctor [--quran] [--deep] [--json]` runs the check suite (26 checks observed live 2026-09-17, verdicts pass/warn/fail/skipped) as pure reads, scrubbing credential-shaped values from both JSON and human output before printing.

**Why this priority**: Constitution VII (doctor never mutates) + VI (no secret leaks through inspection outputs). A doctor that writes or leaks is worse than no doctor.

**Independent Test**: `doctor --json` parses as a single `{"checks": [...]}` doc (doctor_json test); `doctor_report_scrubs_credential_shaped_config_values` asserts pre-scrub echo so the test cannot go vacuous.

**Acceptance Scenarios**:

1. **Given** config containing credential-shaped values, **When** `doctor --json` runs, **Then** the output parses against the schema with zero secret bytes.
2. **Given** any doctor run, **When** compared before/after at the DB level, **Then** zero rows change.

### Edge Cases

- `secret`, `source`, `job` (partial), `audit list`, `completions` print phase stubs (`phase_stub`) — their presence in `--help` means nothing is implemented behind them; specs must not claim otherwise.
- `audit verify` is real (hash-chain verification); `audit list` is a stub.
- `serve --bind` defaults to loopback; non-loopback binds are refused with exit 4 (test-pinned: `serve_defaults_to_loopback`, `non_loopback_bind_without_tls_fails_validation`).
- `handle_config_get` still prints raw Debug (known wart, outside the 001 scope) — config *show* is scrubbed, config *get* is flagged for a follow-up, not documented as safe.
- `QAI_DATA_DIR` isolates demos; reads never require `--yes`.
- **Not built**: `qai quran search` does not exist (search lives at the Rust service API only).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Crate MUST dispatch `Commands::{Status, Version, Doctor, Config, Db, Secret(stub), Source(stub), Job(stub), Audit(verify real/list stub), Serve, Quran, Completions(stub)}` through `dispatch() -> i32` (`lib.rs` L41–327).
- **FR-002**: Crate MUST implement the quran tree: `Get/Context/Surah/Division/Resolve/Edition{show}/Import/Validate/Diff/Activate/Rollback/Deprecate/Hashes/Translation/Gloss/Forms/Index/Normalize` with `--json` reads and `--yes` destructive gates (`quran.rs` L14–359).
- **FR-003**: Crate MUST map all outcomes to public exit codes 0/1/2/3/4/5/6/7/70 (`exit_code.rs`: `OK/GENERIC/USAGE/VALIDATION/POLICY/NOT_FOUND/CONFLICT/CANCELLED/INTERNAL` + error mappers).
- **FR-004**: Crate MUST route `config show` through `render_config_show` (value → `redact_json_value` → print; Debug text → `redact_text`) and `doctor_report`/`run_checks` through the same scrub boundary before printing (`lib.rs` L385–424, `doctor.rs`).
- **FR-005**: Crate MUST keep doctor read-only (no mutations in any check) and serve loopback-only by default.
- **FR-006**: Crate MUST NOT present stub commands as implemented (stub text names the owning phase).

### Key Entities

- **Commands / QuranAction / EditionAction / TranslationAction / GlossAction / FormsAction / IndexAction**: The clap command tree.
- **CheckResult (pass/warn/fail/skipped)**: Doctor verdicts in the `{"checks": [...]}` document.
- **Exit codes 0–7, 70**: The public machine contract.

### Error Surface

Backend `Diagnostic` bodies propagate (`QAI-QUR-*`, `QAI-DB-*`, `QAI-NORM-*`, `QAI-IDX-*`, …) mapped to exit codes; **no dedicated `QAI-CLI-` Diagnostic codes were observed in `crates/cli/src` on 2026-09-17** — the CLI's contract is the exit-code + Diagnostic-body envelope, matching ADR-0010's exit-code half. Do not document `QAI-CLI-` codes as implemented.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo test -p cli --lib` passes with zero failures (16 passed on 2026-09-17) and trycmd acceptance stays green.
- **SC-002**: `config show --json` parses with all top-level keys and zero secret bytes; `doctor --json` is a single schema-valid doc.
- **SC-003**: Doctor runs change zero database rows (read-only by construction + test).
- **SC-004**: Non-loopback serve binds and approval-less activations are refused (exit 4) in 100% of attempts.

## Assumptions

- `QAI_DATA_DIR` env isolates every demo/test run; destructive verbs always require `--yes` (no implicit confirmation).
- Live smoke 2026-09-17 (fresh temp dir): `db migrate`, `config show` (json+text), `doctor --json` all green; older docs citing other doctor-check counts are stale — 26 observed live wins.
- Search CLI, secret/source/job management, and completions are explicitly unbuilt (stubs); TUI/Web GUI are separate future surfaces.
- No canonical text generation anywhere in this crate; all verse output flows from application reader services.
