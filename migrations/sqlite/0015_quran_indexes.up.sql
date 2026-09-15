-- Phase 2 (M2, P2-T33): index pointers + build-run tracking (D2.10).
-- Forward-only: no .down.sql (Phase-1 convention).
--
-- `index_pointers` is a live pointer, not history: exactly one row per
-- index id names the serving generation. It is the ONLY mutable Phase-2
-- catalog row by design (append-only applies to definitions, not to the
-- pointer that selects among them). Flips happen in one transaction with
-- the build-run state change, so a partial index can never serve queries.
-- Old generations are retained on disk for single-step rollback; `qai index
-- gc` (P2-T35) enforces retention.

CREATE TABLE index_pointers (
  index_id      TEXT PRIMARY KEY,
  generation    INTEGER NOT NULL CHECK (generation > 0),
  manifest_json TEXT NOT NULL,
  updated_at    TEXT NOT NULL,
  updated_by    TEXT NOT NULL
);

CREATE TABLE index_build_runs (
  id                TEXT PRIMARY KEY,
  index_id          TEXT NOT NULL,
  generation        INTEGER NOT NULL CHECK (generation > 0),
  corpus_generation INTEGER NOT NULL,
  state             TEXT NOT NULL CHECK (state IN ('staged','verifying','active','failed','superseded')),
  doc_count         INTEGER NOT NULL DEFAULT 0,
  manifest_hash     TEXT NOT NULL DEFAULT '',
  error             TEXT,
  started_at        TEXT NOT NULL,
  finished_at       TEXT,
  UNIQUE (index_id, generation)
);

CREATE INDEX ix_build_runs_index ON index_build_runs(index_id, generation);
