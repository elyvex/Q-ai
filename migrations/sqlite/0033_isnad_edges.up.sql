-- Phase 7 (plan 01): Isnad transmission edges.
-- Forward-only: no .down.sql (Phase-1 convention).
-- Computational links start 'uncertain' (Pitfall 2).

CREATE TABLE isnad_edges (
  id                  TEXT PRIMARY KEY,
  src_stable_id       TEXT NOT NULL,
  edge                TEXT NOT NULL CHECK (edge IN ('TRANSMITS','NARRATES')),
  dst_stable_id       TEXT NOT NULL,
  assertion_id        TEXT REFERENCES narrator_identity_assertions(id),
  provenance_layer    TEXT NOT NULL CHECK (provenance_layer IN ('B','C','D')),
  confidence          REAL,
  label               TEXT NOT NULL DEFAULT 'uncertain' CHECK (label IN ('uncertain','confirmed','rejected')),
  algorithm           TEXT,
  algorithm_version   TEXT,
  reviewer            TEXT,
  decided_at          TEXT,
  created_at          TEXT NOT NULL,
  CHECK (label != 'confirmed' OR
         (reviewer IS NOT NULL AND reviewer != '' AND decided_at IS NOT NULL)),
  CHECK (provenance_layer != 'D' OR (algorithm IS NOT NULL AND algorithm_version IS NOT NULL))
);

CREATE INDEX ix_isnad_src ON isnad_edges(src_stable_id);
CREATE INDEX ix_isnad_dst ON isnad_edges(dst_stable_id);
