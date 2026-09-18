# Feature Specification: quran-normalization rules engine

**Feature Branch**: `007-quran-normalization`

**Created**: 2026-09-17

**Status**: Draft

**Input**: Module-level specification of the implemented `quran-normalization` crate (crates/quran-normalization/src/{lib,rule,rules/n01–n22,pipeline,profile,span,trace,error}.rs), derived from code truth on 2026-09-17. Reverse specification of what exists today. Code is the source of truth over plan docs.

**Constitution compliance**: `.specify/memory/constitution.md` v1.0.2, Principles I/VII (I8: derived copies only, canonical text never mutated), III (I9 exact ordered rule sets via traces; I10 canonical offsets via SpanMap), VII (zero model deps on the derived path).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Strictness-ladder search forms (Priority: P1)

A researcher chooses a profile from L0 (exact canonical) up to L8 (heuristic affix handling) through whitespace → marks → diacritics → hamza → codepoints → skeleton steps. Profiles are versioned and append-only: old analyses stay reproducible because a profile version never changes meaning.

**Why this priority**: Invariants I8 (normalization never mutates canonical text) and I14 (derived indexes reproducible + generation-stamped). Mutable profiles would silently rewrite history.

**Independent Test**: `cargo test -p quran-normalization --lib` green (64 passed on 2026-09-17), including `builtin_rule_lists_match_plan`, `flags_match_plan`, `registry_is_append_only`, and `unknown_profile_versions_never_fall_back`.

**Acceptance Scenarios**:

1. **Given** a query under L0, **When** normalized, **Then** only whitespace handling applies and the trace lists exactly that rule set in order.
2. **Given** a request for a retired/unknown profile version, **When** resolved, **Then** it errors (`QAI-NORM-0002`) — it never silently falls back to `latest`.

---

### User Story 2 - Every hit explains itself (Priority: P1)

Each normalization run emits a rule-by-rule `NormalizationTrace` (the exact ordered rule set, I9) and a bidirectional `SpanMap` from every derived character back to canonical offsets (I10), so highlighting and citations always land on real canonical text.

**Why this priority**: Without traces, results are unexplainable; without span maps, matches cannot be cited. Both are invariants, not nice-to-haves.

**Independent Test**: Span-map property tests green (bidirectional mapping holds); trace required-field assertions green.

**Acceptance Scenarios**:

1. **Given** a normalized match, **When** inspected, **Then** its trace names every applied rule in order (e.g. diacritic-strip ran, hamza-normalize did not).
2. **Given** any derived offset, **When** mapped back, **Then** it resolves to the exact canonical character range (property-tested, not spot-checked).

---

### User Story 3 - Canonical text provably untouched (Priority: P1)

Derived forms (`quran_token_forms`, `quran_ayah_forms`, `quran_skeletons`) live in separate tables; the MV-018 verifier asserts canonical text is byte-identical before and after every rebuild.

**Why this priority**: Invariant I8. A normalization bug that rewrites canonical rows would be a data-integrity catastrophe.

**Independent Test**: MV-018 verifier runs around `qai quran forms rebuild`; QV-028 re-verification green.

**Acceptance Scenarios**:

1. **Given** a completed forms rebuild, **When** MV-018 compares canonical hashes before/after, **Then** they are identical or the build is rejected.

### Edge Cases

- Empty profiles are rejected (`QAI-NORM-0006`); unknown rules rejected (`QAI-NORM-0001`).
- Span mappings outside range yield `QAI-NORM-0004`; invalid mappings yield `QAI-NORM-0005`.
- The DB append-only trigger on normalization tables raises `QAI-NORM-0003` (not `0001`) — code truth per P2 `done.md` DEV-04; plan tables saying otherwise are stale.
- Profiles L0–L8 are nine rungs (`ProfileId::all()` returns 9); rule files are exactly `n01`–`n22`.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Crate MUST implement rules N01–N22 as individual modules (`rules/n01.rs`…`rules/n22.rs` + `rules/mod.rs`) behind the `rule` trait interface (`rule.rs`).
- **FR-002**: Crate MUST expose `ProfileId` L0–L8 (9 rungs, parse/display round-trip), versioned `Profile` records, and an append-only `ProfileRegistry` (`register` rejects mutation; unknown versions never fall back) (`profile.rs` L22–264).
- **FR-003**: Crate MUST run ordered pipelines (`pipeline.rs`) emitting a mandatory `NormalizationTrace` (exact ordered rule set per run) (`trace.rs`).
- **FR-004**: Crate MUST maintain bidirectional `SpanMap`s (derived ↔ canonical offsets) with property-tested round-trip laws (`span.rs`).
- **FR-005**: Crate MUST write derived forms only to separate tables; canonical rows are never an output of this crate (I8; MV-018/QV-028 verification owned by the application/forms service).
- **FR-006**: Crate MUST carry zero model/vector/embeddings dependencies (I2; enforced by `arch-check`).

### Key Entities

- **Rule (N01–N22)**: Single normalization transform with stable id and offset behavior.
- **Profile / ProfileId (L0–L8) / ProfileRegistry**: Versioned ordered rule lists; append-only registry with builtin ladder.
- **NormalizationTrace**: Per-run ordered record of applied rules (I9 evidence).
- **SpanMap**: Bidirectional derived↔canonical offset map (I10 evidence).

### Error Surface

`QAI-NORM-0001` unknown rule · `0002` unknown profile · `0003` profile immutable (also raised by the DB append-only trigger — code truth, DEV-04) · `0004` span out of range · `0005` invalid mapping · `0006` empty profile (`error.rs codes`; namespace test-enforced). `QAI-NORM-0000` appears in profiling paths as the unset/ok sentinel, not an error.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo test -p quran-normalization --lib` passes with zero failures (64 passed on 2026-09-17).
- **SC-002**: Registry append-only + no-fallback + builtin-lists-match-plan tests stay green (profile semantics frozen).
- **SC-003**: Span-map property tests hold bidirectionally on every run (no match without canonical offsets).
- **SC-004**: MV-018 canonical-unchanged verification passes around every forms rebuild (I8 holds in practice, not just in principle).

## Assumptions

- Rule semantics N01–N22 follow the P2 plan's rule catalog; `builtin_rule_lists_match_plan` pins the ladder — any rule change needs a new profile version, never an edit.
- Concatenated-word search (space/diacritic-dropping queries like `بسمالله` → `بِسْمِ ٱللَّهِ`) is served downstream by `quran-search` using these traces/span maps; this crate only produces the derived forms.
- Morphological search (lemma/root) is out of scope until `quran-morphology` + lexicon sprints land; L8 heuristic affix handling is explicitly heuristic (Layer D semantics).
- No Tantivy anywhere; no canonical writes from this crate, ever.
