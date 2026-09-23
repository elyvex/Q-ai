# ADR-0214 — Search Result Caching and Invalidation Keying

- Status: Proposed (implementation complete; acceptance pending owner review)
- Phase: 2 — Quran Search and Linguistics
- Date: 2026-09-23
- Related decisions: ADR-0213 (generations), ADR-0201 (FTS)
- Requirements: plan §13; AC-P2-35; D2.10
- Implementation: `crates/application/src/quran_search_cache.rs`;
  migration `0016_quran_search_cache`; suite `search_cache.rs`

## Context

Verified search results are expensive (FTS recall + exact verification +
quotation checks). Caching must never serve a stale generation's results as
current, must bound memory, and must fail safe on corrupt payloads.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| Generation-keyed cache with LRU cap (chosen) | Stale generations unreachable by construction (key namespace + read-time check + wholesale invalidation); bounded disk | Whole-generation invalidation on rebuild (no fine-grained reuse) |
| Content-hash-keyed cache | Fine-grained reuse | Hash bookkeeping per row; invalidation logic subtle |
| No cache | Simplest | Repeated verified searches pay full cost |

## Decision

- Cache key binds tool + params + profile + extra + generation.
- Three invalidation layers: key namespace includes generation; reads
  re-validate generation (mismatch = miss + delete the stale row);
  generation bumps trigger wholesale invalidation.
- Unparseable payloads are misses (row deleted), never errors.
- 128 MiB default cap with single-statement LRU eviction (`last_hit_at`
  touch on hits, recency-correct).
- Table shape (DEV-08): `(key, generation, payload_json, bytes,
  created_at, last_hit_at)` — `tool_name`/`hit_count` live inside key/payload.

## Accuracy and Religious-Source Implications

Serving a previous generation's results as current would misrepresent
derivations. Mitigation: no stale generation ever served (contract proven
standalone in `search_cache.rs`: round-trip identity, generation-miss
deletes, LRU recency, wholesale keeps-current).

## Licensing Implications

None — derived-data mechanics.

## Security Implications

Cache rows are derived data (safe to wipe); keys never contain secrets;
payloads are verified tool outputs (same trust level as live results).

## Operational Implications

Monitor cache hit rates + bytes vs cap; wipe freely on suspicion
(`cache_invalidate` wholesale). Tool-level wiring consults the cache;
services own verification regardless of hit/miss.

## Migration Strategy

`0016_quran_search_cache`. Cache rows are disposable; future shape changes
wipe + rebuild.

## Reversal Cost

Negligible. Drop the table and bypass the lookup; services verify
independently.

## Acceptance Criteria

- No stale cached result across a generation bump (AC-P2-35).
- LRU cap enforced; corrupt payloads are misses.
