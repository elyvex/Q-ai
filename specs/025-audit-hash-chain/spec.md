# Feature Specification: Audit Hash-Chain Model

**Feature Branch**: `025-audit-hash-chain`

**Created**: 2026-09-19

**Status**: Draft

**Input**: User description: "audit hash-chain model: module-level reverse specification of the implemented audit crate (crates/audit/src/lib.rs, D0.12, ADR-0009), derived from code truth. Document append-only events, chain hashing, verifier, redaction of secrets, and audit_bridge usage in crates/application/src/audit_bridge.rs. Code is source of truth. Gap: foundation crate with zero spec"

**Source of truth**: `crates/audit/src/lib.rs` (Phase 0 audit model, D0.12) and `crates/application/src/audit_bridge.rs` (bridge from storage audit rows to the hash-chained writer). Related: ADR-0009 (audit integrity — hash chain + append-only triggers), PRD §§82, 91. This spec closes the foundation-crate gap: the audit crate previously had zero spec.

**Constitution compliance**: `.specify/memory/constitution.md` v1.2.0 — Principle II (layered trust and provenance: every canonical change requires an audit event; import visibility goes through staging → approval → activation → audit), Principle VI (local-first security: global redaction layer, `Secret<T>` handling, sentinel-leak suite; audit reads via `verify_persisted_audit` open the database read-only), Principle VII (SQLite-adjacent, zero-extra-dependency solutions; contiguous append-only migrations — the `audit_events` table created by `0005_audit`).

> **Reverse specification.** The implementation is the source of truth; this document describes the model it already delivers, and any deviation between this spec and the code is a defect in the spec. No new behavior is proposed.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Every mutation leaves a tamper-evident trace (Priority: P1)

Whenever the system mutates protected state — config changes, source discovery/import/validation/staging/approval/activation/rollback, canonical change sessions, approvals, principal/role changes, dead-lettered jobs, doctor repairs — it appends one audit event. Each event links to the previous event's hash, so any later edit, deletion, or reordering of history breaks the chain and is detected.

**Why this priority**: The audit log is the credibility substrate for every claim about source origin and canonical integrity (ADR-0009). Silent history editing must be structurally impossible, not merely forbidden.

**Independent Test**: Append three events through the writer against a throwaway repository; assert sequences 1/2/3, each event's `prev_chain_hash` equal to the previous event's `chain_hash`, and the first event's `prev_chain_hash` equal to the genesis hash. Then tamper with one stored row's content and assert full verification fails and names the tampered sequence.

**Acceptance Scenarios**:

1. **Given** an event to record, **When** appended through the writer, **Then** it receives the next gapless sequence number, a fresh timestamp, the previous event's chain hash as its `prev_chain_hash`, and a newly computed `chain_hash` — the caller never supplies sequencing or linkage.
2. **Given** a stored event whose content was modified after writing (without updating its hash), **When** the chain is verified, **Then** verification reports invalid and identifies the exact sequence with the recomputed expected hash versus the stored actual hash.
3. **Given** a stored chain with a removed row, **When** the chain is verified, **Then** verification reports invalid and lists the gap.
4. **Given** any direct `UPDATE` or `DELETE` against the audit table, **When** executed, **Then** the database aborts it (append-only triggers fire even for processes holding write permission).

---

### User Story 2 - Operators verify history offline (Priority: P1)

An operator (or a nightly job) verifies the whole audit chain without network access: recompute every hash from genesis, check sequence continuity and linkage, and get a single valid/invalid answer with the evidence needed to locate damage.

**Why this priority**: Local-first verifiability with no external dependency is the accepted ADR-0009 option; a broken chain must visibly fail (report FAIL), never silently degrade.

**Independent Test**: Verify a freshly-written chain (expect valid, correct next-sequence/next-hash expectations); tamper with content, linkage, and sequencing in three separate experiments and assert each is detected with the right evidence.

**Acceptance Scenarios**:

1. **Given** an intact persisted chain, **When** verified, **Then** the report is valid, names the number of events checked, and states the expected next sequence and next hash.
2. **Given** a chain with altered content or a rewritten link hash, **When** verified, **Then** the report is invalid and names each tampered sequence.
3. **Given** a chain with a missing or renumbered sequence, **When** verified, **Then** the report is invalid and names the gap.
4. **Given** a chain-integrity diagnostic, **When** shown to an operator, **Then** it points at the verify command as the next step.

---

### User Story 3 - Secrets never reach the audit log (Priority: P1)

Audit payloads pass through the workspace redaction layer before storage: credential-shaped values are scrubbed while benign prose is preserved. A permanent gate proves a sentinel secret never appears in any audit row.

**Why this priority**: The audit log is long-lived and widely read (operators, backups, verification jobs). One leaked credential in an audit payload persists for the life of the chain, which by design cannot be redacted after the fact (ADR-0009: irreversibility).

**Independent Test**: Submit a payload containing password/api-key-shaped values plus benign prose (e.g. "approval token issued"); assert the credential values are replaced with the redaction marker and the prose survives verbatim.

**Acceptance Scenarios**:

1. **Given** an audit payload containing secret-shaped values, **When** redacted, **Then** each secret value is replaced with the marker while non-secret values are byte-identical.
2. **Given** benign text mentioning credential-adjacent words without credential shape, **When** redacted, **Then** the text is preserved (pattern rules, not bare substring matching).

---

### User Story 4 - State changes and their audit records commit atomically (Priority: P2)

Application services append audit events inside their own units of work: the mutation and its audit record share one transaction, using the exact same hash recipe as the standalone writer — so same-transaction events verify identically under the standard verifier.

**Why this priority**: Without atomicity, a crash between mutation and audit leaves either an unrecorded change or a record of a change that never happened — both break the traceability the chain exists to guarantee (e.g. activation's pointer flip + generation bump + audit in one transaction).

**Independent Test**: Append two events inside one transaction and commit; assert they verify clean under persisted verification. Roll back a transaction containing an event write; assert the event is absent.

**Acceptance Scenarios**:

1. **Given** a unit of work that mutates state, **When** the service appends its audit event through the bridge and commits, **Then** both persist together and the event verifies under the standard recipe.
2. **Given** the bridge's row mapping, **When** an event round-trips (event → row → event), **Then** actor, action, outcome, hashes, and payloads are identical.

---

### Edge Cases

- The genesis `prev_chain_hash` (all-zero SHA-256) is a fixed constant, not stored anywhere — both the writer and every verifier derive it independently.
- A missing predecessor row (sequence N−1 absent when appending N) falls back to genesis rather than failing — writer and bridge agree on this behavior.
- Single-event verification needs two rows (predecessor + target); with fewer rows available it returns false rather than erroring.
- A stored `agent` actor kind has no audit-enum spelling and reads back as a named `System` actor — a documented one-way mapping (agents are representable in rows, normalized on read).
- Unknown stored values (action, outcome, actor kind, hash algorithm, timestamp, non-JSON payload, bad principal id) surface as typed storage errors on read — corrupt rows fail loudly at verification time, never silently.
- The bridge's trait-level `verify_chain` delegates to the storage layer's structural check and carries no per-event tamper detail; the full recompute-and-locate behavior lives in the crate verifier and in persisted verification.
- Chain reversal cost is intentionally very high: removing the chain would invalidate every stored hash. By design this is irreversible.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The model MUST define append-only audit events carrying identity, gapless sequence, timestamp, actor (`Principal` / `System` / `Job`), action (config, secret-ref, migration, source lifecycle, canonical sessions, approvals, principal/role, dead-letter, doctor repair), subject reference, outcome (`Allowed` / `Denied` / `Failed`), optional reason/before/after/request-id, and the `prev_chain_hash` → `chain_hash` link.
- **FR-002**: Appending MUST be sequencer-owned: the writer assigns `latest_sequence + 1`, stamps the current time, links to the predecessor's chain hash (genesis for sequence 1), computes the new chain hash, and then stores — callers MUST NOT supply sequence, timestamp, or linkage.
- **FR-003**: The chain-hash recipe MUST be `SHA-256( prev_hash_hex || canonical_JSON(event with both hash fields blanked) )`, with the recipe public so same-transaction writers reuse it exactly; different events MUST produce different hashes under the same predecessor.
- **FR-004**: Full verification MUST recompute every hash from genesis (not merely check linkage), and MUST report validity plus the expected next sequence/hash, every sequence gap, and every tampered sequence with expected-versus-actual hashes.
- **FR-005**: The audit table MUST reject `UPDATE` and `DELETE` at the database trigger layer, independent of application permissions; audit diagnostics MUST use stable codes (`QAI-AUD-0001`…`0007` covering not-found, append-only violation, chain failure, gap, hash mismatch, secret leak, storage) with the verify command as the operator's next step for integrity failures.
- **FR-006**: Every audit payload MUST pass through the workspace redaction function before storage, scrubbing credential-shaped values by pattern rules while preserving benign text; a standing gate MUST prove a sentinel secret never lands in any audit row.
- **FR-007**: The application bridge MUST map losslessly between audit events and storage rows (hashes in `algorithm:hex` text form, `snake_case` action/outcome spellings, actor kind/id split with the `agent`→`System` read-back rule), MUST append inside the caller's unit of work atomically, and MUST verify persisted chains read-only (sequence continuity, link equality, hash recomputation) reporting checked count, gaps, and tampered sequences.
- **FR-008**: The repository seam MUST be the async trait (`append`, `list_by_subject`, `list_by_sequence`, `verify_chain`, `latest_sequence`) with storage owning the tables and the audit crate owning the recipe — the crate MUST NOT depend on any concrete database.

### Key Entities

- **AuditEvent**: The single immutable record — who (actor), what (action), on what (subject), with what result (outcome), what changed (before/after), and its position in the chain (sequence + hash link).
- **HashChainWriter / append_audit_event**: The two append paths — standalone writer over any repository, and same-transaction bridge append inside a unit of work — both running the identical recipe.
- **AuditVerifier / persisted verification**: Full-recompute chain checking returning validity, next-sequence/next-hash expectations, gaps, and per-sequence tamper evidence.
- **StorageAuditBridge**: The row/event mapper plus trait adapter between the storage audit tables and the audit crate's recipe.
- **Redaction function**: The pre-storage scrub over audit payloads, delegating to the workspace's single key-matcher and marker.
- **AuditError**: Typed failures with stable `QAI-AUD-nnnn` codes distinguishing missing events, append-only violations, chain/gap/hash failures, secret leaks, and storage faults.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Any tampering with stored audit history (edited content, rewritten link, removed or renumbered row) is detected by verification that names the affected sequence — an operator checking history integrity always gets a definitive valid/invalid answer with locatable evidence.
- **SC-002**: No database write path can modify or delete an audit record: direct update/delete attempts are refused at the database layer regardless of caller permissions.
- **SC-003**: Zero secrets reach stored audit payloads: credential-shaped values are scrubbed before storage, benign text is preserved, and the standing sentinel gate stays green.
- **SC-004**: Every protected state change (activation, rollback, import lifecycle, approvals, repairs) commits its audit record in the same transaction as the change itself — no committed change lacks its record and no record exists for a rolled-back change.
- **SC-005**: The model test suite (hash computation, hash uniqueness, redaction, actor coverage, writer linkage, tamper detection, gap detection, persisted round-trip verification, bridge mapping) passes with zero failures.

## Assumptions

- Observed ADR variances are recorded as code truth, not spec defects: the crate's `Actor` enum has no `Agent` variant (ADR-0009 lists `Agent(future)`; the bridge handles stored `agent` rows via the `System` read-back rule); trigger abort messages and crate error codes share the `QAI-AUD-000n` family with different per-code assignments — operators should rely on the code's `code()` mapping.
- The `before`/`after` payload allowlist and writer-schema key rejection described in ADR-0009 are enforced upstream of this model (redaction at the writer boundary); this spec covers the scrub-on-write guarantee, not the schema gate.
- Nightly `audit.verify_chain` scheduling (Phase 1+) and the `qai audit verify` / `qai audit list` operator surface are consumers of this model, specified with their own surfaces — this spec covers the model and bridge they build on.
- Verification cost is linear in chain length by accepted design decision; windowing or checkpointing a long chain is out of scope.
