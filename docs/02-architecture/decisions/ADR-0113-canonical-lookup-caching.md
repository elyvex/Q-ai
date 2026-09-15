# ADR-0113 — Canonical Lookup Caching & Invalidation (Draft)

- Status: Accepted
- Phase: 1 — Canonical Quran Core
- Date: 2026-09-14
- Related decisions: ADR-0107
- Requirements: PRD §40

## Context

Single-digit-millisecond lookups need an in-process cache, but a cache that
serves pre-activation text is a correctness catastrophe, not a performance bug.

## Decision (proposed)

- `lru`-backed cache keyed by
  `(edition_id, version, corpus_generation, ref, options_hash)`.
- Wholesale invalidation on `corpus_generation` change; staleness is
  detectable, never assumed.
- Correctness over hit rate: a cache-consistency test (activate a new version,
  assert no stale text) gates the reader. Performance targets are in plan D1.6;
  the cache must never trade correctness for latency.

## Consequences

- Implemented in M8 (`application::quran_reader`); accepted with the consistency test (`cache_serves_no_stale_text_after_activation`).
