---
phase: 03-quran-search-linguistics
plan: 06
subsystem: quran-linguistics
tags: [rust, sqlite, quran-morphology, license, attribution, d-07, g-04, od-11, adr-0203, fail-closed, tdd]

# Dependency graph
requires:
  - phase: 02-canonical-quran-core
    provides: quran_datasets (0017) carrying license_status/license_json/attribution, the approval-gated activate_morphology gate chain, MorphologyJobError (QAI-MORPH-*) and MorphologyImportParams, quran-morphology as a pure no-I/O crate
  - phase: 03-quran-search-linguistics
    provides: 03-04 the complete family engine + affix fail-closed path; 03-05 the family/frequency surfaces that read the activated dataset
provides:
  - "crates/quran-morphology/src/license.rs: pure LicenseEvidence validation (permissive-status allowlist PublicDomain/OpenLicense/PermissionGranted/UserOwned; mandatory capture fields source_url/capture_date/capturer; redistribution_allowed: true), typed LicenseEvidenceError naming every missing field"
  - "activation gate 6 in activate_morphology: reads the dataset row, rolls back and returns MorphologyJobError::LicenseEvidence (QAI-MORPH-0006) when evidence is absent/non-permissive — before any promotion"
  - "import-side persistence: run_morphology_import writes the dataset row (state staged) with the operator's license/attribution instead of hardcoding Unspecified at activation; an already-active dataset's evidence is never clobbered"
  - "CLI: qai quran morphology import gains --license-status / --license-json / --license-evidence (matrix or capture file); missing/unreadable/invalid evidence is a typed usage error; status derived from spdx_id/redistribution_allowed when absent"
  - "fixtures/quran/morphology/license-matrix.json: machine-readable per-artifact license matrix (synthetic permissive entry; qac pending_license_review) consumed by gate/tests and ratified by the owner"
  - "crates/application/tests/morphology_import.rs: license_gate_rejects, license_gate_passes, license_gate_matrix_is_valid_json_with_capture_fields"
affects: [03-07, 03-08, phase-04-graph, phase-05-result-contract, phase-10-agent-runtime]

# Actuals (#2632) — same estimateTokens scale as the plan's estimate (chars/4 over the realized diff)
actuals:
  tokens: 10253
  tasks: 3
  commits: 4   # plan's own atomic commits (87c7360, a70cca3, e015800, 324d043)
  # NOTE: the ledger window plan_head_before..HEAD also contains one concurrent-session
  # commit (934af8e feat(01-05)) landed by another workstream on the shared tree.
  plan_head_before: 8f324415cb85458b68bce0093076b1b53278e30e

tech-stack:
  added: []
  patterns:
    - "License evidence is validated structurally (fields present, redistribution allowed) and the *status* is decided by an explicit permissive allowlist — absence is never a default-true"
    - "Fail-closed activation: the evidence row is written by the importer (state staged) and only the gate can flip state active, so no code path reaches active without validated evidence"
    - "Purity fence preserved: license.rs has no I/O; the file read lives in the CLI adapter, so quran-morphology stays dependency-pure (arch-check OK)"
    - "Machine-readable policy matrix consumed by both the gate tests and the CLI, with the unverified provider explicitly pending (never bundled)"

key-files:
  created:
    - crates/quran-morphology/src/license.rs
    - fixtures/quran/morphology/license-matrix.json
  modified:
    - crates/quran-morphology/src/lib.rs
    - crates/application/src/quran_morphology.rs
    - crates/application/src/quran_cli.rs
    - crates/cli/src/quran.rs
    - crates/cli/src/lib.rs
    - crates/application/tests/morphology_import.rs
    - crates/application/tests/counting.rs
    - crates/application/tests/family_goldens.rs
    - .planning/WINDOWS.md

key-decisions:
  - "The license evidence reaches the gate through the *dataset row*: run_morphology_import now upserts the row in state 'staged' with the operator's license_status/license_json/attribution, and activate_morphology reads it via get_dataset, validates, and only then flips state to 'active'. This is the only plumbing that fits the plan's 'no migration' constraint (the row already carries the columns)."
  - "Promotion no longer re-upserts the dataset row (which previously hardcoded license_status: \"Unspecified\"/license_json: \"{}\"): it calls set_dataset_state(active). upsert_dataset does not update state ON CONFLICT, so an explicit transition is required anyway, and re-upserting would have re-introduced the placeholder and bypassed the gate (T-03-22)."
  - "Re-import never clobbers an already-active dataset's validated evidence: the import-side upsert is skipped when the existing row is 'active'."
  - "QAI-MORPH-6 is the append-only diagnostic code for the license gate; morphology_exit maps LicenseEvidence to exit::VALIDATION (3), the same class as BlockingFindings."
  - "Permissive allowlist = PublicDomain/OpenLicense/PermissionGranted/UserOwned (the permissive half of domain::LicenseStatus). Unspecified/metadata_only/pending_license_review/Unknown/empty and any unrecognized status are refused; redistribution_allowed is the binding bundling gate (licenses/README.md)."
  - "The CLI derives a status from the evidence file only when --license-status is absent: spdx_id present → OpenLicense; else redistribution_allowed: true → PermissionGranted; else Unspecified (which the gate rejects). It never invents permissiveness."
  - "Task 3's fixture is loaded with include_str! so the matrix-validity test fails to compile if the committed matrix is removed — the artifact cannot silently disappear."
  - "OD-11 stays BLOCKED and is not silently passed: the qac matrix entry is pending_license_review/redistribution_allowed:false and QAC ships only via user-supplied import until licenses/qac/ is captured."

patterns-established:
  - "A pure policy module (typed parse + allowlist + require_* predicate) consulted by an application-layer gate, unit-tested by name filter and integration-tested fail-closed + pass"
  - "Evidence written at import, gated at activation: the state transition is the only place the policy is enforced, so every activation path shares one gate"
  - "Machine-readable license matrix as the single source of record for both code and tests, with non-permissive entries explicitly pending"

requirements-completed: [REQ-quran-linguistics]

# Coverage metadata (#1602)
coverage:
  - id: D1
    description: "A dataset cannot reach state='active' unless its license evidence has a recognized permissive status AND the mandatory capture fields (source_url, capture_date, capturer) plus redistribution_allowed: true; absence/malformed/non-permissive evidence fails with a typed error before any promotion."
    requirement: REQ-quran-linguistics
    verification:
      - kind: unit
        ref: "cargo test -p quran-morphology --lib license_evidence (license_evidence_accepts_captured_evidence, _rejects_unspecified, _rejects_missing_capture_fields, _rejects_redistribution_false)"
        status: pass
      - kind: integration
        ref: "cargo test -p application --test morphology_import -- license_gate (license_gate_rejects, license_gate_passes)"
        status: pass
    human_judgment: false
  - id: D2
    description: "The CLI stops hardcoding the Unspecified status and empty-object license JSON: operators supply --license-status/--license-json/--license-evidence on `qai quran morphology import`; a missing/unreadable/invalid evidence file is a typed usage error, and an evidence file's capture fields populate the dataset row."
    requirement: REQ-quran-linguistics
    verification:
      - kind: unit
        ref: "cargo test -p application --lib license_evidence (6 quran_cli::license_evidence_tests)"
        status: pass
      - kind: other
        ref: "cargo build -p cli; cargo run -q -p cli -- quran morphology import --help (exposes all three flags)"
        status: pass
    human_judgment: false
  - id: D3
    description: "A machine-readable per-artifact license matrix exists at fixtures/quran/morphology/license-matrix.json, is valid JSON, every entry carries the mandatory capture field keys, and the unverified provider (qac) stays pending_license_review/redistribution_allowed:false."
    requirement: REQ-quran-linguistics
    verification:
      - kind: integration
        ref: "cargo test -p application --test morphology_import -- license_gate_matrix_is_valid_json_with_capture_fields"
        status: pass
    human_judgment: false
  - id: D4
    description: "Ratification of the actual QAC license entry and of the morphology dataset selection (OD-11 / ADR-0203)."
    requirement: REQ-quran-linguistics
    verification: []
    human_judgment: true
    rationale: "The gate validates license evidence *structurally* (fields present, redistribution allowed) — it cannot certify that the captured terms are the rights holder's true terms. OD-11 remains BLOCKED: no licenses/qac/ capture has been made and no owner has ratified ADR-0203 Option A vs B, so the qac matrix entry stays pending and nothing from QAC is bundled or asserted as sourced."

# Metrics
duration: 27 min
completed: 2026-09-28
status: complete
---

# Phase 3 Plan 06: D-07 License/Attribution Matrix + Activation-Rejection Gate Summary

**A morphology dataset can no longer become `active` without verified license evidence: a pure `LicenseEvidence` validator, an import-written dataset row carrying operator evidence, activation gate 6 (`QAI-MORPH-0006`, fail-closed with rollback), the three CLI license flags, and a machine-readable `license-matrix.json` are all in place — with `qac` explicitly pending rather than silently bundled.**

## Performance

- **Duration:** 27 min
- **Started:** 2026-09-28T05:06:30Z (plan ledger base `8f32441`)
- **Completed:** 2026-09-28T05:33:21Z
- **Tasks:** 3/3
- **Files modified:** 10 code/fixture files (2 created, 8 modified) + `.planning/WINDOWS.md`

## Accomplishments

- **D-07/G-04 closed in code:** `activate_morphology` now has a license gate (gate 6) after the alignment gate and before promotion. It reads the dataset row, builds `LicenseEvidence`, and on any failure calls `uow.rollback()` and returns `MorphologyJobError::LicenseEvidence` (`QAI-MORPH-0006`) — nothing is promoted and no dataset flips active. A dataset with `Unspecified`, `metadata_only`, `pending_license_review`, `Unknown`, an empty status, a missing capture field, or `redistribution_allowed: false` cannot activate.
- **`licenses/README.md` is now a real gate, not prose:** the mandatory capture fields (`source_url`, `capture_date`, `capturer`) and `redistribution_allowed: true` are enforced by `crates/quran-morphology/src/license.rs`, a pure no-I/O module (arch-check OK). Missing fields are named individually in the typed error.
- **The CLI stops hardcoding `Unspecified`:** `qai quran morphology import` exposes `--license-status`, `--license-json`, and `--license-evidence`; the evidence file's capture fields populate the dataset row and a bad path is a typed usage error (never a silent default). The import persists the row (state `staged`); only the gate flips it `active`.
- **Machine-readable matrix shipped:** `fixtures/quran/morphology/license-matrix.json` is the per-artifact source of record — a permissive `synthetic-test-lexicon` entry (CC0-like, `synthetic_test_only`) and a `qac` entry pinned `pending_license_review` / `redistribution_allowed: false`.
- **Fail-closed AND functional, proven:** `license_gate_rejects` proves both refusal paths (status and redistribution flag) promote zero rows and activate zero datasets; `license_gate_passes` proves permissive + full capture fields activates and the read tools return attributed analyses; `license_gate_matrix_is_valid_json_with_capture_fields` proves the matrix is well-formed and the pending entry never bundles.
- **No migration:** the dataset row already carried `license_status`/`license_json`/`attribution`; the only new state use is `staged` (already in the `0017` CHECK). `xtask migrate-check` reports 21 ordered migrations, checksums stable.

## Task Commits

Each task was committed atomically (Task 1 is a TDD task: RED → GREEN):

1. **Task 1: Pure license-evidence module + activation gate (G-04)**
   - `87c7360` (test — RED: `license.rs` API stub + 4 `license_evidence_*` unit tests, 4/4 failing)
   - `a70cca3` (feat — GREEN: real validation, gate 6, import-side evidence persistence, state-based promotion)
2. **Task 2: Thread operator license evidence through the import CLI (G-04)** - `e015800` (feat)
3. **Task 3: Machine-readable license matrix + fail-closed/pass activation tests (G-04)** - `324d043` (test)

**Plan metadata:** (this commit) `docs(03-06): complete [plan-name] plan`

## Files Created/Modified

- `crates/quran-morphology/src/license.rs` (new) — `LicenseEvidence` + `LicenseEvidenceError`; `PERMISSIVE_LICENSE_STATUSES` / `REJECTED_LICENSE_STATUSES`; `is_permissive_status`; `from_status_and_json` (parses the capture, names every missing field); `is_activation_allowed` / `require_activation_allowed`; four in-module `license_evidence_*` tests.
- `crates/quran-morphology/src/lib.rs` — `pub mod license;`.
- `crates/application/src/quran_morphology.rs` — new `MorphologyJobError::LicenseEvidence` variant + `QAI-MORPH-6` code + remedy; import-side dataset-row upsert (`state: staged`, license/attribution from params, skipped when the existing row is active); gate 6 in `activate_morphology`; promotion via `set_dataset_state(active)`; audit event records `license_status`; `license_gate_error` helper.
- `crates/application/src/quran_cli.rs` — `cmd_morphology_import` signature gains the three license args; `resolve_license_evidence` / `select_evidence_entry` / `derive_license_status`; `morphology_exit` maps the gate to `exit::VALIDATION`; 6 unit tests (`license_evidence_*`).
- `crates/cli/src/quran.rs` — `MorphologyAction::Import` gains `--license-status`/`--license-json`/`--license-evidence`; dispatch threads them.
- `crates/cli/src/lib.rs` — `#[allow(clippy::large_enum_variant)]` on `Commands` (deviation; see below).
- `crates/application/tests/morphology_import.rs` — `import_params` now carries the mandatory capture fields; `LICENSE_MATRIX` (include_str!); `license_gate_rejects`, `license_gate_passes`, `license_gate_matrix_is_valid_json_with_capture_fields`.
- `crates/application/tests/counting.rs`, `crates/application/tests/family_goldens.rs` — their `import_params` helpers carry the mandatory capture fields (required wiring; see deviations).
- `fixtures/quran/morphology/license-matrix.json` (new) — the D-07 machine-readable matrix.
- `.planning/WINDOWS.md` — recorded the Task-3 TDD note, the cli lib lint deviation, and the pending `qac` license item.

## Decisions Made

- **Evidence travels through the dataset row (no migration).** The importer writes the row `staged` with the operator's evidence; the gate reads it and only then flips `active`. This satisfies the plan's "no migration" constraint and gives every activation path one enforcement point.
- **Promotion no longer re-upserts the row.** The old activation hardcoded `license_status: "Unspecified"` / `license_json: "{}"`; that is exactly what G-04 flagged. `upsert_dataset` does not change `state` on conflict, so `set_dataset_state(active)` is both required and cleaner — and re-upserting would have re-introduced the placeholder, bypassing the gate (T-03-22).
- **Re-import does not clobber an active dataset.** The import-side upsert is skipped when the existing row is `active`, so a validated dataset's evidence can never be downgraded to an unvalidated value without going through the gate.
- **Permissive allowlist = the permissive half of `domain::LicenseStatus`.** `PublicDomain`/`OpenLicense`/`PermissionGranted`/`UserOwned`; everything else (including the import placeholders and any unrecognized string) is refused. `redistribution_allowed` remains the binding bundling gate.
- **`QAI-MORPH-6` (append-only) and `exit::VALIDATION`.** The gate is a policy refusal like `BlockingFindings`, so the CLI exit code is 3, not the generic INTERNAL.
- **OD-11 stays BLOCKED.** The `qac` entry is pending and no QAC data is bundled; the gate validates structure, not the truth of the captured terms — that remains the owner's ratification (D4, human_judgment).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] The activation gate needs evidence on the dataset row, so the importer must write it**
- **Found during:** Task 1 (activation gate)
- **Issue:** The plan reads the license via `uow.quran().get_dataset(slug, version)` inside `activate_morphology`, but the dataset row was only ever created *by* `activate_morphology` (hardcoded `Unspecified`). With the row absent, `get_dataset` returned `None` and every activation would fail closed — including the plan's own pass case.
- **Fix:** `run_morphology_import` now upserts the dataset row (state `staged`) with the operator's `license_status`/`license_json`/`attribution` in the write-staging transaction; `activate_morphology` gates on it and flips state to `active` via `set_dataset_state`. This is the plumbing the plan's key_links anticipated ("MorphologyImportParams already carries license_status/license_json/attribution; only plumbing is missing").
- **Files modified:** `crates/application/src/quran_morphology.rs`
- **Verification:** `cargo test -p application --test morphology_import` (15 pass, including the new pass/reject cases).
- **Committed in:** `a70cca3` (Task 1 GREEN)

**2. [Rule 3 - Blocking] Three integration-test import helpers would have blocked every pre-existing activation**
- **Found during:** Task 1 (activation gate)
- **Issue:** The plan named only `crates/application/tests/morphology_import.rs`'s `import_params` as a call site, but `crates/application/tests/counting.rs` and `crates/application/tests/family_goldens.rs` each have their own `import_params` helper with `license_json: "{}"`. Under gate 6 those activations would fail closed, breaking their suites.
- **Fix:** All three helpers now supply the mandatory capture fields (`source_url`/`capture_date`/`capturer`, `redistribution_allowed: true`) with `license_status: "PublicDomain"`, so every activation call site satisfies the gate.
- **Files modified:** `crates/application/tests/morphology_import.rs`, `crates/application/tests/counting.rs`, `crates/application/tests/family_goldens.rs`
- **Verification:** `cargo test -p application --test morphology_import --test counting --test family_goldens` → 15 + 9 + 1 pass.
- **Committed in:** `a70cca3` (Task 1 GREEN)

**3. [Rule 3 - Blocking] Three new CLI flags tripped `clippy::large_enum_variant`**
- **Found during:** Task 2 (CLI flags)
- **Issue:** Adding three `Option<String>` fields to `MorphologyAction::Import` grew `QuranAction`, which is nested in the top-level `Commands` enum, past clippy's large-variant threshold — `cargo clippy -D warnings` failed.
- **Fix:** Added a targeted `#[allow(clippy::large_enum_variant)]` (with a comment) to `Commands`. The CLI subcommand shape is inherently large; boxing would change the clap derive surface for no behavioural gain.
- **Files modified:** `crates/cli/src/lib.rs`
- **Verification:** `cargo clippy -p quran-morphology -p application -p cli --all-targets -- -D warnings` clean.
- **Committed in:** `e015800` (Task 2)

**4. [TDD note] Task 3 is `tdd="true"` but its subject is delivered by Task 1**
- **Found during:** Task 3 (matrix + tests)
- **Issue:** Task 3 adds only a fixture and integration tests for the gate implemented in Task 1, so a distinct RED commit (failing test first) was not achievable within the plan's decomposition.
- **Resolution:** Landed as a single `test` commit. The tests are non-vacuous: `license_gate_rejects` pins the typed error code `QAI-MORPH-0006` and asserts zero promoted rows / zero active datasets, and `license_gate_passes` asserts promoted counts and attributed reads. Recorded in `.planning/WINDOWS.md`.
- **Committed in:** `324d043` (Task 3)

---

**Total deviations:** 3 auto-fixed (3 blocking) + 1 TDD note.
**Impact on plan:** No scope creep: no new dependencies (T-03-SC stays accepted), no migration, no canonical/hash recipe touched. Every declared `files_modified` entry was touched; the extra files are required integration wiring (`cli/src/lib.rs` lint, two test helpers) called out above.

## Issues Encountered

- **Clippy `large_enum_variant` (see deviation 3).** Resolved with a targeted allow rather than boxing the subcommand (which would change the CLI arg surface).
- **One transient CLI test failure under load.** `serve_hosts_the_worker_and_shuts_down_joined` timed out once (90 s, job still `Queued`) during the first full `cargo test -p cli --test quran` while the shared tree was under concurrent cargo load; it passed standalone in 5.3 s and the re-run of the whole suite passed 14/14 in 10 s. Not a regression — a shared-tree contention artifact of the kind 03-05 also recorded.
- **`cargo run -q -p cli -- quran morphology import --help`** confirms the three new flags render; no CLI snapshot covers the morphology import help, so no snapshot changed.

## User Setup Required

None — no external service configuration, no new dependencies, no package installs (`T-03-SC` remains accepted). The qac license capture is an owner gate (OD-11), not a setup step for this plan.

## Known Stubs

One intentional, documented non-permissive placeholder — not code:

- `fixtures/quran/morphology/license-matrix.json` → the `qac` artifact entry is `license_status: "pending_license_review"`, `redistribution_allowed: false`, with `capture_date`/`capturer` null. It is a deliberate "do not bundle" marker (OD-11 BLOCKED), asserted by `license_gate_matrix_is_valid_json_with_capture_fields`; the synthetic entry is fully permissive. No code stub flows to any output; the license gate is fail-closed by construction.

## Threat Flags

None beyond the plan's own register. The changes add no new trust boundary of an uncovered class: `T-03-22` (fail-closed activation gate, proven by `license_gate_rejects`), `T-03-23` (evidence only from operator files; the audit event now records `license_status`; nothing inferred), `T-03-24` (typed `LicenseEvidence` parsing, malformed/missing → typed error, module pure), and `T-03-25` (the gate only reads the dataset row and writes no canonical/hash state; MV-018 untouched; migrate-check stable) are each mitigated. The CLI's `--license-evidence` file read is the operator-supplied-file boundary already named in the register.

## Owner Gates (BLOCKED — not silently passed)

- **OD-11 (morphology dataset selection & licensing, ADR-0203)** — BLOCKED. Implemented here as a gate, not resolved: the `qac` matrix entry stays `pending_license_review` and QAC ships only via the user-supplied import path until `licenses/qac/{LICENSE.txt,capture.json,attribution.txt}` is captured (source_url + capture_date + capturer mandatory). Closing step: capture the QAC license; if `redistribution_allowed: true` with all fields, set the matrix entry permissive and ratify ADR-0203 Option A; otherwise keep Option B; record the decision + date in `docs/05-followups/owner-decisions.md`.
- **OD-12 (normalization catalog + named linguist, ADR-0204/0205/0210/0211/0215)** — BLOCKED (unchanged by this plan). The dataset this gate protects is still the synthetic `synthetic_test_only` lexicon; nothing linguistic is ratified here.

## Next Phase Readiness

- D-07 is enforced: no code path can activate a morphology dataset without permissive, captured license evidence. 03-07/03-08 and the graph phases can rely on the `active` dataset always having validated evidence and a recorded `license_status`.
- The matrix (`fixtures/quran/morphology/license-matrix.json`) and the pure `LicenseEvidence` module are reusable for edition/translation bundling gates (OD-01/OD-04) if a later phase chooses to generalize.
- **Open follow-ups:** the `qac` matrix entry is pending (OD-11); the Task-3 TDD note and the `cli` lint deviation are in `.planning/WINDOWS.md`; the pre-existing out-of-scope failures (`application --test quran_identity` 2× `NotStaged`, `cli --test doctor_json` audit-tamper) remain untouched.
- Verification snapshot: `cargo test -p quran-morphology --lib license_evidence` (4 pass), `cargo test -p application --lib` (51 pass incl. 6 new), `cargo test -p application --test morphology_import` (15 pass), `--test counting` (9 pass), `--test family_goldens` (1 pass), `cargo test -p cli --test quran` (14 pass), `cargo build -p cli` (ok), `cargo clippy -p quran-morphology -p application -p cli --all-targets -- -D warnings` (clean), `rustfmt --check` on all touched files (clean), `xtask migrate-check` (OK), `xtask arch-check` (OK).

---
*Phase: 03-quran-search-linguistics*
*Completed: 2026-09-28*
