# Feature Specification: quran-search lexical engine

**Feature Branch**: `008-quran-search`

**Created**: 2026-09-17

**Status**: Draft

**Input**: Module-level specification of the implemented `quran-search` crate (crates/quran-search/src/{lib,model,tokenizer,fts5,index,regex,highlight,hit,skeleton,error}.rs), derived from code truth on 2026-09-17. Reverse specification of what exists today. Code is the source of truth over plan docs.

**Constitution compliance**: `.specify/memory/constitution.md` v1.0.2, Principles I/VII (I8: search over derived text only; I14: generation-stamped reproducible indexes), III (I9/I10 traces + offsets on every hit), VII (I16: DFA-only bounded regex; zero model/vector deps).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Backend-independent lexical search (Priority: P1)

Callers search through the `FullTextIndex` trait (`backend`, `manifest`, `create`, `add_batch`, `commit`, `search`, `count`, `delete_by_generation`, `stats`, `verify`) without depending on any specific engine. The shipped backend is SQLite FTS5 with custom `ar_*` tokenizers — not Tantivy.

**Why this priority**: Engine independence lets the project ratify or replace backends without rewriting callers; FTS5 keeps the zero-extra-dependency posture (ADR-0201 shipped-as-FTS5 per DEV-05).

**Independent Test**: `cargo test -p quran-search --lib` green (18 passed on 2026-09-17); `qai quran index rebuild` + `index verify` round-trip on a temp data dir.

**Acceptance Scenarios**:

1. **Given** a built generation-stamped index, **When** searched (exact, normalized, phrase, concatenated, cross-ayah window), **Then** hits carry totals, filters, highlighting, and generation keys.
2. **Given** a stale index (generation mismatch), **When** verified, **Then** verification reports `QAI-IDX-0101` instead of serving stale hits.

---

### User Story 2 - Regex without regex-DoS (Priority: P1)

Pattern search runs on a DFA-only engine (regex-automata, dense-DFA builds with NFA/DFA size limits) with query caps, step budgets, and timeouts — no backtracking, ever.

**Why this priority**: Invariant I16. User-supplied patterns must not be able to hang the engine.

**Independent Test**: Regex bound tests green (oversized patterns rejected with `QAI-IDX-0002` before execution).

**Acceptance Scenarios**:

1. **Given** a pathological nested-quantifier pattern, **When** submitted, **Then** it is rejected or bounded — never executed with backtracking.
2. **Given** a legitimate bounded pattern, **When** run over selected normalized fields, **Then** matches map to canonical offsets via SpanMap.

---

### User Story 3 - Atomic generation activation (Priority: P2)

Index builds write under a new generation and flip atomically; the old generation is deleted only after the flip (`delete_by_generation`). Readers never see a half-built index.

**Why this priority**: Invariant I14. A half-built index serving mixed-generation hits would corrupt research reproducibility.

**Independent Test**: `verify` (integrity report) + manifest-mismatch detection (`QAI-IDX-0004`) green.

**Acceptance Scenarios**:

1. **Given** a build for generation N+1 alongside live generation N, **When** readers query mid-build, **Then** they see only generation N until the atomic flip.

### Edge Cases

- `CANONICAL_CHANGED` (`QAI-IDX-0005`) if canonical hashes move under a build — the build fails instead of indexing against moved text.
- `INVALID_HIT` (`0006`) when a hit cannot be resolved to canonical offsets — never returned silently.
- `RATE_LIMITED` (`0007`) under configured query budgets.
- **Not yet exposed**: `qai quran search` CLI and search HTTP/SSE endpoints do not exist (briefing §9) — the Rust service API is the current surface; application wires caching/commands around it.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Crate MUST define the `FullTextIndex: Send + Sync` trait with `backend`, `manifest`, `create`, `add_batch`, `commit → CommitStamp`, `search → FtsResults`, `count`, `delete_by_generation`, `stats`, `verify → FtsIntegrityReport` (`index.rs` L21–51).
- **FR-002**: Crate MUST ship the SQLite FTS5 backend with custom `ar_*` tokenizers over derived (normalized) forms only — never canonical rows (`fts5.rs`, `tokenizer.rs`, `model.rs` `FtsDoc`/`FtsQuery`/`SearchOpts`).
- **FR-003**: Crate MUST serve exact, normalized, phrase, concatenated, and cross-ayah-window searches with filters, totals, highlighting (`highlight.rs`), and generation-keyed caching (`hit.rs`, `skeleton.rs`).
- **FR-004**: Crate MUST restrict pattern search to the DFA-only bounded engine with size caps, step budgets, and timeouts (I16) (`regex.rs`).
- **FR-005**: Crate MUST stamp every index artifact with manifests + `corpus_generation` and detect staleness/manifest mismatch/canonical drift (I14).
- **FR-006**: Crate MUST carry zero model/vector/embeddings/LLM dependencies (I2; `arch-check`).

### Key Entities

- **FullTextIndex / FtsBackend / IndexManifest / FtsSchema**: Backend contract, engine identity, build manifest, index schema.
- **FtsDoc / FtsQuery / SearchOpts / FtsResults / FtsStats**: Indexable derived documents, queries with normalization profile + filters, paged results with totals.
- **CommitStamp / FtsIntegrityReport**: Atomic-build evidence and verify output.

### Error Surface

`QAI-IDX-0001` backend unavailable · `0002` query rejected · `0003` build failed · `0004` manifest mismatch · `0005` canonical changed · `0006` invalid hit · `0007` rate limited · `0101` stale index (`error.rs codes`; namespace test-enforced).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo test -p quran-search --lib` passes with zero failures (18 passed on 2026-09-17).
- **SC-002**: Index rebuild → verify round-trip succeeds on a temp data dir with generation-keyed artifacts.
- **SC-003**: Every served hit resolves to canonical offsets (no `INVALID_HIT` escapes in the green suite); stale generations are reported, never served.
- **SC-004**: No backtracking engine exists anywhere in the dependency closure (DFA-only; I16 holds by construction + `arch-check`).

## Assumptions

- **FTS5 is current truth, not Tantivy.** Plan text (incl. parts of ADR-0201 discussion) still says Tantivy in places; shipped code is FTS5 because the offline Cargo cache had no Tantivy (DEV-05). Ratification is an owner decision — this spec documents FTS5 and must not be "corrected" to Tantivy.
- Search HTTP/SSE endpoints and `qai quran search` CLI are explicitly unbuilt; documenting them as available would be fabrication.
- Morphology-backed search (root/lemma/family) waits on `quran-morphology` + lexicon sprints; current profiles top out at L8 heuristic affix handling.
- Doctor index-staleness checks and evaluation/soak are P2 follow-ups, not in this crate.
