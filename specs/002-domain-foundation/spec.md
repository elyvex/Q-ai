# Feature Specification: domain foundation types

**Feature Branch**: `002-domain-foundation`

**Created**: 2026-09-17

**Status**: Draft

**Input**: Module-level specification of the implemented `domain` crate (crates/domain/src/), derived from code truth on 2026-09-17. This is a reverse specification: it documents what the crate does today, not a proposal. Code is the source of truth over plan docs.

**Constitution compliance**: `.specify/memory/constitution.md` v1.0.2, Principles I (canonical integrity), II (layered trust/provenance), III (traceability), V (gates), VI (deny-by-default), VII (domain has no I/O or async runtime; deterministic canonical path has zero model/vector dependencies).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Shared vocabulary for all crates (Priority: P1)

Every other crate (storage, provenance, audit, jobs, sources, config, quran-*) builds on the pure types in `domain` instead of defining its own IDs, error envelope, trust levels, or hashing. A developer adding a new entity uses a `domain` ID newtype and returns `Diagnostic`-compatible errors.

**Why this priority**: Without a single dependency-light vocabulary, entity IDs get confused at compile time and error codes diverge. This is the foundation everything else stands on.

**Independent Test**: `cargo test -p domain --lib` green (71 passed on 2026-09-17); `cargo run -p xtask -- arch-check` confirms no forbidden dependency edges into `domain`.

**Acceptance Scenarios**:

1. **Given** a new entity needs an identifier, **When** the developer declares it with the `typed_id!` macro pattern, **Then** it is a distinct compile-time type (a `SourceId` can never be used where a `JobId` is expected).
2. **Given** any crate returns an error, **When** it is rendered for a human or as JSON, **Then** it carries a unique namespaced code (e.g. `QAI-DOM-1002`) plus severity, category, remedy, and next-command fields.

---

### User Story 2 - Secret-safe diagnostics and logs (Priority: P1)

When errors or log fields contain credential-shaped material, the rendering layer scrubs it before output, in both human-readable and JSON forms, without mutating the stored struct.

**Why this priority**: Constitution Principle VI (local-first, deny-by-default). Secret leaks through diagnostics are a defect, not a trade-off.

**Independent Test**: `cargo test -p testkit --test secret_leak` sentinel suite passes; `cargo test -p domain diagnostic` renderer tests unchanged and green.

**Acceptance Scenarios**:

1. **Given** a `Diagnostic` whose message contains `api_key=<secret>`, **When** rendered via `render_human` or `render_json`, **Then** zero secret bytes appear in the output and the stored struct is unmodified.
2. **Given** a JSON config value with a secret-named key, **When** passed through `redact_json_value`, **Then** the value is replaced with `***REDACTED***` and structure (keys, nesting, non-secret leaves) is otherwise identical.

---

### User Story 3 - Deny-by-default input guards (Priority: P2)

Callers that accept paths, archives, network targets, or HTML pass them through `domain` security guards first, so path escape, archive bombs, SSRF via DNS rebinding, oversized inputs, and script injection are rejected at the boundary.

**Why this priority**: Principle VI enforcement point used by importers, sources, and CLI before any I/O happens (the guards themselves perform no I/O).

**Independent Test**: `cargo test -p domain --lib security` matrix suites green (private-IPv4/IPv6 matrices, allowlist exact+suffix, archive acceptance).

**Acceptance Scenarios**:

1. **Given** a path resolving outside its declared root, **When** canonicalized against the root, **Then** it is rejected.
2. **Given** a URL whose host resolves to a private IP while not allowlisted, **When** checked, **Then** it is denied.

### Edge Cases

- Uuid parse failure on an ID string yields `UuidParseError`, never a panic.
- `ContentHash::try_new` rejects odd-length or non-lowercase-hex strings.
- `redact_json_value` on a `secrets` subtree replaces the whole subtree with the marker (fail-closed); JSON/text rendering asymmetry is a known wart owned elsewhere, not in this crate.
- Invalid trust-level transitions (e.g. promoting `Quarantined` without review) are rejected via `DomainError::InvalidTrustTransition`.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Crate MUST expose Uuid-backed newtype IDs for every aggregate/entity (`ApprovalId`, `AuditEventId`, `CorpusGenerationId`, `DependencySnapshotId`, `DocumentId`, `EditionId`, `JobId`, `OutboxEventId`, `PrincipalId`, `ProvenanceId`, `RunId`, `SourceId`, `SourceVersionId`, `TombstoneId`, `WorkspaceId`) with `Display`/`FromStr`/serde-as-string, `new()` (v4 random), and `inner()` accessor (`crates/domain/src/ids.rs`).
- **FR-002**: Crate MUST provide the `Diagnostic` envelope (`id`, `timestamp`, `severity`, `code`, `category`, `message`, `location`, `affected_resource`, `remedy`, `next_command`) with `render_human` and `render_json`, both applying `redact_text` to every free-text field at render time (`crates/domain/src/diagnostic.rs`).
- **FR-003**: Crate MUST provide `DiagnosticCode { namespace, code }`, unique workspace-wide; uniqueness is test-enforced (`crates/domain/src/diagnostic.rs`).
- **FR-004**: Crate MUST provide `ContentHash { algorithm, hex }` with `HashAlgorithm::{Sha256, Blake3}` tags and validated lowercase even-length hex, plus `canonical_json_bytes` (stable key order, NFC-preserving, LF, no BOM) (`crates/domain/src/hashing.rs`).
- **FR-005**: Crate MUST provide `DataLayer::{Canonical, Publisher, Scholarly, Computational, User, Agent}`, ordered `TrustLevel` (CanonicalVerified down to Quarantined), `VerificationStatus::{Unverified, Processing, Verified, Rejected}`, and `SideEffectClass::{None, Readonly, Mutation, CanonicalWrite, ExternalAccess}` (`crates/domain/src/types.rs`).
- **FR-006**: Crate MUST provide secret redaction Rules A (secret-named keys), B (`key[:=]value` free text), C (URL userinfo) via `is_secret_key`, `redact_json_value` (returns redaction count), `redact_text` (borrowed-original zero-alloc hot path), and `REDACTED_MARKER = "***REDACTED***"` (`crates/domain/src/redaction.rs`).
- **FR-007**: Crate MUST provide deny-by-default guards: path containment (`security`), archive limits (`security_archive`), SSRF/resolved-IP + allowlist checks (`security_net`), input size/depth caps (`security_input`), HTML sanitization (`security_sanitize`) — all pure (no I/O, no async).
- **FR-008**: Crate MUST provide `LicenseRecord`/`LicenseStatus`, `SemVer`/`Timestamp`/`Language`/`Confidence` primitives, `SubjectRef`/`DerivationVersions` provenance subjects, and `CorpusGeneration`/`CorpusScope`/`OutboxEvent`/`OutboxOperation`/`OutboxState`/`PropagationState`/`Tombstone`/`TombstoneReason` generation types.
- **FR-009**: Crate MUST depend only on `(serde, thiserror, time, uuid)` plus dev/test crates — no I/O, no async runtime, no model/vector/embeddings dependencies (enforced by `cargo xtask arch-check` vs `xtask/allowlist.toml`; Constitution VII / invariant I2).

### Key Entities

- **Typed ID**: Uuid-backed distinct type per aggregate; string-serialized; prevents cross-entity confusion at compile time.
- **Diagnostic**: Structured error event with namespaced code, severity, category, message, location, affected resource, remedy, next command; renders secret-safe in two formats.
- **ContentHash**: Algorithm-tagged (`Sha256`/`Blake3`) lowercase-hex checksum over canonical JSON bytes; basis of reproducibility checksums.
- **TrustLevel / DataLayer**: Ordered trust profile (PRD §23) and five-layer data separation (PRD §6: Canonical, Publisher, Scholarly, Computational, User/Agent).
- **CorpusGeneration / Tombstone / OutboxEvent**: Generation-stamped derived-data lineage and forward-only deactivation records.

### Error Surface

Observed in `crates/domain/src` on 2026-09-17 (registry test asserts global uniqueness):

- `QAI-DOM-0001…0005` — ID parsing and core domain validation errors.
- `QAI-DOM-1001` — invalid trust transition; `QAI-DOM-1002` — diagnostic/code format violations; `QAI-DOM-1005` — invalid diagnostic code format.
- `QAI-SEC-0001…0011` — security-guard rejections (path escape, archive limit, SSRF/allowlist, input caps, sanitization).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo test -p domain --lib` passes with zero failures (71 passed on 2026-09-17).
- **SC-002**: `cargo run -p xtask -- arch-check` reports no forbidden edges involving `domain`.
- **SC-003**: Every error rendered by the crate carries a unique namespaced code parseable by the doctor JSON schema.
- **SC-004**: Sentinel secret bytes appear in zero bytes of rendered diagnostic or redacted-JSON output (testkit sentinel suite green).

## Assumptions

- `domain` intentionally has no async runtime and performs no I/O; all guards are pure predicates over caller-supplied values (DNS resolution results are supplied by the caller, not fetched here).
- The `//! Phase 0 placeholder` header on `lib.rs` is stale doc drift: the crate holds 14 real modules and is Phase-0 complete per `docs/06-progress/status.md`. Code truth wins.
- Blake3 exists as an algorithm tag alongside Sha256; canonical Quran hashing recipes (ADR-0006/0108) standardize on SHA-256.
- No Tantivy, no embeddings, no LLM dependencies anywhere in this crate (invariant I2 holds by construction and is gate-enforced).
- The synthetic `test-edition-min` fixture is unrelated to this crate and is never referenced as scripture here.
