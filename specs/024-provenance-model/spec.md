# Feature Specification: Provenance Model & Canonical Writer

**Feature Branch**: `024-provenance-model`

**Created**: 2026-09-19

**Status**: Draft

**Input**: User description: "provenance model and canonical writer: module-level reverse specification of the implemented provenance crate (crates/provenance/src/lib.rs, D0.11, ADR-0008, PRD section 6 trust layers A-E), derived from code truth. Document record model, ApprovalToken, CanonicalWriter, trust levels, and derivation lineage. Code is source of truth. Gap: foundation crate with zero spec."

> **Reverse specification.** This feature is a *specification* of an already-implemented foundation contract. The
> implementation (`crates/provenance/src/lib.rs`, `domain::{provenance, types, ids}`, and the SQLite storage layer) is the
> source of truth; this document describes the contract it already delivers, and any deviation between this spec and the
> code is a defect in the spec. No new behavior is proposed; the goal is to close the "foundation crate with zero spec" gap.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Trace every assertion to its origin and trust layer (Priority: P1)

A researcher inspects any assertion in Q-ai (canonical passage, dataset metadata, scholarly note, machine-generated
annotation, or user bookmark). The assertion carries a universal provenance record that specifies which of the five
authoritative trust layers (Layer A Canonical Source Text through Layer E User/AI Notes) it belongs to, its attribution
kind and parameters, the exact source location, its verification status, its trust level, and its confidence (mandatory for
machine-generated annotations).

**Why this priority**: Core of PRD §6 and Constitution Principle II (Layered Trust and Provenance). Without a universal
provenance record and strict layer separation, AI-generated material could be confused with canonical or verified scholarly text.

**Independent Test**: Create a provenance record for each data layer; verify that computational annotations without
algorithm or confidence are rejected; verify that scholarly annotations require a named scholar; verify that canonical
provenance records cannot be updated or deleted.

**Acceptance Scenarios**:

1. **Given** any non-canonical or canonical assertion in the system, **When** its provenance record is read, **Then** it
   contains a unique record ID, data layer, URN subject reference, typed attribution, optional source version ID,
   optional source location (with optional quoted text hash), trust level, verification status, derivation version
   metadata, and timestamped creator identity.
2. **Given** a computational annotation (`layer = Computational`), **When** attempting to persist it without an algorithm
   name or without a confidence score, **Then** the record is rejected with error `MissingComputationalMetadata` (`QAI-PROV-0002`).
3. **Given** a computational annotation, **When** attempting to mark it as human-verified without a reviewer identity,
   **Then** the record is rejected with error `ComputationalNeedsReview` (`QAI-PROV-0007`).
4. **Given** an existing canonical provenance record (`layer = Canonical`), **When** an update or deletion is attempted,
   **Then** the operation is rejected with `ImmutableCanonical` (`QAI-PROV-0005`) / database trigger abort (`QAI-PROV-0001` / `QAI-PROV-0002`).

---

### User Story 2 - Gate canonical changes with an explicit approval token (Priority: P1)

A system administrator or corpus ingestion pipeline initiates an update or activation of canonical Quran or source data.
Canonical writes are gated by `CanonicalWriter` and require an `ApprovalToken` representing human editorial approval. No
canonical mutation can begin or commit without presenting this token.

**Why this priority**: Non-negotiable PRD §2.1, §7.3 and Constitution Principle I (Canonical Text Integrity). Prevents
accidental or automated modification of canonical source text.

**Independent Test**: Attempt to initiate a canonical change session via `CanonicalWriter::begin_canonical_change` with and
without an `ApprovalToken`; verify that missing or invalid tokens return typed errors; verify that a complete
`CanonicalChangeRequest` (source version, content hash, structural report, difference report, approver identity) is enforced.

**Acceptance Scenarios**:

1. **Given** a canonical change operation, **When** `begin_canonical_change` is called with a valid `ApprovalToken` and
   a fully specified `CanonicalChangeRequest`, **Then** an active `CanonicalChangeSession` is opened with status `Open`.
2. **Given** an attempt to perform a canonical change, **When** an `ApprovalToken` is missing or invalid, **Then** the
   operation is rejected with `MissingApprovalToken` (`QAI-PROV-0003`) or `InvalidApprovalToken` (`QAI-PROV-0006`).
3. **Given** a `CanonicalChangeRequest`, **When** any mandatory field (source version ID, content hash, structural validation
   report, difference report, approver identity) is missing, **Then** the request is rejected with `MissingChangeRequestField` (`QAI-PROV-0004`).
4. **Given** an open `CanonicalChangeSession`, **When** committed or aborted via `CanonicalWriter`, **Then** the session
   transitions atomically to `Committed` or `Aborted`.

---

### User Story 3 - Full derivation lineage tracking across transformations (Priority: P2)

When an artifact is derived (e.g. parsed, normalized, chunked, or indexed), the system records the exact derivation lineage
(`DerivationVersions`) including source version, parser semver, normalizer semver, optional chunker semver, optional
embedding model, optional graph builder semver, dependency snapshot ID, and schema version.

**Why this priority**: Traceability requirement (PRD §76, §82). Allows detecting when upstream parser or normalizer changes
require invalidating or recomputing derived artifacts.

**Independent Test**: Construct a `ProvenanceRecord` with a `DerivationVersions` payload; verify that parser and normalizer
versions are captured as strict semver (`MAJOR.MINOR.PATCH`) and schema versions as integer values.

**Acceptance Scenarios**:

1. **Given** a derived data record, **When** inspecting its provenance, **Then** `DerivationVersions` contains the
   `source_version_id`, `parser_version`, `normalizer_version`, and `schema_version`.
2. **Given** downstream enrichment processes (chunking, embedding, graph construction), **When** present, **Then** their
   corresponding versions are captured in `chunker_version`, `embedding_model_version`, and `graph_builder_version`.

---

### User Story 4 - Asynchronous review queue for computational assertions (Priority: P2)

When machine-generated annotations, graph edges, morphology proposals, or narrator identity links require human vetting,
they are placed into a structured `ReviewQueue` with an associated evidence payload, priority, and state machine
(`Pending` -> `Accepted` | `Rejected` | `Corrected`).

**Why this priority**: Constitution Principle II & PRD §10.3/§10.6 dictate that computational suggestions must not become
verified edges or canonical text without explicit human review.

**Independent Test**: Enqueue a review item for a graph edge; verify it enters `Pending` state; record a review decision
(`Accepted`, `Rejected`, or `Corrected`) with reviewer principal, decision note, and timestamp.

**Acceptance Scenarios**:

1. **Given** a computational finding requiring human verification, **When** inserted into `ReviewQueue`, **Then** it is
   assigned a specific queue category (`GraphEdge`, `Morphology`, `NarratorIdentity`, `CrossReference`), priority, and
   persists `evidence_json`.
2. **Given** a pending review item, **When** a reviewer records an editorial decision, **Then** the review item records
   `decided_by`, `decided_at`, `decision_note`, and updates the status to `Accepted`, `Rejected`, or `Corrected`.

---

### Edge Cases

- **Immutability of Canonical Provenance**: Any `UPDATE` or `DELETE` executed on a row where `layer = canonical_source` is
  aborted by database triggers with error `QAI-PROV-0001` or `QAI-PROV-0002`.
- **Supersession**: When a provenance record is superseded by a newer version, `superseded_by` points to the new record ID,
  and the verification status transitions to `Superseded`.
- **Zero-diff Canonical Change**: A `DifferenceReport` with empty `added`, `removed`, `changed`, and non-empty `unchanged`
  is valid and can still proceed through approval gating.
- **Malformed Subject URN**: If a subject reference does not match the expected URN format (`urn:qai:<domain>:<type>:<id>`),
  it fails validation with `InvalidSubjectRef` (`QAI-PROV-0008`).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST represent provenance using a universal `ProvenanceRecord` struct containing: `id`, `layer`,
  `subject`, `attribution`, `source_version_id`, `source_location`, `trust_level`, `verification`, `confidence`, `versions`,
  `created_at`, `created_by`, `reviewed_by`, `reviewed_at`, `review_note`, and `superseded_by`.
- **FR-002**: Data layers MUST be categorized into: `Canonical`, `Publisher`, `Scholarly`, `Computational`, `User`, and `Agent`.
- **FR-003**: Trust levels MUST be ordered and categorized into: `CanonicalVerified`, `PublisherVerified`, `ScholarReviewed`,
  `CommunityReviewed`, `ImportedUnverified`, `MachineGenerated`, `UserProvided`, and `Quarantined`.
- **FR-004**: Verification statuses MUST be categorized into: `Unverified`, `Processing`, `Verified`, and `Rejected` (with
  `NeedsReview`, `HumanVerified`, and `Superseded` supported across storage representation).
- **FR-005**: Attribution MUST be a closed, typed enum supporting four variants:
  - `Dataset`: `source_id`, `dataset_name`, `dataset_version`
  - `Scholar`: `name`, `school` (optional), `work` (optional), `edition` (optional)
  - `Computational`: `algorithm`, `version`, `model` (optional), `parameters_hash`
  - `User`: `principal_id`
- **FR-006**: Computational annotations (`layer = Computational`) MUST require an `algorithm` and a valid `confidence` score
  between `0.0` and `1.0`. Computational annotations CANNOT be marked human-verified without a valid `reviewed_by` principal ID.
- **FR-007**: Derivation tracking MUST capture `DerivationVersions` including `source_version_id`, `parser_version` (SemVer),
  `normalizer_version` (SemVer), `chunker_version` (optional SemVer), `embedding_model_version` (optional String),
  `graph_builder_version` (optional SemVer), `dependency_snapshot_id` (optional String), and `schema_version` (u32).
- **FR-008**: Canonical writes MUST be controlled exclusively via the `CanonicalWriter` trait requiring an `ApprovalToken` and
  a complete `CanonicalChangeRequest` (comprising `new_source_version_id`, `content_hash`, `structural_validation_report`,
  `difference_report`, and `approver_identity`).
- **FR-009**: Canonical provenance records MUST be immutable; direct modification or deletion of canonical provenance is
  prevented by database triggers and repository constraints.
- **FR-010**: The system MUST support a `ReviewQueue` for managing pending computational assertions across queues: `GraphEdge`,
  `Morphology`, `NarratorIdentity`, and `CrossReference`, with state machine states `Pending`, `Accepted`, `Rejected`, and `Corrected`.
- **FR-011**: All provenance errors MUST map to stable `QAI-PROV-0001` through `QAI-PROV-0009` diagnostic codes.

### Key Entities *(include if feature involves data)*

- **ProvenanceRecord**: The universal record tying an assertion/subject to its author, trust level, verification status, and
  derivation history.
- **Attribution**: Typed metadata indicating whether an assertion originated from a published dataset, a named scholar, an
  automated algorithm/model, or a human user.
- **SourceLocation**: Exact locator metadata (volume, book, chapter, page, record number, character range, canonical reference,
  and quoted text hash) for full quotation traceability.
- **DerivationVersions**: Structured ledger of all tool, model, parser, normalizer, and schema versions involved in creating a
  derived artifact.
- **ApprovalToken**: An opaque capability token proving human editorial authorization for canonical text updates.
- **CanonicalChangeRequest**: The full manifest of a proposed canonical update (content hash, structural validation report,
  difference report, and approver identity).
- **CanonicalChangeSession**: A transactional session (`Open`, `Committed`, `Aborted`) managing the atomic lifecycle of a
  canonical change.
- **ReviewQueue**: A priority queue of computational findings requiring human scholarly or editorial review.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of non-canonical and canonical assertions in the database are linkable to a valid `ProvenanceRecord`.
- **SC-002**: 100% of attempts to insert computational annotations without confidence scores or algorithm names are rejected.
- **SC-003**: 0% of canonical records can be modified or deleted without an `ApprovalToken` and `CanonicalWriter` session.
- **SC-004**: Database triggers prevent 100% of direct SQL updates or deletes on canonical provenance records.
- **SC-005**: All error conditions map to unambiguous, documented `QAI-PROV-nnnn` diagnostic codes.

## Assumptions

- **Code truth**: Where discrepancies exist between conceptual PRD text and the code, `crates/provenance/src/lib.rs` and
  associated migrations govern.
- **Universal schema**: Single table design (`provenance_records`) with typed JSON blobs is the accepted architecture (ADR-0008).
- **Approval gating**: Human approvals are issued externally or via admin tooling and represented in code as `ApprovalToken`.
- **Diagnostic codes**: Provenance errors implement `storage::error::Diagnostic` and use the `QAI-PROV-` namespace.