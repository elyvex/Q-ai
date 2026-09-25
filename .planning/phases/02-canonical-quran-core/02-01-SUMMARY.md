---
phase: 02-canonical-quran-core
plan: 01
subsystem: database
tags: [quran, migration, sqlite, edition-identity, license, primary-edition, reader]

requires:
  - phase: 01-foundations
    provides: append-only migration runner + checksums ledger, approval-gated activation transaction, generation-keyed reader, quran_identity-style integration harness

provides:
  - migration 0020 (upstream_edition_slug / qai_edition_id / is_primary on quran_editions + quran_stg_editions)
  - manifest-declared upstream identity and primary designation persisted verbatim to the canonical edition
  - manifest-declared license persisted verbatim; undeclared stays explicit Unknown (OD-01)
  - EditionSelector::Primary with typed zero/ambiguous errors
  - quran_identity integration suite (identity, license, primary selector)

affects: [02-03 operator snapshot, 02-04 canonical-write fence, 02-06 translation license, 02-07 owner gates]

actuals:
  tokens: 9855
  tasks: 3
  commits: 3

tech-stack:
  added: []
  patterns:
    - append-only additive ALTER TABLE migration (no in-tree precedent)
    - declared-license verbatim storage + conservative tolerant read mapping
    - explicit primary/default selector that never falls back to the active pointer

key-files:
  created:
    - migrations/sqlite/0020_quran_edition_identity.up.sql
    - crates/application/tests/quran_identity.rs
  modified:
    - migrations/sqlite/checksums.json
    - docs/schemas/quran-edition-source.v1.schema.json
    - crates/quran-corpus/src/format.rs
    - crates/quran-corpus/src/import.rs
    - crates/storage/src/quran.rs
    - crates/storage-sqlite/src/quran.rs
    - crates/storage-sqlite/tests/quran.rs
    - crates/quran-core/src/edition.rs
    - crates/quran-core/src/enums.rs
    - crates/quran-core/src/reference/serializer.rs
    - crates/application/src/quran_reader.rs
    - crates/application/src/quran_cli.rs

key-decisions:
  - "Edition identity surfaces (upstream_edition_slug, qai_edition_id, is_primary) are stored verbatim from the manifest; absent values stay NULL/None/false (OD-01)."
  - "Canonical license_json stores the manifest-declared license object verbatim; the reader maps unmodelled statuses to LicenseStatus::Unknown and never invents redistribution permission."
  - "EditionSelector::Primary resolves the single flagged edition and returns typed errors when zero or more than one is flagged; it never falls back to the active pointer (D-07)."
  - "Primary is programmatic-only in the frozen reference grammar (ADR-0102): it serializes without an edition prefix and is excluded from the round-trip property."

patterns-established:
  - "Append-only additive ALTER TABLE migration: new columns added from a new file, never editing applied migrations (ADR-0002)."
  - "Declared-vs-typed boundary: canonical rows keep operator-declared strings verbatim; typed reader views interpret conservatively and never invent permissiveness."

requirements-completed: [REQ-quran-corpus, REQ-data-separation-layers]

coverage:
  - id: D1
    description: "Manifest-declared upstream_edition_slug / qai_edition_id / is_primary survive import → staging → activation and are readable through the canonical row, the reader view, and the operator edition-show surface."
    requirement: "REQ-quran-corpus"
    verification:
      - kind: integration
        ref: "crates/application/tests/quran_identity.rs#declared_identity_and_primary_survive_to_the_operator_surface"
        status: pass
      - kind: integration
        ref: "cargo test -p storage-sqlite --test quran"
        status: pass
    human_judgment: false
  - id: D2
    description: "Manifest-declared license is persisted verbatim; an undeclared license stays explicit Unknown with no invented redistribution permission."
    requirement: "REQ-data-separation-layers"
    verification:
      - kind: integration
        ref: "crates/application/tests/quran_identity.rs#declared_license_is_persisted_verbatim"
        status: pass
      - kind: integration
        ref: "crates/application/tests/quran_identity.rs#undeclared_license_is_unknown_without_invented_permissions"
        status: pass
    human_judgment: false
  - id: D3
    description: "Append-only migration 0020 adds the three columns to both canonical and staging tables; migrations 0007–0019 are untouched and the checksum ledger gains exactly one entry."
    requirement: "REQ-quran-corpus"
    verification:
      - kind: automated_ui
        ref: "cargo run -q -p xtask -- migrate-check (20 migrations ordered; checksums stable)"
        status: pass
    human_judgment: false
  - id: D4
    description: "EditionSelector::Primary resolves the flagged edition and returns typed errors when none or more than one is flagged, never falling back to the active pointer."
    requirement: "REQ-quran-corpus"
    verification:
      - kind: integration
        ref: "crates/application/tests/quran_identity.rs#primary_selector_resolves_the_flagged_edition"
        status: pass
      - kind: integration
        ref: "crates/application/tests/quran_identity.rs#primary_selector_errors_when_none_is_declared"
        status: pass
      - kind: integration
        ref: "crates/application/tests/quran_identity.rs#primary_selector_errors_when_more_than_one_is_declared"
        status: pass
    human_judgment: false
  - id: D5
    description: "docs/schemas/quran-edition-source.v1.schema.json declares the matching edition.is_primary boolean property and the committed schema is CI-quiet under gen-schema."
    verification:
      - kind: other
        ref: "cargo run -q -p xtask -- gen-schema && git diff --exit-code docs/schemas/"
        status: pass
    human_judgment: false

duration: 22 min
completed: 2026-09-25
status: complete
---

# Phase 02 Plan 01: End-to-end edition identity — migration through the operator surface Summary

**Migration 0020 plus a manifest-sourced edition upstream identity, a verbatim declared license, and an explicit primary/default selector plumbed import → staging → activation → reader → `edition show`.**

## Performance

- **Duration:** 22 min
- **Started:** 2026-09-25T06:03:10Z
- **Completed:** 2026-09-25T06:25:13Z
- **Tasks:** 3
- **Files modified:** 14

## Accomplishments

- Proven the highest-risk phase assumption (research A3) end-to-end: an append-only additive `ALTER TABLE` migration (0020) adds `upstream_edition_slug`, `qai_edition_id`, and `is_primary` to both `quran_editions` and `quran_stg_editions` and flows them through the frozen staging→activation `INSERT … SELECT` without editing 0007–0019 or changing the activation transaction.
- Sourced the missing `EditionMeta.is_primary` field and its schema property, then carried manifest-declared upstream identity plus the primary marker verbatim to the canonical row, the reader view, and the operator `edition show` surface (human + `--json`).
- Stopped the operator import path from synthesizing license identity: the canonical `license_json` now holds the manifest-declared license verbatim, undeclared stays explicit `Unknown` with no invented redistribution flags (OD-01 remains the gate), and the reader interprets declared statuses conservatively without crashing.
- Added `EditionSelector::Primary`, which resolves the single flagged edition and returns typed `PrimaryNotDeclared` / `PrimaryAmbiguous` errors — never silently returning the active pointer ("primary default" ≠ "only edition", D-07).

## Task Commits

Each task was committed atomically:

1. **Task 1: End-to-end edition identity (tracer)** - `18f3cce` (feat)
2. **Task 2: Persist the manifest-declared license** - `7d925a9` (feat)
3. **Task 3: Explicit primary/default selector** - `2d831e4` (feat)

**Plan metadata:** committed with this SUMMARY (docs: complete plan)

## Files Created/Modified

- `migrations/sqlite/0020_quran_edition_identity.up.sql` - append-only additive columns on canonical + staging edition tables
- `migrations/sqlite/checksums.json` - one new checksum entry for 0020
- `docs/schemas/quran-edition-source.v1.schema.json` - declares `edition.is_primary` (boolean)
- `crates/quran-corpus/src/format.rs` - `EditionMeta.is_primary` (serde-defaulted)
- `crates/quran-corpus/src/import.rs` - staging insert carries the three declared fields
- `crates/storage/src/quran.rs` - `QuranEditionRow.{upstream_edition_slug,qai_edition_id,is_primary}`
- `crates/storage-sqlite/src/quran.rs` - staging insert, `decode_edition`, activation `INSERT … SELECT`
- `crates/storage-sqlite/tests/quran.rs` - row helper gains the new fields
- `crates/quran-core/src/edition.rs` - `QuranEdition` gains the three fields
- `crates/quran-core/src/enums.rs` - `EditionSelector::Primary`
- `crates/quran-core/src/reference/serializer.rs` - `Primary` arm (programmatic-only)
- `crates/application/src/quran_reader.rs` - `map_edition` fields, tolerant license parse, `Primary` resolution arm + typed errors
- `crates/application/src/quran_cli.rs` - license derivation in `cmd_import`, identity/primary on the `edition show` human line, `primary` selector keyword, error mapping
- `crates/application/tests/quran_identity.rs` - new identity/license/primary integration suite

## Decisions Made

- Identity strings are preserved byte-for-byte on the canonical row; no trim, case fold, or normalization (edge probe "encoding").
- Canonical `license_json` keeps the declared status/expression/source_url verbatim, while the typed reader view maps unmodelled statuses to `LicenseStatus::Unknown` — conservative, never permissive.
- `Primary` is a reserved selector keyword and a programmatic-only reference edition; it is not added to the frozen reference grammar.
- Task-boundary discipline: the plan's `EditionSelector::Primary` resolution assertions were authored in Task 3 (where the variant is introduced) rather than Task 1, keeping each commit self-consistent.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Tolerant canonical-license read mapping**
- **Found during:** Task 2 (persist the manifest-declared license)
- **Issue:** Storing the declared license verbatim gives `license_json.status` values (`verified`/`restricted`/…) outside the domain `LicenseStatus` enum; the reader's `serde_json::from_str::<LicenseRecord>` would reject them and every declared-license edition would fail to read.
- **Fix:** Added `parse_license_record` in `crates/application/src/quran_reader.rs`: it reads the JSON tolerantly, maps unrecognized statuses to `LicenseStatus::Unknown`, and never invents `redistribution_allowed`/`export_allowed`.
- **Files modified:** crates/application/src/quran_reader.rs
- **Verification:** `quran_identity#declared_license_is_persisted_verbatim` and `quran_reader`/`quran_doctor` suites pass.
- **Committed in:** `7d925a9` (Task 2)

**2. [Rule 3 - Blocking] Exhaustive-match arm for `EditionSelector::Primary` in the reference serializer**
- **Found during:** Task 3 (primary selector)
- **Issue:** `push_edition_prefix` matched `EditionSelector` exhaustively; adding a variant broke the `quran-core` build.
- **Fix:** `Primary` shares the short (prefix-less) form with `Active` and is documented as programmatic-only, mirroring the bare `QuranRef::Edition` exclusion from the round-trip property.
- **Files modified:** crates/quran-core/src/reference/serializer.rs
- **Verification:** `cargo test -p quran-core` (reference grammar + round-trip) and `quran_reader` pass.
- **Committed in:** `2d831e4` (Task 3)

**3. [Rule 3 - Blocking] Struct-literal field additions in existing tests**
- **Found during:** Task 1
- **Issue:** `QuranEditionRow` construction in `crates/storage-sqlite/tests/quran.rs` became non-exhaustive after the struct gained fields.
- **Fix:** Added the three fields (`None`, `None`, `false`).
- **Files modified:** crates/storage-sqlite/tests/quran.rs
- **Verification:** `cargo test -p storage-sqlite --test quran` passes.
- **Committed in:** `18f3cce` (Task 1)

---

**Total deviations:** 3 auto-fixed (1 missing critical, 2 blocking)
**Impact on plan:** All three were necessary for correctness/compilation. No scope creep: no new repository mutator, no canonical-text/hash change, no activation-transaction behavior change, and no new package.

## Issues Encountered

- Pre-existing dirty working-tree files (02-RESEARCH.md, CHANGELOG.md, `quran_doctor_indexes.rs`, the legacy `docs/03-plan` board, task-done rollup, STATE.md) were committed by a concurrent GSD process before this run began; this executor did not modify, revert, or commit them. The plan ledger base is `2a590c6`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Wave 1 (this tracer) is complete and proven; Wave 2 plans (02-02 richer fixtures, 02-04 canonical-write fence) are unblocked.
- The identity/primary operator surface is asserted at the application level; its real-binary `qai quran edition show` trycmd snapshot is owned by plan 02-03.
- Owner gates OD-01 (dataset/license), OD-02 (reviewer), and OD-03 (reference corpus) remain open and are recorded, not closed. `REQ-quran-corpus` and `REQ-data-separation-layers` were not flipped complete here: both are shared with sibling plans (02-03/02-04/02-06/02-07) that have no SUMMARY yet, so the shared-ID gate keeps them pending.

---

*Phase: 02-canonical-quran-core*
*Completed: 2026-09-25*

## Self-Check: PASSED

- Key files exist: `migrations/sqlite/0020_quran_edition_identity.up.sql`, `crates/application/tests/quran_identity.rs`, `.planning/phases/02-canonical-quran-core/02-01-SUMMARY.md`.
- Task commits exist: `18f3cce`, `7d925a9`, `2d831e4`.
- Plan-level verification re-run green: migrate-check (20 ordered), `quran_identity` (7 passed), `storage-sqlite --test quran` (10 passed), `quran-corpus --test fixtures` (7 passed), `gen-schema` clean, `arch-check` OK.
