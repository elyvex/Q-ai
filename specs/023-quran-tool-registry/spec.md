# Feature Specification: Read-Only Quran Tool Registry

**Feature Branch**: `023-quran-tool-registry`

**Created**: 2026-09-19

**Status**: Draft

**Input**: User description: "read-only Quran tool registry: module-level reverse specification of the implemented tool-registry crate (crates/tool-registry/src/lib.rs, D1.8, PRD 11.4/11.5), derived from code truth. Document quran.get_ayah and quran.get_context, ReadOnly permissions (section 29), QuranBackend trait boundary (why it never depends on storage directly), and section-12 contract conformance. Code is source of truth. Gap: no spec; 009 mentions quran_tools.rs but not the registry crate."

**Source of truth**: `crates/tool-registry/src/lib.rs` (minimal local tool registry with the first two read-only tools, D1.8). Related: PRD §11.4 (context/comparison tools: `quran.get_ayah`, `quran.get_context`), §11.5 (graph tools — out of scope here), §12 (tool result contract) / §12.1 (reproducibility checksum), §29 (tool runtime and security — `ReadOnly` side-effect class), AC-P1-15. Complements `022-tools-result-contract-spec` (the `tools`-crate envelope contract this registry conforms to) and `009-application-root` (the `ReaderToolBackend` that implements the registry's backend trait — this spec covers the registry side of that boundary, not the backend).

**Constitution compliance**: `.specify/memory/constitution.md` v1.2.0 — Principle I (canonical reads are deterministic and model-free; tools return canonical retrieval only, never model memory), Principle III (every result self-describing with edition identity, references, and reproducibility checksum), Principle VII (workspace layering — the registry sits below `application` and never reaches sideways into storage).

> **Reverse specification.** The implementation is the source of truth; this document describes the contract it already delivers, and any deviation between this spec and the code is a defect in the spec. No new behavior is proposed; the goal is to close the "no spec" gap for the registry crate.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Exact ayah lookup with zero synthesis (Priority: P1)

A researcher (or, later, an agent) requests one ayah by reference (`1:1`, or edition-embedded `quran:{slug}@{version}:{surah}:{ayah}`). The tool returns the canonical text plus configured metadata (translations, word glosses, surface tokens) inside the standard result envelope — exact lookup, no synthesis, ever.

**Why this priority**: This is the first of the two tools that prove the §12 contract works from all interfaces before agents exist (PRD §11.4). No-fabrication starts here.

**Independent Test**: Call `quran.get_ayah` with a valid reference against a stub backend and assert the envelope carries the view, the canonical reference, edition id/version, and a deterministic reproducibility record; call it with a malformed reference (`:::`) and assert a typed `InvalidInput` error (`QAI-QUR-0311`), never invented text.

**Acceptance Scenarios**:

1. **Given** a valid reference and option flags (translations, glosses, tokens), **When** `quran.get_ayah` executes, **Then** the result envelope names `quran.get_ayah` at version 1.0.0, echoes the exact input query, lists the single canonical reference, attributes one `canonical` analysis source, records the edition actually read, and stamps a deterministic reproducibility record with the backend's corpus generation.
2. **Given** a malformed reference string, **When** `quran.get_ayah` executes, **Then** it fails with a typed invalid-input error identifying the tool — the backend is never consulted and no text is returned.
3. **Given** any `quran.get_ayah` result, **When** serialized and deserialized, **Then** the envelope round-trips identically (contract stability for later consumers).

---

### User Story 2 - Structure-bounded surrounding verses (Priority: P1)

A researcher reading an ayah asks for its surroundings. `quran.get_context` returns the focal verse plus a bounded window of neighbors (defaults: 3 before, 3 after, cap 11) that never crosses a structural boundary (surah, juz, ruku, page) when one is set — surrounding verses without arbitrary chunking (PRD §11.4).

**Why this priority**: Context retrieval must be structurally bounded; an unbounded window is a resource and correctness hazard (mirrors the reader-level caps in `009`).

**Independent Test**: Request context with explicit before/after/boundary/cap and assert the spec handed to the backend matches; request with `max_ayahs: 0` and assert a typed invalid-input rejection.

**Acceptance Scenarios**:

1. **Given** a focal ayah reference with before/after counts and a boundary (surah, juz, ruku, page, or none), **When** `quran.get_context` executes, **Then** the result lists the focal canonical reference plus every neighbor reference, attributes one `canonical` analysis source per reference, and carries the same §12 envelope fields as `get_ayah` at version 1.0.0.
2. **Given** `max_ayahs` below 1, **When** `quran.get_context` executes, **Then** it fails with a typed invalid-input error (`max_ayahs must be >= 1`) before parsing or reading anything.
3. **Given** omitted parameters, **When** defaults apply, **Then** the window is 3 before / 3 after, no structural boundary, cap 11.

---

### User Story 3 - Tools agents may call freely without approval (Priority: P2)

Both tools are `ReadOnly` under PRD §29: freely callable by authorized agents with no approval step. The registry's dependency shape makes this a structural property, not a policy flag — the crate cannot write because it has no path to any store.

**Why this priority**: §29 approval tiers hinge on side-effect class; a read tool that secretly depended on storage could not be audited as read-only.

**Independent Test**: Inspect the crate's dependencies and assert no storage, database, I/O-runtime, model, or embedding dependency exists; assert both registered names resolve and execute against a fake backend with no store present.

**Acceptance Scenarios**:

1. **Given** the registry crate's dependency list, **When** audited, **Then** it contains only the contract, domain-vocabulary, serialization, and async-trait crates — no storage, database driver, model, embedding, or vector dependency.
2. **Given** either registered tool name, **When** invoked through the registry against a non-storage backend, **Then** it executes fully — proving no hidden store coupling.

---

### Edge Cases

- References with an embedded edition (`quran:{slug}@{version}:…`) pass through to the backend untouched; edition resolution is the backend's job, not the registry's.
- A well-formed reference for a non-existent ayah or edition is a backend-mapped error (via `ReaderError` → `ToolError::Backend` codes such as `QAI-QUR-0310` family), never invented text — the no-fabrication half of AC-P1-15.
- `execution_time_ms` is wall-clock and therefore non-deterministic by design; it is excluded from reproducibility comparisons (the existing conformance test zeroes it before asserting round-trip equality).
- Normalization rules are always empty for these two tools (exact canonical reads); the field exists so later normalized-search tools can fill it under the same contract.
- Confidence is always absent (`None`) and warnings always empty on the current paths — quantitative confidence belongs to ranking/search tools, not exact lookup.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Registry MUST expose exactly two tools, `quran.get_ayah` and `quran.get_context`, both at version 1.0.0, listed by `tool_names()` in that order — the D1.8 pair that proves the §12 contract before agents exist.
- **FR-002**: `quran.get_ayah` MUST accept a reference string plus display options (translation slugs, word-gloss flag, token flag), parse the reference with the frozen grammar (malformed → `ToolError::InvalidInput` naming the tool, code `QAI-QUR-0311`), fetch exactly one ayah view through the backend, and return the §12 envelope: tool name/version, exact input query, empty normalization rules, edition id/version read, the single canonical reference, one `canonical` analysis source, results, no confidence, no warnings, wall-clock timing, and a deterministic reproducibility record (tool, version, query hash, edition slug/version, corpus generation).
- **FR-003**: `quran.get_context` MUST accept a focal ayah reference plus window parameters (before default 3, after default 3, boundary default none, cap default 11), reject `max_ayahs < 1` with a typed invalid-input error before any other work, parse the reference (same malformed-input behavior as FR-002), fetch through the backend with a structure spec (surah-header excluded), and return the §12 envelope whose references are the focal canonical reference followed by all before/after neighbor references, with one `canonical` analysis source each.
- **FR-004**: The registry MUST depend on the `QuranBackend` trait only — `backend_get_ayah(reference, options)` and `backend_get_context(reference, spec)` plus read metadata — and MUST NOT depend on storage, the reader, or any database driver. Rationale: (a) layering — workspace architecture routes cross-crate store access through `application`, and the registry sits below that boundary so CLI/server/agents share one backend implementation; (b) auditability — `ReadOnly` (§29) is provable from the dependency list instead of trusted from a flag; (c) testability — every registry test substitutes a fake backend with no store.
- **FR-005**: The backend MUST supply per-read metadata (`BackendMeta`: edition slug, version, row id, corpus generation, edition text hash for ETags, script, optional transmission, numbering scheme) and the registry MUST surface it twice: edition id/version inside the envelope and reproducibility record, and the full metadata alongside the result for API envelopes and ETags.
- **FR-006**: Both tools MUST be `ReadOnly` (§29 side-effect class): no writes, no annotations, no external calls on any path; freely callable by authorized agents with no approval step.
- **FR-007**: Every result MUST conform to the §12 contract documented in `022-tools-result-contract-spec` (self-describing envelope + reproducibility checksum); the envelope MUST serialize/deserialize losslessly for later consumers (CLI, HTTP API, future agents).
- **FR-008**: Non-existent locations or editions MUST surface as typed backend errors mapped from the reader (never synthesized text) — the no-fabrication requirement of AC-P1-15.

### Key Entities

- **ToolRegistry**: The registry over one `QuranBackend`; owns tool names, versions, parameter parsing, and envelope assembly — no store access.
- **QuranBackend**: The two-method async trait (`backend_get_ayah`, `backend_get_context`) implemented by the application layer (`ReaderToolBackend` over `QuranReaderService`); the sole seam between tools and data.
- **BackendMeta**: Per-read metadata — edition slug/version/row-id, corpus generation, edition text hash, script, optional transmission, numbering scheme.
- **GetAyahParams / GetContextParams**: Tool inputs — reference string plus display flags (ayah) or window spec with boundary enum and cap (context); serializable so the exact query is echoed in the envelope.
- **ContextBoundaryArg**: Serializable boundary (surah, juz, ruku, page, none-default) mapping 1:1 onto the domain `ContextBoundary`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A researcher requesting any valid ayah reference receives the exact canonical text with its edition identity and canonical reference in a self-describing result, through every interface (library, CLI, HTTP) — no interface returns a differently-shaped or unattributed answer.
- **SC-002**: Zero tool invocations produce verse text from anything but canonical retrieval: malformed input and missing locations/editions always yield typed errors, and reviewers can confirm read-only status from the dependency list alone.
- **SC-003**: Re-running an unchanged deterministic query reproduces the identical checksum; running after a corpus change produces a different checksum — researchers can detect drift without re-reading the corpus.
- **SC-004**: The registry conformance suite (both tools listed, `get_ayah` contract fields + determinism + round-trip, malformed-input code, context parameter validation) passes with zero failures.

## Assumptions

- Graph, comparison, morphology, frequency, and rhetorical tools (PRD §11.3, §11.5, §11.6 and the rest of §11.4) are out of scope; they arrive as later tools on this same registry pattern with the same §12 envelope.
- The `ReaderToolBackend` implementation (edition resolution, generation lookup, error mapping `to_tool_error`, `tool_get_ayah`/`tool_get_context` conveniences) is specified via `009-application-root`; this spec covers only what the registry demands of any backend.
- The envelope field definitions and checksum input rules live in `022-tools-result-contract-spec`; this spec asserts conformance to them rather than restating them.
- Version 1.0.0 for both tools marks the frozen D1.8 contract; version bumps follow tool-behavior changes per the §12 tool plan.
