# ADR-0301 — RAG Strategy (Proposed)

- Status: Proposed (Phase 3 — not yet decided)
- Phase: 3 — Retrieval
- Date: 2026-09-14
- Related decisions: ADR-0701 (vector store), ADR-0702 (cross-store consistency)
- Requirements: PRD §34, §39, §73, §75–76

## Context

Phase 3 introduces retrieval over the canonical corpus and its annotations. The
retrieval subsystem must respect the platform's epistemic rules: results must be
traceable to a source version, must separate canonical text from interpretation,
and must never present machine-generated similarity as authority (PRD §23, §39).

This ADR is intentionally **open**. Phase 0 ships only the primitives retrieval
will consume — provenance, generation stamping, and a transactional outbox — so
that Phase 3 writes *consumers* rather than inventing the consistency pattern.

## Options Considered

To be evaluated in Phase 3. Candidate directions:

| Option | Advantages | Disadvantages |
|---|---|---|
| **Hybrid BM25 + dense vectors, reranked** | Strong lexical + semantic recall | Two indexes to keep consistent |
| **Graph-augmented retrieval** (ADR-0702 consumers) | Multi-hop, citation-aware | Higher build/latency cost |
| **Long-context-only** | Simple pipeline | Cost, "lost in the middle", poor traceability |

## Decision

**Not yet decided.** Phase 3 must select the retrieval topology, embedding
model, chunking policy, and reranking strategy, and record the decision here
with the same §48 sections. The following are fixed constraints on any choice:

1. Retrieval is a **consumer** of the outbox; indexes are built from dispatched
   events, never by polling authoritative tables directly.
2. Every result carries its `source_version_id` and `corpus_generation`.
3. Tombstoned/deactivated subjects are excluded immediately, even while physical
   cleanup is pending (ADR-0702 §9).

## Accuracy Implications

- Similarity score must not be surfaced as a confidence in a source's authority.
- Answers must distinguish quotation, source summary, and AI analysis.

## Religious-Source Implications

- Retrieval must never elevate a machine-generated annotation above a
  scholar-reviewed one; `TrustLevel` gates whether a result may support a
  definitive claim.

## Licensing Implications

- Retrieval must filter by `LicenseRecord.export_allowed` and
  `redistribution_allowed` before returning content.

## Security Implications

- Retrieved content is untrusted (`Untrusted<T>`); it must be sanitized before
  rendering and never treated as instructions (prompt-injection defense).

## Operational Implications

- Index builds are background jobs with checkpoints and cancellation.
- Generation stamps let a stale index be detected and rebuilt.

## Migration Strategy

Not applicable — no implementation exists yet.

## Reversal Cost

High once indexes are populated; mitigated by storing the embedding model and
generation stamp alongside every vector so a rebuild is deterministic.

## Acceptance Criteria

To be defined in Phase 3. Blocked on: ADR-0701 (vector store), ADR-0204
(normalization rules), and the corpus licensing decisions (ADR-0101).
