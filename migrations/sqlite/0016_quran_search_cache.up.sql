-- Phase 2 (M3, P2-T50): generation-keyed search result cache (D2.10).
-- Forward-only: no .down.sql (Phase-1 convention).
--
-- Correctness contract (I14/cache safety):
-- - the cache key binds the corpus generation, so a generation bump can
--   never serve stale entries (namespace isolation, not best effort);
-- - reads additionally verify the stored generation and drop mismatches;
-- - eviction is LRU over `last_hit_at`, capped at 128 MiB by default
--   (`cache_enforce_cap` runs after every insert; the cap travels as a
--   parameter so tests exercise it without 128 MiB fixtures).
-- Cache rows are derived data: safe to wipe at any time, never canonical.

CREATE TABLE search_result_cache (
  key          TEXT PRIMARY KEY,
  generation   INTEGER NOT NULL,
  payload_json TEXT NOT NULL,
  bytes        INTEGER NOT NULL CHECK (bytes >= 0),
  created_at   TEXT NOT NULL,
  last_hit_at  TEXT NOT NULL
);

CREATE INDEX ix_cache_age ON search_result_cache(last_hit_at);
