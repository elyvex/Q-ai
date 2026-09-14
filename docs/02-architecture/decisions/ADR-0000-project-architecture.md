# ADR-0000 — Project Architecture

- Status: Accepted
- Phase: 0 — Foundations
- Date: 2026-09-14
- Related decisions: ADR-0001, ADR-0002, ADR-0006, ADR-0012, ADR-0702
- Requirements: PRD §6, §22, §32.1, §33, §43, §48, §55, §87

## Context

Q-ai is a research platform for the Quran, hadith, Islamic literature, and
comparative scripture. Its central constraint is **epistemic integrity**: exact
text before generated interpretation, every claim traceable to a source version,
canonical text separated from translations and annotations, and no model allowed
to fabricate a verse, hadith, chain, grading, or citation. Those are not
post-hoc policy checks — they must be enforced by the type system and the
database before any religious corpora are imported.

This ADR records the top-level architecture Phase 0 implements so later phases
extend it rather than re-litigate it. The detailed decisions live in the
numbered ADRs (0001 relational store, 0002 migrations, 0003 job system, 0004
configuration, 0005 secrets, 0006 hashing, 0007 manifests, 0008 provenance,
0009 audit, 0010 errors, 0011 observability, 0012 boundaries, 0702 cross-store
consistency).

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| **Monolith with a single `core` crate** | Simple build | No compile-time separation of canonical integrity; easy to bypass guards |
| **Microservices from day one** | Independent scaling | Premature; operational cost; violates local-first Phase 0 scope |
| **Layered Cargo workspace, local-first, single binary** | Compile-time boundaries; one deployable; storage abstraction keeps PostgreSQL migration open | More crates to wire; discipline required in the allowlist |

## Decision

Adopt a **layered Cargo workspace** with an enforced dependency graph
(ADR-0012) and a **local-first single binary**:

1. **Domain layer** (`domain`) — pure, dependency-light types: newtype IDs,
   `DataLayer`/`TrustLevel`/`VerificationStatus`, `ContentHash` + canonical JSON,
   licensing, and security primitives (path/archive/network/input guards). No
   I/O, no async runtime.
2. **Contract layer** (`storage`, `config`, `provenance`, `audit`, `jobs`,
   `sources`) — traits and models. `storage` defines `Database`, `ReadTx`,
   `UnitOfWork` and the repository traits; concrete backends live elsewhere.
3. **Implementation layer** (`storage-sqlite`, `observability`, `server`) —
   SQLite behind the storage traits, tracing/metrics, and a health server.
4. **Composition root** (`application`) — wires domain + storage + SQLite +
   provenance + audit + jobs + sources + config. This is the only crate that
   selects a concrete backend.
5. **Interfaces** (`cli`, `server`) — the `qai` binary and the HTTP surface.
   Interfaces depend on `application`, never on `storage-sqlite` directly.

Canonical immutability is a **type-level** property: canonical rows are written
only through a `CanonicalWriter` that requires an `ApprovalToken` obtained from
a persisted human `ApprovalRecord`. Provenance and audit are recorded for every
mutation; audit is append-only and hash-chained. Projection-relevant writes and
their outbox rows share one transaction (ADR-0702).

## Accuracy Implications

- Separation of `CanonicalSource` from `PublisherMetadata`, `ScholarlyAnnotation`,
  `ComputationalAnnotation`, and `UserOrAiNote` (PRD §6) is enforced by types, so
  layers cannot be silently merged.
- `TrustLevel::may_support_definitive_claim` encodes "popularity or vector
  similarity never implies authority" (PRD §23).

## Religious-Source Implications

- No religious text is imported in Phase 0; the chassis exists precisely so that
  Phase 1's Quran edition cannot become active without a known license, a verified
  content hash, a structural validation report, and a human approval record.
- Canonical rows are insert-only at the database level (triggers), preventing an
  importer or agent from rewriting scripture after approval.

## Licensing Implications

- Source licensing (`LicenseStatus`, `LicenseRecord`) is first-class: `Approved`
  is impossible while the license status is `Unknown` (ADR-0007, ADR-0010).
- Workspace dependencies are constrained to a permissive allowlist by
  `cargo-deny` (GPL/AGPL denied).

## Security Implications

- **Deny-by-default** posture: network egress, tool execution, and filesystem
  side effects default to empty/denied.
- Security guards (path containment, archive safety, SSRF, input validation,
  sanitization) are reusable, tested libraries used by every later phase, and
  **fail closed** when they cannot be evaluated.
- Secrets never persist in SQLite; the database stores only `SecretRef`s.

## Operational Implications

- One binary (`qai`) plus a local SQLite database; no external broker in local
  mode (ADR-0003). Migrations are append-only and checksummed (ADR-0002).
- `qai doctor` is read-only and produces a stable, schema-validated JSON report.

## Migration Strategy

- The storage abstraction keeps a PostgreSQL backend open without domain changes
  (ADR-0001); SQLite is the Phase 0 default.
- New subsystems land as new crates with an allowlist entry; boundaries tighten
  as the workspace grows.

## Reversal Cost

- **High** for the layer graph once code depends on it; the allowlist is editable
  but relaxing a boundary weakens compile-time integrity guarantees.
- **Moderate** for the local-first/single-binary choice — the composition root
  isolates the storage backend, so a service split is additive rather than a
  rewrite.

## Acceptance Criteria

- `cargo xtask arch-check` enforces the dependency graph (AC-P0-02).
- Canonical provenance cannot be updated or deleted; writes require an
  `ApprovalToken` (AC-P0-06).
- Audit is append-only and hash-chained (AC-P0-13).
- `qai doctor` runs read-only and validates against its schema (AC-P0-14).
