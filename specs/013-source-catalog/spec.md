# Feature Specification: source-catalog domain

**Feature Branch**: `013-source-catalog`

**Created**: 2026-09-18

**Status**: Draft

**Input**: Module-level specification of the implemented `sources` crate (crates/sources/src/{lib,registry}.rs), derived from code truth on 2026-09-18. Reverse specification of what exists today. Code is the source of truth over plan docs.

**Constitution compliance**: `.specify/memory/constitution.md` v1.1.0, Principles II (layered trust, provenance per row), III (canonical-JSON hashes), IV (no invented license — approval requires known license), VI (deny-by-default: unsigned manifests rejected unless explicitly allowed).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Local manifest import ending at Staged (Priority: P1)

An operator imports a local source manifest. The parser validates JSON shape, schema (`manifest_version` required), semantics (every source needs a non-empty version), and signature policy, then returns the first source as a `SourceVersion` in the `Staged` state — ready for the human approval workflow, never auto-activated.

**Why this priority**: Staged-not-active is the trust boundary. Everything downstream (approval, indexing, activation) depends on import stopping here.

**Independent Test**: `cargo test -p sources --lib` green; `ingest_local` on the sample manifest returns `Staged`; tampered content is rejected before any version is produced.

**Acceptance Scenarios**:

1. **Given** a valid manifest, **When** `ingest_local` runs, **Then** the result is a `SourceVersion` in state `Staged` with the manifest's identity preserved.
2. **Given** a manifest whose bytes were altered after signing, **When** ingested with the original signature, **Then** it fails with `QAI-SRC-0007` and no version is returned.

---

### User Story 2 - Approval gated on four preconditions (Priority: P1)

A version moves `Staged → Approved` only when license status is known, a content hash exists, a validation report exists, and a human approver identity is recorded. Any missing precondition rejects the transition; illegal jumps (e.g. `Staged → Active`) are rejected regardless of preconditions.

**Why this priority**: Principle IV + PRD §22.3. A version that becomes active without a known license, verified bytes, a validation report, or a human approver is a compliance failure, not a shortcut.

**Independent Test**: Four dedicated precondition tests each remove one field and assert rejection (`QAI-SRC-0004`); the illegal-transition matrix test asserts every non-listed jump fails (`QAI-SRC-0003`).

**Acceptance Scenarios**:

1. **Given** a staged version with `license_status == Unknown`, **When** transitioned to `Approved`, **Then** it fails with `ApprovalPreconditionNotMet`, even if hash, report, and approver are present.
2. **Given** any version, **When** transitioned along a non-listed edge (e.g. `Downloaded → Active`), **Then** it fails with `IllegalStateTransition`, and quarantine remains reachable from every state.

---

### User Story 3 - Lineage tracing that refuses cycles (Priority: P2)

An operator records that a translation derives from an edition and resolves its lineage. Self-references and cycles are rejected with a typed error; an original source renders as `original (no parents)`.

**Why this priority**: Derivation lineage (translation/summary/edition/abridgment/commentary/original) is how the catalog answers "where did this come from" without merging distinct sources.

**Independent Test**: `genealogy_renders_lineage_and_rejects_cycles` green; self-relationship insert fails immediately.

**Acceptance Scenarios**:

1. **Given** a child→parent relationship, **When** lineage is resolved, **Then** the rendering names each ancestor and its derivation relationship.
2. **Given** a relationship that would close a cycle (including self-reference), **When** added or resolved, **Then** it fails with `QAI-SRC-0008`.

### Edge Cases

- Unsigned manifests are rejected by default; `allow_unsigned` permits them only for explicitly local policy — `unsigned_local_manifest_rejected_under_remote_policy` pins the distinction.
- `Quarantined` is reachable from every state, including terminal ones (`Deprecated`, `Removed` targets still allow quarantine first).
- An empty manifest (zero sources) fails ingestion with a validation error, not an empty success.
- Only `Storage` errors are retryable; every domain error (`0001`–`0010`) is terminal-by-policy and must surface to the operator.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Crate MUST expose `Source` (identity, titles, `SourceContentType` with 7 variants, authorship roles, language/tradition/school, `SourceIdentifiers` with isbn/oclc/other map) and `SourceVersion` (version, schema version, 13-state `SourceState`, trust level, license status + license JSON, manifest/content hashes, source URLs, lifecycle timestamps, quarantine reason, validation report) (`lib.rs` L30–112).
- **FR-002**: Crate MUST enforce the lifecycle state machine: exactly the listed legal edges (`Discovered→PendingReview|Quarantined`, `PendingReview→Downloading|Quarantined`, `Downloading→Downloaded|Quarantined`, `Downloaded→Validating|Quarantined`, `Validating→Staged|ValidationFailed`, `ValidationFailed→Quarantined`, `Staged→Approved|Quarantined`, `Approved→Indexing`, `Indexing→Active|ValidationFailed`, `Active→Deprecated`, `Deprecated→Removed`, any→`Quarantined`); all other jumps fail with `IllegalStateTransition` (`lib.rs` L518–541).
- **FR-003**: Crate MUST gate `→Approved` on four preconditions — license status known, content hash present, validation report present, human approver identity present — each with its own rejection (`lib.rs` L560–584).
- **FR-004**: Crate MUST parse manifests from JSON, require `manifest_version`, require a non-empty version per source, and support canonical re-serialization plus SHA-256 manifest hashing over `canonical_json_bytes` (`lib.rs` L271–316).
- **FR-005**: Crate MUST verify detached signatures two ways: SHA-256 content-hash match (unsigned rejected unless `allow_unsigned`) and strict ed25519 over the canonical signing payload with base64 key/signature inputs (`lib.rs` L326–417).
- **FR-006**: Crate MUST ingest a local manifest end-to-end (parse → schema → semantic → signature → first source → `Staged`) and reject empty manifests (`lib.rs` L360–374).
- **FR-007**: Crate MUST resolve derivation lineage with DFS, reject self-references at insert and cycles at resolve, and render human-readable lineage strings (`lib.rs` L432–511).
- **FR-008**: Crate MUST expose the `SourceRepository` trait (get/insert source and version, list versions, guarded `transition_state`, `record_transition`, list active versions, genealogy and approval persistence) and the `StructureValidator` trait with `ValidationReport`/`ValidationError` and `DifferenceReport` types (`lib.rs` L227–255, L590–616).
- **FR-009**: Crate MUST emit stable unique `QAI-SRC-0001…0011` diagnostic codes with human summaries, and mark only `Storage` as retryable (`lib.rs` L620–675, uniqueness test-enforced).

### Key Entities

- **Source / SourceVersion**: Catalog identity vs. versioned lifecycle state; versions carry trust, license, hashes, and lifecycle timestamps.
- **SourceState**: 13-state lifecycle; quarantine is the universal escape hatch.
- **ManifestParseResult**: Validated manifest document (versions, catalog version, generation time) plus its canonical hash.
- **ApprovalRecord**: Human decision artifact (kind, requester, decider, decision, notes) backing the approval gate.
- **GenealogyResult / LineageNode**: Resolved ancestor chain with derivation relationships and a human rendering.
- **ValidationReport / DifferenceReport**: Validator output (errors vs. warnings) and version-to-version change sets feeding approval.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo test -p sources --lib` passes with zero failures (legal/illegal transition matrix, four precondition tests, signature accept/reject tests, genealogy cycle test, code-uniqueness test).
- **SC-002**: Every non-listed state jump fails with `QAI-SRC-0003`; quarantine succeeds from all 13 states.
- **SC-003**: Each of the four approval preconditions independently rejects with `QAI-SRC-0004`; all four satisfied allows `Staged → Approved`.
- **SC-004**: A tampered manifest fails signature verification with `QAI-SRC-0007`; an unsigned manifest fails under default policy and passes only with explicit local allowance.
- **SC-005**: All `QAI-SRC-0001…0011` codes are unique, stable, and human-renderable; only storage failures are retryable.

## Assumptions

- Remote download/discovery (`Discovered → Downloading → Downloaded`) is state-machine surface only; network fetching lives in a later phase — the machine plans for it without implementing it.
- ed25519 verification is implemented and tested; key distribution and publisher key registry are out of scope here.
- `MultipleActiveVersions` (`QAI-SRC-0009`) is enforced by repository implementations (single-active invariant), not by the pure state machine.
- Single-parent lineage per child in the in-memory resolver; multi-parent derivation graphs are a planning concern, not a current behavior.
