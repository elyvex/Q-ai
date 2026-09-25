---
phase: 02-canonical-quran-core
plan: 03
subsystem: api
tags: [quran, cli, doctor, integrity, reference-corpus, qv-015, adr-0114, edition-identity, license, primary, read-only]

requires:
  - phase: 02-canonical-quran-core
    provides: plan 02-01's manifest-declared upstream identity / qai_edition_id / is_primary / declared license, and plan 02-02's importable rich + identity fixtures and companion reference
  - phase: 01-foundations
    provides: the 19-check quran doctor engine, generation-keyed reader, QV-015 importer path, and the trycmd CLI harness

provides:
  - qai quran verify — a read-only verb classifying the six integrity families (counts, addressing, unicode, checksums, roundtrip, reference_comparison) from persisted state
  - a state-derived quran.reference_corpus doctor check (pass/skipped/fail; a skip is never a pass)
  - the stored per-ayah canonical hash on the qai quran get human line
  - qai quran import --reference <path> — real QV-015 evaluation through the operator path, byte-only fail-closed, with per-difference ADR-0114 DifferenceClass metadata
  - qai quran edition show reporting the declared upstream identity, primary marker, and declared license status
  - crates/application/tests/quran_reference.rs and three new CLI trycmd snapshots

affects: [02-07 corpus-integrity artifact, phase 3 operator surfaces that read the six families]

actuals:
  tokens: 14583
  tasks: 3
  commits: 3
  plan_head_before: 904441c8d8fb80412ad57bdcab3d9a1f7b98c9c8

tech-stack:
  added: []
  patterns:
    - "Read-only operator surface over an existing engine: a new verb groups the 19 doctor checks into six named families and derives family status from persisted state instead of reimplementing integrity logic."
    - "State-derived evidence: a doctor check reads the persisted validation finding rather than hardcoding a warning; a skipped family is serialized as skipped and never as pass."
    - "Job-payload threading: an operator-supplied artifact (the reference corpus) rides the existing import payload into run_import rather than opening a second import path."

key-files:
  created:
    - crates/cli/tests/quran/verify.trycmd
    - crates/cli/tests/quran/reference.trycmd
    - crates/cli/tests/quran/edition_identity.trycmd
    - crates/application/tests/quran_reference.rs
  modified:
    - crates/application/src/quran_cli.rs
    - crates/application/src/quran_doctor.rs
    - crates/application/tests/quran_doctor.rs
    - crates/cli/src/quran.rs
    - crates/cli/tests/quran.rs
    - crates/cli/tests/quran/read_flow.trycmd
    - crates/quran-corpus/src/import.rs
    - crates/application/tests/*.rs (ImportInput constructors gain reference_manifest_text: None)

key-decisions:
  - "qai quran verify derives all six families from the existing run_quran_checks; the reference family is state-derived from the persisted QV-015 finding and a skip is never reported as pass (D-10)."
  - "The operator reference corpus travels on the import payload (ImportInput.reference_manifest_text) and is parsed into ImportOptions.reference inside run_import; QV-015 stays byte-only fail-closed and typed DifferenceClass is report metadata only (ADR-0114 §4)."
  - "A failed reference comparison clears staging and maps to exit 3, so the stage is left untouched and the operator sees a validation failure, not an internal error."
  - "edition show surfaces the declared license status verbatim from the canonical row (the typed reader maps unmodelled statuses to Unknown) and omits it when undeclared; no runtime is_primary setter was added."

patterns-established:
  - "Operator integrity surface over the doctor engine: family status is aggregated from the 19 checks, and a skipped family carries a reason/gate instead of a pass."
  - "Operator-supplied reference corpus as payload data: parsed by the existing JSON adapter, compared byte-exact, and recorded verbatim (no identity invented)."

requirements-completed: [REQ-ingestion-validation-eval, REQ-quran-corpus]

coverage:
  - id: D1
    description: "qai quran verify reports the six integrity families for the active edition from persisted state, exits 3 when any family fails, and is read-only (corpus_generation unchanged)."
    requirement: "REQ-ingestion-validation-eval"
    verification:
      - kind: integration
        ref: "crates/application/tests/quran_doctor.rs#verify_reports_the_six_families_and_does_not_write"
        status: pass
      - kind: e2e
        ref: "crates/cli/tests/quran/verify.trycmd (qai quran verify / --json / --deep)"
        status: pass
    human_judgment: false
  - id: D2
    description: "The reference-comparison family is state-derived: skipped when the persisted QV-015 Info skip applies, pass only on a real comparison, fail on a QV-015 Fatal; a skip is never a pass."
    requirement: "REQ-ingestion-validation-eval"
    verification:
      - kind: integration
        ref: "crates/application/tests/quran_doctor.rs#deep_scan_of_the_fixture_has_no_failures"
        status: pass
      - kind: integration
        ref: "crates/application/tests/quran_reference.rs#operator_reference_path_is_evaluated_and_mutation_fails_closed"
        status: pass
    human_judgment: false
  - id: D3
    description: "The pinned edition reference and the STORED per-ayah canonical hash are printed together on the qai quran get human line (the hash is read from AyahView, never recomputed)."
    requirement: "REQ-quran-corpus"
    verification:
      - kind: e2e
        ref: "crates/cli/tests/quran/read_flow.trycmd (qai quran get 1:1 hashed lookup line)"
        status: pass
    human_judgment: false
  - id: D4
    description: "qai quran import --reference evaluates QV-015 end-to-end: a matching reference persists outcome pass with one DifferenceClass per difference from the existing vocabulary; a mismatch fails closed with the stage untouched and no active edition."
    requirement: "REQ-ingestion-validation-eval"
    verification:
      - kind: integration
        ref: "crates/application/tests/quran_reference.rs#operator_reference_path_is_evaluated_and_mutation_fails_closed"
        status: pass
      - kind: unit
        ref: "cargo test -p quran-corpus --lib differ"
        status: pass
      - kind: e2e
        ref: "crates/cli/tests/quran/reference.trycmd (skipped vs evaluated vs mismatch exit 3)"
        status: pass
    human_judgment: false
  - id: D5
    description: "qai quran edition show (human and --json) reports the manifest-declared upstream identity, primary marker, and declared license status for an activated identity fixture, with no volatile ids on the human line."
    requirement: "REQ-quran-corpus"
    verification:
      - kind: e2e
        ref: "crates/cli/tests/quran/edition_identity.trycmd"
        status: pass
      - kind: integration
        ref: "cargo test -p application --test quran_identity"
        status: pass
    human_judgment: false
  - id: D6
    description: "No migration or architecture boundary change: the plan adds operator surfaces only."
    requirement: "REQ-quran-corpus"
    verification:
      - kind: other
        ref: "cargo run -q -p xtask -- migrate-check (21 ordered)"
        status: pass
      - kind: other
        ref: "cargo run -q -p xtask -- arch-check"
        status: pass
    human_judgment: false

duration: 19 min
completed: 2026-09-25
status: complete
---

# Phase 02 Plan 03: Operator integrity, reference, and identity surfaces Summary

**`qai quran verify` classifies the six integrity families read-only from persisted state, the operator import path evaluates QV-015 byte-exact with typed `DifferenceClass` metadata, the stored hash is visible on the lookup, and `edition show` proves identity/primary/license.**

## Performance

- **Duration:** 19 min
- **Started:** 2026-09-25T15:13:09Z
- **Completed:** 2026-09-25T15:32:38Z
- **Tasks:** 3
- **Files modified:** 26 (4 created, 22 modified)

## Accomplishments

- Closed the largest demonstrated operator gap (QC-03/QC-10, Pitfall 3): `qai quran verify` now reports all six integrity families for the active edition, reusing the existing 19-check `run_quran_checks` engine rather than writing a second one, and exits 3 when any family fails. The reference family is derived from the persisted QV-015 finding, so a recorded skip is printed and serialized as `skipped` — never as `pass` (D-10, T-02-06).
- Replaced the hardcoded `quran.reference_corpus` doctor warning with a state-derived check (`Pass`/`Skipped`/`Fail` from the persisted finding) and proved the whole verb is read-only: `corpus_generation` is unchanged across a `verify` invocation (T-02-07).
- Made QV-015 actually evaluable from the production operator path: `qai quran import --reference <path>` carries the reference manifest on the import payload into `run_import`, compares byte-exact and fail-closed, and attaches exactly one ADR-0114 `DifferenceClass` per difference as report metadata without softening a byte mismatch (D-09 mechanism, D-10).
- Proved criterion 2 on the operator surface: `qai quran get` prints the pinned `slug@version` reference together with the STORED per-ayah canonical hash read from `AyahView` (never recomputed), asserted by `read_flow.trycmd`.
- Proved the plan 02-01 identity/primary/license values on the operator surface: `edition show` (human + `--json`) reports the manifest-declared upstream identity, the primary marker, and the declared license status (read verbatim from the canonical row), with no runtime `is_primary` setter introduced.

## Task Commits

Each task was committed atomically:

1. **Task 1: `qai quran verify` six-family surface, state-derived reference check, stored hash on lookup** - `a7c1695` (feat)
2. **Task 2: Operator reference path — real QV-015 evaluation with typed difference metadata** - `022fb80` (feat)
3. **Task 3: `qai quran edition show` identity/primary/license snapshot** - `0e2d119` (feat)

**Plan metadata:** committed with this SUMMARY (docs: complete plan)

## Files Created/Modified

- `crates/cli/tests/quran/verify.trycmd` - six-family snapshot (human, `--json`, `--deep`)
- `crates/cli/tests/quran/reference.trycmd` - reference-less skip vs reference-configured pass vs mismatch exit 3
- `crates/cli/tests/quran/edition_identity.trycmd` - identity/primary/license snapshot (volatile ids wildcarded)
- `crates/application/tests/quran_reference.rs` - operator reference path integration suite
- `crates/application/src/quran_cli.rs` - `cmd_quran_verify`, `cmd_import --reference` + exit mapping, `cmd_get` hash line, `cmd_edition_show` license
- `crates/application/src/quran_doctor.rs` - state-derived `quran.reference_corpus` check + `skipped` helper
- `crates/application/tests/quran_doctor.rs` - read-only/six-family proof test
- `crates/cli/src/quran.rs` - `QuranAction::Verify`, `--reference` on import
- `crates/cli/tests/quran.rs` - three harness fns
- `crates/cli/tests/quran/read_flow.trycmd` - hashed lookup line
- `crates/quran-corpus/src/import.rs` - `ImportInput.reference_manifest_text`, typed diff metadata, fail-closed staging clear
- `crates/application/tests/*.rs` - `ImportInput` constructors gain `reference_manifest_text: None`

## Decisions Made

- `qai quran verify` aggregates the six families from the existing doctor checks; the reference family reads the persisted QV-015 finding and a skip is never reported as pass (D-10).
- The reference corpus is payload data, not a second import path: `ImportInput.reference_manifest_text` is parsed into `ImportOptions.reference` inside `run_import`, and identity/pins come verbatim from the operator-supplied document (OD-03 remains an open owner gate).
- QV-015 pass/fail stays byte-only; the `DifferenceClass` values are report metadata only (ADR-0114 §4).
- A failed reference comparison clears staging and maps to exit 3 (validation), leaving the stage untouched and the operator with an actionable failure.
- `edition show` reads the declared license status verbatim from the canonical row because the typed reader conservatively maps unmodelled statuses (e.g. `verified`) to `Unknown`; the line is omitted for undeclared editions so `read_flow.trycmd` stays stable.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `ImportInput` gained a field; every existing constructor updated**
- **Found during:** Task 2
- **Issue:** Adding the plan's `reference_manifest_text` field to `ImportInput` made 15 test struct literals non-exhaustive (compile error E0063).
- **Fix:** Added `reference_manifest_text: None` to each constructor (mechanical; no behavior change).
- **Files modified:** `crates/application/tests/{common/mod,counting,doctor_indexes,forms_rebuild,index_build,index_lifecycle,morphology_import,quran_identity,quran_import,quran_reader,quran_tools,quran_verification,search_goldens,search_latency,search_tools}.rs`
- **Verification:** `cargo test -p application` (all suites green); `cargo test -p quran-corpus` (all suites green).
- **Committed in:** `022fb80` (Task 2)

**2. [Rule 2 - Missing Critical] Read-only proof test for `qai quran verify`**
- **Found during:** Task 1
- **Issue:** Acceptance criterion 3 ("`quran.corpus_generation` is unchanged across a `verify` invocation") had no executable proof — the trycmd snapshot cannot observe the generation.
- **Fix:** Added `verify_reports_the_six_families_and_does_not_write` to `crates/application/tests/quran_doctor.rs`, asserting the six keys, the reference skip, and an unchanged `corpus_generation` (T-02-07).
- **Files modified:** `crates/application/tests/quran_doctor.rs`
- **Verification:** `cargo test -p application --test quran_doctor` (4 passed).
- **Committed in:** `a7c1695` (Task 1)

**3. [Rule 2 - Missing Critical] QV-015 mismatch on the import path maps to exit 3**
- **Found during:** Task 2
- **Issue:** `cmd_import` mapped every `run_import_job` error to `exit::INTERNAL` (70), so a fail-closed reference mismatch surfaced as an internal error rather than a validation failure.
- **Fix:** On failure, read the run's persisted validation report; a QV-015 Fatal yields `exit::VALIDATION` (3) with a message naming the report, otherwise the original internal error is preserved.
- **Files modified:** `crates/application/src/quran_cli.rs`
- **Verification:** `crates/cli/tests/quran/reference.trycmd` asserts `? 3` on the mismatch.
- **Committed in:** `022fb80` (Task 2)

**4. [Rule 3 - Blocking] `reference.trycmd` mismatch case uses a distinct fixture**
- **Found during:** Task 2
- **Issue:** `source_versions` enforces a unique `(source_id, version)`, so the CLI cannot import the same `slug@version` twice. Re-importing `test-edition-rich@0.1.0` for the mismatch case failed with a unique-constraint error before QV-015 ran.
- **Fix:** The mismatch case imports the fresh `test-edition-identity@0.1.0` against an incompatible reference, yielding the intended exit 3 and leaving no edition staged. The in-process suite still exercises the mutated rich reference directly.
- **Files modified:** `crates/cli/tests/quran/reference.trycmd`
- **Verification:** `cargo test -p cli --test quran -- quran_reference_snapshots` (passed).
- **Committed in:** `022fb80` (Task 2)

---

**Total deviations:** 4 auto-fixed (2 missing-critical, 2 blocking)
**Impact on plan:** All four were necessary for compilation, honest exit codes, and executable acceptance evidence. No scope creep: no canonical-text/hash change, no new migration, no new integrity engine, no real dataset/reference identity, and no runtime `is_primary` setter.

## Issues Encountered

- Two pre-existing CLI snapshot failures remain in `cargo test -p cli --test quran` (`quran_normalize_snapshots`, `quran_search_snapshots`), caused by a non-deterministic index manifest hash and confirmed failing before plan 02-01. Out of scope; their snapshots were not touched.
- The CLI's single-import-per-`slug@version` behavior (unique `source_versions`) shaped the `reference.trycmd` scenario (see deviation 4).

## Known Stubs

None.

## Threat Flags

None — the plan adds read-only operator surfaces and payload-threaded comparison; no new network endpoint, auth path, file-access pattern, or schema change beyond the plan's `<threat_model>`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Criteria 2 and 3 are reproducible from operator commands: `qai quran verify` (six families, skip labelled honestly, read-only) and `qai quran get` (stored hash on the pinned reference).
- The reference-comparison family is evaluable through the production path with byte-only fail-closed semantics and typed report metadata; plan 02-07 can cite `quran_reference.rs` and the snapshots in the corpus-integrity artifact.
- Owner gates OD-01/OD-02/OD-03 remain open and recorded, not closed; `REQ-quran-corpus` and `REQ-ingestion-validation-eval` are shared with sibling plans (02-05/02-06/02-07) that have no SUMMARY yet, so the shared-ID gate keeps them pending.

---

*Phase: 02-canonical-quran-core*
*Completed: 2026-09-25*

## Self-Check: PASSED

- Key files exist: `crates/cli/tests/quran/verify.trycmd`, `reference.trycmd`, `edition_identity.trycmd`, `crates/application/tests/quran_reference.rs`, `.planning/phases/02-canonical-quran-core/02-03-SUMMARY.md`.
- Task commits exist: `a7c1695`, `022fb80`, `0e2d119`.
- Plan verification re-run green: `cargo test -p cli --test quran -- quran_verify_snapshots quran_reference_snapshots edition_identity_snapshots` (all pass), `cargo test -p application --test quran_doctor` (4), `cargo test -p cli --test doctor_json` (2), `cargo test -p application --test quran_reference` (1), `cargo test -p application --test quran_identity` (7), `cargo test -p quran-corpus` (all), `migrate-check` (21 ordered), `arch-check` (OK).
