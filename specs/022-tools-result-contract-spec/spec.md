# Feature Specification: Tool Result Contract & Reproducibility Checksum

**Feature Branch**: `022-tools-result-contract-spec`

**Created**: 2026-09-19

**Status**: Draft

**Input**: User description: "Tool result contract and reproducibility checksum: module-level reverse specification of the implemented tools crate (crates/tools/src/lib.rs, PRD section 12/12.1), derived from code truth. Document ToolResult fields, canonical_references, ReproducibilityData checksum inputs, deterministic Phase-1 inputs vs deferred model/prompt fields. Code is source of truth. Gap: no spec; required by every quran.* tool."

> **Reverse specification.** This feature is a *specification* of an already-implemented contract. The implementation
> (`crates/tools/src/lib.rs` plus its consumers `tool-registry`, `application::quran_tools`, and the HTTP API layer) is the
> source of truth; this document describes the contract it already delivers, and any deviation between this spec and the
> code is a defect in the spec. No new behavior is proposed; the goal is to close the "no spec" gap so that every
> `quran.*` tool implemented now or later conforms to one documented contract.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A tool result is fully self-describing (Priority: P1)

A researcher runs any `quran.*` tool (e.g. `quran.get_ayah`, `quran.get_context`) through any interface. The result they
receive is a single, self-contained envelope that names the tool that produced it, its version, the exact input that was
sent, where the data came from, and how long it took — so nothing about the result requires out-of-band knowledge to be
interpreted or re-verified.

**Why this priority**: This is the core of PRD §12 and the reason the contract exists. Without a self-describing
envelope, no result can be traced to a source or re-run, which is a non-negotiable research requirement.

**Independent Test**: Request an ayah via `quran.get_ayah` and verify the returned envelope contains every required
fields (tool identity, version, exact query, edition identity/version, results, canonical references, analysis sources,
warnings, timing, reproducibility record). Can be fully tested against any registered read-only tool and delivers the
guaranteed-result shape every downstream consumer depends on.

**Acceptance Scenarios**:

1. **Given** a registered read-only `quran.*` tool, **When** the tool is invoked with valid input, **Then** the returned
   result envelope includes the tool name, tool version, the exact input query, every canonical reference,
   edition identity and version when an edition was read, analysis sources, non-fatal warnings, wall-clock execution
   time, and a reproducibility record — and nothing is omitted.
2. **Given** any result that reads an edition, **When** the result is inspected, **Then** the edition identifier and
   edition version that were actually read are present in both the envelope and the reproducibility record.
3. **Given** a quoted Quran passage in a result, **When** the result is inspected, **Then** the canonically referenced
   location for that passage is listed in the result's canonical references, and the quotation text is taken from
   canonical retrieval, never from model memory.

---

### User Story 2 - Every result re-runs reproducibly (Priority: P1)

A researcher previous run produces a reproducibility checksum. That checksum, combined with the tool result, is enough
to re-run the same deterministic query later and detect whether anything changed (corpus update, edition version change,
tool version change) by comparing checksums.

**Why this priority**: Reproducibility (PRD §12.1) is the mechanism that lets a user trust that a result they relied on
still holds, and it flags exactly which input (query, edition, source, corpus generation) drifted.

**Independent Test**: Run `quran.get_ayah` twice against an unchanged corpus with identical input. Verify both results
carry identical reproducibility checksums and identical query hashes, and both are marked deterministic. Then re-run
against a different corpus generation and verify the checksum changes.

**Acceptance Scenarios**:

1. **Given** an unchanged corpus and edition, **When** the same deterministic tool runs twice with byte-identical input,
   **Then** the two reproducibility checksums are identical and both results are marked fully deterministic.
2. **Given** a reproducibility record, **When** it is inspected, **Then** it exposes a query hash, a tool plan listing
   tool name and version used, the source versions and edition versions read, the corpus generation read, and a combined
   checksum computed over exactly those inputs.
3. **Given** a result whose inputs differ (different corpus generation, edition version, source version, or query),
   **When** the checksums are compared, **Then** the checksums differ — a user can always tell that inputs changed.
4. **Given** a query containing a secret, **When** the result is produced, **Then** the stored query is
   secret-redacted by the caller so secrets never appear in the envelope.

---

### User Story 3 - Deferred inputs are explicitly labeled (Priority: P2)

A user inspecting a Phase-1 result sees that normalization rules, retrieval configuration, and model/prompt fields are
present-but-empty, and the record states whether the result is deterministic. The contract is therefore stable today and
ready for later phases without breaking change.

**Why this priority**: Phase 1 tools are deterministic; model/prompt inputs arrive later. Users must be able to tell
apart "this was deterministic" from "this was model-generated" on every result, now and in the future (PRD §12.1:
non-deterministic or model-generated parts must be labeled separately).

**Independent Test**: Inspect the reproducibility record of any current tool result and verify: normalization rule set,
retrieval config hash, model provider, model name, and prompt version are present as unset/empty, while the
deterministic flag is true. Can be tested with no additional tooling.

**Acceptance Scenarios**:

1. **Given** a Phase-1 read-only tool result, **When** the reproducibility record is inspected, **Then** the
   normalization-rule-set, retrieval-config, model-provider, model-name, and prompt-version fields are explicitly
   unset — never randomly absent — and the record is marked deterministic.
2. **Given** any current tool result, **When** the result carries no confidence value, **Then** confidence is present
   but empty, and warnings are present even when empty — the envelope shape never changes between calls.

---

### User Story 4 - Failures are typed and stable (Priority: P2)

When a tool call fails, the failure is not ad-hoc: invalid input is reported as one stable error class with the tool
name and a human-readable detail, and backend failures carry a machine-readable namespaced code plus detail.
Consumers can branch on error type/code.

**Why this priority**: Tools are called from API and CLI surfaces that need to map failures to user-facing messages and
HTTP codes. Stability of the error taxonomy is part of the contract.

**Independent Test**: Call `quran.get_ayah` with a malformed reference and verify the failure is reported as an invalid
input error with a stable code and the tool name; trigger a backend failure path and verify it carries a namespaced
backend code. Both are fully testable via the existing tool surface.

**Acceptance Scenarios**:

1. **Given** a tool call with malformed input, **When** the call fails, **Then** the failure is reported as an invalid
   input error carrying the tool name, a human-readable detail, and its stable error code.
2. **Given** a tool call whose backend fails, **When** the call fails, **Then** the failure is reported as a backend
   error carrying a machine-readable namespaced code and a human-readable detail.

---

### Edge Cases

- **Empty results**: a tool that legitimately returns no rows still returns a full envelope with empty `results` and
  empty `warnings` — shape is invariant.
- **No edition read**: when a result does not read an edition, edition identity/version are present-but-unset and the
  edition-versions map in the reproducibility record is empty.
- **No analysis sources**: results that carry no analysis sources still include the empty sources list; canonical
  references and analysis sources are distinct and neither may be silently substituted.
- **Timing**: wall-clock execution time is inherently non-deterministic and is *excluded* from the reproducibility
  checksum, so identical runs still get identical checksums despite different timings.
- **Multiple editions/sources**: when more than one edition or source is read, every one appears in the
  edition-versions / source-versions maps — one reading's checksum never stands in for another.
- **Malformed input vs backend failure**: the two failure classes are distinct and reported with different stable codes;
  a parser rejection is never surfaced as a backend failure and vice-versa.
- **Secrets in the query**: secret-bearing inputs must be redacted before the query is placed in the envelope; the
  quoted and hashed form must not leak the secret.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Every `quran.*` tool result MUST be a single envelope containing the tool name, tool version, the exact
  input query, the normalization rules applied, edition identifier and edition version (when an edition was read), the
  results, canonical references, analysis sources, confidence (when reported), non-fatal warnings, wall-clock execution
  time, and a reproducibility record.
- **FR-002**: The reproducibility record MUST include a combined checksum, a hash of the exact query, a tool plan
  listing each tool by `name@version`, the source versions read, the edition versions read (`slug` → version), the
  corpus generation read, and a deterministic flag.
- **FR-003**: The combined checksum MUST be computed over exactly: tool identity and version, the query hash, the
  edition versions read, the source versions read, and the corpus generation — and over nothing else.
- **FR-004**: The query hash and combined checksum MUST be SHA-256 of canonical serialized input and MUST be identical
  for identical inputs and identical read-state, and MUST differ when any checksum input changes.
- **FR-005**: In the current (Phase 1) generation, the reproducibility record MUST mark the result deterministic and
  MUST explicitly leave unset the normalization rule set, retrieval configuration hash, model provider, model name, and
  prompt version — reserving them for later phases without changing the record's shape.
- **FR-006**: Normalization rules reported by the tool MUST be empty in Phase 1 and MUST carry the actual rules applied
  once normalization-aware tools ship, never a fabricated rule list.
- **FR-007**: Canonical references MUST be fully qualified references for every result item, and every quoted canonical
  passage MUST appear in the canonical references; analysis sources MUST separately record each source's kind
  (canonical, translation, or metadata) and reference.
- **FR-008**: Wall-clock execution time MUST be measured per call and MUST NOT participate in the reproducibility
  checksum.
- **FR-009**: Tool input failures and backend failures MUST be distinct error classes, each with a stable, documented
  code, and both MUST carry a human-readable detail; input failures MUST also name the failing tool.
- **FR-010**: Envelope behavior MUST be invariant — empty lists and unset fields are present-but-empty, never omitted —
  so consumers can rely on shape stability across every `quran.*` tool and every call.

### Key Entities *(include if feature involves data)*

- **Tool result envelope**: the self-describing container returned by every `quran.*` tool; carries tool identity,
  exact query, edition identity/version, results, canonical references, analysis sources, confidence, warnings, timing,
  and the reproducibility record.
- **Canonical reference**: a fully qualified, edition-pinned pointer to a canonical location; the identity used in the
  envelope's references list and the basis for quotation verification.
- **Analysis source**: a single source behind a result, typed as canonical, translation, or metadata, with a reference;
  distinct from the canonical-references list.
- **Reproducibility record**: the deterministic audit block of a result; holds the combined checksum, query hash, tool
  plan, source versions, edition versions, reserved-but-unset later-phase fields, corpus generation, and the
  deterministic flag.
- **Reproducibility checksum**: the content-addressed digest over the deterministic inputs (tool identity/version,
  query hash, edition versions, source versions, corpus generation) that lets a user re-run a query and detect drift.
- **Tool identity**: the tool name (e.g. `quran.get_ayah`) plus its version; what appears per-item in the tool plan.
- **Tool failure**: the typed error outcome of a call — either invalid input (with the tool name and detail) or backend
  failure (with a namespaced code and detail) — each with a stable code.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of registered `quran.*` tools return results conforming to the envelope contract; a contract-shape
  conformance test passes for all of them.
- **SC-002**: 100% of deterministic tool calls to an unchanged corpus with identical input reproduce an identical
  reproducibility checksum; a changing checksum input (query, edition version, source version, or corpus generation)
  produces a different checksum in 100% of cases.
- **SC-003**: A user can determine whether any given result was fully deterministic, and which later-phase inputs were
  not yet applied, purely from the result's reproducibility record — no out-of-band knowledge required.
- **SC-004**: Every result that quotes canonical text can be traced to a canonical reference, and 100% of such passages
  are sourced from canonical retrieval rather than model memory.
- **SC-005**: 100% of tool failures resolve to one of the two documented error classes with a stable code; no
  user-visible failure lacks a code.

## Assumptions

- **Code is truth**: where the PRD text and the implementation differ, the implementation described here governs until a
  code change (documented as phase scope) alters the contract.
- **Phase-1 scope**: the implemented contract covers deterministic, read-only tools (`quran.get_ayah`,
  `quran.get_context` at version 1.0.0). Normalization, retrieval config, and model/prompt inputs are reserved fields —
  they are declared but unset until phases 2 and 9 respectively.
- **Scope boundary**: this spec documents the *contract* (the envelope and its checksum behavior). Tool-specific
  behaviors (what each tool computes, how references parse, what views look like) are specified by their own features and
  are out of scope here.
- **Consumers**: the HTTP API, CLI, and future agent surfaces consume the same envelope; the contract does not vary by
  interface.
- **Timing**: wall-clock execution time is measured but excluded from checksums; this is an intentional, documented
  exception to determinism.
- **Secrets**: callers are responsible for redacting secrets from the stored query before it enters the envelope; the
  contract requires the redacted form, not the redaction mechanism.
- **Dependency**: the contract builds on the platform's content-hash and semantic-version primitives; any change to
  those primitives is a downstream consequence outside this feature's scope.