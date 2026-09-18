# Feature Specification: quran-corpus import pipeline

**Feature Branch**: `006-quran-corpus`

**Created**: 2026-09-17

**Status**: Draft

**Input**: Module-level specification of the implemented `quran-corpus` crate (crates/quran-corpus/src/{lib,adapters,format,unicode,tokenize,hashing,validation,differ,import,error}.rs), derived from code truth on 2026-09-17. Reverse specification of what exists today. Code is the source of truth over plan docs.

**Constitution compliance**: `.specify/memory/constitution.md` v1.0.2, Principles I (I1/I5/I7: staging + approval-gated activation, importer cannot activate), II (Layer A/B separation, provenance per row), III (hashes, validation reports, difference reports), VII (deterministic path, zero model deps).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Thirteen-gate import ending at approval (Priority: P1)

An operator imports a Quran edition manifest. The pipeline walks 13 checkpoints — claimed → manifest_hashed → adapter_selected → parsed → unicode_audited → validated → tokenized → hashes_computed → staged → roundtrip_verified → compared → diffed → approval_requested — writing only to `quran_stg_*` staging mirrors, and stops at `ApprovalRequested`. The importer holds no `ApprovalToken` and cannot activate anything.

**Why this priority**: Invariants I5 (partial import never becomes Active) and I7 (corrections = new version + diff + human approval). Activation is a separate human-gated step in `application`.

**Independent Test**: `cargo test -p quran-corpus --lib` green (27 passed on 2026-09-17); restart-is-resume (deterministic re-run reaches the same checkpoint), cancellation cleans staging, `stop_after` dry-run support.

**Acceptance Scenarios**:

1. **Given** a valid manifest, **When** `run_import` completes, **Then** the outcome is `ApprovalRequested` (never `Active`) with staging rows, validation report, and difference report persisted.
2. **Given** a cancelled run, **When** it stops mid-pipeline, **Then** staging rows for that `import_run_id` are cleaned and a re-run resumes deterministically.

---

### User Story 2 - Validation that names its rule (Priority: P1)

Every structural, Unicode, checksum, ordering, and count check emits a `QV-nnn` finding with severity; 16 adversarial fixtures each reject with their specific rule id. QV-015 (reference-corpus comparison) is a recorded skip until ADR-0114 lands.

**Why this priority**: Principle I + III. A failed import must say exactly which rule failed, so dataset problems are fixable instead of mysterious.

**Independent Test**: `quran_edition_v1` validator tests green (`base_fixture_has_no_fatal_or_error`, tamper detection); adversarial fixtures each map to one rule id.

**Acceptance Scenarios**:

1. **Given** the base synthetic fixture, **When** validated, **Then** zero fatal/error findings.
2. **Given** a tampered file (hash mismatch), **When** checked, **Then** `check_file_hash` yields the tamper finding and the import cannot proceed.

---

### User Story 3 - Lossless tokenization with frozen hashes (Priority: P2)

Tokenization preserves whitespace exactly (`reconstruct(tokens, separators) == text`, property-tested) with grapheme/byte offsets; `text_hash`/`structure_hash`/`token_order_hash` follow the frozen ADR-0108 recipes with algorithm tags.

**Why this priority**: Invariants I4 (stable verifiable token order) and III (reproducibility). Lossy tokenization would silently corrupt canonical text handling.

**Independent Test**: proptest round-trip green; `intermediate_hash_is_stable` green; MV-018 verifier re-asserts canonical text unchanged after derived builds.

**Acceptance Scenarios**:

1. **Given** any edition text, **When** tokenized and reconstructed, **Then** output is byte-identical to input.
2. **Given** the same source, **When** hashed twice, **Then** all three hashes are identical.

### Edge Cases

- CSV adapter reproduces the JSON manifest exactly (second dataset shape, D1.2) — adapter mismatch yields `QAI-QUR-0203`.
- Unknown/unsupported formats fail at adapter selection (`0200`/`0202`), before any staging write.
- Reference comparison (`compared` checkpoint) skips QV-015 with a recorded reason while ADR-0114 stays Draft — a skip, not a silent pass.
- Char-level edition differ (`differ.rs`, ADR-0109) persists `difference_reports` used by the human approval step.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Crate MUST implement `run_import` over the 13 `ImportCheckpoint::ALL` stages with `Driver` (checkpoint ledger, `check_cancel`/`cancel_run`/`fail_run`), `ImportOptions` (incl. `stop_after` dry-run/chaos), `ImportProgress` recording, restart-is-resume, and staging cleanup on cancellation (`import.rs` L49–1378).
- **FR-002**: Crate MUST validate via `validate_edition` (`quran_edition_v1` v1.0.0) emitting `QV-nnn` `Finding`s with `Severity`/`Outcome`, persisted as `ValidationReport` rows; QV-015 skip recorded while reference corpus is unavailable (`validation.rs` L33–642).
- **FR-003**: Crate MUST tokenize whitespace-preserving with exact separators + grapheme/byte offsets (`tokenize.rs`), audit Unicode (normalization form, forbidden code points, expected blocks per ADR-0104) (`unicode.rs`), and hash per frozen ADR-0108 recipes (`hashing.rs`).
- **FR-004**: Crate MUST parse JSON and CSV manifests through the `EditionAdapter` trait into `EditionSource` per `docs/schemas/quran-edition-source.v1.schema.json` (`adapters.rs`, `format.rs`).
- **FR-005**: Crate MUST diff editions char-level and persist difference reports (`differ.rs`, ADR-0109) for the human approval gate.
- **FR-006**: Crate MUST end at `ApprovalRequested` holding no `ApprovalToken`; activation/rollback live in `application` services (I5/I7). Zero model/vector dependencies on this path (I2).

### Key Entities

- **EditionSource**: Intermediate validated manifest document (schema v1).
- **ImportCheckpoint / ImportProgress / ImportSuccess / ImportOutcome**: 13-stage ledger, progress recorder, terminal outcomes.
- **ValidationReport / Finding (QV-nnn)**: Rule-identified validation evidence with severity and outcome counts.
- **ComputedToken / Staging rows**: Tokenizer output and `quran_stg_*` mirrors keyed by `import_run_id` (no immutability triggers; cascade deletes).

### Error Surface

`QAI-QUR-0200` unsupported format · `0201` invalid format · `0202` unknown adapter · `0203` adapter failed · `0204` validation failed · `0205` import cancelled · `0206` import failed · `0207` storage failed (`error.rs codes`; uniqueness test-enforced). Validation findings use the separate `QV-001…QV-028` rule namespace (validator-level, not Diagnostic codes).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo test -p quran-corpus --lib` passes with zero failures (27 passed on 2026-09-17).
- **SC-002**: Full import of the synthetic fixture ends at `ApprovalRequested` with staging + validation + difference artifacts and zero writes to canonical tables.
- **SC-003**: All 16 adversarial fixtures reject with their specific rule ids; base fixture validates clean.
- **SC-004**: Tokenizer proptest (`reconstruct == text`) and hash-stability tests pass; cancellation leaves no orphan staging rows.

## Assumptions

- The `test-edition-min` fixture is synthetic Arabic-like plumbing text — never presented as scripture, never expanded into real verses (briefing trap §14.1). Fixture success ≠ editorial approval (ADR-0101/0114 Draft block real-dataset gates).
- Reference-corpus comparison (QV-015) stays a recorded skip until the linguist/reviewer decisions land; the skip is visible in reports, not hidden.
- No Tantivy here (plan-text drift); full-text serving over derived forms is FTS5 in `quran-search`.
- Morphology/families/counting are out of scope (placeholder `quran-morphology`, P2 sprints 2.4–2.5 unstarted).
