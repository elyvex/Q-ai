-- Phase 6 (plan 01): Hadith grading records.
-- Forward-only: no .down.sql (Phase-1 convention).
-- Per-scholar independent records following the graph_assertions pattern
-- (Phase 4). No consensus field exists anywhere (D-03). Each grading carries
-- its own attribution (scholar_name) and source_citation.

CREATE TABLE hadith_gradings (
  id                     TEXT PRIMARY KEY,
  hadith_id              TEXT NOT NULL REFERENCES hadith_records(id) ON DELETE RESTRICT,
  scholar_name           TEXT NOT NULL CHECK (length(trim(scholar_name)) > 0),
  grading_value          TEXT NOT NULL,
  source_citation        TEXT NOT NULL,
  evidence_json          TEXT NOT NULL DEFAULT '{}',
  reviewer               TEXT,
  decision               TEXT NOT NULL CHECK (decision IN
                             ('pending','accepted','rejected','superseded')),
  decided_at             TEXT,
  supersedes_id          TEXT REFERENCES hadith_gradings(id),
  provenance_layer       TEXT NOT NULL CHECK (provenance_layer IN ('B','C','D')),
  confidence             REAL,
  created_at             TEXT NOT NULL,
  CHECK (decision = 'pending' OR
         (reviewer IS NOT NULL AND reviewer != '' AND decided_at IS NOT NULL))
);

CREATE INDEX ix_hadith_gradings_hadith ON hadith_gradings(hadith_id);
CREATE INDEX ix_hadith_gradings_decision ON hadith_gradings(hadith_id, decision);
