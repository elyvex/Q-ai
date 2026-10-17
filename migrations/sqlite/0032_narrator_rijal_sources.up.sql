-- Phase 7 (plan 01): Per-scholar rijal sources for narrators.
-- Forward-only: no .down.sql (Phase-1 convention).
-- One row per scholar/source citation. No consensus column (D-03 pattern).

CREATE TABLE narrator_rijal_sources (
  id                 TEXT PRIMARY KEY,
  narrator_id        TEXT NOT NULL REFERENCES narrators(id) ON DELETE RESTRICT,
  scholar_name       TEXT NOT NULL CHECK (length(trim(scholar_name)) > 0),
  source_citation    TEXT NOT NULL,
  grading_value      TEXT,
  evidence_json      TEXT NOT NULL DEFAULT '{}',
  reviewer           TEXT,
  decision           TEXT NOT NULL CHECK (decision IN ('pending','accepted','rejected','superseded','disputed')),
  decided_at         TEXT,
  supersedes_id      TEXT REFERENCES narrator_rijal_sources(id),
  provenance_layer   TEXT NOT NULL CHECK (provenance_layer IN ('B','C','D')),
  confidence         REAL,
  created_at         TEXT NOT NULL,
  CHECK (decision = 'pending' OR
         (reviewer IS NOT NULL AND reviewer != '' AND decided_at IS NOT NULL)),
  CHECK (provenance_layer != 'D' OR (confidence IS NOT NULL))
);

CREATE INDEX ix_rijal_narrator ON narrator_rijal_sources(narrator_id);
CREATE INDEX ix_rijal_decision ON narrator_rijal_sources(narrator_id, decision);
